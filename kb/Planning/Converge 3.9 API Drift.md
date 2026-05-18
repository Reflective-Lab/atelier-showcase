---
name: Converge 3.9 API Drift Sweep
description: Known blocker — atelier-showcase has un-migrated 3.9 API patterns across ~22 files. Tracked, not hidden.
source: mixed
---

# Converge 3.9 API Drift Sweep

**Status:** known blocker. **Not** in release scope until cleared.

`atelier-showcase` has a dependency-direction breakage against the
current `converge-pack` 3.9 API. The drift is real, mechanical, and
isolated to non-critical paths (library helpers, scenarios, several
tutorials). It is **not** on the organism-dynamics / formation-design
critical path — `scenarios/truth-driven-formation` and the
arena-tests integration tests both pass against the broken
workspace. But `cargo test --workspace` in atelier-showcase **does
not pass** until this is cleared, so the workspace-wide signal is
untrustworthy.

The user's instruction was explicit: do not paper over this with
`[workspace.exclude]`. Leave the broken surfaces visible. Track it
here and clear in a dedicated migration pass.

## Scope

- **22 files** with `fact.content()` (74 call sites)
- **20 files** with `ProposedFact::new` (mostly overlapping)
- Affected crates: `crates/atelier-domain/*`, `crates/organism-domain/*`,
  6 scenarios under `scenarios/*`, several tutorials under `tutorials/*`
- One tutorial (`tutorials/06-reconciliation-loop`) was migrated
  separately on 2026-05-18 as commit `85fbc79`. The other 21 files
  remain.

## Four mechanical patterns

The 3.9 migration introduced four breaking changes that the broken
surfaces pre-date. Each affected file needs the same shape of fix:

1. **`ContextFact::content()` removed.** Replace with a small helper
   that extracts the `TextPayload` and returns its `as_str()`:
   ```rust
   fn fact_text(fact: &ContextFact) -> &str {
       fact.payload::<TextPayload>().map_or("", TextPayload::as_str)
   }
   ```
2. **`ProposedFact::new` payload param** now requires
   `T: FactPayload + PartialEq`. Wrap raw `String` JSON payloads in
   `TextPayload::new(...)`.
3. **`ProposedFact::new` provenance param** takes
   `impl Into<Provenance>`. `Provenance: From<&'static str>` won't
   accept a borrowed `self.name()` — provide a `&'static str`
   constant per crate.
4. **`Suggestor::provenance()`** defaults to `""`; the kernel rejects
   facts with empty provenance at promotion (`EmptyProvenance`).
   Every fact-emitting `Suggestor` must override
   `fn provenance(&self) -> &'static str`.

## Reference port

`tutorials/06-reconciliation-loop/src/main.rs` (commit `85fbc79`) is
the canonical example of all four patterns applied. New cleanup work
should mirror its shape.

## Out of release scope

The four-pattern sweep across the remaining 21 files is mechanical
compatibility debt, distinct from architecture correctness work in
organism/dynamics. It blocks **release readiness** but not feature
work on the critical path. Do not declare the system release-ready
until this is cleared or the broken surfaces are explicitly
documented as out of release scope.
