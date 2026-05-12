# EXP-001: Fuzzy Interpretation for Vendor Selection

## Status

Lifted to applications (2026-05-09).

The original Proposed-state hypothesis, falsification criteria, and non-goals
remain valid as written and are preserved below for reference. The atelier
work paused after producing two design artifacts:

- `fixtures/EXP-001/interviews.json` — 30 synthetic stakeholder interview
  answers across six roles, anchored to the three vendors in
  `scenarios/vendor-selection/`.
- `fixtures/EXP-001/extractor_prompt.md` — model-agnostic LLM extractor prompt
  producing `ExpectationSignal` JSON, including the `applicability` field
  pattern that distinguishes "low signal" from "no signal".

These artifacts remain in place as durable showcase reference. Active
execution moves to two marquee apps:

- **`marquee-apps/quorum-sense/`** picks up the live work. The open M1
  milestone "Fuzzy confidence propagation across hypothesis dimensions" is
  the same problem this experiment was designed to derisk. The applicability
  pattern, slice mapping (`prism::fuzzy` is Mamdani-only by design), and
  falsification framing transfer wholesale. See
  `quorum-sense/kb/fuzzy-confidence-propagation-handoff.md`.
- **`marquee-apps/scout-sourcing/`** — the real-application analogue of the
  vendor-selection scenario used here. The interview corpus and extractor
  prompt are kept available as a template for a future soft-signal
  extension to scout-sourcing's currently structured-data pipeline. No
  active build there. See
  `scout-sourcing/kb/interview-driven-soft-signals.md`.

The atelier experiment is not falsified, not confirmed, and not abandoned —
it has served its derisking purpose and the work is in the right place now.

## Decision Question

Can `prism::fuzzy` and `FuzzyInferencePack` turn loose stakeholder interview
input into a defensible Converge proposal for vendor selection without changing
Prism or the foundation stack?

## Context

The stack already has structured vendor selection in
`scenarios/vendor-selection/`: vendor facts, criterion evaluators, aggregation,
consensus, and policy gates.

That scenario is a good anchor because procurement decisions often have a
pre-structured side and an interpretive side:

- structured facts: price, compliance, risk, delivery history
- soft signals: trust, confidence, perceived implementation fit, stakeholder
  hesitation, expectation gaps

Clustering can group interview themes, and regression can fit outcomes when
there is enough labeled data. This experiment targets the smaller niche where
neither is the main job: a small number of open-text interviews must be turned
into a graded, reviewable decision signal that a domain expert can defend.

## Hypothesis

Given roughly 30-50 open stakeholder interview answers and a small rulebook of
expert-authored linguistic rules, a showcase user can produce a
`FuzzyInferenceOutput`-backed Converge proposal for vendor selection in less
than one working day, without modifying `prism::fuzzy`, `FuzzyInferencePack`, or
Converge.

The value claim is not "fuzzy beats LLMs" or "fuzzy beats analytics". The claim
is narrower:

> fuzzy logic is useful when loose qualitative evidence must become a graded,
> inspectable decision state, and the rulebook itself is part of the artifact
> that needs to be reviewed.

## Non-Goals

- Do not add restaurant, hospitality, or other unrelated domain semantics.
- Do not prove that fuzzy logic is generally better than clustering,
  regression, ranking, or LLM-only workflows.
- Do not add Sugeno, Tsukamoto, ANFIS, type-2 fuzzy logic, or defuzzification
  unless the experiment falsifies the current Mamdani-style slice.
- Do not move domain-specific procurement assumptions into Prism.

## Experiment Shape

Use `scenarios/vendor-selection/` as the concrete anchor, then add a companion
experiment fixture that represents pre-selection stakeholder interviews.

Expected flow:

1. Synthetic interview answers describe stakeholder expectations and concerns
   about candidate vendors.
2. An extractor converts each answer into an `ExpectationSignal` JSON shape.
3. A fuzzy rulebook evaluates signals such as trust, urgency, implementation
   fit, novelty tolerance, and stakeholder hesitation.
4. `FuzzyInferencePack` emits an advisory proposal into the formation.
5. The existing structured vendor-selection flow keeps responsibility for hard
   facts, ranking, and policy gates.
6. The final artifact records activated rules, output memberships, and the
   proposal payload.

## Draft Signal Contract

```json
{
  "vendor_id": "northstar-systems",
  "source": "stakeholder_interview",
  "signals": {
    "trust": 0.72,
    "implementation_fit": 0.64,
    "urgency": 0.81,
    "novelty_tolerance": 0.28,
    "stakeholder_hesitation": 0.58
  },
  "evidence": [
    "The stakeholder expects fast onboarding but worries about handoff quality.",
    "They prefer a familiar operating model over an experimental one."
  ]
}
```

## Draft Fuzzy Rule Intent

The first rulebook should stay small enough for a non-Prism developer to read:

- If trust is high and implementation fit is high, then adoption confidence is
  high.
- If urgency is high and implementation fit is low, then delivery risk is high.
- If stakeholder hesitation is high and trust is medium, then adoption
  confidence is medium.
- If novelty tolerance is low and the vendor approach is highly novel, then
  expectation mismatch is high.
- If trust is low or stakeholder hesitation is high, then escalation need is
  high.

## Falsification Criteria

Any one of these falsifies the drop-in claim:

- The experiment requires a Prism or Converge source change before it can emit
  a useful proposal.
- Rules cannot be authored as data by someone outside `prism::fuzzy` internals.
- `FuzzyInferenceOutput` loses the activated-rule detail needed to explain the
  proposal.
- The pack output cannot be consumed by a normal Converge formation without a
  custom adapter that belongs in Prism.
- The resulting proposal is less inspectable than an LLM-only explanation
  because the rule trace is incomplete or awkward to render.

## Evaluation

Record three results:

- Integration friction: where the implementer had to inspect lower-layer source
  instead of using public docs or schemas.
- Explanation quality: whether a reviewer can see which inputs and rules drove
  the advisory state.
- Decision usefulness: whether the fuzzy proposal adds information that is not
  already captured by the structured vendor facts.

Quantitative scoring is optional for the first run. If the fixture grows beyond
30 examples, add a Brier-style calibration check against hand-labeled graded
targets.

## Expected Outputs

If confirmed:

- a reusable showcase fixture under `scenarios/` or `experiments/fixtures/`
- a small fuzzy rulebook
- example `FuzzyInferenceOutput` payloads
- a short write-up showing where fuzzy adds value alongside ranking, solvers,
  policy, and LLM extraction

If falsified:

- a punch list for the missing consumption surface, such as JSON/YAML rule
  loading, schema docs, proposal payload conventions, or rendering guidance for
  activated rules

## Boundary

The durable capability remains in Prism. The domain-facing worked example lives
in atelier. Applications and engagements can later pull the pattern into a real
business case if the experiment survives contact with the fixture.
