//! 2-of-3 non-custodial vault quorum types.
//!
//! LDX operates a strictly **non-custodial** custody arrangement with BitGo Trust Company, N.A.
//! (OCC-chartered national trust bank). The vault type system enforces a critical invariant:
//! **no valid authorization can be constructed where BitGo alone signs.**
//!
//! Three signer roles: `LDCapital` (operator), `BitGoTrust` (custodian), `RecoveryKey` (offline).
//! A quorum requires two distinct roles. `BitGoTrust` alone cannot form a quorum.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The three signer roles that participate in the LDX vault quorum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SignerRole {
    /// LD Capital operator key (online, held by authorized officers under dual control).
    LDCapital,
    /// BitGo Trust Company, N.A. (custodian, OCC-chartered national trust bank).
    BitGoTrust,
    /// Cold recovery key (offline, held under LD Capital corporate governance).
    RecoveryKey,
}

impl SignerRole {
    pub fn label(&self) -> &'static str {
        match self {
            SignerRole::LDCapital => "LD_CAPITAL",
            SignerRole::BitGoTrust => "BITGO_TRUST",
            SignerRole::RecoveryKey => "RECOVERY_KEY",
        }
    }
}

/// A collected signature from a specific signer role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultSignature {
    pub role: SignerRole,
    /// The signature bytes (opaque to LDX — verified by the counterparty custody service).
    pub signature_bytes: Vec<u8>,
}

/// A quorum authorization. Construct only via [`QuorumAuth::new`] to enforce the invariant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumAuth {
    signatures: Vec<VaultSignature>,
}

/// Errors returned when a proposed quorum fails to satisfy the LDX invariant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuorumError {
    /// Fewer than 2 signatures were provided.
    InsufficientCount(usize),
    /// Two signatures were provided but both were from the same role — quorum requires
    /// distinct roles.
    DuplicateRole(SignerRole),
    /// The signature set consisted solely of `BitGoTrust` (with or without duplicates).
    /// LDX **never** permits BitGo to be the only signer role even at count ≥ 2.
    BitGoAloneForbidden,
}

impl QuorumAuth {
    /// Attempt to construct a valid quorum from a set of collected signatures.
    ///
    /// Enforces:
    /// 1. At least 2 signatures present.
    /// 2. Signatures come from **at least two distinct** signer roles.
    /// 3. `BitGoTrust` alone (however many copies) does not constitute a quorum.
    pub fn new(signatures: Vec<VaultSignature>) -> Result<Self, QuorumError> {
        if signatures.len() < 2 {
            return Err(QuorumError::InsufficientCount(signatures.len()));
        }

        let roles: BTreeSet<SignerRole> = signatures.iter().map(|s| s.role).collect();

        if roles.len() < 2 {
            let only = *roles.iter().next().expect("checked len >= 1");
            if only == SignerRole::BitGoTrust {
                return Err(QuorumError::BitGoAloneForbidden);
            }
            return Err(QuorumError::DuplicateRole(only));
        }

        if roles == [SignerRole::BitGoTrust].into_iter().collect() {
            return Err(QuorumError::BitGoAloneForbidden);
        }

        Ok(Self { signatures })
    }

    pub fn signatures(&self) -> &[VaultSignature] {
        &self.signatures
    }

    pub fn signer_count(&self) -> usize {
        let roles: BTreeSet<_> = self.signatures.iter().map(|s| s.role).collect();
        roles.len()
    }

    /// Returns the distinct roles that contributed to this quorum, sorted.
    pub fn signer_roles(&self) -> Vec<SignerRole> {
        let roles: BTreeSet<_> = self.signatures.iter().map(|s| s.role).collect();
        roles.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(role: SignerRole) -> VaultSignature {
        VaultSignature { role, signature_bytes: vec![0u8; 64] }
    }

    #[test]
    fn accepts_ldcapital_plus_bitgo() {
        let q = QuorumAuth::new(vec![sig(SignerRole::LDCapital), sig(SignerRole::BitGoTrust)]);
        assert!(q.is_ok());
        assert_eq!(q.unwrap().signer_count(), 2);
    }

    #[test]
    fn accepts_ldcapital_plus_recovery() {
        assert!(QuorumAuth::new(vec![
            sig(SignerRole::LDCapital),
            sig(SignerRole::RecoveryKey),
        ]).is_ok());
    }

    #[test]
    fn accepts_bitgo_plus_recovery() {
        assert!(QuorumAuth::new(vec![
            sig(SignerRole::BitGoTrust),
            sig(SignerRole::RecoveryKey),
        ]).is_ok());
    }

    #[test]
    fn accepts_all_three() {
        let q = QuorumAuth::new(vec![
            sig(SignerRole::LDCapital),
            sig(SignerRole::BitGoTrust),
            sig(SignerRole::RecoveryKey),
        ])
        .expect("full 3-of-3 quorum valid");
        assert_eq!(q.signer_count(), 3);
    }

    #[test]
    fn rejects_single_signature() {
        let r = QuorumAuth::new(vec![sig(SignerRole::LDCapital)]);
        assert!(matches!(r, Err(QuorumError::InsufficientCount(1))));
    }

    #[test]
    fn rejects_bitgo_alone_forbidden() {
        // Two copies of BitGoTrust signature — must not form a quorum
        let r = QuorumAuth::new(vec![
            sig(SignerRole::BitGoTrust),
            sig(SignerRole::BitGoTrust),
        ]);
        assert!(matches!(r, Err(QuorumError::BitGoAloneForbidden)));
    }

    #[test]
    fn rejects_duplicate_role_non_bitgo() {
        let r = QuorumAuth::new(vec![
            sig(SignerRole::LDCapital),
            sig(SignerRole::LDCapital),
        ]);
        assert!(matches!(r, Err(QuorumError::DuplicateRole(SignerRole::LDCapital))));
    }

    #[test]
    fn signer_roles_are_sorted_and_unique() {
        let q = QuorumAuth::new(vec![
            sig(SignerRole::RecoveryKey),
            sig(SignerRole::LDCapital),
            sig(SignerRole::BitGoTrust),
        ]).unwrap();
        assert_eq!(q.signer_roles(), vec![
            SignerRole::LDCapital,
            SignerRole::BitGoTrust,
            SignerRole::RecoveryKey,
        ]);
    }
}
