---
source: mixed
---
# Milestones

> See `~/dev/reflective/stack/bedrock-platform/EPIC.md` for the coarse-grained outcomes these milestones advance.

## Current: v1.0.0 — Converge 3.8.1 Showcase Baseline

**Target:** 2026-05 | **Tracks:** Converge 3.8.1

- [x] Keep workspace package version at `1.0.0`.
- [x] Keep Converge dependencies on the `3.8.1` contract baseline.
- [x] Adopt Extension Release Checklist (security-audit, coverage, performance-profile, soak)
- [x] First clean `just release-check` run (2026-05-13, coverage 83.3%)
- [x] Tag v1.0.0 (2026-05-13)
- [x] Add high-risk Arbiter claim portfolio Truth and showcase scenario.

## Next: v1.1.0 — Combinatory Showcase, Cross-Module Pressure Tests

**Target:** 2026-Q3 | **Tracks:** Converge 3.9.x | **Advances:** E1 (Converge publishable), E9 (shared fuzzy substrate)

### Why this milestone exists

v1.0.0 leans heavily on Arbiter and Ferrox. Embassy ports, Mnemos memory, Prism analytic packs + fuzzy inference, Manifold provider swap, and Crucible's training → registry → deployment loop are under-exercised by the showcase. Downstream apps consistently underestimate what's already callable; see `~/dev/reflective/stack/mosaic-extensions/kb/Capability Matrix.md` for the full reach.

atelier's job is to make that reach visible *with specificity* — every scenario must name the exact Mosaic functions it pulls and demonstrate why a generic substitute (one LLM call, a hand-rolled `if`-tree, a single solver) cannot give the same guarantee. A scenario that could be replaced by "ask GPT-4" without losing assurance does not belong in v1.1.0.

This milestone is also a pressure test. Combinatory scenarios surface gaps in real cross-module wiring — missing types, awkward provenance handoffs, contracts that don't compose. Findings feed back into `mosaic-extensions` as issues or same-session fixes (per the always-at-the-edge policy in `~/.claude/projects/.../memory/`).

### Acceptance criteria for any scenario added under v1.1.0

- [ ] Touches **three or more** Mosaic modules wired through Converge contracts, not bespoke glue.
- [ ] Domain-specific enough that the "why this matters" passes atelier's specificity bar (atelier-showcase is one of the few places allowed to speak concretely about domains).
- [ ] Declares its **pressure-test target** up front in the scenario README: the boundary, missing type, or wiring gap it expects to surface.
- [ ] References the Capability Matrix by linking each pulled function: `[matrix](mosaic-extensions/kb/Capability Matrix.md#<module>)`.
- [ ] Produces a finding entry in `kb/History/CHANGELOG.md` naming what broke during wiring, what was fixed, and what stayed as a documented gap with an issue link.
- [ ] An equivalent "generic substitute" attempt is documented (one LLM, one solver, no policy) and shown to fail the assurance bar — otherwise the combinatory cost isn't justified.

### Proposed scenarios

