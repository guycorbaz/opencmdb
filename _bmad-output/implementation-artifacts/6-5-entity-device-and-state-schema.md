# Story 6.5: The entity supertype, the device, and the state column — schema only

Status: **ready-for-dev** — contexted 2026-08-28 against the migrated schema of a live
`mariadb:10.11.11`, then VALIDATED the same day by two fresh-context layers, each in its own
worktree with its own store. **§0's corrections are applied in place and §0g holds the rest.**

✅ **BOTH ARBITRATIONS ARE TAKEN (2026-08-28) and the story is buildable.** §0c is Guy's; §0d is
mine, delegated by him in the same exchange — *"je ne sais pas, c'est toi qui gère les tables"* — and
is recorded as mine so it can be reversed at the right cost.

1. **§0c — OPTION (a) (Guy).** `entity` and `device` are created, `device` is wired as a subtype from
   its first day, and **`interface` stays outside the supertype**, its adoption re-owned BY NAME to
   story 6.12.
2. **§0d — `entity.state`, the architecture's SIX values, `VARCHAR(20) … ascii_bin` + `CHECK (… IN
   (…))` (mine).**

Each is recorded below with the option refused and the cost accepted.

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

🔴 **AND THE COMPOSITE KEY DOES NOT CARRY THE DISJUNCTION — the PRIMARY KEY on `id` alone does.**
Measured on two parents side by side: with `PRIMARY KEY (id)` + `UNIQUE (id, kind)`, a second row
for one id under another kind is refused `ERROR 1062`; with `PRIMARY KEY (id, kind)` — the literal
reading of *"composite UNIQUE on the parent"* — **the same insert succeeds and one id lives under
both kinds at once**. The composite key exists ONLY to give the subtype's FK a parent index. Neither
this AC nor `architecture.md` said so; it is written here because a developer following the letter
ships a supertype that enforces nothing.

**AC2 — `device` exists and has NO business columns.** This is a NEGATIVE requirement and the
architecture states it as one: *"Everything a device 'is' is either observed (via its interfaces) or
declared. A device is an identifier and nothing else. **If anyone proposes adding `hostname` to it,
they have just restored the OBSERVED/DECLARED merge we forbade.**"* 🔑 A criterion phrased as an
absence needs a carrier that is not a test over what exists — see AC7.

**AC3 — `entity.state` is an enumerated domain in DDL over the architecture's six values.**
✅ **SETTLED by §0d's arbitration**, the first draft having named no table while three documents named
three under two domains. The domain is
`active | dormant | superseded | quarantined | pending_migration | sentinel`
(`architecture.md:1502`) — ⚠️ a **divergence from `epics.md:1826`'s two values, registered by name**.
`mac_kind`, on which `dormant`'s scope depends, exists in no table and in no `.rs` file and is
re-owned to story 6.18.

⚠️ **Whatever is chosen, the SPELLING is settled by measurement and not by taste**:
`VARCHAR(n) CHARACTER SET ascii COLLATE ascii_bin` + `CHECK (col IN (…))`, which is the idiom
`identity_link_outcome` and `identity_link_decided_by` already use. A MariaDB `ENUM` lands
`utf8mb4_general_ci` (measured), accepts `'ACTIVE'` and stores `'active'`, is invisible to AC4's
gate, and under `sql_mode=''` — which this product pins nowhere — accepts `'bogus'` and stores the
**empty string**, silently.

This story ships the DOMAIN and no behaviour: nothing sets `dormant`, nothing sweeps, nothing reads
it. **Story 6.18 is where the transition lives** (`epics.md:2046`), and §0d says why that is not the
conflict the first draft described.

**AC4 — Every text column carries an explicit binary collation (D64).** 🔴 **The gate does NOT prove
it, and the AC is narrowed to what was measured**: of twelve planted violations the
`ddl-collation` gate caught seven; of the five it passed, one is correct and **four were applied and
read back as `utf8mb4_general_ci`**. The matcher is line-oriented with no comment stripping and no
per-column split, so `_BIN` **anywhere on the line** — in a trailing comment, in a constraint name,
on a neighbouring column — satisfies it. On story 5.12's precedent this is a **TRIPWIRE against the
good-faith violation, never a barrier**, and AC4's carrier is the DDL's author plus a test that
reads `information_schema`, not the gate alone. ⚠️ It is the first migration since story **6.2**'s
`0005` — story 6.3 shipped none — and the gate has walked real migrations since story 3.2, so
*"it bites once a real migration exists"* is a sentence from before `0001`.

**AC5 — The adapter, and nothing that writes a device.** On story 5.9's precedent: the types and the
repository functions the schema needs, with the write path exercised by tests and by **no production
caller**. ✅ **Satisfiable as written under the chosen option (a)** — the contradiction measured
under (b) (65 tests red without a producer) is what disqualified it, and is kept in §0c rather than
deleted, because *the measurement that eliminated an option is the reason the choice can be
re-derived.*

⚠️ Story 5.9's M3 came back GREEN because its adapter could not emit an incoherent pair — *a DDL
guard reachable only by going around the adapter is measured by raw SQL or by nothing* — and the
validation counted the rate here: with a typed adapter, **four of six CHECKs are unreachable through
it**. Four raw-SQL tests, or four CHECKs measured by nothing. ⚠️ And a second writer already exists
outside the adapter (`diagnostic.rs:1590` inserts an interface in raw SQL): *an invariant the adapter
holds and a second writer that does not* is 5.9's M3 seen from the other side.

**AC6 — The migration is SAFE ON A DEPLOYED DATABASE, and RECOVERABLE when it is not.**
`v0.2.0` is published to Docker Hub and `sqlx::migrate!` runs on every boot (`main.rs:530`).

🔴 **The hazard is not slowness, it is that a failure STICKS — reproduced end to end.** A migration
green on an empty store and red on a populated one records `success = 0` in `_sqlx_migrations`;
MySQL DDL is not transactional; **repairing the DATA does not recover** — the next boot still answers
*"migration 6 is partially applied; fix and remove row from `_sqlx_migrations`"*, and the deployment
stays down until someone with SQL access intervenes. `0003_resolver_guards.sql:13-27` documents this
exact failure and closes with *"Production is unaffected: 0002 is unreleased."* **That sentence is no
longer available: 6.5 is the first migration published `v0.2.0` will receive.**

⚠️ So AC6 owes an **idempotent, re-runnable shape** and a **recovery recipe in the file header**, on
`0003`'s precedent — not `0002`'s, which the first draft named. ⚠️ And **no harness in this
repository can express it**: every DB-backed test migrates a store that is already fully migrated, so
*populate at version 5, then migrate to 6* is a manual procedure the story must record or a helper it
must budget.

**AC7 — What cannot be measured by running code is stated, and what can is measured.**
🔴 **This AC's premise was wrong and is corrected**: story 5.12's rule governs an **unbounded**
absence, and `device`'s columns are bounded and present in one place — so story 6b.3's arbitration
(1) is the precedent, not 5.12's. **A working carrier exists and was built**: a test reading
`information_schema.COLUMNS` for `device` and asserting the column list is exactly `[id, kind]`,
proved to red on the architecture's own named specimen (`hostname`). It catches a column added by any
route, including a later migration or a hand edit — strictly stronger than a text gate.

⚠️ **Its one limit must be written rather than implied**: it is gated on `DATABASE_URL` and passes by
RETURNING when unset, so it runs in CI and not on a dry local suite. A migration-text gate would run
everywhere and see only text. The honest answer is *both, and say which covers what*.

**AC8 — The live count lives in THIS file**, every figure naming the state it was taken against.
Baseline, re-verified independently by both validation layers: **761 tests** (503 bin + 161 core +
97 xtask) at `master` = `4ef546f`, nine `cargo xtask ci` gates, five migrations, five tables (**six
with `_sqlx_migrations`**, which any migration story touches).

**AC9 — The three register rows that name THIS STORY as owner are accepted or re-owned BY NAME.**
🔴 They were missing from the first draft, and `CLAUDE.md` is explicit: *"items are REGISTERED by
the stories that raise them — carry them IN rather than rediscovering them."* §0f lists them.

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
interval CHECK, `0005` adds the document guard (one index, singular). ⚠️ `SHOW TABLES` returns
**six** — the five plus `_sqlx_migrations`, which a migration story touches by definition. And
`identity_link` carries **two** foreign keys, not one: `identity_link_interface_fk` →
`interface(id)` and `identity_link_observation_fk` → `observation_record(id)`.

### §0b. 🔴 THE PRODUCT ALREADY MINTS ENTITY IDS, AND THEY POINT AT NOTHING

`document.rs:124`, inside the one live write gesture story 6.4 put on a screen:

```rust
let entity_id = uuid::Uuid::now_v7().to_string();
```

It goes straight into `declared_attribute` and **no parent row is created**, because there is no
table to create it in. 🔑 *The product has been creating entities since story 6.2; what it has never
had is a place to say that they exist.*

⚠️ **`document.rs:124` is the ONLY production site that mints an entity id** — every other `now_v7`
call site in `crates/` mints an `ObsId`, `LinkId`, `InterfaceId` or `ConnectorId`, and
`insert_declared_attribute`'s call sites are all inside `#[cfg(test)]` modules. Verified by
classifying every site, not by sampling.

