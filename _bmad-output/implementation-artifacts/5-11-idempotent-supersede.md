# Story 5.11: A second pass supersedes what changed and writes nothing for what did not

Status: ready-for-dev

<!-- ⏳ NOT YET VALIDATED. This project requires a validation pass by **two fresh-context agents**
     (fact-check + gap-hunt) before `dev-story` — Guy's decision at the Epic 4 retrospective, which
     overrides the story template's "validation is optional" banner.

     🔑 **THE GAP-HUNT MUST COMPILE THIS STORY AGAINST A LIVE DATABASE.** On stories 5.5 through
     5.9b every HIGH finding came from the agent that COMPILED the story and none from the
     fact-check. `DATABASE_URL` is unset here, every DB-backed test passes by `return`ing, and the
     suite reports the same counts either way — so a story about a WRITE PATH is invisible without
     one. §9 has the `docker run`; host port **13306**, never 3306 (`kesh-mariadb` on 3306 belongs
     to another project and must never be touched).

     🔑 **ASK THE GAP-HUNT EXPLICITLY:** *"does each prescribed mutation actually red, and WHAT
     carries the red — an assertion, an `expect` panic, or the compiler?"* Story 5.9b's review found
     the implementer's carrier claim FALSE because a MIXED set had been collapsed to one label.
     Report the carrier **per test**, never over the whole output.

     ⚠️ **A mutation must preserve the ARITY of a SQL statement's bind parameters.** Removing a
     placeholder without its `.bind` desynchronises the MySQL protocol and HANGS the suite —
     measured at 2 h 48 min at 0 % CPU while holding `DB_TEST_LOCK`. Run every mutation under a
     timeout.

     ⚠️ **Commit before the mutation pass.** The driver's first act is `git checkout -- crates/`, so
     an UNCOMMITTED test is destroyed before the pass runs and comes back "target NOT RED" — the
     green being the test's ABSENCE, not its weakness. This has bitten three times.

     🔴 **Four things in §1–§4 are Guy's arbitrations, taken at contexting with the measurement in
     hand. They are not open questions. Re-opening one is a finding only if a MEASUREMENT refutes
     its premise** — which is exactly what happened to story 5.10's headline, so the door is not
     closed, only guarded. -->

## Story

