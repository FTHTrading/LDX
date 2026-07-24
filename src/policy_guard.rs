//! PolicyGuard — model proposes, deterministic rules dispose.
//!
//! The AI orchestration layer proposes value movements. PolicyGuard evaluates each proposal
//! against deterministic Rust rules that cannot be bypassed by any model output, prompt, or
//! configuration change short of a code deployment reviewed under LD Capital governance.

use crate::color_terminal::{log_status, SystemStatus};

/// A proposed value movement awaiting PolicyGuard adjudication.
#[derive(Debug, Clone)]
pub struct ValueMovementProposal {
    /// Canonical asset identifier (RWA tranche ID, wallet ID, or SPV entity).
    pub asset_id: String,
    /// Movement amount in the asset's smallest unit (USD → cents).
    pub amount_cents: u64,
    /// Destination address (rXRPL, GStellar, 0xEVM, or synthetic-IBAN string).
    pub destination_address: String,
    /// Whether the KYC/AML state machine has cleared this counterparty.
    pub compliance_cleared: bool,
    /// Count of independent BitGo/Anchorage vault signatures collected (0..=3).
    pub bitgo_signatures: u8,
    /// Optional ISO 20022 pacs.008 clearing reference for correlated bank-rail flows.
    pub iso20022_clearing_ref: Option<String>,
}

/// Reasons a proposal may be rejected. Every rejection surface is enumerated and testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyRejection {
    /// KYC/AML compliance state did not clear.
    ComplianceFailed,
    /// Fewer than 2 vault signatures were present. Structural non-custodial invariant.
    CustodyQuorumInsufficient,
    /// Amount exceeds the operator-configured single-transaction ceiling.
    AmountExceedsCeiling,
    /// Destination address failed format sanity checks.
    DestinationMalformed,
}

impl PolicyRejection {
    pub fn as_str(&self) -> &'static str {
        match self {
            PolicyRejection::ComplianceFailed => "Compliance check failed",
            PolicyRejection::CustodyQuorumInsufficient => "Custody quorum failure (2-of-3 minimum)",
            PolicyRejection::AmountExceedsCeiling => "Amount exceeds single-transaction ceiling",
            PolicyRejection::DestinationMalformed => "Destination address failed sanity check",
        }
    }
}

/// Deterministic value-movement approval engine.
pub struct PolicyGuardEngine;

/// Default single-transaction ceiling in cents. Operator can override with a config layer;
/// this constant guards against runaway approvals in the default profile.
///
/// Value: $25,000,000.00 (twenty-five million dollars). Matches LDX operator practice
/// for hospitality construction-draw disbursements; larger movements require explicit
/// operator configuration and secondary human approval outside PolicyGuard.
pub const DEFAULT_CEILING_CENTS: u64 = 2_500_000_000;

impl PolicyGuardEngine {
    /// Evaluate a proposal. Returns `Ok(())` when every gate passes; `Err(reason)` otherwise.
    /// All decisions emit a structured status line to stdout via [`log_status`].
    pub fn evaluate(proposal: &ValueMovementProposal) -> Result<(), PolicyRejection> {
        log_status(
            SystemStatus::PolicyGuard,
            "POLICY_GUARD",
            &format!("Evaluating proposal for Asset: {}", proposal.asset_id),
        );

        // Gate 1 — KYC/AML clearance
        if !proposal.compliance_cleared {
            log_status(
                SystemStatus::SecurityReject,
                "COMPLIANCE_GATE",
                "Value movement rejected: KYC/AML state unverified.",
            );
            return Err(PolicyRejection::ComplianceFailed);
        }

        // Gate 2 — 2-of-3 quorum
        if proposal.bitgo_signatures < 2 {
            log_status(
                SystemStatus::WarningGate,
                "CUSTODY_GATE",
                "Quorum failure: Minimum 2-of-3 signatures required.",
            );
            return Err(PolicyRejection::CustodyQuorumInsufficient);
        }

        // Gate 3 — Amount ceiling
        if proposal.amount_cents > DEFAULT_CEILING_CENTS {
            log_status(
                SystemStatus::WarningGate,
                "AMOUNT_GATE",
                &format!(
                    "Amount ${:.2} exceeds default single-tx ceiling ${:.2}",
                    proposal.amount_cents as f64 / 100.0,
                    DEFAULT_CEILING_CENTS as f64 / 100.0,
                ),
            );
            return Err(PolicyRejection::AmountExceedsCeiling);
        }

        // Gate 4 — Destination sanity (rXRPL / GStellar / 0xEVM / IBAN)
        if !is_plausible_destination(&proposal.destination_address) {
            log_status(
                SystemStatus::SecurityReject,
                "ROUTING_GATE",
                "Destination address failed sanity check.",
            );
            return Err(PolicyRejection::DestinationMalformed);
        }

        // All gates passed
        log_status(
            SystemStatus::Live,
            "EXECUTION_RAILS",
            "Proposal approved. Dispatching to settlement layer.",
        );
        Ok(())
    }
}

