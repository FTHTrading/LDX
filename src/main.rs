//! LDX Core System — reference dispatch flow.
//!
//! Demonstrates end-to-end operation: Lamport signature verification, PolicyGuard evaluation,
//! 2-of-3 vault quorum construction, RWA pipeline stage transitions, and hash-chained audit
//! log emission. Prints color-coded status lines per the LDX five-tier matrix.

use ldx::audit::AuditChain;
use ldx::bitgo_vault::{QuorumAuth, SignerRole, VaultSignature};
use ldx::color_terminal::{log_status, SystemStatus};
use ldx::lamport_core::LamportKeyPair;
use ldx::policy_guard::{PolicyGuardEngine, ValueMovementProposal};
use ldx::rwa_pipeline::{seed_m_helen, PipelineStage};
use ldx::{LDX_BANNER, LDX_VERSION};

fn main() {
    println!("\x1b[1;36m========================================================\x1b[0m");
    println!("\x1b[1;36m       LDX: LAMPORT DIGITAL EXCHANGE CORE SYSTEM       \x1b[0m");
    println!("\x1b[1;36m========================================================\x1b[0m");
    println!("\x1b[2m       {LDX_BANNER}\x1b[0m");
    println!("\x1b[2m       version {LDX_VERSION}\x1b[0m\n");

    let mut audit = AuditChain::new();

    // ── Post-Quantum: Lamport keypair, sign, verify ─────────────────────────
    log_status(SystemStatus::QuantumVerified, "LAMPORT_CORE", "Initializing Post-Quantum KeyPair...");
    let keypair = LamportKeyPair::generate();
    let message = b"DISPATCH_RWA_TRANCHE_001_M_HELEN_HOTEL";
    let sig = keypair.sign(message);
    let verified = LamportKeyPair::verify(&keypair.public_key, message, &sig);

    if verified {
        log_status(SystemStatus::QuantumVerified, "LAMPORT_CORE", "Message signature valid.");
        audit.append("lamport", "signature.verified", Some("RWA-M-HELEN-001".into()), None);
    } else {
        log_status(SystemStatus::SecurityReject, "LAMPORT_CORE", "Invalid quantum signature!");
        return;
    }

    // ── RWA Pipeline: seed M Helen and inspect stage/blockers ────────────────
    let mut deal = seed_m_helen();
    log_status(
        SystemStatus::PolicyGuard,
        "RWA_PIPELINE",
        &format!(
            "Seeded M Helen deal — stage: {}, open blockers: {}",
            deal.stage.label(),
            deal.open_blockers().len()
        ),
    );
    for b in deal.open_blockers() {
        log_status(SystemStatus::WarningGate, "BLOCKER", &format!("{} — {}", b.name, b.description));
    }
    audit.append("rwa_pipeline", "deal.seeded", Some(deal.deal_id.clone()), None);

    // Attempt an early stage jump — must fail (demonstrates type safety)
    match deal.advance_to(PipelineStage::Live) {
        Ok(_) => log_status(SystemStatus::SecurityReject, "RWA_PIPELINE", "UNEXPECTED: skipped stages"),
        Err(e) => log_status(
            SystemStatus::WarningGate,
            "RWA_PIPELINE",
            &format!("Stage jump refused as designed: {e:?}"),
        ),
    }

    // ── 2-of-3 Vault Quorum: construct + enforce non-BitGo-alone invariant ──
    let quorum = QuorumAuth::new(vec![
        VaultSignature { role: SignerRole::LDCapital, signature_bytes: vec![0xAA; 64] },
        VaultSignature { role: SignerRole::BitGoTrust, signature_bytes: vec![0xBB; 64] },
    ]);
    match quorum {
        Ok(q) => log_status(
            SystemStatus::Live,
            "VAULT_QUORUM",
            &format!(
                "Quorum authorized — {} distinct signer roles: {:?}",
                q.signer_count(),
                q.signer_roles().iter().map(|r| r.label()).collect::<Vec<_>>()
            ),
        ),
        Err(e) => log_status(SystemStatus::SecurityReject, "VAULT_QUORUM", &format!("Quorum invalid: {e:?}")),
    }
    audit.append("vault", "quorum.authorized", None, None);

    // ── PolicyGuard: evaluate the value movement ─────────────────────────────
    let proposal = ValueMovementProposal {
        asset_id: "RWA-M-HELEN-001".to_string(),
        amount_cents: 4_960_000_00,
        destination_address: "r9xLDXEngineVault110293".to_string(),
        compliance_cleared: true,
        bitgo_signatures: 2,
        iso20022_clearing_ref: None,
    };

    match PolicyGuardEngine::evaluate(&proposal) {
        Ok(_) => {
            log_status(SystemStatus::Live, "SYSTEM_READY", "LDX Execution Engine active.");
            audit.append("policy_guard", "dispatch.approved", Some(proposal.asset_id.clone()), None);
        }
        Err(e) => {
            log_status(SystemStatus::SecurityReject, "SYSTEM_HALT", e.as_str());
            audit.append("policy_guard", "dispatch.rejected", Some(proposal.asset_id.clone()), None);
        }
    }

    // ── Audit chain summary ──────────────────────────────────────────────────
    let head = audit.verify().unwrap_or_else(|seq| format!("BROKEN@{seq}"));
    log_status(
        SystemStatus::PolicyGuard,
        "AUDIT_CHAIN",
        &format!("{} entries · head: {}", audit.len(), &head[..16.min(head.len())]),
    );
}
