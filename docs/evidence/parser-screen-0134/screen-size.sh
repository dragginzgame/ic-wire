#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export CARGO_TARGET_DIR="$(cd "$root/.." && pwd)/candid-comparison-0133-build"
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
source_root="$root/source"
for variant in baseline bytes-direct buffer-shared varint-tail; do
    python3 - "$root" "$variant" <<'PY'
from pathlib import Path
import shutil,sys
root=Path(sys.argv[1])
for file in ['mod.rs','encode/mod.rs','decode/mod.rs','parser/mod.rs']:
    shutil.copyfile(root/'variants'/sys.argv[2]/file,root/'source/crates/ic-wire/src/native'/file)
PY
    cd "$source_root"
    out="$root/$variant-scale"
    mkdir "$out"
    printf 'profile,total_bytes,code_section_bytes,sha256\n' > "$out/wasm.csv"
    for profile in evaluation benchmark-speed; do
        cargo build --locked --offline --profile "$profile" -p ic-wire-benchmarks --example codec-wasm --target wasm32-unknown-unknown --features wire,derive-scale --message-format=json > "$out/$profile-build.jsonl" 2> "$out/$profile-build.log"
        python3 - "$out/$profile-build.jsonl" <<'PY'
import json,sys
rows=[json.loads(s) for s in open(sys.argv[1])]
for name in ['ic_wire','codec-wasm']:
    found=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name]
    assert len(found)==1 and not found[0]['fresh'],('cached candidate',name)
PY
        artifact="$(cargo run --quiet --locked --offline -p ic-wire-benchmarks --example codec-report -- --artifact "$out/$profile-build.jsonl")"
        cp "$artifact" "$out/$profile.wasm"
        code="$(cargo run --quiet --locked --offline -p ic-wire-benchmarks --example codec-report -- --code-size "$artifact")"
        digest="$(shasum -a 256 "$artifact")"
        printf '%s,%s,%s,%s\n' "$profile" "$(wc -c < "$artifact" | tr -d ' ')" "$code" "${digest%% *}" >> "$out/wasm.csv"
    done
    echo "Completed $variant"
    cat "$out/wasm.csv"
done
