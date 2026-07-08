// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! CRM Helm headless scenario binary.
//!
//! Assembles 7 CRM modules over an in-memory kernel store and event substrate,
//! drives a demo flow via `tower::ServiceExt::oneshot`, and prints JSONL output.
//!
//! No TCP bind. No RunwayAppHost. No StorageKit.

use axum::body::Body;
use http::Request;
use scenario_crm_helm::CrmHelmRun;
use tower::ServiceExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let run = CrmHelmRun::assemble().await?;

    // Drive a representative request through each module's status route.
    let module_routes = [
        "/crm/parties/status",
        "/crm/opportunities/status",
        "/crm/conversations/status",
        "/crm/documents/status",
        "/crm/workflow/status",
        "/crm/facts/status",
        "/crm/metadata/status",
    ];

    let mut probe_events: Vec<serde_json::Value> = Vec::new();

    for path in module_routes {
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

        let status = response.status().as_u16();
        probe_events.push(serde_json::json!({
            "kind": "probe.response",
            "path": path,
            "status": status,
        }));
    }

    // Print assembly events
    print!("{}", run.jsonl());

    // Print probe events as JSONL
    let probe_sequence_start = run.events.len() as u64 + 1;
    for (i, ev) in probe_events.iter().enumerate() {
        let line = serde_json::json!({
            "sequence": probe_sequence_start + i as u64,
            "kind": ev["kind"],
            "payload": {
                "path": ev["path"],
                "status": ev["status"],
            }
        });
        println!("{}", serde_json::to_string(&line).expect("probe event serializes"));
    }

    Ok(())
}
