//! Atelier showcase: min-cost network flow transportation.
//!
//! ## Problem class
//!
//! **Min-cost network flow** — a special case of LP whose
//! constraint matrix is **totally unimodular (TU)**, a structural
//! property guaranteeing that the LP relaxation has integer
//! optimal vertices. Formally: directed graph G = (V, E) with arc
//! capacities u_e ∈ Z_≥0, unit costs c_e ∈ Z, and node supplies
//! b_v ∈ Z (sources b_v > 0, sinks b_v < 0, transshipment b_v = 0,
//! with Σ_v b_v = 0). Find flow f: E → Z_≥0 minimizing Σ_e c_e f_e
//! subject to capacity f_e ≤ u_e and conservation at every node.
//!
//! The TU property collapses what would be NP-hard as a generic
//! 0-1 MIP into a **strongly polynomial-time** problem. The
//! **network simplex algorithm** (Dantzig 1951, modern variants
//! by Orlin and others) runs in O(VE log V) or better and
//! produces provably integer optima without branch-and-bound.
//!
//! This is the lesson the scenario makes concrete: **structural
//! insight pays off**. Solving the same problem through a generic
//! LP or MIP works but runs orders of magnitude slower. Knowing
//! the constraint matrix is TU and dispatching to the specialized
//! algorithm is what production OR practitioners do.
//!
//! See `kb/Architecture/Algorithmic Backbone.md` for the full
//! complexity-class treatment.
//!
//! ## The instance
//!
//! 3-echelon supply chain: 3 plants → 4 distribution centers →
//! 6 retailers. Each edge has a capacity and per-unit cost. Find
//! the flow assignment that satisfies all retail demand at
//! minimum total cost.
//!
//! Why min-cost flow shines: this is a graph problem with
//! conservation constraints at every intermediate node. LP can
//! model it but the specialized network simplex algorithm in
//! `MinCostFlowSuggestor` (OR-Tools) is orders of magnitude
//! faster, exploits integrality of the constraint matrix for
//! provably integer solutions, and gives concrete edge-by-edge
//! routing.
//!
//! LLMs cannot reason about flow conservation at scale — they
//! produce plausible-looking ship plans that violate capacity
//! or leave intermediate nodes unbalanced.
//!
//! Encoding (12 nodes, 17 arcs):
//!   - 3 plant nodes with positive supply (production capacity)
//!   - 4 distribution-center nodes with supply = 0 (transshipment)
//!   - 6 retail nodes with negative supply (demand)
//!   - 17 directed arcs (3 plant→DC, 4×3 DC→retail) with per-unit
//!     cost and capacity
//!   - Mode: BalancedMinCost (total supply == total demand,
//!     every retailer fully served)
//!
//! Verification: flow conservation at every node, capacity respect
//! on every arc, recompute total cost from chosen flows and match
//! the solver's reported optimum.

#[cfg(feature = "with-solver")]
use converge_kernel::{Budget, ContextState, ConvergeResult, Engine, ProposedFact};
#[cfg(feature = "with-solver")]
use converge_pack::ContextKey;

// Node ids: plants 1..=3, distribution 11..=14, retailers 21..=26.
// Total supply 176 == total retail demand 176 (balanced flow).
const PLANTS: &[(i32, &str, i64)] = &[
    (1, "plant_alpha", 70),
    (2, "plant_bravo", 58),
    (3, "plant_charlie", 48),
];
const DISTRIBUTION: &[(i32, &str)] = &[
    (11, "dc_north"),
    (12, "dc_south"),
    (13, "dc_east"),
    (14, "dc_west"),
];
const RETAILERS: &[(i32, &str, i64)] = &[
    (21, "store_red", 25),
    (22, "store_blue", 38),
    (23, "store_green", 42),
    (24, "store_yellow", 18),
    (25, "store_violet", 31),
    (26, "store_amber", 22),
];

#[derive(Debug, Clone, Copy)]
struct ArcSpec {
    name: &'static str,
    tail: i32,
    head: i32,
    capacity: i64,
    unit_cost: i64,
}

