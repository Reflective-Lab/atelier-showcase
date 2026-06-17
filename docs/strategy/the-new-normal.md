# The New Normal

*Status: stable worldview, v1.3, 2026-06-17. This document captures the operating context Reflective Labs believes the world has already entered. It is platform-agnostic — it describes what has changed in how consequential work happens, regardless of any specific product. The paradigm paper ([[01-platform/reflective-paradigm|The Reflective Paradigm]]) describes how the platform serves this world. This document describes the world itself. The spine of v1.3 is the **inversion of the operating default**: continuous action becomes machine-driven and ambient, and human judgment becomes the sparse, authoritative core that governs it. The convened burst is one shape of that core, not the whole of it. v1.3 adds the systems progression from records, to actions, to outcome commitments.*

## What this is

This document is a worldview. It is not a roadmap, a strategy, a pitch, or a product spec. It is the description of the world we believe we are working in — what has changed about consequential work under AI-assisted collaboration, what has not, and which shifts are real enough to design around.

It is meant to be stable. The specifics in any roadmap will move every quarter; the worldview should not. Where it does move, the change itself is information worth keeping.

We accept the risk that some of what is written here will turn out to be partially wrong. The alternative — waiting until certainty arrives before naming the world we are building for — is worse.

## The human-middleware layer is collapsing

The dominant structure of modern knowledge work has been the human-as-system-middleware. A large share of professional time is spent orienting around systems — knowing the shortcuts, learning the layers other humans invented to abstract previous layers other humans invented, translating between tools, keeping records consistent across platforms, escalating when a workflow stalls, summarising the same content for the next audience.

This work is not stupid or worthless. It was the only available mechanism for scaling coordination across organisations larger than a single team. But it is also not what humans are best at, and it absorbs an enormous fraction of cognitive bandwidth that could be spent on judgment, design, dissent, and ratification.

David Graeber's *Bullshit Jobs* (2018) gives the most-cited language for this failure mode: "box tickers," "duct tapers," "taskmasters" — roles whose function is to keep prior layers of system middleware coherent. The book is a polemic, not a quantitative study; the precise share of knowledge work consumed by this layer is debated. But the failure mode is real and the direction of travel is clear: AI-assisted collaboration compresses the repeatable parts of this work dramatically, sometimes close to zero. What used to take a person three weeks of orientation, translation, and follow-up can now happen in a focused day between a person and an AI.

This is the underlying shift. Everything else in this document follows from it.

## The operating default inverts

Middleware collapse does more than free up time. It inverts which part of consequential work is continuous and which is the exception.

For a century the human was the continuous operator. People ran the processes, held the state, moved work from step to step; automation was the narrow exception that handled the few things rigid enough to encode. The scarce resource was machine capability, and humans filled every gap between.

That ratio is reversing. Continuous operation — research, drafting, monitoring, reconciliation, the moment-to-moment moving of work — is increasingly machine-driven and ambient. What becomes scarce and decisive is human judgment: the framing, the dissent, the constraint, the authority to commit. The human stops being the continuous operator and becomes the **governing core** — sparse, high-leverage, and authoritative over everything the ambient layer does.

This is the forward claim of the new normal, and it is the spine of everything below. It is not "AI helps people work faster." It is that the *default operating mode* of consequential work is becoming **continuous autonomous action governed by a sparse human core.** The design question stops being "how do we make people more productive at running the work" and becomes "how does a small, intermittent human core keep continuous, machine-driven action true to what was actually decided."

This inversion does not weaken the human role; it concentrates its authority. The ambient layer may run most of the hours, but it runs *on the core's terms* — within the scope it was granted, against commitments it cannot silently rewrite, and subject to reopening the moment reality drifts. Consequential commitment remains a human act. What changes is that the human is now the governor of a continuous system rather than one of its moving parts.

## Systems move from records, to actions, to outcome commitments

The same inversion changes what organisations should expect from software.

The twentieth-century enterprise stack was built around **systems of record**: databases, ledgers, ERPs, CRMs, document stores. Their core promise was memory — what happened, who owns it, where it lives.

The SaaS era added **systems of action**: workflows, tickets, approvals, automations, notifications. Their core promise was movement — when something happens in one place, a task or process moves somewhere else.

