# Story 1.3: Prove-to-red coverage for the ddl-collation gate

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want an explicit test that the `ddl-collation` gate goes RED on a non-binary text column,
so that the gate is trustworthy before the first real migration exists to exercise it.

## Acceptance Criteria

1. **Given** a synthetic migration line declaring a text-typed column with no `_bin` / `COLLATE BINARY`, **when** the gate's detection function runs, **then** it reports that column as a finding.
2. **Given** the same column declared with an explicit binary collation (e.g. `ascii_bin`), **when** the detection function runs, **then** it produces no finding.
3. **And** if such a red test already exists in `xtask`, this story is limited to confirming and documenting it.

## Tasks / Subtasks

- [x] Task 1 — Confirm the existing prove-to-red test satisfies AC #1 + #2 (AC: #1, #2, #3)
  - [x] Confirmed `ddl_flags_bare_text_column_and_passes_a_collated_one` already asserts bare `VARCHAR` → `Some` (AC #1), `ascii_bin`-collated → `None` (AC #2), `INTEGER` → `None`, comment → `None`. Heuristic left unchanged per AC #3.
  - [x] Added a comment above the test naming it the D45 prove-to-red for `ddl-collation`.
- [x] Task 2 — Fill the accepted-form and "same column" coverage gaps (AC: #1, #2)
  - [x] `ddl_accepts_the_collate_binary_form` — `TEXT COLLATE BINARY` and `VARCHAR(64) COLLATE BINARY` → `None` (the previously-untested accepted form).
  - [x] `ddl_same_varchar_column_toggles_on_the_collation` — same `email VARCHAR(320)` bare → `Some`, `+ COLLATE latin1_bin` → `None`.
  - [x] `ddl_flags_text_and_clob_and_char_variants` — bare `TEXT`, `CLOB`, leading `CHAR(...)` each → `Some`.
- [x] Task 3 — Document the heuristic's known boundary (AC: #3)
  - [x] Comment in `ddl_flags_text_and_clob_and_char_variants` + Dev Notes: `ENUM`/`SET` outside `is_text` is the reflex-gate boundary (D53), a D64 refinement for the first real migration; heuristic not expanded here. A pinning assertion (`ENUM(...)` → `None`) documents the current behaviour.
- [x] Task 4 — Verify (AC: #1–#3)
  - [x] `cargo test -p xtask` — 23 tests green (4 ddl tests incl. 3 new).
  - [x] `cargo xtask ci` — `ddl-collation` reports "no migrations/ yet — nothing to check" and is green.
  - [x] `cargo clippy --workspace -- -D warnings` and `cargo fmt --all --check` clean.

## Dev Notes

### This story is tests only — no production logic changes

Per AC #3, the RED test already exists (`ddl_flags_bare_text_column_and_passes_a_collated_one`, committed in `7d4b1bd`). The `ddl-collation` gate and its pure detection function `text_column_without_binary_collation` are already implemented and unchanged since. **Do not modify the heuristic, the gate, or `run_ci`.** This story confirms the prove-to-red exists and closes two real coverage gaps in the tests. Keep it small.

### Read the code under test: `xtask/src/main.rs`

The pure detector (do not change it):
```rust
fn text_column_without_binary_collation(line: &str) -> Option<String> {
    let l = line.trim();
    if l.starts_with("--") || l.is_empty() { return None; }
    let up = l.to_uppercase();
    let is_text = up.contains("VARCHAR") || up.contains("TEXT")
        || up.contains(" CHAR") || up.starts_with("CHAR") || up.contains("CLOB");
    if !is_text { return None; }
    let has_binary_collation = up.contains("_BIN") || up.contains("COLLATE BINARY");
    if has_binary_collation { None } else { Some(l.trim_end_matches(',').to_string()) }
}
```
Behaviour that the tests must lock in:
- `is_text` matches `VARCHAR`, `TEXT`, ` CHAR` (space-prefixed, to avoid matching `VARCHAR` twice), a leading `CHAR`, and `CLOB` — case-insensitively (it uppercases first).
- Accepted binary-collation forms: a collation name containing `_BIN` (e.g. `ascii_bin`, `utf8mb4_bin`, `latin1_bin`) **or** the literal `COLLATE BINARY`. Either one → `None`.
- A comment line (`--`) or blank line → `None`. A non-text column (e.g. `INTEGER`) → `None`.
- The finding string is the trimmed line with a trailing comma stripped.

### The reflex-gate boundary (do not "fix" it here)

This is a reflex gate (D53), explicitly "not a proof; it bites on a real migration once one exists." Two known boundaries — deliberately out of scope for this test-only story:
- `ENUM`/`SET` columns carry a collation but are not in `is_text`, so a bare `ENUM('a','b')` is not flagged. Whether the gate should cover them is a D64 refinement to settle when the first real migration lands.
- The heuristic is line-based, so a column definition split across lines, or an inline `--` comment mid-line, could fool it. Also a later-Epic concern (D64 condition 1 says the gate is written *with* the first migration).

Flag these in a comment if useful, but do not broaden the heuristic — that would be a production change this story's ACs do not ask for, and would need its own review.

### Testing standards summary

- Tests inline in `#[cfg(test)] mod tests` in `main.rs` (established location).
- These are pure-function assertions on synthetic single lines — no DB, no filesystem, no migrations dir. Fast and hermetic, matching the existing ddl test.
- Run `cargo test -p xtask`. The gate wrapper `gate_ddl_collation` stays green ("no migrations/ yet") because no `crates/opencmdb-bin/migrations/` exists — this story does not add one.

### Project Structure Notes

- All gate logic and its tests stay in Rust in `xtask/` (D56), no YAML.
- No new dependencies; no new production functions. Only test additions + comments.
- Do not create a `migrations/` directory or a real `.sql` file — the first migration (which will exercise `gate_ddl_collation` end-to-end) belongs to a later story, not this one.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.3: Prove-to-red coverage for the ddl-collation gate]
- [Source: xtask/src/main.rs — `text_column_without_binary_collation`, `gate_ddl_collation`, and the existing `ddl_flags_bare_text_column_and_passes_a_collated_one` test]
- [Source: _bmad-output/planning-artifacts/architecture.md#D64 — MariaDB-only; binary collation on every text column + a CI grep over the DDL] · [D53 — reflex gates] · [D45 — a gate must be proven-to-red, not negotiable]
- [Source: _bmad-output/implementation-artifacts/1-2-fixture-manifest-sha256-gate.md — the pure-function-tested idiom applied here]
- [Source: CLAUDE.md — `cargo xtask ci`; D64 "Comparison/normalization never descends into SQL … held by binary collation on every text column + a CI grep over the DDL"]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- `cargo test -p xtask --locked` → 23 passed (4 ddl tests: the pre-existing one + `ddl_accepts_the_collate_binary_form`, `ddl_same_varchar_column_toggles_on_the_collation`, `ddl_flags_text_and_clob_and_char_variants`).
- `cargo run -p xtask -- ci` → `✅ ddl-collation  no migrations/ yet — nothing to check`; all gates green.
- `cargo clippy --workspace -- -D warnings` clean; `cargo fmt --all --check` clean.

### Completion Notes List

- **Tests-only story, no production change** (AC #3): the RED test already existed (`7d4b1bd`); the `ddl-collation` gate and `text_column_without_binary_collation` heuristic are untouched.
- **AC #1/#2** — confirmed by the existing test (bare text → finding; binary-collated → none) and strengthened: the `COLLATE BINARY` accepted form (previously untested — only `_bin` was) and the "same VARCHAR column toggles on the collation" pair (AC #2's "the same column" made literal).
- **AC #3** — the pre-existing red test is confirmed and documented (a naming comment marks it the D45 prove-to-red), plus the two coverage gaps closed. The heuristic was NOT rewritten.
- Documented the reflex-gate boundary: `ENUM`/`SET` are outside `is_text`, a D64 refinement deferred to the first real migration; a pinning assertion records the current behaviour so a future change is a *visible* diff, not a silent one.

### File List

- `xtask/src/main.rs` (modified) — added 3 tests to `#[cfg(test)] mod tests` (`ddl_accepts_the_collate_binary_form`, `ddl_same_varchar_column_toggles_on_the_collation`, `ddl_flags_text_and_clob_and_char_variants`) and a naming comment on the existing `ddl_flags_bare_text_column_and_passes_a_collated_one`. No production code changed.

## Change Log

- 2026-07-19 — Implemented Story 1.3 (prove-to-red coverage for the ddl-collation gate, D45). Tests-only: confirmed the existing RED test and closed two coverage gaps (the `COLLATE BINARY` accepted form; the same-column collation toggle), plus TEXT/CLOB/CHAR variants and an ENUM boundary pin. 23 tests green, gate green, clippy/fmt clean. No production logic changed. Status → review.
