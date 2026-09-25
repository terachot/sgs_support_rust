//! ควบคุม Chrome/Edge ผ่าน Chrome DevTools Protocol

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::Page;
use futures::StreamExt;
use serde::Deserialize;
use tokio::task::JoinHandle;

use crate::excel::Page as ExcelPage;

const CHROME_PATHS: &[&str] = &[
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Production,
    Mock,
}

impl Target {
    pub fn label(self) -> &'static str {
        match self {
            Self::Production => "SGS จริง",
            Self::Mock => "Mockup ในเครื่อง",
        }
    }

    pub fn login_url(self) -> &'static str {
        match self {
            Self::Production => "https://sgs.bopp-obec.info/sgs/",
            Self::Mock => "http://localhost/sgs_tester/",
        }
    }

    pub fn home_url(self) -> &'static str {
        match self {
            Self::Production => {
                "https://sgs.bopp-obec.info/sgs/TblSchoolInfo/Show-TblSchoolInfo.aspx"
            }
            Self::Mock => "http://localhost/sgs_tester/Show-TblSchoolInfo.html",
        }
    }

    pub fn page_url(self, page: ExcelPage) -> &'static str {
        match self {
            Self::Production => page.production_url(),
            Self::Mock => page.mock_url(),
        }
    }
}