The AI-assisted era demands **systems of outcome commitments**. Not systems that magically guarantee results, but systems that govern the relationship between a committed outcome and the work supposedly serving it. Their core promise is coherence: what outcome was committed, who had authority to commit it, what assumptions made it plausible, what actions are authorised, what evidence shows drift, and when the commitment must reopen.

This is why "we have a data problem" is usually an incomplete diagnosis. Data quality, integration, lineage, and storage still matter; bad data can poison everything above it. But for most organisations these are substrate problems, not the business breakthrough. The harder failure is that data does not know which commitment it is meant to serve. A dashboard can report activity while the strategy has already drifted. A workflow can complete every task while the outcome has become incoherent. A record can be perfectly clean and still irrelevant.

In the new normal, the business object is not the record and not the action. It is the governed outcome commitment.

## Humans contribute in bursts, not continuously

The most familiar shape of the governing core is the **burst**: a bounded session where the right people convene, decide, and commit. It is where the inversion is easiest to see — the human role does not vanish, it concentrates into discrete contributions to consequential moments rather than continuous attendance to ongoing systems.

The rhythm we are converging on:

- A bounded session in which humans converge — a deal review, an inquiry, a strategy huddle, a hiring committee. The session has soft edges (someone arrives a beat late, someone lingers on a moment that matters to them) but a defined window. We have been calling this a *burst*.
- Decisions made *in* the burst, not deferred to a future review. Coordination cost — knowledge sharing, evidence checks, dissent surfacing, constraint naming, authority clarity — moves forward into the session, where the right people can resolve it together. We have been calling this *frontloading*.
- Between bursts, AI continues the work. Not as a passive waiting state — as the longer, productive phase. AI runs formations, watches telemetry, drafts candidate *options*, screens evidence, prepares the next burst. We have been calling this *ambient*.
- A new burst when the situation calls for it — drift in reality, a question the ambient phase cannot answer, a commitment that has matured to need re-decision.

The shape of the day stops being "open the laptop, start grinding, close the laptop." It becomes "convene for the moments that need me, trust the ambient phase the rest of the time." For knowledge workers this is a profound change in personal experience. For organisations, it changes what management actually does.

## The governing core takes more than one shape

The convened burst is the most legible shape of the governing core, but it is not the only one — and treating it as the whole would repeat the mistake of treating the meeting as the whole of decision-making. Across industries, consequential commitments take at least three shapes, and they make different demands:

- **The convened decision.** One owner, one room, one bounded burst that produces a commitment the owner ratifies. The hiring committee, the strategy huddle, the clinical case conference. This is the shape most management practice already recognises, and the shape most of this document's group-cognition research speaks to.
- **The commitment between parties.** No single owner. Independent organisations — a supply network, a consortium, a joint venture, a standards body — must form a *shared* commitment while each keeps its own private position it will not fully disclose. The governing core is distributed across a trust boundary; the work is reconciling commitments *between* cores, not ratifying one. The familiar burst machinery (anti-HiPPO, hidden-profile extraction) assumes intra-group good faith and does not transfer cleanly here.
- **Standing governance of autonomy.** The commitment is made rarely but *governs continuously*. A policy, an operating doctrine, a strategy that bounds what an ambient system may do for months between human touches. Here the human core almost never convenes; its job is to set the commitment, watch for drift, and reopen. The long operate phase is the product, not the meeting.

These are not three products. They are three situations the world already contains. What unifies them is the inversion: in each, a sparse human core must keep continuous, increasingly autonomous action faithful to a commitment. They differ in *who holds authority*, *whether anyone convenes at all*, and *where the work lives* — and a worldview that sees only the convened burst will under-build for the other two.

## AI as ambient continuation, not replacement

If the human core is sparse, the ambient layer is what runs the rest of the time — and in the inverted default, that is most of the time. AI in this worldview is not a replacement for human judgment; it is the continuous operating layer that the human core governs.

Two legitimate modes:

