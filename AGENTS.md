# Agents Entrypoint

This is the **Converge showcase**: a tutorial spine plus a scenario
gallery for builders new to Converge and the wider stack.

The repo wears two reader-facing hats and one library-facing hat:

- `tutorials/01-…` through `tutorials/19-…` — the numbered learning
  spine. A Rust dev who knows nothing about Converge walks 01 → 19 and
  ends understanding the stack. Tutorials 01–10 stay inside Converge
  primitives; from 11 onward they pull in the organism layer.
- `scenarios/` — full end-to-end domain demos. Browsable, no required
  order. Today: expense approval, loan application, vendor selection,
  meeting scheduling.
- `truths/` — reserved slot for the domain-expert track at the
  `axiom-truth` / `helms` layer (Truths and Gherkin specifications).
  Empty today; the directory holds the position so contributions land
  here rather than scattering across `tutorials/` or `scenarios/`.

The publishable libraries live in `crates/`: `atelier-domain` (built-in
domain packs and reference agents) and `organism-domain` (organisational
packs and blueprints). Everything in `tutorials/` and `scenarios/` is
`publish = false`.

The dependency arrow stays one-way: foundation contracts ← extensions ←
products ← showcase.

## Standards

This repo follows the **Extension Release Checklist**:

  https://github.com/Reflective-Lab/converge/blob/main/kb/Standards/Extension%20Release%20Checklist.md

Every release must clear all eight pillars (surface hygiene, compile gates,
release-grade gates, coverage floor, test layout, CI, provenance,
versioning) before tagging.

## Topology

- **Foundation:** `~/dev/reflective/stack/bedrock-platform/converge`
- **Sibling checkouts:** `~/dev/reflective/stack/mosaic-extensions/{atelier-showcase, arbiter-policy, embassy-ports, ferrox-solvers, manifold-adapters, mnemos-knowledge, prism-analytics}`
- **Organism platform:** `~/dev/reflective/stack/bedrock-platform/organism` (path-patched into this
  workspace via `[patch.crates-io]` while organism crates remain
  unreleased)
- **Templates:** `~/dev/reflective/templates/converge-extension` (this scaffold)

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
- Not a dumping ground for orphan example code. New examples must land
  in `tutorials/` (with a number that fits the reading order) or
  `scenarios/` (named by domain), and must compile against the current
  Converge version.
