#!/usr/bin/env bash
set -euo pipefail

usage() {
    echo 'usage: prepare-rust-toolchain.sh TOOLCHAIN [--component NAME[,NAME]...] [--target TARGET]...' >&2
    exit 2
}

[[ $# -ge 1 && -n "$1" && "$1" != -* && "$1" != *[[:space:]]* ]] || usage
arguments=(toolchain install "$1" --profile minimal)
shift
while [[ $# -gt 0 ]]; do
    [[ $# -ge 2 && -n "$2" && "$2" != -* && "$2" != *[[:space:]]* ]] || usage
    case "$1" in
        --component)
            [[ "$2" != ,* && "$2" != *, && "$2" != *,,* ]] || usage
            ;;
        --target) ;;
        *) usage ;;
    esac
    arguments+=("$1" "$2")
    shift 2
done
# Admit the complete request before performing any installation. Never change
# the default toolchain or turn a missing validation prerequisite into setup.
exec rustup "${arguments[@]}"
