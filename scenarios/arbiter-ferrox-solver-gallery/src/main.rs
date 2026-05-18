// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Arbiter + Ferrox Solver Gallery.
//!
//! Demonstrates solver-backed Suggestors, catalog-based solver selection, and
//! Cedar policy gating in the same Converge run. The default build stays
//! portable. Enable `native-solvers` to register the OR-Tools/HiGHS Ferrox
//! Suggestors as well.

use std::sync::Arc;

#[cfg(test)]
use arbiter::PolicyOutcome;
use arbiter::{
    ContextIn, DecideRequest, EXPENSE_APPROVAL_POLICY, PolicyDecision, PolicyEngine,
    PolicyGateSuggestor, PrincipalIn, ResourceIn,
};
use async_trait::async_trait;
use converge_kernel::{
    AgentEffect, AuthorityLevel, Budget, Context, ContextFact, ContextKey, ContextState,
    ConvergeResult, Engine, FlowAction, FlowPhase, ProposedFact, Suggestor,
};
use converge_model::formation::{
    FormationPlan, FormationRequest, ProfileSnapshot, SuggestorCapability, SuggestorRole,
};
use converge_optimization::suggestors::{
    AssignmentPlan, AssignmentRequest, AssignmentSuggestor, FlowOptimizationSuggestor, FlowPlan,
    FlowRequest, FormationAssemblySuggestor,
};
use converge_pack::{FactPayload, TextPayload};
use converge_provider::{CostClass, LatencyClass};
use ferrox::catalog::{CommonUseCase, SolverCandidate, recommend_for_use_case, solver_catalog};
use serde::de::DeserializeOwned;

const PROVENANCE: &str = "example:arbiter-ferrox-solver-gallery";
const POLICY_SIGNAL_ID: &str = "policy-request:solver-spend";

const CORE_PLAN_IDS: &[&str] = &[
    "scheduling-plan-greedy:field-ops",
    "jspbench-plan-greedy:factory",
    "vrptw-plan-greedy:delivery",
    "assignment-plan:approvers",
    "flow-plan:capacity",
    "formation-plan:assurance",
];

#[cfg(all(test, feature = "native-solvers"))]
const NATIVE_PLAN_IDS: &[&str] = &[
    "scheduling-plan-cpsat:field-ops",
    "jspbench-plan-cpsat:factory",
    "vrptw-plan-cpsat:delivery",
    "glop-plan:capacity-lp",
    "network-flow-plan-ortools:allocation",
    "cpsat-plan:gate-model",
    "mip-plan:approval-mip",
    "cpsat-formation-plan:assurance-cp",
];

const POLICY_DEPS: [ContextKey; 1] = [ContextKey::Strategies];

#[tokio::main]
async fn main() {
    println!("=== Arbiter + Ferrox Solver Gallery ===\n");

    print_catalog();
    print_recommendations(cfg!(feature = "native-solvers"));
    print_registered_surface();

    let result = run_gallery().await.expect("solver gallery should converge");

    println!(
        "Converged: {} (cycles: {}, stop: {:?})\n",
        result.converged, result.cycles, result.stop_reason
    );

    print_section("Strategies", result.context.get(ContextKey::Strategies));
    print_section("Policy request", result.context.get(ContextKey::Signals));
    print_section("Constraints", result.context.get(ContextKey::Constraints));

    if let Some(decision) = policy_decision(&result.context) {
        println!(
            "Arbiter decision for solver-selected spend: {:?}",
            decision.outcome
        );
        println!("Policy reason: {:?}\n", decision.reason);
    }

    println!("SMT/SAT counterexample search remains cataloged as external/deferred.");
    println!("=== Done ===");
}

async fn run_gallery() -> Result<ConvergeResult, converge_kernel::ConvergeError> {
    let mut engine = build_engine();
    engine.run(seed_context()).await
}

