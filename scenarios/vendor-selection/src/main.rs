// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Vendor Selection — swarm evaluation with consensus and HITL gates.
//!
//! Demonstrates: swarms, consensus/aggregation, multi-criteria scoring, and
//! Cedar-backed procurement gating.
//!
//! This is a Converge kernel fixture, not the canonical vendor-selection
//! workflow. Reusable vendor decision semantics live downstream in Organism
//! domain packs.

use arbiter::{PolicyEngine, VENDOR_SELECTION_POLICY};
use atelier_domain::{DomainRecordPayload, json_value};
use converge_kernel::{
    AgentEffect, AuthorityLevel, Context, ContextFact, ContextKey, ContextState, Engine,
    EngineHitlPolicy, FlowAction, FlowGateAuthorizer, FlowGateContext, FlowGateInput,
    FlowGateOutcome, FlowGatePrincipal, FlowGateResource, FlowPhase, GateDecision, ProposedFact,
    RunResult, Suggestor, TimeoutAction, TimeoutPolicy,
};
use converge_pack::TextPayload;
use std::sync::Arc;

fn record(record_type: &str, data: serde_json::Value) -> DomainRecordPayload {
    DomainRecordPayload::new(record_type, data)
}

fn fact_json(fact: &ContextFact) -> serde_json::Value {
    json_value(fact).unwrap_or_default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WholeDollars(i64);

impl WholeDollars {
    fn from_field(json: &serde_json::Value, field: &str) -> Option<Self> {
        json.get(field)
            .and_then(|value| {
                value
                    .as_i64()
                    .or_else(|| value.as_u64().and_then(|n| i64::try_from(n).ok()))
            })
            .map(Self)
    }

    fn as_i64(self) -> i64 {
        self.0
    }
}

fn vendor_price_dollars(vendor: &serde_json::Value) -> i64 {
    WholeDollars::from_field(vendor, "price").map_or(0, WholeDollars::as_i64)
}

fn vendor_selection_input(
    vendor: &serde_json::Value,
    action: FlowAction,
    human_approval_present: bool,
) -> FlowGateInput {
    let compliant = vendor
        .get("compliant")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let years = vendor
        .get("years_in_business")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let mut gates_passed = vec!["competitive_review".to_string()];
    if compliant && years >= 5 {
        gates_passed.push("vendor_due_diligence".to_string());
    }

    FlowGateInput {
        principal: FlowGatePrincipal {
            id: "agent:procurement-supervisor".into(),
            authority: AuthorityLevel::Supervisory,
            domains: vec!["procurement".into()],
            policy_version: Some("vendor_v1".into()),
        },
        resource: FlowGateResource {
            id: format!(
                "vendor-selection:{}",
                vendor
                    .get("id")
                    .and_then(|value| value.as_str())
                    .unwrap_or("unknown")
            )
            .into(),
            kind: "spend".into(),
            phase: FlowPhase::Commitment,
            gates_passed: gates_passed.into_iter().map(Into::into).collect(),
        },
        action,
        context: FlowGateContext {
            commitment_type: Some("spend".into()),
            amount: Some(vendor_price_dollars(vendor)),
            human_approval_present: Some(human_approval_present),
            required_gates_met: Some(compliant && years >= 5),
        },
    }
}

fn top_vendor(ctx: &dyn Context) -> Option<serde_json::Value> {
    let recommendation = ctx
        .get(ContextKey::Strategies)
        .iter()
        .find(|fact| fact.id() == "recommendation-1")?;
    let recommendation_json = fact_json(recommendation);
    let vendor_id = recommendation_json.get("vendor_id")?.as_str()?;

    ctx.get(ContextKey::Signals).iter().find_map(|fact| {
        let vendor = fact_json(fact);
        let id = vendor.get("id").and_then(|value| value.as_str())?;
        if id == vendor_id { Some(vendor) } else { None }
    })
}

fn has_procurement_approval(ctx: &dyn Context) -> bool {
    ctx.get(ContextKey::Proposals)
        .iter()
        .any(|fact| fact.id() == "procurement-approval")
}

fn load_vendor_policy_engine() -> Arc<dyn FlowGateAuthorizer> {
    Arc::new(
        PolicyEngine::from_policy_str(VENDOR_SELECTION_POLICY)
            .expect("vendor selection policy should parse"),
    )
}

struct VendorDataAgent;

#[async_trait::async_trait]
impl Suggestor for VendorDataAgent {
    fn name(&self) -> &str {
        "VendorDataAgent"
    }

    fn provenance(&self) -> &'static str {
        "atelier-showcase.vendor-selection"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.has(ContextKey::Seeds) && !ctx.has(ContextKey::Signals)
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let seeds = ctx.get(ContextKey::Seeds);
        let seed = seeds.first();

        let vendors = if let Some(s) = seed {
            let json = fact_json(s);
            json.get("vendors").cloned().unwrap_or_default()
        } else {
            serde_json::json!([])
        };

        let mut facts = Vec::new();
        for vendor in vendors
            .as_array()
            .map_or(&[] as &[serde_json::Value], |v| v)
        {
            facts.push(
                ProposedFact::new(
                    ContextKey::Signals,
                    format!(
                        "vendor-{}",
                        vendor.get("id").and_then(|v| v.as_str()).unwrap_or("?")
                    ),
                    record("vendor", vendor.clone()),
                    self.name().to_owned(),
                )
                .with_confidence(1.0),
            );
        }

        AgentEffect::with_proposals(facts)
    }
}

