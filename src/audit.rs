//! SHA-256 hash-chained tamper-evident event log.
//!
//! Every LDX event is appended to an in-memory chain: `entry_hash = SHA-256(prev_hash || canonical(entry))`.
//! Verification walks the chain forward and recomputes every hash; any mismatch identifies the
//! precise entry at which tampering occurred.
//!
//! Genesis anchor: 64 hex-zero characters.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// A single audit entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEntry {
    pub seq: u64,
    pub timestamp_epoch_secs: u64,
    pub actor: String,
    pub action: String,
    pub target: Option<String>,
    pub payload_json: Option<String>,
    pub prev_hash: String,
    pub entry_hash: String,
}

/// The tamper-evident chain.
#[derive(Debug, Default)]
pub struct AuditChain {
    entries: Vec<AuditEntry>,
}

impl AuditChain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// Append a new entry. Returns a reference to the appended entry.
    pub fn append(
        &mut self,
        actor: impl Into<String>,
        action: impl Into<String>,
        target: Option<String>,
        payload_json: Option<String>,
    ) -> &AuditEntry {
        let seq = self.entries.len() as u64 + 1;
        let prev_hash = self
            .entries
            .last()
            .map(|e| e.entry_hash.clone())
            .unwrap_or_else(|| GENESIS_HASH.to_string());
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let actor_s = actor.into();
        let action_s = action.into();
        let canonical = canonicalize(&Canonical {
            seq,
            timestamp_epoch_secs: ts,
            actor: &actor_s,
            action: &action_s,
            target: target.as_deref(),
            payload_json: payload_json.as_deref(),
        });

        let mut h = Sha256::new();
        h.update(prev_hash.as_bytes());
        h.update(canonical.as_bytes());
        let entry_hash = hex(h.finalize().as_slice());

        self.entries.push(AuditEntry {
            seq,
            timestamp_epoch_secs: ts,
            actor: actor_s,
            action: action_s,
            target,
            payload_json,
            prev_hash,
            entry_hash,
        });
        self.entries.last().expect("just pushed")
    }

    /// Verify the entire chain from genesis. Returns `Ok(head_hash)` if intact; else `Err(seq)`
    /// where `seq` is the entry at which the chain broke.
    pub fn verify(&self) -> Result<String, u64> {
        let mut expected_prev = GENESIS_HASH.to_string();
        for e in &self.entries {
            if e.prev_hash != expected_prev {
                return Err(e.seq);
            }
            let canonical = canonicalize(&Canonical {
                seq: e.seq,
                timestamp_epoch_secs: e.timestamp_epoch_secs,
                actor: &e.actor,
                action: &e.action,
                target: e.target.as_deref(),
                payload_json: e.payload_json.as_deref(),
            });
            let mut h = Sha256::new();
            h.update(e.prev_hash.as_bytes());
            h.update(canonical.as_bytes());
            let recomputed = hex(h.finalize().as_slice());
            if recomputed != e.entry_hash {
                return Err(e.seq);
            }
            expected_prev = e.entry_hash.clone();
        }
        Ok(expected_prev)
    }

    pub fn head_hash(&self) -> String {
        self.entries
            .last()
            .map(|e| e.entry_hash.clone())
            .unwrap_or_else(|| GENESIS_HASH.to_string())
    }
}

struct Canonical<'a> {
    seq: u64,
    timestamp_epoch_secs: u64,
    actor: &'a str,
    action: &'a str,
    target: Option<&'a str>,
    payload_json: Option<&'a str>,
}

/// Deterministic serialization for hash input. Field order fixed by construction.
fn canonicalize(c: &Canonical) -> String {
    format!(
        r#"{{"action":"{}","actor":"{}","payload_json":{},"seq":{},"target":{},"timestamp_epoch_secs":{}}}"#,
        c.action,
        c.actor,
        c.payload_json
            .map(quote_json_str)
            .unwrap_or_else(|| "null".into()),
        c.seq,
        c.target
            .map(quote_json_str)
            .unwrap_or_else(|| "null".into()),
        c.timestamp_epoch_secs,
    )
}

fn quote_json_str(s: &str) -> String {
    let escaped: String = s
        .chars()
        .map(|c| match c {
            '\\' => "\\\\".to_string(),
            '"' => "\\\"".to_string(),
            c if (c as u32) < 0x20 => format!("\\u{:04x}", c as u32),
            c => c.to_string(),
        })
        .collect();
    format!("\"{escaped}\"")
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_chain_verifies() {
        let c = AuditChain::new();
        assert_eq!(c.verify(), Ok(GENESIS_HASH.to_string()));
    }

    #[test]
    fn appends_form_valid_chain() {
        let mut c = AuditChain::new();
        c.append(
            "operator",
            "deal.opened",
            Some("RWA-M-HELEN-001".into()),
            None,
        );
        c.append(
            "policy_guard",
            "dispatch.approved",
            Some("RWA-M-HELEN-001".into()),
            None,
        );
        c.append("bitgo", "vault.arrangement.executed", None, None);
        assert_eq!(c.len(), 3);
        assert!(c.verify().is_ok());
    }

    #[test]
    fn tampered_entry_detected() {
        let mut c = AuditChain::new();
        c.append(
            "operator",
            "deal.opened",
            Some("RWA-M-HELEN-001".into()),
            None,
        );
        c.append("operator", "diligence.started", None, None);
        // Mutate an entry in place
        c.entries[0].action = "diligence.completed".to_string();
        let r = c.verify();
        assert_eq!(
            r,
            Err(1),
            "chain break must be detected at the tampered entry"
        );
    }

    #[test]
    fn head_hash_updates_with_appends() {
        let mut c = AuditChain::new();
        assert_eq!(c.head_hash(), GENESIS_HASH);
        c.append("a", "b", None, None);
        assert_ne!(c.head_hash(), GENESIS_HASH);
    }
}
