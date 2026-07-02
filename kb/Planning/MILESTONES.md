---
source: mixed
---
# Milestones

> See `~/dev/reflective/bedrock-platform/EPIC.md` for the coarse-grained outcomes these milestones advance.

## Released: v1.0.0 — Converge 3.8.1 Showcase Baseline

**Target:** 2026-05 | **Tracks:** Converge 3.8.1

- [x] Keep workspace package version at `1.0.0`.
- [x] Keep Converge dependencies on the `3.8.1` contract baseline.
- [x] Adopt Extension Release Checklist (security-audit, coverage, performance-profile, soak)
- [x] First clean `just release-check` run (2026-05-13, coverage 83.3%)
- [x] Tag v1.0.0 (2026-05-13)
- [x] Add high-risk Arbiter claim portfolio Truth and showcase scenario.

## Current: v1.1.0 — Combinatory Showcase, Cross-Module Pressure Tests

**Target:** 2026-Q3 | **Tracks:** Converge 3.9.x | **Advances:** E1 (Converge publishable), E9 (shared fuzzy substrate)

### Why this milestone exists

v1.0.0 leans heavily on Arbiter and Ferrox. Embassy ports, Mnemos memory, Prism analytic packs + fuzzy inference, Manifold provider swap, and Crucible's training → registry → deployment loop are under-exercised by the showcase. Downstream apps consistently underestimate what's already callable; see `~/dev/reflective/mosaic-extensions/kb/Capability Matrix.md` for the full reach.

atelier's job is to make that reach visible *with specificity* — every scenario must name the exact Mosaic functions it pulls and demonstrate why a generic substitute (one LLM call, a hand-rolled `if`-tree, a single solver) cannot give the same guarantee. A scenario that could be replaced by "ask a chat model" without losing assurance does not belong in v1.1.0.

The showcase default is live, not comfortingly mocked. External-provider and
Mosaic source-observation scenarios must call real providers or stay unlanded.
Contract-shape composites are still useful, but their home is `arena-tests`
until the live upstream exists.

This milestone is also a pressure test. Combinatory scenarios surface gaps in real cross-module wiring — missing types, awkward provenance handoffs, contracts that don't compose. Findings feed back into `mosaic-extensions` as issues or same-session fixes (per the always-at-the-edge policy in `~/.claude/projects/.../memory/`).

### Ambition gate: no innovation theatre

v1.1.0 should not add "AI demo" surface area. It should add evidence that the stack can carry outcome-grade work across capability families. A proposed scenario must clear this gate before it gets implementation time:

- Names the **customer outcome** and the risk being reduced, not just the modules being exercised.
- Pulls at least one under-exercised Mosaic family in the current showcase: Embassy, Mnemos, Prism, Manifold, Crucible, or Soter.
- Moves typed facts across module boundaries with provenance intact. Passing JSON blobs, copied scalar values, or prompt text between steps does not count.
- Contains a falsifiable baseline: one LLM call, one solver, or one policy gate must be shown to miss a requirement the combined system satisfies.
- Produces a reusable pressure finding: a type gap, policy gap, provenance gap, feature-flag gap, or docs correction that can be upstreamed to `mosaic-extensions`.
- Has a measurable assurance claim: coverage of a bounded input space, solver witness/UNSAT result, sanctions/provenance evidence, drift threshold, replay hash, cost cap, or evaluation metric.

Theatre smell checklist:

- Three modules appear in `Cargo.toml`, but only one actually decides anything.
- The LLM writes a plausible explanation after the real decision has already been made elsewhere.
- A solver is invoked without a binding objective or invariant that changes the outcome.
- A policy gate repeats a decision already encoded in local `if` statements.
- The README says "end-to-end" but no boundary break, missing type, or upstream issue emerges.

### Acceptance criteria for any scenario added under v1.1.0
**Epic:** E7

- [ ] Touches **three or more** Mosaic modules wired through Converge contracts, not bespoke glue.
- [ ] Domain-specific enough that the "why this matters" passes atelier's specificity bar (atelier-showcase is one of the few places allowed to speak concretely about domains).
- [ ] Declares its **pressure-test target** up front in the scenario README: the boundary, missing type, or wiring gap it expects to surface.
- [ ] Carries a **Resource Declaration** up front in the scenario README and, for verbose demos, in runtime output. New atelier scenarios must be `REAL LIVE` for external-provider/Mosaic source-observation paths or `LOCAL REAL` for real local solvers, policy engines, and product logic. `CONTRACT-SHAPE`, `SIMULATED`, or fake-backed `MIXED` decision paths belong in `arena-tests`, not in a landed atelier scenario.
- [ ] References the Capability Matrix by linking each pulled function with a valid path from the scenario README. For current scenario READMEs, use `[matrix](../../../stack/mosaic-extensions/kb/Capability%20Matrix.md#<module-anchor>)`.
- [ ] Produces a finding entry in `kb/History/CHANGELOG.md` naming what broke during wiring, what was fixed, and what stayed as a documented gap with an issue link.
- [ ] An equivalent "generic substitute" attempt is documented (one LLM, one solver, no policy) and shown to fail the assurance bar — otherwise the combinatory cost isn't justified.

