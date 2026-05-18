//! Reuse arena's shared intent fixtures against the richer showcase menu.
//!
//! Arena validates the front-half contract. This scenario validates the same
//! intents can compile through Organism's catalog-aware path while drawing
//! participants from the Converge + Organism + mosaic descriptor seed.

use arena_intent_cases::{IntentCase, cases};
use chrono::{TimeZone, Utc};
use converge_kernel::formation::{
    FormationCatalog, FormationTemplate, FormationTemplateMetadata, FormationTemplateQuery,
    StaticFormationTemplate, SuggestorCapability, SuggestorRole,
};
use organism_catalog::ProviderDescriptorCatalog;
use organism_catalog_seed as seed;
use organism_pack::{ForbiddenAction, IntentPacket};
use organism_runtime::{
    FormationCompileRequest, FormationCompiler, FormationGuru, standard_formation_catalog,
};
use uuid::Uuid;

fn expiry() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2027, 1, 15, 12, 0, 0)
        .single()
        .expect("fixed expiry should be valid")
}

fn intent(case: &IntentCase) -> IntentPacket {
    let mut intent = IntentPacket::new(case.outcome, expiry());
    intent.constraints = case.constraints.iter().map(|c| (*c).to_string()).collect();
    intent.forbidden = case
        .forbidden_actions
        .iter()
        .map(|action| ForbiddenAction {
            action: (*action).to_string(),
            reason: "arena fixture forbidden action".to_string(),
        })
        .collect();
    intent.context =
        serde_json::from_str(case.context_json).expect("intent fixture context must be JSON");
    intent
}

fn template_keyword(template_id: &str) -> &'static str {
    match template_id {
        "organism-decision" => "decision",
        "organism-research" => "research",
        "organism-evaluation" => "evaluation",
        "organism-planning" => "planning",
        "organism-diligence" => "diligence",
        other => panic!("unknown standard template id: {other}"),
    }
}

fn request(index: usize, case: &IntentCase) -> FormationCompileRequest {
    FormationCompileRequest::new(
        Uuid::from_u128(0xA7E1_0000 + index as u128),
        Uuid::from_u128(0xC0DE_0000 + index as u128),
        FormationTemplateQuery::new().with_keyword(template_keyword(case.expected_template_id)),
    )
    // The showcase pass intentionally asks for a mosaic optimization
    // participant. `cp-sat` gives Ferrox domain affinity without changing
    // the front-half standard-template selection asserted above.
    .with_domain_tag("cp-sat")
}

