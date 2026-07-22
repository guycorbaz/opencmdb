# opencmdb — Project Context

_Auto-loaded by BMad workflows. Concise grounding + resume point. Full detail lives in
`_bmad-output/planning-artifacts/`._

## Published (2026-07-17)

- **Repo (PUBLIC):** https://github.com/guycorbaz/opencmdb — branch `master`, first commit is planning artifacts + the Rust workspace skeleton. GitHub handle is `guycorbaz` (NOT `gcorbaz`, the system user).
- **Landing page (GitHub Pages):** https://guycorbaz.github.io/opencmdb/ — self-contained modern site on the `gh-pages` branch (single `index.html`, no CDN). Edit it there; `master` does not carry the site.
- **README.md** on `master` — English, refreshed 2026-07-22 to match reality: it had still claimed
  *"the code has just begun… a skeleton"* and *"there is no runnable product to deploy"* while
  `v0.1.0`/`v0.1.1` were tagged and on Docker Hub. It now states what runs (one page, one connector,
  a real gap on a small perimeter), what does NOT (triage inbox, IPAM, UniFi, alerts, admin UI), and
  that ~1/5 of the planned work is done. ⚠️ **Still outstanding: there is no `LICENSE` file** —
  AGPL-3.0-or-later is declared only in the `Cargo.toml`s. The README now says so plainly.
- **Docker Hub image is `gcorbaz/opencmdb`** — the SYSTEM-user handle, *not* the GitHub handle
  `guycorbaz`. The two differ; do not "correct" either one into the other.
- **`docker/docker-compose.yml` and `docker/.env.example` DO exist** (under `docker/`, not at the
  repo root — an `ls` at the root misses them, which produced a false "the compose file is missing"
  claim on 2026-07-22).
- **Docs/site are English-only** (locked rule); private network data is scrubbed from all artifacts (see [[no-private-data-in-artifacts]]).

## What it is

A self-hosted, single-binary **Rust** network reconciliation engine (lightweight IPAM + a light
application CMDB + network topology) for advanced home-labs and SMBs without a dedicated IT team.
**Core thesis:** continuously compare the **observed** state (auto-discovered) with the **declared**
state (documented by the operator); the *gap* between them is the product. Open-source, self-hosted
(Docker on Synology priority), distributed via Docker Hub.

## Where the code is (2026-07-22) — READ THIS FIRST

**Planning is done and the code is well past the skeleton.** `v0.1.1` is tagged and released to
Docker Hub; the binary runs and scans on Guy's NAS (frontend on macvlan, no Traefik).
⚠️ The NAS is still on the `0.1.0` image.

| Epic | State |
|---|---|
| **E1** Les gates tiennent | ✅ done (6 stories) |
| **E2** Le contrat de connecteur | ✅ done (5 stories) |
| **E3** Mon premier écart réel — **v0.1** | ✅ done (10 stories), retrospective held |
| **E4** Infra fixtures & corpus de pièges — v0.2 | 🚧 **in progress — 6 of 19** (4.1, 4.2, 4.3, 4.4, 4.5a, 4.5b done; **4.6 is next**) |
| **E5–E23** | backlog |

Live status is `_bmad-output/implementation-artifacts/sprint-status.yaml`, not this file.

**What exists today:** a three-crate workspace that builds and ships. `cargo xtask ci` runs four real
gates — dependency frontier (D47), DDL binary collation (D64), retired vocabulary (D65), and the
fixture corpus lock (both directions: edited AND orphan). `opencmdb-core` holds the domain
(`Observation`, `Fact`, `Capabilities`, the closed `ConnectorError` taxonomy, the `Connector` trait
and its consumer-driven contract test). `opencmdb-bin` holds everything touching the outside world:
MariaDB pool + migrations, axum/askama/HTMX pages, an ARP/ping connector, the fixture reader and
`FixtureConnector`. Test counts as of this commit: **86 (bin) + 46 (core) + 38 (xtask)**.

