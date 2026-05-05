## Summary

- Describe the new or updated exemplar.

## Checks

- [ ] `cargo check --workspace` passes
- [ ] The exemplar runs to completion (`cargo run -p example-...`)
- [ ] CHANGELOG.md updated under `[Unreleased]`
- [ ] README updated if a new exemplar was added

## Showcase Discipline

- [ ] No production secrets, credentials, or org-specific data
- [ ] No vendor-specific code (those belong in `manifold` not in atelier)
- [ ] Uses the Plug Boundary correctly (Suggestor layer for purpose, Backend
      layer for capability)
