//! Atelier showcase: round-driven design Formation selection,
//! end-to-end with the real seeded catalog and the real default
//! executable factories.
//!
//! The flow this scenario demonstrates:
//!
//!   intent
//!     → design huddle Formation
//!         (RoundStarter → CatalogProposerSuggestor[round-driven]
//!          → DraftValidatorCriticSuggestor → AssumptionBreakerAgent
//!          → BeautyContestSuggestor[critic-gated]
//!          → RoundAdvancer)
//!     → two rounds, two batches, per-batch sentinels
//!     → host picks latest_completed_batch
//!     → compile_draft against the real organism-catalog-seed
//!     → ExecutableSuggestorCatalog::instantiate with real
//!       organism-adversarial factories
//!     → work Formation runs to convergence in Converge
//!
//! Nothing here is mocked. The catalog comes from
//! `organism_catalog_seed::organism_only()`. The executable
//! factories come from `organism_runtime::register_default_factories`
//! (real `AssumptionBreakerAgent`, `ConstraintCheckerAgent`,
//! `AnomalySkepticAgent`, etc. behind their catalog ids). If any
//! piece of wiring is missing — descriptor not in the seed, factory
//! not registered, template not covered — the scenario exits with
//! an explicit error from `main()` rather than substituting a
//! placeholder.
//!
//! Two pieces are deliberately not in this scenario:
//!   * `RoundSynthesizer` + a `SynthesisProducer` impl — the
//!     platform does not ship a default LLM-backed producer, so
//!     wiring it would require either a real Manifold-backed
//!     `SynthesisProducer` (next slice) or a synthetic stand-in
//!     (forbidden). The round loop runs without synthesis; the
//!     critic + scorer drive batch completion.
//!   * Per-round notes via a `ShortlistNoteEmitter` — only needed
//!     to feed the (absent) synthesizer.
//!
//! Run with:
//!
//!     cargo run -p scenario-round-driven-formation-design

use async_trait::async_trait;
use converge_kernel::formation::{
    FormationCatalog, FormationTemplate, FormationTemplateMetadata, FormationTemplateQuery,
    StaticFormationTemplate, SuggestorCapability, SuggestorRole,
};
use converge_kernel::{AgentEffect, Context, ContextKey};
use converge_pack::{ProvenanceSource, Suggestor, TextPayload};
use organism_adversarial::AssumptionBreakerAgent;
use organism_catalog::{DiscoveryCatalog, ProviderDescriptorCatalog};
use organism_catalog_seed as seed;
use organism_dynamics::{
    BeautyContestSuggestor, CatalogProposerSuggestor, DraftValidation,
    DraftValidatorCriticSuggestor, FormationDraft, compile_draft, completed_batches,
    critic_pass_complete_marker, extract_draft_validations, extract_drafts,
    extract_drafts_for_batch, latest_completed_batch, scorer_batch_complete_marker,
};
use organism_runtime::huddle::{RoundConventions, RoundStarter};
use organism_runtime::{
    ExecutableSuggestorCatalog, Formation, FormationCompileRequest, FormationCompiler, Seed,
    register_default_factories,
};
use uuid::Uuid;

