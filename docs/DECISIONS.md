# Loom — decisions

Decisions are recorded where they are made. See spec §9 for the initial set (name, ports, crates, test DB, sunset 20/30d, tag floor n>=5, gravity ramp u0=10, anonymous visitor keys, exit codes).

| id | date | decision | rejected | why |
|---|---|---|---|---|
| D01 | 2026-11-17 | bundle is the primary object | feed-first catalogue | a bundle is shareable by a reader with no permission |
| D02 | 2026-11-17 | anonymous edit token in share link | OAuth at launch | first contribution must cost nothing |
| D03 | 2026-11-17 | runtime-typed SQL with `query_as` for now | compile-time `query!` macros | installed sqlx-cli is 0.9 while the crate is 0.8; switching macro mode without matching versions would make builds fail on a version mismatch, not on bad SQL. Revisit at row 13. |
| D04 | 2026-11-17 | empty migration is idempotent (`if not exists`) | one-shot DDL + migration ledger | the first GREEN run proved the second run fails with 42P07; idempotence is now tested, not assumed. |
