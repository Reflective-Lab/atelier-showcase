# EXP-001 — LLM Extractor Prompt

Model-agnostic prompt that converts a single stakeholder interview answer into an `ExpectationSignal` JSON object suitable as crisp input to `prism::fuzzy::FuzzyInferenceEngine` (via `FuzzyInferencePack`).

The extractor is the *interpretive* boundary. Everything downstream (rule firing, defuzzification policy, proposal rendering) operates on the signals this prompt emits. Calibration here is part of the experiment's evaluation surface — instability in the extractor will surface as instability in the fuzzy proposal even when the rules are stable.

## Output Schema

```json
{
  "vendor_id": "vendor-a | vendor-b | vendor-c | null",
  "source": "stakeholder_interview",
  "stakeholder_id": "sh-cfo | sh-cto | sh-ops | sh-pm | sh-sec | sh-user",
  "interview_id": "int-NNN",
  "signals": {
    "trust": 0.0,
    "implementation_fit": 0.0,
    "urgency": 0.0,
    "novelty_tolerance": 0.0,
    "stakeholder_hesitation": 0.0
  },
  "evidence": ["verbatim quote substring", "..."],
  "applicability": {
    "trust": 0.0,
    "implementation_fit": 0.0,
    "urgency": 0.0,
    "novelty_tolerance": 0.0,
    "stakeholder_hesitation": 0.0
  },
  "notes": "optional one-line interpreter note"
}
```

`signals` are crisp values in `[0.0, 1.0]`. `applicability` is a parallel `[0.0, 1.0]` field that says *how strongly the answer carries information about that signal at all* — distinct from the signal's value. An answer that doesn't touch a dimension gets `applicability: 0.0` for it; the rule engine should weight or skip accordingly.

## Prompt Template

```
ROLE
You are an expert interpreter of stakeholder interview transcripts in a vendor-selection context.
Your job is to read ONE interview answer and emit a structured signal record.
You do NOT make recommendations. You do NOT summarize. You extract calibrated signals.

CONTEXT
Three vendors are under review:
- vendor-a (Acme Corp): incumbent, established, mid-price, mid-speed, compliant.
- vendor-b (Beta Solutions): premium-priced, fastest delivery, compliant, polished sales motion.
- vendor-c (Gamma Industries): low-price, fastest-rising, currently non-compliant on one certification, smallest team.

The five signals you must score (each in [0.0, 1.0]):

1. trust — how strongly the answer expresses *earned* belief in the vendor (or, if vendor_id is null, in the decision-making process). High = unprompted credibility, evidence cited. Low = sales-polish without substance, vague answers, prior burns.

2. implementation_fit — how well the vendor's actual operating model is expected to mesh with this organization. High = the vendor's defaults match the org's defaults; Low = prescriptive vendor in flexible org, or vice versa.

3. urgency — the temporal pressure the stakeholder feels. High = "we cannot wait"; Low = "we'd survive a slip"; report what THIS stakeholder feels, not vendor delivery time.

4. novelty_tolerance — the stakeholder's appetite for unfamiliar approaches. High = enthusiastic about new architectures, young vendors; Low = recent burns, prefers boring wins.

5. stakeholder_hesitation — how much THIS stakeholder is themselves hesitating in their reasoning. High = ambivalent, hedged, multiple reversals in the answer; Low = clear directional view. Distinct from urgency and from trust.

CALIBRATION ANCHORS (use these to avoid clustering around 0.5)
- 0.0–0.2: the answer is strongly negative / explicitly stated to be very low on this dimension.
- 0.3–0.4: more negative than positive, but with caveats.
- 0.5: genuinely mixed or balanced — use sparingly. If you reach for 0.5, ask whether applicability is actually low instead.
- 0.6–0.7: more positive than negative, but with caveats.
- 0.8–1.0: the answer is strongly positive / explicitly stated to be very high on this dimension.

If the answer does not address a dimension at all:
- Set applicability for that dimension to 0.0–0.2.
- Set the signal to 0.5 as a non-informative placeholder.
The downstream rule engine treats applicability < 0.3 as "do not fire rules on this dimension".

EVIDENCE RULE
For every signal you score with applicability >= 0.3, include at least one verbatim quoted substring from the answer in the `evidence` array. Quotes must be substrings of the input — no paraphrase. Aim for the shortest substring that carries the signal.

DISAMBIGUATION RULES
- vendor_id: only set if the stakeholder is clearly speaking about that specific vendor. Cross-vendor or general answers get vendor_id: null.
- stakeholder_hesitation refers to THIS stakeholder's own ambivalence, not their report of someone else's hesitation.
- Do not infer signals from biographical context (role, tenure). Score only what's in the answer.
- Sarcasm and understatement count: "I don't hate it. I don't love it. It's there." is low trust and low fit, not neutral.

INPUT
You will receive a JSON object with: stakeholder (id, role, stance), question, answer, vendor_focus.

OUTPUT
Emit exactly one JSON object matching the schema. No prose before or after.
```

