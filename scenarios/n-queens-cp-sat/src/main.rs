//! Atelier showcase: N-Queens via ferrox CP-SAT.
//!
//! ## Problem class
//!
//! **Constraint satisfaction problem** in the framework
//! ⟨V, D, C⟩. N-Queens is NP-complete as a CSP (the decision
//! problem of whether a partial assignment extends to a solution
//! is equivalent to SAT). For N=50 the search space contains
//! 50! ≈ 3 × 10⁶⁴ raw placements; brute force is intractable
//! past N≈12, and naïve backtracking degrades on adversarial
//! variable orderings. CP-SAT's `AllDifferent` global constraint
//! propagation + CDCL conflict-driven learning resolves N=50 in
//! milliseconds.
//!
//! See `kb/Architecture/Algorithmic Backbone.md` for the full
//! complexity-class treatment.
//!
//! ## Encoding
//!
//! Place N queens on an N×N board so no two share a row, column,
//! or diagonal. Trivial for small N; intractable for naive
//! brute force past N≈12 (8! = 40k positions, but 50! is ~10^64).
//! LLMs reliably hallucinate placements past N≈10 — they cannot
//! reason about diagonals at scale.
//!
//! Default N = 50 (a 50×50 board, 2500 cells, full placement
//! enumeration is hopeless). CP-SAT solves it via constraint
//! propagation in well under a second on commodity hardware.
//!
//! Encoding (3 AllDifferent + 2N auxiliary equalities):
//!   - q_r ∈ {0..=N-1}      — column for the queen on row r.
//!   - d_r = q_r + r        — anti-diagonal index for row r.
//!   - e_r = q_r - r        — diagonal index for row r.
//!   - AllDifferent(q_0..q_{N-1})  — no two queens share a column
//!     (rows are distinct by construction).
//!   - AllDifferent(d_0..d_{N-1})  — no two on the same anti-diagonal.
//!   - AllDifferent(e_0..e_{N-1})  — no two on the same diagonal.
//!
//! After the solver returns the scenario re-validates: every pair
//! of queens must differ in column AND in both diagonal indices.
//! Solver lying is detectable.

#[cfg(feature = "with-solver")]
use converge_kernel::{Budget, ContextState, ConvergeResult, Engine, ProposedFact};
#[cfg(feature = "with-solver")]
use converge_pack::ContextKey;

const N: usize = 50;
#[cfg(feature = "with-solver")]
const REQUEST_ID: &str = "n-queens-50";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_banner();

    #[cfg(not(feature = "with-solver"))]
    {
        println!("Problem");
        println!("───────");
        println!("  N = {N} (place {N} queens on a {N}×{N} board)");
        println!(
            "  search space: {N}^{N} ≈ 10^{}",
            (N as f64).log10() * N as f64
        );
        println!();
        println!("──────────────────────────────────────────────────────────");
        println!(
            "Solver disabled. Re-run with `--features with-solver` to\n\
             enable ferrox::cp::CpSatSuggestor. libortools is vendored\n\
             — no system install required. Honest exit; no naive\n\
             backtracking fallback."
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
        max_cycles: 4,
        max_facts: 16,
    });
    engine.register_suggestor(ferrox::cp::CpSatSuggestor);

    let mut ctx = ContextState::new();
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        format!("cpsat-request:{REQUEST_ID}"),
        request,
        "atelier-n-queens-cp-sat",
    ))
    .map_err(|e| format!("seed proposal: {e:?}"))?;

    let result: ConvergeResult = engine.run(ctx).await?;
    print_solution(&result)
}

#[cfg(feature = "with-solver")]
fn build_cpsat_request() -> ferrox::cp::CpSatRequest {
    use ferrox::cp::{ConstraintKind, CpTerm, CpVariable};

    let n = N as i64;
    let n_us = N;

    let queen = |r: usize| format!("q_{r}");
    let anti = |r: usize| format!("d_{r}");
    let diag = |r: usize| format!("e_{r}");

    let mut variables: Vec<CpVariable> = Vec::with_capacity(3 * n_us);

    // Queen column variables: q_r ∈ {0..=N-1}.
    for r in 0..n_us {
        variables.push(CpVariable {
            name: queen(r),
            lb: 0,
            ub: n - 1,
            is_bool: false,
        });
    }
    // Anti-diagonal variables: d_r ∈ {0..=2N-2}.
    for r in 0..n_us {
        variables.push(CpVariable {
            name: anti(r),
            lb: 0,
            ub: 2 * n - 2,
            is_bool: false,
        });
    }
    // Diagonal variables: e_r ∈ {-(N-1)..=N-1}.
    for r in 0..n_us {
        variables.push(CpVariable {
            name: diag(r),
            lb: -(n - 1),
            ub: n - 1,
            is_bool: false,
        });
    }

    let mut constraints: Vec<ConstraintKind> = Vec::new();

    // Link d_r = q_r + r → q_r + r - d_r = 0 → terms = [(q_r,1),(d_r,-1)], rhs = -r
    // (LinearEq form: sum(terms) == rhs)
    for r in 0..n_us {
        constraints.push(ConstraintKind::LinearEq {
            terms: vec![
                CpTerm {
                    var: queen(r),
                    coeff: 1,
                },
                CpTerm {
                    var: anti(r),
                    coeff: -1,
                },
            ],
            rhs: -(r as i64),
        });
    }
    // Link e_r = q_r - r → q_r - e_r = r → terms = [(q_r,1),(e_r,-1)], rhs = r
    for r in 0..n_us {
        constraints.push(ConstraintKind::LinearEq {
            terms: vec![
                CpTerm {
                    var: queen(r),
                    coeff: 1,
                },
                CpTerm {
                    var: diag(r),
                    coeff: -1,
                },
            ],
            rhs: r as i64,
        });
    }

    // Three AllDifferent constraints — the heart of the model.
    constraints.push(ConstraintKind::AllDifferent {
        vars: (0..n_us).map(queen).collect(),
    });
    constraints.push(ConstraintKind::AllDifferent {
        vars: (0..n_us).map(anti).collect(),
    });
    constraints.push(ConstraintKind::AllDifferent {
        vars: (0..n_us).map(diag).collect(),
    });

    ferrox::cp::CpSatRequest {
        id: REQUEST_ID.to_string(),
        variables,
        interval_vars: Vec::new(),
        optional_interval_vars: Vec::new(),
        constraints,
        objective_terms: None,
        minimize: true,
        time_limit_seconds: Some(30.0),
    }
}

