//! Scenario: quorum-open-inquiry
//!
//! End-to-end exercise of Quorum's applet-manifest spine, using the *same*
//! `open-adaptive-inquiry.applet.json` manifest + `tough-decision-v1.flavor.json`
//! flavor that ship in `marquee-apps/quorum-sense/crates/quorum-truths/`.
//!
//! This is the Plan 2 (Track 0d) proof that the spine works end-to-end
//! from outside Quorum's own test suite. No mocks, no parallel fixtures.

use anyhow::Context;
use quorum_app::manifest_mapper::{
    HostSessionInput, blueprint_to_inquiry_contract, compose_contract,
};
use quorum_truths::flavors::load_tough_decision_v1;
use quorum_truths::manifests::load_open_adaptive_inquiry;

fn main() -> anyhow::Result<()> {
    println!("\n=== Quorum open-adaptive-inquiry scenario ===\n");

    println!("step 1: load Axiom-validated JTBD manifest from quorum-truths");
    let manifest =
        load_open_adaptive_inquiry().context("failed to load open-adaptive-inquiry manifest")?;
    println!(
        "  -> primary_job_key = {}, manifest_version = {}",
        manifest.primary_job_key, manifest.manifest_version
    );

    println!("\nstep 2: load tough-decision-v1 flavor from quorum-truths");
    let flavor = load_tough_decision_v1().context("failed to load tough-decision-v1 flavor")?;
    println!(
        "  -> flavor_id = {}, manifest_ref = {}, rulebook_ref = {}",
        flavor.flavor_id, flavor.manifest_ref, flavor.rulebook_ref
    );

    println!("\nstep 3: compose a contract blueprint from a demo host input");
    let input = HostSessionInput {
        topic: "Should we sunset the legacy ingest pipeline?".into(),
        key_questions: vec![
            "Is the customer signal still pointing at the new path?".into(),
            "Can the new pipeline handle the same peak load?".into(),
            "What evidence would change our minds?".into(),
        ],
        decision_rule: "consent_not_consensus".into(),
        time_budget_seconds: 2700,
    };
    let blueprint =
        compose_contract(&manifest, &flavor, &input).context("compose_contract rejected the seed")?;
    println!(
        "  -> core_question = {:?}, decision_rule = {}, time_budget_seconds = {}",
        blueprint.core_question, blueprint.decision_rule, blueprint.time_budget_seconds
    );
    println!(
        "  -> organizer_thread ({} key questions): {:?}",
        blueprint.organizer_thread.len(),
        blueprint.organizer_thread
    );

    println!("\nstep 4: convert the blueprint to a runtime InquiryContract");
    let contract = blueprint_to_inquiry_contract(&blueprint);
    println!(
        "  -> contract.core_question = {:?}",
        contract.core_question
    );
    println!(
        "  -> contract.decision_rule = {:?}, anonymity_policy = {:?}",
        contract.decision_rule, contract.anonymity_policy
    );
    println!(
        "  -> contract.actor_policy = {:?}",
        contract.actor_policy
    );
    println!(
        "  -> contract.time_budget = {:?}",
        contract.time_budget
    );
    println!(
        "  -> contract.evidence_requirements (side-channel metadata):"
    );
    for line in &contract.evidence_requirements {
        println!("       - {line}");
    }

    println!("\n=== scenario complete ===\n");
    Ok(())
}
