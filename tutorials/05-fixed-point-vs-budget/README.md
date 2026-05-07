# 05 — Fixed Point vs Budget

Same loop, two outcomes. Either the context stabilises (fixed point)
or the budget runs out (budget exhausted). Knowing which is which is
table stakes for running Converge in production.

## Prereq

[`04-intent-codec-loop`](../04-intent-codec-loop) — you should be
comfortable with the engine running until *something* makes it stop.
This tutorial is about *what makes it stop*.

## What you'll learn

- `Budget` — the cycle/time bound on a run
- `ConvergeResult` vs `ConvergeError::BudgetExhausted` — the two
  return shapes you must handle
- `StopReason::Converged` as the success signal
- Bonus: a real use of `converge-optimization`'s Dijkstra over a
  discovered artefact graph (the frontier planner picks the nearest
  unexplored node)

## The setup

The tutorial runs the same suggestors twice. First with a generous
budget — converges, returns `Ok(ConvergeResult)`. Then with a tight
budget — exhausts, returns `Err(ConvergeError::BudgetExhausted)`.
Both are valid outcomes; only one is success.

## Run it

```sh
cargo run -p example-fixed-point-vs-budget
```

## Next

→ [`06-reconciliation-loop`](../06-reconciliation-loop) — a domain
loop that uses Hungarian assignment from `converge-optimization`.
