//! # LDX — Lamport Digital Exchange
//!
//! Institutional post-quantum RWA & liquidity engine for LD Capital LLC.
//!
//! ## Modules
//!
//! - [`color_terminal`] — Five-tier ANSI status matrix + structured log output
//! - [`lamport_core`] — Post-quantum one-time signature keypair, sign, verify
//! - [`policy_guard`] — Deterministic value-movement approval state machine
//! - [`bitgo_vault`] — 2-of-3 quorum authorization types with structural safety
//! - [`rwa_pipeline`] — Type-safe RWA state machine (Intake → Diligence → Structuring → Closing → Live)
//! - [`iso20022`] — ISO 20022 pacs.008 and NACHA ACH message types
//! - [`audit`] — SHA-256 hash-chained tamper-evident event log
//!
//! See the [README](https://github.com/FTHTrading/LDX) for architecture, the color
//! matrix, and the five LDX standing rules.

pub mod audit;
pub mod bitgo_vault;
pub mod color_terminal;
pub mod iso20022;
pub mod lamport_core;
pub mod policy_guard;
pub mod rwa_pipeline;

/// Semantic version of the LDX library.
pub const LDX_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Human-readable operator banner.
pub const LDX_BANNER: &str =
    "LDX · Lamport Digital Exchange — LD Capital · Since 1996 · ldrcllc.com";
