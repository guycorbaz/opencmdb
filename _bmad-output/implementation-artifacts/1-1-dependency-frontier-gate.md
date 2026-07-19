# Story 1.1: Dependency-frontier gate (D47)

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want `cargo xtask ci` to fail when the domain crate's dependency graph crosses the frontier,
so that `opencmdb-core` cannot silently gain an infrastructure dependency.

## Acceptance Criteria

1. **Given** a clean workspace, **when** the frontier gate runs, **then** it shells `cargo tree -p opencmdb-core -e normal` and passes only if none of `anyhow`, `axum`, `sqlx`, `askama` appear in the graph, **and** it also fails if `xtask` appears in `cargo tree -p opencmdb-bin -e normal`.
2. **Given** a synthetic tree in which `opencmdb-core` resolves a forbidden dependency, **when** the gate's detection function runs, **then** it reports RED naming the crate and the offending dependency (proven-to-red unit test).
3. **Given** a `Cargo.toml` comment that merely names a banned crate, **when** the gate runs, **then** it does NOT red — the gate reads the dependency GRAPH (`cargo tree` output), not the manifest text, so a comment can never reach the detector (no false positive).

## Tasks / Subtasks

- [x] Task 1 — Add the frontier gate to `xtask/src/main.rs` (AC: #1)
  - [x] Add `use std::process::Command;` to the existing imports. _(Added `Command` to the existing `std::process` import + `use std::collections::HashSet;`.)_
  - [x] Add a pure parser `fn crates_present_in_tree(tree: &str) -> std::collections::HashSet<String>` that, for each line of `cargo tree` text output, strips the leading tree glyphs (`│`, `├──`, `└──`, spaces) and extracts the crate name (the first whitespace-delimited token before the ` v<version>` field). Return the set of crate names. **Exact-token match only** — `anyhow-macros` must never be read as `anyhow`.
  - [x] Add `fn gate_dependency_frontier(root: &Path) -> Result<(bool, String)>` that:
    - runs `cargo_tree(root, "opencmdb-core")` and flags any of `CORE_FORBIDDEN` present;
    - runs `cargo_tree(root, "opencmdb-bin")` and flags `xtask` if present;
    - aggregates offenders into one RED message that names both the consuming crate and each offending dependency; returns `(true, "…")` when clean.
  - [x] Add `fn cargo_tree(root: &Path, pkg: &str) -> Result<String>` that shells `cargo tree -p <pkg> -e normal --locked` from `root`, returns stdout as `String`, and returns `Err` (with `.context`) on a non-zero exit, including captured stderr. _(Uses `env!("CARGO")` for the cargo binary path rather than a bare `"cargo"`.)_
  - [x] Add `const CORE_FORBIDDEN: &[&str] = &["anyhow", "axum", "sqlx", "askama"];`.
- [x] Task 2 — Wire the gate into `run_ci` (AC: #1)
  - [x] In `run_ci`, after the `vocabulary` gate and before the `views-hash` check, call `gate_dependency_frontier(&root)?`, `report("frontier", g, &m)`, and fold its boolean into `ok`. _(Placed FIRST — before `ddl-collation` — as Gate 0, since D47 is the foundational frontier law; still folds into the same `ok` and runs before `views-hash`.)_
  - [x] Update the module doc comment (top of `main.rs`) to list `frontier` alongside `ddl-collation` and `vocabulary`.
- [x] Task 3 — Proven-to-red + no-false-positive tests (AC: #2, #3)
  - [x] In the `#[cfg(test)] mod tests`, add `frontier_flags_a_forbidden_dep_in_core`: feed a synthetic `cargo tree` string for `opencmdb-core` that contains an `anyhow v1.0.103` line; assert the detector reports `anyhow`.
  - [x] Add `frontier_is_clean_on_the_real_core_deps`: feed a synthetic tree containing only `chrono`, `serde`, `thiserror`, `uuid`; assert zero findings.
  - [x] Add `frontier_token_match_rejects_lookalikes`: assert a tree line `anyhow-macros v0.1.0` does NOT register as `anyhow` (word-exact).
  - [x] Add `frontier_flags_xtask_in_bin`: feed a synthetic `opencmdb-bin` tree containing an `xtask v0.1.0` line; assert `xtask` is flagged.
  - [x] Added `frontier_glyph_stripping_extracts_the_name` (bonus): asserts the crate name survives every tree-drawing prefix cargo emits.
- [x] Task 4 — Verify (AC: #1, #2, #3)
  - [x] `cargo test -p xtask` — all new tests green (11 passed).
  - [x] `cargo xtask ci` — the `frontier` line is GREEN on the current workspace.
  - [x] `cargo clippy --workspace -- -D warnings` and `cargo fmt --all --check` clean.

### Review Findings

_Code review 2026-07-19 (Blind Hunter · Edge Case Hunter · Acceptance Auditor). 0 decision-needed · 4 patch · 1 defer · 2 dismissed._

- [x] [Review][Patch] AC#2 proven-to-red tests a duplicated helper, not the production RED path — FIXED: extracted `frontier_offenders(core_tree, bin_tree) -> Vec<String>` (pure), called by both `gate_dependency_frontier` and the tests; tests now assert the real offender strings naming the crate [xtask/src/main.rs:123]
- [x] [Review][Patch] Silent-green under ASCII charset — FIXED: `cargo_tree` now passes `--charset utf8`, pinning the glyph set the parser strips [xtask/src/main.rs:151]
- [x] [Review][Patch] Transitive dep counted as direct — FIXED: `cargo_tree` now passes `--depth 1`, so the gate checks direct deps only (what core can `use`); no more transitive false positives / false messages [xtask/src/main.rs:151]
- [x] [Review][Patch] "xtask is a dependency of nobody" enforced only on opencmdb-bin — FIXED: `frontier_offenders` scans both `opencmdb-core` and `opencmdb-bin` trees for `xtask`; new test `frontier_flags_xtask_in_core` [xtask/src/main.rs:133]
- [x] [Review][Defer] Forbidden dep invisible when optional / non-default-feature / cfg-target / build-dependency — `cargo tree -e normal` resolves default features + host target + normal edges only; latent false-negative, zero impact today (core has no features/cfg/build.rs); documented limit of a reflex gate (D53), → GitHub issue [xtask/src/main.rs:90] — deferred, needs feature-matrix design

## Dev Notes

### What this story is (and is not)

D47's mechanism is: `opencmdb-core`'s `Cargo.toml` simply does not list `anyhow`/`axum`/`sqlx`/`askama`, so `use anyhow::Result` inside core is a **name-resolution error**, not a lint — impossible to bypass without a visible diff in a manifest a human reads. [Source: architecture.md#D47 (lines 2598–2601)] This story does **not** re-implement that compile-level guard (the compiler already is it). It adds a **CI reflex gate** that catches the manifest diff itself: if someone adds a forbidden dependency to core (directly or transitively), `cargo xtask ci` goes RED and names it. It is a reflex gate (D53), not a proof — the same class as the existing `ddl-collation` gate.

### Read the file being modified: `xtask/src/main.rs`

This is a single-file xtask (currently 357 lines). **Match its established shape — do not restructure it.** Current state:

- `main()` dispatches `cargo xtask ci` → `run_ci() -> Result<bool>` (Ok(true)=all green, Ok(false)=a gate RED, Err=harness failure). Preserve this tri-state contract.
- `run_ci()` runs gates in sequence, each `report(name, ok, msg)`, folding `ok &= g`. Two hard gates today (`ddl-collation`, `vocabulary`) then one informational check (`views-hash`). **Insert `frontier` as a third hard gate, before `views-hash`.**
- `workspace_root()` returns the parent of `CARGO_MANIFEST_DIR` — use it as the CWD for `cargo tree`.
- Established test idiom (`#[cfg(test)] mod tests`): **gates are split into a pure detection function + a thin I/O wrapper, and the tests exercise the pure function on synthetic strings** (see `text_column_without_binary_collation`, `copresence_findings`, `contains_word`). Follow this exactly — the proven-to-red test targets the parser, not a live `cargo tree` subprocess.
- `report()`, `Result`/`Context` from `anyhow`, and the `{:<14}` column width for gate names already exist — reuse them; `"frontier"` fits the width.

**What must be preserved:** the module-doc comment enumerates the gates for the reader — add `frontier` to it so the file's header stays truthful. Do not touch the `ddl-collation`, `vocabulary`, or `views-hash` logic or their tests.

### Why a parser + subprocess split, and why it satisfies AC #3 by construction

`cargo tree` prints the RESOLVED graph, e.g.:

```
opencmdb-core v0.1.0 (/…/crates/opencmdb-core)
├── chrono v0.4.45
│   └── … 
├── serde v1.0.228
├── thiserror v2.0.18
└── uuid v1.24.0
```

The detector reads THIS text, never `Cargo.toml`. A comment in a manifest (`# do not add anyhow here`) resolves to nothing in the graph and never appears in `cargo tree` output — so AC #3 (no false positive on a comment) holds **by construction**, not by a special case. The `frontier_token_match_rejects_lookalikes` test locks in the one real parsing risk: a substring match. Extract the crate name as the first token after stripping tree glyphs and compare it **whole** against the forbidden set.

### Command specifics (pin these, don't improvise)

- Use `cargo tree -p <pkg> -e normal --locked`. `-e normal` (edges = normal only) is required by AC #1: it **excludes dev- and build-dependencies**, so `xtask` (a dep of nobody) and any test-only crate never falsely trip the gate; it also excludes proc-macro edges. `--locked` keeps the gate side-effect-free (never mutates `Cargo.lock`), matching the project's `--locked` policy (CLAUDE.md).
- Run from `workspace_root()`. Capture stdout (`Command::output()`); on `!status.success()`, return `Err` carrying the crate spec and `String::from_utf8_lossy(&output.stderr)` via `.context(...)`.
- `-p opencmdb-core` / `-p opencmdb-bin` are valid package specs even though `default-members = ["crates/opencmdb-bin"]` (Cargo.toml). No `--workspace` needed.

### Current graph is clean — the gate must be GREEN today

Verified manifests:
- `crates/opencmdb-core/Cargo.toml` deps: `chrono`, `serde`, `thiserror`, `uuid` — none forbidden. [Source: crates/opencmdb-core/Cargo.toml]
- `crates/opencmdb-bin/Cargo.toml` depends on `opencmdb-core` and outside-world crates including `anyhow`/`axum`/`sqlx`/`askama` (all legitimate here) but **not** `xtask`. [Source: crates/opencmdb-bin/Cargo.toml]
- `xtask/Cargo.toml` depends on `anyhow`, `sha2`, `walkdir` and is a dependency of nobody (D56). [Source: xtask/Cargo.toml]

So on a correct tree: core-check finds nothing, bin-check finds no `xtask` → `frontier` GREEN. If it reds on first run, the graph — not the gate — is wrong; investigate before "fixing" the gate.

### The frontier is load-bearing (repo law)

`opencmdb-core` is the domain: an error there is domain data, not an `anyhow` string; a gap that vanishes into `anyhow::Error` is this product's exact failure mode — it does not crash, it lies. [Source: architecture.md#D47 (lines 2610–2612)] `opencmdb-bin` is the composition root (everything touching the outside world); `xtask` is a workspace member and a dependency of nobody. [Source: CLAUDE.md — "The dependency frontier is load-bearing (D47)"] This gate is the CI backstop for that law.

### Testing standards summary

- Tests live inline in `#[cfg(test)] mod tests` in `main.rs` (the repo's established location for xtask gate tests).
- **Every gate must be proven-to-red, not only green** (D45) — the existing tests do this for `ddl-collation` and `vocabulary`; the frontier gate must match. The RED case (AC #2) and the clean case (AC #1) are both mandatory.
- Run `cargo test -p xtask` (or `cargo test --workspace`). No DB, no network — pure-function tests only.

### Project Structure Notes

- All gate logic stays in Rust in `xtask/`, never in YAML (D56/D65). This story adds no YAML; the GitHub Actions runner that merely calls `cargo xtask ci` is Story 1.4's concern.
- `xtask` must remain a dependency of nobody — do not add it to any other crate's manifest to make testing convenient.
- No new dependencies are needed: `std::process::Command` covers the subprocess; `anyhow` is already a dep of `xtask`. Do **not** add a cargo-metadata / cargo_toml parsing crate — the whole point is to read the resolved graph via `cargo tree`, not to parse manifests.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.1: Dependency-frontier gate (D47)]
- [Source: _bmad-output/planning-artifacts/architecture.md#D47 — The `anyhow` frontier IS the dependency graph, not a rule (lines 2584–2653)]
- [Source: _bmad-output/planning-artifacts/architecture.md — Decision Index, D47 @ line 113]
- [Source: xtask/src/main.rs — existing gate structure, `run_ci`, `report`, `workspace_root`, test idiom]
- [Source: crates/opencmdb-core/Cargo.toml · crates/opencmdb-bin/Cargo.toml · xtask/Cargo.toml — current resolved deps]
- [Source: CLAUDE.md — "The dependency frontier is load-bearing (D47), and it is a gate"; `cargo xtask ci`; `--locked` policy]
- [Source: docs/project-context.md — sequencing step 2 (the D65 gates in xtask), workspace layout D47/D55/D56b]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- `cargo test -p xtask --locked` → 11 passed, 0 failed (5 new frontier tests + 1 bonus).
- `cargo run -p xtask -- ci` → `✅ frontier  domain graph clean; xtask depended on by nobody`.
- `cargo clippy --workspace --locked -- -D warnings` → clean. `cargo fmt --all --check` → clean.
- Empirical `cargo tree -p opencmdb-core -e normal --locked` confirmed the current graph carries none of `anyhow`/`axum`/`sqlx`/`askama`; `opencmdb-bin` carries no `xtask`. Gate is green by fact, not by assumption.

### Completion Notes List

- Implemented the D47 frontier as a **reflex CI gate** (D53) in `xtask/src/main.rs`, split per repo idiom into a pure parser (`crates_present_in_tree`, unit-tested on synthetic `cargo tree` text) and a thin subprocess wrapper (`cargo_tree`).
- **AC #1** — reads `cargo tree -p <pkg> -e normal --locked` for both crates; `-e normal` drops dev/build edges so `xtask` never falsely trips; green on the current workspace.
- **AC #2** — `frontier_flags_a_forbidden_dep_in_core` proves RED naming the crate + offending dep; `frontier_flags_xtask_in_bin` proves the bin→xtask direction.
- **AC #3** — satisfied *by construction*: the detector reads the resolved graph, never `Cargo.toml`, so a comment can't reach it. `frontier_token_match_rejects_lookalikes` locks the one real parsing risk (substring: `anyhow-macros` ≠ `anyhow`).
- Design note: gate is wired as **Gate 0** (before `ddl-collation`) — the frontier is the foundational law, so it reports first; behavior is otherwise identical to the task spec (folds into `ok`, runs before the informational `views-hash`).
- No new dependencies added (uses `std::process::Command` + the already-present `anyhow`). `xtask` remains a dependency of nobody.
- Out of scope (deferred as specified): the GitHub Actions runner that invokes `cargo xtask ci` is Story 1.4; MariaDB service + cargo-deny is Story 1.5.

### File List

- `xtask/src/main.rs` (modified) — added the `frontier` gate (Gate 0): `CORE_FORBIDDEN`, `gate_dependency_frontier`, `cargo_tree`, `crates_present_in_tree`; wired into `run_ci`; updated the module doc comment; added 6 tests in `#[cfg(test)] mod tests`.

## Change Log

- 2026-07-19 — Implemented Story 1.1 (dependency-frontier gate, D47). Added the `frontier` CI gate to `xtask ci` reading the `cargo tree` graph; 6 tests (proven-to-red + no-false-positive); all gates green, clippy/fmt clean. Status → review.
- 2026-07-19 — Code review (Blind Hunter · Edge Case Hunter · Acceptance Auditor). Applied 4 patch findings: (P1) factored `frontier_offenders` pure fn so the RED path is unit-tested and asserts the crate-naming message; (P2) `--charset utf8` kills the ASCII silent-green risk; (P3) `--depth 1` restricts to direct deps (D47 semantics, no transitive false positives); (P4) xtask now checked in core's tree too. 1 defer (feature/target/build-dep coverage → deferred-work.md, needs a GitHub issue), 2 dismissed. 12 tests green, gate green, clippy/fmt clean.
