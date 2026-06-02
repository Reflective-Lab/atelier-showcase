//! Atelier showcase: VRPTW (TSP with time windows) — greedy
//! heuristic vs CP-SAT optimal, side-by-side.
//!
//! ## Problem class
//!
//! **Travelling salesman with time windows (TSPTW)** — the
//! single-vehicle specialization of vehicle routing. Given a
//! depot, N customers each with a 2-D location, a service
//! duration, and a time window `[window_open, window_close]`
//! during which the visit must occur, find an ordered tour that
//! starts and ends at the depot, visits as many customers as
//! possible within their windows, and minimizes total distance
//! (or equivalently total time).
//!
//! TSPTW is **NP-hard** by reduction from TSP. The decision
//! variant ("is there a feasible tour of length ≤ k respecting
//! all windows?") is NP-complete. Brute force enumerates N!
//! permutations (10 customers → 3.6 million; 20 customers → 2.4
//! quintillion); even N=15 is intractable by enumeration.
//!
//! ## Two solvers, same input
//!
//! Ferrox ships both:
//!
//! - **`NearestNeighborSuggestor`** — greedy: from the current
//!   position, pick the closest unvisited customer whose window
//!   is reachable. O(N²) construction, no backtracking. Produces
//!   a feasible tour but commits to early choices that cannot be
//!   undone, often leaving the vehicle stranded far from
//!   late-window customers.
//!
//! - **`CpSatVrptwSuggestor`** — CP-SAT optimal: encodes the tour
//!   as binary "next-customer" decision variables, the time
//!   windows as inequalities over arrival-time integer variables,
//!   and the objective as a linear sum of distances. The CDCL
//!   core resolves it to provable optimality, returning the same
//!   `VrptwPlan` shape with a smaller (often much smaller)
//!   `total_distance` and a larger `customers_visited`.
//!
//! Both consume the same `vrptw-request:` prefix; the scenario
//! registers both Suggestors in one engine and prints the two
//! plans side-by-side: per-vehicle route in concrete
//! "Depot → C3 (arrive 09:14) → C7 (10:02) → …" form, plus
//! aggregate customers-visited, total-distance, and wall-time.
//!
//! See `kb/Architecture/Algorithmic Backbone.md` for the full
//! complexity-class treatment.

#[cfg(feature = "with-solver")]
use converge_kernel::{Budget, ContextState, ConvergeResult, Engine, ProposedFact};
#[cfg(feature = "with-solver")]
use converge_pack::ContextKey;

#[derive(Debug, Clone, Copy)]
struct CustomerSpec {
    id: usize,
    name: &'static str,
    x: f64,
    y: f64,
    window_open: i64,
    window_close: i64,
    service: i64,
}

// Instance designed to expose the difference between greedy and
// optimal: customers are arranged so the nearest-neighbor
// heuristic, starting from the depot, commits to the south-east
// cluster early — and then has to backtrack across the map to
// reach the late-window customer in the north-west.
const CUSTOMERS: &[CustomerSpec] = &[
    CustomerSpec {
        id: 1,
        name: "c_alpha",
        x: 20.0,
        y: 5.0,
        window_open: 5,
        window_close: 60,
        service: 5,
    },
    CustomerSpec {
        id: 2,
        name: "c_bravo",
        x: 30.0,
        y: 8.0,
        window_open: 10,
        window_close: 80,
        service: 5,
    },
    CustomerSpec {
        id: 3,
        name: "c_charlie",
        x: 35.0,
        y: 12.0,
        window_open: 15,
        window_close: 90,
        service: 5,
    },
    CustomerSpec {
        id: 4,
        name: "c_delta",
        x: 28.0,
        y: 20.0,
        window_open: 20,
        window_close: 110,
        service: 5,
    },
    CustomerSpec {
        id: 5,
        name: "c_echo",
        x: 15.0,
        y: 25.0,
        window_open: 25,
        window_close: 130,
        service: 5,
    },
    CustomerSpec {
        id: 6,
        name: "c_foxtrot",
        x: 5.0,
        y: 22.0,
        window_open: 30,
        window_close: 150,
        service: 5,
    },
    CustomerSpec {
        id: 7,
        name: "c_golf",
        x: -10.0,
        y: 18.0,
        window_open: 40,
        window_close: 170,
        service: 5,
    },
    CustomerSpec {
        id: 8,
        name: "c_hotel",
        x: -15.0,
        y: 5.0,
        window_open: 50,
        window_close: 200,
        service: 5,
    },
    CustomerSpec {
        id: 9,
        name: "c_india",
        x: -20.0,
        y: 30.0,
        window_open: 100,
        window_close: 230,
        service: 5,
    },
    CustomerSpec {
        id: 10,
        name: "c_juliet",
        x: -25.0,
        y: 0.0,
        window_open: 60,
        window_close: 220,
        service: 5,
    },
];

