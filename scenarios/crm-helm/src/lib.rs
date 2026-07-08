// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Headless CRM Helm scenario library.
//!
//! Assembles the 7 CRM gRPC modules over an in-memory kernel store and
//! event substrate. No TCP bind, no RunwayAppHost, no StorageKit.
//!
//! ## Hub / lease wiring
//!
//! All 7 module constructors accept `_store: AppKernelStore` and currently
//! ignore it (the gRPC service structs carry the store directly). None of the
//! module constructors accept a hub or a lease store as parameters — both are
//! allocated here and available for future module upgrades, but zero of the
//! 7 modules consume them today.
//!
//! ## RFL-171 T7
//!
//! This file is the headless variant produced for atelier-showcase as part of
//! Seam A (helm-event-substrate extraction). The runway-backed composition
//! root lives at `helms/apps/crm-helm/src/main.rs`.

pub mod conversations;
pub mod documents;
pub mod facts;
pub mod metadata;
pub mod opportunities;
pub mod parties;
pub mod proto;
pub mod shared;
pub mod truths;
pub mod workbench;
pub mod workflow;

use std::sync::Arc;

use application_storage::{AppKernelStore, InMemoryKernelStore};
use helm_event_substrate::{EventHub, InMemoryLeaseStore};
use helm_module_contracts::HelmModule;
use serde_json::{Value, json};

/// A CRM event emitted by the headless scenario.
#[derive(Debug, Clone)]
pub struct CrmEvent {
    pub sequence: u64,
    pub kind: String,
    pub payload: Value,
}

/// Drives a headless CRM assembly and returns the assembled router + events.
pub struct CrmHelmRun {
    pub module_ids: Vec<&'static str>,
    pub events: Vec<CrmEvent>,
    pub router: axum::Router,
}

impl CrmHelmRun {
    /// Assemble the 7 CRM modules over an in-memory store and substrate.
    ///
    /// Returns the assembled router and a JSONL-ready event trace showing the
    /// assembly and init sequence.
    pub async fn assemble() -> anyhow::Result<Self> {
        let store = AppKernelStore::Memory(InMemoryKernelStore::default_local());

        // Substrate — allocated here, available for future module upgrades.
        // Currently zero of the 7 module constructors consume hub or leases.
        let _hub = EventHub::with_capacity(1024);
        let _lease_store = Arc::new(InMemoryLeaseStore::new());

        let modules: Vec<Arc<dyn HelmModule>> = vec![
            Arc::new(parties::PartiesModule::new(store.clone())),
            Arc::new(opportunities::OpportunitiesModule::new(store.clone())),
            Arc::new(conversations::ConversationsModule::new(store.clone())),
            Arc::new(documents::DocumentsModule::new(store.clone())),
            Arc::new(workflow::WorkflowModule::new(store.clone())),
            Arc::new(facts::FactsModule::new(store.clone())),
            Arc::new(metadata::MetadataModule::new(store)),
        ];

        let mut events = Vec::new();
        let mut router = axum::Router::new();
        let mut module_ids = Vec::new();

        for module in modules {
            let id = module.module_id();
            module.init().await?;
            events.push(CrmEvent {
                sequence: events.len() as u64 + 1,
                kind: "module.init".to_string(),
                payload: json!({ "module_id": id }),
            });
            router = router.merge(module.clone().router());
            module_ids.push(id);
        }

        events.push(CrmEvent {
            sequence: events.len() as u64 + 1,
            kind: "assembly.complete".to_string(),
            payload: json!({
                "module_count": module_ids.len(),
                "module_ids": module_ids,
                "hub_capacity": 1024,
                "lease_store": "InMemoryLeaseStore",
                "hub_consumers": 0,
                "lease_consumers": 0,
            }),
        });

        Ok(Self {
            module_ids,
            events,
            router,
        })
    }

