# 11 — Charter from Intent

> *Organism layer begins here.* The intent's properties determine the
> shape of collaboration — not the other way around. Same engine,
> three intents, three different charters, transparent rationale.

## Prereq

[`10-formation-compiler`](../10-formation-compiler) — you've seen
formations get compiled at the converge layer. This tutorial moves
*one layer up*: deciding what shape the collaboration should take in
the first place.

## What you'll learn

- `IntentPacket` — the organism unit of intent (deadline, authority,
  reversibility, constraints, forbidden actions, expiry action)
- `derive_charter(&intent, now)` — the function that maps intent
  properties to a `CollaborationCharter` (topology, discipline, turn
  cadence, consensus rule)
- Why low-stakes / irreversible / urgent intents produce different
  shapes — and why each derivation comes with rationale
- The vocabulary you'll see in 12–19: topology, charter, discipline,
  consensus, formation mode

## The setup

Three intents, same derivation engine:

1. **Low-stakes exploration** ("research market trends") — produces
   a loose, advisory shape.
2. **High-stakes irreversible acquisition** ("acquire Outpost24 for
   €200M") — produces a strict, multi-authority shape with formal
   consensus.
3. **Urgent time-pressured decision** — produces a fast cadence
   with escalation on expiry.

Watch how reversibility, authority, and constraints change the
charter without you writing any if-statements.

## Run it

```sh
cargo run -p example-charter-from-intent
```

## Next

→ [`12-shape-competition`](../12-shape-competition) — the charter
itself is a hypothesis. Multiple candidate shapes compete; the
winning one informs priors for the next derivation.
