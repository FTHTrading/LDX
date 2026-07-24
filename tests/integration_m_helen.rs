//! Full M Helen deal-flow integration.
//!
//! Walks the flagship M Helen Hotel deal from seeded Diligence through Structuring →
//! Closing → Live once all real-world blockers are cleared. Also asserts that the
//! type system refuses to skip stages.

use ldx::audit::AuditChain;
use ldx::bitgo_vault::{QuorumAuth, SignerRole, VaultSignature};
use ldx::lamport_core::LamportKeyPair;
use ldx::policy_guard::{PolicyGuardEngine, ValueMovementProposal};
use ldx::rwa_pipeline::{seed_m_helen, PipelineStage, StageError};

#[test]
fn m_helen_full_lifecycle() {
    let mut audit = AuditChain::new();

    // Seed
    let mut deal = seed_m_helen();
    assert_eq!(deal.stage, PipelineStage::Diligence);
    assert_eq!(deal.open_blockers().len(), 2);
    audit.append("test", "deal.seeded", Some(deal.deal_id.clone()), None);

    // Cannot jump to Live
    assert!(matches!(
        deal.advance_to(PipelineStage::Live),
        Err(StageError::NonAdjacentTransition { .. }),
    ));

    // Cannot even advance one step with blockers open
    assert!(matches!(
        deal.advance_to(PipelineStage::Structuring),
        Err(StageError::BlockersRemaining { .. }),
    ));

    // Clear both blockers (representing real-world reconciliation + 15c2-11 completion)
    assert!(deal.clear_blocker("budget_discrepancy_4_96M"));
    assert!(deal.clear_blocker("disclosure_15c2_11_gap"));

    // Walk the pipeline
    for target in [
        PipelineStage::Structuring,
        PipelineStage::Closing,
        PipelineStage::Live,
        PipelineStage::Servicing,
    ] {
        assert_eq!(deal.advance_to(target).unwrap(), target);
        audit.append("test", "stage.advanced", Some(target.label().into()), None);
    }
    assert_eq!(deal.stage, PipelineStage::Servicing);

    // Audit chain intact through the whole flow
    assert!(audit.verify().is_ok());
    assert_eq!(audit.len(), 5); // seed + 4 advancements
}

#[test]
fn m_helen_dispatch_end_to_end() {
    let kp = LamportKeyPair::generate();
    let msg = b"DISPATCH_RWA_TRANCHE_001_M_HELEN_HOTEL";
    let sig = kp.sign(msg);
    assert!(LamportKeyPair::verify(&kp.public_key, msg, &sig));

    let quorum = QuorumAuth::new(vec![
        VaultSignature {
            role: SignerRole::LDCapital,
            signature_bytes: vec![0xAA; 64],
        },
        VaultSignature {
            role: SignerRole::BitGoTrust,
            signature_bytes: vec![0xBB; 64],
        },
    ])
    .expect("standard LDCapital + BitGo quorum");
    assert_eq!(quorum.signer_count(), 2);

    let proposal = ValueMovementProposal {
        asset_id: "RWA-M-HELEN-001".into(),
        amount_cents: 496_000_000,
        destination_address: "r9xLDXEngineVault110293".into(),
        compliance_cleared: true,
        bitgo_signatures: quorum.signer_count() as u8,
        iso20022_clearing_ref: None,
    };
    assert!(PolicyGuardEngine::evaluate(&proposal).is_ok());
}
