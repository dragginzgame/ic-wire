#!/usr/bin/env bash
# Exact registry-only typed consumer; no release or upload effects.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
out="${1:?provide a fresh absolute output directory}"
case "$out" in /*) ;; *) exit 2;; esac
mkdir "$out"
mkdir "$out/src"
cp "$here/consumer.Cargo.toml.snapshot" "$out/Cargo.toml"
cp "$here/consumer.Cargo.lock.snapshot" "$out/Cargo.lock"
cp "$here/../tooling0732-metrics0241/consumer.rs.snapshot" "$out/src/lib.rs"
cargo metadata --manifest-path "$out/Cargo.toml" --locked --offline --format-version 1 > "$out/metadata.json"
CARGO_TARGET_DIR="$out/target" cargo +1.91.0 test --manifest-path "$out/Cargo.toml" --locked --offline --lib
python3 - "$out/metadata.json" <<'PYTHON'
import json,sys
m=json.load(open(sys.argv[1]))
for name,version in [('ic-wire','0.13.2'),('ic-wire-derive','0.13.2'),('ic-wire-endpoint','0.2.1'),('ic-platform-types','0.15.1'),('ic-testkit','0.75.2'),('ic-metrics','0.24.1'),('ic-host-artifacts','0.73.3'),('ic-host-fs','0.73.3'),('ic-host-process','0.73.3')]:
    packages=[p for p in m['packages'] if p['name']==name]
    assert len(packages)==1 and packages[0]['version']==version and packages[0]['source'].startswith('registry+'),(name,packages)
    print(name,version,'one registry identity')
PYTHON
