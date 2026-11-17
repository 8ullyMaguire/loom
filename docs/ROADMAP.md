# Loom — roadmap

One row per task. A row closes on its own gate output, pasted in the commit message.

| row | item | status | evidence |
|---|---|---|---|
| 0 | spec + plan | CLOSED | vault dffe25d |
| 1 | repo skeleton, gates, state notes | CLOSED | fc8f780: cargo check green, gate repo ok |
| 2 | database schema and store tests | CLOSED | 3 tests green: 9 tables, idempotent, duplicate position rejected |
| 3 | pure domain rules (loom-core) | CLOSED | fc8f780: 4 tests green (RED observed first) |
| 4 | bundle store + anonymous create + edit token | CLOSED | 2 tests green: order kept, wrong/rotated token refused; API routes still row 5 |
| 5 | typed API, OpenAPI, bundle routes | CLOSED | bundle_share 1 passed; openapi 1 passed; /b/{id} renders one Bluesky link per feed; generated paths 2, schemas 5 |
| 6 | tag votes and confidence floor | CLOSED | tag_aggregate 1 passed; RED was 404; mutant floor=4 reddened it, restored |
| 7 | search, facets, cursor, hidden gems | OPEN | — |
| 8 | read-only ingest from both generators | CLOSED | ingest 2 passed; recorded 14+12 live feeds, 26 indexed; PK conflict idempotent; 1 retired on removal; 0 PDS write verbs |
| 9 | recommendations + hidden gravity gates | OPEN | — |
| 10 | sunset rule and capability document | CLOSED | sunset 1 passed; 19 reports -> 201, 20 -> 401 auth_required, old token PATCH -> 200; `>=` mutant reddened it |
| 11 | CLI and TUI | CLOSED | exit_codes 1 passed (0..5 plus scripted TUI); 404->1 mutant reddened it; tui-session.sh added; capabilities now reports tui=true |
| 12 | SvelteKit PWA with real HTML | CLOSED | blank-canary PASS (3039 bytes, exact marker, heading, recommendation strip); service worker + manifest wired; exact-marker mutant reddened the canary |
| 13 | gate, CI, OpenAPI diff | CLOSED | `bash scripts/gate.sh all` -> GATE_ALL_EXIT=0 across 12 stages; CI workflow runs the same gate; OpenAPI contract mutant reddened the check |
| 14 | deploy on thinkcentre | CLOSED locally | backup PASS, 26 feeds ingested, app on 8079, nginx 8082, live HTML/API/share/PWA verified; public tunnel hostname still needs a Cloudflare dashboard change |

## Census log

No user-facing surface built yet. Every new surface must record REUSE / ADAPT / BUILD here before code.

- Row 1 REUSE Cargo workspace pattern from darkhold; ADAPT gate.sh single-command pattern; BUILD nothing new.
- Row 2 REUSE the existing lh-pg-test Postgres cluster (new dedicated database); ADAPT scratch-schema test pattern; BUILD the nine-table schema.
- Row 4 REUSE loom-core Bundle validation and token generation; ADAPT the estate idempotency convention; BUILD bundle persistence and token rotation.
- Row 5 REUSE axum/utoipa typed-handler pattern and loom-store bundle functions; ADAPT the estate error-body shape; BUILD the share page and OpenAPI document. First user-facing surface proven with a real HTTP request.
- Row 6 REUSE loom-core confidence aggregate and the sunset helper; ADAPT the `X-Visitor-Key` anonymous identity pattern; BUILD tag vote/aggregate routes. No new table (schema already had `tag_votes`).
- Row 8 REUSE the generators' public `/feeds.json` and their recorded JSON as fixtures; ADAPT upsert + retire to `feeds`; BUILD a GET-only ingest module. Added migration 0002 for `window_hours` after a real 42703 failure.
- Row 7 REUSE live `feeds` and `tag_votes` rows plus the existing typed error shape; ADAPT keyset paging; BUILD faceted search, deterministic ordering and a measured hidden-gems tail. No gravity exposed.
- Row 9 REUSE loom-core `ramp`/`score`, live tag votes and search results; ADAPT per-surface ceilings from `config.toml`; BUILD the recommendations route, `docs/ROUTE_TABLE.md` and the two gates. The secrecy gate ignores publisher-written `name`/`description`, since user copy is not our mechanism.
- Row 10 REUSE loom-core `should_close` and the existing typed error; ADAPT the report count query; BUILD the capability document. `tui` reported false until row 11 shipped, now true.
- Row 11 REUSE clap/ratatui/crossterm already in the workspace and the estate exit-code convention (0/1/2/3/4/5); ADAPT the typed API into JSON stdout; BUILD loomctl, scripted TUI and tui-session.sh.
- Row 12 REUSE estate `ui/dist/tokens.css` via `scripts/sync-chrome.sh` and the estate nginx chrome injection (deploy row); ADAPT the typed API into Svelte pages; BUILD an adapter-static SvelteKit PWA with prerendered first paint, SPA fallback, service worker and exact-marker canary.
- Row 13 REUSE cargo fmt/clippy/test and the estate probe/gate convention; ADAPT a single gate dispatcher for all rows; BUILD OpenAPI artifact/version drift check and GitHub Actions workflow.
