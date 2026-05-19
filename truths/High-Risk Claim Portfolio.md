---
tags: [truth, arbiter, evidence, portfolio]
source: mixed
date: 2026-05-14
---
# High-Risk Claim Portfolio

This Truth-facing portfolio makes clear that
`expense.non_finance_commit.high_value` is the first worked exemplar, not the
whole assurance story.

Each claim should move through the smallest evidence ladder that fits the risk:

```text
business claim
-> model-adequacy review fixtures
-> runtime positive/negative tests
-> property or mutant tests
-> optional Cedar/SymCC conditional query
-> optional real-CVC5 scheduled gate
```

## Portfolio

| Claim | Business Statement | First Evidence | CVC5 Policy | Status |
|---|---|---|---|---|
| `expense.non_finance_commit.high_value` | Non-finance supervisory principals cannot commit high-value expenses even with approval. | review fixtures + runtime + Cedar/SymCC | nightly-only | exemplar |
| `hitl.no_escalation_when_approval_still_denied` | A denied request escalates only when the approved version would be allowed. | property/runtime tests | optional | implemented in Arena |
| `vendor_selection.due_diligence_required` | Vendor commit requires due diligence and competitive review gates. | runtime + review fixtures | optional | implemented in Arena |
| `delegation.amount_cap_enforced` | Delegation tokens cannot authorize spend above their amount cap. | property/negative tests | not useful | portfolio candidate |
| `flow.phase_promotion.requires_gates` | Promotion or commit cannot cross a phase boundary until required gates pass. | runtime/property tests | optional | implemented in Arena |
| `data_classification.pii_blocks_external_move` | PII detected in a proposal creates a blocking constraint before external movement. | runtime fixtures | not useful | implemented in Arena |

## Rule

Do not promote a claim to CVC5 just because it is important. Promote it when a
conditional Cedar/SymCC model can express the business claim accurately and
counterexamples would be useful.
