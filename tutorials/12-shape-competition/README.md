# 12 — Shape Competition

The collaboration shape itself is a hypothesis. Multiple candidate
shapes compete for the same intent; observations score each;
priors get calibrated; the next derivation is informed by past
outcomes.

## Prereq

[`11-charter-from-intent`](../11-charter-from-intent) — you've seen
a charter *derived* from an intent. This tutorial questions whether
the first derivation was right.

## What you'll learn

- `generate_candidates(&intent, now, &priors)` — produce N candidate
  charters for the same intent
- `ShapeObservation`, `ShapeMetric`, `score_observation` — turn
  trial outcomes into comparable scores
- `select_winner` — pick the best shape under explicit metrics, not
  by hand
- `calibrate_shape` — feed the outcome back into priors so the
  *next* problem of this class starts smarter
- `classify_problem` — what makes two intents the same "class"

## The setup

An irreversible €100M acquisition. Three candidate shapes are
generated. Each is trial-run (simulated here). Observations score
hypothesis count, contradictions, cycles, budget burn, average
confidence. The winner is selected; priors are calibrated. Run the
example again — you'll see the priors have moved.

This is where the platform starts learning *about itself*.

## Run it

```sh
cargo run -p example-shape-competition
```

## Next

→ [`13-topology-transition`](../13-topology-transition) — mid-run
shape changes driven by convergence signals.
