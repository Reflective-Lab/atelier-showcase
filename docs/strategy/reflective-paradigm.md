# The Reflective Paradigm

*Status: canonical paradigm v1.1, 2026-06-17. Authored kpernyer + claude with input from a second helper. This is the **single source of truth** for what Reflective is and the one claim every other document defers to — thesis, characteristics, primitive, integrity model, loop, moat. Boundary and portfolio claims here are **doctrinal direction**; they do not take effect until [[../04-architecture/current-system-map|current-system-map]] and the relevant project READMEs are updated to match. This paper does not silently override the boundary registry. When code conflicts with this paper, code wins and this paper updates.*

## Thesis

Reflective is not "AI for decisions." It is one thing, and we say it one
way:

> **Reflective keeps a consequential decision's truth, authority, and
> dissent intact as it becomes the operating reality of every role it
> touches.**

In one line: group judgment becomes governed commitment, and commitment
becomes shared understanding — without losing what was actually decided.

In the market story we say **decision** because it is the sharpest
entry point. In the platform model, a decision is one subtype of
consequential commitment, alongside agreements, mandates, policies, and
operating doctrines.

The hard problem is not making a decision. It is making the decision land
as the same operating reality for engineering, legal, HR, sales, finance,
the board, the regulator, and the affected employee. Organisations
already record decisions; they fail to make them shared. We own that gap.
We call the category **governed decision translation**, and we intend to
define it before anyone else names it.

Translation here is not after-the-fact communication. It starts before
commitment: forming the room, preserving dissent, ratifying the core,
projecting consequences, and reopening when projections reveal
incoherence. The decision and its faithful propagation are one governed
act, not a hand-off.

Another way to say the same thing: Reflective is not another **system of
record** and not merely a **system of action**. Systems of record remember
what happened; systems of action move work. Reflective is a **system of
outcome commitments**: it governs what outcome was committed, under what
authority, against which assumptions, through which authorised actions,
and with what drift signals that force a reopen. Data and workflow are
necessary substrate, but they are not the business object. The business
object is the governed outcome commitment.

### Category and substrate

Governed decision translation is the **category** — the market-facing
problem we own, sharpest for the convened decision and its translation.
The platform should not be planned as a single Quorum-shaped product.
Beneath the category, Reflective is a **governed commitment substrate**:
it governs any consequential commitment — whether *decided in a room*,
*negotiated between sovereign parties*, or *set once to govern continuous
autonomous action* — through one set of universal requirements:

1. a **canonical commitment or mandate** held as the source of truth;
2. an **explicit authority model** — who may commit, ratify, revoke;
3. **faithful projections** across the roles or parties it touches;
4. **receipts and provenance** for what was decided and why;
5. **drift detection** between commitment, projections, and reality;
6. a **reopen / contest / revoke / re-ratify** path when drift crosses a
   threshold.

The convened decision burst is one *shape* of commitment, not the
template for all of them. The category is what we sell first; the
substrate is what we build. §6 sorts the portfolio by which commitment
shape each app takes, and each shape adds its own machinery on top of the
six requirements.

## 1. The failure — decisions recorded, not shared

Every consequential commitment in a modern organisation fails twice.

It fails first when the decision itself is wrong. That is the classic
strategy problem — visible, well-named, and over-tooled.

It fails a second time, more quietly and more often, when the decision
is right but never becomes the same operating reality across the
organisation. Leadership hears aspiration; engineering hears extra work;
legal hears exposure; HR hears reorg risk; sales hears a quota story.
Each role privately builds its own working theory of what was decided.
None of them are wrong on their own face, and none of them is the
canonical commitment. The organisation drifts not because people are
disloyal or stupid, but because the commitment never became role-specific
meaning, obligations, tradeoffs, and revisit triggers.

This second failure is invisible to current tooling. Meeting apps
capture what was said. Workflow apps capture what was assigned. Knowledge
bases capture what was written. None of them notice that engineering's
working theory has drifted from the canonical decision until the
quarterly review reveals it. By then, weeks of execution have been spent
against an interpretation nobody ratified.

Reflective exists to engineer this second failure out.

## 2. The new normal — five characteristics

The platform is built around five claims about how consequential work
actually shifts under AI-assisted collaboration. Each is paradigm-level,
not feature-level. The longer-form, platform-agnostic worldview these
characteristics compress is [[../the-new-normal|The New Normal]].

**1. Humans are discrete, not continuous.** Humans are needed at moments
of framing, judgment, dissent, authority, and ratification. They are
not needed as middleware between systems, as ceremony hosts, or as the
thing that keeps the workflow alive between exchanges. A large share of
modern knowledge work is currently spent orienting around systems,
knowing the shortcuts, mastering the layers humans invented to abstract
previous layers humans invented. That is the layer that goes away.
David Graeber's *Bullshit Jobs* (2018) — the "box tickers," "duct
tapers," and "taskmasters" whose function is to keep prior layers of
system middleware coherent — provides the most-cited language for this
failure mode. The claim here is qualitative; the precise share would
need stronger quantitative evidence to defend. The new normal does not
delete these humans; it relocates them to the moments that actually
require judgment.

