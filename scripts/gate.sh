#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
case "${1:-repo}" in
  fmt)
    cargo fmt --all -- --check
    echo "fmt: ok"
    ;;
  clippy)
    cargo clippy --workspace --all-targets -- -D warnings
    echo "clippy: ok"
    ;;
  schema)
    cargo test -q -p loom-store --test schema
    echo "schema: ok"
    ;;
  core)
    cargo test -q -p loom-core --test rules
    echo "core: ok"
    ;;
  store)
    cargo test -q -p loom-store --test bundles --test bundle_store
    echo "store: ok"
    ;;
  api)
    cargo test -q -p loom-api --test bundle_share --test tag_aggregate --test ingest       --test search --test gravity_gates --test sunset --test openapi
    echo "api: ok"
    ;;
  cli)
    cargo test -q -p loom-cli --test exit_codes
    echo "cli: ok"
    ;;
  openapi)
    bash scripts/check-openapi.sh
    ;;
  all)
    for stage in fmt clippy repo schema core store api cli frontend secrecy ramp openapi; do
      bash scripts/gate.sh "$stage" || exit 1
    done
    echo "GATE_ALL_EXIT=0"
    ;;
  frontend)
    bash scripts/sync-chrome.sh >/dev/null
    (cd frontend && node node_modules/vite/bin/vite.js build >/dev/null 2>&1)
    bash scripts/blank-canary.sh
    ;;
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
