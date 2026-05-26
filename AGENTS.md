You are an expert [0.7 Dioxus](https://dioxuslabs.com/learn/0.7) assistant. Dioxus 0.7 changes every api in dioxus. Only use this up to date documentation. `cx`, `Scope`, and `use_state` are gone

Provide concise code examples with detailed descriptions

# Dioxus Dependency

You can add Dioxus to your `Cargo.toml` like this:

```toml
[dependencies]
dioxus = { version = "0.7.1" }

[features]
default = ["web", "webview", "server"]
web = ["dioxus/web"]
webview = ["dioxus/desktop"]
server = ["dioxus/server"]
```

# Launching your application

You need to create a main function that sets up the Dioxus runtime and mounts your root component.

```rust
use dioxus::prelude::*;

fn main() {
	dioxus::launch(App);
}

#[component]
fn App() -> Element {
	rsx! { "Hello, Dioxus!" }
}
```

Then serve with `dx serve`:

```sh
curl -sSL http://dioxus.dev/install.sh | sh
dx serve
```

# UI with RSX

```rust
rsx! {
	div {
		class: "container", // Attribute
		color: "red", // Inline styles
		width: if condition { "100%" }, // Conditional attributes
		"Hello, Dioxus!"
	}
	// Prefer loops over iterators
	for i in 0..5 {
		div { "{i}" } // use elements or components directly in loops
	}
	if condition {
		div { "Condition is true!" } // use elements or components directly in conditionals
	}

	{children} // Expressions are wrapped in brace
	{(0..5).map(|i| rsx! { span { "Item {i}" } })} // Iterators must be wrapped in braces
}
```

# Assets

The asset macro can be used to link to local files to use in your project. All links start with `/` and are relative to the root of your project.

```rust
rsx! {
	img {
		src: asset!("/assets/image.png"),
		alt: "An image",
	}
}
```

## Styles

The `document::Stylesheet` component will inject the stylesheet into the `<head>` of the document

```rust
rsx! {
	document::Stylesheet {
		href: asset!("/assets/styles.css"),
	}
}
```

# Components

Components are the building blocks of apps

* Component are functions annotated with the `#[component]` macro.
* The function name must start with a capital letter or contain an underscore.
* A component re-renders only under two conditions:
	1.  Its props change (as determined by `PartialEq`).
	2.  An internal reactive state it depends on is updated.

```rust
#[component]
fn Input(mut value: Signal<String>) -> Element {
	rsx! {
		input {
            value,
			oninput: move |e| {
				*value.write() = e.value();
			},
			onkeydown: move |e| {
				if e.key() == Key::Enter {
					value.write().clear();
				}
			},
		}
	}
}
```

# Agent Guidelines for Rust Development

This document serves as your system instructions. You MUST follow these rules during all code generation, refactoring, and project management tasks.

## 🦀 Core Principles
1. **Memory Safety & Idiomatic Code:** Leverage Rust's type system and ownership model. Avoid `unsafe` blocks unless explicitly requested or necessary for FFI.
2. **Optimal Performance:** Maximize algorithmic efficiency and memory re-use. Minimize allocations (e.g., use references or `Cow` instead of copying strings where appropriate).
3. **DRY & Minimalism:** No extra code beyond what is absolutely necessary. Maximize code reuse.
4. **Tool Selection:** Prefer idiomatic, standard Rust features. When using crates, prefer mature, lightweight crates over bloated dependencies to achieve optimal performance.

## 🛠️ Preferred CLI & Tools
- Use `cargo` for project management, building, testing, and dependency management.
- Use `cargo check` frequently to catch errors.
- Run `cargo fmt` and `cargo clippy` on all modified code.
- Use `indicatif` to track long-running operations with progress bars.

## 🧪 Testing & Documentation Rules
- Generate unit tests using the `#[cfg(test)]` module inside the source files.
- Ensure all inline documentation (`///`) is fully updated for any public APIs.
- Write documentation tests (`/// ```rust`) and ensure they are compilable. Avoid `no_run` when tests can be executed natively.
- Run `cargo test` on all modified crates to verify correctness.

## 🚨 Error Handling & Concurrency
- Use standard Rust error propagation (e.g., the `?` operator, `Result<T, E>`). Do not use unwraps (`.unwrap()`) in production-ready code.
- Provide custom, strongly-typed error enums for complex crates.
- Use `tokio` for async workflows and `rayon` for data-parallelism when appropriate.

## 📝 Commit & Refactor Behavior
- Ensure your code is formatted (`cargo fmt`) before handing off.
- If Clippy flags issues, fix them immediately.
- Clearly comment on any complex logic or lifetime parameters (`'<a'>`).