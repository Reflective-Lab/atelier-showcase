# crm-helm — headless CRM composition over the in-memory substrate

Repatriated from `helms/showcase/` (RFL-171 Seam A): this scenario now depends
only on Bedrock-owned contracts — `helm-module-contracts` (module mounting) and
`helm-event-substrate` (event hub, leases; `memory` + `sse` features) — plus the
in-memory `AppKernelStore`. No `runway-*` dependency (`cargo tree | grep runway`
is empty by design).

## What it demonstrates

Seven CRM capability modules (parties, documents, facts, conversations,
opportunities, workflow, metadata) assembled into one axum `Router` via the
`HelmModule` contract, driven headless with `tower::ServiceExt::oneshot` —
no TCP bind, no `RunwayAppHost`, no `StorageKit`. Output is JSONL events in
the same style as the other `helm-*-headless` scenarios.

## Honest state (assembly.complete event reports it)

- `hub_consumers: 0 / lease_consumers: 0` — the `EventHub` and
  `InMemoryLeaseStore` are allocated and injectable, but no module consumes
  them yet.
- The 7 mounted routers are status surfaces; the real gRPC service structs
  (e.g. `PartiesGrpc`) are not wired into the mounted axum surface yet.
- Both gaps are tracked as RFL-155 stress-gate scope: this scenario graduates
  when consumer counts are > 0 and real services answer through the mount.

## The deployed variant

The runway-backed composition (RunwayAppHost + StorageKit + serve) lives in
the helms repo at `apps/crm-helm` — app-platform territory. This scenario and
that app share the same module code; they differ only in the injected
substrate implementations (in-memory here, runway there). That symmetry is
the point of the seam.

## Run

```bash
cargo run -p scenario-crm-helm       # JSONL to stdout, terminates
cargo test -p scenario-crm-helm      # router + lease integration tests
```
