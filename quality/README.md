---
tags: [quality, navigator, atelier]
audience: contributors, release captains, API owners
source: human + LLM
---

# Quality Navigator

This vault is the **human-facing surface** for understanding how the
Reflective workspace is getting better — and where it isn't. It pairs
with the machine-readable side: [`arena-tests/`](../../arena-tests/)
measures, this vault narrates.

## What lives here

| Path | Purpose |
|---|---|
| [`navigator.md`](navigator.md) | Audience-routed entry: where to start based on who you are and what you're doing. |
| [`dashboard.md`](dashboard.md) | Auto-generated. Current verdicts and trends per quality dimension, regenerated from `arena-tests/reports/history.jsonl`. **Do not edit by hand.** |
| [`properties/`](properties/) | One page per [Recurring System Property](../../QUALITY_BACKLOG.md#recurring-system-properties) (`RP-*`). What the property asserts, why it matters, how we enforce it, where it's broken today. |
| [`dimensions/`](dimensions/) | One page per quality dimension that `arena-tests` measures. The lens through which we keep a property honest. |
| [`incidents/`](incidents/) | One page per anchor incident from the [Quality Backlog](../../QUALITY_BACKLOG.md). What happened, what we paid, what we now check for. |
| [`migrations/`](migrations/) | Runnable, step-by-step guides for fixing real instances of property violations. The "how do I" companion to "why does this matter." |

## How to use it

- **New contributor?** → start at [`navigator.md`](navigator.md).
- **About to cut a release?** → [`navigator.md#release-captain`](navigator.md) lists every gate and the order they should run in.
- **Touching a load-bearing trait (`Suggestor`, `Pack`, `Provenance`, …)?** → check [`properties/RP-SEMVER-GATED.md`](properties/RP-SEMVER-GATED.md) before the PR.
- **Wondering what `arena report` is telling you?** → open the matching page in [`dimensions/`](dimensions/).
- **Just hit a quality gate failure?** → the migration guide in [`migrations/`](migrations/) walks the fix.

## How this stays current

- `arena-tests/` writes structured runs to `arena-tests/reports/history.jsonl`.
- `atelier-showcase/crates/quality-render/` is a small Rust binary that
  reads that file and regenerates [`dashboard.md`](dashboard.md). Run
  `just render-dashboard` after any new arena run, or wire it into CI.
- Everything else here is human-authored. New incidents earn a page
  when they're added to `QUALITY_BACKLOG.md`. New properties earn a
  page when they're promoted out of `Aspired`.

## Tone

These pages are written to **teach**, not just to certify. A page
that says "do X" is failing its job; a page that explains *why* X
matters, *when* it matters, and *what happens if you don't* is doing
its job. The vault improves the more it's used.
