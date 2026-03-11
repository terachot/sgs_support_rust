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
               "📖 หน้า Work"
            }
         }
      }
   }
}

#[component]
pub fn student_data() -> Element {
    rsx! {
      div { class: "p-2 grow flex flex-col justify-center items-center min-h-0 overflow-y-auto",
         h4 { "การแสดงข้อมูลต่อหน้า" }
         input {
            class: "text-center p-1 m-2",
            r#type: "text",
            placeholder: "กรุณาใส่ตัวเลข",
            value: "40",
            onchange: move |_| println!("ตั้งค่าข้อมูลต่อหน้า"),
         }
         h3 { "อัพโหลดข้อมูลนักเรียน" }
         button {
            class: "m-3 px-6 py-2 bg-blue-500 text-white font-semibold rounded-lg shadow-md transition transform duration-150 hover:bg-blue-600 active:scale-95",
            onclick: move |_| println!("ดึงข้อมูลจาก excel"),
            "อัพโหลด"
         }
      }
   }
}

#[component]
pub fn score_before() -> Element {
    rsx! {
      div { class: "p-2 grow flex flex-col justify-center items-center min-h-0 overflow-y-auto",
         h3 { "ลงคะแนนก่อนกลางภาค" }
         button {
            class: "m-3 px-6 py-2 bg-blue-500 text-white font-semibold rounded-lg shadow-md transition transform duration-150 hover:bg-blue-600 active:scale-95",
            onclick: move |_| println!("ลงคะแนนก่อนกลางภาค"),
            "ลงคะแนนก่อนกลางภาค"
         }
      }
   }
}

#[component]
pub fn score_after() -> Element {
    rsx! {
      div { class: "p-2 grow flex flex-col justify-center items-center min-h-0 overflow-y-auto",
         h3 { "ลงคะแนนหลังกลางภาค" }
         button {
            class: "m-3 px-6 py-2 bg-blue-500 text-white font-semibold rounded-lg shadow-md transition transform duration-150 hover:bg-blue-600 active:scale-95",
            onclick: move |_| println!("ลงคะแนนหลังกลางภาค"),
            "ลงคะแนนหลังกลางภาค"
         }
      }
   }
}

#[component]
pub fn score_attribute() -> Element {
    rsx! {
      div { class: "p-2 grow flex flex-col justify-center items-center min-h-0 overflow-y-auto",
         h3 { "ลงคะแนนคุณลักษณะ" }
         button {
            class: "m-3 px-6 py-2 bg-blue-500 text-white font-semibold rounded-lg shadow-md transition transform duration-150 hover:bg-blue-600 active:scale-95",
            onclick: move |_| println!("ลงคะแนนคุณลักษณะ"),
            "ลงคะแนนคุณลักษณะ"
         }
      }
   }
}

#[component]
pub fn score_study() -> Element {
    rsx! {
      div { class: "p-2 grow flex flex-col justify-center items-center min-h-0 overflow-y-auto",
         h3 { "ลงคะแนนการอ่านคิดวิเคราะห์" }
         button {
            class: "m-3 px-6 py-2 bg-blue-500 text-white font-semibold rounded-lg shadow-md transition transform duration-150 hover:bg-blue-600 active:scale-95",
            onclick: move |_| {
                println!(
                    "ลงคะแนนการอ่านคิดวิเคราะห์",
                )
            },
            "ลงคะแนนการอ่านคิดวิเคราะห์"
         }
      }
   }
}

#[component]
pub fn welcome_page() -> Element {
    rsx! {
      div { class: "p-2 w-full flex flex-col items-center",
         h3 { "วิธีการใช้งานแอพ" }
         h5 {
            "1. โหลดข้อมูลตัวอย่างจากปุ่มด้านล่าง"
            br {}
            "2. ลงคะแนนในไฟล์ที่โหลดไป"
            br {}
            "3. กดหน้าข้อมูลนักเรียนเพื่ออัพโหลดข้อมูล"
            br {}
            "4. กดไปหน้าที่ต้องการลงคะแนน เลือกวิชาและชั้น"
            br {}
            "5. กดลงคะแนน เมื่อเสร็จสิ้นจะมีข้อครามแสดงขึ้นมา"
         }
         button {
            class: "m-3 px-6 py-2 bg-blue-500 text-white font-semibold rounded-lg shadow-md transition transform duration-150 hover:bg-blue-600 active:scale-95",
            onclick: move |_| { println!("โหลดไฟล์ตัวอย่าง") },
            "โหลดไฟล์ตัวอย่าง"
         }
      }
   }
}