fn find_system_chrome() -> Option<PathBuf> {
    CHROME_PATHS
        .iter()
        .map(PathBuf::from)
        .find(|path| path.exists())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebStudent {
    pub id: String,
    pub name: String,
    /// token ของ ASP.NET Repeater เช่น `ctl00` หรือ `ctl01`
    pub control_token: String,
}

pub struct Session {
    browser: Browser,
    page: Page,
    _handler: JoinHandle<()>,
    target: Target,
    profile_dir: PathBuf,
}

impl Session {
    pub async fn launch(target: Target) -> Result<(Self, String)> {
        let chrome_path = find_system_chrome();
        let profile_dir = std::env::temp_dir().join(format!(
            "sgs-support-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        let mut builder = BrowserConfig::builder()
            .window_size(1280, 800)
            .with_head()
            .user_data_dir(&profile_dir)
            .viewport(None);

        let source_message = if let Some(path) = chrome_path {
            builder = builder.chrome_executable(path.clone());
            format!("ใช้เบราว์เซอร์ที่ติดตั้ง: {}", path.display())
        } else {
            "ไม่พบ Chrome/Edge ในตำแหน่งมาตรฐาน".to_owned()
        };

        let (browser, mut handler) = Browser::launch(
            builder
                .build()
                .map_err(|error| anyhow!("ตั้งค่าเบราว์เซอร์ไม่สำเร็จ: {error}"))?,
        )
        .await
        .context("เปิด Chrome/Edge ไม่สำเร็จ")?;

        let handler_task = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        });
        let page = browser
            .new_page("about:blank")
            .await
            .context("สร้างแท็บเบราว์เซอร์ไม่สำเร็จ")?;

        Ok((
            Self {
                browser,
                page,
                _handler: handler_task,
                target,
                profile_dir,
            },
            source_message,
        ))
    }

    pub async fn open_login(&self) -> Result<()> {
        self.navigate_to(self.target.login_url()).await
    }

    pub async fn open_page(&self, page: ExcelPage) -> Result<()> {
        self.navigate_to(self.target.page_url(page)).await
    }

    async fn navigate_to(&self, url: &str) -> Result<()> {
        self.page
            .goto(url)
            .await
            .with_context(|| format!("เปิดหน้า {url} ไม่สำเร็จ"))?;
        Ok(())
    }

    pub async fn current_url(&self) -> Result<String> {
        self.page
            .url()
            .await?
            .ok_or_else(|| anyhow!("ไม่พบ URL ของแท็บปัจจุบัน"))
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<()> {
        self.wait_for_login_form().await?;
        let script = format!(
            r#"(function(){{
                const username = document.querySelector('input[name="ctl00$PageContent$UserName"]')
                    || document.querySelector('input[type="text"]');
                const password = document.querySelector('input[name="ctl00$PageContent$Password"]')
                    || document.querySelector('input[type="password"]');
                const button = document.getElementById('ctl00_PageContent_OKButton__Button')
                    || [...document.querySelectorAll('a, button, input[type="submit"]')]
                        .find(el => (el.textContent || el.value || '').includes('เข้าสู่ระบบ'));
                if (!username || !password) return 'ไม่พบช่องชื่อผู้ใช้หรือรหัสผ่าน';
                if (!button) return 'ไม่พบปุ่มเข้าสู่ระบบ';
                username.value = {username};
                password.value = {password};
                for (const el of [username, password]) {{
                    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                }}
                button.click();
                return 'ok';
            }})()"#,
            username = json_string(username),
            password = json_string(password),
        );
        let status: String = self
            .page
            .evaluate(script.as_str())
            .await?
            .into_value()
            .context("อ่านผลการเข้าสู่ระบบไม่สำเร็จ")?;
        if status != "ok" {
            return Err(anyhow!(status));
        }

        let mut home_retries = 0;
        for _ in 0..30 {
            tokio::time::sleep(Duration::from_millis(400)).await;
            let current = self.current_url().await.unwrap_or_default();
            if same_page(&current, self.target.home_url()) {
                if self.target == Target::Mock {
                    return Ok(());
                }

                // SGS อาจ redirect มาหน้าแรกแล้วแสดง ASP.NET Server Error ชั่วคราว
                // จึงต้องเห็นสถานะออกจากระบบก่อนจึงถือว่าเข้าสู่ระบบสำเร็จ
                let page_state: String = match self
                    .page
                    .evaluate(
                        r#"(function(){
                        if (document.querySelector('h1')?.textContent?.includes('Server Error in'))
                            return 'server_error';
                        const signIn = document.getElementById('ctl00__PageHeader__SignIn');
                        return /ออกจากระบบ|Sign Out|Log Out/i.test(signIn?.textContent || '')
                            ? 'authenticated' : 'pending';
                    })()"#,
                    )
                    .await
                {
                    Ok(result) => match result.into_value() {
                        Ok(state) => state,
                        Err(_) => continue,
                    },
                    Err(_) => continue, // หน้าอาจกำลังเปลี่ยนระหว่างตรวจ
                };

                if page_state == "authenticated" {
                    return Ok(());
                }
                if page_state == "server_error" {
                    if home_retries >= 2 {
                        return Err(anyhow!(
                            "หน้าแรกของ SGS ยังแสดงข้อผิดพลาดของเซิร์ฟเวอร์หลังลองโหลดใหม่"
                        ));
                    }
                    home_retries += 1;
                    self.navigate_to(self.target.home_url()).await?;
                }
            }
        }
        Err(anyhow!(
            "เข้าสู่ระบบไม่สำเร็จหรือใช้เวลานานเกินไป (หน้าปัจจุบัน: {})",
            self.current_url().await.unwrap_or_default()
        ))
    }

    async fn wait_for_login_form(&self) -> Result<()> {
        for _ in 0..40 {
            let state: Result<String> = async {
                Ok(self
                    .page
                    .evaluate(
                        r#"(function(){
                            const username = document.querySelector('input[name="ctl00$PageContent$UserName"]')
                                || document.querySelector('input[type="text"]');
                            const password = document.querySelector('input[name="ctl00$PageContent$Password"]')
                                || document.querySelector('input[type="password"]');
                            if (username && password) return 'form';
                            const signIn = document.getElementById('ctl00__PageHeader__SignIn');
                            if (/ออกจากระบบ|Sign Out|Log Out/i.test(signIn?.textContent || ''))
                                return 'authenticated';
                            if (document.querySelector('h1')?.textContent?.includes('Server Error in'))
                                return 'server_error';
                            return 'pending';
                        })()"#,
                    )
                    .await?
                    .into_value()?)
            }
            .await;

            match state.as_deref() {
                Ok("form") => return Ok(()),
                Ok("authenticated") => {
                    return Err(anyhow!("เบราว์เซอร์เข้าสู่ระบบอยู่แล้ว แต่ยังยืนยันไม่ได้ว่าเป็นบัญชีที่กรอก"));
                }
                Ok("server_error") => {
                    return Err(anyhow!("หน้าเข้าสู่ระบบ SGS แสดงข้อผิดพลาดของเซิร์ฟเวอร์"));
                }
                _ => tokio::time::sleep(Duration::from_millis(250)).await,
            }
        }
        Err(anyhow!(
            "รอช่องเข้าสู่ระบบนานเกินไป (หน้าปัจจุบัน: {})",
            self.current_url().await.unwrap_or_default()
        ))
    }

    pub async fn logout(&self) -> Result<()> {
        if self.target == Target::Mock {
            return self.open_login().await;
        }

        let action: String = self
            .page
            .evaluate(
                r#"(function(){
                    if (document.querySelector('input[name="ctl00$PageContent$Password"]'))
                        return 'already_signed_out';
                    const link = document.getElementById('ctl00__PageHeader__SignIn');
                    if (!link || !/ออกจากระบบ|Sign Out|Log Out/i.test(link.textContent || ''))
                        return 'missing_logout';
                    link.click();
                    return 'clicked';
                })()"#,
            )
            .await?
            .into_value()?;
        match action.as_str() {
            "already_signed_out" => return Ok(()),
            "clicked" => {}
            _ => return Err(anyhow!("ไม่พบปุ่มออกจากระบบในหน้า SGS")),
        }

        for _ in 0..40 {
            let signed_out: Result<bool> = async {
                Ok(self
                    .page
                    .evaluate(
                        r#"!!document.querySelector('input[name="ctl00$PageContent$Password"]')"#,
                    )
                    .await?
                    .into_value()?)
            }
            .await;
            if matches!(signed_out, Ok(true)) {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        Err(anyhow!(
            "เว็บ SGS ไม่ยืนยันการออกจากระบบ (หน้าปัจจุบัน: {})",
            self.current_url().await.unwrap_or_default()
        ))
    }

    pub async fn scrape_students(&self, page: ExcelPage) -> Result<Vec<WebStudent>> {
        let script = format!(
            r#"(function(){{
                const students = [];
                for (const row of document.querySelectorAll('td.tre tbody tr')) {{
                    const cells = row.querySelectorAll(':scope > td');
                    const id = cells[{id_index}]?.textContent?.trim() || '';
                    const name = cells[{name_index}]?.textContent?.trim() || '';
                    const input = row.querySelector('input[id*="_ctl"]');
                    const token = input?.id?.match(/_(ctl\d+)_/)?.[1] || '';
                    if (id && name && token) students.push({{ id, name, control_token: token }});
                }}
                return students;
            }})()"#,
            id_index = page.web_id_td() - 1,
            name_index = page.web_name_td() - 1,
        );

        #[derive(Deserialize)]
        struct Row {
            id: String,
            name: String,
            control_token: String,
        }

        let rows: Vec<Row> = self
            .page
            .evaluate(script.as_str())
            .await?
            .into_value()
            .context("อ่านตารางนักเรียนจากหน้าเว็บไม่สำเร็จ")?;
        Ok(rows
            .into_iter()
            .map(|row| WebStudent {
                id: row.id,
                name: row.name,
                control_token: row.control_token,
            })
            .collect())
    }

    pub async fn fill_score(
        &self,
        page: ExcelPage,
        control_token: &str,
        column: &str,
        value: &str,
    ) -> Result<()> {
        if !control_token.starts_with("ctl")
            || !control_token[3..]
                .chars()
                .all(|character| character.is_ascii_digit())
        {
            return Err(anyhow!("control token ไม่ถูกต้อง: {control_token}"));
        }
        let id = format!(
            "ctl00_PageContent_{}_{}_{}",
            page.repeater(),
            control_token,
            column
        );
        let script = format!(
            r#"(function(){{
                const input = document.getElementById({id});
                if (!input) return 'not_found';
                input.value = {value};
                input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                input.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return 'ok';
            }})()"#,
            id = json_string(&id),
            value = json_string(value),
        );
        let status: String = self
            .page
            .evaluate(script.as_str())
            .await?
            .into_value()
            .context("กรอกคะแนนไม่สำเร็จ")?;
        if status != "ok" {
            return Err(anyhow!("ไม่พบช่องกรอก {id}"));
        }
        Ok(())
    }

    pub async fn click_save(&self, page: ExcelPage) -> Result<()> {
        let script = format!(
            r#"(function(){{
                const button = document.querySelector({selector});
                if (!button) return 'not_found';
                button.click();
                return 'ok';
            }})()"#,
            selector = json_string(page.save_selector()),
        );
        let status: String = self
            .page
            .evaluate(script.as_str())
            .await?
            .into_value()
            .context("กดปุ่มบันทึกไม่สำเร็จ")?;
        if status != "ok" {
            return Err(anyhow!("ไม่พบปุ่มบันทึกของหน้า {}", page.thai()));
        }
        Ok(())
    }

    pub async fn close(mut self) -> Result<()> {
        self.browser.close().await.context("ปิดเบราว์เซอร์ไม่สำเร็จ")?;
        tokio::time::timeout(Duration::from_secs(5), self.browser.wait())
            .await
            .context("รอ Chrome/Edge ปิดนานเกินไป")?
            .context("รอ Chrome/Edge ปิดไม่สำเร็จ")?;
        let temp_root = std::env::temp_dir().canonicalize()?;
        let profile_dir = self.profile_dir.canonicalize()?;
        let safe_name = profile_dir
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("sgs-support-"));
        if profile_dir.parent() != Some(temp_root.as_path()) || !safe_name {
            return Err(anyhow!("ตำแหน่งโปรไฟล์เบราว์เซอร์ชั่วคราวไม่ถูกต้อง"));
        }
        tokio::fs::remove_dir_all(&profile_dir)
            .await
            .context("ลบโปรไฟล์เบราว์เซอร์ชั่วคราวไม่สำเร็จ")?;
        Ok(())
    }
}

