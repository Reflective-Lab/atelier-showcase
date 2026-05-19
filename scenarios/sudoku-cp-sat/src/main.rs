//! Atelier showcase: Sudoku via ferrox CP-SAT.
//!
//! ## Problem class
//!
//! **Constraint satisfaction problem (CSP)** — a triple
//! ⟨V, D, C⟩ where V is a set of variables, D is the product of
//! their domains, and C is a set of constraints each restricting
//! some subset of variables. Solving a Sudoku is **NP-complete** in
//! its general (n²×n²) form (Yato & Seta 2003) — equivalent to
//! deciding a SAT instance over the cell-assignment Boolean
//! variables. The 9×9 case is finite but adversarially constructed
//! instances (the "AI Escargot" puzzle below) are engineered to
//! maximize backtracking depth, defeating naïve search.
//!
//! ## The encoding
//!
//! 81 integer variables, one per cell:
//!   x_{r,c} ∈ {1, ..., 9},   r, c ∈ {0, ..., 8}
//!
//! Constraints:
//!   - 9 rows × AllDifferent({x_{r,0}, ..., x_{r,8}})
//!   - 9 cols × AllDifferent({x_{0,c}, ..., x_{8,c}})
//!   - 9 boxes × AllDifferent({x_{r,c} : (r,c) ∈ box_b})
//!   - ~17 fixed-clue equalities x_{r,c} = digit
//!
//! That's 27 `AllDifferent` global constraints (CP-SAT primitive,
//! propagated via Régin's matching algorithm in O(n^{2.5})) plus
//! ~17 linear equalities. The CP-SAT solver translates each
//! `AllDifferent` to a network of binary not-equal clauses
//! consumed by its internal CDCL core (lazy clause generation),
//! and combines propagation with conflict-driven learning to prune
//! the search space exponentially.
//!
//! ## Why this breaks LLMs and trivial backtracking
//!
//! - LLMs reliably hallucinate Sudoku cells. Even on 30+ clue
//!   puzzles, frontier chat models produce grids that violate
//!   row/column/box constraints when asked to solve in one pass.
//! - Naive depth-first backtracking handles easy puzzles fine but
//!   slows dramatically on adversarial 17-clue boards engineered
//!   to maximize backtrack depth.
//! - CP-SAT, with constraint propagation + clause learning, solves
//!   any well-formed Sudoku in milliseconds and PROVES optimality.
//!
//! The encoding is direct:
//!   - 81 integer variables x_{r,c} ∈ {1..=9} (one per cell).
//!   - 9 AllDifferent constraints over each row.
//!   - 9 AllDifferent constraints over each column.
//!   - 9 AllDifferent constraints over each 3×3 box.
//!   - One LinearEq constraint per fixed clue (x_{r,c} == digit).
//!
//! Total: 81 vars + 27 AllDifferent + ~17 LinearEq.
//!
//! After the solver returns, the scenario re-validates the grid
//! itself (rows/cols/boxes each contain {1..=9} as a set) before
//! claiming success. Solver lying is detectable, not assumed away.
//!
//! Default build prints the puzzle and exits honestly. Run with:
//!
//!     cargo run -p scenario-sudoku-cp-sat --features with-solver

#[cfg(feature = "with-solver")]
use converge_kernel::{Budget, ContextState, ConvergeResult, Engine, ProposedFact};
#[cfg(feature = "with-solver")]
use converge_pack::ContextKey;

// "AI Escargot" by Arto Inkala (2006), widely cited as among the
// hardest known Sudoku puzzles for backtracking heuristics. 23 clues.
// 0 = empty cell.
const PUZZLE: [[u8; 9]; 9] = [
    [1, 0, 0, 0, 0, 7, 0, 9, 0],
    [0, 3, 0, 0, 2, 0, 0, 0, 8],
    [0, 0, 9, 6, 0, 0, 5, 0, 0],
    [0, 0, 5, 3, 0, 0, 9, 0, 0],
    [0, 1, 0, 0, 8, 0, 0, 0, 2],
    [6, 0, 0, 0, 0, 4, 0, 0, 0],
    [3, 0, 0, 0, 0, 0, 0, 1, 0],
    [0, 4, 0, 0, 0, 0, 0, 0, 7],
    [0, 0, 7, 0, 0, 0, 3, 0, 0],
];

