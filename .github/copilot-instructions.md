# Copilot Instructions

## Build, test, and lint

- Build the binary with `cargo build`.
- Run the full test suite with `cargo test`. The repository currently has no tests, so this completes with `0 passed`.
- Run a single test with `cargo test <test_name>` once named tests are added.
- Check formatting with `cargo fmt --check`.
- Run linting with `cargo clippy --all-targets --all-features`.

## High-level architecture

- This repository is a single-package Cargo project defined in `Cargo.toml`; it is not a workspace and does not include a `lib.rs`.
- The application is currently a single binary entrypoint in `src/main.rs`.
- Runtime flow is minimal: `main()` prints `Hello, world!` and exits. There are no internal modules, no external dependencies, and no feature flags yet.
- Most non-trivial changes will either expand `src/main.rs` directly or introduce new `src/` modules that are called from `main()`.

## Key conventions

- The crate name is `Convergent` with a capital `C` in `Cargo.toml`, so Cargo-generated binaries and test artifacts use that casing.
- The project targets Rust edition `2024`.
- `Cargo.toml` currently has an empty `[dependencies]` section. Treat this as a dependency-light crate and only add new crates when the change clearly needs them.
- Keep `main()` as the binary entrypoint and move reusable logic into modules if the application grows beyond a single-file CLI.
