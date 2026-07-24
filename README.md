# LDX — Institutional RWA & Liquidity Engine

[![Build Status](https://img.shields.io/badge/tests-passing-00FF66?style=for-the-badge&logo=rust)](https://github.com/FTHTrading/LDX)
[![Crypto](https://img.shields.io/badge/crypto-Lamport%20Post--Quantum-7B2CBF?style=for-the-badge)](https://github.com/FTHTrading/LDX)
[![License](https://img.shields.io/badge/license-Proprietary-00E5FF?style=for-the-badge)](./LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
[![Site](https://img.shields.io/badge/site-fthtrading.github.io%2FLDX-2E568F?style=for-the-badge&logo=github)](https://fthtrading.github.io/LDX/)

> **📖 Public site & learning center:** https://fthtrading.github.io/LDX/  
> Client status page · client letter · interactive Lexicon Academy · outreach templates · brand vault.

**LDX (Lamport Digital Exchange)** is a zero-hallucination, deterministic clearing engine for
tokenized Real-World Assets (RWAs), energy tax-equity flips, and institutional private credit.

LDX is the execution and settlement platform that sits underneath the lending operations of
[LD Capital LLC](https://ldrcllc.com) — a 29-year Atlanta-based commercial finance firm with
more than **$4.5 billion** in commercial real estate loans originated, underwritten, closed, and
serviced since 1996.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            LDX CORE ENGINE                              │
├─────────────────────────────────────────────────────────────────────────┤
│  [Ingestion & Compliance]    [Orchestration Engine]   [Execution Rails] │
│   • KYC / AML State Machine   • PolicyGuard Rules      • Post-Quantum   │
│   • ISO 20022 / NACHA         • Tranche Waterfalls     • XRPL / Stellar │
│   • IPFS Merkle Manifests     • 15c2-11 ATS Audit      • 2-of-3 BitGo   │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Key Features

- **Post-Quantum Core** — Lamport one-time signature schemes anchored by SHA-256 state digests.
- **PolicyGuard Architecture** — AI model _proposes_; deterministic Rust state machines _dispose_.
  Nothing moves value without passing every gate.
- **2-of-3 Vault Governance** — Structural non-custodial quorum. LDX cannot construct a valid
  value-movement authorization where BitGo Trust Company, N.A. alone can sign.
- **RWA Pipeline** — 15c2-11 ATS compliance discipline for commercial real estate and infrastructure
  tranches.
- **Institutional Rails** — ISO 20022 (pacs.008), NACHA ACH, and OCC-chartered national trust bank
  safekeeping arrangement types.
- **Hash-Chained Audit** — Tamper-evident event log with genesis-anchored SHA-256 chain.
- **Color-Coded Terminal** — Five-tier ANSI status matrix for real-time operational clarity.

---

## Color Status Matrix

| ANSI Tag              | Domain                                  | Meaning                                        |
| --------------------- | --------------------------------------- | ---------------------------------------------- |
| `[LIVE]`      🟢       | Operational rails & consensus           | Verified, pass, on-chain execution ready       |
| `[POLICY]`    🔵       | PolicyGuard & compliance state          | Deterministic rule-check in progress or passed |
| `[GATE]`      🟡       | Gate lockdown / human review            | Action required (2-of-3 sig, 15c2-11 gap)      |
| `[REJECT]`    🔴       | Security breach or exception            | Signature mismatch, non-compliant movement     |
| `[LAMPORT]`   🟣       | Post-quantum cryptography               | Lamport one-time signature verification        |

---

## Quick Start

```bash
# Build production release
cargo build --release

# Run test suite
cargo test --workspace

# Run the reference dispatch flow
cargo run --release

# Examples
cargo run --example m_helen_dispatch
cargo run --example bitgo_quorum_demo
```

---

## Modules

| Module              | Purpose                                                                          |
| ------------------- | -------------------------------------------------------------------------------- |
| `color_terminal`    | ANSI/hex status matrix and structured log output                                 |
| `lamport_core`      | Post-quantum one-time signature keypair, sign, verify                            |
| `policy_guard`      | Deterministic value-movement approval state machine                              |
| `bitgo_vault`       | 2-of-3 quorum authorization types with structural safety                         |
| `rwa_pipeline`      | Type-safe RWA state machine — Intake → Diligence → Structuring → Closing → Live  |
| `iso20022`          | ISO 20022 pacs.008 and NACHA ACH message types                                   |
| `audit`             | SHA-256 hash-chained tamper-evident event log                                    |

---

## LDX Standing Rules (Enforced in Docs, Code, and Communications)

1. No crypto / token / blockchain vocabulary in client-facing surfaces. Use "execution platform,"
   "settlement infrastructure," "institutional safekeeping," "custody arrangement."
2. No solicitation of investment. Any private program is described only as "by invitation, through
   official offering documents when released."
3. Only verified claims — $4.5B+ (per LD Realty Capital records), founded 1996, four active
   divisions, OCC-chartered national trust bank custody arrangement executed.
4. Every outbound piece carries the informational-purposes disclaimer.
5. Nothing goes out before Niraj Sheth approves final copy.

---

## About LD Capital

Founded in 1996 by Niraj "Nick" Sheth. Four active divisions: LD Realty Capital · LD Small Business
Finance / The Loan Depot Lending Co. · LD Residential Mortgage LLC · LD FinMave · LD Capital
Bridge to USA (EB-5) · LD Capital Leasing. Flagship sponsor project: a 90-key SpringHill Suites by
Marriott + waterpark + EV in Helen, Georgia (M Helen Hotel LLC · GA #24229189).

**LD Capital LLC** · Atlanta, Georgia · +1 (770) 272-2232 · [ldrcllc.com](https://ldrcllc.com)

---

## Legal

This repository documents institutional infrastructure for LD Capital LLC. It is not an offer to
sell or a solicitation of an offer to buy any security or investment product. Any future private
investment program will be made exclusively through official offering documents to investors who
meet applicable eligibility and verification requirements. All loans subject to underwriting and
credit approval. Historical volume figures per LD Realty Capital company records.

Copyright © 2026 LD Capital LLC. All rights reserved.
