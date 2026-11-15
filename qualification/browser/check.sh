#!/usr/bin/env bash
# Run clients emitted by the package consumer in an explicitly selected browser.
set -euo pipefail
client=${1:?provide generated client directory}
browser=${2:?provide prepared Chromium executable}
test -x "$browser"
node qualification/typescript/node_modules/typescript/bin/tsc \
    --strict --noUncheckedIndexedAccess --exactOptionalPropertyTypes \
    --target ES2022 --module ES2022 --lib ES2022,DOM \
    --rewriteRelativeImportExtensions --outDir "$client/browser" \
    "$client/runtime.ts" "$client/payload.ts" "$client/methods.ts"
cp qualification/browser/smoke.html "$client/browser/index.html"
url=$(node -e 'console.log(require("node:url").pathToFileURL(process.argv[1]).href)' "$client/browser/index.html")
"$browser" --version
"$browser" --headless --no-sandbox --disable-gpu --allow-file-access-from-files \
    --user-data-dir="$client/browser-profile" --dump-dom --timeout=10000 \
    --virtual-time-budget=5000 "$url" > "$client/browser/result.html" 2> "$client/browser/stderr.log"
if ! rg -q '<body data-result="pass">' "$client/browser/result.html"; then
    cat "$client/browser/result.html"
    cat "$client/browser/stderr.log" >&2
    exit 1
fi
printf 'Browser archive-consumer smoke passed: %s\n' "$client/browser/result.html"
