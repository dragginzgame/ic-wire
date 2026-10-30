#!/usr/bin/env bash
# Research-only known-shape experiment. No production source is modified.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd -P)"
base="${1:?frozen baseline Wire workspace}"
out="${2:?fresh absolute output directory}"
: "${POCKET_IC_BIN:?prepared instrumented server from receipt}"
: "${CODEC_REPORT:?prepared codec-report executable}"
[[ "$base" == /* && "$out" == /* ]] || exit 1
node - "$base" "$here/../receipt.json" <<'JS'
const fs=require('fs'),crypto=require('crypto');const [base,receipt]=process.argv.slice(2);
for(const [file,hash] of Object.entries(JSON.parse(fs.readFileSync(receipt)).baseline))if(crypto.createHash('sha256').update(fs.readFileSync(`${base}/${file}`)).digest('hex')!==hash)throw Error(`baseline mismatch: ${file}`);
JS
cmp <(rustc -vV) "$here/../compiler.txt"
mkdir "$out"
mkdir -p "$out/source/examples"
cp "$base/Cargo.toml" "$base/Cargo.lock" "$base/rust-toolchain.toml" "$out/source/"
cp -R "$base/crates" "$out/source/"
cp -R "$base/examples/endpoint" "$out/source/examples/"
cd "$out/source"
patch -p1 < "$here/manifest.patch"
cp -R "$here/fixture" crates/ic-wire-benchmarks/examples/selective_input
export CARGO_TARGET_DIR="$out/target"
rustc -vV > "$out/compiler.txt"
cargo test --locked --offline -p ic-wire-benchmarks --example selective-input --features ic -- --skip physical_updates > "$out/native.log" 2>&1
cargo clippy --locked --offline -p ic-wire-benchmarks --example selective-input --features ic --tests -- -D warnings > "$out/clippy-host.log" 2>&1
cargo clippy --locked --offline -p ic-wire-benchmarks --example selective-input --target wasm32-unknown-unknown --features selective-input -- -D warnings > "$out/clippy-wasm.log" 2>&1
for profile in evaluation benchmark-speed; do
  directory="$out/$profile"
  mkdir "$directory"
  for lane in wire_owned wire_borrowed wire_direct wire_selective; do
    case "$lane" in
      wire_owned) features=wire;;
      wire_borrowed) features=borrowed-input;;
      wire_direct) features=direct-owned-input;;
      wire_selective) features=selective-input;;
    esac
    cargo build --locked --offline --profile "$profile" -p ic-wire-benchmarks --example selective-input --target wasm32-unknown-unknown --features "$features" > "$directory/$lane-build.log" 2>&1
    cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/$profile/examples/selective_input.wasm" "$directory/$lane.wasm"
    "$CODEC_REPORT" --code-size "$directory/$lane.wasm" > "$directory/$lane-code-bytes.txt"
  done
  SELECTIVE_EVIDENCE="$directory" cargo test --locked --offline -p ic-wire-benchmarks --example selective-input --features ic physical_updates -- --ignored --test-threads=1 > "$directory/ic.log" 2>&1
done
node "$here/analyze.mjs" "$out" > "$out/analysis.log"
