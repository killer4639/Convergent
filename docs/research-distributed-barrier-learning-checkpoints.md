# Research Report: Distributed Barrier Performance Metrics, Benchmarks, Scale Ranges & Learning Checkpoints

## Executive Summary

Distributed barrier synchronization is a well-studied primitive in parallel and distributed computing. The standard metrics for evaluating barriers are **barrier latency**, **arrival skew**, **throughput** (barriers/sec), and **scalability** (how metrics change as node count grows). The literature spans from MPI-level microsecond barriers on HPC interconnects (3–100 µs) to coordination-service-backed barriers using ZooKeeper or etcd (1–50 ms range). For your etcd-backed, polling-based prototype running 3 Docker containers on a single laptop, realistic first targets are **sub-100ms barrier latency at 3 nodes** (your current setup) and **sub-20ms at 8 nodes** after switching from polling to etcd's watch API. The research also identifies concrete learning checkpoints that interleave Rust skill growth with distributed systems milestones, giving you measurable goals at every stage.

---

## 1. Standard Performance Metrics for Distributed Barrier Systems

### 1.1 Barrier Latency

**Definition:** The time from when the **last** participant arrives at the barrier to when **all** participants are released and resume execution.[^1][^2]

**What it captures:** The pure synchronization overhead — how fast the coordination mechanism can detect "everyone is here" and fan out the release signal. This is the single most important metric for a barrier.

**Sub-components to measure:**
- **Coordination latency:** Time for the last arrival to be written to the coordination store (etcd put + Raft commit)
- **Detection latency:** Time for all nodes to learn that the barrier condition is met (polling interval or watch notification delay)
- **Release latency:** Time from detection to the application code resuming (local processing overhead)

For your current system: with 50ms polling, the detection latency alone averages **25ms** (half the polling interval), dominating total barrier latency.

### 1.2 Arrival Skew

**Definition:** The time difference between the **earliest** and **latest** process arriving at the barrier.[^2][^3]

**What it captures:** Load imbalance across participants. High arrival skew means some nodes finish their work much faster and sit idle at the barrier.

**Why it matters for you:** In your prototype, each participant sleeps 5–10 seconds randomly before calling `barrier.wait()`. This means arrival skew can be up to **5 seconds** — dwarfing barrier latency. In a real system, arrival skew reflects computational load imbalance.

### 1.3 Throughput (Barriers per Second)

**Definition:** The number of complete barrier synchronization cycles the system can execute per unit time.[^1][^4]

**Formula:** `throughput = 1 / (average_arrival_skew + barrier_latency)`

**What it captures:** The system's capacity for repeated synchronization, relevant in iterative computations (e.g., BSP model, ML training synchronization).

### 1.4 Scalability Curve

**Definition:** How barrier latency and throughput change as the number of participating nodes increases.[^5][^6]

**What to plot:** Barrier latency (y-axis) vs. number of nodes (x-axis), with separate lines for different implementations (polling vs. watch, centralized vs. tree).

**Ideal behavior:** O(log N) latency growth for tree/dissemination algorithms. Centralized algorithms (like yours) grow O(N) in the worst case due to serial contention on the coordination store.

### 1.5 Additional Metrics

| Metric | Description | When to Add |
|--------|-------------|-------------|
| **etcd round-trips per barrier** | Number of etcd operations (get/put/txn) per barrier cycle per node | Phase 2 |
| **P99 barrier latency** | Tail latency — worst-case experiences | Phase 2 |
| **Recovery time** | Time to complete barrier after a node crash | Phase 3 |
| **etcd CPU/memory** | Resource consumption of the coordination service | Phase 3 |

---

## 2. Scale Ranges Studied in the Literature

### 2.1 MPI / HPC Context

MPI barrier research typically studies these ranges:[^7][^8][^9]

| Scale Range | Typical Use | Interesting Behaviors |
|-------------|-------------|----------------------|
| **2–8 nodes** | Unit testing, algorithm validation | Baseline behavior, no contention effects |
| **8–64 nodes** | Small clusters, most published benchmarks | Algorithm differences become visible |
| **64–256 nodes** | Medium HPC clusters | Tree/dissemination clearly outperforms centralized |
| **256–1024 nodes** | Large HPC | Network topology effects dominate |
| **1024–100,000+ nodes** | Supercomputers | Hardware-assisted barriers, specialized algorithms |

