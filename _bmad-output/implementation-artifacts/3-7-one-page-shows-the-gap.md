# Story 3.7: One page shows the gap

Status: review

## Story

As a maintainer,
I want a single web page that renders the real gap with Askama + HTMX + committed CSS,
so that a human sees the observed-vs-declared difference.

## Acceptance Criteria

1. **Given** the running app, **when** the page is served, **then** it renders the declared record, the linked observation, and the gap between them (Askama template, HTMX interactivity, committed CSS — no CDN).
2. **Given** the UX baseline, **when** the page loads, **then** design tokens are applied and `app.js` manages focus on HTMX swaps (UX-DR accessibility); dark theme default.
3. **And** the page shows the abstention count/reach honestly (FR39 min); it never presents an abstention as a reproach.

## Tasks / Subtasks

- [x] Task 1 — Committed front-end assets (AC: #1, #2)
  - [x] `assets/app.css` — design tokens as CSS variables (dark default `[data-theme]`, tinted-neutral base, amber accent reserved, observed=muted / declared=crisp, severity by weight not hue — no red), monospace for addresses, 8px grid, 3px radius, hairline borders. `prefers-reduced-motion` respected.
  - [x] `assets/app.js` — focus the swapped card on `htmx:afterSwap` (keyboard-first accessibility).
  - [x] `assets/htmx.min.js` — vendored, pinned htmx 2.0.4 (served locally, NO CDN).
  - [x] Embed with `rust-embed` (already a dep) and serve `GET /assets/{*path}` with correct content-types.
- [x] Task 2 — The reconciled view + templates (AC: #1, #3)
  - [x] `templates/gap.html` (full page chrome) + `templates/_gap_card.html` (the card, shared by page and fragment) — Askama 0.16.
  - [x] `src/page.rs`: a PURE `build_view(declared, observations, preferred_ipv4) -> ReconciledView` that groups declared by entity, picks the perimeter entity (env `OPENCMDB_ENTITY_IPV4` or the first declared entity carrying an `ipv4`), calls `opencmdb_core::reconcile`, and shapes gaps + observed rows + abstention rows for rendering. Honest empty state when no declared entity.
  - [x] `reconcile_view(pool)` loads `declared_attribute` + `observation_record` (facts deserialized) via the adapter and calls `build_view`.
- [x] Task 3 — Routes + scan wiring (AC: #1)
  - [x] `GET /` → full page; `GET /gap` → the card fragment (HTMX refresh target). Added to `app()`.
  - [x] Startup scan gated on `OPENCMDB_SCAN_CIDR`: a background task builds `ArpPingConnector::from_cidr`, polls, and ingests observations — finally wiring the Story 3.5 connector into the running app (real observed state).
  - [x] Read bodies `load_declared_attributes` / `load_observation_facts` in the adapter (generic over `Executor`, like the others).
- [x] Task 4 — Tests (AC: #1, #2, #3)
  - [x] Pure `build_view` tests (no DB): a drift gap surfaces; an out-of-perimeter observation is counted as reach; the empty state when no declared entity.
  - [x] DB-gated (`DATABASE_URL`, serialized via `DB_TEST_LOCK`): seed a declared entity + a conflicting observation, `GET /` → 200 and the response body carries the drift gap.
- [x] Task 5 — Verify (AC: #1–#3)
  - [x] `cargo test -p opencmdb-bin` green; `cargo xtask ci` green (frontier: all new deps bin-only); `cargo clippy --workspace --all-targets --locked -- -D warnings` + fmt clean.

## Dev Notes

### The page renders whatever the DB yields — honestly

The page reconciles the persisted `declared_attribute` rows against the persisted `observation_record` facts through the SAME pure `reconcile` engine (Story 3.6). On real ping-only data the common outcome is a **clear** match on the entity's `ipv4` plus `OutOfPerimeter` reach (undocumented IPs the scan saw) and `NoObservedValue` reach (declared `hostname`/`mac` the ping cannot observe). A drift gap needs a field both sides carry. No fake data is ever seeded (honesty + no private data); an empty DB shows a calm "no declared record yet" state.

### Design tokens, applied (UX spec §Visual Design Foundation)

Dark is default (`data-theme="dark"`, home-lab audience). Tinted-neutral cold-indigo base, one amber accent reserved for the document action, **observed = muted / declared = crisp**, severity by luminosity/weight never hue (no red). Monospace for IPs/MACs/hostnames so addresses align. Committed CSS (no Tailwind build step for the walking skeleton — hand-authored tokens; the standalone-Tailwind `cargo xtask css` pipeline lands when the UI grows), no CDN — htmx vendored and served locally.

### Abstention is reach, not debt (FR39)

The "Reach" section counts what the engine saw but could not place, framed as coverage — never a reproach. This is the honest self-diagnostic the UX spec insists on ("I don't know, displayed as a map").

### Scan wiring — the walking skeleton closes the loop

`OPENCMDB_SCAN_CIDR` (optional) turns on a one-shot background startup scan: the real `ArpPingConnector` pings the subnet and ingests observations, so the page shows genuinely observed state. Unset → no scan (the page still renders the declared side + `NoObservedValue` reach). The periodic scheduler (FR6) is later.

### Testing standards summary

- Pure `build_view` `#[test]`s (no DB/network) for the drift gap, reach counting, and the empty state — the testable seam, like the gap engine.
- One DB-gated HTTP round-trip (`DATABASE_URL`, `DB_TEST_LOCK`) proving `GET /` renders a real gap end to end.

### Project Structure Notes

- New: `crates/opencmdb-bin/src/page.rs`, `templates/gap.html`, `templates/_gap_card.html`, `assets/{app.css,app.js,htmx.min.js}`.
- Modified: `crates/opencmdb-bin/src/main.rs` (routes, `mod page`, startup scan), `crates/opencmdb-bin/src/repo.rs` (read bodies), `Cargo.toml` (no new deps — rust-embed/askama already present).

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 3.7: One page shows the gap]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Visual Design Foundation · Accessibility Considerations]
- [Source: crates/opencmdb-core/src/gap/mod.rs — the `reconcile` engine the page renders]
- [Source: crates/opencmdb-bin/src/arp_ping.rs — the connector the startup scan wires in]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- Pure `build_view` tests (no DB): drift gap surfaces, out-of-perimeter counted as reach (2 abstentions — OutOfPerimeter + the entity's own `ipv4` becoming NoObservedValue), empty state. 3 pass.
- Full bin suite vs local `mariadb:10.11.11` (port 3307): 11 pass incl. `index_renders_the_real_gap` (GET / → 200, body carries `nas`/`intruder`) and the 3.3/3.5 round-trips. Serialized via `DB_TEST_LOCK`.
- Live smoke: ran the binary, `GET /` renders the entity + the `hostname: nas → intruder` gap + Reach; `/gap` returns `text/html`; `/assets/{app.css,app.js,htmx.min.js}` serve 200 with correct content-types (all local, no CDN).
- Full workspace green (63 tests); `cargo xtask ci` all 4 gates green (frontier confirms no new deps); clippy `--all-targets -D warnings` + fmt clean.

### Completion Notes List

- **AC #1** — `GET /` reconciles persisted `declared_attribute` vs `observation_record` (facts deserialized) through the pure `reconcile` engine (3.6) and renders declared + observed + gap with Askama; HTMX `Refresh` swaps the `/gap` card fragment; htmx vendored + served locally (no CDN).
- **AC #2** — hand-authored `app.css` applies the UX tokens (dark default `[data-theme]`, tinted-neutral base, amber accent reserved, observed=muted / declared=crisp, severity by weight not hue — no red, mono addresses, 3px radius, hairline borders, `prefers-reduced-motion`); `app.js` focuses the swapped card on `htmx:afterSwap` (keyboard-first).
- **AC #3** — a "Reach" section counts abstentions grouped by cause, framed as coverage ("what we saw but could not place — reach, not debt"), never a reproach.
- The view builder `build_view` is a PURE function (no DB) — the testable seam; `reconcile_view` is the thin DB edge. Empty DB → an honest "no declared record yet" state (no fabricated data).
- Startup scan wired: `OPENCMDB_SCAN_CIDR` runs the real `ArpPingConnector` (3.5) on a DEDICATED thread with its own current-thread runtime + own pool. Reason: `Connector::poll` holds a `&mut dyn ObservationSink` across an await → not `Send` → cannot `tokio::spawn` on the multi-thread runtime (2.3 deferred the scheduler's Send story). `block_on` on a current-thread runtime imposes no `Send` bound.
- No new dependencies — `rust-embed`, `askama`, `chrono`, `uuid`, `serde_json` were already present; frontier gate unaffected (all bin-only).

### File List

- `crates/opencmdb-bin/src/page.rs` (new) — pure `build_view` + view models + Askama templates + `GET /`, `GET /gap`, `GET /assets/{*path}` handlers + rust-embed assets + 3 pure tests.
- `crates/opencmdb-bin/templates/gap.html` (new) — full page chrome, includes the card.
- `crates/opencmdb-bin/templates/_gap_card.html` (new) — the reconciliation card (shared by page + fragment).
- `crates/opencmdb-bin/assets/app.css` (new) — hand-authored design tokens.
- `crates/opencmdb-bin/assets/app.js` (new) — focus management on HTMX swaps.
- `crates/opencmdb-bin/assets/htmx.min.js` (new) — vendored, pinned htmx 2.0.4.
- `crates/opencmdb-bin/src/repo.rs` (modified) — `load_declared_attributes`, `load_observation_facts`; `executor()` now `pub(crate)`.
- `crates/opencmdb-bin/src/main.rs` (modified) — `mod page`, `/` `/gap` `/assets` routes, `spawn_startup_scan`, DB-gated `index_renders_the_real_gap` test.

## Change Log

- 2026-07-20 — Implemented Story 3.7 (one page shows the gap). Askama page + HTMX + hand-authored committed CSS (design tokens, dark default, observed/declared distinction, no red, Reach = abstention-as-coverage). Pure `build_view` reconciles persisted declared vs observed through the 3.6 engine; startup scan (`OPENCMDB_SCAN_CIDR`) finally wires the 3.5 connector in. 63 tests green (3 pure page + 1 DB HTTP round-trip); frontier/clippy(--all-targets)/fmt green; live smoke confirms page + assets. Status → review.
