# Story 6.14b: The operator lifts the doubt

Status: **review** — developed 2026-09-26 (see *Dev Agent Record*). Before that: **ready-for-dev** — arbitrated (§0.8, §0.11) and validated (§0.9, two layers, one database each). The arbitration inserts
story 6.14c (A1). ~~a planning act recorded in `epics.md`~~ — ~~⚠️ *not yet: `epics.md` carries no 6.14c; the
fact-check caught this line asserting it.*~~ ✅ Recorded by this story's planning PR (#219): `epics.md` carries 6.14c,
and the glossary edit §0.9(G) owes. **Code-reviewed on that PR (2026-09-26)**, three isolated layers: six decisions
by Guy, 23 patches applied (see *Review Findings*); still `ready-for-dev`.

**Epic 6.** Inserted by Guy's act of 2026-09-25 at story 6.14's arbitration, and sequenced after `v0.6.0`,
which is published and **runs on the NAS**. ~~so the precondition for writing this story is met.~~ ⚠️ *The epic
asks that `obelix` be **seen and used** there; the review found only "runs" recorded.* ✅ **`obelix` was seen on the
NAS as ONE Ambigu question** (Guy, 2026-09-26). That is the *seen* half; nothing it offers can be *used* before
this story, whose gesture is the use.
**Base:** `master` at `75a9037`, clean tree. **Live count at base:** 802 + 219 + 110 as measured at 6.14's
merge, ~~recalled and not measured on this tree~~ and re-measured on `75a9037` by §0.9's prototype layer. It must
be **re-measured again** with `cargo test -p <crate> --locked -- --list` before it is written into the Record,
because the base will have moved by then.

## Story

As the operator,
I want to answer the question an ambiguity asks,
So that the engine stops asking it and the answer is kept as mine.

## The epic's text (`epics.md`, story 6.14b), which is an insertion note and has no criteria

> Its first obligation is a planning act, not code: what each answer WRITES. *"not the same machine"* is an
> OPERATOR `no_match` row in `l2_pair_decision`, D21's shape; it needs a `rule_id`, a `verdicts` and a
> `ruleset_version`. *"the same machine"* is a device, whose mint 6.12 left ownerless, and a `match` outcome
> that `0012` refuses. It must also say whether the glossary's `attach`/*rattacher* already names the positive
> answer. The engine already leaves an operator's pair alone (6.12's code review). **Resolve replaces
> Document on an ambiguous card** is Epic 7's by its coverage line (UX-DR43); this story decides whether to
> take it.

