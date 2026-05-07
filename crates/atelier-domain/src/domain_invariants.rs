use converge_core::{ContextKey, Invariant, InvariantClass, InvariantResult, Violation};

/// Invariant: Access must be explicitly granted by an authority.
/// Derived from @authority_required in trust.feature.
pub struct AuthorityRequired;

impl Invariant for AuthorityRequired {
    fn name(&self) -> &str {
        "authority_required"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Semantic
    }

    fn check(&self, ctx: &dyn converge_core::Context) -> InvariantResult {
        // Simplified check for the 15 jobs alignment:
        // "Given an AccessFact exists, Then a corresponding AuthorityDecision must exist in history"

        // We'll look for facts in any category that contain "access:granted"
        for key in [
            ContextKey::Signals,
            ContextKey::Strategies,
            ContextKey::Hypotheses,
        ] {
            for fact in ctx.get(key) {
                if fact.content().contains("access:granted") {
                    let has_authority = ctx.get(ContextKey::Signals).iter().any(|f| {
                        f.content().contains("authority:approved")
                            && f.content().contains(fact.id().as_str())
                    });

                    if !has_authority {
                        return InvariantResult::Violated(Violation::with_facts(
                            format!(
                                "Access grant {} lacks explicit authority approval",
                                fact.id().as_str()
                            ),
                            vec![fact.id().clone()],
                        ));
                    }
                }
            }
        }

        InvariantResult::Ok
    }
}

/// Invariant: Every transaction must have provenance.
/// Derived from @audit_trail_required in money.feature.
pub struct AuditTrailRequired;

impl Invariant for AuditTrailRequired {
    fn name(&self) -> &str {
        "audit_trail_required"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Structural
    }

    fn check(&self, ctx: &dyn converge_core::Context) -> InvariantResult {
        // In a real system, we'd check the internal 'ContextFact' metadata.
        // For this MVP, we'll check if the content contains a 'provenance:' or 'by:' tag.

        for key in [ContextKey::Strategies, ContextKey::Evaluations] {
            for fact in ctx.get(key) {
                if !fact.content().contains("provenance:") && !fact.content().contains("by:") {
                    return InvariantResult::Violated(Violation::with_facts(
                        format!(
                            "ContextFact {} is missing required provenance metadata",
                            fact.id().as_str()
                        ),
                        vec![fact.id().clone()],
                    ));
                }
            }
        }

        InvariantResult::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use converge_core::ContextFact;
    use std::collections::HashMap;

    struct FakeCtx(HashMap<ContextKey, Vec<ContextFact>>);

    impl converge_core::Context for FakeCtx {
        fn has(&self, key: ContextKey) -> bool {
            self.0.get(&key).is_some_and(|v| !v.is_empty())
        }
        fn get(&self, key: ContextKey) -> &[ContextFact] {
            self.0.get(&key).map_or(&[], Vec::as_slice)
        }
    }

    #[test]
    fn authority_required_name_and_class() {
        let inv = AuthorityRequired;
        assert_eq!(inv.name(), "authority_required");
        assert_eq!(inv.class(), InvariantClass::Semantic);
    }

    #[test]
    fn authority_required_passes_with_empty_ctx() {
        let inv = AuthorityRequired;
        let ctx = FakeCtx(HashMap::new());
        assert_eq!(inv.check(&ctx), InvariantResult::Ok);
    }

    #[test]
    fn audit_trail_required_name_and_class() {
        let inv = AuditTrailRequired;
        assert_eq!(inv.name(), "audit_trail_required");
        assert_eq!(inv.class(), InvariantClass::Structural);
    }

    #[test]
    fn audit_trail_required_passes_with_empty_ctx() {
        let inv = AuditTrailRequired;
        let ctx = FakeCtx(HashMap::new());
        assert_eq!(inv.check(&ctx), InvariantResult::Ok);
    }

    use converge_core::{ContextState, Engine};

    fn promoted(entries: &[(ContextKey, &str, &str)]) -> ContextState {
        let mut ctx = ContextState::new();
        for (key, id, content) in entries {
            ctx.add_input(*key, *id, *content).unwrap();
        }
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Engine::new().run(ctx))
            .unwrap()
            .context
    }

    #[test]
    fn authority_required_violates_on_unapproved_access() {
        let inv = AuthorityRequired;
        let ctx = promoted(&[(
            ContextKey::Strategies,
            "access:1",
            "access:granted to user X",
        )]);
        assert!(matches!(inv.check(&ctx), InvariantResult::Violated(_)));
    }

    #[test]
    fn authority_required_passes_when_authority_present() {
        let inv = AuthorityRequired;
        let ctx = promoted(&[
            (
                ContextKey::Strategies,
                "access:1",
                "access:granted access:1",
            ),
            (
                ContextKey::Signals,
                "auth:1",
                "authority:approved access:1",
            ),
        ]);
        assert_eq!(inv.check(&ctx), InvariantResult::Ok);
    }

    #[test]
    fn authority_required_ignores_unrelated_facts() {
        let inv = AuthorityRequired;
        let ctx = promoted(&[(
            ContextKey::Strategies,
            "info:1",
            "some unrelated content",
        )]);
        assert_eq!(inv.check(&ctx), InvariantResult::Ok);
    }

    #[test]
    fn audit_trail_required_violates_when_provenance_missing() {
        let inv = AuditTrailRequired;
        let ctx = promoted(&[(
            ContextKey::Strategies,
            "draft:1",
            "draft body without metadata",
        )]);
        assert!(matches!(inv.check(&ctx), InvariantResult::Violated(_)));
    }

    #[test]
    fn audit_trail_required_passes_with_provenance_tag() {
        let inv = AuditTrailRequired;
        let ctx = promoted(&[(
            ContextKey::Strategies,
            "draft:1",
            "content provenance: alice",
        )]);
        assert_eq!(inv.check(&ctx), InvariantResult::Ok);
    }

    #[test]
    fn audit_trail_required_passes_with_by_tag() {
        let inv = AuditTrailRequired;
        let ctx = promoted(&[(ContextKey::Evaluations, "eval:1", "result by: bob")]);
        assert_eq!(inv.check(&ctx), InvariantResult::Ok);
    }
}
