# Changelog

All notable changes to atelier will be documented in this file.

## [Unreleased]

### Changed

- Cargo package renamed from `atelier-domain` to `converge-atelier-domain`;
  Rust library name remains `atelier_domain`.
- Workspace reorganised into a single mental model. `crates/` now holds
  only the publishable libraries (`atelier-domain`, `organism-domain`).
  Examples live in `tutorials/` (numbered 01–19 learning spine) and
  `scenarios/` (full end-to-end domain demos). A reserved `truths/`
  slot holds the future domain-expert track at the axiom-truth /
  helms layer.
- Three duplicate examples (`expense-approval`, `loan-application`,
  `vendor-selection`) collapsed to their more-developed copies.
- Five tutorials rehydrated against `converge-pack` 3.8.1's API
  (`ConsensusRule::passes` newtypes, `ContextFact` accessors,
  `Registry` helper rename).

## [1.0.0] - 2026-05-05

### Added

Initial release. Extracted from `converge/crates/domain` and
`converge/examples/` per [ADR-008](https://github.com/Reflective-Lab/converge/blob/main/kb/Architecture/ADRs/ADR-008-extension-crate-boundaries.md).

- `atelier-domain` (formerly `converge-domain`): trust, money, delivery,
  data_metrics packs plus reference domain agents
- 13 example crates demonstrating convergence patterns

### Changed

- Crate `converge-domain` renamed to `atelier-domain`
- All example crates relocated from `converge/examples/` (later
  reorganised into `tutorials/` and `scenarios/` — see Unreleased).
