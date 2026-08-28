# Story 6.5: The entity supertype, the device, and the state column — schema only

Status: **ready-for-dev** — contexted 2026-08-28 against the migrated schema of a live
`mariadb:10.11.11`, not against the DDL text alone.
⚠️ **`create-story validate` (two fresh-context agents) is MANDATORY before `dev-story`** — Guy's
decision of Epic 4's retrospective, which overrides the template's *"validation is optional"* banner.

🔴 **ONE ARBITRATION IS OPEN AND THE STORY CANNOT BE BUILT WITHOUT IT** — §0c. The epic's criterion
says *"no producer: nothing writes a device yet"*, and the product **already mints entity ids and
writes them**. What the supertype does about the rows that exist is not a detail of the DDL; it is
the story.

## Story

As the next developer,
I want the tables a device grouping needs to exist before anything groups,
So that the schema is a decision of its own rather than a side effect of the first writer.

## Acceptance Criteria

*(Source: `epics.md:1812-1828`, story 6.5, quoted in full below. Everything beyond it is §0's, and
§0 says which.)*

**The epic's three criteria, verbatim:**
1. **Given** story 5.9's three deferrals — *no `device`, no `entity` supertype, no `state` column* —
   deferred as one block, **when** this story ships, **then** all three exist, with their binary
   collations (D64) and their adapter, and **no producer**: nothing writes a device yet.
2. **Given** the `state` column, **when** it is defined, **then** it admits the lifecycle FR38b needs
   (`active`, `dormant`) as an **enumerated domain in DDL, not a free string**.
