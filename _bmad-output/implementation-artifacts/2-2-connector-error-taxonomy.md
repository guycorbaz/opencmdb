# Story 2.2: The closed `ConnectorError` taxonomy

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want a closed `ConnectorError` enum,
so that alert suppression (FR5/FR8/FR19) can match on named causes and a connector failure is never an opaque `anyhow` string.

## Acceptance Criteria

1. **Given** `opencmdb-core`, **when** `ConnectorError` is defined, **then** it is a `thiserror` enum with named variants for the real, decided failure causes — with NO `anyhow` and NO `Other(String)` catch-all that would make FR5/FR8/FR19 inexpressible (D33).
2. **Given** a `Cancelled` outcome, **then** it is a distinct variant that leaves the source's liveness unchanged — it produces no gap (NFR7); every other variant is blinding.
3. **And** each variant is exercised by a test, `Display` is meaningful, and `cargo xtask ci` stays green (frontier gate: `anyhow` absent from core by construction).

## Tasks / Subtasks

- [x] Task 1 — Define `ConnectorError` in a `connector` module (AC: #1)
  - [x] Create `crates/opencmdb-core/src/connector/mod.rs` (the `Connector` trait, Story 2.3, will join it). Re-export `ConnectorError` from `lib.rs`.
  - [x] `#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]` — but **NOT `#[non_exhaustive]`**: the taxonomy is CLOSED. Exhaustiveness IS the guardrail (D33) — adding a variant must break every downstream `match` so each is forced to decide blind-vs-gap. (Contrast `Fact`, which IS `#[non_exhaustive]` because it is an open vocabulary.)
  - [x] The seven decided variants, each with a human-readable `detail`/payload and a meaningful `#[error("…")]` message:
    - `Unauthorized { detail: String }` — auth rejected → operator rotates the key (Journey 4).
    - `Unreachable { detail: String }` — DNS/refused/network-timeout collapse here (ONE operator action: check connectivity).
    - `SchemaMismatch { detail: String, os_version_hint: Option<String> }` — response unparseable; aimed at the MAINTAINER (NFR8), carries an OS-version hint.
    - `Timeout` — the poll exceeded its time budget (FR6); a metric, not an alert.
    - `Cancelled` — clean cooperative cancellation; writes nothing.
    - `RemoteFault { status: u16, detail: String }` — a 5xx from the source.
    - `Misconfigured { detail: String }` — cannot start; surfaced at startup, not at 3 a.m.
- [x] Task 2 — The NFR7 safe default (AC: #2)
  - [x] `fn is_blinding(&self) -> bool { !matches!(self, ConnectorError::Cancelled) }` — every cause blinds the source EXCEPT `Cancelled`. Doc-comment: a future non-blinding variant must justify itself here before NFR7. Written as an exhaustive-by-negation `matches!` so the intent is explicit.
- [x] Task 3 — Tests (AC: #1, #2, #3)
  - [x] `is_blinding` is `true` for all six non-cancelled variants and `false` for `Cancelled`.
  - [x] Each variant's `Display` (`to_string()`) is non-empty and mentions its cause; `SchemaMismatch` surfaces the OS-version hint when present.
  - [x] `ConnectorError` implements `std::error::Error` (compile-level: bind it to a `&dyn std::error::Error`).
  - [x] A comment records that the enum is intentionally NOT `#[non_exhaustive]` (the closed-taxonomy guardrail).
- [x] Task 4 — Verify (AC: #1–#3)
  - [x] `cargo test -p opencmdb-core` green; `cargo xtask ci` green (frontier stays clean — no `anyhow`); `cargo clippy --workspace --locked -- -D warnings` and `cargo fmt --all --check` clean.

## Dev Notes

### The taxonomy is of QUESTIONS the source did not answer (D33)

The admission rule: **a variant exists only if it produces a `(source_state, operator action)` pair that no other variant produces.** Not one variant per technical cause — "design it as 'things that can go wrong' and you get `anyhow`." So DNS failure, connection refused, and a network timeout are ONE variant (`Unreachable`) because they share one operator action. `Unauthorized` and `Unreachable` are split precisely because they share a liveness verdict but have OPPOSITE actions ("rotate your key" vs "check connectivity") — "that is the difference between a product and a dashboard."

### Why `anyhow` / `Other(String)` is banned here

"`ConnectorError` will never be `anyhow::Error`… because `anyhow` here makes FR5, FR8, FR19 and NFR7 literally inexpressible — you cannot suppress alerts on a condition you have not named." The discriminant is machine-readable (the engine matches on it); the payload is human-readable (never matched on). No `Other(String)` — that is `anyhow` in disguise and would reopen the hole.

### NFR7 is a structural consequence, not a hope

"The 'gaps' column says only one thing: NO. Everywhere." Every variant is blinding except `Cancelled`. `is_blinding()` encodes the safe default. Because the enum is **closed (not `#[non_exhaustive]`)**, the day someone adds a variant the compiler demands the decision at every `match` site — "`anyhow::Error` wakes nobody; a non-exhaustive `match` does not compile." This is the load-bearing reason the enum must NOT be `#[non_exhaustive]`.

### `Unauthorized → Blind` and `Cancelled → unchanged` are deliberate

- `Unauthorized` blinds even though the controller *responds*: "liveness measures data arrival, not the remote peer's health. A source that answers 'no' is blind to us." Mark it live and the absence of 84 devices becomes legitimate → 84 alerts. This variant "defines the product."
- `Cancelled` is the ONLY variant that writes nothing / leaves liveness unchanged: "if `Cancelled` set blind, a clean shutdown would blind every source and FR19 would suppress everything at restart." It must be distinct so it cannot disguise itself as `Timeout` and take blind wrongly.

### Deliberately deferred variants (D33 marks these "Open/UNKNOWN")

Do NOT add them in this story:
- `RateLimited` — no evidence the local UniFi API rate-limits; add it when a fixture shows it.
- `CapabilityLost` — an EVENT, not a state; in steady state ping-only is an `Ok` with a reduced descriptor (Story 2.1's `Capabilities`), not an error.
- `ImplausibleResponse` — the engine-side net against silent drift (D35), not a poll error.

### The `scope` field is deferred to Epic 13 (approved 2026-07-20)

D33 says "every variant carries `scope`", but that scope is D34 §3's **liveness-blindness scope** — a type built later with `source_state` (Epic 13), NOT Story 2.1's observation `Scope`. And D34 schedules `poll` **per scope**, so the caller (the scheduler) already knows which scope an error belongs to. Decision (Guy): **omit `scope` from `ConnectorError` for now**; the scheduler associates it, and the field (with the real liveness-scope type) is added in Epic 13. This is an additive change then — no variant reshaping. (Documented D33↔D34 tension; do not invent a placeholder scope type.)

### Dependency & crate posture

- `ConnectorError` lives in `opencmdb-core` (one `thiserror` per decider — D47). `thiserror` is already a core dep. **No `anyhow`, no new crates.**
- This story defines ONLY the error. The `Connector` trait, `ObservationSink`, `PollSummary`, and cancellation are Story 2.3 (they share the `connector` module). Do not add them here.
- The frontier gate (Story 1.1) confirms core stays clean.

### Testing standards summary

- Unit tests inline in the `connector` module. Pure: no async, no I/O.
- Run `cargo test -p opencmdb-core`.

### Project Structure Notes

- New: `crates/opencmdb-core/src/connector/mod.rs`; `lib.rs` gains `pub mod connector;` + a `ConnectorError` re-export.
- Sibling to the `observation` module from Story 2.1. No manifest changes (thiserror already present).

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 2.2: The closed `ConnectorError` taxonomy]
- [Source: _bmad-output/planning-artifacts/architecture.md#D33 — `ConnectorError`: a closed taxonomy. Never `anyhow` (lines 1857–1915)] — the variant table, the admission rule, the "Open" variant set.
- [Source: _bmad-output/planning-artifacts/architecture.md#D47 — one `thiserror` per decider; `anyhow` absent from core]
- [Source: _bmad-output/implementation-artifacts/2-1-domain-observation-types.md — the sibling `observation` module; the two-scopes distinction; `#[non_exhaustive]` used there and deliberately NOT here]
- [Source: crates/opencmdb-core/Cargo.toml — thiserror already declared]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- `cargo test -p opencmdb-core` → 10 passed (4 new connector tests: only-cancelled-non-blinding, display-meaningful, schema-hint, implements-std-error).
- `cargo run -p xtask -- ci` → `✅ frontier  domain graph clean` (no anyhow in core); `cargo clippy --workspace --locked -- -D warnings` clean; `cargo fmt --all --check` clean; full workspace test green (10 + 23).

### Completion Notes List

- Added `crates/opencmdb-core/src/connector/mod.rs` with the closed `ConnectorError` taxonomy — 7 decided variants (Unauthorized, Unreachable, SchemaMismatch{+os_version_hint}, Timeout, Cancelled, RemoteFault{status}, Misconfigured), each with a `thiserror` `#[error(...)]` message.
- **AC #1** — `thiserror` enum, no `anyhow`, no `Other(String)`; machine-readable discriminant, human-readable payload. Frontier gate confirms core stays anyhow-free.
- **AC #2** — `is_blinding()` returns `false` only for `Cancelled`; a test asserts the verdict for every variant.
- **AC #3** — each variant's `Display` tested; `SchemaMismatch` surfaces the OS hint; `impl std::error::Error` bound checked.
- **Key architectural choice:** `ConnectorError` is deliberately NOT `#[non_exhaustive]` (contrast `Fact`). Exhaustiveness is the D33/NFR7 guardrail — adding a variant must break downstream matches. The test's `one_of_each()` array also stops compiling if a variant is added, exercising the closed set.
- Deferred per D33's "Open"/approval: `RateLimited`, `CapabilityLost`, `ImplausibleResponse`; and `scope` (Epic 13 liveness-scope). No placeholder types invented.
- Scope held: only the error; the `Connector` trait / `ObservationSink` / `PollSummary` / cancellation are Story 2.3.

### File List

- `crates/opencmdb-core/src/connector/mod.rs` (new) — the `ConnectorError` taxonomy + tests.
- `crates/opencmdb-core/src/lib.rs` (modified) — `pub mod connector;` + `ConnectorError` re-export.

## Change Log

- 2026-07-20 — Implemented Story 2.2 (closed `ConnectorError` taxonomy, D33). 7 decided variants, `is_blinding()` NFR7 safe default (only `Cancelled` non-blinding), no `anyhow`/`Other(String)`, deliberately NOT `#[non_exhaustive]` (exhaustiveness is the guardrail). 4 tests; frontier gate green; clippy/fmt clean. `scope` + 3 uncertain variants deferred. Status → review.
