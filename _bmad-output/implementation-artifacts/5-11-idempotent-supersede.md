# Story 5.11: A second pass supersedes what changed and writes nothing for what did not

Status: review

<!-- ✅ VALIDATED 2026-08-05 by two fresh-context agents (fact-check + gap-hunt), as this project
     requires. **The gap-hunt BUILT the story** against its own live `mariadb:10.11.11` on port
     13311, implementing T1–T4 and T5's four tests in full: **416 → 424 tests**, six gates green,
     both clippy forms clean, all seven prescribed mutations run under timeout plus three of its own.

     🔑 **THE STORY BUILDS. AC1, AC3, AC4 and AC5 are all reachable and all HOLD.** Every finding
     below is about a CLAIM outrunning its measurement, or a guard that measures nothing — the same
     family six consecutive reviews have caught. 14 findings applied.

     🔴 FIVE HIGH, and two of them are mutations that came back GREEN:
       • **M8 — this story silently retires Guy's own 5.9b arbitration.** Deleting
         `!abstained.insert(…)` leaves all 424 tests green: the new write path makes the second
         write "unchanged", so the dedup guard becomes dead code. **AC2c** forces the disposal.
       • **M9 — five of the six comparison columns are unreddenable through the database.** At L1
         the interface, outcome, rule, cause and ruleset version cannot differ within a group;
         evidence is the only reachable difference. **AC2b** makes the comparison a pure function
         with database-free tests, which is what turns five dead columns into five reds.
       • **§3's "EVERY engine supersede is zero-length" was FALSE as a universal.** `valid_from`
         comes from the IN-MEMORY observation and nothing reads `observation_record.observed_at`
         back, so a caller handing the same `obs_id` with a later instant produces a non-zero-length
         supersede. The `0004` relaxation is still right; its justification was too strong.
       • **The evidence is SORTED ascending** (`l1.rs:277-278`), so the prescribed `[o2, o1]` — in
         three places, including the headline test — REDS as written.
       • **M5 is not executable as prescribed**: sqlx checksums migrations, so editing `0004` in
         place gives `VersionMismatch(4)` on every DB test. It needs a `DROP DATABASE` first.

     ⚠️ Also applied: AC7's closure sentence was false (eight TEST sites do compare a held instant
     against a stored one; no PRODUCTION caller does), AC6 measured 0 or n−1 depending on the
     newcomer's `ObsId` and now names both, `current_subject_of` is PRIVATE and its signature costs
     a panic path, §4 overstated a guard story 5.10 already ships, and §9's file-size sentence was
     falsified by its own next clause.

     🔑 FOR THE NEXT STORY'S VALIDATION: the gap-hunt found 11 of the 14; the fact-check found the
     three that were pure citation defects. **Every HIGH came from the agent that COMPILED it** —
     the sixth consecutive story with that split. Keep asking it for the CARRIER per test.

     ⚠️ **A mutation must preserve the ARITY of a SQL statement's bind parameters.** Removing a
     placeholder without its `.bind` desynchronises the MySQL protocol and HANGS the suite —
     measured at 2 h 48 min at 0 % CPU while holding `DB_TEST_LOCK`. Run every mutation under a
     timeout.

     ⚠️ **Commit before the mutation pass.** The driver's first act is `git checkout -- crates/`, so
     an UNCOMMITTED test is destroyed before the pass runs and comes back "target NOT RED" — the
     green being the test's ABSENCE, not its weakness. This has bitten three times.

     🔑 **`DATABASE_URL` is unset in this workspace and every DB-backed test passes by `return`ing,
     so `dev-story` must run its own database too.** §9 has the `docker run`; host port **13306**,
     never 3306 (`kesh-mariadb` on 3306 belongs to another project and must never be touched).

     🔴 **Four things in §1–§4 are Guy's arbitrations, taken at contexting with the measurement in
     hand. They are not open questions. Re-opening one is a finding only if a MEASUREMENT refutes
     its premise** — which is what happened to §3's universal claim during this very validation, so
     the door is not closed, only guarded. -->

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

The measurement behind the split: **the pass's DECISION MACHINERY is already independent of arrival
order by construction**, and nothing in this story changes that. `join` returns a
`BTreeMap<(L2DomainId, MacAddr), BTreeSet<ObsId>>` [`l1.rs:171`]; `candidates` returns a
`BTreeSet<CandidatePair>` [`blocking.rs:171`]; `placement_decision` takes the smallest other `ObsId`
out of a `BTreeSet` [`resolver.rs:250-273`]; `seen_window` is a `min`/`max` fold
[`resolver.rs:297-304`]. Not one of those four reads the slice's order.

🔴 **The ENCLOSING function does, twice, and 5.11b inherits both.** Measured at this story's
validation, and written here rather than left for 5.11b to rediscover:

- `resolver.rs:168` — `by_id` is a `.collect()` into a `BTreeMap`, so it is **last-duplicate-wins**.
  A slice carrying one `obs_id` twice with DIFFERENT content resolves to whichever copy arrives
  last, and that copy's `observed_at` is what `write_link` stores as `valid_from` and what
  `seen_window` folds into the interface window. A fuzzed order over such a slice changes a
  **stored column**;
