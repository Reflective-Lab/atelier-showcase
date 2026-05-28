//! CRM Helm Showcase
//!
//! Demonstrates how Runway + Helm modules compose into a thin app binary.
//! Phase 6 of the Runway/Helm App-Host Boundary plan fills in the CRM module
//! implementations; this is the scenario shell.

mod conversations;
mod documents;
mod facts;
mod metadata;
mod opportunities;
mod parties;
mod truths;
mod workbench;
mod workflow;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    eprintln!("crm-helm-showcase placeholder — Phase 6 fills in CRM modules");
    Ok(())
}
