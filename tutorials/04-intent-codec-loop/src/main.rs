//! Intent Codec: Due Diligence Loop
//!
//! Demonstrates the missing wiring pattern for the matching suggestors:
//!
//! 1. A human or upstream system seeds a loose Gherkin-ish intent.
//! 2. An `IntentCodecSuggestor` compiles that intent into:
//!    - a `FormationRequest` for role coverage
//!    - a `ProviderRequest` for backend capability coverage
//! 3. `FormationAssemblySuggestor` and `ProviderSelectionSuggestor` answer
//!    those requests inside the same engine run.
//! 4. The selected loop members emit their own role-specific outputs until the
//!    run reaches a fixed point.
//!
//! The important pattern is not "hardcode a pipeline". The intent stays loose;
//! Converge finds a viable formation from registered suggestors and providers.

use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
struct AtelierShowcaseProvenance;

impl converge_kernel::ProvenanceSource for AtelierShowcaseProvenance {
    fn as_str(&self) -> &'static str {
        "atelier-showcase.intent-codec-loop"
    }
}

const ATELIER_SHOWCASE_PROVENANCE: AtelierShowcaseProvenance = AtelierShowcaseProvenance;

fn atelier_showcase_provenance() -> Provenance {
    converge_kernel::ProvenanceSource::provenance(ATELIER_SHOWCASE_PROVENANCE)
}

use converge_kernel::Provenance;
use converge_kernel::{
    AgentEffect, Budget, Context, ContextKey, ContextState, ConvergeResult, Engine, FactPayload,
    ProposedFact, Suggestor, TextPayload,
    formation::{
        Capability, CostClass, FormationAssemblySuggestor, FormationPlan, FormationRequest,
        LatencyClass, ProfileSnapshot, ProviderAssignment, ProviderAssignmentPayload,
        ProviderRequest, ProviderRequestPayload, ProviderSelectionSuggestor, SuggestorCapability,
        SuggestorProfile, SuggestorRole,
    },
};
use converge_provider::{Backend, BackendKind};
use serde::{Deserialize, Serialize};

const SPEC_PREFIX: &str = "intent-spec:";
const ROUTE_PREFIX: &str = "intent-route:";
const FORMATION_REQUEST_PREFIX: &str = "formation-request:";
const FORMATION_PLAN_PREFIX: &str = "formation-plan:";
const PROVIDER_REQUEST_PREFIX: &str = "provider-request:";
const PROVIDER_ASSIGNMENT_PREFIX: &str = "provider-assignment:";
const NOTE_PREFIX: &str = "formation-note:";
const PROVIDER_GAP_PREFIX: &str = "missing-provider:";
const LOOP_STATUS_PREFIX: &str = "loop-status:";

const DEFAULT_REQUEST_ID: &str = "dd-acme-metrics";
const ACTIVATE_SUBSCRIPTION_REQUEST_ID: &str = "activate-subscription";
const REFILL_PREPAID_AI_CREDITS_REQUEST_ID: &str = "refill-prepaid-ai-credits";

