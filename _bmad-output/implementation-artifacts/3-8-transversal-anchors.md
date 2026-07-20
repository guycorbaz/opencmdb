# Story 3.8: Transversal anchors

Status: review

## Story

As a maintainer,
I want the empty cross-cutting anchors in place — auth-deny middleware, `/metrics`, i18n `t!()`,
so that later features attach to existing seams instead of inventing them.

## Acceptance Criteria

1. **Given** any HTTP route, **when** it is requested unauthenticated, **then** an auth-deny middleware refuses it by default (deny-by-default seam; real auth is later).
2. **Given** the app, **when** `GET /metrics` is called, **then** it serves Prometheus metrics (raw `prometheus`), behind the scrape auth.
3. **Given** any user-facing string, **when** it is rendered, **then** it goes through `rust-i18n`'s `t!()` (EN/FR scaffolding; the forbidden-word lint seam noted).

## Tasks / Subtasks

- [x] Task 1 — Auth-deny middleware (AC: #1)
  - [x] `src/auth.rs`: `auth_deny(req, next)` layered on the whole router. Deny-by-default (401) for any path that is not explicitly public; the walking-skeleton public surfaces (`/`, `/gap`, `/assets/*`, `/healthz`) are an explicit allowlist (temporary — real user auth is the auth epic); `/metrics` requires the scrape token.
- [x] Task 2 — `/metrics` behind the scrape token (AC: #2)
  - [x] `src/metrics.rs`: a raw `prometheus` registry (D66 — raw prometheus + our handler, no middleware magic) with `opencmdb_build_info` and `opencmdb_http_requests_total`; `GET /metrics` gathers + encodes (`text/plain; version=0.0.4`). The scrape token (`OPENCMDB_METRICS_TOKEN`, `Authorization: Bearer …`) is enforced by the auth middleware; unset token → scrape denied (secure default).
- [x] Task 3 — i18n `t!()` seam (AC: #3)
  - [x] `rust_i18n::i18n!("locales", fallback = "en")` at the crate root; `locales/app.yml` (EN/FR); page-chrome strings and abstention-cause labels routed through `t!()`; locale from `OPENCMDB_LOCALE` (default `en`). D39/D65: the YAML is greppable — the forbidden-word lint seam is noted for the vocabulary gate.
- [x] Task 4 — Tests (AC: #1, #2, #3)
  - [x] Auth: an un-allowlisted path (`/admin`) → 401; `/metrics` without a token → 401; `/metrics` with the right Bearer token → 200 and carries a metric name; `/healthz` and `/` stay reachable.
  - [x] i18n: `t!()` returns the FR string when the locale is `fr`; the page renders through the `Strings` seam.
- [x] Task 5 — Verify (AC: #1–#3)
  - [x] `cargo test -p opencmdb-bin` green; `cargo xtask ci` green (all bin-only); clippy `--all-targets -D warnings` + fmt clean.

## Dev Notes

### Deny-by-default, but the walking skeleton stays viewable

The middleware refuses any route it does not explicitly recognize (401) — that is the seam the real auth epic attaches to. Because v0.1.0 must be viewable on the NAS before login exists, the walking-skeleton UI surfaces (`/`, `/gap`, `/assets/*`) and the `/healthz` probe are an EXPLICIT, temporary allowlist. When real auth lands, those move behind sessions; the seam does not change shape.

### `/metrics` — raw prometheus, behind the scrape token (FR43-44, D66)

Raw `prometheus` and our own handler (D66 — no middleware magic). The scrape auth is a Bearer token (`OPENCMDB_METRICS_TOKEN`); if it is unset, `/metrics` denies (a metrics endpoint must not be open by default — the cost lands on the wrong side, per the architecture's scrape note). Prometheus' `bearer_token` scrape config carries it.

### i18n is a greppable seam (D39/D65)

`rust-i18n` with a YAML source is deliberately greppable and diffable (D39). User-facing strings flow through `t!()` via a `Strings` struct passed to the templates and through the cause labels — so the vocabulary/forbidden-word gate (D65) can later extend to the locale files. This story installs the seam and the EN/FR scaffolding; full localization coverage is ongoing.

### Scope — seams, not features

Empty anchors, wired and tested: a deny-by-default layer, a live `/metrics`, and strings behind `t!()`. Real auth (sessions, login, per-route policy), rich metrics, and full translation coverage are later — they attach here.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 3.8: Transversal anchors]
- [Source: _bmad-output/planning-artifacts/architecture.md#D66 — /metrics = raw prometheus + our handler; i18n = rust-i18n/YAML (D39)]
- [Source: _bmad-output/planning-artifacts/architecture.md — authenticated Prometheus /metrics (FR43-44); the scrape-cost note]
- [Source: crates/opencmdb-bin/src/page.rs · main.rs — the router the middleware layers and the strings the seam routes]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- `cargo test -p opencmdb-bin` → 13 pass incl. `auth_denies_by_default_and_gates_metrics` (no DB, lazy pool) and `i18n_resolves_en_and_fr`. Full workspace 65 tests green vs local `mariadb:10.11.11`.
- Live smoke (running binary, `OPENCMDB_METRICS_TOKEN=s3cret OPENCMDB_LOCALE=fr`): `/` → 200 (public), `/admin` → 401 (deny-by-default), `/metrics` no token → 401, `/metrics` + `Bearer s3cret` → 200 exposing `opencmdb_build_info 1` and `opencmdb_http_requests_total`. Page rendered in FR ("observé vs déclaré", "Aucun enregistrement…").
- `cargo xtask ci` all 4 gates green (all deps bin-only — frontier untouched); clippy `--all-targets -D warnings` + fmt clean.

### Completion Notes List

- **AC #1** — `auth::auth_deny` layered on the whole router: any non-allowlisted path → 401 (the deny-by-default seam). The walking-skeleton public surfaces (`/`, `/gap`, `/assets/*`, `/healthz`) are an explicit temporary allowlist so v0.1.0 stays viewable pre-login; real user auth attaches here later.
- **AC #2** — `metrics::handler` serves the raw `prometheus` registry (`opencmdb_build_info`, `opencmdb_http_requests_total`) in the text format (D66: our own handler). The scrape Bearer token (`OPENCMDB_METRICS_TOKEN`) is enforced by the middleware; unset token → scrape denied (secure default).
- **AC #3** — `rust_i18n::i18n!("locales", fallback="en")`; `locales/app.yml` (EN/FR); page chrome + cause labels routed through `t!()` via a `Strings` struct passed to the templates; locale from `OPENCMDB_LOCALE`. D39/D65: greppable YAML — the forbidden-word lint seam is noted for the vocabulary gate.
- No new dependencies — `prometheus` and `rust-i18n` were already in `bin`'s Cargo.toml; frontier gate unaffected.
- `metrics::HTTP_REQUESTS` is incremented in the auth layer, so `/metrics` is non-empty from the first scrape; `metrics::init()` forces registration at startup.

### File List

- `crates/opencmdb-bin/src/auth.rs` (new) — deny-by-default middleware + scrape-token check.
- `crates/opencmdb-bin/src/metrics.rs` (new) — raw prometheus registry + `/metrics` handler + `init()`.
- `crates/opencmdb-bin/locales/app.yml` (new) — EN/FR translation source (D39 greppable).
- `crates/opencmdb-bin/src/main.rs` (modified) — `mod auth/metrics`, `i18n!`, locale + `metrics::init()`, `/metrics` route, auth layer, auth + i18n tests.
- `crates/opencmdb-bin/src/page.rs` (modified) — `Strings` seam + `strings()`, `cause_label` via `t!()`, templates fed `s`.
- `crates/opencmdb-bin/templates/gap.html` · `_gap_card.html` (modified) — strings read from `s.*`.
- `README.md` (modified) — walking-skeleton note updated (`/metrics`, locale, deny-by-default).

## Change Log

- 2026-07-20 — Implemented Story 3.8 (transversal anchors). Deny-by-default auth middleware (public UI allowlisted, `/metrics` behind a scrape Bearer token), raw-`prometheus` `/metrics` (build_info + request counter), and the `rust-i18n` `t!()` seam (EN/FR, page strings + cause labels). 65 tests green (auth + i18n covered without a DB); live smoke confirms all three anchors incl. FR rendering. Frontier/clippy(--all-targets)/fmt green. Status → review.
