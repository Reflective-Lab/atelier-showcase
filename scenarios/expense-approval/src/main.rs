// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Expense Approval Workflow — multi-level approval with HITL gates.
//!
//! Demonstrates: long-running workflows, humans in the loop, and Cedar-backed
//! gate decisions projected from flow state.
//!
//! This is a Converge kernel fixture, not the canonical expense workflow.
//! Reusable spend-approval semantics live downstream in Organism domain packs.

use arbiter::{EXPENSE_APPROVAL_POLICY, PolicyEngine};
use atelier_domain::{DomainRecordPayload, json_value};
use converge_kernel::Provenance;
use converge_kernel::{
    AgentEffect, AuthorityLevel, Context, ContextFact, ContextKey, ContextState, Engine,
    EngineHitlPolicy, FlowAction, FlowGateAuthorizer, FlowGateContext, FlowGateInput,
    FlowGateOutcome, FlowGatePrincipal, FlowGateResource, FlowPhase, GateDecision, ProposedFact,
    RunResult, Suggestor, TimeoutAction, TimeoutPolicy,
};
use converge_pack::TextPayload;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
struct AtelierShowcaseProvenance;

impl converge_kernel::ProvenanceSource for AtelierShowcaseProvenance {
    fn as_str(&self) -> &'static str {
        "atelier-showcase.expense-approval"
    }
}

const ATELIER_SHOWCASE_PROVENANCE: AtelierShowcaseProvenance = AtelierShowcaseProvenance;

fn atelier_showcase_provenance() -> Provenance {
    converge_kernel::ProvenanceSource::provenance(ATELIER_SHOWCASE_PROVENANCE)
}

struct ExpenseParsingAgent;

fn record(record_type: &str, data: serde_json::Value) -> DomainRecordPayload {
    DomainRecordPayload::new(record_type, data)
}

fn fact_json(fact: &ContextFact) -> serde_json::Value {
    json_value(fact).unwrap_or_default()
}

fn receipt_attached(expense: &serde_json::Value) -> bool {
    expense
        .get("receipt_attached")
        .and_then(|value| value.as_bool())
        .unwrap_or(true)
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

fn expense_amount(expense: &serde_json::Value) -> i64 {
    WholeDollars::from_field(expense, "amount").map_or(0, WholeDollars::as_i64)
}

fn has_human_approval(ctx: &dyn Context) -> bool {
    ctx.get(ContextKey::Proposals)
        .iter()
        .any(|fact| fact.id().ends_with("-approval"))
}

fn expense_policy_input(
    expense: &serde_json::Value,
    action: FlowAction,
    human_approval_present: bool,
) -> FlowGateInput {
    let mut gates_passed = Vec::new();
    if receipt_attached(expense) {
        gates_passed.push("receipt".to_string());
    }
    if human_approval_present {
        gates_passed.push("manager_approval".to_string());
    }

    FlowGateInput {
        principal: FlowGatePrincipal {
            id: "agent:finance-supervisor".into(),
            authority: AuthorityLevel::Supervisory,
            domains: vec!["finance".into()],
            policy_version: Some("expense_v1".into()),
        },
        resource: FlowGateResource {
            id: "expense:demo-001".into(),
            kind: "expense".into(),
            phase: FlowPhase::Commitment,
            gates_passed: gates_passed.into_iter().map(Into::into).collect(),
        },
        action,
        context: FlowGateContext {
            commitment_type: Some("expense".into()),
            amount: Some(expense_amount(expense)),
            human_approval_present: Some(human_approval_present),
            required_gates_met: Some(receipt_attached(expense)),
        },
    }
}

fn load_expense_policy_engine() -> Arc<dyn FlowGateAuthorizer> {
    Arc::new(
        PolicyEngine::from_policy_str(EXPENSE_APPROVAL_POLICY)
            .expect("expense approval policy should parse"),
    )
}

#[async_trait::async_trait]
impl Suggestor for ExpenseParsingAgent {
    fn name(&self) -> &str {
        "ExpenseParsingAgent"
    }

    fn provenance(&self) -> Provenance {
        atelier_showcase_provenance()
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.has(ContextKey::Seeds) && !ctx.has(ContextKey::Strategies)
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let seeds = ctx.get(ContextKey::Seeds);
        let seed = seeds.first();

        let parsed = if let Some(s) = seed {
            let json = fact_json(s);
            ProposedFact::new(
                ContextKey::Strategies,
                "parsed-expense",
                record("expense", json),
                self.provenance(),
            )
            .with_confidence(1.0)
        } else {
            ProposedFact::new(
                ContextKey::Strategies,
                "parsed-expense",
                record("expense", serde_json::json!({})),
                self.provenance(),
            )
            .with_confidence(1.0)
        };

        AgentEffect::with_proposals(vec![parsed])
    }
}

struct PolicyValidationAgent {
    policy: Arc<dyn FlowGateAuthorizer>,
}

#[async_trait::async_trait]
impl Suggestor for PolicyValidationAgent {
    fn name(&self) -> &str {
        "PolicyValidationAgent"
    }

    fn provenance(&self) -> Provenance {
        atelier_showcase_provenance()
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Strategies]
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.has(ContextKey::Strategies)
            && !ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|fact| fact.id() == "expense-validate-policy")
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let strategies = ctx.get(ContextKey::Strategies);
        let strategy = strategies.first();

        let result = strategy
            .map(fact_json)
            .map(|expense| {
                let decision = self
                    .policy
                    .decide(&expense_policy_input(&expense, FlowAction::Validate, false))
                    .expect("policy evaluation should succeed for expense validation");

                serde_json::json!({
                    "gate": "validate",
                    "outcome": decision.outcome,
                    "reason": decision.reason,
                    "amount": expense_amount(&expense),
                    "receipt_attached": receipt_attached(&expense)
                })
            })
            .unwrap_or_else(|| {
                serde_json::json!({
                    "gate": "validate",
                    "outcome": FlowGateOutcome::Reject,
                    "reason": "missing parsed expense"
                })
            });

        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Evaluations,
                "expense-validate-policy",
                record("expense_policy_validation", result),
                self.provenance(),
            )
            .with_confidence(1.0),
        )
    }
}

