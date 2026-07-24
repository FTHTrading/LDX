//! PolicyGuard rejection matrix — every failure mode covered.

use ldx::policy_guard::{PolicyGuardEngine, PolicyRejection, ValueMovementProposal, DEFAULT_CEILING_CENTS};

fn base() -> ValueMovementProposal {
    ValueMovementProposal {
        asset_id: "TEST-001".into(),
        amount_cents: 1_000_00,
        destination_address: "r9xTEST00000000000".into(),
        compliance_cleared: true,
        bitgo_signatures: 2,
        iso20022_clearing_ref: None,
    }
}

#[test]
fn happy_path_approves() {
    assert!(PolicyGuardEngine::evaluate(&base()).is_ok());
}

#[test]
fn compliance_gate_rejects() {
    let mut p = base(); p.compliance_cleared = false;
    assert_eq!(PolicyGuardEngine::evaluate(&p), Err(PolicyRejection::ComplianceFailed));
}

#[test]
fn custody_gate_rejects_below_quorum() {
    let mut p = base(); p.bitgo_signatures = 1;
    assert_eq!(PolicyGuardEngine::evaluate(&p), Err(PolicyRejection::CustodyQuorumInsufficient));
}

#[test]
fn amount_gate_rejects_above_ceiling() {
    let mut p = base(); p.amount_cents = DEFAULT_CEILING_CENTS + 1;
    assert_eq!(PolicyGuardEngine::evaluate(&p), Err(PolicyRejection::AmountExceedsCeiling));
}

#[test]
fn destination_gate_rejects_empty() {
    let mut p = base(); p.destination_address = "".into();
    assert_eq!(PolicyGuardEngine::evaluate(&p), Err(PolicyRejection::DestinationMalformed));
}
