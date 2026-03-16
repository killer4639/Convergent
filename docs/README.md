# Documentation

This folder contains project documentation organized by topic so the repository can grow without losing operational notes, design decisions, or experiment results.

## Structure

- `setup/` for local environment setup, services, and developer workflows.
- `architecture/` for barrier design notes, coordination semantics, and future implementation decisions.
- `testing/` for correctness strategy, harness notes, and validation workflows.
- `experiments/` for measurements, comparisons, and performance findings.

## Documentation rules

- Add or update docs when project behavior, setup steps, or architecture decisions change.
- Prefer focused documents in the matching subfolder instead of expanding unrelated files.
- Keep operational commands copy-paste ready.
- Link new documents from this index when they become important entry points.

## Current documents

- [`setup/local-etcd.md`](setup/local-etcd.md) - Start a local etcd instance in Docker for development.
- [`setup/local-cluster.md`](setup/local-cluster.md) - Build and run three barrier containers against one local etcd instance.
