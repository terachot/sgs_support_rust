//! หน้าเปิดเบราว์เซอร์และเข้าสู่ระบบ SGS

use dioxus::prelude::*;
use dioxus_router::Navigator;

use crate::browser::{Session, Target};
use crate::compos::{push_log, AppState, Header, LogPanel};
use crate::credentials;

#[component]
pub fn Home() -> Element {
    let app_state = use_context::<AppState>();
    let saved = use_hook(|| credentials::load().map_err(|_| ()));
    let mut username = use_signal(|| {
        saved
            .as_ref()
            .ok()
            .and_then(Option::as_ref)
            .map_or_else(String::new, |value| value.username.clone())
    });
    let mut password = use_signal(|| {
        saved
            .as_ref()
            .ok()
            .and_then(Option::as_ref)
            .map_or_else(String::new, |value| value.password.clone())
    });
    let mut remember = use_signal(|| matches!(&saved, Ok(Some(_))));
    let mut target = use_signal(|| Target::Production);
    let is_busy = use_signal(|| false);
    let mut status = use_signal(|| {
        if saved.is_err() {
            "อ่านรหัสผ่านที่บันทึกไว้ไม่สำเร็จ".to_owned()
        } else {
            "เลือกปลายทางและกรอกข้อมูลเข้าสู่ระบบ".to_owned()
        }
    });
    let log = use_signal(|| vec!["พร้อมเริ่มการทำงาน".to_owned()]);
    let navigator = use_navigator();

    rsx! {
        Header {}
        main { class: "login-layout",
            section { class: "intro-panel",
                p { class: "eyebrow", "RUST + DIOXUS 0.7" }
                h2 { "กรอกผลการเรียนได้เร็วขึ้น โดยยังตรวจสอบทุกขั้นตอน" }
                p {
                    "โปรแกรมจะเปิด Chrome หรือ Edge แยกต่างหาก อ่านข้อมูลจาก Excel และจับคู่ด้วยรหัสนักเรียนก่อนกรอก"
                }
                ol { class: "feature-list",
                    li { "เข้าสู่ระบบ SGS ในหน้าต่างเบราว์เซอร์" }
                    li { "เลือกไฟล์ Excel ที่มี stdID และ student Name" }
                    li { "เลือกหน้า ห้อง และรายวิชา แล้วสั่งกรอก" }
                }
            }

            section { class: "login-card",
                div { class: "panel-heading",
                    div {
                        p { class: "eyebrow", "เริ่มต้น" }
                        h2 { "เข้าสู่ระบบ" }
                    }
                    span { class: "status-dot" }
                }

                div { class: "target-switch",
                    button {
                        class: if target() == Target::Production { "target active" } else { "target" },
                        disabled: is_busy(),
                        onclick: move |_| target.set(Target::Production),
                        "SGS จริง"
                    }
                    button {
                        class: if target() == Target::Mock { "target active" } else { "target" },
                        disabled: is_busy(),
                        onclick: move |_| target.set(Target::Mock),
                        "Mockup localhost"
                    }
                }
                p { class: "target-url", "{target().login_url()}" }

                label { class: "field",
                    span { "ชื่อผู้ใช้" }
                    input {
                        r#type: "text",
                        autocomplete: "username",
                        value: "{username}",
                        disabled: is_busy(),
                        oninput: move |event| username.set(event.value()),
                    }
                }
                label { class: "field",
                    span { "รหัสผ่าน" }
                    input {
                        r#type: "password",
                        autocomplete: "current-password",
                        value: "{password}",
                        disabled: is_busy(),
                        oninput: move |event| password.set(event.value()),
                        onkeydown: {
                            let app_state = app_state.clone();
                            move |event| {
                                if event.key() == Key::Enter && !is_busy() {
                                    start_login(LoginTask {
                                        username: username.read().clone(),
                                        password: password.read().clone(),
                                        password_field: password,
                                        target: target(),
                                        remember: remember(),
                                        app_state: app_state.clone(),
                                        log,
                                        status,
                                        is_busy,
                                        navigator,
                                    });
                                }
                            }
                        },
                    }
                }

                label { class: "remember-option",
                    input {
                        r#type: "checkbox",
                        checked: remember(),
                        disabled: is_busy() || target() != Target::Production,
                        oninput: move |event| {
                            if event.checked() {
                                remember.set(true);
                            } else {
                                match credentials::delete() {
                                    Ok(()) => remember.set(false),
                                    Err(_) => status.set("ลบรหัสผ่านที่บันทึกไว้ไม่สำเร็จ".to_owned()),
                                }
                            }
                        },
                    }
                    span { "จำรหัสผ่านใน Windows เครื่องนี้" }
                }

                button {
                    class: "button button-primary button-wide",
                    disabled: is_busy(),
                    onclick: {
                        let app_state = app_state.clone();
                        move |_| start_login(LoginTask {
                            username: username.read().clone(),
                            password: password.read().clone(),
                            password_field: password,
                            target: target(),
                            remember: remember(),
                            app_state: app_state.clone(),
                            log,
                            status,
                            is_busy,
                            navigator,
                        })
                    },
                    if is_busy() { "กำลังเชื่อมต่อ..." } else { "เปิดเบราว์เซอร์และเข้าสู่ระบบ" }
                }
                p { class: "login-status", "{status}" }
            }
        }
        LogPanel { log }
    }
}