**The fixture corpus is the current centre of gravity.** `fixtures/` at the workspace root is a
SPEC, not test data, locked by `MANIFEST.toml` in both directions. A replay stream carries
observation lines (`obs_id`) and control records (`record`) — `failure` ends a poll with a
`ConnectorError`, `capability` changes the source's descriptor mid-stream and the poll continues.
Epic 4 builds the metrics harness and the trap corpus **before** the identity engine, on purpose:
*"a metric written after the engine is bent to fit the engine."*

**Two standing lessons, learned the hard way and worth carrying:**
- **Name the test or command behind every claim.** Four consecutive completion records over-claimed;
  reviews caught each. Write the weaker true sentence instead.
- **A comment asserting a checkable property gets checked.** Story 2.2 shipped a comment saying a
  guardrail existed; it did not, and the claim survived until story 4.5a's review added a variant and
  watched the build succeed. `deferred-work.md` is the register — **append to it, never rewrite a
  bullet**.

**Issue tracking:** GitHub Issues on `guycorbaz/opencmdb` is the single source of truth for bugs and
change requests outside the BMad story flow. Open at this date: #1 (Docker connector), #2 (frontier
gate blind spots), #3/#4 (scan CIDRs from the UI), #11 (scan tuning in the UI), #12 (no admin
surface), #13 (QR-code equipment labels, Brother QL-820NWBc — post-v1.0).

## Planning status (2026-07-17)

Complete and saved in `_bmad-output/planning-artifacts/`:
- **Product Brief** (`product-brief-opencmdb.md` + `-distillate.md`), **competitive analysis**.
- **PRD** (`prd.md`) — 53 functional requirements (FR52 struck, number retained), 31 NFRs (NFR16 struck
  by D64, number retained), 7 user journeys, phased scope.
- **UX Design Specification** (`ux-design-specification.md`) — full 14-step spec.
- **Architecture** (`architecture.md`, **5123 l.** as of 2026-07-22) — **complete, 8/8. Decision register D1–D66, feedback
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

**Planning is DONE. All architecture decisions are closed (D1–D66).**

_The five-step implementation sequencing this section used to carry (workspace skeleton → the D65
gates → D64's DDL grep → story 1 → epics) is **all complete**, through Epic 3 and the v0.1 release.
It was removed on 2026-07-22 rather than left standing: a resume point that describes work finished
weeks ago sends the next reader to the wrong place. See "Where the code is" at the top for the real
one. The two sqlx traps it warned about were both real and are both handled in the shipped code:
`Reads` is two traits, and `query*()` takes `impl SqlSafeStr` (dynamic SQL needs `AssertSqlSafe`)._

> **Note on `architecture-views.md`:** it is STILL stale, deliberately — `cargo xtask ci` reports
> `ℹ views-hash STALE` and exits 0, because the mismatch IS the staleness signal. Regenerate it at a
> MILESTONE (the end of Epic 4 is the natural one), not per-decision or per-story. **Do not
> regenerate it inside a story**; several story files say so explicitly.

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

**Story flow, as actually practised:** `create-story` → (optionally `validate`, which is worth it —
a fresh-context adversarial pass on story 4.5 found eleven defects the author could not see) →
`dev-story` → `code-review` → next story. Stories are sliced FINE: prefer many small ones over few
big ones, and split a story when its halves turn out not to be variants of one idea (4.5 → 4.5a/4.5b).

**Commits:** one per story, landing the story file AND its review together, directly on `master`
(no PR flow). Message names what changed and what was measured. Run the full local gate before
pushing — `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, `cargo xtask ci` — because Epic 3's retrospective recorded four CI-only
failures from skipping exactly that. ⚠️ `DATABASE_URL` is usually unset locally, and the
MariaDB-backed tests `return` early and pass either way: a green suite says **nothing** about the
database.
