//! Atelier showcase: round-driven design Formation selection,
//! end-to-end with the real seeded catalog and the real default
//! executable factories.
//!
//! The flow this scenario demonstrates:
//!
//!   intent
//!     → design huddle Formation
//!         (RoundStarter → CatalogProposerSuggestor[round-driven,
//!          with_round_exclusion_policy(BlockedMinusPassedPolicy) +
//!          with_halt_marker(convergence)] → DraftValidatorCriticSuggestor
//!          → LlmCriticSuggestor[grades PASS/BLOCK + 0..=100 confidence]
//!          → AssumptionBreakerAgent
//!          → EvidenceWeightedScorer[combines all evidence]
//!          → ShortlistNoteEmitter → RoundSynthesizer
//!          → ConvergenceJudge[LLM, halts on CONVERGED]
//!          → RoundAdvancer)
//!     → two rounds, two batches, per-batch sentinels
//!     → round N+1's proposer reads round N's blocked drafts and
//!       excludes descriptors that appeared only in blocked rosters,
//!       so the deliberation actually evolves
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
//! `RoundSynthesizer` is wired with a real Manifold-backed
//! `SynthesisProducer` — provider-agnostic selection via
//! `ChatBackendSelectionConfig::from_env()` + `select_healthy_chat_backend`.
//! The deployment chooses the provider through Manifold-recognized credentials
//! plus optional `CONVERGE_LLM_PROFILE` / `CONVERGE_LLM_PROVIDER`. If no
//! healthy live backend is configured, the scenario exits honestly — no
//! deterministic stand-in.
//!
//! Per-round notes come from a small `ShortlistNoteEmitter` that
//! turns each completed batch's shortlist into one note the
//! synthesizer can read. That's scenario-owned glue, not a mock:
//! it makes a real fact, just one the platform doesn't ship a
//! default for.
//!
//! Run with:
//!
//!     cargo run -p scenario-round-driven-formation-design

use std::sync::Arc;

use arbiter::{ContextIn, DecideRequest, PrincipalIn, ResourceIn};
use async_trait::async_trait;
use converge_core::{AuthorityLevel, FlowAction};
use converge_kernel::formation::{
    FormationCatalog, FormationTemplate, FormationTemplateMetadata, FormationTemplateQuery,
    StaticFormationTemplate, SuggestorCapability, SuggestorRole,
};
use converge_kernel::{AgentEffect, Context, ContextFact, ContextKey};
use converge_pack::{Provenance, ProvenanceSource, Suggestor, TextPayload};
use converge_provider::{
    ChatBackendSelectionConfig, ChatMessage, ChatRequest, ChatRole, DynChatBackend, ResponseFormat,
};
use manifold::llm::select_healthy_chat_backend;
use organism_adversarial::AssumptionBreakerAgent;
use organism_catalog::{DiscoveryCatalog, ProviderDescriptorCatalog};
use organism_catalog_seed as seed;
use organism_dynamics::{
    BlockedMinusPassedPolicy, CatalogProposerSuggestor, DraftValidation,
    DraftValidatorCriticSuggestor, DraftVerdict, FormationDraft, compile_draft, completed_batches,
    critic_pass_complete_marker, extract_draft_validations, extract_drafts,
    extract_drafts_for_batch, latest_completed_batch, proposer_exclusions_marker,
    scorer_batch_complete_marker,
};
use organism_runtime::huddle::{
    RoundConventions, RoundStarter, RoundSynthesizer, SynthesisProducer,
};
use organism_runtime::{
    ExecutableSuggestorCatalog, Formation, FormationCompileRequest, FormationCompiler, Seed,
    register_default_factories,
};
use uuid::Uuid;

const ROUND_SIGNAL_PREFIX: &str = "design-round-";

