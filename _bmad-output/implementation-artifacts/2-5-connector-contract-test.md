# Story 2.5: The consumer-driven contract test

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want a reusable contract test every connector must pass,
so that fixture, ARP, UniFi, and future connectors all honour the same behaviour.

## Acceptance Criteria

1. **Given** any `Connector`, **when** the contract test runs, **then** it exercises the five cases: (1) empty stream, (2) partial emission then error, (3) a missing/absent field — the observation is still valid, no "gone" is fabricated, (4) timeout — `tokio::time::timeout` around `poll` drops the future, yet observations already emitted through the sink survive, (5) cancellation — the token fires, `poll` returns cleanly, emitted observations survive.
2. **Given** the minimal in-memory connector (Story 2.4), **when** it is run through the contract test, **then** it passes all five cases.
3. **And** the reusable harness is a function taking a connector factory, so a future connector plugs in with a single call.

## Tasks / Subtasks

- [x] Task 1 — Add the reusable harness `run_connector_contract` (AC: #1, #3)
  - [x] In `testing/mod.rs` (behind `test-support`, beside `ScriptedConnector`): `pub async fn run_connector_contract<C, F>(make: F, expected: &[Observation]) where C: Connector, F: Fn() -> C`. `make` yields a FRESH connector that emits `expected` in order then completes cleanly; the harness runs it through the runtime-agnostic universal invariants using only `tokio-util`'s `CancellationToken` (the CALLER supplies the async runtime by awaiting the harness).
  - [x] Check A — **clean completion** (also the **empty-stream** case when `expected` is empty, and the **missing-field** case when `expected` holds observations whose `facts` omit kinds): poll an uncancelled connector → `sink.observations == expected`, `Ok(summary)`.
  - [x] Check B — **cancellation**: a pre-cancelled poll must not panic; if it returns `Err` it must be `ConnectorError::Cancelled` and `!is_blinding()` (NFR7); and `sink.observations` must be a PREFIX of `expected` (emitted in order, nothing fabricated).
- [x] Task 2 — Drive the `ScriptedConnector` through the harness + the two scripted cases (AC: #2)
  - [x] In `testing`'s `#[cfg(test)] mod tests`, call `run_connector_contract` against `ScriptedConnector` for: a normal batch (2 observations), the **empty** case (`expected = []`), and the **missing-field** case (an observation whose `facts` is only a `Mac`, no `Hostname` — valid, not "gone").
  - [x] **Partial-then-error** (case 2): a `ScriptedConnector::…failing_with(SchemaMismatch)` scripted to emit one observation then fail → the observation is in the sink AND `poll` returns that error (emitted survives the error, D34 §2). (This behaviour is connector-scripted, so it is verified directly, not via the generic harness.)
  - [x] **Timeout** (case 4): add `"time"` to the `tokio` dev-dependency; with a small inline connector that `tokio::task::yield_now().await`s between emits, wrap `poll` in `tokio::time::timeout(Duration::ZERO or tiny)`; assert no panic and that whatever reached the `VecSink` before the drop SURVIVES (the sink is external, not a returned `Vec`). (Runtime-timer-specific, so it lives in core's own tests, not the reusable harness.)
- [x] Task 3 — Re-export + document (AC: #3)
  - [x] Re-export `run_connector_contract` from `lib.rs` under the same `#[cfg(any(test, feature = "test-support"))]` gate. Doc: how a future connector plugs in — `run_connector_contract(|| MyConnector::new(…), &expected).await` — and which cases are universal (harness) vs connector-scripted (timeout, partial-error).
- [x] Task 4 — Verify (AC: #1–#3)
  - [x] `cargo test -p opencmdb-core` and `cargo test -p opencmdb-core --features test-support` green.
  - [x] `cargo xtask ci` green (frontier stays clean — no new prod deps: the harness uses the existing `tokio-util`; `tokio` `time` is dev-only); `cargo clippy --workspace --locked -- -D warnings` and `cargo fmt --all --check` clean; `opencmdb-bin` still builds without `test-support`.

## Dev Notes

### The contract is the point of Epic 2

D19: every source — fixture, ARP, UniFi, future — implements ONE `Connector` contract, and the contract test is what proves it. This story closes Epic 2 by making that test reusable: a future connector proves itself with a single call, not a re-derivation.

### Universal (harness) vs connector-scripted (module tests)

Three of the five cases are runtime-agnostic invariants ANY connector must satisfy given a factory that emits `expected` then completes — so they live in the reusable `run_connector_contract`:
- **empty stream** — run the harness with `expected = []`.
- **missing field** — run the harness with observations whose `facts` omit kinds; they must round-trip unchanged (a source that cannot see hostnames emits an observation without a `Hostname` fact — valid, NOT a "gone"; NFR7 is a type property from Story 2.1, so there is no absence to fabricate).
- **cancellation** — pre-cancel the token; `poll` returns cleanly, the sink holds only a prefix of `expected`, `Cancelled` is non-blinding.

Two cases require putting the connector in a state you cannot express through a "emits `expected` then completes" factory — so they are verified against the `ScriptedConnector` directly (a real connector's consumer would drive its own source into these states):
- **partial-then-error** — the connector emits, then the source fails; the emitted observations survive the error. `ScriptedConnector::failing_with` scripts this.
- **timeout** — needs a poll with `await` points a `tokio::time::timeout` can drop; a tiny yielding connector demonstrates that the already-emitted observations survive the drop because the `VecSink` is external, not a returned `Vec` (D34 §2). Runtime-timer-specific → core's own test, not the portable harness.

### Why the harness uses only `tokio-util`

Keeping `tokio::time` OUT of the reusable harness avoids making `tokio` a (feature-gated) production dependency of `opencmdb-core`. The harness is `async` and uses only `CancellationToken` (already a dep); the CALLER awaits it inside its own runtime. So an external connector crate enables `test-support` and calls the harness from its `#[tokio::test]` with zero extra dependency wiring. The timeout demonstration, which does need a runtime timer, stays in core's own `#[cfg(test)]` tests where `tokio` is a dev-dependency (add its `time` feature).

### Reuse, don't reinvent

`ScriptedConnector`, `VecSink`, `Connector`, `ConnectorError`, `PollSummary`, and the observation types all exist (Stories 2.1–2.4). This story adds only the harness fn and its tests. Do not change the contract or the scripted connector.

### Dependency & crate posture

- No new PROD dependency. The harness uses `tokio-util` (already present). Add the `time` feature to the existing `tokio` DEV-dependency for the timeout test only.
- No changes to `opencmdb-bin`; it still builds without `test-support`, so none of this ships. Frontier gate stays green.

### Testing standards summary

- `#[tokio::test]` for the harness-driving tests and the timeout test. Run both `cargo test -p opencmdb-core` and `--features test-support`.

### Project Structure Notes

- Modified: `crates/opencmdb-core/src/testing/mod.rs` (add `run_connector_contract` + tests); `lib.rs` (re-export under the gate); `crates/opencmdb-core/Cargo.toml` (tokio dev-dep gains `time`).
- Closes Epic 2. No `opencmdb-bin`/`xtask` changes.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 2.5: The consumer-driven contract test]
- [Source: _bmad-output/planning-artifacts/architecture.md#D19 · D34 — the Connector contract and its incremental-emission / cancellation invariants]
- [Source: _bmad-output/implementation-artifacts/2-3-connector-trait.md · 2-4-minimal-in-memory-connector.md — the contract and the ScriptedConnector this drives]
- [Source: crates/opencmdb-core/src/testing/mod.rs — where the harness lands]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- `cargo test -p opencmdb-core` and `--features test-support` → 23 passed (5 new: contract normal / empty / missing-field via the harness; partial-then-error and timeout against scripted/yielding connectors).
- `cargo build -p opencmdb-bin --locked` → builds without `test-support`. `cargo run -p xtask -- ci` → frontier green. Clippy clean (default AND +feature); fmt clean. Full workspace 23 + 23.

### Completion Notes List

- Added `run_connector_contract<C, F>(make, expected)` to `testing/mod.rs` — a reusable, runtime-agnostic harness (tokio-util only; the caller supplies the runtime). It checks clean completion (== empty-stream when `expected` is empty, == missing-field when observations have sparse `facts`) and cancellation (pre-cancel → no panic, prefix-of-expected, non-blinding `Cancelled`).
- **AC #1/#2** — the five cases are exercised against the minimal connector: empty / missing-field / normal via the harness; partial-then-error via `ScriptedConnector::failing_with` (emitted observation survives the error); timeout via a yielding connector wrapped in `tokio::time::timeout(ZERO)` (already-emitted observations survive the dropped future).
- **AC #3** — the harness is a single-call factory function, re-exported from `lib.rs` (under the `test-support` gate): a future connector runs `run_connector_contract(|| MyConnector::new(…), &expected).await`.
- Design: timeout/partial-error stay in core's own tests (they need a scripted failure / a runtime timer), so the reusable harness needs no `tokio` prod dep — only the existing `tokio-util`. `tokio` dev-dep gained the `time` feature for the timeout test. No prod-dep or frontier change.
- **Closes Epic 2.** No changes to `opencmdb-bin`/`xtask`; the helpers never ship.

### File List

- `crates/opencmdb-core/src/testing/mod.rs` (modified) — `run_connector_contract` + `contract_now` + 5 contract tests (incl. a yielding connector for the timeout case).
- `crates/opencmdb-core/src/lib.rs` (modified) — re-export `run_connector_contract`/`ScriptedConnector`/`ScriptedOutcome` under the gate.
- `crates/opencmdb-core/Cargo.toml` (modified) — `time` feature on the `tokio` dev-dependency.

## Change Log

- 2026-07-20 — Implemented Story 2.5 (consumer-driven contract test). Reusable `run_connector_contract` (tokio-util only) covers completion/empty/missing-field/cancellation for any connector in one call; partial-then-error and timeout are exercised against the scripted/yielding connectors in core's own tests. 5 tests; frontier green; clippy/fmt clean; bin builds without the feature. **Closes Epic 2.** Status → review.
- 2026-07-20 — Committed + pushed (`da37af1`); real GitHub CI run green (29727455430). Status → done. **Epic 2 complete.**
