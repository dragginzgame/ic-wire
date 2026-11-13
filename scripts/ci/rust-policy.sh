#!/usr/bin/env bash
# Consumer policy admission; source in qualification scripts or execute a command.
# Setup remains explicit: this never installs or changes the default toolchain.
set -euo pipefail
_wire_rust_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
export PATH="$_wire_rust_root/.tools/host/bin:$PATH"
_wire_rust_floor="$(bash "$_wire_rust_root/vendor/toko-tooling/scripts/ci/read-cargo-workspace-rust-version.sh" "$_wire_rust_root/Cargo.toml")"
_wire_rust_pin="$(yq -p toml -o yaml -r '.toolchain.channel' "$_wire_rust_root/rust-toolchain.toml")"
[[ "$_wire_rust_floor" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ && "$_wire_rust_floor" == "$_wire_rust_pin" ]] || {
    echo 'Wire requires the same exact Rust floor and pinned toolchain' >&2; exit 1;
}
export RUSTUP_TOOLCHAIN="$_wire_rust_floor" RUSTUP_AUTO_INSTALL=0
_wire_rust_identity="$(rustc -vV)"
_wire_cargo_identity="$(cargo -V)"
[[ "$_wire_rust_identity" == "rustc $_wire_rust_floor ("* &&
   "$_wire_rust_identity" == *$'\n'"release: $_wire_rust_floor"$'\n'* &&
   "$_wire_cargo_identity" == "cargo $_wire_rust_floor ("* ]] || {
    echo 'Selected Cargo/rustc do not match Wire compiler policy' >&2; exit 1;
}
printf '%s\n%s\n' "$_wire_rust_identity" "$_wire_cargo_identity" >&2
if [[ "${BASH_SOURCE[0]}" == "$0" && $# -gt 0 ]]; then exec "$@"; fi
