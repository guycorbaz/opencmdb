# Story 1.4: CI runner calls xtask only (thin YAML)

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want the GitHub Actions workflow to hold no gate logic,
so that every gate is Rust that runs identically on a developer machine and in CI (D56).

## Acceptance Criteria

1. **Given** the CI workflow file, **when** it is inspected, **then** it only invokes `cargo xtask ci`, `cargo fmt --all --check`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace --locked` — and contains no bespoke gate logic.
2. **Given** a pull request where any gate is RED, **when** CI runs, **then** the check fails and names the failing gate.

## Tasks / Subtasks

- [x] Task 1 — Create `.github/workflows/ci.yml` (AC: #1)
  - [x] Triggers on `push` (branches: master) and `pull_request`.
  - [x] One `ci` job on `ubuntu-latest`: `actions/checkout@v4` → `dtolnay/rust-toolchain@stable` (components: clippy, rustfmt) → `Swatinem/rust-cache@v2`. Added a `concurrency` group (cancel-in-progress) — infra, not gate logic.
  - [x] Exactly four named command steps: `cargo fmt --all --check`, `cargo clippy --workspace --locked -- -D warnings`, `cargo xtask ci`, `cargo test --workspace --locked`.
  - [x] No bespoke gate logic in any `run:` step — verified by grep (the only mention of "grep"/"gate logic" is in the header comment). `cargo xtask ci` IS the gate surface.
- [x] Task 2 — Keep scope to the thin runner (AC: #1)
  - [x] No MariaDB service, no `cargo deny` (Story 1.5). No `rust-toolchain.toml`/Renovate (Story 1.6) — `@stable` used, with a comment noting 1.6 pins the MSRV.
  - [x] `--locked` on `clippy` and `test` (the building commands); `Cargo.lock` is the pin.
- [x] Task 3 — Validate locally what can be validated (AC: #1, #2)
  - [x] YAML parses (`yaml.safe_load` → 1 job, 7 steps).
  - [x] All four commands pass locally on the current tree (fmt ✓, clippy ✓, xtask ci all-green ✓, 23 tests ✓).
  - [x] AC #2 satisfied by construction: `run_ci()` → `Ok(false)` → non-zero `ExitCode`, with `report()` having printed `🔴 <gate>` (proven end-to-end in Story 1.2). A red gate fails the `cargo xtask ci` step and names the gate.
  - [x] The four `run:` lines are exactly the four cargo commands — no gate logic (grep confirmed).

## Dev Notes

### What this story is

The single load-bearing idea (D56): **all gates live in `cargo xtask ci`, in Rust, not YAML.** "Five greps scattered across a GitHub Actions YAML is a gate nobody can run locally, therefore a gate discovered broken after the push, therefore a gate that gets disabled. One command, identical locally and in CI; the YAML calls it and holds no logic." [Source: architecture.md#D56, lines 3235–3238] This story writes that thin YAML: checkout → toolchain → cache → four cargo commands. Nothing else.

### The four commands are already green locally

Everything the workflow runs passes on the current tree — verified repeatedly through Stories 1.1–1.3: `cargo xtask ci` (frontier/ddl-collation/vocabulary/fixtures all green), `cargo fmt --all --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace --locked` (23 tests). So the first CI run should be green. If it isn't, the environment differs (toolchain, network) — investigate that, not the gates.

### AC #2 is satisfied by construction

`run_ci()` returns `Ok(false)` when any gate is RED, and `main()` maps that to a non-zero `ExitCode` while `report()` has already printed `🔴 <gate-name> <message>`. So a red gate both fails the CI step (non-zero exit) and names itself in the log. This was proven end-to-end in Story 1.2 (a deliberately-wrong fixture MANIFEST produced `🔴 fixtures  … sha256 mismatch`). No CI-specific code is needed to satisfy AC #2 — the gate binary already does it.

### Exact toolchain / actions

- `actions/checkout@v4` — standard, current.
- `dtolnay/rust-toolchain@stable` with `components: clippy, rustfmt` — the fmt/clippy steps need those components. MSRV pinning (the workspace declares `rust-version = 1.96`, edition 2024) is **Story 1.6**; keep `@stable` here to avoid guessing a patch version, and 1.6 will switch CI to read `rust-toolchain.toml`.
- `Swatinem/rust-cache@v2` — caches the cargo registry + `target/`. This is build infrastructure, not gate logic, and does not violate AC #1's "no bespoke gate logic" (it re-implements no gate).

### Path discipline / scope guards

- File path is exactly `.github/workflows/ci.yml` (no `.github/` exists yet — this story creates it).
- Do not touch `xtask/src/main.rs` or any Rust — this story is CI infra only. The gate surface is complete for Epic 1's current stories.
- Do not add extra steps "for completeness" (build, doc, audit) — AC #1 says *only* those four commands. `cargo test`/`clippy` already build the workspace; a separate `cargo build` is redundant. `cargo deny` (audit) is explicitly Story 1.5.

### Can't fully validate here (honest limit)

A real CI run only happens on a push/PR to GitHub. Locally we can prove: the YAML parses, the four commands pass, the workflow contains no gate logic, and (by construction + prior e2e) a red gate would fail and name itself. The genuine end-to-end — a PR check going red on GitHub — is observed on the first push and is out of local reach. Note this in the completion, do not claim the GitHub run itself was verified.

### Testing standards summary

- No Rust tests to add (this is YAML infra). "Tests" here = the local validations in Task 3 (YAML parses; four commands green; no-logic grep).
- The workflow itself is the thing under test; its correctness is checked by the first real CI run.

### Project Structure Notes

- New file only: `.github/workflows/ci.yml`. No changes to crates, xtask, or Cargo files.
- Keep the YAML minimal and readable — it is a call site, not a program.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.4: CI runner calls xtask only (thin YAML)]
- [Source: _bmad-output/planning-artifacts/architecture.md#D56 — All gates live in `cargo xtask ci`, in Rust not YAML (lines 3235–3238)]
- [Source: CLAUDE.md — build/lint/test commands; "every gate lives here in Rust, never in YAML (D56/D65)"; `--locked` policy]
- [Source: xtask/src/main.rs — `run_ci`/`main` exit-code contract and `report()` naming the gate (AC #2 mechanism)]
- [Source: _bmad-output/implementation-artifacts/1-2-fixture-manifest-sha256-gate.md — the end-to-end proof that a red gate prints `🔴 <gate>` and fails]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- `python3 -c "import yaml; yaml.safe_load(...)"` → parses; 1 job `ci`, 7 steps.
- Local run of the four commands: `cargo fmt --all --check` ✓; `cargo clippy --workspace --locked -- -D warnings` ✓; `cargo xtask ci` → all gates green ✓; `cargo test --workspace --locked` → 23 passed ✓.
- `grep '^\s*run:'` → exactly the four cargo commands; no gate logic in any step.

### Completion Notes List

- Created `.github/workflows/ci.yml` — a thin runner (D56): checkout → `dtolnay/rust-toolchain@stable` (clippy+rustfmt) → `Swatinem/rust-cache@v2` → the four cargo commands as named steps. No gate logic in YAML; `cargo xtask ci` is the single gate surface, runnable identically locally and in CI.
- **AC #1** — the workflow invokes only the four required commands and holds no bespoke gate logic (grep-verified; the only textual mention of "grep/gate logic" is the explanatory header comment).
- **AC #2** — satisfied by construction: a red gate makes `cargo xtask ci` exit non-zero while `report()` prints `🔴 <gate>`, so the CI step fails and names the gate. Proven end-to-end in Story 1.2's fixture mismatch run.
- Scope held: no MariaDB service / `cargo deny` (Story 1.5), no `rust-toolchain.toml` / Renovate (Story 1.6). Added a `concurrency` cancel-in-progress group (infra convenience, not gate logic).
- **Honest limit:** a real GitHub Actions run only occurs on push/PR to GitHub — observed on the first push, out of local reach. Locally proven: YAML parses, four commands green, no gate logic, red-gate-names-itself by construction.

### File List

- `.github/workflows/ci.yml` (new) — the thin CI runner. No Rust or Cargo files changed.

## Change Log

- 2026-07-19 — Implemented Story 1.4 (CI runner thin YAML, D56). Added `.github/workflows/ci.yml`: checkout + stable toolchain (clippy/rustfmt) + rust-cache + the four commands (fmt --check, clippy --locked -D warnings, xtask ci, test --workspace --locked). No gate logic in YAML. YAML parses, all four commands green locally; AC #2 holds by construction (xtask exits non-zero + names the gate). Status → review.
- 2026-07-19 — Committed + pushed (`3f375dd`). **Real CI run verified GREEN on GitHub** (run 29703461241, job `ci`, 2m7s): all four steps + xtask gates green. Non-blocking annotation: Node.js 20 deprecation for actions/checkout@v4 (bump to @v5 → Story 1.6 / Renovate scope). Status → done.
