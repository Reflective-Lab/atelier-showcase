// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Deterministic drafting flow (short path).
//!
//! Seeds -> Signals (research notes) -> Strategies (draft output)

use converge_core::{AgentEffect, ContextKey, Suggestor};

const DRAFT_RESEARCH_PREFIX: &str = "drafting_research:";
const DRAFT_OUTPUT_PREFIX: &str = "drafting_output:";

/// Drafting research agent (deterministic fallback).
pub struct DraftingResearchAgent;

#[async_trait::async_trait]
impl Suggestor for DraftingResearchAgent {
    fn name(&self) -> &'static str {
        "DraftingResearchAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::Context) -> bool {
        ctx.has(ContextKey::Seeds)
            && !ctx
                .get(ContextKey::Signals)
                .iter()
                .any(|fact| fact.id().as_str().starts_with(DRAFT_RESEARCH_PREFIX))
    }

    async fn execute(&self, ctx: &dyn converge_core::Context) -> AgentEffect {
        let summary = ctx
            .get(ContextKey::Seeds)
            .iter()
            .map(|seed| seed.content().to_string())
            .collect::<Vec<_>>()
            .join(" | ");

        AgentEffect::with_proposal(crate::proposal(
            self.name(),
            ContextKey::Signals,
            format!("{DRAFT_RESEARCH_PREFIX}notes"),
            format!("Drafting research notes: {summary}"),
        ))
    }
}

/// Drafting composer agent (deterministic fallback).
pub struct DraftingComposerAgent;

#[async_trait::async_trait]
impl Suggestor for DraftingComposerAgent {
    fn name(&self) -> &'static str {
        "DraftingComposerAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_core::Context) -> bool {
        ctx.get(ContextKey::Signals)
            .iter()
            .any(|fact| fact.id().as_str().starts_with(DRAFT_RESEARCH_PREFIX))
            && !ctx
                .get(ContextKey::Strategies)
                .iter()
                .any(|fact| fact.id().as_str().starts_with(DRAFT_OUTPUT_PREFIX))
    }

    async fn execute(&self, ctx: &dyn converge_core::Context) -> AgentEffect {
        let notes = ctx
            .get(ContextKey::Signals)
            .iter()
            .filter(|fact| fact.id().as_str().starts_with(DRAFT_RESEARCH_PREFIX))
            .map(|fact| fact.content().to_string())
            .collect::<Vec<_>>()
            .join("\n");

        AgentEffect::with_proposal(crate::proposal(
            self.name(),
            ContextKey::Strategies,
            format!("{DRAFT_OUTPUT_PREFIX}v0"),
            format!("Draft output (deterministic):\n{notes}"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use converge_core::{ContextState, Engine};

    fn promoted_context(entries: &[(ContextKey, &str, &str)]) -> ContextState {
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
    fn research_agent_metadata() {
        let agent = DraftingResearchAgent;
        assert_eq!(agent.name(), "DraftingResearchAgent");
        assert_eq!(agent.dependencies(), &[ContextKey::Seeds]);
    }

    #[test]
    fn research_agent_rejects_without_seeds() {
        let agent = DraftingResearchAgent;
        let ctx = promoted_context(&[]);
        assert!(!agent.accepts(&ctx));
    }

    #[test]
    fn research_agent_accepts_with_seeds_no_prior_notes() {
        let agent = DraftingResearchAgent;
        let ctx = promoted_context(&[(ContextKey::Seeds, "seed-1", "topic A")]);
        assert!(agent.accepts(&ctx));
    }

    #[test]
    fn research_agent_idempotent_after_notes_emitted() {
        let agent = DraftingResearchAgent;
        let ctx = promoted_context(&[
            (ContextKey::Seeds, "seed-1", "topic A"),
            (
                ContextKey::Signals,
                "drafting_research:notes",
                "existing notes",
            ),
        ]);
        assert!(!agent.accepts(&ctx));
    }

    #[test]
    fn research_agent_emits_combined_notes() {
        let agent = DraftingResearchAgent;
        let ctx = promoted_context(&[
            (ContextKey::Seeds, "seed-1", "topic A"),
            (ContextKey::Seeds, "seed-2", "topic B"),
        ]);
        let effect = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(agent.execute(&ctx));
        let proposals = effect.proposals();
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].key, ContextKey::Signals);
        assert!(proposals[0].content.contains("topic A"));
        assert!(proposals[0].content.contains("topic B"));
    }

    #[test]
    fn composer_agent_metadata() {
        let agent = DraftingComposerAgent;
        assert_eq!(agent.name(), "DraftingComposerAgent");
        assert_eq!(agent.dependencies(), &[ContextKey::Signals]);
    }

    #[test]
    fn composer_agent_rejects_without_research_notes() {
        let agent = DraftingComposerAgent;
        let ctx = promoted_context(&[(ContextKey::Signals, "other:fact", "unrelated signal")]);
        assert!(!agent.accepts(&ctx));
    }

    #[test]
    fn composer_agent_accepts_with_research_notes() {
        let agent = DraftingComposerAgent;
        let ctx = promoted_context(&[(
            ContextKey::Signals,
            "drafting_research:notes",
            "research output",
        )]);
        assert!(agent.accepts(&ctx));
    }

    #[test]
    fn composer_agent_idempotent_after_draft_emitted() {
        let agent = DraftingComposerAgent;
        let ctx = promoted_context(&[
            (
                ContextKey::Signals,
                "drafting_research:notes",
                "research output",
            ),
            (
                ContextKey::Strategies,
                "drafting_output:v0",
                "existing draft",
            ),
        ]);
        assert!(!agent.accepts(&ctx));
    }

    #[test]
    fn composer_agent_concatenates_research_notes() {
        let agent = DraftingComposerAgent;
        let ctx = promoted_context(&[
            (ContextKey::Signals, "drafting_research:a", "first"),
            (ContextKey::Signals, "drafting_research:b", "second"),
            (ContextKey::Signals, "other:fact", "ignored"),
        ]);
        let effect = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(agent.execute(&ctx));
        let proposals = effect.proposals();
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].key, ContextKey::Strategies);
        assert!(proposals[0].content.contains("first"));
        assert!(proposals[0].content.contains("second"));
        assert!(!proposals[0].content.contains("ignored"));
    }
}