#[cfg(feature = "with-solver")]
const REQUEST_ID: &str = "sudoku-ai-escargot";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_banner();
    print_grid("Puzzle (23 clues — 'AI Escargot' by Arto Inkala)", &PUZZLE);

    #[cfg(not(feature = "with-solver"))]
    {
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
        "atelier-sudoku-cp-sat",
    ))
    .map_err(|e| format!("seed proposal: {e:?}"))?;

    let result: ConvergeResult = engine.run(ctx).await?;
    print_solution(&result)
}

#[cfg(feature = "with-solver")]
fn build_cpsat_request() -> ferrox::cp::CpSatRequest {
    use ferrox::cp::{ConstraintKind, CpTerm, CpVariable};

    let var = |r: usize, c: usize| format!("x_{r}_{c}");

    // 81 cell variables, domain {1..=9}.
    let variables: Vec<CpVariable> = (0..9)
        .flat_map(|r| {
            (0..9).map(move |c| CpVariable {
                name: var(r, c),
                lb: 1,
                ub: 9,
                is_bool: false,
            })
        })
        .collect();

    let mut constraints: Vec<ConstraintKind> = Vec::new();

    // AllDifferent per row.
    for r in 0..9 {
        constraints.push(ConstraintKind::AllDifferent {
            vars: (0..9).map(|c| var(r, c)).collect(),
        });
    }
    // AllDifferent per column.
    for c in 0..9 {
        constraints.push(ConstraintKind::AllDifferent {
            vars: (0..9).map(|r| var(r, c)).collect(),
        });
    }
    // AllDifferent per 3×3 box.
    for br in 0..3 {
        for bc in 0..3 {
            constraints.push(ConstraintKind::AllDifferent {
                vars: (0..3)
                    .flat_map(|dr| (0..3).map(move |dc| var(br * 3 + dr, bc * 3 + dc)))
                    .collect(),
            });
        }
    }
    // Fixed clues as LinearEq(x == digit).
    for (r, row) in PUZZLE.iter().enumerate() {
        for (c, &v) in row.iter().enumerate() {
            if v != 0 {
                constraints.push(ConstraintKind::LinearEq {
                    terms: vec![CpTerm {
                        var: var(r, c),
                        coeff: 1,
                    }],
                    rhs: i64::from(v),
                });
            }
        }
    }

    ferrox::cp::CpSatRequest {
        id: REQUEST_ID.to_string(),
        variables,
        interval_vars: Vec::new(),
        optional_interval_vars: Vec::new(),
        constraints,
        objective_terms: None,
        minimize: true,
        time_limit_seconds: Some(10.0),
    }
}

