#!/usr/bin/env bash
# LDX — first-push initialization script.
# Only run once, after Niraj Sheth has reviewed the initial commit.
set -euo pipefail

echo -e "\x1b[1;36m[+] Initializing local LDX repository...\x1b[0m"

git init -b main
git add README.md LICENSE .gitignore Cargo.toml src/ tests/ examples/ scripts/ .github/

git commit -m "feat(ldx): initialize institutional post-quantum core engine

- Lamport one-time signature core (SHA-256 anchored)
- PolicyGuard deterministic value-movement adjudication (5 gates)
- 2-of-3 non-custodial vault quorum types (BitGo-alone forbidden by construction)
- RWA pipeline state machine with M Helen flagship seed data
- ISO 20022 pacs.008 + NACHA ACH message types
- SHA-256 hash-chained tamper-evident audit log
- Five-tier color status matrix (ANSI + hex)
- Full test coverage on every rejection path
- CI workflow: fmt · clippy · build · test · doc"

git remote add origin https://github.com/FTHTrading/LDX.git
echo -e "\x1b[1;33m[+] Pushing to FTHTrading/LDX main branch...\x1b[0m"
git push -u origin main

echo -e "\x1b[1;35m[✔] LDX repository initialized and pushed.\x1b[0m"
