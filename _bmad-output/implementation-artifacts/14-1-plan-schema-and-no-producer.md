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

⚠️ And it follows from Guy's arbitration (4): **IPAM and `declared_attribute` are two registers**.
`declared_attribute.entity_id` points at `entity`; making a range an entity is the first step
towards fusing the two registers the arbitration separates. `interface` sits outside the supertype
for its own reasons (6.5, Guy's option (a)) — this is a second, different reason for a second table.

### (b) An address is stored as `VARBINARY(16)`, not as text and not as `INT UNSIGNED`

**Refused (text)**: `declared_attribute` stores `ipv4` as text and `inventory_view.rs:261` already
carries the registered consequence — *"the address compares as a STRING, so `192.0.2.9` follows
`192.0.2.10`"*. This store's whole job is **ordering and containment**; inheriting that defect into
the table that must answer *is this address inside that range* would be inheriting it where it stops
being cosmetic.

**Refused (`INT UNSIGNED`)**: it is the simplest thing that works **for IPv4 only**, and **FR25
(IPv6, observation-only) is inside Epic 14** — named in its own scope line. Story 6.5's precedent
governs: *the domain is posed once rather than widened by an `ALTER` running at boot on a published
product*. A 4-byte-to-16-byte type change with a data migration is strictly more expensive than the
enum widening 6.5 avoided.

🔑 **And D10 makes the choice cheap**: comparison and normalisation never descend into SQL, so the
arithmetic (*next free*, *first + n*, containment) happens in Rust either way, where a `[u8; 16]` is
no harder than a `u32`. The database only has to ORDER and COMPARE, which `VARBINARY` does bytewise
and correctly for equal-length values.

⚠️ **The caveat is written rather than discovered**: bytewise ordering is correct **within one
address family**. A 4-byte and a 16-byte value sort by length first. Every range lives inside one
subnet and every subnet is one family, so no comparison the product makes crosses families — **and
the day one does, this sentence is what it must be measured against.** This story adds no IPv6 row
and no IPv6 test; it declines to make IPv6 *impossible*, which is not the same as building it.

### (c) The policy enum lives in `crates/opencmdb-core/src/ipam/`, a NEW subdomain module

`architecture.md:3208` already prescribes it — *"one `thiserror` per subdomain IS one per decider
(D47): `identity::IdentityError`, `gap::GapError`, **`ipam::IpamError`**"* — and `:3366` lists
`ipam/` among the modules that *"arrive"* unbuilt. This story is that arrival.

⚠️ **D47 is the frontier**: the enum and its token mapping go in `opencmdb-core` (no `sqlx`, no
`anyhow`); the DDL and the adapter go in `opencmdb-bin`. Exactly the `EntityState` / `repo.rs` split
story 6.5 shipped.

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
- **AC4** Addresses are `VARBINARY(16)` per §1(b), and a test pins that `192.0.2.9` sorts **before**
  `192.0.2.10` in the store — the defect `inventory_view.rs:261` registers for the text
  representation, refused here by measurement rather than by intention.
- **AC5** A range's bounds are refused when `last < first`, and a range whose bounds fall outside its
  subnet is refused. Both by `CHECK` where the DDL can, by the adapter where it cannot, and **each
  measured by raw SQL where the adapter cannot reach it** — story 5.9's **M3** shape: *a guard the
  adapter cannot violate is measured by raw SQL or by nothing*.
- **AC6** `0007` is re-runnable: `IF NOT EXISTS` throughout, and the header carries the recovery
  recipe on `0003`'s idiom. 🔴 Story 6.5's AC6 claim — *"no `success = 0` to recover from"* — was
  **FALSE and displaced a remedy already specified**: MySQL DDL is not transactional.
- **AC7** **No route, no screen, no producer.** Nothing in this codebase writes an `ip_subnet`, an
  `ip_range` or an `ip_address` outside this story's own tests. Precedent: story 6.5, whose criterion
  said so and was met literally.
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
- [ ] **T2** `migrations/0007_addressing_plan.sql` — the three tables, the `CHECK`s, `IF NOT EXISTS`,
      the header's recovery recipe (AC1, AC2, AC5, AC6).
- [ ] **T3** The adapter in `repo.rs`: insert and read for each table (AC1). ⚠️ `repo.rs` is
      **~3800 code lines and the `file-size` gate is BLIND to it** (it stops at the first
      `#[cfg(test)]` at any nesting — registered at story 6.5, not fixed). **If this story's adapter
      would grow it further, put the IPAM adapter in its own module** rather than relying on a gate
      that cannot see the ceiling. *A gate that cannot measure a file is not permission to grow it.*
- [ ] **T4** The set-comparison test (AC3) — prove-to-red by pointing two variants at one token.
- [ ] **T5** The ordering test (AC4) — `192.0.2.9` before `192.0.2.10`, with the text representation
      as the **control** that would fail.
- [ ] **T6** The refusal tests (AC5), through the adapter AND by raw SQL where the adapter cannot
      reach.
- [ ] **T7** Mutation pass, **predictions written FIRST**, driven by `cargo xtask mutate --baseline`
      against a virgin store. ⚠️ Name the TREE each row was measured on (story 6.7's lesson).
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

### Project Structure Notes

- New: `crates/opencmdb-core/src/ipam/mod.rs`, `migrations/0007_addressing_plan.sql`.
- Touched: `crates/opencmdb-core/src/lib.rs` (module declaration), `repo.rs` **or** a new adapter
  module (T3).
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
