//! Atelier showcase: round-driven design Formation selection.
//!
//! Runs one real Converge Formation that composes the round-driven
//! batch protocol with an existing adversarial Suggestor and the
//! platform `RoundSynthesizer`. No LLMs, no Runtime orchestrator, no
//! new traits — every participant is either a shipped Suggestor or a
//! tiny in-scenario fixture closing test-side glue the platform
//! deliberately does not own.
//!
//! Pipeline:
//!
//!   RoundStarter
//!     → CatalogProposerSuggestor::with_round_signals
//!     → DraftValidatorCriticSuggestor
//!     → AssumptionBreakerAgent      (existing adversarial)
//!     → BeautyContestSuggestor::new_critic_gated
//!     → ShortlistNoteEmitter        (scenario glue)
//!     → RoundSynthesizer
//!     → RoundAdvancer               (scenario glue)
//!
//! The scenario prints the trace as it inspects the converged
//! context, then performs the explicit compile handoff: pick
//! `latest_completed_batch`, extract its shortlist, validate via
//! `compile_draft`, and print the resulting roster.
//!
//! Run with:
//!
//!     cargo run -p scenario-round-driven-formation-design

use async_trait::async_trait;
use converge_kernel::formation::{
    FormationCatalog, FormationTemplate, FormationTemplateMetadata, FormationTemplateQuery,
    ProfileSnapshot, StaticFormationTemplate, SuggestorCapability, SuggestorRole,
};
use converge_kernel::{AgentEffect, Context, ContextFact, ContextKey};
use converge_pack::{ProvenanceSource, Suggestor, TextPayload};
use converge_provider::{CostClass, LatencyClass};
use organism_adversarial::AssumptionBreakerAgent;
use organism_catalog::{
    CatalogSuggestorDescriptor, DiscoveryCatalog, DiscoveryMetadata, LoopContribution,
    ProviderDescriptorCatalog, SuggestorDescriptor,
};
use organism_dynamics::{
    BeautyContestSuggestor, CatalogProposerSuggestor, DraftValidation,
    DraftValidatorCriticSuggestor, FormationDraft, compile_draft, completed_batches,
    critic_pass_complete_marker, extract_draft_validations, extract_drafts,
    extract_drafts_for_batch, latest_completed_batch, scorer_batch_complete_marker,
};
use organism_runtime::huddle::{
    RoundConventions, RoundStarter, RoundSynthesizer, SynthesisProducer,
};
use organism_runtime::{Formation, FormationCompileRequest, FormationCompiler};
use uuid::Uuid;

const ROUND_SIGNAL_PREFIX: &str = "design-round-";
const CONTINUE_PREFIX: &str = "design-round:continue:";
const NOTE_PREFIX: &str = "note:huddle:";
const SYNTHESIS_PREFIX: &str = "design-synthesis:";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_banner();

    let catalog = scenario_catalog();
    let templates = scenario_templates();
    let providers = ProviderDescriptorCatalog::new();
    let request = scenario_request();
    let conventions = design_huddle_conventions();

    print_intent(&request);

    let round_starter = RoundStarter::new(2).with_conventions(conventions);
    let proposer = CatalogProposerSuggestor::new(
        catalog.clone(),
        templates.clone(),
        providers.clone(),
        request.clone(),
        2,
    )
    .with_round_signals(ROUND_SIGNAL_PREFIX);
    let critic = DraftValidatorCriticSuggestor::new(
        catalog.clone(),
        templates.clone(),
        providers.clone(),
        request.clone(),
    );
    let adversarial = AssumptionBreakerAgent::new();
    let scorer = BeautyContestSuggestor::new_critic_gated(1);
    let synthesizer =
        RoundSynthesizer::new(1, HuddleSynthesisProducer).with_conventions(conventions);

    let huddle = Formation::new("round-driven-design-huddle")
        .agent_boxed(Box::new(round_starter))
        .agent_boxed(Box::new(proposer))
        .agent_boxed(Box::new(critic))
        .agent_boxed(Box::new(adversarial))
        .agent_boxed(Box::new(scorer))
        .agent_boxed(Box::new(ShortlistNoteEmitter))
        .agent_boxed(Box::new(synthesizer))
        .agent_boxed(Box::new(RoundAdvancer))
        .seed(
            ContextKey::Seeds,
            "design-seed",
            "design the work formation",
            "atelier-showcase",
        );

    let result = huddle.run().await?;
    if !result.converge_result.converged {
        return Err(format!(
            "design huddle did not converge: {:?}",
            result.converge_result.stop_reason
        )
        .into());
    }

    let ctx = &result.converge_result.context;
    print_rounds(ctx);
    print_adversarial(ctx);
    print_compile_handoff(ctx, &catalog, &templates, &providers, &request)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Pretty-printers
