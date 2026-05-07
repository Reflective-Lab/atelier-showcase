# 09 — Formation Mixed

A formation that mixes types of intelligence: an optimiser, a policy
gate, and an LLM-style reasoning agent — all converging in one
Engine run. Same contract, same governance.

## Prereq

[`08-live-formation`](../08-live-formation) — you've seen LLM-style
agents converge. This tutorial replaces some of them with audited
algorithms and policy.

## What you'll learn

- That "agent" is a contract, not an LLM — an optimiser pack and a
  policy gate satisfy `Suggestor` just as cleanly
- `BudgetAllocationPack` from `converge-optimization` wrapped with
  `PackSuggestor` — algorithms as plug-in agents
- `arbiter::PolicyGateSuggestor` — Cedar-style policy as a gate
- Dependency choreography: the LLM agent depends on `Constraints`
  (written by policy), not `Strategies`, so it only runs after the
  gate has had a chance to block

## The setup

`IntentSeeder` plants a budget-allocation problem. The optimisation
pack solves it. The policy gate validates the allocation against
spending rules. The LLM agent (stub) reasons about the result.
Same Engine, same context, same fixed-point guarantee.

## Run it

```sh
cargo run -p example-formation-mixed
```

## Next

→ [`10-formation-compiler`](../10-formation-compiler) — turn a
formation template + catalog into a compiled, executable plan.
This is the last tutorial inside the converge-only layer.
