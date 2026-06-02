//! Atelier showcase: LP diet problem via ferrox GLOP.
//!
//! ## Problem class
//!
//! **Linear programming** — the canonical *polynomial-time* member
//! of mathematical optimization. Formally:
//!
//! ```text
//!     minimize    c^T x
//!     subject to  A x  ≥  b
//!                 x  ≥  0,   x ∈ R^n
//! ```
//!
//! The feasible region {x : Ax ≥ b, x ≥ 0} is a convex polytope;
//! the objective is linear; the optimum (when finite) is attained
//! at a vertex of the polytope. LP is in **P** — Khachiyan's
//! ellipsoid method (1979) gave the first polynomial-time bound,
//! Karmarkar's interior-point algorithm (1984) made it practical
//! at scale. Modern simplex implementations (GLOP, CLP, HiGHS-LP)
//! exploit sparsity to handle 10⁶-variable LPs in seconds.
//!
//! Stigler's diet problem (1945) is the historical first LP, and
//! the canonical introductory instance. 8 foods with per-serving
//! cost and nutrient profile; 6 nutrient minimums; find the
//! cheapest combination of fractional servings that meets every
//! minimum.
//!
//! See `kb/Architecture/Algorithmic Backbone.md` for the full
//! complexity-class treatment.
//!
//! Why this matters: LP is the foundation of operations research —
//! fractional optimization at the boundary of feasibility. LLMs can
//! parrot the encoding but cannot reliably produce the optimal
//! servings vector. Manual spreadsheet attempts find feasible diets
//! easily but rarely the cheapest one; the optimum sits exactly on
//! the intersection of several active constraints.
//!
//! Ferrox's `GlopLpSuggestor` (OR-Tools GLOP solver) returns the
//! provably optimal real-valued solution in microseconds.
//!
//! Encoding:
//!   - 8 continuous vars (servings per food), lb = 0
//!   - 6 LinearGe constraints (nutrient ≥ minimum)
//!   - Objective: minimize sum(cost · serving)
//!
//! After the solver returns, the scenario re-walks every nutrient
//! constraint and confirms the minimum is met. The objective is
//! recomputed independently and compared to the solver's reported
//! value. Solver lying is detectable.

#[cfg(feature = "with-solver")]
use converge_kernel::{Budget, ContextState, ConvergeResult, Engine, ProposedFact};
#[cfg(feature = "with-solver")]
use converge_pack::ContextKey;

#[derive(Debug, Clone, Copy)]
struct Nutrient {
    name: &'static str,
    minimum: f64,
    unit: &'static str,
}

const NUTRIENTS: &[Nutrient] = &[
    Nutrient {
        name: "calories",
        minimum: 2000.0,
        unit: "kcal",
    },
    Nutrient {
        name: "protein",
        minimum: 55.0,
        unit: "g",
    },
    Nutrient {
        name: "vitamin_c",
        minimum: 60.0,
        unit: "mg",
    },
    Nutrient {
        name: "calcium",
        minimum: 1000.0,
        unit: "mg",
    },
    Nutrient {
        name: "iron",
        minimum: 18.0,
        unit: "mg",
    },
    Nutrient {
        name: "fiber",
        minimum: 25.0,
        unit: "g",
    },
];

#[derive(Debug, Clone, Copy)]
struct Food {
    name: &'static str,
    cost_per_serving: f64, // USD
    // Nutrient values per serving, indexed parallel to NUTRIENTS.
    nutrients: [f64; 6],
}

const FOODS: &[Food] = &[
    Food {
        name: "oatmeal",
        cost_per_serving: 0.50,
        nutrients: [150.0, 6.0, 0.0, 20.0, 1.5, 4.0],
    },
    Food {
        name: "chicken",
        cost_per_serving: 2.40,
        nutrients: [220.0, 42.0, 0.0, 15.0, 1.0, 0.0],
    },
    Food {
        name: "eggs",
        cost_per_serving: 0.70,
        nutrients: [155.0, 13.0, 0.0, 56.0, 1.8, 0.0],
    },
    Food {
        name: "whole_milk",
        cost_per_serving: 0.80,
        nutrients: [149.0, 8.0, 0.0, 276.0, 0.1, 0.0],
    },
    Food {
        name: "broccoli",
        cost_per_serving: 1.10,
        nutrients: [55.0, 3.7, 89.2, 47.0, 0.7, 5.1],
    },
    Food {
        name: "orange",
        cost_per_serving: 0.60,
        nutrients: [62.0, 1.2, 70.0, 52.0, 0.1, 3.0],
    },
    Food {
        name: "spinach",
        cost_per_serving: 1.30,
        nutrients: [23.0, 2.9, 28.1, 99.0, 2.7, 2.2],
    },
    Food {
        name: "brown_rice",
        cost_per_serving: 0.40,
        nutrients: [216.0, 5.0, 0.0, 20.0, 0.8, 3.5],
    },
];