**2. Coordination is frontloaded.** The expensive work moves forward:
knowledge sharing, evidence checks, dissent surfacing, constraint
naming, authority clarity, uncertainty labelling, and option generation
happen before a commitment is admitted, not in scattered prep emails
before it and not in retro post-mortems after. In the convened shape
that means inside the burst; in multi-sovereign and standing-autonomy
shapes it means before the agreement or mandate is ratified. The
discipline is no-postpone: commitment formation ends with a governed
commitment, or with a specific ask and scheduled return — never a vague
defer.

**3. Commitment formation is engineered, not improvised.** In the
convened-burst shape, this means anti-HiPPO process (silent generation
before discussion), hidden-profile extraction (prompting for
uniquely-held knowledge, per Stasser & Titus, 1985), structured
turn-taking, anonymous confidence capture, designated dissent, and
visible AI facilitation as *platform features*, not facilitator skills.
In multi-sovereign commitments it means selective disclosure, symmetric
receipts, obligation views, dispute paths, and arbiters. In standing
governance of autonomy it means delegation scope, telemetry, policy
gates, intervention thresholds, and revoke/reopen controls. Woolley et
al.'s collective-intelligence work (*Science*, 2010) reports social
sensitivity and balanced participation — not individual IQ — as
directional predictors of group performance; the finding has had mixed
replication results and should be treated as directional, not settled.
The weaker claim survives across shapes: default commitment formation is
pathology-prone, and the machinery must be engineered rather than left
to room dynamics or ad-hoc protocol.

**4. AI works in two legitimate modes.** Inside the burst, AI assists
visibly: explains, reframes, challenges, summarizes, detects drift,
surfaces hidden profiles. Between bursts, AI operates ambiently within
**bounded delegated autonomy** — explicit authority, observable actions,
receipts, contestability, and human ratification gates. The
contestability is not theatrical; any participant can pull a
within-scope AI action back into a human burst for re-decision.

**5. Truth is stable; explanation is personal.** The canonical decision,
strategy, rule, claim, or commitment must not mutate. Its explanation
must adapt — to role, background, literacy, priority, available time,
and consequence. A board member, a new engineer, a regulator, an
affected employee, and a sales rep are entitled to different
*projections* of the same canonical truth. They are not entitled to
different truths. Engineering this distinction is the breakthrough.

These five describe the paradigm. Sections 3–5 describe how the platform
delivers them.

## 3. The primitive — Canonical Core + Governed Projection

The breakthrough is a single platform primitive that does not yet exist
in the market: **Canonical Core + Governed Projection**.

### The shape

Every consequential commitment in Reflective has two parts.

A **Canonical Core**, immutable per committed version (reopen creates a
superseding core; the prior version remains in the ledger as
superseded). Stable under projection within a version:

- source artifact (the decision, strategy, rule, claim, or commitment)
- evidence supporting it
- constraints (especially non-negotiables)
- authority (who decided, under what mandate)
- uncertainty (what is not known; confidence intervals)
- dissent (what was argued against and on what grounds)
- allowed interpretations (the projection envelope)
- forbidden distortions (the integrity contract — see §4)
- revisit triggers (under what reality changes this core reopens)

A set of **Governed Projections**, generated and audited for each
audience:

- explain this for engineering
- explain this for legal
- explain this for a new employee
- explain this for the affected operator
- explain this for a skeptical board member
- explain this for someone with three minutes
- explain what changes for my team tomorrow
- explain this for the regulator

Every projection is provenance-bearing, audit-traceable to the canonical
core, and signed by the platform under the integrity contract (target
capability; see §4 for what ships today). Any participant can request a
different projection of the same core. None of them can rewrite the core.

### The bidirectional clause

A projection is not a one-way translation. It is a **two-way contract**.

- Core → Projection: engineering reads the strategy through the
  implementation lens; legal reads it through the obligation lens.
- Projection → Core: when engineering's projection reveals
  implementation incoherence, or legal's projection reveals an
  obligation that contradicts a stated constraint, the projection can
  trigger a **reopen burst** on the core.

Without this second direction the platform just delivers interpretation
more cleanly than email and slides. With it, the platform actively
senses distortion at role boundaries and pushes it back to the canonical
layer. This is where Lost in Translation gets caught early instead of
after the strategy has failed twice.

The bidirectional clause is also where humans-stay-authoritative
becomes load-bearing. AI cannot rewrite the canonical core. But a human
reading their role-projection can challenge it — and the platform's job
is to make that challenge legible, routed, and acted on.

### Inherited shape; new contract

The canonical-core / governed-projection split itself is not new. The
social-science literature on **boundary objects** (Star & Griesemer,
1989; thirty-five years of subsequent work) describes exactly this
shape — artifacts stable enough for shared reference across
communities, plastic enough for local meaning. We inherit that framing
explicitly and do not claim novelty in the shape.