struct PriceEvaluatorAgent;

#[async_trait::async_trait]
impl Suggestor for PriceEvaluatorAgent {
    fn name(&self) -> &str {
        "PriceEvaluatorAgent"
    }

    fn provenance(&self) -> &'static str {
        "atelier-showcase.vendor-selection"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.has(ContextKey::Signals) && !ctx.has(ContextKey::Evaluations)
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let signals = ctx.get(ContextKey::Signals);

        let mut evaluations = Vec::new();
        for signal in signals {
            let vendor = fact_json(signal);
            let id = vendor.get("id").and_then(|v| v.as_str()).unwrap_or("?");
            let price = vendor_price_dollars(&vendor);

            let score = if price < 10_000 {
                1.0
            } else if price < 25_000 {
                0.7
            } else if price < 50_000 {
                0.4
            } else {
                0.1
            };

            evaluations.push(
                ProposedFact::new(
                    ContextKey::Evaluations,
                    format!("price:{}", id),
                    record(
                        "vendor_evaluation",
                        serde_json::json!({
                            "vendor_id": id,
                            "criterion": "price",
                            "score": score,
                            "raw_value": price
                        }),
                    ),
                    self.name().to_owned(),
                )
                .with_confidence(1.0),
            );
        }

        AgentEffect::with_proposals(evaluations)
    }
}

struct ComplianceEvaluatorAgent;

#[async_trait::async_trait]
impl Suggestor for ComplianceEvaluatorAgent {
    fn name(&self) -> &str {
        "ComplianceEvaluatorAgent"
    }

    fn provenance(&self) -> &'static str {
        "atelier-showcase.vendor-selection"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.has(ContextKey::Signals) && !ctx.has(ContextKey::Evaluations)
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let signals = ctx.get(ContextKey::Signals);

        let mut evaluations = Vec::new();
        for signal in signals {
            let vendor = fact_json(signal);
            let id = vendor.get("id").and_then(|v| v.as_str()).unwrap_or("?");
            let compliant: bool = vendor
                .get("compliant")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            let score = if compliant { 1.0 } else { 0.0 };

            evaluations.push(
                ProposedFact::new(
                    ContextKey::Evaluations,
                    format!("compliance:{}", id),
                    record(
                        "vendor_evaluation",
                        serde_json::json!({
                            "vendor_id": id,
                            "criterion": "compliance",
                            "score": score,
                            "raw_value": compliant
                        }),
                    ),
                    self.name().to_owned(),
                )
                .with_confidence(1.0),
            );
        }

        AgentEffect::with_proposals(evaluations)
    }
}

struct RiskEvaluatorAgent;

#[async_trait::async_trait]
impl Suggestor for RiskEvaluatorAgent {
    fn name(&self) -> &str {
        "RiskEvaluatorAgent"
    }

    fn provenance(&self) -> &'static str {
        "atelier-showcase.vendor-selection"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.has(ContextKey::Signals) && !ctx.has(ContextKey::Evaluations)
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let signals = ctx.get(ContextKey::Signals);

        let mut evaluations = Vec::new();
        for signal in signals {
            let vendor = fact_json(signal);
            let id = vendor.get("id").and_then(|v| v.as_str()).unwrap_or("?");
            let years: u32 = vendor
                .get("years_in_business")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;

            let score = if years > 10 {
                1.0
            } else if years > 5 {
                0.7
            } else if years > 2 {
                0.4
            } else {
                0.1
            };

            evaluations.push(
                ProposedFact::new(
                    ContextKey::Evaluations,
                    format!("risk:{}", id),
                    record(
                        "vendor_evaluation",
                        serde_json::json!({
                            "vendor_id": id,
                            "criterion": "risk",
                            "score": score,
                            "raw_value": years
                        }),
                    ),
                    self.name().to_owned(),
                )
                .with_confidence(1.0),
            );
        }