/// Minimal sanity check on destination address format. Blocks obviously empty / short entries.
fn is_plausible_destination(addr: &str) -> bool {
    let t = addr.trim();
    if t.len() < 8 {
        return false;
    }
    // XRPL classic (r...), Stellar (G...), EVM (0x...), synthetic-IBAN (starts with letter)
    let head = t.chars().next().unwrap_or(' ');
    head.is_ascii_alphabetic() || t.starts_with("0x")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_proposal() -> ValueMovementProposal {
        ValueMovementProposal {
            asset_id: "RWA-M-HELEN-001".into(),
            amount_cents: 496_000_000,
            destination_address: "r9xLDXEngineVault110293".into(),
            compliance_cleared: true,
            bitgo_signatures: 2,
            iso20022_clearing_ref: None,
        }
    }

    #[test]
    fn approves_well_formed_proposal() {
        assert!(PolicyGuardEngine::evaluate(&base_proposal()).is_ok());
    }

    #[test]
    fn rejects_uncleared_compliance() {
        let mut p = base_proposal();
        p.compliance_cleared = false;
        assert_eq!(
            PolicyGuardEngine::evaluate(&p),
            Err(PolicyRejection::ComplianceFailed)
        );
    }

    #[test]
    fn rejects_insufficient_quorum() {
        let mut p = base_proposal();
        p.bitgo_signatures = 1;
        assert_eq!(
            PolicyGuardEngine::evaluate(&p),
            Err(PolicyRejection::CustodyQuorumInsufficient)
        );
    }

    #[test]
    fn rejects_amount_over_ceiling() {
        let mut p = base_proposal();
        p.amount_cents = DEFAULT_CEILING_CENTS + 1;
        assert_eq!(
            PolicyGuardEngine::evaluate(&p),
            Err(PolicyRejection::AmountExceedsCeiling)
        );
    }

    #[test]
    fn rejects_malformed_destination() {
        let mut p = base_proposal();
        p.destination_address = "".into();
        assert_eq!(
            PolicyGuardEngine::evaluate(&p),
            Err(PolicyRejection::DestinationMalformed)
        );
    }

    #[test]
    fn accepts_evm_destination() {
        let mut p = base_proposal();
        p.destination_address = "0xAbCd1234000000000000000000000000000000AA".into();
        assert!(PolicyGuardEngine::evaluate(&p).is_ok());
    }

    #[test]
    fn ceiling_boundary_is_inclusive() {
        let mut p = base_proposal();
        p.amount_cents = DEFAULT_CEILING_CENTS;
        assert!(
            PolicyGuardEngine::evaluate(&p).is_ok(),
            "exactly at ceiling should pass"
        );
    }

    #[test]
    fn three_signatures_accepted() {
        let mut p = base_proposal();
        p.bitgo_signatures = 3;
        assert!(PolicyGuardEngine::evaluate(&p).is_ok());
    }
}
