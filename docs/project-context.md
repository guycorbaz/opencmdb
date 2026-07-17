# opencmdb — Project Context

_Auto-loaded by BMad workflows. Concise grounding + resume point. Full detail lives in
`_bmad-output/planning-artifacts/`._

## Published (2026-07-17)

- **Repo (PUBLIC):** https://github.com/guycorbaz/opencmdb — branch `master`, first commit is planning artifacts + the Rust workspace skeleton. GitHub handle is `guycorbaz` (NOT `gcorbaz`, the system user).
- **Landing page (GitHub Pages):** https://guycorbaz.github.io/opencmdb/ — self-contained modern site on the `gh-pages` branch (single `index.html`, no CDN). Edit it there; `master` does not carry the site.
- **README.md** on `master` — English, honest about state (early dev; skeleton builds).
- **Docs/site are English-only** (locked rule); private network data is scrubbed from all artifacts (see [[no-private-data-in-artifacts]]).

## What it is

A self-hosted, single-binary **Rust** network reconciliation engine (lightweight IPAM + a light
application CMDB + network topology) for advanced home-labs and SMBs without a dedicated IT team.
**Core thesis:** continuously compare the **observed** state (auto-discovered) with the **declared**
state (documented by the operator); the *gap* between them is the product. Open-source, self-hosted
(Docker on Synology priority), distributed via Docker Hub.

## Planning status (2026-07-17)

Complete and saved in `_bmad-output/planning-artifacts/`:
- **Product Brief** (`product-brief-opencmdb.md` + `-distillate.md`), **competitive analysis**.
- **PRD** (`prd.md`) — 53 functional requirements (FR52 struck, number retained), 31 NFRs (NFR16 struck
  by D64, number retained), 7 user journeys, phased scope.
- **UX Design Specification** (`ux-design-specification.md`) — full 14-step spec.
- **Architecture** (`architecture.md`, ~4890 l.) — **complete, 8/8. Decision register D1–D65, feedback
  F1–F59.** Readiness: **NOT READY, and deliberately so** — the open gaps are named and countable.
  **Start at its Decision Index (§ near the top, 74 entries, line numbers verified): scan it BEFORE opening
  a question, not after.** Three times in 24h this register "discovered" a hole where its own rules were
  already standing (F56) — D57 sat on the critical path for a day while D25/D21 already answered it.
- **`architecture-views.md`** (~880 l.) — **CROSS-CUTTING VIEWS, derived, never edited by hand.** Gathers
  what the source scatters and nobody can reconstruct by scanning: **every named renunciation, every
  measured number, every recorded dissent, every author amendment, every piece of named theatre, everything
  still open.** It POINTS, it does not restate. **It carries `sourceSha256` — if it no longer matches
  `architecture.md`, the file is STALE and must not be trusted.** Never apply a decision from it.
- Theme visualizer: https://claude.ai/code/artifact/b598a17b-5303-4c32-bb58-f7a79fbb8182

> **⚠️ Read `prd.md`'s `editHistory` frontmatter for the true state of the requirements — NOT the F-tables
> inside `architecture.md`. Start any question at the Decision Index near the top of `architecture.md`
> (75 entries, line numbers verified) — scan it BEFORE opening a question, not after (F56).**

**Planning is DONE. All architecture decisions are closed (D1–D66). What remains is IMPLEMENTATION
SEQUENCING, in order:**
1. **Write `Cargo.toml` + the workspace skeleton** (D47/D55/D56b). Deps chosen and versioned by **D66**:
   MariaDB-only sqlx 0.9.0, axum 0.8.9, askama 0.16, `rust-i18n` (YAML), the `config` crate, `prometheus`
   (raw), distroless/static:nonroot. Pin from the real `Cargo.lock`, commit it, build `--locked`. Confirm
   at `cargo add`: the sqlx `Migrate`-trait surface (D23), the exact `tls-rustls-ring` feature name.
2. **The two D65 gates in `cargo xtask ci`** — AFTER step 1 (they are Rust in `xtask`, which needs the
   workspace). ~40 lines, specified and simulated green. Add a third check: `architecture-views.md`
   `sourceSha256` vs the real source.
3. **D64 condition 1** — the DDL binary-collation grep, written **with the first migration**.
4. **Story 1** — the walking skeleton that shows a REAL gap on a cardinality-1 perimeter and abstains,
   visibly, everywhere else. **Bomb to mind: `Reads` as a single trait does not compile (two traits); and
   `query*()` takes `impl SqlSafeStr` in sqlx 0.9 (dynamic SQL needs `AssertSqlSafe`).**
5. **Epics LAST** — F40: the FR-domain → directory table is a **map of the code**, never a list of epics.
   **Epics are vertical slices crossing several rows.**

