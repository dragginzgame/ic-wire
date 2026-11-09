#!/usr/bin/env bash
# Wire's extra lock participates in Shared's atomic metadata transaction.
# Shared admits the group/paths; Build renders manifests and both lockfiles.
set -euo pipefail
: "${RELEASE_GROUP:?}" "${RELEASE_PREVIOUS:?}" "${RELEASE_VERSION:?}" "${RELEASE_DATE:?}"
: "${CARGO_RELEASE_METADATA_EDITOR:?}" "${CARGO_RELEASE_METADATA_MAX_BYTES:?}"
# Preparation always addresses the live candidate, never an older receipt commit.
export RELEASE_COMMIT=''
source scripts/ci/release-metadata-transaction.sh
plan="$(bash scripts/ci/cargo-release-group.sh plan "$RELEASE_GROUP")"
[[ "$(jq -r .previous <<< "$plan")" == "$RELEASE_PREVIOUS" ]]
[[ "$(perl scripts/ci/compare-stable-versions.pl "$RELEASE_VERSION" "$RELEASE_PREVIOUS")" == 1 ]]
# Refuse a future additional metadata owner until this renderer handles it.
jq -e '.metadata_files == ["qualification/Cargo.lock"]' <<< "$plan" >/dev/null
files=() packages=()
while IFS= read -r file; do files+=("$file"); done < <(jq -r '.files[]' <<< "$plan")
while IFS= read -r package; do packages+=("$package"); done < <(jq -r '.members[]' <<< "$plan")
changelog="$(jq -r .changelog <<< "$plan")"

render_wire_metadata() {
    local original="$1" candidate="$2" manifest recipe
    while IFS= read -r manifest; do
        recipe="$candidate/group-edit.json"
        jq --arg path "$manifest" '.manifests[] | select(.path == $path) | {version,dependencies}' \
            <<< "$plan" > "$recipe" || return
        "$CARGO_RELEASE_METADATA_EDITOR" --max-input-bytes "$CARGO_RELEASE_METADATA_MAX_BYTES" \
            group-manifest "$original/$manifest" "$RELEASE_PREVIOUS" "$RELEASE_VERSION" "$recipe" \
            > "$candidate/$manifest" || return
    done < <(jq -r '.manifests[].path' <<< "$plan")
    for manifest in Cargo.lock qualification/Cargo.lock; do
        "$CARGO_RELEASE_METADATA_EDITOR" --max-input-bytes "$CARGO_RELEASE_METADATA_MAX_BYTES" \
            lock "$original/$manifest" "$RELEASE_PREVIOUS" "$RELEASE_VERSION" "${packages[@]}" \
            > "$candidate/$manifest" || return
    done
    awk -v version="$RELEASE_VERSION" -v previous="$RELEASE_PREVIOUS" -v date="$RELEASE_DATE" \
        -f scripts/ci/finalize-release-changelog.awk "$original/$changelog" > "$candidate/$changelog"
}

check_wire_metadata() {
    bash scripts/ci/cargo-release-group.sh check "$RELEASE_GROUP" || return
    cargo metadata --manifest-path qualification/Cargo.toml --locked --offline --format-version 1 >/dev/null
}

# Shared owns backups, complete candidate admission, installation and rollback.
release_metadata_prepare "${TMPDIR:-/tmp}/wire-metadata.XXXXXX" \
    render_wire_metadata check_wire_metadata "${files[@]}"