The criteria below are therefore this story's own, derived from that note, from the glossary row `resolve`
(`prd.md:1003`, `ux-design-specification.md` twin), from D12/D14/D15/D21, and from the one register row this
story owns (`deferred-work.md:6541`, the Ambigu row's place in the queue).

## 0. What contexting measured, and what it will not settle alone

### §0.1 — 🔴 THE NAS'S OWN CASE NEEDS THE POSITIVE ANSWER, and the negative answer does nothing for it

`obelix` is **one machine with two network cards** (Guy, 2026-09-24). The question 6.14 puts on `/triage` about
it has one true answer: *the same machine*. A story shipping only *"not the same machine"* would be smaller
and fully specified already (D21: *"undo a link → supersede with a `NO_MATCH` `decided_by='OPERATOR'`"*), and
it would leave the one real question on the reference network **unanswerable**. The negative answer is the
right shape for the other real case (two machines that share a factory hostname,
`hostname-collision.toml`), which the NAS does not show today.

⚠️ So the size of this story is decided by the positive answer, and §0.2 shows that answer has three layers.

### §0.2 — 🔴 *"The same machine"* is THREE writes, and only the first is this table's

1. **The answer itself** is a row in `l2_pair_decision`, `decided_by = 'OPERATOR'`, outcome `match`.
   `0012` refuses `match` at two places: `l2_pair_decision_outcome` (`IN ('no_match','abstained')`), and
   `l2_pair_decision_rule_xor_cause`, which has no `match` arm. A migration `0013` must widen both, and
   **only for `OPERATOR`**. An ENGINE `match` stays refused while no L2 rule is `Decisive` (6.12's §0.6).
2. **The device.** D12 says *"the `device` level is non-negotiable"*. `entity`/`device` exist since `0006`
   **with no producer**, and 6.12's re-scope gave the mint to *"whichever story first produces an L2
   `Decisive`"*. 🔑 **An operator's `match` is a REAL producer**, which a hand-built verdict was not, so the
   objection 6.12 raised (*"a guard placed where the defect cannot occur"*) does not apply here. There is
   **no membership table**. D14 says *"`interface.device_id` is NOT a unique FK … N:N from day 1"* (VRRP), and
   `interface` carries no `device_id` column at all (measured: `grep device_id crates/` gives zero code hits).
3. **What the operator SEES.** 🔴 **The inventory does not list interfaces. It lists DECLARED entities**
   (`inventory_view.rs:19`, *"the declared side, and only that"*), one per *Ajouter* pressed. `obelix` is
   two inventory rows because it was documented twice, once per address. So a device over two interfaces
   **does not by itself make `obelix` one row**: the inventory would have to map declared entities to
   interfaces ~~(through the declared `mac`, since PR #163)~~ (⚠️ *§0.9(K): a path by ADDRESS already exists;
6.14c needs no `mac` for it*) and interfaces to a device. And D15 forbids the
   shortcut, because `declared_attribute.entity_id` is never updated.

⚠️ **Layer 3 is where operator-visible value lives, and it is the heaviest.** Layers 1+2 without 3 give:
*the question leaves the queue and stays answered*, a real gain, but `/devices` still shows two rows.

### §0.3 — `attach`/*rattacher* does NOT name the positive answer

The glossary row reads *"Attach a discovery to an existing record: a link; no data moves"*. The positive
answer attaches nothing to an existing record: at the moment of the answer **no device exists**, and the
answer is what creates one. It is also symmetric (two interfaces, neither of which is the record), where
`attach` is directional. The `resolve`/*résoudre* row already covers both answers (*"the operator chooses
among the candidates"*), ~~so **no glossary edit is owed**~~ (⚠️ *refuted by §0.9(G): the row became false and
did not obviously cover "none of them is the other"; §0.11 rewrote it in PR #219*). The two ANSWERS are labels of one gesture, not two
gestures. ⚠️ Their wording (*« Même machine »* / *« Machines distinctes »* or similar) is copy rather than
vocabulary; it must still pass the `copy-vocabulary` gate, which the dev agent's gate run measures (contexting
did not). ⚠️ *Corrected by the fact-check:* contexting had named `fusionner` as a forbidden word for the labels;
it is on no list. The EN column forbids `merge` and its forms, and the KEY-name column forbids `merge*`.

### §0.4 — Measured facts the dev agent inherits

- **The pass already leaves an operator's pair alone.** `l2_pass.rs:152` counts `operator_held` and skips
  the pair when its current row `is_operators()`. So an OPERATOR row written by this story silences the
  question at the next sweep **without touching the pass**. It is pinned by
  `a_pair_an_operator_decided_is_left_to_the_operator` (`l2_pass.rs:795`; the forging `UPDATE` at `:807`), which today forges the operator
  row with a raw `UPDATE`. That test gains a real producer.
- **The readers already exclude operator rows.** `load_current_ambiguous_pairs` reads ENGINE rows only, so
  once answered, the Ambigu row leaves `/triage` by construction. It is pinned at `l2_pass.rs:899` (forging `UPDATE` at `:926`) (*"an
  operator's row is an answer, not a question"*).
- **`insert_l2_decision` takes an engine `Decision`** (`l2_repo.rs:352`), and `is_persisted` turns `Match`
  into `Err(l2_match_not_persisted)`. An operator answer is not a `decide` output: it needs **its own
  adapter** (a sibling, on story 6.2's `adopt_declared_attribute` precedent). Forging a `Decision` for it
  would be the struct-literal `Decision` story 5.9b refused.
- **`rule_id` is a plain `String`** with no level-prefix validation in the store. The trap harness routes on
  the `l1-`/`l2-` PREFIX (story 5.7), so a token outside both, e.g. `operator`, cannot be mistaken for a
  rule by any runner.
- **The gesture routes' precedents**: the Origin check `write_guard::same_origin`; keyed bodies in both
  locales; `HX-Redirect` carrying the confirmation in the URL (6.4); the release **reads no clock**, dating
  itself at what the operator was SHOWN (14.4b's review); both browser gates must PRESS the control (6.4,
  14.2b).
- **The group, not the pair.** 6.14 shows one Ambigu row per connected GROUP (`ambiguity_view::groups`,
  union-find, root = smallest id). An answer is given on a group and must be written as one row per PAIR,
  since that is the table's key and the pass's.
- **Dating.** Every `l2_pair_decision` instant is derived and *"never `NOW()`"* (`0012`). Closing the ENGINE
  row requires `closed_at >= its valid_from` (`l2_pair_decision_interval`).

### §0.5 — The decisions that are Guy's, each with a recommendation

**(A) Scope: how much of *"the same machine"* ships here?**
(A1) Both answers written as OPERATOR rows in `l2_pair_decision` (`0013` widens `match` for `OPERATOR`
only); the question leaves the queue and stays answered. **The device and the inventory go to a story
6.14c, inserted now**, whose producer is this story's `match`.
(A2) A1 **plus** the device mint and a membership table (SCD2, N:N per D14); the inventory unchanged.
(A3) Everything, including the inventory showing `obelix` as one machine.
**Recommendation: (A1).** It is the only option where every write has a reader in the same story: A2 mints
devices **nobody displays**, which is a write without a reader, the shape Epic 5's retrospective counted
twenty stories of. A3 is three stories in one, and its layer 3 raises a D15 question of its own (how a
declared entity reaches a device without its `entity_id` moving) that deserves its own validation. 🔑 A1
also gives 6.14c **the producer 6.12 found missing**. ⚠️ Cost stated rather than hidden: after A1, `/devices`
still shows `obelix` twice.

**(B) What each answer writes (the row's shape).**
`rule_id = 'operator'` for both answers; `verdicts` = **the ENGINE's vector the operator answered over**,
copied from the row it supersedes, so the evidence the operator was shown is kept with the answer;
`ruleset_version` = that row's; `decided_by = 'OPERATOR'`; `abstention_cause` NULL. The ENGINE row is closed,
never deleted (D21: *"annulment is an addition"*).
**Recommendation: as written.** Alternative refused: a synthetic vector such as `operator=decisive`, which
would claim a rule verdict nobody computed.

**(C) The instant.** (C1) the instant the operator was SHOWN, meaning the question's newest candidate
freshness, carried by the form (14.4b's release precedent: no clock). A current ENGINE row whose
`valid_from` is LATER than that instant means the question changed under the operator, and the write is
refused with a keyed 409 (*reload and look again*). (C2) the wall clock.
**Recommendation: (C1).** It is the house rule for derived instants, and it doubles as the stale-page
guard, which a clock cannot give.

**(D) A group of three or more.** (D1) One binary answer for the whole group: *same machine* writes `match`
on every pair; *distinct machines* writes `no_match` on every pair. (D2) A per-candidate choice (partition).
⚠️ *Narrowed by §0.11(1a): "every pair" is every ENGINE-ambiguous pair of the group, and none elsewhere.*
**Recommendation: (D1)**, and the partition case is registered (row written by PR #219's review). The reference LAN's only question is a pair,
and D2 is a form whose correctness is its own story.

**(E) The *Ajouter* control on an ambiguous address (UX-DR43, *"never a blind document"*).** 6.14 kept it
(Guy's decision C2) and recommended C1 *"with 6.14b"*, because a control that did not act would have
replaced a working one.
(E1) Now that *Résoudre* acts: while the question is OPEN, the `Nouveau` row of an address in it shows the
link to the question **instead of** *Ajouter*; once the question is answered, either way, *Ajouter* returns.
(E2) Keep C2.
**Recommendation: (E1).** Documenting `obelix` twice before answering is exactly D12's *"the operator
announces 300 hosts, the tool shows 340"*, and the UX spec binds it three times. ⚠️ It removes the only live
gesture from those rows **only while** a live one exists beside them, which was 6.14's own condition.

**(F) The switch.** The IPAM write routes carry no opt-in; `POST /document-all` sits behind
`OPENCMDB_DOCUMENT_ENABLED`, which guards an AUTHORSHIP hazard (writing declared values).
(F1) No switch, as IPAM. (F2) Behind the documenting switch.
**Recommendation: (F1).** An answer writes an identity INPUT and no declared value, so the hazard the switch
guards is absent. The release notes say so, as 14.2's did.

**(G) Can the operator see, and change, an answer?** D21 says a change is an addition (supersede with
another OPERATOR row). But **no screen shows an answered pair**, so a wrong *distinct machines* would be
invisible and permanent.
(G1) The reach section gains ONE line counting answered questions, with no gesture; changing an answer is
registered with an owner (row written by PR #219's review; the owner is posed to Epic 6's retrospective). (G2) Nothing is shown; the whole matter is registered.
**Recommendation: (G1).** One sentence costs little, and it keeps the answer from vanishing, which is this
story's *so that* (*"kept as mine"*).

**(H) The Ambigu row's place in the queue** (register row owned here). (H1) Directly BEFORE the first
`Nouveau` row of its addresses. (H2) Leave it at the end.
**Recommendation: (H1).** Under E1 the address rows point at it, and a question the eye meets after its own
consequences reads backwards.

### §0.8 — ✅ GUY'S ARBITRATION, 2026-09-25: all eight on the recommendation

**A1 B C1 D1 E1 F1 G1 H1.** Both answers are OPERATOR rows in `l2_pair_decision`, and `0013` admits `match`
for `OPERATOR` only. **Story 6.14c is INSERTED** for the device, the membership and the inventory showing
`obelix` as one machine; its producer is this story's `match`. `rule_id='operator'`, and the superseded
ENGINE row's vector and version are copied. The instant is the one the operator was shown, and a stale page
answers 409. A group gets one binary answer (on its ENGINE-ambiguous pairs, §0.11(1a)); the partition is
registered. *Ajouter* yields to the question
while it is open. No switch. One reach line counts answered questions. The question sits before its
addresses' rows.

### §0.6 — What the validation must try to refute

- §0.2(3), that the inventory lists declared entities, so a device does not collapse `obelix`. If false, A2
  or A3 become cheaper than argued.
- §0.4, that the pass needs no change for an OPERATOR `match` (it was only ever tested with `no_match`/
  forged rows). `CurrentL2Decision::carries` and `is_persisted` must be read against a `match` row, and the
  L2 pass's `vacated` path against a pair whose interfaces the sweep no longer places.
- That widening two CHECKs in `0013` is re-runnable ~~(14.5's four spellings, `ADD CONSTRAINT IF NOT EXISTS`)~~
  and never leaves the table unconstrained between DROP and ADD. ⚠️ MySQL DDL is not transactional. 🔴 *Both
  layers refuted the cited spelling: see §0.9(A).*
- That no gate (vocabulary, `copy-vocabulary`, `authorship`, `observed-immutable`, `entity-id-immutable`) reds on
  the new writer, or, if one does, which sanction is owed.

### §0.9 — 🔴 What the validation found (fact-check + a BUILT prototype, each with its own database)

Both layers tried to refute §0.2–§0.4 and the core holds: **an OPERATOR row is never closed, vacated or
superseded by the engine**, whatever its outcome, because the skip (`l2_pass.rs:150–155`) runs before any
comparison and reads `decided_by` alone. Measured on the prototype: after an OPERATOR `match`, sweep 2 gives
`operator_held: 1, written: 0`, sweep 3 is identical, a sweep where one NIC loses its name gives `vacated: 0`,
and a sweep missing one interface judges no pair and leaves the row untouched. **The answered question leaves
`/triage`** (both readers filter ENGINE rows), **no gate reds on a new writer** (none of `authorship`,
`observed-immutable`, `entity-id-immutable` names `l2_pair_decision`), two concurrent answers leave exactly one
current OPERATOR row, and the live count at base is **802 + 219 + 110**, measured. What moved:

**(A) 🔴 The re-runnable spelling of `0013`. Both obvious forms are silent defects.**
- `ADD CONSTRAINT IF NOT EXISTS x` on a store holding the narrow `x` keeps the OLD body with only
  `Note 1826`.
- `DROP CONSTRAINT IF EXISTS x, ADD CONSTRAINT IF NOT EXISTS x …` exits 0 and **deletes the constraint**,
  because the `IF NOT EXISTS` is evaluated before the drop.

What works, measured twice on a populated `0012` store:
- the form is ONE statement, `ALTER TABLE l2_pair_decision DROP CONSTRAINT IF EXISTS x, ADD CONSTRAINT x
  CHECK (…)`, with **no `IF NOT EXISTS` on the ADD**;
- it is atomic: a failing one (`ERROR 4025`) left the old constraint intact;
- **both** constraints are re-added in that one statement, **`outcome` first**. MariaDB names the first failing
  CHECK in declaration order, and a re-added constraint moves to the end. `outcome` alone re-added makes an
  ENGINE `match` name `rule_xor_cause`, which reds `l2_repo::tests::a_match_or_an_absence_of_proof_is_refused_by_the_schema`
  for no defect.

The shape measured to keep that test green (⚠️ *the TRIM idiom below was written in by PR #219's review; the
prototype measured the shape without it, so T1 measures it with it*):
- `outcome IN ('no_match','abstained') OR (outcome = 'match' AND decided_by = 'OPERATOR' AND
  LENGTH(decided_by) = LENGTH(TRIM(decided_by)))`
- `rule_xor_cause` is three arms:
  - ENGINE `abstained`/`ambiguous`, rule NULL;
  - ENGINE `no_match` with a rule neither empty nor `'operator'`;
  - OPERATOR `match`|`no_match` with `rule_id = 'operator'`, cause NULL, **and** `LENGTH(decided_by) =
    LENGTH(TRIM(decided_by))` **and** `LENGTH(rule_id) = LENGTH(TRIM(rule_id))`.

So an OPERATOR `abstained` is refused, and an ENGINE row may not claim `rule_id='operator'`. The idiom is owed
because `ascii_bin` is PAD SPACE: `'OPERATOR '` passes `l2_pair_decision_decided_by` and `= 'OPERATOR'` alike,
and then reads as the engine's in `is_operators()`; `'operator '` passes `= 'operator'` and is a second
spelling of the token. On the ENGINE arm, `<> 'operator'` already refuses `'operator '` under the same padding,
so no idiom is needed there. ⚠️ `ascii_bin` is case-sensitive: an ENGINE `rule_id = 'Operator'` passes, and is
not the operator's token — `is_operators()` reads `decided_by`, never `rule_id`, and no runner routes a token
outside `l1-`/`l2-`.

**(B) 🔴 The strict CHECK breaks two tests and any reused store holding their residue.**
- `a_pair_an_operator_decided_is_left_to_the_operator` (`l2_pass.rs:807`) and
  `the_screen_reads_engine_ambiguities_and_nothing_else` (`:926`) forge OPERATOR `abstained` rows by `UPDATE`.
  Both must write through the new adapter.
- A store holding such a row fails `0013` with `4025` and boots `Dirty(13)`. The migration header carries the
  recovery recipe: delete the forged rows, then `DELETE FROM _sqlx_migrations WHERE version = 13 AND
  success = 0`.

**(C) 🔴 AC1's *"the type forces every `match`"* was false.** `Gesture::Live { route }` exists, carries ONE
subject through `hx-vals`, and paints the reserved amber `btn-document`. Reusing it would:
- raise no `E0004`;
- post an empty `subject`;
- paint both answers amber;
- be caught by both browser gates as a documenting gesture.

A NEW variant is owed (`Gesture::Answer { route, answer }` or similar). It gets its own render arm, a non-amber
class, and `hx-vals` carrying the group, its members and the instant. Only two sites match on `Gesture`
exhaustively (`GestureView::of` and the askama `match`).

**(D) The instant (C1) needs microseconds, and one case has none.** The connector dates observations below the
second (`DATETIME(6)`). An answer carrying the instant truncated to seconds was measured `l2_answer_stale`: every
answer on a real network would be a 409. The seed makes equality the ordinary case, so the comparison is strict
(`valid_from > shown` refuses). A group whose candidates have no current placement has **no** newest freshness,
so the form needs a defined value for that case, never `UNIX_EPOCH`, which would refuse for ever.

**(E) The group can SHRINK under the page, and C1 does not catch it.** Measured: three NICs, one question; one
loses its name; the group keeps its id (smallest member), and the answer from the three-candidate page is
accepted and writes ONE row. The operator answered for three and the product recorded two. See §0.10(1).

**(F) The answer and a concurrent sweep.** If the answer commits first, the sweep blocks on its close, finds 0
rows, returns `NotFound`, and **rolls back the whole sweep, L1 included**. If the sweep commits first, C1
refuses the answer. See §0.10(2).

**(G) The glossary row `resolve` becomes false the day this ships.** It reads *"**Named here before it acts** —
… what each answer WRITES is story 6.14b's"*, in `prd.md:1003` and its UX twin (`:1351`). Editing it is a
planning act, Guy's, and it goes in the PR that inserts 6.14c. The fact-check also contests §0.3's *"covers
both answers"*: *"chooses among the candidates"* does not obviously cover *"none of them is the other"*. The same
edit can say it.

**(H) Keys and guards the dev agent meets.**
- `gesture.not_built_resolve` leaves `app.yml` and `NOT_A_GLOSSARY_GESTURE` (7 → 6).
- The answer labels cannot be `GLOSSARY_BACKED` under `gesture.*`, because that guard demands the glossary's
  exact words. They go in `NOT_A_GLOSSARY_GESTURE` with a reason, or in another namespace.
- `<p id="gesture-not-built">` renders unconditionally, so it must render only when a planned control is present
  (6.4's defect otherwise: a shipped sentence made false).
- `triage.ambiguous.add_alone` becomes false under E1 while the question is open.
- `Strings (19, 175)`, `MIN_CHECKS = 66`, and the seed's exact count (17) move.
- 6.14's kbd check *"one control, planned, no live documenting gesture"* is rewritten.

**(I) Routing and refusals.**
- `WriteRoute` is `/ipam`-scoped, and the document sub-router holds no pool by design (6.1). The route needs a
  pool-bearing router and its own entry in the auth route-table test.
- *"Already answered"*, *"the group is gone"* and *"no such id"* are indistinguishable to an adapter reading
  ENGINE rows. They get ONE keyed 409 (*this question is no longer open*); 422 stays for a malformed request.

**(J) Browser gates.** CI runs axe before kbd.
- axe must not press anything.
- The *answered* state needs a SECOND seeded question, already answered by an OPERATOR row (14.4b's *"one
  pressed, one already taken"* precedent).
- kbd reads the open question's pane and E1's link **before** pressing, and *Ajouter* returning **after**.

**(K) Smaller corrections.**
- E1 and H1 change nothing for `obelix` on the NAS: both addresses are already documented, so there is no
  `Nouveau` row. H1's rule when the question has no such row: it keeps its place at the end. `?sort=age`
  re-sorts after placement.
- `is_persisted(&decision)?` runs BEFORE the operator skip (`l2_pass.rs:150`, ~~`:149`~~). Harmless while no L2
  rule is `Decisive`; the day one is, an engine `Match` on an operator-held pair aborts the sweep. Registered
  (row written by PR #219's review), owner: the story that first produces an L2 `Decisive`.
- Docs that become false and are corrected in the code change:
  - `CurrentL2Decision::outcome` (*"`no_match` or `abstained`"*);
  - `l2_repo.rs`'s module doc (*"never says `Match`"*);
  - `0012`'s header.
- The `file-size` gate reads `l2_repo.rs` as 76 lines (a `#[cfg(test)]` type alias at line 77) where ≈556 are
  code, and `main.rs` as 43 where ≈1103 are. That is the same blindness registered for `repo.rs`, and it is to
  be registered, since both files grow here.
- A path from a declared entity to an interface already exists **by address** (`ambiguity_rows` maps address →
  interface → group). 6.14c needs no `mac` for it; A1 stands.
- Citations corrected: D14 `:1068–1104`, D21's quote `:1460`, `inventory_view.rs:19`, the tests at
  ~~`l2_pass.rs:794/899`~~ `l2_pass.rs:795` and `:899` (their forging `UPDATE`s at `:807` and `:926`; `:794` is the
  attribute line — corrected by PR #219's review).

### §0.10 — The decisions the validation opened, each with a recommendation

**(1) What an answer on a group writes, and how a changed group is caught.**
(1a) The form carries the group's MEMBER interface ids. The route recomputes the group server-side and answers
the keyed 409 if the members differ in either direction. It writes one OPERATOR row per ENGINE-ambiguous pair
of the group, the pairs that ARE the question, and none on a pair with no row or with an ENGINE `no_match`.
(1b) As (1a), but it also writes on every other pair of the group.
**Recommendation: (1a).**
- (1b) would override an engine `no_match` (`l2-virtual-mac-prefix`) by answering a question it never asked.
- In (1a), the chain's `match` rows are what 6.14c's union-find joins, so the machine is still one.
  ⚠️ *PR #219's review: that join crosses the ENGINE `no_match` (1b) was refused for overriding — transitively,
  one story later. **Guy, 2026-09-26:** carried into 6.14c's entry in `epics.md` and into the register, not
  decided here.*

**(2) The answer that races a sweep.**
(2a) The pass treats a close that finds no current row as *a human took it*: it re-reads the slot, and skips it
if an OPERATOR row is current. The sweep then no longer rolls back.
(2b) Register it (at worst one five-minute sweep lost).
**Recommendation: (2a).** It is a few lines in `l2_pass`, and it keeps an operator's click from costing the L1
placements of a whole sweep. The prototype did not measure whether a lost sweep's observations stay unplaced
for good, and (2a) makes the question moot.

**(3) *Ajouter* after *"the same machine"* (E1).** Once answered, E1 gives *Ajouter* back on both addresses.
Documenting both would create two declared entities for what the operator just called one machine: the D12
over-count E1 exists to prevent, until 6.14c.
(3a) After a `match`, the addresses keep a sentence instead of *Ajouter* (*"answered: one machine; its record
arrives with the device grouping"*); after a `no_match`, *Ajouter* returns.
(3b) *Ajouter* returns either way; registered for 6.14c.
**Recommendation: (3b).** (3a) would take the only live gesture away from those rows for a whole story to
protect a count that 6.14c exists to fix. On the NAS it changes nothing, both addresses being documented
already. The register row names the double documentation, so 6.14c inherits it.

**(4) The reach line (G1): its unit and its screens.**
(4a) It counts answered QUESTIONS (groups, by the same union-find run over current OPERATOR rows), and it is
shown where `ambiguous_groups` is shown: `/triage`, with the existing register row (*"Only `/triage` reads
`l2_pair_decision`"*, owner Epic 6's retrospective) widened to name it.
(4b) It counts pairs, labelled as pairs.
**Recommendation: (4a).** Two counts of one frame in two units is 5.14b's defect, and it is avoided by
construction. Filling `/dashboard` is the retrospective's already-registered question, and should not be
answered here for one line.

### §0.11 — ✅ GUY'S SECOND ARBITRATION, 2026-09-25: all four on the recommendation

**(1a)** The form carries the group's MEMBER interface ids. The route recomputes the group and answers the keyed
409 if the members differ in either direction. It writes one OPERATOR row per ENGINE-ambiguous pair of the
group, and none elsewhere.
**(2a)** A close that finds no current row makes the pass re-read the slot, and skip the pair if an OPERATOR row
is current. The sweep no longer rolls back.
**(3b)** *Ajouter* returns after either answer, and the double documentation after *the same machine* is
registered for 6.14c.
**(4a)** The reach line counts answered QUESTIONS (groups, by union-find over current OPERATOR rows), on
`/triage`, where `ambiguous_groups` is shown. The existing register row is widened to name it.

The planning act rides with this story file, as PR #213 did for 6.14:
- **6.14c** is inserted in `epics.md`, and both epic entries carry the arbitration;
- the glossary row `resolve` is rewritten **identically** in `prd.md` and the UX specification. It now names the
  two answers, keeps *"not `attach`"* with its reason, and drops *"Named here before it acts"*, which this
  story makes false (§0.9(G)).

## 1. Acceptance criteria — under Guy's arbitrations (§0.8, §0.11) and the validation (§0.9)

1. **The answers are live, and a type says so.** **Given** an open Ambigu question on `/triage` whose group has a
   current placement (the other case is AC5's), **when** it is selected, **then** its bar carries the gesture's
   name, *Résoudre* (the glossary's `resolve`), and under it its two answers (*the same machine* / *distinct
   machines*). A test fails if the name is absent while the answers render (Guy, 2026-09-26: the binding term
   must not leave the screen).
   - **A group that re-forms around an already-answered pair** (Guy, 2026-09-26): the pairs the operator
     already answered are not part of the question, and the pane names the members already joined by an earlier
     answer, with that answer. A test answers A–B, adds a third candidate D ambiguous with both, and asserts the
     pane names A–B's answer and the POST writes only D's pairs.
   - Each is a `<button>` reachable by Tab, with an accessible name naming the answer and the group it answers.
   - They are carried by a NEW `Gesture` variant (§0.9(C)), not by `Gesture::Live`. They are not amber, and the
     pane carries no `btn-document`.
   - The planned `Résoudre` control and `gesture.not_built_resolve` are gone, and the not-built note renders
     only when a planned control is present. A test fails if the note renders beside live answers.
2. **An answer writes the question and nothing else.** **Given** a POST carrying the group's id, its member
   interface ids, the answer and the instant shown, **then** in ONE transaction every current ENGINE
   `abstained`/`ambiguous` row among the group's pairs is closed at the instant shown. One OPERATOR row per
   such pair is opened, with:
   - `outcome` `match` or `no_match`;
   - `rule_id = 'operator'`;
   - the closed row's `verdicts` and `ruleset_version`;
   - `valid_from` = the instant shown.

   No other pair is written: not a pair without a row, and not an ENGINE `no_match`. Nothing is deleted. The
   test uses a CHAIN of three interfaces whose third pair carries an ENGINE `no_match`, and asserts that pair
   untouched.
   - The adapter reads the group's current rows with a **locking read** (`FOR UPDATE`), which also serialises
     C1's stale check against a sweep and a second answer. Under MariaDB's default REPEATABLE READ a plain read
     sees a snapshot (measured by PR #219's review).
   - The instant shown is bounded on both sides: an instant LATER than the newest freshness the route
     recomputes for the group is a forged or garbled value and answers 422. A forged `9999-12-31
     23:59:59.999998` passes the stale check and the interval CHECK otherwise (measured).
3. **The schema admits the operator's `match` and nothing more.** **Given** `0013`, a single `ALTER` re-adding
   `l2_pair_decision_outcome` then `l2_pair_decision_rule_xor_cause` with no `IF NOT EXISTS` on the ADD
   (§0.9(A)), **then** each of the following is proved by a raw insert naming its constraint:
   - an OPERATOR `match` or `no_match` with `rule_id='operator'` is accepted;
   - an ENGINE `match` is refused naming **`l2_pair_decision_outcome`** (the existing test stays green
     unmodified);
   - an OPERATOR `abstained`, an ENGINE row with `rule_id='operator'`, a padded `decided_by = 'OPERATOR '` and a
     padded `rule_id = 'operator '` are refused, each test naming the constraint it expects. Predicted, to be
     measured at T1: a padded-`decided_by` `match` names `l2_pair_decision_outcome`; a padded-`decided_by`
     `no_match` and a padded `rule_id` name `l2_pair_decision_rule_xor_cause` (the two are re-added last, and
     `l2_pair_decision_decided_by` accepts the padding).

   Applied twice to a populated `0012` store, it succeeds both times, and `information_schema.CHECK_CONSTRAINTS`
   shows both widened clauses after the second run. The header carries the `Dirty(13)` recovery recipe
   (§0.9(B)).
4. **The engine never asks again.** **Given** an answer written THROUGH THE ROUTE, **then** subsequent sweeps
   leave the answered pairs alone:
   - `operator_held` counts them, and a second sweep writes nothing;
   - a sweep where a member loses its name gives `vacated: 0`;
   - the question does not return.

   The two tests that forged OPERATOR rows by `UPDATE` (`l2_pass.rs:807`, `:926`) write through the adapter
   instead.
5. **Every refusal is named and writes nothing.** Each case below has its own test and a keyed body in both
   locales:
   - a stale page (an ENGINE row whose `valid_from` is later than the instant shown; equality is accepted, and
     the instant carries microseconds, §0.9(D)) → 409;
   - a group whose members changed, grown OR shrunk (§0.9(E)) → 409;
   - a question no longer open (already answered, gone, never existed) → ONE keyed 409 (§0.9(I)); a close
     that finds no row (`NotFound`) and an insert refused by `l2_pair_decision_one_current`
     (`Constraint("unique")`) inside the answer's transaction are this case and map to this 409, never a 500;
   - two concurrent answers to one question: exactly one current OPERATOR row, the winner answers its
     redirect and the loser this keyed 409 — a test drives both;
   - a malformed request → 422;
   - a cross-site Origin → 403.

   A test builds its instant from a sub-second sweep and asserts the answer is accepted. The page's form never
   carries `UNIX_EPOCH`: a group with no current placement is shown without the controls, and says why — **a
   decision, Guy's, 2026-09-26** (§0.9(D) had left it open). The sentence is a key of its own
   (`triage.ambiguous.no_placement` or its measured name), in both locales, and a test renders such a group and
   asserts no answer control and the sentence.
6. **An answer does not cost a sweep.** **Given** an answer committed while a sweep is judging the same pair,
   **then** the sweep skips it (it re-reads the slot, and an OPERATOR row is current). It commits its L1
   placements, and its summary counts the pair as `operator_held` (§0.11(2a)). The test drives both
   transactions.
   - The re-read is a **locking read** (`SELECT … FOR UPDATE`). Measured by PR #219's review under REPEATABLE
     READ: a plain re-`SELECT` in the sweep's transaction returns the stale ENGINE row, and only the locking
     read returns the OPERATOR row. If the locking re-read finds no OPERATOR row, the original error stands.
   - The sweep reaches the close only when its decision DIFFERS from the stored row (`l2_pass.rs:162-177`); an
     unchanged pair counts `unchanged` from the snapshot. The test therefore builds a sweep whose decision
     changes (a supersede or a vacate) — otherwise it measures nothing.
7. **The addresses yield to the question.** **Given** an open question, **then** each `Nouveau` row of its
   addresses shows the link to it instead of *Ajouter*, and `triage.ambiguous.add_alone` no longer says the
   opposite. **Given** the question answered THROUGH THE ROUTE, either way, **then** *Ajouter* returns. The test
   asserts both *"question gone"* and *"Ajouter present"* after the POST (§0.9(J)). The Ambigu row sits before
   the first `Nouveau` row of its addresses, stays at the end when it has none, and `?sort=age` re-sorts after
   placement.
8. **The answer stays visible.** **Given** answered questions, **then** `/triage`'s reach section says how many,
   counted in QUESTIONS. A group of three answered once reads **1**, and the test asserts it.
   - ⚠️ *PR #219's review:* a plain union-find over current OPERATOR rows merges two answers that share a member
     (A–B answered, then B–C) into **1**. The count is per ANSWER: a second test answers A–B, then B–C, and reads
     **2**. The mechanism (a union-find per answer, e.g. over the rows one answer wrote) is the dev agent's to
     measure; if it needs a column, that is a planning question raised, not taken.
9. **Both browser gates reach the answer.** `a11y/seed.sql` gains a SECOND question, already answered by an
   OPERATOR row, so that axe measures the answered state without pressing anything.
   - `kbd-probe.mjs` reads the open pane and E1's link, presses an answer with the keyboard, then reads that
     the row has left the queue and *Ajouter* has returned.
   - 6.14's *"one control, planned"* check is rewritten.
   - `MIN_CHECKS`, the seed's exact count, and `Strings`' literal-field guard move with what they count.
10. **The record says what the product does.**
    - The user manual's triage chapter says what each answer does, that `/devices` still shows a machine
      documented twice until 6.14c, and that **an answer cannot be undone or changed from the screen** (the
      register row *changing an answer* owns it).
    - The docs that become false are corrected: `CurrentL2Decision::outcome`, `l2_repo.rs`'s module doc,
      `0012`'s header, and `CHANGELOG`'s next section (the route has no opt-in).
    - Register rows (⚠️ *the first two, `is_persisted` and the chain across an ENGINE `no_match`, were written by
      PR #219's review; the dev agent re-reads them and edits rather than duplicates*):
      - the partition of a group of three or more (D2);
      - changing an answer;
      - a chain's `match` rows joining across an ENGINE `no_match` (owner 6.14c);
      - the double documentation after *the same machine* (owner 6.14c);
      - `is_persisted` before the operator skip (§0.9(K));
      - the `file-size` gate's blindness to `l2_repo.rs` and `main.rs`;
      - the reach line widening the *"Only `/triage` reads"* row.

## 1b. Tasks

- [x] T1 `0013_operator_answers.sql`: one `ALTER`, the two CHECKs re-added outcome-first, the TRIM idiom, the
      recovery recipe; its raw-insert tests (AC3).
- [x] T2 `l2_repo::record_operator_answer` (a sibling adapter; no forged `Decision`), with group recomputation,
      the group's rows read `FOR UPDATE`, the stale, forged-instant and changed-group checks, `NotFound`/unique
      mapped to the keyed 409, and the single transaction. Rewrite the two forging tests (AC2, AC4, AC5).
- [x] T3 `l2_pass`: the LOCKING re-read on a close that finds nothing (AC6), with its two-transaction test on
      the changed-decision branch.
- [x] T4 The route (pool-bearing router, Origin check, keyed refusals, `HX-Redirect` with the confirmation; its
      row in the auth route-table test).
- [x] T5 `Gesture`'s new variant and render arm; `_action_bar.html`; the keys (§0.9(H)); E1, H1 and the reach
      line (AC1, AC7, AC8).
- [x] T6 Seed, `axe-gate.mjs`, `kbd-probe.mjs` (AC9).
- [x] T7 Mutation pass: predictions first; carriers derived by grepping the mutated token; `--baseline`; one
      database per process. It includes the re-read and the adapter's read made plain (drop `FOR UPDATE`), and
      the TRIM idiom dropped from each OPERATOR arm.
- [x] T8 Docs, register rows, twins, the Record (AC10).

### Review Findings

Review of the planning PR #219 (2026-09-26), three isolated layers: blind (diff only), edge (worktree + a
throw-away `mariadb:10.11.11` on port 13470), acceptance auditor. 49 raw findings → 24 distinct.

⚠️ **Guy, 2026-09-26, on every arbitration of this story:** *the current choices are liable to be questioned once
opencmdb is used regularly.* They are provisional against use, not settled against it.

✅ **All 23 patches applied 2026-09-26** — the 17 below and the six decisions, each turned into a patch.

- [x] [Review][Decision] A group that re-forms around an already-answered pair — `ambiguity_view::groups` is a union-find over ENGINE rows only, so after `obelix` A–B is answered a third same-named NIC D yields a group {A,B,D} shown as a fresh question with nothing saying A–B is answered; one binary answer then writes only the D pairs (*distinct* after an earlier *same* leaves A=B, D≠A, D≠B), and AC8's union-find over OPERATOR rows counts 1 where 2 questions were answered (edge 3, blind 12). → **Guy, 2026-09-26: (a)** — the screen excludes pairs the operator already answered from the group, and names members already joined by an earlier answer. Becomes a patch.
- [x] [Review][Decision] (1a)'s chain joins across an ENGINE `no_match` in 6.14c — §0.10 refuses (1b) for overriding `l2-virtual-mac-prefix`'s `no_match`, then says the chain's `match` rows are what 6.14c's union-find joins "so the machine is still one", which overrides the same Disqualifying transitively one story later; 6.14c's entry in `epics.md` does not inherit the question (blind 6, auditor 9, edge 6). → **Guy, 2026-09-26: (a)** — carried into 6.14c's entry in `epics.md` and into the register. Becomes a patch.
- [x] [Review][Decision] The glossary row `resolve` is in the present tense of behaviour no code has — "it acts since story 6.14b", "*the same machine* is what the device grouping reads (story 6.14c)" stand in two binding documents while 6.14b is `ready-for-dev` and 6.14c `backlog`; and "not `attach`: no record exists to attach to" overstates §0.3, `obelix`'s addresses being documented — it is no DEVICE record that exists (blind 1, 9). → **Guy, 2026-09-26: (a)** — time-neutral wording ("it acts with story 6.14b") and "no DEVICE record exists". Becomes a patch.
- [x] [Review][Decision] AC5 settles an open case no arbitration took — "a group with no current placement is shown without the controls, and says why": §0.9(D) raised it, §0.10/§0.11 record no decision, and "says why" names no key and no test; it also conflicts with AC1's unconditional "its bar carries *Résoudre*'s two answers" (blind 8). → **Guy, 2026-09-26: (a)** — accepted as a decision: a group with no current placement shows no controls, with a named key and test; AC1 narrowed to match. Becomes a patch.
- [x] [Review][Decision] The binding term *Résoudre* may vanish from the screen — AC1 removes the planned control and the two answers are not `GLOSSARY_BACKED` (§0.9(H)); no criterion requires the gesture's own name to appear (blind 19). → **Guy, 2026-09-26: (a)** — AC1 requires the bar to carry the gesture's name, the two answers under it. Becomes a patch.
- [x] [Review][Decision] The precondition is claimed on a weaker fact than the epic states — the epic sequences 6.14b "so that `obelix` is seen and used on the NAS"; the story says v0.6.0 "runs on the NAS, so the precondition is met", and nothing records `obelix` seen as ONE question (blind 11). → **Guy, 2026-09-26: (a)** — `obelix` WAS seen as one Ambigu question on the NAS; the fact is recorded. Becomes a patch.
- [x] [Review][Patch] AC6's re-read must be a LOCKING read, and its test must take the changed-decision branch — measured under REPEATABLE READ: a plain re-`SELECT` in the sweep's transaction returns the stale ENGINE row, `FOR UPDATE` returns the OPERATOR row; and `judge_within` reaches the close only when the decision changed, so an unchanged pair counts `unchanged`, not `operator_held`. Add a plain-read mutation to T7 [§1 AC6, §1b T3] (edge 1, 5; auditor 1; blind 5)
- [x] [Review][Patch] The answer adapter's races: read the group's rows `FOR UPDATE` (serialising C1's stale check too), map `NotFound` and `Constraint("unique")` to AC5's keyed 409, and add a criterion for two concurrent answers and the loser's status [§1 AC2, AC5; §1b T2] (edge 2, auditor 5)
- [x] [Review][Patch] The carried instant is bounded below only — a forged `9999-12-31 23:59:59.999998` passes the stale check and the interval CHECK; require it to equal (or not exceed) the newest `valid_from` of the recomputed group's ENGINE rows [§1 AC2, AC5] (edge 4)
- [x] [Review][Patch] The CHECK shape does not carry the TRIM idiom it claims — under PAD SPACE `'OPERATOR '` and `rule_id = 'operator '` pass as written; write the idiom into the shape for `decided_by` and `rule_id`, and name in AC3 the constraint each padded row is refused by [§0.9(A), §1 AC3] (blind 7, auditor 6)
- [x] [Review][Patch] The live count is called both "recalled and not measured" (header) and "measured" (§0.9) [header :9-11, §0.9 :202] (blind 2, auditor 3)
- [x] [Review][Patch] §0.3's "no glossary edit is owed" stands unstruck while §0.9(G) and §0.11 refute it [§0.3 :74] (blind 3, auditor 3)
- [x] [Review][Patch] §0.2(3)'s "through the declared `mac`" stands unstruck while §0.9(K) says 6.14c needs no `mac` [§0.2] (blind 13)
- [x] [Review][Patch] §0.5(D1) "writes `match` on every pair" is narrowed by §0.11(1a) to the ambiguous pairs, with no mark at D1 or in §0.8's summary [§0.5, §0.8] (blind 6)
- [x] [Review][Patch] The Status line strikes "a planning act recorded in `epics.md`" as "not yet" in the very PR that records it [header :3-5] (blind 14, auditor 3)
- [x] [Review][Patch] Citations: References keep D14 `:1085–1104`, D21 `:1462` and `l2_pass.rs:…795,896`; §0.9(K)'s "corrected" `794/899` is itself off (tests at `:795`, `:899`; forging `UPDATE`s at `:807`, `:926`; `is_persisted` at `:150`, not `:149`) [References, §0.9(K)] (blind 4, auditor 3, edge)
- [x] [Review][Patch] AC7 cites §0.9(K) for asserting "question gone" and "Ajouter present"; the content is §0.9(J)'s [§1 AC7 :424] (blind 17)
- [x] [Review][Patch] §0.3 ends "Corrected by the fact-check: `fusionner` is on no list" with no remaining mention of `fusionner`, and asserts "it still passes the `copy-vocabulary` gate" with no measurement named [§0.3] (blind 16)
- [x] [Review][Patch] "Registered" with no registration — the partition case (§0.8), `is_persisted` before the operator skip (§0.9(K)) and "changing an answer" (§0.5(G1), no owner named) are in no row of `deferred-work.md`, which this PR does not touch [§0.8 :176, §0.9(K), AC10] (auditor 2)
- [x] [Review][Patch] §4 does not say an answer cannot be undone from the screen; AC10's manual sentence omits it too [§4, AC10] (auditor 4)
- [x] [Review][Patch] Stale notes: `sprint-status.yaml` under 6.14b still reads "Next: use it; then this story's planning act", "SEQUENCED AFTER THE NEXT RELEASE" and "First obligation: a planning act…"; `CLAUDE.md`'s and `project-context.md`'s `v0.6.0` paragraph still names that planning act as next [sprint-status.yaml:5147-5150] (blind 10, auditor 10)
- [x] [Review][Patch] `epics.md:433`, Epic 7's coverage line, still claims all of UX-DR43 while `epics.md:2059` takes part of it here [epics.md:433] (auditor 7)
- [x] [Review][Patch] The register row "One machine counts THREE times … follows from Guy's decision C" states a premise E1 revokes while the question is open [deferred-work.md:6547] (auditor 8)
- [x] [Review][Defer] The neighbouring glossary row `triage` differs between the twins (`create / attach` in `prd.md`, `attach / create` in the UX spec) [prd.md, ux-design-specification.md] — deferred, pre-existing

## Dev Agent Record

### Implementation plan, and where it departs from the criteria

- **T1 `0013`** — as §0.9(A) prescribes, with the TRIM idiom written into both OPERATOR arms (the
  review's patch) and `decided_by = 'ENGINE'` added to the two ENGINE arms, so an OPERATOR `abstained` is
  refused. Constraint names measured for AC3's padded rows exactly as predicted: a padded-`decided_by`
  `match` names `l2_pair_decision_outcome`; a padded `no_match` and a padded token name
  `l2_pair_decision_rule_xor_cause`.
- **T2 `l2_answer.rs`** — a NEW module rather than `l2_repo.rs` (at ~586 real code lines behind a
  `file-size` gate that reads 83, registered). The adapter reads the question's rows `FOR UPDATE`,
  RECOMPUTES the group with the screen's own `ambiguity_view::groups`, and refuses in this order: not
  open → changed → (no placement → stale) → forged → stale; then closes each question-pair at the
  instant shown and inserts the OPERATOR row by raw SQL, copying the vector — no forged `Decision`.
  `NotFound` and `Constraint("unique")` inside the transaction map to *no longer open* (409).
- **T3 `l2_pass.rs`** — a close that finds no row re-reads the slot `FOR UPDATE`; an OPERATOR row there is
  `operator_held`, and the sweep continues. **Measured red first**: the race test failed with `NotFound`,
  i.e. the whole sweep, L1 included, rolled back for one click.
- **T4 the route** — `POST /triage/answer` on its own port-state sub-router (no pool on the state), NO
  switch (F1), Origin check first, keyed refusals, `201` + `HX-Redirect: /triage?answered=same|distinct`.
  The auth route-table test walks it (11 → 12 write routes). `ipam_write::Refusal::new` became
  `pub(crate)` so both write surfaces share one refusal type rather than two.
- **T5 the screen** — `Gesture::Answer { route, answer }` and `GestureRender::Answer { route, token,
  name }`; the compiler named the four sites (`GestureView::of`, `ipam_page.rs`, both templates). The
  answer labels live under `triage.answer.*`, NOT `gesture.*` (§0.9(H)). The *not built* note renders
  only when a planned control is on the bar. Keys: 23 added, `gesture.not_built_resolve` removed.
- **T6 browser gates** — the seed gains a SECOND question, already answered (`.202`/`.203`); the keyboard
  gate's Ambigu block now reads the two answers, the gesture's name, E1's link, PRESSES an answer, and
  reads the question gone and *Ajouter* back. `MIN_CHECKS` 66 → **70**, read off the run; the release
  check's outside list gains the two new addresses.

⚠️ **Divergence from AC10's letter: `0012`'s header is NOT corrected.** sqlx checksums every applied
migration, so editing that comment would stop every existing store booting. `0013`'s header and
`l2_repo.rs`'s module doc carry the correction; registered.

⚠️ **Two of my own sentences failed the vocabulary gate**, both carrying the retired « documented » /
« documentée » in `triage.answer.what` and `triage.answer.refused.store`; reworded to *added*.

### Mutation pass (T7) — predictions written first, in `predictions.md`, before any run

Every run on a VIRGIN database, `--baseline` except M4 (a migration mutation: the baseline would migrate
the unmutated `0013` and the mutated one would then fail its checksum). Driver: `cargo xtask mutate`.

| id | mutation | predicted | measured | carriers |
|---|---|---|---|---|
| M1 | the pass's re-read loses `FOR UPDATE` | red:1 | ✅ red 1 | `an_answer_that_races_a_sweep_does_not_cost_the_sweep` |
| M2 | the adapter's read loses `FOR UPDATE` | green | ✅ green | **the finding**: two carriers — the close then finds no row and `NotFound` answers *no longer open* |
| M2b | M2 AND `NotFound` no longer maps to 409 | red:2 | ✅ red 2 | `two_concurrent_answers…`, `every_refusal_is_keyed…` |
| M3 | `NotFound` no longer maps to 409 | red:1 | ✅ red 1 | `every_refusal_is_keyed_and_a_lost_race_is_not_a_500` |
| M4 | `0013`: TRIM idiom on `decided_by` dropped from the OPERATOR arm | red:1 | ✅ red 1 | `the_schema_admits_the_operators_answers_and_nothing_more` |
| M6 | the upper bound (forged instant) neutered | red:1 | ✅ red 1 | `every_refusal_writes_nothing` |
| M7 | the stale check neutered | red:1 | ✅ red 1 | `every_refusal_writes_nothing` |
| M8 | the changed-group check neutered | red:2 | ✅ red 2 | `every_refusal_writes_nothing`, `a_group_that_shrank_under_the_page_is_refused` |
| M9 | E1's `retain` neutered | red:2 | ✅ red 2 | `a_new_row_of_an_open_question_shows_the_link_instead_of_add`, `no_other_kind_carries_a_live_gesture…` |
| M10 | H1: the question pushed to the end | red:1 | ✅ red 1 | `the_question_sits_before_its_addresses_rows` |
| M11 | the *not built* note unconditional | red:1 | ✅ red 1 (+ clippy) | `the_ambiguity_pane_renders_resolves_two_live_answers_under_its_name` |
| M12 | `answered_questions` keys every row together | red:1 | ✅ red 1 (+ clippy) | `answered_questions_are_counted_per_answer…` |
| M13 | the answers painted amber | red:3 | 🔴 **red 2** | `the_ambiguity_pane_renders…`, `the_resolve_gesture_cannot_go_live…` |
| M15 | *the same machine* writes `no_match` | red:5 | 🔴 **red 7** | the five predicted, plus `a_re_formed_group_writes_only_the_pairs_it_did_not_cover` and `page::…a_group_re_formed_around_an_answered_pair_names_the_earlier_answer` |

**Fourteen rows: twelve conform, two contradict, and neither prediction is rewritten.**
- 🔴 **M13**: I counted `ac4_the_amber_is_reserved_for_the_documenting_gesture` as a carrier because it
  greps `btn-document`; it counts the token's READS IN THE SHEET, not its uses in templates. So on this
  pane the amber's reservation is carried by the two render tests alone — *a guard found by grepping its
  subject is not thereby a guard of it*.
- 🔴 **M15**: I derived carriers by grepping the literal `"match"` in two files; the value also reaches
  `Answer::of_outcome` in `page.rs` and a count of `no_match` rows. **The sixth instance in this project
  of enumerating carriers too narrowly** — the grep was mechanical, and its PERIMETER was not.
- M11 and M12 also red clippy (an askama constant condition; unused bindings), which the driver folds and
  reports; the test counts are the prediction's.

### Completion notes

- **Live count: 822 + 219 + 110** (base 802 + 219 + 110, measured on `75a9037`'s successor `3c2f676`, a
  planning-only merge). `RUSTFLAGS="-D warnings" cargo test --workspace --locked` green on a VIRGIN store;
  ten `cargo xtask ci` gates; clippy `--all-targets`; fmt.
- **Browser gates, run as `ci.yml` runs them** against this story's store: axe **10 routes + 6 states,
  0 violation nodes** (empty-plan pass first, then all seven `REQUIRE` flags); kbd **70 checks, 0 failed**
  — it pressed *Distinct machines* with Enter and read `status=201 url=/triage?answered=distinct`, the
  question gone, and *Ajouter* back on `.200`.
- **What the operator gains**: they can ANSWER `obelix`'s question, from the keyboard, and the answer is
  kept as theirs; the engine stops asking, and the reach line counts it. **What they still cannot do**:
  change or undo an answer from the screen; see `obelix` as one row in `/devices` (story 6.14c).

### File List

- `crates/opencmdb-bin/migrations/0013_operator_answers.sql` (new)
- `crates/opencmdb-bin/src/l2_answer.rs` (new)
- `crates/opencmdb-bin/src/l2_pass.rs`, `l2_repo.rs`, `ambiguity_view.rs`, `triage_view.rs`, `page.rs`,
  `identity_view.rs`, `ipam_write.rs`, `ipam_page.rs`, `state_vocabulary.rs`, `sighting_repo.rs`, `main.rs`
- `crates/opencmdb-bin/templates/_action_bar.html`, `_triage.html`, `_identity_section.html`, `_diagnostic.html`
- `crates/opencmdb-bin/locales/app.yml`, `crates/opencmdb-bin/assets/app.css`
- `a11y/seed.sql`, `a11y/kbd-probe.mjs`
- `docs/manuals/user-manual/user-manual.tex`, `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/deferred-work.md`, `sprint-status.yaml`, this file
- `CLAUDE.md`, `docs/project-context.md`

## 2. What this story must NOT do

- Mint a device, add a membership table, or change `/devices` (6.14c).
- Write an ENGINE `match`, or loosen any CHECK for `ENGINE`.
- Write on a pair that is not part of the question, or UPDATE/DELETE any row other than closing the superseded
  ENGINE row.
- Touch `declared_attribute` (D15), or the identity pass's decision logic.
- Offer a partition of a group of three or more (D2).

## 3. Previous-story intelligence (6.12, 6.14)

- The union-find and the evidence sentences are in `ambiguity_view.rs`; the pane's candidates branch is in
  `_triage.html:96–128`, and the open-question link on `Nouveau` panes is at `:131–134`.
- `Strings` carries a literal-field count guard (`page.rs`, `(19, 175)` after 6.14), so every new key moves
  it.
- `state_vocabulary.rs`: `resolve` is `GLOSSARY_BACKED`, and `gesture.not_built_resolve` is in
  `NOT_A_GLOSSARY_GESTURE`. It should leave when the control goes live.
- Lessons that cost this epic: `cargo fmt` breaks mutation anchors; the seed count test moves with
  `a11y/seed.sql`; never `DATABASE_URL=""`; a stray server on 8080 made a gate answer from another database.

## 4. What the operator gains (under A1)

They can answer the question about `obelix`, the answer is kept as theirs, and the product stops asking it.
⚠️ They cannot yet see `obelix` as ONE machine: `/devices` shows its two documented rows until 6.14c.
⚠️ **They cannot undo or change an answer from the screen**: a wrong *distinct machines* stays until someone
supersedes it by hand. The reach line (AC8) says HOW MANY questions were answered, not which. Registered
(*changing an answer*).

## 5. Change Log

| Date | Change |
|---|---|
| 2026-09-25 | Contexted. Eight decisions posed with recommendations (§0.5). |
| 2026-09-25 | **Guy's arbitration**: all eight on the recommendation (§0.8); story 6.14c inserted. |
| 2026-09-25 | Validated by two layers (§0.9); **Guy's second arbitration**, all four on the recommendation (§0.11); criteria rewritten; `ready-for-dev`. |
| 2026-09-26 | **Developed**: `0013`, `l2_answer.rs`, the pass's locking re-read, the route, the screen, the gates; fourteen mutation rows (12 conform, 2 contradict); `review`. |
| 2026-09-26 | Code review of planning PR #219 (three layers): 24 findings, **six decisions by Guy** (all (a)), 23 patches applied — locking reads (AC2, AC6), the instant's upper bound, the TRIM idiom written into the shape, the re-formed group, the per-answer count, the no-placement case decided, *Résoudre* kept on screen; stale sentences struck. Still `ready-for-dev`. |

## References

`epics.md` story 6.14b's insertion note · `prd.md:1000` (`attach`), `:1003` (`resolve`) ·
`architecture.md` D12 (`:909`), D14 (`:1068–1104`), D15, D21 (`:1460`) ·
`0006_entity_device_and_state.sql` · `0012_l2_pair_decision.sql` · `l2_pass.rs:25,150,152,795,807,899,926` ·
`l2_repo.rs:352,406` · `inventory_view.rs:19` · `_triage.html` ·
`6-14-ambiguity-explains-itself.md` §0.5 (C), §0.8 · `deferred-work.md:6541`.

## Record

- live-count: bin=822 core=219 xtask=110
- base: 3c2f6768393fd59241839496237919c0f7fcd58a
- registered: After *the same machine*, both addresses offer Add again and can be recorded twice.
- registered: The `file-size` gate is blind to `l2_repo.rs` and `main.rs`, both grown by story 6.14b.
- registered: `0012`'s header still says `match` is refused outright, and cannot be corrected where it stands.
- file: CHANGELOG.md
- file: CLAUDE.md
- file: _bmad-output/implementation-artifacts/6-14b-the-operator-lifts-the-doubt.md
- file: _bmad-output/implementation-artifacts/deferred-work.md
- file: _bmad-output/implementation-artifacts/sprint-status.yaml
- file: a11y/kbd-probe.mjs
- file: a11y/seed.sql
- file: crates/opencmdb-bin/assets/app.css
- file: crates/opencmdb-bin/locales/app.yml
- file: crates/opencmdb-bin/migrations/0013_operator_answers.sql
- file: crates/opencmdb-bin/src/ambiguity_view.rs
- file: crates/opencmdb-bin/src/identity_view.rs
- file: crates/opencmdb-bin/src/ipam_page.rs
- file: crates/opencmdb-bin/src/ipam_write.rs
- file: crates/opencmdb-bin/src/l2_answer.rs
- file: crates/opencmdb-bin/src/l2_pass.rs
- file: crates/opencmdb-bin/src/l2_repo.rs
- file: crates/opencmdb-bin/src/main.rs
- file: crates/opencmdb-bin/src/page.rs
- file: crates/opencmdb-bin/src/sighting_repo.rs
- file: crates/opencmdb-bin/src/state_vocabulary.rs
- file: crates/opencmdb-bin/src/triage_view.rs
- file: crates/opencmdb-bin/templates/_action_bar.html
- file: crates/opencmdb-bin/templates/_diagnostic.html
- file: crates/opencmdb-bin/templates/_identity_section.html
- file: crates/opencmdb-bin/templates/_triage.html
- file: docs/manuals/user-manual/user-manual.tex
- file: docs/project-context.md
