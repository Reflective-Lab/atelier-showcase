# Changelog

All notable changes to atelier will be documented in this file.

## [Unreleased]

## [1.0.0] - 2026-05-05

### Added

Initial release. Extracted from `converge/crates/domain` and
`converge/examples/` per [ADR-008](https://github.com/Reflective-Lab/converge/blob/main/kb/Architecture/ADRs/ADR-008-extension-crate-boundaries.md).

- `atelier-domain` (formerly `converge-domain`): trust, money, delivery,
  data_metrics packs plus reference domain agents
- 13 example crates demonstrating convergence patterns

### Changed

- Crate `converge-domain` renamed to `atelier-domain`
- All example crates relocated from `converge/examples/` to
  `atelier/crates/example-*`
