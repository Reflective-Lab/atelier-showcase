# Quorum-Sense and the Substrate — Technology Scoping

*Status: scoping, exploratory, v1, 2026-06-17. Translates the substrate/shape
doctrine ([[reflective-paradigm|The Reflective Paradigm]] §6) into what it means
for `quorum-sense` and the platform. Not a roadmap commitment. Doctrine defers to
the paradigm; code-backed boundaries defer to
[[../04-architecture/current-system-map|current-system-map]] and the
`marquee-apps/quorum-sense` repo. Evidence is cited to quorum-sense KB files.*

## The reframe: Quorum is one mode, and already more than a burst

The paradigm names `quorum-sense` the **convened-burst reference**. That is true
of one face of it — but Quorum's own vision is broader than a burst. Its charter
is *"continuous organizational sensemaking … an always-on background process"*
(`marquee-apps/quorum-sense/kb/vision.md` "What It Actually Is"), and its
namesake metaphor is literally continuous signal accumulation crossing a
threshold (`vision.md` "The Name and Its Metaphor"). It already carries
ambient/continuous machinery: Mnemos cross-session recall, anticipatory signal
detection, and an evolving SenseMap (`vision.md` "Mosaic Extensions";
`kb/Home.md` M3 charter).

So the substrate ask is **not** "make Quorum continuous." It already is. The ask
is two moves:

1. **Factor Quorum so its *convening* is one explicit `Decide` mode** of a
   shared loop — not the implicit shape of the whole platform.
2. **Promote the reusable loop primitives** Quorum has already built so the
   other two shapes (`concord-supply` multi-sovereign, `vigil-care` standing
   autonomy) consume them instead of reinventing them.

## What Quorum already gives the substrate (reuse)

Mapping Quorum's existing machinery to the six universal requirements
(paradigm §6) shows Quorum is where most of the substrate spine is already
proven:

| Requirement | Quorum's existing machinery | Reuse status |
|---|---|---|
| Canonical commitment | Axiom Truth Package; `InquiryThread.core_question`; TruthSpecs (`signal-requires-content`, `probe-cites-hypothesis`) | Generalise the typed core beyond "inquiry" |
| Authority model | Helm HITL approval + override/redirect; Converge promotion gate (`vision.md` "Helms", "Converge") | Generalise (see authority section) |
| Faithful projections | Role-aware signals (`ParticipantRole`), fuzzy hypotheses with preserved ambiguity | Generalise to party/machine audiences |
| Receipts & provenance | Lamport+Merkle integrity, `ProcessReceipt`, `/integrity` `/events` surfaces (`kb/Architecture/Integrity Adoption`) | **Strong — reuse close to as-is** |
| Drift detection | Anticipatory signals, SenseMap drift, Mnemos continuity | **Reuse as the `fathom`-class spine** |
| Reopen / re-ratify | `QuorumReached` → revisit; HITL gates before synthesis commits | Reuse; generalise the trigger |

Two of these — **integrity/receipts** and **drift detection** — are the most
valuable to promote, because every shape needs them and Quorum has already paid
the cost.

## Where the loop must be factored

The platform loop is **Decide → Translate → Operate → Detect drift → Reopen**
(paradigm §5). Only `Decide` changes by shape. The seam to cut:

- **`Decide` becomes pluggable:** convene (Quorum), negotiate across a trust
  boundary (Concord), set-mandate (Vigil).
- **`Translate`, `Operate`, `Detect drift`, `Reopen` become shared substrate
  services** living below the app — in Converge (admission/promotion/receipts/
  reopen), Axiom (canonical core + projection contract), and Mosaic
  (Prism/Ferrox/Arbiter/Mnemos for drift, allocation, gates, memory). Quorum
  already wires all of these; the work is making them shape-agnostic so Concord
  and Vigil consume the same crates rather than forking.

## What is genuinely new (Quorum cannot be configured into these)

