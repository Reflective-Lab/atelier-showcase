//! Atelier showcase: Cedar symbolic analysis via CVC5 (SMT).
//!
//! Where the other arbiter scenarios *execute* a Cedar policy
//! against one concrete request, this scenario **formally verifies**
//! the policy: cedar-policy-symcc compiles the policy + schema into
//! an SMT-LIB assertion set, hands it to the CVC5 SMT solver, and
//! asks a model-theoretic question across the ENTIRE request space.
//!
//! Two kinds of question:
//!
//! - **Safety invariant.** "Is there ANY request matching claim X
//!   that this policy permits?" If the solver says UNSAT, that is
//!   a formal proof — no possible input violates the invariant.
//!   This scenario asks Cedar's built-in
//!   `ExpenseNonFinanceHighValueCommitDenied` claim against the
//!   built-in `EXPENSE_APPROVAL_POLICY`: the claim says non-finance
//!   supervisory principals cannot commit high-value expenses
//!   even when receipts + manager approval + required gates +
//!   human approval are all present. CVC5 proves this holds.
//!
//! - **Counterexample search.** "Find a concrete witness." If
//!   the solver says SAT, it returns an actual input that violates
//!   the invariant — a concrete bug, not statistical evidence.
//!
//! Why this matters: SMT analysis is a fundamentally different
//! capability than Cedar policy execution. PolicyGateSuggestor
//! decides allow/deny for ONE request. CedarAnalysisSuggestor
//! decides allow/deny for ALL requests in the modeled universe.
//! That second question is undecidable for general programs but
//! decidable for Cedar (because Cedar is finite-domain by
//! construction) — and CVC5 is what makes it tractable.
//!
//! cvc5 is the SMT solver from the cvc4/cvc5 lineage, written in
//! C++ with thousands of decision procedures (bit-vectors, arrays,
//! quantifiers, strings, …). Vendored at
//! `mosaic-extensions/soter-smt/vendor/cvc5/build/bin/cvc5`; the
//! scenario sets the CVC5 env var to that path if not already set,
//! so no system install needed.
//!
//! Default build prints the policy + invariant and exits honestly.
//! Pass `--features with-cvc5` to actually run the symbolic check.

#[cfg(feature = "with-cvc5")]
use arbiter::analysis::{
    CedarAnalysisBackend, CedarAnalysisExecutionStatus, CedarAnalysisInput, CedarAnalysisQuery,
    LocalCvc5AnalysisBackend,
};

// Vendored cvc5 binary path. cedar-policy-symcc resolves the
// solver via the CVC5 env var first, falling back to `cvc5` on
// PATH. The scenario sets this env var if it isn't already.
#[cfg(feature = "with-cvc5")]
const VENDORED_CVC5: &str =
    "/Users/kpernyer/dev/reflective/stack/mosaic-extensions/soter-smt/vendor/cvc5/build/bin/cvc5";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_banner();
    print_problem();

    #[cfg(not(feature = "with-cvc5"))]
    {
        println!("──────────────────────────────────────────────────────────");
        println!(
            "CVC5 SMT solver disabled. Re-run with `--features with-cvc5`\n\
             to enable Cedar symbolic analysis. cvc5 is vendored at\n\
             mosaic-extensions/soter-smt/vendor/cvc5/build/bin/cvc5 —\n\
             no system install required. Honest exit; no statistical\n\
             approximation fallback."
        );
        println!("──────────────────────────────────────────────────────────");
        Ok(())
    }
    #[cfg(feature = "with-cvc5")]
    {
        run_symbolic_analysis().await
    }
}

