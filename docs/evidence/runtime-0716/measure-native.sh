#!/usr/bin/env bash
# Run after measure.sh decoder has prepared an isolated experiment directory.
set -euo pipefail
[[ $# == 1 && "$1" == /* ]] || { echo 'usage: measure-native.sh ABSOLUTE-EXPERIMENT-DIRECTORY' >&2; exit 2; }
work="$1"
evidence=$(cd -- "$(dirname -- "$0")" && pwd)
repo=$(git -C "$evidence" rev-parse --show-toplevel)
[[ -d "$work/source" && ! -e "$work/baseline-native" && ! -e "$work/candidate-native" ]]
[[ -z "${RUSTFLAGS:-}${CARGO_ENCODED_RUSTFLAGS:-}" ]] || exit 2
export CARGO_TARGET_DIR="$work/target"
for variant in baseline candidate; do
    if [ "$variant" = candidate ]; then git -C "$work/source" apply "$evidence/decoder.patch"; fi
    (cd "$work/source" && cargo bench --locked --offline --no-run --profile benchmark-speed \
        -p ic-wire-benchmarks --bench suite --features wire,candid,hotspots \
        --message-format=json) > "$work/$variant-native-build.jsonl"
    artifact=$(node --input-type=module - "$work/$variant-native-build.jsonl" <<'JS'
import fs from 'node:fs';
const rows=fs.readFileSync(process.argv[2],'utf8').trim().split('\n').map(JSON.parse);
console.log(rows.findLast(r=>r.reason==='compiler-artifact'&&r.target.name==='suite').executable);
JS
)
    cp "$artifact" "$work/$variant-native"
    if [ "$variant" = candidate ]; then git -C "$work/source" apply --reverse "$evidence/decoder.patch"; fi
done
# Serial timings, all compilation finished, with the second pair reversed.
for run in 1 2; do
    variants=(baseline candidate)
    if [ "$run" = 2 ]; then variants=(candidate baseline); fi
    for variant in "${variants[@]}"; do
        (cd "$repo" && "$work/$variant-native" "$work/native-$variant-$run" benchmark-speed 7 1)
    done
done
node "$evidence/native-analysis.mjs" "$work" "$work/native-analysis"
