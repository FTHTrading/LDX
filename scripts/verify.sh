#!/usr/bin/env bash
# LDX — full pre-push verification suite. Matches the CI workflow.
set -euo pipefail

echo "═══════════════════════════════════════════════════"
echo "    LDX · Local Verification Suite"
echo "═══════════════════════════════════════════════════"

echo -e "\n\x1b[1;36m[1/5] fmt --check\x1b[0m"
cargo fmt --all -- --check

echo -e "\n\x1b[1;36m[2/5] clippy -D warnings\x1b[0m"
cargo clippy --all-targets --all-features -- -D warnings

echo -e "\n\x1b[1;36m[3/5] build --release\x1b[0m"
cargo build --release

echo -e "\n\x1b[1;36m[4/5] test --workspace\x1b[0m"
cargo test --workspace

echo -e "\n\x1b[1;36m[5/5] doc --no-deps\x1b[0m"
cargo doc --no-deps --workspace

echo -e "\n\x1b[1;32m[✔] All checks passed. LDX ready for push.\x1b[0m"
