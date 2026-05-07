# 14 — Debate Loop

Real LLM-backed planning with adversarial review. Planner proposes
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
- How a real Anthropic API call slots into a Suggestor (synchronous
  blocking client; the engine awaits the future)

## Requires

`ANTHROPIC_API_KEY` in the environment for real LLM behaviour. Without
it, the example runs in **mock mode** — every call returns a
deterministic stub so the loop still completes and you can see the
shape.

## The setup

`PlanningSuggestor` proposes an initial plan, then revises after
challenges. `SkepticSuggestor` reviews proposals and emits challenges
into `ContextKey::Evaluations`. The engine sequences them via
context dependencies. Convergence is reached when challenges are
resolved or the loop reaches a final-review state.

## Run it

```sh
# real LLM
ANTHROPIC_API_KEY=sk-ant-... cargo run -p example-debate-loop

# mock mode
cargo run -p example-debate-loop
```

## Next

→ [`15-resolution-showcase`](../15-resolution-showcase) — eight
matching dimensions for picking the right packs and capabilities
for an intent.