fn print_banner() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  N-Queens CP-SAT — ferrox stress test                                ║");
    println!("║  atelier-showcase · converge 3.9.1 · ferrox 0.7.1                    ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();
}

#[cfg(feature = "with-solver")]
fn print_request(req: &ferrox::cp::CpSatRequest) {
    println!("Problem");
    println!("───────");
    println!("  N = {N} (place {N} queens on a {N}×{N} board)");
    println!();
    println!("CP-SAT model");
    println!("────────────");
    println!(
        "  variables:   {} (N queens + N anti-diag + N diag aux)",
        req.variables.len()
    );
    let all_diff = req
        .constraints
        .iter()
        .filter(|c| matches!(c, ferrox::cp::ConstraintKind::AllDifferent { .. }))
        .count();
    let lin_eq = req
        .constraints
        .iter()
        .filter(|c| matches!(c, ferrox::cp::ConstraintKind::LinearEq { .. }))
        .count();
    println!(
        "  constraints: {} ({all_diff} AllDifferent + {lin_eq} LinearEq linkages)",
        req.constraints.len(),
    );
    println!("  solver:      ferrox::cp::CpSatSuggestor (OR-Tools CP-SAT)");
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

    let plan_id = format!("cpsat-plan:{REQUEST_ID}");
    let plan_fact = result
        .context
        .get(ContextKey::Strategies)
        .iter()
        .find(|f| f.id().as_str() == plan_id);
    let Some(plan_fact) = plan_fact else {
        return Err("no cpsat-plan fact produced — solver did not run".into());
    };
    let plan = plan_fact.require_payload::<ferrox::cp::CpSatPlan>()?;

    println!("  status:      {:?}", plan.status);
    println!("  wall_time:   {:.3}s", plan.wall_time_seconds);
    println!("  solver:      {}", plan.solver);
    println!();

    // Pull queen columns out of the assignment.
    let mut cols: Vec<i64> = vec![-1; N];
    for (name, val) in &plan.assignments {
        if let Some(rest) = name.strip_prefix("q_")
            && let Ok(r) = rest.parse::<usize>()
            && r < N
        {
            cols[r] = *val;
        }
    }

    print_board(&cols);

    if let Err(msg) = validate(&cols) {
        return Err(msg.into());
    }
    println!("Verification");
    println!("────────────");
    println!("  ✓ {N} queens placed, all rows/cols distinct, no diagonal attacks");
    println!();
    println!(
        "✓ ferrox CP-SAT solved {N}-queens in {:.0}ms; placement independently verified.",
        plan.wall_time_seconds * 1000.0
    );
    Ok(())
}

#[cfg(feature = "with-solver")]
fn print_board(cols: &[i64]) {
    println!("Solved board (queen positions per row)");
    println!("──────────────────────────────────────");
    // For large N, the ascii grid gets unwieldy. Compromise: full
    // grid up to N=20, otherwise just the column vector.
    if N <= 20 {
        for &c in cols {
            let row = (0..N as i64)
                .map(|i| if i == c { 'Q' } else { '.' })
                .collect::<String>();
            println!("  {row}");
        }
    } else {
        println!(
            "  columns: {}",
            cols.iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
        println!("  (grid suppressed for N > 20 — {N} rows × {N} cols would be wide)");
    }
    println!();
}

#[cfg(feature = "with-solver")]
fn validate(cols: &[i64]) -> Result<(), String> {
    if cols.len() != N {
        return Err(format!("expected {N} columns, got {}", cols.len()));
    }
    for (r, &c) in cols.iter().enumerate() {
        if !(0..N as i64).contains(&c) {
            return Err(format!("row {r} column {c} out of range"));
        }
    }
    // Pairwise check: same column, same diagonal, same anti-diagonal.
    for i in 0..N {
        for j in (i + 1)..N {
            let ci = cols[i];
            let cj = cols[j];
            if ci == cj {
                return Err(format!("rows {i} and {j} share column {ci}"));
            }
            let di = i as i64;
            let dj = j as i64;
            if (ci - cj).abs() == (di - dj).abs() {
                return Err(format!("rows {i} ({ci}) and {j} ({cj}) share a diagonal"));
            }
        }
    }
    Ok(())
}
