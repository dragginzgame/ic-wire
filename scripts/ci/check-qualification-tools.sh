#!/usr/bin/env bash
# Explicit qualification features own native fuzzing and allocator profilers.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
cd "$root"
export RUSTUP_AUTO_INSTALL=0
out="${CARGO_TARGET_DIR:-$root/target}/qualification-tools"
mkdir -p "$out"
host="$(rustc -vV | sed -n 's/^host: //p')"
[[ -n "$host" ]]
graph() {
    local label="$1" package="$2" target="$3"
    shift 3
    cargo tree --locked --offline -p "$package" --target "$target" \
        --edges normal,build,dev --prefix none "$@" > "$out/$label.txt"
}
reject() {
    if grep -Eq "^($2) v" "$out/$1.txt"; then
        printf 'Unexpected qualification tool in %s\n' "$1" >&2
        exit 1
    fi
}
graph codec ic-wire "$host"
graph benchmark ic-wire-benchmarks "$host" --features wire,candid
graph qualification ic-wire-qualification "$host" --features allocation-fixture
for label in codec benchmark qualification; do
    reject "$label" 'libfuzzer-sys|dhat|stats_alloc'
done
graph fuzz ic-wire-qualification "$host" --features fuzz-runtime
grep -Eq '^libfuzzer-sys v' "$out/fuzz.txt"
reject fuzz 'dhat|stats_alloc'
for mode in allocations heap; do
    graph "$mode" ic-wire-benchmarks "$host" --features "wire,candid,$mode"
    if [[ "$mode" == allocations ]]; then
        grep -Eq '^stats_alloc v' "$out/$mode.txt"
        reject "$mode" 'libfuzzer-sys|dhat'
    else
        grep -Eq '^dhat v' "$out/$mode.txt"
        reject "$mode" 'libfuzzer-sys|stats_alloc'
    fi
    graph "$mode-wasm" ic-wire-benchmarks wasm32-unknown-unknown --features "wire,candid,$mode"
    reject "$mode-wasm" 'libfuzzer-sys|dhat|stats_alloc'
done
graph fuzz-wasm ic-wire-qualification wasm32-unknown-unknown --features fuzz-runtime
reject fuzz-wasm 'libfuzzer-sys|dhat|stats_alloc'
printf 'Qualification tools are opt-in and native-only; ordinary codec/benchmark graphs are clean.\n'