struct LoginTask {
    username: String,
    password: String,
    password_field: Signal<String>,
    target: Target,
    remember: bool,
    app_state: AppState,
    log: Signal<Vec<String>>,
    status: Signal<String>,
    is_busy: Signal<bool>,
    navigator: Navigator,
}

fn start_login(task: LoginTask) {
    let LoginTask {
        username,
        password,
        mut password_field,
        target,
        remember,
        app_state,
        mut log,
        mut status,
        mut is_busy,
        navigator,
    } = task;
    if target == Target::Production && (username.trim().is_empty() || password.is_empty()) {
        status.set("กรุณากรอกชื่อผู้ใช้และรหัสผ่าน".to_owned());
        return;
    }

    spawn(async move {
        is_busy.set(true);
        status.set(format!("กำลังเปิด {}...", target.label()));
        push_log(&mut log, format!("เชื่อมต่อไปยัง {}", target.login_url()));

        let mut session_guard = app_state.session.lock().await;
        if let Some(old_session) = session_guard.take() {
            if let Err(error) = old_session.close().await {
                push_log(&mut log, format!("ปิดเบราว์เซอร์เดิมไม่สมบูรณ์: {error}"));
            }
        }

        let (session, source_message) = match Session::launch(target).await {
            Ok(result) => result,
            Err(error) => {
                status.set(format!("เปิดเบราว์เซอร์ไม่สำเร็จ: {error}"));
                push_log(&mut log, status.read().clone());
                is_busy.set(false);
                return;
            }
        };
        push_log(&mut log, source_message);

        if let Err(error) = session.open_login().await {
            status.set(format!("เปิดหน้าเข้าสู่ระบบไม่สำเร็จ: {error}"));
            push_log(&mut log, status.read().clone());
            let _ = session.close().await;
            is_busy.set(false);
            return;
        }
        status.set("กำลังเข้าสู่ระบบ...".to_owned());
        if let Err(error) = session.login(&username, &password).await {
            status.set(format!("เข้าสู่ระบบไม่สำเร็จ: {error}"));
            push_log(&mut log, status.read().clone());
            let _ = session.close().await;
            is_busy.set(false);
            return;
        }

        if target == Target::Production {
            let credential_result = tokio::task::spawn_blocking(move || {
                if remember {
                    credentials::save(&username, &password)
                } else {
                    credentials::delete()
                }
            })
            .await;
            if !matches!(credential_result, Ok(Ok(()))) {
                push_log(&mut log, "เข้าสู่ระบบสำเร็จ แต่จัดการรหัสผ่านที่บันทึกไว้ไม่สำเร็จ");
            }
        }

        *session_guard = Some(session);
        drop(session_guard);
        password_field.write().clear();
        status.set(format!("เข้าสู่ระบบ {} สำเร็จ", target.label()));
        push_log(&mut log, status.read().clone());
        is_busy.set(false);
        navigator.push(crate::Route::Work);
    });
}
