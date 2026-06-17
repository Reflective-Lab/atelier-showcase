# Reflective — The Stack Story

*Status: stack one-pager, v0.2, 2026-06-17. A starting-point explanation of how
the Reflective stack serves [[category-one-pager|governed decision translation]].
Canonical boundary source: [[04-architecture/current-system-map|Current System
Map]]. This page is positioning and architecture narrative, not a replacement
for code-backed ownership docs.*

## The stack in one sentence

Reflective is a **governed commitment substrate**: it turns a consequential
commitment into a canonical truth, translates it into role-specific operating
reality, and detects when reality or interpretation drifts enough to reopen it
— whether that commitment is decided in a room, negotiated between parties, or
set once to govern continuous autonomous action.

In enterprise terms, Reflective is not another system of record and not merely
a system of action. It is a **system of outcome commitments**: records and
actions become meaningful because they are bound to the outcome, authority,
assumptions, projections, and reopen conditions they are meant to serve.

## The lifecycle

The platform loop is:

**Decide -> Translate -> Operate -> Detect drift -> Reopen decision.**

Each layer exists because one part of that loop needs a different kind of
authority:

- **Decide:** establish the canonical commitment and its authority.
- **Translate:** project the core for each audience without distorting it.
- **Operate:** let people and AI act within delegated scope.
- **Detect drift:** compare reality, projections, and the canonical core.
- **Reopen:** route incoherence back to whoever holds authority to re-ratify.

## Three primitive modes

The loop is one shape; **Decide** runs in three modes, and the stack must
support all three rather than assuming the convened burst. Translate, Operate,
Detect drift, and Reopen are shared; the mode sets who holds authority and how
often a human returns.

- **Convened burst** — one owner, one room. Decide forms the room, captures
  dissent, and ratifies. Adds anti-HiPPO, hidden-profile extraction, quorum,
  ratification. *Reference: `quorum-sense`.*
- **Multi-sovereign commitment** — no single owner; parties across a trust
  boundary. Decide reconciles independent positions. Adds selective
  disclosure, symmetric receipts, obligation views, dispute paths, arbiters.
  *Reference: `tally-escrow`.*
- **Standing governance of autonomy** — decide rarely, operate continuously.
  Decide sets a mandate that Operate/Detect-drift carry for a long horizon.
  Adds delegation scope, runtime telemetry, policy gates, intervention
  thresholds, kill/reopen controls. *Reference candidate: `triage-keeper`.*

## The layers

```text
Helm / Apps       -> What does the operator see, decide, approve, and challenge?
Axiom             -> What is the canonical core and projection contract?
Organism          -> Which human/agent formation should reason over this?
Converge          -> What is admitted, promoted, refused, recorded, and reopened?
Mosaic            -> Which specialist capabilities provide evidence or checks?
Runtime Runway    -> Where does this run, resume, observe, and continue?
Commerce Rails    -> Who is entitled, billed, owed, or commercially constrained?
```

## What each layer contributes

**Helm / Apps — the room and the consequence.**  
Helm and product surfaces present the commitment surface for whichever mode is
in play: in a burst, who is here and what must be ratified; in a multi-sovereign
commitment, each party's obligations and disputes; in standing governance, the
mandate, telemetry, and intervention controls. Apps own domain consequence: the
strategy update, vendor decision, underwriting rationale, publication gate,
integration call, escrow state, or operations mandate.

**Axiom — the truth and projection shape.**  
Axiom turns the job into structured truth: canonical core fields, evidence,
constraints, authority, uncertainty, dissent, allowed interpretations, forbidden
distortions, and revisit triggers. It is the natural home for the typed shape of
Canonical Core + Governed Projection.

**Organism — the formation.**  
Organism decides which people, agents, roles, adversarial voices, simulations,
and specialist capabilities should reason before the system proposes a path. It
is where group cognition becomes engineered instead of improvised in the burst
mode — and where, in standing governance, the agent formation that operates the
mandate between human returns is composed and bounded.

**Converge — the commitment boundary.**  
Converge governs admission, promotion, refusal, receipts, replay, and reopen
events. It does not merely record what happened; it decides what is allowed to
become a governed fact or commitment, and what must stay draft, disputed, or
blocked.

