// compos.rs — Compatible with Dioxus 0.7.x
use dioxus::prelude::*;
use crate::Route;

#[component]
pub fn Header() -> Element {
    rsx! {
      div { class: "p-2 flex flex-row justify-between",
         h1 { class: "text-lg font-bold", "SGS Support" }
         h1 { class: "text-lg font-bold", "ปุ่มปรับ Theme" }
      }
   }
}

#[component]
pub fn Footer() -> Element {
    rsx! {
      div { class: "p-2",
         h3 { class: "text-base font-bold", "การทำงาน" }
         h4 { class: "overflow-y-auto leading-6 line-clamp-4 h-[6*4px] text-sm",
            "รอการเข้าสู่ระบบ"
         }
      }
   }
}

// ── NavButtons — shared navigation component ───
/// Props สำหรับ NavButtons: รับ current route เพื่อ highlight ปุ่มที่อยู่
#[derive(Props, Clone, PartialEq)]
pub struct NavButtonsProps {
    current: Route,
}

/// Component ปุ่ม navigation 3 ปุ่ม ใช้ร่วมกันได้ในทุกหน้า
/// ปุ่มที่ตรงกับหน้าปัจจุบันจะแสดง style "active"
#[component]
pub fn NavButtons(props: NavButtonsProps) -> Element {
    rsx! {
      section { class: "card nav-buttons-card",
         div { class: "nav-buttons-group",

            // ── ปุ่ม "หน้าแรก" → "/" (Home) ──
            Link {
               to: Route::Home,
               class: if props.current == Route::Home { "btn btn-nav btn-nav-active" } else { "btn btn-nav" },
               "🏠 หน้าแรก"
            }

            // ── ปุ่ม "หน้า Home" → "/home" แสดงเหมือน Home ──
            // (ชี้ไปที่ Route::Home เหมือนกัน เพราะ Home = route "/")
            Link {
               to: Route::Home,
               class: if props.current == Route::Home { "btn btn-nav btn-nav-active" } else { "btn btn-nav" },
               "📄 หน้า Home"
            }

            // ── ปุ่ม "หน้า About" → "/about" ──
            Link {
               to: Route::Work,
               class: if props.current == Route::Work { "btn btn-nav btn-nav-active" } else { "btn btn-nav" },
               "📖 หน้า About"
            }
         }
      }
   }
}