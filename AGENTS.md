# Repository Guidelines

## Project Structure & Module Organization

`prep` is a Rust command-line tool for scaffolding and running machine-setup cookbooks. Source lives in `src/`: `main.rs` wires the CLI to execution, `cli.rs` defines Clap commands, and `cookbook.rs`, `recipe.rs`, and `step.rs` model and run cookbook data. `logging.rs` configures tracing and `utils.rs` contains shared helpers. Embedded starter files for `prep init` live in `assets/`; keep their paths aligned with the `include_str!` calls in `src/utils.rs`. CI workflows are in `.github/workflows/`.

## Build, Test, and Development Commands

- `cargo run -- --help` builds the CLI and shows its interface.
- `cargo build` compiles the debug binary; use `cargo build --release` for an optimized build.
- `cargo test` runs the unit tests.
- `cargo fmt --all` formats Rust sources. Check formatting with `cargo fmt --all -- --check`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` runs the same lint gate as CI.

Before opening a pull request, run the formatting check, Clippy command, and `cargo test --workspace --all-targets --release -- --nocapture`.

## Coding Style & Naming Conventions

Use Rust 2024 idioms and let `rustfmt` control formatting (four-space indentation). Name modules, functions, and fields in `snake_case`; types and enums in `PascalCase`; and constants in `SCREAMING_SNAKE_CASE`. Keep CLI options and TOML keys lowercase with hyphens or underscores as their existing interfaces require. Add `///` documentation to public types and behavior that needs explanation. Preserve contextual `anyhow` errors and structured `tracing` logs around filesystem and command execution.

## Testing Guidelines

Place unit tests in each module's `#[cfg(test)] mod tests` block. Use descriptive names such as `plan_orders_recipes_by_dependencies` and create temporary filesystem fixtures with `tempfile` when testing cookbook input. Cover success cases and invalid TOML, missing files, dependency, or command-failure paths when changing behavior. There is no stated coverage threshold; CI requires all tests and warnings-free Clippy to pass.

## Commit & Pull Request Guidelines

Write concise imperative commit subjects, for example `Add recipe validation` or `Fix missing documentation for struct members`. Keep each commit focused. Pull requests should explain the user-visible change, link the related issue when available, and list validation commands run. Include sample CLI output or screenshots only when documentation, logs, or user-facing output changes.
