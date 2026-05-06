# Agents Entrypoint

This is a Converge extension. It depends on stable Converge contracts and
ships an implementation downstream of the foundation.

## Standards

This repo follows the **Extension Release Checklist**:

  https://github.com/Reflective-Lab/converge/blob/main/kb/Standards/Extension%20Release%20Checklist.md

Every release must clear all eight pillars (surface hygiene, compile gates,
release-grade gates, coverage floor, test layout, CI, provenance,
versioning) before tagging.

## Topology

- **Foundation:** `~/dev/work/converge`
- **Sibling extensions:** `~/dev/extensions/{atelier, mnemos, prism, arbiter, embassy, ferrox, manifold}`
- **Templates:** `~/dev/templates/converge-extension` (this scaffold)

The dependency arrow is one-way: foundation contracts ← extensions ← products.

## The five-command release ritual

```bash
just security-audit
just coverage
PERF_BASELINE=v$(grep -m1 '^version' Cargo.toml | sed -E 's/.*"(.*)".*/\1/') just performance-profile
SOAK_DURATION_MIN=5 just soak
just lint && cargo test --workspace
```

Or `just release-check`. Archive the artefacts under `target/security/`,
`target/coverage/`, `target/criterion/`, `target/soak/`, and `kb/Baselines/`.

## Knowledge base

`kb/` mirrors the foundation structure:

- `kb/Home.md` — moc index
- `kb/INDEX.md` — entity catalog
- `kb/LOG.md` — mutation log (append on every kb/ change)
- `kb/Architecture/` — surface diagrams, ports, ADRs
- `kb/Building/` — getting-started, release commands
- `kb/History/CHANGELOG.md` — release notes
- `kb/Planning/MILESTONES.md` — scheduled delivery

Every kb/ page carries `source:` frontmatter (`human` / `llm` / `mixed`).

## What this repo is not

- Not a place for foundation contracts. Universal contracts live in
  Converge.
- Not a service shell. If you need a server or CLI, separate it from the
  reusable library and mark the shell `publish = false`.
- Not exempt from the checklist. The bar applies to small repos and large
  repos equally.
