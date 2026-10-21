#!/usr/bin/env bash
# Isolated before/after native probe; no real-checkout builds or source mutations.
set -euo pipefail
if [[ $# -lt 2 || $# -gt 3 ]]; then
  echo 'usage: bash scripts/benchmarks/protocol/run.sh NEW_OUTPUT_DIR BASELINE_REF [CANDIDATE_CONTRACT]' >&2
  exit 2
fi
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)
output=$1
baseline=$2
candidate_contract=${3:-$root/crates/ic-wire/src/contract/mod.rs}
if [[ ! -f "$candidate_contract" ]]; then
  echo 'candidate contract must be a regular file' >&2
  exit 2
fi
if [[ -e "$output" ]]; then
  echo 'output directory must not exist' >&2
  exit 2
fi
mkdir -p "$output"
output=$(cd "$output" && pwd)
mkdir -p "$output/repo" "$output/baseline-frames" "$output/candidate-frames"
git -C "$root" archive "$baseline" | tar -x -C "$output/repo"
cp "$root/Cargo.lock" "$output/repo/Cargo.lock"
cp "$root/scripts/benchmarks/protocol/mod.rs" "$output/repo/crates/ic-wire/examples/protocol_probe.rs"
mkdir -p "$output/repo/crates/ic-wire/examples/native_views"
cp "$root/scripts/benchmarks/protocol/native_views/mod.rs" "$output/repo/crates/ic-wire/examples/native_views/mod.rs"
cat >> "$output/repo/crates/ic-wire/src/lib.rs" <<'RUST'

/// Benchmark-only access to the production comparator; never shipped.
pub fn probe_canonical<T: serde::Serialize>(value: &T, payload: &[u8]) -> bool {
    postcard::serialize_with_flavor(value, contract::Compare {
        remaining: payload, allowance: payload.len(), equal: true,
    }).unwrap()
}
RUST
rustc -Vv > "$output/compiler.txt"
git -C "$root" rev-parse "$baseline" > "$output/baseline-ref.txt"
printf 'RUSTFLAGS=%s\nCARGO_ENCODED_RUSTFLAGS=%s\n' "${RUSTFLAGS-}" "${CARGO_ENCODED_RUSTFLAGS-}" > "$output/flags.txt"
for lane in baseline candidate; do
  if [[ "$lane" == candidate ]]; then
    cp "$candidate_contract" "$output/repo/crates/ic-wire/src/contract/mod.rs"
  fi
  cp "$output/repo/crates/ic-wire/src/contract/mod.rs" "$output/$lane-contract.rs"
  CARGO_NET_OFFLINE=true cargo build --locked --release --manifest-path "$output/repo/Cargo.toml" \
    -p ic-wire --example protocol_probe > "$output/build-$lane.log" 2>&1
  cp "$output/repo/target/release/examples/protocol_probe" "$output/$lane"
done
# Inherit CPU affinity from the caller, e.g. taskset -c 0 bash run.sh ... on Linux.
for run in 1 2; do
  lanes=(baseline candidate)
  if [[ "$run" == 2 ]]; then lanes=(candidate baseline); fi
  for lane in "${lanes[@]}"; do
    "$output/$lane" "$output/$lane-frames" > "$output/$lane-$run.csv"
  done
done
for frame in "$output/baseline-frames/"*.bin; do
  cmp "$frame" "$output/candidate-frames/$(basename "$frame")"
done
