#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
cd "$root"
[[ "$(node --version)" == "v$(cat examples/browser/.nvmrc)" ]] || { echo 'Select the Node version in examples/browser/.nvmrc' >&2; exit 1; }
cargo test --locked -p ic-wire --features typescript --test runtime --test borrowed --test native
cargo clippy --locked -p ic-wire --features typescript --all-targets -- -D warnings
RUSTUP_TOOLCHAIN=1.91.0 cargo check --locked -p ic-wire --features typescript --lib
out="$root/target/typescript-conformance"
mkdir -p "$out"
generation_log="$out/generation.log"
generation_status=0
cargo run --locked -p ic-wire --features typescript --example typescript-conformance -- "$out" 2> "$generation_log" || generation_status=$?
# These names belong to the timed synthetic benchmark, not contract fixtures.
# Check before replaying stderr so a regression cannot flood the CI console.
if grep -Eq '^warning: /types/Page[0-9]+/' "$generation_log"; then
    echo "Synthetic benchmark diagnostics leaked; generation log: $generation_log" >&2
    exit 1
fi
cat "$generation_log" >&2
[[ "$generation_status" == 0 ]] || exit "$generation_status"
"$root/examples/browser/node_modules/.bin/tsc" --ignoreConfig --strict --noUncheckedIndexedAccess --exactOptionalPropertyTypes --target ES2022 --module NodeNext --moduleResolution NodeNext --skipLibCheck --outDir "$out/compiled" "$out"/*.ts
node "$out/checks.mjs"
node crates/ic-wire/examples/typescript_conformance/endian.mjs "$out/compiled/wire-runtime.js"
bash "$root/scripts/ci/check-schema-targets.sh"

bash "$root/scripts/ci/check-bulk-advice.sh"
bash "$root/scripts/ci/check-platform-types.sh"
bash "$root/scripts/ci/check-stable-identities.sh"
