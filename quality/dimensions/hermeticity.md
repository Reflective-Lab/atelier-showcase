---
tags: [quality, dimension, hermeticity]
dimension: hermeticity
measures: RP-HERMETIC-UNIT
implementation: arena-tests/crates/dim-hermeticity
source: human + LLM
---

# Dimension — Hermeticity

The arena dimension that measures
[`RP-HERMETIC-UNIT`](../properties/RP-HERMETIC-UNIT.md). One number,
one verdict, one report line: did the workspace's unit tests stay
hermetic this run?

## What this dimension checks

For each workspace under the reflective root that has a `Cargo.toml`
with `[lib]` or `[[bin]]` plus tests, run the unit-test target under
a sandbox and record:

| Signal | Threshold |
|---|---|
| Attempted TCP/UDP `connect()` calls | Must be 0. |
| Reads of env vars matching `(?i).*_(API_KEY|TOKEN|SECRET|CREDENTIAL).*` | Must be 0. |
| Filesystem writes outside `$TMPDIR` (or `tempfile`-scoped paths) | Must be 0. |

## Verdict model

| Condition | Verdict |
|---|---|
| Any test issued a successful socket `connect()`. | **Fail** |
| Any test read a credential-shaped env var but did not open a socket. | **Warn** |
| No socket attempts, no credential reads. | **Pass** |

**Score** = `100 * (hermetic_tests / total_tests)`. A workspace
with 100 tests where 3 violate the property scores 97.

## How to read this dimension on the scoreboard

```text
hermeticity              FAIL   84/100     RP-HERMETIC-UNIT       12 340 ms
```

That line says:
- the workspace has **16% of its tests** doing something they shouldn't,
- aggregate verdict is **Fail** (so the whole arena run is Fail),
- the property at stake is **RP-HERMETIC-UNIT**, which you can
  read in [`../properties/RP-HERMETIC-UNIT.md`](../properties/RP-HERMETIC-UNIT.md).

When you open `reports/latest.md`, every Finding cites a specific
test name + the syscall trace. That's where to start the fix.

## What "implementation roadmap" means

`dim-hermeticity` is currently a `Skip` stub. The source file at
[`arena-tests/crates/dim-hermeticity/src/lib.rs`](../../arena-tests/crates/dim-hermeticity/src/lib.rs)
contains the full implementation roadmap. The short version:

1. Discover the unit-test targets in each workspace via
   `cargo metadata`.
2. Run each under a sandbox that denies network at the kernel layer
   (`unshare --net` on Linux, `DYLD_INTERPOSE` of `libc::connect` on
   macOS, or a `https_proxy=http://127.0.0.1:1` trap).
3. Trace syscalls (`strace -e network` on Linux, `dtruss -n` on
   macOS) or shim `libc` calls to count violations.
4. Emit one `Finding` per violating test.

Until that lands, this dimension reports `Skip` with the roadmap
in `evidence`. The presence of the `Skip` is itself a load-bearing
signal: it tells you the gate isn't enforced yet.

## How this dimension grades the codebase today (qualitative)

Pre-fix (June 2026):
- `bedrock-platform/axiom` — at least one test
  (`guide_heading_falls_back_to_local_on_no_backend`) was issuing
  real network requests. Now hermetic via
  [v0.15.2's DI refactor](../incidents/QF-2026-06-02-05.md).
- Other workspaces — unaudited.

Once `dim-hermeticity` lands, this section gets replaced by the
dashboard numbers in [`../dashboard.md`](../dashboard.md).

## Related dimensions

- [`determinism`](determinism.md) — checks that hermetic tests are
  also deterministic across N runs. (Hermeticity is necessary; not
  sufficient.)
- [`snapshot-portability`](snapshot-portability.md) — a sibling
  hygiene dimension: snapshots that bake in machine state are a
  cousin to tests that bake in env state.
