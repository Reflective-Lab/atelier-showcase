//! High-risk claim portfolio — product-side view of which Arbiter claims have
//! which evidence today.
//!
//! The point is model adequacy and evidence discipline: only selected claims
//! deserve Cedar/SymCC or CVC5, and the expense claim is the worked exemplar,
//! not the whole portfolio.

use arbiter::CedarAnalysisQuery;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EvidenceTier {
    RuntimeTests,
    PropertyTests,
    ReviewFixtures,
    CedarSymccCandidate,
    CedarSymccExemplar,
}

impl EvidenceTier {
    const fn label(self) -> &'static str {
        match self {
            Self::RuntimeTests => "runtime tests",
            Self::PropertyTests => "property tests",
            Self::ReviewFixtures => "review fixtures",
            Self::CedarSymccCandidate => "Cedar/SymCC candidate",
            Self::CedarSymccExemplar => "Cedar/SymCC exemplar",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cvc5Policy {
    NotUseful,
    Optional,
    NightlyOnly,
}

impl Cvc5Policy {
    const fn label(self) -> &'static str {
        match self {
            Self::NotUseful => "not useful",
            Self::Optional => "optional",
            Self::NightlyOnly => "nightly-only",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Claim {
    id: &'static str,
    statement: &'static str,
    owner: &'static str,
    first_evidence: EvidenceTier,
    cvc5_policy: Cvc5Policy,
}

fn portfolio() -> Vec<Claim> {
    vec![
        Claim {
            id: "expense.non_finance_commit.high_value",
            statement: "Non-finance supervisory principals cannot commit high-value expenses even with approval.",
            owner: "expense approval",
            first_evidence: EvidenceTier::CedarSymccExemplar,
            cvc5_policy: Cvc5Policy::NightlyOnly,
        },
        Claim {
            id: "hitl.no_escalation_when_approval_still_denied",
            statement: "A denied request escalates only when the approved version would be allowed.",
            owner: "HITL gate",
            first_evidence: EvidenceTier::PropertyTests,
            cvc5_policy: Cvc5Policy::Optional,
        },
        Claim {
            id: "vendor_selection.due_diligence_required",
            statement: "Vendor commit requires due diligence and competitive review gates.",
            owner: "vendor selection",
            first_evidence: EvidenceTier::ReviewFixtures,
            cvc5_policy: Cvc5Policy::Optional,
        },
        Claim {
            id: "delegation.amount_cap_enforced",
            statement: "Delegation tokens cannot authorize spend above their amount cap.",
            owner: "delegation",
            first_evidence: EvidenceTier::PropertyTests,
            cvc5_policy: Cvc5Policy::NotUseful,
        },
        Claim {
            id: "flow.phase_promotion.requires_gates",
            statement: "Promotion or commit cannot cross a phase boundary until required gates pass.",
            owner: "flow governance",
            first_evidence: EvidenceTier::CedarSymccCandidate,
            cvc5_policy: Cvc5Policy::Optional,
        },
        Claim {
            id: "data_classification.pii_blocks_external_move",
            statement: "PII detected in a proposal creates a blocking constraint before external movement.",
            owner: "data classification",
            first_evidence: EvidenceTier::RuntimeTests,
            cvc5_policy: Cvc5Policy::NotUseful,
        },
    ]
}

fn main() {
    println!("High-risk Arbiter claim portfolio\n");
    for claim in portfolio() {
        println!("{} [{}]", claim.id, claim.owner);
        println!("  {}", claim.statement);
        println!("  evidence: {}", claim.first_evidence.label());
        println!("  CVC5:     {}", claim.cvc5_policy.label());
        println!();
    }

    let expense_claim_is_reviewable = CedarAnalysisQuery::ExpenseNonFinanceHighValueCommitDenied
        .claim_policy_source()
        .is_some();
    println!("expense Cedar claim policy exposed: {expense_claim_is_reviewable}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portfolio_contains_the_initial_claim_set() {
        let claims = portfolio();
        let ids: Vec<&str> = claims.iter().map(|claim| claim.id).collect();

        assert!(ids.contains(&"expense.non_finance_commit.high_value"));
        assert!(ids.contains(&"hitl.no_escalation_when_approval_still_denied"));
        assert!(ids.contains(&"vendor_selection.due_diligence_required"));
        assert!(ids.contains(&"delegation.amount_cap_enforced"));
        assert!(ids.contains(&"flow.phase_promotion.requires_gates"));
        assert!(ids.contains(&"data_classification.pii_blocks_external_move"));
    }

    #[test]
    fn only_the_expense_claim_is_the_symcc_exemplar_today() {
        let exemplar_ids: Vec<&str> = portfolio()
            .iter()
            .filter(|claim| claim.first_evidence == EvidenceTier::CedarSymccExemplar)
            .map(|claim| claim.id)
            .collect();

        assert_eq!(exemplar_ids, vec!["expense.non_finance_commit.high_value"]);
    }

    #[test]
    fn cvc5_is_not_marked_useful_for_non_policy_symbolic_claims() {
        let claims = portfolio();
        let delegation = claims
            .iter()
            .find(|claim| claim.id == "delegation.amount_cap_enforced")
            .expect("delegation cap claim should be present");
        let data = claims
            .iter()
            .find(|claim| claim.id == "data_classification.pii_blocks_external_move")
            .expect("data classification claim should be present");

        assert_eq!(delegation.cvc5_policy, Cvc5Policy::NotUseful);
        assert_eq!(data.cvc5_policy, Cvc5Policy::NotUseful);
    }
}
