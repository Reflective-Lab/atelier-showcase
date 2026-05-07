# 16 — Collaboration: Discussion Group

The first of four collaboration-shape tutorials. A discussion group
sits between strict huddle and loose swarm: moderated discipline, a
moderator who speaks first, and **advisory-only** consensus — the
output is a recommendation, not a binding decision.

## Prereq

[`15-resolution-showcase`](../15-resolution-showcase) — you've seen
how intents get routed. The next four tutorials demonstrate the
shapes intents may *get routed to*.

## What you'll learn

- `CollaborationCharter::discussion_group()` — the canonical preset
- `TurnCadence::ModeratorThenRoundRobin` — moderator frames, then
  rotation
- `CollaborationDiscipline::Moderated` — flexible formation modes
  accepted (vs `Strict`)
- `ConsensusRule::AdvisoryOnly` — outputs a recommendation, doesn't
  block downstream
- The full `ConsensusRule` family (Majority, Supermajority,
  Unanimous, LeadDecides, AdvisoryOnly) compared side by side under
  the same hypothetical 5-voter pool

## The setup

A strategy brainstorm. The moderator opens, the team rotates, the
group recommends. The example then prints the pass/fail of each
consensus rule against vote counts 0/5 through 5/5 so you can see
how strict each rule actually is.

A small `passes()` helper wraps the converge-pack newtypes
(`VoteTally`, `EligibleVoters`) for readability — the API rejects
zero pools, so the helper bumps `total.max(1)` for the demo edge.

## Run it

```sh
cargo run -p example-collab-discussion
```

## Next

→ [`17-collab-huddle`](../17-collab-huddle) — the tightest shape:
strict, round-robin, mandatory dissent mapping, done-gate vote.
