#!/usr/bin/env bash
# Future-generation simulation only; published suppliers qualify the current lane.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mkdir -p target
out="$(mktemp -d "$root/target/stable-identities.XXXXXX")"
printf 'Stable identity evidence: %s\n' "$out"
cargo metadata --locked --offline --format-version 1 > "$out/workspace.json"
python3 - "$root" "$out" <<'PY'
from pathlib import Path
import json,sys,shutil
root,out=map(Path,sys.argv[1:]);m=json.loads((out/'workspace.json').read_text())
p=next(p for p in m['packages'] if p['name']=='ic-platform' and p['version']=='0.14.0')
source=Path(p['manifest_path']).parent
shutil.copytree(source,out/'next-platform')
f=out/'next-platform/Cargo.toml';f.write_text(f.read_text().replace('version = "0.14.0"','version = "0.15.0"',1))
# This prospective runtime generation changes only package identity. No claim
# about an unpublished runtime implementation or its performance is made.
next_endpoint=out/'next-endpoint';shutil.copytree(root/'crates/ic-wire-endpoint/src',next_endpoint/'src')
(next_endpoint/'Cargo.toml').write_text(f'''[workspace]
[package]
name = "ic-wire-endpoint"
version = "0.2.0"
edition = "2024"
[features]
typescript = ["ic-wire/typescript", "dep:serde_json"]
[target.'cfg(not(target_family = "wasm"))'.dependencies]
serde_json = {{ version = "1", optional = true }}
[dependencies]
ic-platform = {{ path = "{out}/next-platform", default-features = false, features = ["cdk"] }}
ic-wire = {{ path = "{root}/crates/ic-wire" }}
ic-wire-derive = {{ path = "{root}/crates/ic-wire-derive", features = ["endpoint"] }}
serde = {{ version = "1", default-features = false, features = ["derive", "alloc"] }}
''')
for lane in ['current','next']:
 d=out/lane;(d/'src').mkdir(parents=True)
 shutil.copy2(root/'crates/ic-wire-qualification/fixtures/platform_types/mod.rs',d/'src/lib.rs')
 platform='version = "=0.14.0"' if lane=='current' else f'path = "{out}/next-platform"'
 endpoint=root/'crates/ic-wire-endpoint' if lane=='current' else next_endpoint
 (d/'Cargo.toml').write_text(f'''[workspace]
[package]
name = "wire-stable-identity-consumer"
version = "0.0.0"
edition = "2024"
[features]
default = ["endpoint"]
endpoint = []
[dependencies]
ic-wire = {{ path = "{root}/crates/ic-wire", features = ["platform-types"] }}
ic-wire-endpoint = {{ path = "{endpoint}", features = ["typescript"] }}
ic-platform = {{ {platform}, default-features = false, features = ["principal"] }}
serde = {{ version = "1", default-features = false, features = ["derive", "alloc"] }}
[dev-dependencies]
serde_bytes = "0.11"
serde_json = "1"
''')
PY
export CARGO_TARGET_DIR="$out/target"
for lane in current next; do
    manifest="$out/$lane/Cargo.toml"
    cargo metadata --manifest-path "$manifest" --offline --format-version 1 > "$out/$lane/metadata.json"
    mkdir "$out/$lane/browser"
    WIRE_PLATFORM_TYPES_OUTPUT="$out/$lane/browser" cargo test --manifest-path "$manifest" --locked --offline --lib
    cargo +1.91.0 check --manifest-path "$manifest" --locked --offline
    cargo +1.91.0 check --manifest-path "$manifest" --locked --offline --target wasm32-unknown-unknown
    cargo tree --manifest-path "$manifest" --locked --offline --target wasm32-unknown-unknown --edges normal,build --prefix none > "$out/$lane/graph.txt"
    [[ "$(grep -c '^ic-platform-runtime-guard v' "$out/$lane/graph.txt")" == 1 ]]
    if grep -Eq '^(candid|pocket-ic|ic-cdk-executor|serde_json) v' "$out/$lane/graph.txt"; then exit 1; fi
done
python3 - "$out" <<'PY'
import json,sys
from pathlib import Path
out=Path(sys.argv[1]);meta=[json.loads((out/l/'metadata.json').read_text()) for l in ['current','next']]
ids={}
for name in ['ic-wire','ic-wire-derive','ic-platform-types']:
 selections=[[p['id'] for p in m['packages'] if p['name']==name] for m in meta]
 assert len(selections[0])==1 and selections[0]==selections[1],(name,selections)
 ids[name]=selections[0][0]
for name in ['ic-platform','ic-wire-endpoint']:
 selections=[[p['id'] for p in m['packages'] if p['name']==name] for m in meta]
 assert len(selections[0])==len(selections[1])==1 and selections[0]!=selections[1]
 ids[name]=dict(zip(['current','simulated-next'],selections))
(out/'identities.json').write_text(json.dumps(ids,indent=2)+'\n')
print('Codec/schema/derive and Principal/Account supplier IDs unchanged across simulated endpoint/runtime generation.')
PY