From the alias-apps `kb/shape.md` and `kb/architecture.md` open questions — these
are platform capabilities Quorum does **not** have and should **not** absorb;
they belong to Bedrock / Mosaic / Runtime Runway:

**Multi-sovereign (Concord):**
- **Selective disclosure across a trust boundary.** Quorum assumes intra-org
  good faith (`vision.md` "Converge: The Engine"); multi-sovereign requires
  proving a projection faithful to a canonical core it may *not* fully disclose
  to the counterparty. This is the hardest new primitive.
- **Cross-org identity and authority** for signers and arbiters.
- **Symmetric receipts** — extend the Merkle receipt chain so no party holds a
  receipt the counterparty cannot independently verify, with no central owner.
- **Obligation projections** and a first-class **dispute path** (distinct from
  Quorum's synthesis-gate use of Arbiter).

**Standing autonomy (Vigil):**
- **Long-horizon delegation scope** where "anything outside scope escalates" is
  *provable*, not best-effort.
- **Executable projections (projection-to-machine):** projections that constrain
  machine action, with the integrity contract holding against actuation, not
  just human comprehension.
- **Kill/reopen under degraded connectivity** without stranding the patient or
  the delegated agents.
- **Drift false-positive thresholds as a gating contract** before any autonomy
  is granted (Quorum's fuzzy confidence is a foundation, not the gate).

## Burst timing: async and realtime are both first-class

Quorum has realtime live concurrent inquiry over SSE (`kb/live-concurrent-inquiry`,
`kb/Architecture/Live Client Architecture`). The substrate also needs **async
bursts** (experts join when free — the rural-clinical and maritime cases in
[[domain-fit-scan|Domain Fit Scan]]). Async should be a session-shape setting,
not a fork.

## What we need for Quorum-sense next (actionable, scoped to the repo)

1. **Name convening as a `Decide` mode** explicitly in Quorum's docs and the
   loop seam, so "Quorum = the platform" stops being the implicit assumption.
2. **Confirm which primitives graduate** from `quorum-sense` into shared platform
   crates as shape-agnostic substrate services — integrity/receipts and drift
   first (they are the strongest and most reused).
3. **Lift the authority model out of Quorum-local Helm logic** into a platform
   concern that can express all three: single ratifier (burst), multi-party
   signers + arbiters (multi-sovereign), delegation scope + revocation (standing
   autonomy).
4. **Write platform RFCs for the genuinely-new capabilities** (selective
   disclosure, cross-org identity, delegation-scope runtime, executable
   projections) — explicitly *outside* Quorum's scope, owned by Bedrock /
   Mosaic / Runtime Runway, so Concord and Vigil have a home to build against.

## Decisions for the human (needs-human, paradigm bucket D)

- Do `concord-supply` and `vigil-care` build on the **same Converge loop** as
  Quorum, or does standing autonomy need a separate long-horizon runtime?
- **Sequencing:** prove the multi-sovereign shape first (Concord/Tally) or the
  standing-autonomy shape first (Vigil/Triage)? Tally and Triage already exist
  in-portfolio, so one of them is the cheaper first proof.
- How much substrate to **promote now** vs. after a second shape exists — promote
  too early and you abstract on one example; too late and Concord/Vigil fork.

## Canonical Links

- [[reflective-paradigm|The Reflective Paradigm]] §5–§6 — the loop, the six requirements, the shapes
- [[domain-fit-scan|Domain Fit Scan]] — the shapes' cross-vertical demands
- [[stack-one-pager|The Stack Story]] — the layers and the three primitive modes
- `marquee-apps/quorum-sense/kb/vision.md` — Quorum's charter and stack usage
- `marquee-apps/quorum-sense/kb/Architecture/Integrity Adoption` — the receipt/integrity spine to promote
- `alias-apps/` — `concord-supply` and `vigil-care` shape exemplars and their open technical questions
