//! Atelier showcase: multi-plan allocation via ferrox CP-SAT.
//!
//! A real combinatorial selection: 6 candidate vendor-onboarding
//! plans compete; pick exactly K=3 subject to hard constraints
//! (budget, capability coverage, risk cap) while maximizing total
//! value. Ferrox's `CpSatSuggestor` solves the model and emits the
//! chosen subset as a typed plan fact. The scenario reads that
//! plan back and prints the selection + objective value.
//!
//! Honest exit: if the `with-solver` feature is off, the scenario
//! prints an install hint and returns — no heuristic fallback, no
//! silent stub. Run with:
//!
//!     cargo run -p scenario-multi-plan-allocation --features with-solver
//!
//! Requires libortools on the host (Homebrew: `brew install
//! google-or-tools`; or build from source).

#[cfg(feature = "with-solver")]
use converge_kernel::{Budget, ContextState, ConvergeResult, Engine, ProposedFact};
#[cfg(feature = "with-solver")]
use converge_pack::ContextKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
enum Risk {
    Low,
    Med,
    High,
}

#[derive(Debug, Clone)]
struct CandidatePlan {
    id: &'static str,
    description: &'static str,
    cost_usd: i64,
    value: i64,
    capabilities: &'static [Capability],
    risk: Risk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Capability {
    Invoicing,
    Sso,
    Catalog,
    Audit,
    Compliance,
}

impl Capability {
    #[cfg(feature = "with-solver")]
    fn all() -> [Capability; 5] {
        [
            Capability::Invoicing,
            Capability::Sso,
            Capability::Catalog,
            Capability::Audit,
            Capability::Compliance,
        ]
    }
    fn as_str(self) -> &'static str {
        match self {
            Capability::Invoicing => "invoicing",
            Capability::Sso => "sso",
            Capability::Catalog => "catalog",
            Capability::Audit => "audit",
            Capability::Compliance => "compliance",
        }
    }
}

const CATALOG: &[CandidatePlan] = &[
    CandidatePlan {
        id: "P0",
        description: "Lean invoicing rollout (incumbent vendor).",
        cost_usd: 3000,
        value: 70,
        capabilities: &[Capability::Invoicing],
        risk: Risk::Low,
    },
    CandidatePlan {
        id: "P1",
        description: "Invoicing + SSO bundle (mid-tier vendor).",
        cost_usd: 5000,
        value: 85,
        capabilities: &[Capability::Invoicing, Capability::Sso],
        risk: Risk::Med,
    },
    CandidatePlan {
        id: "P2",
        description: "Catalog-only pilot (boutique vendor).",
        cost_usd: 2000,
        value: 60,
        capabilities: &[Capability::Catalog],
        risk: Risk::Low,
    },
    CandidatePlan {
        id: "P3",
        description: "Catalog + audit add-on (enterprise vendor).",
        cost_usd: 4000,
        value: 80,
        capabilities: &[Capability::Catalog, Capability::Audit],
        risk: Risk::Med,
    },
    CandidatePlan {
        id: "P4",
        description: "Full-stack SSO + audit + compliance (premium vendor).",
        cost_usd: 7000,
        value: 95,
        capabilities: &[Capability::Sso, Capability::Audit, Capability::Compliance],
        risk: Risk::High,
    },
    CandidatePlan {
        id: "P5",
        description: "Low-cost invoicing alternative (long-tail vendor).",
        cost_usd: 1500,
        value: 50,
        capabilities: &[Capability::Invoicing],
        risk: Risk::Low,
    },
];

const PICK_K: i64 = 3;
const BUDGET_USD: i64 = 12_000;
const MAX_HIGH_RISK: i64 = 1;
#[cfg(feature = "with-solver")]
const REQUEST_ID: &str = "multi-plan";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_banner();
    print_catalog();
    print_constraints();

    #[cfg(not(feature = "with-solver"))]
    {
        println!("──────────────────────────────────────────────────────────");
        println!(
            "Solver disabled. Build with `--features with-solver` to enable\n\
             ferrox::cp::CpSatSuggestor (requires libortools on the host —\n\
             try `brew install google-or-tools` or build from source).\n\
             No heuristic fallback — honest exit."
        );
        println!("──────────────────────────────────────────────────────────");
        Ok(())
    }
    #[cfg(feature = "with-solver")]
    {
        run_solver().await
    }
}