const CONTEXT_KEYS: [ContextKey; 9] = [
    ContextKey::Seeds,
    ContextKey::Hypotheses,
    ContextKey::Strategies,
    ContextKey::Constraints,
    ContextKey::Signals,
    ContextKey::Competitors,
    ContextKey::Evaluations,
    ContextKey::Proposals,
    ContextKey::Diagnostic,
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct IntentRoute {
    request_id: String,
    objective: String,
    formation_kind: String,
    required_roles: Vec<SuggestorRole>,
    required_provider_capabilities: Vec<Capability>,
    notes: Vec<String>,
}

impl FactPayload for IntentRoute {
    const FAMILY: &'static str = "tutorial.intent_codec.route";
    const VERSION: u16 = 1;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct ProviderGapNotice {
    request_id: String,
    suggestor: String,
    role: SuggestorRole,
    missing_capabilities: Vec<Capability>,
    message: String,
}

impl FactPayload for ProviderGapNotice {
    const FAMILY: &'static str = "tutorial.intent_codec.provider_gap";
    const VERSION: u16 = 1;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct MemberNote {
    request_id: String,
    suggestor: String,
    role: SuggestorRole,
    summary: String,
    provider_needs: Vec<Capability>,
    provider_backends: Vec<String>,
}

impl FactPayload for MemberNote {
    const FAMILY: &'static str = "tutorial.intent_codec.member_note";
    const VERSION: u16 = 1;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct LoopStatus {
    request_id: String,
    ready: bool,
    formation_coverage_ratio: f64,
    provider_coverage_ratio: f64,
    assigned_suggestors: Vec<String>,
    provider_backends: Vec<String>,
    unmatched_roles: Vec<SuggestorRole>,
    unmatched_capabilities: Vec<Capability>,
    provider_gaps: Vec<String>,
}

impl FactPayload for LoopStatus {
    const FAMILY: &'static str = "tutorial.intent_codec.loop_status";
    const VERSION: u16 = 1;
}

#[derive(Debug, Clone)]
struct CompiledIntent {
    objective: String,
    formation_kind: &'static str,
    required_roles: Vec<SuggestorRole>,
    required_provider_capabilities: Vec<Capability>,
    notes: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct DemoMode {
    label: &'static str,
    request_id: &'static str,
    spec: &'static str,
    applet: Option<AppletBrief>,
}

#[derive(Debug, Clone, Copy)]
struct AppletBrief {
    truth_key: &'static str,
    functional_need: &'static str,
    emotional_need: &'static str,
    relational_need: &'static str,
    authority_points: &'static [&'static str],
    evidence_sources: &'static [&'static str],
    runtime_needs: &'static [&'static str],
    commercial_needs: &'static [&'static str],
    projection: &'static str,
}

#[derive(Debug, Clone)]
struct ProfiledMember {
    name: &'static str,
    role: SuggestorRole,
    output_keys: Vec<ContextKey>,
    capabilities: Vec<SuggestorCapability>,
    provider_needs: Vec<Capability>,
    synopsis: &'static str,
    cost_hint: CostClass,
    latency_hint: LatencyClass,
    confidence_min: f32,
    confidence_max: f32,
}

impl ProfiledMember {
    fn output_key(&self) -> ContextKey {
        self.output_keys[0]
    }

    fn note_id(&self, request_id: &str) -> String {
        format!("{NOTE_PREFIX}{request_id}:{}", self.name)
    }

    fn provider_gap_id(&self, request_id: &str) -> String {
        format!("{PROVIDER_GAP_PREFIX}{request_id}:{}", self.name)
    }
}

impl SuggestorProfile for ProfiledMember {
    fn role(&self) -> SuggestorRole {
        self.role
    }

    fn output_keys(&self) -> &[ContextKey] {
        &self.output_keys
    }

    fn cost_hint(&self) -> CostClass {
        self.cost_hint
    }

    fn latency_hint(&self) -> LatencyClass {
        self.latency_hint
    }

    fn capabilities(&self) -> &[SuggestorCapability] {
        &self.capabilities
    }

    fn confidence_range(&self) -> (f32, f32) {
        (self.confidence_min, self.confidence_max)
    }
}

#[async_trait::async_trait]
impl Suggestor for ProfiledMember {
    fn name(&self) -> &str {
        self.name
    }

    fn provenance(&self) -> Provenance {
        atelier_showcase_provenance()
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Strategies]
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        assigned_requests(ctx, self.name)
            .into_iter()
            .any(|request_id| {
                !fact_exists_anywhere(ctx, &self.note_id(&request_id))
                    && !fact_exists(
                        ctx,
                        ContextKey::Constraints,
                        &self.provider_gap_id(&request_id),
                    )
            })
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let mut proposals = Vec::new();

        for request_id in assigned_requests(ctx, self.name) {
            if fact_exists_anywhere(ctx, &self.note_id(&request_id))
                || fact_exists(
                    ctx,
                    ContextKey::Constraints,
                    &self.provider_gap_id(&request_id),
                )
            {
                continue;
            }

            let Some(provider_assignment) = provider_assignment_for(ctx, &request_id) else {
                continue;
            };

            let missing: Vec<_> = self
                .provider_needs
                .iter()
                .filter(|need| !provider_covers(&provider_assignment, need))
                .cloned()
                .collect();

            if !missing.is_empty() {
                let gap = ProviderGapNotice {
                    request_id: request_id.clone(),
                    suggestor: self.name().to_string(),
                    role: self.role,
                    missing_capabilities: missing,
                    message:
                        "formation member selected, but provider routing does not yet satisfy its needs"
                            .to_string(),
                };
                proposals.push(
                    ProposedFact::new(
                        ContextKey::Constraints,
                        self.provider_gap_id(&request_id),
                        gap,
                        self.provenance(),
                    )
                    .with_confidence(1.0),
                );
                continue;
            }

            let provider_backends =
                provider_backends_for(&provider_assignment, &self.provider_needs);
            let note = MemberNote {
                request_id: request_id.clone(),
                suggestor: self.name().to_string(),
                role: self.role,
                summary: self.synopsis.to_string(),
                provider_needs: self.provider_needs.clone(),
                provider_backends,
            };
            proposals.push(
                ProposedFact::new(
                    self.output_key(),
                    self.note_id(&request_id),
                    note,
                    self.provenance(),
                )
                .with_confidence(f64::from(self.confidence_max)),
            );
        }

        AgentEffect::with_proposals(proposals)
    }
}

struct IntentCodecSuggestor;

#[async_trait::async_trait]
impl Suggestor for IntentCodecSuggestor {
    fn name(&self) -> &str {
        "intent-codec"
    }

    fn provenance(&self) -> Provenance {
        atelier_showcase_provenance()
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.get(ContextKey::Seeds).iter().any(|fact| {
            fact.id().starts_with(SPEC_PREFIX)
                && !route_exists(ctx, request_id_from_spec(fact.id()))
        })
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let mut proposals = Vec::new();

        for fact in ctx.get(ContextKey::Seeds) {
            if !fact.id().starts_with(SPEC_PREFIX) {
                continue;
            }

            let request_id = request_id_from_spec(fact.id());
            if route_exists(ctx, request_id) {
                continue;
            }

            let Some(spec) = fact.payload::<TextPayload>() else {
                continue;
            };

            let compiled = compile_intent(spec.as_str());
            let formation_request = FormationRequest {
                id: request_id.to_string(),
                required_roles: compiled.required_roles.clone(),
                // Current formation matching only supports a global capability
                // filter. Per-role capability routing belongs on providers.
                required_capabilities: vec![],
            };
            let provider_request = ProviderRequest {
                id: request_id.to_string(),
                required_capabilities: compiled.required_provider_capabilities.clone(),
                backend_requirements: None,
            };
            let route = IntentRoute {
                request_id: request_id.to_string(),
                objective: compiled.objective,
                formation_kind: compiled.formation_kind.to_string(),
                required_roles: compiled.required_roles,
                required_provider_capabilities: compiled.required_provider_capabilities,
                notes: compiled.notes,
            };

            proposals.push(
                ProposedFact::new(
                    ContextKey::Seeds,
                    format!("{FORMATION_REQUEST_PREFIX}{request_id}"),
                    formation_request,
                    self.provenance(),
                )
                .with_confidence(0.85),
            );
            proposals.push(
                ProposedFact::new(
                    ContextKey::Seeds,
                    format!("{PROVIDER_REQUEST_PREFIX}{request_id}"),
                    ProviderRequestPayload::new(provider_request),
                    self.provenance(),
                )
                .with_confidence(0.85),
            );
            proposals.push(
                ProposedFact::new(
                    ContextKey::Strategies,
                    format!("{ROUTE_PREFIX}{request_id}"),
                    route,
                    self.provenance(),
                )
                .with_confidence(0.8),
            );
        }

        AgentEffect::with_proposals(proposals)
    }
}

