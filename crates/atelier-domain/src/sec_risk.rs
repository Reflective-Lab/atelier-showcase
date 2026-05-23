// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! SEC filing risk-review policy pack.
//!
//! This is deliberately small, but it is a real policy surface: the
//! scenario imports rules from here rather than hand-authoring a
//! one-off threshold in `main`.

use arbiter::{ComplianceCondition, ComplianceRule};

pub const SEC_RISK_FRAMEWORK: &str = "SEC-10K-RISK-REVIEW";
pub const SEC_RISK_POLICY_ID: &str = "sec-10k-risk-review-v1";
pub const HEADING_COUNT_REVIEW_RULE_ID: &str = "sec-risk-heading-count-review";
pub const SECTION_SIZE_REVIEW_RULE_ID: &str = "sec-risk-item-1a-section-size-review";
pub const UNTRUSTED_PROVIDER_RULE_ID: &str = "sec-risk-untrusted-provider-review";
pub const SOURCE_SHAPE_RULE_ID: &str = "sec-risk-source-shape-review";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SecRiskPolicyThresholds {
    pub max_headings_for_auto_clearance: f64,
    pub min_item_1a_section_bytes: f64,
    pub max_item_1a_section_bytes: f64,
}

impl Default for SecRiskPolicyThresholds {
    fn default() -> Self {
        Self {
            max_headings_for_auto_clearance: 20.0,
            min_item_1a_section_bytes: 10_000.0,
            max_item_1a_section_bytes: 250_000.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SecRiskPolicyPack {
    thresholds: SecRiskPolicyThresholds,
}

impl SecRiskPolicyPack {
    #[must_use]
    pub fn annual_report_review() -> Self {
        Self {
            thresholds: SecRiskPolicyThresholds::default(),
        }
    }

    #[must_use]
    pub const fn thresholds(self) -> SecRiskPolicyThresholds {
        self.thresholds
    }

    #[must_use]
    pub fn rules(self) -> Vec<ComplianceRule> {
        vec![
            ComplianceRule {
                id: HEADING_COUNT_REVIEW_RULE_ID.to_string(),
                framework: SEC_RISK_FRAMEWORK.to_string(),
                field: "risk_factor_heading_count".to_string(),
                condition: ComplianceCondition::MaxValue(
                    self.thresholds.max_headings_for_auto_clearance,
                ),
            },
            ComplianceRule {
                id: SECTION_SIZE_REVIEW_RULE_ID.to_string(),
                framework: SEC_RISK_FRAMEWORK.to_string(),
                field: "item_1a_section_bytes".to_string(),
                condition: ComplianceCondition::NumericRange {
                    lo: self.thresholds.min_item_1a_section_bytes,
                    hi: self.thresholds.max_item_1a_section_bytes,
                },
            },
            ComplianceRule {
                id: UNTRUSTED_PROVIDER_RULE_ID.to_string(),
                framework: SEC_RISK_FRAMEWORK.to_string(),
                field: "source_vendor".to_string(),
                condition: ComplianceCondition::MembershipInVersionedList {
                    list_id: "atelier-sec-risk-untrusted-source-vendors".to_string(),
                    version: SEC_RISK_POLICY_ID.to_string(),
                    members: vec![
                        "stub_sec_edgar".to_string(),
                        "fixture_sec_edgar".to_string(),
                        "unknown".to_string(),
                    ],
                },
            },
            ComplianceRule {
                id: SOURCE_SHAPE_RULE_ID.to_string(),
                framework: SEC_RISK_FRAMEWORK.to_string(),
                field: "source_payload_family".to_string(),
                condition: ComplianceCondition::CrossField {
                    antecedent_field: "source_form_type".to_string(),
                    antecedent_value: "10-K".to_string(),
                    consequent_field: "source_payload_family".to_string(),
                    consequent_value: "embassy.sec_edgar.filing".to_string(),
                },
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annual_report_pack_names_expected_blocking_rule() {
        let rules = SecRiskPolicyPack::annual_report_review().rules();

        assert!(rules.iter().any(|rule| {
            rule.id == HEADING_COUNT_REVIEW_RULE_ID && rule.framework == SEC_RISK_FRAMEWORK
        }));
    }

    #[test]
    fn annual_report_pack_pins_untrusted_source_list_version() {
        let rules = SecRiskPolicyPack::annual_report_review().rules();
        let provider_rule = rules
            .iter()
            .find(|rule| rule.id == UNTRUSTED_PROVIDER_RULE_ID)
            .expect("untrusted provider rule exists");

        let ComplianceCondition::MembershipInVersionedList {
            list_id,
            version,
            members,
        } = &provider_rule.condition
        else {
            panic!("provider rule must be a versioned-list membership rule");
        };
        assert_eq!(list_id, "atelier-sec-risk-untrusted-source-vendors");
        assert_eq!(version, SEC_RISK_POLICY_ID);
        assert!(members.iter().any(|member| member == "stub_sec_edgar"));
    }
}