/// Cedar policy guarding the work formation. Permits `promote` when
/// the action amount is ≤ $5000 OR the request carries explicit
/// human approval. The candidate plan in this scenario costs $7500
/// and lacks human approval, so the gate denies — and the engine
/// resolves the denial to `Escalate` (no auto-permit fallback).
const WORK_FORMATION_POLICY: &str = r#"
permit(principal, action == Action::"promote", resource)
when {
  context.amount <= 5000 ||
  context.human_approval_present == true
};
"#;
const CONTINUE_PREFIX: &str = "design-round:continue:";
const NOTE_PREFIX: &str = "design-huddle-note:";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // .env is for local dev only. Cloud (GCP Secret Manager) and CI
    // (GitHub Actions secrets) populate env vars without dotenv.
    // dotenvy is silent if .env is absent — that's the right shape.
    let _ = dotenvy::dotenv();

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

    // ── 2. Provider-agnostic LLM selection. The code declares
    //       needs via ChatBackendSelectionConfig (criteria pulled
    //       from CONVERGE_LLM_PROFILE env, or defaulting to
    //       interactive). Manifold's selection layer probes the
    //       configured Manifold credentials and returns the first
    //       healthy backend, or fails honestly with no silent fallback.
    let llm_config = ChatBackendSelectionConfig::from_env()?;
    let selected = select_healthy_chat_backend(&llm_config).await.map_err(
        |err| -> Box<dyn std::error::Error> {
            format!(
                "no healthy Manifold chat backend available: {err}. \
                 Configure a live provider credential supported by Manifold \
                 in .env (local) / GitHub Actions secrets (CI) / GCP Secret \
                 Manager (cloud). CONVERGE_LLM_PROFILE selects the criteria \
                 (interactive / analysis / batch / high_volume)."
            )
            .into()
        },
    )?;

    print_intent(&request, &catalog, &executables, &selected);

    // ── 3. Build the design huddle Formation. Round synthesis is
    //       Manifold-backed — the producer wraps the selected
    //       chat backend and turns per-round notes into a synthesis
    //       fact.
    let round_starter = RoundStarter::new(3).with_conventions(conventions);
    let proposer = CatalogProposerSuggestor::new(
        catalog.clone(),
        templates.clone(),
        providers.clone(),
        request.clone(),
        2,
    )
    .with_round_signals(ROUND_SIGNAL_PREFIX)
    .with_round_exclusion_policy(BlockedMinusPassedPolicy::new())
    .with_halt_marker(CONVERGENCE_STOP_MARKER);
    let critic = DraftValidatorCriticSuggestor::new(
        catalog.clone(),
        templates.clone(),
        providers.clone(),
        request.clone(),
    );
    let adversarial = AssumptionBreakerAgent::new();
    let llm_critic = LlmCriticSuggestor::new(
        selected.backend.clone(),
        catalog.clone(),
        "audit a candidate plan for policy compliance and anomalies".to_string(),
        vec![SuggestorRole::Constraint],
        vec![
            SuggestorCapability::PolicyEnforcement,
            SuggestorCapability::Analytics,
        ],
    );
    let scorer = EvidenceWeightedScorer::new(1);
    let synthesizer = RoundSynthesizer::new(1, LlmSynthesisProducer::new(selected.backend.clone()))
        .with_conventions(conventions);

    let convergence_judge = ConvergenceJudge::new(selected.backend.clone());

    let huddle = Formation::new("round-driven-design-huddle")
        .agent_boxed(Box::new(round_starter))
        .agent_boxed(Box::new(proposer))
        .agent_boxed(Box::new(critic))
        .agent_boxed(Box::new(adversarial))
        .agent_boxed(Box::new(llm_critic))
        .agent_boxed(Box::new(scorer))
        .agent_boxed(Box::new(ShortlistNoteEmitter))
        .agent_boxed(Box::new(synthesizer))
        .agent_boxed(Box::new(convergence_judge))
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

    // ── 4. Print what the design huddle produced.
    print_rounds(ctx);
    print_adversarial(ctx);
    print_convergence(ctx);

    // ── 5. Compile handoff: pick latest_completed_batch, validate
    //       its shortlist against the real catalog, then instantiate
    //       the work Formation against the real factory set.
    let plan = compile_handoff(ctx, &catalog, &templates, &providers, &request)?;

    // ── 6. Actually run the work Formation in Converge.
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
            provenance: ScenarioProvenance.provenance(),
        },
        Seed {
            key: ContextKey::Strategies,
            id: "candidate-plan".into(),
            content: serde_json::to_string(&candidate_plan)?,
            provenance: ScenarioProvenance.provenance(),
        },
    ];
    // Arbiter typed-policy gate. The bespoke ConstraintCheckerAgent
    // does pass/fail against the JSON plan; arbiter expresses a
    // typed Cedar invariant ("permit promote when amount ≤ 5000 OR
    // human approval present"). The plan has cost = 7500 and no
    // human approval, so arbiter should DENY/ESCALATE while the
    // mechanical checker passes — demonstrating richer governance
    // riding alongside the deterministic critic, not replacing it.
    let policy_engine = Arc::new(
        arbiter::PolicyEngine::from_policy_str(WORK_FORMATION_POLICY)
            .map_err(|e| format!("policy parse: {e:?}"))?,
    );
    let policy_gate = arbiter::PolicyGateSuggestor::with_keys(
        policy_engine,
        ContextKey::Hypotheses,
        ContextKey::Constraints,
    );
    let decide_request_emitter = DecideRequestEmitter::for_candidate_plan(7500);

    let work = executables
        .instantiate(&plan, seeds)?
        .agent_boxed(Box::new(decide_request_emitter))
        .agent_boxed(Box::new(policy_gate));
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
    selected: &manifold::llm::SelectedChatBackend,
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
    println!(
        "  llm backend:    {} / {} (provider-agnostic selection via Manifold)",
        selected.provider(),
        selected.model()
    );
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

        let drafts_present = drafts.iter().any(|d| d.draft_batch_id == *batch_id);
        if !drafts_present {
            if convergence_reached(ctx) {
                println!("  (halted by convergence judge — no drafts proposed)");
            } else {
                println!("  (no drafts produced)");
            }
            println!();
            continue;
        }

        let exclusions_id = proposer_exclusions_marker(batch_id);
        if let Some(fact) = ctx
            .get(ContextKey::Diagnostic)
            .iter()
            .find(|f| f.id().as_str() == exclusions_id)
        {
            println!(
                "  Proposer evolution: {}",
                fact.text().unwrap_or("(no text)")
            );
        }

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

        println!("  Draft critic  (mechanical, organism-dynamics):");
        for d in &drafts_in_batch {
            if let Some(v) = find_validation_by_critic(
                &passes,
                batch_id,
                &d.draft_id,
                "organism-draft-validator-critic",
            ) {
                println!("    {:<14}  →  PASS  ({})", d.draft_id, v.reason);
            } else if let Some(v) = find_validation_by_critic(
                &blocks,
                batch_id,
                &d.draft_id,
                "organism-draft-validator-critic",
            ) {
                println!("    {:<14}  →  BLOCK ({})", d.draft_id, v.reason);
            } else {
                println!("    {:<14}  →  (no verdict)", d.draft_id);
            }
        }

        println!("  LLM critic    (semantic, Manifold-selected backend):");
        for d in &drafts_in_batch {
            if let Some(v) =
                find_validation_by_critic(&passes, batch_id, &d.draft_id, LLM_CRITIC_NAME)
            {
                println!("    {:<14}  →  PASS  ({})", d.draft_id, v.reason);
            } else if let Some(v) =
                find_validation_by_critic(&blocks, batch_id, &d.draft_id, LLM_CRITIC_NAME)
            {
                println!("    {:<14}  →  BLOCK ({})", d.draft_id, v.reason);
            } else {
                println!("    {:<14}  →  (no verdict — late or failed)", d.draft_id);
            }
        }

        let scorecard_id = format!("evidence-scorecard-{batch_id}");
        if let Some(fact) = ctx
            .get(ContextKey::Diagnostic)
            .iter()
            .find(|f| f.id().as_str() == scorecard_id)
        {
            println!(
                "  Evidence scorecard: {}",
                fact.text().unwrap_or("(no text)")
            );
        }

        let shortlist_for_batch: Vec<&FormationDraft> = shortlist_all
            .iter()
            .filter(|d| d.draft_batch_id == *batch_id)
            .collect();
        println!("  Shortlist (evidence-weighted):");
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

        let conv_id = format!("design-convergence:{round_n}");
        if let Some(fact) = ctx
            .get(ContextKey::Hypotheses)
            .iter()
            .find(|f| f.id().as_str() == conv_id)
        {
            println!(
                "  Convergence judge: {}",
                fact.text().unwrap_or("(no text)")
            );
        }

        // Synthesis facts land under the synthesis_key in
        // RoundConventions (Hypotheses here) with id
        // `design-synthesis:{N}`. Show the LLM's actual text so the
        // trace proves the chat backend was on the critical path,
        // not just configured.
        let synth_id = format!("design-synthesis:{round_n}");
        if let Some(synth) = ctx
            .get(ContextKey::Hypotheses)
            .iter()
            .find(|f| f.id().as_str() == synth_id)
        {
            println!("  RoundSynthesizer (LLM):");
            for line in synth.text().unwrap_or("(no text)").lines() {
                println!("    {line}");
            }
        } else {
            println!("  RoundSynthesizer:  (no synthesis fact for this round)");
        }
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