What *is* new — and what makes this a product category rather than a
social-coordination concept — is the **enforced integrity contract**
over the projection layer (§4) and the **bidirectional reopen** path
from projection back to canonical core. Boundary-object theory
describes the shape; it does not specify how to prevent silent
distortion, nor how to route downstream challenge back into the
canonical record. Reflective's defensible position is the contract and
the reopen path, not the shape they sit on.

### What this is *not*

Naming the primitive's category clearly is more important than naming
the primitive itself. It is:

- **Not RAG.** RAG fetches and assembles from a source corpus. It has no
  truth-preservation contract, no authority model, and no concept of
  forbidden distortion. A RAG system can quote the canonical core
  perfectly while losing every constraint that gives it meaning.
- **Not summarization.** Summarization compresses. By default it drops
  uncertainty (reads as low-confidence-summary), drops dissent (reads
  as off-topic), and drops authority chain (reads as administrative
  metadata). Each of these is a distortion under §4.
- **Not personalization.** Personalization adapts UI and content to
  preference. It has no truth contract. Two users seeing different
  personalized views of the same article have no platform guarantee
  that the underlying facts match.
- **Not "explain it like I'm five."** ELI5 freely distorts complexity
  for accessibility. That is the wrong move for consequential
  decisions: the audience needs *role-relevant* truth, not *simplified*
  truth.
- **Not language translation.** Language translation has well-developed
  fidelity norms but not for the structural properties (authority,
  constraint, uncertainty, dissent) that consequential commitments
  carry.

Canonical Core + Governed Projection is a new product category — novel
in the contract and the reopen path, not in the shape (see §3) — and the
question to anyone proposing to build it differently is: *what is your
truth-preservation contract, and what does it forbid?*

## 4. The integrity model — the distortion ontology

A projection is allowed to reframe. It is not allowed to distort. The
contract specifies eight categories of distortion the platform must
prevent and audit:

1. **Quantitative distortion** — numbers shift between core and
   projection. Strategy commits to "30% margin"; engineering's
   projection reads "high margin." That is a distortion.
2. **Authority distortion** — who decided, or under what mandate, gets
   re-attributed or laundered. "The CFO decided" becomes "leadership
   decided" becomes "we decided." Each step weakens accountability.
3. **Constraint distortion** — a non-negotiable gets softened or
   dropped. "Must not require additional headcount" becomes "should
   limit hiring" becomes "consider efficiency." The constraint
   evaporates across projections.
4. **Causal distortion** — the *why* gets replaced. The strategy
   commits because of regulatory deadline; engineering's projection
   reads as a market opportunity. The reason changes what trade-offs
   the audience makes downstream.
5. **Scope distortion** — what the commitment applies to widens or
   narrows. "EU customers" becomes "international customers" or vice
   versa. Scope drift is one of the most common Lost in Translation
   failures.
6. **Confidence distortion** — uncertainty gets compressed or inflated.
   The executive-friendly distortion: leadership wants a clean answer,
   so the projection compresses a wide confidence interval into a
   point estimate. This is the most pernicious distortion because the
   audience experiences it as clarity.
7. **Timing distortion** — when something happens or for how long
   shifts. "By Q3" becomes "this year." "Permanent" becomes
   "indefinite." "Six-month pilot" becomes "rollout."
8. **Obligation distortion** — what one party owes shifts in a
   bilateral commitment. The bilateral case specifically: the agreement
   said party A delivers verified code by deadline; the projection for
   party B reads as "code delivered" with the verification step
   weakened or removed.

### How the contract is enforced

The integrity contract is the product, not a feature. It is enforced in
two layers, and we lead with the first.

Today, every projection is ratified against the eight distortion classes
as an explicit rubric: a human with named authority signs that the
projection reframes without distorting, or refuses it. We have not seen a
general-purpose tool impose this discipline at all. The rubric — and the
data model behind it, where canonical core, projection, authority, and
dissent are captured structurally — is already a position we have not seen
a competitor hold.

The roadmap automates the rubric. Every projection is scanned against the
eight classes; high-confidence violations refuse publication outright;
medium-confidence cases escalate to a named ratifier under explicit
authority; edge cases route to a ratification burst. Automated detection
is where the moat deepens, and it is hard on purpose — difficulty is what
keeps the category defensible. Human authority at the integrity gate
stays non-negotiable even after detection ships, for the same reason it
is non-negotiable at the commitment gate (§5): the contract is only as
strong as the escalation path behind it.

Refusal is not failure. A projection that cannot honor the contract
surfaces back as "this audience cannot be served without distortion under
category X" — itself a signal that the canonical core or the audience
role needs work. We build the detector behind the rubric; we do not claim
it ships before it does. That is the one line we keep honest, because the
bet is the contract, not a demo.

## 5. The platform loop

The Reflective platform runs a five-phase closed loop over every
consequential commitment:

**Decide → Translate → Operate → Detect drift → Reopen.**

