//! ISO 20022 and NACHA banking-rail message types.
//!
//! Minimal, correct-by-construction message types for pacs.008 (SEPA/FedWire cross-border
//! credit transfer) and NACHA ACH batch entries. Fields carry only what LDX needs to
//! correlate a fiat rail movement with an RWA on-chain event and to satisfy the informational
//! disclosure requirements around funding flows.

use serde::{Deserialize, Serialize};

/// ISO 20022 pacs.008 — customer credit transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pacs008 {
    /// Message identification, unique per submission.
    pub msg_id: String,
    /// Creation date-time (RFC 3339).
    pub created_at: String,
    /// Debtor account (IBAN or synthetic-IBAN).
    pub debtor_account: String,
    /// Creditor account (IBAN or synthetic-IBAN).
    pub creditor_account: String,
    /// Amount in cents.
    pub amount_cents: u64,
    /// Currency (ISO 4217, e.g. `USD`).
    pub currency: String,
    /// End-to-end identifier — correlates fiat movement with an LDX RWA event.
    pub end_to_end_id: String,
    /// Optional structured remittance information (payment reference).
    pub remittance_ref: Option<String>,
}

impl Pacs008 {
    pub fn new(
        msg_id: impl Into<String>,
        debtor_account: impl Into<String>,
        creditor_account: impl Into<String>,
        amount_cents: u64,
        currency: impl Into<String>,
        end_to_end_id: impl Into<String>,
    ) -> Self {
        Self {
            msg_id: msg_id.into(),
            created_at: chrono_like_now(),
            debtor_account: debtor_account.into(),
            creditor_account: creditor_account.into(),
            amount_cents,
            currency: currency.into(),
            end_to_end_id: end_to_end_id.into(),
            remittance_ref: None,
        }
    }

    pub fn with_remittance(mut self, r: impl Into<String>) -> Self {
        self.remittance_ref = Some(r.into());
        self
    }

    pub fn amount_usd(&self) -> f64 {
        self.amount_cents as f64 / 100.0
    }
}

/// A single NACHA ACH entry within a batch. Simplified for LDX correlation purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NachaEntry {
    /// Standard Entry Class code (`PPD`, `CCD`, `WEB`, etc.).
    pub sec_code: String,
    /// Transaction code (22 = credit checking, 27 = debit checking, etc.).
    pub transaction_code: u8,
    /// Receiving DFI routing number (9 digits).
    pub receiving_dfi_routing: String,
    /// Receiving account number.
    pub receiving_account: String,
    /// Amount in cents.
    pub amount_cents: u64,
    /// Individual identification number.
    pub individual_id: String,
    /// Trace number (unique within batch).
    pub trace_number: String,
}

impl NachaEntry {
    pub fn amount_usd(&self) -> f64 {
        self.amount_cents as f64 / 100.0
    }
}

/// Tiny stand-in for a chrono `Utc::now()` — this crate stays dependency-lean; substitute
/// with a proper timestamp source in production integration wrapping.
fn chrono_like_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("epoch:{secs}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pacs008_builds_and_reports_usd_amount() {
        let m = Pacs008::new(
            "MSG-001",
            "US33LDXX0000000000000001",
            "US33BITG0000000000000001",
            25_000_000_00,
            "USD",
            "E2E-M-HELEN-DRAW-001",
        )
        .with_remittance("M Helen Hotel — draw #1");
        assert_eq!(m.amount_usd(), 25_000_000.0);
        assert_eq!(m.currency, "USD");
        assert_eq!(m.remittance_ref.as_deref(), Some("M Helen Hotel — draw #1"));
    }

    #[test]
    fn nacha_entry_serializes_to_json() {
        let e = NachaEntry {
            sec_code: "CCD".into(),
            transaction_code: 22,
            receiving_dfi_routing: "061000104".into(),
            receiving_account: "0000000001".into(),
            amount_cents: 4_960_000_00,
            individual_id: "M-HELEN-001".into(),
            trace_number: "061000104000001".into(),
        };
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("\"sec_code\":\"CCD\""));
        assert!(json.contains("\"amount_cents\":496000000"));
    }
}
