# 13 — Topology Transition

The shape doesn't have to stay fixed for a whole run. As convergence
signals evolve — hypotheses stabilising, contradictions appearing,
budget burning — the topology can transition mid-run. Swarm to
huddle to panel to synthesis, when the data warrants it.

## Prereq

[`12-shape-competition`](../12-shape-competition) — you've seen
shapes compete. This tutorial shows shapes *handing off* during a
single run.

## What you'll learn

- `ConvergenceSignals` — the runtime state that triggers transitions
  (cycle count, hypothesis count, stable hypotheses, contradictions,
  failed votes, budget remaining, stable cycles)
- `default_transition_rules()` — the catalog of "from-topology →
  to-topology" rules with rationales
- `evaluate_transitions(&signals, &rules)` — picks the right rule
  for the current moment
- Why transitions are signal-driven, not schedule-driven — you
  can't time a swarm-to-huddle in advance

## The setup

The example simulates 8 cycles of evolving signals. Early cycles
look like a swarm that's discovering. Then evidence clusters trigger
**swarm → huddle**. As contradictions appear, **huddle → panel**.
When confidence stabilises, **panel → synthesis**. Each transition
prints the matching rule and rationale.

## Run it

```sh
cargo run -p example-topology-transition
```

## Next

→ [`14-debate-loop`](../14-debate-loop) — concrete adversarial
collaboration with a real LLM in the loop.
