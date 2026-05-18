// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Full ferrox + arbiter + prism end-to-end test.
//!
//! Requires OR-tools (libortools) installed on the host.
//!
//! Run with:  cargo test -p example-solver-policy-allocation \
//!                --features with-solver --test end_to_end
//!
//! The version-skew canary in `smoke.rs` runs unconditionally; this test
//! adds the ferrox CP-SAT solver on top to prove the full triple converges.

#![cfg(feature = "with-solver")]

use std::sync::Arc;

use arbiter::{PolicyEngine, PolicyGateSuggestor, VENDOR_SELECTION_POLICY};
use atelier_domain::resource_routing::{
    ConstraintValidationAgent, FeasibilityAgent, ResourceRetrievalAgent, SolverAgent,
    TaskRetrievalAgent,
};
use converge_kernel::{ContextKey, ContextState, Engine};
use converge_pack::{ProposedFact, Provenance};
use ferrox::cp::{ConstraintKind, CpSatPlan, CpSatRequest, CpSatSuggestor, CpTerm, CpVariable};
use prism::FeatureAgent;

fn two_task_two_worker_assignment() -> CpSatRequest {
    CpSatRequest {
        id: "alloc-e2e".into(),
        variables: vec![
            CpVariable {
                name: "t0".into(),
                lb: 0,
                ub: 1,
                is_bool: false,
            },
            CpVariable {
                name: "t1".into(),
                lb: 0,
                ub: 1,
                is_bool: false,
            },
        ],
        interval_vars: vec![],
        optional_interval_vars: vec![],
        constraints: vec![ConstraintKind::AllDifferent {
            vars: vec!["t0".into(), "t1".into()],
        }],
        objective_terms: Some(vec![
            CpTerm {
                var: "t0".into(),
                coeff: 1,
            },
            CpTerm {
                var: "t1".into(),
                coeff: 1,
            },
        ]),
        minimize: true,
        time_limit_seconds: Some(1.0),
    }
}

#[tokio::test]
async fn full_triple_converges_with_solver() {
    let mut engine = Engine::new();

    engine.register_suggestor(TaskRetrievalAgent);
    engine.register_suggestor(ResourceRetrievalAgent);
    engine.register_suggestor(ConstraintValidationAgent);
    engine.register_suggestor(SolverAgent);
    engine.register_suggestor(FeasibilityAgent);

    let policy = PolicyEngine::from_policy_str(VENDOR_SELECTION_POLICY)
        .expect("VENDOR_SELECTION_POLICY parses");
    engine.register_suggestor(PolicyGateSuggestor::with_keys(
        Arc::new(policy),
        ContextKey::Diagnostic,
        ContextKey::Diagnostic,
    ));

    engine.register_suggestor(FeatureAgent::new(None));
    engine.register_suggestor(CpSatSuggestor);

    let mut ctx = ContextState::new();
    ctx.add_input(ContextKey::Seeds, "tasks", "Task A, Task B")
        .expect("seed tasks");
    ctx.add_input(ContextKey::Seeds, "resources", "Worker 1, Worker 2")
        .expect("seed resources");

    let cp_request = two_task_two_worker_assignment();
    let request_id = cp_request.id.clone();
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        format!("cpsat-request:{request_id}"),
        cp_request,
        Provenance::new("example-solver-policy-allocation"),
    ))
    .expect("seed cpsat-request");

    let result = engine.run(ctx).await.expect("engine run succeeds");

    assert!(result.converged, "formation should converge");

    let strategies = result.context.get(ContextKey::Strategies);
    let plan_id = format!("cpsat-plan:{request_id}");
    let cp_plan = strategies
        .iter()
        .find(|f| f.id() == plan_id.as_str())
        .expect("CpSatSuggestor should emit a cpsat-plan fact");

    let plan = cp_plan
        .require_payload::<CpSatPlan>()
        .expect("cpsat-plan carries CpSatPlan payload");
    let status = plan.status.as_str();
    assert!(
        matches!(status, "optimal" | "feasible"),
        "solver status should be optimal or feasible, got {status}"
    );

    assert_eq!(
        plan.assignments.len(),
        2,
        "two assignments for two variables"
    );
}