struct LoopStatusSuggestor;

#[async_trait::async_trait]
impl Suggestor for LoopStatusSuggestor {
    fn name(&self) -> &str {
        "loop-status"
    }

    fn provenance(&self) -> Provenance {
        atelier_showcase_provenance()
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[
            ContextKey::Hypotheses,
            ContextKey::Strategies,
            ContextKey::Constraints,
            ContextKey::Evaluations,
            ContextKey::Proposals,
        ]
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        ctx.get(ContextKey::Strategies)
            .iter()
            .filter(|fact| fact.id().starts_with(FORMATION_PLAN_PREFIX))
            .filter_map(|fact| fact.payload::<FormationPlan>().cloned())
            .any(|plan| {
                !fact_exists(
                    ctx,
                    ContextKey::Diagnostic,
                    &format!("{LOOP_STATUS_PREFIX}{}", plan.request_id),
                )
            })
    }

    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let mut proposals = Vec::new();

        for plan in ctx
            .get(ContextKey::Strategies)
            .iter()
            .filter(|fact| fact.id().starts_with(FORMATION_PLAN_PREFIX))
            .filter_map(|fact| fact.payload::<FormationPlan>().cloned())
        {
            let status_id = format!("{LOOP_STATUS_PREFIX}{}", plan.request_id);
            if fact_exists(ctx, ContextKey::Diagnostic, &status_id) {
                continue;
            }

            let Some(provider_assignment) = provider_assignment_for(ctx, &plan.request_id) else {
                continue;
            };

            let note_ids: Vec<_> = plan
                .assignments
                .iter()
                .map(|assignment| {
                    format!("{NOTE_PREFIX}{}:{}", plan.request_id, assignment.suggestor)
                })
                .collect();
            let missing_provider_ids: Vec<_> = plan
                .assignments
                .iter()
                .map(|assignment| {
                    format!(
                        "{PROVIDER_GAP_PREFIX}{}:{}",
                        plan.request_id, assignment.suggestor
                    )
                })
                .collect();

            let all_notes_present = note_ids.iter().all(|id| fact_exists_anywhere(ctx, id));
            let provider_gaps: Vec<_> = missing_provider_ids
                .into_iter()
                .filter(|id| fact_exists(ctx, ContextKey::Constraints, id))
                .collect();

            if !all_notes_present && provider_gaps.is_empty() {
                continue;
            }

            let content = LoopStatus {
                request_id: plan.request_id.clone(),
                ready: plan.unmatched_roles.is_empty()
                    && provider_assignment.unmatched.is_empty()
                    && provider_gaps.is_empty()
                    && all_notes_present,
                formation_coverage_ratio: plan.coverage_ratio,
                provider_coverage_ratio: provider_assignment.coverage_ratio,
                assigned_suggestors: plan
                    .assignments
                    .iter()
                    .map(|assignment| assignment.suggestor.clone())
                    .collect(),
                provider_backends: provider_assignment
                    .assignments
                    .iter()
                    .map(|assignment| assignment.backend_name.clone())
                    .collect(),
                unmatched_roles: plan.unmatched_roles.clone(),
                unmatched_capabilities: provider_assignment.unmatched.clone(),
                provider_gaps,
            };

            let confidence = plan.coverage_ratio.min(provider_assignment.coverage_ratio);
            proposals.push(
                ProposedFact::new(
                    ContextKey::Diagnostic,
                    status_id,
                    content,
                    self.provenance(),
                )
                .with_confidence(confidence),
            );
        }

        AgentEffect::with_proposals(proposals)
    }
}

#[derive(Debug, Clone)]
struct MockBackend {
    name: &'static str,
    kind: BackendKind,
    capabilities: Vec<Capability>,
}

impl Backend for MockBackend {
    fn name(&self) -> &str {
        self.name
    }

    fn kind(&self) -> BackendKind {
        self.kind.clone()
    }

    fn capabilities(&self) -> Vec<Capability> {
        self.capabilities.clone()
    }

    fn supports_replay(&self) -> bool {
        true
    }

    fn requires_network(&self) -> bool {
        false
    }
}

#[tokio::main]
async fn main() {
    let mode = mode_from_args();
    println!("=== Intent Codec: {} ===\n", mode.label);
    if let Some(applet) = mode.applet {
        print_applet_brief(applet);
    }

    let result = run_loop(
        mode.request_id,
        mode.spec,
        profiled_members(),
        provider_pool(),
    )
    .await
    .expect("should converge");
    assert!(result.converged);
    let expected_status_id = format!("{LOOP_STATUS_PREFIX}{}", mode.request_id);
    assert!(
        result
            .context
            .get(ContextKey::Diagnostic)
            .iter()
            .any(|fact| fact.id().as_str() == expected_status_id)
    );

    println!(
        "Converged: {} (cycles: {})",
        result.converged, result.cycles
    );
    println!("Stop:      {:?}\n", result.stop_reason);

    print_section("Seeds", result.context.get(ContextKey::Seeds));
    print_section("Strategies", result.context.get(ContextKey::Strategies));
    print_section("Hypotheses", result.context.get(ContextKey::Hypotheses));
    print_section("Constraints", result.context.get(ContextKey::Constraints));
    print_section("Evaluations", result.context.get(ContextKey::Evaluations));
    print_section("Proposals", result.context.get(ContextKey::Proposals));
    print_section("Diagnostic", result.context.get(ContextKey::Diagnostic));
}

