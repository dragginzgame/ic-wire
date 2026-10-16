#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-wire-macro-diagnostics.XXXXXX")"
complete=false
status=0
trap 'status=$?; if [[ "$complete" == true ]]; then rm -rf "$fixture"; else printf "Macro diagnostic fixture retained: %s\n" "$fixture" >&2; [[ "$status" != 0 ]] || status=1; fi; exit "$status"' EXIT
cp "$root/Cargo.toml" "$root/Cargo.lock" "$root/rust-toolchain.toml" "$root/LICENSE" "$root/README.md" "$fixture/"
cp -R "$root/crates" "$fixture/"
mkdir "$fixture/docs"
cp "$root/docs/runtime.md" "$fixture/docs/"
source_file="$fixture/crates/ic-wire-canister-evaluation/src/macro_fixture/mod.rs"
cp "$source_file" "$fixture/original.rs"
check() {
    (cd "$fixture" && CARGO_TARGET_DIR="$fixture/target" cargo check --offline --locked -p ic-wire-canister-evaluation --lib --features macro-fixture) > "$fixture/$1.log" 2>&1
}
check valid
for case_name in request response async_response export_name candid_selector missing_guard async_query; do
    cp "$fixture/original.rs" "$source_file"
    expected='error[E0308]'
    case "$case_name" in
        request) perl -pi -e 's/correlation: u64/correlation: bool/; s/wrapping_add\(correlation\)/wrapping_add(0)/' "$source_file" ;;
        response) perl -0777 -pi -e 's/fn macro_read\(correlation: u64\) -> u64/fn macro_read(correlation: u64) -> bool/; s/transport::balance\(\).wrapping_add\(correlation\)/correlation == 0/' "$source_file" ;;
        async_response) perl -pi -e 's/async fn macro_add\(amount: u64\) -> Result<u64, u8>/async fn macro_add(amount: u64) -> Result<bool, u8>/; s/Ok\(balance\)/Ok(true)/' "$source_file" ;;
        export_name) perl -pi -e 's/"macro_read"/"wrong_export"/' "$source_file"; expected='method/export name mismatch' ;;
        candid_selector) perl -pi -e 's/query\(method = Read, guard = guard\)/query(method = Read, guard = guard, candid)/' "$source_file"; expected='expected method or guard; no codec selector' ;;
        missing_guard) perl -pi -e 's/query\(method = Read, guard = guard\)/query(method = Read)/' "$source_file"; expected='request guard required' ;;
        async_query) perl -pi -e 's/^fn macro_read/async fn macro_read/' "$source_file"; expected='async query is outside this fixture' ;;
    esac
    if check "$case_name"; then printf 'Expected compile failure: %s\n' "$case_name" >&2; exit 1; fi
    if ! grep -F "$expected" "$fixture/$case_name.log" >/dev/null; then cat "$fixture/$case_name.log" >&2; exit 1; fi
    printf 'Macro diagnostic passed: %s\n' "$case_name"
done
complete=true
