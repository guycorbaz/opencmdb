---
stepsCompleted: [1, 2]
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/architecture.md
  - _bmad-output/planning-artifacts/ux-design-specification.md
resumePoint: >
  Step 3 (create-stories) IN PROGRESS. Requirements inventory, FR coverage map,
  and the approved 23-epic list (v4) are complete. Detailed stories written for
  Epic 1 only. NEXT: write stories for Epic 2 (Le contrat de connecteur), then
  Epics 3–23, one epic at a time (chosen cadence), each reviewed before append.
  After all epics: step-04 final validation. Resume by invoking
  bmad-create-epics-and-stories and continuing step-03 from Epic 2.
---

# opencmdb - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for opencmdb, decomposing the requirements from the PRD, UX Design Specification, and Architecture into implementable stories. Slicing bias (per product owner): **many small, independently deliverable stories** and **many fine-grained epics**, each epic ideally a releasable feature increment.

## Requirements Inventory

### Functional Requirements

**Discovery & Data Sources**
- FR1: Connect a UniFi controller as a discovery source (URL + API key).
- FR2: Discover devices, IPs, switch ports, SSIDs, VLANs, DHCP leases from UniFi without elevated network privileges.
- FR3: Declare subnets to be scanned by a generic ARP/ping source.
- FR4: Discover active devices on declared subnets; enrich with hostnames where available.
- FR5: Represent each source's state on two independent axes — liveness (`live`/`blind` + named cause) and capability — per (source, scope); report capability downgrades as notifiable events.
- FR6: Configure per-source discovery cadence within bounds; trigger an on-demand scan.
- FR7: Each source exposes a dated capability descriptor that travels with each observation batch; a capability reduction is a notifiable event; observations interpreted under the descriptor in force when collected.
- FR8: Distinguish a source outage from genuine device disappearance.

**Reconciliation & Device Identity**
- FR9: Stably identify a device across changing IP/MAC via composite identity at two levels — L1 interface identity, L2 device grouping (both inferred).
- FR10: Reconcile observed data against declared records by identity.
- FR11: Keep declared and observed data as distinct, linked records; observations never overwrite declared.
- FR12: Review unreconciled discoveries in a triage inbox.
- FR13: Document a discovery in one action — `document-all` (new) and field-selective `document-field` (drifted re-discovery); observed record never modified.
- FR14: From the inbox: create, attach, exclude, snooze, or accept-gap (gap stays open + keeps counting, mandatory note, wakes on observed change not a clock).
- FR15: Remember triage decisions (incl. exclusions) so resolved items do not reappear.
- FR16: When identity is ambiguous, present candidate matches with evidence and mark unresolved; never guess or merge. Abstention is a first-class persisted outcome.
- FR16b: Display, count, and group abstention by cause; never a reproach; counter measures reach, not debt.
- FR17: Triage multiple discoveries in bulk.
- FR18: View what changed since last visit, prioritized (new/conflict vs routine churn).
- FR19: Suppress observation-derived alerts/divergences for a blind (source, scope); retain last-known state.
- FR20: Surface conflicting observations tagged by source (only when two capable sources disagree); never silently pick; never propagate to declared.

**IP Address Management**
- FR21: Manage subnets, VLANs, and DHCP ranges.
- FR22: View per-subnet IP occupancy (used/free/declared/observed).
- FR23: Find a free IP address within a subnet.
- FR24: Detect IP conflicts (same IP on two MACs; static-declared IP inside a DHCP range) and identify the devices.
- FR25: Document IPv6 subnets/addresses (observation-only; active IPv6 scanning out of MVP).

**Applications & Hosted Software**
- FR26: Record software instances (name, version, listening ports) hosted on a device; anchor follows D15 (entity_id never rewritten).
- FR27: Group software into applications with an owner and criticality.
- FR28: Declare `hosts`/`exposes` relationships (MVP); `depends_on`/`connects_to` are Growth.
- FR29: Device record answers "Hosted here" with one containment hop, no traversal (never called "Impact"; true impact traversal is Growth).

**Alerts & Notifications**
- FR30: Raise alerts for an unknown device appearing, a documented IP unseen for N days, and an IP conflict.
- FR31: Receive alerts in-app and via a generic outbound webhook.
- FR32: Every alert carries a stable deep link opening the exact object in a focused view.
- FR33: Act on an alert (resolve / accept-gap / exclude) from the linked object.
- FR34: Configure alert thresholds; mute or snooze specific alerts.
- FR35: Configure which alert types deliver through which channel (in-app; webhook at MVP).

**Insight, History & Reporting**
- FR36: View a self-diagnostic dashboard (source health, reconciliation lag/queue, declared coverage, open-divergence trend, inbox health).
- FR37: Record timestamped observation history per device (first/last seen, IP↔MAC history).
- FR38: Configure observation retention (default 90 days); first/last-seen and IP-history rollups retained indefinitely.
- FR38b: Ephemeral-interface lifecycle — a locally-administered address unseen for a window (default 30d) moves to `dormant` (excluded from divergence metrics/candidate generation, still queryable, returns to active on re-observation); dormancy window must be shorter than observation retention (else startup failure).
- FR39: Search by IP/MAC/hostname/device and view its full record.

**Data Lifecycle & Editing**
- FR40: Edit the declared attributes of a record (name, tags, owner, role, notes).
- FR41: Decommission, archive, or delete a declared device/subnet/application; reconciliation reflects the change.
- FR42: Back up and restore the full dataset (export/import).

**Integration & API**
- FR43: Read core entities (devices, interfaces, subnets, IPs, applications, alerts, observations) via a read-only JSON API.
- FR44: Scrape opencmdb-specific metrics from an authenticated Prometheus `/metrics` endpoint.

**Administration, Security & Operations**
- FR45: First-run setup wizard from empty state through connecting a source (or declaring a subnet), first scan, and initial triage.
- FR46: Authenticate with a local login and session.
- FR47: Store source credentials and passwords encrypted/hashed at rest.
- FR48: Rotate a source's API key; back up/restore secrets in encrypted form (envelope: master key → data key → credential fields); master-key rotation at MVP.
- FR49: Log security-sensitive events (authentication, secret access/rotation).
- FR50: Use the interface in English or French.
- FR51: Configure the external base URL for deep links, with a tolerant fallback and a warning when unset.
- ~~FR52: opt-in anonymous telemetry — REMOVED FROM MVP (number retained, not reused).~~

**Topology**
- FR53: View network topology as a structured list/table — connections auto-populated for UniFi, manually entered otherwise.

**Architecture Constraint (not a user-facing FR):** data model and auth are multi-user-ready from day one (single admin provisioned at MVP; read-only/multi-user role is Growth).

### NonFunctional Requirements

**Performance**
- NFR1: Full discovery cycle + reconciliation diff over reference dataset at p95 ≤ 120 s while UI stays responsive (< 2 s); read/write path separation (exclusively-owned writer, concurrent readers).
- NFR2: Primary UI views render at p95 ≤ 1.5 s on the reference NAS while a discovery cycle is writing.
- NFR3: Time-to-first-value < 15 min (populated UniFi inventory + first findings); install < 30 min; validated with ~5 testers.

