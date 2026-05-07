# 17 — Collaboration: Huddle

The tightest collaboration shape. Strict discipline, explicit turns,
mandatory dissent mapping, and a done-gate vote before the team can
declare convergence. Use a huddle when the cost of a wrong call is
high.

## Prereq

[`16-collab-discussion`](../16-collab-discussion) — you've seen the
loosest binding shape. Huddle is the opposite end of the spectrum.

## What you'll learn

- `CollaborationCharter::huddle()` — the strict preset
- `CollaborationDiscipline::Strict` — formation mode is enforced;
  the wrong shape gets rejected at validation
- `TurnCadence::RoundRobin` — every member speaks, in order
- `ConsensusRule::Majority` for the done gate — voting is real, not
  advisory
- `CollaborationCharter::validate(&formation)` — what happens when
  a charter rejects a team (too few members; missing critic role)
- `ConsensusRule::Unanimous` as an override — same votes that
  passed under majority now block

## The setup

A due-diligence research team: lead, domain expert, market analyst,
red team, synthesiser. Round-robin debate, a done-gate vote, then
two intentional **validation failures** (too few members; missing
critic) so you see the charter actually rejecting bad teams.

## Run it

```sh
cargo run -p example-collab-huddle
```

## Next

→ [`18-collab-panel`](../18-collab-panel) — formal expert panel,
lead-then-round-robin, judges who vote without contributing,
report-writers who don't vote.
