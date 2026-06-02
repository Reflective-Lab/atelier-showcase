//! Atelier showcase: Fisher-Thompson 6×6 job-shop benchmark.
//!
//! ## Problem class
//!
//! **Job-shop scheduling** with makespan objective is one of the
//! foundational **NP-hard** combinatorial optimization problems
//! (Garey, Johnson & Sethi 1976). Formally: given m machines and
//! n jobs each consisting of an ordered sequence of operations
//! with fixed machine assignments and durations, find a schedule
//! that respects (a) precedence within each job and (b) no two
//! operations overlap on the same machine, while minimizing
//! makespan max_o end(o). The decision form ("does a schedule of
//! makespan ≤ k exist?") is NP-complete.
//!
//! The CP-SAT encoding uses **interval variables** — a CP-SAT
//! primitive that represents a contiguous time block ⟨start, end,
//! duration⟩ — and the **`NoOverlap` global constraint** — a
//! polynomial-time propagator that enforces mutual exclusion on a
//! resource. This pairing is exactly what makes CP-SAT the right
//! tool: the scheduling primitives match the problem structure,
//! and the underlying CDCL core handles the combinatorial branching
//! with conflict-driven clause learning.
//!
//! See `kb/Architecture/Algorithmic Backbone.md` for the full
//! complexity-class treatment.
//!
//! ## The benchmark
//!
//! The `ft06` instance, introduced by Fisher & Thompson (1963), is
//! the smallest standard job-shop scheduling benchmark and the
//! canonical NP-hard scheduling test. Six jobs, each a sequence of
//! six operations on six machines with given durations. Schedule
//! all operations so:
//!   - Operations within a job execute in their given order, and
//!   - No two operations overlap on the same machine.
//!
//! Minimize the makespan (time the last operation finishes).
//!
//! The **proven optimum is 55**. Any solver that converges to a
//! makespan of 55 with valid schedule is correct. Greedy heuristics
//! often produce 70-80; LLMs reliably produce schedules with
//! machine conflicts. Ferrox's `CpSatJobShopSuggestor` uses
//! interval variables + NoOverlap (the exact CP-SAT primitives
//! the problem is designed around) and solves it in milliseconds.
//!
//! After the solver returns, the scenario re-validates the
//! schedule independently:
//!   - Every operation appears exactly once and has start + duration = end
//!   - Within each job, operations don't overlap and respect precedence
//!   - On each machine, no two operations overlap
//!   - Recomputed makespan matches the solver's report and equals 55.

#[cfg(feature = "with-solver")]
use converge_kernel::{Budget, ContextState, ConvergeResult, Engine, ProposedFact};
#[cfg(feature = "with-solver")]
use converge_pack::ContextKey;

// Fisher-Thompson ft06 instance — verbatim from the OR-Library.
// Each row is one job; pairs are (machine, duration).
const FT06: [[(usize, i64); 6]; 6] = [
    [(2, 1), (0, 3), (1, 6), (3, 7), (5, 3), (4, 6)],
    [(1, 8), (2, 5), (4, 10), (5, 10), (0, 10), (3, 4)],
    [(2, 5), (3, 4), (5, 8), (0, 9), (1, 1), (4, 7)],
    [(1, 5), (0, 5), (2, 5), (3, 3), (4, 8), (5, 9)],
    [(2, 9), (1, 3), (4, 5), (5, 4), (0, 3), (3, 1)],
    [(1, 3), (3, 3), (5, 9), (0, 10), (4, 4), (2, 1)],
];
const OPTIMAL_MAKESPAN: i64 = 55;

#[cfg(feature = "with-solver")]
const REQUEST_ID: &str = "ft06";

#[cfg(feature = "with-solver")]
#[derive(Clone, Copy, Debug)]
struct ScenarioProvenance;

#[cfg(feature = "with-solver")]
impl converge_pack::ProvenanceSource for ScenarioProvenance {
    fn as_str(&self) -> &'static str {
        "atelier-jobshop-ft06"
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
             enable ferrox::jobshop::CpSatJobShopSuggestor (OR-Tools\n\
             CP-SAT with interval variables + NoOverlap). Honest exit."
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
    use ferrox::domain_types::{JobId, MachineId, ProcessingTime};
    use ferrox::jobshop::{Job, JobShopRequest, Operation};

    let jobs: Vec<Job> = FT06
        .iter()
        .enumerate()
        .map(|(idx, ops)| Job {
            id: JobId(idx),
            name: format!("j{idx}"),
            operations: ops
                .iter()
                .map(|(m, d)| Operation {
                    machine_id: MachineId(*m),
                    duration: ProcessingTime(*d),
                })
                .collect(),
        })
        .collect();

    let request = JobShopRequest {
        id: REQUEST_ID.to_string(),
        jobs,
        num_machines: 6,
        time_limit_seconds: 10.0,
    };

    println!("CP-SAT job-shop model");
    println!("─────────────────────");
    println!(
        "  jobs:        {} ({} operations each)",
        request.jobs.len(),
        6
    );
    println!("  machines:    {}", request.num_machines);
    println!(
        "  horizon:     {} (trivial UB = sum of durations)",
        request.horizon()
    );
    println!("  solver:      ferrox::jobshop::CpSatJobShopSuggestor");
    println!();

    let mut engine = Engine::with_budget(Budget {
        max_cycles: 4,
        max_facts: 16,
    });
    engine.register_suggestor(ferrox::jobshop::CpSatJobShopSuggestor);

