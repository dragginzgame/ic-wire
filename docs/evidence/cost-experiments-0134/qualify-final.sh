#!/usr/bin/env bash
set -euo pipefail
root=/home/adam/projects/ic-wire/target/research-0134
source_root="$root/final/source"
export CARGO_TARGET_DIR=/home/adam/projects/ic-wire/target/candid-comparison-0133-build
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
export POCKET_IC_BIN=/home/adam/.cache/canic/pocket-ic-server-16.0.0-pocket-ic-x86_64-linux/pocket-ic
export RUNS=2 SAMPLES=11 BATCH_MS=2
for variant in baseline admission-linear; do
    python3 - "$root" "$variant" <<'PY'
from pathlib import Path
import shutil,sys
root=Path(sys.argv[1])
for name in ['mod.rs','encode/mod.rs','decode/mod.rs']:
 p=Path('crates/ic-wire/src/native')/name
 shutil.copyfile(root/sys.argv[2]/p,root/'final/source'/p)
PY
    cd "$source_root"
    mkdir "$root/final/$variant-scale-100"
    for profile in evaluation benchmark-speed; do
        cargo build --locked --offline --profile "$profile" -p ic-wire-benchmarks --example codec-wasm --target wasm32-unknown-unknown --features wire,derive-scale --message-format=json > "$root/final/$variant-scale-100/$profile-build.jsonl" 2> "$root/final/$variant-scale-100/$profile-build.log"
        artifact="$(cargo run --quiet --locked --offline -p ic-wire-benchmarks --example codec-report -- --artifact "$root/final/$variant-scale-100/$profile-build.jsonl")"
        cp "$artifact" "$root/final/$variant-scale-100/$profile.wasm"
        code="$(cargo run --quiet --locked --offline -p ic-wire-benchmarks --example codec-report -- --code-size "$artifact")"
        printf '%s,%s,%s\n' "$profile" "$(wc -c < "$artifact" | tr -d ' ')" "$code" >> "$root/final/$variant-scale-100/wasm.csv"
    done
    # Preserve explicit library/guest rebuild evidence for the smoke comparison.
    touch crates/ic-wire/src/native/mod.rs crates/ic-wire-benchmarks/drivers/wasm.rs
    CODEC_SOURCE_SNAPSHOT="$source_root" CORPUS_FEATURE=cost-smoke bash scripts/ci/evaluate-codecs.sh "$root/final/$variant" > "$root/final/$variant.log" 2>&1
    echo "Qualified $variant"
done
