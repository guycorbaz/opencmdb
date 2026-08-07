# Story 5.12: No code path writes a declared field without a human author

Status: review

<!-- ✅ VALIDATED 2026-08-07 by two fresh-context agents (fact-check + gap-hunt).
     **The gap-hunt BUILT THE WHOLE STORY** on a scratch worktree against its own
     `mariadb:10.11.11` on 13312: the seventh gate (~210 code lines), its tests, AC2's raw-SQL tests
     and all six prescribed mutations — reaching **453 tests, seven gates green**, both clippy forms
     clean. So the story IS buildable; what follows is what broke on the way.

     🔴 THREE THINGS THE FIRST DRAFT GOT WRONG, EACH MEASURED:
       • **§1's mechanism 4 was FALSE.** Adding `origin` to the SELECT compiles cleanly and leaves
         ALL 450 TESTS GREEN — sqlx decodes tuples POSITIONALLY and discards extra columns. The
         compile error fires only if the TUPLE is widened too. So AC3's carrier is the GATE ALONE,
         and the first draft predicted a compiler carrier. **Found independently by BOTH layers.**
       • **AC1 and AC2 contradicted each other.** AC2 needs a raw INSERT in a test; AC1's gate reds
         on any INSERT outside the adapter — the gate reddened on the story's own test. **Found
         independently by BOTH layers.** Resolved by a two-site allowlist rather than the
         `cfg(test)` exemption the gap-hunt tried, which was measured to hide a planted test write.
       • **The write-path census said "whole workspace" and meant "under `crates/`".**
         `docker/seed-example.sql:28` is a second, SHIPPED writer.

     🔴 THREE HOLES THE FIRST DRAFT'S SCOPE LEFT, all measured green when they should red: a planted
     `.sql` MIGRATION (`UPDATE declared_attribute SET actor_id = 'engine'`), backticked and
     schema-qualified table names, and `INSERT` without `INTO` (MariaDB accepts it). Plus: the DDL
     has **THREE** provenance columns and AC3 named two — `origin_obs_id` is the adopted observation
     — and `SELECT *` defeats every column-name rule.

     🔑 THE READ HALF HAD NO EVASION TABLE AT ALL, and the naive matcher produced FALSE POSITIVES on
     the clean tree — `float-free`'s "wrong in both directions" lesson, reproduced exactly.

     🔑 A FIFTH MECHANISM WAS MISSING, AND IT IS THE STRONGEST: the PK `(entity_id, attr_key)` with
     no `ON DUPLICATE KEY UPDATE` anywhere means the adapter cannot overwrite a declared value AT
     ALL (`ERROR 1062`). For a story called *never overwrite*, it belonged first.

     ⚠️ THE TWO LAYERS DISAGREED on `line_has_float`'s line number and **I measured it: 915** (the
     fact-check was right). A subagent finding is a lead, not a fact.

     ⚠️ Also: AC2's prescribed red was PANIC-carried, M6 is entirely silent without `DATABASE_URL`,
     `0001`'s unmodifiability is CONDITIONAL (fresh test DBs migrate fine; the NAS is what breaks),
     the `0002:83` trap belongs to story 5.11 not 5.9, `:1737` is a TEST banner, and §7's port
     collided with story 5.11b's still-running container.
     🔑 AC5 is answered in advance: 1136 → **1346** code lines, 654 to spare. Not at risk. -->

## Story

As the operator,
I want the never-overwrite invariant covered by explicit anti-regression tests,
So that the product's central promise — *linked, never merged* — cannot erode by accident (NFR5).

**🔴 THIS STORY IS THE SAME SHAPE AS 5.11b, AND 5.11b IS WHY IT MUST BE WRITTEN CAREFULLY.** The
invariant it covers is **already true by construction** — measured, FIVE separate mechanisms hold
it and not one of them is a test (one of the five I first listed was itself false; §1). Nothing here is expected to red. Story 5.11b was
the same, and its mutation pass found **four behavioural defects and refuted three claims of the
implementer's own**, the sharpest being a guard that could be deleted with the entire suite green.
§3 is written against that. _(This sentence said "eight claims, five of them the implementer's"
until validation: the eight was 5.11b's count of suspicions its reviewer **refuted** — the opposite
sense — and the conflation had already reached two files.)_

**Its central difficulty is different from 5.11b's, and harder.** 5.11b asserted a property *of a
computation*, which a permutation sweep can measure. This story asserts the **ABSENCE of a code
path** — *"no code path writes a declared field without a human author"*. **You cannot measure the
absence of code by running code.** A test suite exercises what exists; it is silent about what
someone adds tomorrow. That is what §4 is about, and it is the story's real design decision.

**What this story does NOT do:**

- it does **not** add the `entity` table, the triage inbox, or the `document` gesture (FR13's
  action). Those are Epic 6 and the triage epic. This story guards a promise about a table that
  already exists, written by one adapter that already exists;
