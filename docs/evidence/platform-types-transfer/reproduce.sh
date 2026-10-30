#!/usr/bin/env bash
# Reconstruct prepared or published Platform qualification without sibling writes.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo="$(cd "$here/../../.." && pwd -P)"
out="${1:?fresh absolute output directory}"
platform_repo="${2:?read-only Platform Git repository containing the recorded revision}"
supplier="${3:-prepared}"
case "$supplier" in prepared) lock_patch="$here/supplier-lock.patch";; published) lock_patch="$here/published/supplier-lock.patch";; *) exit 1;; esac
case "$out" in /*) ;; *) echo 'output must be absolute' >&2; exit 1;; esac
: "${POCKET_IC_BIN:?explicitly prepared instrumented server}"
mkdir "$out"
mkdir "$out/platform" "$out/baseline" "$out/candidate"
if [[ "$supplier" == prepared ]]; then
 git -C "$platform_repo" archive "$(cat "$here/platform-revision.txt")" | tar -xf - -C "$out/platform"
 patch -d "$out/platform" -p1 < "$here/platform-prepared.patch"
fi
git -C "$repo" archive "$(cat "$here/baseline-revision.txt")" | tar -xf - -C "$out/baseline"
cp -R "$out/baseline/." "$out/candidate/"
patch -d "$out/candidate" -p1 < "$here/wire-transfer.patch"
node - "$out" "$supplier" <<'JS'
const fs=require('node:fs'),crypto=require('node:crypto'),path=require('node:path');
const [out,supplier]=process.argv.slice(2);
const edit=(p,f)=>fs.writeFileSync(p,f(fs.readFileSync(p,'utf8')));
// Version preparation is local to the frozen copy. No registry/publication claim.
if(supplier==='prepared') edit(`${out}/platform/Cargo.toml`,s=>s.replaceAll('version = "0.10.0"','version = "0.11.0"').replace('version = "=0.10.0"','version = "=0.11.0"'));
const source=supplier==='prepared'?`, path = ${JSON.stringify(`${out}/platform/crates/ic-platform`)}`:'';
edit(`${out}/candidate/Cargo.toml`,s=>s.replace(/^ic-platform = .*$/m,`ic-platform = { version = "0.11.0"${source}, default-features = false }`));
for(const lane of ['baseline','candidate','platform']) {
 const lines=[];
 const walk=dir=>{for(const e of fs.readdirSync(dir,{withFileTypes:true}).sort((a,b)=>a.name.localeCompare(b.name))) {
   const p=path.join(dir,e.name);
   if(e.isDirectory()) walk(p);
   else if(e.isFile()) lines.push(`${crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex')}  ${path.relative(`${out}/${lane}`,p)}`);
 }};
 walk(`${out}/${lane}`); fs.writeFileSync(`${out}/${lane}-source.sha256`,lines.join('\n')+'\n');
}
JS
rustc -vV > "$out/compiler.txt"
cmp "$here/compiler.txt" "$out/compiler.txt"
sha256sum "$POCKET_IC_BIN" > "$out/server.sha256"
cargo metadata --manifest-path "$out/candidate/Cargo.toml" --offline --format-version 1 > "$out/prepared-metadata.json"
# Assert that offline resolution reproduced the reviewed supplier-only lock change.
cp "$out/baseline/Cargo.lock" "$out/expected.lock"
patch "$out/expected.lock" < "$lock_patch"
cmp "$out/expected.lock" "$out/candidate/Cargo.lock"
bash "$out/candidate/scripts/ci/check-platform-types.sh" "$repo/examples/browser/node_modules/typescript/bin/tsc"
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$out/candidate-target" bash "$out/candidate/scripts/ci/check-endpoint.sh"
for lane in baseline candidate; do
 for profile in evaluation benchmark-speed; do
  CARGO_TARGET_DIR="$out/$lane-target" cargo build --manifest-path "$out/$lane/Cargo.toml" --locked --offline -p ic-wire-endpoint-fixture --lib --target wasm32-unknown-unknown --profile "$profile"
  mkdir "$out/cost-$lane-$profile"
  cp "$out/$lane-target/wasm32-unknown-unknown/$profile/ic_wire_endpoint_fixture.wasm" "$out/cost-$lane-$profile/endpoint.wasm"
  node "$out/candidate/scripts/ci/summarize-endpoint-matrix.mjs" "$out/cost-$lane-$profile" --exports
 done
done
mkdir "$out/host"
tar -C "$out/candidate" --exclude='./target' -cf - . | tar -xf - -C "$out/host"
(cd "$out/host" && patch -p1 < "$repo/docs/evidence/platform010-tooling071/cost-cases.patch")
export CARGO_TARGET_DIR="$out/host-target"
ENDPOINT_WASM="$out/cost-candidate-evaluation/endpoint.wasm" cargo test --manifest-path "$out/host/Cargo.toml" --locked --offline -p ic-wire-endpoint-fixture --test installed --features ic -- --ignored --nocapture
for profile in evaluation benchmark-speed; do
 for lane in baseline candidate; do
  ENDPOINT_MATRIX_DIR="$out/cost-$lane-$profile" cargo test --manifest-path "$out/host/Cargo.toml" --locked --offline -p ic-wire-endpoint-fixture --test cost-matrix --features ic -- --ignored --test-threads=1 > "$out/cost-$lane-$profile/run.log" 2>&1
 done
done
node "$repo/docs/evidence/platform010-tooling071/analyze.mjs" "$out" "$out"
