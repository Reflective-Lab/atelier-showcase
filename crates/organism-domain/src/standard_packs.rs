//! Register Organism's built-in domain packs into a runtime registry.
//!
//! organism-runtime is mechanism. The fourteen organizational workflow packs
//! (customers, legal, partnerships, …) are content. This module bridges the
//! two: it pushes the standard catalog into a `Registry` so apps can opt in
//! without `organism-runtime` itself depending on this crate.

use organism_runtime::Registry;

use crate::packs;

/// Build a registry preloaded with the fourteen standard organizational packs.
#[must_use]
pub fn registry_with_standard_packs() -> Registry {
    let mut registry = Registry::new();
    register_standard_packs(&mut registry);
    registry
}

/// Register the fourteen standard organizational packs into an existing
/// registry. Idempotent — packs already present (by name) are skipped.
pub fn register_standard_packs(registry: &mut Registry) {
    registry.register_pack_with_profile_if_missing(
        "customers",
        packs::customers::AGENTS,
        packs::customers::INVARIANTS,
        &packs::customers::PROFILE,
    );
    registry.register_pack_with_profile_if_missing(
        "legal",
        packs::legal::AGENTS,
        packs::legal::INVARIANTS,
        &packs::legal::PROFILE,
    );
    registry.register_pack_with_profile_if_missing(
        "autonomous_org",
        packs::autonomous_org::AGENTS,
        packs::autonomous_org::INVARIANTS,
        &packs::autonomous_org::PROFILE,
    );
    registry.register_pack_with_profile_if_missing(
        "partnerships",
        packs::partnerships::AGENTS,
        packs::partnerships::INVARIANTS,
        &packs::partnerships::PROFILE,
    );
    registry.register_pack_with_profile_if_missing(
        "people",
        packs::people::AGENTS,
        packs::people::INVARIANTS,
        &packs::people::PROFILE,
    );
    registry.register_pack_with_profile_if_missing(
        "procurement",
        packs::procurement::AGENTS,
        packs::procurement::INVARIANTS,
        &packs::procurement::PROFILE,
    );
    registry.register_pack_with_profile_if_missing(
        "linkedin_research",
        packs::linkedin_research::AGENTS,
        packs::linkedin_research::INVARIANTS,
        &packs::linkedin_research::PROFILE,
    );
    registry.register_pack_with_profile_if_missing(
        "knowledge",
        packs::knowledge::AGENTS,
        packs::knowledge::INVARIANTS,
        &packs::knowledge::PROFILE,
    );
    registry.register_pack_with_profile_if_missing(
        "growth_marketing",
        packs::growth_marketing::AGENTS,
        packs::growth_marketing::INVARIANTS,
        &packs::growth_marketing::PROFILE,
    );
    registry.register_pack_with_profile_if_missing(
        "ops_support",
        packs::ops_support::AGENTS,
        packs::ops_support::INVARIANTS,
        &packs::ops_support::PROFILE,
    );
    registry.register_pack_with_profile_if_missing(
        "performance",
        packs::performance::AGENTS,
        packs::performance::INVARIANTS,
        &packs::performance::PROFILE,
    );
    registry.register_pack_with_profile_if_missing(
        "product_engineering",
        packs::product_engineering::AGENTS,
        packs::product_engineering::INVARIANTS,
        &packs::product_engineering::PROFILE,
    );
    registry.register_pack_with_profile_if_missing(
        "virtual_teams",
        packs::virtual_teams::AGENTS,
        packs::virtual_teams::INVARIANTS,
        &packs::virtual_teams::PROFILE,
    );
    registry.register_pack_with_profile_if_missing(
        "reskilling",
        packs::reskilling::AGENTS,
        packs::reskilling::INVARIANTS,
        &packs::reskilling::PROFILE,
    );
}