fn build_engine() -> Engine {
    let mut engine = Engine::with_budget(Budget {
        max_cycles: 12,
        max_facts: 128,
    });

    engine.register_suggestor(ferrox::scheduling::GreedySchedulerSuggestor);
    engine.register_suggestor(ferrox::jobshop::GreedyJobShopSuggestor);
    engine.register_suggestor(ferrox::vrptw::NearestNeighborSuggestor);
    engine.register_suggestor(AssignmentSuggestor);
    engine.register_suggestor(FlowOptimizationSuggestor);
    engine.register_suggestor(FormationAssemblySuggestor::new(profile_catalog()));

    register_native_suggestors(&mut engine);

    engine.register_suggestor(PolicyRequestFromCorePlans);
    let policy = Arc::new(
        PolicyEngine::from_policy_str(EXPENSE_APPROVAL_POLICY)
            .expect("expense policy should parse"),
    );
    engine.register_suggestor(PolicyGateSuggestor::with_keys(
        policy,
        ContextKey::Signals,
        ContextKey::Constraints,
    ));

    engine
}

#[cfg(feature = "native-solvers")]
fn register_native_suggestors(engine: &mut Engine) {
    engine.register_suggestor(ferrox::scheduling::CpSatSchedulerSuggestor);
    engine.register_suggestor(ferrox::jobshop::CpSatJobShopSuggestor);
    engine.register_suggestor(ferrox::vrptw::CpSatVrptwSuggestor);
    engine.register_suggestor(ferrox::lp::GlopLpSuggestor);
    engine.register_suggestor(ferrox::network_flow::MinCostFlowSuggestor);
    engine.register_suggestor(ferrox::cp::CpSatSuggestor);
    engine.register_suggestor(ferrox::mip::HighsMipSuggestor);
    engine.register_suggestor(ferrox::formation::CpSatFormationSuggestor::new(
        profile_catalog(),
    ));
}

#[cfg(not(feature = "native-solvers"))]
fn register_native_suggestors(_engine: &mut Engine) {}

fn seed_context() -> ContextState {
    let mut ctx = ContextState::new();

    seed_typed_json::<ferrox::scheduling::SchedulingRequest>(
        &mut ctx,
        "scheduling-request:field-ops",
        scheduling_request(),
    );
    seed_typed_json::<ferrox::jobshop::JobShopRequest>(
        &mut ctx,
        "jspbench-request:factory",
        job_shop_request(),
    );
    seed_typed_json::<ferrox::vrptw::VrptwRequest>(
        &mut ctx,
        "vrptw-request:delivery",
        vrptw_request(),
    );
    seed_typed_json::<AssignmentRequest>(
        &mut ctx,
        "assignment-request:approvers",
        assignment_request(),
    );
    seed_typed_json::<FlowRequest>(&mut ctx, "flow-request:capacity", flow_request());
    seed_formation(&mut ctx, "formation-request:assurance", "assurance");

    seed_native_requests(&mut ctx);

    ctx
}

#[cfg(feature = "native-solvers")]
fn seed_native_requests(ctx: &mut ContextState) {
    seed_typed_json::<ferrox::lp::LpRequest>(ctx, "glop-request:capacity-lp", lp_request());
    seed_typed_json::<ferrox::network_flow::MinCostFlowRequest>(
        ctx,
        "network-flow-request:allocation",
        native_flow_request(),
    );
    seed_typed_json::<ferrox::cp::CpSatRequest>(ctx, "cpsat-request:gate-model", cp_sat_request());
    seed_typed_json::<ferrox::mip::MipRequest>(ctx, "mip-request:approval-mip", mip_request());
    seed_formation(ctx, "cpsat-formation-request:assurance-cp", "assurance-cp");
}

#[cfg(not(feature = "native-solvers"))]
fn seed_native_requests(_ctx: &mut ContextState) {}

fn seed_typed_json<T>(ctx: &mut ContextState, id: &'static str, value: serde_json::Value)
where
    T: DeserializeOwned + FactPayload + PartialEq,
{
    let payload = serde_json::from_value::<T>(value).expect("typed seed literal should parse");
    seed_payload(ctx, id, payload);
}

fn seed_payload<T>(ctx: &mut ContextState, id: &'static str, payload: T)
where
    T: FactPayload + PartialEq,
{
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        id,
        payload,
        PROVENANCE,
    ))
    .expect("seed should be accepted");
}

fn seed_formation(ctx: &mut ContextState, fact_id: &'static str, request_id: &str) {
    let request = FormationRequest {
        id: request_id.to_string(),
        required_roles: vec![
            SuggestorRole::Analysis,
            SuggestorRole::Planning,
            SuggestorRole::Constraint,
        ],
        required_capabilities: vec![],
    };
    seed_payload(ctx, fact_id, request);
}