#[cfg(feature = "with-solver")]
const REQUEST_ID: &str = "diet-min-cost";

#[cfg(feature = "with-solver")]
#[derive(Clone, Copy, Debug)]
struct ScenarioProvenance;

#[cfg(feature = "with-solver")]
impl converge_pack::ProvenanceSource for ScenarioProvenance {
    fn as_str(&self) -> &'static str {
        "atelier-lp-diet"
    }
}

#[cfg(feature = "with-solver")]
fn scenario_provenance() -> converge_pack::Provenance {
    converge_pack::ProvenanceSource::provenance(ScenarioProvenance)
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_banner();
    print_problem();

    #[cfg(not(feature = "with-solver"))]
    {
        println!("──────────────────────────────────────────────────────────");
        println!(
            "Solver disabled. Re-run with `--features with-solver` to\n\
             enable ferrox::lp::GlopLpSuggestor (OR-Tools GLOP). No\n\
             heuristic fallback — honest exit."
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
    let request = build_lp_request();
    print_request(&request);

    let mut engine = Engine::with_budget(Budget {
        max_cycles: 4,
        max_facts: 16,
    });
    engine.register_suggestor(ferrox::lp::GlopLpSuggestor);

    let mut ctx = ContextState::new();
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        format!("glop-request:{REQUEST_ID}"),
        request,
        scenario_provenance(),
    ))
    .map_err(|e| format!("seed proposal: {e:?}"))?;

    let result: ConvergeResult = engine.run(ctx).await?;
    print_solution(&result)
}

#[cfg(feature = "with-solver")]
fn build_lp_request() -> ferrox::lp::LpRequest {
    use ferrox::lp::{LpConstraint, LpObjective, LpRequest, LpTerm, LpVariable};

    let var = |i: usize| format!("x_{}", FOODS[i].name);

    let variables: Vec<LpVariable> = (0..FOODS.len())
        .map(|i| LpVariable {
            name: var(i),
            lb: 0.0,
            ub: f64::INFINITY,
        })
        .collect();

    // One constraint per nutrient: sum(amount_per_serving * x_i) >= minimum
    let constraints: Vec<LpConstraint> = NUTRIENTS
        .iter()
        .enumerate()
        .map(|(n_idx, nut)| LpConstraint {
            name: format!("min_{}", nut.name),
            lb: nut.minimum,
            ub: f64::INFINITY,
            terms: FOODS
                .iter()
                .enumerate()
                .map(|(f_idx, food)| LpTerm {
                    var: var(f_idx),
                    coeff: food.nutrients[n_idx],
                })
                .collect(),
        })
        .collect();

    // Objective: minimize total cost.
    let objective = LpObjective {
        terms: FOODS
            .iter()
            .enumerate()
            .map(|(i, food)| LpTerm {
                var: var(i),
                coeff: food.cost_per_serving,
            })
            .collect(),
        maximize: false,
    };

    LpRequest {
        id: REQUEST_ID.to_string(),
        variables,
        constraints,
        objective,
        time_limit_seconds: Some(5.0),
    }
}

fn print_banner() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  LP Diet — ferrox GLOP (cheapest fractional servings)               ║");
    println!("║  atelier-showcase · converge 3.9.1 · ferrox 0.7.1                    ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_problem() {
    println!("Foods");
    println!("─────");
    for food in FOODS {
        println!(
            "  {:<12}  ${:.2}/serving   cal={:>4.0}  prot={:>5.1}g  vitC={:>5.1}mg  Ca={:>6.1}mg  Fe={:>4.1}mg  fiber={:>4.1}g",
            food.name,
            food.cost_per_serving,
            food.nutrients[0],
            food.nutrients[1],
            food.nutrients[2],
            food.nutrients[3],
            food.nutrients[4],
            food.nutrients[5],
        );
    }
    println!();
    println!("Minimum daily requirements");
    println!("──────────────────────────");
    for nut in NUTRIENTS {
        println!("  {:<10}  ≥ {:>7.1} {}", nut.name, nut.minimum, nut.unit);
    }
    println!();
}

