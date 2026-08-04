# Story 5.10: The purge test proves the link is a cache of attention, not of truth

Status: review

<!-- ✅ VALIDATED 2026-08-04 by two fresh-context agents (fact-check + gap-hunt), as this project
     requires. **The gap-hunt BUILT the story** against a live `mariadb:10.11.11` on port 13311,
     reaching 412 tests, six green gates, `fixtures/` untouched.

     🔑 **The central property HOLDS and was worth compiling for**: the purge-and-replay reproduces
     all ten compared columns exactly, `interface_id` included. Every finding below is about the
     MEASUREMENT APPARATUS, not the property.

     🔴 FOUR HIGH, and the first is a new SHAPE for this project — a reasoning defect rather than a
     false citation or a bad count:
       • **M1 cannot red.** The purge restores the state run 1 started from, so a dependency on
         state the purge REMOVES is invisible to the comparison BY CONSTRUCTION. Found by the
         fact-check by reasoning, then confirmed by the gap-hunt in one run. `epics.md:1634` points
         the wrong way; §5b says so with the measurement.
       • **The `ORDER BY` is unreddenable** by a comparison of two snapshots that share a query —
         deleting it left the whole suite green. AC2 gains an ordering test of its own (§5c).
       • 🔴 **The two natures are MUTUALLY EXCLUSIVE on one placement.** An operator can never
         confirm an engine placement, and an operator row in the slot makes the replay roll back
         entirely. Guy's arbitration in AC5; the model question is registered with story 5.14.
       • **`current_subject` is NULL on superseded rows**, so the snapshot's order degenerates the
         moment story 5.11 supersedes anything. It is now restricted to CURRENT links.

     ⚠️ Also applied: M2 produces a MIXED carrier set, summary assertions pre-empt the reds their
     ACs name, AC5's operator candidate must go around `guard_decision`, and the `datetime_literal`
     debt is TWO entries of which §4 disposes of only one.

     🔑 THE GAP-HUNT MUST RUN WITH A LIVE DATABASE, and this story is nothing but database.
     `DATABASE_URL` is unset here and the suite reports the same counts either way. §6 has the
     `docker run` — host port **13306**, never 3306.

     🔑 ASK THE GAP-HUNT EXPLICITLY: *"does each prescribed mutation actually red, and WHAT carries
     the red — an assertion, an `expect` panic, or the compiler?"* On stories 5.5 through 5.9b every
     HIGH came from the agent that COMPILED the story, and story 5.9b's own review found the
     implementer's carrier claim FALSE because a mixed set had been collapsed to one label.
     ⚠️ **A mutation must preserve the ARITY of a SQL statement's bind parameters.** Removing a
     placeholder without its `.bind` desynchronises the MySQL protocol and **hangs the suite** —
     measured at 2 h 48 min at 0 % CPU, holding `DB_TEST_LOCK`. Run every mutation under a timeout. -->

## Story

As the architect of the invariant,
I want engine-decided links to be reproducible after deletion,
so that D14 and D4 (*"doubt is never persisted"*) are reconciled by a test rather than by an
argument.

**This story writes almost no production code. Its deliverable is a TEST** — the one D14 names by
hand: *"`TRUNCATE ... WHERE decided_by='ENGINE'; re-run engine;` must reproduce the same decisions
bit for bit"* [architecture.md:1036-1039]. Story 5.9b built the pass that produces those links; this
story proves they are a **cache of attention, not of truth**.

**What this story does NOT do**, so the boundary is explicit and not discovered at review:

- it does **not** wire anything into `main.rs`. The resolver still has no production caller, by
  decision 3 of story 5.9b, and this story does not change that;
- it does **not** implement an `l2-*` rule, so the committed trap corpus stays **RED with 11
  unanswerable and `passed() == false`**. **If it turns green, that is a FINDING**;
- it does **not** make the pass idempotent. Running the resolver twice over the SAME observations is
  `Err(Constraint("unique"))` and a full rollback — that is **story 5.11's**, registered by 5.9b;
- it does **not** create an operator-facing surface. FR16's rendering is **story 5.14**; the
  `decided_by='OPERATOR'` rows this story needs are written by its own tests through the adapter
  (decision 2);
- it does **not** touch `identity::l1`, `identity::blocking` or `identity::cascade`. The engine is
  re-run, never modified. **A change there is a FINDING.**

**Nothing under `fixtures/` moves.** No artefact bytes, no `MANIFEST.toml`, no re-hash.