- **In-burst assistance.** When humans are in the room, AI is a visible participant — it explains, reframes, challenges, summarises, surfaces uniquely-held information, detects drift between what the group is saying and what the evidence supports. Its moves are observable in real time. Participants can pull AI's framing back to interrogate it.
- **Ambient continuation.** Between bursts, AI operates within bounded delegated authority — explicit scope, observable actions, receipts, contestability, human ratification gates for anything consequential. We call this *bounded delegated autonomy*. It is not theatre: any participant can pull a within-scope AI action back into a human burst for re-decision.

What this is not, and must not become:

- A replacement for judgment, dissent, or authority.
- A hidden steering mechanism. Recent research is cautionary — AI facilitation can increase information sharing without improving final decision quality, and can create an "illusion of inclusion" where users feel heard while AI quietly steers the conversation. The corrective is visibility: AI's facilitation moves must be observable to participants in the moment, not only in the audit log. The same constraint governs *ambient* preparation, which is the more insidious risk: when AI gathers evidence or drafts work ahead of a burst, it must bring a *map of options*, not a recommended default, and its pre-work must enter the room labelled as AI-originated and open to challenge. Pre-burst framing that arrives as a settled answer is the most powerful anchoring of all — exactly the pathology the burst exists to neutralise.
- A way to skip the human moments. The collapse of middleware does not extend to ratification. Consequential commitment remains a human act.

The accelerator is real. Working with a capable AI compresses what used to take a person weeks of solo grinding into a focused day of human-and-AI collaboration. This is true at the individual level (we have lived it). The bet of this worldview is that the same compression is available at the group level — that a 30-to-60-minute burst with the right people and an engineered AI participant can replace weeks of meeting cycles, prep documents, alignment emails, and silent assumptions.

## Coordination cost moves forward

The discipline that makes the new rhythm work is *no-postpone*: decisions get made when the right people are convened, not deferred to a future review. If a burst lacks the information to commit, it produces a specific ask and a scheduled reconvene — not a vague "let's revisit."

This is harder than it sounds. The default behaviour of every meeting culture is to defer hard calls. Deferral feels safer than commitment, especially when stakes are high or information is incomplete. The argument for frontloading is that the *cost of indecision rot* — strategy drifting because nobody re-opened it, deals stalling because no one would call, conflicts spreading because they were never named — is far higher than the cost of an imperfect commitment made on time.

This is not a new idea. Lean/Toyota-inspired product development, and later Lean Software Development (Poppendieck, 2003), articulated "decide at the last responsible moment, then commit" as an explicit discipline. Amazon operationalised the same principle in narrative memos, two-pizza teams, and the "disagree and commit" norm — a phrase that predates Amazon, usually traced to Intel's Andy Grove. The Agile Manifesto's original 2001 framing — "individuals and interactions, working software, customer collaboration, response to change" — also pointed toward decision agility, even if the Scrum ceremony that grew up around it has rotted into the opposite.

What changes in the new normal is that frontloading becomes *enforceable by tooling*, not only by culture. The burst itself is engineered to make decision easy and visible at its end. The ambient phase has two faces: **commit-bound ambient execution** — work that depends on the canonical commitment being recorded — cannot start until that record exists; **preparatory ambient work** (research, evidence gathering, candidate drafting in advance of a scheduled burst) can start from an explicit ask, since pre-burst preparation is part of the value the rhythm delivers. The discipline that used to depend on a strong facilitator now lives in the platform's session shape.

## Group cognition is engineered, not improvised

A persistent fiction in management literature is that good groups make good decisions. The research is much more sobering. Pure human groups, even with the right people in the room, exhibit well-documented pathologies:

- **Groupthink** (Janis, 1972): cohesive groups suppress dissent and converge prematurely on inferior decisions.
- **HiPPO effect**: the highest-paid person's opinion anchors the group regardless of evidence.
- **Hidden profile** (Stasser & Titus, 1985 and successors): groups disproportionately discuss information everyone already shares, while uniquely-held information from individual members fails to surface — even when surfacing it would change the decision.
- **Noise** (Kahneman, Sibony, Sunstein, 2021): human judgment is far noisier than people believe, even with the right people in the room.

What works — when it works — is structured group process. Delbecq & Van de Ven's Nominal Group Technique (1971) prescribes silent generation, round-robin sharing, structured discussion, then vote — explicitly to neutralise HiPPO and surface hidden information. Liberating Structures (Lipmanowicz & McCandless, 2014) codifies a family of similar moves: 1-2-4-All, TRIZ, Discovery & Action Dialogue.

