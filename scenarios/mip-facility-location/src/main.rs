//! Atelier showcase: MIP facility location via ferrox HiGHS.
//!
//! ## Problem class
//!
//! **Mixed-integer linear programming (MIP)** — LP augmented with
//! integrality constraints, formally:
//!
//! ```text
//!     minimize    c^T x  +  d^T y
//!     subject to  A x + B y  ≥  b
//!                 x  ≥  0,   x ∈ R^n
//!                 y  ∈  Z^m  (or {0,1}^m for 0-1 MIP)
//! ```
//!
//! The integer variables y make the problem **NP-hard** (3-SAT
//! reduces to 0-1 MIP via clause encoding). The canonical solution
//! method is **branch-and-bound**: solve the LP relaxation
//! (drop integrality), branch on a fractional integer variable,
//! prune subtrees whose LP bound is worse than the incumbent,
//! repeat. Modern solvers add cutting planes (Gomory, MIR,
//! lift-and-project) that tighten the relaxation. The **MIP gap** —
//! the relative distance between the best integer solution found
//! and the best LP bound — measures progress; gap = 0 proves
//! optimality.
//!
//! Why MIP and not LP: many real-world decisions are inherently
//! discrete — open/close a warehouse, hire/don't-hire, assign one
//! shift to one worker — and the LP relaxation produces fractional
//! decisions that are operationally meaningless ("open 0.3 of
//! warehouse A"). The MIP/LP split is the whole point.
//!
//! Why HiGHS, not the LP solver alone: HiGHS-MIP is a full
//! branch-and-bound implementation with cutting planes and presolve.
//! Solving an MIP through a generic LP solver requires the same
//! algorithm to be implemented at the application layer; HiGHS
//! ships it as a single primitive.
//!
//! See `kb/Architecture/Algorithmic Backbone.md` for the full
//! complexity-class treatment.
//!
//! ## The classical UFLP instance
//!
//! The Uncapacitated (here: capacitated) Facility Location
//! Problem: given candidate warehouses with fixed opening costs
//! and per-customer shipping costs, decide WHICH warehouses to
//! open AND how much flow to ship from each open warehouse to
//! each customer such that all customer demand is met at minimum
//! total cost.
//!
//! This is the canonical demonstration of MIP's value over pure
//! LP: the "open or not" decision is binary, and that single
//! integrality constraint creates a combinatorial choice the LP
//! relaxation cannot resolve. Naive LP gives fractional warehouse
//! openings (open 0.3 of warehouse A + 0.7 of B) that are useless
//! operationally. The MIP branch-and-bound finds the true integer
//! optimum and proves it via the LP gap.
//!
//! Why this matters for LLMs: combinatorial selection over a
//! continuous shipping plan with capacity constraints. LLMs can
//! identify the structure but cannot reliably enumerate the
//! integer-feasible region. Picking 3 of 5 warehouses with the
//! right shipping plan needs real branch-and-bound.
//!
//! Encoding:
//!   - 5 binary vars `open_w` ∈ {0, 1}.
//!   - 5×8 continuous vars `ship_wc` ∈ [0, capacity_w].
//!   - 8 demand-met constraints: sum_w ship_wc == demand_c.
//!   - 5 capacity-when-open big-M constraints: sum_c ship_wc ≤ capacity_w · open_w.
//!   - Objective: minimize sum_w(open_cost · open_w) + sum_wc(ship_cost · ship_wc).
//!
//! Independent verification: every customer's demand satisfied
//! exactly, no flow from unopened warehouses, recomputed objective
//! matches solver report.

#![allow(clippy::needless_range_loop)] // matrix construction is clearer with indices.

#[cfg(feature = "with-solver")]
use converge_kernel::{Budget, ContextState, ConvergeResult, Engine, ProposedFact};
#[cfg(feature = "with-solver")]
use converge_pack::ContextKey;

#[derive(Debug, Clone, Copy)]
struct Warehouse {
    name: &'static str,
    open_cost: f64,
    capacity: f64,
}

const WAREHOUSES: &[Warehouse] = &[
    Warehouse {
        name: "wh_north",
        open_cost: 4500.0,
        capacity: 60.0,
    },
    Warehouse {
        name: "wh_south",
        open_cost: 3200.0,
        capacity: 40.0,
    },
    Warehouse {
        name: "wh_east",
        open_cost: 5800.0,
        capacity: 80.0,
    },
    Warehouse {
        name: "wh_west",
        open_cost: 4100.0,
        capacity: 55.0,
    },
    Warehouse {
        name: "wh_central",
        open_cost: 6900.0,
        capacity: 100.0,
    },
];