### Execution order

Build the first four v1.1.0 scenarios in the order that maximizes under-exercised capability pull and minimizes theatre risk:

1. **counterparty-kyc-convergence** — strongest first move only after Embassy can run live. It should force real Embassy evidence ports, Arbiter policy, Soter searched evidence, and Mnemos provenance memory into one buyer-legible outcome: "do not onboard a prohibited counterparty." Until the Embassy providers are live, keep contract-shape coverage in `arena-tests`.
2. **drift-triggered-retrain-loop** — proves the outcome-system loop from live monitoring to retrain decision to registry promotion. This is the clearest bridge from showcase examples to production operations.
3. **policy-constrained-allocation** — turns vendor selection from a scoring demo into a constrained decision with proof pressure. It also extends existing `vendor-selection`, `mip-facility-location`, and `solver-policy-allocation` work instead of creating an isolated example.
4. **public-procurement-opportunity** — pulls Embassy, Prism, Mnemos, and Ferrox into an opportunity-discovery workflow where recall and formation selection should materially change the answer.

Defer **cross-llm-adjudication** until the first four land. It is useful, but it has the highest risk of reading like model-vendor choreography unless the reflexion memory and approval quorum materially change a regulated decision. Keep **ip-counterparty-scoring** as the stretch failure case: only start it when the team is ready to grow skeleton Embassy ports through real missing entity shapes.

### Proposed scenarios
**Epic:** E7

- [x] **sec-edgar-live-filing** — landed 2026-05-22 as the first narrow
  `REAL LIVE` Mosaic source-observation proof slice. It fetches Apple Inc.'s
  2025 Form 10-K from official SEC EDGAR by seeding a typed
  `SecEdgarRequest` into Converge, running Embassy
  `SecFilingSuggestor<LiveSecEdgarProvider>`, reading the promoted
  `SecFilingPayload`, and extracting Item 1A risk-factor headings. This is not
  one of the full three-module combinatory scenarios; it is the live-resource
  anchor proving atelier can show a human-verifiable external call through
  Converge without deterministic test providers, recorded fixtures, or fake
  provider output. *Pressure findings resolved:* Embassy now has
  provider-shaped live SEC access, and the showcase now uses downstream
  Converge composition rather than a direct provider call. The first decision
  composition step is also in place: the scenario derives an Arbiter
  `ComplianceDocumentPayload` from the live `SecFilingPayload`, preserves
  source fact id, request hash, provider, CIK, accession, and source URL, and
  lets `ComplianceGateSuggestor` emit a typed constraint blocking
  auto-clearance when the risk-factor heading count exceeds the configured
  review threshold. The simple rule is now a reusable
  `atelier_domain::sec_risk::SecRiskPolicyPack` with source-shape,
  source-vendor, section-size, and heading-count rules. With
  `--features with-solver`, the Arbiter block feeds a real Ferrox HiGHS MIP
  allocation that chooses minimum analyst-review lanes subject to heading
  coverage, breadth, and senior-review constraints. The first memory-backed
  recurrence is now in place: the scenario fetches Apple's prior-year 2024
  10-K through the same live SEC provider, writes current and prior review
  profiles through `converge-storage`'s in-memory `ObjectStore`, reloads the
  bounded profile history, and runs Prism `SimilarityPack` through Converge.
  The next gap is durable Runtime Runway/GCS-backed recurrence and a broader historical
  corpus.