fn mode_from_args() -> DemoMode {
    match std::env::args().nth(1).as_deref() {
        Some("activate-subscription" | "subscription" | "applet") => activate_subscription_mode(),
        Some("refill-prepaid-ai-credits" | "prepaid-credits" | "credits") => {
            refill_prepaid_ai_credits_mode()
        }
        Some("-h" | "--help" | "help") => {
            print_usage();
            std::process::exit(0);
        }
        Some(other) => {
            eprintln!("Unknown mode '{other}'. Falling back to due-diligence.\n");
            default_mode()
        }
        None => default_mode(),
    }
}

fn print_usage() {
    println!("Usage:");
    println!("  cargo run -p example-intent-codec-loop");
    println!("  cargo run -p example-intent-codec-loop -- activate-subscription");
    println!("  cargo run -p example-intent-codec-loop -- refill-prepaid-ai-credits");
}

fn default_mode() -> DemoMode {
    DemoMode {
        label: "Due Diligence Loop",
        request_id: DEFAULT_REQUEST_ID,
        spec: default_spec(),
        applet: None,
    }
}

fn activate_subscription_mode() -> DemoMode {
    DemoMode {
        label: "Activate Subscription Applet",
        request_id: ACTIVATE_SUBSCRIPTION_REQUEST_ID,
        spec: activate_subscription_spec(),
        applet: Some(AppletBrief {
            truth_key: "activate-subscription",
            functional_need: "Turn an agreed commercial plan into active subscription and entitlement state.",
            emotional_need: "Avoid paid-but-locked customer embarrassment and premature access risk.",
            relational_need: "Customer admin, support, finance, RevOps, and partner owner need the same explainable activation state.",
            authority_points: &[
                "payment confirmation required",
                "manual review for non-standard terms",
                "Helm approval owns blocked activation handoff",
            ],
            evidence_sources: &[
                "Commerce Rails subscription contract",
                "runtime or commerce payment confirmation envelope",
                "catalog plan definition",
            ],
            runtime_needs: &[
                "normalized commerce event ingress",
                "secret handling outside the applet",
                "activation telemetry",
            ],
            commercial_needs: &[
                "subscription lifecycle state",
                "catalog plan resolution",
                "entitlement grant",
                "opening ledger context",
            ],
            projection: "operator sees subscription, plan, activation state, entitlements, approval or stop reason",
        }),
    }
}

fn refill_prepaid_ai_credits_mode() -> DemoMode {
    DemoMode {
        label: "Refill Prepaid AI Credits Applet",
        request_id: REFILL_PREPAID_AI_CREDITS_REQUEST_ID,
        spec: refill_prepaid_ai_credits_spec(),
        applet: Some(AppletBrief {
            truth_key: "refill-prepaid-ai-credits",
            functional_need: "Apply a settled top-up to prepaid AI credit balances with financial traceability.",
            emotional_need: "Avoid service interruption from stale balance while preventing credit grants for risky or unsettled payment.",
            relational_need: "Customer admin, finance, support, runtime metering, and partner owner need an explainable balance change.",
            authority_points: &[
                "payment settlement required",
                "risk review for unusual top-up size or signal",
                "Helm approval owns blocked refill handoff",
            ],
            evidence_sources: &[
                "Commerce Rails verified top-up event",
                "active subscription commercial commitment",
                "ledger credit grant receipt",
            ],
            runtime_needs: &[
                "normalized top-up event ingress",
                "provider secret handling outside the applet",
                "balance-change telemetry",
            ],
            commercial_needs: &[
                "payment settlement state",
                "subscription commitment",
                "credit entitlement balance",
                "ledger credit grant",
            ],
            projection: "operator sees payment status, grant amount, subscription, credit entitlement, ledger entry, approval or stop reason",
        }),
    }
}

fn default_spec() -> &'static str {
    r#"Feature: convergent due diligence
  Scenario: Monterro-style diligence for Acme Metrics
    Given due diligence on "Acme Metrics"
    And search breadth before depth
    And graph relationships matter for ownership and integrations
    And analytics should rank evidence quality
    And policy gates should block non-compliant synthesis
    Then assemble a mixed loop instead of a fixed pipeline"#
}

fn activate_subscription_spec() -> &'static str {
    r#"Feature: activate subscription applet
  Scenario: activate paid subscription
    Given functional need is "activate subscription and entitlement state from an agreed commercial plan"
    And emotional need is "avoid customer embarrassment from paid-but-locked access"
    And relational need is "support, finance, customer admin, and partner owner trust the same receipt"
    And payment confirmation is required before activation
    And non-standard plan terms require Helm approval
    And runtime owns event ingress, secrets, and telemetry
    And Commerce Rails owns subscription, entitlement, ledger, and reconciliation
    Then assemble a governed applet loop instead of a billing dashboard"#
}

fn refill_prepaid_ai_credits_spec() -> &'static str {
    r#"Feature: refill prepaid AI credits applet
  Scenario: refill prepaid AI credits
    Given functional need is "apply settled top-up to prepaid AI credit balances with financial traceability"
    And emotional need is "avoid service interruption while preventing risky or unsettled credit grants"
    And relational need is "customer admin, finance, support, runtime metering, and partner owner trust the same balance receipt"
    And payment settlement is required before credit grant
    And unusual top-up size or risk signal requires Helm approval
    And runtime owns event ingress, secrets, and balance-change telemetry
    And Commerce Rails owns settlement state, subscription commitment, ledger grant, and reconciliation
    Then assemble a governed applet loop instead of a metering or billing dashboard"#
}