    let mut ctx = ContextState::new();
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        format!("jspbench-request:{REQUEST_ID}"),
        request,
        scenario_provenance(),
    ))
    .map_err(|e| format!("seed proposal: {e:?}"))?;

    let result: ConvergeResult = engine.run(ctx).await?;
    print_solution(&result)
}

fn print_banner() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  Job-Shop FT06 — ferrox CP-SAT, proven optimum = 55                ║");
    println!("║  atelier-showcase · converge 3.9.1 · ferrox 0.7.1                    ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_problem() {
    println!("Fisher-Thompson ft06 instance");
    println!("─────────────────────────────");
    println!("  Each job is a sequence of operations: (machine, duration)");
    println!();
    for (j, ops) in FT06.iter().enumerate() {
        print!("  j{}:  ", j);
        for (i, (m, d)) in ops.iter().enumerate() {
            if i > 0 {
                print!(" → ");
            }
            print!("(m{m}, {d})");
        }
        println!();
    }
    println!();
    println!("Goal:  minimize makespan (time the last operation finishes).");
    println!("       Proven optimum for this instance: {OPTIMAL_MAKESPAN}.");
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

    let plan_id = format!("jspbench-plan-cpsat:{REQUEST_ID}");
    let plan_fact = result
        .context
        .get(ContextKey::Strategies)
        .iter()
        .find(|f| f.id().as_str() == plan_id)
        .ok_or("no CP-SAT job-shop plan emitted")?;
    let plan = plan_fact.require_payload::<ferrox::jobshop::JobShopPlan>()?;

    println!("  status:      {:?}", plan.status);
    println!("  solver:      {}", plan.solver);
    println!("  wall_time:   {:.3}s", plan.wall_time_seconds);
    println!("  makespan:    {} (reported)", plan.makespan);
    if let Some(lb) = plan.lower_bound {
        println!("  lower bound: {} (proven)", lb);
    }
    println!();

    // Independent verification.
    let mut by_job: Vec<Vec<&ferrox::jobshop::ScheduledOp>> = vec![vec![]; FT06.len()];
    let mut by_machine: std::collections::BTreeMap<usize, Vec<&ferrox::jobshop::ScheduledOp>> =
        std::collections::BTreeMap::new();
    for op in &plan.schedule {
        by_job[op.job_id.0].push(op);
        by_machine.entry(op.machine_id.0).or_default().push(op);
    }
    for ops in by_job.iter_mut() {
        ops.sort_by_key(|o| o.op_index);
    }
    for ops in by_machine.values_mut() {
        ops.sort_by_key(|o| o.start);
    }

    let mut recomputed_makespan = 0i64;

    println!("Schedule by machine");
    println!("───────────────────");
    for (m, ops) in &by_machine {
        print!("  m{m}: ");
        let mut prev_end = 0_i64;
        for (i, op) in ops.iter().enumerate() {
            if i > 0 {
                print!(" → ");
            }
            print!(
                "j{}.op{}@[{}..{}]",
                op.job_id.0, op.op_index, op.start, op.end
            );
            if op.start < prev_end {
                println!();
                return Err(format!(
                    "machine {m} overlap: op{} starts at {} but previous op ended at {}",
                    op.op_index, op.start, prev_end
                )
                .into());
            }
            prev_end = op.end;
            recomputed_makespan = recomputed_makespan.max(op.end);
        }
        println!();
    }
    println!();

    println!("Verification");
    println!("────────────");
    // Each job: precedence + duration consistency.
    for (j, ops) in by_job.iter().enumerate() {
        if ops.len() != FT06[j].len() {
            return Err(
                format!("job {j}: expected {} ops, got {}", FT06[j].len(), ops.len()).into(),
            );
        }
        for (i, op) in ops.iter().enumerate() {
            let (m, d) = FT06[j][i];
            if op.machine_id.0 != m {
                return Err(format!(
                    "job {j} op{i}: expected machine m{m}, scheduled on m{}",
                    op.machine_id.0
                )
                .into());
            }
            if op.end - op.start != d {
                return Err(format!(
                    "job {j} op{i}: duration {} ≠ expected {d}",
                    op.end - op.start
                )
                .into());
            }
            if i > 0 && op.start < ops[i - 1].end {
                return Err(format!(
                    "job {j}: op{i} starts at {} but op{} ends at {}",
                    op.start,
                    i - 1,
                    ops[i - 1].end
                )
                .into());
            }
        }
    }
    println!("  ✓ every operation has end - start == expected duration");
    println!("  ✓ every job respects operation precedence");
    println!("  ✓ no two operations overlap on the same machine");

    let solver_match = recomputed_makespan == plan.makespan;
    let optimum_match = recomputed_makespan == OPTIMAL_MAKESPAN;
    println!(
        "  recomputed makespan = {recomputed_makespan}    solver report = {}  {}",
        plan.makespan,
        if solver_match { "✓" } else { "✗" }
    );
    println!(
        "  recomputed makespan = {recomputed_makespan}    proven optimum = {OPTIMAL_MAKESPAN}  {}",
        if optimum_match { "✓" } else { "✗" }
    );
    if !solver_match || !optimum_match {
        return Err("makespan mismatch — solver or implementation disagreement".into());
    }
    println!();
    println!(
        "✓ ferrox CP-SAT matched the published ft06 optimum (55) in {:.0}ms.",
        plan.wall_time_seconds * 1000.0
    );
    Ok(())
}
