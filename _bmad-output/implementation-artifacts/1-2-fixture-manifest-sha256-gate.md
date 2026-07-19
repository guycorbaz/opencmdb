# Story 1.2: Fixture MANIFEST sha256 gate (scaffold)

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want a gate that verifies every committed fixture matches its recorded sha256,
so that a fixture cannot drift silently once the trap corpus exists.

## Acceptance Criteria

1. **Given** no `fixtures/` directory, **when** the gate runs, **then** it reports "no fixtures — skipped" and is GREEN (vacuous until Epic 4).
2. **Given** a `fixtures/MANIFEST` and fixture files whose bytes match their listed sha256, **when** the gate runs, **then** it passes.
3. **Given** a fixture whose bytes do not match its MANIFEST sha256, **when** the gate's detection function runs, **then** it exits RED naming the file (proven-to-red test).
4. **And** the MANIFEST schema is documented as PROVISIONAL — its final shape is fixed in Epic 4 when the JSONL fixture format is frozen (D56: `fixtures/scenario/replay/MANIFEST.toml` carrying sha256 + seed + generator version).

## Tasks / Subtasks

- [x] Task 1 — Add the fixtures gate to `xtask/src/main.rs` (AC: #1, #2)
  - [x] Add a pure parser `fn parse_manifest(text: &str) -> Vec<(String, String)>` returning `(sha256_hex, relative_path)` pairs. Skip blank lines and `#` comments. Each entry line is `<sha256-hex>  <path>` (whitespace-separated, path relative to `fixtures/`). Document the format as **PROVISIONAL** (see Task 3).
  - [x] Add a pure finding-generator `fn manifest_findings(entries: &[(String, String)], read: &dyn Fn(&str) -> std::io::Result<Vec<u8>>) -> Vec<String>` that, for each entry: reads the bytes via the `read` closure, computes `sha256_hex`, and pushes a RED finding NAMING the file on a mismatch; pushes a "listed in MANIFEST but missing" finding when `read` errors. Injecting `read` keeps this unit-testable without touching disk (repo idiom — the proven-to-red test targets a pure function).
  - [x] Add `fn gate_fixture_manifest(root: &Path) -> Result<(bool, String)>`: no `fixtures/` → "no fixtures — skipped" (AC #1); `fixtures/` but no MANIFEST → skipped scaffold note; else parse + `manifest_findings` with production closure `|p| std::fs::read(fixtures.join(p))`, aggregated into the standard `(bool, String)` shape.
  - [x] Reuse the existing `sha256_hex` helper (already defined for `views-hash`) — do NOT add a new hashing path or a new dependency. `walkdir` is not needed (the MANIFEST is the authority on what to check).
- [x] Task 2 — Wire the gate into `run_ci` (AC: #1, #2)
  - [x] In `run_ci`, add the gate as a hard gate (fold into `ok`) after `vocabulary` and before the informational `views-hash` check.
  - [x] Update the module doc comment (top of `main.rs`) to list `fixtures` alongside the other gates.
- [x] Task 3 — Document the provisional MANIFEST schema (AC: #4)
  - [x] `parse_manifest`'s doc comment states the format is a SCAFFOLD placeholder and that Epic 4 freezes the real `fixtures/scenario/replay/MANIFEST.toml` (sha256 + seed + generator version, `capture/`↔`scenario/` split) per D56; only the parser changes, the verify logic is stable.
- [x] Task 4 — Proven-to-red + green tests (AC: #2, #3)
  - [x] `fixtures_gate_reds_on_a_sha_mismatch` — asserts a finding containing `traps/a.jsonl` + "mismatch".
  - [x] `fixtures_gate_greens_when_bytes_match` — real `sha256_hex(bytes)` in the entry → empty.
  - [x] `fixtures_gate_flags_a_missing_file` — `read` returns `Err` → "missing" finding naming the file.
  - [x] `parse_manifest_skips_comments_and_blanks` — `#` comment + blank + two entries → exactly two pairs.
  - [x] `fixtures_gate_is_green_when_no_fixtures_dir` — nonexistent root → green, "no fixtures" (AC #1).
- [x] Task 5 — Verify (AC: #1–#4)
  - [x] `cargo test -p xtask` — 17 tests green (5 new for `fixtures`).
  - [x] `cargo xtask ci` — the `fixtures` line reports "no fixtures — skipped" and is GREEN on the current workspace.
  - [x] `cargo clippy --workspace -- -D warnings` and `cargo fmt --all --check` clean.
  - [x] End-to-end proven-to-red: a temp `fixtures/` + `MANIFEST` with a wrong sha → gate RED naming the file; corrected sha → GREEN; temp dir removed (not committed).

### Review Findings

_Code review 2026-07-19 (Blind Hunter · Edge Case Hunter · Acceptance Auditor). 0 decision-needed · 3 patch · 1 defer · 3 dismissed. Auditor verdict: PASS on all 4 ACs; the 1.1 duplicated-helper anti-pattern was avoided._

- [x] [Review][Patch] `&expected[..12]` panics on a non-char UTF-8 boundary — FIXED: mismatch prefix now `sha.chars().take(12).collect()`; test `fixtures_gate_mismatch_prefix_is_char_safe` [xtask/src/main.rs:manifest_findings]
- [x] [Review][Patch] Uppercase-hex sha → permanent false RED — FIXED: compare via `actual.eq_ignore_ascii_case(sha)`; test `fixtures_gate_sha_compare_is_case_insensitive` [xtask/src/main.rs:manifest_findings]
- [x] [Review][Patch] Parser fails OPEN + path escape — FIXED: `parse_manifest` now returns `Vec<ManifestLine>` (Entry | Malformed), failing closed on non-2-token lines, spaced paths, and absolute/`..` paths (each becomes a `fixtures/MANIFEST:<lineno>` finding); test `parse_manifest_fails_closed_on_malformed_and_escaping_lines` + e2e [xtask/src/main.rs:parse_manifest]
- [x] [Review][Defer] Untracked fixture on disk but absent from MANIFEST is never checked — drift in the add direction; the gate only verifies MANIFEST-listed files; orphan-fixture detection needs the real MANIFEST + recapture tool (Epic 4 / D56) [xtask/src/main.rs:gate_fixture_manifest] — deferred, inherent Epic 4 scope

## Dev Notes

### What this story is: a scaffold, deliberately vacuous today

There is no `fixtures/` directory yet (verified) and there won't be until Epic 4 builds the trap corpus. So this gate is a **lockfile for data that doesn't exist yet** — it must be GREEN today (AC #1) and only bite once fixtures + a MANIFEST are committed. Do not over-build it: the final MANIFEST schema is explicitly deferred (AC #4). Build the minimum that verifies sha256 and proves-to-red.

### Read the file being modified: `xtask/src/main.rs`

This is the same single-file xtask you extended in Story 1.1 (now ~450 lines). Match its shape:
- Gates are `fn gate_xxx(root: &Path) -> Result<(bool, String)>`, wired into `run_ci` via `report(name, ok, msg)` folding `ok &= g`. Current order: `frontier` (Gate 0), `ddl-collation`, `vocabulary`, then informational `views-hash`. Insert `fixtures` as a hard gate before `views-hash`.
- **`sha256_hex(bytes: &[u8]) -> String` already exists** (used by `check_views_hash`) — reuse it verbatim. `sha2` is already an xtask dependency.
- Established test idiom: split each gate into a **pure function tested on synthetic input** + a thin I/O wrapper (`text_column_without_binary_collation`, `crates_present_in_tree`, and — from Story 1.1 — `frontier_offenders`). Follow it: `parse_manifest` and `manifest_findings` are pure; `gate_fixture_manifest` is the thin wrapper. This is exactly how Story 1.1's code review made the RED path testable — do it from the start here.
- **What must be preserved:** the `frontier`/`ddl-collation`/`vocabulary`/`views-hash` logic and tests are untouched; only add. Update the module doc comment so the header stays truthful.

### Why inject the `read` closure (don't touch disk in the tested function)

AC #3 (proven-to-red naming the file) must be hermetic and fast. Making `manifest_findings` take `read: &dyn Fn(&str) -> io::Result<Vec<u8>>` lets tests feed in-memory bytes (match on path → bytes, or `Err` for the missing-file case) with no temp dirs and no `tempfile` dependency, while production passes `|p| std::fs::read(fixtures_dir.join(p))`. The tested code path IS the production finding-generator, not a duplicate — the Story 1.1 review flagged exactly the duplicated-helper anti-pattern; avoid it here.

### Fixtures location and path discipline (D56 — load-bearing)

`fixtures/` lives at the **workspace root**, outside every crate — a deliberate architecture decision (D56): at the root, editing a trap reads as "I am changing the spec", not "I am fixing a test", so a red is not negotiable by editing the oracle (D45). `xtask`'s `workspace_root()` already returns the workspace root, so `workspace_root().join("fixtures")` is the correct, single source of the path. Do NOT hardcode `../../fixtures` or introduce a second path constant — D56: "if it appears more than once in the tree, it is already broken."

### The MANIFEST is provisional — align direction with D56, don't finalize it

D56 specifies the eventual shape: `fixtures/scenario/replay/MANIFEST.toml` carrying `sha256 + seed + generator version` per artefact, invoked as the sha recompute-and-compare check ("A JSONL modified without a manifest bump is red; one repair: bump it deliberately"). Epic 4 will freeze the JSONL fixture format and this TOML schema together. For THIS scaffold: use a trivial parser-free line format (`<sha256>  <path>`), keep zero new dependencies, and document loudly that Epic 4 replaces the parser (not the verify logic). Adding the `toml` crate now — to parse a manifest for fixtures that do not exist — would be speculative; the seed/generator-version fields have no meaning until the generator (Epic 4) exists.

### Testing standards summary

- Tests inline in `#[cfg(test)] mod tests` in `main.rs`.
- **Every gate proven-to-red, not only green** (D45): AC #3 is mandatory and must name the file. No DB, no network, no temp files — pure-function tests via the injected `read` closure.
- Run `cargo test -p xtask` (or `--workspace`).

### Project Structure Notes

- All gate logic stays in Rust in `xtask/` (D56), no YAML. This story adds no YAML.
- No new dependencies: reuse `sha256_hex` (sha2 already present). Do not add `toml`, `tempfile`, or a second hashing crate.
- The gate reads only `fixtures/MANIFEST` + the files it lists — it does not walk `fixtures/` for unlisted files. "Untracked fixture" detection and the `capture/`↔`scenario/` split (D56) are Epic 4 concerns; note this rather than implementing it.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.2: Fixture MANIFEST sha256 gate (scaffold)]
- [Source: _bmad-output/planning-artifacts/architecture.md#D56 — `fixtures/` at the root; `xtask/` a member and nobody's dependency (lines 3227–3281)] — root location, `MANIFEST.toml` target shape, path discipline, "lockfile for data".
- [Source: _bmad-output/planning-artifacts/architecture.md#D19 — The fixture IS the `Connector` trait (line 1267)] — why fixtures precede/outlive the code and must not be editable to make a red go away.
- [Source: xtask/src/main.rs — existing `sha256_hex`, `run_ci`, `report`, `workspace_root`, and the Story 1.1 pure-function-test idiom (`frontier_offenders`)]
- [Source: _bmad-output/implementation-artifacts/1-1-dependency-frontier-gate.md — Review Findings: the duplicated-helper anti-pattern to avoid; factor detection into a tested pure fn]
- [Source: CLAUDE.md — `cargo xtask ci` gates in Rust not YAML; D56 dependency-frontier discipline]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- `cargo test -p xtask --locked` → 17 passed (5 new: sha-mismatch RED, bytes-match green, missing-file RED, parse skips comments/blanks, no-fixtures-dir green).
- `cargo run -p xtask -- ci` → `✅ fixtures  no fixtures — skipped` (AC #1); all gates green; clippy `-D warnings` + fmt clean.
- End-to-end: temp `fixtures/traps/a.jsonl` + `fixtures/MANIFEST` with a wrong sha → `🔴 fixtures  ... fixtures/traps/a.jsonl: sha256 mismatch (manifest 000000000000… ≠ file b951fe82cd99…)`; corrected sha → `✅ fixtures  1 fixture(s) match their recorded sha256`; temp dir removed.

### Completion Notes List

- Added the `fixtures` gate (D56) to `xtask/src/main.rs`, split per repo idiom into pure `parse_manifest` + pure `manifest_findings` (I/O injected via a `read` closure) + thin `gate_fixture_manifest` wrapper. The RED path is the same code the tests exercise — no duplicated helper (the Story 1.1 review's anti-pattern, avoided here from the start).
- **AC #1** — vacuously green today: no `fixtures/` dir → "no fixtures — skipped". **AC #2** — matching sha256 → pass. **AC #3** — mismatch → RED naming the file (unit + end-to-end). **AC #4** — `parse_manifest`'s doc marks the line format PROVISIONAL and points to D56's `MANIFEST.toml` shape frozen in Epic 4.
- Reused the existing `sha256_hex` (sha2 already an xtask dep); **no new dependencies**. Fixtures path is `workspace_root().join("fixtures")` — the single root path (D56), no `../../fixtures` literal.
- Scope held to scaffold: the gate verifies only what the MANIFEST lists; untracked-fixture detection and the `capture/`↔`scenario/` split (D56) are Epic 4 concerns, noted not implemented.

### File List

- `xtask/src/main.rs` (modified) — added the `fixtures` gate (Gate 3): `gate_fixture_manifest`, the `ManifestLine` enum (Entry|Malformed), `parse_manifest` (fail-closed + path containment), `manifest_findings` (case-insensitive, char-safe); wired into `run_ci`; updated the module doc comment; added 8 tests.
- `_bmad-output/implementation-artifacts/deferred-work.md` (modified) — recorded the untracked-fixture-detection deferral (Epic 4 scope).

## Change Log

- 2026-07-19 — Implemented Story 1.2 (fixture MANIFEST sha256 gate, D56 scaffold). Added the `fixtures` CI gate to `xtask ci`: verifies `fixtures/MANIFEST` sha256 vs file bytes, vacuously green until Epic 4. Pure `parse_manifest`/`manifest_findings` (I/O-injected), 5 tests (proven-to-red naming the file, incl. end-to-end). MANIFEST schema documented PROVISIONAL per D56. 17 tests green, all gates green, clippy/fmt clean. Status → review.
- 2026-07-19 — Code review (Blind Hunter · Edge Case Hunter · Acceptance Auditor; Auditor PASS on all 4 ACs). Applied 3 patch findings: (P1) char-safe mismatch prefix via `chars().take(12)` (was a UTF-8-boundary panic); (P2) case-insensitive sha compare (uppercase MANIFEST was a false RED); (P3) `parse_manifest` now returns `ManifestLine` (Entry|Malformed) and fails CLOSED on malformed lines, spaced paths, and absolute/`..` path escapes. 1 defer (untracked-fixture detection → Epic 4, deferred-work.md), 3 dismissed. 20 tests green (3 new), gate green, e2e RED/GREEN + malformed/escape reproduced, clippy/fmt clean. Status → done.
