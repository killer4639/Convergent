# Convergent

Convergent is a Rust project focused on building a distributed barrier synchronization mechanism.

## Overview

The goal of this repository is to explore how multiple distributed participants can coordinate around a shared barrier and only continue once every required participant has reached the synchronization point.

## Current direction

The project is being developed as an implementation of distributed barrier synchronization in Rust, with room to grow into protocol design, node coordination, and test coverage as the system evolves.

## Quick start

### Prerequisites

- [Docker Desktop](https://www.docker.com/products/docker-desktop/) running

### Run the system

Start etcd, 3 barrier nodes, Prometheus, and Grafana:

```sh
docker compose up --build --scale barrier=3
```

### Visualize detection latency

1. **Prometheus targets** — open `http://localhost:9091/targets` and confirm 3 barrier nodes show as `UP`.
2. **Grafana dashboard** — open `http://localhost:3000` (no login required). The **Convergent — Detection Latency** dashboard loads automatically with:
   - Detection latency heatmap
   - P50 / P95 / P99 percentile time series
   - Per-node average detection latency
   - Barrier generations completed count
3. **Wait ~1–2 minutes** for data to appear — each barrier cycle takes 5–10 s (simulated workload).

### Stop

```sh
docker compose down
```

## Documentation

See [`docs/README.md`](docs/README.md) for the structured project documentation, including local setup notes such as running etcd in Docker.