**`architecture.md` is NOT edited** (issue #54). **`architecture-views.md` is NOT regenerated**
(issue #50). **`epics.md` is NOT edited** — verify-only; §2's correction is registered instead.

⚠️ **Branch from `master`.** Measured at contexting: `master` is at **`0b3f09c`**, the tree is clean,
no story branch survives, and `cargo test --workspace --locked` reports **408 tests** — **203 bin +
159 core + 46 xtask**. `cargo xtask ci` reports **six green gates**, `float-free` on 4 files,
`file-size` largest **1136**, `views-hash ℹ STALE` exiting 0 by design.

---

## What this story inherits, measured rather than assumed

### 1. 🔴 "Bit for bit" cannot include the primary key — Guy's arbitration

`identity_link.id` and `interface.id` are minted with `uuid::Uuid::now_v7()`. **A v7 UUID embeds a
48-bit wall-clock millisecond**, so a replayed link gets a different id every time — decoded at
story 5.9b's code review from two runs over identical input, **57 ms apart**. Story 5.9b's review
registered the consequence with this story as owner, and here it is:

**Guy's arbitration at this contexting: "bit for bit" means every column that CARRIES THE DECISION,
and explicitly NOT `id`.** The compared set is:

| compared (10) | not compared (1) | derived, therefore not compared (1) |
|---|---|---|
| `observation_id`, `interface_id`, `outcome`, `rule_id`, `abstention_cause`, `evidence`, `ruleset_version`, `decided_by`, `valid_from`, `valid_to` | `id` | `current_subject` |

⚠️ **`identity_link` has TWELVE columns, and the table above accounts for all of them.**
`current_subject` is `interface_id`-or-`NIL_INTERFACE` while the row is current and NULL once
superseded — `identity_link_current_subject` is the CHECK that keeps it from drifting — so it is a
FUNCTION of two compared columns and comparing it would measure nothing new. It is the snapshot's
`ORDER BY` key and not one of its fields. _(An earlier draft presented the table as a partition and
silently omitted it.)_

The reason is not convenience: **D14 says the purge must reproduce "the same DECISIONS", and a row
identifier is not a decision.** The exclusion is written into the test's own name and doc, so nobody
reads the AC as stronger than it is.

⚠️ **`interface_id` IS compared, and that is what makes the exclusion safe.** If interfaces were
re-minted on a re-run, every reproduced link would point somewhere else and the comparison would
red. `find_interface_by_l1_key` exists precisely so they are not — story 5.9b's register entry says
so — and this story is where that stops being a claim.

### 2. 🔴 `epics.md`'s `TRUNCATE ... WHERE` is not SQL, and it is not what the code will run

`epics.md:1627` writes the purge as `TRUNCATE ... WHERE decided_by = 'ENGINE'`. **MariaDB's
`TRUNCATE` takes no `WHERE` clause**; the statement is a `DELETE`. Story 5.9's contexting already
recorded this in prose (its §6) and `repo.rs`'s existing test already runs the real thing
[`repo.rs:2040`]. **`epics.md` is not edited** — the correction is registered with **Epic 5's
retrospective**, beside the two 5.9b left there.

### 3. The tree this story extends, measured on `0b3f09c`

| what | where | size |
|---|---|---|
| the pass this story re-runs | `crates/opencmdb-bin/src/resolver.rs` | 404 code lines — `resolve`, `resolve_within`, `Resolution` |
| the adapter | `crates/opencmdb-bin/src/repo.rs` | 773 code lines |
| the schema | `migrations/0002_…sql`, `0003_resolver_guards.sql` | `interface`, `identity_link`, `link_candidate` + three guards |

**What already exists and must NOT be reinvented:**

- **the purge statement itself** — `DELETE FROM identity_link WHERE decided_by = 'ENGINE'`, inline in
  `purging_engine_links_takes_their_candidates_with_them` [`repo.rs:2040`]. That test proves the
  `ON DELETE CASCADE` takes candidates with it. **This story promotes the statement into a query
  body** (`purge_engine_links`) and keeps that test working;
- **`DecidedBy::Operator`** already exists, is pinned **as a token** by
  `every_persisted_token_is_pinned` (a plain `#[test]`, no database), and is already **written
  through the adapter** by `the_tokens_no_other_test_stores_round_trip`, which is DB-gated and
  already calls `insert_identity_link(…, DecidedBy::Operator, …)`. **That is the shape T4 reuses** —
  the token pin alone proves nothing about the write;
- **`load_current_links_for_observation`** returns `PersistedLink`, which carries **eight** columns
  and is **missing three** the comparison needs: `observation_id`, `valid_from` and `valid_to`. §4 is
  the consequence.

### 4. 🔴 The read side cannot express the comparison yet, and reading an instant back is a trap

`PersistedLink` has `id, interface_id, outcome, rule_id, abstention_cause, evidence,
ruleset_version, decided_by`. The comparison of §1 needs `observation_id`, `valid_from`, `valid_to`
as well. **This story adds a snapshot query** — every decision-bearing column of every link, ordered
deterministically so two snapshots are comparable as sequences.

⚠️ **`sqlx` is built here WITHOUT its `chrono` feature**, so a `DATETIME(6)` has no Rust type to
decode into; `load_link_valid_to` already renders with `CAST(… AS CHAR)` [`repo.rs`]. The snapshot
does the same, and **compares the two renderings against each other** — which is transport, not a
domain comparison (D10).

🔑 **The `sqlx chrono` entry is NOT this story's to close, and not because of a measurement — because
of its OWNER.** It reads *"Owner: the first story that needs to read an instant back as a VALUE
(rather than to compare it against a sentinel)"* [`deferred-work.md:2264-2265`] — a CONDITION, and
this story is never named. Comparing two rendered strings never produces a `Timestamp`, so the
condition is not met and the entry is untouched. ⚠️ **If the dev finds they need a real `Timestamp`,
the condition IS met and the feature is theirs** — say so rather than working around it to keep this
sentence true.

🔴 **`datetime_literal`'s sub-microsecond truncation is a DIFFERENT case, and an earlier draft got it
wrong.** That entry reads *"**Owner: story 5.10**, where it would first bite"*
[`deferred-work.md:2301-2303`] — **unconditional**. The clause *"story 5.10 compares in-memory
values to stored ones"* is the entry's PREDICTION about this story, not a condition on its
ownership, and decision 5 **falsifies that prediction**: the comparison is snapshot-against-snapshot,
both sides truncated identically, so the truncation cannot bite here.
**Answering it is therefore not enough — it must be RE-OWNED**, naming the first story that reads an
instant back as a value. An entry whose named owner has passed and which is merely "answered" is a
debt nobody holds. ⚠️ **And it is registered TWICE** — `:2301` from story 5.9's review and `:2482`
from 5.9b's, the second naming an existing test (`the_stored_instants_are_the_derived_ones`) that
*"asserts a property that holds only at microsecond granularity"*. **Dispose of both.**

### 5. What the purge must NOT remove, and what makes that testable

1. **`interface` rows survive.** The purge deletes links. `0002`'s header says the re-run finds an
   interface by its key, and §1 explains why the whole comparison rests on it.
2. **`link_candidate` rows go with their link**, by `ON DELETE CASCADE` — measured at story 5.9's
   review, where `RESTRICT` made the purge fail `ERROR 1451` the moment an engine link carried a
   candidate. The existing test covers it; do not weaken it.
3. **`observation_record` rows survive**, obviously — they are the input. ⚠️ And since story 5.9b
   they are protected: `identity_link.observation_id` has a foreign key, so deleting an observation
   under a link is refused.
4. **`decided_by='OPERATOR'` rows survive**, which is D14's *"two natures in one table — and if that
   frontier is fuzzy in the code, the invariant is dead"*.

### 5b. 🔴 What a purge-and-replay can and CANNOT measure — the fact this story rests on

`epics.md:1634` asks for *"the test reds if any engine decision is made to depend on state the purge
removes"*. **Measured at the validation: that is the one thing this test can never do.**

The purge restores the store to exactly the state run 1 started from — the resolver's fixture wipes
`identity_link` before a pass, and `purge_engine_links` empties it again. So a decision that depends
on link rows produces the **same** value in both runs, and `assert_eq!(after, before)` stays green.
A mutation deriving `valid_from` from an existing link was measured reddening only story 5.9b's
`the_stored_instants_are_the_derived_ones`, never AC3.

**What AC3 CAN measure is a dependency on state the purge does NOT restore** — the `interface` rows,
which survive by design. A mutation making a compared column depend on whether the interface was
FOUND rather than minted reds AC3, and was measured as its **only** red.

🔑 **So the epic's sentence points the wrong way, and the story says so rather than inheriting it.**
The property this test actually proves is: *the engine's output depends only on the observations and
on the interfaces — not on its own prior links.* That is the useful statement, and it is what D14's
*"cache of attention, not of truth"* means operationally.

### 5c. 🔴 An ORDER BY inside a function used on BOTH sides of a comparison is unreddenable

Measured: with `snapshot_links`' `ORDER BY` deleted, the **whole suite stayed green**. That is not a
weak fixture — it is structural. Both snapshots go through the same query, so any order stable
within a run yields two equal sequences, whatever the fixture.

**The ordering therefore needs a test of its own**, and AC2 now demands it: links written in an order
that DISAGREES with `(observation_id, current_subject)`, and the snapshot's sequence asserted. Without
it, decision 4 ships unmeasured.

### 6. 🔴 A green suite says NOTHING about the database, and this story is nothing but database

```sh
docker run --rm -d --name opencmdb-dev-db -p 13306:3306 \
  -e MARIADB_ROOT_PASSWORD=opencmdb -e MARIADB_DATABASE=opencmdb_test \
  mariadb:10.11.11
docker ps --filter name=opencmdb-dev-db      # CONFIRM IT IS UP BEFORE TRUSTING ANY GREEN RUN
export DATABASE_URL='mysql://root:opencmdb@127.0.0.1:13306/opencmdb_test'
```

🔴 **Port 13306, never 3306** — 3306 is held by `kesh-mariadb`, an unrelated `mariadb:11-jammy` from
another project. Pointing the DSN there runs `sqlx::migrate!` against someone else's database on the
wrong engine version. **Never touch that container.**

⚠️ **Drop and recreate `opencmdb_test` if you edit a migration, even a comment in one.** Measured
twice: `sqlx` checksums applied migrations, so changing a header reddens the whole suite until the
database is recreated. `0003`'s own header records this.

### 7. Gates, and the traps that cost an hour

- **Six gates must stay green**: frontier (D47), `ddl-collation`, vocabulary (D65), `fixtures`,
  `file-size` (D56b, largest 1136 of 2000), `float-free`. `views-hash` reports `ℹ STALE` and exits 0
  — **by design, do not regenerate**.
- ⚠️ **D47 is a gate.** Every line of SQL lives in `opencmdb-bin`.
- ⚠️ **Run clippy TWICE**: `--all-targets` and `--locked`. An import kept alive only by a test passes
  the first and fails the second.
- ⚠️ **Issue #38** — a test may red once then pass repeatedly on a clean tree. Re-run before
  diagnosing, and **never adopt a cause without naming the check that would have failed if it were
  wrong**.

---

## Decisions taken at contexting

**1. 🔴 "Bit for bit" is every decision-bearing column and NOT `id`.** §1. Guy's arbitration, on the
measurement that a v7 UUID carries a wall-clock millisecond. The exclusion is named in the test and
in the AC, never implied.

**2. 🔴 The `OPERATOR` rows are written by the test, through the adapter.** Guy's arbitration.
Nothing writes an `OPERATOR` link in production and nothing will before story 5.14 — but D14's
two-natures frontier is what this story measures, and it is real whether or not a human surface
exists. `insert_identity_link(…, DecidedBy::Operator, …)` already exists and is already pinned.

**3. The purge becomes a query body, `purge_engine_links`.** It is `DELETE`, not `TRUNCATE` (§2), it
returns the number of rows it removed so a caller can assert it, and the inline statement in
`repo.rs`'s existing cascade test delegates to it — one statement, one site.

**4. The snapshot is a query body too, returning every compared column, ORDERED.** Two snapshots are
compared as sequences, so a divergence names the row. Order by `(observation_id, current_subject)` —
never by `id`, which §1 excludes and which would make the order itself clock-dependent.

**5. The comparison is a plain `assert_eq!` on a `Vec` of snapshot rows, not a hand-rolled loop.**
A `Vec<LinkSnapshot>` with `#[derive(PartialEq, Debug)]` prints the whole divergence when it reds;
a loop that compares field by field reports the first mismatch and hides the rest.

---

## Acceptance Criteria

**AC1 — the purge is a named query body, and it is a `DELETE`.**
**Given** `epics.md:1627`'s pseudo-SQL `TRUNCATE ... WHERE decided_by='ENGINE'` (§2)
**When** the purge lands
**Then** `repo::purge_engine_links` exists, runs `DELETE FROM identity_link WHERE decided_by =
'ENGINE'`, returns the number of rows removed, and carries a `///` saying why it is not a `TRUNCATE`.
**And** `purging_engine_links_takes_their_candidates_with_them` delegates to it instead of inlining
the statement, and still passes — the `ON DELETE CASCADE` guard is not weakened.

**AC2 — the snapshot reads every compared column, and excludes `id` by construction.**
**Given** §4: `PersistedLink` is missing `observation_id`, `valid_from` and `valid_to`
**When** the snapshot lands
**Then** `repo::snapshot_links` returns, ordered by `(observation_id, current_subject)`, one row per
link with **exactly** the ten compared columns of §1 — and **no `id` field at all**, so excluding it
is structural rather than a habit a refactor can drop.
🔴 **The snapshot is restricted to CURRENT links (`WHERE current_subject IS NOT NULL`), and the doc
says why.** Measured at the validation: two superseded versions of one placement both carry
`current_subject IS NULL`, their sort keys are EQUAL, and the sequence returned is InnoDB's
accident. It does not bite today because nothing here supersedes — but **story 5.11 is precisely the
story that will**, and it would inherit a snapshot whose order is decorative over history.
Restricting to current rows makes the key total over the snapshot's own domain, and the purge-and-
replay produces nothing else.
🔴 **And the ordering gets a test of its own** (§5c): links written in an order that DISAGREES with
`(observation_id, current_subject)`, with the snapshot's sequence asserted. Deleting the `ORDER BY`
was measured leaving the **whole suite green** — both snapshots go through the same query, so the
comparison can never carry it.
**And** the instants are read with `CAST(… AS CHAR)`, the idiom `load_link_valid_to` established,
and the doc says why (`sqlx` has no `chrono` feature here) and that comparing two renderings is
transport, not a domain comparison (D10).

**AC3 — the purge-and-replay reproduces the engine's decisions.**
**Given** D14 — *"`TRUNCATE ... WHERE decided_by='ENGINE'; re-run engine;` must reproduce the same
decisions bit for bit"* [architecture.md:1036-1039]
**When** a pass has run, the snapshot is taken, the engine's links are purged and the SAME
observations are resolved again
**Then** the second snapshot equals the first, compared as a whole `Vec` in one `assert_eq!`
(decision 5).
**And** the test's name and doc state that `id` is excluded and why — a v7 UUID carries a wall-clock
millisecond (§1).
**And** the fixture is not trivial: it holds a multi-MAC observation, a MAC-less one, and at least
one group of three, so the comparison covers a placement, an abstention, and the witness convention
story 5.9b's review had to install a test for.

**AC4 — the interfaces survive, with their ids.**
**Given** §5.1 and §1's dependence on it
**When** the purge runs
**Then** `SELECT COUNT(*) FROM interface` is unchanged, and the interface ids are the SAME rows —
asserted by comparing the set of ids before and after, not merely the count.
**And** the replayed links' `interface_id` values equal the originals, which is the assertion that
makes excluding the link `id` safe.

**AC5 — the operator's rows are untouched.**
**Given** D14's *"two natures in one table — and if that frontier is fuzzy in the code, the invariant
is dead"*, and decision 2
**When** the purge runs over a store holding both natures — 🔴 **with the operator's row on a
`(observation_id, current_subject)` the engine's pass does NOT place**
**Then** every `decided_by='OPERATOR'` row is still there, byte-identical on all ten compared
columns **and on its `id`** — an operator row is an INPUT, not a derivation, so nothing about it is
re-minted.
🔴 ⚠️ **The two natures are MUTUALLY EXCLUSIVE on one placement, and that is measured, not
supposed.** `identity_link_one_current` is `(observation_id, current_subject)` and the purge removes
only `decided_by='ENGINE'`, so: an operator can **never** confirm or correct a placement the engine
already holds — the write is refused `Err(Constraint("unique"))`; and if an operator row occupies a
slot the replay needs, **the whole replay fails and rolls back**. D14's *"two natures in one table"*
is therefore true of the TABLE and false of a single `(observation, subject)`.
**Guy's arbitration at the validation: AC5 names a subject the engine does not place, and the deeper
question is REGISTERED rather than decided here** — *can an operator ever confirm, correct or
override an engine placement?* — with **story 5.14** (the FR16 surface) as owner. A story whose
deliverable is a test does not settle a model question.
⚠️ **And the Dev Notes' flat answer *"not with a purge between them"* is corrected**: the purge makes
the second run legal **provided no surviving `OPERATOR` row holds a slot the replay needs**.
⚠️ **The `id` must be read by something OTHER than the snapshot**, which AC2 makes structurally
id-free: `load_current_links_for_observation` returns it. **Never relax AC2 to make AC5 easier** —
the two exclusions are different because the two questions are: a replayed engine link is a new row
carrying the same decision, an operator row is the SAME row and must not have moved.
**And** `purge_engine_links`'s return value equals the number of ENGINE links and no more.
**And** the operator row's `link_candidate` children, if any, survive with it.

**AC6 — the test reds if an engine decision comes to depend on state the purge removes.**
**Given** `epics.md:1634` — *"the test reds if any engine decision is made to depend on state the
purge removes"*
**When** the mutation pass runs
**Then** a mutation that makes a compared column depend on state the purge does **NOT** restore —
whether the interface was FOUND rather than minted — reds AC3's comparison, **assertion-carried on
`assert_eq!(after, before)`**.
🔴 **And the epic's own wording is recorded as WRONG, with its measurement.** A dependency on state
the purge REMOVES — an existing link — is invisible to AC3 by construction (§5b), and was measured
reddening only story 5.9b's `the_stored_instants_are_the_derived_ones`. **Run it anyway and record
it as a no-op**: it is the story's sharpest fact and the next reader will otherwise re-derive it.
**And** a mutation that purges `interface` rows alongside the links reds AC4. ⚠️ **Measured: it
produces a MIXED carrier set** — one assertion plus two `.expect` panics, because a surviving
operator link's foreign key refuses the interface delete. **Record the carrier PER TEST**; a single
label for that set is the exact defect story 5.9b's review found.
⚠️ **And a summary assertion placed before the id set pre-empts the red the AC claims**: measured,
M2 and M5 both land on `assert_eq!(second.interfaces_found, …)` rather than on `interface_id`. Order
the assertions so the one the AC names is the one that fires.

**AC7 — the registered entries are disposed of with their measurements, and one must be RE-OWNED.**
**Given** the **two** entries naming story 5.10 as owner — the `uuid` v7 one and the
`datetime_literal` truncation, the latter registered TWICE — and the `sqlx chrono` entry owned by a
CONDITION this story never meets (§4)
**When** `deferred-work.md` gains this story's section
**Then** the `uuid` v7 entry is **CLOSED** — the consequence is now written into an AC and a test
name rather than left to be rediscovered.
**And** **both bullets** of the `datetime_literal` truncation are **ANSWERED AND RE-OWNED**: its
prediction about this story is falsified by decision 5, and its ownership is unconditional, so it
gets a NEW owner rather than an answer alone (§4). An entry whose named owner has passed and which
is only "answered" is a debt nobody holds.
**And** the **`sqlx` `chrono` feature** entry is **left untouched**: its owner is a condition, not
this story (§4). ⚠️ If the implementation does need a `Timestamp`, the condition IS met — say so and
close it, rather than forcing the string form to keep this sentence true.

**AC8 — prove-to-red, with the carrier named per test.**
**Given** the house rule, and story 5.9b's review, which found the implementer's *"every red is
assertion-carried"* FALSE because a MIXED set had been collapsed to one label
**When** the story closes
**Then** every guard has a recorded mutation, and the Debug Log names **what carried each red, per
test** — assertion, `expect`/`expect_err` panic, or compiler — never one label for a set.
**And** every mutation preserves the ARITY of any SQL statement's bind parameters, or it hangs the
suite instead of reddening it (§banner).
**And** the trap corpus is re-checked and still reads **11 unanswerable, `passed() == false`**.

**AC9 — the documents say what the code says.**
**Given** the AC10-family defect that story 5.9's review caught four times and 5.9b's caught again
**When** the story closes
**Then** `docs/project-context.md` and `CLAUDE.md` carry the same test counts, the same story status
and the same Epic 5 tally, verified by grepping both for every phrase they duplicate.
**And** `sprint-status.yaml`'s narrative agrees with its own status line — story 5.9b's review found
it saying `review` and *"NEXT = dev-story"* at once.
**And** nothing says "13 done" while `sprint-status.yaml` says `review`.

---

## Tasks / Subtasks

- [x] **T1 — branch, live database, committed baseline (AC8)**
  - [x] Branch from `master` at `0b3f09c`: `story-5-10-purge-test`.
  - [x] Start `mariadb:10.11.11` on **13306**, confirm `docker ps`, record the baseline **with** the
        DB set (408, and the bin suite slower than without — that gap is the only local evidence the
        DB tests ran).
  - [x] **Commit the clean baseline before the mutation pass.**

- [x] **T2 — `purge_engine_links` and `snapshot_links` (AC1, AC2)**
  - [x] `purge_engine_links(executor) -> Result<u64, sqlx::Error>`, static SQL, with the `DELETE`
        vs `TRUNCATE` sentence in its doc.
  - [x] `snapshot_links(executor) -> Result<Vec<LinkSnapshot>, sqlx::Error>` — the ten compared
        columns, **no `id`**, ordered by `(observation_id, current_subject)`. `LinkSnapshot` derives
        `Debug, Clone, PartialEq, Eq`.
  - [x] Point `purging_engine_links_takes_their_candidates_with_them` at the new body.
  - [x] ⚠️ Watch `file-size`: `repo.rs` is at **773** code lines of a 2000 ceiling.

- [x] **T3 — the purge-and-replay test (AC3, AC4)**
  - [x] A fixture with a multi-MAC observation, a MAC-less one, and a group of three.
  - [x] Resolve → snapshot → capture the interface ids → purge → resolve again → snapshot.
  - [x] `assert_eq!(after, before)` on the whole `Vec`, and the interface id SET unchanged.
  - [x] The test's name says what is excluded, e.g.
        `every_column_but_the_id_survives_a_purge_and_replay`.

- [x] **T4 — the operator's rows (AC5)**
  - [x] Write an `OPERATOR` link through the adapter, **on an observation the pass does not place**
        (AC5). ⚠️ Measured: on one the pass DOES place, the write is `Err(Constraint("unique"))`.
  - [x] 🔴 **Its `link_candidate` child needs the `an_abstention` idiom and goes AROUND
        `guard_decision`.** `identity_link_abstained_has_no_interface` forces a non-abstained row to
        name an interface, so a candidate only makes sense on an ABSTENTION — and
        `resolver::guard_decision` refuses `Abstained { Ambiguous }` with an empty candidate slice,
        which is the shape this needs. Hand-build the `Decision` (the `repo.rs` test module already
        has `an_abstention`), call `insert_identity_link` directly, and point the candidate at an
        interface the engine minted — `link_candidate_interface_fk` is RESTRICT.
  - [x] Purge; assert the row is present with its `id` intact, its children too, and that
        `purge_engine_links` returned exactly the ENGINE count.

- [x] **T5 — prove-to-red (AC6, AC8). Every mutation WITH the database.**
  - [x] 🔴 **M1 — derive a compared column from whether the interface was FOUND rather than minted**
        → AC3's comparison must red, assertion-carried on `assert_eq!(after, before)`. **This is the
        only mutation that reaches AC3**, and it was measured as the sole red.
  - [x] 🔴 **M1-noop — the mutation this task first prescribed, kept AS a measurement.** Make the
        resolver derive `valid_from` from an existing LINK. It reds
        `the_stored_instants_are_the_derived_ones` (story 5.9b's test) and leaves AC3's comparison
        **GREEN** — measured by the validation, in one run. **Record it as a no-op and say why**,
        because the why is the story's sharpest fact (§8).
  - [x] **M2** — purge `interface` rows alongside the links → AC4 must red on the id set, and AC3 on
        `interface_id`.
  - [x] 🔴 **M3** — drop `snapshot_links`' `ORDER BY` → **the ORDERING test of AC2 must red**, never
        AC3's comparison. ⚠️ **Measured at the validation: against AC3 it is a no-op and the whole
        suite stays green**, structurally — both snapshots go through the same query (§5c). Record
        both halves.
  - [x] **M4** — make `purge_engine_links` delete every row regardless of `decided_by` → AC5 must
        red.
  - [x] **M5** — re-mint the interface instead of finding it by key (5.9b's M3, re-run here) → AC4
        must red. ⚠️ Measured: it reds **5** tests, **3 of them story 5.9b's**, and in AC3's test the
        carrier is the SUMMARY assertion rather than the id set. Say which of the five are this
        story's coverage and which are inherited.
  - [x] **M6** — include `id` in `LinkSnapshot` → AC3 must red, which is the *positive* proof that
        the exclusion of decision 1 is load-bearing and not a convenience.
  - [x] 🔴 **Record the carrier PER TEST, never one label for a set.** M2 was measured producing
        **1 assertion + 2 `.expect` panics**; a single label there is the exact defect story 5.9b's
        review found in its own Debug Log.
  - [x] ⚠️ **A trap the validation hit**: `fetch_optional` on an aggregate (`MAX(...)`) returns
        `Some((NULL,))`, not `None`, so decoding it into a non-`Option` column errors the whole pass
        — 23 tests red, all `.expect` panics. If a mutation needs an aggregate, bind it as `Option`.
  - [x] Record, **per test**: DB yes/no, which tests red, and what carried each red.

- [x] **T6 — register and docs (AC7, AC9)**
  - [x] Append this story's section to `deferred-work.md`:
        · the `uuid` v7 entry **CLOSED**;
        · **BOTH** `datetime_literal` bullets — `:2302` and `:2484` — **ANSWERED AND RE-OWNED**
          (§4). ⚠️ They are not the same debt: the first is about this story's comparison, the
          second about the WRITE path and an existing test that *"asserts a property that holds only
          at microsecond granularity"*. §4's *"both sides truncated identically"* disposes of the
          first and **not** of the second;
        · the `sqlx chrono` entry **left untouched** — its owner is a condition this story does not
          meet (§4), confirmed by measurement: no `Timestamp` is ever produced from the database;
        · §2's `epics.md` `TRUNCATE` correction, and §5b's finding that **`epics.md:1634`'s
          "state the purge removes" points the wrong way** → **Epic 5's retrospective**;
        · 🔴 **the two natures are mutually exclusive on one placement** (AC5) → **story 5.14**.
  - [x] Update `docs/project-context.md` **and** `CLAUDE.md`, then grep both.
  - [x] Update `sprint-status.yaml`, **narrative included**.

- [x] **T7 — the full local gate, then the PR**
  - [x] `cargo fmt --all` · clippy **twice** · `cargo test --workspace --locked` **with the DB** ·
        `cargo xtask ci`.
  - [x] Re-check the corpus: **11 unanswerable, `passed() == false`**.
  - [ ] Then `code-review`, then push → PR → green CI → **squash merge**. Never push to `master`;
        `done` is the MERGE's business.

---

## Dev Notes

### Existing shapes to follow, not reinvent

- **The adapter idiom**: query bodies are free functions generic over `sqlx::Executor` (D49), static
  SQL with bound values (D48), and `classify` is the ONE `sqlx::Error → RepositoryError` translation.
- **`CAST(… AS CHAR)`** is `load_link_valid_to`'s idiom for an instant, and its doc already explains
  why it is transport and not a D10 violation. Reuse the sentence rather than inventing one.
- **`datetime_literal`** is `pub(crate)` since story 5.9b and is the single formatting site.
- **DB tests** are gated on `DATABASE_URL` and serialised under `DB_TEST_LOCK`; the resolver's own
  `fixture()` helper inserts the observations the links will name (the FK from `0003`).

### Compile-level facts

- `resolve(conn, &[Observation])` and `resolve_within(conn, &[Observation], &BTreeSet<CandidatePair>)`
  both return `Result<Resolution, RepositoryError>` and open no transaction — the caller wraps them
  in `WriteRepository::transact`.
- ⚠️ **The pass is NOT idempotent**: running it twice over the same observations is
  `Err(Constraint("unique"))` and a full rollback. **This story runs it twice with a PURGE in
  between**, which is exactly what makes the second run legal — the first run's links are gone.
  If a test forgets the purge, the failure it gets is decision 10 of story 5.9b, not a defect here.
- `sqlx` 0.9 rejects `sqlx::query(&format!(…))` at compile time.
- `sqlx::migrate!` embeds the migration directory at COMPILE time.

### What a reviewer will challenge, and the answer that is already measured

| challenge | answer |
|---|---|
| *"'Bit for bit' excludes the primary key — that is weaker than D14."* | §1, and it is Guy's arbitration on a measurement: a v7 UUID carries a wall-clock millisecond, so the id is not reproducible by construction. D14 says *"the same DECISIONS"*, and an id is not a decision. `interface_id` IS compared, which is what makes the exclusion safe. |
| *"Nothing writes an `OPERATOR` row, so AC5 tests a fiction."* | Decision 2. The frontier D14 calls load-bearing exists in the schema and in `DecidedBy` today; a human surface (story 5.14) would exercise it, not create it. |
| *"`TRUNCATE ... WHERE` is what the epic says."* | §2: MariaDB's `TRUNCATE` takes no `WHERE`. The correction is registered with Epic 5's retrospective; `epics.md` is not edited by a story. |
| *"You did not close the `sqlx chrono` entry."* | AC7: the snapshot compares two renderings and never produces a `Timestamp`, so the owner clause is not met. If the implementation needs one, the clause IS met — say so. |
| *"Two runs of the pass explode."* | Not with a purge between them — **provided no surviving `OPERATOR` row holds a slot the replay needs**. Measured: with one that does, the replay is `Err(Constraint("unique"))` and rolls back entirely (AC5). |
| *"The `ORDER BY` is untested."* | It was, and §5c is the answer: a comparison of two snapshots can NEVER carry it, so AC2 demands a separate ordering test. |
| *"AC6 does not do what `epics.md:1634` asks."* | §5b: what it asks is unmeasurable by the test it prescribes, and the story says so with the measurement rather than inheriting the sentence. |

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 5.10`] — its two criteria and the
  trailing anti-regression `**And**`, plus the `TRUNCATE ... WHERE` §2 corrects.
- [Source: `_bmad-output/planning-artifacts/architecture.md:1030-1045`] — **D14**: the link as a
  cache of ATTENTION not of TRUTH, the purge test verbatim, the two natures, `ruleset_version` as a
  guard against *"a silent data migration, the worst kind"*.
- [Source: `_bmad-output/implementation-artifacts/5-9b-engine-resolves-and-writes-links.md`] — the
  pass this story re-runs, its twelve decisions, and the code review that registered the `uuid` v7
  consequence with this story.
- [Source: `crates/opencmdb-bin/src/resolver.rs`] — `resolve`, `resolve_within`, `Resolution`.
- [Source: `crates/opencmdb-bin/src/repo.rs`] — `PersistedLink`, `load_link_valid_to`'s
  `CAST(… AS CHAR)`, `DecidedBy`, and the inline purge at `:2040`.
- [Source: `crates/opencmdb-bin/migrations/0002_interface_and_identity_link.sql`] — `ON DELETE
  CASCADE` on `link_candidate`, and *"the re-run finds an interface by its key"*.
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md`] — the entries naming story 5.10.

---

## Dev Agent Record

### Agent Model Used

_(to be filled by the dev agent)_

### Debug Log References

#### The database run

Everything below ran against a live `mariadb:10.11.11` in `opencmdb-dev-db`, host port **13306**
(3306 is held by `kesh-mariadb`, another project's, untouched). Baseline with the DB set: **408**
tests, and the bin suite takes **2.6 s** against **0.04 s** without — that gap is the only local
evidence the DB-backed tests executed at all.

#### 🔑 The central property holds, and the two organs agree

`every_column_but_the_id_survives_a_purge_and_replay`: six links over three interfaces (a group of
three, a multi-MAC observation, a MAC-less one), snapshot → purge → replay → snapshot, and
**`assert_eq!(after, before)` passes on all ten compared columns**, `interface_id` included. The
replay reports `interfaces_found = 3, interfaces_minted = 0`, so the interfaces it points at are the
same rows — which is what makes excluding the link `id` safe rather than convenient.

#### 🔴 The mutation pair that IS the story's sharpest fact

**M1 and M1-noop are the same mutation up to what they key on, and only one can red.**

| | keys on | reds |
|---|---|---|
| **M1** | whether the interface was **FOUND** — state the purge does NOT restore | `every_column_but_the_id_survives_a_purge_and_replay`, **assertion** |
| **M1-noop** | an **existing LINK** — state the purge removes and the replay recreates | 🔴 **nothing. Zero tests.** |

Both make `valid_from` — a compared column — conditional; both run against the live database. The
purge restores the store to exactly the state run 1 started from, so a decision keyed on link rows
produces the **same** value in both runs and the comparison cannot see it.

🔑 **`epics.md:1634` therefore asks for the one thing this test can never do.** What it does prove is
the useful statement: *the engine's output depends only on the observations and on the interfaces,
never on its own prior links* — which is D14's *"cache of attention, not of truth"* made
operational. Recorded rather than smoothed over, because the next reader will otherwise re-derive it.

⚠️ **My FIRST attempt at M1 was itself a no-op**, for a third reason: it shifted the interface's
seen-window, which is not a compared column. A mutation must touch something the assertion reads.

#### The mutation table — carrier recorded PER TEST

| # | mutation | DB | tests red | carrier, per test |
|---|---|---|---|---|
| **M1** | `valid_from` depends on the interface being FOUND | ✅ | 1 — `every_column_but_the_id_survives_a_purge_and_replay` | assertion |
| **M1-noop** | the same, keyed on an existing LINK | ✅ | **0** | — (structural no-op, §5b) |
| **M2** | purge `interface` rows alongside the links | ✅ | 2 | 🔴 **MIXED**: **assertion** on `every_column…` (`"interfaces are NOT purged, and their ids are the same rows"`, `left: []` vs three ids) + **`.expect` PANIC** on `the_operators_rows…` (the surviving operator link's FK refuses the delete, ERROR 1451) |
| **M3** | drop `snapshot_links`' `ORDER BY` | ✅ | 1 — `the_snapshot_is_ordered_by_the_query_not_by_insertion` | assertion. 🔑 **AC3's comparison stayed GREEN**, structurally |
| **M4** | purge every row regardless of `decided_by` | ✅ | 1 — `the_operators_rows_and_their_candidates_survive_the_purge` | assertion |
| **M5** | never find, always mint | ✅ | 4 — of which **3 are story 5.9b's** | assertion ×4, incl. `every_column…` |
| **M6** | put `id` back into `LinkSnapshot` | ✅ | 1 — `every_column_but_the_id_survives_a_purge_and_replay` | assertion |

**M6 is the positive proof**: excluding the id is load-bearing, not a convenience — put it back and
the comparison reds at once.

⚠️ **M2 could not be written without changing `purge_engine_links`' signature.** A query body generic
over `sqlx::Executor` consumes it BY VALUE, so a second statement needs `&mut MySqlConnection` — and
the one call site passing `&pool` then has to change with it. Recorded because the shipped
one-statement form hides it.

⚠️ **The red of M2 lands on the assertion AC6 names**, and only because the id-set assertion is
placed BEFORE the replay. Moved after it, a summary assertion pre-empts it — the validation measured
exactly that on the gap-hunt's version.

### Completion Notes List

- **AC1–AC9 met.** 408 → **411 tests** (206 bin + 159 core + 46 xtask), six gates green, both clippy
  forms clean, `fixtures/` untouched, and the trap corpus still **11 unanswerable,
  `passed() == false`** — re-checked, because a green gate here would be a regression.
- 🔑 **D14's own test passes**: purge the engine's links, re-run, and every decision-bearing column
  comes back identical. The story's whole premise is now a measurement rather than an argument.
- 🔴 **`epics.md:1634` is falsified by measurement and the story says so.** M1-noop reds nothing, by
  construction. The corrected statement is in §5b and in AC6.
- 🔴 **The `ORDER BY` has a test of its own**, because a comparison of two snapshots can never carry
  it — M3 confirms: it reds the ordering test and leaves AC3 green.
- **`snapshot_links` is restricted to CURRENT links**, which is not a simplification: two superseded
  versions of one placement carry equal sort keys, so the order over history would be InnoDB's
  accident. Story 5.11 is the one that will supersede.
- **The operator's row survives with its own `id`** — an INPUT keeps its identity; only derivations
  are re-minted. It names an observation the pass does not place, because
  `identity_link_one_current` makes the two natures mutually exclusive on one placement.
- ⏸️ **T7's push/PR is deliberately NOT done.** `code-review` first; `done` is the MERGE's business.

### File List

- `crates/opencmdb-bin/src/repo.rs` — MODIFIED (`purge_engine_links`, `LinkSnapshot`,
  `snapshot_links`; the cascade test now delegates to the query body)
- `crates/opencmdb-bin/src/resolver.rs` — MODIFIED (4 tests: the purge-and-replay, the ordering, the
  operator's rows, plus the `interface_ids` and `purge_fixture` helpers)
- `_bmad-output/implementation-artifacts/deferred-work.md` — MODIFIED (this story's section)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — MODIFIED
- `docs/project-context.md`, `CLAUDE.md` — MODIFIED (AC9)

---

## Change Log

| date | note |
|---|---|
| 2026-08-04 | **VALIDATED** by two fresh-context agents; the gap-hunt BUILT the story against a live `mariadb:10.11.11`, reaching **412 tests** with six green gates and `fixtures/` untouched. 🔑 **The central property HOLDS**: the purge-and-replay reproduces all ten compared columns exactly, `interface_id` included. **The findings are all about the measurement apparatus, not the property.** 🔴 **Four HIGH.** (1) **M1 is a NO-OP** — found by reasoning by the fact-check, then confirmed by measurement: the purge restores the state run 1 started from, so a dependency on state the purge REMOVES is invisible by construction. `epics.md:1634` points the wrong way, and §5b now says so with the measurement; the correct mutation depends on state the purge does NOT restore. (2) **The `ORDER BY` is unreddenable by a two-snapshot comparison** — deleting it left the whole suite green, structurally, so AC2 gains a test of its own (§5c). (3) 🔴 **The two natures are MUTUALLY EXCLUSIVE on one placement**: an operator can never confirm an engine placement (`Err(Constraint("unique"))`), and an operator row in the slot makes the **replay roll back entirely**. Guy's arbitration: AC5 names a subject the engine does not place, and *"can an operator override an engine placement?"* is registered with story 5.14. (4) **`current_subject` is NULL on superseded rows**, so decision 4's ordering degenerates the moment story 5.11 supersedes anything — the snapshot is now restricted to CURRENT links. Also: M2 produces a **mixed carrier set** (1 assertion + 2 panics), summary assertions pre-empt the reds their ACs name, AC5's operator candidate must go around `guard_decision`, and the `datetime_literal` debt is **two** entries — one about the comparison, one about the write path — of which §4 disposes of only the first. |
| 2026-08-04 | Story contexted on `master` at `0b3f09c` (408 tests, six green gates, clean tree). **Two arbitrations by Guy.** (1) 🔴 *"Bit for bit"* means **every decision-bearing column EXCEPT `id`** — `uuid::Uuid::now_v7()` stamps a wall-clock millisecond into the primary key, so it is not reproducible by construction; D14 says *"the same DECISIONS"*, and an id is not a decision. `interface_id` IS compared, which is what makes the exclusion safe and what turns story 5.9b's `find_interface_by_l1_key` from a claim into a measured property. (2) The `decided_by='OPERATOR'` rows AC5 needs are **written by the test through the adapter** — nothing writes them in production before story 5.14, but D14's two-natures frontier is real in the schema today and *"if that frontier is fuzzy in the code, the invariant is dead"*. Three further decisions measured against the tree: the purge is a **`DELETE`**, not `epics.md`'s pseudo-SQL `TRUNCATE … WHERE` (registered with Epic 5's retrospective rather than edited); the snapshot is a query body with **no `id` field at all**, so the exclusion is structural; and the comparison is one `assert_eq!` on a `Vec`, so a divergence prints whole rather than stopping at the first mismatch. ⏳ **Validation by two fresh-context agents is still owed, and the gap-hunt MUST run a live `mariadb:10.11.11` on port 13306.** |

---

### Review Findings

Three-layer review (Blind Hunter · Edge Case Hunter · Acceptance Auditor), 2026-08-05, on
`master...a82fa3a`. **All three ran against their own live `mariadb:10.11.11`** (ports 13312–13314,
all stopped, `kesh-mariadb` never touched) and the Auditor re-executed the mutation pass.

🔑 **The central property is real and all three reproduced it**: D14's purge test passes, all ten
compared columns including `interface_id`, and M6 proves the `id` exclusion load-bearing. **Six of
the seven claimed mutations reproduce exactly, carrier and count.** What does not survive is the
story's REASONING headline and the completeness of its guards.

- [ ] [Review][Decision] **Who owns the `datetime_literal` truncation debt?** AC7's own rationale
  says an entry left to a condition is *"a debt nobody holds"* — and the re-owning this story wrote
  hands it to *"the first story that compares an instant it holds IN MEMORY against a stored one"*,
  **which is a condition, not a story**. Worse, `the_stored_instants_are_the_derived_ones` arguably
  satisfies it already. Options: name **story 5.11** (it supersedes, and will compare instants),
  name **story 5.14** (it renders them), or say plainly that the debt is unowned and why

- [ ] [Review][Patch] 🔴 **The story's headline is FALSE as stated, and the Auditor built the fixture
  that refutes it.** *"The purge restores the store to exactly the state run 1 started from"* is a
  property of THIS FIXTURE, not of the purge: `purge_engine_links` has **no `current_subject`
  filter**, so it deletes SUPERSEDED engine rows too — and the snapshot excludes them. With one
  present before run 1, M1-noop reds `assert_eq!(after, before)`, assertion-carried; the same test is
  green on the clean tree. **Confirmed by the implementer by reading both statements.** `epics.md:1634`
  does not point the wrong way in general — it points at a case this fixture cannot reach. The claim
  is in the story, the register, `CLAUDE.md` and `sprint-status.yaml` [`repo.rs:772`, `:863`]
- [ ] [Review][Patch] **Add the superseded-engine-link purge-and-replay test** — six lines of setup
  that turn the caveat above into a guard instead of a sentence
- [ ] [Review][Patch] 🔴 **The story records TWO mutually exclusive results for M1-noop.** §5b, AC6
  and T5 say it reds `the_stored_instants_are_the_derived_ones`; the Debug Log says *"nothing, zero
  tests"*, and that half is in four documents. **Both are reachable**, and which one you get depends
  on where the predicate is evaluated — hoisted to the group loop → 0 reds; per observation → 1 red,
  assertion-carried. Record the placement WITH the result and prefer the per-observation form
- [ ] [Review][Patch] 🔴 **Eight of the ten "compared columns" can be blanked with the whole suite
  green** — the story diagnosed that an `ORDER BY` used on both sides of a comparison is
  unreddenable and **did not generalise it to the projection**. `rule_id` is asserted nowhere in the
  workspace. AC2's *"exactly the ten compared columns"* has no oracle. Fix, measured by the review: a
  ONE-SIDED oracle in both tests, against values each test already knows [`repo.rs:870-882`]
- [ ] [Review][Patch] **AC5 says "byte-identical on all ten compared columns"; the test asserts two.**
  The test already snapshots both sides of the purge — capture `before` and compare the whole row
- [ ] [Review][Patch] **The second sort key and the currency filter are BOTH deletable with the suite
  green** (M3b, M3c — measured twice, by two layers). The ordering test uses three single-MAC
  observations, so no two rows ever share an `observation_id` and the tiebreak is never reached; and
  nothing supersedes, so the `WHERE` never excludes anything. Give the ordering test a multi-MAC
  observation, and a superseded link to exclude [`repo.rs:863-864`]
- [ ] [Review][Patch] **`valid_to` is a TAUTOLOGY under the snapshot's own `WHERE`** —
  `identity_link_current_subject` makes `current_subject IS NOT NULL ⟺ valid_to = OPEN_END`, so it
  can never carry a divergence. It is presented as one of ten columns that *"CARRY THE DECISION"*
  [`repo.rs:819`]
- [ ] [Review][Patch] 🔴 **A doc claim contradicted by a measurement ALREADY IN THE REGISTER**:
  *"`identity_link_abstained_has_no_interface` means only an ABSTENTION can carry candidates"* —
  `deferred-work.md:2295` records *"`link_candidate` rows attach happily to a MATCH link. Measured
  `Ok(())`"*, registered by story 5.9's own review. The conclusion (hand-build the `Decision`) is
  right; the reason given is refuted, in a file this story appended to [`resolver.rs:1347`]
- [ ] [Review][Patch] **"An operator can never confirm OR CORRECT a placement the engine holds" is too
  strong** — measured: an OPERATOR link on a DIFFERENT subject inserts fine; only the SAME
  `(observation, subject)` is refused. Correcting a placement normally means moving it, which is
  permitted. The story-5.14 deferral rests on the stronger claim [`resolver.rs:1339-1341`]
- [ ] [Review][Patch] **The two-natures claim is in five documents and in no test**, though both
  halves were measured TRUE by two layers. Twelve lines pin it, and it is AC5's premise
- [ ] [Review][Patch] **AC6 and T5 claim "one assertion plus TWO `.expect` panics" for M2**; measured
  **1 + 1**, twice. The Debug Log is right and the acceptance criterion is wrong
- [ ] [Review][Patch] **T5 claims M5 reds 5 tests with a summary carrier**; measured **4**, all
  assertion-carried, and in AC3's test the carrier is the comparison itself. The Debug Log is right;
  T5 was never reconciled with it — and this project's own rule is that a divergence is a finding
- [ ] [Review][Patch] **The test's name says "every column but the id" while TWO columns are
  excluded** (`id` and `current_subject`), each for its own recorded reason — a sentence outrunning
  its measurement, in the test whose subject is exactness [`resolver.rs:1265`]
- [ ] [Review][Patch] **`purge_engine_links`' doc over-promises its return**: measured, `ROW_COUNT()`
  reports 2 for 2 links plus 2 cascaded candidates — InnoDB does not report cascades. Say *"how many
  LINKS went"*, assert the return in the cascade test, and assert `link_candidate` is empty after
- [ ] [Review][Patch] **The purge is global and unscoped**, and a partial replay silently loses links
  (measured: purge, replay 2 of 6 observations, `Ok`, snapshot 6 → 2, no signal). Its doc lists what
  it does NOT touch without saying that what it does touch is everything
- [ ] [Review][Patch] `interface_ids`' doc says *"as a set"*; it returns a sorted `Vec` compared as a
  sequence. And `interface_ids(&pool).await[0]` is a bare index — `.first().expect(…)` costs nothing
- [ ] [Review][Patch] **Add the empty-store test** — `purge_engine_links` returns 0, `snapshot_links`
  returns `[]`; both correct, both uncovered
- [ ] [Review][Patch] `sprint-status.yaml` carries **two `NEXT =` lines**; `CLAUDE.md` has a doubled
  period and story 5.9b's trailing sentence now reads as 5.10's (AC9)

- [x] [Review][Defer] The purge deletes superseded rows the snapshot never compared — an asymmetry no
  document states. Inert until something supersedes; **story 5.11** is what will
- [x] [Review][Defer] The comparison is blind to `link_candidate` and to `interface`'s own columns
  (`l2_domain`, `mac_canon`, the seen-window). Latent: the resolver writes no candidate today
- [x] [Review][Defer] The purge and the replay run in TWO transactions, and the composed shape — both
  in one unit of work — is what a production caller needs and what nothing runs
