# Contributing to atelier

atelier is the showcase repo for the Reflective Labs platform — a curated
collection of worked exemplars across Converge, Organism, and Axiom.

## What belongs here

- **Domain packs** that demonstrate platform capabilities through a real
  vertical (trust, money, delivery, etc.)
- **Runnable examples** that show end-to-end patterns developers can clone
  and adapt
- **Reference architectures** that combine multiple platform pieces

## What does NOT belong here

- Capability adapters (those go in `mnemos`, `prism`, `manifold`, `arbiter`)
- Production deployments (those are operator territory, not showcase)
- One-off experiments that don't generalize

## Development

```sh
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

While the Converge platform is unreleased, this workspace patches the
relevant crates to local checkouts via `[patch.crates-io]`. You need the
sibling repos checked out:

```
~/dev/
├── work/converge/
├── extensions/{mnemos,prism,manifold,arbiter}/
└── atelier/   <- you are here
```

## License

By contributing, you agree your contributions are licensed under MIT.
