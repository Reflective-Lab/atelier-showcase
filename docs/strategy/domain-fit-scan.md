# Domain Fit Scan — Out-of-Bounds Use-Cases

*Status: exploratory, v2, 2026-06-17. Not doctrine and not a portfolio
commitment. A candidate pool of use-cases from fields outside the current
apps, screened against the substrate so we can tell whether the platform is a
real **governed commitment substrate** or just a description of `quorum-sense`.
v2 reorganises the scan by **commitment shape** ([[reflective-paradigm|The
Reflective Paradigm]] §6): the first version's two deep fits were both
convened bursts — i.e. vertical configurations of Quorum, not new application
classes. The genuinely assumption-breaking fits are the multi-sovereign and
standing-autonomy shapes, and they lead here. This feeds the next step: what
each shape demands of the stack and of `quorum-sense`.*

## The screen

A use-case earns a place only if it satisfies the **six universal substrate
requirements** (paradigm §6): a canonical commitment or mandate; an explicit
authority model; faithful projections across roles or parties; receipts and
provenance; drift detection; and a reopen / contest / revoke / re-ratify path.
Then it is sorted by **shape**, because the shape — not "does it look like
Quorum?" — determines what new machinery it demands.

The honest test for *novelty*: if the platform could serve the use-case as a
convened burst plus connectors and settings, it proves Quorum generalises but
is not a new application class. A use-case is interesting here only if it
**breaks** one of Quorum's assumptions — single owner, episodic convening, or
projection-to-humans.

---

## Shape A — Convened burst (proves generalisation, not novelty)

These are real fits, but they are *vertical applications of the existing
primitive*: one owner, one bounded burst, projection to human roles. Worth
recording as commercial surface; not evidence that the substrate is broader
than Quorum. Listed briefly for that reason.

- **Rural clinical decision room ("the expert across the hall").** A
  generalist in rural Kentucky has no specialist across the hall. A 15-20
  minute burst pulls one or two remote specialists plus an AI that has
  *frontloaded* the literature (PubMed/Cochrane/guidelines); hidden-profile
  extraction is the point (the rural doctor holds local/patient context, the
  specialist depth, the AI breadth) and anti-HiPPO keeps the famous-hospital
  voice from steamrolling the local read. Killer distortion: **confidence**
  ("probably" → "definitely" harms patients) and **authority** (the
  responsible licensed clinician). *This is a convened burst with a PubMed
  connector — Quorum config, not a new class.*
- **Well-control decision-stop (oil & gas).** A Management-of-Change / kick
  decision on the rig, convened across operator, contractor, and shore
  engineering. Macondo, read through our lens, is a convened decision whose
  meaning never became one shared commitment. Killer distortion: **authority**
  and **confidence**. *Also a convened burst — realtime telemetry connector,
  not a new class.*

---

## Shape B — Multi-sovereign commitment (assumption-breaking: no single owner)

These break the single-owner assumption: independent parties across a trust
boundary must form a *shared* commitment while each keeps a private position.
The convened-burst machinery (anti-HiPPO, hidden-profile) does not transfer;
they demand selective disclosure, symmetric receipts, obligation views,
dispute paths, and arbiters. This is the cleanest evidence the substrate is
broader than Quorum.

- **Supply-network production commitment (manufacturing).** A shared build or
  delivery commitment across OEM, tier-1/tier-2 suppliers, and logistics —
  each holding private cost, capacity, and risk it will not fully disclose.
  One canonical agreement; party-specific obligation projections; verification
  produces symmetric receipts; a missed obligation is projection divergence;
  arbiters reopen. Killer distortion: **obligation** and **scope**. Maps to
  `tally-escrow` at network scale, and is sketched as `concord-supply` in the
  `alias-apps` repo.
- **Maritime casualty command (shipping).** After a grounding or collision the
  decision and its record are shared across master, shore-side Designated
  Person, classification society, flag state, P&I club, salvor, and coast
  guard. The official log, the owner's report, the insurer's claim, and the
  regulator's incident report **must be one truth** — divergence is fraud and
  lost cover. No single owner ratifies. Killer distortion: **obligation +
  authority**. Async, intermittent connectivity.
