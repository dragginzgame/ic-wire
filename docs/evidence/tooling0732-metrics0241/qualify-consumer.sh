#!/usr/bin/env bash
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
out="${1:?provide a fresh absolute output directory}"
case "$out" in /*) ;; *) exit 2;; esac
mkdir "$out"
mkdir "$out/src"
cp "$here/Cargo.toml.snapshot" "$out/Cargo.toml"
cp "$here/Cargo.lock.snapshot" "$out/Cargo.lock"
cp "$here/consumer.rs.snapshot" "$out/src/lib.rs"
cargo metadata --manifest-path "$out/Cargo.toml" --locked --offline --format-version 1 > "$out/metadata.json"
CARGO_TARGET_DIR="$out/target" cargo +1.91.0 test --manifest-path "$out/Cargo.toml" --locked --offline --lib
python3 - "$out/metadata.json" <<'PYTHON'
import json,sys
m=json.load(open(sys.argv[1]))
for name in ['ic-wire','ic-wire-derive','ic-platform-types','ic-testkit','ic-metrics']:
 p=[p for p in m['packages'] if p['name']==name];assert len(p)==1,(name,p)
 assert p[0]['source'].startswith('registry+'),p
 print(name,p[0]['id'])
PYTHON
