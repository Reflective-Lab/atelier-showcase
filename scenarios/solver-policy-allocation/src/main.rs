// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Solver + Policy + Analytics Allocation — cross-extension showcase.
//!
//! Composes three Converge extensions inside one formation:
//! - **atelier-domain** `resource_routing` — the solver pipeline
//!   (replaced by a ferrox CP-SAT solver under the `with-solver` feature)
//! - **arbiter** — Cedar policy gate over the solver's output
//! - **prism** — feature extraction over the run's history
//!
//! This scenario is the canary for version-skew across the three extensions.
//! If a Converge bump silently breaks one of them, this binary stops compiling.

use std::sync::Arc;

use arbiter::{PolicyEngine, PolicyGateSuggestor, VENDOR_SELECTION_POLICY};
use atelier_domain::resource_routing::{
    ConstraintValidationAgent, FeasibilityAgent, ResourceRetrievalAgent, SolverAgent,
    TaskRetrievalAgent,
};
use atelier_domain::{admitted_text, domain_text, record_payload};
use converge_kernel::{ContextKey, ContextState, Engine};
use converge_pack::ContextFact;
use prism::FeatureAgent;

#[cfg(feature = "with-solver")]
use converge_pack::{ProposedFact, Provenance};
#[cfg(feature = "with-solver")]
use ferrox::cp::{ConstraintKind, CpSatRequest, CpSatSuggestor, CpVariable};

#[tokio::main]
async fn main() {
    println!("=== Solver + Policy + Analytics Allocation ===\n");

    let mut engine = Engine::new();

    // 1. atelier-domain resource_routing pipeline — the "solver" today.
    //    Step 3 (the `with-solver` feature) swaps in a ferrox CP-SAT backend.
    engine.register_suggestor(TaskRetrievalAgent);
    engine.register_suggestor(ResourceRetrievalAgent);
    engine.register_suggestor(ConstraintValidationAgent);
    engine.register_suggestor(SolverAgent);
    engine.register_suggestor(FeasibilityAgent);

    // 2. arbiter Cedar policy gate. Wired against a Diagnostic-keyed input/output
    //    pair so it stays inert in this stub — step 3 routes the solver's
    //    candidate assignments through it as DecideRequests.
    let policy = PolicyEngine::from_policy_str(VENDOR_SELECTION_POLICY)
        .expect("VENDOR_SELECTION_POLICY should parse");
    engine.register_suggestor(PolicyGateSuggestor::with_keys(
        Arc::new(policy),
        ContextKey::Diagnostic,
        ContextKey::Diagnostic,
    ));

    // 3. prism FeatureAgent. No source path — also inert in the stub.
    //    Step 3 wires real feature extraction over the converged context.
    engine.register_suggestor(FeatureAgent::new(None));

    // 4. Optional ferrox CP-SAT solver. Off by default; opt in with
    //    `--features with-solver` (requires libortools on the system).
    #[cfg(feature = "with-solver")]
    engine.register_suggestor(CpSatSuggestor);

    let mut ctx = ContextState::new();
    let _ = ctx.add_input(ContextKey::Seeds, "tasks", "Delivery A, Delivery B");
    let _ = ctx.add_input(ContextKey::Seeds, "resources", "Vehicle 1, Vehicle 2");

    #[cfg(feature = "with-solver")]
    {
        let cp_request = two_task_two_worker_assignment();
        let request_id = cp_request.id.clone();
        let _ = ctx.add_proposal(ProposedFact::new(
            ContextKey::Seeds,
            format!("cpsat-request:{request_id}"),
            cp_request,
            Provenance::new("example-solver-policy-allocation"),
        ));
    }

    println!("Seeds: 2 tasks, 2 resources.\n");

    match engine.run(ctx).await {
        Ok(result) => {
            println!("Converged: {}", result.converged);
            println!("Cycles:    {}", result.cycles);
            println!("Stop:      {:?}\n", result.stop_reason);

            for key in [
                ContextKey::Strategies,
                ContextKey::Constraints,
                ContextKey::Evaluations,
            ] {
                for fact in result.context.get(key) {
                    println!("  [{:?}/{}] {}", key, fact.id(), fact_preview(fact));
                }
            }
        }
        Err(e) => println!("Allocation failed: {e}"),
    }

    println!("\n=== Done ===");
}

fn fact_preview(fact: &ContextFact) -> String {
    if let Some(text) = domain_text(fact).or_else(|| admitted_text(fact)) {
        return text.to_owned();
    }

    if let Some(record) = record_payload(fact) {
        return format!("{} {}", record.record_type(), record.data());
    }

    format!(
        "<typed payload {} v{}>",
        fact.payload_family(),
        fact.payload_version()
    )
}

/// Two tasks, two workers, each task assigned to a distinct worker.
/// Minimal CP-SAT model used to prove the ferrox wiring; the production
/// version of this scenario would build the model from real seed data.
#[cfg(feature = "with-solver")]
fn two_task_two_worker_assignment() -> CpSatRequest {
    use ferrox::cp::CpTerm;
    CpSatRequest {
        id: "alloc-demo".into(),
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
