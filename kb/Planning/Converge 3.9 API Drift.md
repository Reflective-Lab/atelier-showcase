---
name: Converge 3.9 API Drift Sweep
description: Cleared 2026-05-18 — atelier-showcase fully migrated to Converge 3.9 / Organism 1.9. Retained as the historical record of the four-pattern migration.
source: mixed
---

# Converge 3.9 API Drift Sweep

**Status:** ✅ **Cleared** on 2026-05-18 by commit `82e7767`
(scenarios were bumped separately in `c9d9569`). `cargo build
--workspace` and `cargo test --workspace` are clean against
Converge 3.9.1 and Organism 1.9.0.

This page is retained as the historical record of the four-pattern
migration so the same mistakes are not re-introduced.

## Original problem

`atelier-showcase` had a dependency-direction breakage against the
Converge 3.9 API. The drift was real, mechanical, and isolated to
non-critical paths (library helpers, scenarios, several tutorials).
It was **not** on the organism-dynamics / formation-design critical
path — `scenarios/truth-driven-formation` and the arena-tests
integration tests both passed against the broken workspace — but
`cargo test --workspace` did not, so the workspace-wide signal was
untrustworthy.

The instruction at the time was explicit: do not paper over the
breakage with `[workspace.exclude]`. Leave the broken surfaces
visible, track them here, and clear in a dedicated migration pass.

## Scope (cleared)

- **22 files** with `fact.content()` (74 call sites) — ported.
- **20 files** with `ProposedFact::new` (mostly overlapping) — ported.
- Affected crates: `crates/atelier-domain/*`, `crates/organism-domain/*`,
  scenarios under `scenarios/*`, tutorials under `tutorials/*`.
- `tutorials/06-reconciliation-loop` was the reference port (commit
  `85fbc79`, 2026-05-18). The remaining 21 files landed in
  `82e7767` on 2026-05-18.

## Four mechanical patterns

These four changes were applied to every affected file. They are
kept here so a future regression has a checklist:

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
3. **`ProposedFact::new` provenance param** takes a typed
   `Provenance`. Provide a crate-local `ProvenanceSource` marker and
   pass `MARKER.provenance()` or `MARKER.proposed_fact(...)`; do not
   pass semantic strings.
4. **`Suggestor::provenance()`** defaults to `""`; the kernel
   rejects facts with empty provenance at promotion
   (`EmptyProvenance`). Every fact-emitting `Suggestor` must
   override `fn provenance(&self) -> Provenance`.

## Follow-ons absorbed in the same pass

Two follow-ons from in-flight 1.9.x organism work were carried into
this clearing commit so the workspace would land green in one move:

- Tutorials 11 (`charter-from-intent`) and 12 (`shape-competition`)
  updated for the `UnitInterval` newtypes that now live on
  `DerivedCharter.confidence`, `IntentComplexity.*`,
  `ShapeCandidate.*`, `ShapeObservation.*`, `ShapeCalibration.*`.
  Added `.as_f64()` at print/format sites and wrapped raw float
  literals in `UnitInterval::clamped(...)`. Tutorial 12 gained
  `converge-pack` as a direct dep so the type is in scope.
- Workspace `[patch.crates-io]` extended to local sources for
  `organism-planning` / `-adversarial` / `-simulation` / `-learning`
  / `-notes` / `-intelligence` so the workspace picks up the
  in-flight 1.9.x changes (MembershipDegree, the bounded-numeric
  newtypes, the typed-ID newtypes, etc.) before they are
  republished. **Remove these patches once the 1.9 line stabilises
  on crates.io.**

## Reference port

`tutorials/06-reconciliation-loop/src/main.rs` (commit `85fbc79`)
remains the canonical example of all four patterns applied. New
adapter work that wants to follow the same shape should mirror it.
