# 07 — Adaptive Gap Loop

Open-ended convergence — the loop length is emergent, set by the
data not by a schedule. Survey what you find; if you find references
to artefacts you haven't surveyed yet, reopen the loop with new
requests; close only when every gap is covered.

## Prereq

[`06-reconciliation-loop`](../06-reconciliation-loop) — you should be
comfortable with multi-suggestor loops. This one adds **self-extending
work**: the loop creates its own next steps.

## What you'll learn

- The **gap-driven** pattern: an agent emits "open-gap" facts when
  references are unresolved; another agent picks them up
- Why the closure agent only fires when every gap is covered — the
  "every artefact has been surveyed" invariant
- How to implement open-ended exploration without giving up the
  fixed-point guarantee
- That a shallow graph settles in 2 cycles; a deeper one needs many

## The setup

A seed asks the engine to inspect one artefact. The survey suggestor
emits observations. The gap suggestor sees referenced-but-unsurveyed
artefacts and emits new survey requests. The closure suggestor only
fires when no open gaps remain.

This is the shape of any "explore until exhausted" pattern —
crawling, dependency walking, evidence gathering.

## Run it

```sh
cargo run -p example-adaptive-gap-loop
```

## Next

→ [`08-live-formation`](../08-live-formation) — multiple specialised
agents converging to a single decision under a real-shaped problem.
