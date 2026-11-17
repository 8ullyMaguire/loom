#!/usr/bin/env bash
# Verify the public hostname end to end, not just the local port.
set -euo pipefail
host="${LOOM_HOST:-loom.polarisocial.xyz}"
base="https://$host"
echo "checking $base"
opts=()
if ! getent hosts "$host" >/dev/null 2>&1; then
  ip=$(dig +short @1.1.1.1 "$host" A | head -1)
  test -n "$ip" || { echo "public-hostname: no DNS answer for $host"; exit 1; }
  opts=(--resolve "$host:443:$ip")
  echo "system resolver stale; verifying via public DNS $ip"
fi
get() { curl -fsS --max-time 20 "${opts[@]}" "$1"; }
get "$base/" | grep -q 'data-loom-ssr="1"'
get "$base/" | grep -q 'Worth a look'
get "$base/api/v1/capabilities" | python3 -c "import json,sys;d=json.load(sys.stdin);assert d['tui'] is True;print('public capabilities ok')"
get "$base/manifest.webmanifest" >/dev/null
echo "public-hostname: PASS ($base)"
