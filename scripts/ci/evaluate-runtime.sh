#!/usr/bin/env bash
set -euo pipefail
# Local installed runtime comparison; uses only a prepared PocketIC server.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
: "${POCKET_IC_BIN:?Set POCKET_IC_BIN to a prepared PocketIC server executable}"
cd "$root"
export CARGO_TARGET_DIR="$root/target"
output="$root/target/runtime-comparison"
cargo run --quiet --locked -p ic-wire-canister-evaluation --example build-evaluation -- runtime "$output"
