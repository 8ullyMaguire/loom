#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
case "${1:-repo}" in
  repo)
    test -f Cargo.toml && test -f docs/STATE.md && test -f docs/ROADMAP.md
    echo "repo: ok"
    ;;
  *)
    echo "gate: unknown stage $1" >&2
    exit 2
    ;;
esac
