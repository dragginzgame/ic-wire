#!/usr/bin/env bash
# Registry-only identity consumer; no patches, source alignment or service startup.
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
import sys,json
m=json.load(open(sys.argv[1]))
for n in ['ic-wire','ic-wire-derive','ic-platform-types','ic-testkit']:
 p=[p for p in m['packages'] if p['name']==n];assert len(p)==1,(n,p)
 assert p[0]['source'].startswith('registry+'),p
 print(n,p[0]['id'])
PYTHON