### 2.2 Coordination-Service Context (etcd/ZooKeeper)

For coordination-service-backed barriers, the relevant scale is much smaller:[^10][^11]

| Scale Range | Relevance | Notes |
|-------------|-----------|-------|
| **2–8 nodes** | Your prototype range | etcd handles this trivially; focus on correctness |
| **8–32 nodes** | Typical microservice/container coordination | Watch herd effect starts to appear |
| **32–100 nodes** | Large coordination groups | etcd write throughput becomes the bottleneck |
| **100–500 nodes** | Upper bound for single etcd cluster | Not typical for barrier use; etcd clusters serve 3–7 members, clients can be many more |

**Key insight for you:** etcd-backed barriers are not designed for 1000-node HPC-style use. They're designed for **tens to low hundreds** of coordinating services/containers. Your Docker-on-laptop setup with 3–16 nodes is squarely in the sweet spot.

### 2.3 Recommended Scale Progression for Your Project

```
Checkpoint 1:  3 nodes  (current — prove correctness)
Checkpoint 2:  8 nodes  (first scaling measurement)
Checkpoint 3: 16 nodes  (contention effects visible)
Checkpoint 4: 32 nodes  (watch herd effect, etcd write pressure)
Checkpoint 5: 64 nodes  (upper practical limit for this architecture)
```

---

## 3. Published Benchmarks and Evaluation Methodologies

### 3.1 MPI Barrier Benchmarks

#### OSU Micro-Benchmarks (osu_barrier)
The gold standard for MPI collective operation benchmarking.[^7]

**Typical results (modern HPC clusters with InfiniBand):**
| Cluster Size | Average Latency |
|-------------|----------------|
| 8 nodes (128 ranks) | 3–10 µs |
| 32 nodes (512 ranks) | ~9.25 µs |
| 128 nodes (2048 ranks) | 10–35 µs |
| 512+ nodes | 40–100+ µs |

**Methodology:**
- Run `osu_barrier` with increasing process counts
- Measure average latency over 1000+ iterations
- Report min, max, avg, standard deviation
- Warm-up phase before measurement

