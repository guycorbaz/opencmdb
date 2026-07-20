# Story 2.3: The `Connector` trait, `ObservationSink`, `PollSummary`, cancellation

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want the generalized `Connector` trait with incremental emission and cooperative cancellation,
so that every source implements one contract and a cut-short poll never throws away valid observations.

## Acceptance Criteria

1. **Given** `opencmdb-core`, **when** the trait is defined, **then** `Connector` exposes `fn id(&self) -> ConnectorId` and `async fn poll(&mut self, now: Timestamp, sink: &mut dyn ObservationSink, cancel: CancellationToken) -> Result<PollSummary, ConnectorError>` (native `async fn` in trait, no `async-trait` crate).
2. **Given** `ObservationSink`, **when** a connector emits, **then** it emits observations INCREMENTALLY through the sink, so observations already emitted survive a later timeout or cancellation (no total loss — D34 §2).
3. **Given** `PollSummary`, **when** a poll completes, **then** it carries the batch's `Capabilities` and the `scopes_covered`.
4. **Given** `cancel` fires mid-poll, **when** the connector reaches a cancellation point, **then** it returns cleanly; already-emitted observations remain valid (their `observed_at` is the source's, they do not expire because the poll was cut).
5. **And** cancellation uses `tokio-util`'s `CancellationToken`; `cargo xtask ci` stays green (`tokio-util` is not on the frontier denylist).

## Tasks / Subtasks

- [x] Task 1 — Add `tokio-util` to `opencmdb-core` (AC: #5)
  - [x] `tokio-util` in `[dependencies]` for `sync::CancellationToken`. Confirm the minimal feature set (CancellationToken lives in `tokio_util::sync`). This is the domain trait's one runtime primitive — allowed by the frontier gate (denylist is `anyhow`/`axum`/`sqlx`/`askama` only).
  - [x] `tokio` as a `[dev-dependency]` (features `macros`, `rt`) so the async tests can `#[tokio::test]` and await a poll. Dev-only → excluded from the frontier gate.
- [x] Task 2 — Define `ObservationSink` (AC: #2)
  - [x] In `connector/mod.rs` (sibling of `ConnectorError`): `pub trait ObservationSink { fn emit(&mut self, observation: Observation); }`. **Sync `emit`** so the trait is object-safe (`&mut dyn ObservationSink`) — the connector pushes each observation as it is produced; buffering/backpressure is the sink implementation's concern, not the contract's.
  - [x] Provide a `VecSink` (a `Vec<Observation>` collector) for tests and simple callers.
- [x] Task 3 — Define `PollSummary` (AC: #3)
  - [x] `pub struct PollSummary { pub capabilities: Capabilities, pub scopes_covered: Vec<Scope> }` — what the poll established this cycle: the dated capability descriptor (D34 §1) and which scopes it actually covered.
- [x] Task 4 — Define the `Connector` trait (AC: #1, #4)
  - [x] `pub trait Connector { fn id(&self) -> ConnectorId; async fn poll(&mut self, now: Timestamp, sink: &mut dyn ObservationSink, cancel: CancellationToken) -> Result<PollSummary, ConnectorError>; }` — **native `async fn` in trait** (Rust 1.96), no `async-trait`.
  - [x] Document the dyn-compatibility consequence: a native `async fn` in a trait is NOT `dyn`-compatible, so `Connector` is consumed **generically** (or via an enum of connectors), not as `Box<dyn Connector>`. `ObservationSink` is the `dyn` seam (sync `emit`). If `dyn Connector` is ever required, it is an additive change (box the future); do not pull in `async-trait` pre-emptively.
- [x] Task 5 — Tests (AC: #1–#4)
  - [x] A trivial inline test `Connector` that emits two observations to the sink and returns a `PollSummary`. `#[tokio::test]`: poll it, assert both observations landed in a `VecSink` and the summary carries the expected capabilities/scopes.
  - [x] Cancellation: with a token cancelled BEFORE poll (or after the first emit), the connector checks `cancel.is_cancelled()` at its cancellation point and returns `Err(ConnectorError::Cancelled)` — assert the observations emitted before the cancellation point are STILL in the sink (no total loss, D34 §2), and `ConnectorError::Cancelled.is_blinding()` is false (liveness unchanged).
  - [x] `ObservationSink` object-safety: bind `&mut VecSink` as `&mut dyn ObservationSink` and emit through it.
- [x] Task 6 — Verify (AC: #1–#5)
  - [x] `cargo test -p opencmdb-core` green; `cargo xtask ci` green (frontier stays clean — tokio-util is not forbidden, tokio is dev-only); `cargo clippy --workspace --locked -- -D warnings` and `cargo fmt --all --check` clean.

## Dev Notes

### The trait, from D34 (authoritative)

```rust
#[async_trait]   // ← the architecture sketch used async_trait; we use NATIVE async fn (Rust 1.96)
trait Connector {
    fn id(&self) -> ConnectorId;
    async fn poll(&mut self, now: Timestamp,
                  sink: &mut dyn ObservationSink,   // incremental emission
                  cancel: CancellationToken)        // cooperative cancellation
        -> Result<PollSummary, ConnectorError>;     // capabilities + scopes_covered
}
```

### Why incremental emission + cooperative cancellation (D34 §2)

"A synchronous signature returning a complete `Vec` has no cancellation point and no partial result. A 120 s sweep killed at 119 s throws away 119 s of valid observations." And "`async fn poll` alone is not enough either: `tokio::time::timeout` drops the future, `Vec` included. Clean cancellation, TOTAL LOSS." So observations are pushed to `sink` **as they are produced**, and the connector checks `cancel` at its own cancellation points (between probes, never mid-probe). Observations already emitted are TRUE — `observed_at` comes from the source; they do not expire because the poll was cut. `Cancelled` → liveness unchanged → no gap → NFR7 holds (Story 2.2's `is_blinding()`).

### The dyn-compatibility decision (native async fn)

Guy's decision (2026-07-20): native `async fn` in trait, no `async-trait`. Consequence, stated plainly: a trait with a native `async fn` is **not object-safe**, so there is no `Box<dyn Connector>`. Connectors are dispatched **statically** — generically (`fn run<C: Connector>(c: &mut C, …)`) or via an enum of the known connector types. This is fine for a handful of connector kinds (fixture, ARP, UniFi) and avoids boxing every poll future. The `dyn` seam the design needs is `ObservationSink` (object-safe because `emit` is sync). If a future need forces `dyn Connector`, box the future then — additive, and not a reason to add `async-trait` now.

### `ObservationSink` — sync `emit`, object-safe

`fn emit(&mut self, observation: Observation)` is synchronous so `&mut dyn ObservationSink` is object-safe (a native async method would not be). The connector calls `emit` for each observation the moment it is produced. A real sink (later, in `opencmdb-bin`) may forward to a channel or buffer; backpressure is its concern, not the contract's. `VecSink` is the trivial collector used by tests and the minimal connector (Story 2.4).

### `PollSummary` carries the batch's capabilities (D34 §1)

The poll is the authority on capabilities: it returns the dated `Capabilities` (Story 2.1) it established this cycle, plus the `scopes_covered`. This is how a downgrade is later detected as a diff `caps(N-1) → caps(N)` (FR5) and how the engine tells "no `Uplink` because none" from "blind to topology".

### Dependency & crate posture

- New prod dep: `tokio-util` (for `CancellationToken`) in `opencmdb-core`. Allowed by the frontier gate (denylist = anyhow/axum/sqlx/askama). New dev-dep: `tokio` (macros, rt) for `#[tokio::test]`. Both pull tokio into the graph, but only tokio-util ships; neither is forbidden.
- This story defines the trait + sink + summary only. A concrete connector is Story 2.4; the reusable contract test is Story 2.5. Do NOT implement a real source here.
- `ConnectorError` (Story 2.2) and the observation types (Story 2.1) are reused unchanged.

### Testing standards summary

- `#[tokio::test]` async tests in the `connector` module (needs the tokio dev-dep). Plus sync tests for `ObservationSink`/`VecSink`.
- Run `cargo test -p opencmdb-core`.

### Project Structure Notes

- Modified: `crates/opencmdb-core/src/connector/mod.rs` (add `Connector`, `ObservationSink`, `VecSink`, `PollSummary` beside `ConnectorError`); `lib.rs` re-exports them; `crates/opencmdb-core/Cargo.toml` (+tokio-util dep, +tokio dev-dep); `Cargo.lock`.
- No changes to `opencmdb-bin` or `xtask`.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 2.3]
- [Source: _bmad-output/planning-artifacts/architecture.md#D34 — Three forced corrections to the Connector trait (lines 1917–2000)] — the trait, incremental emission, cancellation, PollSummary.
- [Source: _bmad-output/planning-artifacts/architecture.md#D19 — the Connector/Observation shape (lines 1267–1345)]
- [Source: _bmad-output/implementation-artifacts/2-1-domain-observation-types.md · 2-2-connector-error-taxonomy.md — Observation/Capabilities/Scope and ConnectorError reused here]
- [Source: crates/opencmdb-core/src/connector/mod.rs — the module `ConnectorError` already lives in]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- `cargo test -p opencmdb-core` → 14 passed (4 new: object-safety of `ObservationSink`, incremental poll + summary, cancelled-before-poll emits nothing, cancel-midway keeps the already-emitted observation).
- `cargo run -p xtask -- ci` → `✅ frontier  domain graph clean` — `tokio-util` (a direct prod dep now) is not on the denylist; `tokio` is dev-only. Frontier stays green.
- `cargo clippy --workspace --locked -- -D warnings` clean (the `#[allow(async_fn_in_trait)]` suppresses the public-async-fn lint); `cargo fmt --all --check` clean; full workspace green (14 + 23).

### Completion Notes List

- Added the connector contract to `connector/mod.rs` beside `ConnectorError`: the `Connector` trait (native `async fn poll(now, sink, cancel) -> Result<PollSummary, ConnectorError>`), `ObservationSink` (sync `emit`, object-safe) + `VecSink`, and `PollSummary { capabilities, scopes_covered }`.
- **AC #1** — native `async fn` in trait, no `async-trait`. **AC #2** — `ObservationSink::emit` is incremental; a cancel-midway test proves the already-emitted observation survives (D34 §2). **AC #3** — `PollSummary` carries the dated `Capabilities` + `scopes_covered`. **AC #4** — cancellation via `CancellationToken`; a `Cancelled` return is `!is_blinding()` (liveness unchanged, no gap). **AC #5** — `tokio-util` prod dep; frontier green.
- **Dyn-compatibility (documented):** a native async-fn trait is not object-safe → no `Box<dyn Connector>`; connectors are consumed generically. `ObservationSink` is the `dyn` seam (sync emit). `#[allow(async_fn_in_trait)]` accepts the unspecified-`Send` future; if the scheduler needs `Send` poll futures it becomes a return-position `impl Future + Send` (additive, no `async-trait`).
- Deps: `tokio-util` added to `[dependencies]` (CancellationToken only), `tokio` (macros, rt) to `[dev-dependencies]` for `#[tokio::test]`. Cargo.lock updated.
- Scope held: no concrete source (Story 2.4), no reusable contract test harness (Story 2.5).

### File List

- `crates/opencmdb-core/src/connector/mod.rs` (modified) — `Connector`, `ObservationSink`, `VecSink`, `PollSummary` + async tests.
- `crates/opencmdb-core/src/lib.rs` (modified) — re-export the new items.
- `crates/opencmdb-core/Cargo.toml` (modified) — `tokio-util` dep; `tokio` dev-dep.
- `Cargo.lock` (modified).

## Change Log

- 2026-07-20 — Implemented Story 2.3 (Connector trait + ObservationSink + PollSummary + cancellation, D34). Native `async fn` in trait (no async-trait); incremental emission via a sync object-safe `ObservationSink`; `CancellationToken` cooperative cancellation with the already-emitted observations surviving; `is_blinding()`-consistent Cancelled. `tokio-util` prod dep (frontier stays green), `tokio` dev-dep for async tests. 14 tests. Status → review.
