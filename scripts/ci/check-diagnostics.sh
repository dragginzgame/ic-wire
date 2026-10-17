#!/usr/bin/env bash
set -euo pipefail
# Compile explicit cases against the exact locked Cargo-built libraries.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
cd "$root"
case "${1:-}" in
    wire) source_dir=crates/ic-wire/tests/ui ;;
    macro) source_dir=crates/ic-wire-canister-evaluation/tests/macro_ui ;;
    *) printf 'Usage: check-diagnostics.sh wire|macro\n' >&2; exit 1 ;;
esac
export RUSTUP_AUTO_INSTALL=0
output="$root/target/ui-diagnostics"
mkdir -p "$output"
run="$(mktemp -d "$output/$1.XXXXXX")"
printf 'Diagnostic logs: %s\n' "$run"
status=0
trap 'status=$?; if [[ "$status" != 0 ]]; then printf "Diagnostic failure logs: %s\n" "$run" >&2; fi' EXIT
cargo build --offline --locked -p ic-wire -p ic-wire-derive \
    --features ic-wire-derive/adapter-fixture --message-format=json > "$run/artifacts.jsonl"
jq="$root/.tools/host/bin/jq"
wire="$($jq -r 'select(.reason == "compiler-artifact" and .target.name == "ic_wire") | .filenames[] | select(endswith(".rlib"))' "$run/artifacts.jsonl")"
derive="$($jq -r 'select(.reason == "compiler-artifact" and .target.name == "ic_wire_derive") | .filenames[] | select(endswith(".so") or endswith(".dylib") or endswith(".dll"))' "$run/artifacts.jsonl")"
[[ -f "$wire" && -f "$derive" ]] || { printf 'Missing host library artifacts\n' >&2; exit 1; }
source_file="$root/$source_dir/mod.rs"
check() {
    rustc --edition=2024 --crate-type=lib --emit=metadata --crate-name ui_contract \
        --extern "ic_wire=$wire" --extern "ic_wire_derive=$derive" \
        -L "dependency=$(dirname "$wire")/deps" --cfg "ui_case=\"$1\"" \
        "$source_file" -o "$run/$1.rmeta" > "$run/$1.log" 2>&1
}
# A passing control prevents missing toolchains/dependencies from passing negatives.
check valid
for expected_file in "$root/$source_dir/"*/expected.txt; do
    name="$(basename "$(dirname "$expected_file")")"
    if check "$name"; then printf 'Expected compile failure: %s\n' "$name" >&2; exit 1; fi
    if ! grep -F -f "$expected_file" "$run/$name.log" >/dev/null; then
        cat "$run/$name.log" >&2
        exit 1
    fi
    printf 'Expected diagnostic passed: %s\n' "$name"
done
