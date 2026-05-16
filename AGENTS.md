# Repository Guidelines

## Project Structure & Module Organization
This repository is a small Rust binary crate. Keep application code in `src/`, with the current entry point at `src/main.rs`. Add internal modules under `src/` using focused files such as `src/api.rs` or `src/config/mod.rs`. Put integration tests in `tests/` when they exercise the compiled binary or public behavior. `Cargo.toml` defines crate metadata and dependencies, and `Cargo.lock` should remain committed for reproducible builds.

## Build, Test, and Development Commands
- `cargo build`: compile the crate in debug mode.
- `cargo run`: build and run the local binary.
- `cargo test`: run unit and integration tests.
- `cargo fmt`: format the codebase with Rustfmt.
- `cargo clippy --all-targets --all-features -D warnings`: catch lint issues before review.

Run commands from the repository root.

## Coding Style & Naming Conventions
Use Rust’s standard 4-space indentation and keep formatting tool-driven with `cargo fmt`. Follow idiomatic naming:
- `snake_case` for functions, modules, and file names.
- `CamelCase` for structs, enums, and traits.
- `SCREAMING_SNAKE_CASE` for constants.

Prefer small modules with clear ownership boundaries. If logic grows beyond `main.rs`, move it into named modules instead of building large inline blocks.

## Testing Guidelines
Write unit tests next to the code they validate using `#[cfg(test)] mod tests`. Add integration tests in `tests/` for CLI or end-to-end behavior. Name tests by behavior, for example `returns_404_for_missing_route`. Every feature or bug fix should include tests unless the change is purely refactoring or documentation.

## Commit & Pull Request Guidelines
This repository does not have commit history yet, so use imperative, conventional commit messages such as `feat: add health check endpoint` or `fix: handle missing config`. Keep PRs small and reviewable. Include:
- a short description of the change,
- testing notes with the commands you ran,
- linked issues when applicable,
- sample output or screenshots only when behavior is user-visible.

## Configuration Notes
Do not commit secrets or machine-specific configuration. Prefer environment variables for runtime settings, and document any new required variables in the PR description until a dedicated config section exists.