const ARCS: &[ArcSpec] = &[
    // Plants → DCs
    ArcSpec {
        name: "alpha_to_north",
        tail: 1,
        head: 11,
        capacity: 50,
        unit_cost: 4,
    },
    ArcSpec {
        name: "alpha_to_south",
        tail: 1,
        head: 12,
        capacity: 40,
        unit_cost: 6,
    },
    ArcSpec {
        name: "alpha_to_east",
        tail: 1,
        head: 13,
        capacity: 30,
        unit_cost: 9,
    },
    ArcSpec {
        name: "bravo_to_north",
        tail: 2,
        head: 11,
        capacity: 35,
        unit_cost: 5,
    },
    ArcSpec {
        name: "bravo_to_south",
        tail: 2,
        head: 12,
        capacity: 45,
        unit_cost: 3,
    },
    ArcSpec {
        name: "bravo_to_west",
        tail: 2,
        head: 14,
        capacity: 30,
        unit_cost: 7,
    },
    ArcSpec {
        name: "charlie_to_east",
        tail: 3,
        head: 13,
        capacity: 40,
        unit_cost: 4,
    },
    ArcSpec {
        name: "charlie_to_west",
        tail: 3,
        head: 14,
        capacity: 35,
        unit_cost: 5,
    },
    // DCs → retailers
    ArcSpec {
        name: "north_to_red",
        tail: 11,
        head: 21,
        capacity: 30,
        unit_cost: 3,
    },
    ArcSpec {
        name: "north_to_blue",
        tail: 11,
        head: 22,
        capacity: 40,
        unit_cost: 4,
    },
    ArcSpec {
        name: "south_to_blue",
        tail: 12,
        head: 22,
        capacity: 35,
        unit_cost: 5,
    },
    ArcSpec {
        name: "south_to_green",
        tail: 12,
        head: 23,
        capacity: 50,
        unit_cost: 4,
    },
    ArcSpec {
        name: "south_to_yellow",
        tail: 12,
        head: 24,
        capacity: 25,
        unit_cost: 6,
    },
    ArcSpec {
        name: "east_to_green",
        tail: 13,
        head: 23,
        capacity: 40,
        unit_cost: 3,
    },
    ArcSpec {
        name: "east_to_violet",
        tail: 13,
        head: 25,
        capacity: 35,
        unit_cost: 5,
    },
    ArcSpec {
        name: "west_to_violet",
        tail: 14,
        head: 25,
        capacity: 30,
        unit_cost: 4,
    },
    ArcSpec {
        name: "west_to_amber",
        tail: 14,
        head: 26,
        capacity: 25,
        unit_cost: 6,
    },
];

#[cfg(feature = "with-solver")]
const REQUEST_ID: &str = "supply-chain";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_banner();
    print_problem();

    #[cfg(not(feature = "with-solver"))]
    {
        println!("──────────────────────────────────────────────────────────");
        println!(
            "Solver disabled. Re-run with `--features with-solver` to\n\
             enable ferrox::network_flow::MinCostFlowSuggestor\n\
             (OR-Tools network simplex). Honest exit."
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
    use ferrox::domain_types::NodeId;
    use ferrox::network_flow::{FlowArc, FlowSolveMode, MinCostFlowRequest, NodeSupply};

    let supplies: Vec<NodeSupply> = PLANTS
        .iter()
        .map(|(id, _, supply)| NodeSupply {
            node: NodeId(*id),
            supply: *supply,
        })
        .chain(DISTRIBUTION.iter().map(|(id, _)| NodeSupply {
            node: NodeId(*id),
            supply: 0,
        }))
        .chain(RETAILERS.iter().map(|(id, _, demand)| NodeSupply {
            node: NodeId(*id),
            supply: -*demand,
        }))
        .collect();

    let arcs: Vec<FlowArc> = ARCS
        .iter()
        .map(|a| FlowArc {
            name: a.name.to_string(),
            tail: NodeId(a.tail),
            head: NodeId(a.head),
            capacity: a.capacity,
            unit_cost: a.unit_cost,
        })
        .collect();

    let request = MinCostFlowRequest {
        id: REQUEST_ID.to_string(),
        arcs,
        supplies,
        mode: FlowSolveMode::BalancedMinCost,
    };
    print_request(&request);

    let mut engine = Engine::with_budget(Budget {
        max_cycles: 4,
        max_facts: 16,
    });
    engine.register_suggestor(ferrox::network_flow::MinCostFlowSuggestor);

    let mut ctx = ContextState::new();
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        format!("network-flow-request:{REQUEST_ID}"),
        request,
        "atelier-network-flow-transport",
    ))
    .map_err(|e| format!("seed proposal: {e:?}"))?;

    let result: ConvergeResult = engine.run(ctx).await?;
    print_solution(&result)
}

fn node_name(id: i32) -> &'static str {
    if let Some(p) = PLANTS.iter().find(|(i, _, _)| *i == id) {
        return p.1;
    }
    if let Some(d) = DISTRIBUTION.iter().find(|(i, _)| *i == id) {
        return d.1;
    }
    if let Some(r) = RETAILERS.iter().find(|(i, _, _)| *i == id) {
        return r.1;
    }
    "<unknown>"
}

fn print_banner() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  Min-Cost Network Flow — ferrox (OR-Tools network simplex)         ║");
    println!("║  atelier-showcase · converge 3.9.1 · ferrox 0.7.1                    ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_problem() {
    let total_supply: i64 = PLANTS.iter().map(|(_, _, s)| *s).sum();
    let total_demand: i64 = RETAILERS.iter().map(|(_, _, d)| *d).sum();
    println!("Plants (supply)");
    println!("───────────────");
    for (_, name, supply) in PLANTS {
        println!("  {:<16}  +{}", name, supply);
    }
    println!("  ─────────────────  total {}", total_supply);
    println!();
    println!("Retailers (demand)");
    println!("──────────────────");
    for (_, name, demand) in RETAILERS {
        println!("  {:<16}  -{}", name, demand);
    }
    println!("  ─────────────────  total {}", total_demand);
    println!();
    println!("Arcs (capacity / unit_cost)");
    println!("───────────────────────────");
    for a in ARCS {
        println!(
            "  {:<22} {:<14} → {:<14}  cap = {:>3}   cost = ${}/unit",
            a.name,
            node_name(a.tail),
            node_name(a.head),
            a.capacity,
            a.unit_cost,
        );
    }
    println!();
}

