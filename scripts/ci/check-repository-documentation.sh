#!/usr/bin/env bash
# One documentation owner: maintained Markdown plus the exact Shared export.
set -euo pipefail
script="${BASH_SOURCE[0]}"
[[ "$script" == /* ]] || script="$PWD/$script"
[[ "$script" != *$'\n'* && "$script" != *$'\r'* ]] || { echo 'Unsupported script path' >&2; exit 2; }
root="$(cd "${script%/*}/../.." && pwd -P)"
[[ "$root" != *$'\n'* && "$root" != *$'\r'* ]] || { echo 'Unsupported repository path' >&2; exit 2; }
cd "$root"
mkdir -p target
inventory="$(mktemp "$root/target/documentation-inventory.XXXXXX")"
selection="$(mktemp "$root/target/documentation-selection.XXXXXX")"
trap 'rm -f "$inventory" "$selection"' EXIT
bash scripts/ci/verify-toko-tooling-snapshot.sh --read-manifest .toko-tooling.snapshot > "$inventory"
while IFS=$'\t' read -r kind _ _ file; do
    [[ "$kind" != file || "$file" != *.md ]] || printf '%s\n' "$file" >> "$selection"
done < "$inventory"
git ls-files --cached --others --exclude-standard -z -- AGENTS.md README.md CONTRIBUTING.md LICENSE.md CHANGELOG.md docs releases crates examples qualification > "$inventory"
while IFS= read -r -d '' file; do
    [[ "$file" == *.md ]] || continue
    case "$file" in *$'\n'*|*$'\r'*|*$'\t'*) echo 'Unsupported documentation path' >&2; exit 1 ;; esac
    [[ ! -L "$file" ]] || { echo "Redirected documentation: $file" >&2; exit 1; }
    [[ ! -f "$file" ]] || printf '%s\n' "$file" >> "$selection"
done < "$inventory"
LC_ALL=C sort -u "$selection" > "$inventory"
documents=()
while IFS= read -r file; do documents+=("$file"); done < "$inventory"
[[ "${#documents[@]}" -gt 0 ]] || { echo 'No maintained documentation selected' >&2; exit 1; }
perl scripts/ci/check-documentation-links.pl --root "$root" "${documents[@]}"