- **Standards / interoperability consortium (cross-industry).** A canonical
  specification as the shared core; member-specific conformance obligations as
  projections; an implementation diverging from the spec is drift; disputes go
  to an arbiter. Killer distortion: **scope** and **causal**.

---

## Shape C — Standing governance of autonomy (assumption-breaking: the operate phase is the product)

These break the episodic-convening assumption: a mandate is set rarely and
*governs continuous machine-driven action* for a long horizon. The hard part
is Operate → Detect-drift → Reopen, not the room. They demand delegation
scope, runtime telemetry, policy gates, intervention thresholds, and
kill/reopen controls — and often the projection is to *machines that act*, not
only humans.

- **Autonomous / remote operations mandate (oil, shipping, manufacturing).** A
  policy that bounds what a remote-operations centre, an automated process
  line, or an autonomous-routing system may do between human touches. The
  canonical mandate governs continuous action; telemetry feeds drift; a
  threshold breach triggers human reopen or kill. The *interesting* version of
  the oil case lives here, not in the decision-stop burst.
- **Chronic-care management plan (medical).** Distinct from the rural burst: a
  care plan set rarely that governs continuous, multi-provider action over
  months or years, with the patient's evolving state as drift and reopen when
  thresholds cross. No single burst owns it; the operate phase is the product.
  Killer distortion: **timing** and **constraint**. Sketched as `vigil-care`
  in the `alias-apps` repo.
- **Continuous compliance mandate (regulated industries).** A rule set as a
  standing mandate that governs ongoing automated monitoring; verdicts stream
  back as drift; false-positive/negative episodes cross intervention
  thresholds and reopen. This is the `warden-compliance` shape generalised.
- **Grid / energy dispatch policy.** Operating limits set by humans that
  govern continuous automated dispatch; reality drift (demand, outage) reopens
  the policy. Killer distortion: **constraint** and **timing**.

---

## Cross-cutting read (the bridge to Quorum-sense)

The scan's payload is what the three shapes demand that a burst-only platform
would under-build:

- **Decide runs in three modes**, not one: convene (burst), reconcile
  (multi-sovereign), set-mandate (standing autonomy). Quorum-sense should
  expose its convening as *one* mode of a shared loop, not as the platform.
- **Authority and identity get heavier off-platform**: licensure and command
  authority (burst verticals), multi-party sign-off with non-disclosure
  (multi-sovereign), and delegation scope with revocation (standing autonomy)
  are three different authority models the substrate must carry.
- **Projection has a third target — machines that act.** Burst verticals
  project to humans; multi-sovereign projects across a trust boundary with
  selective disclosure; standing autonomy projects into *executable* bounds
  that constrain machine action. Integrity must hold against all three.
- **Drift detection is the common spine** but the sources differ: reality vs.
  projection (burst), obligation vs. verification (multi-sovereign), telemetry
  vs. mandate (standing autonomy). `fathom-narrative`-style evidence engines
  feed all three.
- **Connectivity and latency are a deployment-profile axis**, not a global
  assumption — forgiving for async clinical and casualty cases, unforgiving
  for well-control and dispatch.

These are *fits*, not roadmap. Every one is regulated, slow, and
liability-bound; the value of the scan is that the substrate survives contact
with fields we have no apps in — and that the multi-sovereign and
standing-autonomy shapes, not the burst verticals, are where the next
`quorum-sense` scoping has to earn the word "substrate." That scoping is now
drafted: see [[quorum-sense-substrate-scoping|Quorum-Sense and the Substrate]].

## Canonical Links

- [[reflective-paradigm|The Reflective Paradigm]] — the substrate, the shapes, the fit test
- [[the-new-normal|The New Normal]] — the worldview and the operating inversion
- [[category-one-pager|The One-Page Story]]
- [[stack-one-pager|The Stack Story]]
- [[quorum-sense-substrate-scoping|Quorum-Sense and the Substrate]] — the technology scoping this scan feeds
- [[../04-architecture/current-system-map|Current System Map]]
- `alias-apps` repo — `concord-supply` (multi-sovereign) and `vigil-care`
  (standing autonomy): KB-only shape exemplars for two of the shapes scanned here
