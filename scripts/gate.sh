#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
case "${1:-repo}" in
  ramp)
    cargo test -q -p loom-api --test gravity_gates ramp -- --exact --nocapture >/dev/null
    echo "ramp: PASS"
    ;;
  secrecy)
    cargo test -q -p loom-api --test gravity_gates secrecy -- --exact --nocapture >/dev/null
    echo "secrecy: PASS"
    ;;
  repo)
    test -f Cargo.toml && test -f docs/STATE.md && test -f docs/ROADMAP.md
    echo "repo: ok"
    ;;
  *)
    echo "gate: unknown stage $1" >&2
    exit 2
    ;;
esac
