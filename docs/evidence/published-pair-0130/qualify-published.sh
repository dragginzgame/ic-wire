#!/usr/bin/env bash
set -euo pipefail
inputs="$(cd "$(dirname "$0")" && pwd -P)"
root="$(cd "$inputs/../../.." && pwd -P)"
out="${1:?provide a fresh output directory}"
mkdir "$out"
out="$(cd "$out" && pwd -P)"
mkdir "$out/src" "$out/browser"
cp "$inputs/Cargo.toml.snapshot" "$out/Cargo.toml"
cp "$inputs/Cargo.lock.snapshot" "$out/Cargo.lock"
cp "$inputs/consumer.rs.snapshot" "$out/src/lib.rs"
export CARGO_TARGET_DIR="$out/target"
cargo metadata --manifest-path "$out/Cargo.toml" --locked --format-version 1 > "$out/metadata.json"
for lane in schema combined; do
    options=(--manifest-path "$out/Cargo.toml" --locked)
    [[ "$lane" != combined ]] || options+=(--features endpoint)
    WIRE_PLATFORM_TYPES_OUTPUT="$out/browser" cargo +1.91.0 test "${options[@]}" --lib
    cargo +1.91.0 check "${options[@]}" --target wasm32-unknown-unknown
    cargo tree "${options[@]}" --target wasm32-unknown-unknown --edges normal,build --prefix none > "$out/$lane-wasm-graph.txt"
    if grep -Eq '^(candid|pocket-ic|ic-cdk-executor|serde_json) v' "$out/$lane-wasm-graph.txt"; then exit 1; fi
    if [[ "$lane" == schema ]]; then
        if grep -Eq '^ic-platform-(runtime-guard|macros) v' "$out/$lane-wasm-graph.txt"; then exit 1; fi
    else
        [[ "$(grep -c '^ic-platform-runtime-guard v' "$out/$lane-wasm-graph.txt")" == 1 ]]
    fi
done
printf '{"type":"module"}\n' > "$out/browser/package.json"
node "$root/examples/browser/node_modules/typescript/bin/tsc" --ignoreConfig --strict --noUncheckedIndexedAccess --exactOptionalPropertyTypes --target ES2022 --module NodeNext --moduleResolution NodeNext --outDir "$out/browser" "$out/browser/principal.ts" "$out/browser/account.ts" "$out/browser/wire-runtime.ts"
cp "$inputs/check.mjs" "$out/browser/check.mjs"
node "$out/browser/check.mjs"
cargo metadata --manifest-path "$out/Cargo.toml" --locked --features endpoint --format-version 1 > "$out/metadata-combined.json"
python3 - "$out/metadata-combined.json" <<'PY'
import json,sys
m=json.load(open(sys.argv[1]))
for n in ['ic-wire','ic-wire-derive','ic-wire-endpoint','ic-platform-types']:
 p=[p for p in m['packages'] if p['name']==n];assert len(p)==1,(n,p)
 assert p[0]['source'].startswith('registry+'),p
 print(n,p[0]['version'],p[0]['source'])
PY
