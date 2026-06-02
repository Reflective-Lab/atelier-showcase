//! Organizational domain packs for Organism.
//!
//! These packs encode reusable organizational workflow patterns.
//! Each pack defines agents (suggestors), invariants, and fact prefixes
//! for a specific organizational domain.
//!
//! When wired to a Converge engine, each agent implements the `Suggestor`
//! trait from `converge-pack`. The patterns here define the organizational
//! logic; Converge enforces the axioms.
//!
//! # Packs
//!
//! ## Knowledge lifecycle (from converge-domain)
//! - [`packs::knowledge`] — Signal → Hypothesis → Experiment → Decision → Canonical
//!
//! ## Organizational workflows
//! - [`packs::customers`] — Revenue operations: Lead → Close → Handoff
//! - [`packs::people`] — People lifecycle: Hire → Onboard → Pay → Offboard
//! - [`packs::legal`] — Contracts, equity, IP governance
//! - [`packs::performance`] — Reviews, goals, improvement plans
//! - [`packs::autonomous_org`] — Governance, policies, budgets, delegations
//! - [`packs::growth_marketing`] — Campaigns, channels, attribution
//! - [`packs::product_engineering`] — Roadmaps, features, releases, incidents
//! - [`packs::ops_support`] — Ticket intake, triage, escalation, SLA
//! - [`packs::procurement`] — Purchase requests, assets, subscriptions
//! - [`packs::partnerships`] — Vendor sourcing, evaluation, contracting
//! - [`packs::virtual_teams`] — Team formation, personas, content publishing
//! - [`packs::linkedin_research`] — Signal extraction, dossier building
//! - [`packs::reskilling`] — Skills assessment, learning plans, credentials
//! - [`packs::due_diligence`] — Convergent research, fact extraction, gap detection, synthesis
//!
//! # Blueprints
//!
//! Multi-pack orchestrations composing packs into end-to-end workflows:
//! - [`blueprints::lead_to_cash`] — Customers → Delivery → Legal → Money
//! - [`blueprints::hire_to_retire`] — Legal → People → Trust → Money
//! - [`blueprints::procure_to_pay`] — Procurement → Legal → Money
//! - [`blueprints::issue_to_resolution`] — Ops Support → Knowledge
//! - [`blueprints::idea_to_launch`] — Product Engineering → Delivery
//! - [`blueprints::campaign_to_revenue`] — Growth Marketing → Customers → Money
//! - [`blueprints::partner_to_value`] — Partnerships → Legal → Delivery
//! - [`blueprints::patent_research`] — Knowledge → Legal → IP pipeline
//! - [`blueprints::diligence_to_decision`] — DueDiligence → Legal → Knowledge

pub mod blueprints;
pub mod packs;
pub mod standard_packs;

pub use standard_packs::{register_standard_packs, registry_with_standard_packs};

#[derive(Clone, Copy, Debug)]
pub struct OrganismDomainProvenance;

impl converge_pack::ProvenanceSource for OrganismDomainProvenance {
    fn as_str(&self) -> &'static str {
        "organism-domain"
    }
}

pub const ORGANISM_DOMAIN_PROVENANCE: OrganismDomainProvenance = OrganismDomainProvenance;

// Pack contract types live in `organism-pack`. Re-exported here so existing
// consumers keep working with `organism_domain::pack::*` paths.
pub use organism_pack::pack;

/// Structured payload for organism-domain pack records that are represented as
/// JSON-shaped domain data.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrgDomainRecord {
    record_type: String,
    data: serde_json::Value,
}

impl OrgDomainRecord {
    #[must_use]
    pub fn new(record_type: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            record_type: record_type.into(),
            data,
        }
    }

    #[must_use]
    pub fn record_type(&self) -> &str {
        &self.record_type
    }

    #[must_use]
    pub fn data(&self) -> &serde_json::Value {
        &self.data
    }
}

impl converge_pack::FactPayload for OrgDomainRecord {
    const FAMILY: &'static str = "organism.domain_record";
    const VERSION: u16 = 1;
}

#[must_use]
pub fn record(record_type: impl Into<String>, data: serde_json::Value) -> OrgDomainRecord {
    OrgDomainRecord::new(record_type, data)
}

#[must_use]
pub fn fact_json(fact: &converge_pack::ContextFact) -> Option<serde_json::Value> {
    fact.payload::<OrgDomainRecord>()
        .map(|payload| payload.data().clone())
}

#[must_use]
pub fn fact_record<'a>(
    fact: &'a converge_pack::ContextFact,
    record_type: &str,
) -> Option<&'a OrgDomainRecord> {
    fact.payload::<OrgDomainRecord>()
        .filter(|payload| payload.record_type() == record_type)
}

#[must_use]
pub fn fact_json_of(
    fact: &converge_pack::ContextFact,
    record_type: &str,
) -> Option<serde_json::Value> {
    fact_record(fact, record_type).map(|payload| payload.data().clone())
}

#[cfg(test)]
mod tests {
    use crate::pack::{ContextKey, InvariantClass, Pack, PackProfile};
    use crate::packs;

    struct TestPack;

