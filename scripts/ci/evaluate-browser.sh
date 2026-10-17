#!/usr/bin/env bash
set -euo pipefail
# Explicit local browser experiment. Dependency/browser installation is separate.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
: "${POCKET_IC_BIN:?Set POCKET_IC_BIN to a prepared PocketIC server executable}"
cd "$root"
[[ "$(node --version)" == "v$(cat examples/browser/.nvmrc)" ]] || { echo 'Select the Node version in examples/browser/.nvmrc' >&2; exit 1; }
[[ "npm@$(npm --version)" == "$(node -p 'JSON.parse(require("fs").readFileSync("examples/browser/package.json", "utf8")).packageManager')" ]] || { echo 'Select the npm version in package.json' >&2; exit 1; }
export CARGO_TARGET_DIR="$root/target"
cargo run --quiet --locked -p ic-wire-canister-evaluation --example build-evaluation -- browser "$root/target/browser-evaluation"
