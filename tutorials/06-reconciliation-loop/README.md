# 06 — Reconciliation Loop

Exact one-to-one matching across two noisy ledgers. A real use of
`converge-optimization`'s Hungarian assignment, framed by a
financial reconciliation task.

## Prereq

[`05-fixed-point-vs-budget`](../05-fixed-point-vs-budget) — you should
know the engine's two stop modes and have seen optimisation algorithms
slot into a Converge loop.

## What you'll learn

- The "data → cost surface → assignment → residue" pipeline as four
  cooperating suggestors
- `converge-optimization::assignment::solve` — Hungarian one-to-one
  matching with explicit unmatched slots
- How a residue summary makes "what still needs human review" first
  class
- Why an audited optimiser inside an agent loop beats hand-rolled
  matching heuristics

## The setup

Two ledgers (left and right) arrive as facts. A scorer agent builds
a candidate matrix with cost, amount delta, day delta, and reference
overlap. The Hungarian solver picks the optimal assignment under
configurable tolerances and an unmatched-slot penalty. A summary
agent explains the residue — what didn't match and why.

## Run it

```sh
cargo run -p example-reconciliation-loop
```

## Next

→ [`07-adaptive-gap-loop`](../07-adaptive-gap-loop) — the
emergent-length loop: depth determined by the data, not a
schedule.
