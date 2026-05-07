//! Integration tests for the standard pack catalog + StructuralResolver.
//!
//! Ported from organism-runtime/src/registry.rs when the registration logic
//! moved out of organism-runtime to keep the runtime crate domain-agnostic.

use organism_domain::{register_standard_packs, registry_with_standard_packs};
use organism_pack::{
    DeclarativeBinding, ForbiddenAction, IntentBinding, IntentPacket, IntentResolver, Reversibility,
};
use organism_runtime::{Registry, StructuralResolver};

fn full_registry() -> Registry {
    let mut r = registry_with_standard_packs();
    r.register_capability("web", "URL capture and metadata extraction");
    r.register_capability("ocr", "Document understanding");
    r.register_capability("linkedin", "Professional network research");
    r.register_capability("social", "Social profile extraction");
    r
}

fn intent(outcome: &str) -> IntentPacket {
    IntentPacket::new(outcome, chrono::Utc::now() + chrono::Duration::hours(1))
}

#[test]
fn standard_registry_includes_builtin_domain_packs() {
    let registry = registry_with_standard_packs();

    assert_eq!(registry.packs().len(), 14);
    assert!(registry.packs().iter().any(|pack| pack.name == "knowledge"));
    assert!(
        registry
            .packs()
            .iter()
            .any(|pack| pack.name == "linkedin_research")
    );
}

#[test]
fn standard_registry_registration_is_idempotent() {
    let mut registry = registry_with_standard_packs();

    register_standard_packs(&mut registry);

    assert_eq!(registry.packs().len(), 14);
}

// ── Dimension 1: Fact prefix ───────────────────────────────────

#[test]
fn dim1_fact_prefix_matches_pack() {
    let r = full_registry();
    let resolver = StructuralResolver::new(&r);
    let i = intent("process lead").with_context(serde_json::json!({ "ref": "lead:abc-123" }));
    let binding = resolver.resolve(&i, &IntentBinding::default());
    assert!(
        binding.packs.iter().any(|p| p.pack_name == "customers"),
        "should match customers from lead: prefix"
    );
}

// ── Dimension 2: Constraint → invariant ────────────────────────

#[test]
fn dim2_constraint_matches_invariant() {
    let r = full_registry();
    let resolver = StructuralResolver::new(&r);
    let mut i = intent("execute contract");
    i.constraints = vec!["signature_required".into()];
    let binding = resolver.resolve(&i, &IntentBinding::default());
    assert!(
        binding.packs.iter().any(|p| p.pack_name == "legal"),
        "should match legal from signature_required constraint"
    );
}

// ── Dimension 3: Context key flow ─────────────────────────────

#[test]
fn dim3_context_keys_without_anchor_do_not_globally_fan_out() {
    let r = full_registry();
    let resolver = StructuralResolver::new(&r);
    let i = intent("aggregate findings into recommendation").with_context(serde_json::json!({
        "evaluations": ["score:a", "score:b"],
        "strategies": "final recommendation needed"
    }));
    let binding = resolver.resolve(&i, &IntentBinding::default());

    assert!(
        !binding
            .packs
            .iter()
            .any(|pack| pack.reason.contains("context flow")),
        "context keys alone should not add context-flow matches"
    );
}

#[test]
fn dim3_context_keys_extend_only_anchored_flow() {
    let r = full_registry();
    let resolver = StructuralResolver::new(&r);
    let i = intent("aggregate vendor scores into strategy").with_context(serde_json::json!({
        "evaluations": ["price:vendor-a", "compliance:vendor-a"],
        "strategies": "final recommendation needed"
    }));
    let binding = resolver.resolve(&i, &IntentBinding::default());
    let pack_names = binding
        .packs
        .iter()
        .map(|pack| pack.pack_name.as_str())
        .collect::<std::collections::HashSet<_>>();

    assert!(
        pack_names.contains("procurement"),
        "vendor entity should anchor procurement"
    );
    assert!(
        pack_names.contains("partnerships"),
        "vendor entity should anchor partnerships"
    );
    assert!(
        pack_names.contains("linkedin_research"),
        "anchored Evaluations → Strategies flow should add linkedin_research"
    );
    assert!(
        pack_names.contains("knowledge"),
        "anchored Evaluations → Strategies flow should add knowledge"
    );
    assert!(
        pack_names.contains("reskilling"),
        "anchored Evaluations → Strategies flow should add reskilling"
    );
    assert!(
        !pack_names.contains("ops_support"),
        "unanchored packs writing Evaluations should not be added"
    );
    assert!(
        !pack_names.contains("virtual_teams"),
        "unanchored packs writing Evaluations should not be added"
    );
    let weak_keyword_matches = binding
        .packs
        .iter()
        .filter(|pack| pack.pack_name == "product_engineering" || pack.pack_name == "performance")
        .collect::<Vec<_>>();
    assert!(
        weak_keyword_matches
            .iter()
            .all(|pack| !pack.reason.contains("context flow")),
        "weak keyword matches must not be upgraded into context-flow matches"
    );
}