- the tail abstention loop [`resolver.rs:226`] iterates the raw slice, so abstention rows are
  INSERTED in arrival order. The row VALUES are invariant; only the mint order moves.

The input class is reachable rather than exotic — story 5.9b shipped `a_repeated_obs_id_writes_one_link`
for it — but that test passes the **same clone twice**, so `by_id`'s last-wins is invisible to all
416 tests. **Owner: story 5.11b**, named, not conditioned.

5.11b also inherits a trap of its own that this story does not have to solve: two passes from an
EMPTY store mint different `interface.id` values (v7 UUIDs), so `snapshot_links` **cannot compare
`interface_id` literally between two arrival orders** — the same shape story 5.10 hit on `id`.

Epic 5: **17 → 18 stories**, 13 done. `epics.md` is verify-only here; the correction is registered
with Epic 5's retrospective beside 5.10's `TRUNCATE ... WHERE` and `epics.md:1634`.

### 2. 🔴 The EVIDENCE is part of the decision — Guy's arbitration

The question this settles: *"no new version for an UNCHANGED decision"* — what is the decision?

The measurement that makes it load-bearing is in the engine already. Within one `join` group every
member shares the group's key, so `decide_singleton` [`l1.rs:345`] and `decide_pair` both produce the
rule **`l1-exact-mac`**; only the evidence differs:

```
run 1 over {o1}          → o1's link: rule l1-exact-mac, evidence [o1]
run 2 over {o1, o2}      → o1's link: rule l1-exact-mac, evidence [o1, o2]
```

⚠️ **The evidence is SORTED ASCENDING by `ObsId`, never witness-first.** `verdict_for_pair` does
`evidence.sort()` [`l1.rs:277-278`] deliberately — *"so the evidence of a pair does not depend on
which side was the left argument"* — and `l1.rs:721` already asserts it. A test written on
`[o2, o1]` REDS. This is stated because the first draft of this story wrote it the other way in
three places.

_(Outside a group the claim does not hold and is not needed: `verdict_for_pair` returns
`l1-distinct-mac`/`Disqualifying` for two MAC-carrying observations sharing no key [`l1.rs:266-271`]
and `Neutral` with empty evidence when either carries none [`l1.rs:258-263`]. `placement_decision`
only ever draws its witness from the same group.)_

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
| `valid_from` | no | the observation's own `observed_at` — see §3 and §6, which is where that gets interesting |
| `id` | no | a v7 UUID; story 5.10 settled that a row identifier is not a decision |

🔴 **FIVE of those six columns are unreddenable THROUGH THE DATABASE, and the story would ship a
guard that measures nothing if it stopped here.** Measured at validation: reducing the comparison to
`evidence` alone leaves **all 424 tests GREEN**. The reason is structural, not accidental — at L1 the
interface is a function of the observation's own key, every group member shares that key so the rule
is always `l1-exact-mac` and the outcome always `match`, `abstention_cause` is `NULL` on any placed
link, and `ruleset_version` is the constant `CURRENT_RULESET_VERSION`. **Evidence is the only
difference an L1 pass can produce.** This is story 5.9's M3 family again: a guard reachable only by
going around the code that feeds it.

**The remedy is prescribed, not deferred.** The comparison must be a **pure function** —
`same_decision(&PersistedLink-ish, …) -> bool` or equivalent — unit-tested directly, with **no
database**, one case per column. That is what turns five dead columns into five reds, and it costs
nothing: the function has no I/O. AC2b names it.

### 3. 🔴 EVERY engine supersede is ZERO-LENGTH, and the constraint is relaxed to admit it

This is the story's central schema finding, and it was **measured at contexting** against a live
`mariadb:10.11.11`, not reasoned.

An SCD2 supersede is *"stamp the old row's `valid_to` and append"*. The engine may not read the
clock — `architecture.md:3364`, and story 5.10's replay test is what HOLDS it: a clock-derived
instant would make the replay produce a different one and red the comparison. So `closed_at` must
come from the data.

**When the caller supplies each observation with a stable `observed_at`, there is no data-derived
instant strictly greater than the old version's `valid_from`.** Both versions are versions of ONE
observation's placement, and `valid_from` is that observation's own `observed_at`. So the two
versions share a `valid_from`, and closing the old one at the new one's `valid_from` means closing it
at its own.

🔴 **That stability is a CALLER'S DISCIPLINE, not a structural property, and the first draft of this
story asserted the stronger thing.** Measured at validation: `valid_from` comes from the **in-memory**
`Observation`, `0003`'s foreign key checks only that `observation_id` EXISTS, and **nothing in the
workspace ever reads `observation_record.observed_at` back as a value** — the only `SELECT` naming
that column uses it as an `ORDER BY` [`repo.rs:205`]. Hand the pass the same `obs_id` with a later
`observed_at` and a **non-zero-length** supersede is produced. `the_stored_instants_are_the_derived_ones`
[`resolver.rs:806-810`] pins *stored equals supplied within one pass*; it does not pin *stable across
passes*, and citing it for the latter is the claim-outrunning-measurement shape this project keeps
catching.

**`0004`'s comment must therefore say the weaker true thing**, because a false comment in DDL is a
defect by this project's own rule. The relaxation is not aimed at the wrong case — `<=` covers both —
but its justification may not overstate.

