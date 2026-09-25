//! สถานะร่วมและคอมโพเนนต์ที่ใช้หลายหน้า

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use dioxus::prelude::*;
use tokio::sync::Mutex;

use crate::browser::Session;
use crate::excel::{self, ExcelData, Page as ExcelPage};

#[derive(Clone)]
pub struct AppState {
    pub session: Arc<Mutex<Option<Session>>>,
    pub excel: Arc<Mutex<Option<ExcelData>>>,
}

impl PartialEq for AppState {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.session, &other.session) && Arc::ptr_eq(&self.excel, &other.excel)
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
            excel: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcelSummary {
    pub file_name: String,
    pub rooms: Vec<(String, usize)>,
    pub total_students: usize,
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
// `cargo run` ไม่ผ่านขั้นตอน bundle ของ `dx`; ฝัง CSS เพื่อให้เดสก์ท็อปมีสไตล์เสมอ
const MAIN_CSS: &str = include_str!("../assets/main.css");
const SAMPLE_EXCEL: &[u8] = include_bytes!("../assets/sgs-all.xlsx");
const SAMPLE_FILE_NAME: &str = "sgs-all.xlsx";

#[component]
pub fn Header() -> Element {
    let mut download_status = use_signal(String::new);
    let mut is_downloading = use_signal(|| false);
    rsx! {
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }
        document::Link { rel: "icon", href: FAVICON }
        document::Style { "{MAIN_CSS}" }
        header { class: "app-header",
            div {
                h1 { "SGS Support" }
                p { "เครื่องมือช่วยกรอกผลการเรียน" }
            }
            nav {
                button {
                    class: "sample-download",
                    disabled: is_downloading(),
                    title: "บันทึกไฟล์ Excel ตัวอย่างที่ฝังอยู่ในโปรแกรม",
                    onclick: move |_| {
                        is_downloading.set(true);
                        spawn(async move {
                            match save_sample_excel().await {
                                Ok(Some(_)) => download_status.set("บันทึกไฟล์แล้ว".to_owned()),
                                Ok(None) => download_status.set("ยกเลิกการบันทึก".to_owned()),
                                Err(error) => download_status.set(format!("บันทึกไม่สำเร็จ: {error}")),
                            }
                            is_downloading.set(false);
                        });
                    },
                    if is_downloading() { "กำลังบันทึก..." } else { "โหลดไฟล์ตัวอย่าง" }
                }
                Link { to: crate::Route::Work, "กรอกข้อมูล" }
                if !download_status().is_empty() {
                    span { class: "download-status", title: "{download_status}", "{download_status}" }
                }
            }
        }
    }
}

async fn save_sample_excel() -> Result<Option<std::path::PathBuf>> {
    tokio::task::spawn_blocking(|| {
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter("Excel Workbook", &["xlsx"])
            .set_file_name(SAMPLE_FILE_NAME)
            .set_title("บันทึกไฟล์ Excel ตัวอย่าง")
            .save_file()
        else {
            return Ok(None);
        };

        if path.extension().is_none() {
            path.set_extension("xlsx");
        } else if !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("xlsx"))
        {
            return Err(anyhow!("ไฟล์ตัวอย่างต้องใช้นามสกุล .xlsx"));
        }

        if path.exists()
            && rfd::MessageDialog::new()
                .set_title("ยืนยันการแทนที่ไฟล์")
                .set_description(format!("ไฟล์ {} มีอยู่แล้ว ต้องการแทนที่หรือไม่?", path.display()))
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()
                != rfd::MessageDialogResult::Yes
        {
            return Ok(None);
        }

        std::fs::write(&path, SAMPLE_EXCEL)
            .with_context(|| format!("เขียนไฟล์ {} ไม่สำเร็จ", path.display()))?;
        Ok(Some(path))
    })
    .await
    .context("เปิดหน้าต่างบันทึกไฟล์ไม่สำเร็จ")?
}

#[component]
pub fn LogPanel(log: Signal<Vec<String>>) -> Element {
    let lines: Vec<String> = log.read().iter().rev().take(80).cloned().collect();
    rsx! {
        section { class: "log-panel",
            div { class: "panel-heading",
                h3 { "สถานะการทำงาน" }
                button { class: "button button-danger button-small", onclick: move |_| log.write().clear(), "ล้าง" }
            }
            div { class: "log-lines",
                if lines.is_empty() {
                    p { class: "muted", "ยังไม่มีข้อความ" }
                }
                for line in lines {
                    p { "{line}" }
                }
            }
        }
    }
}

pub fn push_log(log: &mut Signal<Vec<String>>, message: impl Into<String>) {
    log.write().push(message.into());
}