🔴 **AND THE FIRST DRAFT'S "ROWS THAT ALREADY EXIST" WAS WRONG THREE WAYS — the validation measured
each.** The corrected picture is what the arbitration must be taken against:

- **`docker/seed-example.sql` carries ONE entity id in two rows, not three.** Three is the count of
  distinct UUID *literals* in the file; the other two are a sentinel and an observation id, and the
  file's own header says *"it inserts ONE declared entity"*.
- **That seed is NOT in the published image at all.** The `Dockerfile` copies only the binary, the
  compose declares no database container and no init volume, and the seed's own header tells the
  operator to pipe it in by hand. `a11y/seed.sql` (four entities) runs in CI only.
- 🔴 **`interface` is EMPTY on every real deployment**, and that is the measurement that matters
  most: the shipped ARP/ping connector emits no MAC, so `join` keys on nothing and **no interface is
  ever minted**. Booted against a virgin store with a live perimeter: `observation_record` 1,
  `interface` **0**, `identity_link` 1 — abstained, NULL interface — `declared_attribute` 0.

🔑 **So a backfill of `interface` writes NOTHING in production**, and the tables that actually hold
rows on a deployed `v0.2.0` are `observation_record` and `identity_link`. The backfill's cost lands
on developer and fixture-fed stores — *which does not make it free: see §0c, where the cost turned
out to be somewhere else entirely.*

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

