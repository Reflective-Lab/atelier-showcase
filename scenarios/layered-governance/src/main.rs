//! Atelier showcase: layered governance — rule-based gate +
//! Cedar policy gate + (mock) anomaly detector firing on the
//! same document in a single Converge run.
//!
//! ## Why this scenario exists
//!
//! `scenario-arbiter-governance` shows four runtime gates (budget,
//! approval, PII, rate-limit) on a fact stream.
//! `scenario-arbiter-compliance` shows the rule-based compliance
//! gate in isolation. `scenario-cedar-smt-analysis` shows the
//! Cedar SMT verification path.
//!
//! This scenario demonstrates the **composition** described in
//! the kb page `rule-based-compliance-in-practice.md` and in
//! `warden-compliance/kb/surface.md`: production governance
//! stacks run multiple gates of different *kinds* on the same
//! object, with each emitting its own typed verdict on the
//! shared context. The shape is the same Suggestor /
//! ContextKey contract — three Suggestors registered into one
//! Engine, each reading the same Strategies fact stream,
//! each producing typed Constraints verdicts of different
//! payload families.
//!
//! Three gates, one document:
//!
//! - **RuleGate** (`arbiter::ComplianceGateSuggestor`) — per-field
//!   rule check. Cheap, deterministic, O(rules) per fact.
//! - **PolicyGate** (`arbiter::PolicyGateSuggestor`) — Cedar
//!   authorization decision. Also fast but evaluates against an
//!   authorization-shaped principal/resource/context triple.
//! - **AnomalyGate** (scenario-local) — stand-in for an ML
//!   anomaly detector. Reads the same fact stream; emits a
//!   typed `AnomalyConstraintPayload`. Production replacement
//!   would be a statistical / ML model; here it's a hard-coded
//!   threshold so the trace stays deterministic.
//!
//! After the run the scenario walks `Constraints` and groups
//! verdicts by gate, demonstrating that the layered composition
//! produces a single audit-grade record stream where each gate's
//! verdict carries its own type.

use std::sync::Arc;

use arbiter::{
    ApprovalConstraintPayload, ApprovalGateSuggestor, ApprovalRiskPayload, ComplianceCondition,
    ComplianceConstraintPayload, ComplianceDocumentPayload, ComplianceGateSuggestor,
    ComplianceRule, Confidence,
};
use async_trait::async_trait;
use converge_kernel::Provenance;
use converge_kernel::{AgentEffect, Budget, Context, ContextState, Engine, ProposedFact};
use converge_pack::{ContextKey, FactPayload, ProvenanceSource, Suggestor};
use serde::{Deserialize, Serialize};

/// The document under scrutiny — same payload, three gates examine
/// it through three different lenses.
fn build_document() -> ComplianceDocumentPayload {
    let mut fields = serde_json::Map::new();
    fields.insert(
        "data_region".to_string(),
        serde_json::Value::String("us-east-1".to_string()),
    );
    fields.insert(
        "retention_days".to_string(),
        serde_json::Value::Number(3000.into()),
    );
    fields.insert(
        "anomaly_score".to_string(),
        serde_json::Value::Number(
            serde_json::Number::from_f64(0.94).expect("anomaly score is finite"),
        ),
    );
    ComplianceDocumentPayload { fields }
}

const DOC_ID: &str = "transfer-2026-05-19-001";

#[derive(Clone, Copy, Debug)]
struct ScenarioProvenance;

impl ProvenanceSource for ScenarioProvenance {
    fn as_str(&self) -> &'static str {
        "atelier-layered-governance"
    }
}

