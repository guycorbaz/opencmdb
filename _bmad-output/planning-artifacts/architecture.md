---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
lastStep: 8
status: 'complete'
completedAt: '2026-07-16'
lastEdited: '2026-07-17'
readiness: 'NOT READY — 2 critical gaps open, named and countable. Confidence HIGH. See Architecture Readiness Assessment (written when 4 were open; D31 closed at validation, and the probe + the PRD/UX pass closed the other two on 2026-07-16 evening).'
openCriticalGaps:
  - 'NONE that blocks Cargo.toml. The four crate selections + the sqlx 0.9 verification are CLOSED by D66 (2026-07-17). What remains is IMPLEMENTATION SEQUENCING, not open decisions — see remainingBeforeStory1.'
remainingBeforeStory1:
  - '(a) Write Cargo.toml + the workspace skeleton (D47/D55/D56b). Pin every version from the real Cargo.lock, commit it, build --locked. Confirm at cargo add: sqlx migrate! Migrate-trait surface (D23), the exact tls-rustls-ring feature name, config crate ergonomics.'
  - '(b) The two D65 gates in `cargo xtask ci` — they are Rust in xtask, which needs the workspace, so they come AFTER Cargo.toml (the memory once had this backwards; corrected). ~40 lines, specified and simulated green. Add a third check: architecture-views.md sourceSha256 vs the real source.'
  - '(c) D64 condition 1 — the DDL binary-collation grep, written WITH the first migration.'
  - '(d) STORY 1 — the walking skeleton (D1-rev). MIND THE BOMB: `Reads` as a single trait DOES NOT COMPILE (ReadRepository is &self, Unit<''u''> is &mut self, core cannot name sqlx::Executor). D49''s prose is false in the singular — TWO traits delegating to a generic free function in bin. And: query*() now takes impl SqlSafeStr (sqlx 0.9) — dynamic SQL needs AssertSqlSafe.'
closedAfterCompletion:
  - 'D66 (2026-07-17) — the four crate selections + the sqlx 0.9 verification, done on WEB-VERIFIED facts (not recalled — training runs to Jan 2026). i18n = rust-i18n/YAML (greppable, D39); config = the `config` crate (boring; the invariants are ours); /metrics = raw `prometheus` + our handler (Guy''s call, no middleware magic); Docker base = distroless/static:nonroot (Guy''s call — CA certs + tzdata + nonroot, and Murat''s image-scan-is-theatre verdict HOLDS). sqlx: migrate! folder/table UNCHANGED (calendar risk dissolved), but query*() now takes impl SqlSafeStr (a CODE fact). CLOSES the last gap before Cargo.toml.'
  - 'D57 — CLOSED 2026-07-17, and it closed almost empty. Representation was already decided by D21/D3/D25 (a closure table is a cache that outlives its batch); traversal REFUTED BY MEASUREMENT (a recursive CTE cannot express a visited-set: fast+wrong / correct+featureless / correct+exponential, no fourth option — and the wrong one PASSES NFR2 with 40x margin); the cycle invariant does not exist (a visited-set makes it moot). What survived that D57 never contained: the D15 anchor + AC-M-04, and F55 (the four verbs are not one relation). SCOPE decided by Guy: John''s split — FR26/27 MVP, FR28 splits, FR29 at MVP = FR39 renamed "Hosted here", the graph and the traversal go to Growth. At MVP there is NO GRAPH AT ALL. F55-F58 raised.'
  - 'D57 — CLOSED 2026-07-17, and it closed almost empty. Representation was already decided by D21/D3/D25 (a closure table is a cache that outlives its batch); traversal REFUTED BY MEASUREMENT (a recursive CTE cannot express a visited-set: fast+wrong / correct+featureless / correct+exponential, no fourth option — and the wrong one PASSES NFR2 with 40x margin); the cycle invariant does not exist (a visited-set makes it moot). What survived that D57 never contained: the D15 anchor + AC-M-04, and F55 (the four verbs are not one relation). SCOPE decided by Guy: John''s split — FR26/27 MVP, FR28 splits, FR29 at MVP = FR39 renamed "Hosted here", the graph and the traversal go to Growth. At MVP there is NO GRAPH AT ALL. F55-F58 raised.'
  - 'D31 — crypto crate: CLOSED at validation. Hybrid, decided by a fact: the DEK wrap needs an AAD and the age format exposes none.'
  - 'The unmeasured assumption — CLOSED by the cardinality probe: single-interface comfortably above the 80% threshold. It holds.'
  - 'F42-F51 — APPLIED to prd.md and ux-design-specification.md on 2026-07-16 evening, verified against the sources.'
  - 'D64 (2026-07-17) — SQLite OUT for good, MySQL OUT, MariaDB 10.11+ single engine. Decided by Guy after party mode (Winston/Murat/John). REVOKES the locked non-negotiable and D23/D46/D46b; reshapes D48/D49/D52/D57; AMENDS D51 point 4 (Winston) and D45''s D1 row (Murat). The debate is CLOSED — (B) "SQLite later" is banned in writing. F52-F54 raised against the PRD and the brief.'
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/ux-design-specification.md
  - _bmad-output/planning-artifacts/product-brief-opencmdb.md
  - _bmad-output/planning-artifacts/product-brief-opencmdb-distillate.md
  - _bmad-output/planning-artifacts/competitive-analysis.md
  - docs/project-context.md
  - docs/brief-initial-bmad.md
sourceDocumentState:
  note: 'ALL F-items (F1–F59) are APPLIED. Nothing in this document is outstanding feedback. 31 F-rows remain, and the count is worth being exact about because a vague count is how the last snapshot was misread: of F1–F41, the 28 that NO decision referenced were REMOVED on 2026-07-17 and 13 survive because other decisions CITE them (they are rules, not leftovers) — plus the 18 rows of F42–F59, which are recent, applied, and left intact. 13 + 18 = 31.'
  rule: 'This document records what was DECIDED. `prd.md` records what the requirements ARE. For the state of any requirement, read prd.md''s editHistory frontmatter — never a table in here.'
  prd.md: 'F1–F59 APPLIED (see its editHistory: three entries, 2026-07-16 ×2 and 2026-07-17 ×2)'
  ux-design-specification.md: 'APPLIED (see its editHistory: 2026-07-16 and 2026-07-17)'
  outstanding: 'none'
workflowType: 'architecture'
project_name: 'opencmdb'
user_name: 'Guy'
date: '2026-07-16'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Decision Index — D1 to D65

_Added 2026-07-17. **This index exists because of F56**: three times in 24 hours, this document's own
authors "discovered" a hole where its rules were already standing — D57 sat on the critical path for a day
while **D25 and D21 already answered it**. The register outgrew the working memory of the people writing
it. **A 4 800-line graph with no index is not a register, it is an archive.**_

_**How to use it: scan this before opening a question, not after.** If a decision below sounds like it
touches your question, it probably answers it. The most-cited nodes are **D1** (46×), **D45** (34×),
**D21** (25×) — if your question touches identity, gates or the schema, those three have almost certainly
spoken already._

_**This index is a MAP, never a source.** It carries titles, not rules. **No decision may be applied from
its one-liner** — every one of them is a compression of an argument that was attacked, and the argument is
the part that makes it hold. For the state of any REQUIREMENT, read `prd.md`'s `editHistory` — never here,
never a table in this file (that mistake has been made three times)._

| # | Decision | Line |
|---|---|---|
| **D1** | MariaDB is a day-1 production requirement | 264 |
| **D2** | "Merge" is selective field adoption, never a destructive merge | 276 |
| **D3** | Field-level provenance on the declared record | 305 |
| **D4** | Doubt is a DERIVED predicate, never a persisted state | 335 |
| **D5** | FieldDecision: a cache of ATTENTION, not of TRUTH | 369 |
| **D6** | Two gestures, one lifecycle | 406 |
| **D7** | Three operating regimes, DERIVED not declared | 439 |
| **D8** | The backlog is the user's decision | 470 |
| **D9** | Presence requires explicit hysteresis | 497 |
| **D10** | Testing decisions that gate the architecture | 509 |
| **D11** | Vocabulary | 573 |
| **D12** | The three-level model: observation -> interface -> device | 879 |
| **D13** | Matching engine: option C, with the rule cascade as the verdict function | 929 |
| **D14** | The link is an ENTITY (SCD2), not a foreign key | 1013 |
| **D15** | Identity migration: declared_attribute.entity_id is NEVER updated | 1051 |
| **D16** | virtual_device returns to MVP (reversal) | 1101 |
| **D17** | dormant: the lifecycle of ephemeral interfaces | 1152 |
| **D18** | The release gate: Tier 1 binary, three columns | 1208 |
| **D19** | The fixture IS the Connector trait | 1267 |
| **D20** | Evidence strength: conservative table assumed, under a four-condition ADR | 1348 |
| **D21** | Three flags resolved by the reframe | 1401 |
| **D22** | The gap: DERIVED divergence + PERSISTED reconciliation item (hybrid) | 1489 |
| **D23** | Migrations: per-dialect DDL, sqlx::migrate! wrapped in our own sequence | 1512 |
| **D24** | Retention: FR38 + D17; the temporal-history growth question stays OPEN | 1531 |
| **D25** | Caching: NONE. Explicitly. | 1541 |
| **D26** | NFR9 splits into 9a / 9b / 9c: the threat model must be enforceable | 1552 |
| **D27** | Key provisioning: a KEK file in a SEPARATE DSM shared folder | 1596 |
| **D28** | Envelope encryption (KEK/DEK). KEK rotation only at MVP. | 1647 |
| **D29** | Tokens hashed (SHA-256); Argon2id for passwords only | 1705 |
| **D30** | Sessions: DB-backed, reusing the deadline_at + sweep pattern | 1727 |
| **D31** | Crypto crate: OPEN (age vs pure RustCrypto) | 1739 |
| **D32** | source_state is TWO ORTHOGONAL AXES, not three states | 1815 |
| **D33** | ConnectorError: a closed taxonomy. Never anyhow. | 1857 |
| **D34** | Three forced corrections to the Connector trait | 1917 |
| **D35** | NFR7 is a TYPE, not a test. NFR8's "does not crash" is an admission. | 2002 |
| **D36** | Dynamic capability vindicates the verdict vector | 2057 |
| **D37** | The JS asset pipeline: committed, rust-embed'd, version-pinned. Never a CDN. | 2136 |
| **D38** | Focus management lives in a committed app.js, not scattered in hyperscript | 2153 |
| **D39** | i18n: the format must be greppable and diffable | 2176 |
| **D40** | Askama organisation | 2187 |
| **D41** | FR52 (opt-in telemetry) is OUT of the MVP. Unanimous, by three independent routes. | 2200 |
| **D42** | If FR52 ever ships: the conditions, written now so they are not debated later | 2332 |
| **D43** | The 3-month goal is rewritten: integers, not percentages | 2377 |
| **D44** | The remaining infrastructure decisions | 2416 |
| **D45** | The gate criterion (governing rule, adopted project-wide) | 2458 |
| **D46** | The dual-backend harness: dual!(fn_name) — the macro takes a NAME, not a body | 2481 |
| **D46b** | The verdict join: without it, D1 is a slogan | 2555 |
| **D47** | The anyhow frontier IS the dependency graph, not a rule | 2584 |
| **D48** | Opaque identifiers: CHAR(36) ascii_bin / TEXT COLLATE BINARY. BINARY(16) refused. | 2658 |
| **D49** | Repository: transact(closure), opaque Unit<'u>, no commit() | 2732 |
| **D50** | IdentityIndex<'u> borrows the unit: borrowck enforces D25's red line | 2785 |
| **D51** | MemoryRepository REFUSED — and one bug class assumed without a gate | 2821 |
| **D52** | Server version floor, asserted at startup | 2913 |
| **D53** | The orphan rule: newtypes in bin, http_status() in core | 3004 |
| **D54** | core is organised BY SUBDOMAIN — and the folder is not the gate | 3112 |
| **D55** | The CARGO_MANIFEST_DIR cluster; and xtask css, never build.rs | 3168 |
| **D56** | fixtures/ at the root; xtask/ a member and nobody's dependency | 3227 |
| **D56b** | A test lives in the lowest crate that can see everything it needs | 3289 |
| **D31 (cont.)** | CLOSED: hybrid. RustCrypto for the wrap, age for FR48 only. | 3659 |
| **D32-amended** | Live + Reduced is GREEN with a scope label. Never orange. | 3764 |
| **D11 (cont.)** | ALREADY CLOSED on 2026-07-16, in the PRD. Not reopened. ignore → exclude is the only remainder. | 3796 |
| **D58** | The product has the right to TELL. The six-month test gains its symmetric. | 3839 |
| **D1-rev** | Story 1 is the walking skeleton THAT SHOWS A REAL GAP | 3884 |
| **D19-rev** | The real was always an upstream dependency; it was listed as the last step | 3906 |
| **D57** | The impact graph (FR26-29): representation and traversal REQUIRED before the schema grows | 3934 |
| **D59** | The reference scale is a PRODUCT TARGET, not the developer's install. Nobody had checked. | 4181 |
| **D60** | The UniFi capability descriptor, MEASURED. D34's capabilities() filled in with reality. | 4220 |
| **D61** | l2_domain = network_id. The developer's network cannot verify it. The question stays OPEN, and it is now NAMED. | 4250 |
| **D62** | The UniFi version matrix: the real axis. D35's "3.x and 4.x" names nothing. | 4276 |
| **D63** | D27 binds xtask too. The probe violated it, and the violation is not an anecdote. | 4314 |
| **D64** | SQLite is OUT, for good. MySQL is OUT. MariaDB 10.11+ is the only supported engine. | 4363 |
| **D57 (cont.)** | CLOSED. Representation was never open; traversal was decided by MEASUREMENT; the cycle invariant does not exist. | 4592 |
| **D57-scope** | DECIDED by Guy: John's split. The MVP ships declarations, never beliefs. | 4710 |
| **D65** | The two open gates, grouped into cargo xtask ci. And the second one is not the gate anyone proposed. | 4784 |
| **D66** | Crate selections. Three forced by constraints already written, one real tradeoff. | 4903 |

## Project Context Analysis

### Requirements Overview

**Functional Requirements (53 FRs, 9 domains):** opencmdb is an event-driven
reconciliation engine, not a CRUD web app. The FRs cluster into architecturally
distinct subsystems: Discovery & Sources (FR1-8) behind a `Connector` trait;
Reconciliation & Composite Identity (FR9-20) as the temporal core; IPAM (FR21-25);
Applications & Impact graph (FR26-29); Alerts with stable deep-links (FR30-35);
Insight/History/Retention (FR36-39); Data Lifecycle (FR40-42); read-only JSON API +
authenticated Prometheus `/metrics` (FR43-44); Admin/Security/Ops (FR45-52); and
structured Topology (FR53). Cross-cutting constraint: the data model and auth are
multi-user-ready from day one though a single admin ships at MVP.

**Non-Functional Requirements (30 NFRs) — the architecture drivers:**

- **Portable SQL layer (NFR15/16/21):** one identical suite green on SQLite AND
  MariaDB; WAL + `busy_timeout` + serialized writer; no backend-specific dialect.
  PostgreSQL out at MVP.
- **Reconciliation correctness (NFR4/5/6/7):** precision/recall on a labeled fixture
  set gates release; never-overwrite invariant with anti-regression test; idempotent
  and ingestion-order-independent; 0 false "device-gone" under fault injection.
- **Connector resilience (NFR8/23):** UniFi behind a trait, contract-tested against
  version-tagged fixtures, graceful degradation on API drift.
- **Security & threat model (NFR9-13):** protect against a stolen DB/backup and
  unauthenticated network access (not local root); encryption key kept separate from
  the DB volume; all HTTP surfaces authenticated; TLS is a reverse-proxy concern.
- **Durability & upgrade (NFR14/17/19):** backup/restore round-trips (SHA-256 + row
  counts); versioned, idempotent, resumable migrations with auto-backup, verified on
  both backends; bounded (<30 s) update downtime.
- **Footprint (NFR18/20/27):** single binary + Docker (Synology x86 priority),
  <=~512 MB RSS, <5 s cold start, zero external services.
- **UX-driven (NFR2/24/25/26):** p95 <= 1.5 s primary views under WAL load;
  responsive (360/768/1280); WCAG 2.1 AA on key views with focus-management as the #1
  gate; bilingual EN/FR, all strings externalized.

**UX architectural implications:** server-rendered HTMX + Askama + Tailwind
(standalone CLI, committed CSS, `rust-embed` assets), no JS framework. Reactivity via
HTMX fragment polling at MVP (SSE a clean later upgrade). Optimistic UI with morph
swaps; forms live outside polled fragments; explicit focus management on every swap.
The commit state machine (`in_queue -> pending_commit(server deadline) -> committed | failed`;
**Undo returns to `in_queue`, it is not a failure branch**) with per-item serialization,
server-authoritative timer, and 409/ETag idempotency is both a UX and a backend contract. `source_state in {live, blind}` is an
orthogonal axis. Stable canonical deep-link URIs per entity/alert (UUID-keyed);
`external_base_url` configurable with tolerant fallback.

### Scale & Complexity

- **Primary domain:** self-hosted, event-driven network-reconciliation engine
  (backend-core-heavy; web UI + read-only JSON API + Prometheus `/metrics` as surfaces).
- **Complexity level:** HIGH (technical) — dual-backend test matrix, unofficial UniFi
  API drift, composite device identity under MAC randomization / multi-NIC /
  shared-hardware VMs, temporal linked-never-merged model with a concurrency-safe
  accept state machine.
- **Scale target:** single operator; reference dataset 300 hosts / 36 subnets on an
  x86 Synology Plus-class NAS (~4-core / 8 GB). Not enterprise-scale.
- **Estimated major architectural components (~10):** Connector layer (trait + UniFi +
  generic scanner), Discovery scheduler, Reconciliation/identity engine, Portable
  persistence layer, Domain model (observed/declared/IPAM/apps graph), Alert &
  notification (webhook + deep-link), Web/HTTP surface (HTMX+Askama), JSON API +
  `/metrics`, Auth & secret management, i18n + telemetry.

### Technical Constraints & Dependencies

Locked, non-negotiable: Rust single binary; SQLite + MySQL/MariaDB only (no
PostgreSQL at MVP); Docker/Synology Container Manager priority + native binary;
zero-privilege UniFi connector + generic ARP/ping scanner (NET_RAW -> ping-only
fallback) isolated behind a `Connector` trait; HTMX + Askama + Tailwind standalone CLI
with committed CSS and `rust-embed`; polling (not SSE) at MVP; internal tokio scheduler
(no cron/Redis/workers); bilingual EN/FR UI, English-only docs; dark theme default.

**Amended this session — MariaDB is a day-1 production requirement** (see Decision
Register D1), superseding the PRD's "SQLite-first, MariaDB activated later within MVP"
sequencing.

Key external dependency risk: the unofficial UniFi local API shifts across UniFi OS
versions — contained by the trait, contract tests, and defined degradation.

### Cross-Cutting Concerns Identified

Portable SQL abstraction (both backends, one suite); serialized writer vs scan
throughput tension (NFR1<->NFR16); composite identity as the precision/recall gate; the
accept state machine (per-item serialization, server-authoritative timer, 409/ETag
idempotency, pending-accept quarantine); orthogonal source live/blind state (never
fabricate divergences); secret encryption with an out-of-band key; UUID-keyed deep
links + `external_base_url`; i18n externalization; optimistic UI + focus management
across every HTMX swap; resumable versioned migrations; opt-in telemetry.

**Newly surfaced this session (not in the PRD/UX):**

- **Field-level provenance and selective adoption** on the declared record — see D2/D3.
- **Doubt as a derived predicate** with a named-cause enum — see D4.
- **Presence hysteresis** (absence of proof != proof of absence) — see D9.
- **Three operating regimes** (bootstrap / steady / migration), derived not declared —
  see D7.
- **Collation as a correctness risk**, not a dialect risk: SQLite is case-sensitive by
  default, MariaDB case-insensitive — hostname matching (an identity anchor) would
  differ per backend. All value comparison/normalization happens in Rust, never in SQL.
- **Scan reach is an L2/L3 topology assumption**, not merely a NET_RAW permission: a
  NAS cannot ARP 36 subnets without being trunked on each. To be documented as a
  deployment prerequisite or NFR1 is unattainable for many users.
- **Retention/growth of temporal history** (WAL growth, checkpoint/VACUUM over months)
  — composite identity depends on IP/DHCP history, which accumulates.

### Reframing of the NFR1 <-> NFR16 Tension

The PRD frames this as "write throughput vs scan speed". That framing is wrong. The
discovery+diff cycle is dominated by network I/O (ARP/ping over 36 subnets), not by
SQLite. 300 upserts in one batched transaction is milliseconds; the serialized writer
only becomes a problem if writes are per-host (300 micro-transactions each hitting
`busy_timeout`). The real decision is **physical separation of the read and write
paths**: a single tokio actor *owns* the write connection; everything else (UI polling,
diff computation) reads under WAL and never takes the write lock. The diff computes on
a read snapshot while the writer commits the next batch. Instrument where the 120 s
actually goes before hardening the database — the expectation is >90% network.

## Decision Register — Session 2026-07-16

_Decisions taken during the context-analysis round table (Winston, Amelia, Murat, John,
Sally, Paige). Detailed modelling is deferred to the data-model and decision steps; these
are the durable principles agreed._

### D1 — MariaDB is a day-1 production requirement

**Decision (Guy):** MariaDB ships from the start, not "activated later within MVP".
Rationale: production deployment on Synology (MariaDB is Synology-native and included in
Synology backups) in week 1. This is an operator need, not an elegance argument, and it
settles the "why dual-backend now" scope challenge.

**Consequence:** the first story is a **walking skeleton green on both engines** before
any feature is stacked. The day-1 discipline is not "abstract all possible SQL" but
"**no query ships without passing CI on SQLite AND MariaDB**" — a test constraint, not a
framework. Portability is paid while the schema is small.

### D2 — "Merge" is selective field adoption, never a destructive merge

Three operations hide under the word "merge". One is fatal:

1. **Destructive merge — FORBIDDEN.** Collapsing observed + declared into one record
   destroys the measuring instrument, not just the gap: subsequent scans would compare
   observed against a record that already contains observed. Drift detection dies on that
   field, and source-blindness reasoning dies with it.
2. **Link / attach — ALREADY ALLOWED** (FR14). An edge; no data moves.
3. **Selective field adoption — LEGITIMATE.** The operator decides field by field; the
   declared record changes; **the observed record does not move a single byte**; the link
   holds; the pair stays comparable.

> The line: **a merge destroys one side; adoption writes into declared a value whose
> original the observed keeps intact.**

**"Keeping both sides" happens at the RECORD level, not the field level.** A declared
field has exactly one value — otherwise the CMDB answers "it depends". Both truths
coexist because they are two records, each with its own authority.

**Legitimate only if ALL six hold:** field-level, operator-driven, write-only-on-declared,
journalled, observed-immutable, link-preserved. Remove any one and it is a destructive
merge.

**Forbidden by-the-back-door variants:** an `origin='adopted'` field must NEVER become
`follow_observed` (an auto-following field makes the gap structurally impossible = the
death of NFR5); and no "effective value" coalescing view (users would come to believe
that view *is* the CMDB).

### D3 — Field-level provenance on the declared record

The declared record moves to **attributes-per-row** (the only way to carry per-field
provenance without a parallel drifting table):

```sql
CREATE TABLE declared_attribute (
  entity_id TEXT NOT NULL, attr_key TEXT NOT NULL, attr_value TEXT,
  origin TEXT NOT NULL,          -- 'manual' | 'adopted' | 'imported'
  origin_obs_id TEXT,            -- FK observation_record(id), NOT NULL if 'adopted'
  actor_id TEXT NOT NULL,        -- human. never 'scanner'.
  updated_at TEXT NOT NULL,
  PRIMARY KEY (entity_id, attr_key),
  CHECK (origin <> 'adopted' OR origin_obs_id IS NOT NULL),
  CHECK (actor_id <> 'scanner'));
```

- `origin_obs_id` points at the **exact observation adopted** (immutable -> the pointer
  stays valid and auditable forever). *This* is what "keep both sides" means: both sides
  exist physically, and the link is explicit.
- **The gap computation NEVER reads `origin`** — so a field adopted yesterday can drift
  again tomorrow, automatically. `gap := declared.value != current_observation.value`.
- `origin` is read-only metadata: display ("adopted from the scan of 2026-07-12") and
  audit. Zero role in reconciliation logic.
- NFR5's subject is grammatically **the scanner**. An operator editing declared via an
  explicit action is a normal declarative write with a human author. The `CHECK
  (actor_id <> 'scanner')` makes non-drift **structural, not disciplinary**.
- `KeepDeclared` -> **no write at all**. There is no way to silence a gap. An operator who
  refuses the same field 40 times produces a signal, not a silence.

### D4 — Doubt is a DERIVED predicate, never a persisted state

Storing `field_status: CLEAR|DOUBT|RESOLVED` is forbidden. A `RESOLVED` is a verdict on
the *relationship between two values*: it becomes false the moment either moves, and it
carries nothing with which to detect its own staleness. **It is a cache without an
invalidation key** — in six months it becomes the effective value by the back door.

Authority is a property of the **pair**, not of the field or the source alone:

```
authority(source, field)   = capability(source).field AND provenance_class(field) == Machine
authority(operator, field) = provenance_class(field) == Human
```

A source NEVER has authority over a human-authored field, even when it believes it does
(UniFi's "name" is an alias a human typed in *another* interface — not a measurement).

**Four cases:** (A) observed-vs-observed across sources = **FR20, do not re-derive**;
(B) machine field + capable source = trivial drift = the product; (C) human-authored field
= noise, do not surface; (D) **the axes contradict = the real doubt**, with named causes:

```rust
enum DoubtReason { SourceNotCapable, CapabilityUnknown, ProvenanceClassUnknown }
```

Three named causes, never "the system hesitates". A `Conflict` is resolved by configuring
precedence; a `Doubt` is resolved by **repairing the model**. They merge in the UI, never
in the model — mixing them loses the information about which debt to repay.

**The default IS doubt.** The asymmetry decides it: a false positive (asking when it was
obvious) costs one second, is bounded and **observable**; a false negative (silence when
we should have asked) is unbounded and **unobservable — you don't know what you didn't
see**. A safe default is one whose error is *visible*; silence is not safe, only discreet.

### D5 — `FieldDecision`: a cache of ATTENTION, not of TRUTH

```
FieldDecision { entity_id, field, observed_value_at_decision: Hash,
                decided_at, decided_by, disposition, note }
```

It does not say "this field is resolved". It says *"on 16 July we showed Guy observed H,
he chose X."* A dated, immutable fact — **true forever, because it says nothing about the
present**. Three discriminating tests:

| | `RESOLVED` (banned) | `FieldDecision` (allowed) |
|---|---|---|
| Can it become false? | Yes, silently | **No. A dated fact stays true** |
| Read to compute the effective value? | Yes — the disguised merge | **Never** |
| **Purge the whole table?** | **Data is wrong** | **You are merely re-asked. Noise, zero corruption** |

**The purge test is an architecture acceptance criterion** (see AC-8a). It is what makes
the principle falsifiable instead of decorative.

**Doubt is an EVENT, not a state:** it fires only when the observed value CHANGED since
the operator's last decision on that field. Re-posing a settled decision is amnesia, not
rigour.

```
verdict(field) = classify(observed, declared, authority)      // pure; never reads history
surface(field) = verdict == Doubtful
              AND hash(observed) != last_decision.observed_value_at_decision
              AND last_decision.disposition != AlwaysKeepMine
```

`classify` does not know the journal exists. `surface` is a **presentation filter** above
it. One layer, one responsibility.

**The dike (verbatim):** *if a decision record is ever read by the effective-value
computation, we have rebuilt the merge.* Enforced by the compiler (AC-8b), not convention.

### D6 — Two gestures, one lifecycle

Guy's synthesis, which is the model:

| Gesture | Meaning | The gap |
|---|---|---|
| **Accept the gap** (`accept-gap`) | "I've seen it, I haven't decided what to do yet" | **stays OPEN** — wakes on observed-value change, never on a clock |
| **Merge** (`merge`, see D11) | "the gap becomes the norm, **I decide which fields become the doc**" | **closes** — field by field, writes only to declared |
| **"Accept can lead to merge"** | `accept-gap` is a **waypoint, not a terminal state** | open -> (investigation) -> closed |

**The JTBD `accept-gap` fills:** *"When I see a divergence I cannot resolve now, help me
take note of it without lying, so I can move on without losing it."* No existing gesture
does this. Accept closes what isn't decided; ignore says "I don't want to know" when the
operator *does* want to know; **snooze is a timer when the operator wants a note**.

> Snooze expires **on a date**. `accept-gap` expires **when reality moves**. One is a
> timer, the other a **sensor**.

**Metric integrity (decisive):** `accept-gap` does **not** raise the open-divergence
counter — the item was already open before the gesture and stays open after. Had it been
folded into accept, the counter would go **down**: a gap that exists, counted as closed.
*That* is the poison. **`accept-gap` protects the north-star from a false victory.**

**Guardrails:** the note is **mandatory** (a gesture costing ten seconds of thought is
never the lazy default, and the note is the only thing that will help in six months);
**zero penalty** if it becomes the default (that is the user's decision). It cannot be the
lazy button: ignore silences forever, snooze until Thursday, **`accept-gap` silences
nothing — it says "tell me if reality changes", i.e. *I keep listening*. Laziness seeks
silence; this gesture offers none.**

**Guy's rationale (verbatim):** *"Je préfère savoir qu'il y a une divergence plutôt que de
l'ignorer et la laisser se diluer dans la liste des divergences découvertes."*

### D7 — Three operating regimes, DERIVED not declared

The 340-card wall is expected at **first commissioning and that is normal**; afterwards it
should not recur except during **large infrastructure migrations/updates**. Handling
changes as they come is best practice but **explicitly the user's decision, not the tool's
to enforce**.

| Regime | Predicate (derived) | Semantics |
|---|---|---|
| **R1 bootstrap** | `coverage < 0.80` | the attention journal is empty; the wall is **expected** |
| **R2 steady** | `coverage >= 0.80` AND `churn <= 0.02` | nothing moves outside -> nothing should move inside |
| **R3 migration** | `coverage >= 0.80` AND `churn > 0.02` | infra moved; the wall is **legitimate** |

- Regimes are **derived from the journal**, never declared. A `--bootstrap` flag would be
  cheating: we would be testing an `if`. **No `first_run` flag, no clock, no persisted
  state.** R1 ends by itself. R3 is **bounded by observed churn** — you cannot invoke
  "migration" if nothing changed outside.
- `coverage` counts fields that received a **question**, not an **answer** — otherwise the
  regime definition is coupled to the user's discipline, which Guy explicitly refused.
- **R3 is not a special case — it is R1 recurring.** Therefore the **bootstrap flow is a
  MODE, not an onboarding**: gated by current volume, available for life.
- **Doubt granularity follows the regime:** the **field** in R2; the **MOTIF (pattern)** in
  R1/R3 — day-1 divergences are systemic, not random ("12 uppercase hostnames is *one*
  decision, not twelve"). The number of `FieldDecision` records is identical; the number of
  **human gestures** drops ~100x. In bulk mode **doubt is not displayed — it is counted.**
- The switch to grouped view is **auto-detected by volume, reversible, and without
  judgement**. Holding back by **strangeness** (the confidence threshold) != grouping by
  **volume**. Two mechanisms, two criteria.
- **Doubt is never withheld during a migration** — that is precisely when the operator's
  context is fresh. Deferring converts a resolvable doubt into an unresolvable one.

### D8 — The backlog is the user's decision

> **The tool is allowed to MAKE the backlog EXIST. It is not allowed to HAVE AN OPINION
> about it.**

The backlog is **a map of assumed ignorance, not a moral debt**. An item untouched for
three weeks is information ("the user doesn't care about this"), not a sin.

**Hard bans (NFR-grade):** no nag (a notification may only be triggered by a **new fact**;
"it's been a while" is not a fact) · no badge or growing counter ("47" is not information,
it is a reproach disguised as a number) · **no health gauge — a health gauge IS a grade** ·
no age brandished as reproach (age is sortable, invisible by default) · no gamification ·
**no degradation: the product's insistence is constant regardless of inbox state**.

> **Decisive test, applicable to every screen: if the user does nothing for six months,
> does the product become more unpleasant? If yes -> violation.**

**The metric judges US, never the user.** `open-divergence trends DOWN` is a **product
dashboard / telemetry** signal: if it rises, *our* loop is broken — the sort is bad, the
questions are badly posed. The success metric is **not "is the backlog emptied" but "is it
TREATABLE the day the operator decides to treat it"**.

**Critical:** `open-divergence` must **never count pre-baseline items** — otherwise the
north-star is poisoned from day 1, and worse, we would be tempted to nag. At bootstrap
there IS no divergence: there is no declared yet to diverge from. 340 items at first run is
an **inventory, not a backlog**.

### D9 — Presence requires explicit hysteresis

NFR7 ("0 false device-gone per 1000 probes") is untestable as written. The real
architectural requirement: **the engine must distinguish "absence of proof" from "proof of
absence"**. A device that misses one ping has not left. This requires an explicit
**presence state machine with hysteresis** (flip to "gone" only after N failures over
window T), designed at architecture time, not improvised.

**Invariant:** every observed datum carries freshness/confidence, and **absence of data
NEVER produces a change event**. Under fault injection the system degrades toward
"stale/unknown", never toward "changed/gone".

### D10 — Testing decisions that gate the architecture

- **NFR4 becomes a TRIPLET:** precision, recall, **and false-merge rate <= strict epsilon
  (~0.01)**. False-merge is catastrophic and asymmetric (two hosts fused -> the operator
  loses trust and uninstalls); false-split is benign (two entries instead of one,
  correctable). **They must not carry equal weight in the gate**, or we ship an engine that
  fuses aggressively to inflate recall — the worst possible outcome for a CMDB.
- **The labeled identity fixture is an ARCHITECTURE deliverable**, written before/during
  architecture — it defines the shape of the `Connector` trait and of the engine. Deferring
  it means designing the interface twice. It must encode **deliberate ambiguity** (cases
  where the right answer is "I don't know, I flag a conflict") -> a **third class:
  abstention/conflict**. Adversarial matrix, each trap in **positive AND negative** form:
  randomized MAC, multi-NIC, shared-hardware VM, **cloned/spoofed MAC** (the inverse trap:
  false-merge), DHCP churn, VRRP/HSRP shared virtual MAC, hostname collision, ephemeral
  Docker veth.
- **NFR5 extended:** the anti-regression test must also verify that **after an adoption the
  observed record is bit-for-bit unchanged and the link holds**.
- **NFR15 requalified:** not "one 100% identical suite" but **"identical invariants,
  backend-specific triggers"** — the backends diverge exactly where the product lives
  (`SQLITE_BUSY` vs MariaDB deadlock; collation/type semantics). Plus a small backend-
  specific pack. **Both backends in CI on every PR — never MariaDB nightly** (a nightly
  backend is a backend broken for 24 h).
- **NFR8 — fixture rot is the mortal risk:** *a contract test against a frozen fixture is a
  snapshot test in disguise*. Each fixture carries UniFi OS version + capture date + a
  re-capture job that **diffs the schema**. **Bound the UniFi version matrix as a product
  decision** ("we support 3.x and 4.x, full stop") — the zero-privilege connector helps:
  less API surface = fewer fixtures = a tractable matrix.
- **The doubt budget is a test, not an intention** — but the percentage is the wrong
  instrument: it measures a **stock**, not a **flow**, and cannot distinguish healthy
  bootstrap from a stagnating backlog from an engine that re-fires. **The single real NFR,
  valid in all three regimes:**

  ```
  for all cycles c:   D_new(c) <= Delta_observed(c)
  ```

  *The engine has no right to invent a doubt the outside world did not provoke.* It absorbs
  the three regimes without naming them, and the regime becomes a **consequence** of the
  test rather than a parameter of it. Sally's <5% drops one rung: from CI gate to **R2
  telemetry canary**, never blocking.
- **AC-7b/7c are an indivisible couple:** silence (`D_new = 0` — **exactly zero, not
  "<=5%"; a non-zero threshold here is a licence to make noise**) AND wake (mutate one
  observed field -> `D_new = 1`, exactly that field). *Without 7c, 7b is satisfied by
  `return;` — a silence test without a wake test tests a corpse.*
- **AC-8a (purge) is a build gate:** run N cycles, answer everything, `DELETE FROM
  field_decision`, recompute -> **effective values byte-identical**. If one value moves, the
  journal held truth, not attention -> red build.
- **AC-8b is enforced by the compiler:** the effective-value module simply does not receive
  the `FieldDecision` store handle in its signature. *An architecture test you can bypass by
  carelessness is not a test, it's a post-it.*
- **Portability is obtained by refusing SQL, not by abstracting it:** all value comparison
  and normalization (`norm()`: trim, lowercase, MAC->canonical hex, IP->canonical form)
  happens **in Rust**. No comparison descends into the engine — the only way to be identical
  across `utf8mb4_general_ci` and `BINARY`. Same for `now()`: computed in Rust, bound as a
  parameter, never `NOW()`/`CURRENT_TIMESTAMP` in SQL (this also makes the accept deadlines
  deterministic). Timestamps stored as ISO-8601 UTC `TEXT` so lexicographic order ==
  chronological order on both engines.
- **Open question, to be decided and tested, never left to accident (AC-8e):** value
  flapping — answer on `v1`, observed moves to `v2` (re-fires), **observed returns to `v1`**
  — does the doubt re-fire? Position: **yes** (silently re-approving `v1` rebuilds a
  `RESOLVED` verdict via value equality — the banned auto-following field disguised as an
  optimization). Held weakly; the UX cost is real for an oscillating port. Must be a named,
  owned, reversible test: `#[test] fn flapping_value_refires_doubt()`.

### D11 — Vocabulary

**Guy's decision:** the user-facing gesture that closes the gap is named **"Merger" /
"Merge"**, over the round table's counter-proposals (`document` / `adopt`).

**Recorded dissent and its mitigation.** Three agents argued against the word:

- *"Vocabulary is architecture — if the action is called merge in the UI, someone will end
  up implementing it as a merge."* (Winston)
- The word names the **forbidden operation** while the real operation fuses nothing — it is
  a field-selective write to declared (D2/D3).
- Direct contradiction in English-only docs: a **"Merge"** button in a product whose
  founding pillar is **"linked, never merged"**.

**Mitigation (what actually protects the invariant is the tests, not the label):** extended
NFR5 (observed bit-for-bit unchanged, link holds), the purge test (AC-8a, build gate), and
AC-8b enforced by the compiler make a destructive merge **structurally impossible
regardless of the button's name**. The word is therefore a **pedagogy risk, not an
architecture risk**.

**Open, deferred to the UX/docs step:** reconcile the EN docs contradiction — either keep
"Merger" in FR with a different EN verb, or rename the pillar.

**Also raised, deferred:** `ignore` is the one remaining gesture whose tone disdains;
`exclude` / "exclure" (out of scope) would be more honest and more dignified.

**Free and already covered:** user-defined **tags** for sorting objects (Paperless-ngx
style) are `declared_attribute` rows with `attr_key = 'tag:*'`, `origin='manual'` —
**zero table, zero migration**, already FR40. The guardrail is Winston's test —
**"who is allowed to remove this tag?"** Only the operator; the `CHECK (actor_id <>
'scanner')` makes it structural. A **system-maintained** "to triage" tag is **refused**:
drift is demonstrable (gap closes on its own at t1; the tag still says "to triage" at t2)
and it fails the purge test — it had become truth, not attention.

## Feedback to Carry Back to the PRD / UX Spec

_Raised by the round table; these change source documents, not just the architecture._

| # | Document | Change |
|---|---|---|
| F1 | PRD FR13 | **accept-as-declared only covers a NEW discovery.** The dominant long-run case — re-discovery of an already-declared device with drift — has no FR. It needs its own semantics (selective field merge) and its own label. |
| F2 | PRD FR14 | Triage gains a **6th gesture**: `accept-gap` ("seen, not yet decided"; gap stays open; wakes on observed change; mandatory note). |
| F11 | UX Spec | The **bootstrap flow is a MODE** (volume-gated, available for life), not an onboarding. **No `first_run` flag.** |
| F12 | UX Spec | **Doubt granularity follows the regime**: field in steady state, **motif** in bootstrap/migration. |
| F13 | UX Spec | **Backlog bans** as testable rules: no badge/counter, no health gauge, no age-as-reproach, no degradation. Test: *does the product get more unpleasant after six months of inaction?* |
| F14 | UX Spec | **Glossary refactor** — see D11. The glossary uniqueness gate checks that a term has one *translation*, **not that a term has one *meaning***. That is the hole; extend the test with a denylist of banned words. |
| F16 | UX Spec | **The Tailwind implementation note describes an API that no longer exists.** v4 removed the JS-config `content` and `safelist`; use `@source` / `@source inline()` (v4.1+) in CSS. The decision stands; the syntax does not. Fix at source before someone implements it in good faith. |

## Foundation & Dependencies

_(Section renamed 2026-07-17. It was called **"Starter Template Evaluation"** — **the name lied**, the
same way `status_exhaustive.rs` lied (D53): only its first fifth evaluated starters, while the rest holds
the **dependency version policy** and the **sqlx decision**, which are live and cited. A section named
after its smallest part is a section nobody finds its biggest part in. **That mis-naming is why a cleanup
pass first proposed to delete 256 lines of "finished starter evaluation" — judging a section by its title
is F56 again, and this time the map was a heading.)_

### Decision: NO starter template — `cargo new` + a curated dependency set

**Condensed 2026-07-17 from ~40 lines of per-candidate evaluation. The conclusion is stable and the
reasoning that supports it is kept; the four repo-by-repo scorecards are dropped — nothing cited them,
and they answer a question that cannot be asked twice.**

Four candidates were web-verified in July 2026 (`ReeceRose/rust-askama-htmx-tailwind-todo`,
`chase-lambert/rust-app-template`, `emarifer/rust-axum-askama-htmx-todoapp`,
`ElykDeer/askama-axum-rust-template`). **All four are demonstration TODO apps, and all four predate
askama 0.13's removal of `askama_axum` — stale at the root.**

> **The decisive argument is not "our decisions are already made" — it is that the available starters
> carry decisions ACTIVELY CONTRARY to ours.** Scored against our locked decisions they land at **~1.5
> of 8**, and the ones they do make are wired against us: they all use `sqlx::query!` (banned — it is
> sqlx's default path), none has the `Repository` trait, none has the writer actor, and their Tailwind is
> a v3 whose config no longer exists. **We would start BELOW zero: dismantling before building.** *A plank
> you must plane on both sides before laying, you did not buy — you paid for demolition.*

**What a starter really provided was not code — it was a PROOF OF INTEGRATION** (that axum + askama +
sqlx + Tailwind hold together). **That, and only that, is what must be bought back — via the walking
skeleton, and nothing more.** Corollary, and it is a standing rule: **do not test that axum routes. Do not
test that askama renders. If a test would fail only because of an upstream bug, it is not our test.**

**We READ the four repos for the `rust-embed` + Tailwind-standalone plumbing (someone has already debugged
it). We fork nothing.** **Cost, stated honestly: ~1 day of boilerplate we write instead of inherit** —
worth paying to avoid inheriting a removed dependency, but not a zero price.

**D1-rev defines the real starter:** story 1 is the walking skeleton that shows a REAL gap. No GitHub repo
can supply that. _(Historical note: this read "the DUAL-BACKEND walking skeleton, green CI on two engines"
until D64 removed the second engine.)_

### Dependency Version Policy — Guy's principle (adopted)

> **"Je préfère l'utilisation de librairies récentes car de toute manière, il faudra migrer une
> fois."**

This is D1's own logic applied to dependencies: the migration cost is invariant — pay it while
the codebase is empty. Adopted as the standing rule. It changes exactly one thing: axum 0.8.9,
askama 0.16.0 and Tailwind v4 are *already* the latest stable; **only sqlx moved.**

**Boring technology and "prefer recent" do not conflict — they measure different quantities.**
Boring is a **budget of unknowns**: the whole budget must go into the domain (observed vs
declared, reconciliation) — *that* is where we want to be surprised, because that is where the
value is. "Recent" is **migration debt**: staying old does not remove the work, it accrues it
off balance sheet and makes it dearer. They conflict only when the recent version is *itself*
unknown.

> **A two-year-old version is not boring — it is slow exotic. Nobody tests your combination,
> advisories are no longer backported, and your AI assistant proposes code the world has
> forgotten. Old and brand-new are both exotic, at the two ends of the same axis.**
>
> **What makes a version boring is not its age — it is that other people have already hit its
> bugs. "Boring" is not a property of the library; it is a property of what the world knows
> about it.**

### Verified Current Versions (2026-07-16 — re-verify at implementation time)

| Dependency | Version | Note |
|---|---|---|
| `axum` | **0.8.9** (2026-04-14) | latest STABLE; `0.9` is unreleased breaking work on `main` |
| `askama` | **0.16.0** (2026-04-29) | `askama_axum` **removed since 0.13** |
| `sqlx` | **0.9.0** (2026-05-06) | **decided — see below.** Repo moved to `transact-rs` (May 2026) |
| `tailwindcss` standalone CLI | **v4** | JS-config `safelist`/`content` **gone** -> `@source` / `@source inline()` (v4.1+). Linux x64/arm64 (glibc+musl), macOS, Windows |
| `tokio` | `^1` | the most boring of the set; stable for years |
| `rust-embed`, `uuid` (v7), `serde`, `chrono`, `tracing`, `thiserror` | verify at `cargo add` | pin from the real lock; do not invent numbers |

### Decision: `sqlx = "=0.9.0"` from story 1 — resolved on verified facts

| Verified (2026-07-16) | |
|---|---|
| sqlx **0.9.0** published | **2026-05-06** (per the CHANGELOG; our docs said 05-21 — corrected 2026-07-17) |
| A **0.9.1+**? | **No.** 0.9.0 is the head of the line |
| Crate renamed / deprecated? | **No.** Still `sqlx` on crates.io, not yanked -> the blocking check **passes** |
| sqlx **0.8.6** published | **2025-05-19** -> **14 months old** |

Three readings, all converging:
1. **The decantation rule's own terms allow it:** "wait for the first `.1`, **or 4-6 weeks,
   whichever comes first**". Eight weeks have elapsed — the delay is spent.
2. **The absence of a `0.9.1` after two months reads as stability**, not immaturity: a broken
   `.0` would have a `.1` by now. (Inverse reading acknowledged: the collective's patch cadence
   under the new org is an unknown of a few months — not a structural risk.)
3. **The clincher: `0.8.6` is 14 months old.** It is not "the mature version", it is **a dead
   line** — 0.9 shipped, 0.8 will never move again. **The "prudent" option was in fact the more
   exotic of the two.**

**Why the risk is near-nil for US specifically:** the walking skeleton touches only `query()`
runtime, `.bind()`, `fetch_*`, `Row::try_get()`, concrete pools, and `migrate!`. No `query!`
(banned), no `Any` (banned), no `chrono`/`uuid` type-mapping (timestamps are TEXT, `now()` and
UUIDv7 come from Rust), no `RETURNING`. **sqlx's major-release breaking changes historically
concentrate exactly where we do not go** — type-mapping, compile-time macros, `Any`, custom
encode/decode traits. *"We spent three decisions refusing that surface. Pinning 0.8.6 is
insuring against a fire in a room we bricked up."*

**And it cuts drift rather than adding it:** pinning 0.8.6 puts us in a state where the AI
assistant's suggestions and the installed crate diverge silently. 0.9.0 aligns the code with
what the current docs describe. The dedicated "bump sqlx 0.9" story is **removed from the
backlog — it has no object.**

**The residual risk, named honestly:** *the risk of 0.9.0 is not "sqlx is broken", it is "your
copilot thinks we are on 0.8".* That risk does not decrease with sqlx's quality — it decreases
with **calendar time**. Mitigated by the compiler (an API break is an `E0599`, not a heisenbug)
and by the dual-backend CI for the non-compiled residue (pool, encoding): two independent
drivers that must produce the same result.

**Verify before writing `Cargo.toml`, in order:** (1) ~~crate name resolution~~ **done —
passes**; (2) `query()`/`bind()` signature + lifetimes; (3) `Row::try_get` / the `Row` trait;
(4) **`sqlx::migrate!` — folder format, `_sqlx_migrations` table, checksums** ("the only thing
that could cost us a day"); (5) does `tls-rustls-ring` still exist under that exact name;
(6) are `mysql`/`sqlite` still supported features, not "experimental". Explicitly out of scope:
breaking changes to `query!`, `Any`, `chrono`/`time`/`uuid`, `RETURNING` — we do not touch them.

### Architectural Consequences

**1. We write our own Askama->Axum `IntoResponse` (~15 lines). `askama_web` is REFUSED.**
"Boring" does not mean few lines — it means **few surprises over time**. `askama_web` is a
third-party crate sitting **between two major dependencies**, which must track both: the day
axum 0.9 lands (already on `main`), it becomes a **version lock** on the critical path of our
two most likely upgrades. Upstream already told us: **`askama_axum` was removed in 0.13 precisely
because bridge crates are a maintenance burden. `askama_web` is the same bridge, repainted, with
one fewer maintainer.**

> **A dependency whose entirety we can rewrite in 12 readable lines is not a dependency — it is
> a coordination debt.**

Bonus: we own the error path. On a reconciliation engine, a silently failed template render is
worse than a clean 500.

**2. The sqlx governance move is NOISE — read correctly, it is a de-risking.** "SQLx has not been
owned or maintained by LaunchBadge for a few years..." is not the announcement of a change; it
is **a land-registry filing**. The change happened years ago; the repo name caught up with
reality. **The risk was the situation BEFORE**: an inactive corporate entity holding the
namespace of a foundational dependency is exactly the setup that produces nasty surprises
(acquisition, liquidation, a lawyer discovering an asset, a crates.io publish key belonging to
nobody reachable). `transact-rs` is the **exit** from that grey zone. The signal to read is *are
the same hands on the keyboard?* — yes. Contrast `event-stream` (npm, 2018), where the signal was
**a new identity obtaining publish rights**: here that signal is absent. This is an **inverted
event-stream**. Delta ~ 0. **Mitigating a null delta is theatre.**

**3. What actually mitigates sqlx is D1, not the `Repository` trait — do not confuse them.**
The trait covers **API coupling** (a sqlx refactor has a two-file blast radius) and **cold
replacement**. It does **NOT** cover a pooling bug under WAL, a MariaDB driver regression, an
unpatched CVE, or a behavioural divergence between drivers — **those cross the trait without
seeing it. The trait is an abstraction of API, not of behaviour: it protects us against sqlx
refactoring, not against sqlx being wrong.**
**D1 is the behavioural mitigation:** "no query ships without green CI on SQLite AND MariaDB" is
a **permanent differential test across two independent drivers**. A driver bug rarely manifests
identically on both sides — it produces a divergence, and our CI sees divergences. **We have
built, without naming it, a sqlx bug detector that runs on every commit.** The constraint accepted
for a product reason (MariaDB day-1) is also our insurance on our heaviest dependency. Paid twice,
built once.
Corollary: **the NFR15 invariant suite is written against the `Repository` trait, never against
sqlx** — so the day sqlx rots, a green conformance suite tells us when the rewrite is done.
Marginal cost: zero; it is already D1's work.

**4. `sqlx::Any` / `AnyPool` is FORBIDDEN.** The natural temptation of a dual backend. It levels
down, masks type divergences, and would blind us to exactly the dialect bugs the `Repository`
trait must isolate explicitly. **Two concrete adapters, two concrete pools.**

**5. `tls-rustls-ring`, never `native-tls`** — a system OpenSSL link is the death of the static
single binary on Synology.

**6. Tailwind v4 changes the syntax, not the decision — and the doctrine is REINFORCED** (config
now lives in a file we already commit):

```css
@import "tailwindcss";
@source "./templates/**/*.html";                            /* was `content` */
@source inline("htmx-request htmx-swapping htmx-settling"); /* was `safelist` */
@source inline("{bg,text,border}-{observed,declared,pending,committed,expired}");
```

**The v4 trap specific to us:** any class **built in Rust** (a `match` on a state enum returning
`"bg-pending"`) is **invisible to the static scanner**. Miss one and there is **no build error,
no red test — just a status pill with no colour in production**. On a product whose core is the
visual distinction of observed vs declared, that is a **silent product bug**.
**The CI drift-check catches only ONE of the two drifts:** it sees "class added to a template,
CSS not regenerated"; it does **not** see "class built in Rust, `@source inline()` forgotten" ->
CSS identical, `git diff` green, grey pill in prod. Hence AC-1.12. The CLI version is pinned
in-repo and consumed by both CI and the dev script.

**7. `Cargo.lock` COMMITTED + Docker build `--locked`. Non-negotiable.** We ship binaries to
strangers via Docker Hub. Same doctrine as the committed CSS: **nothing resolves on the fly at
build time.** It turns "any new version of any crate can enter my binary without my deciding" into
"nothing enters without a commit of my hand" — neutralising ~90% of the supply-chain scenario at
~zero cost. And: **pinning in `Cargo.toml` what the lockfile already guarantees is superstition.**

**8. Pin semantics: a RATCHET, not a wall.** An exact pin proposed as a *defence* is a pin that
rots — **a defensive pin has no exit condition; that is how you wake up on 0.9.0 in 2028.** The
exact pin means "do not advance **on your own, without my seeing it**": Renovate opens the PR, the
dual-backend CI arbitrates, the pin rises.

**9. Dependency drift is the #1 risk, and its vector is specific to this project.** The
`askama_axum` damage is not in the compiler — **it is in the training corpus**: every tutorial and
starter still teaches it. **In a solo + AI-assisted project the drift vector is "my assistant
confidently proposes the API from three versions ago, and there is no second human reviewer to say
*that no longer exists*."** Good news: a removed API does not compile — **the compiler is the
reviewer**, and that class is already gated. What is NOT gated is **the drift that still
compiles**.

> **"Prefer recent" is a COST policy, not a security one — it is only affordable if the marginal
> cost of a bump tends to zero.** A treadmill is what you get when every bump costs human
> attention, and in solo, human attention is the scarcest resource. **The whole policy reduces to:
> make the bump free, or do not prefer recent.**

**The gate above all gates, and it is not technical: does the CI deserve a blind auto-merge?**
Here — two green backends + the purge test + AC-8b enforced by the compiler = **yes**. Therefore:
**grouped Renovate + auto-merge on patch/minor when green, in the SAME story as the CI** — *not
later, or you will never have the policy, only the starting point.* Breaking changes: dedicated
PR, never grouped, never auto-merged. **Veto: never two breaking changes in one commit —
bisection must stay trivial, because there is nobody else to bisect.**
Plus: **pinned MSRV + `rust-toolchain.toml`** ("recent" without a reproducible toolchain is
non-determinism distributed to strangers) and **`cargo-deny` limited to `advisories` + `licenses`**
— licences are not cosmetic: **a binary links statically, and a GPL/AGPL entering transitively into
a distributed binary is a licence problem for our users**, found by a third party at the worst
moment. Note this gate becomes *more* necessary under the prefer-recent policy: **a library can
change licence on a minor.**

**10. Named theatre — do not build:** `cargo-audit` **on top of** `cargo-deny` (redundancy
disguised as defence in depth) · a `cargo outdated` CI step that blocks nothing (a report that
blocks nothing is not a gate, it is an RSS feed) · auditing transact-rs's maintainers ·
forking/vendoring sqlx · an SBOM at MVP · **testing that axum routes**.

> **A CI you would not trust to merge a patch bump blind is not a CI — it is a suggestion. Every
> gate that has never caught anything is a tax on the gates that do.**

**11. What we refuse to depend on at all:** `askama_web` · `askama_axum` (named because all four
starters wire it and someone will paste it by reflex) · `sqlx::query!` / `SQLX_OFFLINE` · **any
crate re-abstracting sqlx** (SeaORM, Diesel, a "multi-dialect" query builder) — *two abstractions
stacked on the same problem is where we would lose the collation bugs* · **any crate that brings
Node back into the build path. Red line.**

### Open, carried to Step 4 (Architectural Decisions)

- **A decantation delay on breaking versions?** For: first `.1`, or 4-6 weeks, whichever comes
  first; an advisory that touches us cancels the delay. Against: **refuse any age gate** — *"the
  most important refusal, because it is the one that LOOKS most prudent. It catches nothing the CI
  does not already catch, and turns the preference into ceremony. If your CI is good, age is a
  superstition. If it is bad, age will not save you."* Guy's stated preference leans against the
  delay. Both positions agree on: grouped Renovate + auto-merge on green, shipped with the CI.
- **MariaDB in CI: service container vs testcontainers.** AC-1.3 is dead without it.
- **`sqlx.toml` (new in 0.9.0)** — aimed at multi-database setups; evaluate against our
  dual-backend needs. Not assumed.

## Core Architectural Decisions

### Category 1 — Data Architecture: Identity

_The product's #1 risk. NFR4 gates release on it. Everything else depends on it._

#### D12 — The three-level model: `observation -> interface -> device`

**The reframe:** FR43 already lists `interfaces` as first-class entities and the MVP says
"Device inventory (**multi-interface**)".

> **A MAC identifies an INTERFACE, not a device.**

Identity therefore splits into two problems of very different difficulty:

| | Problem | Main signal | Attacked by |
|---|---|---|---|
| **L1** | **Interface identity** — "is this NIC the same as yesterday?" | MAC | MAC randomization, cloned MAC, Docker veth |
| **L2** | **Device grouping** — "are these 3 interfaces the same host?" | hostname, topology, DHCP | multi-NIC, shared-hardware VMs, VRRP/HSRP |

The adversarial trap matrix splits cleanly: multi-NIC false-split = L1 correct, **L2 failed to
group**. Cloned-MAC false-merge = **L1**. VRRP/HSRP = 1 interface, 2 devices = **L2**, and it
breaks any naive hierarchy. MAC randomization = 1 device, N ephemeral interfaces = **both**.

**Why it is not invented complexity:** the world produces the counter-examples without our
inventing them — the reference Synology NAS is `eth0`+`eth1` bonded + `docker0` + N `veth`
(one device, ten interfaces); any UniFi AP has a LAN MAC + 2.4 radio MAC + 5 radio MAC (the
controller already exposes them separately); a laptop has Wi-Fi + dock Ethernet + a per-SSID
randomized MAC. **These are the median, not the exotic.** The reframe does not add an entity —
it makes explicit an entity the PRD already ordered and that we were about to implement
covertly as a column. **This is not invented complexity, it is complexity discovered late: the
first we refuse, the second we pay now or pay with interest.**

**The economic test:** without the split, a Wi-Fi+Ethernet laptop appears as two devices. The
operator announces 300 hosts, the tool shows 340. **He does not trust the inventory, therefore
he does not trust the gap. The product is dead.**

**Hardening — an observation is a PRESENCE, an interface is INFERRED:**
> What we observe is not an interface. It is a **presence: (MAC, context, time)**. A per-SSID
> randomized MAC produces two distinct presences that are the same physical interface.
> **If `interface` == MAC, we gained nothing — we merely renamed.**

**The dividend — the reframe FACTORS:** both arrows are **the same record-linkage problem with
different signals**. It is not two problems coded twice — it is **one engine, instantiated
twice** with two rule sets and two blocking keys. *A good reframe reduces code.*

**Product validation (JTBD):** *"When the operator opens the IP-conflict alert, what is he going
to unplug? Not an interface. **A BOX.** He walks down the corridor, looks at a shelf, pulls a
cable. The job stops at a physical object."* -> **the `device` level is non-negotiable**; it is
the only level where the product keeps its promise. The `interface` is **an observed fact** —
the operator never declares, names or accepts one. It appears in the API because it exists,
*"as a fingerprint appears in a case file: nobody says 'I identified the fingerprint', they say
'I identified the man'."* It is an implementation detail — **but an honest one, because reality
imposes it: a three-NIC machine IS three MACs, and pretending otherwise is lying in the model.**
Complexity imposed. We pay it.

#### D13 — Matching engine: option C, with the rule cascade as the verdict function

**Decision: C** — candidate generation (blocking) -> verdicts -> **three-way decision**
(`match` / `no-match` / **`ambiguous` -> abstain**).

**Why not A alone** (ordered cascade, first match wins): it has no native abstention, and worse,
**it has no representation for CONFLICT** — "MAC identical" AND "hostname different" AND "subnet
different": which rule fires first? The one written first. **That is not a decision, it is an
accident of file order.** And conflict is exactly the case where the operator needs us.

**Why not B** (weighted scoring): the weights are invented — on a solo project with no training
corpus, calibrating `w_hostname = 0.3` is numerology. NFR4 becomes a tuning exercise against the
fixture — **we would optimise the number, not the truth**. Worse, from the testing chair: **the
fixture would be both the calibration set AND the gate's oracle. That is test-on-train. It is
not a test, it is a mirror.** And a score of `0.73` cannot be explained to the operator, while
"linked, never merged" requires every link to be **justifiable**, not merely correct.

**Why C** (all three chairs converge): **abstention is native, not bolted on.** It is Murat's
third class, it is FR16, it is Sally's Resolve panel — *when three people converge on the same
structure from three angles, the structure is real.* And from the product chair: **"I don't
know" does not erode trust. "I know" followed by "actually no" erodes trust — and irreversibly.
An abstention is handled in ten seconds and teaches the operator the product does not bluff. A
false merge is discovered three weeks later and he never reopens the app.**

**The fusion A+C — and the clause that protects it.** The cascade IS the verdict function, but
the output contract matters:

> **REFUSED: `rule -> confidence: f64`.** A float compares, averages, thresholds — and we are
> back to invented weights via the back door. **If the output is a float, B has won in
> disguise.**

**All rules are evaluated** (not first-match-wins); each yields an enumerated verdict; verdicts
combine by an **algebra, not a sum**:

```rust
enum Verdict { Decisive, Supports, Neutral, Opposes, Disqualifying }
```

| Condition over the verdict set | Decision |
|---|---|
| any `Disqualifying` | `NoMatch` — **absolute priority, short-circuits everything** |
| a `Decisive`, no `Opposes` | `Match` |
| **a `Decisive`, >=1 `Opposes`** | **`Ambiguous`** <- *the cloned-MAC case* |
| no `Decisive`, >=1 `Supports`, no `Opposes` | `Ambiguous` (weak evidence) |
| `Supports` AND `Opposes` | `Ambiguous` (conflict) |
| only `Neutral` / nothing | `NoMatch` (absence of proof) |

**Properties:** zero arbitrary threshold · **abstention EMERGES from conflict** (we did not add
it — that is exactly its semantics) · **explanation is free** (the list of `(rule, verdict,
evidence)` IS the explanation) · **adding a rule recalibrates nothing** — *in B, adding a signal
forces retuning every weight; here we add a line and the truth table is unchanged.* **That is
THE property that matters for a solo dev living with this code for three years: the marginal
cost of a signal must be constant, not quadratic.** · **the cloned-MAC false-merge falls out on
its own — the strict false-merge target is obtained by construction, not by calibration.**

**Level split:** **L1 = pure A** — a deterministic join on the scope-qualified key
`(l2_domain, mac) -> interface`. It is not a probabilistic problem. **L2 = the FORM of C with
A's decision function.** The "two thresholds" of C are not floats — they are **named rules**.

> **Universal rule: floats may RANK, never DECIDE.** A score may order candidates in the Resolve
> panel (UI comfort). It never crosses a decision boundary. **The moment a float decides,
> explainability and the truth table die the same day.**
> Corollary (portability): `confidence` is an **INTEGER in milli-units (0..1000)**, never
> `REAL`/`DOUBLE` — *a threshold at 0.85 compared as a float on two engines = two different
> identity decisions for the same input.*

**Structural facts are never scored:**
> **Everything structurally knowable must be known at ingestion, never scored. We spend scoring
> only on what is genuinely uncertain. Confusing an IANA fact with scoring turns a fact into a
> probability — and that is how weights get invented.**
- VRRP/HSRP MAC prefixes are **IANA-reserved**: `00:00:5e:00:01:xx` (VRRP, last bits = VRID),
  `00:00:0c:07:ac:xx` (HSRP), `00:00:0c:9f:fx:xx` (HSRPv2). **A reading, not an inference.**
- The **U/L bit** (2nd bit of the 1st octet) = locally administered = randomized/virtual.
- Both are `Disqualifying` as grouping anchors, known at ingestion.

**The blocker — the blind spot nobody names:**
> *If the candidate generator does not propose the pair, no downstream logic can ever group.*
> **That is where false-splits are born silently, and nobody tests blockers.** -> a dedicated
> assertion: `blocking_recall >= 0.999`, measured in unit tests, **before the scoring exists**.

And at 300 hosts the blocker is **not** there for performance (90k pairs is noise on a NAS i5):
> **It is there for SEMANTICS — it defines the universe of plausible candidates, hence what
> "ambiguous" MEANS. Without blocking, abstention has no denominator.**

#### D14 — The link is an ENTITY (SCD2), not a foreign key

It carries the rule applied, the evidence, the confidence, when, by whom — and is **revisable
without destroying anything**. `linked-never-merged` applied to identity itself: **a bad link is
UNLINKED, never erased.**

**Shape: SCD2 (append + a single closing stamp)**, not pure event-sourcing, not mutable-in-place.
- *Pure event-sourcing* -> the current link is either re-folded `O(history)` on every read
  (unacceptable: the gap API reads it constantly) or materialised in `link_current` — **and a
  derived materialised table IS a cache**: what is its invalidation key? We would have written a
  cache AND a log to obtain what one SCD2 table gives.
- *Mutable-in-place + `link_history`* -> `link_history` is a cache of the past, same drift, and a
  rewritten row loses the proof it was rewritten.
- **SCD2 survives the "no cache without an invalidation key" rule — and it is not an exception to
  it:** *"the current link is a ROW, not a derivation. No derived state exists, therefore there
  is nothing to invalidate, therefore the rule does not bite. **It is not an exception to the
  rule — it is a design that makes the rule inapplicable.**"*

**`AMBIGUOUS` is a LINK, not an absence.** It materialises "these candidates look alike enough
that we must ask you", with N `link_candidate` rows carrying their evidence. **The ambiguity is
DATA, not a hole — otherwise there is nothing to display and FR16 is vapour: you cannot "present
the candidates with their evidence" if the candidates are nowhere.**

**Consistency with D4 (doubt is never persisted) — the objection met head-on:** `identity_link`
is a **cache of ATTENTION, not of TRUTH**, exactly like `FieldDecision`. It says "look here", not
"this is true". **The purge test applies identically: `TRUNCATE ... WHERE decided_by='ENGINE';
re-run engine;` must reproduce the same decisions bit for bit.** What does not survive the purge
are `decided_by='OPERATOR'` rows — and those are **INPUTS, not derivations**, on a par with an
observation. Same table for query convenience, **two natures — and if that frontier is fuzzy in
the code, the invariant is dead.**

**`ruleset_version` is mandatory:** without it, improving the engine is **a silent data
migration — the worst kind.**

**N:N from day 1** (a schema decision, hence taken now): `interface.device_id` is NOT a unique
FK. VRRP forces it. If `grouping` were an FK, VRRP would be unmodellable and we would learn it in
production.

#### D15 — Identity migration: `declared_attribute.entity_id` is NEVER updated

_The hole nobody had named — and "harder than the matching engine"._

Three cases, three different answers; treating them alike is the fault:
- **A — bad grouping** (interface I attached to D1, belongs to D2): declared attributes live on
  the **device**; the device was not wrong, only the membership. **Nothing to migrate** — close
  the grouping, open a new one. *A strong argument for declared attributes to live on `device`.*
- **B — split** (two observations wrongly fused): do the attributes belong to I1 or I2?
  **The machine cannot know. There is no algorithm. Any heuristic here is a false merge disguised
  as cleanup.**
- **C — merge** (two entities wrongly fused): key conflict. Never auto-resolved.

> **THE RULE: `declared_attribute.entity_id` is NEVER updated. Ever. No UPDATE.**
>
> *A `declared_attribute` and a `FieldDecision` are **a human's testimony about a referent**. If
> the referent changes, the meaning of the testimony changes. Rewriting `entity_id` is making a
> human say something they never said, about an object they never saw. **It is not moving data,
> it is falsification.** `UPDATE declared_attribute SET entity_id = ?` is the most dangerous line
> of SQL in this project — **and it looks like a routine refactor.**

**Mechanism:** an `identity_migration` row (kind: split/merge/relink/quarantine, reason, human
`actor_id`, `CHECK (actor_id <> 'scanner')`); source attributes move to
`state='pending_migration'` **without moving**; **the target entity is born naked**; only a
**human action** re-affirms a NEW row against the target with `origin='migrated'` +
`origin_migration_id` (a non-bypassable CHECK). **Never a DELETE on `entity` — tombstone**,
because historical `declared_attribute` rows FK-reference it and that proof must survive.

**`FieldDecision` gets the same treatment, harder: decisions do not migrate.** A decision means
"for THIS device, I choose the declared value"; for a *different* device it is an unverified
assumption. The target starts at zero decisions. Mitigation, not workaround: the UI offers
"carry the decisions over", which **re-emits** fresh events signed by the human. *The machine
proposes, the human signs* — FR16's spirit applied to correction rather than detection.

**The trap this creates — and it is the AC that decides everything:**
> After a migration, `pending_migration` rows **must not feed the gap computation**. Otherwise an
> honest re-link generates **a storm of phantom gaps**: an operator fixes an identity error and
> gets 40 false alerts. **He will never correct anything again.**
>
> **Identity correction must be cheaper than the error, or nobody corrects and the "revisable"
> model is a fiction.**

-> the gap computation filters `AND declared_attribute.state='asserted' AND entity.state='active'`.
Non-negotiable. **AC-M-03: split a device with 10 declared attributes -> gap delta == 0.** *That
is the AC that decides whether an operator will ever dare correct an identity.*

**Cost, stated before not after:** the split ambiguity is **undecidable** -> a human review queue.
At 300 hosts it is bearable. **The three-level model CREATES this queue; the two-level model did
not have it. It is the real cost of the reframe, and it is in human-hours, not lines of code.**

#### D16 — `virtual_device` returns to MVP (reversal)

VRRP breaks the **tree**, not the hierarchy — but the n:n link alone is **not** enough, because
the question is not *grouping*, it is **gap attribution**.

- **Option A — pure abstention: REJECTED.** The default gateway becomes a permanent hole. And
  abstention means "I don't know, ask the operator" — **but here there is nothing to ask: the
  operator would answer "both". This is not an ambiguity, it is a topology fact.** Abstention was
  designed as the output of an **evidence conflict**; here there is no conflict — both sources
  agree, the VIP is shared, that is the definition of VRRP.
  > **Using abstention here would make it a SEMANTIC DUSTBIN — the catch-all for what we failed
  > to model. That is the drift that kills abstention as a signal: if `Ambiguous` means both
  > "real conflict" and "unmodelled case", it means nothing, and the operator learns to ignore
  > it.**
- **Option B — attribute to the master: REJECTED, firmly.** A lie with a three-second lifetime:
  on failover the history says the IP **moved** — a false gap, a false movement, at every
  failover. And knowing the master requires speaking VRRP, which no MVP connector does — **we
  would invent data we do not observe.** Fundamentally: **choosing a winner between two legitimate
  owners is merging.**
- **Option C — `virtual_device` in MVP: DECIDED.** The VIP gets its own `device` with
  `kind='virtual'`; attribution is clean, the gap computation has a subject again. **And the
  discovery: the n:n imposed for one reason serves another** — not "this presence belongs to two
  devices" but "**this virtual device is BORNE BY two physical devices**". Cleaner, and it is
  what the network *is*.

> **The rule applied: what BREAKS the semantics of a computation is not deferrable; what
> ENRICHES it is.** `virtual_device` breaks. It comes in. *(An error of judgement on my part, not
> a scope arbitration: without it, Journey 3's core feature does not work **on the default
> gateway**. A defect striking the most-consulted object on the network is not deferrable.)*

**Created on a STRUCTURAL FACT** — the IANA prefix, already known at ingestion, already
`Disqualifying` as a grouping anchor. **No new rule, no score.** *The same structural knowledge
that disqualifies the VIP as an anchor qualifies it as a virtual device — one line of truth doing
two jobs.* **Deferred to Growth:** VRID detection, master tracking, failover history, cluster
health views.

**And the IP-conflict alert when the honest answer is "two" — Journey 3 is saved:**

> **Conflict on 192.0.2.1** — carried by the virtual gateway `vrrp-gw-01` (VRRP, MAC
> `00:00:5e:00:01:0a`), **borne by `rtr-a` and `rtr-b`** — **AND by `nas-backup`** (`d4:ca:6d:...`,
> switch port 12, VLAN 10).

*The conflict is between the `virtual_device` and the intruder. **There IS a box to unplug:
`nas-backup`.** The product promise was never "name both sides of the conflict" — it was **"name
the culprit"**. The VIP is not the culprit: it is legitimate, it is the gateway. The culprit is
the one who just sat on its address, and he has a switch port and a VLAN.* **UniFi says "IP
conflict". opencmdb says "unplug the NAS on port 12".**

> **The discipline: never hide behind abstention what we can name. Naming two bearers is an
> answer. Saying nothing is not.**

#### D17 — `dormant`: the lifecycle of ephemeral interfaces

**The finding:** under "a MAC identifies an interface", every randomization rotation **creates an
interface**. Order of magnitude (corrected, and honestly): **~50-200 `local` interfaces/month**,
i.e. ~600-2400/year against ~300 stable `universal` ones — *iOS/Android default to a MAC stable
**per SSID**, so there is no periodic rotation while associated.* **The exact figure is unknown;
the monotonicity is not.**

**It is NOT a memory leak** (~2400 rows ~ 500 KB; NFR18's 512 MB is not threatened by three
orders of magnitude — *"I used 'memory leak' as a figure of speech; the imprecision was a
fault"*). It is not primarily a performance problem either. **It is a CORRECTNESS problem:**

> The gap computation filters `entity.state='active'`. A dead `local` interface left `active`
> counts as an unreconciled asset **forever**. The gap — **the number the operator looks at every
> morning** — drifts monotonically upward and **never comes back down. Within a year, the
> product's central indicator is noise.**

Plus: a dead `local` stays a grouping candidate months after the MAC ceased to exist. **The device
is correct per the model and false per reality.**

**No `presence` level below `interface` — rejected.** At L1 a randomized MAC **IS** a distinct
interface; a level reserved for `local` MACs would **treat the U/L bit as a judgement**.
> **Same nature = same table. The difference is not *what they are*, it is *how long they stay
> true*.** -> a state transition, not a new level.

> **THE RULE: an `interface` with `mac_kind='local'` unseen since `dormancy_window` moves from
> `state='active'` to `state='dormant'`.**

**It does not violate "no DELETE on entity" — it is its application.** `dormant` is a tombstone:
the row stays, the FKs stay valid, `first_seen_at`/`last_seen_at` survive (exactly what FR38
requires). *"I never said 'no logical deletion'. I said 'no DELETE'. **There was no conflict to
resolve; there was a missing state.**"*

**Three non-negotiable properties:**
1. **Reversible** — if the MAC reappears, back to `active`, **same entity**. *That is what
   distinguishes `dormant` from `retired`.*
2. **Only `mac_kind='local'`** — *a server powered off for six months has a `universal` MAC: it
   is **absent**, not dormant. **The absence of a `universal` is information the operator wants to
   see; the absence of a `local` is the protocol's nominal behaviour.*** The only asymmetry, and
   it rests on a **structural fact**.
3. **An interface grouped by human affirmation NEVER goes dormant automatically.** *The machine
   does not revoke a human affirmation on a timer* — same logic as D15.

**The unnamed constraint: `dormancy_window < observation_retention` (FR38).** Otherwise the sweep
marks interfaces dormant **with no remaining observation to justify the decision. The decision
becomes unauditable — and auditability is this product's foundation.** -> a **config validation
constraint**: startup FAILURE naming both keys, not a warning.

Default window: **30 days** (exceeds a normal re-join cycle; well within FR38's 90-day retention).
Configurable. **Open: do the ingestion probes capture probe requests?** If yes the order of
magnitude is off by ~30x and 30 days is too long. **The policy holds either way; the default does
not.**

The blocker excludes `dormant` from **automatic** candidate generation — but `dormant` remains a
candidate for an explicitly human-requested historical reconciliation.

#### D18 — The release gate: Tier 1 binary, three columns

**At 300 hosts, the only measurable threshold is ZERO. Every fraction is theatre.**

The arithmetic that settled it: *with n=300 and zero observed events, the 95% upper bound on the
true rate is 3/300 = **1%** — the strongest statement production can make. Conversely, if the true
rate is 1%, the observed count is Binomial(300, 0.01): mean 3, sd 1.72. **Observing 0 and
observing 6 are both compatible with a true rate of 1%.** A `<= 0.01` threshold cannot distinguish
0.5% from 2%. **It is not a gate, it is a coin toss wearing a badge of authority.*** Three chairs
— product instinct, engineering nose, the binomial — one number.

And the self-inflicted blow that clinched it: *"my zero-tolerance was at the **interface** level,
my float was at the **device** level. L1 is a lookup — I put zero tolerance where it is easy. L2
is the inference, where the promise lives — I put a float where it is hard. **I gated the easy and
hedged the hard: exactly the cowardice I accuse the engine of, applied to myself.**"*

> **THE GATE = Tier 1 only.** ~50 adversarial trap scenarios, each in **positive AND negative**
> form, **binary, zero tolerance, at the device level**. One number blocks: **truth-table failures
> = 0.**

**The truth table has THREE columns, not one** — this is where the anti-cowardice guard lives:

| Column | Failure condition | Guards against |
|---|---|---|
| `must-not-merge` | a merge | the false merge — **the operator loses trust and uninstalls** |
| **`must-merge`** | **an abstention** | **cowardice — *an engine that abstains on everything scores false-merge = 0 and gets demolished by the middle column*** |
| `must-abstain` | a decision | guessing on the honestly ambiguous case (FR16) |

*N5 disappears as a gated metric and is reborn as **rows in the table**: cowardice becomes binary,
deterministic, measurable. Better than the metric it replaces.*

**The distinction that makes it legitimate** (and it must be named that way in the PRD, *"or in
six months someone gates the raw rate and we will have optimised fear instead of cowardice"*):
> **An engine that abstains because there is NOT ENOUGH SIGNAL is being honest — the world is
> ambiguous and it says so. We do not gate that; we measure it and drive it down. An engine that
> abstains WHEN THE SIGNAL IS THERE — identical MAC, identical hostname, a single candidate — is
> being cowardly. That is not ignorance, it is refusal to conclude. That we gate.**

**OBSERVABILITY = Tier 2** (bulk, 300 hosts / 36 subnets), published per release with confidence
intervals, trended — **blocking nothing**: cluster contamination (`contaminated_clusters /
produced_clusters` — *pairwise dilutes: two fused multi-NICs and two fused Raspberry Pis are each
**one rotten record** to the operator; the cluster is the right unit of trust*), pairwise
precision (*"the false-merge in disguise — we already gate it at cluster level; two of the same
judge is a judge and his echo"*), pairwise recall (*"false-split is benign — so why would it block
a release? **A loose threshold on a benign defect is a gate that can never fall, and a gate that
cannot fall is decoration.**"*).

**And the condition that keeps observability from becoming decoration nobody reads:**
> **Tier 2 is Tier 1's trap factory. The gate only grows by proof.** *The day the bulk drops a
> cluster Tier 1 did not foresee, that cluster becomes trap #51.*
>
> **The honest limit, to be said to Guy's face: a trap suite proves nothing about what I failed to
> imagine. At v0.1 the gate is weak and honest rather than strong and false. Tier 2 is the only
> discovery mechanism for the unimagined. That is why it lives.**

**And the residual fragility, named:** no `CHECK` detects a false merge. **The schema makes a
false merge revisable and traceable; it does not make it impossible.** A 300-host corpus is small.
**That is a release risk, not a test detail.**

#### D19 — The fixture IS the `Connector` trait

**The fixture is an ARCHITECTURE deliverable, written before/during architecture** — it defines
the shape of the trait and of the engine. **Deferring it means designing the interface twice.**

The fixture is a serialised stream of Observations + a truth labelling. The `Connector`'s job is
to emit Observations. **Therefore the fixture schema IS the Observation schema. Write the fixture
and the trait falls out.**

```rust
struct Observation { obs_id, connector_id, observed_at: Timestamp, scope: Scope,
                     facts: Vec<Fact>, raw: Option<Value> }   // `raw` = provenance, NEVER read by a decision
struct Scope { l2_domain: L2DomainId,   // the MAC's uniqueness space
               vantage: VantageId }     // WHO saw it
enum Fact { Mac{addr, locally_administered}, IpV4{..}, Hostname{name, source},
            DhcpLease{..}, Uplink{peer_mac, peer_port}, OuiVendor{..}, Rtt{..} }

trait Connector {
    fn id(&self) -> ConnectorId;
    fn poll(&mut self, now: Timestamp) -> Result<Vec<Observation>, ConnectorError>;
    fn capabilities(&self) -> Capabilities;   // which Fact variants it CAN emit
}
```

**What the fixture imposes on the trait — revealed by writing it, and prose never would have:**
`obs_id` stable so truth can point at it · **`scope` mandatory -> a connector must know its L2
domain** (a connector that cannot say so cannot participate in L1) · **`observed_at` comes from
the fixture -> the engine NEVER touches the clock** (determinism = testability; DHCP churn is
tested by replaying timestamps) · the engine is a **pure function**.

**`capabilities()` is false-merge prevention, not decoration:** the engine must never confuse *"no
`Uplink` because there is none"* with *"no `Uplink` because this connector is blind to topology"*.
Without it, a rule like "no diverging uplink -> same switch -> merge" merges happily. **Absence of
proof != proof of absence, encoded in the type.**

> **The punchline: `FixtureConnector` implements `Connector` by replaying JSONL. THE FIXTURE IS A
> CONNECTOR. Zero mocks. Zero network. If the trait does not allow this, the trait is wrong.**
> And it makes the "record mode" bridge trivial later: any real-world oddity becomes a fixture for
> free.

**The fixture asserts the RULE, not just the outcome** (`expect_rule`): *a test that checks only
the verdict goes green for **the right answer reached by the wrong rule** — and that engine will
break on the next fixture.* Hence the link entity carrying `rule_id` + evidence: **a rule that
fires without leaving its `rule_id` in the database is a rule we cannot debug in production.**

**Seeded generator, not real captures — decided:**
- **The generator has a free, perfect oracle: it CONSTRUCTS the devices, THEN emits the
  observations. Truth is an INPUT to generation, not an interpretation of an output.** A real
  capture has **no oracle** — hand-labelling 300 hosts takes days and will contain errors; we
  would gate on a false truth: **the worst possible test artefact.**
- Real captures **rot** (DHCP moves, devices leave); the seed produces the same byte in ten years.
- **Real captures are a privacy liability in a public repo** — MACs, hostnames (`guy-macbook`),
  the topology of Guy's home. **For an open-source project this is disqualifying. Not debatable.**
- **Real captures serve exactly ONE purpose:** a **distributional diff** to validate the
  generator's representativeness ("does this dump contain a `Fact` shape or a distribution my
  generator cannot produce?"). **Never a gate.** Representativeness becomes a *testable property*,
  not an intuition.
- **300 hosts / 36 subnets is amply small for synthetic:** at this scale the problem is not
  statistical, it is **combinatorial** — ~15 trap shapes x 2-4 instances.

> **Non-negotiable: the JSONL is COMMITTED, not generated at test time.** The generator is a dev
> tool; the artefact is versioned. **We never let the oracle be regenerated by the build of the
> code under test** — a generator bug would silently rewrite truth and the gate would go green. If
> the generator changes, the fixture diff shows up in review and truth is *seen* to change.

**The oracle, resolved:**
> **The oracle is the fixture's author, made explicit and versioned, with a mandatory `reason`
> field on every expectation. If Guy cannot write the reason in one sentence, the case is
> genuinely ambiguous -> it becomes a `MustAbstain`. THE INABILITY TO STATE A REASON IS THE
> ABSTENTION LABEL.**

That also resolves the zero-tolerance fragility: a debatable trap is **moved to `MustAbstain`**,
not argued over. Not a weakening — the honest answer.

**Build order (ATDD — the trap suite IS the red phase):** `Observation`/`Fact`/`Scope` types ->
**the ~50 YAML traps BEFORE the engine (not tests yet — the spec)** -> `FixtureConnector` -> **the
metrics harness BEFORE the engine** (*"a metric written after the engine is bent to fit the
engine"*) -> L1 join -> **the blocker + its recall assertion** -> the L2 cascade, driven green one
trap at a time -> seeded generator + committed bulk fixture -> distributional diff -> **finally**
the real UniFi/ARP connectors, with a record mode.

#### D20 — Evidence strength: conservative table assumed, under a four-condition ADR

The verdict algebra has **no notion of evidence strength**: two weak `Opposes` cancel a
`Decisive`. **Assumed as the default**, because the asymmetry justifies conservatism (false-split
is benign and additive — the operator merges; false-merge is destructive with no clean undo) —
**and assumed no longer blindly, because the falsification instrument now exists.**

**But a raw abstention rate is NOT the instrument:** *30% abstention tells you nothing — it may
mean "the data is genuinely ambiguous" (the table is right and the product is unusable for another
reason) or "the table is too conservative". **The raw rate does not discriminate the two
hypotheses, therefore it is not falsifiable, therefore it is not a measurement.*** What is
falsifiable: **abstention where the oracle decides** — per case, binary, against the
non-regenerable oracle. That is the `must-merge` column of D18.

**The criterion that discriminates "the rules are buggy" from "the algebra lacks strength"** —
both produce the same symptom:
- **Hypothesis A (rules):** unjustified abstentions have **heterogeneous** verdict vectors, each
  traceable to a rule emitting `Opposes` where its own contract says `Neutral`. **Localisable.**
- **Hypothesis B (algebra):** they **concentrate in one cell**, and — the decisive criterion —
  **every rule involved emits the CORRECT verdict per its own contract. Nobody lied. Every rule is
  right, the combination is wrong.** Test: *can I fix it by changing a rule without breaking a trap
  it currently passes?* **If every rule-level fix trades an abstention for a false merge elsewhere,
  then the same verdict carries two meanings in two contexts and the algebra lacks the vocabulary
  to distinguish them. THAT is the missing strength** — not "a case is wrong", but "the same word
  must mean two things and only magnitude separates them".

**And if strength returns, it returns as an ORDINAL, not a weight:** `Opposes(Weak) |
Opposes(Strong)`. **The enum grows, the table grows, it stays finite and exhaustively
enumerable. No float decides.** *You get your strength, I keep my table.*

> **ADR — Reintroducing weighted evidence strength.** Four **conjoint** conditions, demonstrated
> in a dedicated ADR **before any code**:
> 1. Unjustified abstention above the threshold, **reproducibly**, not on one run.
> 2. **The cause is named nominally** — the exact (rule, rule, context) triple. *"The engine
>    abstains too much" is a symptom, not a cause.*
> 3. **Exhaustion of the table is demonstrated:** no refinement of an existing verdict fixes the
>    case — typically **the rule that wrongly `Opposes` should return `Neutral`: it does not KNOW,
>    it BELIEVES it knows.** ***This is the real lock: nine parasitic abstentions out of ten are
>    that. Weighting is almost always the wrong fix for a wrong verdict — a rule that claims to
>    know what it does not know IS the bug; the weight merely masks the lie by attenuating it.***
> 4. **The weighting is local and bounded** — a named rule pair, never global; the combination's
>    output stays a `Verdict`, never a score. **A weight producing an `f64` out of the cascade is
>    an automatic reject.**
>
> Any reintroduction increments `ruleset_version`; existing links are not recomputed (they carry
> the version they were decided under). **Immediate rejection, no discussion: a PR introducing a
> weight without a satisfying ADR.**

**And the data requirement that makes the whole thing possible:**
> **The harness records, for every case, the COMPLETE VERDICT VECTOR, not just the outcome.
> Without it the A-vs-B question is undecidable after the fact — and an undecidable question IS
> the Friday-night drift. The anti-drift is not discipline, it is a data requirement.**

#### D21 — Three flags resolved by the reframe

- **`revert` DOES NOT EXIST.** There are three distinct annulments on three different objects:
  (1) undo an adoption -> reset `origin=DECLARED`; the observation never moved. (2) undo a link ->
  supersede with a `NO_MATCH` `decided_by='OPERATOR'`; nothing is erased. (3) undo an attention
  decision -> purge the `FieldDecision`; we are simply re-asked. **None is a revert: there is no
  prior state to restore, because we never destroyed state. That is the dividend of
  linked-never-merged: *annulment is an addition*. BAN the word — it carries a
  destructive-reversible intuition that is false here, and the word will eventually produce code
  that does a `DELETE`.**
- **`source_state` granularity: per `(connector, subnet)`** — the real sweep unit. *Per connector
  is too coarse: 35 subnets scanned out of 36 and `OK` is a **lie** — and **a false "gone" costs
  as much as a false merge**. Per host is too fine and **circular**: a host's state depends on its
  identity, which depends on the observations whose freshness we are trying to qualify.*
  **Consequence: an absent observation in a `STALE` subnet produces NO gap.** That is the guard
  against the false "disappeared", and it cannot be written without this granularity.
- **NO connector precedence — deliberately.** *A precedence is **a merge rule in disguise**:
  "when they disagree, believe that one and discard the other". That is exactly what
  linked-never-merged forbids, applied to sources instead of fields — accept it here and merge has
  entered by the small door, and the invariant is cracked everywhere by contagion.*
  **What replaces it: disagreement is an `Opposes`.** A blind UniFi saying "nothing here" and a
  live ARP saying "something answers" **are not in conflict — they measure different things**; it
  is a **difference of scope**, not a contradiction. **A source that cannot know answers `Neutral`
  — it does not LOSE an arbitration.** *(The same distinction as absence-of-proof vs
  proof-of-absence — and that confusion is the mother of all false merges.)* A genuine
  incompatibility -> `Opposes` -> `Ambiguous` -> ask the operator. **We do not guess. We expose.
  That is the product.**

#### Cascading implications recorded

- **The gap is computed at the `device` level**, therefore it depends on `identity_link`.
  **`identity_link` must be stable before the gap has any meaning. Identity first, gap second —
  not negotiable, it is a data dependency.**
- **Read-your-own-writes:** two observations of the same MAC **within one scan** — the second's
  resolution must see the first's link. The read pool will **not** see it (WAL shows nothing
  uncommitted; MariaDB REPEATABLE READ has a stale snapshot). -> **identity resolution runs INSIDE
  the writer actor**, against the write connection or an in-memory index the actor owns; **the
  read pool only serves the API.** *This was not true in the two-level model. It is the reframe's
  hidden cost, and it lands exactly on the writer/reader frontier.* The in-memory index survives
  the no-cache rule because **it is built at batch start and destroyed at batch end — its lifetime
  is strictly shorter than any possible writer, so there is no invalidation window. If its
  lifetime outgrows the batch, it becomes an ordinary cache and the rule applies fully. Red line.**
- **Transaction unit:** ~100 **decisions** (or 1000 rows, whichever first). **An identity decision
  is NEVER split across two transactions** — a half-written identity is a false merge with a crash
  in the middle. **A migration transaction is never chunked**; beyond `MAX_MIGRATION_ROWS` it
  **refuses and escalates**.
- The priority channel gives human migrations precedence while serialising them against the scan
  batch touching the same entity — **the single writer gives this for free. The best argument for
  the single writer so far, and it comes from the reframe, not from performance.**
- **Supertype `entity(id, kind)`** with a composite FK `(id, kind)` + `CHECK (kind='...')` — the
  disjunction is **enforced by the engine, not by convention**, and portable without a branch
  (MariaDB needs a parent index, SQLite needs a UNIQUE — `UNIQUE(id, kind)` serves both).
  *Polymorphic `(entity_type, entity_id)` = no FK possible on either engine = guaranteed orphans,
  found in production, never in test. Two tables = two copies of the logic = two behaviours.*
- **`device` has NO business columns.** *Everything a device "is" is either observed (via its
  interfaces) or declared. A device is an identifier and nothing else. **If anyone proposes adding
  `hostname` to it, they have just restored the OBSERVED/DECLARED merge we forbade.***
- **D3 amended: `updated_at` -> SCD2 `valid_from`/`valid_to`.** *A mutated row cannot answer "what
  did the human assert about entity X **before the split**?" — and without that answer, identity
  migration is destruction of evidence.* The D3 invariant survives intact: **the gap computation
  still never reads `origin`** — it reads `valid_to = OPEN_END AND state='asserted'`.
- **The NULL trap (both engines):** `valid_to NULL = current` is **wrong** — in a UNIQUE index
  SQLite **and** MariaDB treat NULLs as distinct, so `UNIQUE(observation_id, valid_to)` permits **N
  current rows: the constraint never fires, it is decorative.** And the next reflex — a partial
  index `WHERE valid_to IS NULL` — **MariaDB has no filtered indexes**: it works on SQLite, fails
  on MariaDB, or worse someone omits it and **two engines carry two uniqueness guarantees on
  identity.** -> **`OPEN_END = '9999-12-31T23:59:59.999Z'` sentinel**, which fires on both engines
  with no partial index and no branch. *(And it sorts last naturally — the unforeseen dividend of
  the ISO-8601 TEXT decision.)* Same reasoning for `NIL_INTERFACE`/`NIL_DEVICE`.
- **NO unique index on `interface.mac_canon`. Deliberate.** A cloned MAC = two real interfaces,
  same MAC. **A UNIQUE would turn the exact case we must ABSTAIN on into a 500. The uniqueness of a
  MAC is a DECISION, not a constraint — if we can express it in DDL, we have misunderstood the
  problem.**
- **`PRAGMA foreign_keys = ON` on EVERY SQLite connection** (via the pool's `after_connect` hook).
  *Without it, referential integrity vanishes **silently** on SQLite while MariaDB enforces it —
  two engines, two behaviours, on identity.*
- **`entity.state` extended:** `active | dormant | superseded | quarantined | pending_migration |
  sentinel`. `dormant` is valid only for `kind='interface' AND mac_kind='local'` — **a cross-table
  CHECK is not portable, so it is an application invariant with an explicit test.**

### Feedback to the PRD from Category 1

| # | Document | Change |
|---|---|---|
| F17 | **PRD — new FR** | **FR (new) — Ephemeral-interface lifecycle.** An interface whose MAC is locally administered and unobserved for a configurable window (default 30 d) becomes `dormant`: excluded from gap metrics and from automatic candidate generation, still queryable, retaining `first_seen`/`last_seen` and IP history indefinitely (consistent with FR38), and **returning to `active` if the MAC is re-observed**. A human-affirmed interface is exempt from the automatic transition. **Config constraint: `dormancy_window < observation_retention` — startup failure, not a warning.** |

### Category 1 (cont.) — Gap, migrations, retention, caching

#### D22 — The gap: DERIVED divergence + PERSISTED reconciliation item (hybrid)

- **The divergence is DERIVED, never stored:** `gap := declared.value != current_observation.value`,
  computed on read, with the filters D15/D17 imposed: **`AND declared_attribute.state='asserted'
  AND entity.state='active'`** (excluding `pending_migration` rows — the anti-phantom-storm guard —
  and `dormant` interfaces — the anti-metric-drift guard). **It never reads `origin`** (D3), so a
  field merged yesterday can diverge again tomorrow, automatically.
- **The reconciliation item is PERSISTED:** the commit state machine (`in_queue ->
  pending_commit(server deadline) -> committed | failed`; **Undo returns to `in_queue`**),
  `identity_link`, `FieldDecision` — **all caches of ATTENTION, all subject to the purge test.**
- **The accompanying prohibition (D2, restated because it is where the merge returns):** **no
  "effective value" view coalescing observed over declared.** *Introduce one and the merge is back
  through the service door — and users will come to believe that view IS the CMDB.*

**Why hybrid is forced, not chosen:** at 300 hosts an on-read computation is trivial, but the accept
state machine (serialised per item, server-authoritative deadline, 409/ETag) *implies* a persisted
reconciliation item. Derived alone cannot carry a lifecycle; materialised alone is a cache without
an invalidation key.

**Open, to instrument not to assert:** does the on-read derivation hold NFR2 (p95 <= 1.5 s) at 300
hosts under WAL load? The bet is yes with wide margin, but **it is a bet to instrument** — the same
discipline as NFR1: *measure the budget before optimising the wrong component.*

#### D23 — Migrations: per-dialect DDL, `sqlx::migrate!` wrapped in our own sequence

**The DDL is two files, and we assume it:** `migrations/sqlite/*.sql` + `migrations/mariadb/*.sql`.
*The "no dialect leak" rule applies **above the repository**, not to the DDL.* Trying to make DDL
portable is where the collation and index-semantics divergences would hide.

**Mechanism:** `sqlx::migrate!` for versioning + checksums + the `_sqlx_migrations` table
(boring, proven, free), **wrapped in our own sequence: auto-backup -> migrate -> verify.**

**Rationale on NFR17 (resumable after interruption — a NAS power cut mid-migration):** resumability
comes from **the backup**, not from a magically transactional migration — **DDL is not transactional
on MariaDB anyway**. The honest guarantee is: a backup is taken automatically before migration, the
migration is idempotent and versioned, and the documented rollback path is restore-from-backup. That
is what NFR17 actually asks for; pretending to atomic DDL across two engines would be a lie.

**To VERIFY in the sqlx 0.9 CHANGELOG before writing `Cargo.toml`** (Amelia's point #4 — *"the only
thing that could cost us a day"*): the `migrate!` folder format, the `_sqlx_migrations` table schema,
and the checksum scheme. **Not assumed.**

#### D24 — Retention: FR38 + D17; the temporal-history growth question stays OPEN

- Observations: FR38 (90 d configurable), rollups (first/last-seen, IP history) **indefinite**.
- Inferred entities: **D17 (`dormant`)** with the hard constraint `dormancy_window <
  observation_retention` (startup failure).
- **OPEN and unaddressed — flagged and not resolved:** the growth of the temporal history over
  months (composite identity leans on IP/DHCP history, which accumulates): **WAL growth, checkpoint
  cadence, VACUUM policy on SQLite**. Nobody has treated it. It is not blocking the identity work,
  but it must be decided before a long-running production instance exists.

#### D25 — Caching: NONE. Explicitly.

300 hosts, one operator, one binary. **Recording "no cache" as a DECISION, not an omission.**
Consistent with NFR27 (no external services) and with the session's governing allergy: **a cache
without an invalidation key is how the merge and the effective-value view come back.** The only
in-memory index permitted is the writer actor's per-batch identity index (D21) — and only because
**its lifetime is strictly shorter than any possible writer, so there is no invalidation window.**
If any future cache outlives its batch, it is an ordinary cache and the rule applies in full.

### Category 2 — Authentication & Security

#### D26 — NFR9 splits into 9a / 9b / 9c: the threat model must be enforceable

**NFR9 as written is NOT enforceable.** It says "the key lives separate from the DB volume". The
primary deployment says Docker on Synology. **Hyper Backup selects by *shared folder*, and the #1
exfiltration vector on a consumer NAS is the backup itself** — which leaves the house for C2 / a
Drive / a USB disk in a drawer. The key is "separate from the DB volume" **in the Docker mount
namespace** and **in the same file** in the artefact that actually leaves the house.

> **The separation is true at the level where we wrote it and false at the level where it counts.
> That is the definition of security theatre: a control that satisfies its own wording and
> nothing else.**

**Why an unenforceable model is worse than a narrower enforceable one** — weighed, not postured:
> Cost of an honest narrow model: the user knows what they must do themselves.
> Cost of a broad fictional model: **the user STOPS doing what they should, because we told them
> it was covered. We do not lose a protection — we destroy one that existed: their vigilance.
> That is a NEGATIVE delta.**
> And there is a multiplier: this is open source shipped to strangers via Docker Hub. **The impact
> multiplies by the number of pulls; the truth of the claim does not.**

**The three claims** (a claim that cannot be tested is not a requirement, it is a slogan):

- **NFR9a — GUARANTEED, testable.** Connector credentials are never present in plaintext in the DB
  file, a dump, the WAL/journal, the logs, or API responses. **An attacker holding ONLY the DB file
  cannot read them.**
- **NFR9b — GUARANTEED, testable.** opencmdb never writes the master key into the data volume. Key
  path and DB path are two distinct parameters, and **startup REFUSES if the key path resolves
  inside the data directory** (after symlink resolution and canonicalisation).
- **NFR9c — NOT GUARANTEED, documented as a non-guarantee.** *"If your backup tool copies both the
  key and the DB, encryption at rest no longer protects you. opencmdb cannot and will not prevent
  this."* In the quickstart, before the install — **not in a FAQ**.

> **The 9b move is the whole point: from "the key IS separate" (a state of the world, outside our
> control, untestable) to "we REFUSE to start in a configuration where it is not" (a behaviour of
> our binary, fully testable). Same move as N5 -> a truth-table column: the instrument was wrong,
> the function was right. We cannot measure the world; we can measure our reaction to it.**

**Can we test "the key is not in the backup"?** **Of OUR backup: yes, and it is mandatory** —
a byte-grep of the whole archive for key material and plaintext credentials (raw bytes, not field
inspection). **Of Hyper Backup: no, and any attempt is theatre** — we would be mocking Hyper Backup
and testing our mock; worse, **that test could not fail for any reason at all: it would always
pass. A gate that cannot fail is a gate that has never caught anything, and it taxes every other
gate.**

#### D27 — Key provisioning: a KEK file in a SEPARATE DSM shared folder

The real setting: Container Manager writes the compose into `/volume1/docker/opencmdb/`; the
operator maps `./data:/data`; **Hyper Backup then backs up the `docker` shared folder — entire,
compose included. That is what a careful person does by default.**

| Option | Verdict |
|---|---|
| **Env var in the compose** | Key in cleartext in a file Hyper Backup carries away. **Total theatre.** And independently: **it leaks into `docker inspect`, `/proc/<pid>/environ`, and Synology logs.** Rejected |
| **Secret file inside the data volume** | *The key travels in the same tarball as what it protects.* Rejected without discussion |
| **Synology keystore** | **There is no usable one** — DSM's Key Manager unlocks encrypted shares; it exposes no API to a container. Proprietary, untestable in CI, unavailable for the native binary, and broken by the next DSM release. **Exactly the budget of unknowns we refuse to spend outside the domain.** Rejected |
| **Startup prompt** | Technically cleanest — **and it kills "install-and-forget"**: after every NAS reboot or power cut, *a dead service for three days while the operator is on holiday.* Rejected **as the default**; kept as an **option** (`..._KEY_STDIN`) |

> *"None is clean. Saying it a fifth time does not manufacture a fifth option."*

**DECIDED:**
```yaml
volumes:
  - /volume1/docker/opencmdb/data:/data
  - /volume1/opencmdb-secrets:/secrets:ro   # <- a SEPARATE DSM shared folder
environment:
  - OPENCMDB_MASTER_KEY_FILE=/secrets/master.key
```
> **The point that does all the work: Hyper Backup is configured PER SHARED FOLDER. Making the
> secret a first-level shared folder puts it in a selection unit the operator SEES, ticks or
> unticks explicitly in the wizard. We have not hidden it in a subdirectory that a `docker/**`
> swallows silently. That is the only thing that actually moves the needle: making the exclusion
> VISIBLE AND ATOMIC rather than drowned.**

- **Auto-generated at first boot** if absent, written `0600`, its absolute path logged loudly (a
  security event, FR49) and shown on the health page — *"we make the fact observable by the human
  ticking the boxes in Hyper Backup. That is the only intervention we have on this risk, and it is
  INFORMATION, not control. Do not disguise it as control."* First value < 15 min preserved, zero
  ceremony.
- **Startup refuses** if the key path resolves inside the data volume, or if the file is
  group/other-readable. *"Twelve readable lines, and it catches the most likely fault: the person
  who 'simplifies' the compose into a single volume. **We cannot prevent a bad Hyper Backup config;
  we can prevent the bad config we can see.**"*
- **Same class of decision as NFR13/TLS:** we do not provide it, we document the operator's
  responsibility. **Consistency.** *"We will not pretend to defend the local-root case. We could
  derive from a passphrase, do something TPM-ish, obfuscate — that would be theatre, and **theatre
  is worse than a narrow frontier we actually hold. A threat model we cannot enforce is a delayed
  lie.**"*

| Scenario | Protected? |
|---|---|
| DB exfiltrated alone (dump, `.db` copied, MariaDB stolen) | **Yes** — the key is not in it |
| Backup stolen, secrets **excluded** | **Yes** |
| Backup stolen, secrets **included** | **No** — documented (9c) |
| Local root on the NAS | **No** — NFR9 already says so |

#### D28 — Envelope encryption (KEK/DEK). KEK rotation only at MVP.

**Field-level encryption** (D-level, earlier): SQLCipher is SQLite-only, MariaDB at-rest is
engine-level — **there is no identical encryption strategy across both backends**, so we encrypt
the crown jewels at field level with an out-of-volume key.

**The envelope is near-mandatory, and the argument is not cryptographic:** without it, rotation =
decrypt/re-encrypt **every field**, in a transaction, with a half-migrated state if the NAS dies
mid-way — **a crash-resistant data-migration engine, on two backends, for an operation the solo dev
may run three times in his life. The cost of testability exceeds the feature.** With it: **rotation
= re-wrap ONE DEK. One write. One record.**
> **It is not sophisticated crypto — it is error-handling code we do not write.** A
> developer-productivity argument, not a cryptographic one — which is why it wins.
> And: *"AC12-5 goes from 'prove an invariant over a partial migration' to 'prove one write is
> atomic or rolled back' — from N outcomes to two. **That is what makes it exhaustively testable
> instead of sampled.**"*

**It changes what "the key" means — and that fixes D27's problem:** the out-of-volume file holds the
**KEK**, which encrypts exactly one thing: a ~60-byte wrapped **DEK stored IN the DB**.
- **KEK rotation does not touch the data** beyond one row. We can do it often, without fear.
- **FR48 (secret backup/restore)** becomes: export the DEK wrapped under an operator-chosen
  passphrase — **a short blob, copyable into a password manager. Not a database dump.**
- **Restore on a new machine:** new KEK, unwrap the DEK with the passphrase, re-wrap. The data never
  moved.
- **Invariant: the DEK exists in cleartext only in RAM.**

**What the envelope BREAKS — named, not glossed:**
1. **A new invalid state exists: an orphan DEK** (wrapped by a vanished KEK). **Detected at
   STARTUP**, not on the first credential read three hours later during a reconciliation — *a silent
   failure here turns a config problem into data corruption found too late.* **Fail fast, fail loud.**
2. **Two artefacts to back up.** The wrapped DEK belongs *in* the backup (useless without the KEK);
   the KEK never is. **A backup with the DEK and without the KEK is undecryptable — which is the
   intended behaviour, but that is only true if a test verifies it. Otherwise it is an assumption.**
3. **Versioning becomes necessary:** which KEK wrapped this DEK? **Without a KEK identifier, a
   pre-rotation backup is undiagnosable — you cannot tell "wrong key" from "corrupt archive". The
   envelope adds that requirement; it also adds the only clean place to put it.**
4. **DEK rotation != KEK rotation.** **At MVP we expose KEK rotation ONLY.** DEK rotation (which does
   require re-encrypting every field) is the rare case and does not ship until AC12-5's full test
   budget exists. *"Shipping an operation whose failure paths we cannot all test is shipping a time
   bomb with a button on it."*

> **Net: the envelope REMOVES the catastrophic failure mode (interrupted rotation = partially
> unreadable data) and ADDS failure modes that are diagnosable and testable at startup. Trading a
> silent-corruption failure for a loud startup failure: signed, every time.**

**The read rule that makes a crash survivable:**
> **Always decrypt via `secret.dek_id`. NEVER via "the active DEK". Consequence: a partially applied
> rotation is READABLE. That is the property that makes the crash non-fatal** — not a sentence we
> wrote, a property AC12-5 proves.

**Secrets are NOT SCD2** — *"keeping past versions of a secret means keeping more loot to steal for
zero business value."* `secret` is mutable in place. What is append-only is `security_event` (FR49),
**and it carries neither ciphertext nor nonce — only ids, actor and timestamp: the journal must not
become a second copy of the loot.**

**AAD binds ciphertext to its context:** `aad = owner_kind || owner_id || field_name || aad_version`
— **without it, an attacker with DB write access moves a ciphertext from one row to another.**

#### D29 — Tokens hashed (SHA-256); Argon2id for passwords only

> **Argon2 on a bearer token is a CATEGORY ERROR.** *Argon2 exists to compensate an entropy deficit.
> A token we generate has 256 bits — **there is nothing to compensate.** Brute-forcing SHA-256 over
> 256 bits of entropy is not "expensive", it is **impossible**; slowing each attempt by 10^5 changes
> nothing against a 2^256 search space. **And the cost lands on the wrong side: the `/metrics` scrape
> token is verified on every Prometheus scrape — putting 64 MB of Argon2 on that path is DoS-ing
> ourselves against NFR18, for free.**

- **Tokens (bearer, scrape, session): SHA-256**, constant-time comparison, **no salt** (*"salt
  protects against rainbow tables, which do not exist over 256 random bits"*), direct indexed lookup
  on `token_hash`.
- **Encrypted vs hashed: hashed, settled.** *"Encrypting a token makes it re-readable. **A
  re-readable token is a token stealable from a backup.** There is no business reason to re-read a
  bearer token: auth is a COMPARISON, not a READ."*
- **Passwords (FR46/FR47): Argon2id**, `m = 19 MiB, t = 2, p = 1` (RFC 9106 "second recommended") —
  chosen not because 64 MiB would break NFR18 at p=1, but because login sits on the NFR2 path.
  **Params stored inside the PHC-encoded hash** (`$argon2id$v=19$m=19456,t=2,p=1$...`), never in
  separate columns — that makes on-the-fly re-hashing possible without a migration.
- **UNKNOWN, to be measured not assumed:** the real Argon2id time on the target Synology CPU (*"a
  Celeron J4125 is not a desktop"*). Dedicated AC: **< 300 ms per hash, measured**.

#### D30 — Sessions: DB-backed, reusing the `deadline_at` + sweep pattern

Signed stateless cookie **rejected**: no revocation, and NFR11 + multi-user-ready-from-day-1 make
revocation non-negotiable. **DB-backed**, and it is **exactly the `pending_commit` pattern already
built**: `deadline_at` as ISO-8601 TEXT computed in Rust, a periodic **low-priority bounded
idempotent sweep** on the writer actor's channel, `now()` **bound as a parameter from Rust** — never
`datetime('now')`/`NOW()`. The liveness check (`revoked_at = OPEN_END AND deadline_at > now()`) is
**evaluated in Rust after the SELECT**, not in the WHERE — the comparison never descends into SQL.

The cookie carries the raw 256-bit token (`HttpOnly`, `Secure`, `SameSite=Lax`); the DB stores
`SHA-256(token)`.

#### D31 — Crypto crate: OPEN (`age` vs pure RustCrypto)

**A genuine disagreement, recorded rather than papered over.**

**For `age` (wrapping only, never per-field):** reusing `ring` (already in the tree via
`tls-rustls-ring`) gives raw AEAD but **not** the envelope format, key wrapping, or passphrase
derivation for FR48 — so we would **write the envelope format ourselves**: serialisation, nonces,
versioning, associated data, derivation. *"Precisely the kind of code where 'twelve readable lines'
is a lie — it is two hundred lines I cannot validate, **in the one domain of this project where an
error is SILENT. The compiler says nothing. The tests pass. And it is broken.**"*
> **"The budget of unknowns is not a budget of dependencies — it is a budget of things I can be
> wrong about without knowing it. Home-made crypto is the most expensive line in that budget, even
> if `Cargo.toml` does not grow by one line. That is where my own doctrine, applied mechanically,
> gave the wrong answer."**
Plus a product argument: `age` has a **documented format and a standard CLI** -> *"the operator can
verify their secret backup **without opencmdb**. **A blob only my binary can read is not a backup,
it is a hostage.**"*

**For pure RustCrypto (`chacha20poly1305` + `argon2`):** **XChaCha20 gives a 192-bit random nonce ->
the nonce becomes a non-problem** (no counter, no persisted state, no crash-safe counter to reason
about — `ring` does not expose it, and AES-GCM's 96-bit nonce would force a counter). **No guaranteed
AES-NI on the target Synology CPU -> ChaCha is faster without it.** `argon2` from RustCrypto is
needed anyway (ring has no Argon2id) — one crate family rather than two. And: *"`ring` is already in
the tree, but **'already there' is not a design argument** — it is a binary-size argument, and the
delta is negligible."* On `age`: *"a file format, recipient/PKI-oriented, per-field format overhead.
We encrypt 20 fields, not files. Wrong tool."*

**Note: they partly talk past each other** — the `age` proposal is for **wrapping only** (DEK under
KEK, DEK under passphrase for FR48), never per-field; the RustCrypto objection answers a per-field
use that was not proposed. **The real question: is wrapping a 32-byte DEK + deriving a passphrase for
FR48 trivial enough in raw XChaCha20, or is that exactly where the silent bugs live?**
**To be settled with verification (age's maintenance, versions, fitness for DEK wrapping) before any
code.**

#### Verification list before writing `Cargo.toml` (Category 2)

1. **🔴 `TEXT` columns in MySQL cannot be indexed without a length prefix — this hits `UNIQUE
   (token_hash)` directly**, i.e. the auth path. The answer is probably `VARCHAR(n) CHARACTER SET
   ascii COLLATE ascii_bin` on MariaDB and `TEXT COLLATE BINARY` on SQLite -> **two divergent DDLs
   for one logical type.** *"It is verified, not guessed."* **Check this FIRST.**
2. sqlx 0.9 CHANGELOG: TEXT/BLOB behaviour differences between the two drivers.
3. `chacha20poly1305` + `argon2` versions, MSRV, and **`zeroize` on key types** — verify the DEK in
   RAM is zeroized on Drop.
4. MariaDB index length limits (767/3072 depending on row format) — a non-issue in `ascii_bin`, to be
   confirmed against **the actual MariaDB version on the target Synology, which is UNKNOWN.**
5. Whether the Synology keystore is reachable from a container — **unknown; would not build on it
   without proof.**

#### Security gates in CI at MVP — ranked

**Blocking, every PR:** the NFR12 suite (AC12-1..8) **on both backends** — *"the only gate that tests
OUR security property; everything else tests other people's code"* · **the byte-grep of the backup
artefact** for plaintext leaks — *"cheap, deterministic, catches the most embarrassing class of bug
(a forgotten field, a debug log). This is the gate that will actually catch something one day, and
probably more than once"* · **AC-9b: refuse to start if the key is in the data volume** · `cargo-deny`
(advisories + licences) · `cargo clippy -D warnings` (*the net that catches `unwrap()` in crypto
paths*) · **our AUTHORIZATION MATRIX**: session cookie vs bearer vs scrape token across every surface
class, **including the crossed cases — does the scrape token reach `/api/credentials`? The answer
must be a test, not an assumption.**

**Border:** grepping logs and API responses for seeded secrets — **blocking on the API surface**
(a real surface, FR47/NFR10), **informational on logs** (it degrades into whack-a-mole).

**Theatre — named:** SBOM at MVP · `cargo outdated` that blocks nothing · **a Docker image scanner
(Trivy/Grype) on a static Rust binary in distroless** — *"signal tends to zero, noise tends to
CVE-of-the-month-in-a-libc-we-do-not-have. **Weak position, revisable** once the base image grows"* ·
**a pentest/DAST of the MVP** — *"a solo dev triaging DAST noise is time stolen from AC12-3..7"* ·
auditing dependency maintainers · **"testing that axum authenticates our routes"** — *"axum routing
correctly is axum's problem."*

### Feedback to the PRD from Category 2 — F22–F25: APPLIED, and the snapshot is gone

_F22–F25 were raised here, applied to `prd.md` on 2026-07-16, and **the rows were removed on 2026-07-17**: none was referenced by any decision, and a table of completed work sitting inside a decision register reads as open work. **The change itself is not lost — it is in `prd.md`'s `editHistory`, which is the only source that records what a requirement now SAYS.** This document records what was DECIDED; the PRD records what the requirements ARE._

### Category 3 — API & Communication

#### D32 — `source_state` is TWO ORTHOGONAL AXES, not three states

**The contradiction resolved** (all three chairs reached it independently): FR5 says `full / degraded /
offline`; the locked pillar says `source_state ∈ {live, blind}`. **They are not competing enums —
they are two dimensions flattened into one, and the flattening is exactly what makes FR5/FR7/FR19
unimplementable together.**

- **`live` / `blind`** answers *"did I receive an observation for this scope in this window?"* — a
  question about **data arrival**. Temporal, changes every poll, governs **D9 + FR19**.
- **`full` / `degraded`** answers *"what is this source able to observe right now?"* — a question
  about **the extent of the descriptor**. Structural, governs **D13 + FR7**.
- **`offline` is the portmanteau word.** *"It is the point where someone wrote `blind` into the
  capability column. **It is not a third level of degradation — it is the OTHER axis that slipped
  into the enum.** That is why the two models look incompatible: they are, because **one of the
  three members belongs to the other list.**"*

```rust
struct SourceState {
    liveness: Liveness,          // Live { last_ok } | Blind { since, cause: BlindCause }
    capabilities: Capabilities,  // the CURRENT descriptor — not a level
}
```

| | liveness: **Live** | liveness: **Blind** |
|---|---|---|
| **Full** | complete observations | last-known retained, **NO gap, NO alert** |
| **Reduced** | **the missing cell: ping-only** — it answers perfectly, it just sees less -> partial observations, **no gap on out-of-capability fields** | last-known retained; the capability loss is moot — nothing arrives |

> **The cell the PRD cannot express is `Live + Reduced`.** With a single axis you must call it
> `degraded` — and then `degraded` means *both* "half the packets are lost" (degraded liveness) and
> "I lost NET_RAW" (reduced capability). **Two causes, two operator actions, one word. The same
> error as `anyhow::Error`, one storey up.**

**`full / degraded / offline` SURVIVES — as a UI PROJECTION, never a stored state.** `Blind -> offline`
· `Live + Reduced -> degraded` · `Live + Full -> full`. *"The operator wants three colours; the engine
cannot afford three colours. We keep the PRD's vocabulary at the surface and refuse to let it descend
into the core."* **Corollary, hard: `degraded` is NEVER stored — it is a pure function of the
capability descriptor. Storing it would create two sources of truth that diverge on the first bug.**

**Cost, accepted:** two axes = more cells to test. *"The price of flattening is a false 'gone'; the
price of orthogonality is three more test cases. Not a hard trade."*

#### D33 — `ConnectorError`: a closed taxonomy. Never `anyhow`.

**The admission rule** (converged): **a variant exists only if it produces a `(source_state, operator
action)` pair that no other variant produces.** Not one variant per technical cause. If two errors
lead to the same state and the same human gesture, they are one variant and the detail goes in the
payload. *"We do not have a taxonomy of errors — we have a taxonomy of questions the source did not
answer. Design it as 'things that can go wrong' and you get `anyhow`."*

| Variant | liveness | Gaps (FR19)? | Operator action |
|---|---|---|---|
| `Unauthorized` / `AuthFailed` | **`Blind{AuthRejected}`** | **suppressed** | *"Rotate the UniFi API key"* — **precise, at 3 a.m. This is Journey 4** |
| `Unreachable` | `Blind{NoResponse}` | **suppressed** | *"Check connectivity"* (DNS/refused/timeout are ONE variant — one operator action) |
| `SchemaMismatch` / `SchemaDrift` | `Blind{Unparseable}` | **suppressed** | **the only case aimed at the maintainer, not the operator** (NFR8); carries an OS-version hint |
| `Timeout` (budget exceeded, FR6) | `Blind` | **suppressed** | none — a metric, not an alert |
| `Cancelled` | **unchanged** | n/a — nothing written | none |
| `RateLimited` | **`Live`** (it is talking!) | suppressed this cycle | **none** — backoff, silence |
| `RemoteFault` (5xx) | `Blind{RemoteError}` | **suppressed** | after N failures |
| `CapabilityLost` | **`Live`** | **no gap on lost fields, normal elsewhere** | *"Grant NET_RAW"* |
| `Misconfigured` | `Blind{NeverStarted}` | **suppressed** | at startup, not at 3 a.m. |
| **`ImplausibleResponse`** | `Blind` | **suppressed** | **the only net against silent drift — see D35** |

**Every variant carries** `scope` (**the field everyone forgets, and the one that makes D21 exist** —
without it, one subnet's outage blinds all 36 or none) and its payload; **the discriminant is
machine-readable, the payload is human-readable** — *"if the engine matches on a string from the body,
that is `anyhow` with extra steps."*

**Three properties that make this the decision, not plumbing:**
1. **The "gaps" column says only one thing: NO. Everywhere.** *"NFR7 is not a test we hope to pass —
   it is a **structural consequence of this `match` being exhaustive**. The day someone adds a
   variant, the compiler demands the table row. **`anyhow::Error` wakes nobody; a non-exhaustive
   `match` does not compile.**"* Implementation of the safe default:
   `fn is_blinding(&self) -> bool { !matches!(self, Cancelled) }` — **a future non-blinding variant
   must justify itself before NFR7.**
2. **`Unauthorized` and `Unreachable` share a liveness verdict and have OPPOSITE actions.** That is
   the split criterion. *"Merging them tells Guy 'something is wrong with UniFi'. Splitting them tells
   him 'your key was rotated by the firmware'. **That is the difference between a product and a
   dashboard.**"*
3. **`RateLimited` and `CapabilityLost` are `Live`** — errors that are not outages. A flat taxonomy
   cannot express that.

**`Unauthorized -> Blind` is deliberate and counter-intuitive:** the controller *responds*, it is up.
But it gives us **no observations** — and **liveness measures data arrival, not the remote peer's
health. A source that answers "no" is blind to us.** Mark it `Live` and the absence of 84 devices
becomes legitimate -> 84 alerts. **`Unauthorized` is the variant that defines the product.**
`Cancelled` is the only one that writes nothing: *"if `Cancelled` set `blind`, a clean shutdown would
blind every source and FR19 would suppress everything at restart"* — **necessary, or it would disguise
itself as `Timeout` and take `blind` wrongly.**

**The line nobody crosses:** *"`ConnectorError` will never be `anyhow::Error`. Not because types are
nice. Because `anyhow` here makes FR5, FR8, FR19 and NFR7 **literally inexpressible** — you cannot
suppress alerts on a condition you have not named. **Seven variants cost an afternoon. The missing
variant costs 84 alerts at 3 a.m. — and the only thing this product sells: that you can believe it
when it stays silent.**"*

**Open:** the exact variant count (5 / 7 / 6+`ImplausibleResponse`) — the differences are `RateLimited`
(no evidence the local UniFi API rate-limits — `UNKNOWN`, add it when a fixture shows it),
`CapabilityLost` (an *event*, not a state — in steady state ping-only is an `Ok` with a reduced
descriptor, not an error), and `ImplausibleResponse`. **They converge on what matters: the default is
safe, `Cancelled` is the exception, and the split serves the operator, not the engine.**

#### D34 — Three forced corrections to the `Connector` trait

All three are forced by D19's rule: **the fixture IS a connector — and every time it could not be, the
trait was wrong, not the fixture.**

**(1) `capabilities()` is DYNAMIC — and it leaves the trait.**
> **"`capabilities()` never had to be *constant*. It had to be *known at ingestion*. That is not the
> same requirement — and it is the whole difference between a CONSTANT and a DATED FACT."**
> *"The invalidation key already exists: **it is the poll.**"*

Capabilities **travel with the batch**. *"A capability IS an observation: 'NET_RAW absent' is a fact
dated by the source, exactly like 'MAC aa:bb seen'."* And the decisive argument: **the fixture replays
it for free** — one JSONL line reproduces a mid-scan NET_RAW loss, **zero mocks**; with a separate
getter the fixture would need state outside the JSONL. **`fn capabilities(&mut self)` is also refused:
it implies probing; probing means network in a getter, i.e. network the fixture must simulate outside
`poll()` — which breaks D19.**

*"The connector is no longer the authority — **the poll is.** That is a shift of authority, and it is
the real content of the decision."*

**Persisted AND recomputed — and it is not a cache:** recomputed by the source every poll (authority);
persisted as **the last known fact**, dated — *"same status as a device's `last_seen`. **No
invalidation key because there is nothing to invalidate.** The question 'what is the invalidation key'
presupposes a cache; I answer by refusing the premise."* Its two uses: the UI must show capabilities
**while the source is blind** (FR19: retain last-known), and **downgrade detection (FR5) is a diff**
`caps(N-1) -> caps(N)` — which needs N-1 somewhere.

**The latent D13 bug this exposes:** the doubt predicate must be evaluated **against the BATCH's
descriptor**, not the current one — `authority(source, field, at) = capability_at(source, at).field
AND ...`. *"Otherwise an observation ingested when NET_RAW existed would be re-evaluated later with a
ping-only descriptor and become retroactively `SourceNotCapable`. **The past would change status.**"*
Same doctrine as everywhere: **we never reinterpret an observation with a context it did not have.**
**No new `DoubtReason` is needed** — `SourceNotCapable` already covers it; whether it has been true
forever or since 14:02 does not change the verdict, only what we show the operator. **The predicate is
stable; the explanation gets richer** — and `SourceNotCapable` carries `since: Timestamp` so we can
tell "MAC never observable" from "MAC observable until t1" (the second must **retain** the last MAC,
not deny it).

> **The real danger, named: without the capability<->batch link, the engine sees an absent MAC and
> reads it as "the MAC disappeared" — that is a FALSE MERGE, exactly what `capabilities()` existed to
> prevent. Same doctrine one storey down: absence of proof != proof of absence — including when it is
> THE SENSOR that shrank, not the network that changed.**

**(2) `poll()` becomes async with a `sink` and a `cancel` — the cancellation hole.**
> *"A synchronous signature returning a complete `Vec` has **no cancellation point** and **no partial
> result**. A 120 s sweep killed at 119 s **throws away 119 s of valid observations.** Under NFR1 that
> is a design defect, not a detail."*
> *"And `async fn poll` alone is not enough either: **`tokio::time::timeout` drops the future, `Vec`
> included. Clean cancellation, TOTAL LOSS. Worse than sync: it looks correct.**"*

```rust
#[async_trait]
trait Connector {
    fn id(&self) -> ConnectorId;
    async fn poll(&mut self, now: Timestamp,
                  sink: &mut dyn ObservationSink,   // incremental emission
                  cancel: CancellationToken)        // cooperative cancellation
        -> Result<PollSummary, ConnectorError>;     // capabilities + scopes_covered
}
```
Observations already emitted are **true** — `observed_at` comes from the source, they do not expire
because the poll was cut. The connector chooses its cancellation points (between probes, never
mid-probe). `Cancelled` -> `source_state` **unchanged** -> no gap -> NFR7 holds.
> **"With the original trait, cancellation was UNTESTABLE — therefore untested, therefore wrong in
> production. That is the sharpest demonstration that the trait was wrong: it was not an ergonomics
> problem, it was a COVERAGE HOLE ON NFR7."**

**(3) `(connector, subnet)` -> `(connector, SCOPE)` — D21's granularity is retracted.**
The UniFi connector does not scan subnets; it polls a controller. `(connector, subnet)` on UniFi gives
either `subnet = NULL` (**a guaranteed portability bug**: NULL in a key) or a `0.0.0.0/0` sentinel (a
lie).
> **A SCOPE is the smallest set that can go blind without the others going blind.**
Scanner -> **one scope per subnet**. UniFi -> **one scope, `controller`**. *"That is why subnet worked
for the scanner: **it WAS the scope, by coincidence of shape.**"*
**`source_capability` is per CONNECTOR, not per scope** — `NET_RAW` is a property of the process, not
of a subnet; per-scope would write the same row N times and invite incoherence. (Additive migration if
a scope-dependent capability ever appears — no known example.)

**Coalescing (FR6) lives in the SCHEDULER, exclusively.** `PollSlot ∈ {Idle, Running,
RunningWithPendingRerun}`: a request during `Running` flips a **boolean, not a queue**. Ten requests ->
exactly one re-poll. **"No stacking because there is no container that can grow. The structure forbids
the bug; we do not rely on discipline."** The **per-scope total budget** is the scheduler's (a uniform
policy, it wraps `poll` in `tokio::time::timeout`); the **per-host probe timeout** is the connector's
(the scheduler does not know what a host is).

#### D35 — NFR7 is a TYPE, not a test. NFR8's "does not crash" is an admission.

**NFR7 (0 false device-gone):**
> **"`Observation` must be INCAPABLE of expressing 'gone'. It only says 'here is what I saw'. Absence
> is DERIVED by the engine, and the engine only derives when `liveness == live`."**
> **"The `_ =>` arm in the reconciler is the EXACT place where NFR7 dies. Wildcards forbidden in the
> engine — a lint, not a review."**
> *"The cheapest NFR7 test that exists: **make the bug not compile.**"*

**NFR8 — the proof that the requirement is empty:**
> **"The most dangerous product imaginable — the one that marks 84 devices 'disappeared' at 3 a.m. and
> fires 84 alerts — PASSES NFR8 as written. It did not crash. It ran perfectly. It lied with an
> impeccable uptime. A requirement your worst-case scenario satisfies is not a requirement."**

**The real, falsifiable claim, in four assertions:**
- **(a) Monotone honesty:** for any injected fault, `device_facts(faulted_run) ⊆ device_facts(clean_run)`
  on the same fixture. **A fault can only REMOVE knowledge, never ADD an assertion.** The clean run is
  the oracle — **differential, no magic number.** *Source*-facts may grow. **That is Journey 4 in one
  line: 84 device-facts -> 0, 1 source-fact -> 1.**
- **(b) Bounded blast radius:** a fault scoped to one `(connector, scope)` modifies no other's state.
  36 subnets, inject into one, assert the other 35 intact. **"That is what D21 buys, and nobody ever
  tests it."**
- **(c) Convergence after recovery:** once the fault lifts, `state(clean @ t_n) == state(faulted_then_
  recovered @ t_n)` within N polls. **"The classic hysteresis bug is a one-way door: easy to enter,
  impossible to leave. This is where we catch it."**
- **(d) Exactly ONE actionable notification**, naming the variant and the scope. *"At 300 hosts the
  only measurable thresholds are zero and one."*

**Two-layer fault injection — and the most insidious theatre of the session, caught in its own
proposal:**
> **"Injecting `SchemaDrift` at layer A tests NOTHING. You assert that the engine handles a SchemaDrift
> you handed it — you have not proved that the PARSER PRODUCES one when UniFi renames a field. That is
> theatre, and it is the most insidious of the session because it LOOKS like fault injection."**
- **Layer A — `FixtureConnector`** replays `Result`s (401, timeout, partial). **Tests the engine.**
- **Layer B — `FixtureTransport`** replays **raw bytes under the real UniFi parser**. **Tests
  bytes -> correct variant. Drift lives here.** ~30 mutation fixtures (field deleted / null / retyped /
  renamed) **generated, not hand-written**, from a real captured 3.x body. Zero-privilege = ~6 fields on
  the presence path = a tractable matrix.

**The real terror, worse than Journey 4:**
> **"The 3 a.m. 401 is the EASY failure. The hard one is a firmware that renames `mac` to `macAddress`,
> your `#[serde(default)]` returns an empty vec, and the engine is LIVE, CONFIDENT, AND WRONG. The 401
> fails loudly. The drift can fail SILENTLY."**
-> **`#[serde(default)]` FORBIDDEN on any collection feeding presence** (greppable gate, zero cost).
**But honestly, that is not enough**: the parser *cannot* tell `[]` from a silent drift — an empty
subnet is a legitimate answer. Hence **`ImplausibleResponse`**: **"84 -> 0 in one poll is not 84
departures, it is an implausible response."** The engine reclassifies a population collapse as a
**source event**, not a device event. **The threshold is a product decision, not a magic number** — but
it is the only defence against silent drift.

**Fixture provenance discipline:** **capture** fixtures (real UniFi, version-tagged, dated, with a
re-capture job) prove the **parser**; **scenario** fixtures (synthetic, "written to trap X") prove the
**engine**. **Never run the re-capture job on scenario fixtures — they do not rot; they are right or
wrong. Different rot risk, different treatment.**

#### D36 — Dynamic capability vindicates the verdict vector

**Same data, different capability, different verdict: CORRECT. Non-negotiable.**
> *"The verdict was never a function of the data alone — it is a function of the data **and of what the
> observer could see**. Making capability dynamic does not change the predicate, **it reveals its true
> arity. A predicate whose input moves and whose output does not — THAT would be the bug.**"*
>
> **"The trap is lexical: we require REPRODUCIBILITY, not STABILITY. Replay `(data, capability)` -> the
> same verdict, always. The verdict is allowed to change over time. Requiring stability means pinning
> capability, i.e. reintroducing the false merge. Anyone who 'fixes this flake' by pinning the
> capability has broken the product to make CI green."**
>
> **"A verdict without its capability snapshot is UNFALSIFIABLE: you cannot tell a regression from a
> legitimate re-derivation."** -> the harness records `{verdict, reason, capability_snapshot,
> source_state, fixture_seq}`. **Schema corollary: persisting verdicts without their snapshot makes the
> database uninterpretable after a downgrade. Two verdicts are comparable only under an identical
> snapshot — otherwise they are not two answers, they are two questions.**

**Lattice monotonicity — the law that makes it testable:** **losing a capability can only move a verdict
TOWARD doubt, never toward certainty.** `C' ⊆ C ⟹ verdict(C') at least as doubtful`. With 4-6
capabilities that is 2^6 = 64 subsets x the fixture bank — **exhaustive, in seconds.**

#### Category 3 — theatre named

**wiremock / httpmock** — *"the fixture IS the trait. Adding a network stack to a test that must never
touch the network is testing `reqwest`."* · **chaos engineering / toxiproxy at MVP** — *"**the chaos
here is the vendor's schema, not network jitter — and toxiproxy cannot rename a field.**"* · testing
`Display`/`std::error::Error` on `ConnectorError` (*testing the derive macro*) · **a "degraded mode"
test asserting the process still runs** — *"`assert!(true)` with a heartbeat"* · **fuzzing the parser at
MVP** (weakly held) — *"arbitrary bytes prove you do not panic on garbage UniFi will never send. **The
mutation set derived from a real body is worth 100x at a tenth of the cost.** After, not before."* ·
**Nuance, not rejection:** retry/backoff tests on the tokio clock test the library's timer — **but the
POLICY is ours and depends on the variant** (`AuthFailed`: **never retry — you would DoS your own
controller with a dead key**; `Unreachable`: always). **Test the `variant -> decision` table, pure, no
clock.**

> **"If I had to reduce it all to one assertion: the faulted run cannot invent a single fact.
> Everything else is observability."**

#### Category 3 — remaining decisions (leans, not yet party-tested)

- **JSON envelope & errors:** the UX's locked forbidden-word lint (`Error`, `Failed`, `Invalid`) and
  "never blame the user" **apply to the UI only, not the JSON API** — an API talks to machines, and
  FR43's audience scripts directly against it. **Decided by default; flagged so it is a decision, not
  an accident.**
- **Webhook delivery (FR31/FR32):** **at-most-once with a bounded retry** (a small fixed number, then
  drop with a logged security/ops event). *A webhook that fails silently betrays Journey 7's "quiet
  until it matters"; a webhook that retries forever is a queue in disguise inside a binary that swore
  "zero external services" (NFR27).* **Not party-tested — revisit.**
- **`/metrics` crate:** to select and version-verify. The scrape token belongs in the authorization
  matrix (D26's CI gate: **does the scrape token reach `/api/credentials`? The answer must be a test**).
- **Deep links:** canonical URI keyed by entity UUID; `external_base_url` with tolerant fallback +
  visible warning; deleted/stale objects -> **tombstone, never "not found"** (already locked in the UX).

#### Category 3 — open questions

- **Hysteresis `blind -> live`:** does a single OK poll suffice? **A flap creates a gap on return.**
  Neither FR5 nor NFR7 says. *"I will not invent a threshold — a product question."*
- **The `ImplausibleResponse` threshold** — a product decision.
- **Per-ENDPOINT blindness:** UniFi drops `/stat/device` while `/stat/sta` answers. *"An endpoint is
  neither a scope nor a capability — or it is a capability, and `scopes_covered` should be
  `endpoints_covered`. **I cannot settle it without NFR8's version-tagged fixtures.** Do not freeze it
  before we have them"* — and it touches the schema.
- **Coalescing key:** per connector or per `(connector, scope)`? Depends on whether one poll covers all
  scopes or one.
- **Declared exception:** `last_error_at` must come from the engine clock (a blind source provides no
  clock). **Safe only because it participates in NO reconciliation decision — it is display. If anyone
  reads it in the engine, the rule breaks and it must be raised.**

### Feedback to the PRD from Category 3 — F26–F32: APPLIED, and the snapshot is gone

_F26–F32 were raised here, applied to `prd.md` on 2026-07-16, and **the rows were removed on 2026-07-17**: none was referenced by any decision, and a table of completed work sitting inside a decision register reads as open work. **The change itself is not lost — it is in `prd.md`'s `editHistory`, which is the only source that records what a requirement now SAYS.** This document records what was DECIDED; the PRD records what the requirements ARE._

### Category 4 — Frontend Architecture

_Mostly pre-decided by the UX spec (HTMX + Askama + Tailwind, polling, optimistic UI, tokens,
components, morph swaps, focus management). **State management: N/A** (server-rendered). **Bundle
optimisation: N/A** (no JS build). Two real decisions remain._

#### D37 — The JS asset pipeline: committed, `rust-embed`'d, version-pinned. Never a CDN.

**There IS a JS pipeline, despite "no JS framework".** The UX locked **HTMX** (~14 KB), **idiomorph**
(`hx-swap="morph"`) and mentioned **hyperscript** (keyboard bindings). **Those are three JavaScript
dependencies.**

**Decision — treat them exactly like the CSS (D-level, step 3):** versions **pinned**, files
**committed** to the repo, embedded with **`rust-embed`**, **never a CDN**. Rationale, all four
independent: an **offline Synology/Docker build** must work (network at build = broken build, same rule
as the Tailwind CLI); **privacy** — a self-hosted tool must not phone a third party; **reproducibility**
— `cargo build` sees only static bytes; and **`Cargo.lock`'s doctrine applied to assets: nothing
resolves on the fly.**

> **This is coherent and free — but it is written nowhere, and the default reflex of anyone (an AI
> assistant included — Murat's #1 drift vector) is to paste `<script src="https://unpkg.com/htmx.org">`.
> That is why it is a recorded decision and a CI check, not an understanding.**

#### D38 — Focus management lives in a committed `app.js`, not scattered in hyperscript

**The stakes:** focus management is the UX's **#1 accessibility requirement** (*"focus visible and
never lost after a swap"*), a **blocking CI gate**, plus a **manual screen-reader pass with recorded
proof per release**. It is the most load-bearing client-side code in the product.

| | Inline hyperscript | A committed `app.js` |
|---|---|---|
| ✅ | colocated; the UX mentioned it | **testable** (Playwright), one place, readable |
| ❌ | **scattered across N templates, not unit-testable** — *and scattered focus logic is wrong focus logic* | "real JS", which the doctrine avoided |

**Decision: `app.js`.** The argument is Murat's, applied: *"cancellation was untestable — therefore
untested, therefore wrong in production."* **Focus management is exactly that case — except here it is
the product's blocking a11y gate.** And the UX already named the subtle trap that will not survive
scattering: *"**never rely on `morph` to preserve focus for a removed node — a retracted card is
destroyed, so focus would fall to `<body>`**"*, plus the live-region choreography (two regions,
**outside** swapped fragments; the counter `aria-live="polite"` debounced; the next card announced by
`focus()`, **not** a live region, to avoid a double announcement). **Those rules do not survive spread
across hyperscript attributes.**

**Corollary: if `app.js` carries focus, hyperscript may become unnecessary — one dependency fewer.**
To confirm when the keyboard queue is built (E/J/S/I bindings), not assumed now.

#### D39 — i18n: the format must be greppable and diffable

EN/FR, externalised strings, Askama integration. Candidates: `fluent` (ICU, rich, heavy) ·
`rust-i18n` (simple) · hand-rolled. **To version-verify before choosing.**

**The forgotten constraint that likely decides it:** the UX locked a **glossary uniqueness test** and a
**forbidden-word lint** as CI gates — **and they must run over the translation files.** So the format
must be **greppable and diffable in review**. That probably rules out any binary or compiled catalogue.
*(And note Paige's finding: the uniqueness gate checks that a term has one **translation**, not that a
term has one **meaning** — that is the hole; extend the test with a denylist of banned words.)*

#### D40 — Askama organisation

Partials per the UX's component library (triage card, gap diff, evidence chip, object card, stat card +
sparkline, occupancy grid, data table, undo toast, keyboard queue), our own `IntoResponse` newtype
(D34/step 3, ~15 lines), and **the focus-on-every-swap helper as a reusable convention from story 1,
not a retrofit** (already recorded at step 3).

**Visuals are SVG + CSS, never canvas** (locked in the UX: *canvas is invisible to axe-core and opaque
to snapshots*) — the sparkline is a server-rendered `<polyline>`, the occupancy grid is CSS Grid with a
per-cell `aria-label`. **Zero JS for these.**

### Category 5 — Infrastructure & Deployment

#### D41 — FR52 (opt-in telemetry) is OUT of the MVP. Unanimous, by three independent routes.

**(1) The JTBD route — there is no user job.**
> **"Which user wakes up in the morning with the problem FR52 solves? None. Zero. Not one."**
> *"Stated honestly, FR52's job is: **'As GUY, I want to know whether anyone kept the thing, so I do not
> code for two years into the void.'** That is a real job. It is a legitimate job. **It is not the
> product's job. It is Guy's job — and Guy is not the user.**"*
>
> FR52 asks the user to accept that the binary — **the one holding the complete map of his network and
> his credentials** — opens an outbound socket to a server Guy controls, **to do Guy a favour**. In
> exchange he gets *"the project might survive better"* — **not a consideration, an expectation.**
>
> **"The cost of an outbound channel, even disabled, even opt-in: the user starts wondering what ELSE
> we send. We are not adding a measurement — we are injecting DOUBT into the only asset we own. And
> doubt, unlike telemetry, is not opt-in."**

And the doctrine cuts the same way in both directions: *"'the metric judges US' means **we** carry the
burden of knowing whether the loop works. **It does not mean the user carries the burden of telling
us.** Making the user pay — in trust — for the cost of our own evaluation is exactly the inversion I
refused on the backlog."*

**The opt-in moment lands at the worst possible place:** first run, <15 min to value, **the user's
window of MAXIMUM vigilance** — the one where he decides whether to keep you. He learns three things,
all bad: **the code can talk outward** (the capability exists, it is compiled, the toggle is just a
variable) · **someone is interested** (the first screen is not about his network, it is about *our*
needs) · **he must now verify** (on r/selfhosted: he reads the code, or tcpdumps, or closes the tab).

> **"The trust cost of FR52 multiplies by the number of pulls. The informational value SATURATES. After
> 200 instances the 201st heartbeat teaches you nothing — but the 201st user who sees the prompt pays
> full price."**

**(2) The architecture route — it is an operational commitment disguised as a checkbox.**
> **"FR52 is the only requirement in the PRD that does not run on the user's machine. Every other one
> describes what the binary does. This one describes what GUY does, forever."**

The architectural cost is deceptively small (40 lines) — *"exactly the kind of thing I usually say I
can rewrite in twelve lines. **Except the twelve lines are not the cost.** The cost is everything
around them that is **not written in Rust**: a domain (that expires), a certificate (that expires), a
host, a bill, a retention policy, a privacy page, a contact address that answers, and a backup —
**because the day we lose the data, the retention signal we claimed to measure is a retroactive lie.**"*
The operational cost is the real number: **not 5 EUR/month — ATTENTION**, the project's scarcest
resource. *"An always-on service is a permanent on-call for a solo dev. Not contractual — psychological:
something that can break while you sleep and whose breakage is public. **We pay a permanent on-call for
a monthly glance.**"*

**🚨 The failure mode the PRD never looked at:**
> **"That receiver holds a list of installations of a network-mapping tool. It is not a telemetry
> database. It is a TARGET LIST** — each entry saying 'here is someone who owns a structured inventory
> of their own network'. An attacker who compromises it does not get metrics — **he gets a directory of
> qualified prospects.**"
> **"A receiver a solo dev cannot defend is a receiver we should not own. And a receiver that does not
> exist is the only one we are certain we can defend."**

**The silent-failure trap:** *"if the receiver loses 30% of pings to a badly-set timeout, Guy will not
see an outage — he will see **a declining retention curve**. He will draw product conclusions. **THE
OUTAGE WILL DISGUISE ITSELF AS SIGNAL.** That is the prototype of silent wrongness."*

**On NFR27:** a category error on the letter (it is Guy's infra, not the user's) — **but a flagrant
violation of the doctrine that made NFR27 exist.** *"`Cargo.toml` does not grow, the binary does not
grow, NFR27 holds — **and yet we just added an always-on, legally exposed, security-sensitive component
that nobody tests and whose lying Guy will not detect.** The real principle was never 'the user deploys
nothing' — it was **'this project cannot afford moving parts a solo dev must watch'. The receiver is a
moving part. It is simply on the other side of the wire.**"* And: **"r/selfhosted will not read NFR27 —
they will run `strings` on the binary, see a domain name, and the thread will be over before anyone has
read the word 'opt-in'."**

**GDPR:** *"'Anonymous' is a claim, and a claim must **survive contact with an IP address in an access
log**. The receiver may log nothing — but the reverse proxy logs, the CDN logs, the host logs. And a
stable install ID **does exactly what a cookie does**. So either Guy becomes a data controller — **and
we literally cannot honour an erasure request against an anonymous identifier, a pretty legal paradox**
— or **the 'anonymous' claim is false and we do not know it yet. Being wrong without knowing it is the
exact definition of what my budget of unknowns refuses to spend outside the domain.**"*

**And the queue trap:** *"a telemetry queue is **a local telemetry database** — precisely what we swore
not to have. And a spool that flushes at once produces a burst that looks like **beaconing. We would
have accidentally built the exact network pattern of malware.** The receiver's degraded mode must be
**forgetting**, not memory."*

**(3) The measurement route — the arithmetic.**
> 40 installs at 3 months x **5% opt-in** (generous — *"in r/selfhosted, **the act of self-hosting IS
> the refusal of telemetry**"*) = **n_opt-in = 2.**
> **Wilson 95% on 4/8: [22%, 78%]** — *"the interval 'retention is between catastrophic and excellent'.
> **Not a measurement — a paraphrase of ignorance.**"* Rule of three: zero returns out of 8 leaves an
> upper bound of **37%** — *"even the blackest result refutes nothing."* Power to distinguish 30% from
> 40%: **n ≈ 172. You have 2.**

**The selection bias — not noise, an oriented lie:** *"the opt-in decision and the retention event are
driven by **the same latent variable: engagement.** Whoever enables telemetry is whoever read the docs
to the end and trusts you. **That is exactly the retained user.**"* Toy model (8 enthusiasts at a modest share
opt-in / 70% retention; 32 curious at 1% / 8%): **true retention 20%, observed retention 57% — a factor
of 2.8, undetectable because the segment that would contradict you is precisely the one that did not
tick the box.**

> **"And here is the point that should close the debate: RAISING THE OPT-IN RATE DOES NOT FIX THE BIAS.
> At 40% opt-in: observed 49%, truth 20%. Still 2.4x. More data does not bring you closer to the truth
> — it tightens the interval around THE WRONG NUMBER. It makes you confident. That is worse."**

**"Pulls = vanity, telemetry = honest" is FALSE:** *"pulls are **a bad instrument everyone knows is
bad** — nobody seeing 400 pulls believes they have 400 users. **An instrument whose flaw everyone knows
is socially corrected.** Opt-in telemetry is **a bad instrument that looks like a good one**: IDs,
timestamps, a denominator, a percentage. **It looks like science.**"*
> **"Word for word my line about N2: *a number that looks more rigorous than it is, is a number that
> fools its own author.* Pulls fool the README's reader. **Telemetry fools GUY — and Guy is the only
> person whose decisions depend on the number. At n=2, telemetry is MORE vanity than pulls: it is
> vanity in a lab coat."**

**Risk calculus:** P(usable signal at 3 months) ≈ **0** · P(trust incident | network egress in the
binary, audience r/selfhosted) ≈ **10-20%** · impact on an unknown project with one maintainer:
**potentially terminal**. -> **"Upside bounded at zero, downside unbounded, on the only asset you have.
Negative expectation. It is not even close."**

**DECISION: FR52 is deferred out of the MVP** — not rejected, **deferred with a named reason**, by
Winston's own test: *"what breaks the semantics of a computation is not deferrable; what enriches it
is. **Telemetry enters no computation.** It enriches Guy's knowledge of his market. **The textbook
deferrable — and I brought `virtual_device` INTO the MVP with exactly this test, so I hold to it when it
cuts the other way.**"*

**The two rehabilitated signals:**
- **Docker pulls TAG BY TAG over time are NOT vanity.** *"The PRD is right about **stars** — a star is a
  single free event. But **a pull of version N+1 by someone who already pulled version N is AN ACT OF
  MAINTENANCE ON A PRODUCTION SYSTEM** — literally the behaviour we are trying to measure. Noisy,
  imperfect — **but already there, free, it holds no list on anyone, and it has no failure mode.**"*
- **Issues are a strictly superior signal.** *"A heartbeat tells you **a container is running
  somewhere**. It does not tell you it SERVES anyone — **a forgotten container in an Unraid heartbeats
  for two years. Your heartbeat counts zombies and calls it retention.**"* An issue saying *"I have 412
  IPs and it has been 3 months"* gives you retention **AND** context **AND** the use case **AND** a
  human to talk to. **"Not a fallback — strictly superior, and the heartbeat adds nothing to it."**
  And: **"Telemetry is the tool of products that cannot talk to their users. Guy can. They are three
  hundred, not three million."**
- The closing frame: **"A heartbeat is an 'I know' that counts forgotten containers. That is precisely
  the false knowing we built everything else against."**

#### D42 — If FR52 ever ships: the conditions, written now so they are not debated later

**Unblock condition, verifiable before writing a line:** *"blocked until ~**1000 plausible installs** —
n_opt-in >= 100 is required for a Wilson interval narrower than 20 points. **Below that, the instrument
cannot produce a true sentence.**"*

**The only moment it may ask:** *"There is none at first run. There is exactly ONE: **when the user
comes to us.**"* A link in Settings, a sub-page, never surfaced, never announced — or better, a flag in
the docs the user sets himself. **"The product does not ask. The product makes available."** And the
admission that follows: *"that yields an even more biased, even rarer signal — **which is the confession
that it was never a measuring instrument. It was a GIFT. So call it a gift, treat it as a gift, and stop
budgeting it as a metric."*

**NEVER:**
- **Never at first run.** No prompt, no banner, no visible toggle until the product has done its job at
  least once.
- **Never dormant outbound network code.** Absent from the binary by default — **a compile-time feature
  flag, not an environment variable.** *"The only credible opt-in for this audience: **the code does not
  exist until it is asked for.**"* And: no `strings` may reveal a domain in a binary where telemetry is
  inactive.
- **Never a persistent identifier**, not even hashed. *"An install UUID turns a counter into a
  DOSSIER."* And *"hashing an IP anonymises nothing when the IP space is enumerable in hours — I refuse
  home-made anonymisation for the exact reason I refused home-made crypto."*
- **Never anything touching the data.** No device counts, no network sizes, no divergence counters, no
  equipment versions. **"The number of devices on a network IS information about that network, and
  aggregating it does not make it less true."** Maximum content: *"an instance is alive."*
- **Never re-ask.** A refusal is final. *Re-asking is nagging — ban #1.*
- **Never anything else on the same channel.** No "check for updates", no crash reports, no news.
  **One channel, one use, or zero channel.** *That ping writes the Reddit post by itself.*
- **Never a database, a queue, a spool, or a retry.** *"The day there is an `INSERT`, there is a target
  list, a backup, a retention policy, and a possible incident."*
- **Never opaque.** The exact payload, in cleartext, in the README, printed in the binary's own logs.
  **"He must be able to read the byte that leaves"** — without wireshark. **Same class as NFR13/TLS: a
  cooperative, documented, verifiable guarantee.**

**Minimum architecture if it ships:** a **GET to a static CDN artefact**, versioned by tag
(`/t/0.4.0/heartbeat`), returning 204 and nothing else. **No POST, no database, no application code —
the signal lives in the CDN's aggregated access log, which expires by itself.** *"What compromises this
receiver? Nothing: there is nothing in it. A stateless system cannot leak what it does not have."*
Fire-and-forget, short timeout, silent failure, **zero retry, zero spool.**
**Rejected outright: a third-party host (Plausible/PostHog)** — *"it removes the on-call and removes
nothing else: Guy is still the data controller, the third-party domain still appears in `strings`, and
now we explain to r/homelab that we send their data to an analytics company. **We paid the reputation
cost AND the legal cost, and bought less control. The worst of both worlds.**"*

#### D43 — The 3-month goal is rewritten: integers, not percentages

**The goal as written is NFR8 in a suit:** *"'adoption beyond the author with a **measurable** retention
signal' — no threshold, no instrument, no denominator, no window, no definition of retention.
**'Measurable' there is a decorative adjective: it describes the ambition, not the method.** And
'measurable retention signal' **cannot fail: any non-zero number satisfies it.**"*

> **"At this scale, INTEGERS are honest and PERCENTAGES lie. A percentage implies a denominator, hence a
> population, hence a sampling, hence a bias. A count of events implies nothing. It cannot lie about
> what it does not assert."**

**Rewrite:**
> **At least 3 distinct people other than Guy have run opencmdb on their own infrastructure and produced
> a public artefact proving it** (issue, discussion, PR, Reddit comment with real output), **of whom at
> least 1 returned >= 14 days after the first.**
> **Instrument:** GitHub issue/discussion authors != Guy, timestamped, counted by hand. **Cost: 0 EUR.
> GDPR: none. Receiver: none. Falsifiable: at D+90 you count. It is 2 -> failed. It is 4 -> passed.**
> **Bonus criterion, worth its weight:** *"the tracker contains >= 1 bug report that could only have been
> produced by running the software on infrastructure Guy does not own"* — **impossible to fake; it costs
> the sender something.**

**And the self-named theatre that legitimises the choice:** *"'3 people' is not a measurement. It is a
bet. I chose it. **Telemetry gives you a threshold that LOOKS derived, on a broken instrument. Counting
issues gives you an openly arbitrary threshold on an exact instrument. Always take the second: you can
debate an arbitrary threshold — that is an honest conversation between humans. You cannot debate a
biased instrument: it does not warn you that it is.**"*

**Theatre named inside FR52 itself, for the day it returns:**
- **The "reconciliation ran" ping — delete it.** *"It adds nothing that install+heartbeat does not
  already imply, and it is THE ping that reads as **'it is watching what I do'. Maximum trust cost, zero
  marginal information.** The worst ratio of the lot."*
- **🔴 Cross-instance aggregation of the north-star — the purest theatre in FR52.** *"It aggregates a
  **percentage whose denominator is defined by the user**: one declares 3 hosts, another 300; one
  declares everything, another only what matters. **'68% average coverage' has NO REFERENT — it is not a
  statistic, it is a category error. Locally instructive, globally meaningless.** Keep the local, kill
  the aggregate."* **-> the north-star stays: computed locally, shown to the user, aggregated NOWHERE.**
- **The isolated install event.** *"Without the return, it is a Docker pull with extra steps and a GDPR
  obligation. It only exists as the heartbeat's denominator — it lives or dies with it."*

#### D44 — The remaining infrastructure decisions

- **MariaDB in CI: a SERVICE CONTAINER** (GitHub Actions `services:`) — simpler, faster, no
  Docker-in-Docker. **Testcontainers wins only if we need to test SEVERAL MariaDB versions** — which is
  precisely the open unknown (*"the MariaDB version on the target Synology, I do not know it"*).
  **Revisit once that is known.** Both engines on every PR, never nightly (D1/NFR15).
- **Decantation delay on breaking bumps: NO age gate.** *"If your CI is good, age is a superstition. If
  it is bad, age will not save you."* Guy's stated preference leans this way, and **sqlx 0.9.0 was taken
  by Winston's own rule anyway.** -> grouped Renovate + auto-merge on green, **a dedicated PR for
  breaking changes, never grouped, never auto-merged**, and the veto: **never two breaking changes in
  one commit — bisection must stay trivial, because there is nobody else to bisect.**
- **12-factor config** (file + env, NFR27) — crate to select and web-verify. **The config has CROSS
  INVARIANTS validated at boot, not just parsing:** `dormancy_window < observation_retention` (D17) and
  **the key path must not be inside the data volume** (D27). **Both are startup FAILURES naming the
  offending keys, not warnings.**
- **Tracing/logging:** `tracing` + `tracing-subscriber`. Format TBD (JSON for Synology?). **FR49 security
  events must never carry ciphertext or nonce** (D28).
- **Docker image:** base to decide (distroless/scratch?) — **it changes Murat's verdict** that image
  scanning is theatre on a static Rust binary. x86 priority, ARM best-effort via the native binary.
- **`sqlx.toml`** (new in 0.9.0, aimed at multi-database setups) — evaluate for the dual-backend. Not
  assumed.
- **The scheduler:** tokio; coalescing lives here (D34); and **THREE sweeps — `pending_commit`, sessions
  (D30), `dormant` interfaces (D17). All low-priority on the writer actor's channel, all bounded, all
  idempotent, all with an injectable `now()`. It is the same pattern three times — recorded as such.**

### Feedback to the PRD from Category 5

| # | Document | Change |
|---|---|---|
| F35 | **PRD (3-month goal)** | Rewrite in **integers, not percentages** (D43): >= 3 distinct people with a public artefact, >= 1 returning >= 14 days later, plus >= 1 bug report only producible on infrastructure Guy does not own. **Instrument: hand-counted. Cost: 0. Falsifiable at D+90.** |
| F37 | **PRD Success Criteria** | **Rehabilitate Docker pulls TAG BY TAG over time** — the PRD is right about stars, wrong to bundle pulls with them: **a pull of N+1 by someone who already pulled N is an act of maintenance on a production system.** Noisy, free, holds no list, has no failure mode. |

## Implementation Patterns & Consistency Rules

_Rules that stop two AI agents writing incompatible code. Conventions, not implementation._

_The generic checklist (state management, bundle optimisation, component architecture, GraphQL,
event sourcing) is **N/A** here — server-rendered, no JS framework, no microservices. Most patterns
were already settled by D1–D44. What follows is what remained — and one criterion that turned out to
matter more than any of it. **This step ends with fewer artefacts than it started with: three
proposed gates died, and the mechanisms that replaced them are cheaper and stronger.**_

### D45 — The gate criterion (governing rule, adopted project-wide)

> **A gate is a gate when its red has exactly ONE repair, and it is the one we wanted.**

| Gate | The red is | Repairs |
|---|---|---|
| `dual!(fn_name)` + `deny(dead_code)` | the compiler | one |
| `:memory:` ban | compiler / grep | one |
| `IdentityIndex<'u>` | borrowck | one |
| **D1 (both backends)** | **MariaDB says no** | **none — you do not patch InnoDB** |
| ~~`MemoryRepository`~~ | a test | **two, and the cheaper one is wrong** |

**The corollary is what does the work:** _a gate whose red is repairable from both sides is not a
gate, it is a negotiation._ **And a negotiation with oneself, on a project with no second reviewer,
has a known outcome.** This is not a discipline problem — it is a **cost gradient**: at 10 a.m.,
rested, you also take the cheap repair, because it is usually the right one. _"An ADR does not move a
cost gradient. It adds guilt."_

Companion to the standing rule (_"a gate that has never caught anything is a tax on the gates that
do"_). Both are applied below, **including against their own authors** — D45 killed the parity
counter, `MemoryRepository`, and the `trait-vocabulary` grep; two of the three were proposed by the
person who then withdrew them.

### D46 — The dual-backend harness: `dual!(fn_name)` — the macro takes a NAME, not a body

**This is NFR15's enforcement mechanism, and it is what makes D1 executable rather than aspirational.**
The invariant is a hand-written generic function; the macro emits only glue.

```rust
// tests/invariants/identity.rs — written by hand, first-class item, real span
async fn inv_read_your_own_writes<R: WriteRepository>(repo: R) { ... }
dual!(inv_read_your_own_writes);   // emits ..._sqlite and ..._mariadb
```

**Why this form, and not either of the two it replaces** — the round-table disagreement was false;
both camps wanted this object and were arguing about what the macro swallows:

- **The bound `<R: WriteRepository>` sits in a human-written signature.** That IS the D1 corollary
  ("the invariant suite is written against the trait, never against sqlx") expressed in Rust: there is
  no `pool()` on the trait, so `sqlx::query(...)` **does not compile** inside an invariant. A
  body-swallowing macro does not give this — the body is text, it can call anything, and the
  duplication is syntactic.
- **The macro consumes no user code**, so a failure's span points at the hand-written assertion, not
  at an expansion. `cargo test` names the backend that lied (`mariadb::inv_...` red, `sqlite::` green).
- **Parity is structural:** `dual!(x);` emits two tests or zero. The state _"I wrote the SQLite one and
  forgot MariaDB"_ does not exist. **The generic-function-plus-two-hand-written-entry-points form was
  rejected for exactly this: six lines you can omit are six lines that will be omitted.**
- **`rstest` for the BACKEND axis is REFUSED:** `#[case]` requires one common type → `Box<dyn Repository>`
  or `enum AnyRepo` → **a dialect-erasing enum in the harness is `sqlx::Any` entering by the service
  door, and it will reach production in six months.** `rstest` serves the **case** axis only (the ~50
  D18 traps). Two orthogonal axes, two tools — never mixed in one.
- **`#[sqlx::test]` is REFUSED outside adapter unit tests** — it injects a `SqlitePool` into the test's
  signature: sqlx in every invariant, the literal violation of D1's corollary.
- **`#[cfg(feature = "mariadb")]` is REFUSED** — the binary ships both backends anyway; a cfg gates
  nothing in production, only the dev loop. **A backend behind a feature is a backend that is already
  broken and waiting.**

**The frontier with the backend-specific pack is a SIGNATURE, not a folder.** `fn inv<R: WriteRepository>`
**cannot pronounce** `SQLITE_BUSY` — the type does not expose it. A test that needs the backend's name
is _forced_ to be monomorphic and leaves the folder by itself. **The compiler polices the frontier for
free, with no convention to respect.**

> **NFR15, made operational: the TRIGGER is the parameter, the INVARIANT is the function.** Provoking
> contention is backend-specific; _"a contention yields `RepositoryError::Contention`, and the retry
> converges"_ is generic — because the taxonomy (D47) already unified `SQLITE_BUSY` and MariaDB 1213.
> **The backend-specific pack is not a second suite: it is the test of the adapter's error classifier.**
> Three to five tests. **Do not build a `ContentionHarness` before three of them share a shape.**

**Harness constraints, each closing a hole that would silently void D1:**

- **`harness::mariadb()` PANICS if `OPENCMDB_TEST_MARIADB_URL` is unset.**
  `if let Ok(url) = env::var(..) { } else { return; }` **is the enemy of D1** — it turns "both backends
  on every PR" into "SQLite always, MariaDB when convenient", **and it passes review because it looks
  prudent.** A silently skipping test is nightly wearing a disguise.
- **SQLite fixture = temp file, NEVER `:memory:`.** A pool over `:memory:` gives N distinct empty
  databases; under shared-cache the locking semantics change. **And `:memory:` has no WAL — which is
  precisely what D21 rests on ("the read pool sees nothing uncommitted"). You would not merely be
  testing a database you do not ship: you would be testing a database where D21's premise is
  meaningless.**
- **The fixture calls the SAME `open_sqlite(path)` function as production.** Not the same PRAGMAs —
  the same function. _A fixture configured alongside prod is a fixture that lies, and a
  PRAGMA-conformance test would be a gate where a function call suffices._
- **CI greps:** `#[tokio::test]` banned in `tests/invariants/` — **this one can fail on a real gesture
  (a human writing a test by hand in that folder at 23:00), so it is a gate, not decoration.** Plus
  **`deny(dead_code)` on the test crate**, which catches what the grep misses: **a generic invariant
  written and never `dual!`-ed** — it compiles, it never runs, and the grep is green.

**CI infrastructure (closes D44's "revisit"):** **service container, both backends every PR, never
nightly.** Image pinned **exactly: `mariadb:10.11.11`**, matching the Synology DSM 7 package
(**10.11.11-1551**, verified 2026-07-16), an LTS branch supported until **February 2028**. _(10.6
reached EOL on 2026-07-06 — had the package been a 10.6, this would be a different conversation: an
engine with no security backports under a product whose threat model is a stolen database.)_
**Ratchet semantics, per the dependency policy: the exact pin means "do not advance on your own,
without my seeing it" — Renovate opens the PR, the dual-backend CI arbitrates, the pin rises.**
_The exact pin is what makes "CI has drifted from the NAS" a visible event rather than a discovery._
**One pinned version ⇒ testcontainers has no object.**

### D46b — The verdict join: without it, D1 is a slogan

Each invariant emits `{invariant, case, backend, verdict, rule}` to JSONL; a `diff-verdicts` job joins
the two. **Two independently red jobs are two booleans — and the case that matters is not expressible
in two independent booleans.**

| Join outcome | Reading |
|---|---|
| `sqlite=A, mariadb=A, expected=B` | **our bug.** One fix |
| **`sqlite=A, mariadb=B`** | **engine divergence** — either sqlx lies on one side, or we put semantics in SQL. **The most valuable of the four, and exactly the one a naive harness turns into _"one job red, one green, must be flaky, re-run"_** |
| **`sqlite: verdict=A rule=R1` / `mariadb: verdict=A rule=R2`** | **same output, different reason. BOTH JOBS GREEN.** An engine divergence hiding behind a correct result — **the worst kind: it survives until the data changes** |
| one abstains, the other decides | worst case. D18's `must-abstain` column |

**D1 is not there to check that the code works on SQLite and on MariaDB. It is there to detect that
sqlx lies on one of the two.** A red job says "something is wrong". The join says "the two engines
disagree" — **that is a discovery, not a failure.** And the join **removes human judgement from the
loop**, which, with no second reviewer, is the only place the "flaky, I'll re-run" reflex can be
intercepted. ~80 lines and one CI job: the price of two integration tests, against the fifty written
for D18. **The cost debate does not take place.**

**Three ACs, each closing a way the join goes green while blind:**

1. **Compare `(verdict, rule)`, never `verdict` alone** — `expect_rule` (D19) is part of the verdict.
2. **Fail if one backend emitted no row for a key present in the other.** Otherwise a crashed runner =
   absent file = empty join = **green**. Verify the `(invariant, case)` key sets are _identical_, not
   merely consistent where they overlap.
3. **`needs: [sqlite, mariadb]` with `if: always()`** — a join that runs only when both are green never
   sees the case it exists for.

### D47 — The `anyhow` frontier IS the dependency graph, not a rule

**The frontier by LAYER is a fiction. The frontier by CONSUMER is real**, and the test is one sentence:

> **An error type is CLOSED if and only if a machine decides on it. It may be OPAQUE if only a human
> reads it.**

| Type | Who decides on it | Verdict |
|---|---|---|
| `ConnectorError` | alert suppression (FR5/FR8/FR19) | **closed** — D33, already taken |
| `RepositoryError` | the writer actor: retry / abort / escalate | **closed** — _"you cannot assert that both backends converge to the same state if the state has no name"_ |
| `DomainError` | HTTP status, FR43's stable code, the HTMX swap target | **closed** |
| `main()`, bootstrap, backup→migrate→verify | **one** decision: it boots, or it dies restoring | **`anyhow` legitimate** — nobody matches on the variant, and on a Synology where the user reads stderr, a `.context()` chain is worth money |

**Mechanism, not convention:** `anyhow` is **absent from the core crate's `Cargo.toml`**. `use anyhow::Result`
is then **a name-resolution error**, not a lint — free, instant, and **impossible to bypass without a
diff in `Cargo.toml`: the file a human actually looks at.** _"Make the bug not compile", applied at the
crate level instead of the type level — D33's rule applied to D33 itself._

**`#![deny(...)]` per crate is REFUSED** — an assistant removes it as easily as it adds it, in the same
file as the code it is writing, a one-line diff inside a legitimately modified file. **A guardrail is
worth the difficulty of disabling it silently.**

⇒ **Workspace split, taken NOW while the codebase is empty** (Guy's own principle: the migration cost
is invariant, pay it while it is free):

- `crates/opencmdb-core` — no `anyhow`. **This is the layer where an error is DOMAIN DATA — and a gap
  that vanishes into an `anyhow::Error` is exactly this product's failure mode: it does not crash, it
  lies.**
- `crates/opencmdb-bin` — `anyhow`, `main.rs`, the composition root.
- `[dev-dependencies]` and `xtask/` (fixture generator) — outside the shipped binary.

**The hard rule: `sqlx::Error` DIES in the adapter. `RepositoryError::Backend(String)`, never
`#[from] sqlx::Error`.** A `#[from]` of convenience leaks sqlx into the trait, and the two-file blast
radius becomes the whole codebase — _that is the only protection the trait actually offers; do not
throw it away for ergonomics._ Classification (`SQLITE_BUSY` → `Contention`, MariaDB 1213 → `Contention`)
is the adapter's job, and **the only place in the code where a backend's name may appear in an error.**

```rust
// crates/opencmdb-core/src/repo/error.rs
pub enum RepositoryError {
    Contention,               // <- SQLITE_BUSY and MariaDB 1213 both land here. NFR15 in one word.
    Constraint(&'static str),
    NotFound,
    Backend(String),          // terminal, non-retryable, opaque BY DESIGN
}
```

**`Contention` is NFR15 in one enum:** identical invariant (the actor retries), backend-specific
triggers (each adapter maps its native code). It is the one place in the binary where the two backends
explicitly converge — **and it is an enum, not a comment.**

**One `thiserror` per DECIDER, never one per layer** — a layer with no deciding machine that re-wraps
for symmetry is a newtype tax. **`WebError` does not exist:** a handler _translates_, it does not
produce a sui-generis error. One `DomainError`, two renderers (`impl IntoResponse` for the UI,
`api::json()` for FR43), **one wildcard-free `match`** — adding a variant breaks compilation and forces
its status and code to be decided. **The status code is decided in ONE place for both surfaces**, or the
API and the UI diverge on what a 409 means and no test catches it. The **code** is an API contract
(adding a variant is a versioned change); the **message** is prose, not a contract. The DB error text
goes to the log with the correlation id, **never into the response**.

**`From<ConnectorError> for DomainError` is REFUSED** — FR5/FR8/FR19 suppress alerts on _named_
`ConnectorError`s; a `From` that flattens them destroys exactly the information D33 paid for. The
engine matches on `ConnectorError` explicitly, without a wildcard, and the lint holds it.

**Honest limit, named:** the dependency graph prevents `anyhow`. **It does not prevent
`ConnectorError::Other(String)`, which is `anyhow` in disguise and would make FR5/FR8/FR19 just as
inexpressible.** The closed taxonomy is not guarded by the compiler — **it is guarded by D18 Tier 1
requiring a decision on every variant: an `Other(String)` cannot satisfy an `expect_rule`.** That is
where the real guard lives, not in `Cargo.toml`.

**Theatre named:** a hand-maintained markdown catalogue of error codes next to the `match` that is
already the source of truth · `.context()` on every `?` — noise simulating traceability.

### D48 — Opaque identifiers: `CHAR(36) ascii_bin` / `TEXT COLLATE BINARY`. `BINARY(16)` refused.

**One logical→dialect pattern for all opaque identifiers**, not a per-column decision:

| Logical type | MariaDB | SQLite |
|---|---|---|
| `Id` (UUIDv7) | `CHAR(36) CHARACTER SET ascii COLLATE ascii_bin` | `TEXT COLLATE BINARY` |
| `Hash64` (SHA-256 hex) | `CHAR(64) CHARACTER SET ascii COLLATE ascii_bin` | `TEXT COLLATE BINARY` |

**This closes verification item #1 🔴** (`TEXT` unindexable in MySQL without a prefix length — it hit
`UNIQUE(token_hash)`, i.e. the auth path, **and `entity_id`, i.e. every FK in the schema**). 64 bytes is
far under the index limit. _One divergence to understand instead of two — and every divergent pattern is
a place the assistant can hallucinate the wrong side._

**Bind a `String`.** With `sqlx 0.9` and no `uuid` feature, `.bind(&str)` and `row.try_get::<String,_>()`
behave identically on both drivers — exactly the subset the walking skeleton locks (`query()`, `.bind()`,
`try_get()`, `migrate!`).

**Why `BINARY(16)` loses, three independent ways:**

1. **It diverges the two backends on the HUMAN path** — the literal is `UNHEX(REPLACE(...))` on MariaDB
   and `x'...'` on SQLite. Every ad-hoc query, every fixture, every log→query round trip diverges by
   dialect — **and the human path is the only one with no CI.** A D1 tax paid forever. And on bind:
   SQLite does not implicitly convert BLOB↔TEXT — an adapter binding `&[u8]` where the other binds
   `&str` returns **zero rows, silently.** Precisely the failure class this project is allergic to.
2. **The 3 a.m. dump.** You are debugging a false merge — the product's worst bug, the one that fused two
   real machines. With `CHAR(36)` you grep the UUID from the UI into the logs, the fixture JSONL, the SQL
   dump, a copy-pasted `SELECT`. With `BINARY(16)` you write a `HEX()` per query, get the endianness wrong
   one time in three, and **the grep does not work — the connective tissue between your tools is cut, at
   the exact hour you are worst.**
3. **It hides an engine divergence longer:** MariaDB `BINARY(16)` right-pads `0x00`; SQLite `BLOB` does
   not. Two comparison semantics **on the identity key**, identical 99.99% of the time — the exact profile
   of a bug CI never sees, **manifesting on a FK, i.e. as a false merge.**

**The v7 counter-argument does not hold** — and it was killed by its own likeliest proponent: in
`ascii_bin`, `'0'..'9'` = 0x30..0x39 and `'a'..'f'` = 0x61..0x66 — **monotone**. At fixed width and fixed
case, the lexicographic order of the canonical form **IS** the temporal order: right-edge B-tree inserts,
the same as `BINARY(16)`. _(The swap flag in `UUID_TO_BIN(x, 1)` is a UUIDv1 artefact. Do not carry the
folklore.)_ **Cost:** ~20 bytes × ~10⁶ history rows = tens of MB, InnoDB secondary indexes included. On a
Plus-class x86, noise. _You would pay permanent illegibility for a gain you will never measure._

**`ascii_bin` is not cosmetic — it carries two invariants at once:**

- **An `entity_id` is an identifier, never `norm()`-ed, so collation is its ONLY protection.** Without
  it, MariaDB compares PKs case-insensitively (`utf8mb4_general_ci`) while SQLite does not — **two
  engines, two notions of equality, on identity.**
- **It preserves v7's ordering benefit.** Under `utf8mb4_general_ci` that guarantee goes soft.

**Test:** `inv_entity_id_case_sensitive` — insert `...ab`, look up `...AB`, require `NotFound` on both.
**Trivially green on SQLite, it catches a collation drift on MariaDB. Exact NFR15 shape, written once.**

**Honest restatement of "no comparison descends into SQL":** an id's equality _does_ descend into SQL — a
FK join **is** a comparison. The rule is not "no bytes are compared in SQL", it is **"no comparison whose
result depends on dialect semantics"**. `ascii_bin` / `COLLATE BINARY` reduce it to a memcmp, identical on
both engines. The rule holds because **normalisation is lifted into the TYPE**:

```rust
// crates/opencmdb-core/src/id/mod.rs
pub struct EntityId(Uuid);
impl EntityId {
    pub fn new() -> Self;                                  // UUIDv7, clock injected
    pub fn as_db(&self) -> String;                         // lowercase hyphenated — THE ONLY path to .bind()
    pub fn from_db(s: &str) -> Result<Self, RepositoryError>;
}
```

No ad-hoc `to_string()` in an adapter. `Uuid::to_string()` is lowercase today; **that fact must not be
implicit at 40 call sites.** _`norm()` for an id is the type itself._

**Theatre named:** `BINARY(16)` plus a hex view — rebuilding the readability you just removed · a
benchmark spike — it costs more than the 20 MB it saves · **`ORDER BY entity_id` as a business sort** —
tempting and false (two nodes, two clocks); business ordering goes through `observed_at`, which comes
from the fixture (D19).

### D49 — `Repository`: `transact(closure)`, opaque `Unit<'u>`, no `commit()`

**The trait does not expose `&mut Tx`. It lends a unit of work through a closure.**

```rust
// crates/opencmdb-core/src/repo/mod.rs
trait WriteRepository {
    type Unit<'u>: WriteUnit + Send where Self: 'u;   // opaque GAT: no sqlx::Transaction, no sqlx::Error
    async fn transact<F, T>(&self, f: F) -> Result<T, RepositoryError>
        where F: for<'u> FnOnce(&'u mut Self::Unit<'u>) -> BoxFuture<'u, Result<T, RepositoryError>> + Send;
}
trait WriteUnit: Send { /* reads its own writes. NO commit(). */ }
trait ReadRepository { /* pool, &self, serves the API only (D21) */ }
```

- **`WriteUnit` has NO `commit()`.** An identity decision cannot be split across two transactions
  **because the method does not exist** — the AC-8b mechanism (the absent handle in the signature),
  applied to transactions. Not discipline: lexical scope plus a missing method.
- **`WriteRepository` and `ReadRepository` are two distinct types, and the actor is constructed with the
  first only.** D21's read-your-own-writes becomes **a constructor signature, not a review comment** —
  the actor _cannot_ reach for the wrong pool.
- **A replayable transaction must be a function you can call twice.** On a MariaDB deadlock the engine
  rolls everything back and the batch must be replayed; scattered `begin`/`commit` give the retry no
  purchase. `transact` fails as `Contention`; the actor replays the closure. **One retry path for both
  backends — NFR15, literally.**
- **Does the trait leak sqlx? Wrong question.** The right one: _can a caller name a sqlx type?_
  `grep -r "sqlx::" crates/ --exclude-dir=repo/{sqlite,mariadb}` must be **empty**. A CI gate that will
  catch things — **the two-file blast radius made verifiable.**
- **In the adapter**, query bodies are free functions generic over `sqlx::Executor`; the `Reads` impls for
  `ReadRepository` and for `Unit` are two-line delegations. **sqlx appears there and nowhere else, and the
  query is written once, not four times.**
- **Dispatch: monomorphise.** One `match cfg.db` at the composition root, everything below generic, the
  application compiled twice. **Paid in bytes and compile time, not in bugs** — and it is literally the
  same instantiation shape as `dual!`: the test harness and the composition root do the same thing.

**Known risk, budgeted rather than discovered later:** `for<'u> FnOnce(&'u mut Self::Unit<'u>) -> BoxFuture<'u, _>`
is **an HRTB over a GAT** — the class of signature an AI assistant will not write correctly first try.
**Counter-point accepted:** the drift vector concerns patterns repeated 200 times, each repetition a chance
to drift; **a signature written once in story 1 and thereafter enforced by the compiler across 200 call
sites is the opposite of a drift surface — it is the object that absorbs it.**

⇒ **AC story 1: the skeleton (`WriteRepository` + `transact` + two empty adapters) COMPILES before one
line of identity logic exists.** If it is not green in a day, it is not a test problem — it is a design
problem, and we learn it on day 1, not day 30. **Documented escape hatch: `Box<dyn>` erasure at the unit
level** — one allocation per batch; at 300 hosts / 36 subnets, single-digit allocations per discovery run.
**No performance argument exists against it, so the risk carried alone is bounded to one day.**

**Theatre named:** a Fowler-style `UnitOfWork` with `register_dirty` (one writer, explicit methods) · a
`Repository` per aggregate (`HostRepository`, `SubnetRepository`) — a single transaction crosses them all,
and the two-file argument literally requires one trait · a `savepoint()` "just in case" — **an identity
decision is never split in two, so a savepoint inside one is a category error** · `RepositoryError::Sqlx(#[from] sqlx::Error)`
"to keep the context" — **the leak, dressed as prudence.**

### D50 — `IdentityIndex<'u>` borrows the unit: borrowck enforces D25's red line

**The closure gives the PLACE of the bound. It does not give the FORCE.** A `FnOnce` captures — nothing
stops a closure capturing `&mut self.identity_index` from the actor's struct. **It compiles, it passes
every invariant** (no invariant tests a retry after rollback — and one that did would be a flakiness
test), **and the false merge waits for its NAS reboot.**

```rust
// crates/opencmdb-core/src/writer/index.rs
impl<'u> IdentityIndex<'u> {
    fn for_unit<U>(unit: &'u mut U) -> Self { ... }   // the ONLY constructor, private. new() does not exist.
}
```

**The index HOLDS the borrow and IS the handle** — every write goes through it (`index.insert(...)`,
never `unit.insert(...)` beside an index that merely watches). _If the index sits next to the unit rather
than owning the path to it, the unit is frozen for the index's whole lifetime — and someone "repairs"
that by loosening the bound: **two repairs, and the cheaper one is wrong** (D45, applied to this very
object)._

Putting the index in the actor's struct would require an `IdentityIndex<'static>`, **which is
unconstructible.** A retry is a new `transact` ⇒ a new unit ⇒ a new index. **Structural, not
disciplinary.**

**The force is anchored in the Tx, not in the `PhantomData`:** `Unit<'u>` (D49) is _already_
unconstructible as `'static` because it is anchored to the transaction. The `PhantomData<&'u mut ()>`
does not create the bound — it **propagates** an impossibility that already exists, out to the index.
**The anchor is the Tx; the `PhantomData` is the belt.** _We do not add a constraint; we wire the one we
already have._

**Risk arithmetic:** real retries (MariaDB restart, NAS reboot, connection drop, disk full) ≈ **1–5/year**
— the single writer actor cannot deadlock with itself, and the read pool is MVCC and takes no locks. Low
frequency. **Severity: a false merge is silent, destructive, irreversible, and it makes a GAP DISAPPEAR.
The product does not crash — it lies.** Cost of the mechanism: a `PhantomData` and a private constructor.
**Zero test lines.**

### D51 — `MemoryRepository` REFUSED — and one bug class assumed without a gate

**The class is REAL and D1 is blind to it:** _"the trait absorbed a SQL assumption"_ — both backends stay
green **because both ARE SQL**. The trait covers API coupling; D1 covers sqlx being wrong; **nobody covers
the trait ceasing to be an abstraction.** _Two measurements with the same instrument give you random error,
never systematic error._ **The class exists. The gate does not follow.**

**1 — The instrument is not sensitive to the class.** All four named leaks, run down:

| Leak | in-memory fake | D1 (SQLite ∧ MariaDB) |
|---|---|---|
| `last_insert_id` | **GREEN** — `AtomicU64` + `map.insert`; the counter IS the leak, and the fake implements it natively | **moot** — `RETURNING` banned + D48 ⇒ UUID minted client-side ⇒ no read-back, nothing to absorb |
| implicit ordering, no `ORDER BY` | **RED, flaky** — `HashMap` = per-process SipHash seed; red one run in two. 23:00 fix: `BTreeMap`. One token | **RED, deterministic** — SQLite full-scan = rowid order = insertion order; InnoDB PK `CHAR(36)` clustered = lexicographic. **Diverges on row 2** |
| `LIKE` semantics | **GREEN** — the fake is written _after_ reading the test | **RED** — `ascii_bin` ⇒ MariaDB case-sensitive; SQLite `LIKE` case-insensitive ASCII by default |
| "the unit sees its own writes" | **GREEN** — the native behaviour of a mutable map | green both sides, and correctly |

**0/4 for the fake — and 2/4 refute the premise.** `LIKE` and ordering are **not** in D1's blind spot;
**D48 (`ascii_bin`, PK `CHAR(36)`) put them in its sharpest cone of vision.** _(Same family:
`UNIQUE(hostname)` — SQLite BINARY accepts `Foo` and `foo`; MariaDB `utf8mb4_general_ci` rejects the
second. D1 red. The fake is green — and green **by agreeing with SQLite**.)_

**The 60–70% estimate rested on the `RETURNING` workaround — but banning `RETURNING` FORCES client-side
identity, which REMOVES "the store mints identity" from the trait. That is an elimination of an absorption
site, not an absorption site. The number was a fossil.**

> **The superset theorem — attacked by its own opponent for an hour, unrefuted:** under D21/D25 — single
> writer actor, zero concurrency, no durability or serialisation to honour — the set of semantics realisable
> by `HashMap` + `Vec` is a **strict superset** of SQLite ∪ MariaDB. **A superset never goes red.** It goes
> red only on **ambiguity** — where the contract is silent and the author must _choose_. **And where she
> chooses, she chooses green. A fake written after the test has no degree of freedom that points at red.**

**2 — The asymmetry ("green proves nothing, RED proves something about the trait") is logically valid and
empty in engineering terms.** It is a claim about the **inference**, not about **P(red)** — and P(red)
belongs to the fake's author. **A gate whose author controls its trigger is not a gate, it is an assertion
with a runner.** It would be decisive if the fake were written by someone else. **It already is: MariaDB is
an in-memory repository written by an adversary who never read our trait, in 1995, and refuses to.** _That_
is its proof value, and it is exactly what a hand-written fake will never have.

**3 — The red has two repairs, and the cheaper one is wrong (D45).** Nothing in the red distinguishes
"the trait leaked toward SQL" from "the fake does not model what the trait asks". The ADR line lives in
`planning-artifacts/` — **not in the diff, not in the failure output, not in the assistant's context.** The
fix is `HashMap` → `BTreeMap`; `git commit -m "fix memory fixture ordering"` is an **honest** message, and
it passes review because there is no review. **And "the fake has a bug" is the highest prior when both SQL
backends are green — and that prior is correct 9 times out of 10. The ADR asks us to overturn a correct
prior, at 23:00, using a document we do not read.** _It is not carelessness. It is the gradient._

**4 — The discounted cost of the class is zero. PostgreSQL is SQL. The NFR15 sqlx replacement is SQL. Both
real porting scenarios for this project are intra-SQL.** The class costs something only the day a
**non-relational** backend arrives — for a CMDB whose product is a gap computed over relational invariants.

**⇒ The recorded renunciation** — this is what an ADR is for: not defending a gate, **registering a
renunciation**:

> **This class has no gate. The cost is paid on the day of the port. We know it, we assume it, and we do
> not pretend to cover it.** If a non-relational backend enters the roadmap, **this trait is to be
> re-audited before, not during.**
>
> _A gate that claims to cover what it does not cover is worse than no gate: it extinguishes vigilance._

**Also refused, from the same round — `ci: trait-vocabulary`** (a grep over the trait's signatures),
because it fails its own authors' rules:

- _"No primitives in a `Repository` signature"_ — **`Hostname(String)` passes. `Pattern(String)` passes,
  and `Pattern(String)` IS `LIKE`'s `%` wearing a hat.** The rule does not say "no SQL assumption", it says
  **"not the word `String`" — a grep on the spelling of a leak whose spelling is free.** Worse, **it is
  anti-correlated with its goal: it rewards wrapping.** Whoever writes `fn find(&self, pat: &str)` left the
  leak visible; whoever writes `fn find(&self, pat: Pattern)` passes the grep **by enveloping the leak**.
  The strong version ("`Pattern` must be a closed taxonomy, not a newtype over `String`") is **not
  greppable** — it is a semantic lint, not 25 lines.
- _"Every method returning `Vec<_>` takes an `order: Order`"_ — as a **grep** it has four repairs (add the
  parameter / return a type the grep misses / split the method / `#[allow]`), and at 23:00 the
  one-character one wins: **the red is repairable from both sides. A negotiation** (D45, applied to D45's
  own camp). **But it does not need the grep: `Order` without `Default` as a mandatory parameter IS a
  complete type-system mechanism — the compiler refuses the call.** The grep only checks that you wrote
  what the compiler already demands.
- **It is not a gate, it is a design exercise** (~2–3 reds, all in story 1, while the trait is being
  written; zero after — `mod.rs` barely moves once the second call site exists). **A design exercise does
  not ship as CI.** _"A gate that cannot fail is decoration"_ — this one can fail for one week in the
  project's life.

**What survives, transformed — three objects, zero CI jobs:**

| Object | Form | Why |
|---|---|---|
| **`Order` without `Default`, mandatory parameter** | **mechanism, in the trait** | the compiler is the gate. "No `ORDER BY`" does not compile — in the editor, before the commit exists |
| **No method returns an `Id`** | **consequence of D48, not a rule** | identity is minted client-side — such a method has **nothing to return**. Already unwritable, not forbidden |
| **Closed taxonomy for anything that would become a `%`** | **AC of story 1** | not greppable. A design decision, taken once, while writing |

**And the timing correction that made it land:** _the trait is frozen the day the SECOND call site exists_
— story 1 has one. "Exit condition of story 1" confused _before it is frozen_ with _before it is written_.
**The constraint belongs in the trait itself, during its writing — not in a job afterwards.**

### D52 — Server version floor, asserted at startup

**`CHECK` constraints are enforced from MariaDB 10.2.1 and MySQL 8.0.16. Below those they are parsed and
IGNORED.**

The architecture leans on `CHECK` **structurally**, not decoratively: D3's `CHECK (actor_id <> 'scanner')`
is explicitly called _"structural, not disciplinary"_; D21's supertype disjunction is _"enforced by the
engine, not by convention"_. **On an older server, those claims are silently false** — and opencmdb ships
to strangers via Docker Hub who will point it at their own server, while the PRD says "MySQL/MariaDB", not
"MariaDB 10.11".

> **The claim is true where we wrote it and false where it counts. That is NFR9's error, one storey down —
> and the document already settled that case once: we cannot measure the world; we can measure our reaction
> to it.**

⇒ **Startup REFUSES below the floor**, naming the version found and the version required. Same class as D27
("refuse to start if the key path is inside the data volume") and NFR9b: **a testable behaviour of our
binary instead of an assumed state of the world.** It passes D45 — **its red has exactly one repair: upgrade
the server** — and it can fail on a real gesture. It is not theatre.

**Verified production target (2026-07-16):** Synology DSM 7 package **MariaDB 10.11.11-1551** — an LTS
branch supported until **February 2028**. Well above the `CHECK` floor. _(This also closes verification item
#4: InnoDB DYNAMIC row format, 3072-byte index limit; `CHAR(36)`/`CHAR(64)` in `ascii_bin` are 36 and 64
bytes. Non-issue.)_

**OPEN — a product decision, not an architectural one: where is the supported floor?**
**(a) `MariaDB >= 10.11`** — the production target family: narrower, honest, and exactly what CI tests.
**(b) `MariaDB >= 10.2.1` / `MySQL >= 8.0.16`** — what the schema strictly needs, **but it promises a matrix
we do not test.** _Leaning: (a), by NFR15's own logic — **we do not claim what has no CI.**_

### Ratified conventions (de facto in this document; now explicit)

- **Tables: singular snake_case** — `declared_attribute`, `identity_link`, `security_event`,
  `observation_record`, `field_decision`, `identity_migration`, `link_candidate`, `source_capability`.
  Columns snake_case. Foreign key = `<entity>_id`.
- **JSON fields: snake_case** — serde's default, and FR43's audience scripts against it with `jq`.
- **Timestamps in JSON: ISO-8601 UTC strings**, the same representation as storage. One format, no
  conversion layer.
- **Rust naming: rustfmt + clippy defaults.** Not a decision — the toolchain settles it. **Do not write a
  convention for what a formatter already enforces.**
- **Test naming: a descriptive sentence** — `flapping_value_refires_doubt`, `inv_entity_id_case_sensitive`,
  `blocking_recall_above_999`. **An invariant reads as a claim, so a red test names a broken claim.**
- **Crate layout:** `crates/opencmdb-core` (no `anyhow`) · `crates/opencmdb-bin` (`anyhow`, `main.rs`,
  composition root) · `xtask/` (fixture generator, outside the shipped binary).

### Enforcement — what actually holds each rule

| Rule | Held by | Can its red happen? |
|---|---|---|
| invariants written against the trait, never sqlx | **compiler** (no `pool()` on the trait) | n/a — unwritable |
| backend parity of the invariant suite | **`dual!`** (emits 2 or 0) | n/a — structural |
| no hand-written mono-backend test in `tests/invariants/` | **grep CI** | **yes — and it is exactly the 23:00 gesture** |
| a generic invariant written and never `dual!`-ed | **`deny(dead_code)`** | yes |
| MariaDB never silently skipped | **`harness::mariadb()` panics** | yes |
| no `anyhow` in the engine | **name resolution** (crate absent) | **escape = one line in `Cargo.toml`, visible in the diff** |
| no sqlx outside the adapters | **grep CI** | yes |
| no `_ =>` in the engine | **clippy `wildcard_enum_match_arm` = deny** | yes |
| an identity decision never split in two | **no `commit()` on `WriteUnit`** | n/a — unwritable |
| the identity index never outlives its batch | **borrowck** (`IdentityIndex<'u>`) | n/a — unconstructible |
| "no `ORDER BY`" | **`Order` without `Default`** | n/a — does not compile |
| an unenforced-`CHECK` server | **startup refusal** (D52) | yes |
| the two engines agree | **MariaDB** | **yes — and it has no repair on our side** |

### Still open (Tier 2 — never party-tested; decide before the web layer)

- **HTMX fragment routing convention** — full page vs fragment: a `/partials/…` namespace, branching on the
  `HX-Request` header, or a route suffix? **Agents will diverge on story 2.**
- **axum 0.8 uses `{id}`, NOT `:id`** — the training corpus says `:id`, and axum 0.8 **panics at router
  build time**; it does not fail to compile. **The drift vector #1, dead centre.** Needs a convention and a
  smoke test.
- **JSON envelope + error shape** (`{data, error}` vs direct response) — flagged "leans, not yet
  party-tested" in Category 3.
- **`tracing` conventions + a `batch_id` correlation id in every span** — on a reconciliation engine,
  correlating observation → link decision → gap **is** the debugging. Log format (JSON for Synology?) open
  per D44.
- **i18n key convention** — must be greppable and diffable (D39), because the glossary uniqueness test and
  the forbidden-word lint run over the translation files.

### Feedback to the PRD from Step 5

| # | Document | Change |
|---|---|---|
| F38 | **PRD (scope / NFR15)** | **State the supported server floor explicitly.** The PRD says "MySQL/MariaDB"; the schema's structural guarantees (D3's `CHECK (actor_id <> 'scanner')`, D21's supertype disjunction) are **silently void below MariaDB 10.2.1 / MySQL 8.0.16**, where `CHECK` is parsed and ignored. **opencmdb refuses to start below the floor** (D52). Decide the floor: `MariaDB >= 10.11` (what CI tests) or the bare `CHECK` floor (**a matrix we do not test**). *We do not claim what has no CI.* |

## Project Structure & Boundaries

_Step 6 is 80% assembly: D1–D52 already forced most of this tree. What follows records the decisions
that remained, the complete structure they produce, and the mapping from the PRD's nine FR domains to
concrete directories. **Two constraints nobody chose — the orphan rule and `CARGO_MANIFEST_DIR` — did
most of the deciding.**_

### D53 — The orphan rule: newtypes in `bin`, `http_status()` in `core`

**The conflict, found while assembling and not before:** D47 puts `DomainError` in `opencmdb-core`. D47
also ruled *"`WebError` does not exist — a handler translates; it is an `impl IntoResponse for
DomainError`."* **Those cannot both hold.** `IntoResponse` is axum's, `DomainError` is `core`'s: written
in `bin`, **both are foreign → `rustc E0117`. It does not compile.**

**The ruling is withdrawn, and the withdrawal names itself:** *"my refusal of `WebError` was a
preference — `E0117` is a compiler. A taste argument against a compiler is zero rounds."* The argument
it rested on survives intact and merely **changes target**: *"a hop that names no new state"* was true
and is now beside the point — **the hop exists because `impl ForeignTrait for ForeignType` does not
compile, not to name a state.**

**REFUSED — `core` depends on axum.** The tempting exit, and the wrong one:
> With axum in `core`, the question *"what status code for this gap?"* becomes **answerable in `core`.
> And the day it is answerable there, it will be answered there — not from malice, from convenience, on
> a Tuesday evening, in the `match` already open on screen.**

The failure mode is **D51's shape, seen coming this time**: `DomainError` becomes an HTTP response type.
Someone adds `DomainError::RateLimited` or `::Unauthorized` **because the web needs it — and the domain
now carries concepts that do not exist in the domain. No test goes red.** And it is not gateable:
`grep -r "axum::" crates/opencmdb-core/` is **non-empty by construction** once the dependency exists;
no gate distinguishes "axum for `IntoResponse` only" from "axum everywhere". _A gate whose red has no
clear repair is not a gate (D45)._ Plus a hard fact, not a taste: **axum is a version fixed point —
`core` depending on it gives the IPAM and composite-identity layer an update calendar dictated by the
web.**

**REFUSED — a third `opencmdb-web` crate.** `DomainError` is just as foreign there: **the same wall.**
And D55 kills it a second time — a second `templates/`, a second `askama.toml`, a second `@source`, and
the orphan rule unchanged. **Modularity theatre.**

**DECIDED — the status is decided once, in `core`, without axum; the newtypes live in `bin`:**

```rust
// crates/opencmdb-core/src/error/status.rs
/// The ONLY place a DomainError becomes an HTTP code.
/// Returns u16 — not axum::http::StatusCode. `core` does not know axum.
pub fn http_status(e: &DomainError) -> u16 { match e { /* exhaustive, no wildcard */ } }
```

> **`u16` is the whole point: that is HTTP, not axum.** The domain is entitled to know that an identity
> conflict is a 409 — that is a product decision (alerts, stable deep-links). **What it is not entitled
> to know is what a `Response` is.**

```rust
// crates/opencmdb-bin/src/http/error.rs
pub struct Web(DomainError);              // PRIVATE field. From<DomainError> is the ONLY entry.
pub struct Api(DomainError);
impl From<DomainError> for Web { /* 3 lines */ }
impl From<DomainError> for Api { /* 3 lines */ }
```

Handlers are unchanged: `-> Result<Html<String>, Web>` or `-> Result<Json<T>, Api>`, and `?` does the
`From`. **The tax is ~7 lines once, not per handler.**

**The distinction that makes this consistent with refusing `WebError`, and it is not a rationalisation:**
> **`WebError` was an ENUM with its own variants — a second site of decision. `Web` is a tuple struct
> with zero added fields — an orphan-rule adapter. The first duplicates a state; the second duplicates
> nothing.**

#### What actually carries D53 — and what merely guards it. Do not confuse them.

**The load-bearing mechanism is the compiler, and it is one thing only:** `http_status`'s wildcard-free
`match` in `core`. **A new `DomainError` variant is an `E0004` — a compile error, before it is a wrong
HTTP response in production.** That is free and it is hard.

**Everything below is a guardrail, and the honesty clause is part of the decision:**

```
grep -rn "StatusCode::" crates/opencmdb-bin/src/ | grep -v "StatusCode::from_u16"    # must be EMPTY
grep -rnE "StatusCode as|\.status\(|hyper::" crates/opencmdb-bin/src/                # must be EMPTY
```

> **This gate is NOT airtight, and D53 says so rather than pretending otherwise.** `(409u16, body)`
> passes — and it is **shorter than the fault it replaces**; so would a `hyper::StatusCode`. Against D45
> — *"a gate is a gate when its red has exactly ONE repair, and it is the one we wanted"* — **it fails
> in the strict sense: six exits, two watched.**
>
> **It is kept anyway, and reclassified: a REFLEX gate, not a proof gate.** The threat model is not an
> adversary — it is Guy on a Tuesday evening in the `match` already open on screen. **`StatusCode::CONFLICT`
> is the reflex**: it is what autocomplete offers, what the axum example shows, what the AI writes.
> `use axum::http::StatusCode as SC` or `(409u16, body)` are not reflexes — **they are decisions, and
> writing one is a confession you make to yourself as you type it.** The gate does not close a door; **it
> turns a reflex into a deliberate act.** That is less than we wanted, and it is real.
>
> *If we had written "the only way to obtain a status is `from_u16(http_status(&e))`", we would have
> inscribed a lie that fails the first time someone tests it. **The confusion between a mechanism and a
> guardrail is exactly what produces a D51.**_

**Third layer — the private constructor.** `Web`'s field is private and `From<DomainError>` is its only
entry. Rust cannot forbid a `struct` becoming an `enum` (there is no `#[never_an_enum]`), so the soft
point its own author named — *"nothing stops `Web` drifting into an enum"* — **is not closed. It is made
narrow and smelly:** a `Web::Csrf` variant must be constructed somewhere, and its `IntoResponse` must
produce a status **with no `DomainError` to hand `http_status`**. It hardcodes → the reflex gate reds. Or
it does not exist.

**Three stacked mechanisms, none airtight alone: `E0004` (hard), the reflex gate, the single constructor.
Together the path to the fault is longer than the path to the repair. That is all a gate can do.**

**The mapping test — and its name matters:** `crates/opencmdb-core/tests/status_mapping.rs`, **not
`status_exhaustive.rs`.**
> **The name would have lied, and the lie would have cost the guarantee.** Exhaustiveness comes from the
> `match` (`E0004`), for free. The test checks **the values** — that `IdentityConflict` yields 409 and not
> 500, a product decision the compiler does not care about. *Someone reading `status_exhaustive` believes
> the guarantee comes from the test; the day they refactor `http_status` with a `_ => 500` catch-all,
> **every existing variant still maps correctly, the test stays GREEN, and the guarantee is gone without a
> red.** That is the literal definition of a D51: a property we believe is watched and is not.*

### D54 — `core` is organised BY SUBDOMAIN — and the folder is not the gate

**Layer-first (`domain/`, `repo/`, `engine/`) is refused.** Its frontier is violated by adding
`use crate::repo::field_decision::Store`, and **the red is repairable from both sides**: remove the `use`,
or shrug — *"`domain/` reading `repo/` is a layer below, that's normal."* **D45: a negotiation.** The
frontier **has no reason to exist beyond "layers are tidy" — it says nothing about this product.**
_`identity/` owns composite identity. `gap/` owns observed-vs-declared. **`domain/` owns nothing — it is a
synonym for "code".**_ **A module has a reason to exist when it owns an invariant a neighbour must not be
able to break.**

**But the honest correction, and it came from AC-8b's own author:**
> **"The FOLDER is not a frontier in Rust. The CRATE is. The MODULE is — if and only if visibility demands
> it. Organising `core` by layer or by subdomain produces NO red on its own. Claiming that the tidying
> implements AC-8b is theatre."**

**What implements AC-8b is the visibility; the subdomain is its CONSEQUENCE, not its cause:**

```rust
// crates/opencmdb-core/src/identity/field_decision/store.rs
pub(in crate::identity::field_decision) struct FieldDecisionStore { .. }   // NOT pub(crate)
```
```rust
// crates/opencmdb-core/src/gap/effective.rs
pub fn compute_effective(observed: &Observed, declared: &Declared) -> Effective
// There is NO third parameter. The store type is not nameable here.
// Violation = error[E0603]: struct `FieldDecisionStore` is private.
```
**One red, one repair, and it is the one we wanted:** pass the decision **as a value** inside `Declared`,
or do not pass it. And **`pub(in crate::identity)` only means something if `identity/` is a module that is
a unit of sense.** *"You do not say 'private to the repo layer' — it does not mean anything."* **That is
why the subdomain wins: not preference, but because restricted visibility needs a parent worth restricting
to.**

**AC-8b's residual weakness, named:** if someone writes `pub(crate)` on that store, the tree is intact and
**AC-8b is dead**. A grep helps (`store.rs` must expose only `pub(in ...)`); it is a grep. **Therefore:
AC-8b is the belt; AC-8a — the purge test (`DELETE FROM field_decision` → effective values byte-identical)
— are the braces. If only one survives, AC-8a survives.** _AC-8b without `pub(in ...)` is a post-it —
AC-8b's own words, turned on AC-8b._

**Two consequences that fall out for free:**
- **One `thiserror` per subdomain IS one per decider** (D47): `identity::IdentityError`, `gap::GapError`,
  `ipam::IpamError`. Layer-first would have produced one per layer — **D47 had already ruled against that,
  and nobody had noticed.**
- **Layers do not vanish — they become TRAITS, not folders.** `ports/` holds the contracts. **A contract is
  a file; a layer is a folder.**

**`ports/` is a named exception** — a layer-shaped module inside a subdomain-shaped tree. Assumed: D47's
frontier (`sqlx::Error` dies in the adapter) **is** cross-cutting, and a port per subdomain would give four
`Repository` traits and break D49 (one `WriteRepository`, one `transact`). _A named exception is not an
inconsistency._

> **FALSIFICATION CONDITION OF D54, recorded here and not in the minutes because a falsification condition
> you cannot find is not one: the day `ports/` grows a trait per subdomain, the subdomain organisation has
> failed and layers have been rebuilt through the back door.** It is the only mechanism we have against D54
> turning into its opposite.

### D55 — The `CARGO_MANIFEST_DIR` cluster; and `xtask css`, never `build.rs`

**Four mechanisms resolve relative to the manifest of the crate that invokes them:** `askama`
(`templates/`) · `rust-embed` (`#[folder]`) · `sqlx::migrate!` · and Tailwind's `@source` (relative to the
CSS file). **Where the web and the adapters live is constrained, not chosen.**

**And there is no collision with D47 — because `core` wants none of them:** it renders nothing (no
templates), serves nothing (no assets), knows no SQL (no migrations — `sqlx::Error` dies in the adapter).
**The cluster falls entirely on the `bin` side**, which reveals what `bin` actually is — and it was never
"the main":

> **`opencmdb-bin` is everything that touches the outside world: SQL, HTTP, HTML, files, the clock, the
> network. `core` is what does not.** D47's `anyhow` frontier turns out to be exactly that frontier — **not
> a coincidence: `anyhow` is useful precisely where errors come from the world and carry no domain meaning.**
> **When two independent constraints want the same line, stop looking.**

**Consequence: `sqlx` is not in `core`'s `Cargo.toml` at all**, so D49's `grep -r "sqlx::"` gate becomes
redundant with the compiler. **Keep it** — it costs zero and it catches the day someone adds `sqlx` to
`core`. _A gate made redundant by the structure is a good sign, not a reason to delete it._

**The Tailwind v4 trap — `@source` is relative to the CSS FILE, not to the manifest:**
```css
/* crates/opencmdb-bin/assets/tailwind.css */
@import "tailwindcss";
@source "../templates/**/*.html";     /* ../ — NOT ./ */
@source "../src/**/*.rs";             /* classes BUILT IN RUST — AC-1.12 */
@source inline("{bg,text,border}-{observed,declared,pending,committed,expired}");
@source inline("htmx-request htmx-swapping htmx-settling");
```
> **A wrong `@source` path breaks nothing visibly:** Tailwind simply finds nothing, the build is green,
> `app.css` is missing half its classes, **and you discover the colourless status pill in production.**
> AC-1.12's red is silent.

⇒ **The gate counts classes in the GENERATED CSS**, not in the config: regenerate into a temp file, diff
against the committed `app.css`, and grep each `@source inline` class in the output, expecting >= 1. **One
red, one repair: the `@source` is wrong.** _The `@source inline` list alone is not a gate — it is a list you
forget to update._

**`build.rs` invoking the Tailwind CLI is REFUSED. `cargo xtask css` instead.**
> **`build.rs` contradicts the committed CSS; it does not complete it.** Both answer the same question —
> *"where does `app.css` come from?"* — and give two different answers.
- It regenerates and writes `app.css` → **it contradicts the committed CSS**, dirties the working tree on
  every build, and makes `cargo build` non-deterministic. **A `cargo build` that modifies git-tracked files
  is a fault, full stop.**
- Or it regenerates into `OUT_DIR` → then what does `rust-embed` embed, the committed file or that one?
  **Two sources of truth for the same byte. The worst of both.**
- And either way: **`cargo build` on a machine without the Tailwind CLI fails.** `cargo install --path .`
  fails. A contributor who only wants to compile fails. **For a product whose promise is a single
  self-sufficient binary, making compilation depend on an external non-cargo-versioned binary betrays the
  promise at the build level.**

`build.rs` remains legitimate for what the compiler *must* know and nothing else can supply (a commit hash,
a `cargo:rerun-if-changed`). **Not for orchestrating an external toolchain.** _A `build.rs` is where you put
work you do not want to make explicit — which is exactly what we do not want here._

> **The pattern, and it repeats: `xtask` is nobody's dependency (D56); `build.rs` would be everybody's.
> When a tool must be nobody's dependency, it does not go in `build.rs`.** True for `recapture.rs`, true for
> the CSS — same reason.

### D56 — `fixtures/` at the root; `xtask/` a member and nobody's dependency

**`xtask/` is a workspace member and a dependency of nobody.** `cargo build -p opencmdb-bin` does not build
it; it is in no one's dependency graph, so **the generator structurally cannot enter the shipped binary.**
An *excluded* crate would be worse: no shared `Cargo.lock`, no `cargo check --workspace`, a CI that misses
its drifts. `default-members = ["crates/opencmdb-bin"]` so a bare `cargo build` does not build `xtask`.
**The gate proves it rather than promising it:** `cargo tree -p opencmdb-bin -e normal | grep -q "xtask" && exit 1`.

**All gates live in `cargo xtask ci` — in Rust, not in YAML.** _Five greps scattered across a GitHub Actions
YAML is a gate nobody can run locally, therefore a gate discovered broken after the push, therefore a gate
that gets disabled. One command, identical locally and in CI; the YAML calls it and holds no logic.
**Otherwise we write gates for the photograph.**_

**`fixtures/` lives at the workspace root, outside every crate.** The argument that decides it is D19's own,
applied to the location:
> **A file under `tests/` is read as the property of the test.** These YAML traps **precede** the code and
> **outlive** it — they are the input to epics & stories, they get read in product review, a trap can exist
> for weeks with no engine. **Under `tests/`, the first reflex of someone refactoring the engine is to adjust
> them until the red goes away. A red repairable by editing the spec is not a gate — it is a negotiation
> (D45).** At the root, editing a trap is a commit that says *"I am changing the spec"*, not *"I am fixing a
> test"*.

Plus **three consumers, no owner** (`xtask` writes the JSONL; `FixtureConnector` reads it; the invariant suite
reads it) — _an artefact shared by three crates lives in none of them_ — and **D19's own rule**: *"we never let
the oracle be regenerated by the build of the code under test"* is **one `build.rs` away from violation if the
JSONL sits in `tests/`. At the root, the thought does not occur.**

**The counter-argument was real and it is dissolved, on its own terms.** *"A shared root turns a dependency
into a convention — `../../fixtures` compiles, finds, until the day it does not, and the error is a runtime
panic, not a compile error."* True against an **unverified** `../../fixtures`. **The manifest makes the link
verified rather than conventional:**
```
fixtures/scenario/replay/MANIFEST.toml    # sha256 + seed + generator version, per artefact
cargo xtask ci --verify-fixtures          # recomputes the sha256, compares
```
**A JSONL modified without a manifest bump is red; one repair: bump it deliberately.** _At the root it looks
like what it is: **a lockfile for data.** Under `tests/` the same check would look like an oddity._ **There is
no argument against a hash.**

**And D56b turned the objection against its own proposal:** with the invariant suite necessarily in
`crates/opencmdb-bin/tests/`, fixtures under `crates/opencmdb-core/tests/fixtures/` would be read via
`../../opencmdb-core/tests/fixtures/` — **a path crossing a crate boundary INTO ANOTHER CRATE'S tests
directory. Longer, stranger, and exactly the "convention, not a frontier" being objected to. The root wins on
the objector's own criterion, not despite it.**

**Path discipline, non-negotiable:** there is **no `CARGO_WORKSPACE_DIR`**. The path is
`concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/")` — **a single constant, in one module, never
copied.** _If it appears more than once in the tree, it is already broken._

**`capture/` and `scenario/` are two folders, and the split is not tidying — it is THE DOMAIN OF DEFINITION
OF A DESTRUCTIVE TOOL.** D35's re-capture job diffs the real UniFi schema against `capture/`. **If that job
could see `scenario/`, then the day UniFi changes its schema it would offer to "update" a synthetic fixture
written to trap an L1 join case — and rewrite the truth to make the gate pass.** That is exactly the failure
mode D19 forbids for the generator, re-entering through the back door.
⇒ **The mechanism, not a grep: `recapture.rs` holds `capture/` as a module constant and never takes a path
parameter. `scenario/` is unreachable from any re-capture code.** _Same logic as AC-8b: not a test you can
bypass by carelessness — a signature that does not permit the fault._

**`FixtureConnector` stays in `crates/opencmdb-bin/src/connectors/`, NOT under `tests/`.** D19: **the fixture
IS a connector** — an `impl Connector` like SNMP will be. _Under `tests/` it would not face the same
compilation gates as its siblings, and "zero mocks" would become a slogan._

### D56b — A test lives in the lowest crate that can see everything it needs

**Caught while assembling, and it would have cost a day:** the invariant suite (D46) **cannot live in
`crates/opencmdb-core/tests/`**. An invariant is generic over the trait, but `dual!` must **instantiate** it
on `SqliteRepository` and `MariaRepository` — which live in `bin` (D55). **And `core` does not depend on
`bin`; that is the entire point of D47.**

| Test | Lives in | Because |
|---|---|---|
| **Invariant suite** (D46, `dual!`) | `crates/opencmdb-bin/tests/invariants/` | needs the two concrete adapters |
| **Backend-specific pack** (D46) | `crates/opencmdb-bin/tests/backend_specific/` | needs `SQLITE_BUSY` / MariaDB 1213 by name |
| **Trap suite** (D18 Tier 1) | `crates/opencmdb-bin/tests/traps/` | replays fixtures through the real writer |
| **`status_mapping`** (D53) | `crates/opencmdb-core/tests/` | needs `DomainError` only — **and must run without axum** |
| **Identity engine unit tests** (D19) | `crates/opencmdb-core/src/**` inline | the engine is a pure function: a `FixtureConnector` and nothing else — **no database** |

_The last row is the one that matters: **if a lot of domain logic needs a `Repository` to be tested, the logic
has leaked into persistence.** The engine takes an `&Observation` and a per-batch index (D25) and returns a
`Decision`. It never speaks to the `Repository` outside the writer actor._

### Complete project directory structure

```
opencmdb/
├── Cargo.toml                          # [workspace] members = ["crates/*", "xtask"]
│                                       #             default-members = ["crates/opencmdb-bin"]
├── Cargo.lock                          # COMMITTED — we ship binaries to strangers
├── rust-toolchain.toml                 # pinned MSRV: "recent" without a reproducible toolchain
│                                       #   is non-determinism distributed to strangers
├── deny.toml                           # cargo-deny: advisories + licences ONLY
├── .cargo/config.toml                  # alias xtask = "run --package xtask --"
├── .github/workflows/ci.yml            # calls `cargo xtask ci`. Contains NO logic.
│                                       #   services: mariadb:10.11.11  (D46, exact pin, ratchet)
├── compose.yaml                        # reference deploy: secrets in a SEPARATE shared folder (D27)
├── Dockerfile                          # build --locked. Nothing resolves on the fly.
│
├── fixtures/                           # ARTEFACTS, not tests. Outside every crate. (D19/D56)
│   ├── README.md                       #   "versioned artefact — do not regenerate without a bump"
│   ├── scenario/                       # ★ synthetic. Does NOT rot. Right or wrong.
│   │   ├── traps/*.yaml                #     ~50 traps — THE SPEC, written before the engine
│   │   │                               #     mandatory `reason` per expectation: the ORACLE
│   │   └── replay/
│   │       ├── traps.jsonl             #     committed. Never generated at test time.
│   │       ├── bulk-300h-36subnets.jsonl
│   │       └── MANIFEST.toml           #     sha256 + seed + generator version
│   ├── capture/                        # ★ real UniFi bytes. ROTS. Version-tagged + dated.
│   │   ├── unifi-3.x/*.json            #     the ONLY folder `recapture` can reach (D56)
│   │   ├── unifi-4.x/*.json
│   │   └── mutations/*.json            #     ~30, GENERATED from a real body (D35 layer B)
│   └── verdicts/expected.jsonl         # D46b — the (verdict, rule) join
│
├── crates/
│   ├── opencmdb-core/                  # NO anyhow. NO axum. NO sqlx. NO askama. (D47/D53/D55)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                  #   #![forbid(unsafe_code)]
│   │   │   ├── id/mod.rs               #   EntityId; as_db() is the ONLY path to .bind() (D48)
│   │   │   ├── norm.rs                 #   norm(): trim/lowercase/MAC/IP — never in SQL
│   │   │   ├── error/
│   │   │   │   ├── mod.rs              #   DomainError — aggregates the deciders' errors
│   │   │   │   └── status.rs           # ★ http_status(&DomainError) -> u16. E0004 is the gate. (D53)
│   │   │   ├── ports/                  #   NAMED EXCEPTION: traits only. Zero logic. (D54)
│   │   │   │   ├── repository.rs       #   ReadRepository / WriteRepository + Unit<'u> GAT (D49)
│   │   │   │   ├── connector.rs        #   trait Connector + ConnectorError (D33/D34)  FR1-8
│   │   │   │   └── clock.rs            #   trait Clock — the engine never touches the clock (D19)
│   │   │   ├── identity/               #   FR9-20 — the core. AC-8b lives here.
│   │   │   │   ├── mod.rs              #     IdentityError
│   │   │   │   ├── index.rs            #     IdentityIndex<'u> — borrowck holds D25 (D50)
│   │   │   │   ├── blocking.rs         #     candidate generator + blocking_recall >= 0.999
│   │   │   │   ├── cascade.rs          #     the verdict algebra. No float decides. (D13)
│   │   │   │   ├── field_decision/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── store.rs        # ★ pub(in crate::identity::field_decision) — NOT pub(crate)
│   │   │   │   └── migration.rs        #     D15 — entity_id is NEVER updated
│   │   │   ├── gap/                    #   THE PRODUCT — observed vs declared
│   │   │   │   ├── mod.rs              #     GapError
│   │   │   │   ├── diff.rs             #     filters state='asserted' AND entity.state='active'
│   │   │   │   ├── effective.rs        # ★ AC-8b: cannot NAME the store. error[E0603]. (D54)
│   │   │   │   └── evidence.rs
│   │   │   ├── ipam/                   #   FR21-25
│   │   │   ├── app/                    #   FR26-29 — applications & impact graph
│   │   │   ├── topology/               #   FR53
│   │   │   ├── alert/                  #   FR30-35 — + stable deep-links
│   │   │   ├── insight/                #   FR36-39 — history / retention
│   │   │   └── lifecycle/              #   FR40-42 — incl. D17 dormant
│   │   └── tests/
│   │       └── status_mapping.rs       #   D53 — checks the VALUES. Exhaustiveness is E0004. (D56b)
│   │
│   └── opencmdb-bin/                   # THE OUTSIDE WORLD: SQL, HTTP, HTML, files, clock, network
│       ├── Cargo.toml                  #   anyhow, axum, askama, sqlx, rust-embed, tokio
│       │                               #   NO build.rs — the CSS is `cargo xtask css` (D55)
│       ├── askama.toml                 #   dirs = ["templates"]
│       ├── migrations/                 # ← sqlx::migrate! resolves FROM THIS MANIFEST (D23/D55)
│       │   ├── sqlite/*.sql
│       │   └── mariadb/*.sql
│       ├── templates/                  # ← askama resolves FROM THIS MANIFEST
│       │   ├── layout.html
│       │   ├── components/             #   the UX library: triage_card, gap_diff, evidence_chip,
│       │   │                           #     object_card, stat_card+sparkline, occupancy_grid,
│       │   │                           #     data_table, undo_toast, keyboard_queue
│       │   └── pages/
│       ├── assets/                     # ← rust-embed #[folder] resolves FROM THIS MANIFEST
│       │   ├── tailwind.css            #   @source "../templates/..." — ../ NOT ./ (D55)
│       │   ├── app.css                 #   GENERATED by `cargo xtask css`. COMMITTED.
│       │   ├── app.js                  # ★ focus management — the #1 a11y gate (D38)
│       │   └── vendor/
│       │       ├── htmx-<ver>.min.js       # COMMITTED, version in the filename. Never a CDN. (D37)
│       │       └── idiomorph-<ver>.min.js
│       ├── src/
│       │   ├── main.rs                 #   anyhow. THE composition root: ONE `match cfg.db`,
│       │   │                           #     everything generic below (D49). The writer actor
│       │   │                           #     receives WriteRepository ONLY — D21 as a signature.
│       │   ├── lib.rs
│       │   ├── config.rs               #   12-factor. CROSS-INVARIANTS at boot, not just parsing:
│       │   │                           #     dormancy_window < observation_retention (D17)
│       │   │                           #     key path NOT inside the data volume (D27)
│       │   │                           #     server version >= floor (D52)   → all STARTUP FAILURES
│       │   ├── embed.rs                #   #[derive(RustEmbed)] #[folder = "assets/"]
│       │   ├── render.rs               #   OUR Askama→Axum IntoResponse (~15 lines).
│       │   │                           #     askama_web / askama_axum REFUSED (D34)
│       │   ├── paths.rs                # ★ THE single FIXTURES constant. Never copied. (D56)
│       │   ├── http/
│       │   │   ├── error.rs            # ★ Web(DomainError) / Api(DomainError) — private field,
│       │   │   │                       #     From is the only entry. StatusCode::from_u16 only. (D53)
│       │   │   ├── web/                #   #[derive(Template)] handlers. HX-Retarget #error-banner
│       │   │   └── api/                #   FR43 JSON read-only + FR44 authenticated /metrics
│       │   ├── repo/                   #   THE ONLY zone where `sqlx::` is legal (D47)
│       │   │   ├── queries.rs          #     free fns generic over sqlx::Executor (D49)
│       │   │   ├── sqlite/             #     open_sqlite() — THE SAME fn the fixtures call (D46)
│       │   │   │                       #     after_connect: PRAGMA foreign_keys = ON (D21)
│       │   │   └── mariadb/            #     sqlx::Error → RepositoryError dies here
│       │   ├── connectors/
│       │   │   ├── fixture.rs          #   FixtureConnector — the fixture IS a connector (D19)
│       │   │   ├── unifi/              #   zero-privilege (FR1-8) + FixtureTransport (D35 layer B)
│       │   │   └── scanner/            #   ARP/ping, NET_RAW → ping-only fallback
│       │   ├── writer/actor.rs         #   single writer; identity resolution runs INSIDE it (D21)
│       │   ├── scheduler/              #   tokio. Coalescing lives here (D34). THREE sweeps:
│       │   │                           #     pending_commit, sessions (D30), dormant (D17)
│       │   ├── secret/                 #   KEK/DEK envelope (D28); zeroize on Drop
│       │   └── i18n/                   #   EN/FR — greppable + diffable format (D39)
│       └── tests/                      # ← the invariants live HERE: they need the adapters (D56b)
│           ├── harness.rs              #   harness::mariadb() PANICS if the DSN is absent (D46)
│           ├── invariants/             #   fn inv<R: WriteRepository> + dual!(name)
│           │                           #     #[tokio::test] BANNED here (grep) + deny(dead_code)
│           ├── backend_specific/       #   SQLITE_BUSY / 1213 — segregated BY SIGNATURE (D46)
│           └── traps/                  #   D18 Tier 1: ~50 traps, binary, zero tolerance
│
└── xtask/                              # workspace member, dependency of NOBODY (D56)
    └── src/
        ├── main.rs                     #   anyhow — a dev tool is allowed
        ├── css.rs                      #   the pinned Tailwind CLI. NEVER build.rs. (D55)
        ├── gen_fixtures.rs             #   the seeded generator. A TOOL; the artefact is versioned.
        ├── gen_metrics.rs              #   the metrics harness — written BEFORE the engine (D19)
        ├── recapture.rs                # ★ `capture/` as a module constant. NO path parameter. (D56)
        └── ci.rs                       #   ALL gates, in Rust. Identical locally and in CI.
```

### FR domains → WHERE THE CODE LANDS

> **🔴 This table is a MAP OF THE CODE. It is NOT a list of epics, and it must not be read as one.**
>
> **Nine rows. How many deliver something to an operator? Zero.** *"Reconciliation & Identity" does not
> demo. `ports/connector.rs` does not demo. A green suite of 50 traps does not demo.* These are **horizontal
> layers in a system whose value is vertical**: *"I plug in my UniFi and I see what I never documented"* —
> a sentence that crosses rows 1, 2, 3 and 10. **That is not an epic in this table. That is the product.**
>
> **Every row here is a DEPENDENCY, not a DELIVERY.** Epics are cut into **vertical slices, and each slice
> crosses several rows.** This table is then what it should always have been: **the "where the code lands"
> column, never the "what to ship" column.**

| FR domain | Domain logic (`core`) | World-facing (`bin`) | Tests / fixtures |
|---|---|---|---|
| **Discovery & Sources** FR1-8 | `ports/connector.rs` (trait, `ConnectorError`, `Capabilities`) | `connectors/{unifi,scanner,fixture}/`, `scheduler/` | `fixtures/capture/` + `mutations/` (D35 layer B) |
| **Reconciliation & Identity** FR9-20 | `identity/` — the whole subtree | `writer/actor.rs` (D21) | `fixtures/scenario/traps/` → `bin/tests/traps/` (D18 Tier 1) |
| **IPAM** FR21-25 | `ipam/` | `http/web/`, `http/api/` | `bin/tests/invariants/` |
| **Applications & Impact** FR26-29 | `app/` | `http/web/` | `bin/tests/invariants/` |
| **Alerts & deep-links** FR30-35 | `alert/`, `alert/deeplink.rs` | `http/`, webhook delivery | `bin/tests/invariants/` |
| **Insight / History / Retention** FR36-39 | `insight/` | `scheduler/` (sweeps) | `bin/tests/invariants/` |
| **Data lifecycle** FR40-42 | `lifecycle/` (incl. D17 `dormant`) | `scheduler/` | `bin/tests/invariants/` |
| **JSON API + `/metrics`** FR43-44 | `error/status.rs` | `http/api/` | authorisation matrix (D26) |
| **Admin / Security / Ops** FR45-52 | `error/` | `secret/`, `config.rs`, `repo/` (migrations) | NFR12 suite AC12-1..8, both backends |
| **Topology** FR53 | `topology/` | `connectors/unifi/` | `fixtures/scenario/` |

**Cross-cutting, each landing in exactly one place:** identity → `core/identity/` · gap → `core/gap/` ·
portability → `bin/repo/{sqlite,mariadb}/` + `bin/migrations/{sqlite,mariadb}/` · secrets → `bin/secret/` ·
i18n → `bin/i18n/` + `templates/` · a11y focus → `assets/app.js` (D38) · config cross-invariants →
`bin/config.rs` (three startup failures: D17, D27, D52).

### Architectural boundaries

**The crate boundary is the only real one; everything else is visibility.**

| Boundary | Enforced by | What crosses |
|---|---|---|
| `core` ⇄ `bin` | **the dependency graph** — `core` has no `anyhow`, `axum`, `sqlx`, `askama` (D47/D53/D55) | `bin` depends on `core`. **Never the reverse.** |
| domain ⇄ persistence | **`ports/repository.rs`** — an opaque `Unit<'u>`, no `commit()` (D49) | `RepositoryError`, never `sqlx::Error` |
| domain ⇄ sources | **`ports/connector.rs`** — `ConnectorError`, a closed taxonomy (D33) | `Observation`, `PollSummary`. **No `anyhow`.** |
| `gap` ⇄ `identity::field_decision` | **`pub(in crate::identity::field_decision)`** → `E0603` (D54, AC-8b) | **nothing.** That is the point. |
| write path ⇄ read path | **two types** (`WriteRepository` / `ReadRepository`); the actor is constructed with the first only (D21/D49) | nothing — **the actor cannot reach the read pool** |
| batch ⇄ identity index | **borrowck** — `IdentityIndex<'u>` borrows the unit and IS the handle (D50) | nothing survives the transaction |
| domain ⇄ HTTP | **`http_status() -> u16` in `core`**; the newtypes in `bin` (D53) | a `u16`. **Not a `Response`.** |
| shipped binary ⇄ dev tools | **the dependency graph** — `xtask` is nobody's dependency (D56) | nothing |
| `recapture` ⇄ `scenario/` | **a module constant, no path parameter** (D56) | nothing — **the tool cannot rewrite the truth** |

### Named renunciations (in D51's spirit — recorded, not hidden)

- **Nothing stops `bin` bypassing `core` and putting business logic in a handler.** No gate; one there would
  be repairable from both sides (D45). **Assumed — the symptom is visible: it means writing SQL in `http/`,
  and that you will see.**
- **AC-8b dies if someone writes `pub(crate)` on the `FieldDecision` store.** The grep helps; it is a grep.
  **AC-8a (the purge test) is the real gate.**
- **`Web`/`Api` can still drift into an enum.** Rust cannot forbid it. Three stacked mechanisms make the path
  long and smelly (D53); **none closes it.**
- **The subdomain organisation has no compiler-enforced falsification.** Its condition is recorded instead
  (D54).
- **🔴 THE BUILD ORDER OPTIMISES FOR TRUTH, NOT FOR EARLINESS — and no decision in this document balances
  that.** Recorded as a renunciation because it is the largest one and it is not technical: *every decision
  here makes the product more TRUE. None makes it arrive EARLIER.* On a solo project with no reviewer, **the
  most likely death is not a fragile `identity_link` — it is abandonment.** A build order whose first contact
  with the real network comes last **defers every bad surprise to the moment it costs most.** The
  identity→gap dependency is causality and it stands; **"`identity_link` must be STABLE before the gap has
  meaning" is not — "stable" is a quality threshold, therefore a product arbitration that presented itself as
  a law of physics.** _Product decisions do not disappear when no product voice is in the room; they disguise
  themselves as technical constraints._ **To be settled before epics, not here.**

### Development workflow

- **`cargo xtask ci`** — every gate, in Rust, identical locally and in CI. The YAML calls it and holds no
  logic. Gates: `#[tokio::test]` banned in `tests/invariants/` · `deny(dead_code)` · `StatusCode::` outside
  `from_u16` + `StatusCode as|.status(|hyper::` (D53, **reflex gates, not proof**) · `sqlx::` outside the
  adapters · `pub(in ...)` on the `FieldDecision` store · `cargo tree` (no `xtask` in `bin`) ·
  `--verify-fixtures` (sha256 vs `MANIFEST.toml`) · `--verify-css` (regenerate to temp, diff, count classes
  in the generated `app.css`) · `cargo deny` · `cargo clippy -D warnings` + `wildcard_enum_match_arm`.
- **`cargo xtask css`** — regenerates `app.css` **explicitly, on demand**, when templates or the theme move.
  **`cargo build` compiles Rust. It does not call Node.**
- **`cargo xtask gen-fixtures`** — regenerates the JSONL **by hand, deliberately**. Never at build time. The
  diff shows up in review and truth is **seen** to change (D19).
- **`cargo xtask recapture`** — diffs the real UniFi schema against `capture/` **only** (D56).
- **CI:** both backends every PR, `mariadb:10.11.11` service container, exact pin, ratchet semantics (D46).
  Grouped Renovate + auto-merge on green for patch/minor; a dedicated PR for breaking changes, never grouped,
  never auto-merged.
- **Story 1** is the dual-backend walking skeleton: **the `WriteRepository` + `transact` skeleton and two
  empty adapters COMPILE before one line of identity logic exists** (D49). Escape hatch if the HRTB-over-GAT
  resists: `Box<dyn>` erasure at the unit level — bounded to one day.

### Feedback to the PRD / UX from Step 6

| # | Document | Change |
|---|---|---|
| F40 | **PRD / epics** | **The FR-domain → directory table is a MAP OF THE CODE, not a list of epics.** Nine rows, **zero of which deliver anything to an operator.** Epics must be cut as **vertical slices crossing several rows**; this table is the "where the code lands" column. *If it is read as an epic list, the epics will be horizontal in a product whose value is vertical.* |
| F41 | **PRD (build order)** | **"`identity_link` must be STABLE before the gap has meaning" is a PRODUCT arbitration wearing the clothes of a data dependency.** The dependency (gap is computed at device level → depends on `identity_link`) is causality and stands. **"Stable" is a quality threshold** — there is no `stable` column. Its consequence is that **the first contact with the real network comes last.** Must be decided explicitly, with the counter-proposal on the table: **a thin vertical slice, early, on a perimeter where identity is trivially unambiguous** (one connector, a few L1-clean devices, the gap on screen, end to end) — which does not violate identity→gap, and which is what a solo project needs to not be abandoned. |

## Architecture Validation Results

_Validation run at Step 7 against the complete document (D1–D56b, F1–F41). **The result is NOT READY, and
the gaps are named, countable and small — a different statement from low confidence.** Validation did not
merely check the work: it **closed the #1 critical gap, amended four decisions already taken, and found the
cause they share.**_

### Coherence Validation ✅

**Decision Compatibility — no contradiction survives; four were found and closed, all while ASSEMBLING or
VALIDATING rather than while deciding.** The stack holds: Rust · SQLite + MariaDB 10.11.11 · `sqlx =0.9.0`
(`query!`/`Any`/`RETURNING` banned) · axum 0.8.9 · askama 0.16 · HTMX + idiomorph · Tailwind v4 standalone ·
`rust-embed` · tokio. Versions verified 2026-07-16.
- **D47 × the orphan rule** (`impl IntoResponse for DomainError` in `bin` = `E0117`) → D53.
- **AC-8b × folder-as-frontier** → D54 rests on `pub(in ...)`, not on tidiness.
- **D19 × D35** — the build order put the real connectors last while D35 required *"~30 mutation fixtures
  generated from a real captured 3.x body"*. **A real body is not captured at step 11.** → D19-rev.
- **D28 × D31** — D28 mandated a `kek_id` **without ever saying by what mechanism it binds.** → D31-closed.

**Pattern Consistency — the patterns and the decisions share one mechanism.** Every load-bearing rule
resolves to *make the bug not compile*: D33 · D47 · D49 · D50 · D53 (`E0004`) · D54 (`E0603`). **D45 was
applied against its own authors** and killed five proposed gates — the parity counter, `MemoryRepository`,
the `trait-vocabulary` grep, **15 of the 50 traps**, and **a `kek_id` that nothing verified**; four were
withdrawn by whoever proposed them.

**Structure Alignment — two independent constraints landed on the same line, unsought.** D47's `anyhow`
frontier and the `CARGO_MANIFEST_DIR` cluster both cut between `core` and `bin`. **When two independent
constraints want the same line, that is a coherence result, not a coincidence.**

### Requirements Coverage Validation ⚠️ — PARTIAL, and the shortfall has a cause

| FR domain | Decisions | Verdict |
|---|---|---|
| **Discovery & Sources** FR1-8 | D19, D33, D34, D35 | **Covered, deeply** |
| **Reconciliation & Identity** FR9-20 | D12–D21, D45–D51 | **Covered, exhaustively.** The #1 risk, treated as such |
| **Admin / Security / Ops** FR45-52 | D26–D31, D41, D52 | **Covered — D31 closed at validation** |
| **Data lifecycle** FR40-42 | D17, D11 | **Covered** |
| **Alerts & deep-links** FR30-35 | deep-links locked; **webhook delivery "not party-tested"** | **Partial** |
| **JSON API + `/metrics`** FR43-44 | D53; **envelope open, crate unselected** | **Partial** |
| **Applications & Impact graph** FR26-29 | **→ D57 (required). The one real hole.** | **Decision REQUIRED** |
| **Insight / History / Retention** FR36-39 | D24 (*"OPEN and unaddressed"*), **and now D58** | **Needs a NUMBER, not a decision** |
| **IPAM** FR21-25 | *(a directory)* | **Correct proportionality** |
| **Topology** FR53 | *(a directory)* | **Correct proportionality** |

**Four of ten FR domains had a folder and zero decision. The triage:**
- **IPAM — correct proportionality.** CRUD over subnets and allocations, constrained by D45–D56b. **What
  makes it safe: no decision here cannot be taken at the moment of writing it, because none constrains
  anything else.** _A domain whose choices are local and reversible needs patterns, not architecture._
- **Topology — correct proportionality.** One FR. *If a single FR justified an architectural decision, we
  would have 53._
- **Insight — acceptable, not free.** The risk is the **volume**, not the feature. **A sizing hole that
  becomes an architectural hole the day the answer is "we need a time-series table".** ⇒ **an
  order-of-magnitude calculation, one page, before the epic. Not a decision — a number.**
- **Applications & Impact graph — a real hole.** → **D57.**

> **🔴 THE CAUSE, worth more than the fixes: Step 4's categories — Identity · Security · API · Frontend ·
> Infrastructure — are STACK categories, not PRODUCT categories.** They structured two days of work,
> therefore **they structured what could be seen. Four FR domains had no box to land in, so they did not
> land.** Same failure as F41 below: **a frame with no opposition does not produce a blind spot by mistake —
> it produces one by construction. Two instances, one cause.**

**✅ Cross-document coverage — VERIFIED AGAINST THE SOURCE DOCUMENTS, NOT AGAINST THIS ONE. F1–F37 ARE
APPLIED.**

- **`prd.md`, edited 2026-07-16 16:01** (`bmad-edit-prd`, `editHistory` in its frontmatter): **the 33
  PRD-targeted items are in.** 8 rewrites (FR5 → two orthogonal axes · FR19 → mechanical · NFR4 → the binary
  three-column trap suite · NFR8 → the four falsifiable assertions · NFR9 → 9a/9b/9c · FR52 · Success
  Criteria ×2) · **5 additions — F1 is now FR13's re-discovery case, F17 is now FR38b (`dormant`)**, plus
  FR14's sixth gesture, NFR7b (silent drift), and Journey 4 promoted to a binding requirement · 3 removals ·
  20 clarifications · **a binding Canonical Vocabulary section.**
- **`ux-design-specification.md`, edited 2026-07-16 16:25:** F11 (bootstrap as a MODE, no `first_run` flag) ·
  F12 (granularity per regime) · F13 (the seven bans, each with its test) · **F16 (the dead Tailwind v3 API
  is gone)** · F14 + the full rename.

> **🔴 THE VALIDATION ERROR, RECORDED BECAUSE IT IS THIS DOCUMENT'S OWN FAILURE MODE.** This section first
> asserted that the 39 items were unapplied and that `prd.md` was *"a fossil"*. **That was false, and it was
> false for the exact reason this document spends 4,000 lines refusing.** The feedback tables above were
> written at 14:57; the PRD was edited at 16:01 and the UX spec at 16:25. **The tables were read as the state
> of the world when they were only the state of the world at 14:57 — a claim true where it was written and
> false where it counted.** That is NFR9's error, committed by the validation step itself.
> **The structural fix, so the next reader does not repeat it: the F1–F41 tables in this document are a
> SNAPSHOT, not a state. Nothing in them marks them applied — so anyone reading them, human or agent, will
> conclude what this step wrongly concluded.** ⇒ **F1–F37 are marked APPLIED below. A feedback table with no
> applied-date is a dead document that looks alive.**

**What the divergence check actually finds, now that it is run against the sources:** **nothing open.**
FR52 is struck (number retained, not reused), FR38b exists, NFR7b exists, the success criteria are in
integers, the north-star aggregate is gone, and the vocabulary is binding. **The three planning documents
are mutually consistent.**

**Non-Functional Requirements — covered, several rewritten because they were unenforceable.** NFR4 → D18 ·
NFR7 → D35 (a type, not a test) · NFR8 → D35's four falsifiable assertions · NFR9 → D26 (9a/9b/9c) · NFR15 →
D46 · NFR1/NFR16 → read/write path separation. **Uninstrumented bets, named as bets:** NFR2 (p95 ≤ 1.5 s
on-read) · **Argon2id AND age's scrypt on the real Celeron — two measurements, not one** (D31).

### Implementation Readiness Validation ⚠️

**Decision Completeness — NO, but the list shrank at validation.** **D31 is CLOSED.** Remaining: **D57
required**, and four crates unselected — **i18n** (D39: the glossary uniqueness test and the forbidden-word
lint are CI gates that must run **over the translation files** → greppable and diffable; probably rules out
any binary catalogue) · **config** (must carry three boot cross-invariants as startup failures) ·
**`/metrics`** · **Docker base image** (*it changes the verdict that image scanning is theatre on a static
Rust binary in distroless*).

**Structure Completeness — YES.** D55/D56/D56b, every boundary held by a named mechanism, the FR→directory
map labelled **"where the code lands", never "what to ship"** (F40).

**Pattern Completeness — NO. Five Tier 2 patterns never party-tested**, two biting at story 2: **HTMX
fragment routing** · **axum 0.8 uses `{id}`, NOT `:id`** (*the corpus says `:id`, and axum **panics at
router build**; it does not fail to compile — the drift vector #1, dead centre*) · **JSON envelope** ·
**`tracing` + `batch_id`** · **i18n keys**.

---

## Decisions closed and amended at validation

_Seven decisions moved. Each was amended by the person who wrote it, on an argument from someone who was not
in the room. **That is the finding, not an anecdote.**_

### D31 — CLOSED: hybrid. RustCrypto for the wrap, `age` for FR48 only.

**The recorded "disagreement" was not one.** *"Two correct answers to two different questions, stacked."*
The note asked *"is wrapping a 32-byte DEK + deriving a passphrase for FR48 trivial enough in raw
XChaCha20?"* — **the question had two answers, and that is why it never moved: each camp was answering a
different half.**
- **The wrap: 22 lines of body.** No recipients, no PKI, no passphrase; the KEK **is** already the key.
  **Not the "two hundred lines" of the `age` objection — that was the fantasy of a full envelope format.**
- **FR48: 120–180 lines** — argon2 + format + versioning + nonce + associated data. **And that is where —
  and only where — the silent bugs live.** *"The `age` objection was right; it was aimed at the wrong half."*

**🔴 What decided it is a FACT, twice — and it came from a question nobody had asked: does the DEK wrap need
an AAD?**

**Yes.** D28's own invariant demands it: *"always decrypt via `secret.dek_id`, NEVER via 'the active DEK'"*
means **several wrapped DEKs coexist, selected by an identifier stored BESIDE the blob** — exactly the shape
AAD exists against.
> **The real argument is not the attacker. It is the BUG.** An interrupted KEK rotation, an FR48 restore on
> a new machine, a mis-joined `dek_id`, **and the code unwraps the wrong DEK without anything protesting.**
> Without AAD, a successful unwrap proves only *"this blob was wrapped by this KEK"*. With
> `aad = dek_id || kek_id || aad_version`, it proves *"this blob IS DEK #3 wrapped by KEK #2"* — **and the
> bad join becomes a loud error at startup instead of fields decrypting into mush three hours later.**
>
> *"That is the same reasoning as D28 itself: **not sophisticated crypto — error-handling code we do not
> write.** The AAD is the `dek_id` invariant made verifiable by the cipher rather than by my SQL discipline
> — and my SQL discipline is precisely what a solo dev with no second reviewer should not put on the
> critical path."*

**⇒ `age` is DISQUALIFIED for the wrap. Not by preference — by format constraint: it exposes no AAD API.**
And the second nail: **our KEK is a symmetric 32-byte file; age's non-passphrase recipient is X25519,
asymmetric.** *"To use age on the wrap we would have to derive an X25519 pair from the KEK — i.e. write
ourselves the derivation age was supposed to spare us. The argument self-destructs."*

**And D45 kills the requirement that had no mechanism:** *"D28 wrote 'a KEK identifier is mandatory' and
never said **by what mechanism it binds**. The AAD is that mechanism. Without it, `kek_id` is an
ANNOTATION — a string beside the blob that nothing ties to the blob. **The D28 requirement was theatre: a
field we write and nobody verifies.**"*

**Both integration paths for age on the wrap are CONTORTIONS, and naming that is a result, not an impasse:**
the KEK as an X25519 identity *"pays for a recipient structure for a single-actor case where that actor
already holds the key"* · the KEK as an scrypt passphrase *"runs a deliberately memory-hard KDF, on a
Celeron, on a key that is ALREADY 32 bytes of uniform CSPRNG. **Deriving a key from a random key is cost
with no added entropy.**"*

**DECIDED:**
```toml
chacha20poly1305 = "0.11"     # XChaCha20-Poly1305: DEK wrap + field encryption. AAD mandatory.
argon2           = "0.6"      # FR46/47 password hashes — orthogonal, never in question
age              = "=0.11.3"  # FR48 ONLY — exact pin, [BETA] API
zeroize          = "1"        # key types
rand_core        = "0.9"      # OsRng for DEK and nonces
```
*(versions to confirm at `cargo add` — **not a maintainer audit, a compilation check**)*
**`ring` stays in the tree for TLS and is never used for application crypto.**

- **The wrap is symmetric, needs an AAD, never leaves the process, has no recipient** → XChaCha20-Poly1305.
  **The 192-bit random nonce is the real win, and it is not speed: the counter disappears — and with it the
  crash-safe counter we would have had to make durable on two backends.**
- **FR48 is an artefact meant to LEAVE the process and OUTLIVE the binary.** It has a recipient: the
  operator, in eighteen months, possibly with no opencmdb running. → age scrypt, C2SP format, standard CLI.
  **The verified fact that clinches it: age's scrypt recipient IS the passphrase case** — password → scrypt →
  the resulting key wraps the file key with ChaCha20-Poly1305. **That is FR48, word for word.** And the
  format's hard constraint — *an scrypt stanza must be the ONLY stanza* — **is not an obstacle: our blob has
  exactly one recipient. The format forbids precisely what we did not want to do.**
- **`age -d backup.age` works without opencmdb.** *"A blob only my binary can read is not a backup, it is a
  hostage."* — **the one part of the original `age` argument that was never wrong, and it survives intact.**

**Two crypto families? No. Both envelope in ChaCha20-Poly1305. ONE family, TWO layers, with a frontier
sayable in one sentence: *`age` never touches more than one blob — the one the human carries away;
everything that lives in the DB is RustCrypto.*** *"The budget of unknowns is not a count of crates — it is
a budget of things I can be wrong about without knowing it. **A frontier sayable in one sentence costs it
nothing.**"*

**[BETA] read honestly:** *"What the world knows is **the age FORMAT** — a C2SP spec with independent Go,
Rust and JS implementations. What is [BETA] is the Rust crate's API — **a promise about signatures, not an
admission of cryptographic fragility.** Translated into risk: *a minor bump may break my build*, not *my
encryption is wrong*. On a project that already pins `sqlx =0.9.0` exactly, that label costs one pinned
line. **The cheapest risk in the file.**"*

**Three dry corrections, consigned rather than debated:**
1. **The wrapped blob is 88 bytes, not ~60** (`kek_id` 16 + nonce 24 + ct 48). **D28's figure is wrong.**
2. **`kek_id` is DERIVED from the KEK** (`BLAKE2b(kek)[0..8]` or equivalent), **never a counter — a counter
   lies after an FR48 restore.** It enters the wrap's AAD and drives the fail-fast: orphan DEK = `kek_id` in
   DB ≠ `kek_id` of the file, **detected at startup**, with a message that distinguishes *wrong key* from
   *corrupt archive*.
3. **The FR48 blob carries the DEK *and* its `dek_id`** — otherwise the restore reintroduces exactly the
   ambiguity the AAD removes. And **the wrap's AAD ≠ the fields' AAD**: two distinct `aad_version`s, two
   schemas, documented side by side. *"The kind of collision that only shows up in production."*

**The four lines no test can check, named by the person who will write them:** `OsRng` (*a test cannot tell a
CSPRNG from a counter*) · **the AAD on both sides** (*`wrap→unwrap == identity` passes even with `aad: &[]`
on both — **the bug is symmetric, therefore invisible***) · **`decrypt()`'s intermediate `Vec` is returned
un-zeroized and lingers on the heap — no test sees it.**
**And the three things the AI assistant will hallucinate here, all of which compile and pass:**
`ChaCha20Poly1305` instead of `XChaCha20Poly1305` (**12-byte nonce — and then the random nonce BECOMES a
collision problem**) · `aad: &[]` "to simplify" · `rand::thread_rng()` instead of `OsRng`.

**Two new ACs:**
- **AC12-3 — `kek_id` tampered → unwrap fails. It exists ONLY because of the AAD, and it is INEXPRESSIBLE
  with `age`.** *The judge of the whole decision._
- **AC12-9 — the FR48 blob decrypts with the standard `age` CLI, outside opencmdb.** **An AC a human can
  verify without our binary. The best AC of the set.**
**AC12-1 (`wrap/unwrap == identity`) is DELETED — it tests someone else's crate.** And **the backup
byte-grep is NEUTRAL on D31** — *"do not use it as an argument from either side: that is decision theatre."*

### D32-amended — `Live + Reduced` is GREEN with a scope label. Never orange.

**The two-axis model was right, and the UI projection re-flattened it.** D32 wrote `Live + Reduced →
degraded`. **That line is the lie** — not the projection.
> **`degraded` means "something is wrong". `Live + Reduced` is a source that answers PERFECTLY. It is in
> full health. It just sees less. We wrote a function that takes a healthy source and paints it orange.**
> *"You found the hidden axis in the model, then re-flattened it on the screen. The portmanteau you
> dismantled in the engine, you rebuilt at the surface."*

**The cost is not a colour — it is the credibility of the orange.** *Two orange pills at 23:00: one means
"40% of probes lost, get up"; the other means "ping-only, no NET_RAW, it has been like that for six months,
sit back down." **A pill that sometimes shouts "get up" and sometimes "ignore me" no longer shouts
anything. That is crying wolf, performed by the product itself, at every poll, for six months.**_

**THE RULE:**
> **`Blind` is an INCIDENT — it has a start, it ends, it deserves a COLOUR. `Reduced` is a PROPERTY — it is
> stable, it deserves a SENTENCE.**
> **We colour what CHANGED. We write what IS.**

- **Two colours on the temporal axis:** `live` green · `blind` **grey, not red** — *"I have no news" is not
  "it is broken"; the same respect `accept-gap` extends._
- **A neutral, descriptive scope label on the structural axis:** `ping-only`, beside the name, no judgement.
- **Out-of-capability fields are `non évalué` / not-evaluated — not "in default".** They do not enter the
  gap, they do not count in a total, **they are outside the frame of the question.**
- **`degraded` survives as a word — for what it actually denotes: `live` WITH LOSS, never `live` with less
  reach.**

**And the dividend nobody had seen: reduced scope is the only place in this product where "I see less" is an
INVITATION, not a complaint.** The ping-only screen's real content is **the list of what the source cannot
see** — because the operator's action is not *"fix your source"*, it is *"ah, I could grant it NET_RAW and it
would see MACs."* **A capability to unlock, not a fault to repair. Do not paint it orange.**

### D11 — ALREADY CLOSED on 2026-07-16, in the PRD. Not reopened. `ignore` → `exclude` is the only remainder.

**D11's deferred item — *"reconcile the EN docs contradiction: a `Merge` button in a product whose pillar is
linked, never merged"* — was resolved at 16:01 by the PRD edit, and this document did not know it.** The
binding Canonical Vocabulary section says:

| Concept | EN (docs, API, code) | FR (UI) | Meaning |
|---|---|---|---|
| **Close the gap** — write observed values into the declared record, field by field | **`document`** (`document-field` / `document-all`) | **« Merger »** | The gap **closes**. The observed record is untouched; the link holds |
| Link observed to declared | **`reconcile`** | **« réconcilier »** | **A process, NEVER a button** |
| Seen, not yet decided | **`accept-gap`** | **« Accepter l'écart »** | The gap stays **open** |

> **There is no `Merge` button in an English-language product. English says `document`. « Merger » is only the
> FR UI label — and `merge` is RETIRED in English precisely because the pillar is *linked, never merged*.**
> The rule that made it work: *"the pair need not share a root — only a meaning."* **`revert` is retired
> outright: nothing is ever destroyed.**
> *"You **document** a VALUE; you **accept** a GAP."*

**A `Reconcile` button was proposed at validation and is REFUSED — by the glossary's own rule.**
`reconcile` / « réconcilier » is **already in the binding table, as the PROCESS, with "never a button"
written beside it.** Promoting it to the gesture would make one term carry two meanings — the engine's
continuous work (**the product's own name**) and a discrete act on one gap. **That is exactly the F14 hole,
inverted: the argument for `Reconcile` was "one term, one meaning", and that argument is what kills it.**
*The proposal was answering a contradiction that had already been closed two hours earlier; it was made on a
stale reading of this document's own feedback tables — see the validation error recorded above.*

**Still open, and it is the only piece of D11 that is: `ignore` → `exclude`.** The UX glossary carries it in
plain text: *"flagged: the tone disdains; `exclude` / « exclure » (out of scope) would be more honest. Not
settled."*

**DECIDED: `exclude` / « exclure ». And NOT for the tone** — *"'the tone disdains' is true but soft. The real
reason is structural:*
> **`ignore` describes the OPERATOR'S ATTITUDE. `exclude` describes the OBJECT'S STATE.** Every other gesture
> in this grammar describes the object or the relation. **`ignore` is the only one that describes a human
> feeling. It is out of grammar.** And a product whose invariant is *"I do not judge"* cannot carry a verb
> meaning *"I don't care"* in its action bar — **that is F13 in a single word: the product has no right to an
> opinion about the backlog, and `ignore` is one.**

**And it unifies with D32-amended:** `exclude` and out-of-capability become **the same concept — *this is
outside the frame of the question*.** One concept, one word, two coherent uses. **That is a glossary that
holds.** ⇒ **add `ignore` / « ignorer » to the retired-words denylist** beside `accept-as-declared`, `merge`
(EN), and `revert`.

### D58 — The product has the right to TELL. The six-month test gains its symmetric.

**The blind spot the whole document carries, and it is not the one F41 names:**
> **"You designed, with extraordinary care, everything the product REFUSES to do. Nobody designed what the
> operator FEELS when he opens the tab."**
> No nag · no badge · no gauge · no age-as-reproach · no gamification · no degradation · no `first_run` flag
> · no stored colour. **"That is a complete ethic. And it is entirely NEGATIVE. You have a product that knows,
> with surgical precision, how not to be unpleasant. It has no idea how to be GOOD."**

**And the document's best sentence carries the blind spot inside its own formulation:**
> *"If the user does nothing for six months, does the product become more unpleasant?"* — **that is a LOWER
> BOUND. It guarantees the product does not go down. It guarantees nothing upward. An empty text file passes
> it perfectly. A product that is never opened passes it 100%.**

**THE MISSING SYMMETRIC, and it is D58:**
> **If the user does nothing for six months and re-opens the tab — does the product TEACH HIM SOMETHING HE
> DID NOT KNOW?**

*The scene nobody described: Guy comes back. Six months. He opens the tab braced for pain. **The product
reproaches him with nothing — you won that.** But it tells him nothing either. It shows the same wall as six
months ago, politely, without judgement, dead. **And meanwhile six months of life happened on that network.**
Forty changes. A device gone for four months. Two new ones. A server that changed IP three times and came
back. **The journal knows all of it. And the product says nothing, because saying it would look like a
reproach.**_

> **That is the drift: by designing the refusal to JUDGE, you designed a product that refuses to TELL. You
> protected the operator from reproach so well that you also protected him from information.**

**The line is drawn by your own rule — you wrote its left half and never its right:**
> F13: *"a notification may only be triggered by a **NEW FACT**; 'it's been a while' is not a fact."*
> **Turn it over. A NEW FACT IS A REASON TO SPEAK.** *"This device has not been seen for 4 months"* — **that
> is a fact.** It is not *"you have not done anything in a while"*. **The first speaks about Guy. The second
> speaks about the NETWORK. The product has the right — the duty — to speak about the network.**

**DECIDED: yes.** *"What the backlog is missing is not a screen. **It is a NARRATOR.** D7 already gives you
the three regimes, derived from the journal. The journal knows how to tell. **Nobody ever asked it to, because
in 57 decisions the question 'what is the product allowed to SAY?' was only ever asked in its negative
form.**"*
**And the product bet, stated as a bet:** *"a backlog of gaps is something you avoid. **A story of what
happened on your network while you weren't looking — that, you open.** And the gap is inside it, without
anyone having had to brandish it."*
**Scope: Growth, not MVP** — but it is recorded now because **the anti-nag ban and the narrative look
dangerously alike from a distance: both say "time has passed". Only one makes it a reproach.** Whoever
implements the bans without this decision beside them will implement silence.

### D1-rev — Story 1 is the walking skeleton THAT SHOWS A REAL GAP

**Two different purchases, and only one was being made.** *"D1 buys: THE STACK HOLDS. A skeleton printing
`SELECT 1` on two backends proves the stack compiles. **The risk is never in askama. It is always at the
frontier of the real.** I spent two days saying the gap IS the product, and proposed as story 1 a skeleton
that contains no gap."*

**AMENDED: story 1 is the dual-backend walking skeleton *that displays a real gap on a perimeter where the
L2 cardinality is 1*** — one connector, one line on screen, green on SQLite and MariaDB. **D1's proof of
integration is INCLUDED, not bought twice.**

**The perimeter is a decided, VERIFIED property — never an assumption.** *"A subset where identity is
trivial" does not exist* (the gap is at `device` level = L2, and **L2 is never trivial in general**). What
exists: **devices whose L2 group has cardinality 1** — L2 applies, it is not bypassed, **but its result is
the identity: the box has one socket.**
> **A device only LOOKS single-interface because we have seen only one of its NICs. If the perimeter is an
> assumption, the gap LIES.** ⇒ **the slice shows a gap ONLY where cardinality 1 is ESTABLISHED; everywhere
> else it ABSTAINS, says so, and the count of "I don't know" is displayed.**
> *"That is not a partial gap presented with authority. It is a TOTAL gap over a DECLARED perimeter, plus a
> counter of ignorance. **An operator does not lose trust facing 'I don't know'. He loses it facing a false
> number. The 84 alerts are not abstention — they are its opposite.**"*

### D19-rev — The real was always an upstream dependency; it was listed as the last step

**~35 of the 50 traps hold WITHOUT the real** — traps on the **semantics of the join**, whose oracle is the
spec: MAC randomised per host, same MAC on two subnets, MAC absent on the ARP side, diverging hostnames
across connectors, a stale observation vs a declared fact, a moving DHCP alias, dual-stack, a VM with a MAC
cloned from its host. `FixtureConnector` eats them by construction — **it emits `Observation`, not UniFi
JSON.**
**~15 do NOT** — the wire format: is `mac` `aa:bb:cc`, `AABBCC` or `aa-bb-cc`? Is `hostname` **absent,
`null`, or `""`** — *three different join behaviours, and no way to guess which arrives*? Is `last_seen`
seconds or milliseconds? Does an offline client vanish from the payload or stay with a flag?
> **🔴 Written from belief, those 15 are exactly what D19 exists to forbid: a gate on a false truth.** *"I
> disqualified real captures because they have no oracle — and I was about to manufacture an oracle by
> introspection. **That is my own theatre, and it carries my name.**"* **D45 kills them: their red has no
> repair, because it will never arrive.**

**D19-rev:** **0.** the probe + record mode (the D1-rev slice, red-first on **trap #0**) → **1.** types →
**2.** the ~35 semantic traps → **3.** the ~15 wire-format traps **from the captured body** → **4.**
`FixtureConnector` → metrics harness **before the engine** → L1 join → blocker + recall → the L2 cascade one
trap at a time → seeded generator → bulk fixture → distributional diff. **Unchanged, in order, without
exception.**
**What does not move:** the capture **generates fixtures, never gates** · the generator stays seeded · **the
privacy argument in a public repo remains disqualifying and non-negotiable** · the 50 traps remain the only
guarantee. **The slice greens nothing. It makes the red honest.**
**The probe's cost, not hidden:** a **throwaway** read-only reader, run **once, locally, outside CI, outside
the repo. Nothing from that run enters the repo.** What enters: **hand-written, anonymised traps.** The
connector is deleted. **The Trojan-horse guard: the throwaway reader is NEVER called from the engine — it
writes a file, a human reads it. It never crosses the trait.**

### D57 — The impact graph (FR26-29): representation and traversal REQUIRED before the schema grows

Not CRUD. It carries three things the other uncovered domains do not:
1. **A representation choice that constrains the schema** — edge table / closure table / adjacency — **and
   the schema must hold on two engines (D1). SQLite and MariaDB 10.11.11 do not share the same history on
   recursive CTEs. It is the only one of the four that can make the dual-backend constraint lie.**
2. **Cycles.** *An impact graph with an undetected cycle is a traversal that does not terminate.* **An
   invariant, not a feature.**
3. **It touches the gap.** Impact is *"what does this gap affect"*. **The product, not an annexe.**

**⇒ ONE decision to add, not four**, with **dual-backend portability as the selection criterion and cycle
detection as a named invariant.** *Cheap to reverse only if decided before* — D1's own logic.

---

### Gap Analysis Results

**🔴 CRITICAL**
1. **The new feedback from this session — F42–F46 — is not yet in the sources.** *(F1–F37 ARE applied; see
   the coverage section. This item is five new findings, not thirty-nine old ones.)* **Blocks epics, not
   code** — and **F42 blocks F44/F46**, because the probe's result decides what they have to say.
2. **🔴 The product's SHAPE rests on an unmeasured assumption.** **No YAML trap can learn the L2 cardinality
   of the real network.** A trap encodes an *imagined* case; fifty encode fifty imagined cases. **They cannot
   say what FRACTION of the real network falls in each — a property of the terrain, not of the logic. Not
   derivable. Measurable only by plugging in.**
   - **80% cardinality 1** → abstention is an edge detail; **the specified screen is right.**
   - **30%** → **the screen says "I don't know" two times in three. Abstention IS the main experience — and
     then the UX, the 53 FRs and the screen hierarchy are all FALSE**, because they all assume the gap is the
     content and abstention the exception. *"The wall of gaps becomes the minority of the screen. The operator
     opens the product, sees a field of ruins, and does not re-open it."*
   - **Second fact, same nature: is the observed `l2_domain` the model's `l2_domain`?** If not,
     `(l2_domain, mac)` — the foundation that *"is not a probabilistic problem"* — **is a key over a field
     that does not exist the way we believe.** *No trap will say so: traps use the field as we defined it.
     **It is the DEFINITION that needs verifying, not its use.**_
   ⇒ **Closed by the D19-rev probe. Three hours now, or six weeks later.**
3. **D57 — the impact graph.** Constrains the schema; the schema must hold on two engines.
4. **Four crate selections open** (i18n, config, `/metrics`, Docker base). *(D31 is closed.)*

**🟠 IMPORTANT**
5. **Five Tier 2 patterns** never party-tested.
6. **D24 — temporal-history growth: "OPEN and unaddressed"**. **A number, one page, before the epic.**
7. **Product decisions parked:** D52's floor · the `ImplausibleResponse` threshold · the `blind → live`
   hysteresis · **the recall threshold that defines "stable"** — *the metric can be supplied (join recall over
   the traps); **the number facing it is a product arbitration.***
8. **F41 — the build order optimises for TRUTH and nothing balanced it for EARLINESS.** **Now partially
   paid** by D1-rev and D19-rev — *the only amendments that pay the renunciation in acts rather than in
   confession._

**🟢 NICE-TO-HAVE**
9. **Three measurements:** Argon2id on the real Celeron (< 300 ms) · **age's scrypt derivation on the same
   CPU — its own cost, unmeasured** · NFR2's p95 on-read.
10. **The verification list before `Cargo.toml`** — **`sqlx::migrate!`: folder format, `_sqlx_migrations`
    schema, checksums** (*"the only thing that could cost us a day"*) · **`zeroize` on key types** · **is the
    Synology keystore reachable from a container** (*"unknown; would not build on it without proof"*).

### Validation Issues Addressed

Closed: **D47 × the orphan rule** → D53 · **the invariant suite could not live in `core/tests/`** → D56b ·
**`status_exhaustive.rs` — the name lied** (*a refactor to `_ => 500` leaves the test green and the guarantee
gone*) → `status_mapping.rs` · **`build.rs` × the committed CSS** → `cargo xtask css` · **`@source` is
relative to the CSS file** → the gate counts classes in the **generated** CSS · **MariaDB "10" is a package
name** → `10.11.11-1551`, LTS to Feb 2028, exact pin · **`CHECK` is parsed and IGNORED below MariaDB 10.2.1
/ MySQL 8.0.16** → D52 · **D19 × D35** → D19-rev · **D28's unenforced `kek_id`** → D31's AAD · **D32's
re-flattened projection** → D32-amended · **D11's deferred contradiction** → **found already closed in the
PRD** (EN `document`, FR « Merger », `merge` retired in English); only `ignore` → `exclude` remained, and it
is now decided.
**And the validation step's OWN error → recorded rather than quietly fixed:** it read this document's F1–F41
tables as the state of the world when they were the state of the world **at 14:57** — the PRD had been
rewritten at 16:01 and the UX spec at 16:25. **A snapshot read as a state. That is NFR9's error, committed by
the step whose job was to catch it** — and the structural fix (marking the tables APPLIED, with dates) is
worth more than the correction, because **the next reader would have concluded the same thing.**

**And the amendment that names the pattern:** *"identity first, gap second — not negotiable, it is a data
dependency"*. **The dependency is causality and stands** — `device.gap` reads `identity_link`, so the link
must **EXIST**, satisfied by one correct row. **"STABLE" speaks about the false-positive rate over 300 hosts.
That is NFR4 — a RELEASE criterion, not a compilation precondition.** *"I stacked the two in one sentence with
'non-negotiable' in front, which turned a quality arbitration into a law of physics. **That was engineering
theatre: the vocabulary of hard dependency borrowed to armour a preference for rigour.**"*

**Open and NOT closed:** everything in the Gap Analysis. **Nothing has been ticked that was not verified.**

### Architecture Completeness Checklist

**Requirements Analysis** — [x] context · [x] scale & complexity · [x] constraints · [x] cross-cutting

**Architectural Decisions**
- [ ] **Critical decisions documented with versions** — **D31 closed**; D57 required, 4 crates unselected
- [ ] **Technology stack fully specified** — same cause
- [x] Integration patterns defined
- [ ] **Performance considerations addressed** — D24 unaddressed; NFR2 and two crypto timings are
      uninstrumented bets, named as such

**Implementation Patterns**
- [ ] **Naming conventions established** — DB, JSON, tests, crates: yes. **API endpoints, route params
      (`{id}` vs `:id`), i18n keys: no**
- [x] Structure patterns defined
- [ ] **Communication patterns specified** — JSON envelope, HTMX fragment routing, `tracing`/`batch_id` open
- [x] Process patterns documented

**Project Structure** — [x] tree · [x] boundaries · [x] integration points · [x] requirements mapping

**11 / 16.**

### Architecture Readiness Assessment

**Overall Status: NOT READY**

Five items unchecked, two under Architectural Decisions, four Critical Gaps open. **And the mechanical answer
is the true one: you cannot write `Cargo.toml` today** — D57 constrains the schema, four crates are
unselected, **and the product's shape rests on a number nobody has measured.**

> **A green here would be exactly what this document spends 3,400 lines dismantling: a gate that cannot
> fail.** The status is the instrument working.

**Confidence Level: HIGH — and that is not in tension with NOT READY.**
Confidence is about what is decided; readiness is about what is left. **What is decided has been attacked by
its own authors and survived**: five proposed gates died to D45, four withdrawn by whoever proposed them; the
superset theorem was attacked for an hour by its opponent and held; **and at validation, four authors amended
their own graven decisions on arguments from people who had not been in the room.** What is left is
enumerable: **4 selections, 1 decision, 5 patterns, 1 PRD/UX pass, 1 three-hour probe, 3 numbers.**

**Key Strengths**
- **The gate criterion (D45) is the document's real product.** It killed five gates and validated four
  mechanisms, and it generalises beyond this project.
- **The load-bearing invariants are held by the compiler, not by discipline** — `E0117`, `E0004`, `E0603`,
  borrowck, name resolution — on a project whose defining constraint is **no second human reviewer**.
- **Named theatre as a standing practice**, including by the people who proposed the theatre.
- **Non-guarantees are written as non-guarantees** (NFR9c, D51, F41). *"A threat model we cannot enforce is a
  delayed lie."*
- **The vocabulary is now load-bearing rather than decorative** (D11-closed): *"the invariant lives in two
  places — in the tests, and in the user's head. **Nothing protects the second except the word.**"*

**🔴 The one structural weakness, and it is the same one three times**
> **A frame with no opposition does not produce a blind spot by mistake — it produces one by construction.**
> **(1)** Step 4's categories were **stack** categories, not **product** categories → four FR domains never
> landed. **(2)** 56 decisions were taken with nobody mandated to push the other way → *every decision makes
> the product more TRUE; none makes it arrive EARLIER.* **(3)** The ethic is **entirely negative** → the
> product knows with surgical precision how not to be unpleasant, **and has no idea how to be good.**
> **All three were found by voices that were not in the room** — the PM, then the designer. **That is not
> luck, and it is the lesson: the counterweight must be summoned, because a frame cannot see its own edge.**

**Areas for Future Enhancement**
- **D58 — the narrator** (Growth): *"a backlog of gaps is something you avoid. A story of what happened on
  your network while you weren't looking — that, you open."*
- D24's retention/growth policy before a long-running instance exists.
- IPAM and Topology as they are reached — **deliberately, and the restraint is recorded as a decision.**
- The Postgres port — **D51 says re-audit the trait BEFORE, not during, if a non-relational backend ever
  enters the roadmap.**

### Implementation Handoff

**AI Agent Guidelines**
- **Follow the mechanisms, not the prose.** Where a rule is held by the compiler (D47, D49, D50, D53, D54),
  **the rule IS the signature** — do not restate it in a comment, and do not weaken the signature to make
  something compile.
- **When a gate reds, apply D45 before reaching for the fix:** *how many repairs does this red have, and is
  the cheapest the one we wanted?* **If the cheapest repair is the wrong one, the gate is telling you
  something about the design, not about the code.**
- **`core` never learns about `anyhow`, `axum`, `sqlx` or `askama`.**
- **Refuse to invent a threshold.** Every float that decides, every magic number, every "we'll tune it" is a
  reopened decision — bring it back to Guy.
- **In `crypto/`, four lines cannot be tested and three will be hallucinated** (D31). **Review them by hand,
  every time.**
- **And the one this validation earned:** *"a requirement your worst-case scenario satisfies is not a
  requirement"* — **it applies to your own work. Before writing a test, ask what would make it red. If nothing
  would, you are writing decoration.**

**First Implementation Priority — NOT `cargo new`.**
1. **The cardinality probe (D19-rev step 0).** Three hours, local, nothing to the repo. **First, because its
   result can invalidate the PRD, the UX spec and the screen** — and because it is the only source of the ~15
   wire-format traps and of D35's real body. **Trigger: cardinality 1 minority → STOP and rewrite the PRD
   before a single screen exists.**
2. **A SHORT PRD/UX pass — F42–F46 only.** *(F1–F37 were applied on 2026-07-16 at 16:01 and 16:25. There are
   no "five bombs": FR13's re-discovery case and FR38b's `dormant` lifecycle are already in the PRD.)* What
   remains: **F42/F44** (abstention as a displayed, grouped, first-class state — **and the probe's result
   decides what they say**) · **F43** (the number that defines "stable") · **F45** (D58, the narrator, Growth)
   · **F46** (`Live + Reduced` is green with a scope label — **and the UX spec has no source-state screen at
   all, so this is a design to make, not a line to fix**) · **`ignore` → `exclude`** in both glossaries and on
   the denylist.
3. **D57** — the impact graph, before the schema grows.
4. **The four crate selections + the `sqlx 0.9` verification list** — `migrate!` first.

**Then story 1 (D1-rev):** the dual-backend walking skeleton **that shows a real gap on a cardinality-1
perimeter, and abstains — visibly, with a counter — everywhere else.** The `WriteRepository` + `transact`
skeleton and two empty adapters **COMPILE before one line of identity logic exists** (D49). If the
HRTB-over-GAT is not green in a day, take the `Box<dyn>` escape hatch — **the risk carried alone is bounded to
one day, and that bound is the point.**

### Feedback to the PRD / UX from Step 7

> **⚠️ F1–F37 ARE APPLIED — `prd.md` 2026-07-16 16:01, `ux-design-specification.md` 2026-07-16 16:25.**
> Their tables above are a **SNAPSHOT of 14:57, not a state.** Do not read them as open work. **Only F42–F46
> below are outstanding.** *(This warning exists because the validation step read those tables as current and
> was wrong — see the coverage section. A feedback table with no applied-date is a dead document that looks
> alive.)*

| # | Document | Change |
|---|---|---|
| F42 | **PRD + UX Spec** | **🔴 The whole product assumes the gap is the CONTENT and abstention the EXCEPTION. That assumption is UNMEASURED and no spec or test can derive it.** ⇒ **the probe runs BEFORE the PRD/UX pass.** And what it really measures: *"whether this product is an inventory with holes, or a **radar with a range**. Same code. Not the same product."* |
| F43 | **PRD NFR4** | **Supply the number that defines "stable".** The metric exists (join recall over the trap suite); **the threshold is a product arbitration.** Until it is set, *"identity_link must be stable"* is a preference, not a requirement. |
| F44 | **PRD (new FR) + UX** | **Abstention is a first-class DISPLAYED state with a visible count**, not an engine outcome. **And it is a MOTIF, not a state:** *"96 multi-interface devices is not 96 failures — it is ONE question."* **That is F12's rule, and nobody had connected them.** ⇒ **if the probe returns low, F44 and F12 are the SAME feature** (group by cause, act on the cause) — **and the grouped view is not a bootstrap mode, it is the product.** **The "not evaluated" counter falls under F13: it is the radar's range, not its backlog. It does not redden, it does not grow bold, it reproaches nothing.** |
| F45 | **PRD (Growth) + UX** | **D58 — the six-month test needs its symmetric.** *"If the user does nothing for six months, does the product get more unpleasant?"* is a **lower bound — an empty text file passes it.** The missing question: **does the product TEACH him something when he comes back?** F13's rule turned over: **a new fact IS a reason to speak.** *"This device has not been seen for 4 months"* speaks about the **network**, not about Guy. **What the backlog lacks is not a screen — it is a narrator. The journal knows. Nobody asked it.** |
| F46 | **UX Spec** | **`Live + Reduced` is GREEN with a scope label, never orange** (D32-amended). **`Blind` is an incident — a colour. `Reduced` is a property — a sentence.** And the ping-only screen's content is **the list of what the source cannot see**: *"reduced scope is not a fault to repair, it is a capability to unlock — the only place in this product where 'I see less' is an invitation."* |
| F47 | **UX Spec + PRD (glossary)** | **`ignore` / « ignorer » → `exclude` / « exclure »** — the last open piece of D11; the UX glossary already carries it as *"flagged… not settled"*. **Not for the tone: `ignore` describes the OPERATOR'S ATTITUDE, `exclude` describes the OBJECT'S STATE — it is the only verb in the grammar that names a feeling, and a product whose invariant is "I do not judge" cannot carry "I don't care" in its action bar.** Unifies with D32-amended's out-of-capability: **one concept — *outside the frame of the question* — one word.** ⇒ **add `ignore` to the retired-words denylist.** **`Merger`/`document` is NOT reopened** — the EN contradiction was closed on 2026-07-16, and `reconcile` is already the glossary's PROCESS, *"never a button"*. |

---

## Probe Results — D19-rev step 0, executed 2026-07-16

_A throwaway read-only reader, run once, locally, against the developer's a UniFi gateway. **Nothing from the run
entered the repo** — the probe emits counts and shapes, never values; correlation ran on a per-run salted
hash, discarded at exit. The connector is deleted. **It never crossed the trait.**_

**It answered its question — favourably — and brought back four findings nobody was looking for.**

### F42 — CLOSED, and in the favourable direction. Sally can draw.

Measured over the **few hundred known clients** (the real denominator; only some were active at probe time), using
**only structural facts** — D13's category: *a reading, not an inference*. **The number itself is the output
of an engine that does not exist; the probe measured BOUNDS.**

```
locally administered (U/L bit) :  a large share     <-- D17 dormant territory
universal (stable) MACs        :  most               <-- the denominator that matters

LOWER bound, multi-interface   :  a modest share     (demonstrably grouped by a structural fact)
UPPER bound, cardinality 1     :  a large majority   (not demonstrably grouped)
```

**The threshold was: *"80% → abstention is an edge detail, the specified screen is right."* The upper bound
is a large majority.**
⇒ **The gap is the content; abstention is the exception. The UX hierarchy holds. F44's grouped view is a
bootstrap MODE, not the product** — and **F44 and F12 do NOT collapse into the same feature.**

**Three reservations, stated because the answer is favourable and that is when one stops looking:**

1. **`hostname` is unusable on nearly half** (MISSING or empty — but **null never**). Hostname is **one of L2's
   three named signals**. ⇒ **the multi-interface lower bound is an UNDER-estimate: you cannot group by a hostname you do
   not have.** The true multi-interface fraction is above a modest share by an unknown margin, and the margin is exactly
   the population we are blind to.
2. **a large share of the inventory is locally administered.** **D17/FR38b is not an edge case — it carries nearly half
   rows.** *The feature that keeps the central indicator from drifting upward forever is half the data.*
3. **the UniFi infrastructure devices carry many radio/vap entries — several MACs per box. D12 is validated in the wild**
   (*"any UniFi AP has a LAN MAC + a 2.4 radio MAC + a 5 radio MAC — the controller already exposes them
   separately"*). **Infra is the heavily multi-interface population**, and it is counted separately from the
   general population.

### D59 — The reference scale is a PRODUCT TARGET, not the developer's install. Nobody had checked.

| Every document says | The a UniFi gateway says |
|---|---|
| **~300 hosts** | **a few hundred clients, single-VLAN** |
| **36 subnets** | **4 networks** (2 wan + 2 corporate), **1 with a VLAN id** |
| — | **1 site** · 10 infra devices |

**Confirmed with Guy: 300/36 is the product's target audience, not his install.** It must be **written as
such** — it travelled through the product brief, the PRD, the UX spec and this document as an unqualified
"reference scale", and it is the denominator of several arguments.

**What survives:** **D18 is untouched.** Its `n=300` is the size of the **seeded bulk fixture** (Tier 2
observability), which the generator produces at the target scale by construction. *The generator makes the
reference scale; it does not measure it.* The binomial argument — *"with n=300 and zero observed events the
95% upper bound is 1%; a `<= 0.01` threshold cannot distinguish 0.5% from 2%; it is a coin toss wearing a
badge of authority"* — stands.

**🔴 What does NOT survive — and it is the sharpest consequence:**
> **The dimension the product's target is DEFINED by (36 subnets) is the one dimension the developer's own
> network CANNOT exercise (1 VLAN).**
>
> This is D46's `:memory:` argument, one storey up. There it was *"you would be testing a database you do not
> ship."* Here it is **"you would be testing a network you do not target."**

**And a correction to a claim made during the build-order round:** the abandonment calculus cited *"the real
mitigating factor: Guy is the user — 300 hosts, his NAS, his network, his pain"* as the one strong positive.
**Guy IS a user. He is not the REFERENCE user.** He has a single-VLAN home lab8 on 4 networks. The estimate's only positive
modifier rested on a number nobody had measured. **The direction of the correction is not knowable without
re-running it; the fact that it rested on an unmeasured number is.**

**And the consequence for the seeded generator, which is new work:**
> **The generator must produce a cardinality distribution at 300/36 — and that distribution is now a
> PARAMETER someone must choose, with exactly ONE empirical anchor: a single-VLAN home lab.**
> The direction of the bias is knowable even if its size is not: **infra grows with segmentation, and infra
> is the multi-interface population.** ⇒ **a large majority is very likely an OVER-estimate of cardinality-1 at the target
> scale.** Not fatal to F42 — a large majority has room — **but the generator must not encode a large majority as if it were measured
> at 300/36. It was measured on a single-VLAN home lab.**

### D60 — The UniFi capability descriptor, MEASURED. D34's `capabilities()` filled in with reality.

**🔴 The biggest single risk is CLOSED: `Fact::Uplink` EXISTS at zero privilege.**

```
Fact::Mac              mac          present on all
Fact::IpV4             ip           present on nearly all
Fact::Hostname         hostname     present on most active; usable on about half
Fact::OuiVendor        oui          often empty
Fact::DhcpLease        dhcpend_time present        network_id present on all
Fact::Uplink (wired)   sw_mac / sw_port present -> the large majority OF WIRED CLIENTS
Fact::Uplink (wifi)    ap_mac       present on most
Fact::Rtt              rtt absent · latency absent   <-- ABSENT (a scanner fact, not UniFi)
```

**L2 keeps its topology signal.** The zero-privilege connector, `Capabilities`, D34's dynamic descriptor —
all hold. *The percentages on `sw_mac`/`sw_port` look low only because they are computed over all clients;
over WIRED clients, where the fact is meaningful, it is the large majority.*

**`Fact::Rtt` is absent from UniFi — and that is NOT a reason to remove it from the enum. It is the first
MEASURED capability descriptor, and it is exactly what D34 exists for.** The generic ARP/ping scanner **does**
produce an RTT; UniFi does not. **`Rtt` is a scanner fact, not a UniFi fact — and `capabilities()` is the
mechanism that already says so.** *D19 named a Fact no single connector emits; D34 made that legal before it
was known to be necessary.*

**The trap inside the good news:** `satisfaction` is present on **nearly all** of clients. **It is not a latency.**
An implementer looking for `Fact::Rtt` in this payload will find a plausible-looking number at nearly all coverage
and map it. **`satisfaction` is a UniFi score, not a measurement of ours** — *the exact shape of a silent
wrong: it compiles, it is populated, it is meaningless.*

### D61 — `l2_domain` = `network_id`. The developer's network cannot verify it. The question stays OPEN, and it is now NAMED.

John's second fact — *"is the observed `l2_domain` the model's `l2_domain`? If not, `(l2_domain, mac)` — the
foundation that is not a probabilistic problem — is a key over a field that does not exist the way we
believe"* — **could not be answered, and the reason IS the answer:**

```
client.vlan       : MISSING on 48/48
client.network_id : present 100%   —   DISTINCT VALUES: 1
rest/networkconf  : 4 networks (2 wan, 2 corporate), 1 carrying an explicit vlan id
```

> **On the developer's network, `(l2_domain, mac)` degenerates to `(constant, mac)` = `mac`. The L1 key has
> no scope dimension to exercise.**

**The model is not wrong** — a segmented network has VLANs, and the target is 36 subnets. **But:**
- **`network_id ≡ l2_domain` is UNVERIFIED and cannot be verified here.** With one distinct value there is no
  way to distinguish a genuine L2 domain from a UniFi config object that merely correlates with one.
- **The "same MAC on two subnets" trap — one of the ~35 semantic traps whose oracle is the spec — is not
  reproducible on the developer's network.** It stays a synthetic trap, which is legitimate (its oracle is
  the spec, not the terrain) — **but it means the L1 key's scope dimension ships with zero contact with
  reality until a segmented user appears.**
- ⇒ **Named as an open risk, not closed by silence.** The first user with real VLANs is the first
  verification of the foundation. **`network_id` is adopted as `l2_domain` on evidence of shape, not of
  behaviour.**

### D62 — The UniFi version matrix: the real axis. D35's "3.x and 4.x" names nothing.

```
UniFi OS console : <5.x build>          (<gateway-model>.<build>)
ucore            : <5.x build>
Network application : <10.x build>      (previous: <prior 10.x build>)   <-- THIS is what the connector talks to
```

**D35 bounds the fixture matrix as `unifi-3.x/` and `unifi-4.x/` and calls it a product decision — *"we
support 3.x and 4.x, full stop."* Those numbers correspond to nothing on the target device.** The matrix was
bounded without anyone looking at a real controller.

- **The axis that matters is the Network application version (10.x), not the UniFi OS console version.** They
  are different products on different release trains, and **the connector speaks to the former.**
- **The version is also the fixture's provenance tag** (D35: *"each fixture carries UniFi OS version +
  capture date + a re-capture job that diffs the schema"*). **Tagging a capture `unifi-4.x` when the payload
  came from Network a 10.x build is a fixture that lies about its own provenance** — and the re-capture job diffs
  against that tag.
- ⇒ **`fixtures/capture/network-10.x/`, not `unifi-4.x/`.** And **the supported matrix is a product decision
  that is now RE-OPENED, on the correct axis, with a real number to anchor it.**

### Wire format — the ~15 traps, now written from measurement instead of belief

_D19-rev step 3. **These were the traps D45 killed** — "written from belief they are a gate on a false truth;
their red has no repair, because it will never arrive."_

| Field | Measured | Resolves |
|---|---|---|
| `mac` | **lowercase, `:`-separated, 100%** — no `AABBCC`, no `aa-bb-cc` | one trap, closed |
| `hostname` | **MISSING and empty both occur · null NEVER** | **TWO of the three behaviours occur. `null` does NOT.** *The trap set must encode absent and empty — and must NOT encode null, or it tests a case the source cannot produce* |
| `last_seen` | **10 digits = SECONDS epoch** (not ms) | one trap, closed |
| `oui` | **`""` on a large share** | `Fact::OuiVendor` is often empty — **a named Fact that is usually absent** |
| `vlan` | **MISSING 100%** | see D61 |
| `sw_port` | integer, 1–2 digits | |
| `is_wired` | bool, 100% | the wired/wifi split is total |
| `network_id` | `str(len=24)`, 100%, **1 distinct** | see D61 |
| **distinct keys** | **127** | ***D19's `Fact` enum names 7.*** The payload is many-fold the model's surface — **that ratio is the drift surface D35's mutation fixtures exist to cover** |

### D63 — D27 binds `xtask` too. The probe violated it, and the violation is not an anecdote.

**The probe took the UniFi password from an environment variable on a command line. D27 refuses exactly
that, in this document, three hours earlier:**
> *"**Env var in the compose:** key in cleartext in a file the backup tool carries away. **Total theatre.** And
> independently: **it leaks into `docker inspect`, `/proc/<pid>/environ`, and Synology logs.** Rejected."*
> — and D27 kept **`..._KEY_STDIN`** as the option precisely for the case where a human is present.

**What it cost, concretely:** the harness recorded the approved command verbatim into
`.claude/settings.local.json` — **a file inside the repository**. It never reached git (the repo is not
initialised), and it was removed. **But the mechanism that put it there is the mechanism that would have
committed it.**

> **A decision that does not bind its own tooling is a decision nobody actually believes.** D27 was written
> for the shipped binary. The probe is not the shipped binary — **and it is the first of a family**:
> `xtask recapture` needs the same UniFi credential (D35/D56); FR48 restore testing needs a passphrase; the
> record mode needs a controller. **Every one of them is a dev tool that touches a real secret, and `xtask`
> is where they all live.**

**DECIDED — D27's rule binds `xtask` and every dev tool, not only `opencmdb-bin`:**
- **A secret reaches a dev tool on STDIN, never through the environment, never through an argument, never
  through a file in the tree.** Same mechanism D27 already chose (`..._KEY_STDIN`), applied to the side of
  the wall where a human is always present — *which is the side where stdin is free.*
- **The tool that reads it never echoes it, never logs it, and never writes it anywhere.**
- **`cargo xtask ci` gains the cheapest gate of the set** — the byte-grep of D26, pointed at the repo itself
  rather than at the backup artefact: **no credential-shaped string anywhere in the tree.** *It costs one
  line, it can fail on a real gesture, and it just did.*

*Recorded because the alternative is to treat it as carelessness. It was not carelessness: **the doctrine was
scoped to the product, the tooling was outside the scope, and the scope was never stated.** That is the same
shape as every finding in this document — a guarantee true where it was written and false where it counted.*

### Feedback to the PRD / UX from the probe

| # | Document | Change |
|---|---|---|
| F48 | **PRD + product brief + project-context** | **State that ~300 hosts / 36 subnets is a PRODUCT TARGET, not the developer's install** (measured: a single-VLAN home lab (a few hundred clients, one site)). It travelled through four documents unqualified. **And record the consequence: the dimension the target is defined by — segmentation — is the one dimension the developer's network cannot exercise.** *"You would be testing a network you do not target."* |
| F49 | **PRD NFR8 (D35 matrix)** | **Re-open the supported UniFi version matrix on the correct axis.** *"We support 3.x and 4.x, full stop"* names nothing: the device runs **the UniFi OS console (a 5.x train)** and **the Network application (a 10.x train)**, and **the connector speaks to the Network application.** Fixture paths become `capture/network-10.x/`. **A fixture tagged on the wrong axis lies about its own provenance — and the re-capture job diffs against that tag.** |
| F50 | **PRD FR38b / UX** | **`dormant` is not an edge case: a large share of the known inventory is locally administered.** The feature that keeps the central indicator from drifting upward forever **carries half the data.** Size the UX and the retention policy accordingly. |
| F51 | **PRD FR9 / UX** | **`hostname` is unusable on nearly half of known clients** (MISSING/empty, never null). It is **one of L2's three named signals**, and it is absent on nearly half the population. **The abstention rate is bounded below by this, not by the engine's quality** — and F42's favourable majority is an over-estimate by exactly this margin. |

---

## Post-completion: 2026-07-17 — the storage engine narrows. D64.

_Party mode, requested by Guy: Winston, Murat, John, each having re-read the sources. Guy decided at the
end of it. **The debate is closed and is not to be re-opened** — what follows records the decision, its
motive, the four decisions it revokes, the two an author amended mid-session, and the renunciation it buys._

### D64 — SQLite is OUT, for good. MySQL is OUT. **MariaDB 10.11+ is the only supported engine.**

**DECIDED by Guy, 2026-07-17.** The locked non-negotiable *"SQLite + MySQL/MariaDB (no PostgreSQL)"* is
**revoked and replaced**: `NFR21` becomes **MariaDB 10.11+, single engine**. PostgreSQL stays out at MVP —
**but see the renunciation below: Guy has re-opened it as a possible future addition, and that changes the
price of D51 point 4.**

**"MariaDB/MySQL only" was never one engine — it was two.** Winston and Murat arrived at this separately and
neither was asked to. MySQL 8.0 is not MariaDB 10.11: different default collation (`utf8mb4_0900_ai_ci` vs
`utf8mb4_general_ci`), `RETURNING` in one and not the other, and — D57's own selection criterion — a
different history on recursive CTEs. **Keeping MySQL keeps the 2× matrix on a WORSE axis:** two cousins
diverge rarely, so the differential's yield collapses while its cost stays whole. Claiming it without CI is
**F38 word for word**, and it violates D52's rule, which is ours: *we do not claim what has no CI.*

> **The honest statement of this decision is not "MariaDB/MySQL only". It is "MariaDB 10.11+ only" — narrower
> than the old non-negotiable, narrower than the proposal as first spoken, and it is the README's day-1
> onboarding sentence.** The claimed/tested gap is **0**, and that is the whole point of saying it this way.

#### The motive, and it is deliberately NOT the one that was first given

**Two motives were offered and BOTH are refused. The decision survives without either.**

| Offered | Verdict |
|---|---|
| *"I will use MariaDB and I think most users will too."* | **REFUSED — this is D59, one day old.** *"Guy IS a user. He is NOT the reference user."* Unlike L2 cardinality, **this fact is not probe-able before there are users** — so D59's lesson here is not *measure*, it is **do not found the decision on it**. A motive that cannot be checked collapses the first time one user writes *"I just wanted a docker run"* — and then a correct decision looks wrong. |
| *"An SMB of 200-300 employees approaches a thousand devices, plus logs and history — SQLite won't hold."* | **REFUSED — checkable, and false.** SQLite's practical ceiling is hundreds of GB; what bounds it is **write concurrency, not size** — and **D21/D25 already decided single-writer, zero concurrency.** The argument bites on neither axis. *Inscribing it would put a false premise under a sound decision, which is how a sound decision comes to look wrong to the first reader who knows SQLite.* |

**What was TRUE in the second motive is the observation, not the inference: the target is an SMB, not a
Raspberry Pi.** An organisation of that size already runs a database and already has someone who provisions
one. That does not say *SQLite would not hold*. **It says no target user is blocked** — which is the motive
we record:

> **THE RECORDED MOTIVE (John's, and it is deliberately weaker than Guy's):**
> **We do not need most users to PREFER MariaDB. We need no target user to be BLOCKED by it.**
> Verifiable against our own journeys, and true. **Journey 6, which we wrote ourselves: Marc documents
> "Paperless" — Paperless-ngx + PostgreSQL + Redis. Our reference persona runs Postgres in his basement and
> documents it as an asset.** Nadia (Journey 2) runs Proxmox. **We designed for a man who is not afraid of a
> database, and we were protecting his one-line `docker run`.**
>
> **Second half, Winston's, and it is what makes the ledger decide:** *the measured ledger is one-sided.*
> DSM 7 ships **MariaDB 10.11.11-1551** natively and in the Synology backup · the priority deployment target
> is Synology Container Manager · **the one-line `docker run` DOES NOT EXIST and has not since D27** (the KEK
> lives in a separate shared folder; D44 makes it a startup REFUSAL; `compose.yaml` is already the reference
> deployment in the tree) — the real comparison is **1-service compose vs 2-service compose, six lines** ·
> the differentiator the brief actually claims is **"no Redis/workers/proxy"**, not *"no database"* — NetBox
> is five services · and the population *"SQLite, small installs"* **has never had one shred of evidence, in
> any document, at any time.**
>
> **D59 disqualifies BOTH camps: "SQLite (small installs)" was itself an unmeasured belief about a user
> population, load-bearing since the product brief, never once examined.** D59 does not say *keep the status
> quo when you cannot measure*; it says *do not found the decision on that*. **So we found it on what IS
> measured, and what is measured points one way.**

**John's standing on this, recorded in his own words:** *"Does dropping SQLite change what the product IS?
No. Zero FR moves. Zero journey moves, except Nadia's first five minutes. **This is an installation decision.
The PM has no mandate here.** And I name my own theatre first: the one-line `docker run` is Guy's romance
about his own product. If I fight for it, I am doing FR52 backwards — defending a preference by calling it a
user need."*

#### (B) is banned in writing, today

There are exactly **two** coherent versions of this decision, and the third is the one that arrives by
default:

- **(A)** keep SQLite;
- **(C)** drop it **for good**, written as a **REFUSAL**; ← **this is what was decided**
- **(B)** *"remove it for now, put it back if users ask"* — **strictly dominated: it pays the full re-entry
  cost AND gives up the certainty of the removal.**

**(B) is banned in writing, the way D21 banned the word `revert`.** F35 guarantees ≥3 people at 3 months and
one of them will write *"why MariaDB for 30 hosts?"*. **Re-introducing SQLite at story 20 is not "writing the
adapter" — it is discovering what MariaDB decided on your behalf across twenty stories, one red at a time.
That is D1, performed late, at maximum schema.** *"SQLite later" is the most expensive answer available and
it is the one everybody gives.*

#### What dies, what is reshaped, and what does NOT move

**Revoked entirely:**
- **D23** — per-dialect DDL (`migrations/{sqlite,mariadb}/`). One dialect, one folder.
- **D46** — the `dual!(fn_name)` harness. `harness::mariadb()` and its panic-without-DSN survive; the macro does not.
- **D46b** — **the JSONL verdict join. It has nothing left to join.** *"D1 is not there to check the code works on SQLite and on MariaDB. D1 is there to detect that sqlx is lying about one of them."* Three of its four rows were engine-disagreement rows. **Its AC-1 survives and changes owner:** compare `(verdict, rule)`, never `verdict` alone — that is **D19's oracle**, the mandatory `reason` in the trap YAML, and it becomes `assert_eq!(decision.rule, case.expect_rule)` in the trap runner, where Murat says it should have lived from the start. AC-2 and AC-3 die with the join.

**Reshaped:** **D48** loses its left column (`TEXT COLLATE BINARY`); `CHAR(36) ascii_bin` stands alone · **D49** loses one line — `match cfg.db` at the root; the app compiles once instead of twice · **D52** answers itself: floor (a), `>= 10.11`, and the MySQL floor is moot · **D57** loses its **selection criterion**; cycle detection remains a named invariant and MariaDB 10.11's recursive CTEs are no longer in tension with anything.

**🔴 What does NOT move — and the orchestrator's pre-brief was WRONG here, on the biggest line of its own
ledger:**
> **"D49/D50 simplify radically, the only named calendar risk of story 1 disappears" — FALSE.** The
> `Unit<'u>` GAT does not exist to abstract two engines. **It exists so `core` cannot name an sqlx type** —
> D55: *"`sqlx` is not in `core`'s `Cargo.toml` at all"*, true at one engine as at two. `core` does not
> depend on `bin`, the adapters live in `bin` (D56b), so **`core` needs the abstraction regardless. The
> HRTB-over-GAT is bought by the crate frontier (D47), not by the backend count.** Murat, independently:
> `for<'u> FnOnce(&'u mut Unit<'u>) -> BoxFuture<'u, _>` is an HRTB **over the lifetime**; without SQLite the
> **GAT** falls away, **the HRTB stays**. Risk reduced, not removed: **one day → half a day.** D50 is
> untouched — its arithmetic (MariaDB restart, NAS reboot, dropped connection, full disk ≈ 1-5/year) was
> always pleaded on MariaDB.
>
> *Recorded because the two largest entries in that ledger — one in each column — were both inflated, and
> the pre-brief reasoned from the map instead of the terrain. **That is this document's own signature error,
> committed by the party that convened the room to avoid it.***

**Honest size of the credit** (Winston, D1-rev deducted): **~2 days on story 1** · one migration file per
story instead of two · an SQLite adapter never written · **NFR16 dies entirely** (WAL, `busy_timeout`, 50
writers over 10 min) · NFR17 halves · **D24 does not die, it changes name** — temporal-history growth is a
property of the history, not of the engine; InnoDB does not hand the disk back either. **Still a number, one
page, before the epic.** Murat's arithmetic on the same credit: two DDLs ≈ **10-20 h across the MVP**, ~1-2 %
of a ~1000 h MVP — *"nobody abandons a project over 15 hours"*. **The credit is in decisions, not minutes:
7 of 63 served an engine nobody named will run.**

#### Two authors amended their own decisions, unprompted. Both amendments stand.

- **🔴 Winston amends D51, point 4.** He wrote *"the discounted cost of the class is zero — PostgreSQL is
  SQL, replacing sqlx is SQL, both real porting scenarios are intra-SQL"*, and signed it one day ago.
  **It was false.** D51's four-leak table: `implicit order`, `LIKE semantics`, `UNIQUE(hostname)` — **those
  reds are produced by the DIFFERENTIAL, not by MariaDB. Remove SQLite and they all go green.** The argument
  that made the renunciation bearable was precisely *"2/4 refute the premise — LIKE and order are NOT in
  D1's blind spot"*: he had halved the class by showing D1 saw half of it. **Remove SQLite and the class
  returns to full size while his signature does not move.** Worse: *"PostgreSQL is SQL"* was doing the work
  of *"PostgreSQL agrees with MariaDB"*, and those are not the same claim — PostgreSQL's `LIKE` is
  case-sensitive, its unordered scan is heap order, not PK order. **The only proof our trait did not depend
  on one dialect's defaults was having a second dialect in CI.**
  ⇒ **AMENDED: the cost of the class is not zero. It is unknown, unmeasurable, and paid on the day of the
  port — including an intra-SQL port.** *"I make the amendment because it is true, not because it wins."*
- **🔴 Murat amends D45's own table, on D1.** He wrote *"D1 — the red is MariaDB says no — repairs: none,
  we do not patch InnoDB."* **False.** We do not patch InnoDB — **but D23 gave us two DDL files.** Take
  `UNIQUE(hostname)`: SQLite accepts `Foo` and `foo`, MariaDB `general_ci` refuses the second. Red. Repairs:
  (a) lift the comparison out of SQL into `norm()` — the one we wanted; **(b) put `ascii_bin` on the column
  in ONE of the two DDL files — one line, an honest commit message ("align collations"), green, and the
  comparison stays in SQL.** **The red was repairable from both sides. By his own criterion (D45), D1 was a
  NEGOTIATION.** *"D23 is D1's negotiation surface. I wrote both and I never put them side by side. What we
  lose by dropping SQLite is smaller than I made it look."*

#### 🔴 THE RENUNCIATION, named — and Guy just made it more expensive

**What we give up, stated as a non-guarantee, not buried as a footnote:**

1. **The driver-lie detector is gone.** D46b was the only instrument that could say *sqlx is wrong on one of
   them*. **We are removing it in the same month we adopt `sqlx =0.9.0`** — eight weeks old, no 0.9.1, repo
   moved to a new org in May 2026 — **under the "prefer recent" rule, which was itself carried by "if your
   CI is good, age is a superstition". Half of what made the CI good was two drivers.** These two decisions
   were taken separately and they compose badly. **We know. We accept it. We do not pretend to cover it.**
2. **The "the trait absorbed a SQL assumption" class returns to full size, with no gate** (D51's named
   renunciation, now with Winston's corrected price tag).
3. **🔴 And Guy re-opened PostgreSQL as a possible future addition ("on verra"), which cashes item 2 in.**
   Winston defused his own amendment by noting *"NFR21 locks PostgreSQL out, so the realistic port is
   intra-engine and the discounted cost is still ~0"*. **That lock is now soft. The amendment becomes
   decisive: a PostgreSQL port is exactly the scenario where an unguarded, MariaDB-shaped trait is paid for
   in full, on a schema that has grown for months.** ⇒ **If PostgreSQL ever enters the roadmap, this trait is
   audited BEFORE, not during** — and the audit is scoped by D51's four leaks, at full size.
4. **The evaluator loses five minutes.** Not a user; an evaluator. *Five minutes do not buy a two-engine
   test matrix.* Recorded as a real, small, accepted cost — not argued away.

#### Conditions, and (1) is a blocker on story 1

1. **🔴 D10's enforcement must be REPLACED before story 1, not after.** D10 (*"portability is won by refusing
   SQL, not by abstracting it — no comparison descends into the engine"*) **is not a portability decision, it
   is a CORRECTNESS decision**: `norm()` in Rust is what makes hostname/MAC/IP equality deterministic, and
   identity **is** the product. Its enforcement table says: *"both engines agree | held by: **MariaDB** |
   red possible: yes, with no repair on our side"* — **that was the only row holding D10, and there is no
   compiler check for "you let the collation decide".** At 23:00, `WHERE hostname = ?` is one line shorter
   than the normalised version. Today it goes red. Tomorrow it is green, and `utf8mb4_general_ci` has just
   settled a question of identity that nobody saw pass. **Replacement, and it is CHEAPER at one dialect than
   at two:** explicit **binary collation on every text column** (D48's `ascii_bin` pattern extended) **+ a
   `cargo xtask ci` grep over the DDL** — one dialect, one file. **It passes D45 where `trait-vocabulary`
   failed:** that grep died because *"the spelling of a leak is free"* (`Pattern(String)` wraps the leak and
   passes) — **a collation is a finite, enumerable vocabulary. One red, one repair.** *This is the answer to
   John's closing question: when the 1995 adversary leaves the room, **the grep is what re-reads the trait.***
2. **The tagline dies in the same commit as SQLite.** *"the entire try-it path is a single `docker run`"*
   (brief, Vision) becomes false — **and a product whose thesis is "your documentation lies, I show you
   where" cannot ship a tagline that lies about its own deployment.** If "single binary" reads as "docker run
   and you're done" while the truth is a compose plus a GRANT, **opencmdb's first act is a gap between
   declared and observed. That is our own failure mode, applied to our README** — and it is NFR9's error
   class exactly: *true where it is written, false where it counts.* **"Single binary" itself SURVIVES
   intact** — no Redis, no worker, no queue, no required proxy; two pieces against NetBox's five. The honest
   line: **"One binary + your MariaDB. No Redis, no workers, no queue, no proxy."**
3. **John's probe, and it is the only verifiable fact in the whole debate.** The brief promises *"Synology in
   under 30 minutes, docs included"* — a number that becomes load-bearing the day we add compose + database +
   user + grant. **Guy provisions MariaDB + compose + grant on his own NAS, stopwatch running, writing the
   doc he will ship. One evening. 8 minutes → the objection is dead by measurement. 40 → we re-open, on a
   fact.** Same gesture as the cardinality probe: **it does not measure the population, it measures OUR cost
   — and that is the only one of the two we own.**

#### Dissent, recorded

**🧪 Murat dissented on the TIMING, not on the decision** — *"not no, not yet"*: story 1 is already funded as
dual by D1, and it returns **four numbers this room is currently guessing** (trap-suite time on each engine ·
whether the dual constraint catches anything in the first DDL — D51 predicted *"2-3 reds, all in story 1"* ·
whether the HRTB goes green in a day · whether `docker run` is still one line after D27). *"Three hours now
or six weeks later — here: one story already in the plan, or half a day of documentary surgery on a
prediction."* **He also named the cost nobody else did: D50 makes `IdentityIndex::for_unit` the only
constructor, so `tests/traps/` — the ~50-trap suite, the product's ONLY gate (D18/NFR4) — becomes
Docker-dependent, and D19-rev puts weeks of calendar inside that loop.** *(He named and refused the cheap
repair: an in-memory `WriteUnit` "just for the traps" is `MemoryRepository` in a smaller hat, and D51's four
legs apply word for word.)* His abandonment calculus does **not** move: **|Δ| < 5 points**, low confidence in
the number, high in the decomposition — *"P(abandon) measures whether GUY stops. F35 measures whether
strangers arrive. They are not the same variable, and my instrument only ever measured the first."*

**Guy overrode the timing and closed the debate.** The four numbers will be observed in story 1 anyway; what
they can no longer do is re-open (B).

### Feedback to the PRD / brief from D64

| # | Document | Change |
|---|---|---|
| F52 | **PRD NFR21, NFR15, NFR16, NFR17, NFR20 + scope lines 175/188/277/470/473/718** | **MariaDB 10.11+, single engine.** NFR16 (SQLite WAL/`busy_timeout`) **deleted entirely**. NFR15/NFR17 lose the dual-parity clause. NFR21 rewritten; **PostgreSQL stays out at MVP, and the "future addition" status is recorded WITH its price (D64's renunciation item 3), never as a free option.** |
| F53 | **product brief (l. 50, 65, 76, 88) + distillate (l. 16, 17, 23, 68, 114)** | **The non-negotiable is rewritten**; *"One binary, SQLite or MariaDB"* → *"One binary + your MariaDB. No Redis, no workers, no queue, no proxy."*; **"both backends pass the same suite" deleted**; **🔴 the Vision's *"the entire try-it path is a single `docker run`"* DELETED — it is now false, and it is NFR9's error class in our own shop window.** |
| F54 | **PRD (Vision/§ deployment) + README-to-be + Docker Hub description** | **Record the 30-minute claim as MEASURED or not at all** (D64 condition 3). It was written before the database was a prerequisite. |

---

## Post-completion: 2026-07-17 — D57 CLOSED, and it closes almost empty.

_Party mode: Winston, Murat, Amelia, John, each having re-read the sources. **Amelia stopped arguing and
ran a `mariadb:10.11.11` container for an hour** (destroyed after; nothing entered the repo — the D19-rev
protocol). Guy decided the scope at the end._

### 🔴 The orchestrator's brief was wrong, and all four opened with it

**It said "Journey 4 (binding): Marc creates Paperless".** Journey 4 (`prd.md` l. 425) is **"When the source
goes blind (critical failure path)"** — *that* is the one promoted to a binding requirement. **Paperless is
Journey 6, which the PRD itself titles "Sunday documentation (consultative)"** — and `prd.md` l. 213 files
impact under *consultative* a second time. **The binding label was carried from the critical path onto the
consultative journey — and it was the only procedural argument protecting FR26-29.**

> **Murat refuses to let this be an anecdote, and he is right:** *"Two days, two instances, same failure:
> **this document now has enough decisions that its own authors cannot hold them all in their heads, and it
> discovers holes where its own rules are already standing.**"* **D64 said it of its own pre-brief
> ("reasoning from the map instead of the terrain — this document's signature error"). Third instance in 24
> hours.** ⇒ **F56.**

### D57 — CLOSED. Representation was never open; traversal was decided by MEASUREMENT; the cycle invariant does not exist.

**Nothing in D57 needed a new decision. Three of its four questions were already answered by rules written
on 2026-07-16, and nobody had put them side by side.**

- **Representation — `relationship` is an SCD2 edge table on the `entity(id, kind)` supertype. Decided by
  D21 + D3 + D25, not here.** A **closure table is a cache that outlives its batch**, and D25 says
  verbatim: *"Caching: NONE. Explicitly. […] If any future cache outlives its batch, it is an ordinary cache
  and the rule applies in full."* **D14 had already won this exact argument against `link_current`, one day
  before D57 was written.** *(Winston: "I presented a non-choice as a blocking decision and left it on the
  critical path for a day. That is the step-4 defect — stack categories, not product ones — committed by the
  man who named it.")*
- **🔴 Traversal — REFUTED BY MEASUREMENT, not by doctrine.** Winston argued the recursive CTE is D48-legal
  (a join on `CHAR(36) ascii_bin` is a memcmp chain, so no dialect semantics decide) — **and he is right on
  the doctrine and wrong on the outcome, because he had not measured. Amelia did:**

  | Form | Result (measured, `mariadb:10.11.11`) |
  |---|---|
  | `UNION ALL`, no guard, 2-node cycle A↔B | **1001 rows, exit 0, 99 ms, `Warning 1931` — and `sqlx` does not surface warnings.** *The product does not crash. It lies.* |
  | `UNION DISTINCT` on `dst` alone, 200 nodes / 600 edges | 200 reached, 0.9 ms — **correct** |
  | **the same + a `depth` column** (the one the UI wants) | **199 331 rows instead of 200, 36.5 ms.** `DISTINCT` keys on `(dst, depth)`, `(B,1)` ≠ `(B,3)`, **the guard evaporates** |
  | `UNION ALL` + per-path `INSTR(path,dst)=0` | 50 nodes → 549 721 paths · **600 edges → timeout > 60 s** · 1500/51 000 → killed at 327 s |

  > **The structural result, and it is what decides: a recursive CTE CANNOT express a visited-set.** It has
  > no state shared between sibling branches. It offers exactly three things: **fast+wrong, correct+featureless,
  > correct+exponential. There is no fourth.** Not a MariaDB defect — the semantics of `WITH RECURSIVE`.
  >
  > **🔴 And 36.5 ms PASSES NFR2 with 40× of margin. NFR2 cannot catch this failure: the failure is fast.**

  ⇒ **`SELECT` the edge table via `ReadRepository`, walk it in Rust.** Measured: **51 000 edges = 32 KB;
  full `SELECT` + `ORDER BY` while a writer inserts 300 000 rows = 10.7–23.9 ms ⇒ p95 ≈ 24 ms = 1.6 % of the
  NFR2 budget.** Disjoint tables, MVCC read pool, contention ≈ 0.
  *(Amelia, against herself: she was about to assert `max_recursive_iterations = 4294967295`. **Measured:
  1000.** *"Had I not started the container I would have walked into this room with an invented number, in
  the document that wrote 'refuse to invent a threshold'."*)*
- **The cycle — there is no cycle invariant, and D57's own wording is amended by its author.** Winston wrote
  *"an undetected cycle is a traversal that does not terminate"*: **false on MariaDB** — it bounds itself and
  returns a **plausible truncated answer**, which is worse. **But Murat removes the obligation rather than
  the mechanism:** *"An invariant is a property of the DATA. 'The traversal terminates' is a property of the
  CODE. A BFS with a visited-set terminates on any finite graph. **The visited-set does not DETECT the cycle
  — it makes it moot.** You do not get a decision for writing a BFS correctly."*
- **The cycle is NEVER refused at write time. Two independent routes, and they agree:**
  - **D45:** red = the operator's INSERT is rejected. Repairs: (a) restructure the declaration — the one we
    wanted; (b) **don't declare the edge — one click, cheaper, green, and the impact view now under-states
    the blast radius in silence.** *Two repairs, the cheapest is the wrong one ⇒ a negotiation, and the
    operator wins it every time.*
  - **D8:** `A depends_on B` ∧ `B depends_on A` **is real** — two services calling each other is not a
    modelling error, it is Tuesday. *"Refusing that write tells the operator his network is wrong. FR28 says
    the operator can DECLARE — **a declaration the tool can veto is not a declaration.** On the gap, the
    oracle is the spec. **On the declared, the oracle is the operator.** And a product whose thesis is 'your
    documentation lies' that MANUFACTURES the lie is our own failure mode, in our own schema."*
- **`MAX_DEPTH` does not exist** (Amelia, contradicting the brief head-on): beside a correct visited-set **it
  can never fire** (*"a gate that has never caught anything is a tax on the gates that do"*), and if the
  visited-set is broken **it converts a loud hang into a silently truncated impact list. `MAX_DEPTH` makes
  our failure silent.** Same gesture as D49's absent `commit()` and D50's absent `new()`: **the mechanism is
  an ABSENCE.** *Non-guarantee, stated: nothing STRUCTURAL enforces this absence — a name-grep is theatre
  (D45 killed `trait-vocabulary` because "the spelling of a leak is free"). We do not claim a mechanism we
  do not have.*
- **AC-D57-1 — and the first draft was decoration, thrown out by its author.** `assert_eq!(impact(A),
  btreeset![B,C])` **passes under a `MAX_DEPTH`-only implementation** — the `BTreeSet` de-duplicates for it.
  *"A requirement your worst-case scenario satisfies is not a requirement."* **The AC that bites: the
  traversal returns `Vec<EntityId>` in BFS order, not a set.** A per-path or depth-capped walk emits
  `[B,C,A,B,C,A,…]`; **only a visited-set emits `[B,C,A]`. The de-duplication IS the visited-set.** One red,
  one repair.
  > **AC-D57-1** — `impact_of_mutual_dependency_returns_bfs_order_without_repeats`: `A depends_on B`,
  > `B depends_on A` ⇒ `impact(A) == vec![B, A]`.
  > **Determinism precondition, and it is D51 returning:** adjacency in `BTreeMap`, `SELECT … ORDER BY src,
  > dst`. `HashMap` = per-process SipHash seed = **red every other run.** *D51's row 2 (`implicit ordering`)
  > predicted this in SQL; it reappears identically in Rust, and nobody had said so.*
- **Trap suite: NOTHING** (Murat). D18's three columns are `must-not-merge`/`must-merge`/`must-abstain` —
  **the graph has no merge, no abstention, no ambiguity, and its oracle is a line the operator typed.** An
  impact trap *cannot fail*; it would test that `HashSet` works. Its own gate? **No** — red = "the blast
  radius is wrong", repairs = fix the traversal **or fix the fixture**, and at 23:00 the fixture is cheaper.
  **One inline `#[test]` in `core`, no database, not a gate.** *(He checked his own instrument for lean: his
  answer is identical with zero Docker and with full Docker, so D64's Docker cost — his own dissent — is not
  smuggling a re-litigation through the back door.)*
- **The number: no consumer, and it is NOT D24's twin.** Nobody has ever written a figure for software
  instances, applications or relationships — **verified: "software instance" appears exactly once in the PRD,
  in FR26.** Murat refuses the comfortable argument offered to him: *"'a human types it, therefore it is
  small' is **word for word the motive D64 refused hours ago** — a belief about a user population, unmeasured,
  load-bearing. 'Small' is an adjective dressed as a number."* **The real asymmetry is who holds the pen:
  D24 grows written by the MACHINE while nobody is watching — it gets big while you sleep, which is exactly
  why it needs its number before the epic. This one grows under a hand, on a screen, typed by the person who
  causes it. Not "therefore small" — "therefore not silent." And that formulation survives a big graph.**

#### 🔴 What D57 did NOT contain, and what survives it

- **THE ANCHOR — Winston, and only Winston.** `software.device_id` and `edge.src_id` point at entities **the
  engine owns and migrates** (D14/D15). **An edge is a human testimony about a referent — that is D15's
  definition**, and D15 says `declared_attribute.entity_id` is **NEVER** updated (*"the most dangerous line
  of SQL in the project, and it looks like a routine refactor"*). If the edge is an ordinary FK, a device
  split leaves two outcomes: rewrite the `entity_id` (the forbidden `UPDATE`), or **the edge points at a
  tombstone and impact loses a node without saying so.** ⇒ **the `hosts` edge IS a `declared_attribute` by
  its shape** — same table family, same `state`, same `pending_migration`, same tombstone rule, same *"the
  target entity is born naked"*. **One mechanism, not two.** *(Murat's "the FK points one way, it is a leaf"
  is true of the dependency direction and does not touch identity migration. He did not refute this; he did
  not read it.)*
  > **AC-M-04** (the never-written sibling of AC-M-03) — **split a device hosting 3 software ⇒ the impact
  > answer does not shrink in silence.** **This applies at MVP even with no graph: FR26 alone anchors
  > software to a device that can split.**
- **THE VERBS ARE NOT ONE RELATION — Winston.** `depends_on` must be a DAG · `hosts` is not traversed, it is
  a lookup · **`connects_to` cycles are LEGAL and must NOT be traversed**: everything connects to the switch
  and the switch connects to everything, **so a blast radius walking `connects_to` returns the entire network
  in three hops — at Marc's three nodes.** *Not a scale bug. A semantics bug, visible at n=3.* ⇒ **F55.**
- **🔴 A BOMB THAT IS NOT D57 — `Reads` does not compile.** D49's prose says *"the `Reads` impls for
  `ReadRepository` and for `Unit` are two-line delegations"*. **`ReadRepository` is `&self` (shared pool,
  concurrent API reads); `Unit<'u>` is `&mut self` (one exclusive transaction). One trait cannot be both.**
  sqlx escapes by implementing `Executor` on the *reference* — and **`core` cannot name `sqlx::Executor`
  (D55)**, so `core` cannot express "generic over an executor". ⇒ **`Reads` must be TWO traits, duplicated
  signatures, delegating to a generic free function in `bin`. D49's sentence is false in the singular. This
  is STORY 1, not D57.** *(`trait ReadRepository` at l. 2675 currently has a comment for a body and zero
  methods; `Reads` appears in no signature and no file in the tree.)*
- **A breach in D64 condition 1 — Amelia, against Winston.** The DDL grep is real and justified
  (`collation_server` measured = `utf8mb4_general_ci`), **but it is structurally blind to DERIVED
  expressions**: `INSTR(CONCAT(path, ',', dst), …)` compares an expression whose collation is declared in no
  DDL. *True for columns. False for expressions.* **Moot today because the CTE is refused; real the day
  someone compares an expression rather than a column.** ⇒ **F57.**

### D57-scope — DECIDED by Guy: John's split. The MVP ships declarations, never beliefs.

**John asked his question honestly and it did not cut the way it cut on FR52:** *"Nobody wakes up with it.
**Marc sits down with it on Sunday, with a coffee.** I killed FR52 because the answer was NOBODY — not
because it was 'Sunday'. Sunday is a real moment. **I am not killing that. I am killing something else."***

**His axis is not declared-vs-observed — he rejects it as too coarse** (FR40 lets the operator type notes
nobody observes, and that is fine): *"Everywhere else, what the operator types is a declaration **about an
entity the loop reconciles**. **FR29 is the only screen in the product that PREDICTS** — 'if this falls, this
falls' — a claim about the behaviour of the world, spoken in the product's voice, drawn from a human entry
nothing will ever re-read. **D58 gave the product the right to TELL. Nothing gave it the right to BELIEVE.**"*
**And D57's third leg inverts: there IS no gap in FR26-29.** A gap takes two halves; this has one. *"'It
touches the gap' is doing the work of 'it is next to the gap in the nav bar'."* — **Murat, independently:
FR29 says "if this device **fails**" — a failure is not a gap.**

> **THE RULE, adopted: a relationship the product may one day OBSERVE is a DECLARATION. A relationship
> nothing will ever observe is a BELIEF. The MVP ships declarations. Growth will ship beliefs the day an
> observer exists to contradict them.**

**DECIDED:**
1. **FR26 + FR27 stay at MVP** — software instances on a device, grouped into applications with owner and
   criticality. Two tables, two FKs. **No D57 in that.** And FR26 is not an orphan: **Growth's port/service
   scanning is already planned and gives it its observed half.**
2. **FR28 SPLITS, and the line is not arbitrary.** `hosts` and `exposes` are **containments** — FR26
   restated, and **the two the service scan will one day observe** → **MVP**. `depends_on` and `connects_to`
   → **GROWTH, sequenced with the port/service scanning.** *The PRD already knows how to write this sentence
   — it wrote it for the service fingerprint: "a coherence dependency, not an effort cut."*
3. **FR29 at MVP = FR39, which was already voted and which nobody had read:** *"view its full record
   (declared attributes, observation history, connection point, **hosted applications**)"*. **Marc's answer
   was already in the MVP.** It is a containment, a join, one hop — **not a traversal.**
4. **🔴 And the word "Impact" does not go on it.** *"A screen called Impact that does one hop and concludes
   'nothing else is affected' is the `satisfaction` trap with a nav entry."* **It is called "Hosted here."**
   The real impact view goes to Growth, **to join the interactive topology where the UX spec had already put
   it.** *(FR7's rule, applied: "a field that is present, well-typed and wrong is more dangerous than a
   missing one — the missing one fails loudly." **And worse than `satisfaction`: that one is observed, so a
   connector can contradict it. The impact graph is the only field in the product with no loop behind it. It
   can only rot, and nothing is watching.**)*
5. **FR26/FR27 at MVP ARE THE PROBE, and this is what makes the decision free.** In six months we **count**
   what Guy actually typed on his own network. **11 instances → the graph is a `SELECT` and D57 will never
   have existed. 300 → D57 is a real decision, taken on a fact.** *Same gesture as the cardinality probe and
   the 30-minute stopwatch: **it does not measure the population, it measures OUR terrain — the only one of
   the two we own.** And nobody has to be overridden: shipping FR26/FR27 does not cost the decision, **it
   funds it.**_

**⇒ Consequence, and it is larger than the scope: at MVP there is NO GRAPH AT ALL.** No edge table to
traverse, no representation to choose, no cycle to make moot. **Everything decided above — the Rust BFS, the
measurements, AC-D57-1, the verb semantics — is decided FOR GROWTH, recorded now so that Amelia's hour of
measurement is not re-spent.** What survives into the MVP is **AC-M-04 and the D15 anchor**, because FR26
alone anchors software to a device that can split.

**John names his own theatre, and it is the reason the split is where it is:** *"'The gap is the product' is
my sentence, and a sentence like that kills anything if you let it. My romance here is **purity**. But a CMDB
where you cannot write that Paperless needs PostgreSQL is not a CMDB, and Marc's question is real. **Purity
kills the GRAPH; it does not kill the DOCUMENTATION.** If I let it take FR26/FR27 I would be doing FR52
backwards — and this time nobody would correct me, because it would sound right."*

**Dissent, recorded:** the three technical voices all judged the graph cheap enough to keep whole (Amelia:
p95 ≈ 24 ms, zero new architecture). **John conceded the cost argument before it was made** — *"if the edge
table costs half a day, I have no cost argument. I never had one. My argument is that the product may not
assert what nobody re-reads. That one does not depend on the price."* **Guy decided on the principle, not the
price.**

### Feedback to the PRD / UX from D57

| # | Document | Change |
|---|---|---|
| F55 | **PRD FR28/FR29 + architecture** | **The four verbs are not one relation.** `depends_on` = DAG, traversed · `hosts` = a lookup, never traversed · **`connects_to` = legal cycles, DECLARABLE but NEVER traversed** (blast radius via `connects_to` = the whole network in three hops, **at n=3**). A semantics bug independent of scale. |
| F56 | **architecture (process)** | **FOURTH instance in 24 h of "reasoning from the map instead of the terrain"** (D64's pre-brief ×2 · this brief's Journey 4/6 · **and the `X` binding, below**). **The register is now larger than its authors' working memory, and it discovers holes where its own rules already stand** (D57 vs D25/D10/D21). Not an anecdote — **a property of the document at 4 700 lines.** Mitigation belongs in the epics phase, not here. **The fourth instance is the sharpest, because the map was of the orchestrator's own making:** it wrote an editHistory line claiming to have updated *"keyboard bindings E/G/S/I"*, then **built a standing question on that line for two sessions** — and the letters had never existed in the UX spec at all. **A journal entry is not a state either.** *(Withdrawn 2026-07-17 by Guy; the non-subject is now documented in the UX body: **this spec assigns no keyboard letters BY DECISION** — the criterion is muscle memory of the Superhuman/Gmail corpus, never a mnemonic for our own vocabulary, which is why `E` works and why `I` never should have existed. **Dividend: bindings are decoupled from our vocabulary — a rename can never orphan a key again.**)_ |
| F57 | **architecture D64 condition 1** | The DDL collation grep is **structurally blind to derived expressions** (`INSTR(CONCAT(...))` declares its collation in no DDL). *True for columns, false for expressions.* Moot while the CTE is refused. Cost it before any epic reintroduces SQL-side comparison. |
| F59 | **🔴 FOUR OF THE SIX DOCUMENTS (all fixed 2026-07-17) + the glossary gate** | **The 2026-07-16 rename reached TWO documents out of six.** Applied: `prd.md`, `ux-design-specification.md`. **Missed, and stale for 24 h: `architecture.md` (`pending_accept` ×5, `reverting` ×2 — the OLD state machine, ten pages from D21's own line saying `revert` DOES NOT EXIST, with `revert` on the retired-words denylist: a document contradicting itself across its own length) · `product-brief-opencmdb.md` (`accept-as-declared` ×2, `ignore`) · the distillate (same) · 🔴 `docs/project-context.md` — THE FILE EVERY AGENT LOADS AS GROUND TRUTH, which still described an "Accept-as-declared state machine".** *Found by accident, during a cleanup aimed at something else entirely.* Fixed: `in_queue -> pending_commit(deadline) -> committed \| failed`, **Undo returns to `in_queue`** (it is not a failure branch — *the operator changed his mind, nothing failed*). **THE POINT IS NOT THE FIX. On 2026-07-16 this exact class was caught (`ignore`/`exclude` left standing in the PRD while the UX renamed) and the lesson was WRITTEN: *"a cross-document rename needs a cross-document check — the per-document gate cannot see it."* It was written and it changed nothing, because it was PROSE. It happened again inside 24 hours, on a different pair of names.** ⇒ **D45 applied to our own process: a lesson that is not a gate is a lesson that will be re-learned.** The gate exists but is aimed too narrowly: **the retired-words denylist runs over the TRANSLATION FILES and covers UI VOCABULARY. It must also (a) cover renamed INTERNAL IDENTIFIERS (`pending_accept`, `reverting`) — a state name is vocabulary too, it just has no translator — and (b) run over the PLANNING DOCUMENTS, which are where the stale name survives longest and where every agent reads it as current.** *Cheapest form: one `cargo xtask ci` grep, one denylist file, every `.md` under `_bmad-output/planning-artifacts/` in scope.* |
| F58 | **PRD FR26-29, phased scope + UX nav** | **John's split, decided by Guy** — see D57-scope. FR26/27 MVP · FR28 splits (`hosts`/`exposes` MVP, `depends_on`/`connects_to` Growth) · **FR29 at MVP = FR39, and it is named "Hosted here", never "Impact"** · the impact view + traversal → Growth, beside the interactive topology. **Record the RULE: a relationship the product may one day observe is a declaration; one nothing will ever observe is a belief.** |

---

### D65 — The two open gates, grouped into `cargo xtask ci`. And the second one is not the gate anyone proposed.

**Decided by Guy, 2026-07-17: group D64 condition 1 (collation) and F59 (retired vocabulary) into the
existing `cargo xtask ci`** (D56: *all gates live there, in Rust, not in YAML — the workflow file calls it
and holds no logic*). **Both must exist before story 1**, for the same reason: each replaces something that
used to be enforced by an accident we removed.

#### Gate 1 — `xtask ci --ddl-collation`. A real gate, and its mechanism is an ABSENCE.

**Rule: every text column in `migrations/*.sql` carries an explicit binary collation.** No exceptions, and
**the absence of an allowlist IS the mechanism** — same gesture as D49's missing `commit()`, D50's missing
`new()` and D57's missing `MAX_DEPTH`. *An allowlist would make the red repairable from the cheap side, and
D45 has already named how a negotiation with oneself ends when there is no second reviewer.*

**D45 check — red: "a text column has no explicit binary collation". Exactly one repair: declare it.**
"Remove the column" is not a repair anyone reaches for, and there is no third option. **It passes where
`trait-vocabulary` failed** (*"the spelling of a leak is free"* — `Pattern(String)` wraps the leak and
passes): **a collation is a finite, enumerable vocabulary declared in one place.**

**Why it is not optional and not a chore:** it is what replaces the second engine. **D10 is a CORRECTNESS
decision, not a portability one** — `norm()` in Rust is what makes hostname/MAC/IP equality deterministic,
and identity IS the product. Its enforcement table said *"both engines agree | held by: MariaDB"*, and that
row left with SQLite. **There is no compiler check for "you let the collation decide."** Measured on the
target: `collation_server = utf8mb4_general_ci`. **The exposure is real, not theoretical.**

> **NON-GUARANTEE, named (F57): the grep reads the DDL, so it is structurally blind to DERIVED
> expressions.** `INSTR(CONCAT(path, ',', dst), …)` compares an expression whose collation is declared in
> no DDL. **True for columns. False for expressions.** Moot today because D57 refused the recursive CTE —
> **real the day any epic compares an expression rather than a column.** We do not claim to cover it.

#### Gate 2 — `xtask ci --vocabulary`. Two volets, because usage and MENTION are not the same problem.

**The naive form — "a retired word appears ⇒ red" — is FALSE, and checking saved us from shipping it.**
`ux-design-specification.md` l. 883-893 *must* say **"the state was named `pending_accept`, after one
gesture"** — that is the paragraph explaining the decision. **A document is required to record its own
history (D21: annulment is an addition). A gate that forbids it would be repaired by deleting the
explanation — the cheapest repair, and the destructive one.**

**Volet A — code + translation files: absence. A real gate.** Zero occurrences of any retired term in
`*.rs` / the i18n catalogues. **Source code has no historical mentions**; a retired name in a `.rs` file is
a live name. Clean D45: one red, one repair.

**🔴 Volet B — the six planning documents: CO-PRESENCE, not absence. This is the one that would have caught
F59, and it is structural rather than orthographic.**

> **RULE: if a document's body contains a RETIRED term and does not contain its LIVE replacement anywhere
> at all — red.**
>
> **That is the exact definition of a stale document: it knows the old word and has never heard the new
> one.** And it is precisely what happened: `architecture.md` had `pending_accept` ×5 and `pending_commit`
> **×0**; the brief had `accept-as-declared` ×2 and `document`/`accept-gap` **×0**. **Red, both.** While
> `ux-design-specification.md` — which legitimately explains its own rename — had `pending_accept` ×2 **and
> `pending_commit` ×8. Green, correctly: a document that explains a rename necessarily contains both
> words.**
>
> **D45 check, and this is why the shape is right: red = "this document knows the old word and not the new
> one". The cheap repair is to mention the new word — which IS applying the rename. The cheapest repair and
> the correct repair are THE SAME ACTION.** *There is no orthographic escape: you cannot satisfy it by
> spelling something differently, only by knowing the new vocabulary.*

**The source is the PRD's Canonical Vocabulary section** (binding, added 2026-07-16), extended on the axis
F59 named: **the denylist must cover renamed INTERNAL IDENTIFIERS, not just UI vocabulary — `pending_accept`
and `reverting` have no translator, but a state name is vocabulary too.** Current retired set:
`accept-as-declared` · `revert` · `reverting` · `pending_accept` · `ignore`/`ignorer` · `merge` **in English
only** (the FR UI verb « Merger » stands; the pillar *linked, never merged* is intact).

**Scope: the six documents, and they are finite and enumerable** — `prd.md`, `ux-design-specification.md`,
`architecture.md`, `product-brief-opencmdb.md`, its distillate, and **`docs/project-context.md`, which is the
one every agent loads as ground truth and the one that was stale longest.**

> **NON-GUARANTEE, named: volet B cannot see a rename in a document that never discussed the subject at
> all** (zero old, zero new = green, correctly — it has nothing to say). **And it does not police the
> frontmatter**, which is a journal by construction and must be free to record the old names. *We are not
> claiming a proof. We are claiming that the specific failure that occurred twice in 24 hours now has a
> red.*

**RUN BEFORE IT WAS WRITTEN — the spec was simulated against the six documents on 2026-07-17, and the
result is a measurement, not a prediction: 18 (document × retired-term) pairs in scope, 0 red.** Against
yesterday's state it goes red exactly where it should: `architecture.md` `pending_accept` ×5 / `pending_commit`
**×0**, the brief `accept-as-declared` ×2 / replacement **×0**. **And `ux-design-specification.md`, which
legitimately narrates its own rename, is GREEN with `pending_accept` ×2 alongside `pending_commit` ×8 —
confirming the co-presence shape does not punish a document for having a memory.**

> **🔴 And running it found the weakness that reasoning about it did not: `ignore` is the only retired term
> that is also an ordinary English word.** *"`CHECK` is parsed and IGNORED below MariaDB 10.2.1"* is not the
> UI gesture. **A document that used `ignore` only in that sense and never discussed exclusion would go red,
> and its "repair" — mention `exclude` — would be nonsense: the cheap repair and the correct repair would
> come apart, which is D45's own definition of a bad gate.** *Measured, not assumed: it does not happen in
> this corpus — all six documents describe the triage inbox, so all six contain `exclude`, and all six are
> green with margin.* **Recorded so that the day a seventh document joins the set and reds on an innocent
> "ignored", the gate is understood rather than disabled.** *(The clean answer, if it ever fires: `ignore`
> moves to volet A only — code and i18n catalogues, where the context is constrained and an ordinary English
> `ignore` does not occur.)*

#### Why this is D45 turned on our own process

**On 2026-07-16 this exact class was caught** — `ignore` left standing in the PRD while the UX renamed —
**and the lesson was written:** *"a cross-document rename needs a cross-document check; the per-document
gate cannot see it."* **It was written, and it changed nothing, because it was PROSE. It recurred inside 24
hours on a different pair of names, across four of the six documents, and it was found by accident during a
cleanup aimed at something else.**

> **A lesson that is not a gate is a lesson that will be re-learned.** This document has said, of five
> different mechanisms, that discipline is not a mechanism. **F59 is the day the document failed its own
> rule — and the repair is not "be more careful", it is 40 lines of Rust.**

**And note what volet B actually is: it is this product's own thesis, aimed at its own documents.** The
glossary is the **declared**; the six documents are the **observed**; **a stale document is a gap.** *We
built a reconciliation engine and then maintained our vocabulary by hand.*

---

## Post-completion: 2026-07-17 — D66. The four crate selections + the sqlx 0.9 verification, done on FACTS.

_Web-verified July 2026 (my training runs to January; every version below was checked against crates.io /
docs.rs, not recalled). The two genuine forks — the Docker base image and the `/metrics` crate — were put
to Guy, who took both recommendations. The other two are constraint-forced, recorded as selected and
revisable at `cargo add`. **This is the last thing between here and `Cargo.toml`.**_

### D66 — Crate selections. Three forced by constraints already written, one real tradeoff.

| Concern | Selected | Version (verify at `cargo add`) | Why |
|---|---|---|---|
| **i18n** | **`rust-i18n`, YAML locale files** | **4.2.1** (resolved; the web said 3.1.5 — off by a major) | **D39's decisive constraint: the glossary-uniqueness and forbidden-word CI gates (D65 volet A/B) run OVER the translation files, so the format must be greppable + diffable.** YAML is both. `fluent` (`.ftl`, ICU, heavy) buys pluralisation machinery EN/FR does not need; a binary/compiled catalogue is ruled out by the gate. `rust-i18n` compiles the YAML at build time via `t!`, but **the source of truth on disk is the YAML the gate greps** — which is the whole point. |
| **config** | **`config`** (the crate literally named `config`) | verify | 58.9M downloads = **the most boring of the three** (`config` / `figment` / `twelf`), and *boring is the whole budget-of-unknowns argument*. **It only has to read file + env and deserialize** — the three boot cross-invariants (`dormancy_window < observation_retention`; key-not-in-data-volume, AC-9b; server version floor, D52) are OUR code, run after deserialization as startup FAILURES, and belong to no crate. `figment` (serde-native, ergonomic) is the alternative if `config`'s ergonomics bite; low-stakes either way. |
| **`/metrics`** | **`prometheus` (raw), hand-written axum handler behind the auth layer** | verify | **DECIDED by Guy.** A `Registry` we own, an axum handler that `TextEncoder`-encodes it, the route added explicitly behind D26's authorization matrix. Matches the project's refusal of hidden middleware (*"do not test that axum routes"*) and keeps the scrape token exactly where D26 put it. **Rejected: `axum-prometheus`** — it auto-adds the route and auto-instruments via middleware, the precise magic D26 wants explicit and testable. `metrics` + `metrics-exporter-prometheus` (the facade) buys backend-swappability a single-binary self-hosted tool will never spend. |
| **Docker base** | **`gcr.io/distroless/static:nonroot`** (static musl binary) | — | **DECIDED by Guy.** ~19.5 MB. Bundles **CA certificates** (the connector makes outbound HTTPS to the UniFi controller — a real need the day a controller has a valid cert), **tzdata**, and a **nonroot** user out of the box. **This is the item the gap analysis flagged as "changing Murat's verdict that image scanning is theatre" — and the verdict HOLDS: no libc, no shell, so Trivy/Grype still find ~nothing.** The +7 MB over `scratch` buys three real things; `scratch` would force shipping our own CA bundle, hand-crafting `/etc/passwd` for nonroot, and doing without tzdata — *each megabyte saved paid back in code we write*. `chainguard/static` is functionally equivalent but adds a dependency on a third-party registry's tag policy — one notch less boring than Google's distroless. **Build path: `x86_64-unknown-linux-musl` fully-static → `distroless/static`.** |

### The sqlx 0.9.0 verification — the calendar risk dissolved, one CODE fact surfaced

**Verified against the CHANGELOG (docs.rs), 2026-07-17. Latest = 0.9.0, published 2026-05-06** (our docs
said 05-21 — corrected).

- **✅ The "could cost us a day" fear (migrate!) is largely PAID DOWN.** The migration **file format**
  (`<version>_<description>.sql`, `.up.sql`/`.down.sql`) and the **`_sqlx_migrations` table** (default name,
  tracked per database) are **unchanged from 0.8.** What changed is *"significant changes to the `Migrate`
  trait"* — internal. ⇒ **D23's `backup→migrate→verify` wrapper needs one confirmation at `cargo add` that
  the trait surface we drive still exists, but the day-costing schema/checksum surprise did not happen.**
- **🔴 The one CODE fact, and it is exactly what verification item (2) was for: `all query*() functions now
  take impl SqlSafeStr, only implemented for &'static str and AssertSqlSafe`.** Every literal query passes
  (a string literal IS `&'static str`). **Any dynamically-built SQL** — an `IN (?, ?, …)` with N params, an
  assembled clause — **must be wrapped in `AssertSqlSafe`, a deliberate friction point.** Not a blocker; a
  style constraint to know before the first query is written, and a small anti-injection dividend. *(It
  vindicates the verification list: the item existed precisely because this API moved.)*
- **✅ TLS:** the deprecated *combination* features (`runtime-tokio-native-tls` …) were **deleted** ⇒ use
  the split `runtime-tokio` + `tls-rustls-ring`. Confirm the exact `tls-rustls-ring` feature name at
  `cargo add` (item 5). **`mysql` and `sqlite` remain stable features**, not experimental (item 6). *(We
  only ship `mysql` now — D64 — but the sqlite driver's continued stability is moot, not a risk.)*

### Crypto crate versions (D31), web-verified

`chacha20poly1305` **0.11.0** · `zeroize` **1.9.0** · `argon2` **0.5.3** — resolved from the real lock 2026-07-17 (**the web had said 0.10.1 and 1.8.1 — both off; the lock is the truth, which is the entire reason for the standing rule**). All current, all RustCrypto,
all confirmed present. **The D31 verification (AAD on the DEK wrap, `zeroize` on Drop of key types, Argon2id
timing on the target Celeron < 300 ms) is unchanged and still owed at implementation — versions are not the
risk; the timing measurement on real hardware is.**

### What this closes, and what it leaves

**⇒ `openCriticalGaps` #1 is CLOSED. `Cargo.toml` can now be written** — one engine (D64), the deps named
and versioned, the sqlx surface checked. **Remaining before story 1, in the now-correct order:** (a) write
`Cargo.toml` + the workspace skeleton (D47/D55/D56b); (b) **the two D65 gates in `xtask ci`** (they are Rust
in `xtask`, which needs the workspace — hence they come AFTER this, not before, the ordering the memory once
had backwards); (c) the D64 condition-1 DDL collation, written with the first migration; (d) story 1 — the
walking skeleton, minding the `Reads`-does-not-compile bomb (two traits, not one). **All version pins are
"verify at `cargo add`, pin from the real `Cargo.lock`, commit it, build `--locked`" — never invent a
number** (the standing rule, and the reason this was web-verified rather than recalled).

---

## Post-completion: 2026-07-17 — STEP 1 BUILT. The workspace exists, compiles `--locked`, and the pins are REAL.

_This is the first code in the repo. The walking skeleton's identity logic is story 1 (still ahead); this
is the workspace + dependency foundation, and its whole point is **proof of integration** — the thing a
starter would have given and that D-`Starter` said we would buy back here._

### What was built (Rust 1.96.1, edition 2024, resolver 3)

```
Cargo.toml                       # workspace: members + default-members = opencmdb-bin (D56)
crates/opencmdb-core/            # domain. deps: chrono, serde, thiserror, uuid — NOTHING else
crates/opencmdb-bin/             # composition root. axum, sqlx, askama, tokio, config,
xtask/                           # rust-i18n, prometheus, rust-embed, crypto, anyhow, tracing
```
Three compiling crates, minimal `lib.rs`/`main.rs` placeholders. **No repository traits yet** — the
`Reads`-does-not-compile bomb and the HRTB-over-GAT are story 1, deliberately not touched here.

### Verified, not asserted

- **`cargo build --workspace --locked` → GREEN.** The lock exists and the build honours it.
- **🔴 Proof of integration achieved:** sqlx 0.9.0 (`mysql` + `tls-rustls-ring` + `migrate` + `macros`)
  compiles *alongside* axum 0.8.9, askama 0.16.0, config, prometheus, tokio, the crypto trio and
  rust-embed. **This is the "proof that they hold together" the starter section said must be bought back —
  bought.**
- **D47 frontier holds — verified against the dependency GRAPH, not the manifest text:**
  `cargo tree -p opencmdb-core` contains **no** anyhow / axum / sqlx / askama / tokio. *(A first check
  grepped the TOML and false-🔴'd on the comment that NAMES the ban — the map lied, `cargo tree` gave the
  terrain. F56 in miniature, caught.)*
- **xtask isolation (D56) holds:** `cargo tree -p opencmdb-bin -e normal` does not contain `xtask`. It is a
  member and a dependency of nobody.
- **The `--locked` requirement is now real, not aspirational:** `Cargo.lock` is generated and green.

### The REAL pins (from `Cargo.lock` — never invent a number; these SUPERSEDE any recalled version)

| crate | pinned | note |
|---|---|---|
| `axum` | **0.8.9** | matches the web-verified value exactly |
| `askama` | **0.16.0** | exact |
| `sqlx` | **0.9.0** (`=`) | features confirmed at `cargo add`: `runtime-tokio,tls-rustls-ring,mysql,migrate,macros` all valid under those exact names — **verification item (5) CLOSED** |
| `tokio` | **1.53.0** | features = `full` (trim later if footprint demands) |
| `config` | **0.15.25** | the boring choice, D66 |
| `rust-i18n` | **4.2.1** | 🔴 web said 3.1.5 — off by a MAJOR |
| `prometheus` | **0.14.0** | raw crate, D66 |
| `rust-embed` | **8.12.0** | |
| `uuid` | **1.24.0** | `v7` |
| `serde` | **1.0.228** | `derive` |
| `chrono` | **0.4.45** | `default-features = false` (dropped wasm-bindgen + windows-* cruft — a Linux NAS binary) |
| `thiserror` | **2.0.18** (direct in core) | a transitive 1.0.69 also in the graph — normal semver coexistence |
| `chacha20poly1305` | **0.11.0** | 🔴 web said 0.10.1 |
| `zeroize` | **1.9.0** | 🔴 web said 1.8.1 |
| `argon2` | **0.5.3** | matches |
| `anyhow` | **1.0.103** | bin + xtask only, never core |

**Three of the recalled/web versions were wrong (rust-i18n by a whole major, chacha20poly1305, zeroize).
This is the standing rule vindicated in one afternoon: web-verify to choose, but PIN FROM THE LOCK.**

### sqlx 0.9 — the CODE facts confirmed against the compiler, not the changelog alone

- `mysql` + `tls-rustls-ring` features valid → **`tls-rustls-ring` exists under that exact name** (the
  combination `runtime+tls` features were the ones deleted, not the split ones — confirmed).
- `migrate` + `macros` valid → `sqlx::migrate!` is available. **D23's wrapper surface still needs the
  first real migration to exercise it, but the feature is there.**
- The `impl SqlSafeStr` change on `query*()` (D66) is a source-level constraint that only bites when the
  first query is written (story 1) — no manifest consequence.

### The FIRST `git status` — the test D63 / the .gitignore were written for. It surfaced a real finding.

`git init` was run (not committed — a commit needs Guy). The dry-run add:
- **✅ Zero secret-shaped or `target/` files would be tracked.** The `.gitignore` (written before `git init`,
  deliberately) does its job on the class it was built for. **D63 holds: no credential-shaped path enters.**
- **🔴 But 1216 files would be tracked, and 1174 of them are `.claude/` (installed BMad skills) + 23 are
  `_bmad/` (the installer-managed framework).** These are **vendor, regenerated by the installer, declared
  read-only in CLAUDE.md** — and the `.gitignore` does not exclude them. **This is exactly what "the real
  test is the first git status" was meant to catch** (the memory's words). It is a `.gitignore` decision,
  and it is Guy's: version the agent tooling with the project, or ignore it. **Not resolved here — surfaced.**
- The code + docs + planning artefacts that SHOULD be tracked are ~19 files. The signal is drowned 60:1 by
  vendor until this is decided.

### CLAUDE.md updated

The stack now exists, so CLAUDE.md's "no build/lint/test commands yet — do not invent them" is replaced with
the real ones (`cargo build --workspace --locked`, `cargo test --workspace`, `cargo clippy --workspace -D
warnings`, `cargo fmt`, `cargo xtask ci`).

### What remains before story 1 is now SMALLER

(b) the two D65 gates in `cargo xtask ci` — **now buildable, the workspace exists** · (c) the DDL collation
grep with the first migration · **and one thing Guy must decide: the `.gitignore` vendor question + whether
to authorize the first commit** (step 1's definition includes "commit the Cargo.lock" — that needs his word).

---

## Post-completion: 2026-07-17 — PRIVACY SCRUB. The docs recorded the probe's VALUES; the probe's whole design was "shapes, never values." Guy caught it.

**On the way to creating the GitHub repo, Guy stopped it: *"I asked you not to record my private data."* He
was right, and the violation is precise: the D19-rev probe was designed to emit COUNTS AND SHAPES ONLY,
never a value (D19-rev step 0, verbatim: *"nothing entered the repo; the connector deleted after"*). **But
the write-up of the probe's FINDINGS (D59–D63) recorded the raw values anyway** — the reference gateway's
model, its exact UniFi build numbers, its client count, its gateway IP. **The document violated the exact
principle the probe was built on**, and it did so in the section explaining that principle. *(Same shape as
every finding here: a rule stated in one place, broken in another. It is F56's family, applied to privacy.)*

**Scrubbed across all six documents, 2026-07-17, BEFORE any push (nothing was ever remote):** the gateway
model → `a UniFi gateway`; exact builds (`<5.x build>` / `<10.x build>`) → genericised, **the ARCHITECTURAL
FINDING kept intact** (D62: console and Network-application are different release trains, the connector
speaks to the latter, D35's `3.x/4.x` matrix was bounded on the wrong axis — none of which needs a real
build number); the client counts → *a single-VLAN home lab (a few hundred clients)*, **keeping the
load-bearing D59 contrast** (the developer's net is single-VLAN; the 36-subnet target is what he cannot
exercise); the two illustrative IPs → the RFC 5737 documentation range.

**Kept, deliberately — and flagged for Guy's call:** the DERIVED PERCENTAGES (a large majority single-interface, nearly half
hostname-unusable, a large share dormant, the large majority wired-uplink). These are aggregate proportions, not identifying values —
you cannot fingerprint a network from *"a large majority of its devices are single-interface"* — and they are the
load-bearing conclusions (that majority is what validated the product's whole shape, F42). **If Guy wants even the
proportions gone, they go; the finding would then read qualitatively ("a large majority") at some cost to
falsifiability.**

**Git was purged, not just the working tree:** the commit was amended and `git reflog expire --expire=now
--all && git gc --prune=now` removed the dangling objects, so the raw values exist nowhere in the repo —
history included. **Verified by fixed-string `git grep` across the commit: zero occurrences** (bar
coincidental substrings inside public Cargo.lock checksums). **The lesson, and it is a gate we do not yet
have: a probe that emits shapes-not-values must bind its WRITE-UP too, exactly as D63 found that D27 must
bind `xtask` too. A principle that does not bind its own documentation is a principle nobody believes.**