## Worked Example

### Input

```json
{
  "interview_id": "int-001",
  "stakeholder": {
    "id": "sh-cfo",
    "role": "Chief Financial Officer",
    "stance": "cost-conscious, slightly conservative"
  },
  "question": "What's your read on Acme as the incumbent contender?",
  "vendor_focus": "vendor-a",
  "answer": "Acme is the safe pick on paper. Fifteen years, compliant, the price is fine. But honestly — and I've said this in steering — every time we've worked with them in the past two years they've shown up like they already knew what we needed. They didn't. Their reps are pleasant. Their actual responsiveness when something gets hard is… mid. I'd pick them again if there were no other option, but I'm not enthusiastic. I think we're a B-tier account to them now."
}
```

### Expected Output

```json
{
  "vendor_id": "vendor-a",
  "source": "stakeholder_interview",
  "stakeholder_id": "sh-cfo",
  "interview_id": "int-001",
  "signals": {
    "trust": 0.35,
    "implementation_fit": 0.40,
    "urgency": 0.5,
    "novelty_tolerance": 0.5,
    "stakeholder_hesitation": 0.55
  },
  "evidence": [
    "shown up like they already knew what we needed. They didn't",
    "Their actual responsiveness when something gets hard is… mid",
    "we're a B-tier account to them now",
    "I'd pick them again if there were no other option, but I'm not enthusiastic"
  ],
  "applicability": {
    "trust": 0.85,
    "implementation_fit": 0.65,
    "urgency": 0.0,
    "novelty_tolerance": 0.0,
    "stakeholder_hesitation": 0.55
  },
  "notes": "Trust is eroded by repeated under-delivery. Stakeholder is somewhat hesitant — clear view of weakness, but still leaning toward picking Acme."
}
```

### Reading

- `trust = 0.35` — multiple negative observations ("didn't know what we needed", "mid responsiveness", "B-tier account"). High applicability (0.85) because the answer is largely about trust.
- `implementation_fit = 0.40` — the "shown up like they already knew" passage carries fit signal (vendor's default behavior misaligned with org's needs). Medium applicability.
- `urgency` and `novelty_tolerance` are not addressed → applicability 0.0, signal placeholder 0.5.
- `stakeholder_hesitation = 0.55` — speaker is directionally negative but qualifies ("if there were no other option"), indicating real ambivalence rather than a clean stance.

## Operational Notes

- **Stability check:** run the same interview through the prompt three times. If `signals` drift by more than `±0.10` on dimensions with applicability `>= 0.5`, the extractor is too noisy for the experiment — record this as a falsification trigger #5 ("less inspectable than LLM-only") candidate.
- **Vendor disambiguation failure mode:** if the stakeholder discusses multiple vendors in one answer, prefer to *split the answer at sentence boundaries* upstream rather than asking the extractor to multi-emit. The current schema is one-record-per-call.
- **Authorship constraint (per EXP-001 falsification #2):** this prompt should be authorable and revisable by a domain expert (e.g. a Kantar consultant) without touching Rust. The prompt lives as Markdown for that reason. If during the experiment we find we need to edit Rust to change extraction behavior, that's a logged finding.

## Where this gets consumed

The extracted signal records become the crisp input to `FuzzyInferenceEngine::solve(...)`, with `applicability` used by the surrounding glue (not by `prism::fuzzy` itself) to decide whether to feed a signal into the engine at all. The fuzzy rulebook (next artifact) operates only on signals where `applicability >= 0.3`.
