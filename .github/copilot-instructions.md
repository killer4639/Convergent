# Copilot Instructions

## Build, test, and lint

- Build the binary with `cargo build`.
- Run the full test suite with `cargo test`.
- Run a single test with `cargo test <test_name>` once named tests are added.
- Check formatting with `cargo fmt --check`.
- Run linting with `cargo clippy --all-targets --all-features`.

## Collaboration style

- The project owner is building this solo for learning purposes and is new to Rust.
- When making non-trivial changes, explain the Rust-specific reasoning, ownership model, and synchronization choices in plain language.
- Prefer incremental implementations with tests over large jumps in architecture.
- Use a TDD-oriented workflow: define expected behavior first, implement the smallest working version, then harden and optimize.
- Surface tradeoffs clearly and suggest the smallest safe next step toward the distributed design.

## Project direction

- The current local barrier is only a prototype and learning step.
- The project goal is a distributed barrier prototype.
- Development and evaluation will happen on a single laptop using Docker containers to simulate a distributed deployment locally.
- Start with a consensus-backed centralized barrier using etcd as the coordination service.
- First build the simplest distributed barrier that works, even if it relies on basic polling against the coordination store.
- After the first working implementation, build a correctness and metrics harness before attempting meaningful optimizations.
- Only once correctness checks and metrics are in place should the project compare alternative implementations and iterate on performance.
- Treat this approach as the MVP because it offers well-defined failure semantics, dynamic membership support, production-grade fault tolerance, and defers more complex distributed algorithms.
- Keep a clear migration path for later optimizations such as tree topology and local fast paths, but do not introduce them before measuring the centralized design.
- Validate correctness before optimization, including model-based reasoning and eventual model checking with tools such as TLA+.
- Design the prototype and harness so they can simulate multiple nodes locally, inject timing/failure scenarios, and collect latency and throughput statistics from containerized runs.

## Delivery phases

- Phase 1: create a minimal distributed barrier prototype with straightforward behavior and strong tests, prioritizing clarity over efficiency.
- Phase 2: build tooling to test correctness, capture latency and throughput metrics from local multi-container runs, and make implementation tradeoffs measurable.
- Phase 3: iterate on the implementation, compare designs against the metrics harness, and document the results of each approach.

## High-level architecture

- This repository is a single-package Cargo project defined in `Cargo.toml`; it is not a workspace and does not include a `lib.rs`.
- The application is currently a single binary entrypoint in `src/main.rs`.
- The codebase currently contains local synchronization experiments under `src/local_test/` in addition to the binary entrypoint.
- Most non-trivial changes should introduce focused `src/` modules instead of continuing to grow a single file.

## Key conventions

- The crate name is `Convergent` with a capital `C` in `Cargo.toml`, so Cargo-generated binaries and test artifacts use that casing.
- The project targets Rust edition `2024`.
- Keep dependencies intentional. Add crates only when they materially support the distributed barrier prototype, testability, or correctness work.
- Keep `main()` as the binary entrypoint and move reusable logic into modules as the implementation grows.
- Prefer explicit error types and testable state machines for synchronization and distributed coordination code.
- When adding distributed coordination code, document the expected failure semantics and membership assumptions near the implementation or in the README.
- Prefer local orchestration and observability that work well on one machine, including containerized node simulation and repeatable metric collection.

## Documentation

- Maintain structured project documentation under `docs/`.
- Use `docs/README.md` as the documentation index and update it when new important documents are added.
- Put setup and local environment instructions under `docs/setup/`.
- Put design and architecture notes under `docs/architecture/`.
- Put correctness and validation notes under `docs/testing/`.
- Put benchmark, comparison, and metrics write-ups under `docs/experiments/`.
- When work changes setup, architecture, or evaluation workflow, update the matching document as part of the same change.
