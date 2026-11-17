#!/usr/bin/env bash
# Drive the TUI as a line script against a running Loom API.
set -euo pipefail
cd "$(dirname "$0")/.."
: "${LOOM_API:?set LOOM_API to the running API base, e.g. http://127.0.0.1:8079}"
out=$(printf 'create\nSession bundle\nat://did:plc:test/app.bsky.feed.generator/one\n' \
  | LOOM_API="$LOOM_API" cargo run -q -p loom-cli --bin loomctl -- tui)
printf '%s\n' "$out"
printf '%s' "$out" | grep -Eq '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}'
echo "tui-session: PASS"
