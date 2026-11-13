#!/usr/bin/env bash
# One public lint owner for standalone, aggregate, release and publication calls.
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)/rust-policy.sh"
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
case "${WIRE_LINT_GROUP-${RELEASE_GROUP:-}}" in
    core)
        packages=(-p ic-wire -p ic-wire-derive)
        schema=ic-wire/typescript
        ;;
    endpoint)
        packages=(-p ic-wire-endpoint-macros -p ic-wire-endpoint)
        schema=ic-wire-endpoint/typescript
        ;;
    '')
        packages=(-p ic-wire -p ic-wire-derive -p ic-wire-endpoint-macros -p ic-wire-endpoint)
        schema=ic-wire-endpoint/typescript
        ;;
    *) echo 'Select an existing lint group: core or endpoint' >&2; exit 2 ;;
esac
# Human-readable Cargo output: deny fresh and replayed warnings without changing
# rustc flags/cache identities. Plain JSON collectors retain their own admission.
CARGO_BUILD_WARNINGS=deny cargo clippy --locked "${packages[@]}" --all-targets --features "$schema"