3. **And** it follows the 5.9 / 5.9b split exactly: **this story is the schema, story 6.12 is the
   resolver that fills it.** *(Guy's arbitration 2026-08-12: the supertype comes now, not later.)*

**AC1 — `entity(id, kind)` exists, and the disjunction is STRUCTURAL rather than conventional.**
D21's shape: composite `UNIQUE (id, kind)` on the parent, a `kind` column with an enumerated domain,
and each subtype carrying a constant `kind` with a composite FK back. ⚠️ *Polymorphic
`(entity_type, entity_id)` is refused by name in the architecture: no FK is possible on either
engine, so orphans are guaranteed — found in production, never in test.*

**AC2 — `device` exists and has NO business columns.** This is a NEGATIVE requirement and the
architecture states it as one: *"Everything a device 'is' is either observed (via its interfaces) or
declared. A device is an identifier and nothing else. **If anyone proposes adding `hostname` to it,
they have just restored the OBSERVED/DECLARED merge we forbade.**"* 🔑 A criterion phrased as an
absence needs a carrier that is not a test over what exists — see AC7.

**AC3 — `state` is an enumerated domain in DDL, admitting `active` and `dormant`.** ⚠️ **Its scope is
contested between two documents and §0d says so**: `epics.md` puts it here, `deferred-work.md:2226`
assigns the same column to *"the lifecycle epic (FR40-42)"*. This story ships the DOMAIN and no
behaviour: nothing sets `dormant`, nothing sweeps, nothing reads it.

**AC4 — Every text column carries an explicit binary collation (D64), and the `ddl-collation` gate
proves it.** ⚠️ That gate is *"a reflex gate, not a proof… it bites on a real migration once one
exists"* — this is such a migration, and it is the first since story 6.3's.

**AC5 — The adapter, and nothing that writes a device.** On story 5.9's precedent: the types and the
repository functions the schema needs, with the write path exercised by tests and by **no production
caller**. ⚠️ Story 5.9's M3 came back GREEN because its adapter could not emit an incoherent pair —
*a DDL guard reachable only by going around the adapter is measured by raw SQL or by nothing.*

**AC6 — The migration is SAFE ON A DEPLOYED DATABASE, and that is measured rather than assumed.**
`v0.2.0` is published to Docker Hub and `sqlx::migrate!` runs on every boot (`main.rs:530`), so this
DDL will meet tables that already hold rows. 🔴 §0c is the reason this is an AC and not a note.

**AC7 — What cannot be measured by running code is stated, and what can is measured.** Story 5.12's
sentence, third application: *you cannot measure the absence of code by running code.* AC2's *"no
business columns"* is such an absence. Say whether it gets a gate, a test or a written limit — and
if a written limit, say so rather than implying coverage.

**AC8 — The live count lives in THIS file**, every figure naming the state it was taken against.
Baseline: **761 tests** (503 bin + 161 core + 97 xtask) at `master` = `4ef546f`, nine
`cargo xtask ci` gates, five migrations, five tables.

---

## §0 — What contexting MEASURED

*Method: `master` at `4ef546f`; the schema read from a `mariadb:10.11.11` the binary had migrated,
not from the `.sql` text alone.*

### §0a. THE SCHEMA AS IT IS — five tables, and no `entity` among them

| table | migration | note |
|---|---|---|
| `declared_attribute` | `0001` | PK `(entity_id, attr_key)` · 🔴 **`entity_id` has NO foreign key** |
| `observation_record` | `0001` | |
| `interface` | `0002` | PK `id`; `INDEX interface_l1_key` deliberately NOT unique (D21) |
| `identity_link` | `0002` | FK → `interface (id)` |
| `link_candidate` | `0002` | FK → `identity_link`, FK → `interface` |

**There is no `entity` table and no `device` table.** `0003` adds a resolver FK, `0004` relaxes an
interval CHECK, `0005` adds the document guards.

### §0b. 🔴 THE PRODUCT ALREADY MINTS ENTITY IDS, AND THEY POINT AT NOTHING

`document.rs:124`, inside the one live write gesture story 6.4 put on a screen:

```rust
let entity_id = uuid::Uuid::now_v7().to_string();
```

It goes straight into `declared_attribute` and **no parent row is created**, because there is no
table to create it in. 🔑 *The product has been creating entities since story 6.2; what it has never
had is a place to say that they exist.*

⚠️ And this is not a test-only path: **the shipped seeds carry declared rows too** —
`docker/seed-example.sql` (3 distinct ids) and `a11y/seed.sql` — so a migration that constrains
`declared_attribute` meets rows in the demo image, in CI, and in any deployment where an operator
has pressed the amber control.

### §0c. 🔴 THE ARBITRATION — what the supertype does about the rows that already exist

The epic says *"no producer: nothing writes a device yet"*. That is satisfiable for `device`. It is
**not** a description of what happens to `entity`, and three options differ in what they cost.

**(a) `entity` is created and NOTHING references it.** Cheapest, ships today, and reproduces in a
second form the objection `deferred-work.md:2222` raised against the supertype in the first place —
*"a supertype over one subtype enforces nothing — it is the speculation the 'create tables only when
the story needs them' rule refuses."* Here it would enforce nothing over **zero** subtypes with rows.
⚠️ It also leaves `declared_attribute.entity_id` pointing at nothing for at least seven more stories.

**(b) `interface` becomes a subtype and is BACKFILLED.** This is what makes the disjunction real:
`entity` gains one row per existing interface, and `interface` gains `kind` + the composite FK.
⚠️ **Then the migration WRITES, which "schema only" does not describe** — and a deployed database is
where it writes. Measurable, and AC6 is where it gets measured.

**(c) `declared_attribute.entity_id` gains an FK to `entity`.** 🔴 **This BREAKS the documenting
gesture unless it also gains a producer**: `document.rs` would have to insert an `entity` row, which
is a producer, in the story that says there is none. ⚠️ And it raises a question no document answers:
**what `kind` is a documented subject?** Guy's taxonomy of 2026-08-12 says case three is *unknown →
the operator CREATES THE ENTITY* — but the entity an operator creates from an address is not yet a
device, and `epics.md` reserves device creation for story 6.12.

🔑 **My recommendation: (a) + (b) — create the supertype AND make `interface` its first real subtype
with a backfill, and leave `declared_attribute` alone with the divergence registered by name.** It
buys the structural disjunction the supertype exists for, it keeps the one live gesture working, and
it puts the `declared_attribute` question where it belongs: with the story that decides what kind of
entity the operator creates. ⚠️ **Refused option (c) for now**, not because it is wrong but because it
is a producer wearing a constraint's clothes.

### §0d. ⚠️ THE `state` COLUMN'S OWNER IS CONTESTED, AND ITS CONSTRAINT HAS NO HOME HERE

`epics.md:1824` puts `state` in this story. `deferred-work.md:2226` says: *"No `state` column on
`interface`. D21's extended `entity.state` (`active|dormant|…`) and F17's lifecycle are read by
nothing before the lifecycle epic. **Owner: the lifecycle epic (FR40-42).**"*

Both can be honoured — **the DOMAIN here, the BEHAVIOUR there** — and the story should say so rather
than let a reader conclude FR38b is being implemented. ⚠️ In particular, **FR38b carries a startup
refusal this story must NOT implement and must NOT silently drop**: *"the dormancy window must be
shorter than the observation retention window… Violation is a **startup failure naming both
settings**, not a warning"* (`prd.md:957`). No configuration for either window exists today.

🔑 And FR38b's asymmetry is the reason `dormant` is not a general lifecycle: **only
locally-administered addresses go dormant.** *"A server powered off for six months has a
globally-unique address: it is **absent**, not dormant."* An enumerated domain that invites `dormant`
onto anything is a domain that invites that error.

### §0e. WHAT `device` MAY NOT BECOME, and where the sentinel question lands

- **No business columns** (AC2). The architecture names `hostname` as the specimen refusal.
- **`NIL_DEVICE`**: D21 names it beside `NIL_INTERFACE` (`architecture.md:1494`). `NIL_INTERFACE` is
  a *written sentinel used inside a uniqueness key*, and `interface` carries
  `interface_id_not_nil` to keep a real row out of the abstention's slot. ⚠️ **A `device` sentinel is
  needed only when something PLACES on a device — story 6.12.** What this story owes is the mirror
  CHECK, not the sentinel's use.
- **NOT a unique index on anything that means identity.** D21's rule for `interface.mac_canon` is the
  precedent: *"if we can express it in DDL, we have misunderstood the problem."*

### §0f. HOW A SCHEMA WITH NO PRODUCER IS MEASURED AT ALL

Story 5.9 is the precedent and it carries a warning: **its M3 came back GREEN**. Dropping a CHECK
left the whole suite passing, because the adapter derived both halves from one `match` and *could
not emit an incoherent pair* — the guard was reachable only by going around the adapter, and two raw
SQL inserts are what measure it. ⚠️ Expect the same here for every CHECK the adapter cannot violate.

⚠️ **And the whole story is invisible without a running MariaDB**: `DATABASE_URL` is unset locally,
every DB-backed test passes by returning, and the suite reports the same counts either way. **The
clock is the tell** — roughly 0.6 s dry against ~7 s live. Build and mutate against a real store.

---

## Tasks / Subtasks

- [ ] **T1 — Put §0c's arbitration to Guy before writing DDL** (AC6). Record the answer with the
      options refused. ⚠️ Nothing else in this story is decidable first.
- [ ] **T2 — Re-measure §0 on the tree you start from** (AC8), against a migrated store, and say
      which store and which port.
- [ ] **T3 — `0006_entity_device_and_state.sql`** (AC1, AC2, AC3, AC4): the supertype, `device`, the
      `state` domain, binary collations throughout.
- [ ] **T4 — Whatever T1 decides about existing rows** (AC6), with the migration measured against a
      store that **already holds** interfaces and declared attributes — not against an empty one.
- [ ] **T5 — The adapter** (AC5), with no production caller, and raw-SQL tests for every CHECK the
      adapter cannot violate (AC5's 5.9 warning).
- [ ] **T6 — AC2's absence: decide its carrier and say which** (AC7). A gate, a test, or a written
      limit — never an implication.
- [ ] **T7 — Mutation pass with predictions written FIRST**, through `cargo xtask mutate`. ⚠️ Its
      three carriers are cargo-side and this story is entirely cargo-side, so unlike 6.4c the driver
      covers it — say so, and use it.
- [ ] **T8 — Register every divergence BY NAME** (`deferred-work.md`), diffed before the commit.
      ⚠️ *A section that says "registered" is not a registration* (story 6b.9).

---

## Dev Notes

### What the previous stories leave you

- **Story 5.9** is the shape to copy: a schema story whose deliverable is invisible without a store,
  built and mutated against a real MariaDB. Read its record before writing DDL — **its uniqueness key
  was arbitrated TWICE**, the second time on a measurement that falsified the first.
- **Story 6.4c** (deferred to issue #131) leaves a live rule for this one: **`cargo test` and any
  browser sweep must not share a store**. Irrelevant here only if this story stays cargo-side.
- **Story 6.4b** ships `cargo xtask mutate`. Its `--expect` contract is what makes T7 mechanical.

### The house rules that bite here

- **MariaDB 10.11+ is the ONLY engine** (D64). Do not reintroduce a dialect abstraction.
- **`opencmdb-core` must not depend on `sqlx`** (D47) — the DDL and the adapter are `opencmdb-bin`'s;
  a new id type goes in `observation/mod.rs` beside `InterfaceId`/`LinkId`, as story 5.9's did.
- **Opaque ids are `CHAR(36) ascii_bin`, minted client-side** (D48).
- **`OPEN_END = '9999-12-31T23:59:59.999Z'`** and the NULL trap: MariaDB holds NULLs distinct, so a
  NULL inside a uniqueness key makes the constraint decorative.
- **Prove-to-red, prediction written first**; read every exit status from `$?` on a file, never
  through a pipe; never `git checkout -- <file>`.
- **`cargo clippy --workspace -- -D warnings` does not cover test targets.** CI's equivalent is
  `--all-targets`, plus `RUSTFLAGS="-D warnings" cargo test --workspace --locked`.
- **A guard placed where the defect cannot occur reads as coverage and is none** — Epic 5's dominant
  class, and story 5.9's M3 is its specimen in a schema story.

### Project Structure Notes

- Migrations: `crates/opencmdb-bin/migrations/`, applied by `sqlx::migrate!` at `main.rs:530` on
  **every boot** — including a deployed `v0.2.0`.
- The adapter is `crates/opencmdb-bin/src/repo.rs`; ids live in `opencmdb-core`'s `observation/mod.rs`.
- `page.rs` is at **1978** of the 2000-line ceiling. If this story needs Rust there, it **splits**.
- The `ddl-collation` gate lives in `xtask/src/main.rs` and has **no allowlist — the absence is the
  mechanism**.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#L1812`] — story 6.5's three criteria.
- [Source: `_bmad-output/planning-artifacts/architecture.md#L1475`] — D21's supertype, the composite
  FK, and the refusal of a polymorphic key.
- [Source: `_bmad-output/planning-artifacts/architecture.md#L1483`] — *"`device` has NO business
  columns"*, with `hostname` named as the specimen refusal.
- [Source: `_bmad-output/planning-artifacts/architecture.md#L1494`] — `OPEN_END`, the NULL trap, and
  *"same reasoning for `NIL_INTERFACE`/`NIL_DEVICE`"*.
- [Source: `_bmad-output/planning-artifacts/prd.md#L952`] — FR38b in full, including the asymmetry
  and the startup refusal at `:957`.
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md#L2216`] — story 5.9's three
  deferrals, split into three entries by its code review, with their owners.
- [Source: `crates/opencmdb-bin/migrations/0002_interface_and_identity_link.sql`] — the DDL idiom to
  follow: the reason for every CHECK written beside it.

### Project context reference

`docs/project-context.md` and `CLAUDE.md` are the twins; `sprint-status.yaml` is the live status.
Both twins are updated in the same push as any behaviour change.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
|---|---|
| 2026-08-28 | Story created and CONTEXTED against a migrated `mariadb:10.11.11`, not against the DDL text. 🔴 **ONE ARBITRATION IS OPEN AND THE STORY CANNOT BE BUILT WITHOUT IT**: the epic says *"no producer: nothing writes a device yet"*, and **the product has been minting entity ids since story 6.2** — `document.rs:124` mints a v7 UUID per documented subject and writes it into `declared_attribute`, which has **no foreign key**, because **there is no `entity` table**. So the supertype meets rows that already exist, in the demo image, in CI, and in any deployment where the amber control has been pressed. Three options are costed in §0c; the recommendation is create the supertype **and** make `interface` its first real subtype with a backfill, leaving `declared_attribute` alone with the divergence registered — ⚠️ **which means the migration WRITES, and *"schema only"* does not describe that.** Option (c), an FK on `declared_attribute`, is refused for now as *a producer wearing a constraint's clothes*, and it raises a question no document answers: **what `kind` is a documented subject?** ⚠️ **The `state` column's owner is contested** — `epics.md` puts it here, `deferred-work.md:2226` assigns it to the lifecycle epic; both are honoured by shipping the DOMAIN and no behaviour, and **FR38b's startup refusal** (`prd.md:957`, *"a startup failure naming both settings"*) must be neither implemented nor silently dropped. ⚠️ Measured: five tables, none of them `entity`; three distinct entity ids in `docker/seed-example.sql` alone; `sqlx::migrate!` runs on **every boot** of a published `v0.2.0`. 🔑 Story 5.9's warning is carried forward: **its M3 came back GREEN** because the adapter could not emit an incoherent pair, so every CHECK the adapter cannot violate is measured by raw SQL or by nothing. ⚠️ NEXT: `create-story validate` (two fresh-context agents) is MANDATORY before `dev-story`. |