fn print_convergence(ctx: &dyn Context) {
    print_section("Convergence outcome");
    if let Some(fact) = ctx
        .get(ContextKey::Diagnostic)
        .iter()
        .find(|f| f.id().as_str() == CONVERGENCE_STOP_MARKER)
    {
        println!(
            "  ✓ deliberation halted early: {}",
            fact.text().unwrap_or("(no text)")
        );
    } else {
        println!("  ran to max rounds — no early-stop signal emitted");
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

    // Arbiter typed verdict — show it explicitly so the trace
    // distinguishes the mechanical critic's pass/fail from the
    // Cedar policy's typed Permit/Deny/Escalate.
    if let Some(policy_fact) = constraints
        .iter()
        .find(|f| f.id().as_str() == "policy-decision")
    {
        println!("  Arbiter (Cedar policy gate):");
        if let Ok(decision) = policy_fact.require_payload::<arbiter::PolicyDecision>() {
            println!("    outcome:   {:?}", decision.outcome);
            println!(
                "    reason:    {}",
                decision.reason.as_deref().unwrap_or("(none)")
            );
            println!(
                "    principal: {} → action: {:?} → resource: {}",
                decision.principal_id.as_str(),
                decision.action,
                decision.resource_id.as_str(),
            );
        } else {
            println!("    (raw): {}", policy_fact.text().unwrap_or("(no text)"));
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

impl ScenarioProvenance {
    fn provenance(self) -> Provenance {
        ProvenanceSource::provenance(self)
    }
}

fn round_number_from_design_batch_id(batch_id: &str) -> Option<u8> {
    batch_id
        .strip_prefix(ROUND_SIGNAL_PREFIX)
        .and_then(|n| n.parse::<u8>().ok())
}

/// `EvidenceWeightedScorer` — replaces the platform's
/// `BeautyContestSuggestor` in this scenario with a transparent
/// evidence-weighted ranking. For each batch, once BOTH the
/// mechanical critic sentinel and the LLM critic sentinel are
/// present (so all verdicts are visible), it:
///
/// 1. Drops drafts blocked by ANY critic (mechanical or LLM).
/// 2. For surviving drafts, computes a score:
///    `100 * mechanical_pass + llm_confidence - 10 * adversarial_count`.
/// 3. Emits the top-k as `FormationDraft`s under `ContextKey::Proposals`
///    (same shape as `BeautyContestSuggestor`'s output, so the rest of
///    the pipeline — `print_rounds`, `compile_handoff`,
///    `completed_batches` — is unchanged).
/// 4. Emits a per-batch scorecard fact under `Diagnostic` for trace
///    visibility (id: `evidence-scorecard-{batch_id}`).
/// 5. Emits the standard scorer-batch-complete sentinel via
///    `scorer_batch_complete_marker(batch_id)` so `RoundAdvancer` and
///    `latest_completed_batch` keep working.
///
/// Round-over-round impact: with the LLM critic's confidence
/// available, two drafts that both pass on mechanical+LLM are no
/// longer tied by descriptor-count alone — the model's own grade of
/// how strongly it endorses each roster picks the winner.
struct EvidenceWeightedScorer {
    top_n: usize,
}

const EVIDENCE_SCORER_NAME: &str = "scenario-evidence-weighted-scorer";

#[derive(Debug, Clone)]
struct EvidenceScorecard {
    draft_id: String,
    mechanical_pass: bool,
    llm_pass: bool,
    llm_confidence: u8,
    adversarial_findings: u8,
    eligible: bool,
    score: i32,
}

impl EvidenceScorecard {
    fn format(&self) -> String {
        format!(
            "{draft_id}: mech={mech} llm={llm}(conf={conf}) adv={adv} eligible={elig} score={score}",
            draft_id = self.draft_id,
            mech = if self.mechanical_pass {
                "PASS"
            } else {
                "BLOCK"
            },
            llm = if self.llm_pass { "PASS" } else { "BLOCK" },
            conf = self.llm_confidence,
            adv = self.adversarial_findings,
            elig = self.eligible,
            score = self.score,
        )
    }
}

impl EvidenceWeightedScorer {
    fn new(top_n: usize) -> Self {
        Self { top_n }
    }

    /// Batches with drafts that we haven't scored yet AND for which
    /// both critic sentinels are present.
    fn pending_batches(&self, ctx: &dyn Context) -> Vec<String> {
        let drafts = extract_drafts(ctx, ContextKey::Strategies);
        if drafts.is_empty() {
            return Vec::new();
        }
        let already_completed: std::collections::BTreeSet<String> =
            completed_batches(ctx).into_iter().collect();
        let diagnostic_ids: std::collections::BTreeSet<String> = ctx
            .get(ContextKey::Diagnostic)
            .iter()
            .map(|f| f.id().as_str().to_string())
            .collect();
        let mut by_batch: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for d in &drafts {
            let b = d.draft_batch_id.to_string();
            if already_completed.contains(&b) {
                continue;
            }
            let mech_sentinel = critic_pass_complete_marker(&b);
            let llm_sentinel = format!("llm-critic-pass-complete-{b}");
            if diagnostic_ids.contains(&mech_sentinel) && diagnostic_ids.contains(&llm_sentinel) {
                by_batch.insert(b);
            }
        }
        by_batch.into_iter().collect()
    }

    fn scorecard_for_batch(ctx: &dyn Context, batch_id: &str) -> Vec<EvidenceScorecard> {
        let drafts: Vec<FormationDraft> = extract_drafts(ctx, ContextKey::Strategies)
            .into_iter()
            .filter(|d| d.draft_batch_id.as_str() == batch_id)
            .collect();
        let passes = extract_draft_validations(ctx, ContextKey::Evaluations);
        let blocks = extract_draft_validations(ctx, ContextKey::Constraints);

        drafts
            .iter()
            .map(|d| {
                let draft_id = d.draft_id.to_string();
                let mech_block = blocks.iter().any(|v| {
                    v.draft_batch_id == batch_id
                        && v.draft_id == draft_id
                        && v.critic == "organism-draft-validator-critic"
                });
                let llm_block = blocks.iter().any(|v| {
                    v.draft_batch_id == batch_id
                        && v.draft_id == draft_id
                        && v.critic == LLM_CRITIC_NAME
                });
                let mech_pass = passes.iter().any(|v| {
                    v.draft_batch_id == batch_id
                        && v.draft_id == draft_id
                        && v.critic == "organism-draft-validator-critic"
                });
                let llm_pass = passes.iter().any(|v| {
                    v.draft_batch_id == batch_id
                        && v.draft_id == draft_id
                        && v.critic == LLM_CRITIC_NAME
                });
                let confidence_id = LlmCriticSuggestor::confidence_fact_id(batch_id, &draft_id);
                let llm_confidence: u8 = ctx
                    .get(ContextKey::Diagnostic)
                    .iter()
                    .find(|f| f.id().as_str() == confidence_id)
                    .and_then(|f| f.text())
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(0);
                let suffix = format!("-{}", draft_id.trim_start_matches("candidate-"));
                let adversarial_findings: u8 = ctx
                    .get(ContextKey::Evaluations)
                    .iter()
                    .chain(ctx.get(ContextKey::Constraints).iter())
                    .filter(|fact| {
                        let fid = fact.id().as_str();
                        fid.starts_with("assumption-") && fid.ends_with(&suffix)
                    })
                    .count()
                    .min(255) as u8;
                let eligible = !mech_block && !llm_block;
                let score = if eligible {
                    let m = if mech_pass { 100 } else { 0 };
                    m + i32::from(llm_confidence) - 10 * i32::from(adversarial_findings)
                } else {
                    i32::MIN
                };
                EvidenceScorecard {
                    draft_id,
                    mechanical_pass: mech_pass,
                    llm_pass,
                    llm_confidence,
                    adversarial_findings,
                    eligible,
                    score,
                }
            })
            .collect()
    }
}

#[async_trait]
impl Suggestor for EvidenceWeightedScorer {
    fn name(&self) -> &'static str {
        EVIDENCE_SCORER_NAME
    }
    fn dependencies(&self) -> &[ContextKey] {
        &[
            ContextKey::Strategies,
            ContextKey::Evaluations,
            ContextKey::Constraints,
            ContextKey::Diagnostic,
        ]
    }
    fn provenance(&self) -> Provenance {
        ScenarioProvenance.provenance()
    }
    fn accepts(&self, ctx: &dyn Context) -> bool {
        !self.pending_batches(ctx).is_empty()
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let mut effect = AgentEffect::builder();
        for batch_id in self.pending_batches(ctx) {
            let mut cards = Self::scorecard_for_batch(ctx, &batch_id);
            cards.sort_by(|a, b| {
                b.score
                    .cmp(&a.score)
                    .then_with(|| a.draft_id.cmp(&b.draft_id))
            });

            let drafts: Vec<FormationDraft> = extract_drafts(ctx, ContextKey::Strategies)
                .into_iter()
                .filter(|d| d.draft_batch_id.as_str() == batch_id)
                .collect();

            let scorecard_text = cards
                .iter()
                .map(EvidenceScorecard::format)
                .collect::<Vec<_>>()
                .join(" | ");
            effect = effect.proposal(ScenarioProvenance.proposed_fact(
                ContextKey::Diagnostic,
                format!("evidence-scorecard-{batch_id}"),
                TextPayload::new(format!(
                    "{EVIDENCE_SCORER_NAME}: scorecard for {batch_id} → {scorecard_text}"
                )),
            ));

            let mut shortlisted = 0usize;
            for card in cards.iter().filter(|c| c.eligible).take(self.top_n) {
                let Some(draft) = drafts.iter().find(|d| d.draft_id.as_str() == card.draft_id)
                else {
                    continue;
                };
                let shortlist = FormationDraft::new(
                    draft.draft_id.clone(),
                    draft.draft_batch_id.clone(),
                    draft.descriptor_ids.clone(),
                    format!(
                        "Shortlisted #{shortlisted}/{n} by {EVIDENCE_SCORER_NAME} \
                         (batch: {batch_id}, score={score}: mech={mech} llm={llm} \
                         conf={conf} adv={adv}; source: {src}).",
                        n = self.top_n,
                        score = card.score,
                        mech = if card.mechanical_pass {
                            "PASS"
                        } else {
                            "BLOCK"
                        },
                        llm = if card.llm_pass { "PASS" } else { "BLOCK" },
                        conf = card.llm_confidence,
                        adv = card.adversarial_findings,
                        src = draft.source,
                    ),
                    draft.source.clone(),
                );
                let json = match serde_json::to_string(&shortlist) {
                    Ok(s) => s,
                    Err(err) => {
                        effect = effect.proposal(ScenarioProvenance.proposed_fact(
                            ContextKey::Diagnostic,
                            format!(
                                "evidence-shortlist-serialize-error-{batch_id}-{shortlisted}"
                            ),
                            TextPayload::new(format!(
                                "{EVIDENCE_SCORER_NAME}: failed to serialize shortlist {shortlisted} for batch {batch_id}: {err}"
                            )),
                        ));
                        continue;
                    }
                };
                effect = effect.proposal(ScenarioProvenance.proposed_fact(
                    ContextKey::Proposals,
                    format!("evidence-shortlist-{batch_id}-{shortlisted}"),
                    TextPayload::new(json),
                ));
                shortlisted += 1;
            }

            effect = effect.proposal(ScenarioProvenance.proposed_fact(
                ContextKey::Diagnostic,
                scorer_batch_complete_marker(&batch_id),
                TextPayload::new(format!(
                    "{EVIDENCE_SCORER_NAME}: scoring complete for batch {batch_id}"
                )),
            ));
        }
        effect.build()
    }
}

const CONVERGENCE_STOP_MARKER: &str = "design-convergence-reached";

fn convergence_reached(ctx: &dyn Context) -> bool {
    ctx.get(ContextKey::Diagnostic)
        .iter()
        .any(|f| f.id().as_str() == CONVERGENCE_STOP_MARKER)
}

/// `ConvergenceJudge` — LLM-driven cross-round convergence detector.
///
/// After a round completes scoring, the judge compares its shortlist
/// (descriptor sets) to the prior round's. Even when the descriptor
/// sets differ, the LLM is asked to decide whether the deliberation
/// has *materially* converged: same coverage profile, same capability
/// trade-offs, or a stable preferred roster across rounds. The
/// verdict + reason are emitted as one fact per round under
/// `Hypotheses`, and a single `design-convergence-reached` marker
/// under `Diagnostic` is emitted the first time the judge says
/// CONVERGED. `CatalogProposerSuggestor::with_halt_marker(...)` reads that marker
/// and halts, so subsequent unstarted rounds never produce drafts —
/// the work formation handoff picks `latest_completed_batch` and
/// proceeds.
///
/// Per-round idempotent: judgment fact id is `design-convergence:{N}`.
/// On chat or parse failure: emits Diagnostic and abstains (no
/// silent CONVERGED). Without a verdict, the proposer keeps firing
/// for remaining rounds.
struct ConvergenceJudge {
    backend: Arc<dyn DynChatBackend>,
}

const CONVERGENCE_JUDGE_NAME: &str = "scenario-convergence-judge";

impl ConvergenceJudge {
    fn new(backend: Arc<dyn DynChatBackend>) -> Self {
        Self { backend }
    }

    fn judgment_fact_id(round_n: u8) -> String {
        format!("design-convergence:{round_n}")
    }

    /// Round numbers N >= 2 for which:
    ///   - round N completed scoring (sentinel under Diagnostic)
    ///   - round N-1 completed scoring
    ///   - we have not yet emitted a judgment for round N
    ///   - we have not yet emitted the stop marker (once we say
    ///     CONVERGED we stop checking — subsequent rounds will not
    ///     run anyway thanks to the proposer guard).
    fn pending(ctx: &dyn Context) -> Vec<u8> {
        if convergence_reached(ctx) {
            return Vec::new();
        }
        let completed: std::collections::BTreeSet<u8> = completed_batches(ctx)
            .into_iter()
            .filter_map(|b| round_number_from_design_batch_id(&b))
            .collect();
        let judged: std::collections::BTreeSet<String> = ctx
            .get(ContextKey::Hypotheses)
            .iter()
            .map(|f| f.id().as_str().to_string())
            .collect();
        let mut pending: Vec<u8> = completed
            .iter()
            .copied()
            .filter(|n| {
                *n >= 2
                    && completed.contains(&(n - 1))
                    && !judged.contains(&Self::judgment_fact_id(*n))
            })
            .collect();
        pending.sort_unstable();
        pending.dedup();
        pending
    }

    fn shortlist_descriptors(ctx: &dyn Context, batch_id: &str) -> Vec<String> {
        let mut descriptors: Vec<String> =
            extract_drafts_for_batch(ctx, ContextKey::Proposals, batch_id)
                .iter()
                .flat_map(|d| d.descriptor_ids.iter().map(|id| id.as_str().to_string()))
                .collect();
        descriptors.sort();
        descriptors.dedup();
        descriptors
    }

    fn synthesis_text(ctx: &dyn Context, round_n: u8) -> String {
        let id = format!("design-synthesis:{round_n}");
        ctx.get(ContextKey::Hypotheses)
            .iter()
            .find(|f| f.id().as_str() == id)
            .and_then(|f| f.text())
            .unwrap_or("(no synthesis recorded for this round)")
            .to_string()
    }

    fn compose_prompt(
        prev_n: u8,
        curr_n: u8,
        prev_descriptors: &[String],
        curr_descriptors: &[String],
        prev_synth: &str,
        curr_synth: &str,
    ) -> String {
        format!(
            "Two consecutive rounds of a design huddle completed. \
             Decide whether the deliberation has materially converged \
             (same preferred roster, same coverage profile, same \
             capability trade-offs — even if descriptor lists differ \
             in incidental ways) or is still genuinely shifting.\n\
             \n\
             Round {prev_n} shortlist descriptors: [{prev_descs}]\n\
             Round {prev_n} synthesis:\n{prev_synth}\n\
             \n\
             Round {curr_n} shortlist descriptors: [{curr_descs}]\n\
             Round {curr_n} synthesis:\n{curr_synth}\n",
            prev_descs = prev_descriptors.join(", "),
            curr_descs = curr_descriptors.join(", "),
        )
    }

    fn parse_response(text: &str) -> Result<(bool, String), String> {
        let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
        let first = lines.next().ok_or("empty response")?;
        let converged = match first.to_ascii_uppercase().as_str() {
            "CONVERGED" => true,
            "DIVERGED" => false,
            other => {
                return Err(format!(
                    "expected CONVERGED or DIVERGED on line 1, got: {other:?}"
                ));
            }
        };
        let reason = lines.next().ok_or("missing reason on line 2")?.to_string();
        Ok((converged, reason))
    }
}

#[async_trait]
impl Suggestor for ConvergenceJudge {
    fn name(&self) -> &'static str {
        CONVERGENCE_JUDGE_NAME
    }
    fn dependencies(&self) -> &[ContextKey] {
        &[
            ContextKey::Diagnostic,
            ContextKey::Proposals,
            ContextKey::Hypotheses,
        ]
    }
    fn provenance(&self) -> Provenance {
        ScenarioProvenance.provenance()
    }
    fn accepts(&self, ctx: &dyn Context) -> bool {
        !Self::pending(ctx).is_empty()
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let mut effect = AgentEffect::builder();
        for round_n in Self::pending(ctx) {
            let prev_n = round_n - 1;
            let curr_batch = format!("{ROUND_SIGNAL_PREFIX}{round_n}");
            let prev_batch = format!("{ROUND_SIGNAL_PREFIX}{prev_n}");
            let curr_desc = Self::shortlist_descriptors(ctx, &curr_batch);
            let prev_desc = Self::shortlist_descriptors(ctx, &prev_batch);
            let curr_synth = Self::synthesis_text(ctx, round_n);
            let prev_synth = Self::synthesis_text(ctx, prev_n);
            let prompt = Self::compose_prompt(
                prev_n,
                round_n,
                &prev_desc,
                &curr_desc,
                &prev_synth,
                &curr_synth,
            );
            let request = ChatRequest {
                messages: vec![ChatMessage {
                    role: ChatRole::User,
                    content: prompt,
                    tool_calls: Vec::new(),
                    tool_call_id: None,
                }],
                system: Some(
                    "You judge whether a deliberation has materially converged. \
                     Reply with EXACTLY two non-empty lines:\n\
                     Line 1: CONVERGED or DIVERGED (one word, uppercase, nothing else).\n\
                     Line 2: One-sentence reason citing the specific evidence \
                     (descriptor sets, coverage profile, synthesis themes). Be honest — \
                     prefer DIVERGED if there is real shift; CONVERGED should mean the \
                     deliberation has stabilized."
                        .to_string(),
                ),
                tools: Vec::new(),
                response_format: ResponseFormat::Text,
                max_tokens: Some(150),
                temperature: Some(0.0),
                stop_sequences: Vec::new(),
                model: None,
            };
            match self.backend.chat(request).await {
                Ok(response) => match Self::parse_response(&response.content) {
                    Ok((converged, reason)) => {
                        let verdict_label = if converged { "CONVERGED" } else { "DIVERGED" };
                        let body = format!("round {prev_n}→{round_n}: {verdict_label} — {reason}");
                        effect = effect.proposal(ScenarioProvenance.proposed_fact(
                            ContextKey::Hypotheses,
                            Self::judgment_fact_id(round_n),
                            TextPayload::new(body.clone()),
                        ));
                        if converged {
                            effect = effect.proposal(ScenarioProvenance.proposed_fact(
                                ContextKey::Diagnostic,
                                CONVERGENCE_STOP_MARKER.to_string(),
                                TextPayload::new(format!(
                                    "{CONVERGENCE_JUDGE_NAME}: convergence reached at round {round_n}: {reason}"
                                )),
                            ));
                        }
                    }
                    Err(parse_err) => {
                        effect = effect.proposal(ScenarioProvenance.proposed_fact(
                            ContextKey::Diagnostic,
                            format!("convergence-judge-parse-error-{round_n}"),
                            TextPayload::new(format!(
                                "{CONVERGENCE_JUDGE_NAME}: unparseable response for round {round_n}: {parse_err}. raw: {raw}",
                                raw = response.content,
                            )),
                        ));
                    }
                },
                Err(chat_err) => {
                    effect = effect.proposal(ScenarioProvenance.proposed_fact(
                        ContextKey::Diagnostic,
                        format!("convergence-judge-chat-error-{round_n}"),
                        TextPayload::new(format!(
                            "{CONVERGENCE_JUDGE_NAME}: chat backend error for round {round_n}: {chat_err}"
                        )),
                    ));
                }
            }
        }
        effect.build()
    }
}

/// `DecideRequestEmitter` — converts the work formation's seed
/// framing into a typed `arbiter::DecideRequest` fact under
/// `Hypotheses` so `PolicyGateSuggestor` can evaluate the Cedar
/// policy against the candidate plan. Per-fire idempotent.
struct DecideRequestEmitter {
    request: DecideRequest,
}

const DECIDE_REQUEST_FACT_ID: &str = "work-formation-decide-request";
const DECIDE_REQUEST_EMITTER_NAME: &str = "scenario-decide-request-emitter";

impl DecideRequestEmitter {
    fn for_candidate_plan(amount: i64) -> Self {
        Self {
            request: DecideRequest {
                principal: PrincipalIn {
                    id: "agent:work-formation".into(),
                    authority: AuthorityLevel::Participatory,
                    domains: vec!["procurement".into()],
                    policy_version: None,
                },
                resource: ResourceIn {
                    id: "plan:ALPHA-1".into(),
                    resource_type: None,
                    phase: None,
                    gates_passed: None,
                },
                action: FlowAction::Promote,
                context: Some(ContextIn {
                    commitment_type: Some("rollout".to_string()),
                    amount: Some(amount),
                    human_approval_present: Some(false),
                    required_gates_met: None,
                }),
                delegation_b64: None,
            },
        }
    }
}

#[async_trait]
impl Suggestor for DecideRequestEmitter {
    fn name(&self) -> &'static str {
        DECIDE_REQUEST_EMITTER_NAME
    }
    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }
    fn provenance(&self) -> Provenance {
        ScenarioProvenance.provenance()
    }
    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.has(ContextKey::Seeds)
            && !ctx
                .get(ContextKey::Hypotheses)
                .iter()
                .any(|f| f.id().as_str() == DECIDE_REQUEST_FACT_ID)
    }
    async fn execute(&self, _ctx: &dyn Context) -> AgentEffect {
        AgentEffect::with_proposal(ScenarioProvenance.proposed_fact(
            ContextKey::Hypotheses,
            DECIDE_REQUEST_FACT_ID,
            self.request.clone(),
        ))
    }
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
    fn provenance(&self) -> Provenance {
        ScenarioProvenance.provenance()
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

/// `ShortlistNoteEmitter` turns each completed batch's shortlist
/// into one note fact (under `ContextKey::Hypotheses`) that
/// `RoundSynthesizer` can read. The platform's `RoundSynthesizer`
/// expects notes ending with `:N` for round `N`; the conventions
/// in this scenario use `Hypotheses` as the note key.
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

/// Assemble a structured note that captures the full deliberation
/// for one round: every draft considered, every reviewer's verdict
/// with the reason text, every adversarial finding, the final
/// shortlist, and explicit divergence flags where two critics
/// disagreed. The note is plain text with a stable shape so the
/// synthesizer's prompt can reference the sections by name.
fn build_round_note(ctx: &dyn Context, round: u8, batch_id: &str) -> String {
    use std::fmt::Write as _;

    let drafts: Vec<FormationDraft> = extract_drafts(ctx, ContextKey::Strategies)
        .into_iter()
        .filter(|d| d.draft_batch_id == batch_id)
        .collect();
    let passes = extract_draft_validations(ctx, ContextKey::Evaluations);
    let blocks = extract_draft_validations(ctx, ContextKey::Constraints);
    let shortlist = extract_drafts_for_batch(ctx, ContextKey::Proposals, batch_id);

    let mut out = String::new();
    writeln!(
        out,
        "Round {round} — deliberation trace (batch: {batch_id})"
    )
    .unwrap();
    writeln!(out).unwrap();

    let exclusions_id = format!("evolving-proposer-exclusions-{batch_id}");
    let exclusions_text = ctx
        .get(ContextKey::Diagnostic)
        .iter()
        .find(|f| f.id().as_str() == exclusions_id)
        .and_then(|f| f.text().map(str::to_string));
    if let Some(line) = exclusions_text {
        writeln!(out, "Evolution context (input to this round's proposer):").unwrap();
        writeln!(out, "- {line}").unwrap();
        writeln!(out).unwrap();
    }
    writeln!(out, "Drafts considered:").unwrap();

    for d in &drafts {
        let descriptor_list = d
            .descriptor_ids
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(out, "- {} → [{}]", d.draft_id.as_str(), descriptor_list,).unwrap();
        writeln!(out, "    rationale: {}", d.rationale).unwrap();

        let mech_pass = find_validation_by_critic(
            &passes,
            batch_id,
            d.draft_id.as_str(),
            "organism-draft-validator-critic",
        );
        let mech_block = find_validation_by_critic(
            &blocks,
            batch_id,
            d.draft_id.as_str(),
            "organism-draft-validator-critic",
        );
        match (mech_pass, mech_block) {
            (Some(v), _) => writeln!(out, "    mechanical critic: PASS — {}", v.reason).unwrap(),
            (None, Some(v)) => {
                writeln!(out, "    mechanical critic: BLOCK — {}", v.reason).unwrap()
            }
            (None, None) => writeln!(out, "    mechanical critic: (no verdict)").unwrap(),
        }

        let llm_pass =
            find_validation_by_critic(&passes, batch_id, d.draft_id.as_str(), LLM_CRITIC_NAME);
        let llm_block =
            find_validation_by_critic(&blocks, batch_id, d.draft_id.as_str(), LLM_CRITIC_NAME);
        match (llm_pass, llm_block) {
            (Some(v), _) => writeln!(out, "    LLM critic:        PASS — {}", v.reason).unwrap(),
            (None, Some(v)) => {
                writeln!(out, "    LLM critic:        BLOCK — {}", v.reason).unwrap()
            }
            (None, None) => {
                writeln!(out, "    LLM critic:        (no verdict — late or failed)").unwrap()
            }
        }

        // Adversarial findings — match by suffix on the strategy
        // fact id. The proposer's wire id is
        // `formation-draft-{encoded_batch}-{index}`; the breaker
        // stamps `assumption-{verdict}-{strategy_fact_id}`.
        let strategy_fact_id_suffix =
            format!("-{}", d.draft_id.as_str().trim_start_matches("candidate-"),);
        let adversarial: Vec<String> = ctx
            .get(ContextKey::Evaluations)
            .iter()
            .chain(ctx.get(ContextKey::Constraints).iter())
            .filter_map(|fact| {
                let fid = fact.id().as_str();
                if !fid.starts_with("assumption-") || !fid.ends_with(&strategy_fact_id_suffix) {
                    return None;
                }
                let raw = fact.text().unwrap_or("");
                Some(compact_breaker_payload(raw))
            })
            .collect();
        if adversarial.is_empty() {
            writeln!(out, "    adversarial:       (no findings)").unwrap();
        } else {
            writeln!(out, "    adversarial:       {}", adversarial.join("; ")).unwrap();
        }

        // Divergence flag — explicit so the synthesizer can't miss it.
        let mech_verdict = mech_pass.map(|_| "PASS").or(mech_block.map(|_| "BLOCK"));
        let llm_verdict = llm_pass.map(|_| "PASS").or(llm_block.map(|_| "BLOCK"));
        if let (Some(m), Some(l)) = (mech_verdict, llm_verdict)
            && m != l
        {
            writeln!(out, "    ⚠ DIVERGENCE:      mechanical={m}, LLM={l}",).unwrap();
        }
        writeln!(out).unwrap();
    }

    writeln!(out, "Shortlist (top {n}):", n = shortlist.len()).unwrap();
    if shortlist.is_empty() {
        writeln!(out, "- (none — every draft blocked)").unwrap();
    } else {
        for d in &shortlist {
            let descriptors = d
                .descriptor_ids
                .iter()
                .map(|id| id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(out, "- {} → [{descriptors}]", d.draft_id.as_str()).unwrap();
        }
    }

    out
}

#[async_trait]
impl Suggestor for ShortlistNoteEmitter {
    fn name(&self) -> &'static str {
        "scenario-shortlist-note-emitter"
    }
    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Diagnostic, ContextKey::Proposals]
    }
    fn provenance(&self) -> Provenance {
        ScenarioProvenance.provenance()
    }
    fn accepts(&self, ctx: &dyn Context) -> bool {
        !Self::pending(ctx).is_empty()
    }
    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let mut effect = AgentEffect::builder();
        for round in Self::pending(ctx) {
            let batch_id = format!("{ROUND_SIGNAL_PREFIX}{round}");
            let note_body = build_round_note(ctx, round, &batch_id);
            // The platform's RoundSynthesizer filters notes by
            // suffix `:N` — keep the suffix shape even though the
            // prefix is scenario-specific.
            effect = effect.proposal(ScenarioProvenance.proposed_fact(
                ContextKey::Hypotheses,
                format!("{NOTE_PREFIX}{round}"),
                TextPayload::new(note_body),
            ));
        }
        effect.build()
    }
}