⚠️ **Measured in the other direction too, and undocumented until now:** an `observed_at` moving
BACKWARDS gives `Err(Constraint("check"))` and rolls the whole pass back — `0004`'s N-B firing
through the pass rather than through raw SQL. The relaxation is load-bearing at both ends, and T8
must say so.

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
belief never held over any interval the data can distinguish**, whenever the caller supplies stable
instants — which is the ordinary case and the only one the engine controls. The engine's link history is
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

⚠️ **The guard already exists and is already measured** — `a_superseded_engine_link_is_not_restored_by_the_replay`
[`resolver.rs:1537`] ships with story 5.10 and reds under M7. What this story adds is that the
supersede now comes from **the pass itself** rather than from a hand-built `close_identity_link`
call, which is a different claim and a smaller one. Say that, rather than presenting AC5 as
installing a guard that is already there.

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

**AC7 is therefore a disposal, not an implementation.** The measurement is already taken and its
SCOPE is what matters:

- **no PRODUCTION caller** compares a held instant against a stored one. Production only binds a
  rendering, or compares a rendering against the `OPEN_END` string CONSTANT [`repo.rs:521`, `:557`];
- **eight TEST sites do exactly that** — they compare a `CAST(… AS CHAR)` read against
  `datetime_literal(held)`: `resolver.rs:794`, `:795`, `:808`, `:1324`, `repo.rs:1281`, `:2119`, and
  the two the supersede tests will add.

So the entry is closable, **but only on the qualified sentence**. Closing it on an unqualified
*"no caller does this"* would be a fourth circling of one entry on a measurement that is false. Name
what would have to change for the risk to become real: §3's registered alternative, where a version
would be dated by something other than the observation's own instant.

⚠️ **A third re-own to a CONDITION is a FINDING** — the register's own AC7 calls that *"a debt nobody
holds"*, and this entry has now circled for three stories.

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
  **not** it: it does not filter on `decided_by`, and it returns every subject rather than the one
  being written. `PersistedLink` carries `id` and all six decision columns and **not** `valid_from`,
  which is exactly right — the compare needs the first and not the second. **Add a SIBLING reusing
  `PersistedLink` and `LinkRow`; no new type is needed** (measured at validation). Do not quietly
  widen a function **7 call sites in `repo.rs`'s tests depend on directly, plus 14 more through
  `current_links()` [`resolver.rs:538`]**.
- ⚠️ **`current_subject_of` [`:517`] is PRIVATE**, and its signature fights the call site: it takes
  the `valid_to` LITERAL as `&str`, so a resolver caller must write
  `current_subject_of(iface, &datetime_literal(open_end())).expect(…)` — a rendered instant compared
  against a constant, plus an `expect` on a branch that cannot be taken. **Prescribe a
  `pub(crate) fn subject_of(interface: Option<InterfaceId>) -> String` beside it** and have
  `current_subject_of` delegate, so the sentinel still has ONE derivation site and the resolver gets
  no panic path.
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
- The `file-size` gate counts only lines **before** the first `#[cfg(test)]`, so the files this story
  grows are further from the ceiling than their totals suggest: `repo.rs` is **929** code lines of
  2307 total, `resolver.rs` **405** of 1860. The gate's reported largest, **1136**, is
  `xtask/src/main.rs` — neither of the two this story touches.

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
reachable precisely BECAUSE nothing is written — these are the same rows, not re-minted ones.
_(It is not the project's only cross-run `id` comparison: `the_operators_rows_and_their_candidates_survive_the_purge`
[`resolver.rs:1730`] already asserts an OPERATOR link's `id` across a purge-and-replay. That one is
an INPUT keeping its identity; this one is a derivation that was not re-derived.)_

**AC2 — the pass reports what it did.**
Given `Resolution`, when a pass completes, then it carries `links_superseded` and `links_unchanged`
alongside the existing counts, and every one of them is readable back out of the database by the
test — the field doc's standing promise. An idempotent pass reports `links_written = 0`,
`links_superseded = 0`, and `links_unchanged` equal to the number of current engine links.

**AC2b — the comparison is a PURE FUNCTION with its own database-free tests.** (§2)
Given the six-column comparison, when it is written, then it is a pure function unit-tested directly
with **one case per column**, so that all six are reddenable. Measured at validation: through the
database alone, **five of the six are unreddenable** and dropping them leaves the whole suite green.
🔴 A comparison tested only through a pass is a guard that measures one column and claims six.

**AC2c — story 5.9b's abstention guard is DISPOSED of, not silently retired.** (mutation M8)
Given `resolve_within`'s `!abstained.insert(observation.obs_id)` — Guy's arbitration at 5.9b's code
review, installed after a measured `ABSTAINED_SUBJECT` collision that rolled a whole pass back — when
this story's write path lands, then the guard is measured: deleting it leaves the suite **GREEN**,
because the second write now finds the current row and reports it unchanged. Either the guard keeps a
test that reds on its removal, or it is retired with the decision RECORDED. 🔴 Leaving it as dead
code with no test and no sentence is the outcome this AC exists to forbid.