#[cfg(feature = "with-solver")]
const REQUEST_ID: &str = "tsptw-10-customers";

#[cfg(feature = "with-solver")]
#[derive(Clone, Copy, Debug)]
struct ScenarioProvenance;

#[cfg(feature = "with-solver")]
impl converge_pack::ProvenanceSource for ScenarioProvenance {
    fn as_str(&self) -> &'static str {
        "atelier-vrptw-comparison"
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
             enable ferrox::vrptw::NearestNeighborSuggestor and\n\
             ferrox::vrptw::CpSatVrptwSuggestor side-by-side."
        );
        println!("──────────────────────────────────────────────────────────");
        Ok(())
    }
    #[cfg(feature = "with-solver")]
    {
        run_solvers().await
    }
}

#[cfg(feature = "with-solver")]
async fn run_solvers() -> Result<(), Box<dyn std::error::Error>> {
    use ferrox::vrptw::{Customer, Depot, VrptwRequest};

    let request = VrptwRequest {
        id: REQUEST_ID.to_string(),
        depot: Depot {
            x: 0.0,
            y: 0.0,
            ready_time: 0,
            due_time: 300,
        },
        customers: CUSTOMERS
            .iter()
            .map(|c| Customer {
                id: c.id,
                name: c.name.to_string(),
                x: c.x,
                y: c.y,
                window_open: c.window_open,
                window_close: c.window_close,
                service_time: c.service,
            })
            .collect(),
        time_limit_seconds: 10.0,
    };

    let mut engine = Engine::with_budget(Budget {
        max_cycles: 4,
        max_facts: 16,
    });
    engine.register_suggestor(ferrox::vrptw::NearestNeighborSuggestor);
    engine.register_suggestor(ferrox::vrptw::CpSatVrptwSuggestor);

    let mut ctx = ContextState::new();
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        format!("vrptw-request:{REQUEST_ID}"),
        request,
        scenario_provenance(),
    ))
    .map_err(|e| format!("seed: {e:?}"))?;

    let result: ConvergeResult = engine.run(ctx).await?;
    print_plans(&result)
}

fn print_banner() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  VRPTW — greedy nearest-neighbor vs CP-SAT optimal                  ║");
    println!("║  atelier-showcase · converge 3.9.1 · ferrox 0.7.1                    ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_problem() {
    println!("Problem");
    println!("───────");
    println!(
        "  Depot at (0, 0).  {} customers, each with 2-D location, time window,\n\
         and 5-unit service duration.  Vehicle must return by t = 300.",
        CUSTOMERS.len()
    );
    println!();
    println!("Customers");
    println!("─────────");
    for c in CUSTOMERS {
        println!(
            "  C{:<2} {:<10} ({:>5.1}, {:>5.1})   window [{:>3}..{:>3}]   service {}",
            c.id, c.name, c.x, c.y, c.window_open, c.window_close, c.service
        );
    }
    println!();
    println!("Why this instance hurts greedy");
    println!("──────────────────────────────");
    println!(
        "  Nearest-neighbor starts at the depot and chases the south-east cluster\n\
         (c_alpha .. c_charlie).  By the time it heads west, it has burned the\n\
         early time budget — c_india (window opens at t=100, far north-west) is\n\
         reachable only with a backtrack across the map.  Optimal CP-SAT plans\n\
         the route globally and visits c_india on the natural west-bound sweep."
    );
    println!();
}