- it does **not** wire the resolver into `main.rs` (still 5.14's);
- it does **not** change the identity engine. `identity/` untouched — **a change there is a FINDING**;
- it does **not** touch `fixtures/`. The trap corpus stays **11 unanswerable, `passed() == false`**.
  **If it turns green, that is a FINDING.**

---

## What this story inherits, measured rather than assumed

### 1. 🔑 The invariant is ALREADY held, by FIVE mechanisms, none of them a test

Measured at contexting and re-measured at validation, on `master` at `5634a1d`:

| # | mechanism | where | what it holds |
|---|---|---|---|
| 1 | the write path binds **SQL literals**, not parameters — `origin` is `'manual'`, `actor_id` is `'operator'` | `repo.rs:123-126` | a caller *cannot* supply a non-human author: there is no parameter to pass one through |
| 2 | a **DDL CHECK**, `declared_actor_not_scanner` | `0001_initial.sql:20` | the database refuses `actor_id = 'scanner'` |
| 3 | the read's **column list** omits `origin` and `actor_id` | `repo.rs:187-189` | the divergence computation cannot consult provenance — it never loads it |
| 4 | ~~the **tuple type**~~ | `repo.rs:183` | 🔴 **FALSE — measured at validation by BOTH layers.** Adding `origin` to the SELECT **compiles cleanly and all 450 tests stay green**: sqlx decodes tuples POSITIONALLY (`sqlx-core/src/from_row.rs`) and silently discards extra columns. The compile error fires only if the TUPLE is widened too. Mechanism 4 protects the destructuring, never the SELECT — **which is exactly why AC3 needs the gate** |
| 5 | the **primary key** `(entity_id, attr_key)` with **no `ON DUPLICATE KEY UPDATE`** anywhere | `0001:18`, and `grep` gives 0 in `repo.rs` | 🔑 **The strongest of the five, and it was missing from this table.** The sanctioned adapter cannot overwrite an existing declared value AT ALL — a second write is `ERROR 1062`, measured. For a story called *never overwrite*, that belongs first |

**There is exactly one write path to `declared_attribute` UNDER `crates/`** —
`repo::insert_declared_attribute`. Verified: one `INSERT`, zero `UPDATE`, and two `DELETE`s both
inside `#[cfg(test)]` (`main.rs:413`, `repo.rs:1110`).

🔴 **But NOT in the whole workspace, and the difference is this story's problem.** Validation found a
**second writer**: `docker/seed-example.sql:28` inserts a declared row (`:23` and `:48` delete), and
it is a SHIPPED file the operator is told to run. A gate that walks only `crates/` cannot see the
only other place a declared field is written. §4 takes this as a requirement, not a footnote.

### 2. 🔴 A weakness in mechanism 2, found at contexting and NOT to be papered over

`CHECK (actor_id <> 'scanner')` bans **one string literal**, not a property. A future writer using
`actor_id = 'engine'`, `'resolver'`, `'system'` or `''` passes it. The constraint's *name* —
`declared_actor_not_scanner` — is honest about this; its **doc comment is not**: `0001_initial.sql:16`
says `-- a human; never 'scanner'`, which reads as a guarantee about authorship and delivers a
guarantee about one spelling.

**This is this story's decision to take, not to inherit.** Two readings, and the implementer must
pick ONE and say which:

- **(a) the CHECK is a tripwire, not a gate** — it catches the one name a careless writer would
  reach for, and the real guard is the `xtask` gate of §4. Keep it, correct its comment, and say so.
- **(b) the CHECK should name the property** — an allowlist (`actor_id IN (...)`) or a join to a
  future `actor` table. ⚠️ **An allowlist in DDL is a migration every time an actor is added**, and
  there is no `actor` table and no `entity` table either (both Epic 6). Choosing this means
  designing a table this story has no other reason to touch.

**Prescribed: (a)**, with the comment corrected in the ADAPTER's doc rather than in the migration.
sqlx checksums the migration file with SHA-384 over its whole text, comments included
(`sqlx-core/src/migrate/migration.rs`), so editing `0001` breaks any database that has **already
applied** it. ⚠️ **That is conditional, not absolute, and validation corrected this**: every test
database here is created fresh, so an edited `0001` migrates fine locally AND in CI — what breaks is
the NAS and any long-lived dev database. The trap is story **5.11**'s record for `0002:83`, not
5.9's. Register the residue.

⚠️ Validation also measured that `'scanner '` (trailing space) is refused too — `actor_id` is
`CHAR(36)`, so MariaDB pads and compares padded. The CHECK bans one padded VALUE, not one byte
string. That widens it very slightly and does not change reading (a).

### 3. 🔴 The failure mode: a test that cannot fail — with 5.11b's evidence, not 5.11b's warning

Story 5.11b ran a mutation pass over a property true by construction. What it measured:

- a guard asserting *"the refusal happens before anything is written"* was **deletable with all 446
  tests green**, because every test ran inside `transact`, which rolls back on `Err` — the assertion
  held wherever the guard sat;
- a doc comment asserted what a test **eighty lines below it in the same commit** refuted;
- **four of its six** originally prescribed mutations left the suite entirely green on first
  measurement (the committed record; the eight-row table in its file is the POST-validation set).

**Every AC below therefore names the mutation that must red it.** A mutation leaving the suite green
is a HIGH finding here, not a reassurance. And per Guy's arbitration at 5.11b: **a mutation MAY edit
anything, including `identity/` and the migrations — the ban in AC6 is on the SHIPPED DIFF.**

⚠️ **The 5.11b driver lesson**: the mutation runner's red count includes cargo's own
`result: FAILED` line. **Subtract one**, or report the number the test harness prints.

### 4. 🔑 The design decision: you cannot test the absence of a code path, so the guard is a GATE

AC1 says *"no code path writes a `declared_attribute` with a non-human author, **and the test reds if
one is introduced**"*. A `#[cfg(test)]` test cannot do the second half: it exercises the paths that
exist. The path that violates the invariant is the one a future story **adds**.

This project already has the idiom, twice: **D56/D65 put gates in `cargo xtask ci`, in Rust, never in
YAML** — `float-free` (story 5.4b) reds on a float under `identity/`, `retired-vocabulary` reds on a
denylisted word. Both are exactly this shape: a property about source text that no runtime test can
hold.

**Prescribed: a SEVENTH gate, `declared-authorship`.** Validation BUILT it (~210 code lines) and
took the workspace to 453 tests with seven gates green, so the shape is known to work. What follows
is what it measured on the way — every row below was executed, not reasoned.

#### 4a. 🔴 The contradiction the first draft shipped, and its resolution

AC1 reds on any `INSERT` outside the sanctioned function. **AC2 requires a raw `INSERT` in a test**,
to prove the DDL CHECK bites. Validation wrote AC2's test as prescribed and the gate reddened on the
story's own test — *"1 unsanctioned access … repo.rs:1159"* — taking the gate's own *"green on the
real tree"* test down with it. **Whichever rule you pick, one of the two ACs fails as first written.**

Validation's resolution was to exempt `#[cfg(test)]`, and it MEASURED the hole that costs: a planted
violation inside a test module then goes invisible.

🔑 **Prescribed instead: a TWO-SITE ALLOWLIST, by function name, with no `cfg` exemption.** The gate
sanctions exactly `repo::insert_declared_attribute` and ONE named test helper (e.g.
`raw_declared_write_for_ddl_test`), and reds everywhere else including inside `#[cfg(test)]`. It is
narrower than a blanket exemption and it keeps test modules covered. **Measure that a THIRD site
still reds** — an allowlist that accidentally matches by prefix would be the `float-free` failure
again.

#### 4b. 🔴 `DELETE` and the exemption are ONE decision, not two

Validation measured: putting `delete from` in the verb list with no exemption **reds the committed
tree** at `main.rs:413` and `repo.rs:1110`.
**Prescribed: `DELETE` is OUT of the verb list**, with its reason stated in the gate's doc — NFR5 is
about AUTHORSHIP, and a `DELETE` writes no author. Register that a bulk delete is a different
invariant (data loss) that this gate does not hold.

#### 4c. 🔴 The gate must walk `.sql` and `docker/`, not only `.rs` under `crates/`

Two measured holes in the first draft's scope:

- a planted `migrations/0005_scratch.sql` containing `UPDATE declared_attribute SET actor_id =
  'engine'` left the gate **GREEN**. A migration is the most natural home for a bulk author rewrite;
- `docker/seed-example.sql` (§1) is outside `crates/` entirely.

**Prescribed:** walk `.rs` **and** `.sql`, under `crates/` **and** `docker/`; exempt `0001`'s
`CREATE TABLE`. If any scope is left out, say which and why — D18: a gate that cannot fall is
decoration.

#### 4d. The WRITE evasions — every row below was executed

| evasion | gate | note |
|---|---|---|
| `INSERT INTO declared_attribute` in a new file | **RED** | |
| `insert into declared_attribute` (lowercase) | **RED** | |
| `UPDATE declared_attribute SET …` | **RED** | |
| `REPLACE INTO declared_attribute` | **RED** | |
| `INSERT INTO  declared_attribute` (two spaces) | **RED** | |
| `INSERT INTO` ⏎ `declared_attribute` (newline) | **RED** | 🔑 forces a **whole-file, whitespace-normalised** matcher, not `float-free`'s per-line one — the one structural difference from the precedent |
| the name in a doc comment | **green** | as intended |
| `` `declared_attribute` `` (backticked) | 🔴 **green — HOLE** | must red |
| `opencmdb.declared_attribute` (schema-qualified) | 🔴 **green — HOLE** | must red |
| `INSERT declared_attribute` (no `INTO`; MariaDB accepts it) | 🔴 **green — HOLE** | must red |
| `INSERT /*c*/ INTO declared_attribute` | 🔴 **green — HOLE** | must red |
| the name built by `format!` | **green** | 🔑 unavoidable for a text gate — **STATE it** (D18) |
| `/* INSERT INTO declared_attribute */` (block comment) | **RED — false positive** | inherited from `float-free`, which strips LINE comments only. List as a known limit |

#### 4e. 🔴 The READ half needs its own table, and the naive matcher produces FALSE POSITIVES

The first draft gave the read half no evasion table at all. Validation wrote the obvious backward
`SELECT` search (`before.rfind("select")`) and it ran off the end of its own statement, reporting
**two phantom findings on the clean tree** — a bare `DELETE FROM declared_attribute` at `repo.rs:1184`
blamed for an `origin`/`actor_id` appearing in an unrelated INSERT string 24 lines above. Fix
measured: require no `"` between the `SELECT` and the table, so the match cannot span two literals.

| read evasion | must the gate red? |
|---|---|
| `SELECT entity_id, attr_key, attr_value FROM declared_attribute` | **no** — this is the sanctioned read |
| `SELECT origin FROM declared_attribute` | **yes** |
| `SELECT actor_id FROM declared_attribute` | **yes** |
| `SELECT origin_obs_id FROM declared_attribute` | 🔴 **yes** — the DDL has **THREE** provenance columns, and the first draft named two. `origin_obs_id` is *"the adopted observation"* (`0001:15`): it is *how the value was obtained*, arguably more so than `origin` |
| `SELECT * FROM declared_attribute` | 🔴 **yes** — it loads all three and defeats every column-name rule |
| a bare `DELETE FROM declared_attribute` with an unrelated `origin` earlier in the file | **no** — the false positive above |

### 5. The second AC's invariant is held by a COLUMN LIST, three layers below where it is stated

*"A divergence computation never consults HOW a declared value was obtained"* (FR13's invariant,
NFR5's second clause). Today that is true because `load_declared_attributes` selects three columns
and `build_view` receives `Vec<(String, String, String)>` — **provenance never enters the process**.

🔴 **Weaker than it looks, and the first draft over-claimed it.** Widening the TUPLE is a compile
error at `build_view` (measured: `error[E0308]`). Widening only the **SELECT** is not — measured by
both validation layers: the query gains `origin`, **nothing fails to compile and all 450 tests stay
green**, because sqlx decodes tuples positionally and drops the extra column on the floor. Nor does
anything stop a future story adding a *second* query. So the gate is not a belt-and-braces here: for
the SELECT it is **the only guard there is**, and AC3 rests on it alone.

⚠️ Note the asymmetry with §1's mechanism 4: the compile error protects the EXISTING call site, not
the addition of a new one. Story 5.11b measured the same shape and it is registered there —
*"the pure tests of `contradicts` do not protect its wiring"*.

### 6. The tree this story extends, measured on `5634a1d`

- **`crates/opencmdb-bin/migrations/0001_initial.sql`** — `declared_attribute` with **three**
  provenance columns (`origin` [`:14`], `origin_obs_id` [`:15`], `actor_id` [`:16`]), the PK [`:18`]
  and two CHECKs [`:19`, `:20`]. Checksummed: editing it breaks any database that has ALREADY
  applied it — which is the NAS, not CI (§2).
- **`crates/opencmdb-bin/src/repo.rs`** — `insert_declared_attribute` [`:113`], its SQL literals
  [`:123-126`]; `load_declared_attributes` [`:181`], its column list [`:187-189`], its return type
  [`:183`]; `count_declared_attributes` (the free fn at [`:101`], the delegating method at [`:88`]).
- **`crates/opencmdb-bin/src/page.rs`** — `reconcile_view` [`:258`] → `build_view`; the divergence.
- **`xtask/src/main.rs`** — the gate to copy is `gate_float_free` [`:1038`], its matcher
  `line_has_float` [`:915`], `strip_line_comment` [`:930`], `float_literal_kind` [`:964`],
  `fn report` [`:195`], and the scratch-tree idiom `scratch(tag)` [`:1340`].
  ⚠️ **`:1737` is a section banner INSIDE the test module** (`#[cfg(test)]` begins at `:1137`) — the
  first draft sent the implementer there. The two validation layers then disagreed about
  `line_has_float`'s line and **I measured it myself: 915.** A subagent finding is a lead, not a fact.
  **1975 lines total, 1136 of CODE.** Validation built the gate and measured the cost:
  **1136 → 1346 code lines**, 654 to spare under the 2000 ceiling. The ceiling is NOT at risk and
  splitting is not this story's call.
- **`master` is at 450 tests** (245 bin + 159 core + 46 xtask), six gates green.

### 7. 🔴 A green suite says NOTHING about the database

`DATABASE_URL` is unset locally and every DB-backed test passes by `return`ing. Story 5.12 has a DDL
CHECK to prove bites (§8's AC2), which is **invisible without a live server** — and story 5.9's
review measured **four DDL guards droppable with the whole suite green** for exactly this reason.

```
docker run -d --rm --name opencmdb-5-12 -p 13307:3306 \
  -e MARIADB_ROOT_PASSWORD=<choose> -e MARIADB_DATABASE=opencmdb mariadb:10.11.11
export DATABASE_URL='mysql://root:<choose>@127.0.0.1:13307/opencmdb'
```

⚠️ **Never 3306** (held by an unrelated container). **13307 rather than 13306** — story 5.11b's
container may still be on 13306; validation hit exactly that collision. Tests take `crate::DB_TEST_LOCK`.
⚠️ **A CHECK reachable only by going around the adapter needs RAW SQL to measure it** — story 5.9's
M3 came back green until two raw inserts were written.

### 8. Gates

`cargo xtask ci` (six, becoming **seven**) · `cargo clippy --workspace -- -D warnings` **and**
`--all-targets` · `#![deny(missing_docs)]` is on for `opencmdb-bin` and `xtask` · `file-size` counts
only lines before the first `#[cfg(test)]`.

---

## Decisions taken at contexting, and revised at validation

1. **The guard is an `xtask` GATE, not only a test** (§4) — because the invariant is about a code
   path that does not exist yet, and no runtime test can hold that. D56/D65's idiom, `float-free`'s
   precedent.
2. **Its matcher's evasions are measured BEFORE it is declared sound** (§4) — `float-free`'s first
   matcher was measured wrong in both directions.
3. **The DDL CHECK is a tripwire, not a gate** (§2, reading (a)) — it bans one padded value; the gate
   holds the property. Its false comment is registered and corrected in the ADAPTER's doc.
4. **The divergence blindness is held by the gate ALONE** (§5) — not by the tuple's arity, which was
   measured NOT to fire on a widened SELECT. This is the correction validation forced.
5. **A TWO-SITE ALLOWLIST by function name, no `cfg(test)` exemption** (§4a) — because AC1 and AC2
   contradicted each other as first written, and a blanket exemption was measured to hide a planted
   write inside a test module.
6. **`DELETE` is out of the verb list, and the gate walks `.sql` and `docker/` too** (§4b, §4c) —
   both forced by measurement: DELETE reddened the committed tree, and a planted migration was
   invisible.
7. **Every AC names the mutation that must red it** (§3), and a green mutation is a HIGH finding.

---

## Acceptance Criteria

**AC1 — no code path writes a `declared_attribute` with a non-human author, and the GATE reds if one
is introduced.**
Given the workspace, when `cargo xtask ci` runs, then `declared-authorship` walks **`.rs` and `.sql`
under `crates/` AND `docker/`** and reds on any `INSERT`/`UPDATE`/`REPLACE` targeting
`declared_attribute` outside its **two-site allowlist** (§4a): `repo::insert_declared_attribute` and
one named test helper. `DELETE` is deliberately OUT of the verb list (§4b).
**And the gate is shown RED on a planted violation and green on the real tree**, both recorded.
**Mutations:** every row of §4d, one at a time. The four rows marked 🔴 HOLE must red once fixed; the
`format!` row must stay green **and be stated as a known limit**.

**AC2 — the DDL CHECK bites, measured through RAW SQL, and its limit is pinned.**
Given a live MariaDB, when a raw `INSERT` supplies `actor_id = 'scanner'`, then the database refuses
it (`ERROR 4025` → `RepositoryError::Constraint("check")`, measured). 🔴 The adapter cannot produce
this, so a test written through `insert_declared_attribute` measures **nothing** — story 5.9's M3, a
fourth time. **The raw write lives in the allowlisted helper of §4a**, which is what stops it
reddening AC1's gate.
⚠️ **The assertion must be `assert!(result.is_err(), …)`, not `.expect_err()`** — validation measured
the prescribed shape reddening as a PANIC, and this story must not repeat 5.11b's mislabelled carrier.
**And** a second test records what the CHECK does NOT hold: `actor_id = 'engine'` is **accepted**
(measured), while `'scanner '` with a trailing space IS refused (CHAR padding). That test pins §2's
limit and must be labelled as the honest limit, not as a defect.
**Mutation:** drop `declared_actor_not_scanner` in a scratch migration; the test must red.

**AC3 — the divergence computation never consults how a declared value was obtained.**
Given `reconcile_view`, when the page renders, then no provenance column reaches `build_view`.
🔴 **The gate is the ONLY guard here** (§5): widening the SELECT alone compiles cleanly and leaves all
450 tests green. So the gate must red on a divergence-path read naming **`origin`, `actor_id` OR
`origin_obs_id`** — three columns, not two — **and on `SELECT *` against `declared_attribute`** (§4e).
It must NOT red on the sanctioned three-column read, nor on the false positive §4e measured.
**Mutation M4:** add `origin` to `load_declared_attributes`' SELECT. **The carrier is the gate ALONE**
— measured: zero compile errors, 450 tests green, only the gate's test reds. State it that way; the
first draft predicted a compiler carrier and was wrong.

**AC4 — the tests are guards that red on removal, not assertions over current behaviour.**
Given each new test and the gate, when it is removed or neutered, then something reds, recorded per
guard **with its carrier**. 🔴 A guard deletable with the suite green is a HIGH finding.
⚠️ **Two carriers are already known to be awkward and must be reported honestly**: AC2's red is
DB-dependent (below), and M6's is too.

**AC5 — the gate's cost is stated.** Measured at validation: `xtask/src/main.rs` goes
**1136 → 1346** code lines against the 2000 ceiling, 654 to spare. Record the number `cargo xtask ci`
actually prints. The ceiling is not at risk.

**AC6 — nothing else moves, in the SHIPPED DIFF.**
`identity/` untouched · `fixtures/` untouched · trap corpus still **11 unanswerable**,
`passed() == false` · `0001_initial.sql` **NOT edited** (§2) · no new dependency in any `Cargo.toml`
or in `Cargo.lock` (`walkdir` and `anyhow::Context` are already `xtask`'s) · six gates become seven,
all green · both clippy forms clean · `ci.yml` needs no edit — it is a thin runner with no gate list.

**AC7 — the doc twins say the same thing.**
`CLAUDE.md`, `docs/project-context.md`, `sprint-status.yaml` and this file agree on status and counts.
🔴 This failed on seven consecutive stories; 5.11b broke the streak. Check by opening them.

**AC8 — what this story does NOT cover is named.**
NFR5 has **three** assertions (`prd.md:1208-1221`). This story covers the third (*"no code path
writes a declared field with a non-human author"*, `:1218-1219`) and FR13's blindness corollary
(`:1220-1221`). ⚠️ **The first two are NOT covered and the story must say where they go**: (1)
*ingesting an observation that contradicts a declared field leaves it unchanged and opens a
divergence* and (2) *documenting leaves the observation record bit-for-bit unchanged* both need the
`document` gesture, which is the triage epic's. Register both with an owner rather than leaving AC1's
scope to imply the whole NFR is met.

---

## Tasks / Subtasks

**T1 — the gate's matcher, and BOTH evasion tables first.** (AC1, AC3)
Write the write matcher (§4d) and the read matcher (§4e) as tests before wiring anything. The read
matcher's false positive is measured and must be reproduced then fixed. Note the structural
difference from `float-free`: the newline-split case forces a **whole-file, whitespace-normalised**
matcher, not a per-line one.

**T2 — wire it as the seventh gate, over the full scope.** (AC1, AC5)
Copy `gate_float_free` [`:1038`]; report through `fn report` [`:195`]. Walk `.rs` and `.sql` under
`crates/` and `docker/`. Show it red on a planted violation and green on the real tree; record the
`file-size` number.

**T3 — the DDL CHECK, through raw SQL, in the allowlisted helper.** (AC2)
Two tests: `'scanner'` refused (assertion-carried, not `.expect_err`), `'engine'` accepted and
labelled as the honest limit.

**T4 — the divergence blindness.** (AC3) Three columns and `SELECT *`. The gate is the only guard.

**T5 — prove-to-red.** (AC4) Commit first. Each mutation under a timeout, **carrier PER TEST**, and
the driver's red count **corrected by one** (it includes cargo's `result: FAILED` line).

| | mutation | measured at validation |
|---|---|---|
| M1 | plant an `INSERT INTO declared_attribute` with a scanner author | gate reds; no `mod` line needed — the gate walks the filesystem |
| M2 | each row of §4d | six red, one green-as-intended, **four HOLES**, one unavoidable, one false positive |
| M3 | drop `declared_actor_not_scanner` in a scratch migration | AC2 reds — **1 test, and it was PANIC-carried in the prescribed shape** |
| M4 | add `origin` to `load_declared_attributes`' SELECT | 🔴 **carrier is the GATE ALONE**: 0 compile errors, 450 tests green |
| M5 | neuter the gate | its own test reds, assertion-carried |
| M6 | change the sanctioned `'operator'` literal to `'scanner'` | gate stays green (the write is inside the sanctioned fn); **the DDL CHECK catches it at runtime** — 2 tests red, both `.expect()`-carried. ⚠️ **Entirely silent without `DATABASE_URL`**: 246/159/48 all green |

**T6 — docs and register.** Register: §2's weakness and `0001`'s false-but-conditionally-editable
comment (owner: the migration-consolidation milestone, issue #50); §4d's `format!` hole and the block-comment
false positive; §4b's DELETE exclusion; and AC8's two uncovered NFR5 assertions (owner: the triage
epic). Then the twins (AC7).

---

## Dev Notes

### Shapes to follow, not reinvent

- The six existing gates in `xtask/src/main.rs`: each returns `(bool, String)` and is reported
  through `report(name, ok, message)` at `:178`. The `float-free` gate at `:1737` is the closest
  precedent — read it before writing the seventh.
- Gate tests build a scratch tree under a per-module `scratch_dir` embedding `std::process::id()`.
- Deliberate redundancies a DRY pass may NOT collapse: `fixtures.rs`'s `expected()`, `score.rs`'s two
  representations pinned by an equality test, the per-module `scratch_dir`.

### Compile-level facts

- `sqlx` is built without `chrono`: a `DATETIME(6)` has no Rust type to decode into; instants come
  back as strings via `CAST(… AS CHAR)`.
- `insert_declared_attribute` takes `(executor, entity_id, attr_key, attr_value)` — **there is no
  actor parameter**, which is mechanism 1 and must survive this story.
- `sqlx` checksums migration FILES, comments included. `0001_initial.sql` is applied everywhere and
  **must not be edited** — the same trap `0002:83` carries, registered at story 5.9's review.

### What a reviewer will challenge, and the answer that must already be measured

- *"Can any of these tests fail?"* → §3, and every AC names its mutation. This is the question the
  story is designed around, and 5.11b is the evidence that it is not rhetorical.
- *"Why a gate rather than a test?"* → §4. A test exercises what exists; the violation is what a
  future story adds.
- *"Is the CHECK enough?"* → No, and §2 says exactly what it is worth. AC2's second test pins it.

### References

- `epics.md:1654-1670` (story 5.12 as written), `prd.md:1208-1215` (NFR5's three assertions),
  `:884-888` (FR13, and *"the divergence computation never consults how a declared value was
  obtained"*).
- `architecture.md` — D3 (field-level provenance), D10 (comparison never descends into SQL), D18 (a
  gate that cannot fall is decoration), D56/D65 (gates live in `xtask`, in Rust).
- `deferred-work.md` — story 5.11b's entries, whose shape this story repeats.

---

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context) — `claude-opus-5[1m]`, via `dev-story`, 2026-08-07.

### Debug Log References

**Environment.** Built and mutated against a live `mariadb:10.11.11` on host port **13307**
(container `opencmdb-5-12`), chosen over 13306 because story 5.11b's container still holds it —
the collision validation predicted.

**Counts.** `master` 450 (245 bin + 159 core + 46 xtask) → **463 (248 + 159 + 56)**.
`file-size` largest: 1136 → **1421** (validation predicted 1346 for a slightly smaller gate). 579
lines of headroom under the 2000 ceiling; splitting is not this story's call.

**The mutation table, measured. Zero compiler-carried — and the carrier is MIXED, which the
header of this table denied until the repair pass.**

🔴 It read *"Every red assertion-carried; zero compiler-carried"* while its own **M6** row, four
lines below, recorded `.expect()` panic ×3. Story 5.9b shipped that exact defect in five documents
and this story reproduced it: the driver read the carrier off the whole test output, so a MIXED set
collapsed to one label. The driver now prints each red test's panic MESSAGE and the carrier is
classified by reading them, one by one.

| | mutation | measured | carrier |
|---|---|---|---|
| M1 | plant a scanner-authored `INSERT` in a new `.rs` file | gate RED; 1 test | assertion |
| M1sql | plant `UPDATE declared_attribute SET actor_id = 'engine'` in a new `.sql` migration | 🔴 **GATE GREEN at first** — see below; RED after the fix | assertion |
| M2backtick | neuter the backtick / schema-qualified handling | 1 test RED | assertion |
| M2noInto | drop the bare `insert` / `replace` verbs | 1 test RED | assertion |
| M2wildcard | drop the `SELECT *` rule | 1 test RED | assertion |
| M2thirdcol | drop `origin_obs_id` from the provenance columns | 1 test RED | assertion |
| M2quote | drop the `"` bound on the statement fragment | 4 tests RED | assertion |
| M3 | drop `declared_actor_not_scanner` from the live database | 1 test RED, at `repo.rs:1158` | **assertion** — not `.expect_err`, which AC2 required |
| M4 | add `origin` to `load_declared_attributes`' SELECT | 🔴 **carrier is the GATE ALONE**: 0 compile errors, all 248 bin + 159 core tests GREEN | assertion (the gate's test) |
| M5 | neuter the gate — always returns green | 5 tests RED | assertion |
| | ⚠️ **M5's label is wrong, and the code review's headline defect hid behind it.** The mutation was applied to `authorship_findings` — the MATCHER — never to `gate_declared_authorship`'s body. Re-measured during the repair: neutering the matcher reds **8** tests today; replacing the gate BODY with a pass reddened **0** before this pass, and 2 now (M20). *"Neuter the gate"* named the thing that was not tested after the thing that was. | | |
| M6 | the sanctioned adapter writes `'scanner'` | gate stays GREEN (the write is inside the sanctioned fn); the **DDL CHECK** catches it — 3 tests RED. ⚠️ **0 RED without `DATABASE_URL`** | `.expect()` panic ×3 |
| M7allowlist | make the allowlist match by PREFIX | 1 test RED | assertion |

### Completion Notes List

- **The seventh gate ships.** `declared-authorship` walks `.rs` **and** `.sql` under `crates/`
  **and** `docker/` — 31 files — and reds on an unsanctioned write to `declared_attribute` or a
  divergence read naming a provenance column. ⚠️ *"And the shape works"* stood here until the code
  review measured **16 of 30** hand-written evasions passing it. See §12.
- 🔴 **M1sql was GREEN on first measurement, and the reason was not the scope.** The gate DID read
  the planted migration (its count rose 31 → 32) and found nothing: `strip_line_comment` handles
  `//` only, so the `--` header ran into the statement under whitespace normalisation and the
  fragment no longer BEGAN with `update`. Closed by `strip_sql_comment`, pinned by
  `a_bulk_author_rewrite_in_a_sql_migration_reds`, which also asserts a `--` inside a quoted literal
  stays data.
- 🔴 **Two false positives, both the *"wrong in both directions"* family.** `SELECT COUNT(*)` was
  read as a wildcard — **the gate's very first red, on the committed tree at `repo.rs:106`** — and a
  match could span two string literals. Both closed and both pinned by a test.
- 🔴 **A prediction of the story's, refuted — and the refutation was refuted in turn.** The story
  said this gate would inherit `float-free`'s block-comment false positive; it did not, and the
  reason recorded was the statement-HEAD anchor, which matched no verb inside `/* … */`. **The code
  review then defeated that same anchor**: probe `e08` puts a real `INSERT` behind a CLOSED comment
  (`/* hi */ INSERT INTO declared_attribute …`) and the gate went green on it for exactly the
  reason it was green on a commented-out one. The anchor is gone (see the repair below); block
  comments are now stripped outright, which keeps the commented-out case green for a reason that
  survives its own probe.
- 🔴 **THREE sanctioned sites, where the story prescribed two.** The third is
  `docker/seed-example.sql`, forced by the story's own requirement to walk `docker/`. Registered with
  its consequence: an edit to that file could change its actor and pass the gate.
- **AC1 and AC2 no longer contradict each other.** AC2's raw write lives in
  `raw_declared_write_for_ddl_test`, an allowlisted site, so the gate stays green on it — measured.
  No `#[cfg(test)]` exemption, which validation had measured hiding a planted write.
- 🔑 **A fifth mechanism, absent from the story's first draft and the strongest of the five**: the PK
  with no `ON DUPLICATE KEY UPDATE` means the adapter cannot overwrite a declared value at all.
  `the_adapter_cannot_overwrite_an_existing_declared_value` measures it and reads the old value back.
- ⚠️ **A driver defect cost 64 spurious reds and is registered as a method note**: `git checkout`
  does not restore a database, and a planted migration leaves `_sqlx_migrations` referencing a file
  that no longer exists.

### 12. 🔴 The code review, and the repair Guy arbitrated (voie A)

Three review layers ran on 2026-08-07. What they found is not a list of small defects: **the gate
did not hold the property it claimed.**

**The measurement.** The Edge Case Hunter wrote **thirty** violations of NFR5 against the shipped
gate, one per mechanism. **Sixteen passed it** — and three of those executed successfully against a
live `mariadb:10.11.11`. They are now committed, at `xtask/probes/authorship/`, because a repair
argued about is worth nothing next to a repair measured: whatever this gate becomes, it is measured
against all thirty-two.

**The second finding, and the structural one.** The whole body of `gate_declared_authorship` — its
walk, its two roots, its extension filter, its sanctioned-path match, both fail-closed arms — was
**deletable with the entire xtask suite green (56/56)**. Every test attacked `authorship_findings`
directly. `float-free` had carried the end-to-end test since story 5.4b, and its doc says in so many
words that testing only `line_has_float` *"would leave all three untested while the gate read as
covered"*. This story was written on `float-free`'s precedent and did not copy the part that
mattered. ⚠️ It hid behind a mutation label: **M5 said *"neuter the gate"* and was applied to the
matcher** (§T5's table, now annotated).

**Guy's arbitration: option A — repair AND narrow the promise.** Not (B) close it with a database
`GRANT`, not (C) drop the gate. The narrowing is half the deliverable, not an apology for the other
half.

#### What was repaired

| mechanism the review defeated | repair | probes |
|---|---|---|
| statement-HEAD anchor | `governing_keyword` — the write verb or `select` whose match ENDS latest before the reference governs it; longest wins at equal ends | `e08`, `e09`, `e27`, `e29`, `e30` |
| block comments unread | `CommentState` + `strip_comments`, carried ACROSS lines; `/*! … */` is MariaDB's **executable** comment, so its markers are dropped and its body KEPT, where an ordinary `/* … */` is dropped whole | `e07`, `e08`, `e09`, `e10`, `e21` |
| invisible characters | `is_invisible` — deleted, not treated as whitespace: a zero-width space INSIDE a word is one word to the server and two to a collapsing matcher | `e06`, `e32` |
| `enclosing_fn`'s unbounded `rfind("fn ")` | a candidate must look like a DECLARATION (its name followed by `(` or a generic list) and its braces must still be OPEN at the reference | `e16`, `e17` |
| verb list | `load data`, `create or replace table` added | `e13`, `e22` |
| the read half inspected only what came BEFORE the table name | `statement_after` — a provenance column read in a predicate is read | `e18`, `e19`, `e28` |

**Result: 32 probes, 29 red, 3 green by stated decision.**

#### What the promise is now, and what it is not

🔴 **A tripwire against the good-faith violation, not a barrier against a determined one.** The three
green probes are the shape of the residual hole, and they are two classes, not three:

- **A query assembled at runtime** (`e02`, and the `format!` case the story already pinned — ONE
  limit with two witnesses). A matcher that reads source text cannot follow a table name that does
  not exist until the program runs. This is the class, not an oversight.
- **Neutralising the guard instead of writing under a false author** (`e14` `RENAME TABLE`, `e31`
  `ALTER … DROP CONSTRAINT` — both touch no row at all). **This gate guards the write, not the guard
  itself, and the guard of the guard is a privilege the database refuses.** It is the one place the
  gate is green on something that DESTROYS the mechanism rather than routing around it.

⚠️ **The criterion does not cleanly separate `e22`, and the first draft of this section pretended it
did.** `CREATE OR REPLACE TABLE` writes no row under a false author either — it destroys the table
and every declared value in it — and it REDS. Mutation **M19b proves the red is incident**: it
survives removing the verb from `WRITE_VERBS`, because the `REPLACE` inside the phrase governs the
same reference. It is kept because the gesture annihilates the guarded table from inside a `.sql`
migration, the place this story measured as the most natural home for a bulk rewrite — not because
the authorship test demanded it. **Found by reading, not by the mutation pass**, by the second
session that had been launched on this same story.

**Voie B is registered as the real closure**, not implied by this one: a MariaDB `GRANT` denying the
application's own role the right to write `declared_attribute` outside the sanctioned path holds
against source text this gate never reads, and against a hand-run statement no gate reads at all.
Read this gate as *"a future story will not add such a write by accident"* — never as *"such a write
cannot exist"*.

#### The repair's own mutation pass

Ten mutations, **ten reds, every one carried by a NAMED assertion; zero compiler-carried and zero
`.expect()`-carried** — and the carrier was classified by reading each red test's panic message, one
by one, because the header of §T5's table had denied its own M6 row by reading the carrier off the
whole output.

| | mutation | measured |
|---|---|---|
| M13b | `is_invisible` never fires | 1 red — `e32` |
| M14 | `/*!` treated as an ordinary comment (body dropped) | 1 red |
| M15 | block comments unhandled | 1 red — and it is `a_write_inside_a_block_comment_stays_green` that catches it |
| M16 | `enclosing_fn` back to the unbounded `rfind` | 1 red, 2 probes |
| M17 | `governing_keyword` back to the statement-head anchor | 2 red, 2 probes |
| M18 | the read half stops inspecting the text after the table name | 1 red, 3 probes |
| M19b | `create or replace table` dropped from the verb list | 1 red |
| M20 | **the gate BODY replaced by a pass** | **2 red — 0 before this pass. The review's headline defect, now measured.** |
| M21b | an orphan probe file planted in the corpus | 1 red |
| M22b | the gate walks only its first root | 2 red — including the file-COUNT assertion |

🔴 **Two of these came back GREEN on their first run, and both refutations are the deliverable.**
`is_invisible` was carried by nothing: `e06` puts its zero-width space BEFORE the verb, where a token
boundary already exists — hence `e32`, which puts one inside both the verb and the table name, where
deleting it is the only thing that finds the statement at all. And `create or replace table` changes
no VERDICT — the `replace` hiding inside it governs the same reference — only the finding's NAME, so
the longest-wins rule got a test of its own rather than a verb nobody measured.

⚠️ **And two mutations were mis-designed rather than the guards being weak**: removing a test's own
assertion cannot red that test. For an assertion, the honest mutation breaks the INVARIANT — plant
an orphan, make the count lie. That is M21b and M22b.

#### Three documents that described a state the code had already passed

Counted together, because it is the same defect three times and the sixth consecutive story with the
shape — **two of them in the gate's own file**:

1. §T5's table header: *"every red assertion-carried"*, denied by its own M6 row four lines below
   (`.expect()` panic ×3). Story 5.9b's defect, reproduced.
2. `xtask/src/main.rs`'s module doc enumerated **six** gates while the file implemented seven — the
   gate this story adds was missing from the list the file gives of itself.
3. `docs/project-context.md` announced six gates.

### File List

| file | change |
|---|---|
| `xtask/src/main.rs` | **the seventh gate** — `gate_declared_authorship`, `authorship_findings`, `normalise_sql_text`, `outside_parens`, `is_table_reference`, `statement_before`, `enclosing_fn`, plus 10 tests. **Repaired under voie A**: `CommentState`, `strip_comments` (replacing `strip_sql_comment`), `is_invisible`, `statement_after`, `governing_keyword`, `WRITE_VERBS`, `is_word_at`, and 4 new tests including the gate's own end-to-end test |
| `xtask/probes/authorship/` | **new** — the code review's 30 evasion probes, `e31` and `e32` added during the repair, and a README stating the three green-by-decision verdicts |
| `crates/opencmdb-bin/src/repo.rs` | `declared_fixture`, the allowlisted `raw_declared_write_for_ddl_test`, and 3 database tests |
| `_bmad-output/implementation-artifacts/deferred-work.md` | 8 new entries |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | status |
| `_bmad-output/implementation-artifacts/5-12-never-overwrite-anti-regression.md` | this record |
| `CLAUDE.md`, `docs/project-context.md` | the doc twins (AC7) |

---

## Change Log

| Date | Change |
|---|---|
| 2026-08-07 | **Code-reviewed (three layers), then REPAIRED under Guy's arbitration — voie A: repair AND narrow the promise.** 🔴 The review measured **16 of 30** hand-written evasions passing the gate, three of them executing against a live MariaDB, and **the whole gate BODY deletable with the xtask suite green** — every test attacked the matcher, and mutation **M5's label said *"neuter the gate"* while the mutation hit `authorship_findings`**, which is where the defect hid. The 30 probes are committed as a regression corpus (`xtask/probes/authorship/`), `e31` and `e32` added during the repair, all 32 verdicts pinned in BOTH directions. The repair: `governing_keyword` replacing the statement-head anchor, block comments across lines with `/*!` kept as executable code, invisible characters deleted, `enclosing_fn` bounded, `statement_after` for the read half, two verbs added. **29 red, 3 green by stated decision** — one runtime-assembly limit with two witnesses, and guard NEUTRALISATION, whose honest closure is the database `GRANT` of voie B, registered rather than implied. Ten mutations, **ten reds, every one carried by a named assertion**; 🔴 **two came back GREEN first** (`is_invisible` carried by nothing until `e32`; the long verb changing the finding's NAME but no verdict) and two were **mis-designed** — removing a test's assertion cannot red that test. **463 → 467 tests** (60 xtask). Three documents described a state the code had passed, two of them in the gate's own file. |
| 2026-08-07 | Implemented by `dev-story` against a live `mariadb:10.11.11` on 13307. **450 → 463 tests**, **seven gates green**, both clippy forms clean, no new dependency. 🔴 **M1sql was GREEN on first measurement** — the gate read the planted `.sql` migration but `strip_line_comment` handles `//` only, so the `--` header ran into the statement and it no longer began with its verb; closed and pinned. 🔴 **Two false positives**, both *"wrong in both directions"*: `SELECT COUNT(*)` read as a wildcard (the gate's first red, on the committed tree) and a match spanning two string literals. 🔴 **A prediction of the story's refuted**: the gate does NOT inherit `float-free`'s block-comment false positive, because it anchors on a statement's head. **Three sanctioned sites, not two** — `docker/seed-example.sql` is forced by the story's own scope. M4's carrier is the **gate alone** (0 compile errors, all runtime tests green), M3's is an **assertion** as AC2 demanded, and M6 is **entirely silent without `DATABASE_URL`**. |
| 2026-08-07 | **Validated** by two fresh-context agents; the gap-hunt BUILT the story (453 tests, seven gates green). **3 HIGH from the fact-check, 6 from the gap-hunt**, and 🔑 **two were found independently by BOTH layers**: §1's mechanism 4 is FALSE (a widened SELECT compiles cleanly — sqlx decodes tuples positionally — so AC3's carrier is the GATE ALONE, not the compiler), and AC1 contradicted AC2 (the gate reddens on the raw INSERT that AC2 requires). Resolved by a **two-site allowlist**, not the `cfg(test)` exemption, which was measured to hide a planted test write. Also: the census said *"whole workspace"* and meant *"under `crates/`"* — `docker/seed-example.sql:28` is a second SHIPPED writer; the gate was blind to `.sql` migrations, to backticked and schema-qualified names, and to `INSERT` without `INTO`; AC3 named two provenance columns where the DDL has **three**, and `SELECT *` defeats them all; the READ half had no evasion table and its naive matcher produced FALSE POSITIVES on the clean tree. A **fifth mechanism** was missing and is the strongest — the PK with no upsert means the adapter cannot overwrite at all. ⚠️ The two layers disagreed on a line number and **I measured it myself** (915, the fact-check was right). |
| 2026-08-07 | Created by `create-story`. Five decisions at contexting. 🔴 **The load-bearing finding: the invariant is already held by FOUR mechanisms and none is a test** — the write path binds SQL literals rather than parameters, a DDL CHECK, the read's column list, and the tuple's arity. So this story is 5.11b's shape, and 5.11b's mutation pass is the evidence that the shape is dangerous. 🔴 **Its harder half: you cannot test the ABSENCE of a code path by running code**, so the guard is a SEVENTH `xtask` gate on `float-free`'s precedent — whose own first matcher was measured wrong in both directions. ⚠️ Also found: `CHECK (actor_id <> 'scanner')` bans **one string literal**, not a property (`'engine'` passes), and `0001_initial.sql`'s comment claiming *"a human; never 'scanner'"* is false AND unmodifiable, sqlx checksumming migration files including comments. |