#[component]
pub fn FilePickerButton(
    log: Signal<Vec<String>>,
    app_state: AppState,
    mut summary: Signal<Option<ExcelSummary>>,
) -> Element {
    rsx! {
        button {
            class: "button button-success",
            onclick: move |_| {
                let app_state = app_state.clone();
                let mut log = log;
                spawn(async move {
                    let path = match pick_excel_file().await {
                        Ok(path) => path,
                        Err(error) => {
                            if error != "ยกเลิกการเลือกไฟล์" {
                                push_log(&mut log, error);
                            }
                            return;
                        }
                    };
                    match excel::load_excel(&path) {
                        Ok(data) => {
                            let new_summary = ExcelSummary {
                                file_name: path.file_name()
                                    .and_then(|name| name.to_str())
                                    .unwrap_or("ไฟล์ Excel")
                                    .to_owned(),
                                rooms: data.rooms.iter()
                                    .map(|room| (room.name.clone(), room.students.len()))
                                    .collect(),
                                total_students: data.total_students(),
                            };
                            push_log(
                                &mut log,
                                format!(
                                    "โหลด {} สำเร็จ: {} ห้อง {} คน",
                                    new_summary.file_name,
                                    new_summary.rooms.len(),
                                    new_summary.total_students
                                ),
                            );
                            *app_state.excel.lock().await = Some(data);
                            summary.set(Some(new_summary));
                        }
                        Err(error) => push_log(&mut log, format!("โหลด Excel ไม่สำเร็จ: {error}")),
                    }
                });
            },
            "เลือกไฟล์ Excel"
        }
    }
}

async fn pick_excel_file() -> Result<std::path::PathBuf, String> {
    tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new()
            .add_filter("Excel Workbook", &["xlsx", "xlsm"])
            .set_title("เลือกไฟล์คะแนน SGS")
            .pick_file()
            .ok_or_else(|| "ยกเลิกการเลือกไฟล์".to_owned())
    })
    .await
    .map_err(|error| format!("เปิดหน้าต่างเลือกไฟล์ไม่สำเร็จ: {error}"))?
}

#[component]
pub fn ExcelDataInfo(summary: Signal<Option<ExcelSummary>>) -> Element {
    let current = summary.read().clone();
    rsx! {
        section { class: "file-summary",
            if let Some(data) = current {
                strong { "{data.file_name}" }
                span { "{data.rooms.len()} ห้อง · {data.total_students} คน" }
                div { class: "room-list",
                    for (room, count) in data.rooms {
                        span { "{room}: {count} คน" }
                    }
                }
            } else {
                strong { "ยังไม่ได้เลือกไฟล์ Excel" }
                span { "หัวตารางต้องเริ่มด้วย stdID, student Name และตามด้วยคอลัมน์คะแนน" }
            }
        }
    }
}

#[component]
pub fn TabPage(
    page: ExcelPage,
    log: Signal<Vec<String>>,
    app_state: AppState,
    summary: Signal<Option<ExcelSummary>>,
    mut is_busy: Signal<bool>,
) -> Element {
    rsx! {
        section { class: "workspace-card",
            div { class: "workspace-title",
                div {
                    p { class: "eyebrow", "ประเภทข้อมูล" }
                    h2 { "{page.thai()}" }
                }
                span { class: "step-badge", "ตรวจรหัสนักเรียนก่อนกรอกทุกครั้ง" }
            }

            div { class: "action-grid",
                FilePickerButton { log, app_state: app_state.clone(), summary }
                button {
                    class: "button button-secondary",
                    disabled: is_busy(),
                    onclick: {
                        let app_state = app_state.clone();
                        let mut log = log;
                        move |_| {
                            let app_state = app_state.clone();
                            spawn(async move {
                                let guard = app_state.session.lock().await;
                                let Some(session) = guard.as_ref() else {
                                    push_log(&mut log, "ยังไม่ได้เข้าสู่ระบบ กรุณากลับไปหน้าเข้าสู่ระบบ");
                                    return;
                                };
                                match session.open_page(page).await {
                                    Ok(()) => push_log(&mut log, format!("เปิดหน้า {} แล้ว", page.thai())),
                                    Err(error) => push_log(&mut log, format!("เปิดหน้าไม่สำเร็จ: {error}")),
                                }
                            });
                        }
                    },
                    "1. เปิดหน้า SGS"
                }
                button {
                    class: "button button-primary",
                    disabled: is_busy(),
                    onclick: {
                        let app_state = app_state.clone();
                        let mut log = log;
                        move |_| {
                            let app_state = app_state.clone();
                            spawn(async move {
                                is_busy.set(true);
                                push_log(&mut log, format!("เริ่มกรอก {}", page.thai()));
                                if let Err(error) = fill_current_page(page, &mut log, app_state).await {
                                    push_log(&mut log, format!("หยุดการทำงาน: {error}"));
                                }
                                is_busy.set(false);
                            });
                        }
                    },
                    if is_busy() { "กำลังกรอก..." } else { "2. กรอกและบันทึกหน้านี้" }
                }
            }

            p { class: "hint",
                "หลังเปิดหน้า SGS ให้เลือกปีการศึกษา ชั้น ห้อง และรายวิชาในหน้าต่างเบราว์เซอร์ให้ถูกต้อง ก่อนกดกรอกข้อมูล"
            }
            ExcelDataInfo { summary }
        }
    }
}