🔴 **THE VALIDATION BUILT ALL THREE AS REAL MIGRATIONS, APPLIED THEM TO A POPULATED STORE THROUGH
`sqlx::migrate!` AT BOOT, AND PRESSED THE LIVE GESTURE AGAINST EACH. It refuted this story's first
recommendation on the single point that fed it.**

| | **(a)** | **(b)** + backfill | **(c)** |
|---|---|---|---|
| applies on a populated store | ✅ 0.13 s | ✅ 0.17 s | ✅ 0.15 s |
| boot delta at **200 003 interfaces** | **37 ms** | **2.045 s** | independent |
| the 761-test suite | **761 pass** | 🔴 **65 fail** | 🔴 **16 fail** |
| `POST /document-all`, pressed for real | **201** | **201** | 🔴 **500** |
| needs a producer | no | **YES** | YES |

🔑 **OPTION (b) NEEDS A PRODUCER TOO, AND A HEAVIER ONE THAN (c)'s.** The first draft refused (c)
because it *"is a producer wearing a constraint's clothes"* and recommended (b) because it *"keeps
the one live gesture working"*. Both halves survive; the conclusion does not. **A backfill repairs
the rows that exist and nothing repairs the rows the resolver writes on the NEXT SCAN** — 65 tests
red, every path that mints an interface. The validation then built the minimal producer (four lines
inside `insert_interface`, plus a signature change) and re-ran: **65 → 1**, the production call site
compiling unchanged and the compiler naming five test sites itself. The survivor is
`diagnostic.rs:1590`, a raw-SQL interface insert **outside the adapter**.

🔑 ***So the honest comparison is not "(b) is free and (c) is a producer". It is: (b) puts a write to
`entity` in the resolver's HOT PATH, on every scan; (c) puts one in a handler pressed by hand.***
(b)'s producer is the larger commitment of the two, and it lands squarely on `epics.md`'s *"no
producer"*. **Only option (a) satisfies that criterion.**

🔴 **And (c) is worse than the first draft said: it breaks the BOOT, unrecoverably.** Not the gesture
first — the migration itself, `ERROR 1452` on rows that already exist, after which the deployment
will not start (AC6). Its `kind` question is confirmed to have **no answer**: the validation had to
write `'device'` for six existing subjects, which is false, and `'interface'` is equally false.

### ✅ TAKEN (Guy, 2026-08-28): **OPTION (a)**

`entity` and `device` are created; **`device` is wired as a subtype from its first day** (composite
FK, constant `kind`); **`interface` stays outside the supertype**, and its adoption is re-owned BY
NAME to **story 6.12**.

🔑 **The argument that decides it is not (b)'s cost — it is that (b) SAVES NOTHING.** §0g measured
that a device cannot be a placement subject: `identity_link.interface_id` points at `interface`, so
**story 6.12 must write a second migration widening `identity_link` whatever is chosen here**, and
`identity_link_current_subject`'s CHECK and the `NIL_INTERFACE` sentinel go with it. 6.12 *is* the
resolver writing groupings — it already touches the resolver, which is exactly where (b)'s producer
would live. **Adopting `interface`, widening `identity_link` and adding the producer are one gesture;
splitting them across 6.5 and 6.12 operates on the resolver twice.**