struct PolicyRequestFromCorePlans;

#[async_trait]
impl Suggestor for PolicyRequestFromCorePlans {
    fn name(&self) -> &str {
        "PolicyRequestFromCorePlans"
    }

    fn provenance(&self) -> &'static str {
        "atelier-showcase.arbiter-ferrox-solver-gallery"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &POLICY_DEPS
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        CORE_PLAN_IDS
            .iter()
            .all(|plan_id| fact_exists(ctx, ContextKey::Strategies, plan_id))
            && !fact_exists(ctx, ContextKey::Signals, POLICY_SIGNAL_ID)
    }

    async fn execute(&self, _ctx: &dyn Context) -> AgentEffect {
        let request = expense_commit_request(
            "agent:finance-solver-gallery",
            vec!["finance"],
            AuthorityLevel::Supervisory,
            4_200,
            false,
        );
        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Signals,
                POLICY_SIGNAL_ID,
                request,
                self.name().to_owned(),
            )
            .with_confidence(0.95),
        )
    }
}

fn expense_commit_request(
    principal_id: &str,
    domains: Vec<&str>,
    authority: AuthorityLevel,
    amount: i64,
    human_approval_present: bool,
) -> DecideRequest {
    DecideRequest {
        principal: PrincipalIn {
            id: principal_id.into(),
            authority,
            domains: domains.into_iter().map(Into::into).collect(),
            policy_version: Some("expense_v1".into()),
        },
        resource: ResourceIn {
            id: "expense:solver-gallery".into(),
            resource_type: Some("expense".into()),
            phase: Some(FlowPhase::Commitment),
            gates_passed: Some(vec!["receipt".into(), "manager_approval".into()]),
        },
        action: FlowAction::Commit,
        context: Some(ContextIn {
            commitment_type: Some("expense".to_string()),
            amount: Some(amount),
            human_approval_present: Some(human_approval_present),
            required_gates_met: Some(true),
        }),
        delegation_b64: None,
    }
}

#[cfg(test)]
async fn run_policy_gate_case(request: DecideRequest) -> Result<PolicyDecision, String> {
    let mut engine = Engine::with_budget(Budget {
        max_cycles: 4,
        max_facts: 16,
    });
    let policy = Arc::new(
        PolicyEngine::from_policy_str(EXPENSE_APPROVAL_POLICY)
            .expect("expense policy should parse"),
    );
    engine.register_suggestor(PolicyGateSuggestor::new(policy));

    let mut ctx = ContextState::new();
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        "policy-request:single",
        request,
        PROVENANCE,
    ))
    .expect("policy seed should be accepted");

    let result = engine.run(ctx).await.map_err(|err| err.to_string())?;
    policy_decision(&result.context)
        .ok_or_else(|| "policy gate did not emit a decision".to_string())
}

#[cfg(test)]
fn evaluate_policy_request(request: &DecideRequest) -> PolicyOutcome {
    PolicyEngine::from_policy_str(EXPENSE_APPROVAL_POLICY)
        .expect("expense policy should parse")
        .evaluate(request)
        .expect("policy request should evaluate")
        .outcome
}

fn policy_decision(ctx: &dyn Context) -> Option<PolicyDecision> {
    ctx.get(ContextKey::Constraints)
        .iter()
        .find(|fact| fact.id().as_str() == "policy-decision")
        .and_then(|fact| fact.payload::<PolicyDecision>().cloned())
}

fn fact_exists(ctx: &dyn Context, key: ContextKey, id: &str) -> bool {
    ctx.get(key).iter().any(|fact| fact.id().as_str() == id)
}

fn profile_catalog() -> Vec<ProfileSnapshot> {
    vec![
        profile(
            "cedar-policy-analysis",
            SuggestorRole::Analysis,
            vec![SuggestorCapability::PolicyEnforcement],
            0.93,
            LatencyClass::Interactive,
            CostClass::Low,
        ),
        profile(
            "ferrox-solver-planning",
            SuggestorRole::Planning,
            vec![SuggestorCapability::Optimization],
            0.88,
            LatencyClass::Interactive,
            CostClass::Medium,
        ),
        profile(
            "arbiter-policy-gate",
            SuggestorRole::Constraint,
            vec![SuggestorCapability::PolicyEnforcement],
            0.96,
            LatencyClass::Realtime,
            CostClass::Low,
        ),
        profile(
            "review-synthesis",
            SuggestorRole::Synthesis,
            vec![],
            0.72,
            LatencyClass::Interactive,
            CostClass::Medium,
        ),
    ]
}