**AC3 — a changed decision supersedes, and the old version stays readable.**
Given a store where `o1` sits alone on its interface with evidence `[o1]`, when a pass runs over
`{o1, o2}` sharing that MAC, then `o1`'s link is superseded: the old row is still there with
`current_subject IS NULL` and its evidence `[o1]` intact, a new current row carries evidence
**`[o1, o2]` — sorted ascending by `ObsId`, never witness-first (§2)** — and `o1` has **exactly one**
current link.
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

**AC6 — the write amplification is a number, not a worry — and BOTH cases are measured.**
Given the reference scale already used by `one_full_pass_at_the_reference_scale`, when the Debug Log
is written, then it records the wall-clock and the `Resolution` counts for **four** runs, because a
single "add one observation" figure is ambiguous: the witness is the SMALLEST OTHER `ObsId`, so a
newcomer with a LARGER id supersedes **nothing** while a newcomer with the smallest id supersedes
**every** other member of its group.

| run | expected shape (measured at validation) |
|---|---|
| 300 observations, cold | `written 300, superseded 0, unchanged 0`, 44 850 pairs, ~165 ms |
| identical rerun | `written 0, superseded 0, unchanged 300`, ~82 ms |
| +1 observation into a singleton group | `written 2, superseded 1, unchanged 299`, ~73 ms |
| a group of 59 + a **smallest-id** newcomer | `written 60, **superseded 59**, unchanged 0`, ~21 ms |

The last row is what confirms §2's *"O(group size) in the worst case"*. No refusal threshold is
installed — *"a bound with no measured need"* is the speculation the create-only-what-the-story-needs
rule refuses.

**AC7 — the `datetime_literal` debt is DISPOSED of.**
Given the register's open half, when this story completes, then the entry is either discharged with
its test or **closed with the measurement showing no caller compares a held instant against a stored
one** (§6). ⚠️ A third re-own to a condition is a FINDING.

**AC8 — nothing else moves.**
The trap corpus stays **11 unanswerable, `passed() == false`**. `fixtures/` is untouched.
`identity::l1`, `identity::blocking` and `identity::cascade` are unchanged. `main.rs` gains no
caller. Six gates green, both clippy forms clean.

**AC10 — a slot the input no longer supports is CLOSED.** (added at the code review, Guy's
arbitration)
Given a store where an observation holds links on two L1 keys, when a pass runs over that
observation carrying only one of them, then the other slot is closed and `Resolution` reports it as
`links_vacated`.
🔴 **This story is what made the case silent.** The blind append failed LOUDLY on
`identity_link_one_current`; the compare routes around the key. The uniqueness key was doing the
detection, and taking its job means taking its duty. The orphan is also a reachable counterexample
to story 5.10's replay invariant, so a second test replays after the loss.

**AC11 — an instant that runs BACKWARDS is refused by name.** (added at the code review, Guy's
arbitration)
Given a version stored at `t1`, when the same observation is re-supplied at `t0 < t1` with a changed
decision, then the pass returns `RepositoryError::InstantRegressed` and rolls back whole.
🔴 Before the guard: fatal to the WHOLE batch on that branch, under an anonymous `Constraint("check")`
from the DDL naming no cause — and entirely SILENT on the branch where the decision had not changed,
because `same_decision` does not compare `valid_from`. One condition, two opposite answers.

**AC9 — the doc twins say the same thing.**
`CLAUDE.md`, `docs/project-context.md`, `sprint-status.yaml` and this file agree on the story's
status, the test count and the split. Four of story 5.9's review defects were twins out of step, and
two of story 5.8's were the same — this AC exists because that keeps happening.

---

## Tasks / Subtasks

**[x] T1 — `0004`, the relaxed interval.** (AC3)
Write `crates/opencmdb-bin/migrations/0004_*.sql` altering `identity_link_interval` to the form in
§3, with the reasoning in the comment: why a closed row may be zero-length, why a current one may
not, and that the measured alternative was rejected for a named reason. Run `ddl-collation`.

**[x] T2 — the read the compare needs.** (AC1, AC2, AC4)
In `repo.rs`, add a **sibling** of `load_current_links_for_observation` returning the CURRENT ENGINE
link for one `(observation_id, subject)`, reusing `PersistedLink` and `LinkRow` — no new type is
needed (§7). The `decided_by = 'ENGINE'` filter is load-bearing (§5); write it, then measure that
removing it reds AC4 **and** 5.10's `an_operator_cannot_take_a_slot_the_engine_holds`.
Also add `pub(crate) fn subject_of(interface) -> String` and have `current_subject_of` delegate, so
the sentinel keeps ONE derivation site and the resolver gets no `expect` on an unreachable branch
(§7).

