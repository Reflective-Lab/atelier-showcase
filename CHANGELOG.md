# Changelog

All notable changes to atelier will be documented in this file.

## [Unreleased]

## [1.0.0] - 2026-05-13

### Added

Initial release. Extracted from `converge/crates/domain` and
`converge/examples/` per [ADR-008](https://github.com/Reflective-Lab/converge/blob/main/kb/Architecture/ADRs/ADR-008-extension-crate-boundaries.md).

- `atelier-domain` (formerly `converge-domain`): trust, money, delivery,
  data_metrics packs plus reference domain agents.
- 19-tutorial learning spine (`tutorials/01-…` through `tutorials/19-…`)
  plus a `scenarios/` gallery for full end-to-end domain demos.
- Cross-extension showcase scenarios composing solvers, Cedar policy
  gates, and analytics inside one Converge formation:
  - `scenarios/solver-policy-allocation` — focused triple
    (atelier-domain resource routing + arbiter + prism). Includes a
    workspace-runnable version-skew canary smoke test and an
    OR-tools-gated end-to-end test (`with-solver` feature).
  - `scenarios/arbiter-ferrox-solver-gallery` — broad gallery pairing
    every ferrox solver surface with arbiter gates; native
    OR-Tools/HiGHS surfaces behind the `native-solvers` feature.
- `ferrox` promoted to a workspace dependency.
- `just solver-check` recipe and a manual-trigger `solver-tests` CI job
  for the gated ferrox end-to-end test.

### Changed

- Cargo package renamed from `atelier-domain` to
  `converge-atelier-domain`; Rust library name remains `atelier_domain`.
- Workspace reorganised into a single mental model. `crates/` now holds
  only the publishable libraries (`atelier-domain`, `organism-domain`).
  Examples live in `tutorials/` (numbered 01–19 learning spine) and
  `scenarios/` (full end-to-end domain demos). A reserved `truths/`
  slot holds the future domain-expert track at the axiom-truth / helms
  layer.
- Three duplicate examples (`expense-approval`, `loan-application`,
  `vendor-selection`) collapsed to their more-developed copies.
- Five tutorials rehydrated against `converge-pack` 3.8.1's API
  (`ConsensusRule::passes` newtypes, `ContextFact` accessors, `Registry`
  helper rename).
- Security policy synced with prism-analytics: allow `NCSA` license
  (libfuzzer-sys transitive); ignore RUSTSEC-2025-0141 (bincode
  unmaintained) and RUSTSEC-2026-0002 (lru unsound `IterMut`).

### Verified

- First clean `just release-check` run on 2026-05-13:
  security-audit, coverage 83.3% (floor 80%), performance-profile,
  5-min soak, lint + `cargo test --workspace`.
