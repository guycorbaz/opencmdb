# Story 3.3: The Repository skeleton — two traits

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want the read/write repository contract as distinct traits with a MariaDB adapter skeleton,
so that the domain names persistence abstractly and `sqlx::Error` dies in the adapter (D47/D49).

## Acceptance Criteria

1. **Given** `opencmdb-core`, **when** the repository contract is defined, **then** it is the D49 shape — `WriteRepository` (GAT `Unit<'u>` + `transact(closure)`), `WriteUnit` (no `commit()`), and a separate `ReadRepository` — and `sqlx` is NOT named in core (a single `Reads` trait does not compile; the read query bodies live in the adapter).
2. **Given** the MariaDB adapter in `opencmdb-bin`, **when** it maps errors, **then** `sqlx::Error` is classified into the closed `RepositoryError` (`Contention`/`Constraint`/`NotFound`/`Backend`) — never `#[from] sqlx::Error` leaking into core.
3. **And** the skeleton COMPILES (the D49 story-1 bar) and is exercised by a minimal `transact` round-trip against a real MariaDB (local container + CI service); `cargo xtask ci` stays green (frontier: `sqlx` absent from core).

## Tasks / Subtasks

- [x] Task 1 — The contract in `opencmdb-core` (AC: #1, #2)
  - [x] `crates/opencmdb-core/src/repo/mod.rs`: `RepositoryError` (`thiserror`; `Contention`/`Constraint(&'static str)`/`NotFound`/`Backend(String)`); `pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>` (std, so core needs no `futures` crate); `WriteUnit: Send` (marker — no `commit()`); `WriteRepository` with `type Unit<'u>: WriteUnit + Send where Self: 'u` and `async fn transact<F, T>(&self, f: F) -> Result<T, RepositoryError> where F: for<'u> FnOnce(&'u mut Self::Unit<'u>) -> BoxFuture<'u, Result<T, RepositoryError>> + Send`; `ReadRepository` (separate trait). Re-export from `lib.rs`.
  - [x] `#[allow(async_fn_in_trait)]` on `WriteRepository` (as with `Connector`); the Send-ness of the transact future is the single-writer actor's concern.
- [x] Task 2 — The MariaDB adapter in `opencmdb-bin` (AC: #2, #3)
  - [x] `MariaRepository { pool }` implements `WriteRepository` with `Unit<'u> = MariaUnit<'u>` wrapping a `sqlx::Transaction`. `transact` begins a tx, runs the closure, commits on `Ok` / rolls back on `Err`. A `MariaUnit<'u>: WriteUnit`.
  - [x] `classify(sqlx::Error) -> RepositoryError`: lock-wait/deadlock (1213/1205) → `Contention`; `RowNotFound` → `NotFound`; constraint violations → `Constraint`; else → `Backend(String)`. **sqlx appears ONLY here.**
  - [x] The read query bodies are FREE FUNCTIONS generic over `sqlx::Executor` (e.g. `count_declared_attributes<'e, E: Executor<'e>>`); `ReadRepository` (via the pool) and `MariaUnit` (via its tx) each delegate in two lines — the query is written once.
  - [x] Escape hatch if the HRTB-over-GAT fights the compiler: `Box<dyn>` erasure at the unit level (D49-documented; one alloc/batch).
- [x] Task 3 — Round-trip test (AC: #3)
  - [x] A `DATABASE_URL`-gated `#[tokio::test]`: build `MariaRepository`, `migrate!`, then `transact` a closure that writes a `declared_attribute` row through the unit and reads it back (read-your-own-writes) → assert the count; a `ReadRepository` read after commit sees it. Proves transact + commit + the shared read body.
- [x] Task 4 — Verify (AC: #1–#3)
  - [x] `cargo build --workspace` COMPILES (the D49 bar). `cargo test -p opencmdb-bin` green against a local MariaDB; skips without `DATABASE_URL`.
  - [x] `cargo xtask ci` green (frontier: no `sqlx` in core); clippy `-D warnings` + fmt clean.

## Dev Notes

### The D49 shape (authoritative) — and how the `Reads` bomb is defused

```rust
trait WriteRepository {
    type Unit<'u>: WriteUnit + Send where Self: 'u;   // opaque GAT — no sqlx::Transaction, no sqlx::Error
    async fn transact<F, T>(&self, f: F) -> Result<T, RepositoryError>
        where F: for<'u> FnOnce(&'u mut Self::Unit<'u>) -> BoxFuture<'u, Result<T, RepositoryError>> + Send;
}
trait WriteUnit: Send { /* reads its own writes. NO commit(). */ }
trait ReadRepository { /* pool, &self, serves the API only (D21) */ }
```

**There is no single `Reads` trait** — it would not compile (`ReadRepository` is `&self`, `Unit` is `&mut self`, and core cannot name `sqlx::Executor`). The resolution (D49): the read query bodies are **free functions generic over `sqlx::Executor` in the adapter**, and both `ReadRepository` and `MariaUnit` delegate to them in two lines. So `sqlx` appears only in the adapter, and each query is written once.

### Why this shape (D49)

- **No `commit()` on `WriteUnit`:** an identity decision cannot be split across two transactions because the method does not exist — the missing-handle mechanism applied to transactions.
- **`transact(closure)` is replayable:** on a MariaDB deadlock, `transact` fails `Contention` and the actor replays the whole closure — one retry path (NFR15). Scattered `begin`/`commit` give a retry no purchase.
- **Two distinct types (Write vs Read):** the writer actor is constructed with `WriteRepository` only, so it cannot reach the read pool — read-your-own-writes as a constructor signature (D21), not a review comment.
- **MariaDB-only (D64):** no dual backend, no `dual!` — one `MariaRepository`.

### The budgeted risk (D49) — and the escape hatch

`for<'u> FnOnce(&'u mut Self::Unit<'u>) -> BoxFuture<'u, _>` is an **HRTB over a GAT** — "the class of signature an AI assistant will not write correctly first try." The AC is literally that the skeleton COMPILES; expect to iterate against the compiler. If it resists, D49's documented escape is **`Box<dyn>` erasure at the unit level** (one allocation per batch — negligible at 300 hosts / 36 subnets). Do not spend more than the budgeted effort fighting the zero-alloc form; the `Box<dyn>` form is an approved, correct fallback.

### `sqlx::Error` dies in the adapter (D47/D49)

`RepositoryError` is closed and lives in core; the adapter classifies (`1213`/`1205` → `Contention`, `RowNotFound` → `NotFound`, constraint → `Constraint`, else `Backend(String)`). NEVER `#[from] sqlx::Error` — that leaks sqlx into the trait and the two-file blast radius becomes the whole codebase. The frontier gate confirms `sqlx` stays out of core.

### Scope — the skeleton, not the engine

This story delivers the compiling plumbing + a round-trip, NOT identity logic, the writer actor, or `IdentityIndex` (D50, later). Keep the read body trivial (e.g. `SELECT COUNT(*) FROM declared_attribute`). Static SQL only → no `AssertSqlSafe` yet.

### Testing standards summary

- `DATABASE_URL`-gated `#[tokio::test]` against a real MariaDB (local `mariadb:10.11.11` container; CI service). Skips without `DATABASE_URL`.
- `cargo build --workspace` compiling IS the primary AC (the D49 story-1 bar).

### Project Structure Notes

- New: `crates/opencmdb-core/src/repo/mod.rs`; `crates/opencmdb-bin/src/repo.rs` (or a module). Modified: `opencmdb-core/src/lib.rs` (re-exports); `opencmdb-bin/src/main.rs` (declare the module; wire `MariaRepository` from the pool). No manifest changes (sqlx/thiserror already present).
- No `xtask` change; the frontier gate confirms core stays sqlx-free.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 3.3: The Repository skeleton — two traits]
- [Source: _bmad-output/planning-artifacts/architecture.md#D49 — Repository: transact(closure), opaque Unit<'u>, no commit() (lines 2732–2783)]
- [Source: _bmad-output/planning-artifacts/architecture.md#D47 — RepositoryError closed; sqlx::Error dies in the adapter (Backend(String), never #[from])]
- [Source: _bmad-output/planning-artifacts/architecture.md#D64 — MariaDB-only; no dual backend]
- [Source: crates/opencmdb-core/src/connector/mod.rs — the `#[allow(async_fn_in_trait)]` pattern to reuse; ConnectorError as the sibling closed taxonomy]
- [Source: crates/opencmdb-bin/src/main.rs — the pool/migrate from Story 3.2 the adapter builds on]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- The HRTB-over-GAT `transact` **compiled on the first real attempt** — the covariance design (see below) avoided the `Box<dyn>` escape hatch.
- Round-trip test against a local `mariadb:10.11.11` (`DATABASE_URL` set): `transact_writes_and_reads_its_own_write` passed — inserted through the unit, read-its-own-writes counted 1, committed, and the read side saw 1.
- Frontier gate GREEN (core names no `sqlx`); clippy `-D warnings` + fmt clean; workspace tests green (repo test skips without `DATABASE_URL`).

### Completion Notes List

- **AC #1** — the D49 contract in `opencmdb-core/src/repo/mod.rs`: `WriteRepository` (GAT `Unit<'u>` + `async fn transact` with the `for<'u> FnOnce(&'u mut Unit<'u>) -> BoxFuture<'u, …>` HRTB), `WriteUnit` (no `commit()`), separate `ReadRepository`, `RepositoryError` (thiserror), and a std-only `BoxFuture` (no `futures` crate). `#[allow(async_fn_in_trait)]` as with `Connector`. No `sqlx` in core.
- **The `Reads` bomb, defused:** no single `Reads` trait. The MariaDB adapter (`opencmdb-bin/src/repo.rs`) holds the query bodies as free functions generic over `sqlx::Executor`; the read side (pool) and a unit (its tx connection) each delegate — the query is written once, and `sqlx` appears only here.
- **The HRTB-over-GAT compiled without erasure:** `MariaUnit<'u>` holds `&'u mut MySqlConnection` (covariant in `'u`), so `for<'u> &'u mut Unit<'u>` unifies. `transact` owns the `Transaction`, lends the unit to the closure in a scoped block, then commits on `Ok` / rolls back on `Err`. The `Box<dyn>` escape hatch (D49) was not needed.
- **AC #2** — `classify(sqlx::Error)`: `1213`/`1205` → `Contention`; unique/FK/check → `Constraint`; `RowNotFound` → `NotFound`; else `Backend(String)`. Never `#[from] sqlx::Error`.
- **AC #3** — the skeleton compiles (the D49 story-1 bar) and the round-trip proves transact + read-your-own-writes + commit + the shared read body. `#![allow(dead_code)]` on the adapter (skeleton wired into the app in Stories 3.5+; the test proves it works). No manifest/lock changes.

### File List

- `crates/opencmdb-core/src/repo/mod.rs` (new) — the D49 contract (WriteRepository/WriteUnit/ReadRepository/RepositoryError/BoxFuture).
- `crates/opencmdb-core/src/lib.rs` (modified) — `pub mod repo` + re-exports.
- `crates/opencmdb-bin/src/repo.rs` (new) — the MariaDB adapter (MariaRepository/MariaUnit/MariaReadRepository, query bodies, classify) + round-trip test.
- `crates/opencmdb-bin/src/main.rs` (modified) — `mod repo;`.

## Change Log

- 2026-07-20 — Implemented Story 3.3 (Repository skeleton, D49). The two-trait contract (WriteRepository GAT+transact HRTB, WriteUnit no-commit, ReadRepository) in core; a MariaDB adapter in bin with query bodies generic over `sqlx::Executor` and `sqlx::Error` classified into a closed `RepositoryError`. The `Reads` bomb defused; the HRTB-over-GAT compiled first try via a covariant `&mut Conn` unit (no `Box<dyn>` needed). Proven by a `transact` read-your-own-writes round-trip against real MariaDB. Frontier stays sqlx-free; clippy/fmt green. Status → review.
- 2026-07-20 — CI (29736528783) FAILED: the two DB tests (3.2's healthz + 3.3's round-trip) run in parallel on CI's single MariaDB and raced on `migrate!` (`Duplicate entry '1' for key 'PRIMARY'` on `_sqlx_migrations`). Locally the round-trip had only been run filtered, so the race was never triggered. FIX: a crate-level `DB_TEST_LOCK` (`LazyLock<tokio::sync::Mutex>`) held for each DB test's duration serializes them. Reproduced + verified locally: both DB tests unfiltered, 3× on a fresh DB, all green. Re-pushed.
- 2026-07-20 — Fix pushed (`b08707d`); real GitHub CI run green (29737639588). Status → done.