fn scenario_provenance() -> Provenance {
    ScenarioProvenance.provenance()
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_banner();
    print_document();

    let mut engine = Engine::with_budget(Budget {
        max_cycles: 16,
        max_facts: 256,
    });

    // Gate 1: rule-based compliance.
    engine.register_suggestor(ComplianceGateSuggestor::new(
        ContextKey::Strategies,
        build_compliance_rules(),
    ));

    // Gate 2: a stand-in for a Cedar PolicyGate. Production would
    // construct an arbiter::PolicyEngine from a Cedar source and
    // emit ApprovalConstraintPayload through PolicyGateSuggestor.
    // Here we use ApprovalGateSuggestor with a synthetic
    // ApprovalRiskPayload to keep the wiring deterministic and
    // independent of Cedar policy text.
    engine.register_suggestor(ApprovalGateSuggestor::new(
        ContextKey::Strategies,
        0.85, // anomaly score threshold for human review
    ));

    // Gate 3: scenario-local mock anomaly detector.
    engine.register_suggestor(AnomalyGate::new(0.90));

    let mut ctx = ContextState::new();
    let doc = build_document();
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Strategies,
        DOC_ID,
        doc.clone(),
        scenario_provenance(),
    ))?;
    // Approval gate reads ApprovalRiskPayload, not the document.
    // Emit one alongside the doc so the gate has something typed
    // to react to.
    let anomaly_score = doc
        .fields
        .get("anomaly_score")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Strategies,
        format!("{DOC_ID}-risk"),
        ApprovalRiskPayload {
            confidence: Confidence::clamped(anomaly_score),
        },
        scenario_provenance(),
    ))?;

    let result = engine.run(ctx).await?;
    print_outcome(&result)
}

fn build_compliance_rules() -> Vec<ComplianceRule> {
    vec![
        ComplianceRule {
            id: "gdpr-data-residency".into(),
            framework: "GDPR".into(),
            field: "data_region".into(),
            condition: ComplianceCondition::MustNotContain(vec![
                "us-east".into(),
                "us-west".into(),
            ]),
        },
        ComplianceRule {
            id: "soc2-retention-max".into(),
            framework: "SOC2".into(),
            field: "retention_days".into(),
            condition: ComplianceCondition::MaxValue(2555.0),
        },
    ]
}

// ---------------------------------------------------------------------------
// Mock anomaly detector — emits an AnomalyConstraintPayload when
// the document's anomaly_score exceeds threshold. Production
// replacement is a statistical / ML model that reads the same
// ComplianceDocumentPayload and emits the same typed verdict
// shape.

#[derive(Debug, Clone, Copy)]
struct AtelierLayeredProvenance;
impl ProvenanceSource for AtelierLayeredProvenance {
    fn as_str(&self) -> &'static str {
        "atelier-layered-governance"
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnomalyConstraintPayload {
    pub fact_id: String,
    pub field: String,
    pub score: f64,
    pub threshold: f64,
    pub action: String,
}

impl FactPayload for AnomalyConstraintPayload {
    const FAMILY: &'static str = "atelier.layered_governance.constraint.anomaly";
    const VERSION: u16 = 1;
}

struct AnomalyGate {
    threshold: f64,
}

impl AnomalyGate {
    fn new(threshold: f64) -> Self {
        Self { threshold }
    }
}

#[async_trait]
impl Suggestor for AnomalyGate {
    fn name(&self) -> &'static str {
        "atelier-anomaly-gate"
    }
    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Strategies]
    }
    fn provenance(&self) -> Provenance {
        AtelierLayeredProvenance.provenance()
    }
    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.has(ContextKey::Strategies)
            && !ctx
                .get(ContextKey::Constraints)
                .iter()
                .any(|f| f.id().as_str().starts_with("anomaly-"))
    }
    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let mut effect = AgentEffect::builder();
        for fact in ctx.get(ContextKey::Strategies).iter() {
            let Ok(doc) = fact.require_payload::<ComplianceDocumentPayload>() else {
                continue;
            };
            let score = doc
                .fields
                .get("anomaly_score")
                .and_then(serde_json::Value::as_f64);
            let Some(score) = score else {
                continue;
            };
            if score >= self.threshold {
                effect = effect.proposal(AtelierLayeredProvenance.proposed_fact(
                    ContextKey::Constraints,
                    format!("anomaly-{}", fact.id().as_str()),
                    AnomalyConstraintPayload {
                        fact_id: fact.id().as_str().to_string(),
                        field: "anomaly_score".to_string(),
                        score,
                        threshold: self.threshold,
                        action: "Pause".to_string(),
                    },
                ));
            }
        }
        effect.build()
    }
}

// ---------------------------------------------------------------------------

