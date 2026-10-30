#!/usr/bin/env bash
# Standalone qualification only; no dependency or codec is added to the workspace.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd -P)"
out="${1:?fresh absolute output directory}"
wire="${2:?frozen Wire source directory}"
tripwire="${3:?SDK checkout at the receipt revision}"
: "${POCKET_IC_BIN:?prepared instrumented PocketIC server}"
: "${CODEC_REPORT:?prepared ic-wire codec-report executable}"
[[ "$out" == /* && "$wire" == /* && "$tripwire" == /* ]] || exit 1
mkdir "$out"
mkdir -p "$out/probe/src" "$out/probe/examples"
python3 - "$here" "$out" "$wire" "$tripwire" <<'PY'
import hashlib, json, pathlib, sys
here, out, wire, tripwire = map(pathlib.Path, sys.argv[1:])
for row in json.loads((here/'upstream-files.json').read_text())['files']:
    data = (tripwire/row['path']).read_bytes()
    blob = hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()
    assert blob == row['git_blob'], row['path']
receipt = json.loads((here/'receipt.json').read_text())
for name, digest in receipt['wire_runtime_files'].items():
    assert hashlib.sha256((wire/name).read_bytes()).hexdigest() == digest, name
manifest = (here/'fixture/Cargo.toml.in').read_text().replace('@WIRE_SOURCE@',str(wire)).replace('@TRIPWIRE_SOURCE@',str(tripwire))
(out/'probe/Cargo.toml').write_text(manifest)
PY
cp "$here/fixture/lib.rs" "$out/probe/src/lib.rs"
cp "$here/fixture/measure.rs" "$out/probe/examples/measure.rs"
cp "$here/fixture/Cargo.lock" "$out/probe/Cargo.lock"
export CARGO_TARGET_DIR="$out/target"
export RUSTUP_TOOLCHAIN=nightly-2026-10-15
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
cd "$out/probe"
rustc -vV > "$out/compiler.txt"
sha256sum "$POCKET_IC_BIN" Cargo.lock > "$out/hashes.txt"
for profile in evaluation benchmark-speed; do
    mkdir "$out/$profile"
    for lane in wire_owned wire_borrowed candid tripwire_owned tripwire_selective; do
        cargo build --locked --offline --profile "$profile" --lib --target wasm32-unknown-unknown --features "$lane" > "$out/$profile/$lane-build.log" 2>&1
        cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/$profile/wire_tripwire_probe.wasm" "$out/$profile/$lane.wasm"
        "$CODEC_REPORT" --code-size "$out/$profile/$lane.wasm" > "$out/$profile/$lane-code-bytes.txt"
    done
done
cargo build --locked --offline --example measure --features wire_owned,wire_borrowed,candid,tripwire_owned,tripwire_selective > "$out/host-build.log" 2>&1
for profile in evaluation benchmark-speed; do
    TRIPWIRE_EVIDENCE="$out/$profile" "$CARGO_TARGET_DIR/debug/examples/measure" > "$out/$profile/ic.log" 2>&1
done
node "$here/analyze.mjs" "$out" > "$out/analysis.log"