fn profile(
    name: &str,
    role: SuggestorRole,
    capabilities: Vec<SuggestorCapability>,
    confidence_max: f32,
    latency_hint: LatencyClass,
    cost_hint: CostClass,
) -> ProfileSnapshot {
    ProfileSnapshot {
        name: name.to_string(),
        role,
        output_keys: vec![ContextKey::Strategies],
        cost_hint,
        latency_hint,
        capabilities,
        confidence_min: 0.5,
        confidence_max,
    }
}

fn scheduling_request() -> serde_json::Value {
    serde_json::json!({
        "id": "field-ops",
        "horizon_min": 240,
        "time_limit_seconds": 2.0,
        "agents": [
            {"id": 10, "name": "Ada", "capabilities": ["finance", "routing"]},
            {"id": 20, "name": "Ben", "capabilities": ["approval", "inspection"]},
            {"id": 30, "name": "Cy", "capabilities": ["finance", "inspection"]}
        ],
        "tasks": [
            {"id": 1, "name": "receipt audit", "required_capability": "finance", "duration_min": 35, "release_min": 0, "deadline_min": 90},
            {"id": 2, "name": "manager approval", "required_capability": "approval", "duration_min": 25, "release_min": 10, "deadline_min": 100},
            {"id": 3, "name": "site inspection", "required_capability": "inspection", "duration_min": 45, "release_min": 40, "deadline_min": 160},
            {"id": 4, "name": "reimbursement batch", "required_capability": "finance", "duration_min": 50, "release_min": 90, "deadline_min": 220}
        ]
    })
}

fn job_shop_request() -> serde_json::Value {
    serde_json::json!({
        "id": "factory",
        "num_machines": 3,
        "time_limit_seconds": 2.0,
        "jobs": [
            {"id": 0, "name": "invoice-A", "operations": [
                {"machine_id": 0, "duration": 3},
                {"machine_id": 1, "duration": 2},
                {"machine_id": 2, "duration": 2}
            ]},
            {"id": 1, "name": "invoice-B", "operations": [
                {"machine_id": 1, "duration": 2},
                {"machine_id": 2, "duration": 4},
                {"machine_id": 0, "duration": 3}
            ]},
            {"id": 2, "name": "invoice-C", "operations": [
                {"machine_id": 2, "duration": 2},
                {"machine_id": 0, "duration": 2},
                {"machine_id": 1, "duration": 3}
            ]}
        ]
    })
}

fn vrptw_request() -> serde_json::Value {
    serde_json::json!({
        "id": "delivery",
        "time_limit_seconds": 2.0,
        "depot": {"x": 0.0, "y": 0.0, "ready_time": 0, "due_time": 120},
        "customers": [
            {"id": 1, "name": "receipt pickup", "x": 3.0, "y": 1.0, "window_open": 0, "window_close": 35, "service_time": 6},
            {"id": 2, "name": "manager signature", "x": 6.0, "y": 2.0, "window_open": 10, "window_close": 60, "service_time": 8},
            {"id": 3, "name": "finance desk", "x": 8.0, "y": 7.0, "window_open": 40, "window_close": 95, "service_time": 8},
            {"id": 4, "name": "archive", "x": 2.0, "y": 8.0, "window_open": 60, "window_close": 115, "service_time": 5}
        ]
    })
}

fn assignment_request() -> serde_json::Value {
    serde_json::json!({
        "id": "approvers",
        "agents": ["finance-lead", "manager", "auditor"],
        "tasks": ["receipt-check", "approval-check", "audit-sample"],
        "costs": [
            [2, 7, 6],
            [6, 2, 5],
            [5, 4, 1]
        ]
    })
}