- [ ] **counterparty-kyc-convergence** — not landed in atelier until the Embassy leg is `REAL LIVE`. The first contract-shape slice proved the intended chain (Embassy `gleif`, `bolagsverket`, `ofac-sls`, `eu-sanctions` → Arbiter `ComplianceGateSuggestor` → Soter `SmtSuggestor` → Mnemos `agentic::causal` + `agentic::temporal`) and surfaced/fixed an upstream contract gap: Embassy lookup request payloads implemented `FactPayload` but not `PartialEq`, blocking direct downstream seeding through `ProposedFact::new`; request payloads now derive `PartialEq` upstream. A second contract-shape slice (2026-05-22) lives at `~/dev/reflective/arena-tests/crates/counterparty-kyc-convergence`. It enforces REAL-by-default at the binary boundary: `cargo run` exits code 2 with a diagnostic naming the Embassy-stubs-only gap; `cargo run -- --mock-ok` runs end-to-end against `StubGleifProvider` + `StubOfacSlsProvider` and clearly labels every step as CONTRACT-SHAPE. *Next required fix:* implement or select live Embassy providers for the identity and sanctions sources, then move (or template a new) `REAL LIVE` scenario into atelier with a Resource Declaration.
- [ ] **drift-triggered-retrain-loop** — Crucible `MonitoringAgent` → Mnemos `agentic::temporal` recall of historical drift → Arbiter `BudgetGateSuggestor` + `ApprovalGateSuggestor` → Crucible `ModelRegistryAgent` + `DeploymentAgent`. *Pressure-tests:* the closed-loop "experience → drift → retrain → deploy" story end-to-end; forces the drift signal into a Converge fact shape; surfaces whether registry promotion authority crosses the Converge boundary cleanly. Extends `loan-application`.
- [ ] **policy-constrained-allocation** — Prism `RankingPack` → Embassy (`sam-gov`, `ofac-sls`, `commerce-csl`) → Ferrox `HighsMipSuggestor` → Arbiter `ComplianceGateSuggestor` → Soter `CedarAnalysisSuggestor`. *Pressure-tests:* analytic-score → solver-objective coupling (does `UnitFraction` flow into HiGHS coefficients without a homemade conversion?); whether Soter can prove "no sanctioned counterparty is allocatable under any feasible input." Extends `vendor-selection` and `mip-facility-location`.
- [ ] **fuzzy-gated-routing** — Prism `fuzzy::Mamdani` (urgency / time-pressure rules) → Ferrox `CpSatVrptwSuggestor` → Manifold `llm` (operator-readable explanation, **provider swapped mid-run** to validate uniformity). *Pressure-tests:* typed `MembershipDegree` → solver-weight conversion; Manifold provider-shape uniformity across at least three of the seven LLM backends; whether `prism::fuzzy` outputs are wireable into Ferrox without an adapter crate.
- [ ] **cross-llm-adjudication** — Manifold (three `llm` backends in parallel) → Mnemos `agentic::reflexion` (per-backend track record) → Arbiter `ApprovalGateSuggestor` quorum decision. *Pressure-tests:* whether reflexion memory shape supports per-backend track records; whether the approval gate composes over LLM outputs; whether `retry_with_backoff` behaves identically across providers under induced failure.
- [ ] **public-procurement-opportunity** — Embassy `ted` + `usaspending` → Prism `ClassificationPack` + `RankingPack` → Mnemos temporal recall of similar past wins → Ferrox `CpSatFormationSuggestor` (which agents to assemble for the bid). *Pressure-tests:* Embassy → Prism analytic-pack chain (does the observation envelope feed Polars cleanly?); whether recall can shift formation selection through the existing capability descriptors.
- [ ] **ip-counterparty-scoring** *(stretch — forces skeleton growth)* — Embassy skeletons (`uspto`, `epo`, `openalex`, `arxiv`) → Prism `SimilarityPack` + `RankingPack` → Soter SMT invariant "score ≥ X requires legal sign-off." *Pressure-tests:* the P1 skeleton ports — pulling on them must surface the missing entity shapes and grow them through real use, not paper over them. This is the scenario most likely to fail on first attempt; that failure is the point.

### Definition of done for v1.1.0
**Epic:** E7

- [ ] At least **four of the seven** proposed scenarios land with `just release-check` clean and coverage at or above v1.0.0's 83.3% floor.
- [ ] Each landed scenario carries a Capability-Matrix back-link in its `README.md` naming which functions it pulls.
- [ ] Each landed scenario carries a Resource Declaration in its `README.md`, and runtime output prints the declaration before results when a user could plausibly mistake the run for live integration. Any scenario that needs `Stub*`, `Mock*`, `Fake*`, recorded HTTP, or canned provider data moves to `arena-tests` or stays unlanded until live wiring exists.
- [ ] Each landed scenario carries a one-paragraph "Why generic substitutes fail" section.
- [ ] Findings rolled up into `kb/History/CHANGELOG.md` under a v1.1.0 heading, and each gap either fixed in the relevant `mosaic-extensions` repo in the same session (always-at-the-edge policy) or filed as a tracked issue with a link.
- [ ] `kb/Architecture/Algorithmic Backbone.md` extended to cover algorithm families pulled in by v1.1.0: Prism fuzzy inference (Mamdani / Sugeno / Tsukamoto), Mnemos vector recall + agentic-memory shapes, Manifold provider abstraction as an *anti*-algorithm (the value is uniformity, not a new complexity class).
- [ ] At least one finding upstreamed to `mosaic-extensions/kb/Capability Matrix.md` — either a corrected tagline, a clarified boundary, or a newly-pulled function being promoted from skeleton to live.
