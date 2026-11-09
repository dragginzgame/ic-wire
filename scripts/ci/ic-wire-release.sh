#!/usr/bin/env bash
set -euo pipefail
# Local policy only; the shared runner owns all staging, commit, tag and push effects.
operation="${1:?release adapter operation}"
# Preflight admits these selected tools before metadata and validation children.
export PATH="$PWD/.tools/host/bin:$PWD/.tools/rust/common/bin:$PATH"
case "$operation" in
    version|files) ;;
    *) bash scripts/dev/toko-tooling.sh check --consumer "$PWD" ;;
esac
: "${RELEASE_GROUP:?select RELEASE_GROUP=core or endpoint explicitly}"
export RELEASE_GROUP
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
    elif [[ "$1" == prepare ]]; then
        bash scripts/ci/prepare-wire-metadata.sh
    else
        bash scripts/ci/cargo-release-metadata.sh "$1" "$primary"
    fi
}
source_check() {
    bash scripts/ci/check-release-source.sh "${source_args[@]}"
}
validation_identity() {
    release_receipt_identity || return
    [[ -f "$state/validation-proof" && ! -L "$state/validation-proof" ]] || return 1
    local proof
    proof="$(cat "$state/validation-proof")" || return
    [[ "$proof" =~ ^run\.[A-Za-z0-9]+$ ]] || return 1
    node scripts/ci/wire-release-validation.mjs check "$RELEASE_GROUP" "$RELEASE_SOURCE" "$state/proofs/$proof"
}
prepared() {
    release_receipt_check "$state/validated" validation_identity || fail 'validation receipt missing or changed'
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
        # Captured groups were prepared by Shared before version lookup; exact
        # standard-runner recovery retains its existing preparation contract.
        if [[ "${RELEASE_BATCH_ACTIVE:-}" != 1 ]]; then
            bash scripts/ci/cargo-release-preflight.sh "$PWD" "${RELEASE_CACHE_PREPARE:-1}" \
                --pin Cargo.lock --pin qualification/Cargo.lock --pin ci/tool-versions.env --pin ci/ic-tools.tsv \
                --prepare install-tools --prepare tools-check --prepare install-release-editor --prepare release-editor-check \
                --check tools-check --check release-editor-check
        fi
        metadata check-graph
        ;;
    verify)
        bash scripts/ci/check-release-source.sh
        mkdir -p "$state"
        release_receipt_archive "$state/prior-validation.XXXXXX" "$state/validated" "$state/validation-proof"
        rm -f -- "$state/validated" "$state/validation-proof"
        mkdir -p target/release-validation "$state/proofs"
        # Replaces the former shared ci gate with a source-bound plan; old whole-gate
        # receipts cannot satisfy the new per-stage proof or recovery checks.
        attempt="$(mktemp -d "$PWD/target/release-validation/run.XXXXXX")"
        node scripts/ci/wire-release-validation.mjs select "$RELEASE_GROUP" "$RELEASE_SOURCE" "$attempt"
        # A widened plan must lint both groups; narrow plans and publication
        # pass their selected group through to the one public lint owner.
        export WIRE_LINT_GROUP="$RELEASE_GROUP"
        if [[ "$(jq -r '.scope' "$attempt/plan.json")" == workspace ]]; then
            export WIRE_LINT_GROUP=''
        fi
        export VALIDATION_LOG_DIR="$attempt/logs" CARGO_NET_OFFLINE=true
        bash scripts/ci/run-validation-targets.sh --plan "${attempt#$PWD/}/plan.json"
        bash scripts/ci/check-release-source.sh
        proof="${attempt##*/}"
        node scripts/ci/wire-release-validation.mjs seal "$RELEASE_GROUP" "$RELEASE_SOURCE" "$attempt" "$state/proofs/$proof"
        release_receipt_write "$state/validation-proof" printf '%s\n' "$proof" || fail 'cannot retain proof selection'
        release_receipt_write "$state/validated" validation_identity || fail 'cannot retain validation receipt'
        ;;
    prepare)
        release_receipt_check "$state/validated" validation_identity || fail 'validation receipt missing or changed'
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
        exclusions=(.)
        for path in "${files[@]}"; do exclusions+=(":(exclude,literal)$path"); done
        git diff --quiet "$RELEASE_SOURCE" "$RELEASE_COMMIT" -- "${exclusions[@]}" || fail 'release contains unrelated source'
        if [[ "$operation" != committed ]]; then
            bash scripts/ci/check-release-tag.sh "$RELEASE_COMMIT" "$RELEASE_VERSION"
        fi
        ;;
    *) fail "unknown adapter operation: $operation" ;;
esac
