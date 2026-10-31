#!/usr/bin/env bash
# Supplier-only artifact comparison; no release or registry-write effects.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo="$(cd "$here/../../.." && pwd -P)"
out="${1:?provide a fresh absolute output directory}"
case "$out" in /*) ;; *) echo 'absolute output path required' >&2; exit 2;; esac
mkdir "$out"
mkdir "$out/baseline" "$out/candidate"
git -C "$repo" archive "$(cat "$here/baseline-revision.txt")" | tar -xf - -C "$out/baseline"
cp -R "$out/baseline/." "$out/candidate/"
patch -d "$out/candidate" -p1 < "$here/inputs.patch"
rustc -vV > "$out/compiler.txt"
for lane in baseline candidate; do
    cargo metadata --manifest-path "$out/$lane/Cargo.toml" --locked --offline --format-version 1 > "$out/$lane-metadata.json"
    for profile in evaluation benchmark-speed; do
        CARGO_TARGET_DIR="$out/$lane-target" cargo build --manifest-path "$out/$lane/Cargo.toml" --locked --offline -p ic-wire-endpoint-fixture --lib --target wasm32-unknown-unknown --profile "$profile"
        cp "$out/$lane-target/wasm32-unknown-unknown/$profile/ic_wire_endpoint_fixture.wasm" "$out/$lane-$profile.wasm"
    done
done
node "$here/summarize.mjs" "$out"
