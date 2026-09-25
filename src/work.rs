//! หน้าทำงานสำหรับข้อมูลทั้งสี่ประเภท

use dioxus::prelude::*;
use dioxus_router::Navigator;

use crate::compos::{push_log, AppState, ExcelSummary, Header, LogPanel, TabPage};
use crate::excel::Page as ExcelPage;

#[component]
pub fn Work() -> Element {
    let app_state = use_context::<AppState>();
    let mut active: Signal<ExcelPage> = use_signal(|| ExcelPage::Before);
    let log: Signal<Vec<String>> = use_signal(|| vec!["พร้อมทำงาน".to_string()]);
    let summary: Signal<Option<ExcelSummary>> = use_signal(|| None);
    let is_busy = use_signal(|| false);
    let navigator = use_navigator();

    rsx! {
        Header {}

        main { class: "work-layout",
            div { class: "tab-bar",
                TabButton {
                    label: "ก่อนกลางภาค",
                    active: active() == ExcelPage::Before,
                    onclick: move |_| active.set(ExcelPage::Before),
                }
                TabButton {
                    label: "หลังกลางภาค",
                    active: active() == ExcelPage::After,
                    onclick: move |_| active.set(ExcelPage::After),
                }
                TabButton {
                    label: "คุณลักษณะ",
                    active: active() == ExcelPage::Attribute,
                    onclick: move |_| active.set(ExcelPage::Attribute),
                }
                TabButton {
                    label: "อ่าน คิด เขียน",
                    active: active() == ExcelPage::Study,
                    onclick: move |_| active.set(ExcelPage::Study),
                }
                button {
                    class: "button button-danger button-small logout-button",
                    disabled: is_busy(),
                    onclick: {
                        let app_state = app_state.clone();
                        move |_| start_logout(app_state.clone(), log, is_busy, navigator)
                    },
                    "ออกจากระบบ"
                }
            }

            TabPage { page: active(), log, app_state: app_state.clone(), summary, is_busy }
        }
        LogPanel { log }
    }
}

fn start_logout(
    app_state: AppState,
    mut log: Signal<Vec<String>>,
    mut is_busy: Signal<bool>,
    navigator: Navigator,
) {
    spawn(async move {
        is_busy.set(true);
        push_log(&mut log, "กำลังออกจากระบบ SGS...");
        let mut session_guard = app_state.session.lock().await;
        if let Some(session) = session_guard.as_ref() {
            if let Err(error) = session.logout().await {
                push_log(&mut log, format!("ออกจากระบบบนเว็บไม่สำเร็จ: {error}"));
                is_busy.set(false);
                return;
            }
        }
        let session = session_guard.take();
        drop(session_guard);

        if let Some(session) = session {
            if let Err(error) = session.close().await {
                push_log(&mut log, format!("ปิดเบราว์เซอร์ไม่สมบูรณ์: {error}"));
                is_busy.set(false);
                return;
            }
        }
        *app_state.excel.lock().await = None;
        navigator.push(crate::Route::Home);
    });
}

#[component]
fn TabButton(label: String, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: if active {
                "tab-button active"
            } else {
                "tab-button"
            },
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}