/// `LlmSynthesisProducer` — implements
/// `organism_runtime::huddle::SynthesisProducer` by calling a
/// Manifold-selected chat backend. The backend was picked at
/// startup from env-driven needs spec; this producer just composes
/// a prompt from the round's notes and routes through the trait.
///
/// On chat errors the producer returns the error string. The
/// scenario's `RoundSynthesizer` routes producer errors to
/// `ContextKey::Diagnostic` and skips that round — the work
/// formation still runs, the audit trail captures the failure.
struct LlmSynthesisProducer {
    backend: Arc<dyn DynChatBackend>,
}

impl LlmSynthesisProducer {
    fn new(backend: Arc<dyn DynChatBackend>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl SynthesisProducer for LlmSynthesisProducer {
    type Payload = TextPayload;

    async fn synthesize(
        &self,
        round: u8,
        notes: &[ContextFact],
        _ctx: &dyn Context,
    ) -> Result<Self::Payload, String> {
        let note_block = notes
            .iter()
            .filter_map(|fact| fact.text())
            .collect::<Vec<_>>()
            .join("\n");
        let user_prompt = format!(
            "Round {round} of a design huddle just completed. The deliberation trace below \
             lists the proposer's evolution context (descriptors excluded based on prior \
             rounds' Block verdicts, if any), every draft considered, every reviewer's \
             verdict with the verbatim reason, any adversarial findings, the final \
             shortlist, and explicit DIVERGENCE flags where two critics disagreed.\n\n\
             ---\n\
             {note_block}\n\
             ---\n\n\
             Write a synthesis (≤90 words, plain prose, no headings) that:\n\
             1. If an Evolution context is present, state what this round excluded and why \
                (cite the descriptor ids that were dropped from consideration).\n\
             2. Names which draft was shortlisted and why (cite the reviewer text).\n\
             3. Calls out any DIVERGENCE between the mechanical and LLM critics — \
                explain which reviewer's reasoning prevailed and the consequence.\n\
             4. Flags adversarial findings only if they materially affect the decision.\n\n\
             Do NOT restate the shortlist as your whole answer. Do NOT invent details \
             the trace does not state."
        );
        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: user_prompt,
                tool_calls: Vec::new(),
                tool_call_id: None,
            }],
            system: Some(
                "You synthesize the outcome of a deliberation round. Your job is to extract \
                 the load-bearing signal from a structured trace — what was decided, what \
                 disagreement was resolved, what was rejected and why. Be terse. Cite specific \
                 descriptor ids and reviewer reasons by name. Do not invent facts not in the \
                 trace; if a section is empty, do not claim there is open work where the trace \
                 does not say so."
                    .to_string(),
            ),
            tools: Vec::new(),
            response_format: ResponseFormat::Text,
            max_tokens: Some(260),
            temperature: Some(0.1),
            stop_sequences: Vec::new(),
            model: None,
        };
        let response = self
            .backend
            .chat(request)
            .await
            .map_err(|e| format!("chat backend error: {e}"))?;
        Ok(TextPayload::new(response.content))
    }
}

