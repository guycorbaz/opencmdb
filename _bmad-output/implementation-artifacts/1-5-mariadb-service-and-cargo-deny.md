# Story 1.5: MariaDB service container and cargo-deny in CI

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want CI to run against the exact target database and to gate dependency risk,
so that dev = CI = prod (ARCH-8) and advisories/licenses cannot slip in.

## Acceptance Criteria

1. **Given** a pull request, **when** CI runs, **then** a `mariadb:10.11.11` service container is available for DB-touching tests to connect to.
2. **Given** a dependency with a security advisory or a disallowed license, **when** `cargo deny check advisories licenses` runs in CI, **then** the check fails.

## Tasks / Subtasks

- [x] Task 1 — Add the MariaDB 10.11.11 service container to the `ci` job (AC: #1)
  - [x] Added a `services.mariadb` block with `image: mariadb:10.11.11` (exact DSM 7 version, D64).
  - [x] Service `env`: `MARIADB_ROOT_PASSWORD: opencmdb`, `MARIADB_DATABASE: opencmdb_test`; `ports: - 3306:3306`.
  - [x] Health check `--health-cmd="healthcheck.sh --connect --innodb_initialized"` (+ interval/timeout/retries) so the job waits for a reachable DB.
  - [x] Job-level `env.DATABASE_URL: mysql://root:opencmdb@127.0.0.1:3306/opencmdb_test` for future DB tests.
  - [x] No DB test invented — the container is provisioned and idle (AC #1 = availability).
- [x] Task 2 — Add `deny.toml` at the workspace root (AC: #2)
  - [x] `deny.toml` with `[advisories]` (v2, vulnerabilities deny by default, `yanked = "deny"`) and `[licenses]` (v2). Allowlist tuned against the real 309-crate tree: MIT, Apache-2.0, BSD-3-Clause, BSL-1.0, ISC, Unicode-3.0, Zlib, CDLA-Permissive-2.0, CC0-1.0. `private = { ignore = true }` skips the unpublished `xtask`; AGPL-3.0-or-later allowed only for `opencmdb-core`/`opencmdb-bin` via scoped exceptions (so a copyleft *dependency* still fails).
  - [x] Minimal, cargo-deny 0.19.x schema; no `[bans]`/`[sources]`.
- [x] Task 3 — Add the cargo-deny step to CI (AC: #2)
  - [x] `taiki-e/install-action@v2` (tool: cargo-deny) + a named step `run: cargo deny check advisories licenses`. Tool + config, not bespoke YAML logic.
- [x] Task 4 — Validate locally (AC: #1, #2)
  - [x] `cargo deny check advisories licenses` → `advisories ok, licenses ok`.
  - [x] Prove-to-red: removing `Unicode-3.0` from the allowlist → `error[rejected]: failed to satisfy license requirements` naming the license; restored → green.
  - [x] YAML parses (1 job, services: mariadb:10.11.11, 9 steps).
  - [x] The four existing commands still pass locally (fmt/xtask ci/tests ✓).
- [ ] Task 5 — Push and verify the real CI run (AC: #1, #2)
  - [ ] After commit+push, watch the GitHub run (`gh run watch`): the MariaDB service starts healthy and the `cargo deny` step is green. (Verified post-push, as with Story 1.4 — status moves to done once the real run is green.)

## Dev Notes

### Two independent additions to the existing thin runner

Story 1.4 built `.github/workflows/ci.yml` (checkout → toolchain → cache → fmt/clippy/xtask ci/test), and its first real run was verified green on GitHub. This story adds two things to that same `ci` job, without disturbing the four existing steps:
1. a **MariaDB 10.11.11 service container** (AC #1) — the exact DSM 7 package, so CI == prod;
2. a **`cargo deny` step** + `deny.toml` (AC #2) — advisories + licenses gate.

### MariaDB service — the version is load-bearing (D64)

`mariadb:10.11.11` is not "a recent 10.11" — it is **the exact DSM 7 package version**, pinned precisely so that dev = CI = prod (project-context: "CI is pinned to `mariadb:10.11.11` — the exact DSM 7 package"). Do not float it to `10.11` or `latest`. **MariaDB is the ONLY supported engine (D64); SQLite and MySQL are OUT.** sqlx is built with the `mysql` feature (Cargo.toml) because MariaDB speaks the MySQL wire protocol — that is not a second backend, it is the driver for the one engine.

- GitHub service containers: the `mariadb` image exposes `healthcheck.sh`; `--health-cmd="healthcheck.sh --connect --innodb_initialized"` makes the runner wait until the server accepts connections and InnoDB is up.
- Expose `3306:3306` and set `DATABASE_URL` at job level so future DB tests connect with zero extra wiring.
- **No DB test exists yet** — this is intentional. The container being present + healthy satisfies AC #1; the walking skeleton (a later story) will be the first consumer.

### cargo-deny — a tool with config, not YAML logic

`cargo deny check advisories licenses` is a standard external check with its own config file (`deny.toml`), exactly like clippy has lints. Running it as a CI step does not violate D56's "no bespoke gate logic in YAML" — the logic is in cargo-deny + `deny.toml`, and the command is identical locally (`cargo deny check advisories licenses`). It is runnable on a dev machine (cargo-deny 0.19.8 is already installed here).

- **advisories**: checks the RustSec advisory DB; a dependency with a known vulnerability fails the check.
- **licenses**: an SPDX allowlist. Build it from what the 309-crate tree actually uses — run `cargo deny check licenses` and add each legitimately-present license (do not blanket-allow; the point is that a *disallowed* license fails, AC #2). Expect the usual permissive set (MIT, Apache-2.0, Unicode-3.0/Unicode-DFS-2016, BSD-2/3-Clause, ISC, Zlib, possibly MPL-2.0). If a crate needs a license clarification/exception, add a scoped `[[licenses.exceptions]]` or `[licenses.clarify]`, not a blanket allow.
- Pin the cargo-deny schema to 0.19.x (the installed version). The `[advisories]` and `[licenses]` sections changed across cargo-deny majors — write for 0.19, and let the local run confirm the schema.

### Prove-to-red for AC #2

The AC demands that a disallowed license (or an advisory) FAILS the check. Prove it locally: after the allowlist is green, temporarily delete one required license from `deny.toml`, run `cargo deny check licenses`, and confirm it exits non-zero naming the offending crate(s); then restore. Record the observed failure in the Debug Log — this is the D45 "proven-to-red" for this gate, mirroring how the xtask gates are proven.

### Scope guards

- Do NOT modify `xtask/src/main.rs` or any Rust — this story is CI + config only.
- Do NOT add `[bans]` dependency-graph rules or `[sources]` restrictions — advisories + licenses only (AC #2). Those are a later hardening if ever wanted.
- Do NOT change the `rust-toolchain` action or add Renovate — Story 1.6. (The Node-20 `checkout@v4` deprecation annotation is also 1.6's bump to `@v5`.)
- `deny.toml` lives at the workspace root (alongside `Cargo.toml`), the conventional location cargo-deny discovers.

### Honest limit

The MariaDB service being reachable by an actual test can't be proven until a DB test exists (later story). What this story proves: the service is declared correctly and starts **healthy** in the real GitHub run (visible in the job log), and `cargo deny` is green in that run. Watch the run to confirm both; don't claim a DB connection was exercised.

### Testing standards summary

- No Rust tests added (CI + config only). "Validation" = `cargo deny check advisories licenses` green locally, its proven-to-red, YAML parses, the four existing commands still pass, and the real GitHub run shows a healthy MariaDB service + green cargo-deny step.

### Project Structure Notes

- Files touched: `.github/workflows/ci.yml` (modified — add service + cargo-deny step), `deny.toml` (new, workspace root).
- No changes to crates, xtask, or `Cargo.toml`/`Cargo.lock`.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.5: MariaDB service container and cargo-deny in CI]
- [Source: docs/project-context.md — "CI is pinned to `mariadb:10.11.11` — the exact DSM 7 package — so dev = CI = prod"; MariaDB-only (D64), SQLite/MySQL OUT]
- [Source: _bmad-output/planning-artifacts/architecture.md#D64 — MariaDB 10.11+ the only engine; sqlx `mysql` feature is the driver, not a second backend]
- [Source: crates/opencmdb-bin/Cargo.toml — sqlx `=0.9.0` with `mysql` + `tls-rustls-ring`]
- [Source: .github/workflows/ci.yml — the Story 1.4 thin runner this story extends]
- [Source: _bmad-output/implementation-artifacts/1-4-ci-runner-thin-yaml.md — the runner's structure and the verified-green real CI run]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- `cargo deny check advisories licenses` → `advisories ok, licenses ok` (cargo-deny 0.19.8, 309-crate tree).
- Prove-to-red: `sed` out `Unicode-3.0` → `error[rejected]: failed to satisfy license requirements` / `rejected: license is not explicitly allowed`; restored → green.
- YAML parse: 1 job `ci`, `services.mariadb.image = mariadb:10.11.11`, 9 steps.
- No regression: `cargo fmt --all --check` ✓, `cargo xtask ci` all-green ✓, `cargo test --workspace --locked` ✓.

### Completion Notes List

- **AC #1** — added a `services.mariadb` container pinned to `mariadb:10.11.11` (exact DSM 7 version, D64: dev == CI == prod) with a `healthcheck.sh` readiness gate and `DATABASE_URL` set job-level. No DB test exists yet (later story), so the container is provisioned + healthy but idle — availability, per the AC.
- **AC #2** — added `deny.toml` (advisories + licenses) and a `cargo deny check advisories licenses` CI step. The license allowlist was tuned against the real tree, not blanket-allowed; a disallowed license fails (proven-to-red locally). AGPL is scoped to our own crates only, so a copyleft dependency would still be rejected; `xtask` (publish=false) is skipped via `private.ignore`.
- Consistent with D56: cargo-deny is a tool + config (`deny.toml`), runnable identically locally, not bespoke YAML gate logic.
- Scope held: no Rust/xtask/Cargo changes; no `[bans]`/`[sources]`; toolchain-pin/Renovate/checkout@v5 remain Story 1.6.
- **Honest limit / Task 5:** the MariaDB service starting healthy in a real GitHub run, and the cargo-deny step being green there, are verified on the first push (watched via `gh run`), exactly as Story 1.4's real run was. Status → done once that run is green. A real DB *connection* is not exercised until a DB test exists (later story).

### File List

- `.github/workflows/ci.yml` (modified) — added the `mariadb:10.11.11` service + `DATABASE_URL` job env, the `taiki-e/install-action` cargo-deny install, and the `cargo deny check advisories licenses` step.
- `deny.toml` (new, workspace root) — advisories + licenses config with a tree-tuned allowlist and scoped AGPL exceptions.

## Change Log

- 2026-07-19 — Implemented Story 1.5 (MariaDB service + cargo-deny, D64/ARCH-8). CI `ci` job now provisions a `mariadb:10.11.11` service (healthcheck + DATABASE_URL) and runs `cargo deny check advisories licenses` against a tree-tuned `deny.toml`. Local: cargo-deny green + proven-to-red (removing a license fails naming it), YAML parses, no regression. Real GitHub run watched at push (Task 5). Status → review.