**Mosaic — the specialist bench.**  
Mosaic provides reusable capabilities that the formation can call: policy
checks, source ports, models, solvers, memory, analytics, provider adapters, and
formal reasoning. These specialists propose evidence and checks; they do not own
the final commitment.

**Runtime Runway — the operating substrate.**  
Runtime Runway makes burst-and-ambient work possible in practice: auth,
accounts, hosting, storage, secrets, telemetry, event streams, resume, scheduled
or triggered ambient work, and deployment paths.

**Commerce Rails — the commercial authority.**  
Commerce Rails owns billing, entitlement, app installs, payouts, refunds,
disputes, reconciliation, and commercial obligations. Commercial facts can feed
the governed flow, but apps should not reinvent commercial truth locally.

## The end-to-end path

1. A human or app opens a consequential job in Helm or a product surface.
2. Axiom shapes the job into a canonical core candidate and projection contract.
3. Organism selects a formation: humans, agents, dissenters, and specialists.
4. Mosaic specialists provide evidence, memory, models, policies, and checks.
5. Converge governs what can be promoted, refused, ratified, or reopened.
6. Runtime Runway keeps the burst, ambient continuation, receipts, and telemetry
   running across time and surfaces.
7. Commerce Rails gates any commercial authority involved in the job.
8. Apps project the governed result back into the domain consequence the user
   actually hired the system to produce.

## What this stack is not

- It is not a generic agent stack. Agents propose; they do not own commitment.
- It is not a meeting stack. Meetings record discussion; this stack governs
  commitment and translation.
- It is not RAG plus workflow. Retrieval can inform the core, but it does not
  provide an authority model or distortion contract.
- It is not another data layer. Data quality, lineage, and integration are
  frontloaded substrate work; the product difference is governing what outcome
  the data and actions serve.
- It is not SaaS feature parity. Apps stay thin; shared primitives move down
  into Bedrock, Mosaic, Runway, and Commerce Rails.

## Engineering posture

The stack's reliability story has to match the promise. Outcome commitments are
not safe if the layers that govern them depend on implicit contracts, late-bound
semantics, or opaque runtime behaviour at the places where truth, authority,
obligation, money, or reopen decisions cross boundaries.

AI-assisted engineering changes the economics here. We can afford more
frontloaded implementation complexity when it buys predictable runtime
characteristics. Core substrate code should therefore prefer typed, compiled,
observable systems — Rust where the existing platform pattern already fits, Go
where operational simplicity is the better tradeoff, and thin web surfaces for
human interaction. Dynamic languages, VM-heavy deployments, and high-churn UI
frameworks are not banned; they must earn their place outside the commitment
boundary or behind a typed adapter.

The platform should spend abstraction on business semantics: canonical cores,
projections, formations, policies, entitlements, receipts, disputes, and drift
signals. It should avoid spending abstraction budget on accidental runtime
uncertainty in the system core.

## Proof path

The proof path deliberately spans shapes, so the stack is validated as a
substrate and not just a burst engine.

**`quorum-sense` builds the convened-burst primitive.** It exercises the burst:
inquiry, formation, dissent, confidence, canonical core, and projection-to-core
challenge.

**`plumb-execution` commercialises it as a standing mandate.** It turns strategy
into a canonical commitment, translates it for every function, then governs it as
a standing mandate — watching for drift and reopening before execution waste
compounds.

**`tally-escrow` proves the multi-sovereign shape.** One canonical agreement,
party-specific obligation projections, symmetric receipts, dispute paths, and
arbiter reopen — the assumption-breaking proof that the substrate works with no
single owner.

**A standing-autonomy proof** (candidate: `triage-keeper`) proves the third
shape: a sparse human core governing continuous machine operation through
delegation scope, telemetry, policy gates, intervention thresholds, and
kill/reopen controls.

The stack is successful when a user can trace one commitment from human intent
through formation, evidence, governance, projection, ambient continuation, drift,
and reopen — in any of the three modes — without losing truth, authority,
dissent, or consequence.

## Canonical Links

- [[category-one-pager|The One-Page Story]] — the category claim
- [[01-platform/reflective-paradigm|The Reflective Paradigm]] — platform doctrine
- [[the-new-normal|The New Normal]] — worldview
- [[04-architecture/current-system-map|Current System Map]] — code-backed boundaries
- [[glossary|Glossary]] — canonical terminology