> **Note on `architecture-views.md`:** it is one decision behind (its hash reflects pre-D66) — **by design,
> the mismatch IS the staleness signal.** Regenerate it at the next MILESTONE (after steps 1–4 settle), not
> per-decision — a views file is regenerated at milestones, like the Cargo.lock is committed at a known
> state, not on every edit.

## Locked, non-negotiable decisions (do not reopen)

- **Language/runtime:** Rust, single binary.
- **Database:** **MariaDB 10.11+ is the ONLY supported engine** (Synology-native, included in DSM's
  automatic backups). CI is pinned to `mariadb:10.11.11` — the exact DSM 7 package — so **dev = CI =
  prod**. **SQLite: NOT supported, and it is a REFUSAL, not a deferral ("SQLite later" is banned in
  writing). MySQL: NOT supported** — a different product from MariaDB, and we do not claim what has no
  CI. **PostgreSQL: not supported at MVP**; re-opened as a *possible* future addition, and it is not
  free — the repository trait gets audited BEFORE any such port. **Comparison and normalization never
  descend into the engine** (a correctness rule, not a portability one: identity is the product), held
  by binary collation on every text column + a CI grep over the DDL.
  _(D64, 2026-07-17. This line previously read "SQLite (small) and MySQL/MariaDB (larger) … SQLite-first,
  MariaDB activated later within MVP" — **stale twice over**: D1 had already made MariaDB day-1.)_
- **Deployment:** Docker (Synology Container Manager) priority; native binary also supported.
- **Discovery:** first-class **zero-privilege UniFi connector** + generic ARP/ping scanner
  (NET_RAW → ping-only fallback). Connectors isolated behind a Rust `Connector` trait, contract-tested.
- **Web stack:** HTMX + Askama templates + Tailwind (standalone CLI, no Node; assets via `rust-embed`,
  generated CSS committed). **Polling, not SSE, at MVP.** Internal tokio scheduler (no cron/Redis/workers).
- **UI:** bilingual EN/FR; **docs English-only**. Dark theme default. WCAG 2.1 AA on key views.
- **Audience:** unified (advanced home-labber = SMB-without-IT). Single admin at MVP;
  **multi-user-ready schema** from day 1 (multi-user role = Growth).
- **Delivery:** phased MVP / Growth / Vision. Reference scale: ~300 hosts / 36 subnets on an
  x86 Synology Plus-class NAS.

## Key design pillars for the Architecture phase

- **Composite device identity** (MAC + hostname + IP/DHCP history + connection topology; service
  fingerprint = Growth) — NOT raw MAC. This is the highest-leverage/riskiest problem; precision/recall
  on labeled fixtures gates release.
- **Linked-never-merged** observed/declared model — reconciliation *links*, never overwrites declared.
- **Commit state machine:** `in_queue → pending_commit(server deadline) → committed | failed`;
  **Undo returns to `in_queue` — it is not a failure branch** (*an undo is not a failure: the operator
  changed his mind*). A scan touching a `pending_commit` item is quarantined (`superseded_by_pending`);
  the **server timer is authoritative**; transitions serialized per `item_id`; idempotency via 409/ETag.
  _(Vocabulary corrected 2026-07-17 — F59. This line carried `Accept-as-declared`, `pending_accept` and
  `reverting`: all three RETIRED on 2026-07-16, and `accept-as-declared` is on the denylist. The state was
  named after ONE gesture when the protocol covers document / accept-gap / exclude alike.)_
- **Triage gestures (canonical, binding):** **document** (EN) / « Merger » (FR UI) — closes the gap, field
  by field, and carries the amber accent · **accept-gap** / « Accepter l'écart » — *seen, not yet decided*;
  the gap stays OPEN and it is deliberately NEUTRAL, never amber · **exclude** / « exclure » · attach ·
  create · snooze. **Retired, denylisted: `accept-as-declared`, `revert`, `ignore`, and `merge` in ENGLISH
  only** (the pillar *linked, never merged* is intact; the FR UI verb stays « Merger »). *You document a
  VALUE; you accept a GAP.*
- **Orthogonal `source_state ∈ {live, blind}`:** when a source is blind, freeze last-known state,
  suppress observation-derived alerts, and **never fabricate divergences**.
- **Optimistic UI** (client-instant, server-reconciled) with **explicit focus management on every
  HTMX swap** (accessibility requirement #1).
- **Security/threat model:** protect against a stolen DB/backup + unauthenticated network access
  (not local root); encryption key kept separate from the DB volume; API-key rotation + encrypted
  secret backup at MVP; TLS via reverse proxy (deployment concern).

## Working conventions

Guy works in **French**; decisive; uses BMad **Party Mode** heavily at each step; guiding mantra
**"on affinera à l'usage"** — freeze durable *principles*, calibrate *details/values* in V1.