fn print_applet_brief(applet: AppletBrief) {
    println!("Applet truth: {}", applet.truth_key);
    println!("Functional:   {}", applet.functional_need);
    println!("Emotional:    {}", applet.emotional_need);
    println!("Relational:   {}", applet.relational_need);
    print_list("Authority", applet.authority_points);
    print_list("Evidence", applet.evidence_sources);
    print_list("Runtime", applet.runtime_needs);
    print_list("Commerce", applet.commercial_needs);
    println!("Projection:   {}\n", applet.projection);
}

fn print_list(label: &str, values: &[&str]) {
    println!("{label}:");
    for value in values {
        println!("  - {value}");
    }
}

fn print_section(title: &str, facts: &[converge_kernel::ContextFact]) {
    println!("{title}:");
    if facts.is_empty() {
        println!("  (none)\n");
        return;
    }

    for fact in facts {
        let preview = fact_preview(fact);
        println!("  {} ({preview})", fact.id());
    }
    println!();
}

fn fact_preview(fact: &converge_kernel::ContextFact) -> String {
    if let Some(text) = fact.payload::<TextPayload>() {
        return text.as_str().to_owned();
    }
    if let Some(payload) = fact.payload::<FormationRequest>() {
        return format!("{payload:?}");
    }
    if let Some(payload) = fact.payload::<ProviderRequestPayload>() {
        return format!("{payload:?}");
    }
    if let Some(payload) = fact.payload::<FormationPlan>() {
        return format!("{payload:?}");
    }
    if let Some(payload) = fact.payload::<ProviderAssignmentPayload>() {
        return format!("{payload:?}");
    }
    if let Some(payload) = fact.payload::<IntentRoute>() {
        return format!("{payload:?}");
    }
    if let Some(payload) = fact.payload::<ProviderGapNotice>() {
        return format!("{payload:?}");
    }
    if let Some(payload) = fact.payload::<MemberNote>() {
        return format!("{payload:?}");
    }
    if let Some(payload) = fact.payload::<LoopStatus>() {
        return format!("{payload:?}");
    }
    format!(
        "<typed payload {} v{}>",
        fact.payload_family(),
        fact.payload_version()
    )
}

fn compile_intent(spec: &str) -> CompiledIntent {
    let spec_lower = spec.to_ascii_lowercase();
    if spec_lower.contains("activate subscription")
        || spec_lower.contains("subscription and entitlement")
    {
        return compile_subscription_applet_intent();
    }
    if spec_lower.contains("refill prepaid ai credits")
        || spec_lower.contains("prepaid ai credit balances")
    {
        return compile_prepaid_credit_applet_intent();
    }

    let objective = extract_company(spec)
        .map(|company| format!("due diligence on {company}"))
        .unwrap_or_else(|| "due diligence".to_string());

    let mut required_roles = vec![
        SuggestorRole::Analysis,
        SuggestorRole::Planning,
        SuggestorRole::Constraint,
        SuggestorRole::Evaluation,
        SuggestorRole::Synthesis,
    ];
    let mut required_provider_capabilities = vec![
        Capability::WebSearch,
        Capability::FullTextSearch,
        Capability::GraphTraversal,
        Capability::AccessControl,
        Capability::RuleEvaluation,
        Capability::Reasoning,
        Capability::StructuredOutput,
        Capability::TextGeneration,
    ];
    let mut notes = vec![
        "due diligence uses a mixed convergence loop rather than a fixed pipeline".to_string(),
        "formation matching covers roles; provider matching covers backend capabilities"
            .to_string(),
    ];

    if spec_lower.contains("analytics") || spec_lower.contains("evidence quality") {
        required_provider_capabilities.push(Capability::Classification);
        notes.push("analytics evidence ranking requested".to_string());
    }

    if spec_lower.contains("graph") {
        notes.push("graph traversal requested for relationship mapping".to_string());
    }

    if spec_lower.contains("policy") || spec_lower.contains("compliant") {
        notes.push("policy gate required before synthesis".to_string());
    }

    if spec_lower.contains("scheduler") {
        required_roles.push(SuggestorRole::Meta);
        required_provider_capabilities.push(Capability::Scheduling);
        notes.push("scheduler support requested".to_string());
    }

    dedup(&mut required_roles);
    dedup(&mut required_provider_capabilities);

    let formation_kind = if spec_lower.contains("mixed loop")
        || spec_lower.contains("instead of a fixed pipeline")
    {
        "open_claw"
    } else {
        "deliberated"
    };

    CompiledIntent {
        objective,
        formation_kind,
        required_roles,
        required_provider_capabilities,
        notes,
    }
}

fn compile_subscription_applet_intent() -> CompiledIntent {
    compile_revenue_applet_intent(
        "activate paid subscription",
        "commerce concerns stay in Commerce Rails: subscription, entitlement, ledger",
        "Helm owns approval and trust-transfer for blocked activation",
    )
}

fn compile_prepaid_credit_applet_intent() -> CompiledIntent {
    compile_revenue_applet_intent(
        "refill prepaid AI credits",
        "commerce concerns stay in Commerce Rails: settlement, subscription, credit balance, ledger",
        "Helm owns approval and trust-transfer for blocked refill",
    )
}

fn compile_revenue_applet_intent(
    objective: &str,
    commerce_note: &str,
    helm_note: &str,
) -> CompiledIntent {
    CompiledIntent {
        objective: objective.to_string(),
        formation_kind: "governed_applet",
        required_roles: vec![
            SuggestorRole::Planning,
            SuggestorRole::Constraint,
            SuggestorRole::Evaluation,
            SuggestorRole::Synthesis,
        ],
        required_provider_capabilities: vec![
            Capability::GraphTraversal,
            Capability::AccessControl,
            Capability::RuleEvaluation,
            Capability::Classification,
            Capability::Reasoning,
            Capability::StructuredOutput,
            Capability::TextGeneration,
        ],
        notes: vec![
            "Intent Codec captured functional, emotional, and relational JTBD".to_string(),
            "runtime concerns stay outside the applet: ingress, secrets, telemetry".to_string(),
            commerce_note.to_string(),
            helm_note.to_string(),
        ],
    }
}

