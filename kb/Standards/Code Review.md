---
tags: [standards, review]
source: mixed
---
# Code Review

Code review in `atelier-showcase` is not limited to the files changed in
this checkout. The workspace path-patches foundation and organism crates,
so review must follow the actual build graph.

## Platform Drift

When a review finding exposes version drift, pin the intended platform
versions explicitly and run the workspace gates against those pins.

- Do not rely on caret resolution or local patches to imply the target
  Converge or Organism version.
- Do not use `[workspace.exclude]` to hide broken members from review.
- If a path-patched sibling checkout fails because the pinned API has
  moved, treat that as in-scope compatibility debt for the review.
- Keep release readiness blocked until either `cargo test --workspace`
  is clean or the broken surface is explicitly documented as out of
  release scope.

## Protocol Boundaries

Prefer typed protocol repair over string escape hatches.

- Do not fix API drift by converting typed ids or payloads through
  arbitrary `to_string()` / parse cycles inside the protocol.
- Preserve typed ids inside compiler, catalog, runtime, and dynamics
  data structures.
- Convert typed ids to `String` only at true external boundaries, such
  as human-facing output, error surfaces whose public contract is
  string-based, or factory maps keyed by string ids.
- Payload handoffs between agents should use real `FactPayload` types
  when the repo owns the schema. `TextPayload` is acceptable for
  genuinely unstructured human text, not for serialized local structs.
- Local `FactPayload` structs should use `serde(deny_unknown_fields)`
  unless there is a documented compatibility reason not to.

## Domain Records

Context key alone is not a schema. Domain-pack agents must verify the
payload family and record type before consuming a fact.

- Do not read a wrong-family fact and silently default to `{}`.
- `accepts()` should wake only when the expected record type is present.
- `execute()` should return an empty effect if the expected record is
  absent by the time it runs.
- Regression tests should include wrong-record-family cases for reusable
  domain-pack suggestors.

## Verification

A review fix that touches protocol, pinned versions, or path-patched
platform crates should normally clear:

- `cargo check --workspace`
- `cargo test --workspace`
- `just lint`
- Any feature-specific check that was part of the reviewed surface
- Scenario smoke runs for binaries that were reported broken