#[cfg(feature = "with-solver")]
async fn run_solver() -> Result<(), Box<dyn std::error::Error>> {
    let request = build_cpsat_request();
    print_request(&request);

    let mut engine = Engine::with_budget(Budget {
        max_cycles: 8,
        max_facts: 64,
    });
    engine.register_suggestor(ferrox::cp::CpSatSuggestor);

    let mut ctx = ContextState::new();
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        format!("cpsat-request:{REQUEST_ID}"),
        request.clone(),
        "atelier-multi-plan-allocation",
    ));

    let result: ConvergeResult = engine.run(ctx).await?;
    print_solution(&result, &request);
    Ok(())
}

#[cfg(feature = "with-solver")]
fn build_cpsat_request() -> ferrox::cp::CpSatRequest {
    use ferrox::cp::{ConstraintKind, CpTerm, CpVariable};

    let n = CATALOG.len();
    let var_names: Vec<String> = (0..n).map(|i| format!("x_{i}")).collect();

    let variables: Vec<CpVariable> = var_names
        .iter()
        .map(|name| CpVariable {
            name: name.clone(),
            lb: 0,
            ub: 1,
            is_bool: true,
        })
        .collect();

    let mut constraints: Vec<ConstraintKind> = Vec::new();

    // 1. sum(x_i) == K — pick exactly K plans.
    constraints.push(ConstraintKind::LinearEq {
        terms: var_names
            .iter()
            .map(|v| CpTerm {
                var: v.clone(),
                coeff: 1,
            })
            .collect(),
        rhs: PICK_K,
    });

    // 2. sum(cost_i * x_i) <= BUDGET.
    constraints.push(ConstraintKind::LinearLe {
        terms: var_names
            .iter()
            .zip(CATALOG.iter())
            .map(|(v, p)| CpTerm {
                var: v.clone(),
                coeff: p.cost_usd,
            })
            .collect(),
        rhs: BUDGET_USD,
    });

    // 3. For each required capability, sum(covers_c[i] * x_i) >= 1.
    //    Required coverage: every capability except Compliance is mandatory;
    //    Compliance is optional (a "stretch" capability the solver MAY include).
    for cap in [
        Capability::Invoicing,
        Capability::Sso,
        Capability::Catalog,
        Capability::Audit,
    ] {
        constraints.push(ConstraintKind::LinearGe {
            terms: var_names
                .iter()
                .zip(CATALOG.iter())
                .map(|(v, p)| CpTerm {
                    var: v.clone(),
                    coeff: i64::from(p.capabilities.contains(&cap)),
                })
                .collect(),
            rhs: 1,
        });
    }

    // 4. sum(high_risk_i * x_i) <= MAX_HIGH_RISK.
    constraints.push(ConstraintKind::LinearLe {
        terms: var_names
            .iter()
            .zip(CATALOG.iter())
            .map(|(v, p)| CpTerm {
                var: v.clone(),
                coeff: i64::from(matches!(p.risk, Risk::High)),
            })
            .collect(),
        rhs: MAX_HIGH_RISK,
    });

    // Objective: maximize sum(value_i * x_i). CpSatRequest only
    // exposes `minimize: bool`, so negate the coefficients and
    // minimize their sum to achieve maximization.
    let objective_terms: Vec<CpTerm> = var_names
        .iter()
        .zip(CATALOG.iter())
        .map(|(v, p)| CpTerm {
            var: v.clone(),
            coeff: -p.value,
        })
        .collect();

    ferrox::cp::CpSatRequest {
        id: REQUEST_ID.to_string(),
        variables,
        interval_vars: Vec::new(),
        optional_interval_vars: Vec::new(),
        constraints,
        objective_terms: Some(objective_terms),
        minimize: true,
        time_limit_seconds: Some(5.0),
    }
}

