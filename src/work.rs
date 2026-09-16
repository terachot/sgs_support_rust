//! หน้าทำงานสำหรับข้อมูลทั้งสี่ประเภท

use dioxus::prelude::*;

use crate::compos::{AppState, ExcelSummary, Header, LogPanel, TabPage};
use crate::excel::Page as ExcelPage;

#[component]
pub fn Work() -> Element {
    let app_state = use_context::<AppState>();
    let mut active: Signal<ExcelPage> = use_signal(|| ExcelPage::Before);
    let log: Signal<Vec<String>> = use_signal(|| vec!["พร้อมทำงาน".to_string()]);
    let summary: Signal<Option<ExcelSummary>> = use_signal(|| None);
    let is_busy = use_signal(|| false);

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
            }

            TabPage { page: active(), log, app_state, summary, is_busy }
        }
        LogPanel { log }
    }
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