- [ ] **counterparty-kyc-convergence** — Embassy (`linkedin`, `gleif`, `bolagsverket`, `ofac-sls`, `eu-sanctions`) → Arbiter `ComplianceGateSuggestor` → Soter `Cvc5FfiBackend` → Mnemos `agentic::causal` + `agentic::temporal`. *Pressure-tests:* cross-port observation aggregation under one `CallContext`, whether `SanctionsHit` (Exact/Fuzzy/Alias + confidence) propagates as typed input to Cedar policies, whether causal memory carries provenance back to the originating port. *Generic substitute fails because:* a single LLM cannot produce signed, auditable sanctions evidence and cannot be proved free of bypass paths.
- [ ] **drift-triggered-retrain-loop** — Crucible `MonitoringAgent` → Mnemos `agentic::temporal` recall of historical drift → Arbiter `BudgetGateSuggestor` + `ApprovalGateSuggestor` → Crucible `ModelRegistryAgent` + `DeploymentAgent`. *Pressure-tests:* the closed-loop "experience → drift → retrain → deploy" story end-to-end; forces the drift signal into a Converge fact shape; surfaces whether registry promotion authority crosses the Converge boundary cleanly. Extends `loan-application`.
- [ ] **policy-constrained-allocation** — Prism `RankingPack` → Embassy (`sam-gov`, `ofac-sls`, `commerce-csl`) → Ferrox `HighsMipSuggestor` → Arbiter `ComplianceGateSuggestor` → Soter `CedarAnalysisSuggestor`. *Pressure-tests:* analytic-score → solver-objective coupling (does `UnitFraction` flow into HiGHS coefficients without a homemade conversion?); whether Soter can prove "no sanctioned counterparty is allocatable under any feasible input." Extends `vendor-selection` and `mip-facility-location`.
- [ ] **fuzzy-gated-routing** — Prism `fuzzy::Mamdani` (urgency / time-pressure rules) → Ferrox `CpSatVrptwSuggestor` → Manifold `llm` (operator-readable explanation, **provider swapped mid-run** to validate uniformity). *Pressure-tests:* typed `MembershipDegree` → solver-weight conversion; Manifold provider-shape uniformity across at least three of the seven LLM backends; whether `prism::fuzzy` outputs are wireable into Ferrox without an adapter crate.
- [ ] **cross-llm-adjudication** — Manifold (three `llm` backends in parallel) → Mnemos `agentic::reflexion` (per-backend track record) → Arbiter `ApprovalGateSuggestor` quorum decision. *Pressure-tests:* whether reflexion memory shape supports per-backend track records; whether the approval gate composes over LLM outputs; whether `retry_with_backoff` behaves identically across providers under induced failure.
- [ ] **public-procurement-opportunity** — Embassy `ted` + `usaspending` → Prism `ClassificationPack` + `RankingPack` → Mnemos temporal recall of similar past wins → Ferrox `CpSatFormationSuggestor` (which agents to assemble for the bid). *Pressure-tests:* Embassy → Prism analytic-pack chain (does the observation envelope feed Polars cleanly?); whether recall can shift formation selection through the existing capability descriptors.
- [ ] **ip-counterparty-scoring** *(stretch — forces skeleton growth)* — Embassy skeletons (`uspto`, `epo`, `openalex`, `arxiv`) → Prism `SimilarityPack` + `RankingPack` → Soter SMT invariant "score ≥ X requires legal sign-off." *Pressure-tests:* the P1 skeleton ports — pulling on them must surface the missing entity shapes and grow them through real use, not paper over them. This is the scenario most likely to fail on first attempt; that failure is the point.

### Definition of done for v1.1.0

- [ ] At least **four of the seven** proposed scenarios land with `just release-check` clean and coverage at or above v1.0.0's 83.3% floor.
- [ ] Each landed scenario carries a Capability-Matrix back-link in its `README.md` naming which functions it pulls.
- [ ] Each landed scenario carries a one-paragraph "Why generic substitutes fail" section.
- [ ] Findings rolled up into `kb/History/CHANGELOG.md` under a v1.1.0 heading, and each gap either fixed in the relevant `mosaic-extensions` repo in the same session (always-at-the-edge policy) or filed as a tracked issue with a link.
- [ ] `kb/Architecture/Algorithmic Backbone.md` extended to cover algorithm families pulled in by v1.1.0: Prism fuzzy inference (Mamdani / Sugeno / Tsukamoto), Mnemos vector recall + agentic-memory shapes, Manifold provider abstraction as an *anti*-algorithm (the value is uniformity, not a new complexity class).
- [ ] At least one finding upstreamed to `mosaic-extensions/kb/Capability Matrix.md` — either a corrected tagline, a clarified boundary, or a newly-pulled function being promoted from skeleton to live.