Anita Woolley and colleagues, in their *Science* (2010) paper on collective intelligence, found that the consistent predictors of group performance were *social sensitivity* and *balanced participation* — not the individual IQ of the smartest person in the room. Replications have been mixed, so the finding deserves directional weight rather than law-like authority, but the implication holds: a room engineered to surface dispersed knowledge and balance voice outperforms a room of high-IQ individuals left to default group dynamics.

The new normal does not assume good groups happen. It assumes well-engineered groups outperform default groups, and that the engineering — anti-HiPPO process, hidden-profile prompting, structured turn-taking, anonymous confidence capture, designated dissent, visible AI facilitation — is platform-level, not facilitator-skill-level.

Edmondson's psychological safety research is the necessary ground for any of this. People do not surface dissent in rooms where dissent is punished. But psychological safety alone is not sufficient — safe rooms can still drift into consensus theatre. Safety plus frontloaded standards plus engineered process is the combination that converts dispersed knowledge into traceable commitment.

## Truth is stable; explanation is personal

A consequential commitment is not finished when it is recorded. It only matters when it becomes shared operating reality across the roles it touches. This second half — making the decision land as the same operating truth for engineering, legal, HR, sales, finance, the board, the regulator, the affected employee — is where most strategic work currently fails.

A strategy fails twice. First, it can be wrong on its merits. Second, even when it is right, it is translated badly. Leadership hears aspiration; engineering hears extra work; legal hears exposure; HR hears reorg risk; sales hears a quota story. Each role builds a private working theory of what was decided. None of them is wrong on its own face, and none of them is the canonical commitment. The organisation drifts — not because people are disloyal, but because the commitment never became role-specific meaning.

The corrective is not "send a clearer memo." It is the recognition that a stable canonical truth and a personalised explanation are not in conflict — they are the natural shape of how consequential commitments propagate. Star and Griesemer's 1989 work on *boundary objects* names this exactly: artifacts that are stable enough for shared reference across communities, plastic enough for local meaning. A strategy is a boundary object. A clinical guideline is a boundary object. A regulatory rule is a boundary object. The work is making each role's local reading provably traceable to the same canonical core, while preventing local readings from quietly drifting into something that contradicts the core.

Karl Weick's *Sensemaking in Organizations* (1995) provides the theoretical ground for why this matters: organisations are held together by ongoing acts of sensemaking, not by their formal structures. When sensemaking fragments across roles, the organisation fragments with it, regardless of what the org chart says. Edgar Schein's culture work points to the same dynamic at a deeper layer — culture is the shared assumptions about what is real and what matters, and Lost in Translation is the culture failure mode at role boundaries.

The new normal assumes this is solvable — that a tooled, governed approach to translation between canonical truth and role-specific reality can be designed, measured, and held to account.

## What this means for organisations

Three claims about the structures organisations have inherited from the twentieth century.

**Hierarchical authority structures are no longer sufficient as the cognition system of the company.** They are not dead. They persist because they solve real problems: regulatory accountability needs a named owner per decision; principal-agent problems at scale need supervision chains; legal liability needs hierarchy. In regulated industries — banking, pharma, defence, government — they will dominate for decades. What has changed is that hierarchy is no longer where the actual sense-making happens. The cognitive work has moved elsewhere — usually to a mix of shadow networks, ad-hoc collaborations, and AI-assisted individual work — and the hierarchy has become primarily a ratification and accountability surface. The new normal accepts hierarchy as a governance system and replaces it as a cognition system. Max Weber (*Economy and Society*, 1922) named the bureaucratic form; it remains useful in its original purpose.

**Matrix organisations are troubled, not dead.** Most large technology companies still run a matrix structure under different names — Spotify's tribes-and-chapters, Amazon's two-pizza-teams plus functional spines, the standard product/function cross at any company over a few hundred people. The original case for matrix (Jay Galbraith, *Designing Complex Organizations*, 1973) was about sharing scarce specialists across products; that problem has not gone away. What has changed is the cost of the dual-reporting tax. When decisions need to happen weekly, the "consult both bosses" penalty was tolerable. When decisions need to happen in a frontloaded burst, the penalty becomes intolerable. Matrix structures will continue to zombify back into existence even where companies declare they have killed them — but they will run with increasing pain at the speeds the new normal requires.