async fn fill_current_page(
    page: ExcelPage,
    log: &mut Signal<Vec<String>>,
    app_state: AppState,
) -> Result<()> {
    let excel_data = app_state
        .excel
        .lock()
        .await
        .clone()
        .ok_or_else(|| anyhow!("ยังไม่ได้เลือกไฟล์ Excel"))?;
    let guard = app_state.session.lock().await;
    let session = guard.as_ref().ok_or_else(|| anyhow!("ยังไม่ได้เข้าสู่ระบบ"))?;

    let current_url = session.current_url().await?;
    if !page.matches_url(&current_url) {
        return Err(anyhow!(
            "หน้าเว็บปัจจุบันไม่ใช่หน้า {} กรุณากด ‘เปิดหน้า SGS’ ก่อน",
            page.thai()
        ));
    }

    let web_students = session.scrape_students(page).await?;
    if web_students.is_empty() {
        return Err(anyhow!("ไม่พบตารางนักเรียน กรุณาตรวจตัวกรองในหน้า SGS"));
    }
    push_log(log, format!("พบรายชื่อบนหน้า SGS {} คน", web_students.len()));

    let students: HashMap<i64, _> = excel_data
        .sorted_students()
        .into_iter()
        .map(|student| (student.id, student))
        .collect();
    let mut matched = 0usize;
    let mut filled_fields = 0usize;
    let mut failed_fields = 0usize;

    for web_student in &web_students {
        let Ok(student_id) = web_student.id.trim().parse::<i64>() else {
            continue;
        };
        let Some(student) = students.get(&student_id) else {
            continue;
        };
        matched += 1;

        let mut scores: Vec<_> = student
            .scores
            .iter()
            .filter(|(column, _)| page.accepts_column(column))
            .collect();
        scores.sort_by(|(left, _), (right, _)| {
            natural_column_key(left).cmp(&natural_column_key(right))
        });

        for (column, value) in scores {
            match session
                .fill_score(
                    page,
                    &web_student.control_token,
                    column,
                    &format_number(*value),
                )
                .await
            {
                Ok(()) => filled_fields += 1,
                Err(error) => {
                    failed_fields += 1;
                    push_log(
                        log,
                        format!(
                            "กรอก {} ({}) ช่อง {} ไม่สำเร็จ: {error}",
                            student.name, student.id, column
                        ),
                    );
                }
            }
        }
    }

    if matched == 0 {
        return Err(anyhow!(
            "รหัสนักเรียนบนหน้า SGS ไม่ตรงกับข้อมูลใน Excel แม้แต่คนเดียว จึงยังไม่บันทึก"
        ));
    }
    if filled_fields == 0 {
        return Err(anyhow!(
            "พบรายชื่อตรงกัน {matched} คน แต่ไม่มีคะแนนสำหรับหน้า {} จึงยังไม่บันทึก",
            page.thai()
        ));
    }
    if failed_fields > 0 {
        return Err(anyhow!(
            "มีช่องที่กรอกไม่สำเร็จ {failed_fields} ช่อง จึงยังไม่กดบันทึก กรุณาตรวจ log"
        ));
    }

    session.click_save(page).await?;
    push_log(
        log,
        format!(
            "บันทึกสำเร็จ: ตรงกัน {matched}/{} คน กรอก {filled_fields} ช่อง",
            web_students.len()
        ),
    );
    Ok(())
}

fn natural_column_key(column: &str) -> (u8, u16) {
    if column == "Midterm" {
        return (0, 100);
    }
    if column == "Final" {
        return (0, 200);
    }
    let prefix = column.as_bytes().first().copied().unwrap_or_default();
    let number = column
        .get(1..)
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    (prefix, number)
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::{Reader, Xlsx};
    use std::io::Cursor;

    #[test]
    fn embedded_sample_has_expected_headers() -> Result<()> {
        let mut workbook = Xlsx::new(Cursor::new(SAMPLE_EXCEL))?;
        let sheet_names = workbook.sheet_names().to_vec();
        assert!(!sheet_names.is_empty());
        for sheet_name in sheet_names {
            let range = workbook.worksheet_range(&sheet_name)?;
            let headers = range.rows().next().context("ไฟล์ตัวอย่างไม่มีหัวตาราง")?;
            assert_eq!(
                headers.first().map(ToString::to_string).as_deref(),
                Some("stdID")
            );
            assert_eq!(
                headers.get(1).map(ToString::to_string).as_deref(),
                Some("student Name")
            );
        }
        Ok(())
    }

    #[test]
    fn formats_whole_numbers_without_decimal() {
        assert_eq!(format_number(30.0), "30");
        assert_eq!(format_number(2.5), "2.5");
    }

    #[test]
    fn sorts_numbered_columns_naturally() {
        assert!(natural_column_key("S2") < natural_column_key("S10"));
    }
}
