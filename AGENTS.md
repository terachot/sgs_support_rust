# SGS Support — agent instructions

These instructions apply to this repository. Keep changes focused on the user's request and preserve unrelated work in the working tree.

## Project and toolchain

- This is a Windows desktop application written in Rust 2021 with Dioxus **0.7.10**. `Cargo.toml` and `Cargo.lock` are the source of truth for dependency versions.
- The default feature is `desktop`; do not turn this into a web or server app unless requested. The `web` feature exists, but the browser automation and native file picker are desktop-oriented.
- Use Dioxus 0.7 APIs: `#[component]`, `Element`, `rsx!`, `Signal` / `use_signal`, router `Routable` / `Router` / `Link`, and `document::Stylesheet`. Do not introduce removed patterns such as `cx`, `Scope`, or `use_state`.
- Check the [Dioxus 0.7 documentation](https://dioxuslabs.com/learn/0.7/) and the installed crate source when uncertain about an API. Avoid examples from older Dioxus versions.
- Prefer existing dependencies. Add a crate only when the standard library and current dependencies do not cover the task. Keep Dioxus-family versions aligned.

## Repository layout

| Path | Responsibility |
| --- | --- |
| `src/main.rs` | Desktop launch, shared context, and routes |
| `src/home.rs` | Login UI and production/mock target selection |
| `src/work.rs` | Four score-category tabs |
| `src/compos.rs` | Shared UI, Excel file picker, progress log, and fill workflow |
| `src/browser.rs` | Chrome/Edge session, navigation, scraping, field entry, and save action |
| `src/excel.rs` | Workbook parsing, score-column rules, URLs, and SGS selectors |
| `assets/main.css` | Desktop UI styling |
| `README.md` | User-facing setup and usage instructions |

The Rust implementation lives in this repository. `D:\Doc\Work\sgs-support` is the legacy Python reference; read it when comparing behavior, but do not edit it as part of routine Rust changes.

## SGS workflow and safety invariants

- Support both the production SGS site and `http://localhost/sgs_tester/`. Use `Target` and `excel::Page` as the central URL/selector definitions; avoid scattering literal URLs or control IDs through UI components.
- Each workbook sheet represents a room. The first two headers must be `stdID` and `student Name`; score columns are optional. Preserve the supported groups: `S1`–`S9` + `Midterm`, `S10`–`S18` + `Final`, `Q1`–`Q8`, and `L1`–`L5`.
- Match records by student ID, not row order or name. Read the actual ASP.NET repeater token from each web row instead of assuming `ctl00` or `ctl01`.
- Fill only students visible on the currently selected SGS page. Do not automatically cycle through workbook rooms or change the site's year/class/room/subject filters.
- Before saving, verify the page type, that at least one student ID matches, that at least one field was filled, and that no field failed. Do not click Save after a partial failure.
- Production SGS may contain real student data. Never log passwords or full workbook contents, commit local `.xlsx` files, or use the live site for write tests without a user request. Prefer the localhost mockup for end-to-end checks.
- Keep any browser-driven test of login, field entry, or Save narrowly scoped; do not assume a mockup result proves the production site is unchanged.

## Rust and Dioxus conventions

- Write idiomatic, memory-safe Rust. Avoid `unsafe` unless required for a documented FFI boundary. Propagate errors with `Result` and `?`; do not use `.unwrap()` in application paths.
- Use `tokio` for asynchronous work already present in the application. Keep blocking file-dialog work off the async runtime. Do not hold a mutex longer than the workflow requires.
- Keep UI state reactive with `Signal` and use shared `AppState` context for the browser session and loaded workbook. Do not introduce global mutable state.
- Prefer small, task-specific functions and avoid broad refactors. Update documentation when user-visible behavior or file format changes.
- Add unit tests under `#[cfg(test)]` in the relevant source file, especially for score-column mappings, URL/selector rules, Excel parsing, and student matching. Tests must not require production credentials or modify production data.

## Verification before handing off Rust changes

Run from the repository root:

```powershell
cargo fmt --all
cargo check --all-features
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
git diff --check
```

For documentation-only changes, inspect the diff and run `git diff --check`; a rebuild is unnecessary. Report any test or environment limitation honestly. Commit or push only when the user requests it; never include local student workbooks, credentials, or generated `target/` files.
