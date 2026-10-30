#!/usr/bin/env bash
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd -P)"
base="${1:?frozen baseline workspace at the receipt revision}"
out="${2:?fresh absolute output directory}"
: "${POCKET_IC_BIN:?prepared instrumented server from receipt}"
: "${CODEC_REPORT:?prepared codec-report executable}"
[[ "$base" == /* && "$out" == /* ]] || exit 1
node - "$base" "$here/receipt.json" <<'JS'
const fs=require('fs'),crypto=require('crypto');const [base,receipt]=process.argv.slice(2);
for(const [file,hash] of Object.entries(JSON.parse(fs.readFileSync(receipt)).baseline))if(crypto.createHash('sha256').update(fs.readFileSync(`${base}/${file}`)).digest('hex')!==hash)throw Error(`baseline mismatch: ${file}`);
JS
cmp <(rustc -vV) "$here/compiler.txt"
mkdir "$out"
printf 'lane,types,profile,total_bytes,code_bytes\n' > "$out/scale.csv"
for lane in baseline candidate; do
  src="$out/$lane/source"
  mkdir -p "$src/examples"
  cp "$base/Cargo.toml" "$base/Cargo.lock" "$base/rust-toolchain.toml" "$src/"
  cp -R "$base/crates" "$src/"
  cp -R "$base/examples/endpoint" "$src/examples/"
  cd "$src"
  patch -p1 < "$here/cases.patch"
  if [[ "$lane" == candidate ]]; then patch -p1 < "$here/endpoint.patch"; fi
  # Cargo path packages must never share this target across frozen workspaces.
  export CARGO_TARGET_DIR="$out/$lane/target"
  for profile in evaluation benchmark-speed; do
    directory="$out/$lane/$profile"
    mkdir "$directory"
    cargo build --locked --offline -p ic-wire-endpoint-fixture --lib --target wasm32-unknown-unknown --profile "$profile" > "$directory/build.log" 2>&1
    cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/$profile/ic_wire_endpoint_fixture.wasm" "$directory/endpoint.wasm"
    "$CODEC_REPORT" --code-size "$directory/endpoint.wasm" > "$directory/code-bytes.txt"
    node "$base/scripts/ci/summarize-endpoint-matrix.mjs" "$directory" --exports
    ENDPOINT_MATRIX_DIR="$directory" cargo test --locked --offline -p ic-wire-endpoint-fixture --test cost-matrix --features ic -- --ignored --test-threads=1 > "$directory/ic.log" 2>&1
    node "$base/scripts/ci/summarize-endpoint-matrix.mjs" "$directory" > "$directory/summary.log"
  done
  mkdir -p examples/endpoint/examples/admission_scale
  cat >> examples/endpoint/Cargo.toml <<'TOML'

[[example]]
name = "admission-scale"
path = "examples/admission_scale/mod.rs"
crate-type = ["cdylib"]
TOML
  for count in 1 10 100; do
    node - "$count" <<'JS'
const fs=require('fs');let source='//! Qualification-only independent types and generated entrypoints.\n';
for(let i=0;i<Number(process.argv[2]);i++)source+=`#[derive(serde::Serialize, serde::Deserialize)]\nstruct Request${i} { value: u64, body: ic_wire::Bytes<32> }\n#[ic_wire::endpoint::update(id = ${1000+i})]\nfn method_${i}(value: Request${i}) -> Request${i} { value }\n`;
fs.writeFileSync('examples/endpoint/examples/admission_scale/mod.rs',source);
JS
    for profile in evaluation benchmark-speed; do
      cargo build --locked --offline -p ic-wire-endpoint-fixture --example admission-scale --target wasm32-unknown-unknown --profile "$profile" > "$out/$lane/scale-$count-$profile.log" 2>&1
      wasm="$out/$lane/scale-$count-$profile.wasm"
      cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/$profile/examples/admission_scale.wasm" "$wasm"
      code=$("$CODEC_REPORT" --code-size "$wasm")
      printf '%s,%s,%s,%s,%s\n' "$lane" "$count" "$profile" "$(wc -c < "$wasm")" "$code" >> "$out/scale.csv"
    done
  done
done
node "$here/analyze-header.mjs" "$out" candidate > "$out/analysis.log"
