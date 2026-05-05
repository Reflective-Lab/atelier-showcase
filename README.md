# atelier

Worked exemplars for the Converge platform — domain packs and runnable examples
that demonstrate what the platform makes possible.

`atelier` is a **showcase** repo, not an extension. It depends on Converge
contracts (and on extension repos like `mnemos`, `prism`, `manifold` where
relevant) to demonstrate end-to-end patterns. Use it to learn the platform,
seed a new engagement, or prove out an architectural idea.

This is the workshop. Where Converge says *what's possible*, atelier says
*here's how it looks*.

## Layout

- `crates/atelier-domain` — built-in domain packs (trust, money, delivery,
  data_metrics) and reference domain agents
- `crates/example-*` — runnable demonstrations:
  - `hello-convergence`, `custom-agent`, `custom-provider`
  - `meeting-scheduler`, `expense-approval`, `vendor-selection`,
    `loan-application`
  - `formation-mixed`, `live-formation`
  - `intent-codec-loop`, `adaptive-gap-loop`, `fixed-point-vs-budget`,
    `reconciliation-loop`

## Future contents

Future releases will gather worked exemplars from `organism` and `axiom`
alongside the converge-side material. Atelier is the cross-platform showcase
for the Reflective Labs ecosystem.

## Status

Extracted from `converge/crates/domain` and `converge/examples/` on 2026-05-05
as part of the v3.8 foundation cleanup (ADR-008). Pre-1.0 — no published
versions yet.

## Build

```sh
cargo check --workspace
```

While Converge platform crates are unreleased, this workspace patches them to
the local checkout via `[patch.crates-io]`. You need converge checked out at
`~/dev/work/converge`.

## License

MIT — see [LICENSE](LICENSE).