// ---------------------------------------------------------------------------

fn print_banner() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  Round-driven design Formation selection                             ║");
    println!("║  atelier-showcase · organism-dynamics · converge 3.9                 ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_intent(request: &FormationCompileRequest) {
    println!("Intent");
    println!("  template query: keyword \"work-template\"");
    println!("  plan id:        {}", request.plan_id);
    println!("  correlation id: {}", request.correlation_id);
    println!();
}

fn print_rounds(ctx: &dyn Context) {
    let round_signals = round_signal_ids(ctx);
    let drafts = extract_drafts(ctx, ContextKey::Strategies);
    let passes = extract_draft_validations(ctx, ContextKey::Evaluations);
    let blocks = extract_draft_validations(ctx, ContextKey::Constraints);
    let shortlist_all = extract_drafts(ctx, ContextKey::Proposals);
    let synthesis = synthesis_by_round(ctx);

    for (round_idx, batch_id) in round_signals.iter().enumerate() {
        let round_n = round_idx as u8 + 1;
        let header = format!("Round {round_n}  (batch: {batch_id})");
        println!("{header}");
        println!("{}", "─".repeat(header.len()));

        let drafts_in_batch: Vec<&FormationDraft> = drafts
            .iter()
            .filter(|d| d.draft_batch_id == *batch_id)
            .collect();
        println!("  Drafts:");
        for d in &drafts_in_batch {
            println!(
                "    {:<14}  →  [{}]",
                d.draft_id,
                d.descriptor_ids.join(", ")
            );
        }

        println!("  Draft critic:");
        for d in &drafts_in_batch {
            if let Some(v) = find_validation(&passes, batch_id, &d.draft_id) {
                println!("    {:<14}  →  PASS  ({})", d.draft_id, v.reason);
            } else if let Some(v) = find_validation(&blocks, batch_id, &d.draft_id) {
                println!("    {:<14}  →  BLOCK ({})", d.draft_id, v.reason);
            } else {
                println!("    {:<14}  →  (no verdict)", d.draft_id);
            }
        }

        let shortlist_for_batch: Vec<&FormationDraft> = shortlist_all
            .iter()
            .filter(|d| d.draft_batch_id == *batch_id)
            .collect();
        println!("  BeautyContest shortlist:");
        for d in &shortlist_for_batch {
            println!(
                "    {:<14}  →  [{}]",
                d.draft_id,
                d.descriptor_ids.join(", ")
            );
        }

        let critic_marker = critic_pass_complete_marker(batch_id);
        let scorer_marker = scorer_batch_complete_marker(batch_id);
        let has_critic = has_diagnostic(ctx, &critic_marker);
        let has_scorer = has_diagnostic(ctx, &scorer_marker);
        println!(
            "  Sentinels:        critic={}  scorer={}",
            tick(has_critic),
            tick(has_scorer)
        );

        if let Some(content) = synthesis.get(&round_n) {
            println!("  RoundSynthesizer: {content}");
        } else {
            println!("  RoundSynthesizer: (no synthesis for this round)");
        }

        println!();
    }
}

/// AssumptionBreaker is per-fact idempotent: it wakes on every
/// strategy fact it has not yet judged and emits exactly one
/// judgment per fact. With the round-driven batch protocol that
/// means it scrutinizes drafts from every round, not just the first.
fn print_adversarial(ctx: &dyn Context) {
    let facts = adversarial_facts(ctx);
    println!("AssumptionBreaker  (per-fact adversarial, organism-adversarial)");
    println!("───────────────────────────────────────────────────────────────");
    if facts.is_empty() {
        println!("  (no findings)");
    } else {
        for f in &facts {
            println!("  • {}", f.summary);
            println!("    target fact: {}", f.target_draft_id);
        }
    }
    println!();
}

