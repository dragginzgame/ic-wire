#!/usr/bin/env bash
set -euo pipefail

# Compile production runtime derives and consumer-shaped adapters in isolation.
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
cp "$root/docs/runtime.md" "$fixture/docs/"
cp "$root/docs/evidence/codec-screen.json" "$fixture/docs/evidence/"
source_file="$fixture/crates/ic-wire/tests/runtime/mod.rs"
cat >> "$source_file" <<'RUST'

// Private consumer-shaped expansions: Canic owns the actual macros and policy.
fn adapter(
    bytes: &[u8],
    handler: impl FnOnce(<Read as Method>::Request) -> <Read as Method>::Response,
) -> Result<ic_wire::ResponseBytes<Read>, Error> {
    let request = decode_request::<Read>(bytes)?;
    encode_response::<Read>(&handler(request))
}
async fn async_adapter<F: core::future::Future<Output = <Read as Method>::Response>>(
    bytes: &[u8],
    handler: impl FnOnce(<Read as Method>::Request) -> F,
) -> Result<ic_wire::ResponseBytes<Read>, Error> {
    let request = decode_request::<Read>(bytes)?;
    encode_response::<Read>(&handler(request).await)
}
fn valid_adapters() {
    let _ = adapter(&[], |_| Ok(7));
    let _future = async_adapter(&[], |_| async { Ok(7) });
}
RUST
cp "$source_file" "$fixture/original.rs"
check() {
    (cd "$fixture" && CARGO_TARGET_DIR="$fixture/target" cargo check --offline --locked -p ic-wire --test runtime) > "$fixture/$1.log" 2>&1
}
# First prove the copied valid source compiles, so environmental failures cannot
# masquerade as expected diagnostics. All failing cases reuse these exact inputs.
check valid
for case_name in missing_id duplicate_field_id duplicate_variant_id unsupported_field generic_contract tuple_record wrong_request wrong_reply wrong_handler wrong_handler_argument wrong_async_handler; do
    cp "$fixture/original.rs" "$source_file"
    expected='error[E0308]'
    case "$case_name" in
        missing_id)
            printf '\n#[derive(Wire)] struct Invalid { value: u64 }\n' >> "$source_file"
            expected='explicit #[wire(id = N)] required'
            ;;
        duplicate_field_id)
            printf '\n#[derive(Wire)] struct Invalid { #[wire(id = 1)] a: u64, #[wire(id = 1)] b: bool }\n' >> "$source_file"
            expected='duplicate wire id'
            ;;
        duplicate_variant_id)
            printf '\n#[derive(Wire)] enum Invalid { #[wire(id = 1)] A, #[wire(id = 1)] B }\n' >> "$source_file"
            expected='duplicate wire id'
            ;;
        unsupported_field)
            printf '\n#[derive(Wire)] struct Invalid { #[wire(id = 1)] name: String }\n' >> "$source_file"
            expected='String: Wire` is not satisfied'
            ;;
        generic_contract)
            printf '\n#[derive(Wire)] struct Invalid<T> { #[wire(id = 1)] value: T }\n' >> "$source_file"
            expected='Wire requires concrete contracts'
            ;;
        tuple_record)
            printf '\n#[derive(Wire)] struct Invalid(u64);\n' >> "$source_file"
            expected='use a named-field record (empty records are allowed)'
            ;;
        wrong_request)
            printf '\nfn invalid() { let _ = encode_request::<Read>(&true); }\n' >> "$source_file"
            ;;
        wrong_reply)
            printf '\nfn invalid() { let _ = encode_response::<Read>(&7_u64); }\n' >> "$source_file"
            ;;
        wrong_handler)
            printf '\nfn invalid() { let _ = adapter(&[], |_| 7_u64); }\n' >> "$source_file"
            ;;
        wrong_handler_argument)
            printf '\nfn invalid() { let _ = adapter(&[], |_: bool| Ok(7)); }\n' >> "$source_file"
            expected='error[E0631]'
            ;;
        wrong_async_handler)
            printf '\nfn invalid() { let _future = async_adapter(&[], |_| async { 7_u64 }); }\n' >> "$source_file"
            expected='error[E0271]'
            ;;
    esac
    if check "$case_name"; then
        printf 'Expected compile failure: %s\n' "$case_name" >&2
        exit 1
    fi
    if ! grep -F "$expected" "$fixture/$case_name.log" >/dev/null; then
        cat "$fixture/$case_name.log" >&2
        printf 'Missing expected diagnostic for %s: %s\n' "$case_name" "$expected" >&2
        exit 1
    fi
    printf 'Expected diagnostic passed: %s\n' "$case_name"
done
complete=true