const ROUND_SIGNAL_PREFIX: &str = "design-round-";
const CONTINUE_PREFIX: &str = "design-round:continue:";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_banner();

    // ── 1. Wire the real catalog and the real executable
    //       factories — coherently. The proposer must only choose
    //       descriptors for which a factory exists, otherwise the
    //       compile handoff would pick a roster that can't be
    //       instantiated. We start from the full organism seed,
    //       wire all the default factories, then filter the
    //       catalog down to descriptors covered by a factory.
    //       Any drift between seed and factories surfaces here.
    let full_catalog = seed::organism_only();
    let mut executables = ExecutableSuggestorCatalog::new();
    register_default_factories(&mut executables)?;
    let catalog = catalog_covered_by_factories(&full_catalog, &executables);
    let templates = scenario_templates();
    let providers = ProviderDescriptorCatalog::new();
    let request = scenario_request();
    let conventions = design_huddle_conventions();

    print_intent(&request, &catalog, &executables);

    // ── 2. Build the design huddle Formation.
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

    let huddle = Formation::new("round-driven-design-huddle")
        .agent_boxed(Box::new(round_starter))
        .agent_boxed(Box::new(proposer))
        .agent_boxed(Box::new(critic))
        .agent_boxed(Box::new(adversarial))
        .agent_boxed(Box::new(scorer))
        .agent_boxed(Box::new(RoundAdvancer))
        .seed(
            ContextKey::Seeds,
            "design-seed",
            "audit this plan for policy compliance and anomalies",
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

    // ── 3. Print what the design huddle produced.
    print_rounds(ctx);
    print_adversarial(ctx);

    // ── 4. Compile handoff: pick latest_completed_batch, validate
    //       its shortlist against the real catalog, then instantiate
    //       the work Formation against the real factory set.
    let plan = compile_handoff(ctx, &catalog, &templates, &providers, &request)?;

    // ── 5. Actually run the work Formation in Converge.
    print_section("Work Formation execution");
    // The work formation audits a candidate plan. The plan is fed
    // in via Strategies — the gates read from there. The seed under
    // ContextKey::Seeds is the human framing for downstream
    // discovery; the actual auditable payload is the JSON under
    // Strategies. Both are scenario input — not mocks of a real
    // upstream proposer's output.
    let candidate_plan = serde_json::json!({
        "id": "ALPHA-1",
        "description": "Roll out the new vendor onboarding workflow.",
        "annotation": {
            "actions": ["enable-vendor-onboarding", "notify-procurement"],
            "tags": ["procurement", "rollout"],
            "costs": [{"item": "engineering", "estimate": 7500.0}],
            "approvals": ["procurement-lead"],
        },
    });
    let seeds = vec![
        Seed {
            key: ContextKey::Seeds,
            id: "work-seed".into(),
            content: "audit candidate plan #ALPHA-1".to_string(),
            provenance: "atelier-showcase".to_string(),
        },
        Seed {
            key: ContextKey::Strategies,
            id: "candidate-plan".into(),
            content: serde_json::to_string(&candidate_plan)?,
            provenance: "atelier-showcase".to_string(),
        },
    ];
    let work = executables.instantiate(&plan, seeds)?;
    let work_result = work.run().await?;
    if !work_result.converge_result.converged {
        return Err(format!(
            "work Formation did not converge: {:?}",
            work_result.converge_result.stop_reason
        )
        .into());
    }
    print_work_outcome(&work_result.converge_result.context);

    println!();
    println!("✓ Round-driven design Formation produced and ran a real work plan.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Template + request — picks a real-Suggestor-satisfiable template
// ---------------------------------------------------------------------------

/// Project the full catalog down to descriptors that actually have a
/// registered factory. This is the boundary between "described" and
/// "runnable" — keeping them aligned in the scenario means the
/// proposer cannot produce a draft the host can't instantiate.
fn catalog_covered_by_factories(
    full: &DiscoveryCatalog,
    executables: &ExecutableSuggestorCatalog,
) -> DiscoveryCatalog {
    let mut filtered = DiscoveryCatalog::new();
    for entry in full {
        if executables.contains(entry.id().as_str()) {
            filtered.register(entry.clone());
        }
    }
    filtered
}

fn scenario_templates() -> FormationCatalog {
    // Two Constraint-role gates, one PolicyEnforcement + one Analytics.
    // The organism-catalog-seed gives us multiple candidates for each
    // capability (constraint-checker covers PolicyEnforcement;
    // anomaly-skeptic and economic-skeptic both cover Analytics) —
    // enough for the k-best proposer to produce at least two
    // distinct rosters for tournament diversity.
    let metadata = FormationTemplateMetadata::new(
        "policy-and-anomaly-audit",
        "Audit a candidate plan for policy compliance and anomalies.",
        vec![SuggestorRole::Constraint],
    )
    .with_keyword("policy-and-anomaly-audit")
    .with_required_capability(SuggestorCapability::PolicyEnforcement)
    .with_required_capability(SuggestorCapability::Analytics);
    FormationCatalog::new().with_template(FormationTemplate::static_template(
        StaticFormationTemplate::new(metadata),
    ))
}

fn scenario_request() -> FormationCompileRequest {
    FormationCompileRequest::new(
        Uuid::from_u128(0xA7E1_1E12_5C0C_A5E0),
        Uuid::from_u128(0xA7E1_1E12_5C0C_A5E1),
        FormationTemplateQuery::new().with_keyword("policy-and-anomaly-audit"),
    )
}

fn design_huddle_conventions() -> RoundConventions {
    RoundConventions {
        round_signal_key: ContextKey::Signals,
        round_signal_prefix: ROUND_SIGNAL_PREFIX,
        continue_key: ContextKey::Constraints,
        continue_prefix: CONTINUE_PREFIX,
        note_key: ContextKey::Hypotheses,
        synthesis_key: ContextKey::Hypotheses,
        synthesis_prefix: "design-synthesis:",
    }
}

// ---------------------------------------------------------------------------
// Pretty-printers
// ---------------------------------------------------------------------------

fn print_banner() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  Round-driven design Formation — real catalog, real factories       ║");
    println!("║  atelier-showcase · organism 1.9.1 · converge 3.9.1                  ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_section(title: &str) {
    println!("{title}");
    println!("{}", "─".repeat(title.len()));
}

fn print_intent(
    request: &FormationCompileRequest,
    catalog: &DiscoveryCatalog,
    executables: &ExecutableSuggestorCatalog,
) {
    print_section("Intent + wiring");
    println!("  template query: keyword \"policy-and-anomaly-audit\"");
    println!("  plan id:        {}", request.plan_id);
    println!("  correlation id: {}", request.correlation_id);
    println!("  catalog:        organism-catalog-seed::organism_only()");
    println!(
        "                  {} descriptors total",
        catalog.iter().count()
    );
    println!(
        "  executables:    register_default_factories() → {} factories",
        executables.suggestor_ids().len()
    );
    for id in executables.suggestor_ids() {
        println!("                    · {id}");
    }
    println!();
}

fn print_rounds(ctx: &dyn Context) {
    let round_signals = round_signal_ids(ctx);
    let drafts = extract_drafts(ctx, ContextKey::Strategies);
    let passes = extract_draft_validations(ctx, ContextKey::Evaluations);
    let blocks = extract_draft_validations(ctx, ContextKey::Constraints);
    let shortlist_all = extract_drafts(ctx, ContextKey::Proposals);

    for (round_idx, batch_id) in round_signals.iter().enumerate() {
        let round_n = round_idx as u8 + 1;
        let header = format!("Round {round_n}  (batch: {batch_id})");
        print_section(&header);

        let drafts_in_batch: Vec<&FormationDraft> = drafts
            .iter()
            .filter(|d| d.draft_batch_id == *batch_id)
            .collect();
        println!("  Drafts:");
        for d in &drafts_in_batch {
            println!(
                "    {:<14}  →  [{}]",
                d.draft_id,
                d.descriptor_ids
                    .iter()
                    .map(|id| id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
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
                d.descriptor_ids
                    .iter()
                    .map(|id| id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        let critic_marker = critic_pass_complete_marker(batch_id);
        let scorer_marker = scorer_batch_complete_marker(batch_id);
        println!(
            "  Sentinels:        critic={}  scorer={}",
            tick(has_diagnostic(ctx, &critic_marker)),
            tick(has_diagnostic(ctx, &scorer_marker))
        );
        println!();
    }
}

fn print_adversarial(ctx: &dyn Context) {
    let facts = adversarial_facts(ctx);
    print_section("AssumptionBreaker  (per-fact adversarial, organism-adversarial)");
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

fn compile_handoff(
    ctx: &dyn Context,
    catalog: &DiscoveryCatalog,
    templates: &FormationCatalog,
    providers: &ProviderDescriptorCatalog,
    request: &FormationCompileRequest,
) -> Result<organism_runtime::CompiledFormationPlan, Box<dyn std::error::Error>> {
    print_section("Compile handoff");
    let completed = completed_batches(ctx);
    println!("  completed batches (in order): {completed:?}");
    let latest = latest_completed_batch(ctx)
        .ok_or("no batch completed — design huddle produced nothing to compile")?;
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
        let role = catalog
            .get(entry.suggestor_id.as_str())
            .map(|d| format!("{:?}", d.descriptor.profile.role))
            .unwrap_or_else(|| "?".to_string());
        println!("    - {:<32}  ({role})", entry.suggestor_id.as_str());
    }
    println!();
    Ok(plan)
}

fn print_work_outcome(ctx: &dyn Context) {
    let evaluations = ctx.get(ContextKey::Evaluations);
    let constraints = ctx.get(ContextKey::Constraints);
    let diagnostics = ctx.get(ContextKey::Diagnostic);
    println!(
        "  facts produced:    Evaluations={}  Constraints={}  Diagnostic={}",
        evaluations.len(),
        constraints.len(),
        diagnostics.len(),
    );
    if !evaluations.is_empty() {
        println!("  Evaluations:");
        for fact in evaluations.iter().take(5) {
            println!(
                "    · {}: {}",
                fact.id().as_str(),
                fact.text().unwrap_or("(no text)")
            );
        }
        if evaluations.len() > 5 {
            println!("    · … {} more", evaluations.len() - 5);
        }
    }
    if !constraints.is_empty() {
        println!("  Constraints:");
        for fact in constraints.iter().take(5) {
            println!(
                "    · {}: {}",
                fact.id().as_str(),
                fact.text().unwrap_or("(no text)")
            );
        }
        if constraints.len() > 5 {
            println!("    · … {} more", constraints.len() - 5);
        }
    }
    println!();
}

// ---------------------------------------------------------------------------
// Scenario-owned glue (NOT mocks — close the round loop)
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
    serde_quick::extract_messages(raw).unwrap_or_else(|| raw.chars().take(80).collect())
}

mod serde_quick {
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

fn has_diagnostic(ctx: &dyn Context, id: &str) -> bool {
    ctx.get(ContextKey::Diagnostic)
        .iter()
        .any(|fact| fact.id().as_str() == id)
}

fn tick(b: bool) -> &'static str {
    if b { "✓" } else { "✗" }
}
