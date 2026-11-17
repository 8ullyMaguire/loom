#!/usr/bin/env bash
# The committed contract must match the code, and any change needs a version bump.
set -euo pipefail
cd "$(dirname "$0")/.."
test -f openapi.json || { echo "openapi: missing committed openapi.json"; exit 1; }
test -f openapi.version || { echo "openapi: missing openapi.version"; exit 1; }
version=$(tr -d '[:space:]' < openapi.version)
current=$(mktemp)
trap 'rm -f "$current"' EXIT
cargo run -q -p loom-api --bin loom-server -- --print-openapi > "$current"
python3 - "$current" openapi.json "$version" <<'PY'
import json, sys
new = json.load(open(sys.argv[1]))
old = json.load(open(sys.argv[2]))
version = sys.argv[3]
if new == old:
    if new["info"]["version"] != version:
        print("openapi: info.version and openapi.version disagree"); raise SystemExit(1)
    print(f"openapi: PASS (unchanged, version {version})"); raise SystemExit(0)
if new.get("info", {}).get("version") == old.get("info", {}).get("version"):
    print("openapi: contract changed without a version bump"); raise SystemExit(1)
if new["info"]["version"] != version:
    print("openapi: version bumped in document but not in openapi.version"); raise SystemExit(1)
print(f"openapi: PASS (version {old['info']['version']} -> {version})")
PY