fn print_compile_handoff(
    ctx: &dyn Context,
    catalog: &DiscoveryCatalog,
    templates: &FormationCatalog,
    providers: &ProviderDescriptorCatalog,
    request: &FormationCompileRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Compile handoff");
    println!("───────────────");
    let completed = completed_batches(ctx);
    println!("  completed batches (in order): {completed:?}");
    let latest =
        latest_completed_batch(ctx).ok_or("expected at least one batch to have completed")?;
    println!("  latest_completed_batch:        {latest}");

    let shortlist = extract_drafts_for_batch(ctx, ContextKey::Proposals, &latest);
    let chosen = shortlist
        .first()
        .ok_or("no draft shortlisted for the latest batch")?;
    println!(
        "  selected draft:                {} (rationale: {})",
        chosen.draft_id, chosen.rationale
    );

    let compiler = FormationCompiler::new();
    let plan = compile_draft(&compiler, request, templates, catalog, providers, chosen)?;
    println!("  compiled template:             {}", plan.template_id);
    println!("  compiled roster:");
    for entry in &plan.roster {
        // Show descriptor + role/capability for narrative value;
        // both come from the catalog the proposer drew from.
        let role = catalog
            .get(entry.suggestor_id.as_str())
            .map(|d| format!("{:?}", d.descriptor.profile.role))
            .unwrap_or_else(|| "?".to_string());
        println!("    - {:<16}  ({role})", entry.suggestor_id);
    }
    println!();
    println!("✓ Round-driven design Formation produced a validated work plan.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Catalog / templates / request — same shape as the dynamics tests so
// the showcase tracks the protocol it demonstrates.
// ---------------------------------------------------------------------------

fn synthetic_descriptor(
    id: &'static str,
    role: SuggestorRole,
    capability: SuggestorCapability,
    writes: ContextKey,
    blurb: &'static str,
) -> CatalogSuggestorDescriptor {
    let descriptor = SuggestorDescriptor::new(
        id,
        ProfileSnapshot {
            name: id.to_string(),
            role,
            output_keys: vec![writes],
            cost_hint: CostClass::Low,
            latency_hint: LatencyClass::Interactive,
            capabilities: vec![capability],
            confidence_min: 0.7,
            confidence_max: 0.95,
        },
    );
    let discovery = DiscoveryMetadata::new(blurb, "Showcase descriptor.")
        .with_loop_contribution(LoopContribution::Synthesize);
    CatalogSuggestorDescriptor::new(descriptor, discovery)
}

fn scenario_catalog() -> DiscoveryCatalog {
    DiscoveryCatalog::new()
        .with_entry(synthetic_descriptor(
            "signal-a",
            SuggestorRole::Signal,
            SuggestorCapability::KnowledgeRetrieval,
            ContextKey::Hypotheses,
            "Signal source A.",
        ))
        .with_entry(synthetic_descriptor(
            "signal-b",
            SuggestorRole::Signal,
            SuggestorCapability::KnowledgeRetrieval,
            ContextKey::Hypotheses,
            "Signal source B.",
        ))
        .with_entry(synthetic_descriptor(
            "constraint-a",
            SuggestorRole::Constraint,
            SuggestorCapability::PolicyEnforcement,
            ContextKey::Constraints,
            "Policy constraint A.",
        ))
        .with_entry(synthetic_descriptor(
            "constraint-b",
            SuggestorRole::Constraint,
            SuggestorCapability::PolicyEnforcement,
            ContextKey::Constraints,
            "Policy constraint B.",
        ))
}

fn scenario_templates() -> FormationCatalog {
    let metadata = FormationTemplateMetadata::new(
        "work-template",
        "Work formation: one signal + one constraint",
        vec![SuggestorRole::Signal, SuggestorRole::Constraint],
    )
    .with_keyword("work-template")
    .with_required_capability(SuggestorCapability::KnowledgeRetrieval)
    .with_required_capability(SuggestorCapability::PolicyEnforcement);
    FormationCatalog::new().with_template(FormationTemplate::static_template(
        StaticFormationTemplate::new(metadata),
    ))
}

fn scenario_request() -> FormationCompileRequest {
    FormationCompileRequest::new(
        Uuid::from_u128(0xA7E1_1E12_5C0C_A5E0),
        Uuid::from_u128(0xA7E1_1E12_5C0C_A5E1),
        FormationTemplateQuery::new().with_keyword("work-template"),
    )
}

fn design_huddle_conventions() -> RoundConventions {
    RoundConventions {
        round_signal_key: ContextKey::Signals,
        round_signal_prefix: ROUND_SIGNAL_PREFIX,
        continue_key: ContextKey::Constraints,
        continue_prefix: CONTINUE_PREFIX,
        note_key: ContextKey::Hypotheses,
        synthesis_key: ContextKey::Strategies,
        synthesis_prefix: SYNTHESIS_PREFIX,
    }
}

// ---------------------------------------------------------------------------
// Scenario fixtures: scenario-local glue that the platform deliberately
// does not own. RoundAdvancer turns a scorer completion sentinel into a
// continue marker; ShortlistNoteEmitter turns a scorer completion into
// a per-round note that RoundSynthesizer can synthesize.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct ScenarioProvenance;
impl ProvenanceSource for ScenarioProvenance {
    fn as_str(&self) -> &'static str {
        "atelier-round-driven-design-scenario"
    }
}

fn round_number_from_design_batch_id(batch_id: &str) -> Option<u8> {
    batch_id
        .strip_prefix(ROUND_SIGNAL_PREFIX)
        .and_then(|n| n.parse::<u8>().ok())
}

struct RoundAdvancer;

impl RoundAdvancer {
    fn pending(ctx: &dyn Context) -> Vec<u8> {
        let mut rounds: Vec<u8> = completed_batches(ctx)
            .into_iter()
            .filter_map(|b| round_number_from_design_batch_id(&b))
            .filter(|round| {
                let marker = format!("{CONTINUE_PREFIX}{round}");
                !ctx.get(ContextKey::Constraints)
                    .iter()
                    .any(|fact| fact.id().as_str() == marker)
            })
            .collect();
        rounds.sort_unstable();
        rounds.dedup();
        rounds
    }
}

#[async_trait]
impl Suggestor for RoundAdvancer {
    fn name(&self) -> &'static str {
        "scenario-round-advancer"
    }
    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Diagnostic]
    }
    fn provenance(&self) -> &'static str {
        ScenarioProvenance.as_str()
    }
    fn accepts(&self, ctx: &dyn Context) -> bool {
        !Self::pending(ctx).is_empty()
    }
    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let mut effect = AgentEffect::builder();
        for round in Self::pending(ctx) {
            effect = effect.proposal(ScenarioProvenance.proposed_fact(
                ContextKey::Constraints,
                format!("{CONTINUE_PREFIX}{round}"),
                TextPayload::new(format!("round {round} scoring complete; advance")),
            ));
        }
        effect.build()
    }
}