⚠️ **THE COST, ACCEPTED AND NOT DISGUISED: for seven stories the supertype constrains nothing about
interfaces** — which is precisely the objection `deferred-work.md:2222` raised against a premature
supertype. It is accepted because **`device` is born correct**, and because the asymmetry is honest:
a new table is created under the supertype, an older one is ADOPTED when something needs it.
**Registered by name (T8), not glossed.**

**Refused with the reason recorded**: **(b)**, because its producer lands in the resolver's hot path
in a story whose criterion says there is none, and because it does not remove the second migration
that made it look economical; **(c)**, because it breaks the boot unrecoverably on a published
`v0.2.0` and its `kind` has no answer in any document.

### §0d. 🔴 THE `state` COLUMN — the first draft's "contested owner" DISSOLVES, and a harder question replaces it

The first draft framed a two-document conflict: `epics.md:1824` against `deferred-work.md:2226`'s
*"Owner: the lifecycle epic (FR40-42)"*. Three measurements dissolve it:

- **FR38b is Epic 6's** — `epics.md:377` assigns it there — and **its behaviour already has a story:
  6.18, *The ephemeral-interface dormant lifecycle*** (`epics.md:2046`), thirteen stories after this
  one. The first draft never named it.
- **FR40-42 is Epic 21** (`epics.md:501-503`: edit declared attributes, decommission/archive/delete,
  export/import — v0.19). It has nothing to do with dormancy. `deferred-work.md:2226`'s owner is a
  **misattribution, not a competing claim**, and the register row is corrected rather than quoted.
- So the split the draft proposed is not an invention reconciling two documents: **`epics.md` already
  carries both halves.** What remains open is narrow — does the COLUMN come at 6.5 or wait for 6.18.

🔴 **AND THE HARDER QUESTION THE DRAFT NEVER ASKED: WHICH TABLE?** Three documents name three, under
two different domains:

| source | table | domain |
|---|---|---|
| `architecture.md:1200` | **`interface.state`** | `active` / `dormant` |
| `architecture.md:1502` | **`entity.state`** | **six**: `active \| dormant \| superseded \| quarantined \| pending_migration \| sentinel` |
| `architecture.md:1118`, `:1517` | **`declared_attribute.state`** | a different domain again (`asserted`), read by the gap computation |
| `deferred-work.md:2226` | `interface.state` | — |

⚠️ **The epic's two values CONTRADICT `architecture.md:1502`'s six, and AC1 cites that same decision
as its source.** A two-value CHECK refuses a value the architecture names — measured, `ERROR 4025` on
`superseded`. Widening later is an ALTER at boot: at 200 000 rows a CHECK costs **168 ms**, an ENUM
appended 4.2 ms, and **an ENUM inserted mid-list 433 ms** (a full table rebuild, a hundredfold).

🔴 **And `dormant`'s own scope names a column that does not exist.** `architecture.md:1503`: *"`dormant`
is valid only for `kind='interface' AND mac_kind='local'` — a cross-table CHECK is not portable, so
it is an application invariant with an explicit test."* `rg mac_kind` over the planning artefacts and
`crates/`: **three hits, all in `architecture.md`, zero in any migration and zero in any `.rs`.**

### ✅ TAKEN (mine, delegated by Guy 2026-08-28): **`entity.state`, SIX values, `VARCHAR … ascii_bin` + `CHECK`**

- **The table is `entity`.** It is the architecture's own choice (`:1502`), and it is coherent: an
  interface **is** an entity, so its state lives on the supertype. `architecture.md:1200` describes
  that same state seen from the interface — not a second column. `declared_attribute.state`
  (`asserted`) is the same word for another thing, already in place, and does not conflict.
- **Six values, not two.** The epic's two are the subset FR38b needs; the architecture enumerates the
  domain. Shipping two buys an `ALTER` at boot on a published product later — **exactly the hazard
  AC6 has just measured** — for a widening that costs 168 ms at 200 000 rows today. The domain is
  posed once. ⚠️ **This DIVERGES from `epics.md:1826` and is registered** (T8): a story may not edit
  the epic file.
- **The spelling is a measurement, not a taste** — see AC3.
- ⚠️ **Consequence accepted: under option (a), `entity` holds no interface row, so the column is
  INERT until story 6.12.** That is what *"the domain and no behaviour"* means, and it is said rather
  than discovered.
- ⚠️ **`mac_kind` is re-owned to story 6.18**, which carries FR38b's behaviour: the scoping invariant
  belongs with the transition it scopes, not with the column.

