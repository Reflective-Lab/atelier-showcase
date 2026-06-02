// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Atelier fact payloads for the Converge 3.9 typed fact boundary.

use converge_core::{
    ContextFact, ContextKey, FactPayload, Provenance, ProvenanceSource, TextPayload,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// Canonical provenance marker for facts emitted by this crate.
#[derive(Clone, Copy, Debug)]
pub struct AtelierDomainProvenance;

impl ProvenanceSource for AtelierDomainProvenance {
    fn as_str(&self) -> &'static str {
        "atelier-domain"
    }
}

impl AtelierDomainProvenance {
    #[must_use]
    pub fn provenance(self) -> Provenance {
        ProvenanceSource::provenance(self)
    }
}

pub const ATELIER_DOMAIN_PROVENANCE: AtelierDomainProvenance = AtelierDomainProvenance;

/// Structured record payload for domain-pack records that are still
/// represented as JSON-shaped domain data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DomainRecordPayload {
    record_type: String,
    data: serde_json::Value,
}

impl DomainRecordPayload {
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

    #[must_use]
    pub fn string_field(&self, field: &str) -> Option<&str> {
        self.data.get(field).and_then(serde_json::Value::as_str)
    }

    #[must_use]
    pub fn bool_field(&self, field: &str) -> Option<bool> {
        self.data.get(field).and_then(serde_json::Value::as_bool)
    }

    #[must_use]
    pub fn has_field(&self, field: &str) -> bool {
        self.data.get(field).is_some()
    }

    #[must_use]
    pub fn string_field_is(&self, field: &str, value: &str) -> bool {
        self.string_field(field) == Some(value)
    }

    #[must_use]
    pub fn bool_field_is(&self, field: &str, value: bool) -> bool {
        self.bool_field(field) == Some(value)
    }

    pub fn parse_data<T: DeserializeOwned>(&self) -> Option<T> {
        serde_json::from_value(self.data.clone()).ok()
    }
}

impl FactPayload for DomainRecordPayload {
    const FAMILY: &'static str = "atelier.domain_record";
    const VERSION: u16 = 1;
}

/// Free-form human text whose semantics are intentionally textual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DomainTextPayload {
    text_type: String,
    text: String,
}

impl DomainTextPayload {
    #[must_use]
    pub fn new(text_type: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            text_type: text_type.into(),
            text: text.into(),
        }
    }

    #[must_use]
    pub fn text_type(&self) -> &str {
        &self.text_type
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

impl FactPayload for DomainTextPayload {
    const FAMILY: &'static str = "atelier.domain_text";
    const VERSION: u16 = 1;
}

/// Reads a structured record payload from a fact.
#[must_use]
pub fn record_payload(fact: &ContextFact) -> Option<&DomainRecordPayload> {
    fact.payload::<DomainRecordPayload>()
}

/// Deserializes a structured record payload into a domain type.
pub fn record_data<T: DeserializeOwned>(fact: &ContextFact) -> Option<T> {
    record_payload(fact).and_then(DomainRecordPayload::parse_data)
}

/// Reads JSON domain data from either a typed domain record or an admitted
/// text seed carrying JSON at the external boundary.
#[must_use]
pub fn json_value(fact: &ContextFact) -> Option<serde_json::Value> {
    record_payload(fact)
        .map(|payload| payload.data().clone())
        .or_else(|| admitted_text(fact).and_then(|text| serde_json::from_str(text).ok()))
}

/// Reads a free-form domain text payload from a fact.
#[must_use]
pub fn domain_text(fact: &ContextFact) -> Option<&str> {
    fact.payload::<DomainTextPayload>()
        .map(DomainTextPayload::as_str)
}

/// Reads text admitted through Converge's external input boundary.
#[must_use]
pub fn admitted_text(fact: &ContextFact) -> Option<&str> {
    fact.payload::<TextPayload>().map(TextPayload::as_str)
}

/// Typed payload predicate for legacy heuristic evals. It checks only text
/// payloads and JSON string/object fields; it does not serialize payloads back
/// to ad hoc strings.
#[must_use]
pub fn payload_contains(fact: &ContextFact, needle: &str) -> bool {
    domain_text(fact).is_some_and(|text| text.contains(needle))
        || admitted_text(fact).is_some_and(|text| text.contains(needle))
        || record_payload(fact).is_some_and(|payload| json_contains(payload.data(), needle))
}

#[must_use]
pub fn payload_any(fact: &ContextFact, needles: &[&str]) -> bool {
    needles.iter().any(|needle| payload_contains(fact, needle))
}

fn json_contains(value: &serde_json::Value, needle: &str) -> bool {
    if json_matches_fragment(value, needle) {
        return true;
    }

    match value {
        serde_json::Value::String(text) => text.contains(needle),
        serde_json::Value::Array(items) => items.iter().any(|item| json_contains(item, needle)),
        serde_json::Value::Object(map) => map
            .iter()
            .any(|(key, value)| key.contains(needle) || json_contains(value, needle)),
        serde_json::Value::Bool(value) => {
            (needle == "true" && *value) || (needle == "false" && !*value)
        }
        serde_json::Value::Number(_) => false,
        serde_json::Value::Null => needle == "null",
    }
}

fn json_matches_fragment(value: &serde_json::Value, needle: &str) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };

    let trimmed = needle.trim();
    let Some((field, expected)) = trimmed
        .strip_prefix('"')
        .and_then(|rest| rest.split_once("\":"))
    else {
        return false;
    };

    let Some(actual) = object.get(field) else {
        return false;
    };

    if let Some(expected) = expected
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        return actual.as_str() == Some(expected);
    }

    match expected {
        "true" => actual.as_bool() == Some(true),
        "false" => actual.as_bool() == Some(false),
        _ => false,
    }
}

/// Returns true when any fact under `key` has `id`.
#[must_use]
pub fn has_fact(ctx: &dyn converge_core::Context, key: ContextKey, id: &str) -> bool {
    ctx.get(key).iter().any(|fact| fact.id().as_str() == id)
}
