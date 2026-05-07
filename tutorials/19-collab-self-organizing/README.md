# 19 — Collaboration: Self-Organising Swarm

The loosest collaboration shape. Open-call formation, loose
discipline, "figure it out" turn cadence. The team self-selects,
roles are fluid, and the only hard requirement is round synthesis.

## Prereq

[`18-collab-panel`](../18-collab-panel) — you've now seen the full
discipline gradient: huddle (strict), panel (curated), discussion
(moderated), and now self-organising (loose). Pick the shape that
matches the stakes, not your habit.

## What you'll learn

- `CollaborationCharter::self_organizing()` — the loose preset
- `TeamFormationMode::OpenCall` — anyone can join
- `CollaborationDiscipline::Loose` — formation mode not enforced
- `TurnCadence::FigureItOut` — agents decide order themselves
- `CollaborationRole::Generalist` — the role that contributes,
  votes, AND writes
- `ConsensusRule::AdvisoryOnly` — the done gate is a formality;
  the team decides when it's done
- Dynamic team scaling — start with one, grow as needed

## The setup

A research swarm. The team starts with a single seed agent; more
join as the work expands. Generalists can do anything; the done
gate always passes (advisory). Use this when the problem is open-
ended and the cost of a wrong call is low.

## End of the spine

You've walked the stack. From here:

- **Browse [`scenarios/`](../../scenarios)** for full end-to-end
  domain demos: expense approval, loan application, vendor
  selection, meeting scheduling.
- **Build your own** by copying the closest tutorial as a starter.

## Run it

```sh
cargo run -p example-collab-self-organizing
```
