#!/usr/bin/env bash
# Source-only receipt mechanics. Caller owns policy, trusted state directory,
# selected files and exclusive source/state custody throughout release delivery.

release_receipt_identity() {
    printf '%s\n' "${RELEASE_SOURCE:?}" "${RELEASE_PREVIOUS:?}" "${RELEASE_VERSION:?}" \
        "${RELEASE_DATE:?}" "${RELEASE_REMOTE:?}" "${RELEASE_BRANCH:?}" || return
    git remote get-url --push --all "$RELEASE_REMOTE"
}

release_receipt_payload() {
    local commit="$1" path
    shift
    [[ $# -gt 0 ]] || return 1
    for path in "$@"; do
        if [[ -n "$commit" ]]; then
            git rev-parse --verify "$commit:$path" || return
        else
            git hash-object -- "$path" || return
        fi
    done
}

# Generate into a private adjacent temporary file; never clobber a receipt if
# generation fails, and never follow an existing receipt symlink.
release_receipt_write() {
    local receipt="$1" temporary
    shift
    [[ ! -L "$receipt" && ( ! -e "$receipt" || -f "$receipt" ) ]] || return 1
    temporary="$(mktemp "$receipt.XXXXXX")" || return
    if "$@" > "$temporary" && mv -f -- "$temporary" "$receipt"; then
        return 0
    fi
    rm -f -- "$temporary"
    return 1
}

# Exact byte comparison also rejects changed trailing newlines and symlinks.
release_receipt_check() {
    local receipt="$1" temporary result=1
    shift
    [[ -f "$receipt" && ! -L "$receipt" ]] || return 1
    temporary="$(mktemp "$receipt.XXXXXX")" || return
    if "$@" > "$temporary" && cmp -s -- "$temporary" "$receipt"; then
        result=0
    fi
    rm -f -- "$temporary" || return
    return "$result"
}
