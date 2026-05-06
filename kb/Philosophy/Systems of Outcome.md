---
tags: [philosophy, vision, saas, organization]
source: human
---
# Systems of Outcome

A continuation of [[From Instructions to Intent]]. If the first doc explains
*what* changed, this one explains the consequences for the two domains we
care about most: **how software is sold and consumed (SaaS)**, and **how
organizations are structured to build and own it**.

## 1. The SaaS shift: tools → systems of outcome

### What SaaS has been

Classical SaaS is a **tool**. You rent a UI and a database wrapped around a
workflow. The customer brings:

- the intent (*"we want to close books faster"*)
- the decisions (*"so we'll use these fields, this approval chain, this
  report"*)
- the labor (*humans clicking through screens to enact the decisions*)

The vendor's job is to ship a reliable, configurable surface and stay out
of the way. Pricing follows the surface: per-seat, per-feature, per-tier.

This works as long as **the customer is willing to do the
intent-to-decision translation themselves**.

### What is changing

Once a system can resolve intent into decisions at runtime, the line moves.
Customers stop asking *"give me a tool to do X"* and start asking *"give me
the outcome of X"*. The vendor's surface is no longer the workflow — it is
the **promise**:

- not "an expense tool" → "expenses reconciled and policy-clean by month-end"
- not "a hiring ATS" → "a shortlist of five qualified candidates per role
  per week"
- not "a CRM" → "a pipeline that stays accurate without manual hygiene"

The product is the outcome. The workflow is implementation detail.

### What this means concretely

- **Surface inverts.** Screens become the *fallback*, not the *primary
  interaction*. The primary interaction is the contract: what outcome,
  under what constraints, with what evidence.
- **Pricing inverts.** Per-seat assumes humans driving the tool.
  Outcome-priced SaaS charges per resolved case, per closed loop, per SLA
  met. Seats become a vestigial unit.
- **Differentiation inverts.** "Most features" loses to "most reliably
  produces the outcome." Evaluation suites and audit trails become more
  important than feature matrices.
- **Defensibility inverts.** Workflow lock-in (*you can't leave because
  your team learned the screens*) gets replaced by outcome lock-in
  (*you can't leave because the system has accumulated context, policy,
  and proven reliability for your specific domain*).

### Why most incumbents will struggle

A SaaS company built around a tool has:

- product orgs structured around features
- pricing structured around seats
- success structured around adoption
- engineering structured around UI surface area

None of these map cleanly onto outcome delivery. Bolting an LLM into the
sidebar does not convert a tool into a system of outcome — it just decorates
the tool.

The hard part is not the model. It is the **substrate**: contracts, packs,
constraints, evaluation loops, provenance. That substrate is what this
ecosystem (Converge, the extensions, this repo) exists to build.

### What this looks like in atelier

Each runnable example is a small system of outcome, not a workflow:

- `expense-approval` — outcome: a policy-compliant decision with audit
  trail, not "an approval screen"
- `vendor-selection` — outcome: a justifiable choice across criteria, not
  "a comparison spreadsheet"
- `loan-application` — outcome: a defensible underwriting decision, not
  "an application form"
- `meeting-scheduler` — outcome: a meeting that respects everyone's
  constraints, not "a calendar grid"

The job of an exemplar is to show the **relocation of structure** for a
real outcome — what becomes contract, what becomes guardrail, what becomes
evaluation, what becomes runtime adaptation.

### Unit economics: from rent to cost-per-outcome

Classic SaaS gross margins look like this: ship the same software to many
tenants, marginal serving cost approaches zero, gross margin sits at
75–85%. The whole valuation framework — ARR multiples, payback periods,
NRR — assumes that shape.

Outcome systems have a different cost shape. Each resolved outcome
consumes inference, retrieval, evaluation, and sometimes human review.
The marginal cost is **not** zero. A bad-fit customer can be unprofitable.

Consequences:

- Pricing has to reflect variable cost. Per-outcome, per-resolved-case,
  per-SLA-tier — not flat per-seat.
- Customer selection matters. Some workloads are profitably servable,
  others are not. SaaS could afford to take everyone; outcome systems
  cannot.
- Cost engineering becomes part of product. Caching, model routing,
  distillation, eval-driven prompt compression — these are not
  optimizations, they are how the product stays solvent.
- Valuation models will lag reality. Until the market internalizes the
  new shape, expect mispricing in both directions.

### Procurement and the contractable promise

Selling a tool, the contract says *"you get access to the software."*
Selling an outcome, the contract has to say what outcome, under what
conditions, with what recourse. That is harder, in interesting ways.

Customers will demand:

- **Definition of done.** What counts as the outcome being delivered?
  Who arbitrates?
- **SLAs on quality, not just uptime.** "99.9% available" is meaningless
  when the question is "is the answer right?"
- **Audit trails.** Regulated industries need to know how a decision was
  made and be able to reproduce it.
- **Indemnification for wrong outcomes.** If the system mis-categorizes
  an expense or mis-screens a candidate, who eats the loss?

Most SaaS legal teams cannot draft these contracts today. The vendors
that learn to — and that build the substrate (provenance, evaluation,
replay) to back the promises — pull ahead. The substrate becomes part
of the sale, not an internal concern.

### Vertical specialization re-emerges

Horizontal SaaS — one product, many industries — was the dominant
winning strategy of the last cycle. It worked because the workflow was
generic enough to abstract.

Outcomes are not generic. The constraints, edge cases, regulatory
regime, and acceptable defaults for *"approve an expense"* differ across
industries. Outcome systems that try to be horizontal end up watered
down — generic enough to be safe, weak enough to be replaced.

Expect the next cycle to favor:

- Vertical-first products with deep domain packs.
- Horizontal **substrate** providers (this is us) selling into vertical
  builders.
- Two-layer markets: substrate vendors and outcome vendors, with the
  latter being domain-specialized.

This also means industries that were too small for horizontal SaaS to
bother with become economic again. Substrate amortizes the heavy
engineering; the vertical layer can be lean.

### Implementation as the new onboarding surface

SaaS taught the industry that onboarding should be self-serve, low-touch,
and measurable in days. Outcome systems break that because the first job
is to **encode the customer's actual constraints** — their policies,
their edge cases, their tolerance for wrong answers.

That is implementation work. It looks more like the consulting
engagements of the pre-SaaS era than the click-through wizards of modern
SaaS. This is not a regression. It is a real cost that has to be priced in:

- The first 30–90 days are constraint elicitation, not feature
  configuration.
- Customer success becomes a domain-implementation role, not a
  cheerleader role.
- Time-to-value is longer. The lock-in is deeper. The contract values
  are higher.
- "Self-serve" becomes a goal of the **substrate**, not the product.
  Substrate can be self-serve; an outcome cannot, until it has been
  instantiated for a customer.

Companies that try to keep the SaaS onboarding model will under-encode
constraints, ship hallucinations into production, and lose customers
fast. Companies that lean into implementation as a craft will earn long
contracts.

### The data and trust flywheel

Once an outcome system is running for a customer, two things compound:

1. **Encoded judgment.** Every constraint added, every adversarial case
   captured in evals, every replay of a tricky decision becomes part of
   the system's installed knowledge for that customer. Replacing the
   vendor means re-encoding all of it.
2. **Demonstrated reliability.** Provenance and eval history accumulate
   into a track record. New buyers will weight a vendor with two years
   of clean replays far more than one with a slick demo.

The flywheel is **per-customer-per-domain**, not generic. This is why
early entrants in a vertical are surprisingly hard to dislodge even
when a competitor's model is better — the competitor is selling
capability, the incumbent is selling accumulated trust.

## 2. The org shift: rethinking architecture and ownership

### What stops working

Most engineering orgs are factored around the old pipeline:

- **Frontend / backend / data** splits assume a UI-driven product.
- **Feature teams** assume features are the unit of value.
- **Product managers** assume someone has already done the
  intent-to-decision translation, and their job is to specify the
  decisions.
- **Platform teams** assume the platform is a shared library of plumbing.

When the unit of value becomes an outcome under guardrails, none of these
fit cleanly:

- The "frontend" may be a chat surface, an API, an inbox listener, or
  nothing at all.
- The "feature" may be a new contract, a new constraint, or a new
  evaluator — not a screen.
- The PM is now specifying **constraints and acceptable outcomes**, not
  click paths.
- The platform is now the **substrate that makes outcomes possible** —
  contracts, capability registries, evaluation, provenance. It is far
  more central, and far more opinionated.

### A more honest factoring

Three layers, each with a different posture:

1. **Substrate (deterministic, compiled, slow-moving).**
   Contracts, packs, provider APIs, policy, provenance, evaluation
   harnesses. Owned by a small platform group. Treated like infrastructure
   — boring on purpose, hard to change, heavily versioned.

2. **Domain (semi-structured, medium-moving).**
   Domain packs, capability implementations, guardrails, eval suites for
   a specific outcome. Owned by domain teams who know the business well
   enough to write the constraints. This is where most engineering effort
   actually lives.

3. **Adaptive surface (intent-driven, fast-moving).**
   The runtime that takes user intent, picks plans under constraints,
   resolves ambiguity, executes, and reports. Often thin. Often largely
   declarative. Owned close to product/users.

The mistake is to staff layer 3 like it is layer 1, or to hide layer 1
inside layer 2. Each layer has a different change rate, a different risk
profile, and a different definition of "done."

### Ownership shifts

- **Contracts are the new APIs.** Whoever owns a contract owns a promise
  to every consumer. Contract review becomes as load-bearing as code
  review used to be.
- **Evals become a first-class artifact.** A capability without an eval
  suite is unowned. Evals belong to the domain team, not to QA.
- **Provenance is product, not paperwork.** "How did we arrive at this
  decision?" is a feature, not a compliance afterthought. It needs an
  owner.
- **Guardrails are policy as code.** They sit between domain and
  substrate, and they need explicit ownership — usually domain, with
  substrate enforcing the shape.
- **Models are vendors, not architecture.** Treat them like any other
  capability provider: swappable, versioned, evaluated. Do not let model
  choice become an org silo.

### What dies

- The pure "frontend team" as a unit of org design.
- The PRD that specifies click-by-click behavior.
- The platform team that ships only plumbing.
- The QA team that owns "testing" as a separate phase.
- The "AI team" as a parallel org branch — adaptive behavior is a
  property of the whole stack, not a department.

### What gets stronger

- Domain expertise. Writing good constraints requires *understanding the
  business deeply enough to know what must never happen*. This is a
  scarce skill and it becomes central.
- Platform engineering, but reframed. Less "build the rails," more
  "design the contracts that make outcomes composable."
- Evaluation engineering. A new discipline. Closer to SRE in temperament
  than to QA.

### New roles, named

The work has to land somewhere. The roles that emerge:

- **Capability owner.** Owns one or more capabilities exposed via the
  provider API — their contracts, their evals, their failure modes,
  their cost. Closer to a product manager for an internal API than to
  an engineer for a feature.
- **Eval engineer.** Builds and maintains evaluation suites: regression
  sets, adversarial sets, calibration sets, deployment gates. Adjacent
  to SRE in temperament — failure-curious, statistically literate.
- **Contract engineer.** Owns the contracts (Pack, ProposedPlan,
  ProblemSpec, capability schemas). The job is API design at the
  substrate level, with consequences spanning many teams.
- **Domain modeler.** Translates business policy into pack structure
  and constraints. Often the most senior engineer on a domain team.
  Requires deep domain knowledge plus the ability to write executable
  specifications.
- **Provenance engineer.** Owns the audit-trail surface: what gets
  recorded, what gets exposed, what gets replayed. Product-facing,
  not just compliance.

Not every org needs all five named, but the work has to live somewhere.
Hiding it inside "platform" or "AI" is how you end up with no one
accountable.

### The "AI team" anti-pattern

The most common org mistake: create an "AI team" parallel to the
existing engineering org. It is staffed by ML/LLM specialists. Other
teams send them feature requests.

It fails predictably:

- Domain context lives in the other teams. The AI team writes
  plausible-looking solutions that miss the constraints.
- Substrate decisions (contracts, packs, evaluation harnesses) get made
  by people without the leverage to change them across the org.
- Adaptive behavior becomes a "feature" instead of a property of the
  system. Other teams keep building deterministic flows; the AI team
  keeps trying to bolt model calls onto them.
- The political dynamic (*who owns the AI work?*) blocks anything that
  requires actual collaboration.

Adaptivity belongs in the same place as the rest of the system.
Substrate, domain, adaptive-surface — those are the real splits.
ML/LLM expertise is a **skill**, distributed across teams, not a team.

### The PM job, redefined

Product management built around tools means: write a PRD that specifies
the screens, the click paths, the validation rules, the empty states.
The PM does the intent-to-decision translation; engineering encodes it.

Product management built around outcomes is different work:

- Specify the **outcome** (what the system promises).
- Specify the **constraints** (what must never happen).
- Specify the **acceptable failure modes** (what counts as a wrong
  answer, and how often is too often).
- Specify the **evaluation criteria** (how we will know it is working).
- *Not* specify the click path. That belongs to the system at runtime.

This is closer to the work of writing a regulation than the work of
writing a wireframe. PMs who learn it become enormously valuable. PMs
who keep writing click paths get politely worked around.

### Risk, legal, and compliance move left

In tools, risk and legal show up at the end: *"before we ship, run this
past legal."* That cadence breaks when the system is making decisions
at runtime. By the time legal sees a screenshot, the system has already
shipped a thousand wrong answers.

Risk-aware orgs move legal and compliance left, into the development
loop:

- Legal reviews **constraints**, not screens. Constraints are written;
  constraints can be reviewed.
- Compliance owns parts of the **eval suite** — the adversarial cases,
  the regulatory must-not-happens.
- Risk reviews **failure modes** before launch, not incidents after.
- Provenance is a compliance-facing product surface, designed jointly
  with legal from day one.

This is uncomfortable for most legal/compliance functions because it
asks them to engage earlier and more technically. The orgs that get it
right ship faster *and* have fewer regulatory incidents — because the
constraints were enforced in the substrate, not in a policy PDF.

### The eval suite is the new test pyramid

Test pyramids assumed a deterministic system: unit tests at the bottom,
integration in the middle, end-to-end on top. Coverage was the headline
metric.

Outcome systems need a different shape:

- **Regression sets** — known cases the system must keep getting right.
- **Calibration sets** — distribution-representative cases used to
  measure quality and cost over time.
- **Adversarial sets** — designed to break the system: edge cases,
  prompt injections, confusing inputs, regulatory traps.
- **Replay tests** — production traces re-run against new versions to
  catch silent regressions.
- **Cost and latency budgets** — run alongside quality, gated together.

A capability without a maintained eval suite is unowned, regardless of
who is listed as owner. Treat eval suites the way mature orgs treat
runbooks: required, reviewed, and load-bearing in incidents.

### Career ladders and hiring filters

Existing ladders reward feature shipping. That maps poorly onto outcome
work, where the most valuable engineers spend months on contracts,
constraints, and evals that ship as silent infrastructure.

Updates worth making:

- Reward **substrate work** explicitly. Promotions cannot require
  visible feature ships if substrate engineers will never have any.
- Hire for **judgment under ambiguity**, not just coding fluency.
  Asking *"what would you constrain, and why?"* separates the
  candidates who get this from those who do not.
- Hire **domain-fluent engineers**. The next decade rewards engineers
  who can sit with a domain expert and translate policy into pack
  structure. That is a different skill set from React or Kubernetes.
- Treat eval engineering as a **promotable specialty**, not a tax.

## 3. How this shapes our roadmap

When deciding what to invest in:

- Build **substrate** that other people can stand on. Contracts, packs,
  provider APIs, evaluation harnesses. This is the moat.
- Ship **worked exemplars** that demonstrate outcomes, not features.
  Atelier exists for this.
- Resist the urge to ship a tool when an outcome is what is wanted. If a
  workflow shows up in the design, ask whether it is a fallback or the
  product.
- Resist the urge to centralize all adaptivity. Push determinism down.
  Push adaptivity up. Keep the middle thin and well-contracted.

## 4. The one-liner, extended

> Tools sold workflows. Systems of outcome sell promises kept under
> constraints. The substrate that makes those promises trustworthy is
> what we are building.
