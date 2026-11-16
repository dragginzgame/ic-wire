#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."
unsorted="$(mktemp "${TMPDIR:-/tmp}/wire-hook-manifest.XXXXXX")"
trap 'rm -f "$unsorted"' EXIT
# Exercise real dependency sorting without changing dependency selections.
perl -0pe 's/(proc-macro2[^\n]*\n)(quote[^\n]*\n)/$2$1/' \
    crates/ic-wire-derive/Cargo.toml > "$unsorted"
bash vendor/toko-tooling/scripts/ci/check-formatting-hooks.sh "$PWD" \
    crates/ic-wire-derive/src/lib.rs crates/ic-wire-derive/Cargo.toml \
    "$unsorted" --rust-sources rust-toolchain.toml \
    Cargo.lock qualification/Cargo.lock
