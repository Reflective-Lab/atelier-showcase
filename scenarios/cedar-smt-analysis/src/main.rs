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

    let input = CedarAnalysisInput::new(
        "expense-non-finance-high-value-commit-denied",
        CedarAnalysisQuery::ExpenseNonFinanceHighValueCommitDenied,
        arbiter::EXPENSE_APPROVAL_POLICY.to_string(),
        arbiter::EXPENSE_APPROVAL_SCHEMA.to_string(),
    );

    println!("Symbolic-analysis input");
    println!("───────────────────────");
    println!("  invariant_id: {}", input.invariant_id);
    println!("  query:        {:?}", input.query);
    println!(
        "  policy:       arbiter::EXPENSE_APPROVAL_POLICY ({} bytes)",
        input.policy_source.len()
    );
    println!(
        "  schema:       arbiter::EXPENSE_APPROVAL_SCHEMA ({} bytes)",
        input.schema_source.len()
    );
    println!("  backend:      arbiter::LocalCvc5AnalysisBackend (external CVC5 process)");
    println!();

    let backend = LocalCvc5AnalysisBackend;
    println!("Running CVC5 ...");
    let started = std::time::Instant::now();
    let report = backend.analyze(&input).await?;
    let elapsed = started.elapsed();
    println!("  done in {:.2}s", elapsed.as_secs_f64());
    println!();

    println!("Solver outcome");
    println!("──────────────");
    println!("  rollup status: {:?}", report.status);
    println!(
        "  plan:          {} policies, {} request environments analyzed",
        report.plan.policy_count,
        report.plan.request_env_count(),
    );
    println!(
        "  cedar_symcc:   v{} · cedar_policy: v{}",
        report.plan.cedar_symcc_version, report.plan.cedar_policy_version,
    );
    println!();

    println!("Per-environment checks");
    println!("──────────────────────");
    let mut total_assertions = 0usize;
    let mut violation_count = 0usize;
    let mut unknown_count = 0usize;
    for check in &report.checks {
        let env = &check.environment;
        total_assertions += env.assertion_count;
        match check.status {
            CedarAnalysisExecutionStatus::NoViolation => {}
            CedarAnalysisExecutionStatus::CounterexampleFound => violation_count += 1,
            CedarAnalysisExecutionStatus::Unknown => unknown_count += 1,
            CedarAnalysisExecutionStatus::Error => {}
        }
        let glyph = match check.status {
            CedarAnalysisExecutionStatus::NoViolation => "✓ proved",
            CedarAnalysisExecutionStatus::CounterexampleFound => "✗ counterexample",
            CedarAnalysisExecutionStatus::Unknown => "? unknown",
            CedarAnalysisExecutionStatus::Error => "‼ error",
        };
        println!(
            "  {glyph}  principal = {}   action = {}   resource = {}   ({} assertions)",
            env.principal_type, env.action, env.resource_type, env.assertion_count
        );
        if let Some(c) = &check.counterexample {
            println!("    counterexample: {c}");
        }
        if let Some(d) = &check.diagnostics {
            println!("    diagnostics: {d}");
        }
    }
    println!();
    println!(
        "  totals:  {} environments,  {} SMT assertions total",
        report.checks.len(),
        total_assertions
    );
    println!();

    println!("Verification");
    println!("────────────");
    match report.status {
        CedarAnalysisExecutionStatus::NoViolation => {
            println!(
                "  ✓ CVC5 proved the safety invariant across ALL {} modeled request \n\
                   environments. No non-finance supervisory principal can commit a \n\
                   high-value expense under the expense-approval policy, regardless \n\
                   of input.",
                report.checks.len()
            );
        }
        CedarAnalysisExecutionStatus::CounterexampleFound => {
            println!(
                "  ✗ CVC5 found {} concrete counterexample(s) — the policy permits a \n\
                   request the invariant says it must deny. The counterexamples above \n\
                   are real bugs, not statistical signal.",
                violation_count
            );
            return Err(format!(
                "policy fails invariant — {violation_count} counterexamples found"
            )
            .into());
        }
        CedarAnalysisExecutionStatus::Unknown => {
            println!(
                "  ? CVC5 returned 'unknown' on {} environment(s). The invariant is \n\
                   neither proved nor refuted — the policy may still hold, but this \n\
                   analysis cannot establish it.",
                unknown_count
            );
        }
        CedarAnalysisExecutionStatus::Error => {
            return Err("solver hit an error — see per-environment diagnostics above".into());
        }
    }
    println!();
    println!(
        "✓ Cedar symbolic analysis completed via CVC5 in {:.2}s; {} SMT assertions resolved.",
        elapsed.as_secs_f64(),
        total_assertions,
    );
    Ok(())
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