- **Decide (mode-specific commitment formation).** The platform forms
  the commitment in the mode appropriate to its authority: a convened
  burst, a multi-sovereign reconciliation across parties, or a standing
  mandate for continuous autonomy. In the burst shape, humans converge
  in a bounded session with rubberband edges. In the other shapes,
  equivalent frontloading happens through protocol, disclosure, dispute,
  delegation, and ratification machinery. The phase ends with the
  canonical core — authority, evidence, constraints, uncertainty, and
  dissent recorded in full. Coordination cost has moved forward.
- **Translate (projection generation).** Role-specific projections are
  generated under the integrity contract. Each projection is
  audit-traceable, provenance-bearing, and signed by the platform
  against the distortion ontology (target capability; see §4 for what
  ships today). Projections are not a one-time
  deliverable; they are queryable surfaces — any participant can
  request a fresh projection at any depth, at any time, for any
  audience.
- **Operate (ambient).** AI continues within bounded delegated
  autonomy: explicit scope, observable actions, receipts,
  contestability. Each role acts on its projection as operating truth.
  The canonical core is the platform's reference; the role projection
  is what people actually use. Ambient AI runs formations, watches
  telemetry, drafts candidates, screens evidence, and prepares the
  next burst.
- **Detect drift (ambient).** The platform continuously monitors three
  divergences: reality vs. projections (the world is not behaving as
  expected), projections vs. canonical core (a projection has stopped
  honoring the integrity contract), and projection vs. projection
  (engineering's working theory and sales' working theory have drifted
  apart enough to matter). Threshold crossings trigger reopen.
- **Reopen (mode-specific human return).** The canonical core comes back
  to the authority surface for its shape: a governed burst, an
  arbiter/dispute path, or a mandate review with revoke/re-ratify
  controls. The cycle restarts with the same humans-authoritative
  discipline at the gate.

This is not a "decision app." It is a closed-loop governance system
over the canonical commitment as reality changes around it.

It is also not "a data platform with AI on top." Clean records,
lineage, integration, storage, and retrieval are necessary, but they are
not sufficient. A data layer can tell the truth about yesterday while the
organisation executes against an obsolete promise. Reflective treats data
problems as frontloaded substrate work: use agents, typed importers,
lineage checks, reconciliation, and human ratification to make the record
usable, then bind that record to the outcome commitment it is supposed to
serve. The moat is not having more data; it is knowing when data,
interpretation, action, and committed outcome have drifted apart.

The loop is the same across all three commitment shapes (§6); only the
**Decide** phase changes mode. In the *convened burst* it is a room that
ratifies; in *multi-sovereign commitment* it is a negotiation that
reconciles independent parties across a trust boundary; in *standing
governance of autonomy* it is a mandate set once that the Operate and
Detect-drift phases then carry for a long horizon, with Reopen as the
rare human return. Translate, Operate, Detect drift, and Reopen are
common machinery; the shape determines who holds authority at Decide and
how often the loop returns to a human.

A note on terminology. "AI sovereignty" is the wrong term for the
ambient phase — too absolute externally, triggers the wrong reaction,
and obscures the contract. The right term is **bounded delegated
autonomy**: explicit scope, traceable actions, contestable receipts,
human ratification gates. The AI never owns a consequential commitment;
it operates between commitments under authority that humans can
withdraw at any time.

### Technical posture — reliability is part of the thesis

The substrate cannot be a vague orchestration layer over fragile runtime
convenience. If Reflective is going to govern outcome commitments, the
core layers need predictable runtime behaviour, typed boundaries,
deterministic checks, replayable receipts, and failure modes that can be
reasoned about.

AI-assisted development changes the engineering tradeoff. Languages and
frameworks with steeper learning curves but stronger runtime properties
are now easier to use well because coding agents can carry more of the
local complexity. That does not mean every product surface must be Rust,
or that Python, JavaScript, virtual machines, or interpreted runtimes are
never appropriate. It means they must earn their place. In core
governance, runtime, billing, provenance, policy, and commitment
boundaries, Reflective should prefer compiled, typed, observable systems
with small runtime surprises — Rust first where the current architecture
already proves the pattern, Go where operational simplicity is the
better fit, and thin web surfaces where the browser is the right delivery
medium.

The rule is not aesthetic. It follows from the platform promise. We can
spend AI-assisted effort up front to tame implementation complexity; we
should not spend operational trust forever on late binding, implicit
contracts, or opaque runtime behaviour in the layers that decide what is
true, allowed, owed, or reopened. Save abstraction and virtualisation for
business semantics — commitments, projections, formations, policies,
entitlements — not accidental uncertainty in the system core.

## 6. The app portfolio under this lens

This section is the doctrinal cut of the portfolio under the
canonical-core lens. Physical directory homes lag the doctrine on
purpose — moving repositories is follow-on work, not part of this paper
(see Open follow-ons). This is positioning, not a boundary change;
[[../04-architecture/current-system-map|current-system-map]] remains the
source of truth for where code actually lives.