**Reliability & Reconciliation Correctness**
- NFR4: Release gate = binary adversarial trap suite (truth-table failures = 0 at device level, ~50 scenarios × positive/negative, three gating columns must-not-merge / must-merge / must-abstain). Bulk stats are observability and gate nothing. Labeled fixture is a synthetic+seeded architecture deliverable with a mandatory one-sentence reason per expectation.
- NFR5: Never-overwrite invariant enforced + anti-regression tests; no code path writes a declared field with a non-human author; divergence computation never consults how a declared value was obtained.
- NFR6: Reconciliation cycles and inter-source precedence are idempotent and independent of ingestion order (fuzzed arrival order).
- NFR7: 0 false "device-gone" events — an observation is structurally incapable of expressing "gone"; absence derived only when liveness is `live`; presence requires explicit hysteresis (N failures over window T).
- NFR7b: Silent schema-drift defence — collections feeding presence never default to empty on a parse miss; a population collapse is classified a source event, not mass device departure.
- NFR8: Four falsifiable degradation assertions under fault injection — monotone honesty (fault only removes knowledge), bounded blast radius, convergence after recovery, exactly one actionable notification; version drift tested by replaying raw recorded bytes; bounded tested version matrix (Network application 10.4.x).

**Security**
- NFR9: Threat model in three claims — NFR9a credentials never plaintext in DB/dump/WAL/logs/API (byte-scan testable); NFR9b app never writes master key into data volume, startup FAILS if key path resolves inside the data dir or key file is group/other-readable; NFR9c backup copying both key and DB is a documented non-guarantee.
- NFR10: Credentials and passwords never stored in plaintext (stored blob is not plaintext).
- NFR11: All HTTP surfaces (UI, JSON API, `/metrics`) require authentication.
- NFR12: Secret round-trip (rotate→backup→restore→decrypt) verified end-to-end, majority failure paths; non-regenerable oracle; interrupted-rotation crash-kill assertion; verified on the backend on every PR.
- NFR13: TLS in transit is a documented deployment responsibility (reverse proxy), not provided by the app.

**Data Integrity & Durability**
- NFR14: Full dataset backup/restore round-trips with equal SHA-256 and row counts.
- NFR15: Invariant suite runs on MariaDB 10.11.11 on every PR; engine never decides a comparison (app-code comparison/normalization, app-generated identifiers, time bound as a parameter); enforced by binary collation on every text column + a CI DDL grep.
- ~~NFR16: SQLite WAL mode / concurrent writers — STRUCK by D64 (number retained).~~

**Upgrade, Migration & Footprint**
- NFR17: Schema migrations versioned, idempotent, resumable after interruption; auto-backup before migration; verified on populated MariaDB with zero-loss invariant; documented rollback via backup.
- NFR18: Resident memory ≤ ~512 MB at rest; cold start < 5 s; binary and image size bounded and tracked in CI.
- NFR19: An update incurs bounded downtime (target < 30 s); app resumes cleanly with no data loss.

**Compatibility & Portability**
- NFR20: Runs as a single binary and a Docker container (Synology Container Manager x86 priority; ARM best-effort native); requires a MariaDB alongside; try-it path never described as a single `docker run`.
- NFR21: MariaDB 10.11+ is the only supported engine; SQLite and MySQL not supported; PostgreSQL not at MVP (Repository trait audited before any port).
- NFR22: UI supports current evergreen browsers (Chrome, Firefox, Safari, Edge).
- NFR23: UniFi connector supports a stated minimum version matrix (defined in architecture) and is tested against it.

**Usability & Accessibility**
- NFR24: UI responsive (breakpoints 360/768/1280 px; no horizontal overflow; touch targets ≥ 44 px), snapshot-verified; deep-linked object views usable on a phone.
- NFR25: WCAG 2.1 AA on key views; axe-core 0 violations per theme (blocking floor); scripted keyboard checklist + per-release screen-reader pass are also blocking gates.
- NFR26: UI available in English and French; all user-facing strings externalized.

**Operability & Maintainability**
- NFR27: 12-factor configuration (file + env vars); no external services (cron, Redis, workers) required.
- NFR28: Install on a clean environment ≤ 30 min wall-clock (measured on a clean Ubuntu 22.04 VM; Synology validated separately).
- NFR29: Self-diagnostic dashboard + authenticated `/metrics` give operator visibility on their own instance; no operational data leaves the deployment.

**Scalability (bounded)**
- NFR30: Designed for a single operator and ~300 hosts / 36 subnets reference target (not enterprise-scale); seeded generator must produce an interface-per-device distribution at that scale.

### Additional Requirements

_From the Architecture (`architecture.md`). Several items marked DONE already exist in the committed workspace but are retained because they carry acceptance criteria constraining later stories._

**Workspace & dependency frontier**
- ARCH-1: No starter template — `cargo new` + curated deps; the integration proof a starter would give is bought back by the Story 1 walking skeleton. (DONE: workspace compiles `--locked`.)
- ARCH-2: Three-crate Cargo workspace, edition 2024, resolver 3, Rust 1.96+: `opencmdb-core` (domain), `opencmdb-bin` (composition root + outside world), `xtask` (member, dependency of nobody); `Cargo.lock` committed; all builds `--locked`.
- ARCH-3: Dependency-frontier rule (D47) as a CI-enforced invariant — `core` graph must not contain `anyhow`/`axum`/`sqlx`/`askama` (verified via `cargo tree`, not TOML grep); `xtask` absent from `bin` normal deps; one `thiserror` enum per subdomain.
- ARCH-4: `core` organized by subdomain (D54), `ports/` the named exception; orphan rule (D53) — domain errors in `core` with `http_status()`, `impl IntoResponse` newtype in `bin` (a core-side impl must not compile).

