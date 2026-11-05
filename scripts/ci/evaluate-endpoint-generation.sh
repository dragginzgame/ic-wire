#!/usr/bin/env bash
# Matched host expansion study; does not time Platform expansion or Rust const evaluation.
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)/rust-policy.sh"
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
[[ $# == 2 ]] || { echo 'usage: evaluate-endpoint-generation.sh BEFORE_SOURCE NEW_OUTPUT' >&2; exit 2; }
before="$1"
mkdir -p "$(dirname "$2")"
mkdir "$2"
out="$(cd "$2" && pwd -P)"
mkdir "$out/before" "$out/after"
cp "$before" "$out/before/mod.rs"
cp crates/ic-wire-endpoint-macros/src/endpoint/mod.rs "$out/after/mod.rs"
cargo build --locked -p ic-wire-endpoint-macros --message-format=json > "$out/artifacts.jsonl"
node - "$out" <<'JS'
const fs = require('fs');
const dir = process.argv[2];
const artifacts = fs.readFileSync(`${dir}/artifacts.jsonl`, 'utf8').trim().split('\n').map(JSON.parse);
for (const name of ['proc_macro2','quote','syn']) {
 const artifact = artifacts.find(a=>a.reason==='compiler-artifact' && a.target.name===name && a.filenames.some(p=>p.endsWith('.rlib')));
 if (!artifact) throw new Error(`missing ${name}`);
 fs.writeFileSync(`${dir}/${name}.path`, artifact.filenames.find(p=>p.endsWith('.rlib')));
}
JS
cp crates/ic-wire-qualification/benches/endpoint_generation/main.rs "$out/main.rs"
rustc -Vv > "$out/compiler.txt"
shasum -a 256 Cargo.lock "$out/before/mod.rs" "$out/after/mod.rs" "$out/main.rs" > "$out/inputs.sha256"
rustc --edition=2024 -O "$out/main.rs" \
    --extern "proc_macro2=$(cat "$out/proc_macro2.path")" \
    --extern "quote=$(cat "$out/quote.path")" \
    --extern "syn=$(cat "$out/syn.path")" \
    -L "dependency=${CARGO_TARGET_DIR:-$PWD/target}/debug/deps" -o "$out/measure"
"$out/measure" > "$out/samples.csv"