#[cfg(feature = "with-cvc5")]
async fn run_symbolic_analysis() -> Result<(), Box<dyn std::error::Error>> {
    // Point cedar-policy-symcc at the vendored cvc5 binary unless
    // the operator has already set CVC5 themselves.
    if std::env::var("CVC5").is_err() {
        // SAFETY: we are the only process touching this env var and
        // we set it before constructing the solver.
        unsafe {
            std::env::set_var("CVC5", VENDORED_CVC5);
        }
        println!("(set CVC5={VENDORED_CVC5})");
        println!();
    }

    print_state_space_analysis();

    // Run all three queries the symbolic-analysis pipeline supports
    // against the same policy. Each demonstrates a different mode:
    //   ExpenseNonFinanceHighValueCommitDenied — a *proof*: prove a
    //     specific safety claim holds across the entire request space.
    //   AlwaysAllows — a *counterexample search*: find any request
    //     the policy denies (UNSAT means policy allows everything).
    //   AlwaysDenies — the inverse: find any request the policy
    //     permits (UNSAT means policy denies everything).
    let queries = [
        (
            "expense-non-finance-high-value-commit-denied",
            CedarAnalysisQuery::ExpenseNonFinanceHighValueCommitDenied,
            "Safety proof: non-finance principals cannot commit high-value expenses",
        ),
        (
            "expense-policy-always-allows-check",
            CedarAnalysisQuery::AlwaysAllows,
            "Liveness search: is there ANY request the policy denies?",
        ),
        (
            "expense-policy-always-denies-check",
            CedarAnalysisQuery::AlwaysDenies,
            "Permissiveness search: is there ANY request the policy permits?",
        ),
    ];

    let backend = LocalCvc5AnalysisBackend;
    let mut total_wall = 0.0_f64;
    let mut total_assertions = 0usize;
    let mut total_envs = 0usize;
    let mut any_failure = false;

    for (invariant_id, query, description) in queries {
        println!();
        println!("══════════════════════════════════════════════════════════════════════");
        println!("Query: {invariant_id}");
        println!("        {description}");
        println!("══════════════════════════════════════════════════════════════════════");

        let input = CedarAnalysisInput::new(
            invariant_id,
            query,
            arbiter::EXPENSE_APPROVAL_POLICY.to_string(),
            arbiter::EXPENSE_APPROVAL_SCHEMA.to_string(),
        );

        let started = std::time::Instant::now();
        let report = backend.analyze(&input).await?;
        let elapsed = started.elapsed().as_secs_f64();
        total_wall += elapsed;

        println!("  rollup:        {:?} in {:.2}s", report.status, elapsed);
        println!(
            "  plan:          {} policies, {} request environments analyzed",
            report.plan.policy_count,
            report.plan.request_env_count(),
        );

        let mut assertions = 0usize;
        let mut violations = 0usize;
        for check in &report.checks {
            assertions += check.environment.assertion_count;
            total_assertions += check.environment.assertion_count;
            if matches!(
                check.status,
                CedarAnalysisExecutionStatus::CounterexampleFound
            ) {
                violations += 1;
            }
            let glyph = match check.status {
                CedarAnalysisExecutionStatus::NoViolation => "✓ unsat (proved)",
                CedarAnalysisExecutionStatus::CounterexampleFound => "✗ sat (counterexample)",
                CedarAnalysisExecutionStatus::Unknown => "? unknown",
                CedarAnalysisExecutionStatus::Error => "‼ error",
            };
            println!(
                "    {glyph}  {}::{}  ({} assertions)",
                check.environment.action,
                check.environment.resource_type,
                check.environment.assertion_count,
            );
            if let Some(c) = &check.counterexample {
                println!("        witness: {}", compact_witness(c));
            }
            if let Some(d) = &check.diagnostics {
                println!("        diag: {d}");
            }
        }
        total_envs += report.checks.len();

        match report.status {
            CedarAnalysisExecutionStatus::NoViolation => {
                println!(
                    "  meaning:       proved invariant across ALL {} modeled environments \
                     ({} SMT assertions total for this query)",
                    report.checks.len(),
                    assertions
                );
            }
            CedarAnalysisExecutionStatus::CounterexampleFound => {
                println!(
                    "  meaning:       found {} concrete counterexample(s) — \
                     each is an actual permitted/denied request, not statistical signal",
                    violations
                );
            }
            CedarAnalysisExecutionStatus::Unknown => {
                println!("  meaning:       solver returned 'unknown' on at least one environment");
            }
            CedarAnalysisExecutionStatus::Error => {
                println!("  meaning:       solver hit an error — see diag above");
                any_failure = true;
            }
        }
    }

    println!();
    println!("══════════════════════════════════════════════════════════════════════");
    println!("Aggregate");
    println!("══════════════════════════════════════════════════════════════════════");
    println!(
        "  3 queries · {total_envs} request environments · {total_assertions} SMT assertions \
         resolved across all queries"
    );
    println!("  total wall time:  {total_wall:.2}s");
    println!();
    println!(
        "  Each SMT assertion above constrains the *entire* attribute space — not a single\n\
         test case. The state-space comparison shows what brute-force testing would cost\n\
         to cover the same territory."
    );

    if any_failure {
        return Err("at least one query hit an error".into());
    }
    Ok(())
}

