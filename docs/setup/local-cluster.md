# Run three barrier containers locally

This setup runs one local etcd container and three identical barrier containers on the same Docker network.

## Prerequisites

- Docker Desktop running locally.
- The repository root as the current working directory.

## How it works

- `etcd` runs as the coordination service.
- `barrier` is built once from the repository `Dockerfile`.
- The builder image installs `protobuf-compiler` because `etcd-client` needs `protoc` at compile time.
- `docker compose up --scale barrier=3` starts three copies of the same barrier container.
- Each barrier container gets the same `ETCD_ENDPOINT`, `PARTIES`, and `BARRIER_KEY` environment variables.

## Start the cluster

```bash
docker compose up --build --scale barrier=3
```

## What to expect

- You should see one `etcd` container and three `barrier` containers start.
- Each barrier container sleeps for a random interval, increments the shared etcd counter, then polls every 50 ms until the generation reaches the configured party count.
- Barrier progress logs appear in the combined compose output.

## Stop the cluster

```bash
docker compose down
```

## Reset the etcd state

Because the current barrier is only a basic polling prototype, previous keys remain in etcd between runs. The simplest reset is to remove the compose stack and start fresh:

```bash
docker compose down -v
```

## Notes

- `ETCD_ENDPOINT` is set to `http://etcd:2379` inside Docker because `localhost` inside a container points back to that same container, not to the etcd service.
- `PARTIES` is set to `3`, which matches the three scaled barrier containers.
- The current barrier logic is still a learning prototype. It is not yet a fully reusable multi-generation distributed barrier.