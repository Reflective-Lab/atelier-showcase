---
source: mixed
---
# Changelog

All notable changes to `atelier` are recorded here.

## [Unreleased]

- Added `scenario-sec-edgar-live-filing`, a narrow `REAL LIVE` proof slice that
  fetches Apple Inc.'s 2025 Form 10-K from official SEC EDGAR through a
  Converge engine run: `SecEdgarRequest` is seeded into `ContextKey::Seeds`,
  Embassy's `SecFilingSuggestor<LiveSecEdgarProvider>` calls the live SEC
  provider, Converge promotes a typed `SecFilingPayload` under
  `ContextKey::Hypotheses`, and the scenario extracts Item 1A risk-factor
  headings, derives an Arbiter `ComplianceDocumentPayload` preserving SEC
  provenance fields, and lets `ComplianceGateSuggestor` emit a typed
  auto-clearance-blocking `ComplianceConstraintPayload` without deterministic
  test providers, recorded HTTP, canned fixtures, or fake policy gates.
  Findings resolved: Embassy now has a live `SecEdgarProvider` trait
  implementation instead of only live helper functions, the showcase now
  exercises the downstream Converge composition boundary instead of calling the
  provider directly from `main`, and the live filing fact now feeds a real
  downstream Arbiter decision while preserving source fact id, request hash, and
  provider. The simple review threshold is now a reusable
  `atelier_domain::sec_risk::SecRiskPolicyPack` with source-shape,
  source-vendor, section-size, and heading-count rules; when run with
  `--features with-solver`, the Arbiter block feeds a real Ferrox HiGHS MIP
  allocation that chooses minimum analyst-review lanes subject to heading
  coverage, review breadth, and senior-review constraints. The first recurring
  analysis slice is now also wired: the scenario fetches Apple's prior-year
  2024 Form 10-K live through Embassy, persists current and prior review
  profiles behind `converge-storage`'s in-memory `ObjectStore`, reloads the
  bounded profile history, and runs Prism `SimilarityPack` through Converge.
- Raised the atelier example bar to live-by-default:
  - External-provider and Mosaic source-observation scenarios must be
    `REAL LIVE`; real local solvers, policy engines, and product logic may be
    `LOCAL REAL`.
  - `CONTRACT-SHAPE`, `SIMULATED`, and fake-backed `MIXED` decision paths now
    belong in `arena-tests`, not in landed atelier showcase scenarios.
  - Removed the contract-shape `scenario-counterparty-kyc-convergence` from
    the atelier workspace because its Embassy leg used extension-provided
    deterministic `Stub*Provider` backends rather than live registry/sanctions
    calls. The pressure finding remains valid: Embassy lookup request payloads
    implemented `FactPayload` but not `PartialEq`, blocking direct downstream
    seeding through `ProposedFact::new`; fixed upstream in Embassy.
  - `tutorials/14-debate-loop` now uses Manifold provider APIs for live chat
    backend selection and exits honestly when no healthy backend is configured,
    instead of falling back to deterministic mock mode or naming a provider
    directly.
  - Added `just resource-declarations` and included it in `release-check` so
    fake-backed scenario declarations fail the release path.
- Added the `Example Resource Declarations` standard so runnable examples name
  their live/local boundary and any `Stub*`, `Mock*`, or `Fake*` backend on the
  decision path. Added declarations to representative live/local examples
  (`arbiter-ferrox-solver-gallery`, `high-risk-claim-portfolio`,
  `14-debate-loop`).
- Adopted the [Extension Release Checklist](https://github.com/Reflective-Lab/converge/blob/main/kb/Standards/Extension%20Release%20Checklist.md):
  - Wired `just security-audit`, `just coverage`, `just performance-profile`, `just soak`.
  - Added `.github/workflows/{ci,coverage,security,stability}.yml`.
  - Coverage floor 80% enforced in coverage workflow.

## [0.1.0] — YYYY-MM-DD

- Initial release.