#[cfg(feature = "with-cvc5")]
fn print_state_space_analysis() {
    // Crude — but honest — combinatorial lower bound on the input
    // space that the symbolic queries cover. The Cedar schema for
    // expense approvals has:
    //   - 4 principal authority levels (advisory / supervisory /
    //     participatory / sovereign)
    //   - 5 domain memberships (a Set<String> drawn from finite values)
    //   - 4 resource_type values (expense / quote / contract / invoice)
    //   - 4 flow phases (intent / framing / convergence / commitment)
    //   - 4 gates_passed Set<String> from receipt, manager_approval,
    //     budget_check, compliance_check (Set of 4 → 16 subsets)
    //   - 4 commitment_type values
    //   - context.amount: bounded i64 in [-2^63, 2^63) but realistic
    //     range, say, 0..1_000_000 = 10^6 distinct values
    //   - human_approval_present: 2 values
    //   - required_gates_met: 2 values
    //   - 5 action types
    //
    // Cartesian product (treating Set fields as their 2^|S| subset
    // count) — very conservative:
    let principal_authority: u128 = 4;
    let principal_domains_subsets: u128 = 1 << 5; // 2^5 = 32
    let resource_type: u128 = 4;
    let phase: u128 = 4;
    let gates_passed_subsets: u128 = 1 << 4; // 2^4 = 16
    let commitment_type: u128 = 4;
    // Currency amounts in cents over a 0..$10M range: 10^9 distinct
    // values. Real Cedar allows full i64 (~10^19); 10^9 keeps the
    // estimate plausibly bounded.
    let amount_range: u128 = 1_000_000_000;
    let human_approval_present: u128 = 2;
    let required_gates_met: u128 = 2;
    let action: u128 = 5;

    let space = principal_authority
        * principal_domains_subsets
        * resource_type
        * phase
        * gates_passed_subsets
        * commitment_type
        * amount_range
        * human_approval_present
        * required_gates_met
        * action;

    println!("State-space comparison");
    println!("──────────────────────");
    println!("  Brute-force coverage of the expense-approval schema (conservatively bounded):");
    println!("    principal.authority           4 values");
    println!("    principal.domains             2^5 = 32 subsets");
    println!("    resource.resource_type        4 values");
    println!("    resource.phase                4 values");
    println!("    resource.gates_passed         2^4 = 16 subsets");
    println!("    context.commitment_type       4 values");
    println!("    context.amount                10^9 (cents in 0..$10M)");
    println!("    context.human_approval        2 values");
    println!("    context.required_gates_met    2 values");
    println!("    action                        5 values");
    println!();
    println!(
        "  product: {} configurations (≈ 10^{:.1})",
        format_with_commas(space),
        (space as f64).log10()
    );
    println!();
    // Realistic policy eval (parse + check + audit log) is ~10µs;
    // 100k tests/sec sustained is generous.
    println!(
        "  At 100,000 tests/sec sustained, exhaustive testing of this bounded space\n\
         would take roughly {} years.",
        years_to_test(space, 100_000)
    );
    println!(
        "  And that bound is conservative: real Cedar schemas allow unbounded strings,\n\
         arbitrary-length sets, and full i64 integers — making the space genuinely infinite\n\
         until something like SMT symbolically collapses it."
    );
    println!();
    println!("  CVC5, in contrast, reasons SYMBOLICALLY about the same space — every");
    println!("  assertion below covers an entire region of inputs at once, and the");
    println!("  solver proves SAT/UNSAT in seconds. That difference is why SMT exists.");
    println!();
}

