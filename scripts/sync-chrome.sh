#!/usr/bin/env bash
# REUSE the estate's generated design tokens; never hand-copy them into a product.
set -euo pipefail
cd "$(dirname "$0")/.."
estate="${ESTATE_DIR:-$HOME/code/estate}"
source_file="$estate/ui/dist/tokens.css"
test -f "$source_file" || { echo "sync-chrome: missing $source_file"; exit 1; }
mkdir -p frontend/static/estate
cp "$source_file" frontend/static/estate/tokens.css
echo "sync-chrome: PASS ($(wc -c < frontend/static/estate/tokens.css) bytes)"