        AgentEffect::with_proposals(evaluations)
    }
}

struct TimelineEvaluatorAgent;

#[async_trait::async_trait]
impl Suggestor for TimelineEvaluatorAgent {
    fn name(&self) -> &str {
        "TimelineEvaluatorAgent"
    }

    fn provenance(&self) -> &'static str {
        "atelier-showcase.vendor-selection"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.has(ContextKey::Signals) && !ctx.has(ContextKey::Evaluations)
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let signals = ctx.get(ContextKey::Signals);

        let mut evaluations = Vec::new();
        for signal in signals {
            let vendor = fact_json(signal);
            let id = vendor.get("id").and_then(|v| v.as_str()).unwrap_or("?");
            let weeks: u32 = vendor
                .get("delivery_weeks")
                .and_then(|v| v.as_u64())
                .unwrap_or(52) as u32;

            let score = if weeks <= 4 {
                1.0
            } else if weeks <= 8 {
                0.8
            } else if weeks <= 12 {
                0.5
            } else {
                0.2
            };

            evaluations.push(
                ProposedFact::new(
                    ContextKey::Evaluations,
                    format!("timeline:{}", id),
                    record(
                        "vendor_evaluation",
                        serde_json::json!({
                            "vendor_id": id,
                            "criterion": "timeline",
                            "score": score,
                            "raw_value": weeks
                        }),
                    ),
                    self.name().to_owned(),
                )
                .with_confidence(1.0),
            );
        }

        AgentEffect::with_proposals(evaluations)
    }
}

struct ConsensusAgent;

#[async_trait::async_trait]
impl Suggestor for ConsensusAgent {
    fn name(&self) -> &str {
        "ConsensusAgent"
    }

    fn provenance(&self) -> &'static str {
        "atelier-showcase.vendor-selection"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Evaluations]
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.has(ContextKey::Evaluations) && !ctx.has(ContextKey::Strategies)
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let evaluations = ctx.get(ContextKey::Evaluations);

        let mut vendor_scores: std::collections::HashMap<String, (f64, u32)> =
            std::collections::HashMap::new();

        for eval in evaluations {
            let eval_json = fact_json(eval);
            let vendor_id = eval_json
                .get("vendor_id")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let score: f64 = eval_json
                .get("score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let entry = vendor_scores
                .entry(vendor_id.to_string())
                .or_insert((0.0, 0));
            entry.0 += score;
            entry.1 += 1;
        }

        let _weights = serde_json::json!({
            "price": 0.30,
            "compliance": 0.25,
            "risk": 0.20,
            "timeline": 0.15,
            "quality": 0.10
        });

        let mut weighted_scores: Vec<(String, f64)> = Vec::new();
        for (vendor_id, (total_score, count)) in vendor_scores {
            let avg_score = total_score / count as f64;
            weighted_scores.push((vendor_id, avg_score));
        }

        weighted_scores.sort_by(|a, b| b.1.total_cmp(&a.1));

        let proposals: Vec<ProposedFact> = weighted_scores
            .iter()
            .enumerate()
            .map(|(i, (vendor_id, score))| {
                ProposedFact::new(
                    ContextKey::Strategies,
                    format!("recommendation-{}", i + 1),
                    record(
                        "vendor_recommendation",
                        serde_json::json!({
                            "vendor_id": vendor_id,
                            "rank": i + 1,
                            "score": score,
                            "recommendation": if i == 0 { "recommended" } else { "alternative" }
                        }),
                    ),
                    "consensus-agent",
                )
                .with_confidence(if i == 0 { 0.85 } else { 0.6 })
            })
            .collect();

        AgentEffect::with_proposals(proposals)
    }
}

struct ProcurementRoutingAgent {
    policy: Arc<dyn FlowGateAuthorizer>,
}

const PROCUREMENT_ROUTING_DEPS: [ContextKey; 2] = [ContextKey::Signals, ContextKey::Strategies];

#[async_trait::async_trait]
impl Suggestor for ProcurementRoutingAgent {
    fn name(&self) -> &str {
        "ProcurementRoutingAgent"
    }