**Source:** [MVAPICH OSU Benchmarks](https://mvapich.cse.ohio-state.edu/benchmarks/)[^7], [Sigma2 documentation](https://documentation.sigma2.no/jobs/arm-perf/osu.html)[^8]

#### Intel MPI Benchmarks (IMB)
Similar methodology to OSU, part of the Intel oneAPI toolkit. Reports latency for `MPI_Barrier` at various process counts.[^9]

### 3.2 ZooKeeper Barrier Evaluation

ZooKeeper's barrier recipe uses ephemeral znodes and watches:[^12][^13]

**Performance characteristics:**
- **Notification latency:** ~1–5 ms for watch triggers on a healthy ensemble
- **Herd effect:** When N clients watch one znode, a single change triggers N notifications simultaneously, creating load spikes
- **Throughput:** ZooKeeper can handle tens of thousands of reads/sec but writes are slower (leader must replicate via Zab)

**Key paper:** "ZooKeeper: Wait-free coordination for Internet-scale systems" (Hunt et al., 2010)[^14]

### 3.3 etcd Performance Baselines

From etcd's official performance documentation and community benchmarks:[^15][^16]

**Single-node, SSD, Docker:**
| Operation | Light Load | Moderate (100 clients) | Heavy (1000 clients) |
|-----------|-----------|----------------------|---------------------|
| PUT latency | <1 ms | 1–3 ms | 3–10+ ms |
| GET latency | <1 ms | <2 ms | 2–6 ms |
| Throughput | 5K–15K req/s | 10K–30K req/s | 20K–50K req/s |

**Watch notification latency:** ~1 ms under light load, increasing with number of concurrent watchers[^17]

**Docker overhead:** 5–20% latency penalty vs. bare metal[^16]

### 3.4 Academic Barrier Algorithm Survey

**Key reference:** "A Survey of Barrier Algorithms for Coarse Grained Supercomputers" (Hoefler et al.)[^18]

**Algorithms compared:**
| Algorithm | Rounds | Messages | Best For |
|-----------|--------|----------|----------|
| **Centralized** (your design) | 1 | 2N | Small N, simple |
| **Dissemination** | ⌈log₂ N⌉ | N⌈log₂ N⌉ | Point-to-point networks |
| **Tree (k-ary)** | 2⌈log_k N⌉ | 2(N-1) | Shared memory, reducing contention |
| **Tournament** | ⌈log₂ N⌉ | N log₂ N | Low contention, moderate scale |
| **Butterfly** | log₂ N | N log₂ N | Specialized network topologies |

**Key reference:** "Algorithms for Scalable Synchronization on Shared-Memory Multiprocessors" (Mellor-Crummey & Scott, 1991)[^19]

### 3.5 Benchmarking Distributed Coordination Systems

**Key recent survey:** "How to Evaluate Distributed Coordination Systems" (arXiv:2403.09445, 2024)[^20]

This paper emphasizes the **lack of standardized benchmarks** for coordination primitives like barriers and proposes methodologies for comparing etcd, ZooKeeper, and similar systems. It confirms etcd generally outperforms ZooKeeper for barrier-like workloads.

---

## 4. Concrete Targets and Baselines for Your Project

### 4.1 Barrier Latency Targets

These are **realistic checkpoints** for an etcd-backed distributed barrier running in Docker on a single laptop:

| Implementation Stage | Nodes | Target Latency (median) | Target Latency (P99) | Notes |
|---------------------|-------|------------------------|-----------------------|-------|
| **Current (polling 50ms)** | 3 | ~50–75 ms | ~100 ms | Dominated by polling interval |
| **After watch API** | 3 | ~5–10 ms | ~20 ms | etcd watch ≈ 1–3ms + processing |
| **Watch API + tuning** | 8 | ~10–20 ms | ~30 ms | Slight increase from more arrivals |
| **Watch API** | 16 | ~15–30 ms | ~50 ms | Watch herd effect starts |
| **Watch API** | 32 | ~25–50 ms | ~80 ms | etcd write pressure grows |
| **Optimized (batched/tree)** | 32 | ~15–30 ms | ~50 ms | Tree reduces etcd operations |

### 4.2 Throughput Targets

| Implementation | Nodes | Target (barriers/sec) |
|---------------|-------|-----------------------|
| Polling (current) | 3 | ~10–15 (limited by polling) |
| Watch-based | 3 | ~50–100 |
| Watch-based | 8 | ~30–60 |
| Watch-based | 16 | ~20–40 |
| Optimized | 32 | ~15–30 |

### 4.3 etcd Operations Per Barrier Cycle

| Stage | Reads/node | Writes/node | Total ops (N nodes) |
|-------|-----------|------------|---------------------|
| Current (polling) | ~(50ms/latency) polls | 1 txn | ~N×(polls) + N |
| Watch-based | 0 (event-driven) | 1 txn | ~2N |
| Optimized (tree) | 0 | 1 txn (leaf) or 1 (inner) | ~2N (but less contention) |

---

## 5. Learning Checkpoints: Rust + Distributed Barriers

These checkpoints interleave Rust language skills with project milestones. Each one builds on the previous and gives you a **concrete, testable deliverable**.

### Phase 1: Foundation & Correctness (You Are Here → Next 4 Checkpoints)

---

#### ✅ Checkpoint 0: Where You Are Now
**What you've built:**
- Local barrier with `Mutex + Condvar` (single process)
- Distributed barrier with etcd optimistic locking + 50ms polling
- Docker Compose setup for 3 containers + etcd
- Basic multi-generation support

**Rust skills demonstrated:** async/await basics, `tokio::main`, basic error handling with `Result`, struct/impl, environment variable parsing.

---

#### 🔲 Checkpoint 1: Proper Error Types & Multi-Generation Reset
**Goal:** Make the distributed barrier robust and reusable across generations without manual etcd cleanup.

**Rust learning:**
- Define a custom `BarrierError` enum with `thiserror` or manual `impl Display + Error`
- Learn `From` trait for error conversion (etcd errors → your errors)
- Understand `enum` variants with data (e.g., `BarrierError::EtcdError(etcd_client::Error)`)

**Deliverable:**
- `DistBarrier::wait()` returns `Result<usize, BarrierError>` with your own error type
- Barrier automatically resets etcd state between generations (use generation-keyed keys — you already have `get_key()` doing this!)
- Unit test: 3 tasks on a single etcd → 5 consecutive generations complete correctly

**Measurable:** All `cargo test` pass, `cargo clippy` clean, no `#![allow(dead_code)]` needed.

---

#### 🔲 Checkpoint 2: Replace Polling with etcd Watch API
**Goal:** Eliminate the 50ms polling loop by using etcd's watch mechanism.

**Rust learning:**
- Working with `Stream` / async iterators (etcd watch returns a stream)
- `tokio::select!` for racing a watch event against a timeout
- Pinning (`Pin<Box<...>>`) if needed for stream handling
- Lifetime annotations — the watch stream may borrow from the client

**Deliverable:**
- `DistBarrier::wait()` uses `client.watch()` instead of `sleep + get` loop
- Barrier latency drops from ~50ms to ~5–10ms at 3 nodes
- Add a simple latency measurement: print elapsed time in `wait()`

**Measurable:** `docker compose up --scale barrier=3` shows barrier generation completing in <20ms consistently (vs. ~50–75ms before).

---

#### 🔲 Checkpoint 3: Structured Observability with `tracing`
**Goal:** Add production-quality logging and timing instrumentation.

**Rust learning:**
- The `tracing` crate ecosystem: `tracing`, `tracing-subscriber`, `tracing-futures`
- `#[instrument]` attribute macro for automatic span creation
- Structured fields: `tracing::info!(generation = gen, latency_ms = elapsed, "barrier released")`
- Layer composition in `tracing-subscriber` (fmt layer, filter layer)

**Deliverable:**
- Every barrier operation emits structured trace events (arrival, wait-start, released)
- JSON-formatted logs with timestamps, node ID, generation, and timing
- Can grep logs to extract barrier latency per generation per node

**Measurable:** `RUST_LOG=info cargo run` produces structured JSON logs. Can parse logs with `jq` to compute barrier latency statistics.

**Concrete crate additions:**
```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
```

---

#### 🔲 Checkpoint 4: First Benchmark Harness
**Goal:** Build a repeatable benchmark that measures barrier latency and throughput across generations.

**Rust learning:**
- `std::time::Instant` for high-resolution timing
- `Arc<Mutex<Vec<Duration>>>` or channel-based result collection
- `tokio::sync::Barrier` (Tokio's built-in) as a comparison baseline
- Writing integration tests that spawn multiple tasks

**Deliverable:**
- A `src/bench/` module (or separate binary) that:
  1. Spawns N participants as tokio tasks
  2. Runs K barrier generations
  3. Collects per-generation latency (last-arrival to all-released)
  4. Reports: min, max, median, P95, P99 latency; throughput (barriers/sec)
- Run against both local (Mutex+Condvar) and distributed (etcd) barriers
- Output a simple CSV or table

**Measurable:** Can produce a table like:
```
Implementation    | Nodes | Generations | Median Latency | P99 Latency | Throughput
local_barrier     |     8 |         100 |        0.02 ms |      0.1 ms |   5000/sec
dist_barrier_poll |     3 |          20 |       52.00 ms |     78.0 ms |     13/sec
dist_barrier_watch|     3 |          20 |        7.00 ms |     15.0 ms |     85/sec
```

---

### Phase 2: Correctness & Metrics Infrastructure (Checkpoints 5–8)

---

#### 🔲 Checkpoint 5: Correctness Properties as Tests
**Goal:** Codify the safety and liveness properties your barrier must satisfy.

**Rust learning:**
- Property-based testing with `proptest` or `quickcheck`
- `tokio::time::pause()` for deterministic async testing
- `Arc` + `AtomicUsize` for lock-free shared counters in tests

**Properties to test:**
1. **Safety:** No node passes the barrier before all N have arrived
2. **Liveness:** If all N nodes call `wait()`, all eventually return
3. **Generation isolation:** Generation G+1 doesn't start until generation G is fully released
4. **Uniqueness:** Each generation number is returned exactly once per barrier cycle

**Deliverable:** At least 4 integration tests encoding these properties, passing for both local and distributed barriers.

---

#### 🔲 Checkpoint 6: Prometheus Metrics Endpoint
**Goal:** Expose real-time metrics from each barrier participant.

**Rust learning:**
- `axum` or `hyper` for a lightweight HTTP server
- `prometheus` crate for metric types (Histogram, Counter, Gauge)
- Running a metrics server alongside the barrier logic with `tokio::spawn`
- `lazy_static!` or `once_cell` for global metric registries

**Metrics to expose:**
| Metric | Type | Description |
|--------|------|-------------|
| `barrier_latency_seconds` | Histogram | Time spent in `wait()` |
| `barrier_generations_total` | Counter | Total completed generations |
| `barrier_arrival_skew_seconds` | Histogram | First-to-last arrival time |
| `etcd_operations_total` | Counter | etcd ops per barrier cycle |
| `barrier_participants` | Gauge | Current node count |

**Deliverable:** `curl http://localhost:9090/metrics` returns Prometheus-formatted metrics. Can scrape with Prometheus in Docker Compose.

---

#### 🔲 Checkpoint 7: Docker Compose Observability Stack
**Goal:** Run the full system with Prometheus + Grafana locally.

**Rust learning:** (Primarily infrastructure, less Rust)
- Docker Compose multi-service orchestration
- Prometheus scrape configuration
- Grafana dashboard JSON provisioning

**Deliverable:**
- `compose.yaml` includes: etcd, N barrier nodes, Prometheus, Grafana
- Pre-built Grafana dashboard showing barrier latency, throughput, etcd ops
- `docker compose up --scale barrier=8` → open Grafana → see live metrics

---

#### 🔲 Checkpoint 8: Failure Injection Tests
**Goal:** Verify barrier behavior when nodes crash or are slow.

**Rust learning:**
- `tokio::time::timeout` for deadline enforcement
- Graceful shutdown patterns (`tokio::signal`, `CancellationToken`)
- `Drop` trait for cleanup (ensuring etcd state is correct when a node dies)

**Scenarios:**
1. Node crashes before arriving → barrier should eventually timeout or reconfigure
2. Node crashes after arriving but before release → other nodes must still release
3. etcd leader election during barrier → barrier must retry/recover
4. Straggler node (very slow) → measure impact on barrier latency

**Deliverable:** Integration tests for each scenario. Document expected vs. actual behavior.

---

### Phase 3: Optimization & Comparison (Checkpoints 9–12)

---

#### 🔲 Checkpoint 9: Scale Testing to 16–32 Nodes
**Goal:** Measure how your barrier performs as node count grows.

**Deliverable:**
- Scalability plot: barrier latency vs. node count (3, 8, 16, 32)
- Identify bottleneck (etcd write contention? watch fan-out? network?)
- Document findings in `docs/experiments/scale-test.md`

**Target baselines:**
| Nodes | Acceptable Median Latency |
|-------|--------------------------|
| 3 | <10 ms |
| 8 | <20 ms |
| 16 | <30 ms |
| 32 | <50 ms |

---

#### 🔲 Checkpoint 10: Tree-Based Barrier Reduction
**Goal:** Implement a tree-structured barrier that reduces etcd contention.

**Rust learning:**
- Generic trait design: `trait Barrier { async fn wait(&mut self) -> Result<usize, BarrierError>; }`
- Trait objects vs. generics (`dyn Barrier` vs. `impl Barrier`)
- Advanced async patterns: coordinating tree levels

**Algorithm:** Instead of N nodes all writing to one key, organize into a k-ary tree. Leaf nodes write to sub-barrier keys; inner nodes aggregate. Only the root touches the main barrier key.[^18]

**Deliverable:** Tree barrier implementation + comparison benchmarks vs. flat barrier.

---

#### 🔲 Checkpoint 11: TLA+ Model of Barrier Protocol
**Goal:** Formally specify and verify your barrier's safety properties.

**Learning:**
- TLA+ basics: variables, Init, Next, invariants
- TLC model checker
- Modeling distributed state transitions

**Deliverable:**
- TLA+ spec for your barrier protocol (flat and tree variants)
- Model-checked invariants: Safety (no early release), Liveness (all eventually release)
- Document in `docs/testing/tla-spec.md`

**Resources:**
- [Learn TLA+](https://www.learntla.com/)[^21]
- [TLA+ By Example](https://learning.tlapl.us/)[^22]
- [tlaplus/Examples repository](https://github.com/tlaplus/Examples)[^23]

---

#### 🔲 Checkpoint 12: Dynamic Membership
**Goal:** Allow nodes to join/leave the barrier group at runtime.

**Rust learning:**
- etcd lease + keepalive for ephemeral membership registration
- `tokio::sync::watch` channel for propagating membership changes
- State machine design: `Joining → Active → Leaving → Gone`

**Algorithm:** Use etcd's lease mechanism — each node creates an ephemeral key with a TTL. The barrier reads the current member set from etcd before each generation. If a node's lease expires (crash), it's automatically removed.[^24][^25]

**Deliverable:**
- Nodes register/deregister with etcd leases
- Barrier adapts party count per generation
- Tests: node joins mid-run, node crashes mid-barrier, node leaves gracefully

---

## 6. Context: Where Distributed Barriers Are Used in Practice

Understanding *why* barriers exist helps motivate the benchmarks:

| Domain | Typical Scale | Barrier Frequency | Latency Tolerance |
|--------|--------------|-------------------|-------------------|
| **HPC / MPI** | 100–100K nodes | Every ~ms | µs (microseconds) |
| **BSP computation** (Pregel, Apache Giraph) | 10–1000 workers | Every superstep (~seconds) | ms (milliseconds) |
| **ML distributed training** (AllReduce) | 8–512 GPUs | Every gradient step | ms |
| **Microservice coordination** | 3–50 services | Rare (deployment, migration) | seconds |
| **Database schema migration** | 3–20 replicas | Once per migration | seconds |

**Your project sits in the BSP/coordination range** — ms-level latency, tens of nodes, moderate frequency. This is the right scope for an etcd-backed design.

---

## Confidence Assessment

**High confidence:**
- MPI barrier latency numbers are well-documented by OSU and Intel benchmarks
- etcd performance baselines are published in official documentation
- Barrier algorithm complexity (centralized O(N), tree O(log N)) is well-established
- The recommended scale ranges for etcd-backed coordination are well-understood

**Medium confidence:**
- Specific latency targets for "your laptop, your Docker setup" will vary based on hardware. The targets given are reasonable ranges, not guarantees.
- Watch API latency improvement (5–10x over polling) is based on etcd documentation and general experience, not a direct benchmark of your specific code.

**Lower confidence:**
- Tree barrier improvement over flat barrier in the etcd context specifically — this is extrapolated from MPI literature. The etcd write pattern may not show the same gains.
- Dynamic membership complexity estimates — this depends heavily on etcd lease behavior under load.

**Assumptions made:**
- Your laptop has an SSD (not HDD). HDD would make etcd dramatically slower.
- Docker Desktop with WSL2 (typical Windows setup). Native Linux would be ~10–20% faster.
- etcd v3.5.x as specified in your Docker Compose.

---

## Footnotes

[^1]: [Workload Analysis – RisingWave documentation](https://docs.risingwave.com/performance/workload-analysis) — throughput and latency definitions for distributed system benchmarking
[^2]: [Barrier Synchronization: A Comprehensive Guide for 2025 – Shadecoder](https://www.shadecoder.com/topics/barrier-synchronization-a-comprehensive-guide-for-2025) — arrival skew and barrier latency definitions
[^3]: [arXiv:2307.10248 – Synchronization strategies](https://arxiv.org/pdf/2307.10248) — arrival skew analysis in distributed barrier contexts
[^4]: [Benchmarking Distributed Systems: Metrics and Methodologies – ResearchGate](https://www.researchgate.net/profile/Stella-Amelia/publication/391773002) — throughput optimization methodologies
[^5]: [Distributed hardwired barrier synchronization – IEEE](https://ieeexplore.ieee.org/document/388040) — scalability analysis for barrier systems
[^6]: [Performance Evaluation for Distributed Systems – GeeksforGeeks](https://www.geeksforgeeks.org/system-design/performance-evaluation-for-distributed-systems/) — general distributed system performance metrics
[^7]: [MVAPICH OSU Micro-Benchmarks](https://mvapich.cse.ohio-state.edu/benchmarks/) — the standard MPI benchmark suite including osu_barrier
[^8]: [OSU Benchmark – Sigma2 documentation](https://documentation.sigma2.no/jobs/arm-perf/osu.html) — documented osu_barrier run: 32 nodes, 512 ranks, 9.25 µs average
[^9]: [OSU Microbenchmarks – ATS Benchmarks (LANL)](https://lanl.github.io/benchmarks/09_Microbenchmarks/M3_OSUMB/OSUMB.html) — Los Alamos National Lab benchmark documentation
[^10]: [Benchmarking Distributed Coordination Systems: A Survey – arXiv:2403.09445](https://arxiv.org/html/2403.09445v1) — comprehensive survey comparing etcd, ZooKeeper, and other coordination systems
[^11]: [Performance Evaluation of Apache ZooKeeper – Springer](https://link.springer.com/content/pdf/10.1007/978-3-030-11890-7_35.pdf) — ZooKeeper coordination performance evaluation
[^12]: [ZooKeeper Recipes: Barriers](https://zookeeper.apache.org/doc/current/recipes.html) — official barrier recipe using ephemeral znodes and watches
[^13]: [ZooKeeper barrier recipe source – GitHub](https://github.com/apache/zookeeper/blob/master/zookeeper-docs/src/main/resources/markdown/recipes.md)
[^14]: [ZooKeeper: Wait-free coordination for Internet-scale systems – USENIX](https://pdos.csail.mit.edu/6.824/papers/zookeeper.pdf) — original ZooKeeper paper with throughput benchmarks
[^15]: [Performance – etcd official documentation](https://etcd.io/docs/v3.5/op-guide/performance/) — sub-millisecond latency under light load, 30K+ req/sec
[^16]: [etcd Benchmark – OpenBenchmarking.org](https://openbenchmarking.org/test/pts/etcd) — community benchmark results
[^17]: [etcd ABFS Latency Reduction – GitHub](https://github.com/satyaram-tsaliki/etcd-abfs-latency-reduction) — research on optimizing etcd watch notification latency
[^18]: [A Survey of Barrier Algorithms for Coarse Grained Supercomputers – Hoefler et al.](http://htor.inf.ethz.ch/publications/img/hoefler-barrier-survey.pdf) — comprehensive algorithm comparison
[^19]: [Algorithms for Scalable Synchronization – Mellor-Crummey & Scott](https://read.seas.harvard.edu/~kohler/class/aosref/mellor-crummey91algorithms.pdf) — classic paper on scalable barrier algorithms
[^20]: [How to Evaluate Distributed Coordination Systems – arXiv:2403.09445](https://arxiv.org/abs/2403.09445) — 2024 survey on coordination system benchmarking methodology
[^21]: [Learn TLA+](https://www.learntla.com/) — comprehensive TLA+ learning resource
[^22]: [TLA+ By Example](https://learning.tlapl.us/) — interactive TLA+ tutorial
[^23]: [tlaplus/Examples – GitHub](https://github.com/tlaplus/Examples) — collection of TLA+ specifications including synchronization protocols
[^24]: [Dynamic Membership – Async Raft](https://async-raft.github.io/async-raft/dynamic-membership.html) — dynamic membership patterns in Raft-based systems
[^25]: [Jepsen – Distributed Systems Safety Research](https://jepsen.io/) — correctness testing framework for distributed systems; [Maelstrom](https://github.com/jepsen-io/maelstrom) for learning
