#!/usr/bin/env bash
set -euo pipefail
source_root="$1"
out="$2"
mkdir "$out"
cd "$source_root"
printf 'profile,codec,wasm_bytes,wasm_sha256,code_section_bytes\n' > "$out/wasm.csv"
for profile in evaluation benchmark-speed; do
    for codec in wire candid; do
        cargo build --locked --offline --profile "$profile" -p ic-wire-benchmarks --example codec-wasm --target wasm32-unknown-unknown --features "$codec,cost-smoke" --message-format=json > "$out/$profile-$codec-build.jsonl" 2> "$out/$profile-$codec-build.log"
        if [[ "$codec" == wire ]]; then
            python3 - "$out/$profile-$codec-build.jsonl" <<'VERIFY'
import json,sys
rows=[json.loads(s) for s in open(sys.argv[1])]
for name in ['ic_wire','codec-wasm']:
    records=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name]
    assert len(records)==1 and not records[0]['fresh'], ('cached candidate artifact',name)
VERIFY
        fi
        artifact="$(cargo run --quiet --locked --offline -p ic-wire-benchmarks --example codec-report -- --artifact "$out/$profile-$codec-build.jsonl")"
        cp "$artifact" "$out/$profile-$codec.wasm"
        code="$(cargo run --quiet --locked --offline -p ic-wire-benchmarks --example codec-report -- --code-size "$artifact")"
        bytes="$(wc -c < "$artifact" | tr -d ' ')"
        digest="$(shasum -a 256 "$artifact")"
        printf '%s,%s,%s,%s,%s\n' "$profile" "$codec" "$bytes" "${digest%% *}" "$code" >> "$out/wasm.csv"
    done
done
cargo run --locked --offline -p ic-wire-benchmarks --example codec-instructions --features wire,candid,ic,cost-smoke -- "$out" > "$out/instructions.log" 2>&1