**Agile-as-ceremony is exhausted; agile-as-principles is alive.** The Agile Manifesto's original 2001 framing — individuals and interactions over processes and tools, working software over documentation, customer collaboration over contracts, response to change over plans — remains correct. What has rotted is the Scrum-ceremony industrial complex that grew up around it: ritual standups, story-pointing theatre, retrospectives that loop without learning, sprint reviews where nobody decides anything. Steve Denning's *The Age of Agile* (2018) names the difference between "being agile" and "doing agile"; Bain and McKinsey research suggests most enterprise agile transformations fail to deliver the productivity gains promised. The replacement is not anti-agile. It is *frontloaded decision agility* — the same Manifesto principles powered by burst-and-ambient session shape and engineered group cognition, instead of by ceremony.

The pattern across all three: not death, but insufficiency. The structures we inherited still solve problems, but they are no longer adequate as the cognitive infrastructure of consequential work. The new normal puts a different infrastructure on top.

## What it means for individuals

The personal experience of work shifts. We can describe it concretely from the user-of-AI perspective most of us have already lived:

- **You are needed discretely, not continuously.** Long stretches of uninterrupted focus are valuable again; the hours between bursts are not "absence from work" but the structural shape of the new rhythm.
- **Your time, when convened, is treated as expensive.** Bursts are bounded, prepared, and have explicit purpose. You arrive knowing why you are present, what the expected outcome is, and what authority lives in the room.
- **Your contribution is visible and protected.** Engineered group cognition means your unique knowledge has a route into the room that does not depend on you out-shouting the loudest voice. Anonymous confidence capture, silent generation, and round-robin sharing are platform features, not facilitator skills.
- **You can dissent without penalty.** Psychological safety is necessary; standards and frontloading make it productive. Your dissent enters the canonical record.
- **You see the outcome.** Clayton Christensen and Tony Ulwick's lineage of *Jobs to be Done* / outcome-driven thinking returns to its original meaning: you are paid to deliver value, not to attend meetings or write status emails. The platform's structure makes the outcome legible, which makes the work legible.
- **You read the projection that fits you.** A strategy from leadership arrives with role-specific consequences for your team, not as a leadership memo you have to interpret on your own.

The shadow side is real and should not be glossed: the collapse of middleware reduces the number of roles in which "showing up reliably" was a sufficient career. The new normal rewards judgment, dissent, framing, and ratification — which not every existing role naturally maps to. The honest claim is not that everyone wins; it is that the work that survives is closer to what humans are actually good at.

## What we don't yet know

This document commits to a worldview, but several large questions remain open:

- **How fast organisations actually adopt.** The technology shift is real; the cultural shift is slower. The five-to-fifteen year horizon is the realistic range. Inside that range, prediction is hard.
- **Scale of the burst.** A six-person decision burst and a sixty-person decision burst are different products with different mechanisms. The structured-cognition moves that neutralise HiPPO in a small room (silent generation, round-robin) do not survive at scale; parallel small rooms with synthesis is the more probable shape for larger groups. We do not yet know how cleanly that scales.
- **Regulatory adaptation.** Bounded delegated autonomy for AI between bursts has obvious legal questions in regulated industries. The frameworks we will need do not yet exist.
- **Measurement.** We claim that frontloaded bursts compress weeks of coordination into hours. The evidence is currently anecdotal. Organisational adoption requires measurement that does not yet exist.
- **The middleware-collapse range.** We claim a "large share" of knowledge work is human middleware. The precise share is unknown; estimates from Graeber's qualitative case to recent McKinsey AI-augmentation studies vary by an order of magnitude.
- **The synchronisation cost of frontloading.** Bursts are synchronous. Part of what the middleware layer bought was *asynchronous* progress when the right people were unavailable. If consequential coordination concentrates into synchronous bursts, organisational velocity can become bottlenecked by the calendars of its scarcest people. Whether frontloading nets out positive depends on burst frequency, duration, and how much ambient work genuinely substitutes for synchronous time.
- **The translation-fidelity ceiling.** This worldview assumes role-specific translation of a canonical truth can be done faithfully and held to account. It is not yet known whether AI can detect meaning-level drift across contextual, politically charged role boundaries at an acceptable false-positive rate. If the error rate is too high, governed translation degrades into human review with AI assistance — still useful, but a weaker claim than the one made here.
- **Adversarial gaming of the process.** Engineered group cognition and an explicit integrity rubric are both gameable: a canonical commitment can be written vaguely enough that no projection technically distorts it, and structured process can be performed without being honoured. This worldview assumes good-faith use; sustained adversarial use — actors optimising to defeat the record — is an open question.
- **Whether the non-convened shapes are governable at all.** Most of the design intuition here, and nearly all of the cited group-cognition research, addresses the *convened* shape — one owner, one room. The commitment-between-parties shape (distributed authority across a trust boundary, deliberate non-disclosure) and the standing-governance-of-autonomy shape (the operate phase as the product, almost no convening) are far less charted. It is not yet known how much of the convened-burst machinery transfers, or what genuinely new mechanisms each demands.

