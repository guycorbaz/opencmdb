# Story 3.4: The `Clock` port, routed by the reader

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want time to enter as a `Clock` port routed by the reader, never read inside the domain,
so that the engine is a deterministic pure function (D10/D19/D25).

## Acceptance Criteria

1. **Given** the domain, **when** it needs "now", **then** it receives a `Timestamp` from a `Clock` port bound at the composition root — the domain never calls a wall clock (core's `chrono` has `clock` OFF, so it cannot).
2. **Given** a test, **when** it supplies a fixed `Clock`, **then** behaviour is reproducible.
3. **And** the `Clock` is wired at the composition root (a `SystemClock`), so a later replay/fixture can drive time by substituting the port.

## Tasks / Subtasks

- [x] Task 1 — The `Clock` port in `opencmdb-core` (AC: #1)
  - [x] `crates/opencmdb-core/src/clock.rs`: `pub trait Clock: Send + Sync { fn now(&self) -> Timestamp; }`. Re-export from `lib.rs`. It returns a `Timestamp` (= `chrono::DateTime<Utc>`); the domain reads time only through this.
- [x] Task 2 — A `FixedClock` for tests (AC: #2)
  - [x] In `opencmdb-core`'s `testing` module (`test-support` feature): `pub struct FixedClock(pub Timestamp)` with `impl Clock`. A test asserts `now()` returns the fixed instant → reproducibility.
- [x] Task 3 — A `SystemClock` in `opencmdb-bin` (AC: #1, #3)
  - [x] `SystemClock` implementing `Clock` by reading the wall clock **via `std::time`**, NOT chrono's `clock` feature: `let d = SystemTime::now().duration_since(UNIX_EPOCH)?; chrono::DateTime::from_timestamp(d.as_secs() as i64, d.subsec_nanos())`. Add `chrono` to `bin` with `default-features = false` (so the `clock` feature stays OFF workspace-wide — otherwise feature unification would re-enable it in core and the domain COULD call `Utc::now()`).
  - [x] Wire it at the composition root: `main` constructs a `SystemClock` and logs the startup instant via `clock.now()` — the one place the wall clock is read. (Routing it into the reader/engine happens when they exist, 3.5/3.6.)
- [x] Task 4 — Verify (AC: #1–#3)
  - [x] `cargo test -p opencmdb-core` and `--features test-support` green; `cargo test -p opencmdb-bin` green.
  - [x] `cargo xtask ci` green (frontier: `chrono` is allowed — not on the denylist; `clock` feature stays off so core still cannot read the clock); clippy `-D warnings` + fmt clean.

## Dev Notes

### Time is data, and the domain cannot read a clock (D19/D25)

`observed_at` comes from the source; the engine's "now" (for a gap's as-of, later) is bound from a `Clock` port at the composition root. The domain is a pure function of its inputs. core's `chrono` already has `clock` OFF (manifest `default-features = false`), so `Utc::now()` is not callable in core — the `Clock` port is how time gets in, deliberately.

### The feature-unification trap (important)

If `opencmdb-bin` enabled chrono's `clock` feature, Cargo's feature unification would turn it ON for the whole build — including core — and the domain COULD then call `Utc::now()`, silently breaking the guarantee. So `SystemClock` reads the wall clock through **`std::time::SystemTime`** (allowed in bin — it is the composition root) and converts with `chrono::DateTime::from_timestamp(secs, nanos)` (a pure constructor needing no `clock` feature). `bin` declares `chrono` with `default-features = false`. The `clock` feature must stay OFF workspace-wide.

### `SystemClock` is the ONLY wall-clock read

Reading the clock is a composition-root privilege. `SystemClock::now()` is that single place; everything else takes a `Timestamp` or a `&dyn Clock`. A fixture/replay later substitutes a different `Clock` (e.g. `FixedClock`) without touching the engine — determinism by construction.

### Dependency & crate posture

- New `bin` dependency: `chrono` (`default-features = false`, `std` for `from_timestamp`/`Duration`) — NOT the `clock` feature. `chrono` is not on the frontier denylist, so the gate stays green.
- `FixedClock` lives in core's `testing` module (behind `test-support`), reusable by future engine tests.
- No `xtask` change.

### Testing standards summary

- `FixedClock` reproducibility test in core (`test-support`). `SystemClock` sanity in bin (returns a plausible, monotone-ish instant).
- Run `cargo test -p opencmdb-core --features test-support` and `cargo test -p opencmdb-bin`.

### Project Structure Notes

- New: `crates/opencmdb-core/src/clock.rs`; a `SystemClock` in `opencmdb-bin` (its own small module or in `main.rs`). Modified: `opencmdb-core/src/lib.rs` (re-export `Clock`), `testing/mod.rs` (`FixedClock`), `opencmdb-bin/Cargo.toml` (`chrono` no-clock), `opencmdb-bin/src/main.rs` (wire `SystemClock`), `Cargo.lock`.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 3.4: The `Clock` port, routed by the reader]
- [Source: _bmad-output/planning-artifacts/architecture.md#D19 — observed_at from the source; the engine never touches the clock]
- [Source: crates/opencmdb-core/Cargo.toml — chrono `default-features = false` (clock OFF); Timestamp = DateTime<Utc>]
- [Source: crates/opencmdb-core/src/observation/mod.rs — `Timestamp` the port returns]
- [Source: crates/opencmdb-core/src/testing/mod.rs — where `FixedClock` lands, beside the scripted connector]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- `cargo test -p opencmdb-core --features test-support` → `fixed_clock_is_reproducible` ok (24 core tests). `cargo test -p opencmdb-bin` → 3 passed.
- **Critical check:** `cargo tree -e features` confirms chrono's `clock` feature is OFF workspace-wide — so `opencmdb-core` still cannot call `Utc::now()` (the feature-unification trap avoided).
- `cargo run -p xtask -- ci` frontier/all-gates green; clippy `-D warnings` + fmt clean.

### Completion Notes List

- **AC #1** — `Clock` port (`fn now(&self) -> Timestamp`) in `opencmdb-core/src/clock.rs`; the domain reads time only through it, and core's chrono still has `clock` off.
- **AC #2** — `FixedClock(Timestamp)` in core's `testing` module (test-support); a test asserts `now()` never moves.
- **AC #3** — `SystemClock` in `bin` reads the wall clock via `std::time::SystemTime` + `chrono::DateTime::from_timestamp` (NOT chrono's `clock` feature); `main` constructs it and logs the startup instant via `clock.now()` — the one wall-clock read. A replay/fixture substitutes a different `Clock` later.
- **The feature-unification trap, avoided:** `bin`'s `chrono` is `default-features = false` (`std` only), so enabling it does not turn on `clock` for core. Verified via `cargo tree -e features`.
- No `xtask` change; `chrono` is not on the frontier denylist, so the gate stays green.

### File List

- `crates/opencmdb-core/src/clock.rs` (new) — the `Clock` port.
- `crates/opencmdb-core/src/lib.rs` (modified) — `pub mod clock` + `Clock`/`FixedClock` re-exports.
- `crates/opencmdb-core/src/testing/mod.rs` (modified) — `FixedClock` + reproducibility test.
- `crates/opencmdb-bin/Cargo.toml` (modified) — `chrono` (default-features off, no `clock`).
- `crates/opencmdb-bin/src/main.rs` (modified) — `SystemClock`, wired at startup.
- `Cargo.lock` (modified if the resolution changed).

## Change Log

- 2026-07-20 — Implemented Story 3.4 (the `Clock` port). `Clock` in core (the one seam for time); `FixedClock` in test-support; `SystemClock` in bin reading the wall clock via std::time + `DateTime::from_timestamp` (chrono `clock` feature stays OFF workspace-wide, so core still cannot read the clock — verified). Wired at the composition root (startup log). 24 core + 3 bin tests; frontier/clippy/fmt green. Status → review.