#[derive(Debug, Clone, Copy)]
struct Customer {
    name: &'static str,
    demand: f64,
}

const CUSTOMERS: &[Customer] = &[
    Customer {
        name: "c_alpha",
        demand: 18.0,
    },
    Customer {
        name: "c_bravo",
        demand: 24.0,
    },
    Customer {
        name: "c_charlie",
        demand: 15.0,
    },
    Customer {
        name: "c_delta",
        demand: 32.0,
    },
    Customer {
        name: "c_echo",
        demand: 12.0,
    },
    Customer {
        name: "c_foxtrot",
        demand: 28.0,
    },
    Customer {
        name: "c_golf",
        demand: 22.0,
    },
    Customer {
        name: "c_hotel",
        demand: 19.0,
    },
];

// Ship cost per unit from warehouse w to customer c. Rows = warehouses,
// cols = customers. Geographic-ish — central is well-connected but
// expensive to open; north/south are cheap to open but have higher
// long-haul costs for distant customers.
const SHIP_COSTS: [[f64; 8]; 5] = [
    // alpha  bravo  charlie delta  echo   foxtrot golf   hotel
    [3.20, 4.80, 5.10, 6.20, 4.40, 8.10, 7.30, 5.90], // wh_north
    [6.80, 3.50, 4.20, 5.40, 6.10, 3.90, 4.80, 6.70], // wh_south
    [5.20, 6.10, 2.80, 3.30, 6.40, 5.20, 3.10, 3.60], // wh_east
    [4.10, 5.50, 6.20, 4.80, 3.20, 6.40, 5.90, 4.20], // wh_west
    [3.80, 3.90, 3.60, 3.70, 3.80, 3.60, 3.70, 3.80], // wh_central
];

#[cfg(feature = "with-solver")]
const REQUEST_ID: &str = "facility-location";

#[cfg(feature = "with-solver")]
#[derive(Clone, Copy, Debug)]
struct ScenarioProvenance;

#[cfg(feature = "with-solver")]
impl converge_pack::ProvenanceSource for ScenarioProvenance {
    fn as_str(&self) -> &'static str {
        "atelier-mip-facility-location"
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
             enable ferrox::mip::HighsMipSuggestor (HiGHS branch-and-\n\
             bound). No heuristic fallback — honest exit."
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
    let request = build_mip_request();
    print_request(&request);

    let mut engine = Engine::with_budget(Budget {
        max_cycles: 4,
        max_facts: 16,
    });
    engine.register_suggestor(ferrox::mip::HighsMipSuggestor);

    let mut ctx = ContextState::new();
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        format!("mip-request:{REQUEST_ID}"),
        request,
        scenario_provenance(),
    ))
    .map_err(|e| format!("seed proposal: {e:?}"))?;

    let result: ConvergeResult = engine.run(ctx).await?;
    print_solution(&result)
}

