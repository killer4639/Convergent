# Run etcd locally with Docker

Use this container to run a single local etcd instance for the distributed barrier prototype.

## Start etcd

```bash
docker run -d --name etcd \
  -p 2379:2379 \
  -p 2380:2380 \
  gcr.io/etcd-development/etcd:v3.5.18 \
  /usr/local/bin/etcd \
  --name etcd0 \
  --listen-client-urls http://0.0.0.0:2379 \
  --advertise-client-urls http://0.0.0.0:2379 \
  --listen-peer-urls http://0.0.0.0:2380 \
  --initial-advertise-peer-urls http://0.0.0.0:2380 \
  --initial-cluster etcd0=http://0.0.0.0:2380 \
  --initial-cluster-state new
```

## Notes

- Client traffic uses port `2379`.
- Peer traffic uses port `2380`.
- The container name is `etcd`, which makes it easy to stop and remove later.

## Useful follow-up commands

Stop the container:

```bash
docker stop etcd
```

Remove the container:

```bash
docker rm etcd
```
