#!/usr/bin/env bash
# Refuse a deployed-looking frontend whose first paint is empty.
set -euo pipefail
cd "$(dirname "$0")/.."
root="${1:-frontend/build}"
page="$root/index.html"
test -f "$page" || { echo "blank-canary: no $page"; exit 1; }
test -s "$page" || { echo "blank-canary: empty $page"; exit 1; }
grep -q 'data-loom-ssr="1"' "$page" || { echo "blank-canary: no SSR marker in $page"; exit 1; }
grep -q '<h1' "$page" || { echo "blank-canary: no heading in $page"; exit 1; }
grep -q 'Worth a look' "$page" || { echo "blank-canary: no recommendation surface in $page"; exit 1; }
if [ -f "$root/200.html" ]; then
  test -s "$root/200.html" || { echo "blank-canary: empty fallback"; exit 1; }
  grep -q 'data-loom-ssr="1"' "$root/200.html" || { echo "blank-canary: no marker in fallback"; exit 1; }
fi
echo "blank-canary: PASS ($page $(wc -c < "$page") bytes)"
