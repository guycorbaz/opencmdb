---
stepsCompleted: [1, 2]
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/architecture.md
  - _bmad-output/planning-artifacts/ux-design-specification.md
resumePoint: >
  Step 3 (create-stories) IN PROGRESS, now JUST-IN-TIME (Guy, 2026-07-19): each
  epic is decomposed into stories only when its turn to be built comes, so the
  breakdown incorporates learnings from the epics already shipped.
  EPICS 1 and 2 are COMPLETE — shipped to master, all real GitHub CI runs green
  (2026-07-19/20). Epic 1 = the gates + CI. Epic 2 = the Connector contract
  (observation types, ConnectorError, the Connector trait, a scripted connector,
  the reusable contract test) in opencmdb-core.
  EPIC 3 (Mon premier écart réel, v0.1) is DECOMPOSED into 10 small stories
  (3.1–3.10) below — the walking skeleton + a 0.1.0 Docker Hub release. Create
  and implement them one at a time (just-in-time); the riskiest (3.2 migration/
  AssertSqlSafe, 3.3 the two-traits Repository / `Reads` bomb, 3.6 the gap
  engine) get a design nod at story-creation time.
  EPICS 3 and 4 are COMPLETE (Epic 4 closed 2026-07-25, retrospective held
  2026-07-26; story 4.19 was SPLIT, 4.19b re-scoped to Epic 11 via issue #34).
  EPIC 5 (Identité d'interface fiable, v0.3) is DECOMPOSED into 17 stories
  (5.1–5.14 on 2026-07-26 with Guy, plus 5.2b INSERTED the same day, 5.4b
  INSERTED 2026-07-29 at story 5.4's contexting and 5.9b INSERTED 2026-08-03 at
  story 5.9's contexting — see the Epic 5 preamble) below. Three arbitrations were taken at
  decomposition time and live in the stories that carry them: (a) NFR4 CANNOT go
  green in Epic 5 — D18 gates at the DEVICE level, and of the 24 committed traps
  13 name an `l1-*` rule, 8 an `l2-*` rule and 3 a cause; the Epic List entry was
  corrected rather than left over-promising, and story 5.8 keeps unanswerable
  traps in the denominator; (b) the identity engine gets its OWN abstention cause,
  with `Expectation` and the sha256-locked corpus untouched (5.3); (c) Epic 5
  creates `interface`/`identity_link`/`link_candidate` but NOT `device`, which is
  Epic 6's (5.9). Stories 5.1, 5.2 and 5.2b are inherited debt placed at the HEAD
  on Guy's decision, because this epic bumps the corpus and hardening after the
  bump costs more. Create and implement them one at a time (just-in-time). Epics 6–23 remain
  at epic level only. After all epics: step-04.
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
The L1 interface-identity join — the deterministic `(l2_domain, mac) -> interface` lookup of D13 — built one trap at a time until **every trap whose expected rule is `l1-*` passes**. Abstention is a first-class persisted outcome, displayed and counted by cause. Includes the monotone-honesty invariant trap family.
**FRs covered:** FR9 (interface level), FR16, FR16b. NFR5,6. **NFR4: advanced, NOT met — see below.**

> **⚠️ NFR4 CANNOT go green in this epic, by construction — and the epic says so rather than discovering it late.** D18 places the gate **at the device level**, deliberately: *"my zero-tolerance was at the interface level, my float was at the device level. L1 is a lookup — I put zero tolerance where it is easy. L2 is the inference, where the promise lives — I put a float where it is hard. **I gated the easy and hedged the hard: exactly the cowardice I accuse the engine of, applied to myself.**"* Declaring the gate green on the L1 subset would replay that exact move. Measured against the committed corpus (24 traps): **13 name an `l1-*` rule, 8 name an `l2-*` rule, 3 are `must-abstain` and name a cause instead.** Four families are pure L1 (randomized-mac, dhcp-churn, hostname-collision, hostname-absence), two are pure L2 (multi-nic, shared-hardware-vm), and **three are MIXED** (cloned-mac, docker-veth, vrrp-virtual-mac) — their poles live in different epics and they do not move as a block. **Epic 5's commitment is the 13 L1-ruled traps; NFR4 stays RED and is closed by Epic 6.** _(This paragraph replaces "built one trap at a time until the NFR4 binary gate is green", which promised what the level split makes impossible — the same half-made admission as Epic 4's "authored here but only become executable in Epic 11". Finished 2026-07-26 at the Epic 5 decomposition.)_

### Epic 6: Ne pas compter deux fois la même boîte (v0.4)
🔴 **OPENS with the minimal actionable gesture** — a one-click promote of an observed value into a declared record (FR13) — **then** device grouping (L2): the cascade one rule per story, the seeded generator, the bulk fixture, the distributional diff, and the ephemeral-interface dormant lifecycle.
**FRs covered:** FR9 (device level), FR13 (minimal promote), FR37 (min), FR38b. NFR4,30.

_**REORDERED at Epic 5's retrospective, 2026-08-12, by Guy's decision (option a).** This epic used to CLOSE with the promote gesture. Three measurements moved it to the front. **(1)** Two thirds of all delivered work is invisible to the operator: epics 1-3 are 21 stories and everything usable today, epics 4-5 are **43 stories and nothing operator-visible**. **(2)** The documenting gesture does not exist in the product at all — five routes, all read-only, and the only call to `insert_declared_attribute` outside `repo.rs` is inside a `#[cfg(test)]`; to declare anything an operator writes SQL by hand. **(3)** ⚠️ **And it does not depend on grouping**, which is this epic's subject: promoting *"this sighting at 192.0.2.99 — declare it as a device"* needs an observation and a write surface, nothing more. FR13(a) is what the PRD itself calls **"the day-one case"**, and it is case three of the operator's three cases (no ambiguity → the software decides; ambiguity → the operator lifts the doubt; **unknown → the operator creates the entity**), arrived at independently during story 5.14b's code review. Case three is **the only case reachable today**, because the shipped ARP/ping connector emits no MAC and every abstention on a real network is therefore `AbsenceOfProof`. Recorded in `epic-5-retro-2026-08-12.md` and in a GitHub issue, not silently — the house rule since story 4.19's split._

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

_**Inherited from Epic 4 (2026-07-25 re-scope — see its closure note and GitHub issue #34):**_
- _**Run the 4.18 wire spec under the REAL parser** (D35 layer B — raw bytes, no mocks):
  `fixtures/scenario/wire/unifi-clients.json` must yield the facts and `observed_at` of
  `unifi-clients.expected.jsonl` (ids/scope are harness context — the runner injects its own).
  The charter at `fixtures/scenario/wire/README.md` lists the NAMED HOLES (the envelope, the
  `ip` key, `Hostname.source`, the OuiVendor empty-vs-absent mapping, the absent `Uplink`,
  `sw_port` on wireless) to CONFIRM or bump deliberately against the first real capture._
- _**Implement 4.19b**: the mutation generator (the MANIFEST's `generator` field gets its first
  real use), its ~30 generated fixtures under `capture/mutations/`, and their expected parse
  outcomes — all of which needed the parser to exist before they could be anything but belief._
- _**Bound by 4.19a's charter**: a renamed field must produce an explicit error, never a
  silently empty collection; `#[serde(default)]` is FORBIDDEN on any collection feeding
  presence; injecting a drift error at layer A is theatre._

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

## Epic 2: Le contrat de connecteur

Define the generalized, source-agnostic `Connector` trait and its closed error taxonomy, plus the consumer-driven contract test every connector (fixture, ARP, UniFi, future) must satisfy. No throwaway readers — every source implements this one contract. First real domain code in `opencmdb-core`. Decisions: the `Connector` trait and all its types live in `opencmdb-core` (the domain contract; D19 — the fixture, a domain test double, IS the connector); native `async fn` in trait (Rust 1.96, no `async-trait`); cancellation via `tokio-util`'s `CancellationToken` (the frontier gate D47 forbids only `anyhow`/`axum`/`sqlx`/`askama`). Covers ARCH-19/20/21; refs D19, D33, D34, D35 (NFR7).

### Story 2.1: Domain observation types

As a maintainer,
I want the core observation types (`Observation`, `Scope`, `ConnectorId`, a dated `Capabilities` descriptor),
So that every connector emits the same shape and an observation can never express "gone".

**Acceptance Criteria:**

**Given** the `opencmdb-core` crate
**When** the observation types are defined
**Then** an `Observation` records what a source saw, dated by the source (an `observed_at: Timestamp`), and has NO variant or field meaning "absent" / "gone" / "disappeared" — absence is DERIVED by the engine, never emitted by a source (NFR7 / D35).

**Given** a `Scope` on an `Observation`
**When** an observation is emitted
**Then** it carries the observation's `Scope { l2_domain: L2DomainId, vantage: VantageId }` (D19) — the MAC's uniqueness space and WHO saw it. (Note: this is the OBSERVATION scope of D19, distinct from D34 §3's liveness-blindness scope `(connector, scope)` — the "smallest set that can go blind" — which keys `source_state` and is built later with liveness, Epic 13. Do not conflate them.)

**Given** a `Capabilities` descriptor
**When** it is produced
**Then** it is a DATED FACT (carries a `Timestamp`), able to travel with a batch — not a constant (D34 §1).

**And** the types live in `opencmdb-core`, are unit-tested, and `cargo xtask ci` stays green (the frontier gate: no `anyhow`/`axum`/`sqlx`/`askama` in core).

### Story 2.2: The closed `ConnectorError` taxonomy

As a maintainer,
I want a closed `ConnectorError` enum,
So that alert suppression (FR5/FR8/FR19) can match on named causes and a connector failure is never an opaque `anyhow` string.

**Acceptance Criteria:**

**Given** `opencmdb-core`
**When** `ConnectorError` is defined
**Then** it is a `thiserror` enum with named variants covering the real failure causes (e.g. authentication, unreachable/transport, per-poll timeout, cancelled, protocol/parse) — with NO `anyhow` and NO `Other(String)` catch-all that would make FR5/FR8/FR19 inexpressible (D33).

**Given** a `Cancelled` outcome
**When** it is returned
**Then** it is a distinct variant that leaves `source_state` unchanged — it produces no gap (NFR7).

**And** each variant is exercised by a test, `Display` is meaningful, and the frontier gate stays green (`anyhow` absent from core by construction).

### Story 2.3: The `Connector` trait, `ObservationSink`, `PollSummary`, cancellation

As a maintainer,
I want the generalized `Connector` trait with incremental emission and cooperative cancellation,
So that every source implements one contract and a cut-short poll never throws away valid observations.

**Acceptance Criteria:**

**Given** `opencmdb-core`
**When** the trait is defined
**Then** `Connector` exposes `fn id(&self) -> ConnectorId` and `async fn poll(&mut self, now: Timestamp, sink: &mut dyn ObservationSink, cancel: CancellationToken) -> Result<PollSummary, ConnectorError>` (native `async fn` in trait, no `async-trait` crate).

**Given** `ObservationSink`
**When** a connector emits
**Then** it emits observations INCREMENTALLY through the sink, so observations already emitted survive a later timeout or cancellation (no total loss — D34 §2).

**Given** `PollSummary`
**When** a poll completes
**Then** it carries the batch's `Capabilities` and the `scopes_covered`.

**Given** `cancel` fires mid-poll
**When** the connector reaches a cancellation point
**Then** it returns cleanly; already-emitted observations remain valid (their `observed_at` is the source's, they do not expire because the poll was cut).

**And** cancellation uses `tokio-util`'s `CancellationToken`; `cargo xtask ci` stays green (`tokio-util` is not on the frontier denylist).

### Story 2.4: A minimal in-memory connector

As a maintainer,
I want a trivial in-memory `Connector` implementation,
So that the contract can be exercised before any real source or the JSONL fixture exists.

**Acceptance Criteria:**

**Given** a scripted batch of observations plus a `Capabilities` descriptor and scopes
**When** `poll` runs to completion
**Then** it emits them through the sink and returns a `PollSummary` carrying those capabilities and scopes.

**Given** the connector is scripted to stop early or the `cancel` token fires
**When** `poll` runs
**Then** it stops at a cancellation point and returns cleanly with what it emitted so far.

**Given** it can be scripted for the contract cases (empty batch; partial emission then an error)
**When** the contract test drives it
**Then** those behaviours are reproducible with zero mocks.

**And** it is a pure, in-memory helper (no I/O) — NOT the JSONL `FixtureConnector` of Epic 4 — and it does not enter the shipped binary path.

### Story 2.5: The consumer-driven contract test

As a maintainer,
I want a reusable contract test every connector must pass,
So that fixture, ARP, UniFi, and future connectors all honour the same behaviour.

**Acceptance Criteria:**

**Given** any `Connector`
**When** the contract test runs
**Then** it exercises the five cases: (1) empty stream, (2) partial emission then error, (3) a missing/absent field — the observation is still valid, no "gone" is fabricated, (4) timeout — `tokio::time::timeout` wrapping `poll` drops the future, yet observations already emitted through the sink survive, (5) cancellation — the token fires, `poll` returns cleanly, emitted observations survive.

**Given** the minimal in-memory connector (Story 2.4)
**When** it is run through the contract test
**Then** it passes all five cases.

**And** the harness is reusable — a function taking a connector factory — so a future connector plugs in with a single call.

## Epic 3: Mon premier écart réel

The walking skeleton: the whole stack holds (Askama + HTMX + Tailwind + sqlx + MariaDB), a real minimal ARP/ping source (implementing the E2 `Connector` trait, passing `run_connector_contract`) is ingested, and one page shows a real gap on a cardinality-1 perimeter while abstaining + counting everywhere else. Lands the compiling Repository skeleton (TWO traits, HRTB over GAT — the `Reads`-is-not-one-trait bomb), the first migration (binary collation, D64; sqlx 0.9 needs `AssertSqlSafe` for dynamic SQL), the `Clock` port routed by the reader, and the empty transversal anchors (auth-deny middleware, `/metrics`, i18n `t!()`, design tokens + focus `app.js`, `/healthz`). Closes with a **0.1.0 release published to Docker Hub** so live testing can start. FRs: FR10, FR11, FR16 (min), FR39 (min), FR3/FR4 (min). ARCH-2,30,31,32,33,37; UX-DR1,2,3,11,65,66; D66 (packaging). Slicing is many small vertical/horizontal slices (Guy). The riskiest slices (3.2, 3.3, 3.6) get a design nod at story-creation time.

### Story 3.1: Binary bootstrap and `/healthz`

As a maintainer,
I want `opencmdb-bin` to boot an axum server with config, tracing, and a `/healthz` endpoint,
So that the composition root exists and is observable before any feature is built.

**Acceptance Criteria:**

**Given** the binary, **when** it starts, **then** it loads configuration (via the `config` crate) and initialises `tracing`/`tracing-subscriber`, and serves an axum app.
**Given** a running server, **when** `GET /healthz` is called, **then** it returns `200 OK` (liveness, no dependencies checked yet).
**And** `cargo xtask ci`, clippy `-D warnings`, and fmt stay green; the frontier gate is unaffected (this is all `opencmdb-bin`).

### Story 3.2: MariaDB pool and the first migration

As a maintainer,
I want a MariaDB connection pool and an embedded first migration for declared + observed records,
So that the stack persists to the one supported engine, correctly.

**Acceptance Criteria:**

**Given** a `DATABASE_URL`, **when** the binary starts, **then** it builds a sqlx MariaDB pool (`mysql` + `tls-rustls-ring`) and applies the embedded migration(s) on startup.
**Given** the first migration, **when** it is inspected, **then** every text column carries an explicit binary collation (D64) — so `cargo xtask ci`'s `ddl-collation` gate now bites on a real migration and passes.
**Given** any dynamic SQL, **when** it is written, **then** it uses sqlx 0.9's `AssertSqlSafe` (the static `query*()` path takes `impl SqlSafeStr`).
**And** `GET /healthz` reports database reachability; CI's MariaDB service container (Story 1.5) exercises this.

### Story 3.3: The Repository skeleton — two traits

As a maintainer,
I want the read/write repository contract as TWO traits with a MariaDB adapter skeleton,
So that the domain names persistence abstractly and `sqlx::Error` dies in the adapter (D47).

**Acceptance Criteria:**

**Given** `opencmdb-core`, **when** the repository contract is defined, **then** it is TWO traits — a `&self` read side (`ReadRepository`) and a `&mut self` write/unit side (`Unit<'u>`) — because a single `Reads` trait does not compile (HRTB over a GAT); `sqlx` is NOT named in core.
**Given** the MariaDB adapter in `opencmdb-bin`, **when** it maps errors, **then** `sqlx::Error` is classified into the closed `RepositoryError` taxonomy (`Contention`/`Constraint`/`NotFound`/`Backend`) — never `#[from] sqlx::Error` leaking into core.
**And** the skeleton compiles and is exercised by a minimal round-trip test against the CI MariaDB; the frontier gate stays green.

### Story 3.4: The `Clock` port, routed by the reader

As a maintainer,
I want time to enter as a `Clock` port routed by the reader, never read inside the domain,
So that the engine is a deterministic pure function (D10/D19/D25).

**Acceptance Criteria:**

**Given** the domain, **when** it needs "now", **then** it receives a `Timestamp` bound from a `Clock` port at the composition root — the domain never calls a wall clock (core's chrono has `clock` off, so it cannot).
**Given** a test, **when** it supplies a fixed `Clock`, **then** behaviour is reproducible.
**And** the `Clock` is wired through the reader path so a later replay/fixture can drive time.

### Story 3.5: A minimal ARP/ping connector, ingested

As a maintainer,
I want a real minimal ARP/ping source that implements the `Connector` trait and whose observations are ingested,
So that observed state comes from a genuine source, not a stub.

**Acceptance Criteria:**

**Given** a declared subnet (FR3), **when** the connector polls, **then** it discovers active hosts (FR4, ping-only fallback without `NET_RAW`) and emits `Observation`s through the `ObservationSink` — and it PASSES `run_connector_contract` (Story 2.5).
**Given** a poll's observations, **when** ingestion runs, **then** they are persisted as observed records (linked-never-merged, FR11), dated by the source.
**And** the connector lives in `opencmdb-bin` (it touches the network); no private network data is committed (tests use documentation ranges).

### Story 3.6: A first real gap, abstaining elsewhere

As a maintainer,
I want the engine to compute one real gap on a cardinality-1 perimeter and abstain + count everywhere else,
So that the product's core thesis — the gap — is demonstrated end to end.

**Acceptance Criteria:**

**Given** a declared record and a linked observation that differ on a cardinality-1 perimeter, **when** reconciliation runs, **then** it reconciles by identity (FR10) and surfaces exactly that gap.
**Given** ambiguous or out-of-perimeter data, **when** the engine runs, **then** it ABSTAINS (never guesses/merges, FR16 min) and the abstention is counted and grouped by cause (reach, not debt).
**And** the gap computation is a pure function (no clock, no SQL) and is unit-tested on synthetic inputs.

### Story 3.7: One page shows the gap

As a maintainer,
I want a single web page that renders the real gap with Askama + HTMX + Tailwind,
So that a human sees the observed-vs-declared difference.

**Acceptance Criteria:**

**Given** the running app, **when** the page is served, **then** it renders the declared record, the linked observation, and the gap between them (Askama template, HTMX interactivity, committed Tailwind CSS — no CDN).
**Given** the UX baseline, **when** the page loads, **then** design tokens are applied and `app.js` manages focus on HTMX swaps (UX-DR accessibility); dark theme default.
**And** the page shows the abstention count/reach honestly (FR39 min); it never presents an abstention as a reproach.

### Story 3.8: Transversal anchors

As a maintainer,
I want the empty cross-cutting anchors in place — auth-deny middleware, `/metrics`, i18n `t!()`,
So that later features attach to existing seams instead of inventing them.

**Acceptance Criteria:**

**Given** any HTTP route, **when** it is requested unauthenticated, **then** an auth-deny middleware refuses it by default (deny-by-default seam; real auth is later).
**Given** the app, **when** `GET /metrics` is called, **then** it serves Prometheus metrics (raw `prometheus`), behind the scrape auth.
**Given** any user-facing string, **when** it is rendered, **then** it goes through `rust-i18n`'s `t!()` (EN/FR scaffolding; the forbidden-word lint seam noted).

### Story 3.9: Packaging — Dockerfile, compose template, `.env.example`

As a maintainer,
I want a Docker image and a reference compose that targets an external MariaDB,
So that opencmdb can be deployed on the NAS without leaking secrets.

**Acceptance Criteria:**

**Given** the workspace, **when** the image is built, **then** a `Dockerfile` produces a distroless, static, non-root image of `opencmdb-bin` (D66), built `--locked`.
**Given** the `docker/` directory, **when** it is inspected, **then** it holds a `docker-compose.yml` running ONLY the opencmdb service pointed at an EXISTING external MariaDB (not a bundled DB container), plus a `.env.example` with documented placeholders (RFC 5737 addresses, `CHANGE_ME`) — and the real `.env` is git-ignored.
**And** no production secrets, no real hostnames, and no NAS path appear in any committed file (they live only on the NAS).

### Story 3.10: Release 0.1.0 to Docker Hub

As a maintainer,
I want a `0.1.0` image published to Docker Hub via CI on a version tag,
So that live testing can begin from a real published artifact.

**Acceptance Criteria:**

**Given** a pushed git tag `v0.1.0`, **when** the release workflow runs, **then** it builds the image and pushes `gcorbaz/opencmdb:0.1.0` (and `:latest`) to Docker Hub using the `DOCKERHUB_USERNAME`/`DOCKERHUB_TOKEN` repository secrets.
**Given** the release workflow, **when** it completes, **then** it syncs `docker/README.dockerhub.md` to the Docker Hub repository description.
**And** `docker pull gcorbaz/opencmdb:0.1.0` works and the container starts against a MariaDB; the release is reachable for live testing. Closes Epic 3 (v0.1).

---

## Epic 4: Infra fixtures & corpus de pièges

Build the substrate the identity engine is driven against: a frozen, committed fixture format, a `FixtureConnector` that replays it through the real trait, a metrics harness written **before** the engine, and the adversarial trap corpus of D18/NFR4 — each trap in **positive AND negative** form, each asserting the RULE rather than the outcome. No FRs; this epic realizes NFR4's infrastructure (ARCH-22, ARCH-24). It ships nothing a user can see, and Epic 5 cannot start without it.

_Build order is dictated, not chosen (ARCH-24 / D19): types → the traps as SPEC → `FixtureConnector` → the metrics harness → (the L1 join is Epic 5). The order is the point: "a metric written after the engine is bent to fit the engine"._

_Two sequencing facts, recorded so they are not rediscovered late:_
- _**The wire-format traps are authored here but only become executable in Epic 11.** Layer B runs mutation fixtures under the REAL UniFi parser (D35), and that parser does not exist yet. D19 already says the traps are written "not tests yet — the spec"; the wire stories therefore deliver committed fixtures plus their expected variant, and the harness that runs them lands with the connector._
- _**The seeded generator, the bulk fixture and the distributional diff are NOT in this epic.** ARCH-24 places them after the engine. The MANIFEST's `generator` field must therefore tolerate hand-authored artefacts._

> **CLOSURE NOTE — 2026-07-25.** Epic 4 is DONE at 19/19 authored, with **story 4.19 SPLIT** and
> the split recorded rather than silent (party-mode decision: Winston, Murat, John; full record
> in `_bmad-output/implementation-artifacts/epic-4-correct-course-2026-07-25.md`).
> **4.18 shipped in full** as a spec: a synthetic body whose every field behaviour is a
> measurement, plus the Observations the future parser must produce from it, plus a shape test
> and a charter naming every hole — at `fixtures/scenario/wire/` (a declared deviation from the
> architecture tree's `capture/mutations/` placement, forced by the privacy rule: a spec is not
> a capture and does not rot). **4.19a shipped with it** — the drift-surface record (127 payload
> keys vs 7 `Fact` variants) and the layer charter as binding constraints on Epic 11.
> **4.19b — the mutation generator, its ~30 generated fixtures and their expected parse
> outcomes — moved to Epic 11**, because expected outcomes for an error taxonomy that does not
> exist would be written from belief (D45) and a generator has no test that reds without the
> parser it attacks. The promise is carried by **GitHub issue #34**, by the wire charter, and by
> `CONSUMER PENDING: Epic 11` markers on both MANIFEST entries — not by this paragraph alone.
> _The lesson, for the next decomposition: a story belongs to the epic of its CONSUMER, not the
> epic of its theme. The clause above ("authored here but only become executable in Epic 11")
> was that admission, half made._

### Story 4.1: Freeze the JSONL observation stream

As the engine's test infrastructure,
I want the on-disk fixture format frozen as a committed JSONL stream of `Observation`s, resolved through a single path constant and locked by a sha256,
So that every trap that follows is written against a format that cannot drift and cannot leak a real network.

**Acceptance Criteria:**

**Given** `fixtures/` at the workspace root with `scenario/` and `capture/`
**When** a scenario fixture is committed
**Then** each line is exactly one serialized `Observation` in the existing serde representation — no DTO, no wrapper, no hand-rolled format (D19: the fixture schema IS the Observation schema).

**Given** a committed fixture
**When** it is read
**Then** `obs_id`, `connector_id`, `scope` ids and `observed_at` come from the FILE — nothing is generated and no clock is read (D19: `obs_id` stable so truth can point at it; the engine never touches the clock).

**Given** the fixtures root must be located
**When** any code resolves it
**Then** it uses ONE constant, `concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/")`, appearing exactly once in the tree (D56 path discipline).

**Given** a fixture listed in `fixtures/MANIFEST`
**When** `cargo xtask ci` runs
**Then** the fixtures gate reports a real count instead of "no fixtures — skipped" — it stops being vacuous.

**And** every value is synthetic: RFC 5737 addresses, locally-administered MACs, invented hostnames. Real captures in a public repo are disqualifying (D19).

### Story 4.2: Freeze the truth-labelling format

As the author of the trap corpus,
I want the expectation format frozen alongside the observation stream,
So that a trap states which RULE must fire and why, not merely what the answer was.

**Acceptance Criteria:**

**Given** a trap
**When** its expectation is written
**Then** it carries exactly one of the three D18 columns — `must-not-merge`, `must-merge`, `must-abstain` — and the column is what the gate counts.

**Given** any expectation
**When** it is authored
**Then** a one-sentence `reason` is MANDATORY and the format rejects an expectation without one (D19: the oracle is the author, made explicit and versioned).

**Given** an author who cannot state the reason in one sentence
**When** the trap is classified
**Then** it becomes `must-abstain` — the inability to state a reason IS the abstention label (D19), not a case to argue over.

**Given** an expectation
**When** it names the expected outcome
**Then** it also names the expected `rule_id` (`expect_rule`), so a verdict reached by the wrong rule fails (D19: "a test that checks only the verdict goes green for the right answer reached by the wrong rule").

**And** the format is committed, versioned and readable by a human in review — it is a spec, not test data.

### Story 4.3: The MANIFEST becomes a lockfile for data

As a maintainer,
I want `fixtures/scenario/replay/MANIFEST.toml` carrying sha256, seed and generator version per artefact, and a gate that also catches files nobody listed,
So that neither an edited fixture nor an unlisted one can enter the corpus silently.

**Acceptance Criteria:**

**Given** the provisional line-based `fixtures/MANIFEST`
**When** this story lands
**Then** it is replaced by the D56 `MANIFEST.toml` carrying, per artefact, its sha256, its seed and its generator version — with the generator field allowed to be absent for a hand-authored fixture.

**Given** a fixture whose bytes no longer match its recorded sha256
**When** the gate runs
**Then** it exits RED naming the file, and the single repair is a deliberate manifest bump.

**Given** a file present under `fixtures/` but absent from the manifest
**When** the gate runs
**Then** it exits RED naming the orphan (closes the drift-in-the-ADD-direction hole deferred from the story-1.2 review; proven-to-red test).

**And** `capture/` and `scenario/` are documented as different rot risks with different treatment: captures are version-tagged, dated and re-captured; scenario traps do not rot — they are right or wrong, and the future re-capture job must be structurally unable to reach them (D56).

### Story 4.4: `FixtureConnector` replays JSONL through the real trait

As the engine's test infrastructure,
I want a connector that replays a committed JSONL fixture and passes the connector contract test unchanged,
So that "the fixture IS a connector" is a fact the compiler checks rather than a slogan.

**Acceptance Criteria:**

**Given** a committed fixture
**When** `FixtureConnector` polls
**Then** it emits exactly the fixture's observations, in file order, into the `ObservationSink` — zero mocks, zero network, zero I/O beyond reading the committed file.

**Given** `FixtureConnector`
**When** it is driven through `run_connector_contract`
**Then** it passes the same contract as every other connector, with no special-casing.

**Given** the trait cannot express what the fixture needs
**When** that is discovered
**Then** the TRAIT is wrong, not the fixture (D19) — and the finding is recorded rather than worked around.

**And** it lives in the shipped crate beside the other connectors, not under `tests/` — under `tests/` it would not face the same compilation gates and "zero mocks" would become a slogan (D56).

### Story 4.5: `FixtureConnector` replays outcomes, not only observations

_Split into **4.5a** and **4.5b** during story preparation (2026-07-22). The two record kinds are not variants of one idea: a failure ends the poll with `Err`, a capability change leaves it `Ok` with a different descriptor (D33: "`CapabilityLost` is an event, not a state — ping-only is an `Ok` with a reduced descriptor, not an error"). 4.5a lands the format mechanism; 4.5b lands the meaning, and carries all the doctrinal risk. The criteria below are distributed between the two, and **sharpened**: the split added a
positive-marker/diagnostic criterion and a "nothing may follow a terminal failure" criterion to
4.5a, an `as_of` ordering criterion and a positional-containment criterion to 4.5b, and the clause
requiring observations AFTER the failure record — without which the D35(a) assertion cannot fail.
4.5a's story sentence names two outcomes rather than three because the capability change moved to
4.5b. Nothing was dropped._

#### Story 4.5a: `FixtureConnector` replays a terminal failure

As the engine's test infrastructure,
I want the fixture to replay a poll's OUTCOME — clean or partial-then-failed — from the file,
So that layer-A fault injection needs no state outside the JSONL.

**Acceptance Criteria:**

**Given** a fixture scripting an error outcome
**When** the connector polls
**Then** it returns that `ConnectorError` variant, and any observations scripted before it are still emitted first — they are true (D34).

**Given** a stream carrying a line that is not an observation
**When** it is read
**Then** it is classified by a POSITIVE marker key before parsing, and every malformed line is still named by its 1-indexed number with a message saying what is wrong — the diagnostic story 4.1 froze must not be traded for the new record.

**And** injecting a fault may only REMOVE knowledge, never ADD an assertion, compared with the clean run of the same fixture (D35(a)/NFR8(a)) — and the fixture must place observations AFTER the failure record, or the assertion cannot fail.

**And** nothing may follow a terminal failure record: an unreachable observation would pass `read_traps`' cross-check and yield a trap that can never fire, which is the hole 4.1/4.2 exist to close.

#### Story 4.5b: `FixtureConnector` replays a capability change, dated by the file

As the engine's test infrastructure,
I want a mid-scan capability change to be one line of the fixture, with the descriptor dated by the file,
So that a downgrade trap needs no state outside the JSONL and a verdict can be replayed under the capability that produced it (D36).

**Acceptance Criteria:**

**Given** a fixture scripting a mid-scan capability loss
**When** it is replayed
**Then** one line reproduces it, with no state held outside the file (D19: "the fixture replays it for free — one JSONL line reproduces a mid-scan NET_RAW loss, zero mocks"), and the poll still returns `Ok` — a source that lost NET_RAW is `Live`, not blind.

**Given** a capability record
**When** it is loaded
**Then** its `as_of` comes from the FILE and is non-decreasing, never predating an observation it postdates — closing the recorded finding that a replay could otherwise date its descriptor in a moment its own stream contradicts.

**And** fact-kind containment becomes POSITIONAL — each observation is checked against the descriptor in force at its own position, not against one set for the whole file, because otherwise "the past would change status" (D34 §1). This supersedes story 4.4's global containment rather than dropping it.

### Story 4.6: The metrics harness, written before the engine

_Split into **4.6a**, **4.6b** and **4.6c** during story preparation (2026-07-22), after a validation
pass found the single story bundled three different review problems: a pure algebra, a file-reading
harness, and a run-comparison surface. **4.6a** owns the scored outcome, the record and the pass/fail
matrix — D18 names one failure condition per column, and the 3×3 has nine cells, so the remaining
five are derived there and written down. **4.6b** owns the harness and its vacuity report. **4.6c**
owns run comparability. The criteria below are distributed, not reduced._

_One criterion was REMOVED from the family: a guard refusing cross-stream `obs_id` collisions. It
closes an item from the story-4.1 review and touches only the corpus reader — corpus hygiene, not
metrics — and it dragged the largest regression surface into the story that must be conceptually
cleanest. It lands as its own change._

As the author of the release gate,
I want the harness that scores a run against the trap corpus to exist before any engine does,
So that the metric cannot be bent to fit the engine.

**Acceptance Criteria:**

**Given** the trap corpus and no engine at all
**When** the harness runs
**Then** it reports truth-table failures per D18 column and is GREEN vacuously — it must not require an engine to exist.

**Given** a scored trap
**When** its result is recorded
**Then** the record carries `{verdict, reason, capability_snapshot, source_state, fixture_seq}` — a verdict without its capability snapshot is unfalsifiable (D36).

**Given** two runs of the same corpus
**When** they are compared
**Then** they are comparable only under an identical capability snapshot; otherwise the harness says so rather than reporting a difference.

**And** the number the gate publishes is one: truth-table failures = 0. No fraction, no threshold — at n=300 the only measurable threshold is zero (D18).

### Story 4.7: The trap runner asserts the rule, not the outcome

As the author of the release gate,
I want each trap executed so that a right answer reached by the wrong rule FAILS,
So that the gate cannot be satisfied by an engine that will break on the next trap.

_Split into **4.7a** and **4.7b** during story preparation (2026-07-23), same idiom as 4.5/4.6. The
two are independent: 4.7a is a change to the SCORING — the `(verdict, rule)` comparison the trap
runner owns, which `score()` (4.6a) deliberately leaves it (its module doc: "compare `(verdict,
rule)`, never `verdict` alone… it becomes `assert_eq!(decision.rule, case.expect_rule)` in the trap
runner", D19/D46b). 4.7b is a check on the CORPUS — every trap present in positive AND negative form.
AC1 and AC2 go to 4.7a (AC2 as the Epic-5 contract it is pre-engine: no rule fires until Epic 5, so
4.7a records what a firing rule must leave behind rather than building a producer); AC3 goes to 4.7b.
Nothing was dropped._

#### Story 4.7a: A right verdict by the wrong rule fails

As the author of the release gate,
I want a trap scored on `(verdict, rule)` and not the verdict alone,
So that an engine reaching the right answer by the wrong rule FAILS rather than passing.

**Acceptance Criteria:**

**Given** a trap whose expectation names a `rule_id`
**When** the answer reaches the expected outcome via a DIFFERENT rule
**Then** the trap FAILS, and the failure names both the expected and the actual rule.

**And** a trap whose answer reaches the expected outcome via the EXPECTED rule still PASSES — the
rule comparison tightens the gate, it does not reject every answer.

**And** the rule comparison is layered on the 4.6a truth table, not folded into it: `score()` stays
rule-blind (a new failure mode is added beside it), so the 9-cell table's meaning is unchanged and
the wrong-rule failure is a distinct, separately-counted condition.

**Given** a rule that fires _(the Epic-5 contract — no rule fires in Epic 4)_
**When** it produces its verdict
**Then** it leaves its `rule_id` and its evidence behind — a rule that fires without leaving its
`rule_id` is a rule we cannot debug in production (D19). At v0.1 this is recorded as the contract the
Epic-5 producer must honour and pinned by the uninhabited `verdict_vector` placeholder (4.6a), not
built against an engine that does not exist.

#### Story 4.7b: Every trap exists in positive and negative form

As the author of the trap corpus,
I want a corpus missing one polarity of a trap reported as incomplete,
So that the gate cannot pass on a family that was only ever tested one way.

**Acceptance Criteria:**

**Given** the trap suite
**When** a trap family exists in only one form (positive OR negative, not both)
**Then** the corpus is reported as INCOMPLETE rather than passing — a one-sided family is a gate that
was never shown it can fail the other way.

### Story 4.8: Open the reality-debt register

As a maintainer,
I want a register of what the corpus does NOT cover, opened with the corpus itself,
So that the gate's honest limit is written down rather than discovered by a user.

**Acceptance Criteria:**

**Given** the trap suite
**When** the register is opened
**Then** it states the limit in the architecture's own words: a trap suite proves nothing about what was not imagined; at v0.1 the gate is weak and honest rather than strong and false (D18).

**Given** a real-world case the corpus cannot produce
**When** it is met
**Then** it is recorded in the register with its source, and the register is the queue from which trap #51 and beyond are drawn.

**And** the register names Tier 2 (bulk observability) as the only discovery mechanism for the unimagined — blocking nothing, feeding the gate.

### Story 4.9: Trap family — randomized MAC

As the author of the trap corpus,
I want the randomized-MAC family committed in positive and negative form,
So that the engine is proven against the median case, not an exotic one.

**Acceptance Criteria:**

**Given** a locally-administered MAC (U/L bit set)
**When** the trap is scored
**Then** at L1 it IS a distinct interface — the positive form asserts no merge across two randomized presences of the same physical interface.

**Given** the negative form
**When** it is scored
**Then** it asserts the case where merging WOULD be correct and abstaining is cowardice (D18's middle column).

**And** every expectation carries its one-sentence reason.

### Story 4.10: Trap family — multi-NIC

As the author of the trap corpus,
I want the multi-NIC family committed in positive and negative form,
So that a false SPLIT at device level is caught where it actually lives.

**Acceptance Criteria:**

**Given** two interfaces of one host
**When** the trap is scored
**Then** L1 is correct to keep them distinct and L2 must group them — the failure being tested is L2's, not L1's (architecture.md:893).

**And** the inverse form asserts that two genuinely different hosts are not grouped.

### Story 4.11: Trap family — shared-hardware VM

As the author of the trap corpus,
I want the shared-hardware VM family committed in positive and negative form,
So that virtual interfaces sharing a host are neither fused nor split wrongly.

**Acceptance Criteria:**

**Given** several virtual interfaces on one physical host
**When** the trap is scored
**Then** the expectation states explicitly which grouping is correct and why, in one sentence.

**And** the ambiguous variant is labelled `must-abstain` rather than argued.

### Story 4.12: Trap family — cloned/spoofed MAC (the inverse trap)

As the author of the trap corpus,
I want the cloned-MAC family committed,
So that the catastrophic failure — a false MERGE — has its own dedicated traps.

**Acceptance Criteria:**

**Given** two distinct hosts presenting the same MAC
**When** the trap is scored
**Then** it is a `must-not-merge`: fusing them is the failure that makes an operator lose trust and uninstall (D10/D18).

**And** the corpus records that no database CHECK can detect a false merge — the schema makes it revisable and traceable, not impossible (D18).

### Story 4.13: Trap family — DHCP churn

As the author of the trap corpus,
I want DHCP churn expressed purely as replayed timestamps,
So that time-dependent behaviour is tested without the engine ever reading a clock.

**Acceptance Criteria:**

**Given** an address reassigned between two observations
**When** the trap is replayed
**Then** the churn comes entirely from `observed_at` values in the file (D19: "DHCP churn is tested by replaying timestamps").

**And** replaying the same fixture twice yields identical verdicts — reproducibility, which is not the same as stability (D36).

### Story 4.14: Trap family — VRRP/HSRP shared virtual MAC

As the author of the trap corpus,
I want the shared virtual-MAC family committed,
So that a redundancy protocol does not read as one device.

**Acceptance Criteria:**

**Given** two routers sharing a virtual MAC
**When** the trap is scored
**Then** the expectation states whether the correct answer is not-merge or abstain, with its reason.

**And** the negative form covers the case where the same evidence legitimately belongs to one device.

### Story 4.15: Trap family — hostname collision

As the author of the trap corpus,
I want hostname collisions committed as traps,
So that a signal present on fewer than half the population cannot be over-trusted.

**Acceptance Criteria:**

**Given** two distinct hosts reporting the same hostname
**When** the trap is scored
**Then** it is a `must-not-merge`, and the expectation says why the hostname alone is insufficient.

**And** the corpus records that hostname is unusable on nearly half of known clients, so the abstention rate is bounded below by the data, not by engine quality (F51).

### Story 4.16: Trap family — ephemeral Docker veth

As the author of the trap corpus,
I want ephemeral container interfaces committed as traps,
So that short-lived interfaces do not inflate the inventory or the gap.

**Acceptance Criteria:**

**Given** an interface that appears and disappears within the observation window
**When** the trap is scored
**Then** the expectation distinguishes "gone" from "never a device", and says which in one sentence.

**And** the family is consistent with the dormant-interface lifecycle (a locally-administered MAC unobserved for the configured window becomes `dormant`, excluded from gap metrics, still queryable — F17).

### Story 4.17: Trap family — absent and empty hostname (and never null)

As the author of the trap corpus,
I want the hostname-absence family encoded from measurement,
So that the corpus tests what the source can actually produce and nothing it cannot.

**Acceptance Criteria:**

**Given** the measured behaviour of the source
**When** the family is written
**Then** it encodes MISSING and EMPTY — and **must NOT** encode `null`, which never occurs; a trap on a case the source cannot produce is a gate on a false truth (D45).

**And** each form carries its expected outcome and reason.

### Story 4.18: Wire-format traps written from measurement

As the author of the trap corpus,
I want the wire-format traps derived from the measured payload rather than from belief,
So that their red can actually arrive.

**Acceptance Criteria:**

**Given** the measured field behaviours
**When** the wire traps are written
**Then** they encode: `mac` lowercase colon-separated (100%), `last_seen` as a 10-digit SECONDS epoch (not milliseconds), `oui` empty on a large share, `vlan` missing, and `network_id` fixed-length — each closed by measurement, not conjecture.

**Given** a trap that cannot be produced by the real source
**When** it is proposed
**Then** it is rejected: a trap written from belief is a gate on a false truth whose red will never arrive (D45).

**And** the traps are committed as fixtures plus their expected variant; the harness that runs them under the real parser lands with the UniFi connector (Epic 11).

### Story 4.19: Mutation fixtures for silent schema drift

As the author of the trap corpus,
I want generated mutation fixtures covering field deleted, null, retyped and renamed,
So that the failure that fails SILENTLY is covered rather than the one that fails loudly.

**Acceptance Criteria:**

**Given** a captured response body
**When** the mutation fixtures are produced
**Then** they are GENERATED from it — deleted, nulled, retyped, renamed — not hand-written (D35), and committed with their expected parse outcome.

**Given** a renamed field
**When** the parser meets it
**Then** the expectation asserts an explicit error rather than a silently empty collection — `#[serde(default)]` is forbidden on any collection feeding presence.

**And** the corpus records why this layer exists: injecting a drift error at layer A tests nothing — it asserts the engine handles an error you handed it, without proving the parser produces one. That is the most insidious theatre of all, because it looks like fault injection (D35).

**And** it records the drift surface being defended: the payload carries 127 distinct keys where the `Fact` enum names 7.

## Epic 5: Identité d'interface fiable

Build the L1 interface-identity join and drive it against the corpus Epic 4 froze. L1 is **pure A** (D13): a deterministic lookup on the scope-qualified key `(l2_domain, mac) -> interface`. It is not a probabilistic problem — no score, no threshold, no float. The epic's product-visible promise is FR16/FR16b: **abstention is a persisted, displayable, counted-by-cause outcome**, not an error path.

_Decomposed 2026-07-26 with Guy, immediately after the Epic 4 retrospective. Three arbitrations were taken at decomposition time and are recorded in the stories that carry them: the NFR4 level boundary (see the Epic List entry above and story 5.8), the engine's own abstention cause (5.3), and what persistence Epic 5 does and does not create (5.9)._

_**Three** of the seventeen stories are **inherited debt**, placed at the HEAD on Guy's decision: the corpus byte-fidelity and corpus privacy themes had each accumulated three to four unowned entries in `deferred-work.md`. They come first because **this epic bumps the corpus**, and hardening after the bump means replaying every entry against artefacts that have moved._

_**Story 5.2b was INSERTED on 2026-07-26**, one day after the decomposition and before any Epic 5 code existed. It was surfaced while preparing story 5.1: four committed trap families (randomized-mac, multi-nic, shared-hardware-vm, cloned-mac) turned out to be named by no test at all, so 5.1's AC1 — which strengthens byte-pins that EXIST — could not reach them. It was neither absorbed into 5.1 nor left in the register, because the corpus is the oracle the L1 join is about to be judged against: hardening it after its first consumer exists means bending the engine to fit whatever the corpus happens to say. The letter suffix is the house idiom for an inserted item (D56b, AC5b/7b/7c), chosen so 5.3–5.14 keep their numbers._

_Build order: the three debt stories (5.1, 5.2, 5.2b) -> the engine's vocabulary (5.3, 5.4) -> the verdict algebra (5.4b) -> the pure join (5.5) -> the blocker (5.6) -> wiring it to the corpus (5.7, 5.8) -> persistence (5.9 the schema, 5.9b the resolver that fills it, 5.10 the purge) -> the invariants (5.11, 5.12, 5.13) -> the operator-visible surface (5.14). No story depends on a later one._

### Story 5.1: The corpus pins the obs_id-to-line binding, and every stream goes through the connector

As the owner of the corpus,
I want each byte-pin test to assert the `obs_id` of the line it reads, and every committed stream to be loaded through `FixtureConnector::load`,
So that a re-authored stream cannot invert what its traps judge while every assertion stays green.

**Acceptance Criteria:**

**Given** the byte-pin tests of the dhcp-churn and vrrp families, which read observations by INDEX while their traps judge by `obs_id`
**When** the pins are strengthened
**Then** each pinned observation asserts its `obs_id`, so a deliberate swap of two lines' ids with a re-hashed manifest reds instead of silently inverting the family (registered in `deferred-work.md` under story 4.15's review; 4.15 fixed its own).

**Given** the committed replay streams, of which only `minimal.jsonl` is exercised through the connector's admissibility checks
**When** the corpus walk runs
**Then** every stream under `scenario/replay/` is loaded through `FixtureConnector::load`, so corpus-level parseability and connector-level admissibility stop being two different claims (registered under story 4.12's review).

**And** the round-trip byte-shape witness covers every committed stream, not `minimal.jsonl` alone (registered under story 4.10's review).

**And** each strengthened guard is proven to red before it passes — the mutation is recorded (house rule, story 1.3).

### Story 5.2: The privacy floor reaches the bytes it always claimed to cover

As the owner of the corpus,
I want trap-file text and the `Observation.raw` field routed through the synthetic-text scanner, and the scanner's named evasions closed,
So that the privacy rule covers the COMMITTED BYTES rather than only the fields a decision happens to read.

**Acceptance Criteria:**

**Given** `assert_text_is_synthetic`, whose only corpus call site is the `Record::Failure` walk over replay streams
**When** the walk runs
**Then** the headers and `reason` strings of `fixtures/scenario/traps/*.toml` are scanned too, comments included, before TOML parsing discards them (registered under story 4.14's review; the rule is held by review today, not by a gate).

**Given** `Observation.raw`, documented as "never read by a decision" and carrying uninspected prose in `minimal.jsonl`
**When** the corpus privacy walk runs
**Then** `raw` is scanned by the same rule — the charter is the committed bytes, not what decisions read (registered under story 4.16's review).

**Given** the scanner's evasions, named by its own doc as "a floor, not a proof"
**When** they are closed
**Then** a MAC or IP followed by kept punctuation no longer tokenizes unparseable, dash-form MACs are seen, and the U/L-bit rule stops admitting IPv6-multicast-shaped strings while refusing IPv4-multicast ones (registered under story 4.14's review).

**And** each closure is proven to red on a committed-shaped input before it passes.

### Story 5.2b: The four unpinned families — and dhcp-churn's authored values — state their premise in a test, not only in prose

_Inserted 2026-07-26 with Guy, one day after the decomposition and before any Epic 5 code was written — surfaced while preparing story 5.1, which could not absorb it (its AC1 strengthens byte-pins that EXIST; these four families have none). The letter suffix follows the house idiom for an INSERTED item (D56b, AC5b/7b/7c) so that 5.3–5.14 keep their numbers. It sits HERE, in the inherited-debt block and ahead of 5.5, deliberately: the corpus is the oracle the L1 join will be judged against, and hardening an oracle after its first consumer exists means bending the engine to fit whatever the corpus happens to say._

As the owner of the corpus,
I want a byte-pin test for the randomized-mac, multi-nic, shared-hardware-vm and cloned-mac families, and the authored values of dhcp-churn pinned by value,
So that no committed family can state a premise its own bytes contradict, and no trap can pass for the wrong reason.

**Acceptance Criteria:**

**Given** that `randomized-mac.jsonl`, `multi-nic.jsonl`, `shared-hardware-vm.jsonl` and `cloned-mac.jsonl` are named by no VALUE test in the tree — their only mention is the per-stream context table story 5.1 added (`fixture_connector.rs`, `committed_stream_contexts()`), which states each stream's declared context and asserts nothing about its contents — so their authored values and their `obs_id` ↔ line binding are asserted by nothing narrower than the corpus walks and the sha256 lock, and `read_traps` only checks that a trap's `obs_id`s EXIST, never which line they name _(this clause read "named by NO test in the tree (`grep -rn "<name>.jsonl" --include=*.rs crates xtask` returns nothing for all four)" until 2026-07-27: story 5.1's own commit falsified that grep — it now returns four hits, all in the table above. The conclusion is unchanged; only the check that establishes it is.)_
**When** each family gains a byte-pin, in the shape 4.15–4.18 established (stream length · fact count per line · `assert_obs_ids` from story 5.1 · value pins on the premise)
**Then** a deliberate re-authoring with a refreshed manifest can no longer invert what the family judges while every assertion stays green.

**Given** `randomized-mac.jsonl` — 3 presences whose whole family rests on ONE octet
**When** it is pinned
**Then** N1 and N2 carry the byte-identical MAC `02:00:5e:00:53:20` (value-pinned, not merely asserted equal), N3's differs and is value-pinned too, the three addresses `192.0.2.30/.31/.32` are pinned, ALL THREE lines carry exactly 2 facts (`Mac` + `IpV4` — measured 2026-07-27; this read "both lines" until story 5.1's review caught a three-presence premise constrained for two), and the instants are the authored vector — so a one-octet edit that turns the `l1-exact-mac` pair into a distinct-MAC pair reds HERE rather than at the engine's first run, where it would read as the engine's fault.

**Given** `multi-nic.jsonl`, whose premise is entirely geometric and which the harness validates nowhere (the VRRP byte-pin's own doc says uplink geometry is pinned "here or nowhere" — true precisely because VRRP HAS a byte-pin)
**When** it is pinned
**Then** M1 and M2 carry the SAME `peer_mac` and DIFFERENT `peer_port` (`swport-1` / `swport-2`) — the whole `Uplink` fact value-pinned, both halves — and M3 carries a DIFFERENT `peer_mac` with `swport-7`, so neither "same switch, different port = agrees" nor "different switch = opposes" can be silently exchanged, and collapsing the two ports into one (which would make this the shared-hardware-vm shape) reds.

**Given** `shared-hardware-vm.jsonl`, whose trap header declares the uplink "shared by construction (the same `peer_mac` and `peer_port` on every observation)" — prose that no test asserts
**When** it is pinned
**Then** all FOUR observations carry the byte-identical `Uplink` (peer `[2,0,94,0,96,10]`, port `swport-1`), W1/W2 share `doc-vm-alpha` and W3 carries `doc-vm-beta` (all value-pinned), W4 carries NO `Hostname` fact as an assertion rather than an accident (`.iter().all(|f| !matches!(f, Fact::Hostname { .. }))`, story 4.17's idiom), and the four MACs are distinct and value-pinned — so the discriminator stays the hostname and cannot silently become the topology.

**Given** `cloned-mac.jsonl`, the corpus's ONLY pre-release guard against the false merge — D21 refuses a unique index on `interface.mac_canon` deliberately, so the schema cannot be one, and D10 calls the false merge catastrophic and asymmetric
**When** it is pinned
**Then** all THREE presences carry the one byte-identical MAC `02:00:5e:00:53:70` (value-pinned on each line, not pairwise), K1 and K3 carry the identical hostname `doc-host-echo` while K2 carries `doc-host-foxtrot` (value-pinned), the three addresses are pinned and distinct, and the three `obs_id`s are pinned — so neither a one-octet edit (which would turn the `must-not-merge` into a tautology any engine passes) nor an `obs_id` permutation (which would make the corpus DEMAND the false merge) can land unseen.

**Given** `dhcp-churn.jsonl`, whose byte-pin asserts its MACs and hostnames only RELATIONALLY while both `reason` strings cite `02:00:5e:00:53:78`, `doc-host-golf` and `doc-host-hotel` — values no test asserts (registered under story 4.13's review, the last open corpus byte-fidelity entry)
**When** the existing byte-pin is extended
**Then** those three values are pinned by value, so a re-authored stream carrying different synthetic values can no longer strand its own reasons.

**And** every pin is proven to red before it passes, one recorded mutation per family, each aimed at a stream no OTHER value test reads so the red cannot be observed for the wrong reason (story 5.1's lesson).

**And** the register is closed by appending, never rewriting: `deferred-work.md`'s story-4.13 entry and the story-5.1 entry are both marked `✅ CLOSED by story 5.2b` — after which the corpus byte-fidelity theme carries no open item.

**And** nothing under `fixtures/` moves: no bytes, no `MANIFEST.toml`. A pin that requires re-authoring an artefact is a FINDING, reported rather than absorbed.

### Story 5.3: The identity engine gets its own abstention cause

As the identity engine,
I want an abstention cause vocabulary of my own rather than the reconciliation engine's,
So that `Ambiguous` — which emerges from the verdict algebra and which none of the three reconciliation causes names — can be produced and displayed honestly.

**Acceptance Criteria:**

**Given** `AbstentionCause { OutOfPerimeter, NoObservedValue, ConflictingObservations }`, which `reconcile` matches exhaustively and which `score.rs` records as inadequate for the cascade
**When** the engine abstains
**Then** it names a cause from its OWN enum, and `Ambiguous` — a `Decisive` verdict with at least one `Opposes`, the cloned-MAC case (D13) — is one of its variants.

**Given** the committed corpus, which writes `must-abstain = { cause = "NoObservedValue" }` in three sha256-locked trap files
**When** the new type lands
**Then** `Expectation::MustAbstain` keeps `AbstentionCause` and **no corpus artefact is re-hashed** — the change is confined to `Outcome::Abstained`.

**Given** that the two sides of a trap now carry DIFFERENT cause types
**When** a reader meets them
**Then** both types document the asymmetry and why it is sound: nothing ever compares the two — `score` is cause-blind by construction — so the gate cannot go asymmetric. An undocumented asymmetry would read as a defect, which is the class of finding this project's reviews keep catching.

**And** `reconcile` is not widened with a variant it can never produce.

### Story 5.4: `Decision` — the engine's return type, and its ruleset version

As the identity engine,
I want a return type carrying the verdict, the rule that produced it, its evidence and the ruleset version,
So that the explanation is free (D13) and improving the engine is not a silent data migration (D14).

**Acceptance Criteria:**

**Given** the name `Decision`, which `score.rs` deliberately left unclaimed ("taking it here would squat a type Epic 5 has to define")
**When** the type is defined
**Then** it carries the `(rule, verdict, evidence)` triple that IS the explanation, and an abstention carries a cause and no rule — the same shape `Outcome` mirrors, so `run_trap`'s existing assertion needs no runtime guard.

**Given** D13's refusal of `rule -> confidence: f64` ("if the output is a float, B has won in disguise")
**When** the type is reviewed
**Then** no float crosses a decision boundary; any ranking value is an INTEGER in milli-units and may order candidates for display only. _(SPLIT 2026-07-29: the TYPE-level half is story 5.4's — no type it defines carries a float or a magnitude; the **gate** that holds it mechanically is story 5.4b's, below. Measured at the split: zero `f32`/`f64` in the whole Rust workspace, so the rule was true by accident.)_

**Given** D14's "`ruleset_version` is mandatory"
**When** a decision is produced
**Then** it carries the version of the ruleset that produced it.

**And** the verdict algebra of D13 (`Decisive` / `Supports` / `Neutral` / `Opposes` / `Disqualifying`) is expressed as an enumeration, and combining verdicts is an algebra, never a sum. _(SPLIT 2026-07-29 at story 5.4's contexting: the enumeration is story 5.4's; **combining** them is story 5.4b's, below.)_

### Story 5.4b: The verdict algebra is a total function, and no float can reach it

_Inserted 2026-07-29 with Guy, at story 5.4's contexting and before any Epic 5 engine code existed. Two reasons, both measured. **(1)** Enumerating D13's table [architecture.md:967-974] over the PRESENCE of each verdict yields seven input classes and the six rows cover six of them: **`≥1 Opposes` with no `Decisive`, no `Supports` and no `Disqualifying` is covered by no row.** A function that must be total therefore needs an arbitration D13 does not supply, and that arbitration deserves to be the subject of a story rather than a subsection of one. **(2)** D13's *"if the output is a float, B has won in disguise"* is held today by nothing — measured at `505379e`: zero `f32`/`f64` in the whole Rust workspace, so the rule is true by accident. The letter suffix follows the house idiom for an INSERTED item (D56b, AC5b/7b/7c, story 5.2b) so 5.5–5.14 keep their numbers._

As the identity engine,
I want the verdict algebra as a total pure function, and a gate that refuses a float in the identity subtree,
So that no input class falls through the table unnoticed and no weight can enter through the back door.

**Acceptance Criteria:**

**Given** story 5.4's `Verdict`, `RuleVerdict`, `Conclusion`, `Decision` and `RulesetVersion`, and D13's six-row table
**When** the algebra is written
**Then** `decide` is a **pure function over a verdict set** — no clock, no I/O, no SQL — implementing every row, with each arm citing the architecture line it comes from.

**Given** the input class D13's table does not cover — `≥1 Opposes`, no `Decisive`, no `Supports`, no `Disqualifying`
**When** it is met
**Then** the conclusion is **`Abstained { AbsenceOfProof }`** _(Guy's arbitration, 2026-07-29)_: nothing argues FOR the merge, so there is no merge to refuse, and D13 deliberately reserves the refusal-that-names-a-rule for `Disqualifying`. The gap and the arbitration are documented at the function, and recorded as a correction to be carried into D13 at a milestone — never patched into `architecture.md` inside a story.

**Given** that a decision names ONE rule while several rules may be `Disqualifying` or `Decisive` at once
**When** more than one qualifies
**Then** the rule named is chosen **deterministically and independently of the order the verdicts arrive in** — a property tested by permuting the input, not asserted in prose. *"The one written first… is not a decision, it is an accident of file order"* [architecture.md:936-937]; this also pre-empts story 5.11.

**Given** D13's *"REFUSED: `rule -> confidence: f64`… if the output is a float, B has won in disguise"* [architecture.md:956-958]
**When** `cargo xtask ci` runs
**Then** a gate reds on any `f32` or `f64` under `crates/opencmdb-core/src/identity/`, in the idiom of the existing DDL-collation and retired-vocabulary greps (D56/D65 — every gate lives in Rust, never in YAML), with its own prove-to-red.

**And** no ranking value is invented: D13's milli-units corollary [architecture.md:988-993] binds the day a float would otherwise appear, and no candidate ordering exists in Epic 5 (L1 is a deterministic lookup). The deferral is registered with its owner.

**And** the table's totality is proven by exercising **every** input class, not a sample — and each guard is proven to red before it passes, the mutation recorded (house rule, story 1.3).

### Story 5.5: The L1 join, as a pure function

As the identity engine,
I want the interface-identity join expressed as a pure function over observations,
So that the deterministic part of identity is testable without a database, a clock, or an ingestion order.

**Acceptance Criteria:**

**Given** a set of observations carrying `Scope { l2_domain, vantage }` and `Fact::Mac`
**When** the join runs
**Then** it resolves `(l2_domain, mac) -> interface` deterministically — D13's "L1 = pure A… it is not a probabilistic problem" — with no clock read, no SQL, and no consultation of how a declared value was obtained.

**Given** two observations carrying the same MAC in DIFFERENT `l2_domain`s
**When** the join runs
**Then** they are not the same interface: the key is scope-qualified, not the bare address.

**Given** structural facts about an address — locally administered (the U/L bit), or an IANA redundancy-protocol prefix
**When** they are met
**Then** they are read at ingestion and never scored (D13: "everything structurally knowable must be known at ingestion… confusing an IANA fact with scoring turns a fact into a probability, and that is how weights get invented").

**And** the function is tested against synthetic inputs directly, independently of the corpus harness.

### Story 5.6: The blocker, and the recall assertion nobody writes

As the identity engine,
I want candidate generation to be an explicit component with a measured recall floor,
So that false splits cannot be born silently before any rule has a chance to speak.

**Acceptance Criteria:**

**Given** D13's named blind spot — *"if the candidate generator does not propose the pair, no downstream logic can ever group. That is where false-splits are born silently, and nobody tests blockers"*
**When** the blocker is built
**Then** a dedicated assertion measures `blocking_recall >= 0.999` in unit tests, and it exists BEFORE the scoring that consumes it.

**Given** that at 300 hosts 90k pairs is not a performance concern on the reference NAS
**When** the blocker's purpose is documented
**Then** it states that the blocker exists for SEMANTICS: it defines the universe of plausible candidates, hence what "ambiguous" means — *"without blocking, abstention has no denominator."*

**And** the assertion is proven to red by a blocker that drops a pair the corpus requires.

### Story 5.7: The trap runner stops scoring nothing

As the release gate,
I want the L1 engine wired into the corpus harness,
So that the committed traps become a gate that runs instead of data that is merely discovered and parsed.

**Acceptance Criteria:**

**Given** `score_corpus`, which today discovers, reads and validates the corpus while every trap is "scored by nothing"
**When** the harness runs
**Then** each trap whose expected rule is `l1-*` is answered by the real engine, and the resulting `(verdict, rule)` is compared by `run_trap` — the assertion story 4.7a built and left waiting.

**Given** the four pure-L1 families — randomized-mac, dhcp-churn, hostname-collision, hostname-absence
**When** the gate runs
**Then** their traps pass in BOTH poles, and a failure is readable per column through the existing `Tally`.

**Given** a right verdict reached by the WRONG rule
**When** it is scored
**Then** it fails in `Report.rule_mismatches`, distinct from a truth-table failure — the separation story 4.7a established is preserved.

**And** replaying the corpus twice yields an identical `Report` (D36 reproducibility, pinned by story 4.13's test).

### Story 5.8: A trap whose level the engine does not implement counts as NOT PASSING

As the release gate,
I want traps expecting a rule level the current engine does not implement to be counted in a named bucket,
So that a green gate can never mean "we did not ask the question".

**Acceptance Criteria:**

**Given** the **11** committed traps the L1 engine cannot answer, in **three** classes — **8** whose expected rule is `l2-*` (the level is not implemented), **2** `must-abstain` traps that name a pair but no rule to route on, and **1** that names no pair at all
**When** the corpus is scored
**Then** they are counted as NOT PASSING in a fourth named bucket, beside truth-table failures, rule mismatches and incomplete families — they never silently leave the denominator.

_(This criterion said **"the 8 traps whose expected rule is `l2-*`"** until 2026-08-02. Measured by story 5.7 and again at 5.8's contexting: the residue is **eleven**. The three `must-abstain` traps are invisible to an `l2-*` selector because `Expectation::MustAbstain` carries a CAUSE and no rule, so `Expectation::rule()` returns `None` for all three — a bucket built to hold eight would have left three traps outside it, which is the very silence this story exists to close. Story 5.7 registered the correction in `deferred-work.md` with story 5.8 as owner rather than editing this file, because 5.7 was verify-only here; story 5.8 owns it and applies it. The 8 / 2 / 1 split is asserted by `l1_runner`'s `the_residue_decomposes_into_eight_two_and_one`, not merely quoted.)_

**Given** D18's rejection of decoration — *"a loose threshold on a benign defect is a gate that can never fall, and a gate that cannot fall is decoration"*
**When** the gate reports
**Then** its output states plainly how many traps were unanswerable at this level and why, and `passed()` is blocked by that bucket exactly as it is by the other three.

**Given** the three MIXED families (cloned-mac, docker-veth, vrrp-virtual-mac), each holding both an `l1-*` and an `l2-*` pole
**When** they are scored
**Then** the L1 pole is answered and the L2 pole is bucketed — the family does not move as a block, and its completeness check is not read as a failure of Epic 5.

**And** the gate's own report names NFR4 as NOT MET at this epic, at the device level, closed by Epic 6.

### Story 5.9: The interface and its identity link are persisted, ambiguity included

_**SPLIT 2026-08-03 with Guy, at this story's contexting and before any persistence code existed.** As written this story carried two ideas: **(a)** the schema and the persistence contract, and **(b)** the path that runs the engine over a set of observations and WRITES the links it derives. (b) is the heavier half, it is the first production caller of `identity::l1::join` and `identity::blocking::candidates` (a residue the register has owned as *"story 5.9 or Epic 6"* since story 5.6), and it is what story 5.10's *"the engine re-runs"* requires. Splitting keeps each story one idea, in the house idiom for an INSERTED item (D56b, stories 5.2b and 5.4b) so 5.10–5.14 keep their numbers: **5.9 is the schema and its adapter, 5.9b is the resolver that fills it.** No acceptance criterion below is weakened — all four are schema statements and all four stay here._

_**Arbitration taken at the same contexting: what an `identity_link` LINKS at this level is `observation -> interface`.** `decide_pair` judges a PAIR of observations and returns no interface; `join` groups observations by `(l2_domain, mac)` and therefore **is** what forms an interface at L1. So the persisted link binds one `observation_record` to one `interface`, carrying the rule, the evidence, when, by whom and `ruleset_version`; `link_candidate` carries the N candidate **interfaces** of an ambiguous observation, which is what FR16's *"present the candidate matches with their evidence"* asks to display. The `interface -> device` grouping is a different relation and belongs to Epic 6 with `device`. **No `entity` supertype table is created either** (D21's disjunction has one arm while `device` does not exist); its owner is Epic 6, with `device`._

As the operator,
I want interfaces and their identity links stored as revisable records carrying their evidence,
So that "present the candidate matches with their evidence" (FR16) is something the product can actually do.

**Acceptance Criteria:**

**Given** a schema holding only `declared_attribute` and `observation_record`
**When** this story lands
**Then** it creates `interface`, `identity_link` and `link_candidate` — and **NOT** `device`: the grouping level is Epic 6, and a table this epic would not populate is speculation (the "create tables only when the story needs them" rule).

**Given** D14's "the link is an ENTITY, not a foreign key"
**When** a link is written
**Then** it is SCD2 (append plus a single closing stamp), carrying the rule applied, the evidence, when, by whom, and `ruleset_version` — *"a bad link is UNLINKED, never erased."*

**Given** an ambiguous outcome
**When** it is persisted
**Then** it is a LINK with its `link_candidate` rows and their evidence, never an absence — *"the ambiguity is DATA, not a hole; otherwise there is nothing to display and FR16 is vapour."*

**And** every text column carries an explicit binary collation (D64), and the DDL gate stays green.

### Story 5.9b: The engine resolves a set of observations and writes the links it derives

_Inserted 2026-08-03 with Guy, at story 5.9's contexting and before any persistence code existed — see the SPLIT note on story 5.9 above for why. It is the first story that hands `identity::blocking::candidates` and `identity::l1::join` something OTHER than a trap: a trap NAMES its pair, so the trap runner (5.7) had nothing to generate and deliberately called neither. The register has carried that residue as *"story 5.9 or Epic 6, whichever first hands the blocker a set of observations"* since story 5.6; this story is it._

As the operator,
I want a scan's observations turned into interfaces and identity links by the engine, in one deterministic pass,
So that what the engine decides is written down rather than recomputed, and story 5.10 has something to purge.

**Acceptance Criteria:**

**Given** a set of observations and the tables story 5.9 created
**When** the resolver runs
**Then** each observation carrying a MAC lands on exactly ONE `interface` — the one its `(l2_domain, mac)` key names — and each such placement is written as an `identity_link` with the rule that settled it, its evidence and `ruleset_version`.

**Given** `identity::blocking::candidates`, which until this story had no production caller, and `identity::l1::join`, which had no cross-crate caller at all
**When** the resolver runs
**Then** both are called by production code, and D13's order — *candidate generation (blocking) -> verdicts -> three-way decision* — is the order of the pass; the blocker is not bypassed by reading the join's key directly.

**Given** D21's *"identity resolution runs INSIDE the writer actor, against the write connection"*
**When** two observations of the same MAC arrive in ONE pass
**Then** the second sees the first's link — read-your-own-writes inside one transaction — and **an identity decision is never split across two transactions**.

**Given** an observation the engine abstains on
**When** the pass writes
**Then** it writes a LINK carrying the cause and its `link_candidate` rows, never an absence (D14/FR16) — the same refusal story 5.9's schema makes representable.

**And** the pass reads no clock for anything it stores as an interface's `first_seen_at`/`last_seen_at`: those are derived from the observations, so a re-run reproduces them — which is the precondition story 5.10 tests.

**And** each guard is proven to red before it passes, the mutation recorded (house rule, story 1.3).

### Story 5.10: The purge test proves the link is a cache of attention, not of truth

As the architect of the invariant,
I want engine-decided links to be reproducible bit for bit after deletion,
So that D14 and D4 ("doubt is never persisted") are reconciled by a test rather than by an argument.

**Acceptance Criteria:**

**Given** persisted links, some `decided_by = 'ENGINE'` and some `decided_by = 'OPERATOR'`
**When** `TRUNCATE ... WHERE decided_by = 'ENGINE'` is run and the engine re-runs
**Then** the engine-decided links are reproduced identically, bit for bit (D14's stated test).

**Given** operator-decided rows
**When** the purge runs
**Then** they are untouched — they are INPUTS, not derivations, on a par with an observation. *"Two natures in one table — and if that frontier is fuzzy in the code, the invariant is dead."*

**And** the test reds if any engine decision is made to depend on state the purge removes.

### Story 5.11: Reconciliation is independent of the order observations arrive in

As the operator,
I want the same observations to produce the same identity decisions regardless of arrival order,
So that a scan's timing cannot change what the product believes (NFR6).

**Acceptance Criteria:**

**Given** a corpus replay stream
**When** its observations are presented in a fuzzed arrival order
**Then** the resulting decisions are identical to the in-order run.

**Given** the same stream replayed twice into an already-populated store
**When** the second run completes
**Then** it is idempotent — no new link version is written for an unchanged decision.

**And** the fuzzing is seeded and the seed recorded, so a failure is reproducible rather than anecdotal.

### Story 5.12: No code path writes a declared field without a human author

As the operator,
I want the never-overwrite invariant covered by explicit anti-regression tests,
So that the product's central promise — linked, never merged — cannot erode by accident (NFR5).

**Acceptance Criteria:**

**Given** the identity engine, now writing to the database for the first time
**When** the anti-regression tests run
**Then** no code path writes a `declared_attribute` with a non-human author, and the test reds if one is introduced.

**Given** a divergence computation
**When** it runs
**Then** it never consults HOW a declared value was obtained (FR13's invariant, NFR5's second clause).

**And** the tests are written as guards that red on removal, not as assertions over current behaviour.

### Story 5.13: The monotone-honesty trap family — a faulted run cannot invent a fact

As the operator,
I want a fault to be able to REMOVE knowledge and never to ADD any,
So that degradation is honest rather than creative (NFR8's first falsifiable assertion).

**Acceptance Criteria:**

**Given** a run under fault injection — a source blinded mid-scan, a poll failing, a capability reduced
**When** its output is compared with the clean run
**Then** the faulted run's facts are a SUBSET: it may know less, it may never assert something the clean run did not.

**Given** the replay format's control records, which already express a failed poll and a mid-stream capability change (stories 4.5a/4.5b)
**When** the family is authored
**Then** it uses them rather than inventing a new mechanism, and each trap is committed in positive AND negative form, naming the RULE rather than the outcome — the corpus doctrine.

**And** the family is added to `MANIFEST.toml` as a deliberate bump, and the corpus lock reports the new count.

### Story 5.14: Abstention is displayed, counted and grouped by cause — and never as a reproach

As the operator,
I want to see how many interfaces the product could NOT place, broken down by why,
So that the number measures the product's reach rather than my debt (FR16b).

**Acceptance Criteria:**

**Given** a population of interfaces, some of which the engine abstained on
**When** the page renders
**Then** it shows, beside the evaluated population, the count NOT evaluated, **grouped by cause** — each cause one line, not N failures.

**Given** the UX spec's backlog bans (*Dignity*)
**When** the counter is styled
**Then** it does not redden, does not grow bold, carries no gauge and no badge, and does not age visibly: after six months of inaction it reads the same number, in the same grey. *"It measures the product's REACH, not the operator's debt."*

**Given** an abstention whose cause is `Ambiguous`
**When** it is displayed
**Then** its candidates and their evidence are shown from the persisted `link_candidate` rows (FR16) — the abstention explains itself.

**And** the measured floor is stated where it is displayed: the abstention rate is bounded below by DATA AVAILABILITY, not by engine quality (FR9, NFR30) — hostname is unusable on nearly half of known clients, and no amount of correctness recovers a signal the source never sent.

---

## Epic 6: Ne pas compter deux fois la même boîte

**Goal:** the operator can DOCUMENT what the product found, and the product stops counting one machine twice. **FRs:** FR9 (device level), FR13 (minimal promote), FR37, FR38b. **NFRs:** 4, 5 (assertions 2 and 3), 30.

_**Decomposed 2026-08-12 with Guy, after Epic 5's retrospective and its project review.** Nineteen stories, sliced deliberately finer than Epic 5's fourteen: Epic 5 was planned at fourteen and delivered twenty, and all six insertions were found while contexting a story that carried two ideas. **The order is the reordering of 2026-08-12 (issue #85): the documenting gesture OPENS the epic**, because it is what J3 — *a real gap detected AND CORRECTED* — has been missing, and it does not depend on grouping._

_**Four measured constraints this decomposition respects, so no story rediscovers them:**_

_**(1) The corpus already NAMES the L2 rules, and the names are load-bearing.** The committed traps expect `l2-different-hostname` (3 traps), `l2-uplink-agrees` (2), `l2-hostname-agrees`, `l2-different-switch` and `l2-virtual-mac-prefix`. Epic 4's retrospective warned it: implementing one under a different name reds the trap as `rule_mismatch` and forces a deliberate corpus bump. The rule order below follows the corpus, most-expected first._

_**(2) 🔴 The five rules do NOT turn the release gate green.** They take the unanswerable bucket from **11 to 3**, and the repository already says so. The last three are `must-abstain` traps that name NO rule, so an `l2-` prefix selector cannot route them (story 5.7's finding). **`epics.md` has always said the gate closes with this epic and milestone J4 says the same** — story 6.15 is what makes that true, and it exists because the decomposition checked the arithmetic instead of trusting the sentence._

_**(3) The documenting gesture writes `declared_attribute`, which is what story 5.12's authorship gate guards.** `SANCTIONED_SITES` admits three sites today; a promote adds one. And **two of NFR5's three assertions were registered against the triage epic while waiting for this gesture** — they arrive here, in story 6.3._

_**(4) `Ambiguous` becomes producible in this epic, and `guard_decision` REFUSES an `Ambiguous` carrying no candidates.** `link_candidate` has had no production writer since story 5.9; whoever produces the first `Supports`/`Opposes` owns filling it (story 6.13). ⚠️ **Story 5.14b's tripwire — `the_production_pass_produces_no_ambiguous_abstention` — will RED there, by design**, and its message names `epics.md`'s FR16 clause as the work that has come due. Do not delete it; implement 6.14._

### Story 6.1: The write route exists, and it writes nothing

As the operator,
I want the product to expose the shape of a documenting action before it can perform one,
So that the route's refusals are settled while nothing is at stake.

**Acceptance Criteria:**

**Given** a running binary
**When** the documenting route is called
**Then** it answers with an enumerated refusal — the request shape, the unknown subject, the absent switch — and **writes no row**.

**Given** a default deployment
**When** the write switch is unset
**Then** the route does not exist at all: **a deployment nobody configured is not writable.** ⚠️ **This is a SWITCH, not authentication, and no document may call it authentication.** `auth.rs` is a deny-by-default seam whose public allowlist its own doc calls temporary; real sessions are Epic 19's, seven months out in the Gantt. *(Guy's arbitration, 2026-08-12, over the alternative of joining the temporary allowlist and registering the exposure.)*

**And** it follows story 5.3's precedent — the vocabulary ships before the engine, so the refusals are testable before any write path exists.

### Story 6.2: The route writes a declared value, through the adapter and nowhere else

As the operator,
I want an observed value to become a declared one,
So that what the product found becomes what I documented.

**Acceptance Criteria:**

**Given** an observation the engine could not place
**When** the operator documents it
**Then** `declared_attribute` rows are written **through `insert_declared_attribute`** and through no other path, and `SANCTIONED_SITES` gains exactly one entry — named, not a blanket exemption.

**Given** story 5.12's authorship gate
**When** `cargo xtask ci` runs
**Then** it stays green **because the new site is declared**, and reds if the write is moved outside it. ⚠️ The gate is a TRIPWIRE against a good-faith change, never a barrier — 5.12 measured its residual classes and this story does not widen the promise.

**And** the write carries a HUMAN author: NFR5's grammatical subject is the SCANNER, and an operator writing a declared value through an explicit action is a normal declarative write.

### Story 6.3: NFR5's two remaining assertions are measured, not asserted

As the next developer,
I want the never-overwrite invariant proven on the gesture that could break it,
So that the two assertions parked since Epic 5 stop waiting for a precondition that now exists.

**Acceptance Criteria:**

**Given** an observation and a documenting action over it
**When** the action completes
**Then** the observation record is **bit-for-bit unchanged** and its link is intact — NFR5's second assertion, which had no subject until this epic.

**Given** a declared field and an ingestion that contradicts it
**When** the scan runs
**Then** the declared field is unchanged and a divergence opens — NFR5's first assertion, already covered, **re-asserted here through the new write path** rather than assumed to survive it.

**And** each guard names the code path where the violation could be WRITTEN, not merely a path where it would be visible. ⚠️ **Epic 5's dominant defect class, named at its retrospective: *a guard placed where the defect cannot occur reads as coverage and is none.*** It was found nine times, never by reading.

### Story 6.4: The abstention line carries the gesture

As the operator,
I want to document an unplaced sighting from where I see it,
So that the reach section stops being a number and becomes a door.

**Acceptance Criteria:**

**Given** the identity section story 5.14b shipped
**When** a cause line is an `AbsenceOfProof` abstention
**Then** it carries the documenting action — **one line, one gesture, never N failures** (FR16b) — and the whole record is documented at once (**FR13(a)**, which the PRD calls *the day-one case*).

**Given** the `Ambiguous` cause
**When** it is displayed
**Then** it carries **no** documenting gesture: an ambiguity is a doubt to LIFT, not an entity to CREATE. ⚠️ **The operator's three cases (Guy, 2026-08-12)**: no ambiguity → the software decides; ambiguity → the operator lifts the doubt; unknown → the operator creates the entity. **The gesture belongs to the CAUSE, not to the count.**

**And** the UX bans still hold: the section does not redden, carries no gauge, and does not age. ⚠️ Story 5.14b left them **stated, not met** — the number still grows with scan count — and this story does not claim otherwise.

### Story 6.5: The entity supertype, the device, and the state column — schema only

As the next developer,
I want the tables a device grouping needs to exist before anything groups,
So that the schema is a decision of its own rather than a side effect of the first writer.

**Acceptance Criteria:**

**Given** story 5.9's three deferrals — *no `device`, no `entity` supertype, no `state` column* — deferred as one block
**When** this story ships
**Then** all three exist, with their binary collations (D64) and their adapter, and **no producer**: nothing writes a device yet.

**Given** the `state` column
**When** it is defined
**Then** it admits the lifecycle FR38b needs (`active`, `dormant`) as an enumerated domain in DDL, not a free string.

**And** it follows the 5.9 / 5.9b split exactly: **this story is the schema, story 6.12 is the resolver that fills it.** *(Guy's arbitration 2026-08-12: the supertype comes now, not later.)*

### Story 6.6: L2 candidate generation, and no rule

As the next developer,
I want the set of interface pairs that could be one device, computed by something that consults no rule,
So that a blocker cannot become the echo of the rule it feeds.

**Acceptance Criteria:**

**Given** a population of interfaces
**When** L2 candidates are generated
**Then** the result is a set of unordered pairs of distinct interfaces, and the generator **calls no `l2-*` rule and no `decide`** — story 5.6's rule, and its reason: *a blocker that consults a rule is that rule's echo.*

**Given** the committed trap corpus
**When** the L2 recall is measured
**Then** it is asserted against D13's floor in **milli-units** (`u32`), never a float — the `float-free` gate walks `identity/` and must stay green.

**And** the measurement is a real one: story 5.6 found that blocking on the MAC scores 700‰ and on the hostname 400‰, so **the recall assertion must be able to fail.**

### Story 6.7: `l2-different-hostname` — the first producer of `Opposes`

As the operator,
I want two interfaces whose hostnames disagree to argue against being one device,
So that the cascade gains its first opposing voice.

**Acceptance Criteria:**

**Given** a candidate pair whose hostnames are both present and differ
**When** the rule is evaluated
**Then** it yields `Verdict::Opposes` — **the first producer of that variant in this codebase** — and the three committed traps expecting `l2-different-hostname` are answered by it.

**Given** a pair where either hostname is absent or empty
**When** the rule is evaluated
**Then** it yields `Neutral`, never `Opposes`. ⚠️ **D20 names this as the common bug**: *"the rule that wrongly `Opposes` should return `Neutral` — it does not KNOW, it BELIEVES it knows; nine parasitic abstentions out of ten are that."* The `hostname-absence` family exists to catch it.

**And** the rule id is spelled exactly as the corpus spells it, or the trap reds as `rule_mismatch`.

### Story 6.8: `l2-uplink-agrees` — the first producer of `Supports`

As the operator,
I want two interfaces on the same switch port to argue for being one device,
So that the multi-NIC host stops counting twice.

**Acceptance Criteria:**

**Given** a candidate pair sharing an uplink
**When** the rule is evaluated
**Then** it yields `Verdict::Supports` — **the first producer of that variant** — and the two traps expecting `l2-uplink-agrees` are answered.

**Given** the pair from `multi-nic`'s `must-merge`
**When** the cascade decides
**Then** it merges, and **`decide`'s existing table is what combines the verdicts** — no new algebra, no sum, no float.

**And** the `vrrp-virtual-mac` trap's warning is respected: a shared uplink is *the temptation*, and story 6.11's structural anchor is what refuses it.

### Story 6.9: `l2-hostname-agrees`

As the operator,
I want two interfaces reporting the same hostname to argue for being one device,
So that the signal the sources do send is used.

**Acceptance Criteria:**

**Given** a candidate pair whose hostnames are present and equal
**When** the rule is evaluated
**Then** it yields `Supports`, and the trap expecting `l2-hostname-agrees` is answered.

**Given** the measured floor (FR9, NFR30)
**When** the rule's reach is documented
**Then** it states that **hostname is unusable on nearly half of known clients** — so this rule's silence is data availability, not engine weakness, and the abstention it leaves is honest.

**And** it does not contradict story 6.7: agreeing and differing are two rules, and the `hostname-collision` family exists because agreement is not proof.

### Story 6.10: `l2-different-switch`

As the operator,
I want two interfaces on different switches to argue against being one device,
So that geography counts as evidence.

**Acceptance Criteria:**

**Given** a candidate pair whose switches differ
**When** the rule is evaluated
**Then** it yields `Opposes`, and the trap expecting `l2-different-switch` is answered.

**Given** a pair where either switch is unknown
**When** the rule is evaluated
**Then** `Neutral` — D20 again.

**And** the rule reads only what an observation carries; it descends into no SQL (D10).

### Story 6.11: The virtual-MAC anchor — a structural fact that is NOT a rule

As the operator,
I want an IANA virtual-router address never to anchor a grouping,
So that a floating gateway is not folded into whichever master holds it this minute.

**Acceptance Criteria:**

**Given** an interface whose hardware address sits in an IANA redundancy-protocol range
**When** grouping is attempted
**Then** the pair is disqualified as a grouping anchor — **and this is a STRUCTURAL FACT READ AT INGESTION, not a rule that scores.** *(Guy's arbitration, 2026-08-12: **there is no rule**.)* D13 says it in those words: the U/L bit and the IANA prefixes are `Disqualifying` *"as GROUPING anchors, known at ingestion"*, and story 5.5 deliberately refused to implement it at L1 for exactly that reason — two committed tests pin the refusal.

**Given** that the corpus nevertheless expects the identifier `l2-virtual-mac-prefix`
**When** the decision is recorded
**Then** it **names `l2-virtual-mac-prefix` as what settled it** *(Guy's arbitration (a), 2026-08-12)*, because `Conclusion::NoMatch { rule }` requires a `RuleId` and D19 wants the id left behind — *a rule that fires without one is undebuggable*. ⚠️ **The corpus is NOT bumped.** And the story must say, where a reader meets it, **why a rule identifier names something that is not a rule** — otherwise the next reader takes it for an oversight.

**And** `vrrp-virtual-mac`'s two poles both pass: do not fold the VIP into its master, do not fuse the two bearers, **DO track the one virtual gateway across a failover** (D16's geometry).

### Story 6.12: The resolver writes device groupings

As the operator,
I want the engine's grouping decisions persisted,
So that a device is a record rather than a computation repeated at each page load.

**Acceptance Criteria:**

**Given** a population of interfaces and the L2 cascade
**When** the pass runs
**Then** it writes `device` rows and their memberships, in D13's order, and **every placement is justified by a rule id and its evidence** — never *"merged, with no explanation"*.

**Given** a second identical pass
**When** it runs
**Then** it writes nothing — idempotence, story 5.11's rule, and the same purge-and-replay invariant story 5.10 pinned at L1 must hold at L2.

**And** ⚠️ **two races registered at Epic 5 lose their shield here.** Two concurrent passes mint two interfaces for one MAC (`interface_l1_key` is a plain index and the mint is read-then-insert), and `current_subject IS NOT NULL` is not equivalent to `valid_to = OPEN_END`. Epic 5 recorded that *the connector story that gives it a MAC removes the shield*; **a device grouping that keys on interfaces reaches the same code.** This story carries them or names the story that does.

### Story 6.13: The first `Ambiguous` is persisted with its candidates

As the operator,
I want an ambiguity stored as data rather than as a hole,
So that *"I don't know"* can show its work.

**Acceptance Criteria:**

**Given** a verdict set that `decide` resolves to `Abstained { Ambiguous }`
**When** the link is written
**Then** `link_candidate` rows are written with it — **its first production writer since the table was created in story 5.9.**

**Given** `resolver::guard_decision`
**When** an `Ambiguous` carries no candidates
**Then** it is refused (`Constraint("ambiguity_without_candidates")`), and that refusal is now reachable through the ordinary path rather than only through a hand-built decision. ⚠️ Story 5.9b recorded the inverse danger: *the day a producer of `Ambiguous` arrives, this guard would refuse a LEGITIMATE ambiguity rather than let it be written with its candidates.* **This story is that day.**

**And** ⚠️ **story 5.14b's tripwire `the_production_pass_produces_no_ambiguous_abstention` REDS here, by design.** Its message names `epics.md`'s FR16 clause as the work that has come due. **Implement story 6.14; do not delete the assertion.**

### Story 6.14: The ambiguity explains itself on the page

As the operator,
I want an unresolved grouping to show its candidates and their evidence,
So that I can lift the doubt the engine refused to guess at.

**Acceptance Criteria:**

**Given** an abstention whose cause is `Ambiguous`
**When** it is displayed
**Then** its candidates and their evidence are shown **from the persisted `link_candidate` rows** (FR16) — the abstention explains itself. _(This is `epics.md`'s story-5.14 clause, re-owned to this epic at 5.14b's contexting with its unreachability ASSERTED; story 6.13 makes it reachable.)_

**Given** the operator's three cases
**When** the line is rendered
**Then** it carries the gesture that LIFTS A DOUBT — choose among candidates — and **not** the documenting gesture, which belongs to `AbsenceOfProof`.

**And** the count keeps the honest unit story 5.14b's arbitration 13 fixed, and ⚠️ **Epic 6 is where that unit stops being honest**: once grouping exists, *sighting* is no longer the true word, and the locale keys change with it. Registered at 5.14b as a scheduled consequence, not a correction.

### Story 6.15: The `must-abstain` traps are routed, and the release gate falls green

As the next developer,
I want every committed trap to be answered,
So that NFR4's gate measures the engine instead of measuring what the harness can reach.

**Acceptance Criteria:**

**Given** the trap corpus after stories 6.7 to 6.11
**When** the gate runs
**Then** the unanswerable bucket is **3, not 0** — the three `must-abstain` traps carry a CAUSE and no rule, so the `l2-` prefix selector cannot route them (story 5.7's measurement). **This story routes them.**

**Given** all 26 traps
**When** the gate runs
**Then** **`passed() == true`**: truth-table failures = 0 at the device level, unanswerable = 0. 🔴 **This is the first time the gate has been green since story 4.6b, and it closes NFR4 and milestone J4.**

**And** ⚠️ **an `Unanswerable` must still not be an abstention.** Story 5.8's whole point: map it to `Outcome::Abstained` and `example-must-abstain` passes because nothing was asked — D18's cowardice moved from the engine up to the harness. The routing must not reintroduce it.

### Story 6.16: The seeded generator and the bulk fixture, at reference scale

As the next developer,
I want a reproducible population of 300 hosts,
So that the gate's floor is measured at the scale NFR30 names instead of on a handful of traps.

**Acceptance Criteria:**

**Given** a fixed seed
**When** the generator runs twice
**Then** it produces the identical population — and the seed's PROVENANCE is guarded by a test that reads the constant's values. ⚠️ **Story 5.11b's finding, verbatim**: *reproducible WITHIN one process is trivially true for every seed; reproducible ACROSS runs is what a fixed seed buys, and only the second was ever at stake.* A clock-derived sweep left the whole suite green.

**Given** the reference scale (300 hosts, 36 subnets)
**When** the pass runs over it
**Then** the wall-clock is recorded, and the figure carries the machine it was measured on. ⚠️ Story 5.11b's timings rested on no check and differed between machines.

**And** the corpus lock is bumped deliberately and re-hashed: **the corpus is a locked SPEC** (Epic 4's third eyes-open item).

### Story 6.17: The distributional diff

As the operator,
I want to see how the population's shape changed between two runs,
So that a regression that shifts a distribution without failing a trap is still visible.

**Acceptance Criteria:**

**Given** two runs over the same seeded population
**When** their distributions are compared
**Then** the diff names what moved — and **the comparison is exact, not a threshold**: NFR4's reasoning applies here too, *any fraction is theatre* at this scale.

**Given** a run identical to its predecessor
**When** the diff runs
**Then** it reports no change, and a test plants a single moved observation to prove the diff can fail.

**And** it reads no clock: a diff whose output varies between two identical runs measures the clock.

### Story 6.18: The ephemeral-interface dormant lifecycle

As the operator,
I want a randomized address that has not been seen for a month to stop counting against reach,
So that the metric measures the network rather than the churn of privacy features.

**Acceptance Criteria:**

**Given** an interface whose hardware address is **locally administered — a structural fact read at ingestion, never an inference** — and unobserved for the configured window (default 30 days)
**When** the lifecycle pass runs
**Then** it moves to `dormant`: excluded from divergence metrics and from automatic candidate generation, **still queryable, retaining first/last-seen and address history indefinitely** (FR38b, FR38).

**Given** a dormant interface whose address is observed again
**When** the pass runs
**Then** it returns to active — **the same entity, not a new one.**

**And** the window is a parameter, never the clock read at the point of use: every instant this codebase compares is data-derived (D19), and story 5.11's `InstantRegressed` is the precedent for refusing a clock that runs backwards.

### Story 6.19: Observation history per device

As the operator,
I want to see when a device was first and last seen, and how its addresses moved,
So that a change has a date rather than a rumour.

**Acceptance Criteria:**

**Given** a device grouping several interfaces
**When** its record is read
**Then** it reports first-seen, last-seen and the IP↔MAC history (FR37), **derived from the observations** and never written by hand.

**Given** the seen-window derivation
**When** it is computed
**Then** it is a `min`/`max` over the observations, exactly as story 5.9b derived the interface's window — **the clock is never read**, and story 5.11b found that no order test touched those derived instants until one was written for them.

**And** the history survives a purge-and-replay: it is derived, so D14's *"a cache of attention, not of truth"* applies.

_**FR13(b) is NOT in this epic, and the decomposition briefly put it here by mistake.** A twentieth story — documenting a re-discovery field by field — was written and then removed the same day: **the FR coverage map already assigns it to Epic 7** (`FR13: E6 (minimal promote) / E7 — document (all/field)`), and Epic 7's own description carries *"document (all/field)"* as part of its triage inbox. The error was in the question, not the answer: the decomposition asked *"FR13(b) in the opening slice, or later?"* meaning later **within this epic**, while the plan had already placed it in the **next** one. 🔑 On the merits Epic 7 is also the right home: **field-selective documenting only has meaning once declared content exists and has drifted**, which is the re-discovery Epic 7's inbox triages. This epic ships FR13(a), the day-one case, which is what milestone J3 was missing. (Guy's arbitration, 2026-08-12.)_