struct ApprovalRoutingAgent {
    policy: Arc<dyn FlowGateAuthorizer>,
}

const ROUTING_DEPS: [ContextKey; 2] = [ContextKey::Strategies, ContextKey::Evaluations];

#[async_trait::async_trait]
impl Suggestor for ApprovalRoutingAgent {
    fn name(&self) -> &str {
        "ApprovalRoutingAgent"
    }

    fn provenance(&self) -> Provenance {
        atelier_showcase_provenance()
    }

    fn dependencies(&self) -> &[ContextKey] {
        &ROUTING_DEPS
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.has(ContextKey::Strategies)
            && ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|fact| fact.id() == "expense-validate-policy")
            && !ctx
                .get(ContextKey::Constraints)
                .iter()
                .any(|fact| fact.id() == "expense-approval-routing")
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let evaluations = ctx.get(ContextKey::Evaluations);
        let strategies = ctx.get(ContextKey::Strategies);

        if let (Some(e), Some(s)) = (evaluations.first(), strategies.first()) {
            let eval = fact_json(e);
            let expense = fact_json(s);
            let validate_outcome = eval
                .get("outcome")
                .and_then(|value| value.as_str())
                .unwrap_or("reject");

            let commit_decision = self
                .policy
                .decide(&expense_policy_input(&expense, FlowAction::Commit, false))
                .expect("policy evaluation should succeed for commit routing");

            let (required_approvers, current_approver) = match commit_decision.outcome {
                FlowGateOutcome::Escalate => (vec!["manager".to_string()], Some("manager")),
                FlowGateOutcome::Reject if validate_outcome != "promote" => {
                    (vec!["finance".to_string()], Some("finance"))
                }
                FlowGateOutcome::Reject => (vec!["finance".to_string()], Some("finance")),
                FlowGateOutcome::Promote => (Vec::new(), None),
            };

            let routing = serde_json::json!({
                "required_approvers": required_approvers,
                "current_approver": current_approver,
                "pending": if current_approver.is_some() { 1 } else { 0 },
                "validate_outcome": validate_outcome,
                "commit_outcome": commit_decision.outcome,
                "commit_reason": commit_decision.reason
            });

            return AgentEffect::with_proposal(
                ProposedFact::new(
                    ContextKey::Constraints,
                    "expense-approval-routing",
                    record("expense_approval_routing", routing),
                    self.provenance(),
                )
                .with_confidence(1.0),
            );
        }

        AgentEffect::default()
    }
}

struct CommitDecisionAgent {
    policy: Arc<dyn FlowGateAuthorizer>,
}

const COMMIT_DEPS: [ContextKey; 3] = [
    ContextKey::Strategies,
    ContextKey::Constraints,
    ContextKey::Proposals,
];

#[async_trait::async_trait]
impl Suggestor for CommitDecisionAgent {
    fn name(&self) -> &str {
        "CommitDecisionAgent"
    }

    fn provenance(&self) -> Provenance {
        atelier_showcase_provenance()
    }

    fn dependencies(&self) -> &[ContextKey] {
        &COMMIT_DEPS
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.has(ContextKey::Strategies)
            && ctx
                .get(ContextKey::Constraints)
                .iter()
                .any(|fact| fact.id() == "expense-approval-routing")
            && !ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|fact| fact.id() == "expense-commit-policy")
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let Some(strategy) = ctx.get(ContextKey::Strategies).first() else {
            return AgentEffect::default();
        };

