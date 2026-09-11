# Story 14.1: The plan has a schema and no producer

Status: ready-for-dev

Epic 14 (IPAM), decomposed 2026-09-11 in `epics.md` (`4b5db27`). Opens the epic.
Baseline: `4b5db27` (master).

## Story

As the operator,
I want the addressing plan to have a place to live before anything writes to it,
so that the shape of a range is settled while nothing depends on it.

## §0 — What was established before this story, and must not be re-opened

**The vocabulary is RATIFIED** (PR #166, `b466be7`). The PLAN axis is binding: `static` ·
`dhcp-pool` · `reserved` · `infrastructure`. **No story may extend it.** `undeclared` was widened in
the same act, and `structural` is retired as a displayed word with **story 14.2** carrying the
removal — not this one.

**Guy's six arbitrations** (2026-09-10) are in `epics.md`'s Epic 14 preamble as six numbered
constraints. This story is downstream of all six and re-decides none.

## §1 — Three decisions this story TAKES, each with the alternative refused

These are the three the epic left open, and settling them here is why the story exists. A dev agent
that has to choose one of these mid-implementation will choose it silently.

### (a) The three tables live OUTSIDE the `entity` supertype

**Refused**: adding `Subnet`/`Range` to `EntityKind` and hanging them off `entity(id, kind)`.

🔑 The supertype exists for **D21's interface/device disjunction** — things the identity engine
FORMS from observations, carrying a lifecycle `state`. A range is **declared by the operator and
formed by nothing**: it has no sighting, no identity, no `state` in that sense. Widening
`EntityKind` would be a domain change buying no constraint.

⚠️ And it follows from Guy's arbitration (4): **IPAM and `declared_attribute` are two registers**,
and putting a range under the supertype is the first step towards fusing them. `interface` sits
outside the supertype for its own reasons (6.5, Guy's option (a)) — this is a second, different
reason for a second table.

🔴 **A second argument stood here and was REFUTED by the validation**: *"`declared_attribute.entity_id`
points at `entity`"*. It does not. Measured on the migrated schema — **zero foreign keys** on
`declared_attribute`; the only table referencing `entity` is `device`. Story 6.5 records it in words
(*"the documented subject is still in no entity model"*). The decision stands on its first argument
and on arbitration (4); the refuted one is struck rather than quietly deleted.

### (b) An address is stored as a ZERO-PADDED dotted quad, `CHAR(15)` `ascii_bin`

`192.0.2.9` is stored `192.000.002.009`. **Guy, 2026-09-11**, after the validation refuted the first
answer's argument. The whole of this sub-section is a replacement, and the refuted reasoning is kept
below rather than erased.

🔑 **Lexicographic order IS numeric order**, which is the property the store exists for — and it is
**D10's own precedent applied to addresses**: *"timestamps stored as ISO-8601 UTC `TEXT` so
lexicographic order == chronological order"* (`architecture.md:564`). The registered defect at
`inventory_view.rs:261` — *"the address compares as a STRING, so `192.0.2.9` follows
`192.0.2.10`"* — is caused by the ABSENCE of padding, not by text.

🔑 **And D48's live reason argues for it.** D48 refuses `BINARY(16)` for opaque ids and its second
reason is *"the 3 a.m. dump … with `BINARY(16)` you write a `HEX()` per query, get the endianness
wrong one time in three, and **the grep does not work — the connective tissue between your tools is
cut, at the exact hour you are worst**"*. D48 is scoped to identifiers and does not bind here; but
for a product whose subject **is** addresses, `WHERE addr = UNHEX('C0000209')` cuts exactly that
tissue. ⚠️ The first draft of this section refused two alternatives and never mentioned the register
entry that most nearly governs the choice.

🔑 **Fixed width makes the family a same-row CHECK.** `LENGTH(addr)` is 15 for IPv4 and, when FR25
arrives, 39 for an expanded IPv6 form. So *"both bounds are the same family"* is
`CHECK (LENGTH(first_addr) = LENGTH(last_addr))` — expressible, which is exactly what AC5 needed and
could not get from a variable-width binary column.

**Refused: `VARBINARY(16)`** — the first draft's answer. It works, and the validation measured four
costs the draft did not see:

- **The families INTERLEAVE.** The draft's own caveat said *"a 4-byte and a 16-byte value sort by
  length first"* and nominated that sentence as what a future story must measure against. Measured:
  `VARBINARY` compares bytewise on the common prefix, so `0.0.0.1` sorts before `2001:db8::1` and
  `255.255.255.255` after it. **The stated mechanism is the opposite of the real one.**
- **An IPv6 address was returned by a containment query over IPv4 bounds**, sorted between
  `192.0.2.9` and `192.0.2.10`. Built and measured, not argued.
- **One address, two rows.** `INET6_ATON` gives 4 bytes for `192.0.2.7` and 16 for
  `::ffff:192.0.2.7`; both inserted, and the `UNIQUE (subnet_id, addr)` refused neither. The column
  type was chosen and the **encoding** never was.
- **`[u8; 16]` does not exist for sqlx MySQL** — `E0277` on both `Type` and `Decode`, measured on the
  pinned `=0.9.0`. The draft named it as the cheap Rust type.

**Refused: `INT UNSIGNED`** — unchanged, and for the reason the draft gave: **FR25 (IPv6) is inside
Epic 14**, and story 6.5's precedent poses a domain once rather than `ALTER`ing at boot on a
published product.

⚠️ **The costs of the chosen form, written rather than discovered**: 15 bytes where 4 would do;
one canonical form that a `CHECK` must IMPOSE (an unpadded `192.0.2.9` must be refused, or the store
holds two spellings of one address — the same defect as `VARBINARY`'s, reachable by a different
road); and the arithmetic still happens in Rust (D10), parsing and re-rendering the padded form.

⚠️ **And 14.3's seam is foreseeable and named here**: the audit joins plan rows to
`declared_attribute.ipv4`, which stores the **unpadded** dotted quad. That join converts at the
boundary whatever this story chooses. It is 14.3's to build; it is 14.1's to have said.

### (c) The policy enum lives in `crates/opencmdb-core/src/ipam/`, a NEW subdomain module

`architecture.md:3208` already prescribes it — *"one `thiserror` per subdomain IS one per decider
(D47): `identity::IdentityError`, `gap::GapError`, **`ipam::IpamError`**"* — and `:3366` lists
`ipam/` among the modules that *"arrive"* unbuilt. This story is that arrival.

⚠️ **D47 is the frontier**: the enum and its token mapping go in `opencmdb-core` (no `sqlx`, no
`anyhow`); the DDL and the adapter go in `opencmdb-bin`. Exactly the `EntityState` / `repo.rs` split
story 6.5 shipped.

## §2 — Seven things the first draft left a dev agent to settle SILENTLY

§1 opens with *"a dev agent that has to choose one of these mid-implementation will choose it
silently"* — and then settled three while leaving these. The validation BUILT the schema and
measured every one of them OPEN. Each is decided here.

| question | measured in the built schema | decided |
|---|---|---|
| two OVERLAPPING ranges in one subnet | **accepted**, 3 of them; no `CHECK` can express it (`ERROR 1901` on the self-referencing subquery) | **the ADAPTER refuses it**, by name. Same instrument as AC5's third refusal, and the same control |
| duplicate CIDRs | **accepted** — no unique key | **`UNIQUE (base, prefix_len)`** on `ip_subnet`. A plan with one subnet twice is not a plan |
| a subnet base that is not the network address (`192.0.2.5/24`) | **accepted** | **refused by the adapter** — the base must be the network address for containment to mean anything |
| an `ip_address` outside its subnet (`8.8.8.8` in `192.0.2.0/24`) | **accepted** — AC5 named the rule for `ip_range` and never for `ip_address` | **refused, same rule, both tables** |
| `prefix_len` against the address family | **`192.0.2.0/24` accepted with `prefix_len = 64`**, and the natural Rust containment then computes `32 - 64` and **PANICS on subtract-with-overflow** (the validation wrote the test; it panics) | **the adapter binds `prefix_len` to the family**, beside the containment check. ⚠️ The panic is the finding: a DDL that admits an impossible pair hands the arithmetic an impossible input |
| deleting a subnet that still holds ranges | **refused, `ERROR 1451`** — a plain FK, the good default | **kept**, and named: 14.4 asks for *"refused by name"*, and a raw `1451` is not a named refusal. 14.4's, not this story's |
| `'static '` in `policy` | **accepted and stored with its trailing space** — `ascii_bin` is PAD SPACE, so the `IN` comparison pads and the `CHECK` passes | **refused by the DDL.** ⚠️ AC3's set comparison reads the `CHECK` CLAUSE and never a stored row, so it is **structurally blind** to this — *a guard placed where the defect cannot occur* |

⚠️ **`ip_address`'s columns are not specified anywhere in this story**, and *"which addresses are
individually defined"* is the whole spec a dev agent gets. Decide them in T2 and list them in the
File List: at minimum the subnet it belongs to, the address, and a label.

## Acceptance Criteria

- **AC1** `0007` creates `ip_subnet`, `ip_range` and `ip_address`. `ip_range` carries its policy.
  Ids are `CHAR(36)` `ascii_bin`, minted client-side (D48); every column holding letters carries a
  binary collation (D64).
- **AC2** The policy column is `VARCHAR` + `CHECK`, **never a MariaDB `ENUM`**. Story 6.5 measured
  that an `ENUM` lands `utf8mb4_general_ci`, accepts `'ACTIVE'`, is **invisible to the
  `ddl-collation` gate**, and under `sql_mode=''` stores the **empty string**.
- **AC3** `IpPolicy` exists in `crates/opencmdb-core/src/ipam/`, with `ALL` and a token mapping, and
  a test compares the enum's tokens against the schema's `CHECK` **as SETS**. 🔴 Story 6.5's **M8**:
  *a count is not a set* — comparing counts left two variants pointing at one token green, with one
  variant unreachable and one schema value nothing could read back.
- **AC3b** 🔴 **And the OTHER direction, which the set comparison does not close.** `IpPolicy` becomes
  a row in `screens.rs::every_variant_of_a_navigated_enum_is_listed_in_all`. The validation measured
  the hole by adding a fifth variant absent from `ALL` with a token the `CHECK` refuses: **866 tests,
  ten gates and clippy all GREEN**, while anything writing it takes `ERROR 4025` in production.
  ⚠️ **The same mutation is green TODAY on `EntityState`** — the very precedent AC3 cites — because
  neither it nor `EntityKind` is a row in that table. **That is registered, not fixed here**: this
  story adds its own row and names the two that are missing.
- **AC4** Addresses are `CHAR(15)` `ascii_bin` holding the ZERO-PADDED dotted quad per §1(b). A test
  pins that `192.0.2.9` sorts **before** `192.0.2.10` in the store, with the UNPADDED form as the
  **control that fails** — the defect `inventory_view.rs:261` registers, refused here by measurement
  rather than by intention.
- **AC4b** 🔴 **One address has ONE spelling, and a `CHECK` imposes it.** An unpadded `192.0.2.9`, a
  short octet, a value with a letter in it — each refused by the DDL. The validation measured the
  same defect on the refused alternative from the other side (`192.0.2.7` and `::ffff:192.0.2.7`
  both inserted, the `UNIQUE` refusing neither); **a canonical form that is not imposed is not a
  canonical form**, whatever the column type.
- **AC5** Three refusals, **and they need THREE instruments, not one**:
  - `last < first` → a same-row `CHECK`. Measured by **raw SQL** (5.9's M3: *a guard the adapter
    cannot violate is measured by raw SQL or by nothing*). Verified expressible: `ERROR 4025`.
  - `LENGTH(first_addr) = LENGTH(last_addr)` → a same-row `CHECK`, the family guard §1(b) buys.
    ⚠️ Without it the validation built a range running **from an IPv4 address to an IPv6 one** that
    passed every constraint the first draft prescribed.
  - **a range outside its subnet** → the ADAPTER, because a `CHECK` referencing another table is
    `ERROR 1901` (measured). 🔴 **And raw SQL does NOT measure this one — it BYPASSES it**: the
    validation ran the write raw and it succeeded, which proves nothing about the guard. It is
    measured **through the adapter**, with a raw insert as the **control showing the DDL does not
    refuse it**. *The first draft applied 5.9's M3 to the half it does not govern, and a dev agent
    following it literally writes one useful test and one vacuous one.*
- **AC6** `0007` is re-runnable: `IF NOT EXISTS` throughout, and the header carries the recovery
  recipe on `0003`'s idiom. 🔴 Story 6.5's AC6 claim — *"no `success = 0` to recover from"* — was
  **FALSE and displaced a remedy already specified**: MySQL DDL is not transactional.
- **AC7** **No route, no screen, no producer.** Nothing in this codebase writes an `ip_subnet`, an
  `ip_range` or an `ip_address` outside this story's own tests. Precedent: story 6.5, whose criterion
  said so and was met literally.
- **AC7b** 🔴 **And that makes the adapter DEAD CODE, which fails CI.** Measured: five
  `error: … is never used` under `RUSTFLAGS="-D warnings"`. Story 6.5 escaped it only because
  `repo.rs` carries a module-level `#![allow(dead_code)]`; a new module carries none. **The new
  module takes the attribute, and the reason is WRITTEN rather than assumed** — ⚠️ because
  `arp_ping.rs:22` records, from 2026-09-10, that the ABSENCE of that same attribute is load-bearing
  there (severing the MAC read left clippy green while it stood). *Both conclusions are right and
  they are opposite: an allow is a trade between "the compiler cannot see a producer that does not
  exist yet" and "the compiler is the only thing watching this wiring." Say which one you are in.*
  The attribute is **removed by the story that gives these functions a producer** — 14.2.
- **AC8** Every store-backed test takes `crate::DB_TEST_LOCK` **first**. 🔴 Story 6.5's review found
  seven new tests without it and measured the cost: **6/6 RED on a virgin store where the parent
  commit was 6/6 green** — the shape CI's own `Tests` step runs. *The per-test ids were a remedy for
  a symptom of a rule that was simply not followed.*
- **AC9** THE LIVE COUNT lives here: **859 → N tests** (bin + core + xtask, the sum re-added rather
  than recalled), instrument named — `cargo test --workspace --locked`, wall clock, warm, against a
  **VIRGIN** `mariadb:10.11.11` whose port is stated. ⚠️ The bin suite is non-deterministic against a
  REUSED database (Epic 6's register row); drop it first.
- **AC10** No regression: ten `cargo xtask ci` gates, `clippy --all-targets`,
  `RUSTFLAGS="-D warnings"`, fmt, `cargo deny`. ⚠️ **Both browser gates are NOT claimed** — this
  story renders nothing — and saying so is the criterion, not an omission.

## Tasks / Subtasks

- [ ] **T1** `crates/opencmdb-core/src/ipam/mod.rs` — `IpPolicy`, `ALL`, token mapping, `IpamError`
      (AC3, §1c). ⚠️ D47: no `sqlx`, no `anyhow`, no `axum`, no `askama`.
- [ ] **T2** `crates/opencmdb-bin/migrations/0007_addressing_plan.sql` — the three tables, the
      `CHECK`s, `IF NOT EXISTS`, the header's recovery recipe (AC1, AC2, AC4b, AC5, AC6).
      ⚠️ **There is no `migrations/` at the repository root**; the first draft wrote the path two
      ways and `sqlx::migrate!("./migrations")` is relative to the bin crate.
- [ ] **T3** The adapter in a NEW `crates/opencmdb-bin/src/ipam_repo.rs` — **unconditionally**, not
      "if it would grow `repo.rs`" (AC1, AC5, AC7b).
      🔴 **The first draft justified this with a false number and the instruction is now given
      without one.** It said `repo.rs` is *"~3800 code lines"*; measured three ways, the gate reads
      **183**, the real production half is **1743**, and **4025** is the whole file including tests.
      `~3800` is the TOTAL reported as a CODE count — `repo.rs` is at **87 % of the ceiling, not
      190 %**. ⚠️ And the provenance is the finding: story 6.5 records ~1620/~1700 **correctly,
      twice**, while `CLAUDE.md` carries `~3800`, and this story inherited `CLAUDE.md` rather than
      the source it cites. *`CLAUDE.md`'s figure is itself a defect and is registered.*
      The gate genuinely cannot see the file (it stops at the first `#[cfg(test)]`, a test-only
      helper at `repo.rs:184`) — ⚠️ and the gate's own doc claims it *"would over-count itself,
      never under-count"*, which `repo.rs` falsifies by 1560 lines. *A gate that cannot measure a
      file is not permission to grow it*, and a separate module costs nothing.
- [ ] **T4** The set-comparison test (AC3) — prove-to-red by pointing two variants at one token.
- [ ] **T5** The ordering test (AC4) — `192.0.2.9` before `192.0.2.10`, with the text representation
      as the **control** that would fail.
- [ ] **T6** The refusal tests (AC5), through the adapter AND by raw SQL where the adapter cannot
      reach.
- [ ] **T7** Mutation pass, **predictions written FIRST**, against a virgin store. ⚠️ Name the TREE
      each row was measured on (story 6.7's lesson).
      🔴 **`cargo xtask mutate` CANNOT drive a DDL mutation, and the register names THIS story**:
      *"a changed migration cannot re-apply to a store that already ran it — `sqlx` checksums it —
      so every DDL mutation needs a virgin schema AND a warm-up that migrates once before the
      concurrent suite starts. **Owner: whichever story next writes a migration**"*
      (`deferred-work.md:5132`). **Guy, 2026-09-11: a purpose-built script, and the debt STAYS
      registered** — repairing the driver means teaching it to drop and re-migrate a database, which
      is a subject of its own and is what made 6.4b a whole story. ⚠️ 14.1 is the **third** story to
      route around it; the register row must say so, because *a debt three stories route around is
      not a debt, it is a practice*, and the row is what keeps that visible.
- [ ] **T8** AC9's count, re-added rather than recalled; AC10's sweep.

## Dev Notes

### What the store must be able to answer, and what it must not

**Must**: is this address inside a defined range · what is that range's policy · which addresses are
individually defined · what is the subnet's extent. **Must not, in this story**: anything about
observations. The audit is 14.3's and it joins the two sides; **this story does not read
`observation_record` at all**, and a join written here would be the producer AC7 forbids.

### Traps this project has already paid for, and which apply here

- 🔴 **A DDL mutation that leaves invalid SQL measures the parser, not the guard** (story 6.5) — it
  once reddened 101 tests over a guard never exercised. Mutate a DDL guard into *valid* SQL that
  omits the constraint.
- 🔴 **`ascii_bin` is PAD SPACE**, so a raw `'static '` satisfies every CHECK and reads back
  unfamiliar (story 6.5, registered). Decide whether this story refuses it or registers it — **and
  say which**.
- ⚠️ **`cargo test` on a VIRGIN database races itself on concurrent `migrate!` calls** (story 6.5).
  `DB_TEST_LOCK` is the remedy and AC8 is the criterion.
- ⚠️ **The mutation driver refuses an anchor that matches twice.** A comment quoting the line you
  are mutating is a second match — met twice on 2026-09-10. It is right to refuse; widen the anchor.
- ⚠️ **A `/*` inside a double-quoted string blinded a whole file to three gates** (story 6.5's
  review, fixed). Any new SQL text this story adds is walked by `sql_text.rs`.

### What the validation measured about the instruments themselves

- 🔴 **A store that ran the WITHDRAWN `0007` is poisoned and says nothing useful.** The MAC connector
  story wrote a `0007` UNIQUE index, ran it, and withdrew it (issue #161). No artefact remains in the
  repository — four checks, including `git rev-list --all --objects`. But a developer's own store
  that applied it meets `VersionMissing(7)` or, once this story's `0007` exists,
  `VersionMismatch(7)`, **and the error names no remedy**. AC9's *"drop it first"* cures it by
  accident. Drop it on purpose.
- ⚠️ **`ddl-collation` is a TRIPWIRE and a green one proves little.** Its matcher is
  `up.contains("_BIN") || up.contains("COLLATE BINARY")` — **anywhere on the line**. Measured on this
  story's own shape: `label VARCHAR(64) NOT NULL, -- indexed by ip_subnet_label_bin` passes GREEN;
  the same line without the comment reds. Story 6.5 measured four of five planted violations passing.
  Do not read a green `ddl-collation` as *the collation is right*.
- ⚠️ **A DDL mutation that leaves INVALID SQL measures the parser, not the guard** (6.5) — it once
  reddened 101 tests over a guard never exercised. Mutate into *valid* SQL that omits the constraint.
- ⚠️ **The mutation driver refuses an anchor that matches twice.** A comment quoting the line you are
  mutating is a second match — met twice on 2026-09-10. It is right to refuse; widen the anchor.
- ✅ **Refuted by the validation, so nobody re-chases them**: `0007` IS the next free number (four
  checks); `build.rs` watches the `migrations` DIRECTORY, so a new file is not invisible to
  `cargo build`; the new `ipam/` module trips neither the D47 frontier gate nor `missing_docs`
  (`opencmdb-core` does not deny it yet — the house rule still applies); the SQL gates DO walk a new
  migration (a planted `UPDATE observation_record` reds `observed-immutable` naming
  `0007…sql:40`); and the 859 baseline is exact on a virgin store.

### Two questions this story does NOT answer, and says so

- 🔑 **Containment runs in RUST, not in SQL** (D10: *"all value comparison and normalization happens
  in Rust"*). The Dev Notes' *"Must: is this address inside a defined range"* describes what the
  STORE holds, not a query this story writes. ⚠️ `architecture.md:4847` (F57) asks that SQL-side
  comparison be **costed before any epic reintroduces it** — so if 14.3 wants
  `WHERE ? BETWEEN first_addr AND last_addr`, that is the story that pays F57, not this one.
- ⚠️ **`ipam/` is OUTSIDE the `float-free` gate**, which walks
  `crates/opencmdb-core/src/identity/` alone. That is correct today and stated rather than
  discovered: 14.1 ships an enum, a DDL and an adapter — **no arithmetic at all** — and the
  occupancy figure is three COUNTS, not a ratio (`example_data.rs:829`: *"why the occupancy is a
  LIST and never a formula"*). **14.3 is where a ratio could first appear**, and it is that story's
  to decide whether the perimeter widens. D13's ban is scoped to identity DECISIONS; extending it
  by reflex would apply a decision outside its stated domain.
- ⚠️ **`IpamError` is created by T1 and has nowhere to go.** The adapter returns
  `RepositoryError` (`Contention` · `Constraint` · `NotFound` · `InstantRegressed` ·
  `ContradictoryObservation` · `Backend(String)`). The validation's prototype had to write
  `.map_err(|e| RepositoryError::Backend(e.to_string()))` — which is precisely *"an error there is
  domain data, not a string"* that D47 forbids. `InstantRegressed` and `ContradictoryObservation`
  are the precedent for a NAMED variant. **T1 and T3 must join: decide which side gains it, and say
  which in the File List.**

### Project Structure Notes

- New: `crates/opencmdb-core/src/ipam/mod.rs`, `crates/opencmdb-bin/src/ipam_repo.rs`,
  `crates/opencmdb-bin/migrations/0007_addressing_plan.sql`.
- Touched: `crates/opencmdb-core/src/lib.rs` and `crates/opencmdb-bin/src/main.rs` (module
  declarations), `screens.rs` (AC3b's row). **`repo.rs` is NOT touched** (T3).
- **Not touched**: `page.rs`, `templates/`, `locales/app.yml`, `a11y/`, `epics.md`, the UX spec.
  A story may not edit the last two.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Epic 14` — the six constraints]
- [Source: `_bmad-output/planning-artifacts/prd.md#The PLAN axis` — the binding vocabulary, PR #166]
- [Source: `_bmad-output/planning-artifacts/architecture.md:3208,3366` — `ipam::IpamError`, `ipam/`]
- [Source: `crates/opencmdb-bin/migrations/0006_entity_device_and_state.sql` — the DDL precedent]
- [Source: `crates/opencmdb-bin/src/inventory_view.rs:261` — the registered string-ordering defect]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