#[cfg(feature = "with-solver")]
fn print_request(req: &ferrox::network_flow::MinCostFlowRequest) {
    println!("Network model");
    println!("─────────────");
    println!(
        "  nodes:  {} ({} plants + {} distribution + {} retailers)",
        req.supplies.len(),
        PLANTS.len(),
        DISTRIBUTION.len(),
        RETAILERS.len(),
    );
    println!("  arcs:   {}", req.arcs.len());
    println!("  mode:   {:?}", req.mode);
    println!("  solver: ferrox::network_flow::MinCostFlowSuggestor");
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

    let plan_id = format!("network-flow-plan-ortools:{REQUEST_ID}");
    let plan_fact = result
        .context
        .get(ContextKey::Strategies)
        .iter()
        .find(|f| f.id().as_str() == plan_id)
        .ok_or("no network-flow-plan fact produced")?;
    let plan = plan_fact.require_payload::<ferrox::network_flow::MinCostFlowPlan>()?;

    println!("  status:      {:?}", plan.status);
    println!("  solver:      {}", plan.solver);
    println!(
        "  flow:        fulfilled {} / expected {} ({:.1}%)",
        plan.fulfilled_flow,
        plan.expected_flow,
        plan.fulfillment_ratio * 100.0,
    );
    println!("  cost:        ${} (reported)", plan.optimal_cost);
    println!();

    println!("Chosen flows");
    println!("────────────");
    for arc in &plan.arcs {
        if arc.flow > 0 {
            println!(
                "  {:<22} {:<14} → {:<14}   {:>3} / cap {:>3}  @ ${}/unit = ${}",
                arc.name,
                node_name(arc.tail.0),
                node_name(arc.head.0),
                arc.flow,
                arc.capacity,
                arc.unit_cost,
                arc.flow * arc.unit_cost,
            );
        }
    }
    println!();

    // Independent verification: flow conservation at every node,
    // capacity respect on every arc, recompute total cost.
    println!("Verification");
    println!("────────────");
    let mut net: std::collections::BTreeMap<i32, i64> = std::collections::BTreeMap::new();
    let mut recomputed: i64 = 0;
    for arc in &plan.arcs {
        if arc.flow < 0 || arc.flow > arc.capacity {
            return Err(format!(
                "arc {} has flow {} outside [0, capacity={}]",
                arc.name, arc.flow, arc.capacity,
            )
            .into());
        }
        *net.entry(arc.tail.0).or_insert(0) -= arc.flow;
        *net.entry(arc.head.0).or_insert(0) += arc.flow;
        recomputed += arc.flow * arc.unit_cost;
    }

    let mut all_ok = true;
    // Plants must emit exactly `supply` units (net flow = -supply).
    for (id, name, supply) in PLANTS {
        let n = net.get(id).copied().unwrap_or(0);
        let ok = n == -supply;
        if !ok {
            all_ok = false;
        }
        println!(
            "  {:<16}  emitted {:>4}  expected {:>4}  {}",
            name,
            -n,
            supply,
            if ok { "✓" } else { "✗" }
        );
    }
    // DCs must balance (in == out).
    for (id, name) in DISTRIBUTION {
        let n = net.get(id).copied().unwrap_or(0);
        let ok = n == 0;
        if !ok {
            all_ok = false;
        }
        let inflow: i64 = plan
            .arcs
            .iter()
            .filter(|a| a.head.0 == *id)
            .map(|a| a.flow)
            .sum();
        println!(
            "  {:<16}  in = out = {:>4}  net = {:>+4}  {}",
            name,
            inflow,
            n,
            if ok { "✓" } else { "✗" }
        );
    }
    // Retailers must receive exactly `demand` units.
    for (id, name, demand) in RETAILERS {
        let n = net.get(id).copied().unwrap_or(0);
        let ok = n == *demand;
        if !ok {
            all_ok = false;
        }
        println!(
            "  {:<16}  received {:>4}  expected {:>4}  {}",
            name,
            n,
            demand,
            if ok { "✓" } else { "✗" }
        );
    }
    let cost_ok = recomputed == plan.optimal_cost;
    println!(
        "  recomputed cost: ${}    solver report ${}   {}",
        recomputed,
        plan.optimal_cost,
        if cost_ok { "✓" } else { "✗" }
    );
    if !all_ok || !cost_ok {
        return Err("flow conservation or cost mismatch — solver disagreement".into());
    }
    println!();
    println!(
        "✓ ferrox network simplex solved the 3-echelon flow at status {:?}; conservation + cost verified.",
        plan.status
    );
    Ok(())
}
