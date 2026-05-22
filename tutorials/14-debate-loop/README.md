# 14 — Debate Loop

Real Manifold-backed planning with adversarial review. Planner proposes
a plan; Skeptic challenges it; Planner revises. The debate **is**
the convergence loop — no special debate machinery, just two
suggestors reading and writing shared context until fixed point.

## Prereq

[`13-topology-transition`](../13-topology-transition) — you've seen
the shape vocabulary. This tutorial uses the simplest of all shapes
(two adversarial agents) but with real LLM calls behind them.

## What you'll learn

- That an "adversarial debate" is just two suggestors with
  complementary `accepts()` predicates — no debate framework needed
- `Severity` and `SkepticismKind` from `organism_pack` — the
  vocabulary the Skeptic uses to push back
- `CONFIDENCE_STEP_MAJOR` / `CONFIDENCE_STEP_MEDIUM` — calibrated
  confidence updates after a successful challenge
- How a Manifold-selected live chat backend slots into a Suggestor;
  atelier declares the need, Manifold chooses the provider

## Requires

At least one live chat provider credential supported by Manifold. Atelier
examples run real provider paths by default; without a healthy Manifold backend
this tutorial exits honestly instead of falling back to a deterministic mock.
Use `CONVERGE_LLM_PROFILE` to select criteria and `CONVERGE_LLM_PROVIDER` only
when the operator wants to pin a provider. Mocked debate fixtures belong in
`arena-tests`.

## The setup

`PlanningSuggestor` proposes an initial plan, then revises after
challenges. `SkepticSuggestor` reviews proposals and emits challenges
into `ContextKey::Evaluations`. The engine sequences them via
context dependencies. Convergence is reached when challenges are
resolved or the loop reaches a final-review state.

## Run it

```sh
cargo run -p example-debate-loop
```

## Resource Declaration

**Trust label:** `REAL LIVE`.

- Live external resources: **yes**. The tutorial calls the live chat provider
  selected by Manifold.
- Mosaic extensions: none are mocked or exercised here; this tutorial is on the
  organism/converge provider path rather than a Mosaic extension path.
- Backend mode: Manifold-selected live chat backend only. Missing or unhealthy
  provider credentials cause an honest non-zero exit.
- Credentials / feature flags: any live provider credential supported by
  Manifold; optional `CONVERGE_LLM_PROFILE` / `CONVERGE_LLM_PROVIDER`.
- Trust boundary: trust this as provider-neutral live LLM wiring through
  Manifold and a real adversarial convergence loop. It is not a Mosaic
  extension integration.

## Next

→ [`15-resolution-showcase`](../15-resolution-showcase) — eight
matching dimensions for picking the right packs and capabilities
for an intent.
