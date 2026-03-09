// home.rs — Compatible with Dioxus 0.7.x
use dioxus::prelude::*;

use crate::Route;
use crate::compos::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("assets/tailwind.css");

/// Home page component
#[component]
pub fn Home() -> Element {
    rsx! {
      document::Meta {
         name: "viewport",
         content: "width=device-width, initial-scale=1.0",
      }
      document::Link { rel: "icon", href: FAVICON }
      document::Link { rel: "stylesheet", href: MAIN_CSS }
      document::Link { rel: "stylesheet", href: TAILWIND_CSS }

      div { class: "flex flex-col h-screen justify-between min-w-lg min-h-60",
         Header {}
         Content {}
         Footer {}
      }
   }
}

#[component]
fn Content() -> Element {
   // use_navigator() ให้ navigator object สำหรับเปลี่ยนหน้าแบบ programmatic
   let nav = use_navigator();

   rsx! {
      div { class: "p-2 grow flex flex-col justify-center items-center min-h-0 overflow-y-auto",
         h2 { class: "text-lg font-bold w-md text-center", "ลงชื่อ" }
         input {
            class: "m-2 p-2 w-3/5 min-w-xs max-w-lg border-1 rounded-lg",
            placeholder: "ชื่อผู้ใช้",
         }
         input {
            class: "m-2 p-2 w-3/5 min-w-xs max-w-lg border-1 rounded-lg",
            placeholder: "รหัสผ่าน",
            r#type: "password",
         }
         button {
            class: "px-6 py-2 bg-blue-500 text-white font-semibold rounded-lg shadow-md transition transform duration-150 hover:bg-blue-600 active:scale-95",
            onclick: move |_| {
                nav.push(Route::Work);
            },
            "เข้าใช้งาน"
         }
      }
   }
}