**Infrastructure & deployment**
- ARCH-5: MariaDB 10.11+ only (D64); no second backend, no dialect abstraction, no `sqlx::Any`/`AnyPool`; "remove now, re-add later" banned in writing; PostgreSQL port requires a trait audit first.
- ARCH-6: Two-service Docker Compose (opencmdb + operator's MariaDB), never a single `docker run`; README states "One binary + your MariaDB. No Redis, no workers, no queue, no proxy."; the "Synology in under 30 minutes" claim must be measured or omitted.
- ARCH-7: Docker base `gcr.io/distroless/static:nonroot` from a static `x86_64-unknown-linux-musl` binary; bundles CA certs, tzdata, nonroot user; no image scanner (theatre on static distroless).
- ARCH-8: MariaDB in CI as a GitHub Actions service container on every PR; Renovate grouped auto-merge on green (shipped with CI); pinned MSRV + `rust-toolchain.toml`; `cargo-deny` for advisories + licenses.

**Project gates (`cargo xtask ci`)**
- ARCH-9: All CI gates live in `cargo xtask ci` as Rust, never YAML (D56); every gate proven-to-RED, not merely passing. (DONE: 6/6 xtask tests, proven to red.)
- ARCH-10: Gate `ddl-collation` (D64 cond. 1) — reds if any text column in `migrations/**/*.sql` lacks explicit binary collation; no allowlist; blind to derived expressions (F57); green vacuously until the first migration.
- ARCH-11: Gate `vocabulary` (D65/F59) — Volet A: zero retired code identifiers (`pending_accept`, `reverting`, `accept-as-declared`) in `crates/`; Volet B: co-presence check over the seven planning docs (a doc with a retired term but not its live replacement reds).
- ARCH-12: Gate `views-hash` — compares `architecture.md` sha256 vs `architecture-views.md` declared `sourceSha256`; informational, not a hard gate.
- ARCH-13: Security gates in CI (D26) — NFR12 suite, backup byte-grep for plaintext secrets, AC-9b refuse-to-start, `cargo-deny`, clippy `-D warnings`, and an authorization-matrix test (session vs bearer vs scrape token across every surface incl. crossed cases).

**Data model & migrations**
- ARCH-14: Migrations via `sqlx::migrate!` wrapped in own `backup → migrate → verify` (D23); one dialect / one folder `crates/opencmdb-bin/migrations/`; confirm sqlx 0.9 `Migrate` surface at `cargo add`.
- ARCH-15: Opaque identifiers — `Id`(UUIDv7) `CHAR(36) ascii_bin`, `Hash64` `CHAR(64) ascii_bin`; UUIDv7 clock injected from Rust; `EntityId::as_db()` the only bind path; case-sensitivity invariant test.
- ARCH-16: Multi-user-ready from day 1 — DB-backed sessions (D30, `deadline_at` + sweep), cookie carries raw 256-bit token, DB stores `SHA-256(token)`; tables singular snake_case; FK `<entity>_id`; ISO-8601 UTC TEXT timestamps.
- ARCH-17: `declared_attribute.entity_id` is NEVER updated (D15); identity migration writes an `identity_migration` record, target entity "born naked"; anchors AC-M-04 (splitting a device hosting software must not silently shrink an answer).
- ARCH-18: TEXT columns need a length prefix to index on MariaDB — resolved via `CHAR(n) ascii_bin`; three bounded idempotent sweeps on the writer actor (`pending_commit`, sessions, dormant interfaces), each with injectable `now()` bound from Rust.

**Integration / external systems**
- ARCH-19: The `Connector` trait IS the fixture (D19) — async `poll(now, sink, cancel)`, incremental emission, cooperative cancellation; `capabilities()` dynamic and travels with the batch; scope mandatory (scanner = one per subnet, UniFi = one `controller`); engine never touches the clock.
- ARCH-20: `ConnectorError` is a closed taxonomy (D33), never `anyhow` — one variant per (source_state, operator-action); every variant carries `scope`; `is_blinding()` default-safe via exhaustive match (NFR7 compiler-enforced).
- ARCH-21: `source_state` = two orthogonal axes (D32) — liveness (Live/Blind) and capabilities (Full/Reduced); `full/degraded/offline` survives only as a UI projection.
- ARCH-22: Labeled fixture / adversarial trap suite (NFR4/D18) — ~50 seeded synthetic scenarios × positive/negative, three columns, binary zero-tolerance at device level; each trap asserts the RULE; JSONL committed; `FixtureConnector` replays JSONL (zero mocks/network); fixtures at repo-root `fixtures/`.
- ARCH-23: Real captures never gate — only a distributional-diff of generator representativeness; `cargo xtask recapture` diffs real UniFi schema vs `capture/` via a module constant; Story-1 probe/record reader is throwaway, run once locally, never in the engine/CI/repo.
- ARCH-24: ATDD/red-first build order (D19-rev) — probe+record → types → ~35 semantic traps → ~15 wire-format traps → `FixtureConnector` → metrics harness → L1 join → L2 cascade one trap at a time → seeded generator → bulk fixture → distributional diff → real connectors last.

**Security implementation**
- ARCH-25: Envelope encryption (D28) — out-of-volume KEK encrypts a wrapped DEK stored in DB; field-level credential encryption under the DEK; always decrypt via `secret.dek_id`; AAD binds ciphertext to context; DEK zeroized on Drop; orphan-DEK detected at startup; KEK rotation at MVP (DEK rotation deferred).
- ARCH-26: Key-path startup checks (D26/D27, NFR9b) — refuse to start if key path resolves inside the data volume (post-symlink) or key file is group/other-readable; KEK via a separate DSM shared folder, auto-generated `0600` at first boot, path logged loudly; NFR9c documented as a non-guarantee.
- ARCH-27: Tokens = SHA-256 constant-time no salt (D29); passwords = Argon2id `m=19MiB,t=2,p=1`, < 300 ms on target Celeron J4125; crypto crates pinned (`chacha20poly1305` 0.11, `zeroize` 1.9, `argon2` 0.5.3); crypto crate choice (D31, `age` vs pure RustCrypto) owed before code.

**Stack pins (D66) & sqlx gotchas**
- ARCH-28: Exact pins from `Cargo.lock` (supersede any recalled version) — axum 0.8.9, askama 0.16.0, sqlx `=0.9.0` (`runtime-tokio,tls-rustls-ring,mysql,migrate,macros`), tokio 1.53.0, config 0.15.25, rust-i18n 4.2.1, prometheus 0.14.0, rust-embed 8.12.0, uuid 1.24.0, serde 1.0.228, chrono 0.4.45, thiserror 2.0.18, anyhow 1.0.103 (bin+xtask only); Tailwind v4 standalone CLI via `cargo xtask css`.
- ARCH-29: sqlx 0.9 gotchas — all `query*()` take `impl SqlSafeStr` (dynamic SQL needs `AssertSqlSafe`); write own ~15-line Askama→Axum `IntoResponse` (`askama_web`/`askama_axum` refused); `tls-rustls-ring` never `native-tls`; any Rust-built CSS class needs `@source inline(...)` (a silent bug the drift-check cannot catch, AC-1.12).

**Walking skeleton / Story 1**
- ARCH-30: Story 1 = walking skeleton that DISPLAYS A REAL GAP on a perimeter where L2 cardinality is 1 (one connector, one line, green on MariaDB); proof-of-integration included; abstains + shows an "I don't know" count everywhere cardinality-1 is not established.
- ARCH-31: Story 1 lands the Repository skeleton that COMPILES before any identity logic — `WriteRepository::transact` (HRTB over GAT), `WriteUnit` with no `commit()`, `ReadRepository` a distinct type; `IdentityIndex::for_unit` the only constructor.
- ARCH-32: THE STORY-1 BOMB — `Reads` cannot be a single trait: `ReadRepository` is `&self`, `Unit<'u>` is `&mut self`, and `core` cannot name `sqlx::Executor` (D55); it must be TWO traits delegating to a generic free function in `bin`.
- ARCH-33: CI gate — `grep -r "sqlx::" crates/` (minus the adapter zone) must be empty (sqlx confined to the adapter); dispatch by monomorphization (one `match cfg.db` at the composition root, everything below generic).

**Cross-cutting technical**
- ARCH-34: Observability — `/metrics` is a raw `prometheus::Registry` + a hand-written axum handler behind the scrape-token authorization matrix; `axum-prometheus` rejected.
- ARCH-35: i18n — `rust-i18n` YAML locale files (greppable/diffable so vocabulary gates run over them), EN/FR; glossary uniqueness + forbidden-word denylist run over translation files.
- ARCH-36: Frontend — HTMX polling (not SSE), `idiomorph` morph swaps, server-rendered Askama + Tailwind, optimistic UI; all JS deps pinned/committed/`rust-embed`'d, never a CDN (D37, CI-checked); focus management in a committed testable `app.js` (D38), a blocking a11y gate; visuals SVG + CSS, never canvas.
- ARCH-37: Config — the `config` crate (12-factor); three boot-time cross-invariants as startup FAILURES naming keys (dormancy < retention D17; key-path-not-in-data-volume D27; MariaDB ≥ 10.11 floor D52); `tracing`; tokio scheduler with poll coalescing as `PollSlot ∈ {Idle, Running, RunningWithPendingRerun}`.
- ARCH-38: No caching anywhere (D25) except the writer actor's per-batch identity index; NO impact graph at MVP (D57 scope) — FR26/FR27 ship, FR28 splits, FR29 = "Hosted here" one-hop join; the four verbs are not one relation (F55: `depends_on` DAG-traversed, `hosts` lookup-only, `connects_to` declarable never traversed) — all Growth.

### UX Design Requirements

_From the UX Design Specification (`ux-design-specification.md`). Condensed; full detail in the spec._

**Design tokens & visual system**
- UX-DR1: Design tokens as single source of truth in Tailwind theme + CSS vars — `--radius: 3px` everywhere, no shadows (elevation via hairline border), one token source drives both themes via `[data-theme]`.
- UX-DR2: Tinted-neutral cold-indigo base palette (never pure gray) — dark bg `#0f1420`/surface `#161c2b`/border `#2a3346`/text `#e2e8f0`; light bg `#f6f7f9`/surface `#fff`/border `#dfe3ec`/text `#1a2233`; text never pure white.
- UX-DR3: Single locked accent `--accent-document: #d99a4e` (desaturated amber), reserved SOLELY for the document action (« Merger »); never decorative, never elsewhere.
- UX-DR4: `accept-gap` styled deliberately NEUTRAL — never amber/accent, visibly lower-emphasis than document.
- UX-DR5: Token rename across codebase — `--accent-accept`→`--accent-document`, `.btn-accept`→`.btn-document`.
- UX-DR6: Severity encoded by luminosity + font-weight only, never hue; no red anywhere; rare destructive confirm uses restrained warm `#b5654a`, never document amber.
- UX-DR7: Observed = muted/grayed (factual, non-editable), declared = crisp/full-contrast; positive = calm base + check glyph, never a green flood.
- UX-DR8: Source-tag color tokens — low-saturation tinted chips, one hue per source (UniFi indigo, scan teal, manual violet).
- UX-DR9: Spacing scale — 8 px base grid with 4 px half-steps for tight card internals.
- UX-DR10: Typography — bundled variable sans (Inter, `system-ui` fallback) + a monospace for IPs/MACs/ports/hostnames; scale 12/14/16/20/24; hierarchy by weight; tabular figures in tables/grid/metrics; fonts embedded via `rust-embed`.
- UX-DR11: Dark mode first-class from MVP and the DEFAULT; one token source drives both; both pass contrast; Playwright + axe per theme.
- UX-DR12: Iconography — lightweight open SVG set (Lucide/Heroicons), inlined, monochrome via `currentColor`.

**Reusable components**
- UX-DR13: Triage Card — object header + status chip + gap-diff (observed→declared) + one signature evidence chip + action row (amber Document + ghost Accept-gap/Snooze/Exclude/Attach) + kbd hints; preview is the card (no post-tap modal); Document a real `<button>`; evidence via `aria-expanded`.
- UX-DR14: Triage Card states — `new`/`conflict`/`ambiguous` (Resolve badge replaces Document)/`pending_commit`/`failed` (re-inserted at top)/`snoozed`/`gap_accepted`; undo returns to `in_queue` (not a state); variants desktop list-row / mobile full-card / bulk compact multi-select.
- UX-DR15: Gap Diff component (reused in cards, object view, alerts) — two columns Observed (grayed) → Declared (crisp), each source-tagged + timestamped; states match/divergence/declared-empty/observed-empty; never color-only.
- UX-DR16: Source-Tagged Evidence Chip — provenance + freshness ("seen 2 min ago"); signature (one on the card) vs expanded list; only signature shows by default.
- UX-DR17: Occupancy Grid — CSS Grid, one cell = one address, GitHub-contributions density; used/free/reserved states; no green/red, no legend; fill rate readable in 3 s.
- UX-DR18: Stat Card + Sparkline — tiny label, big tabular number, thin server-rendered SVG polyline with emphasized endpoint, trend caption; drills to a SECONDARY view off the 10 s path.
- UX-DR19: Source-Health Banner — neutral cool/desaturated (not scarlet/amber) + "frozen" + elapsed time + affected source; observation-derived alerts grayed with "unverifiable — source offline".
- UX-DR20: Undo Toast + undo history — summary + Undo link (5 s window) + history for rapid keyboard runs; `role="status"`, focus-reachable Undo.
- UX-DR21: Resolve Panel (candidates + evidence + confidence, never a blind document) and Compare View (two objects side by side, mobile-capable).
- UX-DR22: Foundation primitives as Askama partials on Tailwind tokens (no third-party lib) — button (+ ghost / warm-destructive), text field, select, table, chip/badge, toast, tabs/toggle, left-nav, skeleton, focus ring, kbd hint; all keyboard-operable.
- UX-DR23: Inbox Queue — single-column focused queue (Direction A), keyboard-navigable, auto-advancing, slim queue rail on desktop; priority-first ordering.
- UX-DR24: Bulk Mode (Direction C) as a toggle within the same inbox — compact table with multi-select + inline actions, same card/token vocabulary; Direction B (two-pane) rejected.
- UX-DR25: Data Table — sort, multi-select, filter; used for bulk mode and the motif/grouped presentation.

**Key views / screens**
- UX-DR26: First-Run Setup Wizard (resumable) — choose source (UniFi URL+key or declare subnets) → Test Connection → first cycle → inbox pre-filled → bulk baseline → name ~10 key devices → first value; empty-scan branch routes to a diagnostic, never a dead-end.
- UX-DR27: Test-Connection interaction — bounded 5 s `reqwest` call, no retry, trigger disabled via `hx-disabled-elt`; validate reachability + auth + read permission; credentials in memory, persisted only after HTTP 200; typed error hints; success echo ("controller reachable, 84 clients").
- UX-DR28: Per-subnet IP Occupancy + free-IP lookup view (Occupancy Grid) with accessible "jump to next free IP" and a synthetic summary.
- UX-DR29: Device Record view with "Hosted here" panel — one containment hop (FR39): declared attributes, observation history, connection point, hosted applications; traversal-based "Impact" view explicitly OUT.
- UX-DR30: Deep-Linked Focused Object View (mobile conflict) — lands precisely on the object, resolvable in place, phone-usable, auto-updates via polling; states Live/Resolved-elsewhere/Deleted (tombstone, never "not found"); auth-expired resumes with decision preserved.
- UX-DR31: Self-Diagnostic Dashboard led by "what changed since last visit" (calm, no guilt) + source health; Grafana-style stat cards and drill-down on a SECONDARY screen.
- UX-DR32: Login/Setup screens as key (WCAG) views; auth-expired flow preserves the in-progress decision and resumes.
- UX-DR33: Shallow left-nav (Inbox · Dashboard · Devices · IPAM · Applications · Topology); Inbox "N to triage" count NOT styled as an accusatory badge; interactive graphical topology is Growth.

**Source-state & degradation UI**
- UX-DR34: Frozen-Banner / source-blind view (Flow 4) — after > N cycles unreachable set `blind`: frozen banner ("state frozen, X min, nothing is lost"), suppress observation-derived alerts, keep last-known state, don't touch in-flight `pending_commit`; on return one reconciliation re-scan, banner clears.
- UX-DR35: Two-axis Source-State UI — liveness = a COLOR (live calm; blind neutral cool, never scarlet); capability = a SCOPE LABEL beside the name (`ping-only`), neutral, never a color/severity; `Live + Reduced` never painted as degradation.
- UX-DR36: Reduced-capability (`ping-only`) screen content = the LIST of what the source cannot see, framed as a capability to UNLOCK, never a fault; out-of-capability fields render `not evaluated` (never blank/dash/N-A), excluded from divergence count.

**Interaction patterns**
- UX-DR37: Optimistic UI on document/accept-gap/exclude — client instantly applies class, retracts card, mounts next, decrements counter while `hx-post` runs; each action an idempotent `action_id`; counter server-authoritative; server failure re-inserts card at top with error toast.
- UX-DR38: Commit state machine — `in_queue → pending_commit (server deadline t+5s) → committed | failed`; server timer authoritative (never browser `setTimeout`); one transition per item serialized by `item_id`; idempotency via version/ETag (duplicate → 409/Gone); state gesture-agnostic (`pending_commit`).
- UX-DR39: Pending-commit vs concurrent-scan quarantine — a scan touching a `pending_commit` item quarantines the delta (`superseded_by_pending`), reconciled at commit; user's decision outranks a concurrent observation.
- UX-DR40: Session-integrity queue freeze — queue frozen during an active session; new scans stack behind an "N new — refresh" pill; a snoozed item re-fetches current state on return and is flagged if it changed.
- UX-DR41: Six keyboard triage gestures — Document/Next/Snooze/Exclude/Accept-gap/Attach, mouse-free, queue auto-advances; letters deliberately unassigned in the spec, chosen all at once when the inbox is built, decoupled from vocabulary so a rename can't orphan a key.
- UX-DR42: Gesture semantics — Document (amber, closes gap); Accept-gap (neutral, gap stays open + counting, note MANDATORY, returns only on observed change); Snooze (a timer); Exclude (remembered, out of scope); Attach/Create (link/new record).
- UX-DR43: Ambiguity handling — ambiguous matches pre-computed at render; such a card shows a Resolve badge instead of Document from the start; ambiguity always routes to Resolve, never a blind document.
- UX-DR44: Optimistic-swap FOCUS MANAGEMENT (requirement #1) — on `htmx:afterSwap` move focus to the next card (`tabindex="-1"` + `focus()`); empty queue focuses the section heading; never rely on morph to preserve focus for a removed node; focus visible and NEVER lost after a swap.
- UX-DR45: Live-region choreography — two live regions outside swapped fragments: counter `aria-live="polite" aria-atomic="true"` (debounced), undo toast `role="status"` (assertive only for errors); "next card" announced by `focus()`, not a live region.
- UX-DR46: Auto-update polling (NOT SSE) — editable card lives outside the polled fragment; swaps use `hx-swap="morph"` (idiomorph) with `hx-sync` + `queue:none` to avoid clobbering an in-progress interaction.
- UX-DR47: Regime-dependent question granularity — steady state = the FIELD (one line, two values, two buttons); bootstrap/migration = the MOTIF (sortable/multi-select column); bulk = the question is COUNTED not displayed; same recorded decisions, ~100× fewer gestures; never withhold a question during migration.
- UX-DR48: Bootstrap as a MODE gated by VOLUME (never a `first_run` flag) — available for life, migrations re-trigger it; grouped/bulk switch auto-detected by volume, reversible in one click, carries no judgement; confidence threshold (strangeness) kept separate from the volume switch.
- UX-DR49: Baselining ("adopt current state as baseline") — explicit bulk gesture distinct from per-item, idempotent + resumable; a confidence threshold routes anomalies to single triage so bulk absorbs only the obviously-expected; never a blanket "document everything".

**Dignity / backlog constraints**
- UX-DR50: "What the Product Does Not Know" abstention MAP (FR16b) — "N evaluated · N not evaluated" with actionable causes ("96 multi-interface — grouping unresolved → [Resolve this pattern]"); a MOTIF (one question per pattern); counter measures REACH not debt (never reddens/bolds/ages).
- UX-DR51: Recovery copy with the tool-as-subject rule — empty scan: "I couldn't reach anything — let's find where it's stuck" + three clickable testable leads; the tool's "I" only for a failed attempted action, never for a state.
- UX-DR52: Loading states — skeletons over spinners; bounded calls disable their trigger; success = calm check + concise past-participle echo (`role="status"`, no green flood); error = cause + next step, never blaming the user.
- UX-DR53: Offline/connectivity pattern — offline is a VISIBLE state (banner), never silence; optimistic actions queued (not falsely confirmed) and replayed on reconnect; heartbeat detection (not `navigator.onLine`); bounded offline queue; neutralize pull-to-refresh.
- UX-DR57: Backlog Bans as testable rules — no numeric/growing badge; no health gauge/score/percentage/dial; age sortable but invisible by default; no degradation (t+1d and t+6mo render identical); no nag (notification only on a new fact); no gamification; constant insistence (3 items and 3000 → same tone).
- UX-DR58: Replacement backlog surface — never pushed, available at a stable place in factual language ("Pending · 23 fields, 14 devices · Oldest: 2 March · [Sort by age] [Group by motif]"); a fact, never a judgement.
- UX-DR59: Abstention/backlog counter measures REACH not debt; the six-month test applies to every screen (if inaction makes the product more unpleasant → violation).
- UX-DR60: (Growth, recorded) The Narrator — reports FACTS about the network, never elapsed time about the operator; any sentence whose subject is the operator is a banned nag.

**Responsive & accessibility**
- UX-DR54: Mobile responsive layout — left-nav → bottom bar with a permanent search magnifier; inbox becomes one focused card; swipe to defer/dismiss WITH a visible button equivalent for every gesture; undo toast above the mobile nav; swipe-direction mapping a V1 hypothesis.
- UX-DR55: Breakpoint strategy — mobile-first, three breakpoints ≤360/768/1280; relative units throughout; wide content scrolls in its own container; no horizontal overflow.
- UX-DR56: Touch targets ≥ 44 px (NFR24); honor `prefers-reduced-motion`; deep-linked object views fully phone-usable, resolvable one-handed.
- UX-DR69: Occupancy Grid non-mute for a11y — `role="grid"` + `role="gridcell"` with short labels (address + state), a synthetic summary, an accessible "jump to next free IP"; per-cell hover tooltip.
- UX-DR70: Never meaning by color alone — severity by position + weight + icon; observed/declared by label + weight; semantic HTML before ARIA; skip links; no keyboard traps; visible focus outlines.
- UX-DR71: Name WCAG 2.1 AA "key views" explicitly (inbox, occupancy grid, deep-linked object, login/setup, dashboard) with the no-loophole rule that any critical-path view is key; axe-core a floor not a ceiling.

**i18n / microcopy**
- UX-DR61: Canonical bilingual glossary — binding EN(docs/API/code)/FR(UI) pairs, one meaning per term (observed/observé, declared/déclaré, gap/écart, reconcile/réconcilier, document/« Merger », accept-gap/« Accepter l'écart », snooze/« mettre en veille », attach/« rattacher », exclude/« exclure », triage, source); all strings externalized (rust-i18n YAML).
- UX-DR62: Retired-vocabulary denylist (never reintroduce) — `accept-as-declared`, `merge`/`fusionner` in EN, `revert`, `ignore`/« ignorer »; `exclude`/« exclure » replaces `ignore` (same word as an out-of-capability field); microcopy past-participle renames.
- UX-DR63: Microcopy rules as conventions + gates — button = action verb, feedback = same verb as past participle; error = cause + next step never blame; one term = one translation; empty ≠ failure; the tool's "I" only for a failed attempted action.

**Build & CI (UX-owned gates)**
- UX-DR64: CI consistency gates — exactly one active `.btn-document` per view (Playwright); forbidden-word lint over templates + i18n; glossary uniqueness + retired-words denylist; ghost/link class lint; offline-banner test; axe-core contrast per theme.
- UX-DR65: Tailwind v4 build chain — standalone CLI (pinned, read by CI + dev), generated CSS committed, assets `rust-embed`'d in release / disk in dev; CSS generation a pre-build step never in `cargo build`; CSS-first config with `@source` incl. `@source inline(...)` for HTMX/state classes; no dynamically concatenated classes.
- UX-DR66: v4-trap gate — every state-enum variant that renders a class in Rust must have that class in the generated CSS (the `git diff --exit-code` check does NOT catch a Rust-built class missing from `@source inline()`); all visuals SVG + CSS Grid, never canvas.
- UX-DR67: Every custom component ships a Playwright visual snapshot PER THEME + an `@axe-core/playwright` check (blocking on key views) + an explicit keyboard test; accessibility is a build gate, not a later audit.
- UX-DR68: Manual accessibility gates (blocking) — scripted keyboard checklist on any PR touching inbox/grid; per-release screen-reader pass (NVDA+Firefox, VoiceOver+Safari) with recorded proof; plus real-device mobile, color-blindness sim, low-end-monitor contrast.

**Surface hierarchy & search**
- UX-DR72: Surface hierarchy visible — Primary = the reconciliation loop (inbox + documenting, ≤ 1 tap from "see the gap" to "documented"); Supporting = dashboard + source-health; Task-specific = IP occupancy/free-IP, device-record "Hosted here", deep-linked object; modals rare (destructive confirm + Compare only), never a post-tap confirmation modal.
- UX-DR73: Global search — IP/MAC/hostname/device → full record; keyboard `/` on desktop, permanent bottom-bar magnifier on mobile.

### FR Coverage Map

_Every active FR (FR52 removed) maps to at least one epic. Multi-epic FRs are split minimal→full._

- FR1: E11 — connect a UniFi controller
- FR2: E11 — discover from UniFi without elevated privileges
- FR3: E3 (minimal) / E12 (full) — declare subnets to scan
- FR4: E3 (minimal) / E12 (full) — discover active devices + hostname enrichment
- FR5: E3 (minimal) / E11 / E12 / E13 — two-axis source state
- FR6: E11 / E12 — per-source cadence + on-demand scan
- FR7: E11 — dated capability descriptor
- FR8: E12 / E13 — outage vs genuine disappearance
- FR9: E5 (interface identity) / E6 (device grouping)
- FR10: E3 — reconcile observed vs declared by identity
- FR11: E3 — distinct linked records; never overwrite
- FR12: E7 — triage inbox
- FR13: E6 (minimal promote) / E7 — document (all/field)
- FR14: E7 — create/attach/exclude/snooze/accept-gap
- FR15: E7 — remember triage decisions
- FR16: E3 (minimal) / E5 — abstention as first-class outcome
- FR16b: E5 — abstention displayed/counted/grouped by cause
- FR17: E9 (bootstrap) / E18 (bulk steady-state)
- FR18: E7 (minimal) / E17 — what changed since last visit
- FR19: E13 — suppress alerts for a blind (source, scope)
- FR20: E13 — surface source conflicts, never merge
- FR21: E14 — manage subnets/VLANs/DHCP ranges
- FR22: E14 — per-subnet occupancy
- FR23: E14 — find a free IP
- FR24: E14 — detect IP conflicts
- FR25: E14 — IPv6 documentation (observation-only)
- FR26: E15 — record software instances
- FR27: E15 — group software into applications
- FR28: E15 — declare hosts/exposes (MVP)
- FR29: E15 — "Hosted here" one-hop panel
- FR30: E16 — raise alerts (unknown device, stale IP, IP conflict)
- FR31: E16 — in-app + generic webhook
- FR32: E16 — stable deep links
- FR33: E16 — act on an alert from the object
- FR34: E16 — configure thresholds; mute/snooze
- FR35: E16 — channel routing
- FR36: E17 — self-diagnostic dashboard (**partial MVP**: source health + what-changed; rich trend/lag/queue analytics → Growth)
- FR37: E6 (minimal) / E17 — observation history
- FR38: E17 — observation retention
- FR38b: E6 — ephemeral-interface dormant lifecycle
- FR39: E3 (minimal) / E17 — search + full record view
- FR40: E21 — edit declared attributes
- FR41: E21 — decommission/archive/delete
- FR42: E21 — full dataset backup/restore
- FR43: E20 — read-only JSON API
- FR44: E20 — authenticated Prometheus /metrics
- FR45: E22 — first-run setup wizard
- FR46: E19 — local login and session
- FR47: E10 (minimal) / E19 — encrypted credential storage
- FR48: E19 — API-key rotation + secret backup/restore
- FR49: E19 — security event log
- FR50: E22 — EN/FR interface
- FR51: E16 — external base URL for deep links
- FR53: E17 — topology as structured list/table

## Epic List

_23 epics, riskiest-first, each a releasable increment. E1–E2 are foundation (no release); v0.1 begins at E3. a11y keyboard+focus is a per-epic Definition-of-Done gate, not an epic. Auth/i18n/metrics/design-tokens/Clock anchors are born empty in E3 and filled by their later epics (E19/E22/E20) — anchoring early avoids cross-cutting retrofit debt._

### Epic 1: Les gates tiennent (foundation)
Complete `cargo xtask ci` so every project gate is green and proven-to-red: the DDL binary-collation grep (D64 cond. 1), the retired-vocabulary check (D65), the fixture MANIFEST sha256, the architecture-views.md staleness hash, and the D47 dependency-frontier assertion (`cargo tree`). No user-facing value; hard prerequisite for every later epic.
**FRs covered:** none (enables all). ARCH-9,10,11,12,3.

### Epic 2: Le contrat de connecteur (foundation)
Define the generalized, source-agnostic `Connector` trait (async `poll` + `ObservationSink`, dynamic `capabilities()`, mandatory `Scope`) and the closed `ConnectorError` taxonomy, plus the consumer-driven contract test (empty stream, partial error, missing field, timeout, cancellation) that every connector — fixture, ARP, UniFi, future — must satisfy. No throwaway readers; every source implements this one contract.
**FRs covered:** none directly. ARCH-19,20,21.

### Epic 3: Mon premier écart réel (v0.1)
The walking skeleton: the whole stack holds (Askama + HTMX + Tailwind + sqlx + MariaDB), a real minimal ARP/ping source (implementing the E2 trait) is ingested, and one page displays a real gap on a cardinality-1 perimeter while abstaining + counting everywhere else. Lands the compiling Repository skeleton (two traits, HRTB over GAT), the first migration, and the empty transversal anchors (auth-deny middleware, `/metrics`, i18n `t!()`, design tokens + focus `app.js`, the `Clock` port routed by the reader, `/healthz`).
**FRs covered:** FR10, FR11, FR16 (min), FR39 (min), FR3/FR4 (min). ARCH-2,30,31,32,33,37; UX-DR1,2,3,11,65,66.

### Epic 4: Infra fixtures & corpus de pièges (v0.2)
Freeze the JSONL fixture schema, build the `FixtureConnector` (replay, zero mocks/network, same trait + passes the contract test), the test metrics harness, and the ~50 adversarial traps (~35 semantic + ~15 wire), one story per trap family, each asserting the RULE. Open the "reality-debt" register for traps the real connectors will later add.
**FRs covered:** none (realizes NFR4 infrastructure). ARCH-22,24.

### Epic 5: Identité d'interface fiable (v0.3)
The L1 interface-identity join, built one trap at a time until the NFR4 binary gate is green (truth-table failures = 0, three columns must-not-merge/must-merge/must-abstain). Abstention is a first-class persisted outcome, displayed and counted by cause. Includes the monotone-honesty invariant trap family.
**FRs covered:** FR9 (interface level), FR16, FR16b. NFR4,5,6.

### Epic 6: Ne pas compter deux fois la même boîte (v0.4)
Device grouping (L2): the cascade one rule per story, the seeded generator, the bulk fixture, the distributional diff, and the ephemeral-interface dormant lifecycle. Closes with the minimal actionable gesture — a one-click promote of an observed value into a declared record, so the loop becomes actionable one release before the rich inbox.
**FRs covered:** FR9 (device level), FR13 (minimal promote), FR37 (min), FR38b. NFR4,30.

### Epic 7: La boucle se ferme (v0.5)
The rich triage inbox on a correct synchronous commit: review unreconciled discoveries; document (all/field); accept-gap (gap stays open, mandatory note, wakes on observed change); exclude; snooze; create; attach. This is the MVP "you could stop here" line — the product now reconciles, not just observes.
**FRs covered:** FR12, FR13, FR14, FR15, FR18 (min). UX-DR13,14,15,16,22,23,42,43.

### Epic 8: Triage fluide (v0.6)
The optimistic UI over the inbox: the server-authoritative commit state machine (`in_queue → pending_commit → committed | failed`), scan-vs-triage quarantine (`superseded_by_pending`), undo, and the focus-management contract on every HTMX swap (accessibility requirement #1 — focus never lost).
**FRs covered:** none new (UX layer over E7). UX-DR37,38,39,40,44,45,46.

### Epic 9: Bootstrap jour-1 (v0.7)
Baseline a whole network at once on day one: the volume-gated bootstrap mode, baselining ("adopt the current state as baseline"), and regime-dependent motif granularity (twelve identical divergences = one decision), with a confidence threshold routing anomalies to single triage.
**FRs covered:** FR17 (bootstrap). UX-DR47,48,49,25.

### Epic 10: Secrets au repos (minimal) (v0.8)
Minimal encrypted credential storage (the envelope skeleton) with a minimal NFR12 round-trip, so the UniFi connector has a safe place for its API key before the full vault epic exists.
**FRs covered:** FR47 (minimal). ARCH-25 (subset).

### Epic 11: Source UniFi complète (v0.9)
The real UniFi connector implementing the trait: the dated capability descriptor, the bounded test-connection interaction, the tested version matrix, and record mode — plus the story that freezes N raw captures into version-drift traps.
**FRs covered:** FR1, FR2, FR5, FR6, FR7. NFR7b,8,23; UX-DR27.

### Epic 12: Scan ARP/ping complet (v0.10)
Extend the E3 connector into the full generic scanner: hostname enrichment, the NET_RAW → ping-only fallback, capability reduction as a notifiable event, on-demand scan — plus its own drift-capture story. Additive extension, no rewrite.
**FRs covered:** FR3, FR4, FR5, FR8. UX-DR35,36.

### Epic 13: Ma source devient aveugle (v0.11)
Journey 4: the two-axis liveness/capability model made real — the frozen banner, suppression of observation-derived alerts for a blind (source, scope), zero fabricated "device-gone" events (fault injection validating NFR7/NFR8), and conflict surfacing only between two capable sources.
**FRs covered:** FR8, FR19, FR20. NFR7,7b,8; UX-DR19,34,35,36.

### Epic 14: IPAM (v0.12)
Manage subnets, VLANs, and DHCP ranges; view per-subnet occupancy; find a free IP; detect IP conflicts and identify the devices; document IPv6 (observation-only).
**FRs covered:** FR21, FR22, FR23, FR24, FR25. UX-DR17,28,69.

### Epic 15: Applications & « Hosted here » (v0.13)
Record software instances (name, version, ports); group them into applications with owner and criticality; declare hosts/exposes; and the device-record "Hosted here" one-hop panel (never called "Impact").
**FRs covered:** FR26, FR27, FR28, FR29. ARCH-17,38; UX-DR29.

### Epic 16: Alertes & notifications (v0.14)
Full MVP alerting: rules for an unknown device appearing, a documented IP unseen for N days, and an IP conflict; delivery in-app and via a generic outbound webhook; stable deep links; act-from-object; threshold configuration; mute/snooze; and channel routing.
**FRs covered:** FR30, FR31, FR32, FR33, FR34, FR35, FR51. UX-DR30,33.

### Epic 17: Retour après une absence (v0.15)
Journeys 4 & 5: the slim self-diagnostic surface led by "what changed since last visit" plus source health, observation history and retention, global search + full record view, and the topology list. (Rich stat-card/sparkline/trend analytics are deferred to Growth.)
**FRs covered:** FR18, FR36 (partial), FR37, FR38, FR39, FR53. NFR29; UX-DR18,31,50,58,72,73.

### Epic 18: Bulk steady-state (v0.16)
Everyday multi-select bulk triage for the steady state (distinct from day-one bootstrap): the data table with sort/multi-select/filter, and bulk actions over the same card/token vocabulary.
**FRs covered:** FR17 (steady-state bulk). UX-DR24,25.

### Epic 19: Exposer sans risque (v0.17)
The full security layer filling the E3 enforcement point and extending the E10 minimal vault: DB-backed sessions, envelope encryption in full, API-key rotation, encrypted secret backup/restore, security event log, key-path startup checks, and the authorization matrix across every surface.
**FRs covered:** FR46, FR47, FR48, FR49. NFR9,10,11,12; ARCH-13,16,25,26,27.

### Epic 20: Le réseau visible depuis ailleurs (v0.18)
Journey 7: the read-only JSON API over core entities and the authenticated Prometheus `/metrics` endpoint (filling the E3 metrics anchor).
**FRs covered:** FR43, FR44. NFR11,29; ARCH-34.

### Epic 21: Maîtriser ses données (v0.19)
Day-2 operations and disaster recovery: edit declared attributes, decommission/archive/delete a record, and full dataset export/import with resumable migrations.
**FRs covered:** FR40, FR41, FR42. NFR14,17; ARCH-14,18.

### Epic 22: First-light soigné & bilingue complet (v0.20)
The narrative first-run wizard that stages first-light (refining an experience already reachable), the complete FR translation, and the in-UI language selector — filling the E3 i18n anchor.
**FRs covered:** FR45, FR50. NFR3,26; ARCH-35; UX-DR26,32,61,62,63.

### Epic 23: Prêt à installer (v1.0)
Release hardening: the distroless static image, the two-service Docker Compose, the footprint/memory/cold-start bounds, the final responsive pass, the per-release screen-reader pass, evergreen-browser support, and bounded-downtime updates.
**FRs covered:** none new (cross-cutting NFR/ARCH/UX). NFR1,2,18,19,20,21,22,24,25,28; ARCH-6,7,8; UX-DR54,55,56,67,68,70,71.

---

## Epic 1: Les gates tiennent

Complete `cargo xtask ci` so every project gate is green and proven-to-red, and wire the CI runner so all gate logic lives in Rust (D56), never YAML. Foundation epic, no user-facing value; hard prerequisite for every later epic.

_Already implemented (commit `7d4b1bd`), out of scope here except where a story closes verification: the `ddl-collation` gate (D64 cond. 1), the `vocabulary` gate (D65 volets A+B), the `views-hash` informational check._

### Story 1.1: Dependency-frontier gate (D47)

As a maintainer,
I want `cargo xtask ci` to fail when the domain crate's dependency graph crosses the frontier,
So that `opencmdb-core` cannot silently gain an infrastructure dependency.

**Acceptance Criteria:**

**Given** a clean workspace
**When** the frontier gate runs
**Then** it shells `cargo tree -p opencmdb-core -e normal` and passes only if none of `anyhow`, `axum`, `sqlx`, `askama` appear in the graph
**And** it also fails if `xtask` appears in `cargo tree -p opencmdb-bin -e normal`.

**Given** a synthetic manifest where `opencmdb-core` declares a forbidden dependency
**When** the gate runs
**Then** it exits RED naming the crate and the offending dependency (proven-to-red test).

**Given** a `Cargo.toml` comment that merely names a banned crate
**When** the gate runs
**Then** it does not red — the gate reads the dependency GRAPH, not the manifest text (no false positive).

### Story 1.2: Fixture MANIFEST sha256 gate (scaffold)

As a maintainer,
I want a gate that verifies every committed fixture matches its recorded sha256,
So that a fixture cannot drift silently once the trap corpus exists.

**Acceptance Criteria:**

**Given** no `fixtures/` directory
**When** the gate runs
**Then** it reports "no fixtures — skipped" and is green (vacuous until Epic 4).

**Given** a `fixtures/MANIFEST` and fixture files whose bytes match their listed sha256
**When** the gate runs
**Then** it passes.

**Given** a fixture whose bytes do not match its MANIFEST sha256
**When** the gate runs
**Then** it exits RED naming the file (proven-to-red test).

**And** the MANIFEST schema is documented as provisional — its final shape is fixed in Epic 4 when the JSONL fixture format is frozen.

### Story 1.3: Prove-to-red coverage for the ddl-collation gate

As a maintainer,
I want an explicit test that the `ddl-collation` gate goes RED on a non-binary text column,
So that the gate is trustworthy before the first real migration exists to exercise it.

**Acceptance Criteria:**

**Given** a synthetic migration line declaring a text-typed column with no `_bin` / `COLLATE BINARY`
**When** the gate's detection function runs
**Then** it reports that column as a finding.

**Given** the same column declared with an explicit binary collation (e.g. `ascii_bin`)
**When** the detection function runs
**Then** it produces no finding.

**And** if such a red test already exists in `xtask`, this story is limited to confirming and documenting it.

### Story 1.4: CI runner calls xtask only (thin YAML)

As a maintainer,
I want the GitHub Actions workflow to hold no gate logic,
So that every gate is Rust that runs identically on a developer machine and in CI (D56).

**Acceptance Criteria:**

**Given** the CI workflow file
**When** it is inspected
**Then** it only invokes `cargo xtask ci`, `cargo fmt --all --check`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace --locked` — and contains no bespoke gate logic.

**Given** a pull request where any gate is RED
**When** CI runs
**Then** the check fails and names the failing gate.

### Story 1.5: MariaDB service container and cargo-deny in CI

As a maintainer,
I want CI to run against the exact target database and to gate dependency risk,
So that dev = CI = prod (ARCH-8) and advisories/licenses cannot slip in.

**Acceptance Criteria:**

**Given** a pull request
**When** CI runs
**Then** a `mariadb:10.11.11` service container is available for DB-touching tests to connect to.

**Given** a dependency with a security advisory or a disallowed license
**When** `cargo deny check advisories licenses` runs in CI
**Then** the check fails.

### Story 1.6: Pinned toolchain and Renovate automation

As a maintainer,
I want a pinned toolchain and safe automated dependency updates,
So that builds are reproducible and dependencies stay current without manual toil.

**Acceptance Criteria:**

**Given** the repository
**When** a build runs
**Then** `rust-toolchain.toml` pins the MSRV and CI uses exactly that toolchain.

**Given** a patch or minor dependency update
**When** Renovate opens it and CI is green
**Then** it is grouped and auto-merged.

**Given** a breaking (major) dependency update
**When** Renovate opens it
**Then** it is a dedicated, non-grouped PR (never two breaking changes in one commit).
