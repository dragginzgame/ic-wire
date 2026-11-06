#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
[[ $# == 1 ]] || { echo 'usage: prepare-browser-tools.sh prepare|check' >&2; exit 2; }
case "$1" in prepare|check) ;; *) echo 'select prepare or check' >&2; exit 2 ;; esac
[[ "$(node --version)" == "v$(cat examples/browser/.nvmrc)" ]] || { echo 'Select the pinned browser Node version' >&2; exit 1; }
[[ "npm@$(npm --version)" == "$(node -p 'require("./examples/browser/package.json").packageManager')" ]] || { echo 'Select the pinned browser npm version' >&2; exit 1; }
if [[ "$1" == prepare ]]; then
    npm --prefix examples/browser ci --ignore-scripts --no-audit --no-fund
else
    # Recovery admits the saved installation without fetching or reinstalling.
    npm --prefix examples/browser ls --offline --depth=0
fi
expected="$(node -p 'require("./examples/browser/package.json").devDependencies.typescript')"
[[ "$(examples/browser/node_modules/.bin/tsc --version)" == "Version $expected" ]] || { echo 'Prepare the locked browser TypeScript compiler' >&2; exit 1; }
