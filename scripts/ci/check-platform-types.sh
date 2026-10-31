#!/usr/bin/env bash
# Standalone schema/Serde consumer; explicit prepared Node/TypeScript, no installs.
set -euo pipefail
export RUSTUP_AUTO_INSTALL=0
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
compiler="${1:-$root/examples/browser/node_modules/typescript/bin/tsc}"
[[ -f "$compiler" ]] || { echo 'Prepare the browser TypeScript compiler first' >&2; exit 1; }
compiler="$(cd "$(dirname "$compiler")" && pwd -P)/$(basename "$compiler")"
node "$compiler" --version
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/platform-types.XXXXXX")"
printf 'Platform type qualification: %s\n' "$fixture"
mkdir "$fixture/src" "$fixture/browser"
cp "$root/crates/ic-wire-qualification/fixtures/platform_types/mod.rs" "$fixture/src/lib.rs"
cp "$root/Cargo.lock" "$fixture/Cargo.lock"
cargo metadata --manifest-path "$root/Cargo.toml" --locked --offline --format-version 1 > "$fixture/workspace.json"
# Reuse the exact selected supplier; isolated pre-release qualification may use
# a path source, but the normal consumer uses the selected registry release.
node - "$root" "$fixture" <<'JS'
const fs=require('node:fs'),path=require('node:path');
const [root,fixture]=process.argv.slice(2),m=JSON.parse(fs.readFileSync(`${fixture}/workspace.json`));
const wire=m.packages.find(p=>p.name==='ic-wire'&&p.manifest_path===`${root}/crates/ic-wire/Cargo.toml`);
const endpoint=m.packages.find(p=>p.name==='ic-wire-endpoint'&&p.manifest_path===`${root}/crates/ic-wire-endpoint/Cargo.toml`);
const dep=m.resolve.nodes.find(n=>n.id===endpoint.id).deps.find(d=>d.name==='ic_platform');
const platform=m.packages.find(p=>p.id===dep.pkg);
const source=platform.source===null?`, path = ${JSON.stringify(path.dirname(platform.manifest_path))}`:'';
fs.writeFileSync(`${fixture}/Cargo.toml`, `[workspace]
[package]
name = "wire-platform-types-consumer"
version = "0.0.0"
edition = "2024"
[features]
endpoint = ["dep:ic-wire-endpoint"]
[dependencies]
ic-wire = { path = ${JSON.stringify(`${root}/crates/ic-wire`)}, default-features = false, features = ["platform-types"] }
ic-platform = { version = "=${platform.version}"${source}, default-features = false, features = ["principal"] }
ic-wire-endpoint = { path = ${JSON.stringify(`${root}/crates/ic-wire-endpoint`)}, optional = true, features = ["typescript"] }
serde = { version = "1", default-features = false, features = ["derive", "alloc"] }
[dev-dependencies]
serde_bytes = "0.11"
serde_json = "1"
`);
JS
# Only this disposable consumer lock is reconciled; checks thereafter are locked.
export CARGO_TARGET_DIR="$fixture/target"
cargo metadata --manifest-path "$fixture/Cargo.toml" --offline --format-version 1 > "$fixture/metadata.json"
for selection in types combined; do
    options=(--manifest-path "$fixture/Cargo.toml" --locked --offline)
    [[ "$selection" != combined ]] || options+=(--features endpoint)
    for target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
        graph="$fixture/$selection-$target.txt"
        cargo tree "${options[@]}" --target "$target" --edges normal,build --prefix none --format '{p} features={f}' > "$graph"
        if grep -Eq '^(candid|candid_parser|pocket-ic|ic-cdk|ic-cdk-executor|ic0) v' "$graph"; then
            echo 'External codec/runtime dependency leaked into platform types' >&2; exit 1
        fi
        if [[ "$selection" == types ]]; then
            if grep -Eq '^ic-platform-(macros|runtime-guard) v|^ic-platform v.*features=.*(cdk|executor|runtime|candid|wire-schema)' "$graph"; then
                echo 'Schema-only consumer selected endpoint/runtime machinery' >&2; exit 1
            fi
        else
            grep -Eq '^ic-platform-runtime-guard v' "$graph"
            grep -Eq '^ic-platform-macros v' "$graph"
        fi
        if [[ "$target" == wasm32-unknown-unknown ]]; then
            if grep -Eq '^serde_json v' "$graph"; then echo 'Host schema dependency leaked to Wasm' >&2; exit 1; fi
        else
            grep -Eq '^serde_json v' "$graph"
        fi
        cargo +1.91.0 check "${options[@]}" --target "$target"
    done
    WIRE_PLATFORM_TYPES_OUTPUT="$fixture/browser" cargo +1.91.0 test "${options[@]}" --lib
    cargo clippy "${options[@]}" --all-targets -- -D warnings
done
for target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    cargo tree --manifest-path "$root/Cargo.toml" --locked --offline -p ic-wire --no-default-features --target "$target" --edges normal,build --prefix none > "$fixture/core-$target.txt"
    if grep -Eq '^(ic-platform|serde_json|candid|pocket-ic) v' "$fixture/core-$target.txt"; then echo 'Optional integration leaked into core' >&2; exit 1; fi
done
printf '{"type":"module"}\n' > "$fixture/browser/package.json"
node "$compiler" --ignoreConfig --strict --noUncheckedIndexedAccess --exactOptionalPropertyTypes --target ES2022 --module NodeNext --moduleResolution NodeNext --outDir "$fixture/browser" "$fixture/browser/principal.ts" "$fixture/browser/account.ts" "$fixture/browser/wire-runtime.ts"
cp "$root/crates/ic-wire-qualification/fixtures/platform_types/check.mjs" "$fixture/browser/check.mjs"
node "$fixture/browser/check.mjs"
