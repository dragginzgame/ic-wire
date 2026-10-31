#!/usr/bin/env bash
set -euo pipefail
out="${1:?prepared study directory}"
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)"
: "${POCKET_IC_BIN:?explicit instrumented server}"
rustc -vV > "$out/compiler.txt"
sha256sum "$POCKET_IC_BIN" > "$out/server.sha256"
for lane in baseline candidate; do
 for profile in evaluation benchmark-speed; do
  CARGO_TARGET_DIR="$out/$lane-target" cargo build --manifest-path "$out/$lane/Cargo.toml" --locked --offline -p ic-wire-endpoint-fixture --lib --target wasm32-unknown-unknown --profile "$profile"
  mkdir -p "$out/cost-$lane-$profile"
  cp "$out/$lane-target/wasm32-unknown-unknown/$profile/ic_wire_endpoint_fixture.wasm" "$out/cost-$lane-$profile/endpoint.wasm"
  node "$repo/scripts/ci/summarize-endpoint-matrix.mjs" "$out/cost-$lane-$profile" --exports
 done
done
cp -R "$out/candidate" "$out/host"
patch -d "$out/host" -p1 < "$repo/docs/evidence/platform010-tooling071/cost-cases.patch"
export CARGO_TARGET_DIR="$out/host-target"
ENDPOINT_WASM="$out/cost-candidate-evaluation/endpoint.wasm" cargo test --manifest-path "$out/host/Cargo.toml" --locked --offline -p ic-wire-endpoint-fixture --test installed --features ic -- --ignored --nocapture
for profile in evaluation benchmark-speed; do
 for lane in baseline candidate; do
  ENDPOINT_MATRIX_DIR="$out/cost-$lane-$profile" cargo test --manifest-path "$out/host/Cargo.toml" --locked --offline -p ic-wire-endpoint-fixture --test cost-matrix --features ic -- --ignored --test-threads=1 > "$out/cost-$lane-$profile/run.log" 2>&1
 done
done
node "$repo/docs/evidence/platform010-tooling071/analyze.mjs" "$out" "$out"