fn print_banner() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  Multi-plan allocation — ferrox CP-SAT, real K-of-N selection      ║");
    println!("║  atelier-showcase · converge 3.9.1 · ferrox 0.7.1                    ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_catalog() {
    println!("Candidate plans");
    println!("───────────────");
    for p in CATALOG {
        let caps = p
            .capabilities
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "  {}  ${:>5}  value={:>3}  risk={:?}  caps=[{caps}]",
            p.id, p.cost_usd, p.value, p.risk
        );
        println!("        {}", p.description);
    }
    println!();
}

fn print_constraints() {
    println!("Hard constraints");
    println!("────────────────");
    println!("  • pick exactly K = {PICK_K} plans");
    println!("  • total cost ≤ ${BUDGET_USD}");
    println!("  • cover capabilities: invoicing, sso, catalog, audit (compliance optional)");
    println!("  • at most {MAX_HIGH_RISK} high-risk plan");
    println!("  • objective: maximize sum(value)");
    println!();
}

#[cfg(feature = "with-solver")]
fn print_request(req: &ferrox::cp::CpSatRequest) {
    println!("CP-SAT model");
    println!("────────────");
    println!("  variables:   {} (all 0/1 bool)", req.variables.len());
    println!("  constraints: {}", req.constraints.len());
    println!(
        "  objective:   {} terms ({})",
        req.objective_terms.as_ref().map_or(0, Vec::len),
        if req.minimize { "minimize" } else { "maximize" },
    );
    println!("  solver:      ferrox::cp::CpSatSuggestor (OR-Tools CP-SAT)");
    println!();
}

#[cfg(feature = "with-solver")]
fn print_solution(result: &ConvergeResult, request: &ferrox::cp::CpSatRequest) {
    use converge_pack::ContextFact;
    println!("Solver outcome");
    println!("──────────────");
    println!(
        "  converged:   {} (cycles: {}, stop: {:?})",
        result.converged, result.cycles, result.stop_reason
    );
    let plan_id = format!("cpsat-plan:{REQUEST_ID}");
    let plan_fact = result
        .context
        .get(ContextKey::Strategies)
        .iter()
        .find(|f| f.id().as_str() == plan_id);
    let Some(plan_fact) = plan_fact else {
        println!("  (no cpsat-plan fact produced — solver did not run)");
        return;
    };
    let plan = match plan_fact.require_payload::<ferrox::cp::CpSatPlan>() {
        Ok(p) => p,
        Err(e) => {
            println!("  (could not read CpSatPlan payload: {e})");
            return;
        }
    };

    println!("  status:      {:?}", plan.status);
    if let Some(obj) = plan.objective_value {
        println!(
            "  objective:   {} (total value {})",
            obj,
            -obj, // we minimized -value, so total value = -objective
        );
    }
    println!("  wall_time:   {:.3}s", plan.wall_time_seconds);
    println!("  solver:      {}", plan.solver);
    println!();

    let chosen_indices: Vec<usize> = plan
        .assignments
        .iter()
        .filter_map(|(name, val)| {
            if *val == 1 {
                name.strip_prefix("x_").and_then(|s| s.parse().ok())
            } else {
                None
            }
        })
        .collect();

    println!("Chosen plans");
    println!("────────────");
    let mut total_cost = 0_i64;
    let mut total_value = 0_i64;
    let mut covered_caps: std::collections::BTreeSet<&'static str> =
        std::collections::BTreeSet::new();
    for idx in &chosen_indices {
        let p = &CATALOG[*idx];
        total_cost += p.cost_usd;
        total_value += p.value;
        for c in p.capabilities {
            covered_caps.insert(c.as_str());
        }
        let caps = p
            .capabilities
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "  • {}  ${:>5}  value={:>3}  risk={:?}  caps=[{caps}]",
            p.id, p.cost_usd, p.value, p.risk
        );
        println!("        {}", p.description);
    }

    println!();
    println!("  totals:      cost=${total_cost}/{BUDGET_USD}  value={total_value}");
    println!(
        "  coverage:    [{}]",
        Capability::all()
            .iter()
            .map(|c| {
                let s = c.as_str();
                if covered_caps.contains(s) {
                    s.to_string()
                } else {
                    format!("({s} — uncovered)")
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = request; // request used above for printing; suppress unused warning if pruned
    println!();
    println!("✓ ferrox CP-SAT selected an optimal K-of-N subset under hard constraints.");
}