struct ShortlistNoteEmitter;

impl ShortlistNoteEmitter {
    fn pending(ctx: &dyn Context) -> Vec<u8> {
        let mut rounds: Vec<u8> = completed_batches(ctx)
            .into_iter()
            .filter_map(|b| round_number_from_design_batch_id(&b))
            .filter(|round| {
                let id = format!("{NOTE_PREFIX}{round}");
                !ctx.get(ContextKey::Hypotheses)
                    .iter()
                    .any(|fact| fact.id().as_str() == id)
            })
            .collect();
        rounds.sort_unstable();
        rounds.dedup();
        rounds
    }
}

#[async_trait]
impl Suggestor for ShortlistNoteEmitter {
    fn name(&self) -> &'static str {
        "scenario-shortlist-note-emitter"
    }
    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Diagnostic, ContextKey::Proposals]
    }
    fn provenance(&self) -> &'static str {
        ScenarioProvenance.as_str()
    }
    fn accepts(&self, ctx: &dyn Context) -> bool {
        !Self::pending(ctx).is_empty()
    }
    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let mut effect = AgentEffect::builder();
        for round in Self::pending(ctx) {
            let batch_id = format!("{ROUND_SIGNAL_PREFIX}{round}");
            let shortlist = extract_drafts_for_batch(ctx, ContextKey::Proposals, &batch_id);
            let descriptors: Vec<String> = shortlist
                .first()
                .map(|d| d.descriptor_ids.iter().map(ToString::to_string).collect())
                .unwrap_or_default();
            effect = effect.proposal(ScenarioProvenance.proposed_fact(
                ContextKey::Hypotheses,
                format!("{NOTE_PREFIX}{round}"),
                TextPayload::new(format!(
                    "round {round} shortlist: [{}]",
                    descriptors.join(", ")
                )),
            ));
        }
        effect.build()
    }
}

