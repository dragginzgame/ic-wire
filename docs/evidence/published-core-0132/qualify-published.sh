#!/usr/bin/env bash
# Exact registry-only release qualification; no upload or release effects.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
root="${WIRE_ROOT:-$(cd "$here/../../.." && pwd -P)}"
out="${1:?provide a fresh absolute output directory}"
case "$out" in /*) ;; *) exit 2;; esac
mkdir "$out"
export CARGO_TARGET_DIR="$out/target"
for lane in schema typed; do
    mkdir -p "$out/$lane/src"
    cp "$here/$lane.Cargo.toml.snapshot" "$out/$lane/Cargo.toml"
    cp "$here/$lane.Cargo.lock.snapshot" "$out/$lane/Cargo.lock"
    if [[ "$lane" == schema ]]; then
        cp "$here/../published-pair-0131/schema.rs.snapshot" "$out/$lane/src/lib.rs"
    else
        cp "$here/../tooling0732-metrics0241/consumer.rs.snapshot" "$out/$lane/src/lib.rs"
    fi
done
mkdir "$out/schema/browser"
for selection in schema combined; do
    options=(--manifest-path "$out/schema/Cargo.toml" --locked --offline)
    [[ "$selection" != combined ]] || options+=(--features endpoint)
    WIRE_PLATFORM_TYPES_OUTPUT="$out/schema/browser" cargo +1.91.0 test "${options[@]}" --lib
    cargo +1.91.0 check "${options[@]}" --target wasm32-unknown-unknown
    cargo tree "${options[@]}" --target wasm32-unknown-unknown --edges normal,build --prefix none > "$out/$selection-wasm-graph.txt"
    if grep -Eq '^(candid|pocket-ic|ic-cdk-executor|serde_json|libfuzzer-sys|dhat|stats_alloc) v' "$out/$selection-wasm-graph.txt"; then exit 1; fi
    if [[ "$selection" == schema ]]; then
        if grep -Eq '^ic-platform-(runtime-guard|macros) v' "$out/$selection-wasm-graph.txt"; then exit 1; fi
    else
        [[ "$(grep -c '^ic-platform-runtime-guard v' "$out/$selection-wasm-graph.txt")" == 1 ]]
    fi
done
printf '{"type":"module"}\n' > "$out/schema/browser/package.json"
node "$root/examples/browser/node_modules/typescript/bin/tsc" --ignoreConfig --strict --noUncheckedIndexedAccess --exactOptionalPropertyTypes --target ES2022 --module NodeNext --moduleResolution NodeNext --outDir "$out/schema/browser" "$out/schema/browser/principal.ts" "$out/schema/browser/account.ts" "$out/schema/browser/wire-runtime.ts"
cp "$here/../published-pair-0131/check.mjs" "$out/schema/browser/check.mjs"
node "$out/schema/browser/check.mjs"
cargo +1.91.0 test --manifest-path "$out/typed/Cargo.toml" --locked --offline --lib
for lane in schema typed; do
    options=(--manifest-path "$out/$lane/Cargo.toml" --locked --offline)
    [[ "$lane" != schema ]] || options+=(--features endpoint)
    cargo metadata "${options[@]}" --format-version 1 > "$out/$lane-metadata.json"
    python3 - "$out/$lane-metadata.json" <<'PYTHON'
import json,sys
m=json.load(open(sys.argv[1]))
names=['ic-wire','ic-wire-derive','ic-wire-endpoint','ic-platform-types']
if any(p['name']=='ic-testkit' for p in m['packages']): names+=['ic-testkit','ic-metrics']
for name in names:
 p=[p for p in m['packages'] if p['name']==name];assert len(p)==1,(name,p)
 assert p[0]['source'].startswith('registry+'),p
 print(name,p[0]['id'])
PYTHON
done