**Refused with the reason recorded**: `interface.state`, because it puts an entity's state outside
the entity model this very story creates; the epic's two-value domain, on the ALTER-at-boot cost
above; and `ENUM`, on the four measurements in AC3.

**This story ships the DOMAIN and no behaviour** — and the reader must not conclude FR38b is being
implemented. ⚠️ In particular, **FR38b carries a startup
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

### §0f. 🔴 THREE REGISTER ROWS NAME **THIS STORY** AS OWNER, and the first draft carried none

`CLAUDE.md` is explicit — *"items are REGISTERED for a retrospective by the stories that raise them
— carry them IN rather than rediscovering them"* — and the first draft's T8 registered divergences
**outgoing** while nothing carried the **incoming** ones. Each is accepted or re-owned BY NAME (AC9):

- 🔴 **`deferred-work.md:3478-3482` — D15's sibling rule, held by NEITHER gate.**
  *"`declared_attribute.entity_id` is NEVER updated. Ever. No UPDATE"*, which
  `architecture.md:1064-1069` calls ***"the most dangerous line of SQL in this project, and it looks
  like a routine refactor."*** `authorship` guards the AUTHOR of a declared write and
  `observed-immutable` guards a different table. **Owner: this story — *"where `entity_id` acquires
  meaning."*** ⚠️ On `observed-immutable`'s precedent this is a gate or a written limit, and story
  5.12's rule DOES govern it: the absence is unbounded.
- **`deferred-work.md:3395-3397` — the invisible entity.** `build_view` selects an entity by its
  declared `ipv4`, so a hostname-only subject documents 201 and mints an entity the view can never
  select. *"Not this story's bug (the entity model is 6.5's)."*
- **`deferred-work.md:3426-3431` — one value per `attr_key`**, routed to *"the entity model of 6.5"*.

### §0g — WHAT THE VALIDATION MEASURED, 2026-08-28 (two fresh-context layers, own worktree and own store each)

*The corrections above are applied in place. What follows is what has no other home.*

#### 🔴 A NEW MIGRATION FILE IS INVISIBLE TO `cargo build`, AND THE BOOT LOG SAYS "MIGRATIONS APPLIED"

`crates/opencmdb-bin/build.rs` exists — written at story 6b.10 for exactly this class — and its
entire body is `cargo::rerun-if-changed=locales/app.yml`. **It does not cover `migrations/`.**

```
$ cp 0006_entity_device_and_state.sql crates/opencmdb-bin/migrations/
$ cargo build --workspace --locked
    Finished `dev` profile … in 0.07 s          ← no `Compiling` line
$ grep -ac "CREATE TABLE entity" target/debug/opencmdb
0
```

Boot that binary: `INFO opencmdb: database connected and migrations applied`, and
`_sqlx_migrations` holds **five** rows. Touching any source file rebuilds and the string appears.
🔑 **MODIFYING an existing migration rebuilds** (the proc macro tracks the files it read);
**ADDING one does not** — and this story's central deliverable is an added file. The ordinary
gesture — write it, build, boot, look — produces a binary without it and a log asserting success.
⚠️ The validation layer fell into it itself and caught it only by `grep -a` on the artefact, which
is story 6b.4b's own defence: *grep the artefact you are about to believe, not the source you just
edited.*

#### ⚠️ A DEVICE CANNOT BE A PLACEMENT SUBJECT, and no criterion says so

With the supertype applied, `entity` + `device` rows insert fine and a link on that device does not:
`ERROR 1452 … identity_link_interface_fk`. **Story 6.12 cannot fill this schema without a SECOND
migration widening `identity_link`** — on a table carrying story 5.14's registered accumulation
(~105 000 rows a year on one host at a five-minute interval) — and `identity_link_current_subject`'s
CHECK and the `NIL_INTERFACE` sentinel both name `interface_id` by hand and go with it. **No
criterion mentions `identity_link` or `link_candidate`.**

#### ⚠️ Measurements to spend rather than re-take

- **`interface.id` serves as `entity.id` unchanged** — same `CHAR(36) ascii_bin`; 200 003 rows
  backfilled and the FK validated. No new column.
- **`ALTER TABLE … ADD CONSTRAINT CHECK` is `ALGORITHM=COPY` on MariaDB 10.11, always.** `ADD COLUMN`
  alone accepts `INSTANT`; the same statement with a CHECK appended answers `ERROR 1845`. Splitting
  the two ALTERs makes the column instant and leaves the CHECK a rebuild.