struct HuddleSynthesisProducer;

#[async_trait]
impl SynthesisProducer for HuddleSynthesisProducer {
    async fn synthesize(
        &self,
        round: u8,
        notes: &[ContextFact],
        _ctx: &dyn Context,
    ) -> Result<String, String> {
        let summaries: Vec<&str> = notes.iter().filter_map(|f| f.text()).collect();
        Ok(format!(
            "round {round} synthesis ({} note(s)): {}",
            notes.len(),
            summaries.join(" | "),
        ))
    }
}

// ---------------------------------------------------------------------------
// Trace inspection helpers
// ---------------------------------------------------------------------------

fn round_signal_ids(ctx: &dyn Context) -> Vec<String> {
    let mut ids: Vec<String> = ctx
        .get(ContextKey::Signals)
        .iter()
        .map(|f| f.id().as_str().to_string())
        .filter(|id| id.starts_with(ROUND_SIGNAL_PREFIX))
        .collect();
    ids.sort();
    ids
}

fn find_validation<'a>(
    verdicts: &'a [DraftValidation],
    batch_id: &str,
    draft_id: &str,
) -> Option<&'a DraftValidation> {
    verdicts
        .iter()
        .find(|v| v.draft_batch_id == batch_id && v.draft_id == draft_id)
}

struct AdversarialFact {
    target_draft_id: String,
    summary: String,
}

fn adversarial_facts(ctx: &dyn Context) -> Vec<AdversarialFact> {
    ctx.get(ContextKey::Evaluations)
        .iter()
        .filter_map(|fact| {
            let id = fact.id().as_str();
            if !id.starts_with("assumption-") {
                return None;
            }
            let summary = fact.text().unwrap_or("(no payload text)").to_string();
            let summary = compact_breaker_payload(&summary);
            Some(AdversarialFact {
                target_draft_id: id.to_string(),
                summary,
            })
        })
        .collect()
}

fn compact_breaker_payload(raw: &str) -> String {
    // The breaker writes a small JSON blob. Pull the human-readable
    // findings/warnings out so the printout reads cleanly.
    serde_quick::extract_messages(raw).unwrap_or_else(|| raw.chars().take(80).collect())
}

mod serde_quick {
    /// Minimal field grab — find the first `"findings"` or `"warnings"`
    /// array and join its string elements with `; `. Avoids pulling
    /// serde_json into the scenario.
    pub(crate) fn extract_messages(raw: &str) -> Option<String> {
        for key in ["findings", "warnings"] {
            let needle = format!("\"{key}\":[");
            if let Some(start) = raw.find(&needle) {
                let after = &raw[start + needle.len()..];
                let end = after.find(']')?;
                let inner = &after[..end];
                let parts: Vec<String> = inner
                    .split(',')
                    .map(str::trim)
                    .map(|s| s.trim_matches('"').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !parts.is_empty() {
                    return Some(parts.join("; "));
                }
            }
        }
        None
    }
}

fn synthesis_by_round(ctx: &dyn Context) -> std::collections::BTreeMap<u8, String> {
    ctx.get(ContextKey::Strategies)
        .iter()
        .filter_map(|fact| {
            let id = fact.id().as_str();
            let suffix = id.strip_prefix(SYNTHESIS_PREFIX)?;
            let round: u8 = suffix.parse().ok()?;
            let content = fact.text().unwrap_or_default().to_string();
            Some((round, content))
        })
        .collect()
}

fn has_diagnostic(ctx: &dyn Context, id: &str) -> bool {
    ctx.get(ContextKey::Diagnostic)
        .iter()
        .any(|fact| fact.id().as_str() == id)
}

fn tick(b: bool) -> &'static str {
    if b { "✓" } else { "✗" }
}