    fn provenance(&self) -> &'static str {
        "atelier-showcase.vendor-selection"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &PROCUREMENT_ROUTING_DEPS
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.has(ContextKey::Signals)
            && ctx
                .get(ContextKey::Strategies)
                .iter()
                .any(|fact| fact.id() == "recommendation-1")
            && !ctx
                .get(ContextKey::Constraints)
                .iter()
                .any(|fact| fact.id() == "vendor-procurement-routing")
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let Some(vendor) = top_vendor(ctx) else {
            return AgentEffect::default();
        };

        let decision = self
            .policy
            .decide(&vendor_selection_input(&vendor, FlowAction::Commit, false))
            .expect("policy evaluation should succeed for vendor selection routing");

        let (required_approvers, current_approver) = match decision.outcome {
            FlowGateOutcome::Escalate => (vec!["procurement".to_string()], Some("procurement")),
            FlowGateOutcome::Reject => (vec!["procurement".to_string()], Some("procurement")),
            FlowGateOutcome::Promote => (Vec::new(), None),
        };

        let routing = serde_json::json!({
            "required_approvers": required_approvers,
            "current_approver": current_approver,
            "pending": if current_approver.is_some() { 1 } else { 0 },
            "commit_outcome": decision.outcome,
            "commit_reason": decision.reason
        });

        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Constraints,
                "vendor-procurement-routing",
                record("vendor_procurement_routing", routing),
                self.name().to_owned(),
            )
            .with_confidence(1.0),
        )
    }
}

struct ProcurementApprovalSimulationAgent;

#[async_trait::async_trait]
impl Suggestor for ProcurementApprovalSimulationAgent {
    fn name(&self) -> &str {
        "ProcurementApprovalSimulationAgent"
    }

    fn provenance(&self) -> &'static str {
        "atelier-showcase.vendor-selection"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Constraints]
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.get(ContextKey::Constraints)
            .iter()
            .any(|fact| fact.id() == "vendor-procurement-routing")
            && !has_procurement_approval(ctx)
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let Some(constraint) = ctx
            .get(ContextKey::Constraints)
            .iter()
            .find(|fact| fact.id() == "vendor-procurement-routing")
        else {
            return AgentEffect::default();
        };

        let routing = fact_json(constraint);
        let pending = routing
            .get("pending")
            .and_then(|value| value.as_u64())
            .unwrap_or(0);
        if pending == 0 {
            return AgentEffect::default();
        }

        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Proposals,
                "procurement-approval",
                TextPayload::new("Approved by procurement"),
                "procurement approval agent",
            )
            .with_confidence(0.95),
        )
    }
}

struct VendorCommitDecisionAgent {
    policy: Arc<dyn FlowGateAuthorizer>,
}

const VENDOR_COMMIT_DEPS: [ContextKey; 3] = [
    ContextKey::Signals,
    ContextKey::Constraints,
    ContextKey::Proposals,
];

#[async_trait::async_trait]
impl Suggestor for VendorCommitDecisionAgent {
    fn name(&self) -> &str {
        "VendorCommitDecisionAgent"
    }

    fn provenance(&self) -> &'static str {
        "atelier-showcase.vendor-selection"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &VENDOR_COMMIT_DEPS
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.has(ContextKey::Signals)
            && ctx
                .get(ContextKey::Constraints)
                .iter()
                .any(|fact| fact.id() == "vendor-procurement-routing")
            && !ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|fact| fact.id() == "vendor-commit-policy")
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let Some(vendor) = top_vendor(ctx) else {
            return AgentEffect::default();
        };

        let human_approval_present = has_procurement_approval(ctx);
        let constraint = ctx
            .get(ContextKey::Constraints)
            .iter()
            .find(|fact| fact.id() == "vendor-procurement-routing");

        if !human_approval_present {
            let pending = constraint
                .map(fact_json)
                .and_then(|json| json.get("pending").and_then(|value| value.as_u64()))
                .unwrap_or(0);
            if pending > 0 {
                return AgentEffect::default();
            }
        }

        let decision = self
            .policy
            .decide(&vendor_selection_input(
                &vendor,
                FlowAction::Commit,
                human_approval_present,
            ))
            .expect("policy evaluation should succeed for vendor selection commit");

        let result = serde_json::json!({
            "gate": "commit",
            "outcome": decision.outcome,
            "reason": decision.reason,
            "human_approval_present": human_approval_present
        });

        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Evaluations,
                "vendor-commit-policy",
                record("vendor_commit_policy", result),
                self.name().to_owned(),
            )
            .with_confidence(1.0),
        )
    }
}

