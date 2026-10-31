#!/usr/bin/env bash
# Source-alignment qualification of released Metrics/Testkit implementations.
# These isolated manifest edits are evidence, not published downstream adoption.
set -euo pipefail
root="$(cd "$(dirname "$0")/../../.." && pwd -P)"
cd "$root"
mkdir -p target
out="${1:?provide a fresh absolute output directory}"
mkdir "$out"
printf 'Consumer identity evidence: %s\n' "$out"
cargo metadata --locked --offline --format-version 1 > "$out/workspace.json"
python3 - "$root" "$out" <<'PY'
import json,sys,shutil,re
from pathlib import Path
root,out=map(Path,sys.argv[1:]);m=json.loads((out/'workspace.json').read_text())
for name in ['ic-metrics','ic-testkit']:
 p=next(p for p in m['packages'] if p['name']==name)
 shutil.copytree(Path(p['manifest_path']).parent,out/name)
 manifest=out/name/'Cargo.toml';s=manifest.read_text()
 s,n=re.subn(r'(\[[^\n]*dependencies\.ic-wire\]\n)version = [^\n]+',r'\g<1>version = "=0.13.0"',s)
 assert n==1,(name,n)
 manifest.write_text(s)
(out/'src').mkdir()
(out/'Cargo.toml').write_text(f'''[workspace]
[package]
name = "wire-aligned-identity-consumer"
version = "0.0.0"
edition = "2024"
[dependencies]
ic-wire = {{ version = "=0.13.0", features = ["platform-types"] }}
ic-wire-endpoint = {{ version = "=0.2.0" }}
ic-metrics = {{ path = "{out}/ic-metrics" }}
ic-testkit = {{ path = "{out}/ic-testkit", features = ["wire"] }}
ic-platform = {{ version = "=0.15.0", default-features = false, features = ["principal"] }}
''')
(out/'src/lib.rs').write_text('''#[test]
fn shared_schema_limits_and_errors() {
    use ic_testkit::pic::{WireCallSpec, WireCallError};
    use ic_wire::{Limits, Error};
    let limits = Limits { bytes: 512, allocation: 65536, nodes: 1024, depth: 16 };
    let spec = WireCallSpec { method: "echo", method_id: 7, request_limits: limits, response_limits: limits };
    let contract = ic_wire_endpoint::Contract::<ic_platform::Principal, ic_platform::Account>::new("echo", "query", 7, spec.request_limits, spec.response_limits);
    assert_eq!(contract.request.bytes, 512);
    ic_wire::typescript::native_schema::<ic_metrics::MeasurementCounter>().unwrap();
    ic_wire::typescript::native_schema::<ic_metrics::MeasurementAggregate>().unwrap();
    ic_wire::typescript::native_schema::<ic_metrics::MeasurementSummary>().unwrap();
    ic_wire::typescript::native_schema::<ic_metrics::MeasurementHistogram<3>>().unwrap();
    ic_wire::typescript::native_schema::<ic_platform::Principal>().unwrap();
    ic_wire::typescript::native_schema::<ic_platform::Account>().unwrap();
    fn codec_cause(error: WireCallError) -> Option<Error> {
        match error { WireCallError::Encode { source, .. } | WireCallError::Decode { source, .. } => Some(source), _ => None }
    }
    let _ = codec_cause;
    let _ = ic_testkit::pic::query_wire::<ic_platform::Principal, ic_platform::Principal>;
    let _ = ic_testkit::pic::update_wire::<ic_platform::Account, ic_platform::Account>;
}
''')
PY
export CARGO_TARGET_DIR="$out/target"
cargo metadata --manifest-path "$out/Cargo.toml" --offline --format-version 1 > "$out/metadata.json"
cargo +1.91.0 test --manifest-path "$out/Cargo.toml" --locked --offline --lib

python3 - "$out/metadata.json" <<'PY'
import sys,json
m=json.load(open(sys.argv[1]))
for name in ['ic-wire','ic-wire-derive','ic-platform-types']:
 p=[p for p in m['packages'] if p['name']==name];assert len(p)==1,(name,p)
 print(name,p[0]['id'])
PY
