// work.rs — Compatible with Dioxus 0.7.x
use dioxus::prelude::*;

use crate::Route;
use crate::compos::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("assets/tailwind.css");

/// About page component
#[component]
pub fn Work() -> Element {
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
   let active_tab = use_signal(|| "tablinks");

   // fn รับ Signal เข้าไปแล้ว set ได้เลย
   fn show_content(mut tab: Signal<&'static str>, tabcontent: &'static str) {
        tab.set(tabcontent);
        println!("{}", tabcontent);
   }

   rsx! {
      //NavButtons { current: Route::Work }
      div { class: "grow flex flex-col justify-center items-center min-h-0 overflow-y-auto",
         div { class: "tab w-full",
            button {
               class: if active_tab() == "std_data" { "active" } else { "tablinks" },
               onclick: move |_| { show_content(active_tab, "std_data") },
               "ข้อมูลนักเรียน"
            }
            button {
               class: if active_tab() == "score_before" { "active" } else { "tablinks" },
               onclick: move |_| { show_content(active_tab, "score_before") },
               "ก่อนกลางภาค"
            }
            button {
               class: if active_tab() == "score_after" { "tablinks active" } else { "tablinks" },
               onclick: move |_| { show_content(active_tab, "score_after") },
               "หลังกลางภาค"
            }
            button {
               class: if active_tab() == "score_attribute" { "tablinks active" } else { "tablinks" },
               onclick: move |_| { show_content(active_tab, "score_attribute") },
               "คุณลักษณะ"
            }
            button {
               class: if active_tab() == "score_study" { "tablinks active" } else { "tablinks" },
               onclick: move |_| { show_content(active_tab, "score_study") },
               "อ่าน คิด เขียน"
            }
         }

         // แสดง content ตาม tab ที่ active อยู่
         div { class: "tabcontent",
            match active_tab() {
                "std_data" => rsx! {
                  student_data {}
               },
                "score_before" => rsx! {
                  score_before {}
               },
                "score_after" => rsx! {
                  score_after {}
               },
                "score_attribute" => rsx! {
                  score_attribute {}
               },
                "score_study" => rsx! {
                  score_study {}
               },
                _ => rsx! {
                  welcome_page {}
               },
            }
         }

         div { class: "grow flex flex-col justify-center items-center",
            button {
               class: "p-1 bg-blue-500 text-white font-semibold rounded-lg shadow-md transition transform duration-150 hover:bg-blue-600 active:scale-95",
               onclick: move |_| {
                   nav.push(Route::Home);
               },
               "กลับหน้าหลัก"
            }
         }
      }
   }
}