#[tokio::main]
async fn main() {
    println!("=== Vendor Selection Example ===\n");

    let mut engine = Engine::new();
    let policy = load_vendor_policy_engine();

    engine.register_suggestor(VendorDataAgent);
    engine.register_suggestor(PriceEvaluatorAgent);
    engine.register_suggestor(ComplianceEvaluatorAgent);
    engine.register_suggestor(RiskEvaluatorAgent);
    engine.register_suggestor(TimelineEvaluatorAgent);
    engine.register_suggestor(ConsensusAgent);
    engine.register_suggestor(ProcurementRoutingAgent {
        policy: Arc::clone(&policy),
    });
    engine.register_suggestor(ProcurementApprovalSimulationAgent);
    engine.register_suggestor(VendorCommitDecisionAgent { policy });

    let hitl_policy = EngineHitlPolicy {
        confidence_threshold: None,
        gated_keys: vec![ContextKey::Proposals],
        timeout: TimeoutPolicy {
            timeout_secs: 300,
            action: TimeoutAction::Reject,
        },
    };
    engine.set_hitl_policy(hitl_policy);

    let rfp = serde_json::json!({
        "vendors": [
            {
                "id": "vendor-a",
                "name": "Acme Corp",
                "price": 15000,
                "compliant": true,
                "years_in_business": 15,
                "delivery_weeks": 6
            },
            {
                "id": "vendor-b",
                "name": "Beta Solutions",
                "price": 22000,
                "compliant": true,
                "years_in_business": 8,
                "delivery_weeks": 4
            },
            {
                "id": "vendor-c",
                "name": "Gamma Industries",
                "price": 8000,
                "compliant": false,
                "years_in_business": 3,
                "delivery_weeks": 10
            }
        ]
    });

    let mut ctx = ContextState::new();
    let _ = ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        "rfp-1",
        record("vendor_rfp", rfp),
        "example-vendor-selection",
    ));

    println!("Evaluating 3 vendors with swarm of 5 agents...\n");

    match engine.run_with_hitl(ctx).await {
        RunResult::HitlPause(pause) => {
            println!("⏸️  HITL Gate: Cedar required procurement approval");
            println!("    Approval request: {}", pause.request.summary);
            println!();
            if let Some(vendor) = top_vendor(&pause.context)
                && let Some(id) = vendor.get("id").and_then(|value| value.as_str())
            {
                println!("    Top vendor: {}", id);
            }

            let decision =
                GateDecision::approve(pause.request.gate_id.clone(), "procurement@company.com");

            println!("▶️  Approved by procurement. Finalizing...\n");

            match engine.resume(*pause, decision).await {
                RunResult::Complete(Ok(result)) => {
                    println!("✅ Vendor Selected!\n");
                    for fact in result.context.get(ContextKey::Strategies) {
                        let p = fact_json(fact);
                        let rank = p.get("rank").and_then(|v| v.as_u64()).unwrap_or(0);
                        let vendor = p.get("vendor_id").and_then(|v| v.as_str()).unwrap_or("?");
                        let score = p.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let rec = p
                            .get("recommendation")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?");
                        println!("  #{}. {} (score: {:.2}) - {}", rank, vendor, score, rec);
                    }
                    for fact in result.context.get(ContextKey::Evaluations) {
                        if fact.id() == "vendor-commit-policy" {
                            println!("  [commit] {}", fact_preview(fact));
                        }
                    }
                }
                _ => println!("❌ Selection failed"),
            }
        }
        RunResult::Complete(Ok(result)) => {
            println!("✅ Vendor Selected!\n");
            for fact in result.context.get(ContextKey::Strategies) {
                let p = fact_json(fact);
                let rank = p.get("rank").and_then(|v| v.as_u64()).unwrap_or(0);
                let vendor = p.get("vendor_id").and_then(|v| v.as_str()).unwrap_or("?");
                let score = p.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let rec = p
                    .get("recommendation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                println!("  #{}. {} (score: {:.2}) - {}", rank, vendor, score, rec);
            }
            for fact in result.context.get(ContextKey::Evaluations) {
                if fact.id() == "vendor-commit-policy" {
                    println!("  [commit] {}", fact_preview(fact));
                }
            }
        }
        RunResult::Complete(Err(e)) => {
            println!("❌ Selection failed: {e}");
        }
    }

    println!("\n=== Done ===");
}

fn fact_preview(fact: &ContextFact) -> String {
    if let Some(text) = fact.payload::<TextPayload>() {
        return text.as_str().to_owned();
    }
    if let Some(value) = json_value(fact) {
        return format!("{value}");
    }
    format!(
        "<typed payload {} v{}>",
        fact.payload_family(),
        fact.payload_version()
    )
}