    /// Emit events as JSONL.
    pub fn jsonl(&self) -> String {
        let mut out = String::new();
        for event in &self.events {
            out.push_str(
                &serde_json::to_string(&json!({
                    "sequence": event.sequence,
                    "kind": event.kind,
                    "payload": event.payload,
                }))
                .expect("CrmEvent serializes"),
            );
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use http::Request;
    use tower::ServiceExt;

    use super::*;

    /// Assembly emits init events for all 7 modules followed by an
    /// assembly.complete event.
    #[tokio::test]
    async fn assembly_emits_module_init_and_complete_events() {
        let run = CrmHelmRun::assemble().await.expect("assembly succeeds");

        assert_eq!(run.module_ids.len(), 7);

        let init_events: Vec<_> = run
            .events
            .iter()
            .filter(|e| e.kind == "module.init")
            .collect();
        assert_eq!(init_events.len(), 7, "one init event per module");

        let complete = run
            .events
            .last()
            .expect("at least one event");
        assert_eq!(complete.kind, "assembly.complete");
        assert_eq!(
            complete.payload["module_count"],
            serde_json::json!(7)
        );
        // Honest: hub and leases are allocated but no module consumes them yet.
        assert_eq!(complete.payload["hub_consumers"], serde_json::json!(0));
        assert_eq!(complete.payload["lease_consumers"], serde_json::json!(0));
    }

    /// Each module family answers its status route with HTTP 200 via oneshot.
    #[tokio::test]
    async fn assembled_router_answers_all_module_status_routes() {
        let run = CrmHelmRun::assemble().await.expect("assembly succeeds");

        let routes = [
            "/crm/parties/status",
            "/crm/opportunities/status",
            "/crm/conversations/status",
            "/crm/documents/status",
            "/crm/workflow/status",
            "/crm/facts/status",
            "/crm/metadata/status",
        ];

        for path in routes {
            let req = Request::builder()
                .method("GET")
                .uri(path)
                .body(Body::empty())
                .expect("request builds");

            let response = run
                .router
                .clone()
                .oneshot(req)
                .await
                .expect("router responds");

            assert_eq!(
                response.status().as_u16(),
                200,
                "module route {path} must return 200"
            );
        }
    }

    /// JSONL output is valid JSON on every line and includes assembly and probe events.
    #[tokio::test]
    async fn jsonl_output_is_valid_per_line() {
        let run = CrmHelmRun::assemble().await.expect("assembly succeeds");
        let jsonl = run.jsonl();

        for line in jsonl.lines() {
            let parsed: serde_json::Value =
                serde_json::from_str(line).expect("each JSONL line parses as JSON");
            assert!(
                parsed.get("sequence").is_some(),
                "every line has a sequence field"
            );
            assert!(
                parsed.get("kind").is_some(),
                "every line has a kind field"
            );
        }

        // 7 init events + 1 assembly.complete
        assert_eq!(jsonl.lines().count(), 8);
    }

    /// InMemoryLeaseStore direct ownership check — honest about the absence of
    /// module-surface lease consumption in the current T7 state.
    ///
    /// Until a module constructor accepts a LeaseStore, this test exercises the
    /// store's contract directly: acquire succeeds for the first holder and
    /// returns HeldByOther for a competing holder while the lease is live.
    #[tokio::test]
    async fn in_memory_lease_store_contract_acquires_and_blocks() {
        use helm_event_substrate::{AcquireOutcome, InMemoryLeaseStore, LeaseScope, LeaseStore};
        use std::time::Duration;

        let store = InMemoryLeaseStore::new();
        let scope = LeaseScope {
            org_id: "org-001".to_string(),
            app_id: "crm-helm".to_string(),
            session_id: "session-001".to_string(),
        };

        let outcome_a = store
            .try_acquire(&scope, "holder-a", Duration::from_secs(30))
            .await
            .expect("acquire does not error");
        assert!(
            matches!(outcome_a, AcquireOutcome::Acquired(_)),
            "first acquire succeeds"
        );

        let outcome_b = store
            .try_acquire(&scope, "holder-b", Duration::from_secs(30))
            .await
            .expect("acquire does not error");
        assert!(
            matches!(outcome_b, AcquireOutcome::HeldByOther(_)),
            "second holder sees HeldByOther while first holds the lease"
        );
    }
}
