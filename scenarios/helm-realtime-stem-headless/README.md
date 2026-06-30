# Helm Realtime Stem Headless

Headless interactive cases that stress the **Session Intelligence Spine** —
the multi-user realtime path Marquee apps such as Quorum use for live
concurrent inquiry — without a GUI or dependency on `marquee-apps/`.

Each case drives real [`helm-client`](https://github.com/Reflective-Lab/helms/tree/main/crates/helm-client)
[`ClientHelm`](https://github.com/Reflective-Lab/helms/tree/main/crates/helm-client/src/client.rs)
instances per simulated participant, coordinated with a deterministic server
loop pool for **short-running probes** and **long-running DD / synthesis**
formations.

## Resource declaration

**LOCAL REAL** — uses real Helm spine crates (`helm-client`,
`helm-session-contracts`, `director-contracts`) with scripted in-process
server pushes and server loop timing. No network, no SSE transport, no Converge
engine execution. This is the headless contract-shape proof Atelier owns; live
SSE transport proofs belong in `arena-tests` or the app repo.

## Interactive cases

| Case | What it stresses |
|------|------------------|
| `single-user-short-loop` | Baseline informational push → local spawn → completion |
| `server-offload-under-load` | Disruptive push offloads a long DD server loop while local loop continues |
| `preemptive-pivot` | Preemptive urgency pauses and reinjects local context |
| `three-user-concurrent-room` | Three participants with staggered parallel local loops |
| `long-and-short-server-mix` | Short probe + long synthesis server handles on one client |
| `gate-while-loops-active` | Gate surface while advisory traffic continues |
| `budget-exhaustion` | ClientHelm wall-clock budget fails a stuck local loop |
| `marquee-burst-room` | Marquee-shaped burst: 5 humans, mixed urgencies, server loops, gate, Converge admission |

List cases:

```bash
cargo run -p scenario-helm-realtime-stem-headless -- --list
```

Run the default Marquee burst room (Markdown):

```bash
cargo run -p scenario-helm-realtime-stem-headless
```

Run a specific case as JSONL:

```bash
cargo run -p scenario-helm-realtime-stem-headless -- --case three-user-concurrent-room --format jsonl
```

## Why generic substitutes fail

A single shared in-memory event list cannot prove the **local vs server-handle
registry invariant** (at most one local `Running` loop, unlimited server
handles), the **urgency routing table** (informational vs disruptive vs
preemptive), or **gate / temperature submission draining** — those behaviors
live in `ClientHelm` and must be exercised through the real coordinator.

## Pressure-test target

Surfaces cross-participant loop registry semantics and server offload timing
before Arena wires cross-repo assertions against `helm-session-host` SSE
mounts.