#[cfg(feature = "with-solver")]
fn print_request(req: &ferrox::lp::LpRequest) {
    println!("LP model");
    println!("────────");
    println!("  variables:   {} continuous, lb=0", req.variables.len());
    println!(
        "  constraints: {} (nutrient minimums)",
        req.constraints.len()
    );
    println!(
        "  objective:   {} terms ({})",
        req.objective.terms.len(),
        if req.objective.maximize {
            "maximize"
        } else {
            "minimize"
        }
    );
    println!("  solver:      ferrox::lp::GlopLpSuggestor (OR-Tools GLOP)");
    println!();
}

#[cfg(feature = "with-solver")]
fn print_solution(result: &ConvergeResult) -> Result<(), Box<dyn std::error::Error>> {
    println!("Solver outcome");
    println!("──────────────");
    println!(
        "  converged:   {} (cycles: {}, stop: {:?})",
        result.converged, result.cycles, result.stop_reason
    );

    let plan_id = format!("glop-plan:{REQUEST_ID}");
    let plan_fact = result
        .context
        .get(ContextKey::Strategies)
        .iter()
        .find(|f| f.id().as_str() == plan_id)
        .ok_or("no glop-plan fact produced — solver did not run")?;
    let plan = plan_fact.require_payload::<ferrox::lp::LpPlan>()?;

    println!("  status:      {:?}", plan.status);
    println!("  solver:      {}", plan.solver);
    println!("  objective:   ${:.4}/day (reported)", plan.objective_value);
    println!();

    // Pull serving values out by food name.
    let mut servings = vec![0.0; FOODS.len()];
    for (name, val) in &plan.values {
        if let Some(rest) = name.strip_prefix("x_")
            && let Some(idx) = FOODS.iter().position(|f| f.name == rest)
        {
            servings[idx] = *val;
        }
    }

    println!("Chosen diet (servings per day)");
    println!("──────────────────────────────");
    let mut shown = 0;
    for (i, &s) in servings.iter().enumerate() {
        if s > 1e-6 {
            println!(
                "  {:<12}  {:>6.3} servings   = ${:>5.2}/day",
                FOODS[i].name,
                s,
                s * FOODS[i].cost_per_serving
            );
            shown += 1;
        }
    }
    if shown == 0 {
        println!("  (no foods selected — feasibility failure)");
    }
    println!();

    // Independent verification: walk every constraint, confirm minimum
    // met. Recompute cost from servings and compare to solver report.
    let mut recomputed_cost = 0.0;
    for (i, &s) in servings.iter().enumerate() {
        recomputed_cost += s * FOODS[i].cost_per_serving;
    }
    println!("Verification");
    println!("────────────");
    println!("  recomputed cost: ${:.4}/day", recomputed_cost);
    println!(
        "  solver vs recomputed: |Δ| = {:.6} {}",
        (recomputed_cost - plan.objective_value).abs(),
        if (recomputed_cost - plan.objective_value).abs() < 1e-4 {
            "✓"
        } else {
            "✗ MISMATCH"
        }
    );
    for (n_idx, nut) in NUTRIENTS.iter().enumerate() {
        let actual: f64 = (0..FOODS.len())
            .map(|f_idx| servings[f_idx] * FOODS[f_idx].nutrients[n_idx])
            .sum();
        let ok = actual >= nut.minimum - 1e-6;
        println!(
            "  {:<10}  delivered = {:>7.2} {} ≥ {:>6.1} {}  {}",
            nut.name,
            actual,
            nut.unit,
            nut.minimum,
            nut.unit,
            if ok { "✓" } else { "✗ BELOW MINIMUM" }
        );
        if !ok {
            return Err(format!(
                "{} delivered {:.2}{} below minimum {:.1}",
                nut.name, actual, nut.unit, nut.minimum
            )
            .into());
        }
    }
    if (recomputed_cost - plan.objective_value).abs() >= 1e-4 {
        return Err("objective mismatch between solver report and recomputation".into());
    }
    println!();
    println!(
        "✓ ferrox GLOP solved the diet LP at status {:?}; minimums met; cost independently verified.",
        plan.status
    );
    Ok(())
}
