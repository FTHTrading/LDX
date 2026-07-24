//! LDX color-coded terminal log & status engine.
//!
//! Five-tier status matrix used across every CLI tool, build log, and health check for
//! real-time operational clarity. ANSI codes for terminal output; hex codes for dashboard rendering.

use std::fmt;

/// Five-tier operational status vocabulary used by every LDX component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemStatus {
    /// Fully verified, pass, on-chain execution ready.
    Live,
    /// Deterministic PolicyGuard rule-check in progress or passed.
    PolicyGuard,
    /// Gate lockdown — human review required (2-of-3 vault signature, 15c2-11 gap).
    WarningGate,
    /// Security breach or exception — signature mismatch, non-compliant value movement.
    SecurityReject,
    /// Post-quantum Lamport one-time signature verification.
    QuantumVerified,
}

impl SystemStatus {
    /// Bold ANSI-prefixed tag for terminal output.
    pub fn ansi_prefix(&self) -> &'static str {
        match self {
            SystemStatus::Live => "\x1b[1;32m[LIVE]\x1b[0m",
            SystemStatus::PolicyGuard => "\x1b[1;36m[POLICY]\x1b[0m",
            SystemStatus::WarningGate => "\x1b[1;33m[GATE]\x1b[0m",
            SystemStatus::SecurityReject => "\x1b[1;31m[REJECT]\x1b[0m",
            SystemStatus::QuantumVerified => "\x1b[1;35m[LAMPORT]\x1b[0m",
        }
    }

    /// Hex code for dashboard / SSE / web-rendering.
    pub fn hex_code(&self) -> &'static str {
        match self {
            SystemStatus::Live => "#00FF66",
            SystemStatus::PolicyGuard => "#00E5FF",
            SystemStatus::WarningGate => "#FFD700",
            SystemStatus::SecurityReject => "#FF0055",
            SystemStatus::QuantumVerified => "#7B2CBF",
        }
    }

    /// Short label (no ANSI, no brackets) — for JSON logs.
    pub fn label(&self) -> &'static str {
        match self {
            SystemStatus::Live => "LIVE",
            SystemStatus::PolicyGuard => "POLICY",
            SystemStatus::WarningGate => "GATE",
            SystemStatus::SecurityReject => "REJECT",
            SystemStatus::QuantumVerified => "LAMPORT",
        }
    }
}

impl fmt::Display for SystemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Emit a structured status line to stdout.
///
/// `component` is left-padded to 15 chars for column alignment in operational logs.
pub fn log_status(status: SystemStatus, component: &str, message: &str) {
    println!(
        "{} \x1b[1m{:15}\x1b[0m | {}",
        status.ansi_prefix(),
        component,
        message
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_codes_match_specification() {
        assert_eq!(SystemStatus::Live.hex_code(), "#00FF66");
        assert_eq!(SystemStatus::PolicyGuard.hex_code(), "#00E5FF");
        assert_eq!(SystemStatus::WarningGate.hex_code(), "#FFD700");
        assert_eq!(SystemStatus::SecurityReject.hex_code(), "#FF0055");
        assert_eq!(SystemStatus::QuantumVerified.hex_code(), "#7B2CBF");
    }

    #[test]
    fn ansi_prefixes_terminate_reset() {
        for s in [
            SystemStatus::Live,
            SystemStatus::PolicyGuard,
            SystemStatus::WarningGate,
            SystemStatus::SecurityReject,
            SystemStatus::QuantumVerified,
        ] {
            assert!(
                s.ansi_prefix().ends_with("\x1b[0m"),
                "{s:?} must reset ANSI"
            );
        }
    }

    #[test]
    fn labels_are_uppercase_no_brackets() {
        assert_eq!(SystemStatus::Live.label(), "LIVE");
        assert_eq!(SystemStatus::QuantumVerified.label(), "LAMPORT");
        for s in [
            SystemStatus::PolicyGuard,
            SystemStatus::WarningGate,
            SystemStatus::SecurityReject,
        ] {
            assert!(!s.label().contains('['));
            assert_eq!(s.label(), s.label().to_uppercase());
        }
    }

    #[test]
    fn display_impl_returns_label() {
        assert_eq!(format!("{}", SystemStatus::Live), "LIVE");
        assert_eq!(format!("{}", SystemStatus::QuantumVerified), "LAMPORT");
    }
}
