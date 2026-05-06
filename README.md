# atelier

Worked exemplars for the Converge platform — domain packs and runnable examples
that demonstrate what the platform makes possible.

`atelier` is a **showcase** repo, not an extension. It depends on Converge
contracts (and on extension repos like `mnemos`, `prism`, `manifold` where
relevant) to demonstrate end-to-end patterns. Use it to learn the platform,
seed a new engagement, or prove out an architectural idea.

This is the workshop. Where Converge says *what's possible*, atelier says
*here's how it looks*.

## Why this exists

> We are moving from writing explicit instructions upfront to designing
> systems that can turn intent into decisions at runtime — safely.

The old stack existed because machines needed unambiguous instructions, so
all ambiguity had to be resolved before execution. That constraint has
shifted: ambiguity can now be resolved at runtime, under guardrails. The
work is not to remove structure but to **relocate it** — from hardcoded
flows into contracts, constraints, evaluation loops, and orchestration.

That relocation is the reason Converge, the extensions, and this repo
exist. Every worked exemplar in `crates/example-*` is a small proof that
the relocation works for a concrete outcome — expense approval, vendor
selection, loan underwriting, scheduling — without giving up
correctness or auditability.

It also reframes what we are selling and how we organize to build it.
SaaS shifts from *tools that wrap workflows* to *systems that deliver
outcomes under constraints*; engineering shifts from feature teams around
UIs to substrate, domain, and adaptive-surface layers with different
change rates and different ownership.

For the full framing:

- [`kb/Philosophy/From Instructions to Intent`](kb/Philosophy/From%20Instructions%20to%20Intent.md)
  — the higher-order frame
- [`kb/Philosophy/Systems of Outcome`](kb/Philosophy/Systems%20of%20Outcome.md)
  — implications for SaaS, architecture, and ownership

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

Converge platform crates resolve from crates.io. Do not add local `[patch.crates-io]` overrides unless a task explicitly requires testing unpublished foundation changes.

## License

MIT — see [LICENSE](LICENSE).