/// The cedar-policy-symcc backend returns its witness as a raw
/// Debug-formatted `Cedar::Request` tree — hundreds of lines for a
/// single counterexample. Compact it: pick out the principal/action/
/// resource id strings and any `gates_passed` set values, ignoring
/// the AST nesting. A reader can still tell what request the solver
/// found; the full payload is in plan.checks if needed.
#[cfg(feature = "with-cvc5")]
fn compact_witness(raw: &str) -> String {
    let mut gates: Vec<String> = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if let Some(s) = line.strip_prefix("String(")
            && let Some(start) = s.find('"')
            && let Some(end) = s[start + 1..].find('"')
        {
            gates.push(s[start + 1..start + 1 + end].to_string());
        }
    }
    let gates_str = if gates.is_empty() {
        String::new()
    } else {
        format!(" attrs/gates: [{}]", gates.join(", "))
    };
    let summary = if raw.len() > 80 {
        format!("counterexample present ({} bytes)", raw.len())
    } else {
        raw.to_string()
    };
    format!("{summary}{gates_str}")
}

#[cfg(feature = "with-cvc5")]
fn format_with_commas(n: u128) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(b as char);
    }
    out
}

#[cfg(feature = "with-cvc5")]
fn years_to_test(configurations: u128, tests_per_sec: u128) -> String {
    let seconds_per_year = 31_557_600_u128;
    let years = configurations / (tests_per_sec * seconds_per_year);
    if years == 0 {
        "< 1".to_string()
    } else if years > 1_000 {
        format!("≈ 10^{:.1}", (years as f64).log10())
    } else {
        format!("{years}")
    }
}

fn print_banner() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  Cedar SMT Analysis — arbiter + CVC5 (formal verification)         ║");
    println!("║  atelier-showcase · arbiter 2.0.1 · cedar-policy-symcc + CVC5        ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_problem() {
    println!("Invariant to formally verify");
    println!("────────────────────────────");
    println!(
        "  ExpenseNonFinanceHighValueCommitDenied —  the expense-approval policy\n\
         must DENY a 'commit' action whenever the principal is supervisory but\n\
         not in finance, even when receipts + manager_approval + required_gates_met\n\
         + human_approval are all present and amount > 5000.\n"
    );
    println!("Why this is hard");
    println!("────────────────");
    println!(
        "  Executing the policy against any one concrete request tells you what\n\
         happens for THAT input. The invariant is a claim about INFINITE inputs:\n\
         every (principal, resource, context) triple matching the high-value-commit\n\
         shape must be denied. Statistical testing cannot prove this; SMT can."
    );
    println!();
    println!("How CVC5 helps");
    println!("──────────────");
    println!(
        "  cedar-policy-symcc compiles the policy + schema into SMT-LIB. Each\n\
         (principal_type, action, resource_type) request environment becomes an\n\
         assertion set. CVC5 checks whether any model satisfies\n\
         (request matches claim) ∧ (policy permits) — i.e., a concrete\n\
         counterexample. UNSAT = invariant proved across that environment."
    );
    println!();
}