- **The product pins no `sql_mode`** — no `after_connect`, no `SET SESSION` anywhere — and the target
  platform is a Synology-packaged MariaDB whose mode is not ours to assume. A `CHECK` refuses a bad
  value in every mode; an `ENUM` under `sql_mode=''` accepts it and stores the empty string.
- **`sqlx` 0.9.0 decodes a MySQL `ENUM` straight into `String`**, so the decode question does not
  discriminate between the spellings. The collation and the mode do.
- **No existing index or foreign key blocks any of the three ALTERs.**
- **The suite is 0.41 s dry against 6.04 s live** — the clock is the tell, as the draft said.

#### ⚠️ Two process rules carried in from the previous validation, and honoured by this one

**`cargo test` and any other consumer must not share a store**; and **never a broad
`pkill -f 'target/debug/opencmdb'`** — it killed a sibling layer's server yesterday. Kill by port.

---

## Tasks / Subtasks

- [x] **T1 — BOTH arbitrations taken, 2026-08-28** — §0c option (a) (Guy), §0d `entity.state` with
      six values (mine, delegated). Each recorded with the option refused and the cost accepted.
- [ ] **T1b — `cargo::rerun-if-changed=migrations` in `build.rs`, as the FIRST act of T3** — or T2
      greps the artefact instead. §0g's finding: without it the deliverable is invisible to
      `cargo build` and the boot log asserts success anyway.
- [ ] **T2 — Re-measure §0 on the tree you start from** (AC8), against a migrated store, and say
      which store and which port.
- [ ] **T3 — `0006_entity_device_and_state.sql`** (AC1, AC2, AC3, AC4): the supertype, `device`, the
      `state` domain, binary collations throughout.
- [ ] **T4 — Whatever T1 decides about existing rows** (AC6), with the migration measured against a
      store that **already holds** interfaces and declared attributes — not against an empty one —
      and with the **recovery path** measured too, not only the happy one: a failed migration sticks
      at `success = 0` and repairing the data does not clear it.