fn flow_request() -> serde_json::Value {
    serde_json::json!({
        "id": "capacity",
        "num_nodes": 4,
        "source": 0,
        "sink": 3,
        "demand": 5,
        "edges": [
            {"from": 0, "to": 1, "capacity": 3, "cost": 1, "label": "finance"},
            {"from": 0, "to": 2, "capacity": 4, "cost": 2, "label": "ops"},
            {"from": 1, "to": 3, "capacity": 3, "cost": 2, "label": "approved"},
            {"from": 2, "to": 3, "capacity": 4, "cost": 1, "label": "fallback"}
        ]
    })
}

#[cfg(feature = "native-solvers")]
fn lp_request() -> serde_json::Value {
    serde_json::json!({
        "id": "capacity-lp",
        "variables": [
            {"name": "finance_hours", "lb": 0.0, "ub": 100.0},
            {"name": "ops_hours", "lb": 0.0, "ub": 80.0}
        ],
        "constraints": [
            {"name": "combined_capacity", "lb": 0.0, "ub": 120.0, "terms": [
                {"var": "finance_hours", "coeff": 1.0},
                {"var": "ops_hours", "coeff": 1.0}
            ]}
        ],
        "objective": {
            "maximize": true,
            "terms": [
                {"var": "finance_hours", "coeff": 40.0},
                {"var": "ops_hours", "coeff": 30.0}
            ]
        },
        "time_limit_seconds": 2.0
    })
}

#[cfg(feature = "native-solvers")]
fn native_flow_request() -> serde_json::Value {
    serde_json::json!({
        "id": "allocation",
        "mode": "balanced_min_cost",
        "supplies": [
            {"node": 0, "supply": 5},
            {"node": 2, "supply": -5}
        ],
        "arcs": [
            {"name": "direct", "tail": 0, "head": 2, "capacity": 5, "unit_cost": 5},
            {"name": "via-review", "tail": 0, "head": 1, "capacity": 5, "unit_cost": 1},
            {"name": "review-to-sink", "tail": 1, "head": 2, "capacity": 5, "unit_cost": 1}
        ]
    })
}

#[cfg(feature = "native-solvers")]
fn cp_sat_request() -> serde_json::Value {
    serde_json::json!({
        "id": "gate-model",
        "variables": [
            {"name": "receipt", "lb": 0, "ub": 1, "is_bool": true},
            {"name": "manager_approval", "lb": 0, "ub": 1, "is_bool": true},
            {"name": "risk", "lb": 0, "ub": 10}
        ],
        "interval_vars": [],
        "optional_interval_vars": [],
        "constraints": [
            {"kind": "linear_ge", "terms": [{"var": "receipt", "coeff": 1}], "rhs": 1},
            {"kind": "linear_ge", "terms": [{"var": "manager_approval", "coeff": 1}], "rhs": 1},
            {"kind": "linear_le", "terms": [{"var": "risk", "coeff": 1}], "rhs": 4}
        ],
        "objective_terms": [{"var": "risk", "coeff": 1}],
        "minimize": true,
        "time_limit_seconds": 2.0
    })
}

#[cfg(feature = "native-solvers")]
fn mip_request() -> serde_json::Value {
    serde_json::json!({
        "id": "approval-mip",
        "variables": [
            {"name": "choose_finance", "lb": 0.0, "ub": 1.0, "kind": "binary"},
            {"name": "choose_ops", "lb": 0.0, "ub": 1.0, "kind": "binary"}
        ],
        "constraints": [
            {"name": "single_route", "lb": 0.0, "ub": 1.0, "terms": [
                {"var": "choose_finance", "coeff": 1.0},
                {"var": "choose_ops", "coeff": 1.0}
            ]}
        ],
        "objective": {
            "maximize": true,
            "terms": [
                {"var": "choose_finance", "coeff": 7.0},
                {"var": "choose_ops", "coeff": 5.0}
            ]
        },
        "time_limit_seconds": 2.0,
        "mip_gap_tolerance": 0.0
    })
}

fn print_catalog() {
    println!("Cataloged solver surfaces:");
    for candidate in solver_catalog() {
        println!(
            "  - {} => {} ({:?}, feature: {})",
            candidate.id,
            candidate.symbol,
            candidate.exposure,
            candidate.feature.unwrap_or("default")
        );
    }
    println!();
}

