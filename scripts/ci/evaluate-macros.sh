#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
: "${POCKET_IC_BIN:?Set POCKET_IC_BIN to a prepared PocketIC server executable}"
cd "$root"
export CARGO_TARGET_DIR="$root/target"
output="$root/target/macro-adoption-evidence"
cargo run --quiet --locked -p ic-wire-canister-evaluation --example build-evaluation -- macros "$output"
