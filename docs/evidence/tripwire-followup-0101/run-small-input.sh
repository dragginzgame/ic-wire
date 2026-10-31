#!/usr/bin/env bash
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
out="${1:?prepared study directory}"
cp -R "$out/input" "$out/stack-input"
patch -d "$out/stack-input" -p1 < "$here/small-input.patch"
for profile in evaluation benchmark-speed; do
 CARGO_TARGET_DIR="$out/stack-input-target" cargo build --manifest-path "$out/stack-input/Cargo.toml" --locked --offline --profile "$profile" -p ic-wire-benchmarks --example selective-input --target wasm32-unknown-unknown --features stack-input,borrowed-input
 cp "$out/stack-input-target/wasm32-unknown-unknown/$profile/examples/selective_input.wasm" "$out/stack-input-$profile.wasm"
done