### The fit test — substrate first, then shape

Earlier drafts forced one test on every app: it had to carry the
convened-burst machinery (engineered group cognition) to count. The New
Normal makes that too narrow — the convened burst is one shape of the
governing core, not the only one ([[../the-new-normal|The New Normal]]).
The fit test is now two levels.

**Level one — the universal substrate requirements.** A core app must,
regardless of shape, satisfy all six: (1) a **canonical commitment or
mandate** held as the source of truth; (2) an **explicit authority
model** — who may commit, ratify, revoke; (3) **faithful projections**
across the roles or parties it touches; (4) **receipts and provenance**;
(5) **drift detection** between commitment, projections, and reality; (6)
a **reopen / contest / revoke / re-ratify** path. These six are the
governed commitment substrate.

**Level two — the commitment shape.** Each shape adds its own machinery,
and an app is judged on the machinery of *its* shape, not on the burst:

- **Convened burst** — one owner, one room, a bounded session that
  ratifies. Adds anti-HiPPO, dissent capture, hidden-profile extraction,
  quorum, and ratification. *Reference: `quorum-sense`.*
- **Multi-sovereign commitment** — no single owner; independent parties
  across a trust boundary. Adds selective disclosure, symmetric receipts,
  obligation views, dispute paths, and arbiters. *Reference:
  `tally-escrow`; clean vertical exemplar: `concord-supply` (alias-apps).*
- **Standing governance of autonomy** — humans decide rarely, machines
  operate continuously, humans reopen or revoke when drift appears. Adds
  delegation scope, runtime telemetry, policy gates, intervention
  thresholds, and kill/reopen controls. *Reference candidate:
  `triage-keeper`; clean vertical exemplar: `vigil-care` (alias-apps).*

A fourth, *partial* fit is worth naming: **truth-preserving projection**
— an app that exercises requirements 3-4 strongly (faithful projection +
provenance) over an external canonical core it does not itself commit.
Real category fit on the projection slice, not a miss; it simply forms
no commitment in-app (e.g. `folio-editor`). To avoid becoming a
catch-all, this partial fit requires three things: a named external core,
explicit provenance back to that core, and an integrity gate that can
refuse distorted projections. Without those, it is just explanation or
content tooling, not Reflective-category work.

The **driver vs. applier** axis still sorts what to do. A *driver*
defines and advances the category and carries narrative and commercial
leverage; Marquee is reserved for drivers and stays deliberately small.
Everything else *applies* the substrate in one of the shapes. Quorum
should stop being the implied template for every app: an app fits if it
satisfies the six universal requirements **and** the machinery of *its
own shape*. Keeping Marquee strict is what stops the portfolio from
becoming "everything that kind of fits."

### Marquee — category drivers

**`quorum-sense` (reference primitive, driver — convened burst).** The
inquiry is the canonical core; advisor hypotheses are governed
projections of the unresolved question; dissent capture is first-class;
confidence gates are the integrity contract. Quorum is where the
convened-burst shape gets built and proven. JTBD: a governed decision
burst for business-critical group judgment.

**`plumb-execution` (commercial cash-out, driver — convened burst →
standing mandate).** The strongest commercial application, and the
clearest bridge between two shapes. Strategy is decided in a burst, then
becomes a **standing mandate** that ambient systems and teams operate
against until drift triggers a reopening. Engineering, GTM, legal, HR,
finance each receive governed projections; drift runs against reality and
against role-projection divergence; revisit bursts reopen the strategy.
This is the app where Lost in Translation becomes a measurable economic
outcome.

Marquee stays just these two until another app proves category-*driving*
leverage, not merely category fit.

### Applied — by commitment shape

These apply the substrate to a domain, sorted by shape and judged on the
machinery of that shape rather than on "does it look like Quorum?"

**Convened burst.**

- **`vouch-lending` (flagship candidate).** Applicant file = canonical
  core (rationale, evidence, constraints, dissent, uncertainty);
  compliance + borrower + regulator projections = governed projections;
  the 7-Suggestor formation is the engineered group cognition inside the
  burst. Full-fit on the burst shape.
- **`atlas-integration` (flagship candidate).** Deal thesis = canonical
  core; team-level integration consequences = projections; post-deal
  failure = drift; reopen on cartography evidence. Every M&A failure is
  Lost in Translation at scale — the highest-stakes burst-shape vertical.
- **`scout-sourcing`.** Vendor selection. The screening verdict is the
  canonical core; legal/security/finance/procurement projections are
  governed projections; selection commits with a full integrity trail.

**Multi-sovereign commitment.**

- **`tally-escrow` (flagship candidate, multi-sovereign reference).** The
  cleanest multi-sovereign app, and *stronger* under this lens, not
  weaker. One canonical agreement is the core; each party's obligations
  are governed projections across a trust boundary; verification produces
  symmetric receipts; disputes are projection divergence; arbiter bursts
  reopen the core. Full-fit on the *multi-sovereign* shape — the earlier
  "partial" label measured it against the wrong shape's machinery.

