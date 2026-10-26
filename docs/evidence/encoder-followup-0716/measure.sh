#!/usr/bin/env bash
# Reconstruct one frozen experiment; every mutation stays in the new directory.
set -euo pipefail
[[ $# == 2 ]] || { echo 'usage: measure.sh push|extend|one-pass|one-pass-shared|one-pass-fast NEW-ABSOLUTE-DIRECTORY' >&2; exit 2; }
name="$1"
work="$2"
case "$name" in push|extend|one-pass|one-pass-shared|one-pass-fast) ;; *) exit 2;; esac
case "$work" in /*) ;; *) echo 'use an absolute output directory' >&2; exit 2;; esac
[[ -z "${RUSTFLAGS:-}${CARGO_ENCODED_RUSTFLAGS:-}" ]] || { echo 'recorded measurements use no extra RUSTFLAGS' >&2; exit 2; }
evidence=$(cd -- "$(dirname -- "$0")" && pwd)
repo=$(git -C "$evidence" rev-parse --show-toplevel)
mkdir "$work"
mkdir "$work/source"
git -C "$repo" archive 47f011702edd0fce99d4272fd80b02991ee5f94f | tar -x -C "$work/source"
git -C "$work/source" apply "$evidence/baseline.patch"
export CARGO_TARGET_DIR="$work/target"
printf 'cohort,variant,profile,total_bytes,code_bytes,sha256\n' > "$work/sizes.csv"
for cohort in standard hundred; do
    features=wire
    if [ "$cohort" = hundred ]; then features=wire,derive-scale; fi
    for variant in baseline candidate; do
        if [ "$variant" = candidate ]; then git -C "$work/source" apply "$evidence/$name.patch"; fi
        out="$work/$cohort-$variant"
        mkdir "$out"
        for profile in evaluation benchmark-speed; do
            (cd "$work/source" && cargo build --locked --offline --profile "$profile" \
                -p ic-wire-benchmarks --example codec-wasm --target wasm32-unknown-unknown \
                --features "$features" --message-format=json) > "$out/$profile-build.jsonl"
            artifact=$(node --input-type=module - "$out/$profile-build.jsonl" <<'JS'
import fs from 'node:fs';
const rows = fs.readFileSync(process.argv[2],'utf8').trim().split('\n').map(JSON.parse);
console.log(rows.findLast(r=>r.reason==='compiler-artifact'&&r.target.name==='codec-wasm').filenames.find(f=>f.endsWith('.wasm')));
JS
)
            cp "$artifact" "$out/$profile-wire.wasm"
            node --input-type=module - "$artifact" "$cohort" "$variant" "$profile" >> "$work/sizes.csv" <<'JS'
import fs from 'node:fs';
import crypto from 'node:crypto';
const bytes = fs.readFileSync(process.argv[2]);
let code = 0;
for (let p=8; p<bytes.length;) {
    const kind=bytes[p++]; let size=0,shift=0,byte;
    do { byte=bytes[p++]; size+=(byte&127)*2**shift; shift+=7; } while (byte&128);
    if (kind===10) code=size;
    p+=size;
}
console.log([...process.argv.slice(3),bytes.length,code,crypto.createHash('sha256').update(bytes).digest('hex')].join(','));
JS
        done
        if [ "$variant" = candidate ]; then git -C "$work/source" apply --reverse "$evidence/$name.patch"; fi
    done
done
printf 'Evidence: %s\n' "$work"