/// `LlmCriticSuggestor` — the LLM as a real design-huddle
/// participant. Reads each unjudged FormationDraft, gathers the
/// catalog metadata for every descriptor in the proposed roster,
/// composes a strict two-line judgment prompt, calls the selected
/// chat backend, parses the response, and emits a typed
/// `DraftValidation` (`Pass` → Evaluations / `Block` → Constraints)
/// just like the deterministic critic. The scorer's existing
/// blocked-id filter already respects any `Block` regardless of
/// which critic emitted it — no protocol change.
///
/// Per-fact idempotent: judgment fact ids are `llm-validation-
/// {batch_id}-{draft_id}`. The same draft cannot be re-judged.
/// Per-batch sentinel `llm-critic-pass-complete-{batch_id}` lands
/// in Diagnostic so a host can wait for LLM verdicts before
/// advancing — RoundAdvancer in this scenario does not, so late
/// LLM blocks are recorded but may not influence the current
/// round's shortlist. That's honest: the trace shows the
/// disagreement when it happens.
///
/// On any failure path (chat error, unparseable response) the
/// critic emits a Diagnostic fact and skips that draft. No
/// silent Pass-by-default; no fallback verdict.
struct LlmCriticSuggestor {
    backend: Arc<dyn DynChatBackend>,
    catalog: DiscoveryCatalog,
    intent: String,
    required_roles: Vec<SuggestorRole>,
    required_capabilities: Vec<SuggestorCapability>,
}

