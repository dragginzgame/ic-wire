#!/usr/bin/env bash
set -euo pipefail
# Local policy only; the shared runner owns all staging, commit, tag and push effects.
operation="${1:?release adapter operation}"
files=(Cargo.toml Cargo.lock CHANGELOG.md)
fail() { echo "IC Wire release refused: $*" >&2; exit 1; }
[[ "${RELEASE_DELIVERY:-direct}" == direct ]] || fail 'only direct delivery is configured'
if [[ "$operation" == version ]]; then
    exec bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml
fi
if [[ "$operation" == files ]]; then
    printf '%s\0' "${files[@]}"
    exit
fi
perl scripts/ci/validate-release-inputs.pl version "${RELEASE_VERSION:?}" >/dev/null
perl scripts/ci/validate-release-inputs.pl date "${RELEASE_DATE:?}" >/dev/null
[[ "${RELEASE_SOURCE:?}" =~ ^([0-9a-f]{40}|[0-9a-f]{64})$ ]] || fail 'invalid source identity'
state="$(git rev-parse --git-path release-state)/ic-wire-$RELEASE_VERSION"
identity() {
    printf '%s\n' "$RELEASE_SOURCE" "$RELEASE_PREVIOUS" "$RELEASE_VERSION" "$RELEASE_DATE" "$RELEASE_REMOTE" "$RELEASE_BRANCH"
    git remote get-url --push --all "$RELEASE_REMOTE"
}
evidence() {
    [[ -f "$state/validated" && ! -L "$state/validated" ]] || fail 'missing validation receipt'
    [[ "$(cat "$state/validated")" == "$(identity)" ]] || fail 'validation identity changed'
}
metadata() { bash scripts/ci/cargo-release-metadata.sh "$1" ic-wire; }
source_check() {
    bash scripts/ci/check-release-source.sh --allow Cargo.toml --allow Cargo.lock --allow CHANGELOG.md
}
payload() {
    local path
    for path in "${files[@]}"; do
        if [[ -n "${RELEASE_COMMIT:-}" ]]; then
            git rev-parse "$RELEASE_COMMIT:$path"
        else
            git hash-object -- "$path"
        fi
    done
}
prepared() {
    evidence
    metadata check
    [[ -f "$state/prepared" && "$(cat "$state/prepared")" == "$(payload)" ]] || fail 'prepared bytes differ from receipt'
}
case "$operation" in
    preflight)
        bash scripts/ci/check-release-source.sh
        [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" ]] || fail 'source changed'
        [[ "$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)" == "$RELEASE_PREVIOUS" ]] || fail 'previous version changed'
        awk -v version="$RELEASE_VERSION" -v previous="$RELEASE_PREVIOUS" -v date="$RELEASE_DATE" \
            -f scripts/ci/finalize-release-changelog.awk CHANGELOG.md >/dev/null
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
        identity > "$state/validated.tmp"
        mv "$state/validated.tmp" "$state/validated"
        ;;
    prepare)
        evidence
        bash scripts/ci/check-release-source.sh
        metadata prepare
        payload > "$state/prepared.tmp"
        mv "$state/prepared.tmp" "$state/prepared"
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
        git diff --quiet "$RELEASE_SOURCE" "$RELEASE_COMMIT" -- . ':!Cargo.toml' ':!Cargo.lock' ':!CHANGELOG.md' || fail 'release contains unrelated source'
        if [[ "$operation" != committed ]]; then
            bash scripts/ci/check-release-tag.sh "$RELEASE_COMMIT" "$RELEASE_VERSION"
        fi
        ;;
    *) fail "unknown adapter operation: $operation" ;;
esac