fn same_page(actual: &str, expected: &str) -> bool {
    normalized_url(actual).eq_ignore_ascii_case(normalized_url(expected))
}

fn normalized_url(url: &str) -> &str {
    url.split(['?', '#'])
        .next()
        .unwrap_or(url)
        .trim_end_matches('/')
}

fn json_string(value: &str) -> String {
    serde_json::Value::String(value.to_owned()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_urls_are_complete() {
        assert_eq!(
            Target::Production.login_url(),
            "https://sgs.bopp-obec.info/sgs/"
        );
        assert!(Target::Production
            .page_url(ExcelPage::Attribute)
            .contains("TblTranscriptsQ"));
        assert!(Target::Mock
            .page_url(ExcelPage::Before)
            .starts_with("http://localhost/sgs_tester/"));
    }

    #[test]
    fn compares_trailing_slashes() {
        assert!(same_page(
            "http://localhost/sgs_tester",
            "http://localhost/sgs_tester/"
        ));
        assert!(same_page(
            "https://sgs.example/Home.aspx?session=1",
            "https://sgs.example/home.aspx"
        ));
    }

    #[tokio::test]
    #[ignore = "ต้องใช้ Chrome/Edge และเชื่อมต่อเว็บไซต์ SGS จริง"]
    async fn production_login_form_smoke() -> Result<()> {
        let (session, _) = Session::launch(Target::Production).await?;
        session.open_login().await?;
        session.wait_for_login_form().await?;
        assert!(session.current_url().await?.contains("SignIn.aspx"));
        session.close().await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore = "ต้องเปิด Chrome/Edge"]
    async fn logout_button_smoke() -> Result<()> {
        let (session, _) = Session::launch(Target::Production).await?;
        session
            .page
            .evaluate(
                r##"document.body.innerHTML = '<a id="ctl00__PageHeader__SignIn" href="#">ออกจากระบบ</a>';
                   document.getElementById('ctl00__PageHeader__SignIn').onclick = event => {
                       event.preventDefault();
                       document.body.innerHTML = '<input name="ctl00$PageContent$Password" type="password">';
                   };"##,
            )
            .await?;
        session.logout().await?;
        session.close().await?;
        Ok(())
    }
}
