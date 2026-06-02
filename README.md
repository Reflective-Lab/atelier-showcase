# atelier

[![CI](https://github.com/Reflective-Lab/atelier-showcase/actions/workflows/ci.yml/badge.svg)](https://github.com/Reflective-Lab/atelier-showcase/actions/workflows/ci.yml)
[![Coverage](https://github.com/Reflective-Lab/atelier-showcase/actions/workflows/coverage.yml/badge.svg)](https://github.com/Reflective-Lab/atelier-showcase/actions/workflows/coverage.yml)
[![Security](https://github.com/Reflective-Lab/atelier-showcase/actions/workflows/security.yml/badge.svg)](https://github.com/Reflective-Lab/atelier-showcase/actions/workflows/security.yml)
[![Stability](https://github.com/Reflective-Lab/atelier-showcase/actions/workflows/stability.yml/badge.svg)](https://github.com/Reflective-Lab/atelier-showcase/actions/workflows/stability.yml)
[![Crates.io](https://img.shields.io/crates/v/converge-atelier-domain.svg)](https://crates.io/crates/converge-atelier-domain)
[![docs.rs](https://docs.rs/converge-atelier-domain/badge.svg)](https://docs.rs/converge-atelier-domain)
[![dependency status](https://deps.rs/repo/github/Reflective-Lab/atelier-showcase/status.svg)](https://deps.rs/repo/github/Reflective-Lab/atelier-showcase)
![MSRV](https://img.shields.io/badge/MSRV-1.94.0-blue)
<img alt="gitleaks badge" src="https://img.shields.io/badge/protected%20by-gitleaks-blue">
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

The Converge **showcase** — a numbered tutorial spine plus a gallery of
end-to-end domain demos for builders new to Converge and the wider stack.

Where Converge says *what's possible*, atelier says *here's how it looks*.
Use it to learn the platform, seed a new engagement, or prove out an
architectural idea.

## Where to start

- **New to Converge?** Walk `tutorials/` in order, starting at
  [`tutorials/01-hello-convergence`](tutorials/01-hello-convergence). Each
  step builds on the last; the spine carries you from a minimal Converge
  program to organism-layer collaboration patterns.
- **Want a full domain demo?** Browse [`scenarios/`](scenarios) — pick by
  interest. No required order.
- **Domain expert (Truths / Gherkins)?** That track lives in
  [`truths/`](truths). Reserved slot, contributions welcome.

Cargo package for the core domain library: `converge-atelier-domain`.
Rust library name remains `atelier_domain`.

## Why this exists

> We are moving from writing explicit instructions upfront to designing
> systems that can turn intent into decisions at runtime — safely.

The old stack existed because machines needed unambiguous instructions, so
all ambiguity had to be resolved before execution. That constraint has
shifted: ambiguity can now be resolved at runtime, under guardrails. The
work is not to remove structure but to **relocate it** — from hardcoded
flows into contracts, constraints, evaluation loops, and orchestration.

That relocation is the reason Converge, the extensions, and this repo
exist. Every worked exemplar in `tutorials/` and `scenarios/` is a small
proof that the relocation works for a concrete outcome — expense
approval, vendor selection, loan underwriting, scheduling — without
giving up correctness or auditability.

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

- `crates/` — publishable libraries
  - `atelier-domain` — built-in domain packs (trust, money, delivery,
    data_metrics) and reference domain agents
  - `organism-domain` — organisational domain packs and blueprints
- `tutorials/` — the numbered learning spine. Read 01 → 19 to learn the
  stack. The 10/11 break is the point where the converge-only tutorials
  hand off to the organism layer.
- `scenarios/` — full end-to-end domain demos. Browse by interest:
  expense approval, loan application, vendor selection, meeting
  scheduling, Arbiter + Ferrox solver selection, and the high-risk
  Arbiter claim portfolio.
- `truths/` — reserved slot for the domain-expert track
  (axiom-truth / helms layer).

## Status

Extracted from `converge/crates/domain` and `converge/examples/` on 2026-05-05
as part of the v3.8 foundation cleanup (ADR-008). The workspace is versioned
from 1.0.0.

## Build

```sh
cargo check --workspace
```

In the root `~/dev/reflective` checkout, selected Converge, Organism, and
Prism crates are patched to local sources under `../stack/` so Atelier can
exercise unreleased foundation changes without forking the dependency graph.
Keep those path patches aligned with the root workspace layout; remove them
only when the corresponding published crate line contains the needed contract.

## License

MIT — see [LICENSE](LICENSE).
