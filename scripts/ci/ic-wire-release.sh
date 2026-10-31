#!/usr/bin/env bash
set -euo pipefail
# Local policy only; the shared runner owns all staging, commit, tag and push effects.
operation="${1:?release adapter operation}"
# Preflight admits these selected tools before metadata and validation children.
export PATH="$PWD/.tools/host/bin:$PWD/.tools/rust/common/bin:$PATH"
export RELEASE_GROUP="${RELEASE_GROUP:-core}"
plan="$(bash scripts/ci/cargo-release-group.sh plan "$RELEASE_GROUP")"
primary="$(printf '%s' "$plan" | jq -er .primary)"
changelog="$(printf '%s' "$plan" | jq -er .changelog)"
files=()
while IFS= read -r -d '' path; do files+=("$path"); done < <(bash scripts/ci/cargo-release-group.sh files "$RELEASE_GROUP")
source_args=()
for path in "${files[@]}"; do source_args+=(--allow "$path"); done
# shellcheck source=scripts/ci/release-receipts.sh
source scripts/ci/release-receipts.sh
fail() { echo "IC Wire release refused: $*" >&2; exit 1; }
[[ "${RELEASE_DELIVERY:-direct}" == direct ]] || fail 'only direct delivery is configured'
if [[ "$operation" == version ]]; then
    exec bash scripts/ci/cargo-release-group.sh version "$RELEASE_GROUP"
fi
if [[ "$operation" == files ]]; then
    printf '%s\0' "${files[@]}"
    exit
fi
perl scripts/ci/validate-release-inputs.pl version "${RELEASE_VERSION:?}" >/dev/null
perl scripts/ci/validate-release-inputs.pl date "${RELEASE_DATE:?}" >/dev/null
[[ "${RELEASE_SOURCE:?}" =~ ^([0-9a-f]{40}|[0-9a-f]{64})$ ]] || fail 'invalid source identity'
state="$(git rev-parse --git-path release-state)/$primary-$RELEASE_VERSION"
metadata() {
    if [[ "$1" == prepare ]]; then
        CARGO_RELEASE_METADATA_EDITOR="$("${RELEASE_MAKE:-make}" --no-print-directory -s release-editor-check)" || return
        export CARGO_RELEASE_METADATA_EDITOR
    fi
    if [[ "$1" == check ]]; then
        bash scripts/ci/cargo-release-metadata.sh check "$primary" --manifest-order-grouped
    else
        bash scripts/ci/cargo-release-metadata.sh "$1" "$primary"
    fi
}
source_check() {
    bash scripts/ci/check-release-source.sh "${source_args[@]}"
}
prepared() {
    release_receipt_check "$state/validated" release_receipt_identity || fail 'validation receipt missing or changed'
    metadata check
    release_receipt_check "$state/prepared" release_receipt_payload "${RELEASE_COMMIT:-}" "${files[@]}" || fail 'prepared bytes differ from receipt'
}
case "$operation" in
    preflight)
        bash scripts/ci/check-release-source.sh
        [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" ]] || fail 'source changed'
        [[ "$(bash scripts/ci/cargo-release-group.sh version "$RELEASE_GROUP")" == "$RELEASE_PREVIOUS" ]] || fail 'previous version changed'
        awk -v version="$RELEASE_VERSION" -v previous="$RELEASE_PREVIOUS" -v date="$RELEASE_DATE" \
            -f scripts/ci/finalize-release-changelog.awk "$changelog" >/dev/null
        bash scripts/ci/cargo-release-preflight.sh "$PWD" "${RELEASE_CACHE_PREPARE:-1}" \
            --pin Cargo.lock --pin ci/tool-versions.env --pin ci/ic-tools.tsv \
            --prepare install-tools --prepare tools-check --check tools-check
        metadata check-graph
        ;;
    verify)
        bash scripts/ci/check-release-source.sh
        mkdir -p "$state"
        export VALIDATION_LOG_DIR="$state/validation"
        export CARGO_NET_OFFLINE=true
        # Same complete scaffold gate for every release kind; no publication.
        bash scripts/ci/run-validation-targets.sh --fail-fast ci
        bash scripts/ci/check-release-source.sh
        release_receipt_write "$state/validated" release_receipt_identity || fail 'cannot retain validation receipt'
        ;;
    prepare)
        release_receipt_check "$state/validated" release_receipt_identity || fail 'validation receipt missing or changed'
        bash scripts/ci/check-release-source.sh
        metadata prepare
        release_receipt_write "$state/prepared" release_receipt_payload '' "${files[@]}" || fail 'cannot retain prepared receipt'
        ;;
    prepared)
        source_check
        prepared
        ;;
    commit)
        source_check
        prepared
        git diff --quiet || fail 'index does not match prepared worktree'
        ;;
    committed|tagged|push)
        [[ -n "${RELEASE_COMMIT:-}" ]] || fail 'missing selected release commit'
        prepared
        exclusions=()
        for path in "${files[@]}"; do exclusions+=(":!$path"); done
        git diff --quiet "$RELEASE_SOURCE" "$RELEASE_COMMIT" -- . "${exclusions[@]}" || fail 'release contains unrelated source'
        if [[ "$operation" != committed ]]; then
            bash scripts/ci/check-release-tag.sh "$RELEASE_COMMIT" "$RELEASE_VERSION"
        fi
        ;;
    *) fail "unknown adapter operation: $operation" ;;
esac
