//! Example: demonstrates 2-of-3 quorum invariants — accepts valid combinations and
//! rejects BitGo-alone.
//!
//! `cargo run --example bitgo_quorum_demo`

use ldx::bitgo_vault::{QuorumAuth, QuorumError, SignerRole, VaultSignature};
use ldx::color_terminal::{log_status, SystemStatus};

fn sig(role: SignerRole) -> VaultSignature {
    VaultSignature { role, signature_bytes: vec![0u8; 64] }
}

fn try_quorum(label: &str, sigs: Vec<VaultSignature>) {
    match QuorumAuth::new(sigs) {
        Ok(q) => log_status(
            SystemStatus::Live,
            "VAULT_OK",
            &format!(
                "{label}: quorum authorized · {} distinct roles",
                q.signer_count()
            ),
        ),
        Err(QuorumError::BitGoAloneForbidden) => log_status(
            SystemStatus::SecurityReject,
            "VAULT_DENY",
            &format!("{label}: BitGo alone forbidden — structural invariant"),
        ),
        Err(e) => log_status(
            SystemStatus::WarningGate,
            "VAULT_GATE",
            &format!("{label}: {e:?}"),
        ),
    }
}

fn main() {
    println!("\x1b[1;36m─── LDX Example · BitGo 2-of-3 Vault Quorum Invariants ───\x1b[0m\n");

    try_quorum("LDCapital + BitGo",         vec![sig(SignerRole::LDCapital), sig(SignerRole::BitGoTrust)]);
    try_quorum("LDCapital + Recovery",      vec![sig(SignerRole::LDCapital), sig(SignerRole::RecoveryKey)]);
    try_quorum("BitGo + Recovery",          vec![sig(SignerRole::BitGoTrust), sig(SignerRole::RecoveryKey)]);
    try_quorum("All three",                 vec![sig(SignerRole::LDCapital), sig(SignerRole::BitGoTrust), sig(SignerRole::RecoveryKey)]);
    println!();
    try_quorum("Only LDCapital",            vec![sig(SignerRole::LDCapital)]);
    try_quorum("Only BitGo",                vec![sig(SignerRole::BitGoTrust)]);
    try_quorum("Two BitGo copies",          vec![sig(SignerRole::BitGoTrust), sig(SignerRole::BitGoTrust)]);
    try_quorum("Two LDCapital copies",      vec![sig(SignerRole::LDCapital), sig(SignerRole::LDCapital)]);
}
