// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Cross-extension version-skew canary.
//!
//! Runs the solver + arbiter + prism composition on every `cargo test
//! --workspace`. If a Converge bump silently breaks one of the three
//! extensions' integration with kernel types, this test fails before
//! the regression reaches a release.
//!
//! Runs on a vanilla dev machine — no C++ deps. The full ferrox-backed
//! path lives in `end_to_end.rs` behind the `with-solver` feature.

use std::sync::Arc;

use arbiter::{PolicyEngine, PolicyGateSuggestor, VENDOR_SELECTION_POLICY};
use atelier_domain::resource_routing::{
    ConstraintValidationAgent, FeasibilityAgent, ResourceRetrievalAgent, SolverAgent,
    TaskRetrievalAgent,
};
use converge_kernel::{ContextKey, ContextState, Engine};
use prism::FeatureAgent;

#[tokio::test]
async fn solver_policy_analytics_compose_in_one_formation() {
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

    let mut ctx = ContextState::new();
    ctx.add_input(ContextKey::Seeds, "tasks", "Task A, Task B")
        .expect("seed tasks");
    ctx.add_input(ContextKey::Seeds, "resources", "Worker 1, Worker 2")
        .expect("seed resources");

    let result = engine.run(ctx).await.expect("engine run succeeds");

    assert!(result.converged, "formation should converge");
    assert!(
        !result.context.get(ContextKey::Strategies).is_empty(),
        "solver should produce at least one assignment strategy"
    );
    assert!(
        !result.context.get(ContextKey::Constraints).is_empty(),
        "constraint validation should populate Constraints"
    );
    assert!(
        !result.context.get(ContextKey::Evaluations).is_empty(),
        "feasibility should populate Evaluations"
    );
}