        let expense = fact_json(strategy);
        let human_approval_present = has_human_approval(ctx);
        let constraint = ctx
            .get(ContextKey::Constraints)
            .iter()
            .find(|fact| fact.id() == "expense-approval-routing");

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
            .decide(&expense_policy_input(
                &expense,
                FlowAction::Commit,
                human_approval_present,
            ))
            .expect("policy evaluation should succeed for final commit");

        let result = serde_json::json!({
            "gate": "commit",
            "outcome": decision.outcome,
            "reason": decision.reason,
            "human_approval_present": human_approval_present
        });

        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Evaluations,
                "expense-commit-policy",
                record("expense_commit_policy", result),
                self.provenance(),
            )
            .with_confidence(1.0),
        )
    }
}

struct ApprovalSimulationAgent;

#[async_trait::async_trait]
impl Suggestor for ApprovalSimulationAgent {
    fn name(&self) -> &str {
        "ApprovalSimulationAgent"
    }

    fn provenance(&self) -> Provenance {
        atelier_showcase_provenance()
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Constraints]
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.get(ContextKey::Constraints)
            .iter()
            .any(|fact| fact.id() == "expense-approval-routing")
            && !has_human_approval(ctx)
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        if let Some(c) = ctx
            .get(ContextKey::Constraints)
            .iter()
            .find(|fact| fact.id() == "expense-approval-routing")
        {
            let routing = fact_json(c);
            let pending = routing
                .get("pending")
                .and_then(|value| value.as_u64())
                .unwrap_or(0);
            if pending == 0 {
                return AgentEffect::default();
            }

            let current = routing
                .get("current_approver")
                .and_then(|v| v.as_str())
                .unwrap_or("manager");

            let proposal = ProposedFact::new(
                ContextKey::Proposals,
                format!("{current}-approval"),
                TextPayload::new(format!("Approved by {current}")),
                self.provenance(),
            )
            .with_confidence(0.95);

            return AgentEffect::with_proposal(proposal);
        }

        AgentEffect::default()
    }
}

#[tokio::main]
async fn main() {
    println!("=== Expense Approval Workflow Example ===\n");

    let mut engine = Engine::new();
    let policy = load_expense_policy_engine();

    engine.register_suggestor(ExpenseParsingAgent);
    engine.register_suggestor(PolicyValidationAgent {
        policy: Arc::clone(&policy),
    });
    engine.register_suggestor(ApprovalRoutingAgent {
        policy: Arc::clone(&policy),
    });
    engine.register_suggestor(ApprovalSimulationAgent);
    engine.register_suggestor(CommitDecisionAgent { policy });

    let hitl_policy = EngineHitlPolicy {
        confidence_threshold: Some(0.8),
        gated_keys: vec![ContextKey::Proposals],
        timeout: TimeoutPolicy {
            timeout_secs: 300,
            action: TimeoutAction::Reject,
        },
    };
    engine.set_hitl_policy(hitl_policy);

    let expense = serde_json::json!({
        "employee": "john.doe@example.com",
        "amount": 4200,
        "category": "entertainment",
        "description": "Client dinner",
        "date": "2026-04-15",
        "receipt_attached": true
    });

    let mut ctx = ContextState::new();
    let _ = ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        "expense-1",
        record("expense", expense.clone()),
        atelier_showcase_provenance(),
    ));

    println!(
        "Expense submitted: ${} {} - {}\n",
        expense["amount"], expense["category"], expense["description"]
    );
    println!("Running approval workflow...\n");

    match engine.run_with_hitl(ctx).await {
        RunResult::HitlPause(pause) => {
            println!("⏸️  HITL Gate: Cedar required human approval");
            println!("    Proposal: {}", pause.request.summary);
            println!(
                "    Approver: {}",
                pause.request.rationale.as_deref().unwrap_or("manager")
            );
            println!();

            let decision =
                GateDecision::approve(pause.request.gate_id.clone(), "manager@company.com");

            println!("▶️  Manager approved. Resuming workflow...\n");

            match engine.resume(*pause, decision).await {
                RunResult::Complete(Ok(result)) => {
                    println!("✅ Expense flow completed.\n");
                    for fact in result.context.get(ContextKey::Evaluations) {
                        println!("  [{}] {}", fact.id(), fact_preview(fact));
                    }
                }
                RunResult::HitlPause(_) => println!("❌ Unexpected extra approval stage"),
                _ => println!("❌ Approval workflow failed"),
            }
        }
        RunResult::Complete(Ok(result)) => {
            println!("✅ Expense flow completed without HITL.\n");
            for fact in result.context.get(ContextKey::Evaluations) {
                println!("  [{}] {}", fact.id(), fact_preview(fact));
            }
        }
        RunResult::Complete(Err(e)) => {
            println!("❌ Workflow failed: {e}");
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
