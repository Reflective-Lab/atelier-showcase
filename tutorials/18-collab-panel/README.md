# 18 — Collaboration: Panel

A curated expert panel. The lead frames the discussion, then the
rest follow in round-robin. Roles diverge: judges vote without
contributing content; report-writers contribute without voting.

## Prereq

[`17-collab-huddle`](../17-collab-huddle) — you've seen strict
discipline with uniform roles. Panel keeps the strictness but lets
roles do *different jobs*.

## What you'll learn

- `CollaborationCharter::panel()` — the formal-panel preset
- `TurnCadence::LeadThenRoundRobin` — the lead opens, the rest rotate
- `CollaborationRole::Judge` — votes on consensus, doesn't write
  content
- `CollaborationRole::ReportWriter` — writes the panel's report,
  doesn't vote
- `TeamFormationMode::Curated` — strict matching: an open-call
  formation is rejected
- `CollaborationRole::votes_on_done_gate()` and friends — the
  per-role permission API

## The setup

An investment-committee review. Lead (committee chair), three judges
(partners), two report writers, plus a domain advisor. The chair
opens with the deal frame; the partners rotate; judges vote on the
recommendation; the report writers compile the memo.

## Run it

```sh
cargo run -p example-collab-panel
```

## Next

→ [`19-collab-self-organizing`](../19-collab-self-organizing) — the
loosest shape, the swarm. The team self-selects, the order is
"figure it out", roles are fluid.
