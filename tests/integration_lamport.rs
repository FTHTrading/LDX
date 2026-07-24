//! End-to-end Lamport signature integration.

use ldx::lamport_core::LamportKeyPair;

#[test]
fn lamport_signs_and_verifies_across_module_boundary() {
    let kp = LamportKeyPair::generate();
    let msg = b"cross-module integration test";
    let sig = kp.sign(msg);
    assert!(LamportKeyPair::verify(&kp.public_key, msg, &sig));
}

#[test]
fn lamport_rejects_swapped_signature_bit() {
    let kp = LamportKeyPair::generate();
    let msg = b"cross-module integration test";
    let mut sig = kp.sign(msg);
    // Corrupt a random signature secret
    sig[42] = vec![0u8; 32];
    assert!(!LamportKeyPair::verify(&kp.public_key, msg, &sig));
}
