# Story 3.1: Binary bootstrap and `/healthz`

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want `opencmdb-bin` to boot an axum server with config, tracing, and a `/healthz` endpoint,
so that the composition root exists and is observable before any feature is built.

## Acceptance Criteria

1. **Given** the binary, **when** it starts, **then** it loads configuration (via the `config` crate) and initialises `tracing`/`tracing-subscriber`, and serves an axum app.
2. **Given** a running server, **when** `GET /healthz` is called, **then** it returns `200 OK` (liveness only — no dependencies checked yet).
3. **And** `cargo xtask ci`, `cargo clippy --workspace -- -D warnings`, and `cargo fmt --all --check` stay green; the frontier gate is unaffected (all of this is `opencmdb-bin`).

## Tasks / Subtasks

- [x] Task 1 — Configuration loading (AC: #1)
  - [x] In `opencmdb-bin/src/main.rs`, load a `bind` address via the `config` crate: `Config::builder().set_default("bind", "0.0.0.0:8080")` + `Environment::with_prefix("OPENCMDB")`, then `config.get_string("bind")`. Use `config.get_string`, NOT a `#[derive(Deserialize)]` struct, so `serde` need not become a direct `bin` dependency for this minimal bootstrap.
  - [x] `main` returns `anyhow::Result<()>` (bin is the composition root — `anyhow` is legitimate here, D47); wrap failures with `.context(...)`.
- [x] Task 2 — Tracing init (AC: #1)
  - [x] Initialise `tracing_subscriber::fmt` with an `EnvFilter` read from `OPENCMDB_LOG` (default `info`). One `init()` at startup.
- [x] Task 3 — The axum app + `/healthz` (AC: #2)
  - [x] Factor the router into `fn app() -> Router` (testable without a socket): `Router::new().route("/healthz", get(healthz))`.
  - [x] `async fn healthz() -> StatusCode { StatusCode::OK }` — liveness only; database reachability is Story 3.2.
  - [x] `main` binds a `tokio::net::TcpListener` to `bind` and `axum::serve`s `app()`; log the bind address at `info`.
- [x] Task 4 — Test (AC: #2)
  - [x] Add `tower = { version = "0.5", features = ["util"] }` as a `[dev-dependency]` of `opencmdb-bin`.
  - [x] `#[tokio::test]`: call `app().oneshot(GET /healthz)` and assert the status is `200 OK` — no real socket, no config, no DB.
- [x] Task 5 — Verify (AC: #1–#3)
  - [x] `cargo test -p opencmdb-bin` green; `cargo run -p opencmdb-bin` starts and `GET /healthz` returns 200 (smoke check acceptable locally).
  - [x] `cargo xtask ci` green (frontier unaffected — this is bin); `cargo clippy --workspace --locked -- -D warnings` and `cargo fmt --all --check` clean.

## Dev Notes

### What this is

The first real `opencmdb-bin` code: the composition root boots, is configurable, logs, and answers a liveness probe. It replaces the `println!("opencmdb — skeleton")` placeholder. No domain logic, no DB, no HTML yet — those attach to this seam in 3.2+.

### Read the file being replaced: `src/main.rs`

Currently a placeholder `fn main() { println!(...) }` with a doc comment about the walking skeleton. Replace `main` with the async bootstrap; keep/extend the module doc. Everything the later stories add (the Repository, the axum surface, askama) hangs off this `main`/`app()`.

### `bin` is the outside world (D47/D55)

`opencmdb-bin` is the composition root — HTTP, config, the clock, secrets all live here, and `anyhow` is legitimate (nobody matches on the variant; a `.context()` chain the operator reads on stderr is worth money). None of this touches `opencmdb-core`, so the frontier gate is unaffected. Do NOT add HTTP/config to core.

### Keep it minimal — no serde in bin yet

Use `config.get_string("bind")` rather than deserialising a settings struct, so this story adds no `serde` dependency to `bin`. A typed `Settings` struct (and `serde`) can come when the config surface actually grows (Story 3.2+ / the configuration story). `0.0.0.0:8080` is the default bind (containers bind all interfaces); `OPENCMDB_BIND` overrides it.

### Testing standards summary

- One `#[tokio::test]` driving `app()` via `tower::ServiceExt::oneshot` — no socket, deterministic. `tower` is a `bin` dev-dependency (axum already pulls tower transitively; the dev-dep just exposes `util`/`ServiceExt`).
- Run `cargo test -p opencmdb-bin`.

### Project Structure Notes

- Modified: `crates/opencmdb-bin/src/main.rs` (the bootstrap + `app()` + `/healthz` + test); `crates/opencmdb-bin/Cargo.toml` (`tower` dev-dep); `Cargo.lock`.
- No changes to `opencmdb-core` or `xtask`.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 3.1: Binary bootstrap and `/healthz`]
- [Source: _bmad-output/planning-artifacts/architecture.md#D47/D55 — bin is the composition root; anyhow legitimate here]
- [Source: crates/opencmdb-bin/Cargo.toml — axum 0.8, config, tokio (full), tracing, tracing-subscriber already present]
- [Source: crates/opencmdb-bin/src/main.rs — the placeholder being replaced]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- `cargo test -p opencmdb-bin` → 2 passed (`healthz_returns_200` via `oneshot`; `default_bind_is_all_interfaces_port_8080`).
- Smoke test: `OPENCMDB_BIND=127.0.0.1:18080 ./target/debug/opencmdb` → `GET /healthz` returned **HTTP 200**; tracing logged `opencmdb listening bind=127.0.0.1:18080`.
- `cargo run -p xtask -- ci` frontier/all-gates green; clippy `-D warnings` + fmt clean; full workspace green (2 bin + 23 core + 23 xtask).

### Completion Notes List

- Replaced the `println!` placeholder in `opencmdb-bin/src/main.rs` with an async bootstrap: `#[tokio::main]` → init tracing (`OPENCMDB_LOG` EnvFilter, default `info`) → load `bind` via the `config` crate (`OPENCMDB_BIND`, default `0.0.0.0:8080`) → bind a `TcpListener` → `axum::serve(app())`.
- **AC #1** — config + tracing + axum app. **AC #2** — `/healthz` returns `200 OK` (liveness; DB reachability is Story 3.2), proven by unit test AND a real-socket smoke test. **AC #3** — CI/clippy/fmt green; frontier unaffected (all in `bin`).
- Kept minimal: `config.get_string("bind")` (no `serde` added to `bin`); `app()` factored out so it is testable via `tower::ServiceExt::oneshot` without a socket.
- Docs-current-before-push: updated the GitHub `README.md` line that said `cargo run` starts a "placeholder binary" — it now serves `/healthz`.
- `tower` added as a `bin` dev-dependency (test-only; axum already pulls tower transitively).

### File List

- `crates/opencmdb-bin/src/main.rs` (modified) — the bootstrap, `app()`, `/healthz`, tests.
- `crates/opencmdb-bin/Cargo.toml` (modified) — `tower` dev-dependency.
- `README.md` (modified) — the `cargo run` line now reflects the `/healthz` server.
- `Cargo.lock` (modified).

## Change Log

- 2026-07-20 — Implemented Story 3.1 (binary bootstrap + `/healthz`). `opencmdb-bin` now boots an axum server (config `bind`, tracing) and answers `GET /healthz` with 200; replaces the placeholder `main`. 2 tests + a real-socket smoke test (HTTP 200). Frontier unaffected; CI/clippy/fmt green. First real bin code — start of Epic 3. Status → review.