fn print_banner() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  Sudoku CP-SAT — ferrox end-to-end verification                     ║");
    println!("║  atelier-showcase · converge 3.9.1 · ferrox 0.7.1                    ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_grid(title: &str, grid: &[[u8; 9]; 9]) {
    println!("{title}");
    println!("{}", "─".repeat(title.len()));
    for (r, row) in grid.iter().enumerate() {
        if r % 3 == 0 && r > 0 {
            println!("  ───────┼───────┼───────");
        }
        let cells = row
            .iter()
            .enumerate()
            .map(|(c, v)| {
                let s = if *v == 0 {
                    ".".to_string()
                } else {
                    v.to_string()
                };
                if c % 3 == 2 && c < 8 {
                    format!("{s} │")
                } else {
                    s
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        println!("  {cells}");
    }
    println!();
}

#[cfg(feature = "with-solver")]
fn print_request(req: &ferrox::cp::CpSatRequest) {
    println!("CP-SAT model");
    println!("────────────");
    println!("  variables:   {} (each domain 1..=9)", req.variables.len());
    println!(
        "  constraints: {} ({} AllDifferent + {} fixed clues)",
        req.constraints.len(),
        req.constraints
            .iter()
            .filter(|c| matches!(c, ferrox::cp::ConstraintKind::AllDifferent { .. }))
            .count(),
        req.constraints
            .iter()
            .filter(|c| matches!(c, ferrox::cp::ConstraintKind::LinearEq { .. }))
            .count(),
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

    let mut grid: [[u8; 9]; 9] = [[0; 9]; 9];
    for (name, val) in &plan.assignments {
        let coords = name.strip_prefix("x_").and_then(|rest| {
            let mut it = rest.split('_');
            let r: usize = it.next()?.parse().ok()?;
            let c: usize = it.next()?.parse().ok()?;
            Some((r, c))
        });
        if let Some((r, c)) = coords
            && (1..=9).contains(val)
        {
            grid[r][c] = *val as u8;
        }
    }

    print_grid("Solved grid", &grid);

    // Self-validation: trust nothing. Recompute row/col/box sets and
    // confirm each is {1..=9}. Also confirm the original clues are
    // preserved. If the solver lied we want to know.
    let validation = validate_sudoku(&PUZZLE, &grid);
    println!("Verification");
    println!("────────────");
    match &validation {
        Ok(()) => println!("  ✓ rows, columns, 3×3 boxes each contain {{1..=9}}; clues preserved"),
        Err(msg) => println!("  ✗ INVALID: {msg}"),
    }
    println!();

    if let Err(msg) = &validation {
        return Err(msg.clone().into());
    }
    println!(
        "✓ ferrox CP-SAT solved a hard 23-clue Sudoku in {:.0}ms; \
         grid independently verified.",
        plan.wall_time_seconds * 1000.0
    );
    Ok(())
}

#[cfg(feature = "with-solver")]
fn validate_sudoku(puzzle: &[[u8; 9]; 9], grid: &[[u8; 9]; 9]) -> Result<(), String> {
    // Clue preservation.
    for (r, (puzzle_row, grid_row)) in puzzle.iter().zip(grid.iter()).enumerate() {
        for (c, (&p, &g)) in puzzle_row.iter().zip(grid_row.iter()).enumerate() {
            if p != 0 && p != g {
                return Err(format!(
                    "clue at ({r},{c}) was {p}, solver replaced with {g}"
                ));
            }
            if !(1..=9).contains(&g) {
                return Err(format!("cell ({r},{c}) holds {g} — out of domain"));
            }
        }
    }
    // Each row / column / box must be {1..=9}.
    for (i, row) in grid.iter().enumerate() {
        validate_unit("row", i, row)?;
        validate_unit("col", i, &(0..9).map(|r| grid[r][i]).collect::<Vec<_>>())?;
    }
    for br in 0..3 {
        for bc in 0..3 {
            let cells: Vec<u8> = (0..3)
                .flat_map(|dr| (0..3).map(move |dc| grid[br * 3 + dr][bc * 3 + dc]))
                .collect();
            validate_unit("box", br * 3 + bc, &cells)?;
        }
    }
    Ok(())
}

#[cfg(feature = "with-solver")]
fn validate_unit(kind: &str, idx: usize, cells: &[u8]) -> Result<(), String> {
    let mut seen = [false; 10];
    for &v in cells {
        if !(1..=9).contains(&v) {
            return Err(format!("{kind} {idx} has cell value {v} outside 1..=9"));
        }
        if seen[v as usize] {
            return Err(format!("{kind} {idx} has duplicate {v}"));
        }
        seen[v as usize] = true;
    }
    Ok(())
}
