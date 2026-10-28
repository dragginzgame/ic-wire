#!/usr/bin/env bash
# Focused explicit study; never part of ordinary correctness/release validation.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
cd "$root"
: "${POCKET_IC_BIN:?select a prepared PocketIC server}"
if [[ -n "${IC_TESTKIT_POCKET_IC_URL+x}" ]]; then
    echo 'unset IC_TESTKIT_POCKET_IC_URL; this study owns its prepared local server' >&2
    exit 1
fi
out="${1:?new evidence directory}"
baseline="${2:-c748379325266d7fa614fefee9d745eb4a88acca}"
baseline="$(git rev-parse "${baseline}^{commit}")"
case "$out" in /*) ;; *) out="$root/$out" ;; esac
mkdir "$out"
mkdir "$out/source"
cp Cargo.toml Cargo.lock rust-toolchain.toml "$out/source/"
cp -R crates "$out/source/"
cp "$root/scripts/ci/evaluate-caller-storage.sh" "$out/runner.sh"
cp -R "$out/source" "$out/baseline"
git archive "$baseline" crates/ic-wire crates/ic-wire-derive | tar -x -C "$out/baseline"
printf '%s\n' "$baseline" > "$out/baseline-commit.txt"
rustc -vV > "$out/compiler.txt"
git rev-parse HEAD > "$out/base-commit.txt"
git diff > "$out/dirty.patch"
shasum -a 256 Cargo.lock "$POCKET_IC_BIN" crates/ic-wire-benchmarks/examples/caller_storage/mod.rs > "$out/hashes.txt"
printf 'profile,lane,total_wasm_bytes,code_section_bytes\n' > "$out/wasm.csv"
for profile in evaluation benchmark-speed; do
    for lane in before-vec after-vec after-slice; do
        features=()
        if [[ "$lane" == after-slice ]]; then features=(--features caller-slice); fi
        if [[ "$lane" == before-vec ]]; then
            CARGO_TARGET_DIR="$out/baseline-target" cargo build --locked --offline \
                --manifest-path "$out/baseline/Cargo.toml" --profile "$profile" \
                -p ic-wire-benchmarks --example caller-storage --target wasm32-unknown-unknown \
                > "$out/$profile-$lane-build.log" 2>&1
            artifact="$out/baseline-target/wasm32-unknown-unknown/$profile/examples/caller_storage.wasm"
        else
            cargo build --locked --offline --profile "$profile" -p ic-wire-benchmarks \
            --example caller-storage --target wasm32-unknown-unknown "${features[@]}" \
            > "$out/$profile-$lane-build.log" 2>&1
            artifact="${CARGO_TARGET_DIR:-$root/target}/wasm32-unknown-unknown/$profile/examples/caller_storage.wasm"
        fi
        cp "$artifact" "$out/$profile-$lane.wasm"
        bytes="$(wc -c < "$artifact" | tr -d ' ')"
        code="$(cargo run --locked --offline -p ic-wire-benchmarks --example codec-report -- --code-size "$artifact" 2>> "$out/$profile-$lane-build.log")"
        printf '%s,%s,%s,%s\n' "$profile" "$lane" "$bytes" "$code" >> "$out/wasm.csv"
    done
    mkdir "$out/$profile-native"
    for mode in time allocations heap; do
        features=()
        if [[ "$mode" != time ]]; then features=(--features "$mode"); fi
        IC_WIRE_STORAGE_EVIDENCE="$out/$profile-native" cargo test --locked --offline \
            --profile "$profile" -p ic-wire-benchmarks --example caller-storage "${features[@]}" \
            native_measurement -- --ignored --test-threads=1 \
            > "$out/$profile-native-$mode.log" 2>&1
    done
done
IC_WIRE_STORAGE_EVIDENCE="$out" cargo test --locked --offline -p ic-wire-benchmarks \
    --example caller-storage --features ic ic_measurement -- --ignored --test-threads=1 \
    > "$out/ic.log" 2>&1
printf 'Evidence: %s\n' "$out"
