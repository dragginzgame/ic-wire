#!/usr/bin/env bash
set -euo pipefail
out="${1:?prepared study directory}"
for lane in ${2:-baseline candidate}; do
 src="$out/$lane"
 mkdir -p "$src/examples/endpoint/examples/reply_scale"
 if ! rg -q '^name = "reply-scale"$' "$src/examples/endpoint/Cargo.toml"; then
 cat >> "$src/examples/endpoint/Cargo.toml" <<'TOML'

[[example]]
name = "reply-scale"
path = "examples/reply_scale/mod.rs"
crate-type = ["cdylib"]
TOML
 fi
 for count in 1 10 100; do
  node - "$src" "$count" <<'JS'
const fs=require('node:fs');const [src,n]=process.argv.slice(2);let text='//! Research-only independent reply types.\n';
for(let i=0;i<+n;i++)text+=`#[derive(serde::Serialize, serde::Deserialize)]\nstruct Reply${i} { value: u64, body: ic_wire::Bytes<32> }\n#[ic_wire::endpoint::update(id = ${1000+i})]\nfn method_${i}(value: Reply${i}) -> Reply${i} { value }\n`;
fs.writeFileSync(`${src}/examples/endpoint/examples/reply_scale/mod.rs`,text);
JS
  for profile in evaluation benchmark-speed; do
   CARGO_TARGET_DIR="$out/$lane-target" cargo build --manifest-path "$src/Cargo.toml" --locked --offline -p ic-wire-endpoint-fixture --example reply-scale --target wasm32-unknown-unknown --profile "$profile"
   cp "$out/$lane-target/wasm32-unknown-unknown/$profile/examples/reply_scale.wasm" "$out/scale-$lane-$count-$profile.wasm"
  done
 done
done
