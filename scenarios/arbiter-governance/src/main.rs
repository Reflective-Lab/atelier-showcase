//! Atelier showcase: consolidated arbiter governance gates.
//!
//! Where `scenario-cedar-smt-analysis` formally verifies a Cedar
//! policy over the entire request space, this scenario shows the
//! everyday runtime governance gates that ride alongside any
//! Converge formation: typed pass/block verdicts emitted in
//! response to actual fact streams.
//!
//! Four gates fire on a single synthetic context:
//!
//! 1. `BudgetGateSuggestor` — sums `CostEstimatePayload` facts and
//!    blocks when the running total exceeds the configured budget.
//! 2. `ApprovalGateSuggestor` — pauses high-stakes proposals
//!    (those carrying an `ApprovalRiskPayload` with confidence ≥
//!    threshold) until a human-approval signal arrives.
//! 3. `DataClassificationGateSuggestor` — scans every proposal's
//!    text for PII patterns (emails, SSNs, credit-card numbers,
//!    phone numbers) and emits a typed verdict per match.
//! 4. `RateLimitGateSuggestor` — caps the count of facts on a
//!    watched key within the run.
//!
//! Each gate emits a typed `*ConstraintPayload` fact under
//! `Constraints` — not a string, not a side-effect, an audit-grade
//! payload that downstream Suggestors can route on. Compare to a
//! naive imperative `if cost > 100` check: the gate carries the
//! exact accumulated total, the threshold, the suggested action
//! (Block vs Pause), and the provenance.
//!
//! After the run, the scenario walks every constraint fact and
//! pretty-prints its typed contents. Independent count assertion
//! confirms each gate fired exactly when expected.

use arbiter::{
    ApprovalConstraintPayload, ApprovalGateSuggestor, ApprovalRiskPayload, BudgetConstraintPayload,
    BudgetGateSuggestor, Confidence, CostEstimatePayload, CostUsd,
    DataClassificationConstraintPayload, DataClassificationGateSuggestor,
    RateLimitConstraintPayload, RateLimitGateSuggestor,
};
use converge_kernel::{Budget as ConvergeBudget, ContextState, Engine, ProposedFact};
use converge_pack::{ContextKey, TextPayload};

const BUDGET_LIMIT: f64 = 2.00;
const APPROVAL_THRESHOLD: f64 = 0.90;
const RATE_LIMIT_MAX: usize = 4;

#[derive(Debug, Clone)]
struct SyntheticProposal {
    id: &'static str,
    cost: f64,
    confidence: f64,
    text: &'static str,
}

const PROPOSALS: &[SyntheticProposal] = &[
    SyntheticProposal {
        id: "proposal-1",
        cost: 0.30,
        confidence: 0.55,
        text: "Outline the rollout phases for the new vendor catalog.",
    },
    SyntheticProposal {
        id: "proposal-2",
        cost: 0.45,
        confidence: 0.70,
        text: "Send the contract draft to legal@reflective.example for review.",
    },
    SyntheticProposal {
        id: "proposal-3",
        cost: 0.60,
        confidence: 0.95,
        text: "Approve the high-stakes wire transfer summary.",
    },
    SyntheticProposal {
        id: "proposal-4",
        cost: 0.50,
        confidence: 0.40,
        text: "Customer SSN 123-45-6789 needs to be redacted from the audit log.",
    },
    SyntheticProposal {
        id: "proposal-5",
        cost: 0.80,
        confidence: 0.60,
        text: "Charge card 4111 1111 1111 1111 for the renewal invoice.",
    },
];

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_banner();
    print_inputs();

    let mut engine = Engine::with_budget(ConvergeBudget {
        max_cycles: 16,
        max_facts: 256,
    });

    // Four governance gates, each watching Strategies (where we'll
    // seed the proposals) and emitting typed verdicts on Constraints.
    engine.register_suggestor(BudgetGateSuggestor::new(
        ContextKey::Strategies,
        BUDGET_LIMIT,
    ));
    engine.register_suggestor(ApprovalGateSuggestor::new(
        ContextKey::Strategies,
        APPROVAL_THRESHOLD,
    ));
    engine.register_suggestor(DataClassificationGateSuggestor::default_patterns(
        ContextKey::Strategies,
    ));
    engine.register_suggestor(RateLimitGateSuggestor::new(
        ContextKey::Strategies,
        RATE_LIMIT_MAX,
    ));

    let mut ctx = ContextState::new();
    for p in PROPOSALS {
        // Cost fact (read by BudgetGateSuggestor).
        ctx.add_proposal(ProposedFact::new(
            ContextKey::Strategies,
            format!("{}-cost", p.id),
            CostEstimatePayload {
                cost: CostUsd::clamped(p.cost),
            },
            "atelier-arbiter-governance",
        ))
        .map_err(|e| format!("seed cost: {e:?}"))?;
        // Approval-risk fact (read by ApprovalGateSuggestor).
        ctx.add_proposal(ProposedFact::new(
            ContextKey::Strategies,
            format!("{}-risk", p.id),
            ApprovalRiskPayload {
                confidence: Confidence::clamped(p.confidence),
            },
            "atelier-arbiter-governance",
        ))
        .map_err(|e| format!("seed risk: {e:?}"))?;
        // Text fact (scanned by DataClassificationGateSuggestor).
        ctx.add_proposal(ProposedFact::new(
            ContextKey::Strategies,
            format!("{}-text", p.id),
            TextPayload::new(p.text),
            "atelier-arbiter-governance",
        ))
        .map_err(|e| format!("seed text: {e:?}"))?;
    }

    let result = engine.run(ctx).await?;
    print_outcome(&result)
}