    impl Pack for TestPack {
        fn name(&self) -> &'static str {
            "test_pack"
        }
        fn agents(&self) -> &[crate::pack::AgentMeta] {
            &packs::customers::AGENTS[..1]
        }
        fn invariants(&self) -> &[crate::pack::InvariantMeta] {
            &packs::customers::INVARIANTS[..1]
        }
    }

    #[test]
    fn pack_trait_implementation() {
        let pack = TestPack;
        assert_eq!(pack.name(), "test_pack");
        assert_eq!(pack.agents().len(), 1);
        assert_eq!(pack.invariants().len(), 1);
    }

    #[test]
    fn customers_pack_agents_non_empty() {
        assert!(!packs::customers::AGENTS.is_empty());
        for agent in packs::customers::AGENTS {
            assert!(!agent.name.is_empty());
            assert!(!agent.fact_prefix.is_empty());
            assert!(agent.fact_prefix.ends_with(':'));
            assert!(!agent.description.is_empty());
        }
    }

    #[test]
    fn customers_pack_invariants_non_empty() {
        assert!(!packs::customers::INVARIANTS.is_empty());
        for inv in packs::customers::INVARIANTS {
            assert!(!inv.name.is_empty());
            assert!(!inv.description.is_empty());
        }
    }

    #[test]
    fn customers_profile_has_entities() {
        let p = &packs::customers::PROFILE;
        assert!(!p.entities.is_empty());
        assert!(p.entities.contains(&"lead"));
        assert!(p.entities.contains(&"deal"));
    }

    #[test]
    fn knowledge_pack_agents_non_empty() {
        assert!(!packs::knowledge::AGENTS.is_empty());
        for agent in packs::knowledge::AGENTS {
            assert!(!agent.name.is_empty());
            assert!(agent.fact_prefix.ends_with(':'));
        }
    }

    #[test]
    fn knowledge_pack_invariant_classes() {
        let classes: Vec<_> = packs::knowledge::INVARIANTS
            .iter()
            .map(|i| i.class)
            .collect();
        assert!(classes.contains(&InvariantClass::Structural));
        assert!(classes.contains(&InvariantClass::Semantic));
        assert!(classes.contains(&InvariantClass::Acceptance));
    }

    #[test]
    fn due_diligence_uses_llm() {
        let p = &packs::due_diligence::PROFILE;
        assert!(p.uses_llm);
        assert!(p.requires_hitl);
        assert!(p.required_capabilities.contains(&"web"));
        assert!(p.required_capabilities.contains(&"llm"));
    }

    #[test]
    fn all_packs_have_valid_fact_prefixes() {
        let all_agents: Vec<&crate::pack::AgentMeta> = [
            packs::customers::AGENTS,
            packs::knowledge::AGENTS,
            packs::people::AGENTS,
            packs::legal::AGENTS,
            packs::performance::AGENTS,
            packs::autonomous_org::AGENTS,
            packs::growth_marketing::AGENTS,
            packs::product_engineering::AGENTS,
            packs::ops_support::AGENTS,
            packs::procurement::AGENTS,
            packs::partnerships::AGENTS,
            packs::virtual_teams::AGENTS,
            packs::linkedin_research::AGENTS,
            packs::reskilling::AGENTS,
            packs::due_diligence::AGENTS,
        ]
        .iter()
        .flat_map(|agents| agents.iter())
        .collect();

        assert!(!all_agents.is_empty());
        for agent in &all_agents {
            assert!(
                agent.fact_prefix.ends_with(':'),
                "Agent {} has fact_prefix '{}' without trailing ':'",
                agent.name,
                agent.fact_prefix
            );
        }
    }

    #[test]
    fn all_packs_have_unique_agent_names_within_pack() {
        let pack_agents: Vec<(&str, &[crate::pack::AgentMeta])> = vec![
            ("customers", packs::customers::AGENTS),
            ("knowledge", packs::knowledge::AGENTS),
            ("people", packs::people::AGENTS),
            ("legal", packs::legal::AGENTS),
            ("due_diligence", packs::due_diligence::AGENTS),
        ];
        for (pack_name, agents) in pack_agents {
            let mut names: Vec<&str> = agents.iter().map(|a| a.name).collect();
            let original_len = names.len();
            names.sort_unstable();
            names.dedup();
            assert_eq!(
                names.len(),
                original_len,
                "Pack '{pack_name}' has duplicate agent names"
            );
        }
    }

    #[test]
    fn all_profiles_have_keywords() {
        let profiles: Vec<(&str, &PackProfile)> = vec![
            ("customers", &packs::customers::PROFILE),
            ("knowledge", &packs::knowledge::PROFILE),
            ("due_diligence", &packs::due_diligence::PROFILE),
        ];
        for (name, profile) in profiles {
            assert!(
                !profile.keywords.is_empty(),
                "Pack '{name}' has no keywords"
            );
        }
    }

    #[test]
    fn context_key_equality_and_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ContextKey::Seeds);
        set.insert(ContextKey::Seeds);
        set.insert(ContextKey::Signals);
        assert_eq!(set.len(), 2);
    }
}