- [ ] **T5 — The adapter** (AC5), with no production caller, and raw-SQL tests for every CHECK the
      adapter cannot violate (AC5's 5.9 warning).
- [ ] **T6 — AC2's absence: the carrier is BUILT and proved to red** (AC7) — a test over
      `information_schema.COLUMNS` asserting `device`'s columns are exactly `[id, kind]`, with its
      `DATABASE_URL` gating WRITTEN. Add a text gate beside it only if you can say what it covers
      that the test does not.
- [ ] **T6b — D15's rule** (AC9, §0f): `declared_attribute.entity_id` is never UPDATEd, held today by
      no gate. A gate or a written limit — and say which, on story 5.12's narrowing.
- [ ] **T7 — Mutation pass with predictions written FIRST**, through `cargo xtask mutate`. ⚠️ Its
      three carriers are cargo-side and this story is entirely cargo-side, so unlike 6.4c the driver
      covers it — say so, and use it.
- [ ] **T8 — Register every divergence BY NAME** (`deferred-work.md`), diffed before the commit.
      ⚠️ *A section that says "registered" is not a registration* (story 6b.9). **Incoming rows are
      answered too** (§0f), and `deferred-work.md:2226`'s wrong owner and `:2216`'s false
      *"three entries"* self-description are corrected in passing.
- [ ] **T9 — The two gate holes this story exposed are registered, not fixed here**: the
      `ddl-collation` matcher (four of five planted violations green) and the `file-size` gate's
      blindness to `repo.rs`. Both are wider than this story.

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
- **`OPEN_END` is `"9999-12-31 23:59:59.999999"`** — `repo.rs:666`, the MariaDB spelling. ⚠️ The
  architecture writes it `'9999-12-31T23:59:59.999Z'`, an ISO-8601 literal from the two-engine era;
  `repo.rs:657-660` calls the difference *"a transposition, not a contradiction"*, and both committed
  DDL sites use the MariaDB form. **A CHECK written from the architecture's literal never matches.**
  The NULL trap itself holds: MariaDB keeps NULLs distinct, so a NULL inside a uniqueness key makes
  the constraint decorative.
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
- 🔴 **The `file-size` gate is BLIND to `repo.rs`, which is where AC5's adapter goes.** It stops at
  the first `#[cfg(test)]` **at any nesting**, and in `repo.rs` that is line **182** — an attribute on
  a test-only struct — while the trailing test module starts at **1621**. So the gate reads the file
  as **181** code lines while it carries roughly 1620, and the 2000-line ceiling does not protect it.
  Registered, not this story's to fix (T9). `page.rs` at **1978** is correct and is the file the gate
  does see.
- The `ddl-collation` gate lives in `xtask/src/main.rs` and has **no allowlist — the absence is the
  mechanism**. ⚠️ **It is also line-oriented, with no comment stripping and no per-column split**, so
  `_BIN` anywhere on a line satisfies it; keep every column's type and `COLLATE` on **one** line, as
  all five committed migrations do, or the gate reds for the wrong reason and passes for another.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#L1812`] — story 6.5's three criteria.
- [Source: `_bmad-output/planning-artifacts/architecture.md#L1475`] — D21's supertype, the composite
  FK, and the refusal of a polymorphic key.
- [Source: `_bmad-output/planning-artifacts/architecture.md#L1480`] — *"`device` has NO business
  columns"*, with `hostname` named as the specimen refusal. *(`:1483` is the next bullet; the first
  draft's anchor was off by three.)*
- [Source: `_bmad-output/planning-artifacts/architecture.md#L1487`] — the NULL trap; `OPEN_END` at
  `:1492`; *"same reasoning for `NIL_INTERFACE`/`NIL_DEVICE`"* at `:1494`.
- [Source: `_bmad-output/planning-artifacts/prd.md#L952`] — FR38b in full, including the asymmetry
  and the startup refusal at `:957`.
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md#L2216`] — story 5.9's deferrals.
  ⚠️ Its header announces *"three distinct entries, split here because an earlier draft merged the
  first two"* and the bullet below **still merges them**: there are a header and **two** entries. The
  register's own self-description is false and is corrected by T8.
- [Source: `crates/opencmdb-bin/migrations/0002_interface_and_identity_link.sql`] — the DDL idiom to
  follow: the reason for every CHECK written beside it, and the
  `VARCHAR … ascii_bin` + `CHECK (… IN (…))` spelling of an enumerated domain.
- [Source: `crates/opencmdb-bin/migrations/0003_resolver_guards.sql#L13`] — **AC6's real precedent**:
  a foreign key that validates existing rows, the `success = 0` trap, and why repairing the data does
  not recover. Its closing *"production is unaffected: 0002 is unreleased"* is the sentence this
  story cannot write.
- [Source: `_bmad-output/planning-artifacts/epics.md#L2046`] — story **6.18**, which owns FR38b's
  behaviour; and `#L377`, which assigns FR38b to Epic 6.
- [Source: `_bmad-output/planning-artifacts/architecture.md#L1502`] — `entity.state`'s **six** values
  and `dormant`'s scope on a `mac_kind` column that does not exist.

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
| 2026-08-28 | ✅ **BOTH ARBITRATIONS TAKEN — the story is buildable.** **§0c: OPTION (a)** (Guy) — `entity` and `device` created, **`device` wired as a subtype from its first day**, **`interface` stays outside the supertype** and its adoption is re-owned BY NAME to story 6.12. 🔑 **The argument is not (b)'s cost, it is that (b) SAVES NOTHING**: a device cannot be a placement subject, so story 6.12 must widen `identity_link` whatever is chosen here — and 6.12 *is* the resolver, which is exactly where (b)'s producer would live. Adopting `interface`, widening `identity_link` and adding the producer are **one gesture**; splitting them operates on the resolver twice. ⚠️ **Cost accepted and registered, not glossed: for seven stories the supertype constrains nothing about interfaces** — `deferred-work.md:2222`'s own objection — accepted because `device` is born correct and the asymmetry is honest. **§0d: `entity.state`, the architecture's SIX values, `VARCHAR(20) … ascii_bin` + `CHECK (… IN (…))`** (mine, delegated by Guy — recorded as mine so it can be reversed at the right cost). The table is `entity` because an interface **is** an entity; six because shipping two buys an ALTER at boot on a published product for a widening that costs 168 ms today — ⚠️ **a divergence from `epics.md:1826`, registered**; the spelling because an `ENUM` lands `utf8mb4_general_ci`, accepts `'ACTIVE'`, is invisible to the gate and under `sql_mode=''` stores the empty string. ⚠️ **Consequence accepted: the column is INERT until 6.12**, `entity` holding no interface row — which is what *"the domain and no behaviour"* means. ⚠️ **`mac_kind` re-owned to story 6.18**: the scoping invariant belongs with the transition it scopes. |
| 2026-08-28 | **VALIDATED by two fresh-context layers, and the corrections are applied in place.** 🔴 **The story's own §0c recommendation is WITHDRAWN, refuted on the point that fed it**: the validation built all three options as real migrations, applied them to a populated store through `sqlx::migrate!` at boot and pressed the live gesture against each — **option (b) needs a producer TOO, and a heavier one than (c)'s** (65 tests red; a backfill repairs what exists and nothing repairs what the resolver writes on the next scan; the minimal producer takes it 65 → 1 and lives in the resolver's HOT PATH). **Only option (a) satisfies the epic's *"no producer"*.** 🔴 **(c) breaks the BOOT, unrecoverably** — `ERROR 1452`, `success = 0`, and repairing the data does not clear it; `0003`'s *"production is unaffected: 0002 is unreleased"* is the sentence this story cannot write, `v0.2.0` being published. 🔴 **A NEW MIGRATION FILE IS INVISIBLE TO `cargo build`** — `build.rs` tracks `locales/app.yml` alone, so writing `0006`, building, booting and looking yields a binary without it **and a log saying *"migrations applied"***; modifying an existing migration rebuilds, adding one does not. 🔴 **§0b's premise was wrong three ways**: the demo seed carries ONE entity id not three, it is **not in the published image at all**, and **`interface` is EMPTY on every real deployment** (measured on a boot: 1 observation, **0 interfaces**, 1 abstained link) because the shipped connector emits no MAC. 🔴 **§0d's "contested owner" DISSOLVES** — FR38b is Epic 6's and **story 6.18 owns its behaviour**, while `deferred-work.md:2226`'s *"lifecycle epic (FR40-42)"* names **Epic 21** and is a misattribution — and **a harder question replaces it: AC3 names no table**, three documents name three under two domains, the epic's two values contradict `architecture.md:1502`'s **six**, and `mac_kind`, on which `dormant`'s scope depends, **exists in no table and no `.rs`**. 🔴 **AC4's gate does not hold its property**: four of five planted violations were applied and read back `utf8mb4_general_ci`, `_BIN` anywhere on a line satisfying the matcher — narrowed to a TRIPWIRE on story 5.12's precedent. 🔴 **AC1's composite key does NOT carry the disjunction**: under `PRIMARY KEY (id, kind)` one id lives under both kinds at once; the PK on `id` alone refuses it. 🔴 **Three register rows name THIS STORY as owner and the first draft carried none** — D15's *"`entity_id` is NEVER updated"*, which the architecture calls *"the most dangerous line of SQL in this project"* and which **no gate holds**; the invisible entity; the one-value-per-`attr_key` model. New **AC9** answers them. ✅ **AC7's premise was corrected in the story's favour**: `device`'s absence is BOUNDED, so story 5.12's rule does not govern it and a working carrier was built and proved to red on `hostname`. ⚠️ Also corrected: the last migration is story **6.2**'s, not 6.3's; `OPEN_END` is the MariaDB spelling and a CHECK written from the architecture's literal never matches; **the `file-size` gate is BLIND to `repo.rs`** (181 code lines read where ~1620 are), which is where AC5's adapter goes; `identity_link` has two FKs; `SHOW TABLES` returns six. ⚠️ **BOTH ARBITRATIONS REMAIN OPEN AND ARE GUY'S.** |
| 2026-08-28 | Story created and CONTEXTED against a migrated `mariadb:10.11.11`, not against the DDL text. 🔴 **ONE ARBITRATION IS OPEN AND THE STORY CANNOT BE BUILT WITHOUT IT**: the epic says *"no producer: nothing writes a device yet"*, and **the product has been minting entity ids since story 6.2** — `document.rs:124` mints a v7 UUID per documented subject and writes it into `declared_attribute`, which has **no foreign key**, because **there is no `entity` table**. So the supertype meets rows that already exist, in the demo image, in CI, and in any deployment where the amber control has been pressed. Three options are costed in §0c; the recommendation is create the supertype **and** make `interface` its first real subtype with a backfill, leaving `declared_attribute` alone with the divergence registered — ⚠️ **which means the migration WRITES, and *"schema only"* does not describe that.** Option (c), an FK on `declared_attribute`, is refused for now as *a producer wearing a constraint's clothes*, and it raises a question no document answers: **what `kind` is a documented subject?** ⚠️ **The `state` column's owner is contested** — `epics.md` puts it here, `deferred-work.md:2226` assigns it to the lifecycle epic; both are honoured by shipping the DOMAIN and no behaviour, and **FR38b's startup refusal** (`prd.md:957`, *"a startup failure naming both settings"*) must be neither implemented nor silently dropped. ⚠️ Measured: five tables, none of them `entity`; three distinct entity ids in `docker/seed-example.sql` alone; `sqlx::migrate!` runs on **every boot** of a published `v0.2.0`. 🔑 Story 5.9's warning is carried forward: **its M3 came back GREEN** because the adapter could not emit an incoherent pair, so every CHECK the adapter cannot violate is measured by raw SQL or by nothing. ⚠️ NEXT: `create-story validate` (two fresh-context agents) is MANDATORY before `dev-story`. |
