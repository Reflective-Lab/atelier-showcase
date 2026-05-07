//! Truth-driven formation selection — three Truths, one per problem class,
//! handed to Organism's FormationGuru. The demo prints the classification
//! and the resulting formation pick for each Truth, with the visible
//! selection trace the milestone calls for.
//!
//! This scenario does *not* run the picked formations end-to-end. Running
//! requires a host-supplied SuggestorDescriptorCatalog + ExecutableSuggestorCatalog
//! (real provider handles, real factories) — out of scope for the routing
//! demo. The point here is: routing is real, deterministic, and explainable.

use anyhow::{Context as _, Result};
use converge_kernel::formation::SuggestorCapability;
use organism_runtime::{FormationGuru, GuruSelection, Runtime, standard_formation_catalog};

fn main() -> Result<()> {
    let truths: &[(&str, &str)] = &[
        ("decide-expense", DECIDE_EXPENSE_TRUTH),
        ("research-market", RESEARCH_MARKET_TRUTH),
        ("vet-acquisition", VET_ACQUISITION_TRUTH),
    ];

    let catalog = standard_formation_catalog();
    let guru = FormationGuru::new(&catalog);
    let capabilities = host_capabilities();

    println!(
        "Truth-driven formation selection — {} truths\n",
        truths.len()
    );
    println!("Available capabilities on this host:");
    for cap in &capabilities {
        println!("  · {cap:?}");
    }
    println!();

    for (label, source) in truths {
        let intent = axiom_truth::compile_intent_from_source(source)
            .with_context(|| format!("compiling Truth `{label}`"))?;

        // Sanity: organism's structural admission would also gate this in
        // production. We're focused on selection, not admission.
        let _runtime = Runtime::new();

        let selection = guru
            .select(&intent, &capabilities)
            .with_context(|| format!("guru selecting formation for `{label}`"))?;

        print_selection(label, &intent, &selection);
    }

    Ok(())
}

fn host_capabilities() -> Vec<SuggestorCapability> {
    vec![
        SuggestorCapability::LlmReasoning,
        SuggestorCapability::KnowledgeRetrieval,
        SuggestorCapability::Analytics,
        SuggestorCapability::PolicyEnforcement,
        SuggestorCapability::HumanInTheLoop,
    ]
}

fn print_selection(
    label: &str,
    intent: &organism_pack::IntentPacket,
    selection: &GuruSelection<'_>,
) {
    println!("──── {label} ────");
    println!("  outcome:        {}", intent.outcome);
    println!(
        "  classification: {} ({})",
        selection.classification.class,
        if selection.classification.defaulted {
            "default — no keywords matched"
        } else {
            "keyword-driven"
        }
    );
    if !selection.classification.matched_keywords.is_empty() {
        println!(
            "  matched words:  {}",
            selection.classification.matched_keywords.join(", ")
        );
    }
    println!("  picked:         {}", selection.primary.id());
    if !selection.alternates.is_empty() {
        let alts: Vec<&str> = selection.alternates.iter().map(|t| t.id()).collect();
        println!("  alternates:     {}", alts.join(", "));
    }
    println!("  reason:         {}", selection.trace.primary_reason);
    println!();
}

// ── Truth fixtures ───────────────────────────────────────────────────────

const DECIDE_EXPENSE_TRUTH: &str = r#"Truth: vendor approval gate

  Intent:
    Outcome: decide whether to approve the vendor renewal
    Goal: pick approve or reject under finance policy

  Authority:
    Actor: cfo_office
    May: approve_renewal
    Must Not: approve_unverified_vendor
    Requires Approval: spend_over_50k
    Expires: 2027-01-15T12:00:00Z

  Constraint:
    Budget: 50_000_USD
    Cost Limit: 12_500_USD/quarter

  Scenario: a renewal arrives
    Given a vendor proposal at 11_000_USD/quarter
    When the policy gate runs
    Then approve the renewal
"#;

const RESEARCH_MARKET_TRUTH: &str = r#"Truth: competitive landscape research

  Intent:
    Outcome: research the competitive landscape for the Q3 launch
    Goal: discover three credible competitors and summarize their positioning

  Authority:
    Actor: product_team
    May: investigate_public_sources

  Scenario: surface competitors
    Given the public-sources only directive
    When the research formation runs
    Then three positioning summaries are produced
"#;

const VET_ACQUISITION_TRUTH: &str = r#"Truth: acquisition diligence

  Intent:
    Outcome: vet the acquisition target end-to-end
    Goal: validate financial, legal, and operational claims before close

  Authority:
    Actor: corp_dev
    May: audit_financials
    Must Not: contact_target_employees
    Requires Approval: term_sheet_changes

  Constraint:
    Cost Limit: 250_000_USD
    Budget: 6_weeks

  Exception:
    Escalates To: legal_counsel

  Scenario: fact validation
    Given the data room access
    When the diligence formation runs
    Then a defended verdict is produced
"#;