**Standing governance of autonomy.**

- **`warden-compliance`.** Rule = standing mandate; operator and
  regulated-app projections = governed projections; verdicts streaming
  back = drift signal; false-positive/negative episodes cross
  intervention thresholds and reopen. Read as a burst app it looked
  "partial"; read as standing governance it is a clean fit — continuous
  monitoring against a mandate with a human reopen path.
- **`triage-keeper` (standing-autonomy proof candidate).** Reconsidered.
  Under "governed decision translation" alone it sat outside the
  category. Under the standing-governance shape it may be a *core* proof:
  continuous operational autonomy governed by a sparse human core, with
  delegation scope, telemetry, policy gates, intervention thresholds, and
  kill/reopen controls. Promote from Operations to a watched
  standing-autonomy candidate; confirm against the six universal
  requirements before counting it core.

**Truth-preserving projection (partial shape).**

- **`folio-editor` / Newspaper.** Reporting = canonical core (claims,
  provenance, sources, constraints, allowed interpretations,
  fake-detection verdicts); the published article and reader-projection
  are governed projections. Its fit is truth-preserving projection —
  faithful projection + provenance over a canonical core — not the group
  burst. Judged on that slice it is a real fit, not demoted for lacking a
  burst. Physically at `studio-apps/folio-editor`.

### Reframe-watch — gestures at the category, not yet a clean fit

**`catalyst-biz`.** "Small-room governed commitments": each deal,
renewal, or campaign is a canonical core with role-projections for sales,
finance, marketing. The thinnest, most horizontal fit. The platform-floor
exemption in `marquee-apps/CLAUDE.md` must loosen to take on the memory
and projection surfaces (Open follow-ons).

### Platform components — promoted out of the app portfolio

**`fathom-narrative` → Mosaic specialist.** Temporal-narrative drift
detection over public corpora is an evidence engine consumable by Plumb
(strategy drift), Warden (regulatory drift), Folio (claim drift), Atlas
(deal-thesis drift), and Triage (operational drift) — it supplies the
substrate's drift-detection requirement to apps across all three shapes,
stronger as a platform component than as an app. **Candidate move, not completed**; requires confirmation and a
[[../04-architecture/current-system-map|current-system-map]] update
before it lands (Open follow-ons).

### Studio — single-creator craft

**`inkling-notes`, `moosemen-writer`, `wykkid-preso`.** Single-creator
craft tools. Anti-fragile memory applies (vault gets smarter; voice
profile accretes); canonical-core-with-projection does not. No
multi-participant pretence.

**`wolfgang-chat` (flagship, adjacent).** Branded expert reasoning panels
over a canonical corpus: the corpus is the core, expert personas are
projections of it for different audiences and depths. Commercially the
flagship and a genuine truth-preservation story over a corpus — but it
carries no consequential *commitment*, no group burst, and no reopen
loop, so it is category-adjacent, not core. Stays Studio; revisit if a
decision/commitment surface is added.

### Blueprint — KB doctrine and meta

**`keystone-architecture`, `shoal-meta`.** KB-first doctrine, not
products: Keystone is constraint-driven structure search across the
decision lifecycle; Shoal maps how the apps compose as stages of a single
decision. They explain the portfolio; they do not ship as apps.

### New apps the platform should be used for next

Under the lens, the clearest unbuilt apps sort by commitment shape.

*Convened burst* — decisions currently made through bad meetings with no
integrity contract:

- **Hiring committee.** Candidate review with anti-HiPPO, hidden-profile
  extraction (interviewers hold unique signal), decision rationale as
  canonical core, role-projections for hiring manager, recruiter,
  candidate, legal. Strongest near-term commercial bet.
- **Investment committee.** VC/PE deal decisions. Authority clear,
  dissent under-captured, decision memory weak. Investment thesis =
  canonical core; LP communication, portfolio-company onboarding, board
  reporting = projections.
- **Architecture review board.** Internal tech-org decisions on
  cross-cutting designs. Architecture decision = canonical core;
  affected-team projections = governed projections. RFC tooling exists;
  reasoning-process tooling does not.
- **Board meeting / corporate governance.** Highest authority stakes,
  weakest tooling. Board decision = canonical core; management
  projection + investor projection + regulator projection = governed
  projections.
- **Tumor board / multidisciplinary medical case conference.** Medical
  case decision = canonical core; patient projection + primary-care
  projection + payer projection = governed projections. Distinct from
  triage-keeper.

*Multi-sovereign commitment* — agreements between parties that no single
owner ratifies:

- **Supply-network / consortium commitment.** A shared production or
  delivery commitment across independent firms; each keeps a private
  position; obligation projections and symmetric receipts per party;
  disputes are projection divergence; arbiters reopen.
- **Standards / interoperability body.** A canonical specification as the
  shared core; member-specific conformance obligations as projections;
  drift when an implementation diverges from the spec.

*Standing governance of autonomy* — a mandate set rarely that governs
continuous machine-driven action:

- **Autonomous-operations mandate.** A policy that bounds what an agent
  fleet or automated pipeline may do, with delegation scope, runtime
  telemetry, policy gates, intervention thresholds, and kill/reopen
  controls. The human core convenes only on drift.

For use-cases deliberately *outside* the current portfolio's fields —
the rural clinical decision room, oil-and-gas well-control, maritime
casualty command, GMP change control, and more — screened against the fit
test to pressure-test the category and surface what they demand of the
stack, see [[domain-fit-scan|Domain Fit Scan]].

The hiring-committee and investment-committee opportunities read as the
strongest near-term commercial bets, on the **hypothesis** that
billions-of-dollars decisions currently made through pathology-rich
meetings represent a market willing to pay for integrity-contract
tooling. This is positioning, not evidence. Market validation —
willingness to pay, displacement of existing spend, sales cycle, and
buyer identification — is follow-on work, not assumed here.

## 7. The moat

The defensible position against general-purpose LLM products and against
incumbent meeting/workflow tools rests on seven properties we have not
seen any competitor deliver together:

1. **Truth-preservation contract.** General LLMs translate without one.
   The integrity model (§4) is the product, not a feature.
2. **Authority model.** General LLMs have no concept of who decided
   under what mandate. The canonical core treats authority as
   first-class data, not a citation footnote.
3. **Provenance under transformation.** General LLMs lose source
   attribution under summarization and paraphrase. Reflective signs
   every projection back to the canonical core.
4. **Dissent capture.** General LLMs converge on consensus output and
   drop minority voice as off-topic. Reflective treats dissent as a
   first-class element of the canonical core.
5. **Drift signal across role boundaries.** General LLMs are stateless
   per query; they cannot notice that engineering's working theory has
   drifted from the canonical commitment. Reflective's ambient layer
   is built around that detection.
6. **Bidirectional reopen.** General LLMs do not trigger upstream
   revision when a downstream projection reveals a problem.
   Reflective's projection-to-core challenge path is structural.
7. **Engineered group cognition.** General LLMs and meeting apps do not
   actively neutralize HiPPO, anchor effects, or hidden-profile failure.
   Reflective's burst design is built around these countermeasures from
   Stasser & Titus, Janis, Woolley et al., Kahneman/Sibony/Sunstein.

### The landscape, and the space we take

Five categories of tool sit near this problem; none occupies it:

- **General assistants.** Optimize for reader satisfaction, which
  actively rewards authority, constraint, and confidence distortion
  (§4). No truth-preservation contract.
- **Enterprise search.** Retrieves and assembles from a corpus. No
  authority model, no canonical commitment, no concept of forbidden
  distortion.
- **Workspace and document suites.** Store and personalize. No truth
  contract binding the views of the same decision together.
- **Meeting intelligence.** Captures what was said. It records the
  decision; it does nothing about whether the decision becomes shared.
- **Governance and workflow tools.** Track tasks and attestations. They
  route obligations; they do not preserve the meaning of the commitment
  across roles.

The white space is unoccupied on purpose. Holding one canonical
commitment immutable while serving every role a provably faithful
projection of it is harder than anything above, and none of these
categories is built to pay that cost. That is the space Reflective takes.
We do not need the incumbents to be absent — we need to be the only ones
who answer the question below. A dated mapping of these categories to
named products lives in Appendix A; the category claims and the test
question are what stay durable.

The competitive question to ask of any incumbent or new entrant is the
same: *what is your truth-preservation contract, and what does it
forbid?* If they cannot answer it precisely, they are not in this
category.

## Theoretical anchors

The paradigm draws on four well-developed research lineages, used
lightly:

- **Boundary objects** (Star & Griesemer, *Social Studies of Science*,
  1989). The canonical-core-and-projection split is essentially a
  boundary object: stable enough for shared reference, plastic enough
  for local meaning. There is a 35-year tradition of social-science
  work on exactly this shape. This is the closest existing academic
  framing for what the primitive does.
  <https://en.wikipedia.org/wiki/Boundary_object>
- **Sensemaking** (Karl Weick, *Sensemaking in Organizations*, 1995).
  Explicitly about how shared reality forms or fails in organisations.
  Reflective is sensemaking infrastructure.
  <https://en.wikipedia.org/wiki/Sensemaking>
- **Organisational culture** (Edgar Schein). Culture as shared
  assumptions about what is real and what matters. Lost in Translation
  is the culture failure mode at role boundaries.
  <https://en.wikipedia.org/wiki/Edgar_Schein>
- **Cynefin** (Dave Snowden). Distinguishes complicated (expertise
  sufficient) from complex (sensemaking required). Consequential
  collective-reasoning is complex-domain work; the platform is built
  for that domain, not the complicated one.

Supporting empirical references already woven in:

- **Hidden profile / shared-information bias** (Stasser & Titus, 1985
  and successors). <https://en.wikipedia.org/wiki/Hidden_profile>
- **Collective intelligence (c-factor)** (Woolley et al., *Science*,
  2010). Cited directionally; replications mixed.