#[cfg(feature = "with-solver")]
fn build_mip_request() -> ferrox::mip::MipRequest {
    use ferrox::mip::{MipConstraint, MipObjective, MipRequest, MipTerm, MipVariable, VarKind};

    let open_var = |w: usize| format!("open_{}", WAREHOUSES[w].name);
    let ship_var =
        |w: usize, c: usize| format!("ship_{}_{}", WAREHOUSES[w].name, CUSTOMERS[c].name);

    let mut variables: Vec<MipVariable> = Vec::new();
    for w in 0..WAREHOUSES.len() {
        variables.push(MipVariable {
            name: open_var(w),
            lb: 0.0,
            ub: 1.0,
            kind: VarKind::Binary,
        });
    }
    for w in 0..WAREHOUSES.len() {
        for c in 0..CUSTOMERS.len() {
            variables.push(MipVariable {
                name: ship_var(w, c),
                lb: 0.0,
                ub: WAREHOUSES[w].capacity,
                kind: VarKind::Continuous,
            });
        }
    }

    let mut constraints: Vec<MipConstraint> = Vec::new();

    // Demand-met: sum_w ship_wc == demand_c.
    for (c, cust) in CUSTOMERS.iter().enumerate() {
        constraints.push(MipConstraint {
            name: format!("demand_{}", cust.name),
            lb: cust.demand,
            ub: cust.demand,
            terms: (0..WAREHOUSES.len())
                .map(|w| MipTerm {
                    var: ship_var(w, c),
                    coeff: 1.0,
                })
                .collect(),
        });
    }

    // Capacity-when-open: sum_c ship_wc - capacity_w · open_w ≤ 0.
    // Equivalently: sum_c ship_wc ≤ capacity_w · open_w.
    for (w, wh) in WAREHOUSES.iter().enumerate() {
        let mut terms: Vec<MipTerm> = (0..CUSTOMERS.len())
            .map(|c| MipTerm {
                var: ship_var(w, c),
                coeff: 1.0,
            })
            .collect();
        terms.push(MipTerm {
            var: open_var(w),
            coeff: -wh.capacity,
        });
        constraints.push(MipConstraint {
            name: format!("cap_{}", wh.name),
            lb: f64::NEG_INFINITY,
            ub: 0.0,
            terms,
        });
    }

    // Objective: minimize fixed open + variable shipping.
    let mut obj_terms: Vec<MipTerm> = Vec::new();
    for (w, wh) in WAREHOUSES.iter().enumerate() {
        obj_terms.push(MipTerm {
            var: open_var(w),
            coeff: wh.open_cost,
        });
    }
    for w in 0..WAREHOUSES.len() {
        for c in 0..CUSTOMERS.len() {
            obj_terms.push(MipTerm {
                var: ship_var(w, c),
                coeff: SHIP_COSTS[w][c],
            });
        }
    }

    MipRequest {
        id: REQUEST_ID.to_string(),
        variables,
        constraints,
        objective: MipObjective {
            terms: obj_terms,
            maximize: false,
        },
        time_limit_seconds: Some(10.0),
        mip_gap_tolerance: Some(1e-6),
    }
}

fn print_banner() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  MIP Facility Location — ferrox HiGHS                                ║");
    println!("║  atelier-showcase · converge 3.9.1 · ferrox 0.7.1                    ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_problem() {
    println!("Warehouses (open + capacity)");
    println!("────────────────────────────");
    for w in WAREHOUSES {
        println!(
            "  {:<12}  open_cost = ${:>6.0}   capacity = {:>5.0}",
            w.name, w.open_cost, w.capacity
        );
    }
    println!();
    println!("Customers (demand)");
    println!("──────────────────");
    let total_demand: f64 = CUSTOMERS.iter().map(|c| c.demand).sum();
    for c in CUSTOMERS {
        println!("  {:<10}  demand = {:>4.0}", c.name, c.demand);
    }
    println!("  ──────────────  ────────────");
    println!("  total              {:>4.0}", total_demand);
    println!();
    println!("Shipping cost ($/unit)");
    println!("──────────────────────");
    print!("              ");
    for c in CUSTOMERS {
        print!(" {:>6}", c.name.trim_start_matches("c_"));
    }
    println!();
    for (w, wh) in WAREHOUSES.iter().enumerate() {
        print!("  {:<12}", wh.name);
        for c in 0..CUSTOMERS.len() {
            print!(" {:>6.2}", SHIP_COSTS[w][c]);
        }
        println!();
    }
    println!();
}

