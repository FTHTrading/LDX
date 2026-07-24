//! Example: full M Helen dispatch flow with structured status output.
//!
//! `cargo run --example m_helen_dispatch`

use ldx::audit::AuditChain;
use ldx::bitgo_vault::{QuorumAuth, SignerRole, VaultSignature};
use ldx::color_terminal::{log_status, SystemStatus};
use ldx::lamport_core::LamportKeyPair;
use ldx::policy_guard::{PolicyGuardEngine, ValueMovementProposal};
use ldx::rwa_pipeline::seed_m_helen;

fn main() {
    println!("\x1b[1;36m─── LDX Example · M Helen Dispatch Flow ───────────────\x1b[0m\n");

    let mut audit = AuditChain::new();

    // 1. Lamport signature
    log_status(SystemStatus::QuantumVerified, "LAMPORT", "Generating post-quantum keypair...");
    let kp = LamportKeyPair::generate();
    let sig = kp.sign(b"M_HELEN_DRAW_001");
    assert!(LamportKeyPair::verify(&kp.public_key, b"M_HELEN_DRAW_001", &sig));
    log_status(SystemStatus::QuantumVerified, "LAMPORT", "Draw #1 signature verified.");
    audit.append("lamport", "sig.verified", Some("M_HELEN_DRAW_001".into()), None);

    // 2. Seed the deal
    let deal = seed_m_helen();
    log_status(
        SystemStatus::PolicyGuard,
        "PIPELINE",
        &format!("{} @ {}", deal.name, deal.stage.label()),
    );
    for b in deal.open_blockers() {
        log_status(SystemStatus::WarningGate, "BLOCKER", &b.name);
    }

    // 3. Assemble quorum
    let quorum = QuorumAuth::new(vec![
        VaultSignature { role: SignerRole::LDCapital, signature_bytes: vec![1u8; 64] },
        VaultSignature { role: SignerRole::BitGoTrust, signature_bytes: vec![2u8; 64] },
    ])
    .expect("standard 2-of-3");
    log_status(SystemStatus::Live, "VAULT", &format!("{} signer roles authorized", quorum.signer_count()));
    audit.append("vault", "quorum.assembled", None, None);

    // 4. Evaluate proposal
    let p = ValueMovementProposal {
        asset_id: deal.deal_id.clone(),
        amount_cents: 250_000_00, // $250K draw
        destination_address: "r9xContractorEscrow".into(),
        compliance_cleared: true,
        bitgo_signatures: quorum.signer_count() as u8,
        iso20022_clearing_ref: Some("ISO-M-HELEN-DRAW-001".into()),
    };
    match PolicyGuardEngine::evaluate(&p) {
        Ok(_)  => audit.append("policy_guard", "approved", Some(p.asset_id.clone()), None),
        Err(e) => audit.append("policy_guard", &format!("rejected:{}", e.as_str()), Some(p.asset_id.clone()), None),
    };

    // 5. Audit summary
    let head = audit.verify().expect("chain intact");
    log_status(
        SystemStatus::PolicyGuard,
        "AUDIT",
        &format!("{} entries · head {}", audit.len(), &head[..16]),
    );
}
