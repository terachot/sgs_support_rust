// ─────────────────────────────────────────────
//  main.rs  —  Entry point for the SGS Support app
//  Dioxus 0.7.x (desktop)
// ─────────────────────────────────────────────

mod browser;
mod compos;
mod excel;
mod home;
mod work;

use dioxus::prelude::*;

use crate::compos::AppState;
use crate::home::Home;
use crate::work::Work;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[route("/")]
    Home,

    #[route("/work")]
    Work,

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
                    .with_inner_size(LogicalSize::new(960, 720)),
            )
        }))
        .launch(|| {
            use_context_provider(AppState::new);
            rsx! { Router::<Route> {} }
        });
}
