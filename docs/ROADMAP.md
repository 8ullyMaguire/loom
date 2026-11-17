# Loom — roadmap

One row per task. A row closes on its own gate output, pasted in the commit message.

| row | item | status | evidence |
|---|---|---|---|
| 0 | spec + plan | CLOSED | vault dffe25d |
| 1 | repo skeleton, gates, state notes | CLOSED | fc8f780: cargo check green, gate repo ok |
| 2 | database schema and store tests | CLOSED | 3 tests green: 9 tables, idempotent, duplicate position rejected |
| 3 | pure domain rules (loom-core) | CLOSED | fc8f780: 4 tests green (RED observed first) |
| 4 | bundle store + anonymous create + edit token | CLOSED | 2 tests green: order kept, wrong/rotated token refused; API routes still row 5 |
| 5 | typed API, OpenAPI, bundle routes | OPEN | — |
| 6 | tag votes and confidence floor | OPEN | — |
| 7 | search, facets, cursor, hidden gems | OPEN | — |
| 8 | read-only ingest from both generators | OPEN | — |
| 9 | recommendations + hidden gravity gates | OPEN | — |
| 10 | sunset rule and capability document | OPEN | — |
| 11 | CLI and TUI | OPEN | — |
| 12 | SvelteKit PWA with real HTML | OPEN | — |
| 13 | gate, CI, OpenAPI diff | OPEN | — |
| 14 | deploy on thinkcentre | OPEN | — |

## Census log

No user-facing surface built yet. Every new surface must record REUSE / ADAPT / BUILD here before code.

- Row 1 REUSE Cargo workspace pattern from darkhold; ADAPT gate.sh single-command pattern; BUILD nothing new.
- Row 2 REUSE the existing lh-pg-test Postgres cluster (new dedicated database); ADAPT scratch-schema test pattern; BUILD the nine-table schema.
- Row 4 REUSE loom-core Bundle validation and token generation; ADAPT the estate idempotency convention; BUILD bundle persistence and token rotation.
