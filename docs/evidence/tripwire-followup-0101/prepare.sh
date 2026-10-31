#!/usr/bin/env bash
# Creates only isolated research sources. Never writes a sibling checkout.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo="$(cd "$here/../../.." && pwd -P)"
out="${1:?fresh absolute study directory}"
[[ "$out" == /* ]] || exit 1
python3 - "$here/environment.json" <<'PYENV'
import json,os,sys
for key,value in json.load(open(sys.argv[1])).items():
    if key.isupper():
        assert os.environ.get(key)==value, f"build environment mismatch: {key}"
PYENV
mkdir "$out"
mkdir "$out/baseline"
git -C "$repo" archive "$(cat "$here/baseline-revision.txt")" | tar -xf - -C "$out/baseline"
patch -d "$out/baseline" -p1 < "$here/inputs.patch"
cargo metadata --manifest-path "$out/baseline/Cargo.toml" --locked --offline --format-version 1 > "$out/metadata.json"
python3 - "$out" <<'PY'
from pathlib import Path
import json,shutil,sys
out=Path(sys.argv[1]);metadata=json.loads((out/'metadata.json').read_text())
p=next(p for p in metadata['packages'] if p['name']=='ic-platform-macros')
assert p['version']=='0.11.1'
shutil.copytree(Path(p['manifest_path']).parent,out/'platform-macros')
shutil.copytree(out/'baseline',out/'candidate')
shutil.copytree(out/'baseline',out/'input')
PY
patch -d "$out/platform-macros" -p1 < "$here/platform-hook.patch"
patch -d "$out/candidate" -p1 < "$here/inline-reply.patch"
cp -R "$repo/docs/evidence/tripwire-adoption-0725/selective/fixture" "$out/input/crates/ic-wire-benchmarks/examples/selective_input"
python3 - "$out" <<'PY'
from pathlib import Path
import sys
p=Path(sys.argv[1])/'input/crates/ic-wire-benchmarks/Cargo.toml'
p.write_text(p.read_text().replace('[features]','[features]\nselective-input = []\ndirect-owned-input = []').replace('features = ["candid", "ic0"]','features = ["candid", "ic0", "cdk"]')+'''
[[example]]
name = "selective-input"
path = "examples/selective_input/mod.rs"
crate-type = ["cdylib"]
''')
PY
python3 "$here/prepare-compression.py" "$out"
cargo metadata --manifest-path "$out/compression/Cargo.toml" --offline --format-version 1 > "$out/compression-metadata.json"
rustc -vV > "$out/compiler.txt"
cmp "$here/compiler.txt" "$out/compiler.txt"