Where these uncertainties resolve, this document should be updated.

## Theoretical anchors

The worldview rests on four primary research lineages:

- **Boundary objects** (Star & Griesemer, *Social Studies of Science*, 1989). Stable enough for shared reference, plastic enough for local meaning. The closest existing academic framing for the canonical truth / role projection split.
  <https://en.wikipedia.org/wiki/Boundary_object>
- **Sensemaking** (Karl Weick, *Sensemaking in Organizations*, 1995). Organisations are held together by ongoing sensemaking, not formal structure. When sensemaking fragments at role boundaries, the organisation fragments.
  <https://en.wikipedia.org/wiki/Sensemaking>
- **Organisational culture** (Edgar Schein). Culture as shared assumptions about what is real. Lost in Translation is the culture failure mode at role boundaries.
  <https://en.wikipedia.org/wiki/Edgar_Schein>
- **Cynefin** (Dave Snowden). Distinguishes *complicated* (expertise sufficient) from *complex* (sensemaking required). Consequential collective-reasoning is complex-domain work; the new normal is built for that domain.

Supporting empirical and conceptual sources:

- **Hidden profile / shared-information bias** (Stasser & Titus, 1985 and successors). <https://en.wikipedia.org/wiki/Hidden_profile>
- **Collective intelligence (c-factor)** (Woolley et al., *Science*, 2010). Directional finding; replications mixed.
- **Groupthink** (Janis, 1972).
- **Noise** (Kahneman, Sibony, Sunstein, 2021).
- **Nominal Group Technique** (Delbecq & Van de Ven, 1971). <https://en.wikipedia.org/wiki/Nominal_group_technique>
- **Liberating Structures** (Lipmanowicz & McCandless, 2014).
- **Psychological safety** (Edmondson). Necessary but not sufficient without high-standards frontloading. <https://en.wikipedia.org/wiki/Psychological_safety>
- **Agile Manifesto** (2001). The principles, distinct from the rotted ceremony. <https://agilemanifesto.org/>
- **Jobs to be Done / outcome-driven innovation** (Tony Ulwick predates and is credited by Clayton Christensen; Christensen, *Competing Against Luck*, 2016).
- **Lean Software Development** (Poppendieck, 2003) on the "last responsible moment" / commitment discipline; the broader Lean tradition (Womack & Jones; Liker) on the Toyota Production System principles that informed it.
- **Bullshit Jobs** (Graeber, 2018). Most-cited language for the human-middleware failure mode. The claim is qualitative.
- **The Age of Agile** (Denning, 2018). The "being agile vs doing agile" distinction.
- **LLM facilitation effects** (recent arXiv work on increased sharing without improved decision quality, and on AI steering plus illusion of inclusion). Read as a cautionary baseline for AI-in-burst design.

## Canonical Links

- [[01-platform/reflective-paradigm|The Reflective Paradigm]] — how the platform serves this world
- [[category-one-pager|The One-Page Story]] — single-page category claim
- [[01-platform/README|Platform Vision & Operating Model]]
- [[stack-narrative|The Reflective Labs Story]]
- [[business-pitch|Business Pitch]]
- [[investor-pitch|Investor Pitch]]
- [[glossary|Glossary]]
