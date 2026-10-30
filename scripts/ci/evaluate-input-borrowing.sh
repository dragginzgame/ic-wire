#!/usr/bin/env bash
# Explicit experiment, isolated from concurrent product work. No ordinary CI hook.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
: "${POCKET_IC_BIN:?select a prepared instrumented PocketIC server}"
out="${1:?fresh absolute evidence directory}"
profile="${2:-evaluation}"
entry="${3:-raw}"
case "$out" in /*) ;; *) echo 'output must be absolute' >&2; exit 1;; esac
case "$profile" in evaluation|benchmark-speed) ;; *) exit 1;; esac
case "$entry" in raw|endpoint) ;; *) exit 1;; esac
mkdir "$out"
mkdir "$out/source"
if [[ "$entry" == raw ]]; then
    # Freeze the released runtime, excluding concurrent endpoint implementation work.
    git -C "$root" archive HEAD | tar -x -C "$out/source"
    cp "$root/crates/ic-wire-benchmarks/Cargo.toml" "$out/source/crates/ic-wire-benchmarks/Cargo.toml"
    # The released manifest has no endpoint feature; this runner does not use it.
    sed 's/^endpoint-input = .*/endpoint-input = []/' "$root/crates/ic-wire-benchmarks/Cargo.toml" > "$out/source/crates/ic-wire-benchmarks/Cargo.toml"
    cp -R "$root/crates/ic-wire-benchmarks/examples/input_borrowing" "$out/source/crates/ic-wire-benchmarks/examples/"
else
    # Freeze this working tree, including pending endpoint implementation, once.
    cp "$root/Cargo.toml" "$root/Cargo.lock" "$root/rust-toolchain.toml" "$out/source/"
    cp -R "$root/crates" "$root/examples" "$out/source/"
fi
cp "$0" "$out/runner.sh"
cp "$root/scripts/ci/summarize-input-borrowing.mjs" "$out/summarize.mjs"
printf '%s\n' "$profile" > "$out/profile.txt"
printf '%s\n' "$entry" > "$out/entry.txt"
git -C "$root" rev-parse HEAD > "$out/base-commit.txt"
rustc -vV > "$out/compiler.txt"
sha256sum "$POCKET_IC_BIN" "$out/source/Cargo.lock" "$out/source/crates/ic-wire-benchmarks/examples/input_borrowing/mod.rs" > "$out/hashes.txt"
find "$out/source" -type f -print0 | sort -z | xargs -0 sha256sum > "$out/source-hashes.txt"
export CARGO_TARGET_DIR="$out/target"
cd "$out/source"
cargo test --locked --offline -p ic-wire-benchmarks --example input-borrowing --features ic -- --test-threads=1 > "$out/native.log" 2>&1
base_features=""
if [[ "$entry" == endpoint ]]; then base_features="endpoint-input,"; fi
cargo clippy --locked --offline -p ic-wire-benchmarks --example input-borrowing --target wasm32-unknown-unknown --features "${base_features}borrowed-input" -- -D warnings > "$out/clippy.log" 2>&1
printf 'lane,total_wasm_bytes,code_section_bytes\n' > "$out/wasm.csv"
for lane in owned borrowed; do
    features=(--features "${base_features}")
    if [[ "$lane" == borrowed ]]; then features=(--features "${base_features}borrowed-input"); fi
    cargo build --locked --offline --profile "$profile" -p ic-wire-benchmarks --example input-borrowing --target wasm32-unknown-unknown "${features[@]}" > "$out/$lane-build.log" 2>&1
    artifact="$CARGO_TARGET_DIR/wasm32-unknown-unknown/$profile/examples/input_borrowing.wasm"
    cp "$artifact" "$out/$lane.wasm"
    code="$(cargo run --locked --offline -p ic-wire-benchmarks --example codec-report -- --code-size "$artifact" 2>> "$out/$lane-build.log")"
    printf '%s,%s,%s\n' "$lane" "$(wc -c < "$artifact" | tr -d ' ')" "$code" >> "$out/wasm.csv"
done
IC_WIRE_INPUT_EVIDENCE="$out" cargo test --locked --offline -p ic-wire-benchmarks --example input-borrowing --features ic physical_updates -- --ignored --test-threads=1 > "$out/ic.log" 2>&1
node "$out/summarize.mjs" "$out" > "$out/summary.log"
