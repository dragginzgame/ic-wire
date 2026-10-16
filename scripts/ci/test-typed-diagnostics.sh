#!/usr/bin/env bash
set -euo pipefail

# Compile the actual private prototype with invalid callers in an isolated copy.
# Requires the selected Rust toolchain and already fetched locked dependencies.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-wire-diagnostics.XXXXXX")"
fixture="$(cd "$fixture" && pwd -P)"
complete=false
status=0
trap 'status=$?; if [[ "$complete" == true ]]; then rm -rf "$fixture"; else printf "Diagnostic fixture retained: %s\n" "$fixture" >&2; [[ "$status" != 0 ]] || status=1; fi; exit "$status"' EXIT
cp "$root/Cargo.toml" "$root/Cargo.lock" "$root/rust-toolchain.toml" "$root/LICENSE" "$root/README.md" "$fixture/"
cp -R "$root/crates" "$fixture/"
mkdir -p "$fixture/docs/evidence"
cp "$root/docs/evidence/codec-screen.json" "$fixture/docs/evidence/"
source_file="$fixture/crates/ic-wire/examples/typed_journey/mod.rs"
cp "$source_file" "$fixture/original.rs"
check() {
    (cd "$fixture" && CARGO_TARGET_DIR="$fixture/target" cargo check --offline --locked -p ic-wire --example typed-journey) > "$fixture/$1.log" 2>&1
}
# First prove the copied valid source compiles, so environmental failures cannot
# masquerade as expected diagnostics. All failing cases reuse these exact inputs.
check valid
for case_name in unsupported_field wrong_request wrong_reply wrong_handler; do
    cp "$fixture/original.rs" "$source_file"
    expected='error[E0308]'
    case "$case_name" in
        unsupported_field)
            printf '\nwire_record! { struct Invalid { name: String } }\n' >> "$source_file"
            expected='wire_record supports only u64, u32 and bool fields'
            ;;
        wrong_request)
            printf '\nfn invalid() { let _ = BalanceMethod::prepare(&Outcome::NotFound); }\n' >> "$source_file"
            ;;
        wrong_reply)
            printf '\nfn invalid() { let reply = encode(1, &Outcome::NotFound).unwrap(); let _ = BalanceMethod::call(&reply, |_, _| Err(Error::Transport)); }\n' >> "$source_file"
            ;;
        wrong_handler)
            printf '\nfn invalid() { let _ = BalanceMethod::dispatch(&[], |_| 7u64); }\n' >> "$source_file"
            ;;
    esac
    if check "$case_name"; then
        printf 'Expected compile failure: %s\n' "$case_name" >&2
        exit 1
    fi
    grep -F "$expected" "$fixture/$case_name.log" >/dev/null
    printf 'Expected diagnostic passed: %s\n' "$case_name"
done
complete=true
