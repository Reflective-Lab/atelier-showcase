# Reflective — System Component Review Brief

*Status: standing task template, v1, 2026-06-17. A reusable brief for a Cloud
Agent running inside a **single** Reflective component repo (e.g.
`runtime-runway`, `bedrock-platform`, `mosaic-extensions`, `commerce-rails`,
`lattice-mesh`, a Helm repo). It asks that component to review what the **New
Normal** worldview and the **Reflective positioning** (the governed commitment
substrate) mean for it, and to produce a structured readiness review.*

*Canonical source for the positioning is the reflective KB (`KB/the-new-normal.md`,
`KB/01-platform/`). Because a Cloud Agent usually mounts only one repo, a dated
snapshot of the relevant docs is copied into that repo under `docs/strategy/`.
**Snapshots are read-only copies — edit upstream in the KB, never the snapshot.**
This brief itself is copied in as `docs/strategy/00-REVIEW-TASK.md`.*

## What changed (the 60-second version)

Enough context to start; the authoritative text is in the snapshots listed below.

- **The operating default inverts.** Continuous action is becoming machine-driven
  and ambient; human judgment becomes a *sparse, authoritative governing core*.
  The design question shifts from "make people more productive at running the
  work" to "how does a small, intermittent human core keep continuous,
  machine-driven action true to what was decided."
- **Systems of record → systems of action → systems of outcome commitments.**
  The business object is no longer the record or the workflow step; it is the
  governed outcome commitment.
- **Category vs. substrate.** *Governed decision translation* is the
  market-facing category. Beneath it, Reflective is a **governed commitment
  substrate** with six universal requirements: (1) canonical commitment/mandate,
  (2) explicit authority model, (3) faithful projections, (4) receipts and
  provenance, (5) drift detection, (6) reopen / contest / revoke / re-ratify.
- **Three commitment shapes.** The convened burst is only one. Each adds its own
  machinery:
  - **Convened burst** (`quorum-sense`) — one owner, one room; anti-HiPPO,
    dissent capture, quorum, ratification; realtime *or* async.
  - **Multi-sovereign** (`tally-escrow`, `concord-supply`) — no single owner;
    parties across a trust boundary; selective disclosure, symmetric receipts,
    obligation views, dispute paths, arbiters.
  - **Standing governance of autonomy** (`triage-keeper`, `vigil-care`) — decide
    rarely, operate continuously; delegation scope, runtime telemetry, policy
    gates, intervention thresholds, kill/reopen.
  - Plus a *partial* fit: **truth-preserving projection** (`folio-editor`).
- **One loop, pluggable Decide.** Decide → Translate → Operate → Detect drift →
  Reopen. Only **Decide** changes by shape (convene / negotiate / set-mandate);
  the rest is shared substrate machinery.
- **Reliability is part of the thesis.** Core commitment/governance/runtime/
  provenance layers prefer typed, compiled, observable systems; abstraction is
  spent on business semantics, not on accidental runtime uncertainty.
- **Layer ownership.** Apps own domain meaning (truths, projections, copy).
  The platform owns auth, runtime, telemetry, secrets, storage, deployment
  (Runtime Runway), governance and commitment boundaries (Converge/Axiom),
  formations (Organism), specialist capabilities (Mosaic), and commercial
  authority (Commerce Rails). Stay in your lane; flag anything that belongs to a
  sibling.

Read the snapshots for the authoritative text:

- `the-new-normal.md` — the worldview and the inversion
- `reflective-paradigm.md` — esp. §5 (the loop) and §6 (the fit test + shapes)
- `stack-one-pager.md` — layer ownership and the three primitive modes
- `quorum-sense-substrate-scoping.md` — what the substrate demands of the stack

## Your review task

You are **one** component of the platform. Produce:

1. **Requirement extraction.** From the positioning, list the explicit technical
   capabilities the platform now needs — across the inversion (continuous
   autonomous action), the six universal requirements, and especially the three
   commitment shapes. Separate *burst-only* needs from *all-shape* needs.
2. **Component mapping.** Map each requirement to this repo's crates, milestones,
   and quality-backlog items. Cite files.
3. **Readiness verdict** per requirement: **Ready / Partial / Missing /
   Wrong-layer** (name the sibling that should own it).
4. **Shape-specific gaps.** Explicitly: what do the *non-burst* shapes
   (multi-sovereign; standing governance of autonomy) demand of **this**
   component that convened-burst apps never did? These are the assumption-breakers
   and the most important output.
5. **Gap-ordered work plan.** What to build here vs. in siblings, sequenced.
6. **Findings for the ledger.** Each finding with bucket (A/B/C/D), effort
   (S/M/L), `Codex-safe now?`, and evidence (file/line/commit). Follow the root
   `AGENTS.md` + `QUALITY_BACKLOG.md` conventions.

## Shape lens cheat-sheet (apply to your component)

- **Convened burst** — episodic, one owner, realtime or async room.
- **Multi-sovereign** — no single owner; parties across a trust boundary. *New
  platform demands: cross-org identity; selective-disclosure-safe storage and
  transport; tamper-evident cross-party (symmetric) receipts; dispute/arbiter
  surfaces.*
- **Standing governance of autonomy** — set rarely, operate continuously; the
  operate phase is the product. *New platform demands: long-horizon runtime;
  provable delegation scope ("outside scope escalates"); runtime telemetry
  compared against a mandate; intervention thresholds; kill/reopen including
  under degraded connectivity; executable (machine-actionable) projections.*

## Output

Write your review to `docs/strategy/REVIEW-<repo>-<YYYY-MM-DD>.md` in this repo
and summarise the top findings. **Do not edit the snapshot docs** in
`docs/strategy/` — they are read-only copies of the canonical KB.
