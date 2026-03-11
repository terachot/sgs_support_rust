// ─────────────────────────────────────────────
//  main.rs  —  Entry point for the Dioxus app
//  Compatible with: Dioxus 0.7.x
// ─────────────────────────────────────────────

mod home;
mod work;
mod compos;

use dioxus::prelude::*;

use home::Home;
use work::Work;

//to run test: "dx serve"
//to build: "dx bundle --desktop"

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[route("/")]
    Home,

    #[route("/work")]
    Work,

    // Catch-all 404
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

// ── 404 Not Found page ─────────────────────
#[component]
fn NotFound(segments: Vec<String>) -> Element {
    let path = segments.join("/");
    rsx! {
        div { class: "page",
            section { class: "hero",
                h1 { "404 — Page Not Found" }
                p { "No route matched: /{path}" }
                Link { to: Route::Home, class: "btn btn-primary", "← Go Home" }
            }
        }
    }
}

// ── Application entry point ────────────────
fn main() {
    dioxus::LaunchBuilder::new()
        .with_cfg(desktop!({
            use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
            Config::new().with_window(
                WindowBuilder::default()
                    .with_title("SGS Support")
                    .with_inner_size(LogicalSize::new(600, 460)),
            )
        }))
        .launch(|| rsx! {
            Router::<Route> {}
        });
}