fn print_banner() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  Arbiter Governance — typed gates on a synthetic fact stream        ║");
    println!("║  atelier-showcase · converge 3.9.1 · arbiter 2.0.1                   ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_inputs() {
    println!("Synthetic proposals");
    println!("───────────────────");
    let mut running_cost = 0.0;
    for (i, p) in PROPOSALS.iter().enumerate() {
        running_cost += p.cost;
        let mut flags: Vec<&str> = Vec::new();
        if p.confidence >= APPROVAL_THRESHOLD {
            flags.push("high-stakes");
        }
        if running_cost > BUDGET_LIMIT {
            flags.push("over-budget");
        }
        if p.text.contains('@') || p.text.contains("SSN") || p.text.contains("4111") {
            flags.push("likely-PII");
        }
        let tag = if flags.is_empty() {
            String::new()
        } else {
            format!("  [{}]", flags.join(", "))
        };
        println!(
            "  {}: cost=${:.2}  conf={:.2}  text={:?}  (cumulative ${:.2}){}",
            p.id, p.cost, p.confidence, p.text, running_cost, tag,
        );
        if i == RATE_LIMIT_MAX - 1 {
            println!(
                "  ── (rate-limit boundary: {} facts in window allowed before block) ──",
                RATE_LIMIT_MAX
            );
        }
    }
    println!();
    println!("Gate thresholds");
    println!("───────────────");
    println!(
        "  budget:    cumulative cost > ${:.2} → Block",
        BUDGET_LIMIT
    );
    println!(
        "  approval:  confidence ≥ {:.2} → Pause (await human signal)",
        APPROVAL_THRESHOLD
    );
    println!(
        "  rate:      > {} facts on Strategies → Block further proposals",
        RATE_LIMIT_MAX
    );
    println!("  pii:       email / SSN / credit-card / phone patterns → typed verdict");
    println!();
}

fn print_outcome(
    result: &converge_kernel::ConvergeResult,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Convergence");
    println!("───────────");
    println!(
        "  converged: {} (cycles: {}, stop: {:?})",
        result.converged, result.cycles, result.stop_reason
    );
    println!(
        "  facts:     Strategies={}  Constraints={}",
        result.context.get(ContextKey::Strategies).len(),
        result.context.get(ContextKey::Constraints).len(),
    );
    println!();

    // Walk every constraint and dispatch by typed family.
    let constraints = result.context.get(ContextKey::Constraints);

    let mut budget_blocks = 0usize;
    let mut approval_blocks = 0usize;
    let mut pii_blocks = 0usize;
    let mut rate_blocks = 0usize;

    println!("Typed verdicts under Constraints");
    println!("────────────────────────────────");
    for fact in constraints.iter() {
        let id = fact.id().as_str();
        if id == "budget-exceeded"
            && let Ok(payload) = fact.require_payload::<BudgetConstraintPayload>()
        {
            budget_blocks += 1;
            println!(
                "  [budget]    {id}  total ${:.2}  >  limit ${:.2}  → action: {:?}",
                payload.total_cost.value(),
                payload.limit.value(),
                payload.action,
            );
            continue;
        }
        if id == "approval-pending"
            && let Ok(payload) = fact.require_payload::<ApprovalConstraintPayload>()
        {
            approval_blocks += 1;
            println!(
                "  [approval]  {id}  status: {:?}  threshold: {:.2}  → action: {:?}",
                payload.status,
                payload.threshold.value(),
                payload.action,
            );
            continue;
        }
        if id == "rate-limit-exceeded"
            && let Ok(payload) = fact.require_payload::<RateLimitConstraintPayload>()
        {
            rate_blocks += 1;
            println!(
                "  [rate]      {id}  count {}  >  limit {}  → action: {:?}",
                payload.count.0, payload.limit.0, payload.action,
            );
            continue;
        }
        if let Ok(payload) = fact.require_payload::<DataClassificationConstraintPayload>() {
            pii_blocks += 1;
            println!(
                "  [pii]       {id}  fact_id: {}  detected: {:?}  → action: {:?}",
                payload.fact_id.as_str(),
                payload.detected_types,
                payload.action,
            );
            continue;
        }
        println!("  [other]     {id}  (untyped) text={:?}", fact.text());
    }
    println!();

    // Expected gate firings.
    // Budget: cumulative cost crosses $2.00 at proposal-5 (0.30+0.45+0.60+0.50+0.80 = 2.65)
    // Approval: proposal-3 has confidence 0.95 ≥ 0.90 → at least one pending.
    // PII: proposals 2 (email), 4 (SSN), 5 (credit card) trigger detections.
    // Rate: 15 facts on Strategies (5 proposals × 3 facts each) > 4 → blocked.
    println!("Verification (count assertions)");
    println!("───────────────────────────────");
    println!(
        "  budget   verdicts: {budget_blocks}  (expected ≥ 1; cumulative cost ${:.2} > ${BUDGET_LIMIT:.2})",
        PROPOSALS.iter().map(|p| p.cost).sum::<f64>()
    );
    println!(
        "  approval verdicts: {approval_blocks}  (expected ≥ 1; proposal-3 conf 0.95 ≥ {APPROVAL_THRESHOLD:.2})"
    );
    println!("  pii      verdicts: {pii_blocks}  (expected ≥ 3; email + SSN + credit-card)");
    println!("  rate     verdicts: {rate_blocks}  (expected ≥ 1; 15 facts > {RATE_LIMIT_MAX})");
    let all_ok = budget_blocks >= 1 && approval_blocks >= 1 && pii_blocks >= 3 && rate_blocks >= 1;
    if !all_ok {
        return Err("at least one gate did not fire as expected".into());
    }
    println!();
    println!(
        "✓ all four arbiter gates fired typed verdicts on the synthetic fact stream;\n  each verdict carries an audit-grade payload (not just a boolean)."
    );

    Ok(())
}