#[cfg(feature = "with-solver")]
fn print_request(req: &ferrox::mip::MipRequest) {
    let binary = req
        .variables
        .iter()
        .filter(|v| matches!(v.kind, ferrox::mip::VarKind::Binary))
        .count();
    let cont = req
        .variables
        .iter()
        .filter(|v| matches!(v.kind, ferrox::mip::VarKind::Continuous))
        .count();
    println!("MIP model");
    println!("─────────");
    println!(
        "  variables:   {} ({binary} binary + {cont} continuous)",
        req.variables.len()
    );
    println!("  constraints: {}", req.constraints.len());
    println!("  solver:      ferrox::mip::HighsMipSuggestor (HiGHS branch-and-bound)");
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

    let plan_id = format!("mip-plan:{REQUEST_ID}");
    let plan_fact = result
        .context
        .get(ContextKey::Strategies)
        .iter()
        .find(|f| f.id().as_str() == plan_id)
        .ok_or("no mip-plan fact produced")?;
    let plan = plan_fact.require_payload::<ferrox::mip::MipPlan>()?;

    println!("  status:      {:?}", plan.status);
    println!("  solver:      {}", plan.solver);
    println!("  mip_gap:     {:.6}", plan.mip_gap);
    println!("  objective:   ${:.2} (reported)", plan.objective_value);
    println!();

    // Extract decision variables.
    let mut opened = vec![false; WAREHOUSES.len()];
    let mut ships = vec![vec![0.0; CUSTOMERS.len()]; WAREHOUSES.len()];
    for (name, val) in &plan.values {
        if let Some(rest) = name.strip_prefix("open_") {
            if let Some(idx) = WAREHOUSES.iter().position(|w| w.name == rest) {
                opened[idx] = *val > 0.5;
            }
        } else if let Some(rest) = name.strip_prefix("ship_") {
            // ship_<wh_name>_<cust_name>
            let mut parts = None;
            for (w_idx, wh) in WAREHOUSES.iter().enumerate() {
                let prefix = format!("{}_", wh.name);
                if let Some(cust_part) = rest.strip_prefix(&prefix)
                    && let Some(c_idx) = CUSTOMERS.iter().position(|c| c.name == cust_part)
                {
                    parts = Some((w_idx, c_idx));
                    break;
                }
            }
            if let Some((w, c)) = parts {
                ships[w][c] = *val;
            }
        }
    }

    println!("Solution");
    println!("────────");
    let mut open_total = 0.0_f64;
    let mut ship_total = 0.0_f64;
    for (w, wh) in WAREHOUSES.iter().enumerate() {
        if opened[w] {
            open_total += wh.open_cost;
            let used: f64 = ships[w].iter().sum();
            println!(
                "  OPEN  {:<12}  fixed = ${:.0}   used capacity {:.1}/{:.0}",
                wh.name, wh.open_cost, used, wh.capacity
            );
            for (c, cust) in CUSTOMERS.iter().enumerate() {
                if ships[w][c] > 1e-6 {
                    let cost = ships[w][c] * SHIP_COSTS[w][c];
                    ship_total += cost;
                    println!(
                        "          → {:<10} {:>5.1} units @ ${:.2}  = ${:.2}",
                        cust.name, ships[w][c], SHIP_COSTS[w][c], cost
                    );
                }
            }
        } else {
            println!("  closed {:<12}", wh.name);
        }
    }
    println!();
    println!(
        "  totals:   open = ${:.2}   shipping = ${:.2}   sum = ${:.2}",
        open_total,
        ship_total,
        open_total + ship_total
    );
    println!();

    // Verification: demand met exactly, no ship from closed wh,
    // recomputed objective matches solver report.
    println!("Verification");
    println!("────────────");
    let recomputed = open_total + ship_total;
    let delta = (recomputed - plan.objective_value).abs();
    println!(
        "  recomputed objective: ${:.4}    |Δ vs solver| = {:.6}  {}",
        recomputed,
        delta,
        if delta < 1e-3 { "✓" } else { "✗ MISMATCH" }
    );
    for (c, cust) in CUSTOMERS.iter().enumerate() {
        let received: f64 = (0..WAREHOUSES.len()).map(|w| ships[w][c]).sum();
        let ok = (received - cust.demand).abs() < 1e-3;
        println!(
            "  {:<10}  received {:>5.1} of {:>4.0}  {}",
            cust.name,
            received,
            cust.demand,
            if ok { "✓" } else { "✗ DEMAND MISMATCH" }
        );
        if !ok {
            return Err(format!(
                "{} received {:.2}, demand {:.0}",
                cust.name, received, cust.demand
            )
            .into());
        }
    }
    for (w, wh) in WAREHOUSES.iter().enumerate() {
        let used: f64 = ships[w].iter().sum();
        if used > 1e-6 && !opened[w] {
            return Err(format!(
                "wh {} closed but ships {:.2} — solver violated capacity-when-open",
                wh.name, used
            )
            .into());
        }
        if used > wh.capacity + 1e-3 {
            return Err(format!(
                "wh {} ships {:.2} > capacity {:.0}",
                wh.name, used, wh.capacity
            )
            .into());
        }
    }
    if delta >= 1e-3 {
        return Err("recomputed objective doesn't match solver".into());
    }
    println!();
    println!(
        "✓ ferrox HiGHS solved the facility-location MIP at status {:?}; all checks passed.",
        plan.status
    );
    Ok(())
}
