---
tags: [quality, navigator, atelier]
source: human + LLM
---

# Quality Navigator — by audience

Reflective is a multi-repo, AI-augmented software factory. The quality
surface is large; this page routes you by what you're trying to do.

## New contributor — first hour

Read in this order. Each item is 10–20 minutes.

1. [`../README.md`](../README.md) — the showcase itself: what atelier
   is, what scenarios demonstrate.
2. [`../../README.md`](../../README.md) — the workspace's coordination
   layer. Maps which repo owns what.
3. [`README.md`](README.md) — this vault's purpose.
4. [`dimensions/hermeticity.md`](dimensions/hermeticity.md) and
   [`dimensions/semver-integrity.md`](dimensions/semver-integrity.md)
   — the two dimensions that bit the hardest in the most recent
   release. Reading them tells you what *not* to do.
5. One incident page that interests you — e.g.
   [`incidents/QF-2026-06-02-05.md`](incidents/QF-2026-06-02-05.md)
   (unit tests making real API calls).

You should now know:
- where the codebase keeps its standards (here + [`KB/05-engineering/standards/`](../../KB/05-engineering/standards/))
- where the codebase keeps its scars ([`QUALITY_BACKLOG.md`](../../QUALITY_BACKLOG.md))
- what your first PR should not do.

## Release captain — pre-flight

Before you propose a release of any train-member (converge, axiom,
organism, helms, mosaic-*, atelier, arena, runway, commerce):

1. **Run the quality scoreboard.** From the repo root:
   ```text
   just status              # see the train layout
   cd arena-tests && just report
   ```
   Aggregate verdict must be `Pass` or `Warn`. `Fail` blocks the
   release. `Skip` is informational (dimension not yet implemented).

2. **Run preflight on the candidate.** From the repo root:
   ```text
   just release-preflight <name>
   ```
   You'll get a per-project readiness report (branch state, version,
   credentials, downstream consumer scan).

3. **Verify the [`properties/`](properties/) load-bearing for this
   crate are green.** At minimum:
   - [`RP-SEMVER-GATED`](properties/RP-SEMVER-GATED.md) — your bump
     matches your public-API diff.
   - [`RP-LAYERING`](properties/RP-LAYERING.md) — no publishable crate
     depends on an UNLICENSED one.
   - [`RP-SNAPSHOT-PORTABLE`](properties/RP-SNAPSHOT-PORTABLE.md) —
     no fixture carries your machine's path.
   - [`RP-CRATE-SIZE-BUDGET`](properties/RP-CRATE-SIZE-BUDGET.md) —
     `cargo package` stays under 10 MiB.

4. **Eyeball [`QUALITY_BACKLOG.md`](../../QUALITY_BACKLOG.md) for
   open Bucket-A items.** If any blocks your release, decide:
   address, defer with [`Accepted Risk`](../../QUALITY_BACKLOG.md#lifecycle),
   or downgrade.

5. **Ship the release.** The actual sequence (build → test → commit
   → push → bump → tag → publish → gh release → update downstreams)
   is documented in [`migrations/release-train.md`](migrations/release-train.md)
   once that page lands.

6. **Update the backlog.** If anything went sideways, file a `QF-*`
   entry. Future-you reads it next release.

## API owner — touching a load-bearing trait

You're about to modify `Suggestor`, `Pack`, `Provenance`,
`ContextState`, `ProposedFact`, or any other type exported from
`converge-pack` / `converge-kernel` / `converge-model`.

1. Read [`incidents/QF-2026-06-02-04.md`](incidents/QF-2026-06-02-04.md)
   — the `Suggestor::provenance() -> Provenance` change that rode
   patch-version coattails into production and cascaded through 60+
   downstream crates. Do not be the next entry.

2. **Classify your change before you commit:**
   - **Breaking** — any change that requires downstream code edits.
     Major bump. Plan the cascade.
   - **Additive** — new method, new variant, new public type. Minor
     bump. Should compose without breaking.
   - **Patch** — internal only, no public-API delta.

3. **Run `cargo public-api diff` against the last released tag.**
   The diff should classify the same way you did. If it doesn't,
   one of you is wrong.

4. **List your downstream consumers.** Use
   [`just release-preflight <name>`](../../Justfile) — it scans for
   path-deps and version-deps across the workspace.

5. **Coordinate the cascade.** A breaking change in converge means
   every downstream needs a release immediately after. Plan that
   train before you merge the breaking commit, not after.

## Anyone — when a quality gate fails

The arena scoreboard just said `Fail` for some dimension. Now what?

1. Find the dimension's page in [`dimensions/`](dimensions/). It
   explains the verdict model and the implementation roadmap.

2. Find the matching property in [`properties/`](properties/). It
   explains *why* the property exists.

3. If there's a migration guide in [`migrations/`](migrations/) for
   this kind of violation, follow it.

4. Open a `QF-*` entry in [`QUALITY_BACKLOG.md`](../../QUALITY_BACKLOG.md)
   citing the dimension, the property, and the concrete evidence
   (file path, line, scoreboard line).

5. If you're an AI agent: respect
   [`RP-AI-SHORTCUT-DECLARED`](properties/RP-AI-SHORTCUT-DECLARED.md).
   If your fix involves changing a test, you owe the human a
   classification: "this test was wrong" vs. "this contract changed"
   vs. "this is a workaround pending design call."