- **Groupthink** (Janis, 1972).
- **Noise** (Kahneman, Sibony, Sunstein, 2021).
- **Nominal Group Technique** (Delbecq & Van de Ven, 1971).
  <https://en.wikipedia.org/wiki/Nominal_group_technique>
- **Psychological safety** (Edmondson). Necessary but not sufficient
  without high-standards frontloading.
  <https://en.wikipedia.org/wiki/Psychological_safety>
- **Agile Manifesto principles** (2001). Distinct from rotted Scrum
  ceremony. <https://agilemanifesto.org/>
- **Bullshit Jobs** (Graeber, 2018). Empirical backing for "humans as
  system middleware."
- **LLM facilitation effects** (recent arXiv work on increased sharing
  without improved decision quality; on steering and illusion of
  inclusion). Read as a cautionary baseline for facilitation design.

## Open follow-ons

This paper *proposes* the cut as doctrinal direction. The boundary
registry [[../04-architecture/current-system-map|current-system-map]]
continues to track physical code homes and is not restructured by this
paper; the app READMEs and `marquee-apps/CLAUDE.md` carry the proposed
category as an overlay, not as a completed move. Four things remain open;
none is blocking.

1. **Physical repository moves.** The doctrine now places the appliers in
   Applied (sorted by commitment shape), `folio-editor` as a
   truth-preserving projection fit, `triage-keeper` as a standing-autonomy
   proof candidate, and `fathom-narrative` in Mosaic — but the directories
   have not moved. Decide whether Applied becomes its own top-level home
   (as `blueprint-apps/` did) or stays a logical grouping, then move the
   repositories and re-point the registry. Until then, the physical homes
   in [[../04-architecture/current-system-map|current-system-map]] are
   the source of truth.
2. **The `catalyst-biz` floor exemption.** The exemption line in
   `marquee-apps/CLAUDE.md` (`⚪ exempt — no converge/organism/helms
   deps`) must loosen so Catalyst takes on the memory and projection
   surface every category app needs, while keeping exemption for the
   heavier `bedrock-platform/converge`, `bedrock-platform/organism`, and
   `bedrock-platform/helms` wiring it does not yet need.
3. **`wolfgang-chat` placement.** Classified Studio (flagship, adjacent)
   because it carries no consequential commitment or group burst. Revisit
   if a decision/commitment surface is added; it would then re-enter the
   category.
4. **`fathom-narrative` confirmation.** The promotion to a Mosaic
   specialist is a candidate move requiring platform-owner confirmation
   before the code is relocated and consumers (Plumb, Warden, Folio,
   Atlas) are wired to it.
5. **`alias-apps` shape proofs.** The `alias-apps` repo holds two
   exploratory, KB-only exemplars — `concord-supply` (multi-sovereign) and
   `vigil-care` (standing governance of autonomy) — that prove the two
   non-convened shapes cleanly per vertical. They are not Marquee and not a
   portfolio commitment; the boundary registry should register the repo
   when it lands as code. See [[domain-fit-scan|Domain Fit Scan]].

The paradigm itself — thesis, five characteristics, Canonical Core +
Governed Projection, integrity model, five-phase loop, portfolio cuts,
moat — is the canonical position to be argued with first.

## Appendix A — Market scan (2026-06-17)

This appendix is a dated snapshot, not canonical doctrine. Named products
move, reposition, and ship features quarterly; treat this as a market
observation with a shelf life, and re-scan before relying on it. The
durable claim lives in §7, and the test question — *what is your
truth-preservation contract, and what does it forbid?* — is what does not
rot.

As of 2026-06-17, mapping the five categories in §7 to named tools we
have looked at:

- **General assistants** — Microsoft Copilot, ChatGPT Enterprise.
- **Enterprise search** — Glean.
- **Workspace and document suites** — Notion AI.
- **Meeting intelligence** — Granola, Otter, Fireflies.
- **Governance and workflow tools** — ServiceNow, Workiva.

We have not seen any of these adopt a truth-preservation contract over a
canonical commitment. That is an observation as of this date, not a
permanent claim.

## Canonical Links

- [[../category-one-pager|The One-Page Story]] — single-page category claim
- [[README|Platform Vision & Operating Model]]
- [[../02-product/README|Product & User Experience Architecture]]
- [[../03-commerce/README|Business & Commerce Architecture]]
- [[../04-architecture/README|System Architecture & Technical Platform]]
- [[../04-architecture/current-system-map|Current System Map]]
- [[../07-knowledge/README|Knowledge Map & Decision Register]]
- [[../stack-narrative|The Reflective Labs Story]]
- [[../business-pitch|Business Pitch]]
- [[../investor-pitch|Investor Pitch]]
- [[../glossary|Glossary]]
- [[../02-product/intent-codec-jtbd-schema|Intent Codec JTBD Schema]]
- `marquee-apps/CLAUDE.md` — Marquee portfolio table (needs catalyst exemption update; see §6)