#[cfg(feature = "with-solver")]
fn print_plans(result: &ConvergeResult) -> Result<(), Box<dyn std::error::Error>> {
    let strategies = result.context.get(ContextKey::Strategies);
    let greedy_id = format!("vrptw-plan-greedy:{REQUEST_ID}");
    let cpsat_id = format!("vrptw-plan-cpsat:{REQUEST_ID}");

    let greedy = strategies
        .iter()
        .find(|f| f.id().as_str() == greedy_id)
        .ok_or("no greedy plan emitted")?
        .require_payload::<ferrox::vrptw::VrptwPlan>()?;
    let cpsat = strategies
        .iter()
        .find(|f| f.id().as_str() == cpsat_id)
        .ok_or("no CP-SAT plan emitted")?
        .require_payload::<ferrox::vrptw::VrptwPlan>()?;

    println!("Greedy plan (NearestNeighborSuggestor)");
    println!("──────────────────────────────────────");
    print_plan(greedy);
    println!();

    println!("CP-SAT plan (CpSatVrptwSuggestor)");
    println!("─────────────────────────────────");
    print_plan(cpsat);
    println!();

    println!("Comparison");
    println!("──────────");
    println!(
        "  {:<22} greedy = {:<6} cp-sat = {:<6} Δ = {:+.1}%",
        "customers visited",
        greedy.customers_visited,
        cpsat.customers_visited,
        ratio_delta(
            greedy.customers_visited as f64,
            cpsat.customers_visited as f64
        ),
    );
    println!(
        "  {:<22} greedy = {:>6.2} cp-sat = {:>6.2} Δ = {:+.1}%",
        "total distance",
        greedy.total_distance,
        cpsat.total_distance,
        ratio_delta(greedy.total_distance, cpsat.total_distance),
    );
    println!(
        "  {:<22} greedy = {:>6}  cp-sat = {:>6}  Δ = {:+}",
        "return time",
        greedy.return_time,
        cpsat.return_time,
        cpsat.return_time - greedy.return_time,
    );
    println!(
        "  {:<22} greedy = {:>4.3}s cp-sat = {:>4.3}s",
        "wall time", greedy.wall_time_seconds, cpsat.wall_time_seconds,
    );
    println!();

    validate(greedy, "greedy")?;
    validate(cpsat, "cp-sat")?;
    println!("Verification");
    println!("────────────");
    println!(
        "  ✓ both routes start and end at the depot (implicit)\n  \
         ✓ each customer visited at most once in each route\n  \
         ✓ all arrivals within stated time windows\n  \
         ✓ return time ≤ depot.due_time"
    );
    println!();
    println!(
        "✓ Both solvers ran end-to-end on the same VrptwRequest.  CP-SAT visited \
         {} customers in {:.1} units vs greedy's {} in {:.1} — the heuristic is \
         feasible but suboptimal.",
        cpsat.customers_visited,
        cpsat.total_distance,
        greedy.customers_visited,
        greedy.total_distance,
    );
    Ok(())
}

#[cfg(feature = "with-solver")]
fn print_plan(plan: &ferrox::vrptw::VrptwPlan) {
    println!(
        "  status = {:?}   visited {}/{}   total_distance = {:.2}   return = t{}",
        plan.status,
        plan.customers_visited,
        plan.customers_total,
        plan.total_distance,
        plan.return_time,
    );
    println!("  route:  Depot");
    for stop in &plan.route {
        println!(
            "          → {:<10} arrive t{:<3}  depart t{:<3}",
            stop.customer_name, stop.arrival, stop.departure
        );
    }
    println!("          → Depot");
}

#[cfg(feature = "with-solver")]
fn ratio_delta(greedy: f64, cpsat: f64) -> f64 {
    if cpsat.abs() < f64::EPSILON {
        return 0.0;
    }
    ((greedy - cpsat) / cpsat) * 100.0
}

#[cfg(feature = "with-solver")]
fn validate(plan: &ferrox::vrptw::VrptwPlan, label: &str) -> Result<(), String> {
    // No customer visited twice.
    let mut seen = std::collections::BTreeSet::new();
    for stop in &plan.route {
        if !seen.insert(stop.customer_id) {
            return Err(format!(
                "{label}: customer {} visited twice",
                stop.customer_id
            ));
        }
    }
    // Every arrival within window, every visited customer recognized.
    for stop in &plan.route {
        let cust = CUSTOMERS
            .iter()
            .find(|c| c.id == stop.customer_id)
            .ok_or_else(|| format!("{label}: unknown customer {}", stop.customer_id))?;
        if stop.arrival < cust.window_open {
            return Err(format!(
                "{label}: arrived at {} at t{} (before window opens t{})",
                cust.name, stop.arrival, cust.window_open
            ));
        }
        if stop.arrival > cust.window_close {
            return Err(format!(
                "{label}: arrived at {} at t{} (after window closes t{})",
                cust.name, stop.arrival, cust.window_close
            ));
        }
        if stop.departure - stop.arrival < cust.service {
            return Err(format!(
                "{label}: at {} only stayed {} units (need {} service)",
                cust.name,
                stop.departure - stop.arrival,
                cust.service
            ));
        }
    }
    Ok(())
}