impl LlmCriticSuggestor {
    fn new(
        backend: Arc<dyn DynChatBackend>,
        catalog: DiscoveryCatalog,
        intent: String,
        required_roles: Vec<SuggestorRole>,
        required_capabilities: Vec<SuggestorCapability>,
    ) -> Self {
        Self {
            backend,
            catalog,
            intent,
            required_roles,
            required_capabilities,
        }
    }

    fn judgment_fact_id(batch_id: &str, draft_id: &str) -> String {
        format!("llm-validation-{batch_id}-{draft_id}")
    }

    fn batch_sentinel_id(batch_id: &str) -> String {
        format!("llm-critic-pass-complete-{batch_id}")
    }

    fn already_judged(ctx: &dyn Context, draft: &FormationDraft) -> bool {
        let id = Self::judgment_fact_id(&draft.draft_batch_id, &draft.draft_id);
        ctx.get(ContextKey::Evaluations)
            .iter()
            .chain(ctx.get(ContextKey::Constraints).iter())
            .any(|fact| fact.id().as_str() == id)
    }

    fn compose_prompt(&self, draft: &FormationDraft) -> String {
        let roles = self
            .required_roles
            .iter()
            .map(|r| format!("{r:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        let caps = self
            .required_capabilities
            .iter()
            .map(|c| format!("{c:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        let mut roster_block = String::new();
        for desc_id in &draft.descriptor_ids {
            let line = match self.catalog.get(desc_id.as_str()) {
                Some(entry) => {
                    let profile = &entry.descriptor.profile;
                    let cap_list = profile
                        .capabilities
                        .iter()
                        .map(|c| format!("{c:?}"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(
                        "- {} (role: {:?}, capabilities: [{}])\n  summary: {}\n  use_when: {}\n",
                        desc_id.as_str(),
                        profile.role,
                        cap_list,
                        entry.discovery.summary,
                        entry.discovery.use_when,
                    )
                }
                None => format!("- {} (NOT IN CATALOG)\n", desc_id.as_str()),
            };
            roster_block.push_str(&line);
        }
        format!(
            "Intent: {intent}\n\
             Template required roles: [{roles}]\n\
             Template required capabilities: [{caps}]\n\
             \n\
             Proposed roster:\n{roster_block}\n\
             Does this roster fit the intent and satisfy the template requirements?",
            intent = self.intent,
        )
    }

    /// Three-line strict parse. Returns (verdict, reason, confidence
    /// 0..=100) or an error string suitable for Diagnostic. The
    /// confidence is the model's own grade of how strongly its
    /// verdict holds — used by the evidence-weighted scorer to break
    /// ties between drafts that both passed.
    fn parse_response(text: &str) -> Result<(DraftVerdict, String, u8), String> {
        let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
        let first = lines.next().ok_or("empty response")?;
        let verdict = match first.to_ascii_uppercase().as_str() {
            "PASS" => DraftVerdict::Pass,
            "BLOCK" => DraftVerdict::Block,
            other => return Err(format!("expected PASS or BLOCK on line 1, got: {other:?}")),
        };
        let reason = lines.next().ok_or("missing reason on line 2")?.to_string();
        let conf_line = lines
            .next()
            .ok_or("missing confidence on line 3 (integer 0..=100)")?;
        let confidence: u8 = conf_line
            .parse()
            .map_err(|e| format!("confidence on line 3 not a u8 0..=100: {conf_line:?}: {e}"))?;
        if confidence > 100 {
            return Err(format!("confidence on line 3 out of range: {confidence}"));
        }
        Ok((verdict, reason, confidence))
    }

    fn confidence_fact_id(batch_id: &str, draft_id: &str) -> String {
        format!("llm-confidence-{batch_id}-{draft_id}")
    }
}

const LLM_CRITIC_NAME: &str = "scenario-llm-critic";

#[async_trait]
impl Suggestor for LlmCriticSuggestor {
    fn name(&self) -> &'static str {
        LLM_CRITIC_NAME
    }
    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Strategies]
    }
    fn provenance(&self) -> Provenance {
        ScenarioProvenance.provenance()
    }
    fn accepts(&self, ctx: &dyn Context) -> bool {
        extract_drafts(ctx, ContextKey::Strategies)
            .iter()
            .any(|d| !Self::already_judged(ctx, d))
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let drafts: Vec<FormationDraft> = extract_drafts(ctx, ContextKey::Strategies)
            .into_iter()
            .filter(|d| !Self::already_judged(ctx, d))
            .collect();

        let mut effect = AgentEffect::builder();
        let mut batches_touched: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();

        for draft in &drafts {
            batches_touched.insert(draft.draft_batch_id.to_string());
            let prompt = self.compose_prompt(draft);
            let request = ChatRequest {
                messages: vec![ChatMessage {
                    role: ChatRole::User,
                    content: prompt,
                    tool_calls: Vec::new(),
                    tool_call_id: None,
                }],
                system: Some(
                    "You judge proposed Suggestor rosters for a deliberation Formation. \
                     Reply with EXACTLY three non-empty lines:\n\
                     Line 1: PASS or BLOCK (one word, uppercase, nothing else).\n\
                     Line 2: One-sentence reason. Cite descriptor ids by name when relevant. \
                     Do NOT invent capabilities, descriptors, or template requirements that the \
                     input does not state.\n\
                     Line 3: Confidence integer 0..=100. How strongly your verdict holds: \
                     100 = certain, 50 = leaning, 0 = guessing. Calibrate honestly — \
                     downstream scoring uses this to break ties between drafts that all passed."
                        .to_string(),
                ),
                tools: Vec::new(),
                response_format: ResponseFormat::Text,
                max_tokens: Some(150),
                temperature: Some(0.0),
                stop_sequences: Vec::new(),
                model: None,
            };

            match self.backend.chat(request).await {
                Ok(response) => match Self::parse_response(&response.content) {
                    Ok((verdict, reason, confidence)) => {
                        let target_key = match verdict {
                            DraftVerdict::Pass => ContextKey::Evaluations,
                            DraftVerdict::Block => ContextKey::Constraints,
                        };
                        let validation = DraftValidation::new(
                            &draft.draft_id,
                            &draft.draft_batch_id,
                            verdict,
                            reason,
                            LLM_CRITIC_NAME,
                        );
                        let json = match serde_json::to_string(&validation) {
                            Ok(s) => s,
                            Err(e) => {
                                effect = effect.proposal(ScenarioProvenance.proposed_fact(
                                    ContextKey::Diagnostic,
                                    format!(
                                        "llm-critic-serialize-error-{}-{}",
                                        draft.draft_batch_id, draft.draft_id
                                    ),
                                    TextPayload::new(format!(
                                        "{LLM_CRITIC_NAME}: serialize error for {}/{}: {e}",
                                        draft.draft_batch_id, draft.draft_id
                                    )),
                                ));
                                continue;
                            }
                        };
                        effect = effect.proposal(ScenarioProvenance.proposed_fact(
                            target_key,
                            LlmCriticSuggestor::judgment_fact_id(
                                &draft.draft_batch_id,
                                &draft.draft_id,
                            ),
                            TextPayload::new(json),
                        ));
                        effect = effect.proposal(ScenarioProvenance.proposed_fact(
                            ContextKey::Diagnostic,
                            LlmCriticSuggestor::confidence_fact_id(
                                &draft.draft_batch_id,
                                &draft.draft_id,
                            ),
                            TextPayload::new(confidence.to_string()),
                        ));
                    }
                    Err(parse_err) => {
                        effect = effect.proposal(ScenarioProvenance.proposed_fact(
                            ContextKey::Diagnostic,
                            format!(
                                "llm-critic-parse-error-{}-{}",
                                draft.draft_batch_id, draft.draft_id
                            ),
                            TextPayload::new(format!(
                                "{LLM_CRITIC_NAME}: unparseable response for {}/{}: {parse_err}. raw: {raw}",
                                draft.draft_batch_id, draft.draft_id,
                                raw = response.content,
                            )),
                        ));
                    }
                },
                Err(chat_err) => {
                    effect = effect.proposal(ScenarioProvenance.proposed_fact(
                        ContextKey::Diagnostic,
                        format!(
                            "llm-critic-chat-error-{}-{}",
                            draft.draft_batch_id, draft.draft_id
                        ),
                        TextPayload::new(format!(
                            "{LLM_CRITIC_NAME}: chat backend error for {}/{}: {chat_err}",
                            draft.draft_batch_id, draft.draft_id
                        )),
                    ));
                }
            }
        }

        // Per-batch sentinel for any batch this fire touched. Hosts
        // that want to gate round advance on LLM completion can read
        // this; RoundAdvancer in this scenario does not (the trace
        // shows late verdicts if they arrive after the scorer).
        for batch_id in batches_touched {
            effect = effect.proposal(ScenarioProvenance.proposed_fact(
                ContextKey::Diagnostic,
                Self::batch_sentinel_id(&batch_id),
                TextPayload::new(format!("{LLM_CRITIC_NAME}: pass complete for {batch_id}")),
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

fn find_validation_by_critic<'a>(
    verdicts: &'a [DraftValidation],
    batch_id: &str,
    draft_id: &str,
    critic: &str,
) -> Option<&'a DraftValidation> {
    verdicts
        .iter()
        .find(|v| v.draft_batch_id == batch_id && v.draft_id == draft_id && v.critic == critic)
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