fn extract_company(spec: &str) -> Option<String> {
    let start = spec.find('"')?;
    let rest = &spec[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn register_profiled(
    engine: &mut Engine,
    catalog: &mut Vec<ProfileSnapshot>,
    suggestor: ProfiledMember,
) {
    let snapshot = ProfileSnapshot::from_profile(suggestor.name().to_string(), &suggestor);
    catalog.push(snapshot);
    engine.register_suggestor(suggestor);
}

fn build_engine(members: Vec<ProfiledMember>, backends: Vec<Arc<dyn Backend>>) -> Engine {
    let mut engine = Engine::with_budget(Budget {
        max_cycles: 12,
        max_facts: 256,
    });

    engine.register_suggestor(IntentCodecSuggestor);

    let mut catalog = Vec::new();
    for member in members {
        register_profiled(&mut engine, &mut catalog, member);
    }

    engine.register_suggestor(FormationAssemblySuggestor::new(catalog));
    engine.register_suggestor(ProviderSelectionSuggestor::new(backends));
    engine.register_suggestor(LoopStatusSuggestor);
    engine
}

fn seed_spec(request_id: &str, spec: &str) -> ContextState {
    let mut context = ContextState::new();
    context
        .add_input_with_provenance(
            ContextKey::Seeds,
            format!("{SPEC_PREFIX}{request_id}"),
            spec,
            "gherkin:human-intent",
        )
        .expect("should stage input");
    context
}

async fn run_loop(
    request_id: &str,
    spec: &str,
    members: Vec<ProfiledMember>,
    backends: Vec<Arc<dyn Backend>>,
) -> Result<ConvergeResult, converge_kernel::ConvergeError> {
    let mut engine = build_engine(members, backends);
    engine.run(seed_spec(request_id, spec)).await
}

fn profiled_members() -> Vec<ProfiledMember> {
    vec![
        ProfiledMember {
            name: "knowledge-scout",
            role: SuggestorRole::Analysis,
            output_keys: vec![ContextKey::Hypotheses],
            capabilities: vec![
                SuggestorCapability::KnowledgeRetrieval,
                SuggestorCapability::Analytics,
            ],
            provider_needs: vec![Capability::WebSearch, Capability::FullTextSearch],
            synopsis: "maps breadth, sources, and loose ends for diligence",
            cost_hint: CostClass::Low,
            latency_hint: LatencyClass::Interactive,
            confidence_min: 0.55,
            confidence_max: 0.82,
        },
        ProfiledMember {
            name: "graph-planner",
            role: SuggestorRole::Planning,
            output_keys: vec![ContextKey::Strategies],
            capabilities: vec![SuggestorCapability::Optimization],
            provider_needs: vec![Capability::GraphTraversal],
            synopsis: "turns entity relationships into a diligence work graph",
            cost_hint: CostClass::Medium,
            latency_hint: LatencyClass::Interactive,
            confidence_min: 0.6,
            confidence_max: 0.86,
        },
        ProfiledMember {
            name: "policy-sentinel",
            role: SuggestorRole::Constraint,
            output_keys: vec![ContextKey::Constraints],
            capabilities: vec![SuggestorCapability::PolicyEnforcement],
            provider_needs: vec![Capability::AccessControl, Capability::RuleEvaluation],
            synopsis: "guards compliance and escalation boundaries before synthesis",
            cost_hint: CostClass::Low,
            latency_hint: LatencyClass::Realtime,
            confidence_min: 0.75,
            confidence_max: 0.95,
        },
        ProfiledMember {
            name: "evidence-ranker",
            role: SuggestorRole::Evaluation,
            output_keys: vec![ContextKey::Evaluations],
            capabilities: vec![SuggestorCapability::Analytics],
            provider_needs: vec![Capability::Classification, Capability::Reasoning],
            synopsis: "scores evidence quality so the loop knows what to trust next",
            cost_hint: CostClass::Medium,
            latency_hint: LatencyClass::Interactive,
            confidence_min: 0.6,
            confidence_max: 0.88,
        },
        ProfiledMember {
            name: "llm-synthesizer",
            role: SuggestorRole::Synthesis,
            output_keys: vec![ContextKey::Proposals],
            capabilities: vec![SuggestorCapability::LlmReasoning],
            provider_needs: vec![Capability::StructuredOutput, Capability::TextGeneration],
            synopsis: "turns stabilized findings into a diligence memo skeleton",
            cost_hint: CostClass::High,
            latency_hint: LatencyClass::Background,
            confidence_min: 0.58,
            confidence_max: 0.84,
        },
        ProfiledMember {
            name: "loop-governor",
            role: SuggestorRole::Meta,
            output_keys: vec![ContextKey::Diagnostic],
            capabilities: vec![SuggestorCapability::ExperienceLearning],
            provider_needs: vec![Capability::Scheduling],
            synopsis: "tracks loop cadence when scheduling is explicitly requested",
            cost_hint: CostClass::Low,
            latency_hint: LatencyClass::Interactive,
            confidence_min: 0.5,
            confidence_max: 0.8,
        },
    ]
}

fn provider_pool() -> Vec<Arc<dyn Backend>> {
    vec![
        backend(
            "breadth-search",
            BackendKind::Search,
            &[Capability::WebSearch],
        ),
        backend(
            "archive-search",
            BackendKind::Search,
            &[Capability::FullTextSearch],
        ),
        backend(
            "graph-navigator",
            BackendKind::Search,
            &[Capability::GraphTraversal],
        ),
        backend(
            "policy-gate",
            BackendKind::Policy,
            &[Capability::AccessControl],
        ),
        backend(
            "rule-engine",
            BackendKind::Policy,
            &[Capability::RuleEvaluation],
        ),
        backend("reasoning-core", BackendKind::Llm, &[Capability::Reasoning]),
        backend(
            "schema-writer",
            BackendKind::Llm,
            &[Capability::StructuredOutput],
        ),
        backend(
            "memo-writer",
            BackendKind::Llm,
            &[Capability::TextGeneration],
        ),
        backend(
            "analytics-ranker",
            BackendKind::Analytics,
            &[Capability::Classification],
        ),
        backend(
            "loop-scheduler",
            BackendKind::Optimization,
            &[Capability::Scheduling],
        ),
    ]
}

fn backend(name: &'static str, kind: BackendKind, capabilities: &[Capability]) -> Arc<dyn Backend> {
    Arc::new(MockBackend {
        name,
        kind,
        capabilities: capabilities.to_vec(),
    })
}

fn assigned_requests(ctx: &dyn Context, suggestor_name: &str) -> Vec<String> {
    ctx.get(ContextKey::Strategies)
        .iter()
        .filter(|fact| fact.id().starts_with(FORMATION_PLAN_PREFIX))
        .filter_map(|fact| fact.payload::<FormationPlan>().cloned())
        .filter(|plan| {
            plan.assignments
                .iter()
                .any(|assignment| assignment.suggestor == suggestor_name)
        })
        .map(|plan| plan.request_id)
        .collect()
}

fn provider_assignment_for(ctx: &dyn Context, request_id: &str) -> Option<ProviderAssignment> {
    let assignment_id = format!("{PROVIDER_ASSIGNMENT_PREFIX}{request_id}");
    ctx.get(ContextKey::Strategies)
        .iter()
        .find(|fact| fact.id().as_str() == assignment_id)
        .and_then(|fact| {
            fact.payload::<ProviderAssignmentPayload>()
                .map(|payload| payload.assignment().clone())
        })
}

fn route_exists(ctx: &dyn Context, request_id: &str) -> bool {
    fact_exists(
        ctx,
        ContextKey::Strategies,
        &format!("{ROUTE_PREFIX}{request_id}"),
    ) && fact_exists(
        ctx,
        ContextKey::Seeds,
        &format!("{FORMATION_REQUEST_PREFIX}{request_id}"),
    ) && fact_exists(
        ctx,
        ContextKey::Seeds,
        &format!("{PROVIDER_REQUEST_PREFIX}{request_id}"),
    )
}

fn request_id_from_spec(spec_id: &str) -> &str {
    spec_id.trim_start_matches(SPEC_PREFIX)
}

fn fact_exists(ctx: &dyn Context, key: ContextKey, id: &str) -> bool {
    ctx.get(key).iter().any(|fact| fact.id().as_str() == id)
}

fn fact_exists_anywhere(ctx: &dyn Context, id: &str) -> bool {
    CONTEXT_KEYS
        .into_iter()
        .any(|key| ctx.get(key).iter().any(|fact| fact.id().as_str() == id))
}

fn provider_covers(assignment: &ProviderAssignment, capability: &Capability) -> bool {
    assignment
        .assignments
        .iter()
        .any(|item| &item.capability == capability)
}

fn provider_backends_for(
    assignment: &ProviderAssignment,
    provider_needs: &[Capability],
) -> Vec<String> {
    provider_needs
        .iter()
        .filter_map(|need| {
            assignment
                .assignments
                .iter()
                .find(|item| item.capability == *need)
                .map(|item| item.backend_name.clone())
        })
        .collect()
}

fn dedup<T>(items: &mut Vec<T>)
where
    T: PartialEq + Clone,
{
    let mut unique = Vec::new();
    for item in items.iter().cloned() {
        if !unique.contains(&item) {
            unique.push(item);
        }
    }
    *items = unique;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn due_diligence_codec_emits_mixed_loop_requirements() {
        let compiled = compile_intent(
            r#"Given due diligence on "Acme"
And graph relationships matter for ownership and integrations
And analytics should rank evidence quality
And policy gates should block non-compliant synthesis"#,
        );

        assert!(compiled.required_roles.contains(&SuggestorRole::Analysis));
        assert!(compiled.required_roles.contains(&SuggestorRole::Planning));
        assert!(compiled.required_roles.contains(&SuggestorRole::Constraint));
        assert!(compiled.required_roles.contains(&SuggestorRole::Evaluation));
        assert!(compiled.required_roles.contains(&SuggestorRole::Synthesis));
        assert!(
            compiled
                .required_provider_capabilities
                .contains(&Capability::Classification)
        );
        assert!(
            compiled
                .required_provider_capabilities
                .contains(&Capability::AccessControl)
        );
        assert!(
            compiled
                .required_provider_capabilities
                .contains(&Capability::GraphTraversal)
        );
        assert_eq!(compiled.formation_kind, "deliberated");
    }

    #[test]
    fn mixed_loop_clause_selects_open_claw_formation() {
        let compiled = compile_intent(default_spec());
        assert_eq!(compiled.formation_kind, "open_claw");
    }

    #[test]
    fn scheduler_clause_adds_meta_role() {
        let compiled = compile_intent(
            r#"Given due diligence on "Acme"\nAnd a scheduler should pace the loop"#,
        );
        assert!(compiled.required_roles.contains(&SuggestorRole::Meta));
        assert!(
            compiled
                .required_provider_capabilities
                .contains(&Capability::Scheduling)
        );
    }

    #[test]
    fn prepaid_credit_applet_selects_governed_applet_loop() {
        let compiled = compile_intent(refill_prepaid_ai_credits_spec());

        assert_eq!(compiled.objective, "refill prepaid AI credits");
        assert_eq!(compiled.formation_kind, "governed_applet");
        assert!(compiled.required_roles.contains(&SuggestorRole::Planning));
        assert!(compiled.required_roles.contains(&SuggestorRole::Constraint));
        assert!(compiled.required_roles.contains(&SuggestorRole::Evaluation));
        assert!(compiled.required_roles.contains(&SuggestorRole::Synthesis));
        assert!(
            compiled
                .required_provider_capabilities
                .contains(&Capability::RuleEvaluation)
        );
        assert!(
            compiled
                .notes
                .iter()
                .any(|note| note.contains("settlement, subscription, credit balance, ledger"))
        );
    }

    #[test]
    fn profile_snapshot_registration_captures_member_name() {
        let member = profiled_members()
            .into_iter()
            .find(|member| member.name == "knowledge-scout")
            .expect("member exists");
        let snapshot = ProfileSnapshot::from_profile(member.name().to_string(), &member);

        assert_eq!(snapshot.name, "knowledge-scout");
        assert_eq!(snapshot.role, SuggestorRole::Analysis);
        assert_eq!(snapshot.output_keys, vec![ContextKey::Hypotheses]);
    }

    #[test]
    fn register_profiled_populates_catalog_and_engine() {
        let member = profiled_members()
            .into_iter()
            .find(|member| member.name == "policy-sentinel")
            .expect("member exists");
        let mut engine = Engine::new();
        let mut catalog = Vec::new();

        register_profiled(&mut engine, &mut catalog, member);

        assert_eq!(engine.suggestor_count(), 1);
        assert_eq!(catalog.len(), 1);
        assert_eq!(catalog[0].name, "policy-sentinel");
        assert_eq!(catalog[0].role, SuggestorRole::Constraint);
    }

    #[tokio::test]
    async fn rerun_is_idempotent_for_routing_outputs() {
        let first = run_loop(
            DEFAULT_REQUEST_ID,
            default_spec(),
            profiled_members(),
            provider_pool(),
        )
        .await
        .expect("first run should converge");
        let expected_strategy_ids: HashSet<_> = first
            .context
            .get(ContextKey::Strategies)
            .iter()
            .map(|fact| fact.id().clone())
            .collect();
        let expected_diagnostic_ids: HashSet<_> = first
            .context
            .get(ContextKey::Diagnostic)
            .iter()
            .map(|fact| fact.id().clone())
            .collect();

        let mut rerun_engine = build_engine(profiled_members(), provider_pool());
        let second = rerun_engine
            .run(first.context.clone())
            .await
            .expect("rerun should converge");

        let rerun_strategy_ids: HashSet<_> = second
            .context
            .get(ContextKey::Strategies)
            .iter()
            .map(|fact| fact.id().clone())
            .collect();
        let rerun_diagnostic_ids: HashSet<_> = second
            .context
            .get(ContextKey::Diagnostic)
            .iter()
            .map(|fact| fact.id().clone())
            .collect();

        assert_eq!(expected_strategy_ids, rerun_strategy_ids);
        assert_eq!(expected_diagnostic_ids, rerun_diagnostic_ids);
        assert_eq!(
            first.context.get(ContextKey::Strategies).len(),
            second.context.get(ContextKey::Strategies).len()
        );
        assert_eq!(
            first.context.get(ContextKey::Diagnostic).len(),
            second.context.get(ContextKey::Diagnostic).len()
        );
    }

    #[tokio::test]
    async fn loop_status_is_not_ready_when_provider_coverage_is_partial() {
        let limited_pool: Vec<_> = provider_pool()
            .into_iter()
            .filter(|backend| backend.name() != "schema-writer")
            .collect();

        let result = run_loop(
            DEFAULT_REQUEST_ID,
            default_spec(),
            profiled_members(),
            limited_pool,
        )
        .await
        .expect("run should converge");

        let status = result
            .context
            .get(ContextKey::Diagnostic)
            .iter()
            .find(|fact| fact.id() == "loop-status:dd-acme-metrics")
            .and_then(|fact| fact.payload::<LoopStatus>())
            .expect("loop status should exist");

        assert!(!status.ready);
        assert!(
            result
                .context
                .get(ContextKey::Constraints)
                .iter()
                .any(|fact| fact.id() == "missing-provider:dd-acme-metrics:llm-synthesizer")
        );
    }

    #[tokio::test]
    async fn loop_status_is_not_ready_when_formation_is_partial() {
        let reduced_members: Vec<_> = profiled_members()
            .into_iter()
            .filter(|member| member.name != "policy-sentinel")
            .collect();

        let result = run_loop(
            DEFAULT_REQUEST_ID,
            default_spec(),
            reduced_members,
            provider_pool(),
        )
        .await
        .expect("run should converge");

        let plan = result
            .context
            .get(ContextKey::Strategies)
            .iter()
            .find(|fact| fact.id() == "formation-plan:dd-acme-metrics")
            .and_then(|fact| fact.payload::<FormationPlan>().cloned())
            .expect("formation plan should exist");
        let status = result
            .context
            .get(ContextKey::Diagnostic)
            .iter()
            .find(|fact| fact.id() == "loop-status:dd-acme-metrics")
            .and_then(|fact| fact.payload::<LoopStatus>())
            .expect("loop status should exist");

        assert_eq!(plan.unmatched_roles, vec![SuggestorRole::Constraint]);
        assert!(!status.ready);
    }
}
