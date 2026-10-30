#!/usr/bin/env bash
# Matched host expansion study; does not time Platform expansion or Rust const evaluation.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
[[ $# == 2 ]] || { echo 'usage: evaluate-endpoint-generation.sh BEFORE_SOURCE NEW_OUTPUT' >&2; exit 2; }
before="$1"
mkdir -p "$(dirname "$2")"
mkdir "$2"
out="$(cd "$2" && pwd -P)"
mkdir "$out/before" "$out/after"
cp "$before" "$out/before/mod.rs"
cp crates/ic-wire-derive/src/endpoint/mod.rs "$out/after/mod.rs"
cargo build --locked -p ic-wire-derive --features endpoint --message-format=json > "$out/artifacts.jsonl"
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
cat > "$out/main.rs" <<'RUST'
mod before;
mod after;
use proc_macro2::TokenStream;
use std::{hint::black_box, time::Instant};
fn fresh_ident(source: &str, base: &str) -> syn::Ident {
    let mut name = base.to_owned();
    while source.contains(&name) { name.push('_'); }
    quote::format_ident!("{name}")
}
fn main() {
    println!("case,lane,sample,repetitions,host_ns_per_expansion");
    for (case, item) in [
        ("unit", "fn echo() {}"),
        ("scalar", "fn echo(value: u64) -> u64 { value }"),
        ("record", "async fn echo(value: ApplicationRequest) -> Result<ApplicationResponse, ApplicationError> { handle(value).await }"),
    ] {
        let item: TokenStream = item.parse().unwrap();
        let args: TokenStream = "contract = ECHO, id = 3346031142226386015u64, request = REQUEST, response = RESPONSE".parse().unwrap();
        let defaults: TokenStream = "contract = ECHO".parse().unwrap();
        for sample in 0..11 {
            // Alternate ordering to avoid consistently favouring one warmed lane.
            let lanes = if sample % 2 == 0 { ["before", "after", "defaults"] } else { ["defaults", "after", "before"] };
            for lane in lanes {
                let mut total = 0;
                let start = Instant::now();
                for _ in 0..1000 {
                    let selected = if lane == "defaults" { defaults.clone() } else { args.clone() };
                    let result = if lane == "before" { before::expand(true, selected, item.clone()) }
                        else { after::expand(true, selected, item.clone()) };
                    total += black_box(result.unwrap()).into_iter().count();
                }
                black_box(total);
                println!("{case},{lane},{sample},1000,{}", start.elapsed().as_nanos() as f64 / 1000.0);
            }
        }
    }
}
RUST
rustc -Vv > "$out/compiler.txt"
shasum -a 256 Cargo.lock "$out/before/mod.rs" "$out/after/mod.rs" "$out/main.rs" > "$out/inputs.sha256"
rustc --edition=2024 -O "$out/main.rs" \
    --extern "proc_macro2=$(cat "$out/proc_macro2.path")" \
    --extern "quote=$(cat "$out/quote.path")" \
    --extern "syn=$(cat "$out/syn.path")" \
    -L "dependency=${CARGO_TARGET_DIR:-$PWD/target}/debug/deps" -o "$out/measure"
"$out/measure" > "$out/samples.csv"
