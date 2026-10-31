#!/usr/bin/env bash
# Focused supplier comparison using the existing endpoint probe and analysis.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo="$(cd "$here/../../.." && pwd -P)"
out="${1:?fresh absolute output directory}"
case "$out" in /*) ;; *) exit 1;; esac
: "${POCKET_IC_BIN:?explicit prepared instrumented server}"
mkdir "$out"
mkdir "$out/baseline" "$out/candidate"
git -C "$repo" archive "$(cat "$here/baseline-revision.txt")" | tar -xf - -C "$out/baseline"
cp -R "$out/baseline/." "$out/candidate/"
patch -d "$out/candidate" -p1 < "$here/inputs.patch"
rustc -vV > "$out/compiler.txt"
cmp "$here/compiler.txt" "$out/compiler.txt"
sha256sum "$POCKET_IC_BIN" > "$out/server.sha256"
for lane in baseline candidate; do
 for profile in evaluation benchmark-speed; do
  CARGO_TARGET_DIR="$out/$lane-target" cargo build --manifest-path "$out/$lane/Cargo.toml" --locked --offline -p ic-wire-endpoint-fixture --lib --target wasm32-unknown-unknown --profile "$profile"
  mkdir "$out/cost-$lane-$profile"
  cp "$out/$lane-target/wasm32-unknown-unknown/$profile/ic_wire_endpoint_fixture.wasm" "$out/cost-$lane-$profile/endpoint.wasm"
  node "$out/candidate/scripts/ci/summarize-endpoint-matrix.mjs" "$out/cost-$lane-$profile" --exports
 done
done
cp -R "$out/candidate" "$out/host"
(cd "$out/host" && patch -p1 < "$repo/docs/evidence/platform010-tooling071/cost-cases.patch")
export CARGO_TARGET_DIR="$out/host-target"
ENDPOINT_WASM="$out/cost-candidate-evaluation/endpoint.wasm" cargo test --manifest-path "$out/host/Cargo.toml" --locked --offline -p ic-wire-endpoint-fixture --test installed --features ic -- --ignored --nocapture
for profile in evaluation benchmark-speed; do
 for lane in baseline candidate; do
  ENDPOINT_MATRIX_DIR="$out/cost-$lane-$profile" cargo test --manifest-path "$out/host/Cargo.toml" --locked --offline -p ic-wire-endpoint-fixture --test cost-matrix --features ic -- --ignored --test-threads=1 > "$out/cost-$lane-$profile/run.log" 2>&1
 done
done
node "$repo/docs/evidence/platform010-tooling071/analyze.mjs" "$out" "$out"