**[x] T3 — supersede or do nothing, in `write_link`.** (AC1, AC2b, AC3)
Three branches: no current engine row → insert (today's path); one that MATCHES the six columns of
§2 → return without writing; one that DIFFERS → `close_identity_link` at the new version's
`valid_from`, then insert. Use `subject_of` for the lookup.
**The comparison itself is a pure function**, separate from the branch that calls it, so AC2b can
test all six columns without a database.

**[x] T4 — the counters.** (AC2)
`Resolution` gains `links_superseded` and `links_unchanged`, each documented, each readable back out
of the database by the test that asserts it.

**[x] T5 — the tests.** (AC1, AC2b, AC2c, AC3, AC4, AC5)
At minimum:
`a_second_identical_pass_writes_nothing_at_all` (AC1 — compare `id`s, not just the snapshot);
`a_changed_witness_supersedes_and_the_old_version_survives` (AC3 — assert the zero-length interval
explicitly, and the evidence as **`[o1, o2]` sorted**, not witness-first);
`the_engine_never_supersedes_an_operators_link` (AC4);
`a_purge_after_a_supersede_loses_history_and_still_replays` (AC5);
plus **database-free** unit tests of the comparison, one per column (AC2b).
Every DB-backed one takes `DB_TEST_LOCK` and returns early without `DATABASE_URL`, in the
established shape. Validation reached **416 → 424 tests** on this set.

**[x] T6 — the reference-scale measurement.** (AC6)
Extend or mirror `one_full_pass_at_the_reference_scale`; record the numbers in the Debug Log.

**[x] T7 — the debt.** (AC7)
Measure, then discharge or close. Write the measurement, not the conclusion.

**[x] T8 — docs and register.** (AC5, AC7, AC8, AC9)
`purge_engine_links`' doc (AC5). `resolver.rs`'s module doc [`:70`]: the *"It is not idempotent, and
that is story 5.11's"* section is now FALSE and must be rewritten, not softened. `0002`'s header
comment about *"story 5.11's 'no new version for an unchanged decision'"* is now history — say what
shipped. `0004`'s comment must carry the WEAKER true sentence (§3) and must record the
backwards-`observed_at` corollary: an instant moving backwards gives `Constraint("check")` and rolls
the pass back, so the relaxation is load-bearing at both ends.
`deferred-work.md`: the three entries this story disposes of, plus §3's registered alternative with a
NAMED owner, plus §1's two order-dependencies handed to **5.11b** by name.
⚠️ `deferred-work.md:2534` names story 5.10's test `every_column_but_the_id_survives_a_purge_and_replay`;
the shipped name is `every_decision_bearing_column_survives_a_purge_and_replay` [`resolver.rs:1278`].
Correct it while you are in the file. Then the twins (AC9).

**[x] T9 — prove-to-red.** Commit first (the driver runs `git checkout -- crates/`). Suggested mutations,
each under a timeout, each with its carrier recorded **per test**:

**Every row predicts its CARRIER, not only its colour.** Story 5.9b shipped a false *"every red is
assertion-carried"* headline in five documents because a mixed set was collapsed to one label; a
table that predicts only red/green invites the same mistake.

| | mutation | predicted result | predicted carrier |
|---|---|---|---|
| M1 | drop the `decided_by = 'ENGINE'` filter in T2 | RED — AC4's test **and 5.10's `an_operator_cannot_take_a_slot_the_engine_holds`** | assertion ×2 |
| M2 | drop `evidence` from the comparison set | RED — AC3 and AC5. ⚠️ **AC1 stays GREEN**; predict that before running | assertion ×2 |
| M3a | on a match, fall through to close+insert | RED — AC1, on the `id` equality | assertion |
| M3b | on a match, insert **without** closing | RED — AC1 and AC6 | **`.expect` panic** ×2 (`Constraint("unique")`) |
| M4 | close at `valid_from + 1 µs` instead of at `valid_from` | RED — AC3's zero-length assertion | assertion |
| M5 | revert `0004` to `0002`'s strict form | RED — AC3, AC5, both AC6 runs | **`.expect` panic** ×4 (`Constraint("check")`) |
| M6 | keep the old row current instead of closing | RED — same four | **`.expect` panic** ×4 (`Constraint("unique")`) |
| M7 | restrict `purge_engine_links` to current rows | RED — AC5 **and 5.10's `a_superseded_engine_link_is_not_restored_by_the_replay`** | assertion ×2 |
| M8 | drop 5.9b's `abstained.insert()` dedup | 🔴 **GREEN — that is the finding**, and AC2c is what it forces | — |
| M9 | compare only `evidence`, drop the other five columns | 🔴 **GREEN through the database** — AC2b's pure-function tests are what make it red | — |

🔴 **M5 is NOT executable by editing `0004` in place.** sqlx checksums applied migrations, so an
edited `0004` gives `migrate: VersionMismatch(4)` on **every** DB-backed test — a red that has nothing
to do with the guard M5 targets and that would read as *"M5 reds everything"*. **`DROP DATABASE
opencmdb; CREATE DATABASE opencmdb;` first.** Measured at validation; without it the pass records a
false result.

⚠️ The error a test actually SEES under M5 is `Constraint("check")`, not the raw `ERROR 4025` — the
domain's `classify` flattens it first. Write the assertion against the classified value.

⚠️ A red carried by a database error surfacing through an `.expect` is legitimate — a constraint IS
the guard — but it must be **labelled as such**, per test.

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

- *"Why is a zero-length version acceptable?"* → §3, four measurements (M-A, N-A/N-B/N-C on the
  relaxed form, and the backwards-`observed_at` `Constraint("check")`). The alternative is registered
  with a named owner, not dismissed. ⚠️ Do not restate the universal claim §3 retracts.
- *"Does the six-column comparison measure six columns?"* → Not through the database: five are
  unreddenable there and were measured green. AC2b's pure-function tests are the answer, and saying
  "six columns" without them would be the guard-that-asserts-nothing defect.
- *"Isn't `snapshot_links` enough for AC1?"* → No. Both sides go through one query, so it cannot see
  a rewrite that reproduces the same values with new `id`s. That is the bilateral-oracle shape story
  5.10's review found in this very function.
- *"Does the operator case change?"* → No, and AC4 exists so that stays true by test rather than by
  intention.

### References

- `architecture.md:1016-1017` (unlinked, never erased), `:1036-1039` (D14's purge), `:1462-1468`
  (D21's sentinels), `:3364` (the engine never touches the clock), `:931` (D13's order).
- `epics.md:1636-1652` (this story as written — AC3, the seeded-fuzz clause that goes to 5.11b, is
  at `:1652`), `:136` (NFR6), `prd.md:1224-1225` (NFR6).
- `0002_interface_and_identity_link.sql:48-54` (why `valid_to` is not in the uniqueness key),
  `:82-83` (the interval comment this story revises).
- `deferred-work.md` — the three entries owned here: idempotence, the `datetime_literal` open half,
  the purge/history asymmetry.

---

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context), `dev-story`, 2026-08-05. Live `mariadb:10.11.11` on host port 13306.

### Debug Log References

**Baseline** — 416 tests (211 bin + 159 core + 46 xtask), green WITH the database attached, so the
DB-backed tests genuinely ran rather than returning early.

**AC6 — the reference scale**, printed by `one_full_pass_at_the_reference_scale`:

```
reference scale, cold:             n=300, pairs=44850, interfaces=300, links=300, pass=145.9 ms
reference scale, idempotent rerun: written=0, superseded=0, unchanged=300,        pass=76.0 ms
```

**AC6 — the amplification, both ends**, from `the_write_amplification_is_measured_at_both_ends`:

| newcomer | superseded | written | unchanged |
|---|---|---|---|
| a LARGER `ObsId` joins a group of six | **0** | 1 | 6 |
| the SMALLEST `ObsId` joins a group of seven | **7** | 8 | 0 |

The witness is the smallest-other id, so *"add one observation"* has two answers and the story
records both rather than passing one off as the figure.

**T9 — the mutation pass.** Ten mutations, each under a 600 s timeout, tree restored between each.
🔑 **Ten reds, zero green, zero compiler-carried.** Carrier recorded PER TEST.

| | mutation | result | tests reddened | carrier |
|---|---|---|---|---|
| M1 | drop `decided_by = 'ENGINE'` from the read | RED 2 | `the_engine_never_supersedes_an_operators_link` · `an_operator_cannot_take_a_slot_the_engine_holds` *(5.10's)* | assertion ×2 |
| M2 | drop `evidence` from the comparison | RED 4 | `each_decision_bearing_column_…` · `a_changed_witness_…` · `a_purge_after_a_supersede_…` · `the_write_amplification_…` | assertion ×4 |
| M3a | on a match, fall through to close+insert | RED 3 | `a_second_identical_pass_…` · `one_full_pass_at_the_reference_scale` · `the_write_amplification_…` | assertion ×3 |
| M3b | on a match, insert WITHOUT closing | RED 5 | the four above plus `a_purge_after_a_supersede_…` | **`.expect` panic ×5** — `Constraint("unique")` |
| M4 | close at `valid_from + 1 µs` | RED 1 | `a_changed_witness_…`, on the zero-length assertion | assertion |
| M5 | revert `0004` to `0002`'s strict form | RED 3 | `a_changed_witness_…` · `a_purge_after_a_supersede_…` · `the_write_amplification_…` | **`.expect` panic ×3** — `Constraint("check")` |
| M6 | keep the old row current instead of closing | RED 3 | same three | **`.expect` panic ×3** — `Constraint("unique")` |
| M7 | restrict the purge to current rows | RED 2 | `a_purge_after_a_supersede_…` · `a_superseded_engine_link_is_not_restored_by_the_replay` *(5.10's)* | assertion ×2 |
| M8 | drop 5.9b's `abstained.insert()` dedup | RED 1 | `a_repeated_obs_id_abstains_once_and_the_pass_says_so` | assertion |
| M9 | compare ONLY `evidence` | RED 2 | `each_decision_bearing_column_…` · `an_abstention_and_a_placement_…` | assertion ×2 |

**Five more at the code review**, same discipline, tree restored between each:

| | mutation | result | tests reddened | carrier |
|---|---|---|---|---|
| M10 | drop the orphan-closure tail loop | RED 2 | `a_slot_the_input_no_longer_supports_is_closed` · `the_replay_invariant_survives_an_observation_that_lost_a_key` | assertion ×2 |
| M11 | drop the `InstantRegressed` guard | RED 1 | `an_instant_that_runs_backwards_is_refused_by_name` — and its output shows the pre-guard behaviour verbatim, `Err(Constraint("check"))` | assertion |
| M12 | forget to record a PLACEMENT as visited, so the tail closes slots it just wrote | RED 6+ | broad, incl. `a_second_identical_pass_…` and `a_purge_after_a_supersede_…` | assertion, mixed with `.expect` |
| M13 | drop `outcome` from `same_decision` | RED 1 | `each_decision_bearing_column_…` | assertion |
| M14 | drop the `decided_by = 'ENGINE'` filter, WITH the new operator test present | RED 3 | M1's two plus `the_engine_never_adopts_or_supersedes_a_differing_operator_row` | assertion ×3 |

⚠️ **One honest negative.** The `SELECT`-based oracles added to
`the_write_amplification_is_measured_at_both_ends` are a real improvement — the test asserted only
`Resolution` fields, which its own module doc forbids — but **no mutation was found that reds the
new post-state assertions WITHOUT also reddening an existing summary assertion**. They are recorded
as a strengthened oracle, not as an independently measured guard. Claiming otherwise would be the
defect this whole review is about.

🔴 **M5 confirmed the validation's warning end to end.** Editing `0004` in place, WITHOUT a database
reset, gives `migrate: VersionMismatch(4)` on every DB-backed test — a red with nothing to do with
the guard M5 targets. After `DROP DATABASE opencmdb; CREATE DATABASE opencmdb;` it reds the three
tests above. And the error a test SEES is `Constraint("check")`, not the raw `ERROR 4025`:
`classify` flattens it first, exactly as prescribed.

🔴 **M8 and M9 were measured GREEN at the story's validation and are RED here.** That is not a
contradiction, it is the point: AC2c and AC2b were written *because* they came back green, and they
are the tests that red them. Without AC2b the six-column comparison would have measured one column.

**Divergences from the predicted table — a divergence is a FINDING, per §9 of the house rules:**

- **M2** predicted AC3 + AC5; measured **4** tests, including AC2b's pure test and AC6. The extra
  reds are legitimate — more of the suite depends on evidence than the prediction credited.
- **M3a** predicted AC1 alone; measured **3** (AC1, the reference scale, AC6).
- **M5** predicted *"AC3, AC5, both AC6 runs"*; measured **3** — `one_full_pass_at_the_reference_scale`
  does **not** red, because nothing in it ever supersedes.
- **M6** predicted four tests; measured **3**, for the same reason.
- **M1, M4, M7** matched their predictions exactly.

⚠️ One self-inflicted error, recorded because it is the kind this project keeps catching: a comment
was first added to `0002_interface_and_identity_link.sql`. **sqlx checksums the migration FILE,
comments included**, so that edit would have made every existing database refuse to migrate —
precisely what `0003`'s own header warns about. Reverted to a byte-identical file before any commit;
the note lives in `0004`, which is where it belongs.

### Completion Notes List

- **AC1** ✅ `a_second_identical_pass_writes_nothing_at_all` — the six link `id`s are identical after
  the second pass, `links_written = 0`, `links_unchanged = 6`, table still 6 rows. Comparing `id`
  across runs is reachable precisely because nothing was written. M3a/M3b red it.
- **AC2** ✅ `Resolution` carries `links_superseded` and `links_unchanged`, both documented, both read
  back against the database rather than against the pass's own summary. `abstentions` is kept a
  SUBSET of `links_written` — `Resolution::record` says why.
- **AC2b** ✅ `same_decision` is pure; three database-free tests, one perturbation per column plus the
  unchanged baseline and the abstention-cause case. M9 reds two of them.
- **AC2c** ✅ Story 5.9b's dedup guard is **kept**, and now measured through the COUNTS
  (`links_unchanged == 0`) rather than the surviving rows — a row count cannot tell the dedup from
  the compare. M8 reds it.
- **AC3** ✅ Old version survives with its own `id`, its evidence `[o1]` and `valid_to == valid_from`;
  new version carries `[o1, o2]` **sorted ascending**. M4 and M5 red it.
- **AC4** ✅ Operator row untouched — same `id`, same `valid_to`, still `OPERATOR` — and the pass is
  `Err(Constraint("unique"))` with a full rollback. M1 reds it and 5.10's sibling.
- **AC5** ✅ 3 rows before the purge, 2 after the replay, snapshots equal. Both numbers asserted.
  M7 reds it and 5.10's sibling.
- **AC6** ✅ Numbers above; no threshold installed.
- **AC7** ✅ **Closed, on the qualified sentence.** No PRODUCTION caller compares a held instant
  against a stored one, story 5.11 included — `same_decision` deliberately omits `valid_from`
  because both versions of one placement carry the same one. Eight TEST sites do compare, and the
  register already records why they cannot catch a truncation change. Closed rather than re-owned a
  third time; what would make the risk real is registered with story 5.14 as owner.
- **AC8** ✅ Trap corpus still **11 unanswerable, `passed() == false`**. `fixtures/` untouched.
  `identity::{l1,blocking,cascade}` untouched. `main.rs` untouched. Six gates green, both clippy
  forms clean.
- **AC9** ✅ Twins, sprint status and this file agree.

**416 → 429 tests** (224 bin + 159 core + 46 xtask) after the code review; 425 at implementation.

---

### Code review (2026-08-05, three layers)

Blind Hunter (diff only), Edge Case Hunter (live database on 13312, 19 probes executed), Acceptance
Auditor (live database on 13313, six mutations re-run). **2 arbitrations by Guy, 14 patches.**

🔴 **The review found that this story turned a LOUD refusal into a SILENT orphan, and it attributed
that by MEASUREMENT rather than by argument.** `write_link` only ever reads the slot it is about to
FILL, so a key that vanished from the input left a current link standing — an observation pointing
at an interface no fact supports. The Edge Case Hunter neutralised the new read with `AND 1 = 0`,
recovered the pre-5.11 behaviour, and put the two side by side:

| | second pass over an observation that lost a MAC |
|---|---|
| before 5.11 | `Err(Constraint("unique"))` — loud, full rollback |
| 5.11 as first written | `Ok(links_unchanged: 1)` — silent, orphan left current |

**The uniqueness key had been doing the detection work, and the compare routed around it.** The
orphan is also a reachable counterexample to story 5.10's purge-and-replay invariant — through pure
engine input, no operator row, no doctored `obs_id`. **Guy's arbitration: close it here** (AC10).

🔴 **Second arbitration** (AC11): an `observed_at` running backwards was fatal to the whole batch on
one branch and silent on the other. Now `RepositoryError::InstantRegressed`, above the DDL.

🔴 **And that guard REVERSES a disposal made the same day.** It is the first production caller in
this codebase to compare an instant it HOLDS against one it STORED, so the `datetime_literal` debt
is **DISCHARGED**, not *"closed as unreachable"* as AC7 and the register said an hour earlier. The
register records the reversal rather than rewriting it. The truncation residue is named and is
pinned by story 5.10's own test.

🔴 **Four of the auditor's findings were DOCUMENT defects, and the first lands inside the criterion
written to stop it**: AC9 was ticked ✅ while **neither doc twin had been updated** — both still
said *"the pass is NOT idempotent, which is story 5.11's"*, and the File List named them as changed.
Seventh consecutive story with a twin out of step.

Also applied:

- the `decided_by = 'ENGINE'` doc claimed the engine would **supersede** a human, while every test
  in the workspace measured it **adopting** one — the operator rows were byte-identical to what the
  engine writes, so `same_decision` returned `true` and the pass reported `Unchanged`.
  `the_engine_never_adopts_or_supersedes_a_differing_operator_row` measures the claimed path, and
  M14 reds three tests where M1 reddened two;
- `same_decision`'s *"at L1"* reason was wrong for `interface_id`: the lookup key IS the subject and
  `identity_link_current_subject` makes it equal to `interface_id`, so it is unreddenable through
  the database **structurally and forever**, not until Epic 6;
- `the_write_amplification_is_measured_at_both_ends` asserted only `Resolution` fields — an oracle
  restating the pass's own summary, which the test module's own doc forbids **370 lines above it**.
  It now `SELECT`s the post-state (15 rows, 8 current);
- `an_abstention_and_a_placement_are_never_the_same_decision` compared an abstention with an
  abstention twice and never made the cross-nature comparison its name promises;
- `resolve`'s `# Errors` still documented the non-idempotence the same commit deleted;
- `Constraint("id")` named a constraint present in no migration for what is a decode failure — now
  `Backend`, via `link_id_of`;
- two of the four new register entries carried a CONDITION rather than a named owner, and the
  commit message claimed otherwise;
- `0002:83`'s *"never zero-length"* is now false and **cannot be edited** (sqlx checksums the file —
  measured during implementation), so it is registered rather than left silent.

⚠️ **One finding was raised and is NOT a defect**: the read-then-write is a TOCTOU under concurrency,
and a losing pass would surface `NotFound`, which `classify` does not treat as retryable. The
resolver has no production caller and no concurrent one; registered, not fixed.

### File List

- `crates/opencmdb-bin/migrations/0004_supersede_admits_a_zero_length_version.sql` — NEW
- `crates/opencmdb-bin/src/repo.rs` — `subject_of`, `load_current_engine_link`, `outcome_token` and
  `cause_token` widened to `pub(crate)`, `purge_engine_links`' history warning
- `crates/opencmdb-bin/src/resolver.rs` — `WriteOutcome`, `same_decision`, the three-branch
  `write_link`, `Resolution::{links_superseded, links_unchanged, record}`, the rewritten module doc,
  nine new tests and the reference-scale rerun
- `_bmad-output/implementation-artifacts/deferred-work.md` — three disposals, four new entries, one
  inherited test name corrected
- `_bmad-output/implementation-artifacts/sprint-status.yaml`, `CLAUDE.md`,
  `docs/project-context.md` — bookkeeping

---

## Change Log

| Date | Change |
|---|---|
| 2026-08-05 | Created by `create-story`. 🔴 SPLIT at contexting: 5.11b inserted, Epic 5 → 18 stories. Four arbitrations by Guy (§1–§4), the third taken with a live-database measurement in hand: closing a version at its own `valid_from` is `ERROR 4025`, and the relaxed form admits it while still refusing an inverted closed interval and a zero-length current one. |
| 2026-08-05 | Validated by two fresh-context agents; **14 findings applied**, 5 HIGH. The story BUILDS — the gap-hunt reached 416 → 424 tests with six green gates. 🔴 Two mutations came back GREEN and became `AC2b` and `AC2c`: five of the six comparison columns are unreddenable through the database, and this story silently retires story 5.9b's abstention-dedup guard. 🔴 §3's *"EVERY engine supersede is zero-length"* was retracted to the conditional it can support — `valid_from` comes from the in-memory observation and nothing reads `observation_record.observed_at` back. The evidence literal `[o2, o1]` was corrected to `[o1, o2]` in three places, `M5` gained the `DROP DATABASE` it cannot run without, and `AC6` now names both amplification cases (0 or n−1) with their measured numbers. |
