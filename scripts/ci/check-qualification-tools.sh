#!/usr/bin/env bash
# Native dev-dependencies own test tools; fuzzing remains explicitly selected.
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)/rust-policy.sh"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
cd "$root"
export RUSTUP_AUTO_INSTALL=0
out="${CARGO_TARGET_DIR:-$root/target}/qualification-tools"
mkdir -p "$out"
host="$(rustc -vV | sed -n 's/^host: //p')"
[[ -n "$host" ]]
graph() {
    local label="$1" package="$2" target="$3" edges="$4"
    shift 4
    local manifest="$root/qualification/Cargo.toml"
    [[ "$package" != ic-wire ]] || manifest="$root/Cargo.toml"
    cargo tree --manifest-path "$manifest" --locked --offline -p "$package" --target "$target" \
        --edges "$edges" --prefix none "$@" > "$out/$label.txt"
}
reject() {
    if grep -Eq "^($2) v" "$out/$1.txt"; then
        printf 'Unexpected qualification tool in %s\n' "$1" >&2
        exit 1
    fi
}
graph codec ic-wire "$host" normal,build,dev
graph benchmark ic-wire-benchmarks "$host" normal,build --features wire,candid
graph qualification ic-wire-qualification "$host" normal,build --features allocation-fixture
for label in codec benchmark qualification; do
    reject "$label" 'libfuzzer-sys|dhat|stats_alloc'
    reject "$label" 'ic-build|ic-testkit|ic-platform|tokio'
done
graph schema-cost ic-wire-qualification "$host" normal,build,dev --features schema-cost
grep -Eq '^stats_alloc v' "$out/schema-cost.txt"
grep -Eq '^ic-build v' "$out/schema-cost.txt"
reject schema-cost 'libfuzzer-sys|dhat'
graph schema-cost-wasm ic-wire-qualification wasm32-unknown-unknown normal,build,dev --features schema-cost
reject schema-cost-wasm 'libfuzzer-sys|dhat|stats_alloc|ic-build|ic-testkit|tokio'
graph fuzz ic-wire-qualification "$host" normal,build --features fuzz-runtime
grep -Eq '^libfuzzer-sys v' "$out/fuzz.txt"
reject fuzz 'dhat|stats_alloc'
for mode in allocations heap; do
    graph "$mode" ic-wire-benchmarks "$host" normal,build,dev --features "wire,candid,$mode"
    for tool in stats_alloc dhat ic-testkit; do
        grep -Eq "^$tool v" "$out/$mode.txt"
    done
    reject "$mode" 'libfuzzer-sys'
    graph "$mode-wasm" ic-wire-benchmarks wasm32-unknown-unknown normal,build,dev --features "wire,candid,$mode"
    reject "$mode-wasm" 'libfuzzer-sys|dhat|stats_alloc|ic-build|ic-testkit|tokio'
done
graph fuzz-wasm ic-wire-qualification wasm32-unknown-unknown normal,build,dev --features fuzz-runtime
reject fuzz-wasm 'libfuzzer-sys|dhat|stats_alloc|ic-build|ic-testkit|tokio'
printf 'Qualification tools remain native dev-only; library/Wasm graphs exclude them and fuzzing remains opt-in.\n'