fn enriched_catalog(case: &IntentCase) -> FormationCatalog {
    let keyword = template_keyword(case.expected_template_id);
    let (description, roles, capabilities) = match case.expected_template_id {
        "organism-decision" => (
            "Decision with mosaic optimization pressure",
            vec![
                SuggestorRole::Signal,
                SuggestorRole::Constraint,
                SuggestorRole::Evaluation,
                SuggestorRole::Synthesis,
            ],
            vec![
                SuggestorCapability::LlmReasoning,
                SuggestorCapability::PolicyEnforcement,
                SuggestorCapability::Analytics,
                SuggestorCapability::Optimization,
            ],
        ),
        "organism-research" => (
            "Research with mosaic optimization pressure",
            vec![
                SuggestorRole::Signal,
                SuggestorRole::Analysis,
                SuggestorRole::Synthesis,
            ],
            vec![
                SuggestorCapability::LlmReasoning,
                SuggestorCapability::KnowledgeRetrieval,
                SuggestorCapability::Optimization,
            ],
        ),
        "organism-evaluation" => (
            "Evaluation with mosaic optimization pressure",
            vec![
                SuggestorRole::Signal,
                SuggestorRole::Evaluation,
                SuggestorRole::Synthesis,
            ],
            vec![
                SuggestorCapability::LlmReasoning,
                SuggestorCapability::Analytics,
                SuggestorCapability::PolicyEnforcement,
                SuggestorCapability::Optimization,
            ],
        ),
        "organism-planning" => (
            "Planning with mosaic optimization pressure",
            vec![
                SuggestorRole::Signal,
                SuggestorRole::Planning,
                SuggestorRole::Constraint,
                SuggestorRole::Synthesis,
            ],
            vec![
                SuggestorCapability::LlmReasoning,
                SuggestorCapability::Analytics,
                SuggestorCapability::PolicyEnforcement,
                SuggestorCapability::Optimization,
            ],
        ),
        "organism-diligence" => (
            "Diligence with mosaic optimization pressure",
            vec![
                SuggestorRole::Signal,
                SuggestorRole::Evaluation,
                SuggestorRole::Constraint,
                SuggestorRole::Synthesis,
            ],
            vec![
                SuggestorCapability::LlmReasoning,
                SuggestorCapability::KnowledgeRetrieval,
                SuggestorCapability::PolicyEnforcement,
                SuggestorCapability::HumanInTheLoop,
                SuggestorCapability::Optimization,
            ],
        ),
        other => panic!("unknown standard template id: {other}"),
    };

    let mut metadata = FormationTemplateMetadata::new(
        case.expected_template_id.to_string(),
        description.to_string(),
        roles,
    )
    .with_keyword(keyword);
    for capability in capabilities {
        metadata = metadata.with_required_capability(capability);
    }
    FormationCatalog::new().with_template(FormationTemplate::static_template(
        StaticFormationTemplate::new(metadata),
    ))
}

fn is_mosaic_descriptor(id: &str) -> bool {
    id.starts_with("arbiter-")
        || id.starts_with("embassy-")
        || id.starts_with("ferrox-")
        || id.starts_with("mnemos-")
        || id.starts_with("prism-")
        || id.starts_with("soter-")
}

#[test]
fn arena_intents_select_expected_templates_in_showcase() {
    let templates = standard_formation_catalog();
    let guru = FormationGuru::new(&templates);
    let caps = [
        SuggestorCapability::LlmReasoning,
        SuggestorCapability::KnowledgeRetrieval,
        SuggestorCapability::Analytics,
        SuggestorCapability::Optimization,
        SuggestorCapability::PolicyEnforcement,
        SuggestorCapability::HumanInTheLoop,
        SuggestorCapability::ExperienceLearning,
    ];

    for case in cases() {
        let selection = guru
            .select(&intent(case), &caps)
            .unwrap_or_else(|err| panic!("{} should select a template: {err}", case.id));
        assert_eq!(
            selection.classification.class.as_str(),
            case.expected_problem_class,
            "{} should classify as {}",
            case.id,
            case.expected_problem_class
        );
        assert_eq!(
            selection.primary.id(),
            case.expected_template_id,
            "{} should select {}",
            case.id,
            case.expected_template_id
        );
    }
}

#[test]
fn arena_intents_compile_with_mosaic_participants_available() {
    let catalog = seed::all();
    let providers = ProviderDescriptorCatalog::new();

    for (index, case) in cases().iter().enumerate() {
        let templates = enriched_catalog(case);
        let outcome = FormationCompiler::new()
            .compile_from_catalog(
                &request(index, case),
                &templates,
                &catalog,
                &providers,
                None,
            )
            .unwrap_or_else(|err| {
                panic!(
                    "{} should compile from catalog seed for {}: {err:?}",
                    case.id, case.expected_template_id
                )
            });

        assert_eq!(
            outcome.template_id, case.expected_template_id,
            "{} should compile the expected template",
            case.id
        );

        let roster_ids: Vec<&str> = outcome
            .roster
            .iter()
            .map(|role| role.suggestor_id.as_str())
            .collect();
        assert!(
            roster_ids.iter().any(|id| is_mosaic_descriptor(id)),
            "{} should draw at least one participant from mosaic; roster: {roster_ids:?}",
            case.id
        );
    }
}
