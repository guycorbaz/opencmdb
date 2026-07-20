# Story 3.2: MariaDB pool and the first migration

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want a MariaDB connection pool and an embedded first migration for declared + observed records,
so that the stack persists to the one supported engine, correctly.

## Acceptance Criteria

1. **Given** a `DATABASE_URL`, **when** the binary starts, **then** it builds a sqlx MariaDB pool (`mysql` + `tls-rustls-ring`) and applies the embedded migration(s) on startup.
2. **Given** the first migration, **when** it is inspected, **then** every text column carries an explicit binary collation (D64) — so `cargo xtask ci`'s `ddl-collation` gate now bites on a real migration and passes.
3. **Given** dynamic SQL, **when** it is written, **then** it uses sqlx 0.9's `AssertSqlSafe` (the static `query*()` path takes `impl SqlSafeStr`) — none is needed in this story (only a static `SELECT 1` and `migrate!`).
4. **Given** a running server, **when** `GET /healthz` is called, **then** it returns `200 OK` when the database answers and `503` when it does not; CI's MariaDB service (Story 1.5) exercises this.

## Tasks / Subtasks

- [x] Task 1 — The first migration (AC: #2)
  - [x] Create `crates/opencmdb-bin/migrations/0001_initial.sql` with two tables — `declared_attribute` (D3, attributes-per-row with field-level provenance) and `observation_record` (the immutable observed side). Opaque ids are `CHAR(36) CHARACTER SET ascii COLLATE ascii_bin` (D48); text values are `TEXT`/`LONGTEXT`/`VARCHAR ... COLLATE ..._bin`. EVERY text column carries a binary collation (D64). `declared_attribute` keeps the D3 `CHECK`s (`origin<>'adopted' OR origin_obs_id IS NOT NULL`; `actor_id<>'scanner'`).
- [x] Task 2 — Pool + migrate on startup (AC: #1)
  - [x] In `main`, read `DATABASE_URL` (`std::env::var`), build a `MySqlPool` (sqlx `mysql`+`tls-rustls-ring`, already a dep), and run `sqlx::migrate!("./migrations").run(&pool)` before serving. Fail with `.context(...)` if the DB is unreachable or a migration fails.
  - [x] Use the runtime-checked `query()` API (not the `query!` macro) with `.bind(&str)`/`try_get::<String,_>()` (D48) — so no compile-time database is needed.
- [x] Task 3 — `/healthz` checks the database (AC: #4)
  - [x] `app(pool: MySqlPool) -> Router` puts the pool in axum state. `healthz(State(pool)) -> StatusCode` runs a static `sqlx::query("SELECT 1").fetch_one(&pool)`: `200 OK` on success, `503 SERVICE_UNAVAILABLE` on error (logged at `warn`).
- [x] Task 4 — Tests (AC: #1, #4)
  - [x] A DB-touching `#[tokio::test]` gated on `DATABASE_URL`: connect, run the migration, and assert `GET /healthz` returns `200`. It runs against CI's MariaDB service (and a local MariaDB container in dev); it no-ops when `DATABASE_URL` is unset.
  - [x] Keep a non-DB test for the default bind address.
- [x] Task 5 — Verify (AC: #1–#4)
  - [x] `cargo test -p opencmdb-bin` green (DB test runs against a local `mariadb:10.11.11` container; skips without `DATABASE_URL`).
  - [x] `cargo xtask ci` green — the `ddl-collation` gate now scans `0001_initial.sql` and PASSES (it was "no migrations/ yet" before).
  - [x] `cargo clippy --workspace --locked -- -D warnings` and `cargo fmt --all --check` clean; frontier unaffected (all `bin`).

## Dev Notes

### The schema (approved 2026-07-20) — minimal walking-skeleton, aligned with D3/D48

Two tables, no bitemporal history yet (identity/links grow in later epics):

- **`declared_attribute`** (D3): the declared side as attributes-per-row so each field carries provenance. `entity_id` is the never-rewritten anchor (D15). `origin ∈ manual|adopted|imported`; `origin_obs_id` points at the exact adopted observation (linked-never-merged). The `CHECK (actor_id <> 'scanner')` makes non-drift STRUCTURAL — the scanner can never author a declared value. **The gap computation never reads `origin`** (D3): a field adopted yesterday can drift again tomorrow.
- **`observation_record`**: the observed side, immutable. `id`, `connector_id`, `observed_at` (dated by the source — never the clock), the `Scope` (`l2_domain`, `vantage`, D19), the `facts` serialized as text (the engine reads them into Rust; SQL never compares — D10), and opaque `raw` provenance.

### Identifiers & collation (D48/D64)

Opaque ids: `CHAR(36) CHARACTER SET ascii COLLATE ascii_bin` — `ascii_bin` is monotone over canonical UUIDv7, so right-edge B-tree inserts, and the id stays greppable across UI → logs → SQL dump (the 3-a.m.-false-merge argument). `BINARY(16)` is refused. Text values use a `_bin` collation. **Every text column has a binary collation** so identity comparison is byte-exact and never depends on the DB locale (D64) — and so `cargo xtask ci`'s `ddl-collation` gate (Stories 1.1/1.3), which has been vacuous, now bites on a real migration.

### sqlx surface — the walking skeleton locks it (D48)

`query()` (function, runtime-checked), `.bind(&str)`, `try_get::<String,_>()`, `migrate!` — NOT the `query!` macro, so there is **no compile-time database**. Bind `String`s (no `uuid` sqlx feature). `AssertSqlSafe` is only needed for dynamic SQL; this story writes only a static `SELECT 1` and the embedded migration.

### `/healthz` grows a dependency

3.1's `/healthz` was pure liveness. Now it is readiness: it pings the DB. `app()` takes the pool via axum state; the `/healthz` test therefore needs a database and is gated on `DATABASE_URL` (CI has the MariaDB service; dev uses a `mariadb:10.11.11` container). Keep it honest: `503` when the DB is down, not `200`.

### Testing standards summary

- DB-touching tests gate on `DATABASE_URL` (`let Ok(url) = std::env::var("DATABASE_URL") else { return }`), so `cargo test` stays green with or without a database, and CI (which sets `DATABASE_URL`) always exercises the real path.
- Locally, run against `docker run mariadb:10.11.11` with a matching `DATABASE_URL`.

### Project Structure Notes

- New: `crates/opencmdb-bin/migrations/0001_initial.sql`. Modified: `crates/opencmdb-bin/src/main.rs` (pool, migrate, `app(pool)`, DB `/healthz`).
- No new dependency (sqlx already declared with `mysql`/`migrate`/`macros`/`tls-rustls-ring`). No `opencmdb-core`/`xtask` changes.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 3.2: MariaDB pool and the first migration]
- [Source: _bmad-output/planning-artifacts/architecture.md#D3 — Field-level provenance; the `declared_attribute` table (lines 305–333)]
- [Source: _bmad-output/planning-artifacts/architecture.md#D48 — Opaque identifiers CHAR(36) ascii_bin; bind a String; the walking-skeleton sqlx subset (lines 2658–2697)]
- [Source: _bmad-output/planning-artifacts/architecture.md#D64 — MariaDB-only; binary collation on every text column]
- [Source: crates/opencmdb-bin/Cargo.toml — sqlx `=0.9.0` (`mysql`, `migrate`, `macros`, `tls-rustls-ring`) already present]
- [Source: crates/opencmdb-bin/src/main.rs — the 3.1 bootstrap this extends; the `ddl-collation` gate in xtask/src/main.rs]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- Tested against a local `mariadb:10.11.11` container (`DATABASE_URL=mysql://root:…@127.0.0.1:13306/opencmdb_test`): `cargo test -p opencmdb-bin` → 2 passed (the DB-gated `healthz_reports_200_when_database_answers` connected, migrated, and got 200).
- `cargo run -p xtask -- ci` → `✅ ddl-collation  every text column carries an explicit binary collation` — the gate now bites on the real `0001_initial.sql` (was "no migrations/ yet") and passes.
- Real-binary smoke test: `database connected and migrations applied` → `opencmdb listening` → `GET /healthz` = **HTTP 200**.
- Without `DATABASE_URL`: `cargo test --workspace` green (the DB test no-ops). clippy `-D warnings` + fmt clean; frontier unaffected.

### Completion Notes List

- Added `crates/opencmdb-bin/migrations/0001_initial.sql`: `declared_attribute` (D3 attributes-per-row + provenance `CHECK`s) and `observation_record` (immutable observed side, serialized facts). Opaque ids `CHAR(36) ascii_bin`, text values `_bin` — every text column has a binary collation (D64).
- **AC #1** — `main` builds a `MySqlPool` from `DATABASE_URL` and runs `sqlx::migrate!("./migrations")` on startup. **AC #2** — the `ddl-collation` gate now scans a real migration and passes. **AC #3** — only static `SELECT 1` + `migrate!` (no dynamic SQL, so no `AssertSqlSafe` yet); runtime-checked `query()` (no compile-time DB). **AC #4** — `/healthz` pings the DB → 200/503.
- `/healthz` grew from liveness to readiness (pings the DB); its test is `DATABASE_URL`-gated so it runs in CI's MariaDB service and locally against a container, and no-ops otherwise.
- No new dependency (sqlx already declared); no `Cargo.lock`/core/xtask changes.

### File List

- `crates/opencmdb-bin/migrations/0001_initial.sql` (new) — the initial schema.
- `crates/opencmdb-bin/src/main.rs` (modified) — pool, `migrate!`, `app(pool)`, DB-checking `/healthz`.

## Change Log

- 2026-07-20 — Implemented Story 3.2 (MariaDB pool + first migration). `main` connects a sqlx `MySqlPool` from `DATABASE_URL` and applies the embedded `0001_initial.sql` (declared_attribute + observation_record, binary collation on every text column, D64) on startup; `/healthz` now reports DB readiness (200/503). Verified against a real MariaDB 10.11.11 (local container + CI service); the `ddl-collation` gate now bites on a real migration and passes. clippy/fmt green. Status → review.
- 2026-07-20 — Committed + pushed (`2443c3e`); real GitHub CI run green (29735462983 — the DB test ran against CI's MariaDB service). Status → done.
