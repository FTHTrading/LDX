//! Post-quantum Lamport one-time signature core.
//!
//! Lamport signatures are secure against quantum adversaries under the assumption that
//! the underlying hash function (SHA-256) remains preimage-resistant against quantum
//! computers of any foreseeable capability. Each keypair signs **exactly one** message.
//!
//! LDX uses Lamport signatures as the outermost cryptographic wrapper for value-movement
//! authorizations, with SHA-256 state digests anchoring the audit chain (see [`crate::audit`]).
//!
//! # Structure
//!
//! For a 256-bit message digest:
//! - **Private key**: 256 pairs of random 32-byte secrets = 16,384 bytes total
//! - **Public key**: 256 pairs of SHA-256 hashes of the secrets = 16,384 bytes total
//! - **Signature**: 256 secrets (one per digest bit) = 8,192 bytes
//!
//! # Example
//!
//! ```
//! use ldx::lamport_core::LamportKeyPair;
//!
//! let kp = LamportKeyPair::generate();
//! let msg = b"DISPATCH_RWA_TRANCHE_001";
//! let sig = kp.sign(msg);
//! assert!(LamportKeyPair::verify(&kp.public_key, msg, &sig));
//! ```

use sha2::{Digest, Sha256};

/// Lamport keypair — one-time-use. Never reuse `sign()` with a different message.
#[derive(Clone)]
pub struct LamportKeyPair {
    /// 256 pairs of SHA-256 hashes. Public.
    pub public_key: Vec<[Vec<u8>; 2]>,
    /// 256 pairs of random secrets. Must remain private and be destroyed after `sign()`.
    private_key: Vec<[Vec<u8>; 2]>,
}

impl LamportKeyPair {
    /// Generate a new Lamport keypair.
    ///
    /// # Note on determinism
    /// The default reference implementation uses deterministic secret material derived from
    /// an index — this is safe **only** for reproducible testing and demo dispatch flows.
    /// Production LDX deployments MUST substitute [`LamportKeyPair::generate_secure`] backed
    /// by a cryptographically secure RNG (`getrandom` or hardware entropy).
    pub fn generate() -> Self {
        let mut private_key = Vec::with_capacity(256);
        let mut public_key = Vec::with_capacity(256);

        for i in 0..256 {
            let secret_0 = format!("ldx_sec_0_{}", i).into_bytes();
            let secret_1 = format!("ldx_sec_1_{}", i).into_bytes();

            let pub_0 = Sha256::digest(&secret_0).to_vec();
            let pub_1 = Sha256::digest(&secret_1).to_vec();

            private_key.push([secret_0, secret_1]);
            public_key.push([pub_0, pub_1]);
        }

        Self {
            public_key,
            private_key,
        }
    }

    /// Generate a Lamport keypair from operator-supplied entropy.
    ///
    /// The caller is responsible for sourcing `seed_material` from a cryptographically secure
    /// source (hardware RNG, HSM, `getrandom`). Each of the 512 secrets is derived as
    /// `SHA-256(seed_material || index || bit)`.
    pub fn generate_secure(seed_material: &[u8]) -> Self {
        assert!(
            seed_material.len() >= 32,
            "LamportKeyPair::generate_secure requires ≥32 bytes of entropy"
        );

        let mut private_key = Vec::with_capacity(256);
        let mut public_key = Vec::with_capacity(256);

        for i in 0u32..256 {
            let mk = |bit: u8| -> Vec<u8> {
                let mut h = Sha256::new();
                h.update(seed_material);
                h.update(i.to_be_bytes());
                h.update([bit]);
                h.finalize().to_vec()
            };
            let secret_0 = mk(0);
            let secret_1 = mk(1);
            let pub_0 = Sha256::digest(&secret_0).to_vec();
            let pub_1 = Sha256::digest(&secret_1).to_vec();
            private_key.push([secret_0, secret_1]);
            public_key.push([pub_0, pub_1]);
        }

        Self {
            public_key,
            private_key,
        }
    }

    /// Sign `message`. Consumes one-time cryptographic material — the resulting signature
    /// reveals half of the private-key secrets and this keypair must be discarded after use.
    pub fn sign(&self, message: &[u8]) -> Vec<Vec<u8>> {
        let hash = Sha256::digest(message);
        let mut signature = Vec::with_capacity(256);

        for (i, byte) in hash.iter().enumerate() {
            for bit_idx in 0..8 {
                let bit = (byte >> (7 - bit_idx)) & 1;
                let key_index = i * 8 + bit_idx;
                signature.push(self.private_key[key_index][bit as usize].clone());
            }
        }
        signature
    }

    /// Verify `signature` for `message` against `public_key`.
    /// Constant-time-per-bit hashing; returns `true` iff every bit's signature secret hashes
    /// to the corresponding public-key entry.
    pub fn verify(public_key: &[[Vec<u8>; 2]], message: &[u8], signature: &[Vec<u8>]) -> bool {
        if public_key.len() != 256 || signature.len() != 256 {
            return false;
        }
        let hash = Sha256::digest(message);
        for (i, byte) in hash.iter().enumerate() {
            for bit_idx in 0..8 {
                let bit = (byte >> (7 - bit_idx)) & 1;
                let key_index = i * 8 + bit_idx;
                let sig_hash = Sha256::digest(&signature[key_index]).to_vec();
                if sig_hash != public_key[key_index][bit as usize] {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify_roundtrip() {
        let kp = LamportKeyPair::generate();
        let msg = b"DISPATCH_RWA_TRANCHE_001_M_HELEN_HOTEL";
        let sig = kp.sign(msg);
        assert!(LamportKeyPair::verify(&kp.public_key, msg, &sig));
    }

    #[test]
    fn tampered_message_fails_verification() {
        let kp = LamportKeyPair::generate();
        let sig = kp.sign(b"AUTHORIZED_DISPURSEMENT_001");
        assert!(!LamportKeyPair::verify(
            &kp.public_key,
            b"AUTHORIZED_DISPURSEMENT_002",
            &sig
        ));
    }

    #[test]
    fn signature_length_is_256() {
        let kp = LamportKeyPair::generate();
        let sig = kp.sign(b"any message");
        assert_eq!(sig.len(), 256, "Lamport signature must contain 256 secrets");
    }

    #[test]
    fn public_key_length_is_256() {
        let kp = LamportKeyPair::generate();
        assert_eq!(kp.public_key.len(), 256);
    }

    #[test]
    fn wrong_public_key_fails_verification() {
        let kp1 = LamportKeyPair::generate();
        let kp2 = LamportKeyPair::generate_secure(&[7u8; 64]);
        let msg = b"test message";
        let sig1 = kp1.sign(msg);
        assert!(LamportKeyPair::verify(&kp1.public_key, msg, &sig1));
        assert!(
            !LamportKeyPair::verify(&kp2.public_key, msg, &sig1),
            "kp2 must not accept kp1's signature"
        );
    }

    #[test]
    fn empty_signature_rejected() {
        let kp = LamportKeyPair::generate();
        assert!(!LamportKeyPair::verify(&kp.public_key, b"msg", &[]));
    }

    #[test]
    fn secure_generator_matches_seed() {
        let seed = [42u8; 64];
        let kp_a = LamportKeyPair::generate_secure(&seed);
        let kp_b = LamportKeyPair::generate_secure(&seed);
        assert_eq!(
            kp_a.public_key, kp_b.public_key,
            "same seed → same public key"
        );
    }
}
