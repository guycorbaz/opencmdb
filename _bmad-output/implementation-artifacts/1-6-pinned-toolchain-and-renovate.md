# Story 1.6: Pinned toolchain and Renovate automation

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want a pinned toolchain and safe automated dependency updates,
so that builds are reproducible and dependencies stay current without manual toil.

## Acceptance Criteria

1. **Given** the repository, **when** a build runs, **then** `rust-toolchain.toml` pins the MSRV and CI uses exactly that toolchain.
2. **Given** a patch or minor dependency update, **when** Renovate opens it and CI is green, **then** it is grouped and auto-merged.
3. **Given** a breaking (major) dependency update, **when** Renovate opens it, **then** it is a dedicated, non-grouped PR (never two breaking changes in one commit).

## Tasks / Subtasks

- [x] Task 1 — Add `rust-toolchain.toml` pinning the MSRV (AC: #1)
  - [x] `rust-toolchain.toml` at root: `channel = "1.96"`, `components = ["clippy", "rustfmt"]`, `profile = "minimal"`. (rustup resolves `"1.96"` to the latest 1.96.x = 1.96.1, ≥ MSRV.)
  - [x] Verified: build/test/clippy(`-D warnings`)/fmt all pass on the pinned toolchain.
- [x] Task 2 — Make CI use exactly the pinned toolchain (AC: #1)
  - [x] Replaced `dtolnay/rust-toolchain@stable` + `Swatinem/rust-cache@v2` with `actions-rust-lang/setup-rust-toolchain@v1` (reads `rust-toolchain.toml`, installs channel + components, and caches).
  - [x] The four command steps + cargo-deny unchanged; now run on the pinned toolchain.
- [x] Task 3 — Bump `actions/checkout` v4 → v5
  - [x] `actions/checkout@v4` → `@v5` (clears the Node-20 deprecation annotation).
- [x] Task 4 — Add Renovate config (AC: #2, #3)
  - [x] `renovate.json` at root extends `config:recommended`.
  - [x] Rule A: `["patch","minor"]` → grouped + `automerge: true` (AC #2).
  - [x] Rule B: `["major"]` → `automerge: false`, not grouped → dedicated PR per major dep (AC #3).
  - [x] Cargo + github-actions managers via `config:recommended`.
- [x] Task 5 — Validate locally (AC: #1, #2, #3)
  - [x] On the pinned toolchain: `cargo fmt --all --check` ✓, `clippy --workspace --locked -- -D warnings` ✓, `cargo xtask ci` all-green ✓, `cargo test --workspace --locked` ✓, `cargo deny check advisories licenses` ✓.
  - [x] `renovate.json` valid JSON (2 rules, extends config:recommended). `renovate-config-validator` not installed locally → JSON-parse + schema-shape check only.
  - [x] `.github/workflows/ci.yml` parses (8 steps; checkout@v5, setup-rust-toolchain@v1).
- [ ] Task 6 — Push and verify the real CI run (AC: #1)
  - [x] Pushed (`4a681a1`); GitHub run `29704381552` GREEN in 1m50s: `Set up Rust (pinned via rust-toolchain.toml)` installed 1.96, `checkout@v5` used, all steps green, and the run has NO annotations (Node-20 deprecation cleared).
  - [ ] Renovate activates only once its GitHub App is installed on `guycorbaz/opencmdb` — a one-time maintainer action (Guy). The config is the deliverable; AC #2/#3 are satisfied by config, live when the app is enabled.

## Dev Notes

### This closes Epic 1

Stories 1.1–1.5 built the gates and the thin CI runner (verified green on real GitHub CI). This story pins the toolchain so CI, dev, and prod compile identically on the declared MSRV, and wires Renovate so dependencies stay current safely. After this, Epic 1 ("the gates tie") is complete.

### The toolchain pin — MSRV, exactly

`Cargo.toml` declares `rust-version = "1.96"` (edition 2024). `rust-toolchain.toml` pins `channel = "1.96"` so every build — local, CI, and a future prod build — uses that exact toolchain. Pinning the MSRV *floor* (not the latest patch) is deliberate: CI then fails if code accidentally relies on a Rust feature newer than the declared minimum. Verified locally: build + test + clippy(`-D warnings`) + fmt all pass on 1.96, and clippy/rustfmt are installed on it.

Adding `rust-toolchain.toml` also changes LOCAL behaviour: `cargo` in this repo now selects 1.96 via the rustup override — that is the point (dev == CI). `profile = "minimal"` keeps the toolchain lean (no docs/etc.).

### CI must use the file, not an independent pin

Today CI installs `dtolnay/rust-toolchain@stable`, which resolves to whatever stable the runner offers — independent of the file. To satisfy "CI uses exactly that toolchain", switch to `actions-rust-lang/setup-rust-toolchain@v1`, which reads `rust-toolchain.toml` and installs the pinned channel + components, and which includes the build cache (so `Swatinem/rust-cache@v2` is folded in and removed). This makes `rust-toolchain.toml` the single source of truth. The four cargo steps and the cargo-deny step are untouched.

### Renovate — grouped/auto-merge for safe updates, isolated for breaking ones

`renovate.json` extends `config:recommended` (enables the Cargo + github-actions managers). Two rules:
- **patch + minor** → grouped into one PR and `automerge: true`. With the CI gate green, these low-risk updates merge without manual toil (AC #2).
- **major** → `automerge: false`, and NOT grouped — Renovate opens a dedicated PR per major dependency by default, so two breaking changes never share a commit (AC #3).

**Activation is a GitHub-side step:** Renovate only runs once its GitHub App is installed on `guycorbaz/opencmdb`. The config is the deliverable here; enabling the app is a one-time maintainer action noted for Guy. This is why AC #2/#3 are validated by config correctness, not a live Renovate run.

### Scope guards

- No Rust/xtask/Cargo source changes (the code already builds on 1.96). `Cargo.toml`'s `rust-version` stays `1.96` — `rust-toolchain.toml` complements it, it does not replace it.
- Do not add `[bans]`/dependency pins in Renovate beyond the update-type rules — keep it to grouping + automerge policy.
- No new gate in `cargo xtask ci` — this story is toolchain + CI + Renovate config only.

### Honest limit

Two things are only fully confirmed off-machine: the real CI run installing 1.96 from the file (watched at push, as with 1.4/1.5), and Renovate's behaviour (only observable once the app is installed and it opens its first PRs). Local validation covers: 1.96 builds everything, `renovate.json` is valid, the YAML parses.

### Testing standards summary

- No Rust tests (toolchain + CI + config). "Validation" = the four commands green on 1.96 locally, `renovate.json` valid, YAML parses, and the real GitHub run green with the Node-20 annotation gone.

### Project Structure Notes

- New files: `rust-toolchain.toml` (root), `renovate.json` (root). Modified: `.github/workflows/ci.yml`.
- Per the docs-current-before-push rule: this change touches the build toolchain and CI, which the README and Administrator Manual mention ("Rust 1.96+"). Confirm those still read correctly (they say "1.96+", which remains accurate) and update if the pin changes what they claim.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.6: Pinned toolchain and Renovate automation]
- [Source: Cargo.toml — `rust-version = "1.96"`, edition 2024]
- [Source: .github/workflows/ci.yml — the runner from Stories 1.4/1.5 this story adjusts]
- [Source: _bmad-output/implementation-artifacts/1-4-ci-runner-thin-yaml.md · 1-5-mariadb-service-and-cargo-deny.md — prior CI stories and the Node-20 annotation]
- [Source: CLAUDE.md — `--locked`/reproducible-builds policy; Docs-current-before-push convention]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- On the pinned toolchain (rustc 1.96.1 via the `"1.96"` override): fmt ✓, clippy `-D warnings` ✓, `cargo xtask ci` all-green ✓, `cargo test --workspace --locked` ✓, `cargo deny check advisories licenses` ✓.
- `renovate.json` → valid JSON, 2 packageRules, extends `config:recommended`.
- `.github/workflows/ci.yml` → parses; 8 steps; `actions/checkout@v5`; `actions-rust-lang/setup-rust-toolchain@v1`.

### Completion Notes List

- **AC #1** — `rust-toolchain.toml` pins `channel = "1.96"` (the declared MSRV); CI switched to `actions-rust-lang/setup-rust-toolchain@v1`, which reads that file, so CI uses exactly the pinned toolchain. Everything builds/tests/lints on it locally.
- **AC #2** — Renovate rule groups patch+minor updates into one PR and auto-merges them (green CI gates it).
- **AC #3** — Renovate rule leaves major updates ungrouped and non-auto-merged → a dedicated PR per major dependency, so two breaking changes never share a commit.
- Bumped `actions/checkout@v4 → @v5`, clearing the Node-20 deprecation annotation seen on 1.4/1.5.
- No Rust/Cargo source changes; `Cargo.toml`'s `rust-version = "1.96"` stays and is complemented by the toolchain file.
- **Docs-current-before-push:** checked the docs that mention the toolchain — README ("recent Rust toolchain (1.96+)") and the Administrator Manual ("Rust 1.96+") remain accurate; no change needed.
- **Honest limits / Task 6:** the real CI installing 1.96 from the file (watched at push) and Renovate's live behaviour (only after the GitHub App is installed) are confirmed off-machine. Status → done once the real run is green.

### File List

- `rust-toolchain.toml` (new, root) — pins the MSRV toolchain + clippy/rustfmt, profile minimal.
- `renovate.json` (new, root) — grouped auto-merge for patch/minor; isolated non-auto-merged PRs for major.
- `.github/workflows/ci.yml` (modified) — `checkout@v5`; `actions-rust-lang/setup-rust-toolchain@v1` reading the pinned toolchain (replacing dtolnay + Swatinem).

## Change Log

- 2026-07-19 — Implemented Story 1.6 (pinned toolchain + Renovate). Added `rust-toolchain.toml` (channel 1.96, MSRV) and switched CI to `actions-rust-lang/setup-rust-toolchain@v1` so CI uses exactly the pinned toolchain; bumped `actions/checkout@v5`. Added `renovate.json`: patch/minor grouped + auto-merged, major isolated + non-auto-merged. Local: everything green on the pinned toolchain, renovate.json valid, YAML parses. Real GitHub run watched at push (Task 6); Renovate live once its app is installed. Closes Epic 1. Status → review.
- 2026-07-19 — Pushed (`4a681a1`); **real GitHub CI run verified GREEN** (run 29704381552, 1m50s): pinned toolchain installed from `rust-toolchain.toml`, `checkout@v5`, and NO annotations (Node-20 deprecation cleared). Status → done. **Epic 1 complete.**
- 2026-07-20 — **Renovate policy CHANGED (Guy):** this story's grouped-auto-merge (AC #2/#3) is superseded by a **notify-only** config (`dependencyDashboardApproval: true`, commit `1f17bf2`) — Renovate records updates in the Dependency Dashboard issue and touches no code without approval. Renovate app installed by Guy; `master` branch protection now requires the `ci` check (enforce_admins off, so admin pushes still pass).