As the operator whose scanner runs on a timer,
I want a second pass over observations it has already seen to change nothing,
So that a cycle that learned nothing writes nothing, and a cycle that learned something writes
exactly that (NFR6's idempotence clause).

**This story changes the resolver's WRITE PATH.** Story 5.9b's pass appends: `insert_identity_link`
inserts unconditionally and `identity_link_one_current` refuses the second current row for one
`(observation_id, current_subject)`. Running the pass twice over the same observations is therefore
`Err(Constraint("unique"))` and a **full rollback** — 0 interfaces, 0 links. This story replaces the
blind append with **read the current version, compare, then supersede or do nothing**, which is
`0002_interface_and_identity_link.sql`'s own header calling it *"story 5.11's 'no new version for an
unchanged decision'"*.

**It is the FIRST story in this project that supersedes anything.** Three registered debts land here
for that reason and no other, and §3, §4 and §6 dispose of them.

**What this story does NOT do**, so the boundary is explicit rather than discovered at review:

- it does **not** fuzz arrival order. 🔴 **The story was SPLIT at contexting (Guy's arbitration,
  §1): `5.11b` carries `epics.md`'s AC1 and AC3** — the seeded fuzz — and Epic 5 goes from 17
  stories to **18**. `epics.md` is NOT edited; the split is registered with Epic 5's retrospective;
- it does **not** wire the resolver into `main.rs`. Still no production caller, by story 5.9b's
  decision 3, still owned by story 5.14;
- it does **not** implement an `l2-*` rule, so the committed trap corpus stays **RED with 11
  unanswerable and `passed() == false`**. **If it turns green, that is a FINDING**;
- it does **not** answer *"may an operator override the engine?"*. That is story 5.14's, registered
  by 5.10. This story PINS today's behaviour instead (§5): an operator-held slot makes the engine's
  pass fail exactly as it does now, and a test says so, so the answer cannot be given by accident;
- it does **not** touch `identity::l1`, `identity::blocking` or `identity::cascade`. The engine's
  decisions are unchanged; only what the resolver does with them changes. **A change there is a
  FINDING.**

**Nothing under `fixtures/` moves.** No artefact bytes, no `MANIFEST.toml`, no re-hash.

---

## What this story inherits, measured rather than assumed

### 1. 🔴 The story was SPLIT at contexting — Guy's arbitration

`epics.md:1636` gives story 5.11 three acceptance criteria that are two different deliverables:

| `epics.md` | nature | goes to |
|---|---|---|
| AC1 — fuzzed arrival order reproduces the in-order run | a MEASUREMENT of what already holds | **5.11b** |
| AC2 — a second pass is idempotent, no new version for an unchanged decision | new PRODUCTION code | **5.11 (this story)** |
| AC3 — the fuzzing is seeded and the seed recorded | belongs to AC1 | **5.11b** |

The measurement behind the split: **the pass is already independent of arrival order by
construction**, and nothing in this story changes that. `join` returns a
`BTreeMap<(L2DomainId, MacAddr), BTreeSet<ObsId>>`; `candidates` returns a `BTreeSet<CandidatePair>`;
`placement_decision` takes the smallest other `ObsId` out of a `BTreeSet`; `seen_window` is a
`min`/`max` fold. Not one of them reads the slice's order. 5.11b's job is to make that FALSIFIABLE,
and it inherits a trap of its own that this story does not have to solve: two passes from an EMPTY
store mint different `interface.id` values (v7 UUIDs), so `snapshot_links` **cannot compare
`interface_id` literally between two arrival orders** — the same shape story 5.10 hit on `id`.

Epic 5: **17 → 18 stories**, 13 done. `epics.md` is verify-only here; the correction is registered
with Epic 5's retrospective beside 5.10's `TRUNCATE ... WHERE` and `epics.md:1634`.

### 2. 🔴 The EVIDENCE is part of the decision — Guy's arbitration

The question this settles: *"no new version for an UNCHANGED decision"* — what is the decision?

The measurement that makes it load-bearing is in the engine already. `decide_singleton`
[`l1.rs:345`] and `decide_pair` both produce the rule **`l1-exact-mac`**; only the evidence differs:

```
run 1 over {o1}          → o1's link: rule l1-exact-mac, evidence [o1]
run 2 over {o1, o2}      → o1's link: rule l1-exact-mac, evidence [o2, o1]
```

Same outcome, same rule, same interface. **If evidence is not part of the decision, o1's link is
"unchanged" and goes on asserting a justification that is FALSE** — and FR16 renders it. Guy's
arbitration: **evidence supersedes.**

The cost is real and is stated rather than discovered: every observation joining an interface
supersedes the existing links of that group whose witness it becomes — **O(group size) writes per
pass** in the worst case. AC6 measures it at the reference scale rather than leaving it a worry.

The comparison set follows from the arbitration and from what the engine can actually change:

| column | in the comparison? | why |
|---|---|---|
| `interface_id` | **yes** | the placement itself |
| `outcome` | **yes** | `match` / `no_match` / `abstained` |
| `rule_id` | **yes** | which rule settled it |
| `abstention_cause` | **yes** | why it did not |
| `evidence` | **yes** | §2, Guy's arbitration |
| `ruleset_version` | **yes** | D14 — a ruleset change is a decision change |
| `observation_id` | no | it is half the lookup key |
| `current_subject` | no | a FUNCTION of `interface_id` on a current row, held there by `identity_link_current_subject` |
| `valid_to` | no | the sentinel on every current row, by that same constraint |
| `decided_by` | no | the read is filtered to `ENGINE` (§5) |
| `valid_from` | no | the observation's own `observed_at`, and an observation is immutable — see §6, which is where that gets interesting |
| `id` | no | a v7 UUID; story 5.10 settled that a row identifier is not a decision |

### 3. 🔴 EVERY engine supersede is ZERO-LENGTH, and the constraint is relaxed to admit it

This is the story's central schema finding, and it was **measured at contexting** against a live
`mariadb:10.11.11`, not reasoned.

An SCD2 supersede is *"stamp the old row's `valid_to` and append"*. The engine may not read the
clock — `architecture.md:3364`, and story 5.10's replay test is what HOLDS it: a clock-derived
instant would make the replay produce a different one and red the comparison. So `closed_at` must
come from the data.

**There is no data-derived instant strictly greater than the old version's `valid_from`.** Both
versions are versions of ONE observation's placement, and `valid_from` is that observation's own
`observed_at` — immutable, and pinned by an existing test,
`the_stored_instants_are_the_derived_ones` [`resolver.rs:806-810`], on a PAIRED link. So the two
versions necessarily share a `valid_from`, and closing the old one at the new one's `valid_from`
means closing it at its own.

Measured, on `identity_link` as `0002` ships it:

```
M-A  UPDATE identity_link SET valid_to = <its own valid_from>, current_subject = NULL
     → ERROR 4025 (23000): CONSTRAINT `identity_link_interval` failed
```

`0002:82-83` refuses it deliberately: *"A version covers a half-open interval, so it can never be
zero-length or inverted."* That comment was written before anything superseded.

Also measured, and worth knowing before writing the write path:

```
M-B  close at a STRICTLY LATER instant                                    → accepted
M-C  the new version then OPENS at the same valid_from as the closed one  → accepted
     the two versions OVERLAP on [t1, t2) and NOTHING refuses it
```

**The schema constrains each row's own interval and does not constrain the CHAIN at all.** A
correct chain is the writer's business, not the DDL's — say so rather than trusting it.

🔑 **Guy's arbitration: relax the constraint.** A closed row may be zero-length; a current row may
not. `0004` ships:

```sql
CONSTRAINT identity_link_interval CHECK (
  (valid_to =  '9999-12-31 23:59:59.999999' AND valid_from <  valid_to)
  OR (valid_to <> '9999-12-31 23:59:59.999999' AND valid_from <= valid_to)
)
```

Measured on that exact form:

| | | |
|---|---|---|
| N-A | close a version at its own `valid_from` | **accepted** (was `ERROR 4025`) |
| N-B | an INVERTED closed interval | still `ERROR 4025` |
| N-C | a CURRENT row that would be zero-length (`valid_from` = the sentinel) | still `ERROR 4025` |

The reading, and it must go in the migration's comment because it is not obvious: **the first
belief never held over any interval the data can distinguish.** The engine's link history is
ordered by insertion, not by time, because the engine dates a link by the OBSERVATION and not by
when it came to believe it. That is a property of the model, and pretending otherwise would take
either a clock (forbidden) or an invented microsecond (a duration that never happened).

⚗️ **Registered rather than done:** dating a version by the instant that CAUSED it — the maximum
`observed_at` over the new evidence — would give real intervals in the ordinary case and degenerate
only when the newcomer is OLDER than the incumbent. It is not taken here because it changes what
`valid_from` means for every paired link, reds `the_stored_instants_are_the_derived_ones`, and
changes the content of story 5.10's snapshots. **Owner: the first story that needs a link's history
to be readable as a chronology** — story 5.14 is the candidate, and it must be NAMED there rather
than left as a condition.

### 4. 🔴 The replay owes history NOTHING — Guy's arbitration

Story 5.10's review left this at 5.11's name: `purge_engine_links` has no `current_subject` filter,
so it deletes **superseded** engine rows, while `snapshot_links` only ever compared **current** ones.
Inert until now because nothing superseded. This story is what makes it real.

Guy's arbitration: **the purge is an assumed reset.** A link is *"a cache of attention, not of
truth"*; what the engine believed yesterday is not a truth to preserve, and a purge-and-replay
rebuilds the current state only. `architecture.md:1016`'s *"a bad link is UNLINKED, never erased"* is
about an OPERATOR's correction of a live belief, not about the engine's own scratch history.

Two consequences the story must MEASURE rather than assert (AC5):

- after a supersede, a purge-and-replay leaves **fewer rows in the table than before it**, and the
  snapshots still compare equal — which is exactly why 5.10's comparison could not see this;
- `purge_engine_links`' doc must SAY it deletes history, in the same voice as its other four
  warnings. A doc that is silent here would be the *"claim outrunning its measurement"* shape six
  consecutive reviews have caught.

### 5. What the engine must NOT touch: an OPERATOR's slot

Story 5.10 measured that the two natures are **mutually exclusive on one placement**:
`identity_link_one_current` is `(observation_id, current_subject)`, so an operator row in a slot the
pass needs makes the engine's insert `Err(Constraint("unique"))` and the **whole pass roll back**.
*"May an operator override the engine?"* is registered with **story 5.14** and this story does not
answer it.

The trap: a compare-then-supersede path that reads the current row **without filtering on
`decided_by`** would find the operator's row, see a different decision, and **supersede a human's
assertion** — the engine silently overwriting a person, which is the one thing this product exists
not to do.

**The read is filtered to `decided_by = 'ENGINE'`.** The operator case then falls through to the
INSERT and fails exactly as it does today. That is not an accident to be re-derived later: AC4 pins
it with a test, so a future story that changes it has to change a test that says what it is doing.

### 6. The `datetime_literal` debt's open half is 5.11's, and it may be UNREACHABLE here

Story 5.10 split that debt. The closed half is a property of a pure function and is now
`repo::tests::datetime_literal_truncates_below_the_microsecond` (M10/M11 red it). The open half was
re-owned here with a substantive reason: *"5.11 supersedes, so it is the first story that holds TWO
instants for one placement and must decide whether they denote the same thing."*

🔴 **That reason is weaker than it looks, and the story must resolve it rather than pass it on a
fourth time.** §3 establishes that both versions of a placement carry the SAME `valid_from`, from the
same immutable observation — so the comparison set in §2 excludes `valid_from`, and including it
would be a guard that can never differ, which is the *"asserts nothing"* defect this project keeps
finding.

**AC7 is therefore a disposal, not an implementation.** Measure whether any caller in the workspace
now compares an instant it HOLDS against one it STORED. If one does, the debt discharges here with
its test. If none does, **close the entry with that measurement** and name what would have to change
for the risk to become real (§3's registered alternative). ⚠️ **A third re-own to a CONDITION is a
FINDING** — the register's own AC7 calls that *"a debt nobody holds"*, and this entry has now
circled for three stories.

### 7. The tree this story extends, measured on `63e452d`

- **`crates/opencmdb-bin/src/resolver.rs`** — 1860 lines, the pass. `resolve` → `resolve_within` →
  per group: `find_interface_by_l1_key` / `insert_interface` / `widen_interface_seen_window`, then
  `placement_decision` → `write_link` → `insert_identity_link`. The tail loop writes one abstention
  per unplaced observation. **`write_link` [`resolver.rs:316`] is where this story lands.**
- **`crates/opencmdb-bin/src/repo.rs`** — 2307 lines. Already present and NOT to be reinvented:
  `close_identity_link` [`:545`] with its three measured refusals (only a current row closes;
  closing nothing is `NotFound`; `closed_at` may not be the sentinel), `snapshot_links` [`:861`],
  `purge_engine_links` [`:782`], `current_subject_of` [`:517`], `datetime_literal` [`:447`],
  `OPEN_END`, `ABSTAINED_SUBJECT`, `open_end()`.
- **`load_current_links_for_observation` [`:646`]** returns `Vec<PersistedLink>` for an observation —
  plural, current only, ordered by `current_subject`. It is CLOSE to what the compare needs and is
  **not** it: it does not carry `decided_by` as a filter, it returns every subject rather than the
  one being written, and `PersistedLink` carries `id` (which the compare needs) but not
  `valid_from`. Decide deliberately between extending it and adding a sibling; do not quietly widen
  a function five tests depend on.
- **`Resolution` [`resolver.rs:106`]** — the counts a test can read back out of the database. It
  gains two fields (AC2), and the doc's *"every field is something a test can also read back"* must
  stay TRUE of the new ones.
- **Migrations** — `0001_initial.sql`, `0002_interface_and_identity_link.sql`,
  `0003_resolver_guards.sql`. This story adds **`0004`** and adds no table and no column.
- **`master` is at 416 tests** (six gates green, `views-hash` STALE and exiting 0 by design).

### 8. 🔴 A green suite says NOTHING here, and this story is nothing but database

`DATABASE_URL` is unset locally. Every DB-backed test begins with a `let Some(pool) = fixture(…)
else { return; }` and PASSES by returning. **The suite reports the same test count either way.** A
story that changes a write path and is validated without a database has been validated by nothing.

```
docker run -d --rm --name opencmdb-5-11 -p 13306:3306 \
  -e MARIADB_ROOT_PASSWORD=<choose> -e MARIADB_DATABASE=opencmdb mariadb:10.11.11
export DATABASE_URL='mysql://root:<choose>@127.0.0.1:13306/opencmdb'
```

⚠️ **Port 13306, never 3306.** `kesh-mariadb` holds 3306 and belongs to another project; story 5.9's
validation caught that before it could migrate someone else's database. Tests serialise on
`crate::DB_TEST_LOCK`.

### 9. Gates, and the shapes that cost time

- `cargo xtask ci` — six gates. **`ddl-collation` reads every migration**, so `0004` must carry
  explicit binary collations on anything holding letters (it alters a CHECK and adds no column, so
  the gate should be a no-op — **verify, do not assume**).
- `cargo clippy --workspace -- -D warnings` **and** `cargo clippy --workspace --all-targets -- -D
  warnings`. Both, always: the first is what CI runs, the second catches test-code lints. Epic 3's
  retrospective is the record of that gap.
- `#![deny(missing_docs)]` is ON for `opencmdb-bin`. Every new `pub` item, **field and variant
  included**, carries a `///` — and a doc comment that is FALSE is a defect, so prefer the weaker
  true sentence.
- The `file-size` gate: `resolver.rs` and `repo.rs` are the two largest files in the workspace but
  the ceiling counts only lines **before** the first `#[cfg(test)]`. Largest today is 1136.

---

## Decisions taken at contexting

1. **SPLIT — `5.11b` inserted, Epic 5 → 18 stories** (Guy, §1). This story is idempotence; 5.11b is
   the seeded arrival-order fuzz. `epics.md` NOT edited; registered with Epic 5's retrospective.
2. **The evidence is part of the decision** (Guy, §2). A changed witness supersedes. The cost is
   measured by AC6, not assumed away.
3. **`identity_link_interval` is relaxed for closed rows only** (Guy, §3), on the measurement that
   every engine supersede is zero-length by construction. `0004` carries the reasoning, not just the
   SQL.
4. **The replay owes history nothing** (Guy, §4). The purge stays global; its doc says so and AC5
   measures it.
5. **The compare is filtered to `decided_by = 'ENGINE'`** (§5). The operator case keeps today's
   behaviour and gains a test that pins it. The model question stays story 5.14's.
6. **No new table, no new column.** `0004` alters one CHECK.

---

## Acceptance Criteria

**AC1 — a second pass over unchanged observations writes NOTHING.**
Given a store populated by one pass, when the identical pass runs again inside its own transaction,
then it returns `Ok`, `identity_link` holds **exactly the same rows** — `id` included, so this is
strictly stronger than story 5.10's comparison — and `snapshot_links` is unchanged.
🔴 The `id` equality is what distinguishes "wrote nothing" from "rewrote the same thing", and it is
the only place in the project where a link's `id` is compared across runs.

**AC2 — the pass reports what it did.**
Given `Resolution`, when a pass completes, then it carries `links_superseded` and `links_unchanged`
alongside the existing counts, and every one of them is readable back out of the database by the
test — the field doc's standing promise. An idempotent pass reports `links_written = 0`,
`links_superseded = 0`, and `links_unchanged` equal to the number of current engine links.

**AC3 — a changed decision supersedes, and the old version stays readable.**
Given a store where `o1` sits alone on its interface with evidence `[o1]`, when a pass runs over
`{o1, o2}` sharing that MAC, then `o1`'s link is superseded: the old row is still there with
`current_subject IS NULL` and its evidence `[o1]` intact, a new current row carries evidence
`[o2, o1]`, and `o1` has **exactly one** current link.
**And** the old row's interval is **zero-length** — `valid_to = valid_from` — which `0004` admits and
`0002` refused (§3). A test names that equality; it is not left to the constraint.

**AC4 — the engine never supersedes an OPERATOR's row.**
Given an operator link current on `(observation, subject)`, when the engine's pass reaches that slot,
then the operator row is untouched — same `id`, same `valid_to`, still current — and the pass fails
`Constraint("unique")` and rolls back, exactly as it does today.
🔴 This pins behaviour rather than changing it (§5). A test that instead shows the engine
superseding the operator is not a passing test, it is the finding.

**AC5 — a purge-and-replay after a supersede loses history, and that is measured.**
Given a store carrying one superseded and one current version of a placement, when the engine's
links are purged and the pass replayed, then `snapshot_links` compares **equal** while
`count_identity_links` is **strictly smaller** than before the purge.
**And** `purge_engine_links`' doc states that it deletes superseded rows, in the voice of its four
existing warnings.

**AC6 — the write amplification is a number, not a worry.**
Given the reference scale already used by `one_full_pass_at_the_reference_scale`, when a pass adds
one observation to an existing group, then the story records how many links were superseded and the
wall-clock, in the Debug Log. No refusal threshold is installed — *"a bound with no measured need"*
is the speculation the create-only-what-the-story-needs rule refuses.

**AC7 — the `datetime_literal` debt is DISPOSED of.**
Given the register's open half, when this story completes, then the entry is either discharged with
its test or **closed with the measurement showing no caller compares a held instant against a stored
one** (§6). ⚠️ A third re-own to a condition is a FINDING.

**AC8 — nothing else moves.**
The trap corpus stays **11 unanswerable, `passed() == false`**. `fixtures/` is untouched.
`identity::l1`, `identity::blocking` and `identity::cascade` are unchanged. `main.rs` gains no
caller. Six gates green, both clippy forms clean.

**AC9 — the doc twins say the same thing.**
`CLAUDE.md`, `docs/project-context.md`, `sprint-status.yaml` and this file agree on the story's
status, the test count and the split. Four of story 5.9's review defects were twins out of step, and
two of story 5.8's were the same — this AC exists because that keeps happening.

---

## Tasks / Subtasks

**T1 — `0004`, the relaxed interval.** (AC3)
Write `crates/opencmdb-bin/migrations/0004_*.sql` altering `identity_link_interval` to the form in
§3, with the reasoning in the comment: why a closed row may be zero-length, why a current one may
not, and that the measured alternative was rejected for a named reason. Run `ddl-collation`.

**T2 — the read the compare needs.** (AC1, AC2, AC4)
In `repo.rs`, add the accessor that returns the CURRENT ENGINE link for one
`(observation_id, subject)` with its `id` and its decision-bearing columns. Decide explicitly
whether it extends `load_current_links_for_observation` or sits beside it (§7), and say which in the
doc. The `decided_by = 'ENGINE'` filter is load-bearing (§5) — write it, then measure that removing
it reds AC4.

**T3 — supersede or do nothing, in `write_link`.** (AC1, AC3)
Three branches: no current engine row → insert (today's path); one that MATCHES the six columns of
§2 → return without writing; one that DIFFERS → `close_identity_link` at the new version's
`valid_from`, then insert. The subject for the lookup is `current_subject_of` — the single
derivation site; do not spell the sentinel a second time.

**T4 — the counters.** (AC2)
`Resolution` gains `links_superseded` and `links_unchanged`, each documented, each readable back out
of the database by the test that asserts it.

**T5 — the tests.** (AC1, AC3, AC4, AC5)
At minimum:
`a_second_identical_pass_writes_nothing_at_all` (AC1 — compare `id`s, not just the snapshot);
`a_changed_witness_supersedes_and_the_old_version_survives` (AC3 — assert the zero-length interval
explicitly);
`the_engine_never_supersedes_an_operators_link` (AC4);
`a_purge_after_a_supersede_loses_history_and_still_replays` (AC5).
Every one takes `DB_TEST_LOCK` and returns early without `DATABASE_URL`, in the established shape.

**T6 — the reference-scale measurement.** (AC6)
Extend or mirror `one_full_pass_at_the_reference_scale`; record the numbers in the Debug Log.

**T7 — the debt.** (AC7)
Measure, then discharge or close. Write the measurement, not the conclusion.

**T8 — docs and register.** (AC5, AC8, AC9)
`purge_engine_links`' doc (AC5). `resolver.rs`'s module doc: the *"It is not idempotent, and that is
story 5.11's"* section is now FALSE and must be rewritten, not softened. `0002`'s header comment
about *"story 5.11's 'no new version for an unchanged decision'"* is now history — say what shipped.
`deferred-work.md`: the three entries this story disposes of, plus §3's registered alternative with
a NAMED owner. Then the twins (AC9).

**T9 — prove-to-red.** Commit first (the driver runs `git checkout -- crates/`). Suggested mutations,
each under a timeout, each with its carrier recorded **per test**:

| | mutation | prediction |
|---|---|---|
| M1 | drop the `decided_by = 'ENGINE'` filter in T2 | AC4's test reds — the engine supersedes the operator |
| M2 | drop `evidence` from the comparison set | AC3's test reds; ⚠️ predict whether AC1's does, and say so before running |
| M3 | compare all six columns but write the insert anyway on a match | AC1 reds on the `id` equality |
| M4 | close at `valid_from + 1µs` instead of at `valid_from` | AC3's zero-length assertion reds |
| M5 | revert `0004` to `0002`'s strict form | AC3 reds with `ERROR 4025`, **panic-carried, not assertion-carried** — say so |
| M6 | keep the old row current instead of closing it | `Constraint("unique")` on the append |
| M7 | restrict `purge_engine_links` to current rows | AC5's row-count assertion reds |

⚠️ M5's red is carried by a database error surfacing through an `.expect`, not by an assertion. That
is legitimate — a constraint IS the guard — but it must be **labelled**, because story 5.9b shipped a
false *"every red is assertion-carried"* headline in five documents for exactly this reason.

---

## Dev Notes

### Existing shapes to follow, not reinvent

- The test harness: `fixture()`, `pass()`, `try_pass()`, `within()`, `interface_count()`,
  `current_links()` [`resolver.rs:468-543`]. `fixture()` inserts the observations because
  `0003_resolver_guards.sql` gives `identity_link.observation_id` a foreign key.
- One pass runs inside ONE `transact` (D21). `resolve` opens no transaction — that is the caller's
  precondition, and story 5.9b's review measured 2 interfaces and 2 links committed under autocommit
  when a caller did not cooperate.
- `close_identity_link` already refuses the three things that were measured going wrong. Use it;
  do not write a second `UPDATE`.
- Deliberate redundancies that a DRY pass may NOT collapse: `expected_l1_conclusion` restating D13's
  text, `fixtures.rs`'s `expected()`, the per-module `scratch_dir`.

### Compile-level facts

- `sqlx` is built without its `chrono` feature: a `DATETIME(6)` has **no Rust type to decode into**.
  Instants come back as strings via `CAST(… AS CHAR)` and go out as `datetime_literal` renderings.
  This is transport, not the domain-value comparison D10 forbids.
- A query body generic over `sqlx::Executor` **cannot issue two statements** — the executor is
  consumed by value. A supersede is two statements, so the function that does both takes
  `&mut MySqlConnection`, as `resolve_within` already does. Story 5.10 measured this while writing a
  mutation it then could not apply.
- `serde_json` round-trips `evidence` as a `Vec<ObsId>`. Compare the DECODED vector, not the JSON
  string: two encodings of one vector would compare unequal and supersede forever.

### What a reviewer will challenge, and the answer that is already measured

- *"Why is a zero-length version acceptable?"* → §3, three measurements (M-A, and N-A/N-B/N-C on the
  relaxed form). The alternative is registered with a named owner, not dismissed.
- *"Isn't `snapshot_links` enough for AC1?"* → No. Both sides go through one query, so it cannot see
  a rewrite that reproduces the same values with new `id`s. That is the bilateral-oracle shape story
  5.10's review found in this very function.
- *"Does the operator case change?"* → No, and AC4 exists so that stays true by test rather than by
  intention.

### References

- `architecture.md:1016-1017` (unlinked, never erased), `:1036-1039` (D14's purge), `:1462-1468`
  (D21's sentinels), `:3364` (the engine never touches the clock), `:931` (D13's order).
- `epics.md:1636-1651` (this story as written), `:136` (NFR6), `prd.md:1224-1225` (NFR6).
- `0002_interface_and_identity_link.sql:44-54` (why `valid_to` is not in the uniqueness key),
  `:82-83` (the interval comment this story revises).
- `deferred-work.md` — the three entries owned here: idempotence, the `datetime_literal` open half,
  the purge/history asymmetry.

---

## Dev Agent Record

### Agent Model Used

_(to be filled by `dev-story`)_

### Debug Log References

_(to be filled — AC6's numbers go here, and the mutation table's measured column)_

### Completion Notes List

_(to be filled)_

### File List

_(to be filled)_

---

## Change Log

| Date | Change |
|---|---|
| 2026-08-05 | Created by `create-story`. 🔴 SPLIT at contexting: 5.11b inserted, Epic 5 → 18 stories. Four arbitrations by Guy (§1–§4), the third taken with a live-database measurement in hand: closing a version at its own `valid_from` is `ERROR 4025`, and the relaxed form admits it while still refusing an inverted closed interval and a zero-length current one. |
