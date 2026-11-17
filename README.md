# Loom

A bundle-first, user-driven directory for ATProto custom feeds. See the [spec](https://github.com/) in the vault: `~/secondbrain/10-Projects/loom-spec.md`, plan: `loom-plan.md`.

One Rust workspace, one Postgres, one SvelteKit PWA. No feed generator, no post ranking, no accounts before the declared sunset trigger.

```bash
cargo check --workspace --locked
bash scripts/gate.sh repo
```