fn print_recommendations(native_available: bool) {
    println!("Use-case recommendations:");
    for use_case in common_use_cases() {
        let candidates = recommend_for_use_case(use_case);
        let selected = choose_candidate(&candidates, native_available)
            .map(|candidate| candidate.id)
            .unwrap_or("none");
        println!("  - {:?}: {}", use_case, selected);
    }
    println!();
}

fn choose_candidate(
    candidates: &[&'static SolverCandidate],
    native_available: bool,
) -> Option<&'static SolverCandidate> {
    candidates
        .iter()
        .copied()
        .find(|candidate| native_available || candidate.feature.is_none())
        .or_else(|| candidates.first().copied())
}

fn common_use_cases() -> [CommonUseCase; 10] {
    [
        CommonUseCase::FieldCrewScheduling,
        CommonUseCase::FactoryJobShop,
        CommonUseCase::DeliveryTimeWindows,
        CommonUseCase::AssignmentMatching,
        CommonUseCase::SourceSinkFlow,
        CommonUseCase::LinearProgram,
        CommonUseCase::MixedIntegerProgram,
        CommonUseCase::CustomCpSatModel,
        CommonUseCase::FormationAssembly,
        CommonUseCase::CedarPolicyCounterexample,
    ]
}

fn print_registered_surface() {
    println!("Registered in this build:");
    for name in portable_suggestors() {
        println!("  - {name}");
    }

    if cfg!(feature = "native-solvers") {
        for name in native_suggestors() {
            println!("  - {name}");
        }
    } else {
        println!("  - native Ferrox Suggestors available with --features native-solvers");
    }
    println!();
}

fn portable_suggestors() -> [&'static str; 7] {
    [
        "ferrox::scheduling::GreedySchedulerSuggestor",
        "ferrox::jobshop::GreedyJobShopSuggestor",
        "ferrox::vrptw::NearestNeighborSuggestor",
        "converge_optimization::suggestors::AssignmentSuggestor",
        "converge_optimization::suggestors::FlowOptimizationSuggestor",
        "converge_optimization::suggestors::FormationAssemblySuggestor",
        "arbiter::PolicyGateSuggestor",
    ]
}

fn native_suggestors() -> [&'static str; 8] {
    [
        "ferrox::scheduling::CpSatSchedulerSuggestor",
        "ferrox::jobshop::CpSatJobShopSuggestor",
        "ferrox::vrptw::CpSatVrptwSuggestor",
        "ferrox::lp::GlopLpSuggestor",
        "ferrox::network_flow::MinCostFlowSuggestor",
        "ferrox::cp::CpSatSuggestor",
        "ferrox::mip::HighsMipSuggestor",
        "ferrox::formation::CpSatFormationSuggestor",
    ]
}

fn print_section(title: &str, facts: &[ContextFact]) {
    println!("{title}:");
    if facts.is_empty() {
        println!("  (none)\n");
        return;
    }

    for fact in facts {
        let preview = fact_preview(fact);
        println!("  {} ({preview})", fact.id());
    }
    println!();
}

