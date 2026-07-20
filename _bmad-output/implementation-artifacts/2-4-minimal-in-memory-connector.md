# Story 2.4: A minimal in-memory connector

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want a trivial in-memory `Connector` implementation,
so that the contract can be exercised before any real source or the JSONL fixture exists.

## Acceptance Criteria

1. **Given** a scripted batch of observations plus a `Capabilities` descriptor and scopes, **when** `poll` runs to completion, **then** it emits them through the sink and returns a `PollSummary` carrying those capabilities and scopes.
2. **Given** the connector is scripted to stop early or the `cancel` token fires, **when** `poll` runs, **then** it stops at a cancellation point and returns cleanly with what it emitted.
3. **Given** it can be scripted for the contract cases (empty batch; partial emission then an error), **when** the contract test drives it, **then** those behaviours are reproducible with zero mocks.
4. **And** it is a pure, in-memory helper (no I/O) — NOT the JSONL `FixtureConnector` of Epic 4 — and it does not enter the shipped binary path.

## Tasks / Subtasks

- [x] Task 1 — Add a `test-support` feature to `opencmdb-core` (AC: #4)
  - [x] `[features] test-support = []` in `crates/opencmdb-core/Cargo.toml`. The scripted connector (and the contract harness of Story 2.5) live behind it, so they compile only when a consumer asks — they never enter the shipped binary (`opencmdb-bin` builds without the feature).
- [x] Task 2 — Add the `testing` module with `ScriptedConnector` (AC: #1, #2, #3)
  - [x] `crates/opencmdb-core/src/testing/mod.rs`, gated `#[cfg(any(test, feature = "test-support"))]` in `lib.rs`, so it is available to core's own unit tests AND to external crates that enable the feature.
  - [x] `pub struct ScriptedConnector` carrying: a `ConnectorId`, a `Vec<Observation>` to emit, a `Capabilities`, a `Vec<Scope>` (scopes covered), and a terminal `ScriptedOutcome`.
  - [x] `pub enum ScriptedOutcome { Complete, Fail(ConnectorError) }` — after emitting the observations, either return `Ok(PollSummary)` (Complete) or `Err(e)` (Fail — the "partial emission then error" case).
  - [x] `impl Connector for ScriptedConnector`: emit each observation through the sink, checking `cancel.is_cancelled()` BEFORE each emit (the cancellation point); on cancel return `Err(ConnectorError::Cancelled)` with whatever was already emitted; otherwise resolve the `ScriptedOutcome`.
  - [x] A small constructor/builder (e.g. `ScriptedConnector::new(id, capabilities, scopes)` returning a Complete/empty connector, plus `with_observations(...)` and `failing_with(err)`), so the contract cases are one-liners.
- [x] Task 3 — Tests (AC: #1, #2, #3)
  - [x] Complete: scripted with two observations + Complete → both land in a `VecSink`, `PollSummary` carries the scripted capabilities + scopes.
  - [x] Empty: scripted with no observations + Complete → sink empty, `Ok` summary.
  - [x] Partial-then-error: scripted with one observation + `Fail(SchemaMismatch{..})` → the observation is in the sink AND `poll` returns that error (D34 §2 — the emitted observation survives the error).
  - [x] Cancel: token cancelled before poll → `Err(Cancelled)`, sink empty; and cancelled after the first emit is exercised by the mid-poll semantics (the `is_cancelled()` check before each emit).
- [x] Task 4 — Verify (AC: #1–#4)
  - [x] `cargo test -p opencmdb-core` green (core's unit tests see `testing` via `cfg(test)`).
  - [x] `cargo test -p opencmdb-core --features test-support` green (the feature path compiles and the module is public).
  - [x] `cargo xtask ci` green (frontier stays clean); `cargo clippy --workspace --locked -- -D warnings` and `cargo fmt --all --check` clean. Confirm `opencmdb-bin` still builds WITHOUT the feature (the helper does not ship).

## Dev Notes

### What this is (and is not)

A pure, in-memory `Connector` you can SCRIPT — emit these observations, cover these scopes, then complete-or-fail. It exists so the contract (Story 2.3) can be exercised, and the consumer-driven contract test (Story 2.5) has something to run, before any real source or the committed JSONL fixture exists. It is explicitly **NOT** the `FixtureConnector` of Epic 4 (which replays committed JSONL and IS the oracle); this one is hand-scripted in Rust for unit-level contract exercise, with zero mocks and zero I/O.

### Where it lives — behind a `test-support` feature

Future real connectors (UniFi, ARP) live in `opencmdb-bin` and must be able to run the contract test (Story 2.5) against themselves. So the scripted connector and the harness cannot be `#[cfg(test)]`-only (that is invisible to other crates). They live in a `testing` module gated by a `test-support` feature: core's own unit tests see it via `cfg(test)`; `opencmdb-bin`'s tests enable it as a dev-dependency feature. Because the feature is OFF for the normal `opencmdb-bin` build, the helper never ships (AC #4).

### Reuse Story 2.3's contract, don't reinvent it

`Connector`, `ObservationSink`/`VecSink`, `PollSummary`, and `ConnectorError` all exist (Stories 2.1–2.3). `ScriptedConnector` is a thin `impl Connector`. The cancellation semantics mirror the `TinyConnector` proven in 2.3's tests (check `is_cancelled()` before each emit; already-emitted observations survive). Promote that pattern into a reusable public type — do not duplicate the trait or the sink.

### Scriptable for exactly the contract's cases (Story 2.5)

The five contract cases (2.5) map onto scripts:
- empty stream → no observations, `Complete`.
- partial emission then error → some observations, `Fail(e)`.
- missing field → an `Observation` whose `facts` omit a kind (a data choice, not a new mechanism).
- timeout → the caller wraps `poll` in `tokio::time::timeout`; the sink keeps what was emitted.
- cancellation → the caller cancels the token; `poll` returns `Cancelled` with what was emitted.
Keep the scripting surface just rich enough for these; no more.

### Dependency & crate posture

- No new crate dependencies. The `testing` module uses only what Stories 2.1–2.3 defined plus `tokio-util`'s `CancellationToken` (already a dep). `tokio` (dev-dep) remains test-only.
- No changes to `opencmdb-bin`. The frontier gate stays green (no forbidden crates; the feature adds none).

### Testing standards summary

- `#[tokio::test]` async tests for `poll`; sync where possible. Run both `cargo test -p opencmdb-core` and `cargo test -p opencmdb-core --features test-support`.

### Project Structure Notes

- New: `crates/opencmdb-core/src/testing/mod.rs`. Modified: `lib.rs` (gated `pub mod testing;` + re-exports under the same gate), `crates/opencmdb-core/Cargo.toml` (`[features] test-support`).
- Sibling to `observation` and `connector`. Story 2.5 adds the contract harness into this same `testing` module.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 2.4: A minimal in-memory connector]
- [Source: _bmad-output/planning-artifacts/architecture.md#D19 — the fixture IS the Connector (this is the hand-scripted cousin, not the JSONL fixture)]
- [Source: _bmad-output/implementation-artifacts/2-3-connector-trait.md — the `Connector`/`ObservationSink`/`PollSummary` contract and the `TinyConnector` cancellation pattern to promote]
- [Source: crates/opencmdb-core/src/connector/mod.rs — the contract this implements]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- `cargo test -p opencmdb-core` → 18 passed; `cargo test -p opencmdb-core --features test-support` → 18 passed (4 new `testing` tests: complete, empty, partial-then-error keeps emitted obs, cancelled-before).
- `cargo build -p opencmdb-bin --locked` → builds WITHOUT `test-support` (the helper does not ship — AC #4).
- `cargo run -p xtask -- ci` → `✅ frontier` green; clippy clean at default AND with `--features test-support`; `cargo fmt --all --check` clean; full workspace 18 + 23.

### Completion Notes List

- Added `crates/opencmdb-core/src/testing/mod.rs` (gated `#[cfg(any(test, feature = "test-support"))]`): `ScriptedConnector` + `ScriptedOutcome { Complete, Fail(ConnectorError) }`, a builder (`new`/`with_observations`/`failing_with`).
- **AC #1** — Complete → emits all observations, returns `PollSummary` with the scripted capabilities + scopes. **AC #2** — cancel token checked before each emit; cancelled → `Err(Cancelled)` keeping what was emitted. **AC #3** — scriptable for empty batch and partial-then-error (the observation emitted before the error survives, D34 §2). **AC #4** — behind the `test-support` feature; `opencmdb-bin` builds without it, so it never ships.
- Reused Stories 2.1–2.3 unchanged (no new deps); promoted 2.3's `TinyConnector` cancellation pattern into a reusable public type.
- Scope held: Story 2.5 adds the consumer-driven contract-test harness into this same `testing` module and drives `ScriptedConnector` through the five cases.

### File List

- `crates/opencmdb-core/src/testing/mod.rs` (new) — `ScriptedConnector`, `ScriptedOutcome`, tests.
- `crates/opencmdb-core/src/lib.rs` (modified) — gated `pub mod testing;`.
- `crates/opencmdb-core/Cargo.toml` (modified) — `[features] test-support`.

## Change Log

- 2026-07-20 — Implemented Story 2.4 (minimal in-memory connector). `ScriptedConnector` behind a `test-support` feature: scriptable to emit observations then complete or fail, respecting cancellation (emitted observations survive). Never ships (bin builds without the feature). 4 tests; frontier green; clippy/fmt clean. Status → review.