// ── Dimension 4: Domain entity ─────────────────────────────────

#[test]
fn dim4_entity_matches_pack() {
    let r = full_registry();
    let resolver = StructuralResolver::new(&r);
    let i = intent("evaluate this vendor for compliance");
    let binding = resolver.resolve(&i, &IntentBinding::default());
    assert!(
        binding.packs.iter().any(|p| p.pack_name == "partnerships"),
        "should match partnerships from 'vendor' entity"
    );
}

// ── Dimension 5: Keyword ───────────────────────────────────────

#[test]
fn dim5_keyword_matches_pack() {
    let r = full_registry();
    let resolver = StructuralResolver::new(&r);
    let i = intent("plan the next marketing campaign for Q3");
    let binding = resolver.resolve(&i, &IntentBinding::default());
    assert!(
        binding
            .packs
            .iter()
            .any(|p| p.pack_name == "growth_marketing"),
        "should match growth_marketing from 'campaign' keyword"
    );
}

#[test]
fn dim5_keyword_does_not_match_pack_descriptions() {
    let r = full_registry();
    assert!(
        r.packs_for_keyword("strategy").is_empty(),
        "description substrings should not count as keyword matches"
    );
    assert!(
        r.packs_for_keyword("aggregate").is_empty(),
        "description substrings should not count as keyword matches"
    );
}

// ── Dimension 6: Reversibility ─────────────────────────────────

#[test]
fn dim6_irreversible_adds_governance_packs() {
    let r = full_registry();
    let resolver = StructuralResolver::new(&r);
    let i = intent("terminate employee access").with_reversibility(Reversibility::Irreversible);
    let binding = resolver.resolve(&i, &IntentBinding::default());
    let governance_packs: Vec<_> = binding
        .packs
        .iter()
        .filter(|p| p.reason.contains("irreversible"))
        .collect();
    assert!(
        !governance_packs.is_empty(),
        "irreversible intent should add governance packs"
    );
}

// ── Dimension 7: Forbidden filtering ───────────────────────────

#[test]
fn dim7_forbidden_actions_filter_packs() {
    let r = full_registry();
    let resolver = StructuralResolver::new(&r);
    let mut i = intent("research this lead but no linkedin outreach");
    i.forbidden = vec![ForbiddenAction {
        action: "linkedin".into(),
        reason: "not authorized for external contact".into(),
    }];
    i = i.with_context(serde_json::json!({ "ref": "lead:abc" }));
    let binding = resolver.resolve(&i, &IntentBinding::default());
    assert!(
        !binding
            .packs
            .iter()
            .any(|p| p.pack_name == "linkedin_research"),
        "linkedin_research should be filtered out by forbidden action"
    );
}

// ── Dimension 8: Capability affinity ───────────────────────────

#[test]
fn dim8_pack_adds_required_capabilities() {
    let r = full_registry();
    let resolver = StructuralResolver::new(&r);
    let binding = DeclarativeBinding::new()
        .pack("linkedin_research", "research leads")
        .build();
    let binding = resolver.resolve(&intent("research leads"), &binding);
    let cap_names: Vec<_> = binding
        .capabilities
        .iter()
        .map(|c| c.capability.as_str())
        .collect();
    assert!(
        cap_names.contains(&"linkedin"),
        "should add linkedin capability"
    );
    assert!(cap_names.contains(&"web"), "should add web capability");
    assert!(
        cap_names.contains(&"social"),
        "should add social capability"
    );
}

// ── Deduplication ──────────────────────────────────────────────

#[test]
fn deduplicates_packs_keeping_highest_confidence() {
    let r = full_registry();
    let resolver = StructuralResolver::new(&r);
    let i = intent("qualify this lead for the pipeline")
        .with_context(serde_json::json!({ "ref": "lead:abc" }));
    let binding = resolver.resolve(&i, &IntentBinding::default());
    let customer_matches: Vec<_> = binding
        .packs
        .iter()
        .filter(|p| p.pack_name == "customers")
        .collect();
    assert_eq!(customer_matches.len(), 1, "should deduplicate to one entry");
    assert!(
        customer_matches[0].confidence.as_f64() >= 0.75,
        "should keep highest confidence match"
    );
}