fn fact_preview(fact: &ContextFact) -> String {
    if let Some(payload) = fact.payload::<TextPayload>() {
        return payload.as_str().to_owned();
    }
    if let Some(payload) = fact.payload::<FormationRequest>() {
        return format!("{payload:?}");
    }
    if let Some(payload) = fact.payload::<DecideRequest>() {
        return format!("{payload:?}");
    }
    if let Some(payload) = fact.payload::<PolicyDecision>() {
        return format!("{payload:?}");
    }
    if let Some(payload) = fact.payload::<AssignmentPlan>() {
        return format!("{payload:?}");
    }
    if let Some(payload) = fact.payload::<FlowPlan>() {
        return format!("{payload:?}");
    }
    if let Some(payload) = fact.payload::<FormationPlan>() {
        return format!("{payload:?}");
    }
    if let Some(payload) = fact.payload::<ferrox::scheduling::SchedulingPlan>() {
        return format!("{payload:?}");
    }
    if let Some(payload) = fact.payload::<ferrox::jobshop::JobShopPlan>() {
        return format!("{payload:?}");
    }
    if let Some(payload) = fact.payload::<ferrox::vrptw::VrptwPlan>() {
        return format!("{payload:?}");
    }
    #[cfg(feature = "native-solvers")]
    {
        if let Some(payload) = fact.payload::<ferrox::lp::LpPlan>() {
            return format!("{payload:?}");
        }
        if let Some(payload) = fact.payload::<ferrox::network_flow::MinCostFlowPlan>() {
            return format!("{payload:?}");
        }
        if let Some(payload) = fact.payload::<ferrox::cp::CpSatPlan>() {
            return format!("{payload:?}");
        }
        if let Some(payload) = fact.payload::<ferrox::mip::MipPlan>() {
            return format!("{payload:?}");
        }
    }
    format!(
        "<typed payload {} v{}>",
        fact.payload_family(),
        fact.payload_version()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[tokio::test]
    async fn gallery_runs_portable_solver_suggestors_and_policy_gate() {
        let result = run_gallery().await.expect("gallery should run");

        for plan_id in CORE_PLAN_IDS {
            assert!(
                fact_exists(&result.context, ContextKey::Strategies, plan_id),
                "missing {plan_id}"
            );
        }

        #[cfg(feature = "native-solvers")]
        for plan_id in NATIVE_PLAN_IDS {
            assert!(
                fact_exists(&result.context, ContextKey::Strategies, plan_id),
                "missing {plan_id}"
            );
        }

        assert!(fact_exists(
            &result.context,
            ContextKey::Signals,
            POLICY_SIGNAL_ID
        ));
        let decision = policy_decision(&result.context).expect("policy decision");
        assert_eq!(decision.outcome, PolicyOutcome::Escalate);
    }

    #[tokio::test]
    async fn policy_gate_promotes_finance_commit_with_human_approval() {
        let decision = run_policy_gate_case(expense_commit_request(
            "agent:finance-supervisor",
            vec!["finance"],
            AuthorityLevel::Supervisory,
            4_200,
            true,
        ))
        .await
        .expect("policy gate should run");

        assert_eq!(decision.outcome, PolicyOutcome::Promote);
    }

    #[tokio::test]
    async fn policy_gate_rejects_non_finance_commit() {
        let decision = run_policy_gate_case(expense_commit_request(
            "agent:ops-supervisor",
            vec!["operations"],
            AuthorityLevel::Supervisory,
            4_200,
            true,
        ))
        .await
        .expect("policy gate should run");

        assert_eq!(decision.outcome, PolicyOutcome::Reject);
    }

    #[test]
    fn catalog_recommends_external_smt_for_cedar_counterexamples() {
        let recommendations = recommend_for_use_case(CommonUseCase::CedarPolicyCounterexample);
        assert_eq!(
            recommendations.first().map(|candidate| candidate.id),
            Some("external.smt.counterexample")
        );
    }

    #[test]
    fn catalog_contains_every_showcase_native_surface() {
        let ids: Vec<_> = solver_catalog()
            .iter()
            .map(|candidate| candidate.id)
            .collect();

        for expected in [
            "ferrox.task-scheduling.cpsat",
            "ferrox.job-shop.cpsat",
            "ferrox.vrptw.cpsat",
            "ferrox.flow.simple-min-cost",
            "ferrox.lp.glop",
            "ferrox.mip.highs",
            "ferrox.cp.cpsat",
            "ferrox.formation.cpsat",
        ] {
            assert!(ids.contains(&expected), "missing {expected}");
        }
    }

    proptest! {
        #[test]
        fn low_value_supervisory_finance_commit_without_human_approval_escalates(
            amount in 1_i64..=5_000
        ) {
            let request = expense_commit_request(
                "agent:finance-supervisor",
                vec!["finance"],
                AuthorityLevel::Supervisory,
                amount,
                false,
            );
            prop_assert_eq!(evaluate_policy_request(&request), PolicyOutcome::Escalate);
        }

        #[test]
        fn high_value_supervisory_finance_commit_without_human_approval_rejects(
            amount in 5_001_i64..=50_000
        ) {
            let request = expense_commit_request(
                "agent:finance-supervisor",
                vec!["finance"],
                AuthorityLevel::Supervisory,
                amount,
                false,
            );
            prop_assert_eq!(evaluate_policy_request(&request), PolicyOutcome::Reject);
        }
    }
}
