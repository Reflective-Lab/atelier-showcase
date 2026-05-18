// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Ask Converge domain pack (grounded Q&A).
//!
//! This pack enforces grounded answering with explicit recall-only sources.

use converge_core::invariant::{Invariant, InvariantClass, InvariantResult, Violation};
use converge_core::{AgentEffect, ContextKey, FactPayload, Suggestor, TextPayload};
use serde::{Deserialize, Serialize};

const QUESTION_SEED_ID: &str = "ask:question";
const SOURCE_SEED_PREFIX: &str = "ask:source:";
const ANSWER_ID: &str = "ask:answer";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct AskSourcePayload {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    url: Option<String>,
    content: String,
}

#[derive(Debug, Clone)]
struct AskSource {
    id: String,
    title: Option<String>,
    url: Option<String>,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct AskAnswerPayload {
    question: String,
    answer: String,
    grounded: bool,
    recall_only: bool,
    sources: Vec<AskSourcePayload>,
}

impl FactPayload for AskAnswerPayload {
    const FAMILY: &'static str = "atelier.ask.answer";
    const VERSION: u16 = 1;
}

fn seed_text(seed: &converge_core::ContextFact) -> Option<&str> {
    seed.payload::<TextPayload>().map(TextPayload::as_str)
}

fn parse_question(ctx: &dyn converge_core::Context) -> Option<String> {
    ctx.get(ContextKey::Seeds)
        .iter()
        .find(|seed| seed.id().as_str() == QUESTION_SEED_ID)
        .and_then(seed_text)
        .map(str::to_string)
}

fn parse_sources(ctx: &dyn converge_core::Context) -> Vec<AskSource> {
    ctx.get(ContextKey::Seeds)
        .iter()
        .filter(|seed| seed.id().as_str().starts_with(SOURCE_SEED_PREFIX))
        .filter_map(|seed| {
            let text = seed_text(seed)?;
            let payload: Option<AskSourcePayload> = serde_json::from_str(text).ok();
            if let Some(payload) = payload {
                Some(AskSource {
                    id: payload.id.unwrap_or_else(|| seed.id().as_str().to_string()),
                    title: payload.title,
                    url: payload.url,
                    content: payload.content,
                })
            } else {
                Some(AskSource {
                    id: seed.id().as_str().to_string(),
                    title: None,
                    url: None,
                    content: text.to_string(),
                })
            }
        })
        .collect()
}

fn build_answer(question: &str, sources: &[AskSource]) -> AskAnswerPayload {
    let source_ids: Vec<&str> = sources.iter().map(|source| source.id.as_str()).collect();
    let answer_text = format!(
        "Grounded response based on sources: {}.",
        source_ids.join(", ")
    );

    AskAnswerPayload {
        question: question.to_string(),
        answer: answer_text,
        grounded: true,
        recall_only: true,
        sources: sources
            .iter()
            .map(|source| AskSourcePayload {
                id: Some(source.id.clone()),
                title: source.title.clone(),
                url: source.url.clone(),
                content: source.content.clone(),
            })
            .collect(),
    }
}

/// Suggestor that produces a grounded answer based on provided sources.
#[derive(Debug, Clone, Default)]
pub struct AskConvergeAgent;

#[async_trait::async_trait]
impl Suggestor for AskConvergeAgent {
    fn name(&self) -> &'static str {
        "ask_converge"
    }

    fn provenance(&self) -> &'static str {
        crate::ATELIER_DOMAIN_PROVENANCE
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::Context) -> bool {
        let has_question = parse_question(ctx).is_some();
        let has_answer = ctx
            .get(ContextKey::Strategies)
            .iter()
            .any(|fact| fact.id().as_str() == ANSWER_ID);
        has_question && !has_answer
    }

    async fn execute(&self, ctx: &dyn converge_core::Context) -> AgentEffect {
        let Some(question) = parse_question(ctx) else {
            return AgentEffect::empty();
        };
        let sources = parse_sources(ctx);

        if sources.is_empty() {
            return AgentEffect::empty();
        }

        let answer = build_answer(&question, &sources);
        let fact = crate::proposal(self.name(), ContextKey::Strategies, ANSWER_ID, answer);

        AgentEffect::with_proposal(fact)
    }
}

/// Enforces grounded answering (answers must include sources).
pub struct GroundedAnswerInvariant;

impl Invariant for GroundedAnswerInvariant {
    fn name(&self) -> &'static str {
        "grounded_answer_required"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Semantic
    }

    fn check(&self, ctx: &dyn converge_core::Context) -> InvariantResult {
        for fact in ctx.get(ContextKey::Strategies) {
            if fact.id().as_str() != ANSWER_ID {
                continue;
            }

            let Some(payload) = fact.payload::<AskAnswerPayload>() else {
                return InvariantResult::Violated(Violation::new(
                    "Ask answer must use AskAnswerPayload".to_string(),
                ));
            };

            if !payload.grounded || payload.sources.is_empty() {
                return InvariantResult::Violated(Violation::new(
                    "Ask answer must be grounded with at least one source".to_string(),
                ));
            }
        }

        InvariantResult::Ok
    }
}

/// Enforces recall-only usage (sources are recall, not evidence).
pub struct RecallNotEvidenceInvariant;

impl Invariant for RecallNotEvidenceInvariant {
    fn name(&self) -> &'static str {
        "recall_not_evidence"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Semantic
    }

    fn check(&self, ctx: &dyn converge_core::Context) -> InvariantResult {
        for fact in ctx.get(ContextKey::Strategies) {
            if fact.id().as_str() != ANSWER_ID {
                continue;
            }

            let Some(payload) = fact.payload::<AskAnswerPayload>() else {
                return InvariantResult::Violated(Violation::new(
                    "Ask answer must use AskAnswerPayload".to_string(),
                ));
            };

            if !payload.recall_only {
                return InvariantResult::Violated(Violation::new(
                    "Ask answer must be marked recall_only".to_string(),
                ));
            }
        }

        InvariantResult::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use converge_core::{ContextState, Engine, ProposedFact};

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
    fn ask_agent_emits_answer_with_sources() {
        let source = serde_json::json!({
            "id": "source-1",
            "content": "Converge is a semantic governance runtime."
        })
        .to_string();
        let ctx = promoted_context(&[
            (ContextKey::Seeds, QUESTION_SEED_ID, "What is Converge?"),
            (ContextKey::Seeds, "ask:source:1", source.as_str()),
        ]);

        let agent = AskConvergeAgent;
        let effect = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(agent.execute(&ctx));
        assert!(!effect.is_empty());
        assert_eq!(effect.proposals().len(), 1);
    }

    #[test]
    fn invariants_accept_grounded_answer() {
        let payload = build_answer(
            "What is Converge?",
            &[AskSource {
                id: "source-1".to_string(),
                title: None,
                url: None,
                content: "Converge is a semantic governance runtime.".to_string(),
            }],
        );
        let mut ctx = ContextState::new();
        ctx.add_proposal(ProposedFact::new(
            ContextKey::Strategies,
            ANSWER_ID,
            payload,
            crate::ATELIER_DOMAIN_PROVENANCE,
        ))
        .unwrap();
        let ctx = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Engine::new().run(ctx))
            .unwrap()
            .context;

        assert!(matches!(
            GroundedAnswerInvariant.check(&ctx),
            InvariantResult::Ok
        ));
        assert!(matches!(
            RecallNotEvidenceInvariant.check(&ctx),
            InvariantResult::Ok
        ));
    }
}