fn print_banner() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  Layered Governance — rule + (Cedar-like) policy + anomaly gates    ║");
    println!("║  atelier-showcase · arbiter 2.0.1 · converge 3.9.1                   ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_document() {
    println!("Document under scrutiny");
    println!("───────────────────────");
    println!("  id:              {DOC_ID}");
    let doc = build_document();
    for (k, v) in &doc.fields {
        println!("  {k:<20}  {v}");
    }
    println!();
    println!("Three gates registered against the same Strategies stream:");
    println!("  - RuleGate       (arbiter::ComplianceGateSuggestor)");
    println!("  - ApprovalGate   (arbiter::ApprovalGateSuggestor, Cedar-shaped stand-in)");
    println!("  - AnomalyGate    (scenario-local, ML-replacement stand-in)");
    println!();
    println!("Each gate emits its own typed Constraints payload family.");
    println!();
}

fn print_outcome(
    result: &converge_kernel::ConvergeResult,
) -> Result<(), Box<dyn std::error::Error>> {
    let constraints = result.context.get(ContextKey::Constraints);

    println!("Convergence");
    println!("───────────");
    println!(
        "  converged: {} (cycles: {}, stop: {:?})",
        result.converged, result.cycles, result.stop_reason
    );
    println!("  facts:     Constraints = {}", constraints.len());
    println!();

    let mut by_gate: std::collections::BTreeMap<&'static str, Vec<String>> =
        std::collections::BTreeMap::new();

    for fact in constraints.iter() {
        let id = fact.id().as_str();
        let summary = if let Ok(c) = fact.require_payload::<ComplianceConstraintPayload>() {
            by_gate.entry("RuleGate").or_default().push(format!(
                "[{:<5}] {}  rule={:<22}  field={}  action={:?}",
                c.framework, id, c.rule_id, c.field, c.action
            ));
            continue;
        } else if let Ok(a) = fact.require_payload::<ApprovalConstraintPayload>() {
            format!(
                "approval status={:?} threshold={:.2} action={:?}",
                a.status,
                a.threshold.value(),
                a.action
            )
        } else if let Ok(a) = fact.require_payload::<AnomalyConstraintPayload>() {
            format!(
                "anomaly score={:.2} threshold={:.2} field={} action={}",
                a.score, a.threshold, a.field, a.action
            )
        } else {
            // Pass through with raw text, helps when a payload schema is unknown.
            format!("(unknown payload) text={}", fact.text().unwrap_or("(none)"))
        };
        let gate = if id.starts_with("anomaly-") {
            "AnomalyGate"
        } else if id == "approval-pending" {
            "ApprovalGate"
        } else {
            "<other>"
        };
        by_gate
            .entry(gate)
            .or_default()
            .push(format!("{id}  {summary}"));
    }

    println!("Verdicts grouped by gate");
    println!("────────────────────────");
    for (gate, entries) in &by_gate {
        println!("  {gate}");
        for e in entries {
            println!("    {e}");
        }
        println!();
    }

    let saw_rule = by_gate.get("RuleGate").is_some_and(|v| !v.is_empty());
    let saw_approval = by_gate.get("ApprovalGate").is_some_and(|v| !v.is_empty());
    let saw_anomaly = by_gate.get("AnomalyGate").is_some_and(|v| !v.is_empty());

    println!("Verification");
    println!("────────────");
    println!(
        "  RuleGate    fired: {}",
        if saw_rule {
            "✓"
        } else {
            "✗ expected at least one GDPR/SOC2 verdict"
        }
    );
    println!(
        "  ApprovalGate fired: {}",
        if saw_approval {
            "✓"
        } else {
            "✗ expected pending review on high-risk doc"
        }
    );
    println!(
        "  AnomalyGate  fired: {}",
        if saw_anomaly {
            "✓"
        } else {
            "✗ expected anomaly threshold breach"
        }
    );
    if !saw_rule || !saw_approval || !saw_anomaly {
        return Err("at least one layered gate did not produce a verdict".into());
    }
    println!();
    println!(
        "✓ Three gates ran against one document on one Strategies stream;\n  \
         each emitted its own typed Constraints payload family (Compliance, Approval, Anomaly).\n  \
         The verdict log is the audit-grade record described in\n  \
         marquee-apps/kb/rule-based-compliance-in-practice.md §6 (layered governance)."
    );
    let _ = Arc::<()>::default(); // silence unused-import warning for std::sync::Arc
    Ok(())
}
