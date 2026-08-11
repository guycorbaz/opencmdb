# Story 5.14: Wire the identity pass into the shipped binary — and measure what it cannot say

Status: review

<!-- ✅ VALIDATED TWICE. The first pass (as a DISPLAY story) SPLIT it; the second pass, on the
     rewritten story, produced arbitrations 7 and 8. Both gap-hunts BUILT the story and ran the real
     binary against a real MariaDB.

     🔴 **THE SECOND PASS'S HEADLINE, found by BOTH layers: AC3 pinned the DECLARATION while its
     mutation muted the EMISSION.** `arp_ping.rs` carries two independent literals forty lines apart
     with no cross-check — `facts: vec![Fact::IpV4, Fact::Rtt]` (`:177`) and
     `Capabilities { kinds: {IpV4, Rtt} }` (`:183`). Adding `Fact::Mac` to the emitted vector left
     **all 495 tests green, the pin among them**. And **the structural zero is caused by what is
     EMITTED** — that is what `join` reads — so the pin was on the wrong literal. §5a is the
     narrowed, honest form.

     🔴 **Two of the story's own ACs contradicted each other**: AC4 forbade a new read, AC5 required
     one. `Resolution` is per-PASS and reports `links_written: 1` after two passes, never "2 current
     links" — AC5's fact is only visible through the database.

     ⚠️ **This story does NOT display anything.** 5.14b owns the surface. -->

## Story

As the operator,
I want the identity pass to actually run in the product I deploy,
So that what the engine knows — and what it cannot know — is a fact about my network rather than a
property of the test suite.

**And as the next developer, I want to be told what this wiring does NOT buy**, in tests rather than
in prose, so that the day a connector emits a MAC or a scan stops accumulating, something reds and
names what changed.

⚠️ **This story implements NONE of `epics.md`'s story 5.14 acceptance criteria, and that is worth
stating plainly rather than implying a partition.** All four of them are display clauses — the
grouped count, the Dignity bans, the `Ambiguous` candidates, the measured-floor sentence — and all
four are **5.14b's**. The wiring this story does is asked for nowhere in `epics.md`; it is asked for
by `deferred-work.md:2407` and by the fact that without it the display cannot mean anything.

**What this story does NOT do:**

- it does **not** display anything. No template, no locale key, no view-builder change — a diff
  touching `crates/opencmdb-bin/templates/` is a FINDING;
- it does **not** decide what the abstention counter's population IS (§4b). That needs a notion of
  *"the same unplaceable thing seen twice"*, which is grouping, which is Epic 6's;
- it does **not** implement an `l2-*` rule; the trap gate stays **`passed() == false`**, 11
  unanswerable;
- it does **not** change BEHAVIOUR in `opencmdb-core` — scoped to behaviour on purpose, 5.13b having
  measured that a bare *"does not touch X"* becomes a reason not to look at X;
- it does **not** touch the ARP/ping connector. §4a MEASURES what it emits; giving it a MAC is a
  connector story with its own privilege question (the neighbour table is not free);
- it does **not** add a dependency, and it does **not** edit `epics.md` (§7 registers instead).

---

## 1. 🔴 Guy's arbitrations

| # | when | question | decision |
|---|---|---|---|
| 1 | contexting | the pass is not wired; the counter would read zero forever | **5.14 wires it.** A counter that cannot fall is decoration (D18). |
| 2 | contexting | `epics.md`'s `Ambiguous` AC is unreachable | Re-owned to Epic 6 with the unreachability ASSERTED → **moved to 5.14b**. |
| 3 | contexting | ~22 registered entries name 5.14 | Answer the conditions one by one; split between the two stories by §7. |
| 4 | validation 1 | wiring replaces one structural zero with another, and the counter AGES | 🔴 **SPLIT.** 5.14 wires and MEASURES; **5.14b INSERTED** for the display. Epic 5 → 20 stories. |
| 5 | validation 1 | the section inherits the declared side's visibility gate | Hoist it out — **5.14b's**. |
| 6 | validation 1 | AC1's guard is unsatisfiable: deleting the wiring leaves the suite green | The seam IS the helper, **written** with the mutation as evidence. _(SUPERSEDED IN PART by 8 — kept, not overwritten: it was right that the last link cannot be carried, and wrong that one extraction was the end of it.)_ |
| 7 | **validation 2** | a DB-refused observation costs every other observation its link, and AC2 cannot see it | 🔴 **Hand the pass only what LANDED, and REPLACE AC2.** Its subject changes from *"the observations survive a refused pass"* — unwritable, since the only two refusals are unreachable from a scan slice — to **"the blast radius of a refused ingest is bounded to its own row"**, which is the reachable failure mode. |
| 8 | **validation 2** | the cheap guard arbitration 6 dismissed EXISTS, in ~40 lines | 🔴 **Take the generic seam.** Generic over `Connector`, a test drives poll→ingest→resolve with the committed `FixtureConnector`. The uncarried region shrinks from the whole wiring to **three lines**. 🔑 *Recording an unavoidable GREEN is honest; recording it without measuring how much it covers is not.* |

## 2. What both validations measured, so nobody re-derives it

- **`resolve` runs from the startup path and writes rows** — real binary, `127.0.0.1/32`:
  `links_written=1 abstentions=1 interfaces_minted=0`;
- **`InstantRegressed` and `ContradictoryObservation` are UNREACHABLE from a scan slice** (fresh
  `Uuid::now_v7()` per observation, one `observed_at` per poll, five runs). This is what makes
  arbitration 7 necessary: without it AC2 had no reachable failure to assert on;
- **the ingest is already one `repo.transact(…)` PER OBSERVATION** — there is no single unit to join;
- **the accumulation is structural, not incidental**: the vacating loop iterates over `observations`
  only (*"an observation absent from the slice is not evidence that its links are stale"*), so a
  second pass with different `obs_id`s does not close the first pass's abstention;
- ⚠️ **`Resolution::record`'s own doc**: an IDEMPOTENT pass over a MAC-less observation reports
  `links_written = 0, abstentions = 0, links_unchanged = 1`. **So "abstentions == slice length" holds
  on a FIRST pass over fresh ids only** — assert it that way or it is false on the second run.

## 3. The wiring (arbitrations 6 + 8)

Extract the work out of `spawn_startup_scan` (`main.rs:172-263` — a `std::thread::spawn` whose
handle is dropped, inseparable from a live ICMP poll) into a seam **generic over `Connector`**, so a
test can drive `poll → ingest → resolve` with the already-committed `FixtureConnector`.

🔴 **And say exactly what is still uncarried.** Three lines remain outside any test: build the
connector, open the pool, call the seam. **Measure that**, do not merely assert it — arbitration 8's
whole content is that *"the last link is carried by nothing"* was true and incomplete.

**Two transaction units**, and the pass receives **only the observations that LANDED** (arbitration
7). `identity_link.observation_id` is a foreign key onto `observation_record`
(`0003_resolver_guards.sql`), so an un-ingested row fails the whole pass: measured, one refused
observation beside one good one gave `resolution=None` and **0 current engine links**.

## 4. 🔴 The two structural zeros — this story's real deliverable

### 4a. The only connector `main.rs` reaches emits NO MAC, ever

`arp_ping.rs:177` emits `vec![Fact::IpV4, Fact::Rtt]`; `:183` declares
`Capabilities { kinds: {IpV4, Rtt} }`. `join` keys on `(L2DomainId, MacAddr)`, so **every observation
the shipped product produces falls to the abstention path**.

🔴 **The two literals are independent and nothing cross-checks them** — see §5a, which is where the
first draft's pin went wrong.

### 4b. The population ACCUMULATES: the count measures uptime, not reach

Each scan mints fresh `obs_id`s, so each scan writes a NEW current abstention link and nothing
supersedes anything: five runs over ONE host → `current engine links = 1, 2, 3, 4, 5`. ⚠️ The
`current_subject IS NOT NULL` filter does NOT stop it; the population is OBSERVATIONS, and
observations accumulate.

🔑 **Why this story does not fix it**, and the argument is now a MEASUREMENT rather than prose: the
production mutation that would fix it — widening the vacate pass to close engine slots belonging to
observations it never saw — **erases a host that missed a single scan**. Over-vacating is a worse
defect than accumulating, which is why the denominator is Epic 6's and not a one-line change here.

## 5. What must be PINNED

### 5a. 🔴 The connector pin, narrowed to what it can honestly carry

The first draft pinned the DECLARED kinds and prescribed a mutation on the EMITTED facts. **Both
layers measured that green.** The narrowed form, on story 5.12's precedent:

- **pin the DECLARATION** (`Capabilities.kinds` carries no `FactKind::Mac`) — this runs in CI;
- **pin the EMISSION** (the fact vector carries no `Fact::Mac`) — ⚠️ **this is the one that carries
  §4a's structural zero**, because `join` reads facts, not descriptors;
- ⚠️ **and STATE the asymmetry**: every existing test that reaches a live emit is gated on
  `OPENCMDB_NET_TESTS`, which **CI never sets** (verified: no occurrence under `.github/`). So write
  the emission pin against the smallest reachable surface, and **if it can only run gated, say that
  the pin which runs in CI is not the one that carries the fact.** A narrowed true promise beats a
  wide false one.

### 5b. The structural zero, on the pass's own outcome

Over a MAC-less slice, `interfaces_minted == 0` and every observation abstains — asserted on
`Resolution`, on a **FIRST** pass over fresh ids (§2's caveat).

### 5c. The accumulation, pinned and named as a DEFECT

Two passes over slices carrying different `obs_id`s for one address leave TWO current links.

⚠️ **This asserts a defect on purpose**, and three things follow:
- the "this is a defect, not a specification" sentence goes in the **doc comment as well as** the
  assertion message — *the reader who mistakes it for a spec is reading it while it passes*, and a
  message is only read on failure;
- the message must **never say "delete this test"**. Under the honest production mutation the count
  falls to **0**, which is the OPPOSITE defect. Say: *do not repair this number — take it to 5.14b /
  Epic 6*;
- ⚠️ **and this pin FORCES a database read** (§6), which is why AC4's old *"no new read"* is gone.

## 6. The read this story does add — and its guards

`Resolution` is per-pass, so §5c's fact is invisible to it. **A read is required**, and the first
draft's *"this story adds no read"* was contradicted by its own AC5.

⚠️ **Every predicate of that read is carried by nothing unless a test creates the row it excludes.**
Measured: dropping `decided_by = 'ENGINE'`, dropping `current_subject IS NOT NULL`, and dropping both
each leave the suite green — **fifth recurrence of that family**. So the read ships with an operator
row and a superseded row in the fixture, or its `WHERE` is decoration.

## 7. Prove-to-red

| id | mutation | predicted |
|---|---|---|
| **M1** | delete the `resolve` call inside `spawn_startup_scan` | 🔴 **GREEN, unavoidably** — and §3 states the size of what that leaves uncarried (three lines) rather than only its existence |
| **M1b** | delete the `resolve` call inside the generic seam | **RED, exactly 1** — arbitration 8's payoff, measured |
| **M2** | run the pass in the SAME transaction as the ingest | RED |
| **M3** | hand the pass the unfiltered slice | RED — the bounded-blast-radius assertion (AC2, arbitration 7). ⚠️ Put the database-count assertion FIRST: the first draft's ordering killed the test on a preceding assertion and its explanatory message never printed |
| **M4** | add `FactKind::Mac` to the connector's **DECLARATION** | RED — §5a's first pin. ⚠️ The first draft mutated the EMISSION here and was measured GREEN |
| **M4b** | add `Fact::Mac` to the connector's **EMISSION** | RED — §5a's second pin, the one that carries the structural zero |
| **M5** | make `join` key on something a MAC-less observation carries | RED ×7 |
| **M6** | widen `resolve_within`'s vacate pass to close slots of observations it never saw | RED ×7, the accumulation pin among them. ⚠️ **This is the honest production mutation**; changing the test's own fixture to reuse `obs_id`s measures the test's input, not the code — 5.12's recorded family |
| **M7** | drop `decided_by = 'ENGINE'` from §6's read | RED once the operator row exists; **GREEN without it** — and that green is what §6 exists to prevent |
| **M7b** | drop `current_subject IS NOT NULL` from §6's read | RED once the superseded row exists |

## 8. The register, split between the two stories

Of the **twenty-two** entries naming 5.14 (counted; the count was verified by both layers), this
story takes what the WIRING answers. **5.14b takes the rest**, and they are listed in
`sprint-status.yaml`'s 5.14b block — ⚠️ **not in "5.14b's §7", which does not exist**: 5.14b is
`backlog` with no file, and a register bullet pointing at a section of an unwritten story points at
nothing.

| entry | disposition here |
|---|---|
| **`:2407` the resolver is NOT wired into `main.rs`** | 🔴 **CLOSED — with BOTH halves of its own sentence named.** It reads *"no page to display them **and no purge to remove them**"*. The first draft quoted only the first half. **The purge half is the one §4b makes acute**: `purge_engine_links` exists since 5.10 and has **no production caller**, and with the wiring live the accumulation is ~105 000 rows/year for one host. ⚠️ **That half is owned by NEITHER story** — register it |
| `:2700` `observed_at` stability across passes | handled — §4b's accumulation is its consequence |
| `:2772` `ContradictoryObservation`'s REACHABILITY | **answered by MEASUREMENT** — unreachable from a scan slice; the entry survives for the seam's other callers |
| `:2391` `count_identity_links` has no production caller | ⚠️ **RE-OPENED here, not re-owned.** §6 adds a read, so *"this story adds no read"* was false. But `count_identity_links` is `SELECT COUNT(*) FROM identity_link`, unfiltered — it **cannot** serve §6, which needs both predicates. Record why the existing function does not fit rather than leaving it looking unused |
| the page-less deployment | ⚠️ **until 5.14b ships, which may be with Epic 6** — the first draft said *"for one story"*, which over-claims a duration nothing fixes |

---

## Acceptance Criteria

**AC1 — the pass runs in the shipped binary, through a seam a test can drive.**
`resolve` runs over the scan's landed observations through a seam **generic over `Connector`**, which
a test drives end-to-end with `FixtureConnector`; a failure is logged at `error!` naming the refusal.
🔴 **And the story states the MEASURED SIZE of what remains uncarried** — the three lines outside the
seam — with M1's green as the evidence and M1b's single red as what arbitration 8 bought.
_Reddened by: M1b. M1 is GREEN by construction and that is recorded, not repaired._

**AC2 — a refused ingest is bounded to its own row (arbitration 7).**
**Given** a slice in which one observation is refused by the database
**When** the scan ingests and the pass runs over what LANDED
**Then** every other observation still gets its link.
⚠️ **This replaces the first draft's "the observations survive a refused pass"**, which was
unwritable: the only two refusals are unreachable from a scan slice (§2).
⚠️ **The database-count assertion comes FIRST**, or the explanatory message never prints.
_Reddened by: M3._

**AC3 — the connector's MAC-lessness is pinned on BOTH literals, and the asymmetry is stated.**
The declaration and the emission each get a pin; the doc says **which one carries §4a's structural
zero** (the emission) and **which one runs in CI** (the declaration), and does not claim the second
is the first.
_Reddened by: M4 and M4b._

**AC4 — the structural zero is pinned on `Resolution`.**
`interfaces_minted == 0` and every observation abstains, **on a first pass over fresh ids** (§2).
_Reddened by: M5._

**AC5 — the accumulation is pinned, named as a defect in BOTH the message and the doc, and does not
tell a reader to delete it.**
_Reddened by: M6, the production mutation._

**AC6 — §6's read ships with its guards.**
An operator row and a superseded row exist in the fixture, so each predicate of the `WHERE` is
carried.
_Reddened by: M7 and M7b._

**AC7 — nothing is displayed.** No file under `templates/`, no locale key, no view change.

**AC8 — gates and corpus untouched.** `cargo xtask ci`: 28 fixtures, seven gates green; trap gate
**26 discovered, 15 scored, 11 unanswerable, `passed() == false`**.

**AC9 — the register.** §8's five dispositions appended to `deferred-work.md`, never rewriting a
bullet, **`:2407` closed with both halves of its sentence named**, and the purge half registered as
owned by neither story.

**AC10 — documents in the same commit**, including 5.14b's insertion (Epic 5 → 20 stories). One live
count, in one place.

---

## Tasks / Subtasks

- [x] **T1 — the generic seam** (AC1): extract, make it generic over `Connector`, drive it with
      `FixtureConnector`; two transaction units; the pass receives only what landed
- [x] **T2 — measure the uncarried region** (AC1): M1 and M1b, and write the SIZE
- [x] **T3 — the pins** (AC3, AC4, AC5): both connector literals; `Resolution` on a first pass; the
      accumulation with its doc-comment sentence
- [x] **T4 — the read and its guards** (AC6): the operator row and the superseded row
- [x] **T5 — prove-to-red** (AC1–AC6): M1–M7b, predictions first, carriers from panic messages, the
      command that carried each red named
- [x] **T6 — the register** (AC9)
- [x] **T7 — gates and documents** (AC8, AC10)

---

## Dev Notes

### Traps, each measured on this project

- ⚠️ **DB-heavy.** Your OWN `mariadb:10.11.11` on your OWN port; 13306–13313 are taken;
- ⚠️ **DB tests take `crate::DB_TEST_LOCK`** (`main.rs:41`);
- ⚠️ **`cargo test --workspace A B` passes two filters where cargo accepts one** — 0 red for a sound
  mutation;
- ⚠️ **Commit before mutating; revert the MUTATION, never the FILE**;
- ⚠️ **Never read a measurement through a truncation**;
- ⚠️ **sqlx 0.9 refuses a `format!`ed SQL string** (`SqlSafeStr` is `&'static str` only);
- ⚠️ **`OPENCMDB_NET_TESTS` is set nowhere under `.github/`** — a test gated on it does not run in CI,
  and a mutation against it comes back green because the test SKIPPED.

### The tree this story extends, to be RE-MEASURED

`master` at `6ceb284`: **494 tests** (273 bin + 159 core + 62 xtask), seven gates green, 28 fixtures,
26 traps across ten families, trap gate RED. ⚠️ Earlier drafts quoted **282 bin tests**; that figure
is a validation worktree's, WITH the display work, and is not this tree's.

### References

- [Source: `epics.md#Story 5.14`] — **not edited**; all four of its clauses are 5.14b's
- [Source: `crates/opencmdb-bin/src/arp_ping.rs:177`, `:183`] — the two independent literals
- [Source: `crates/opencmdb-bin/src/main.rs:172-263`, `:244-257`, `:41`]
- [Source: `crates/opencmdb-bin/src/resolver.rs:207`, `:124-165`, `:346-361`]
- [Source: `crates/opencmdb-bin/src/repo.rs:889`, `:1037-1041`] — `count_identity_links`, `snapshot_links`
- [Source: `deferred-work.md:2407`, `:2700`, `:2772`, `:2391`]

---

## Dev Agent Record


### Agent Model Used

Claude Opus 5 (1M context) — `claude-opus-5[1m]`.

### Debug Log References

**Built and mutated against a live `mariadb:10.11.11`**, container `opencmdb-5-14`, host port
**13314** (13306–13313 are earlier stories' and the validation worktrees'). ⚠️ **`DATABASE_URL` is
unset locally and the six new DB tests pass by `return`ing** — the suite reported 502 green either
way. The witness that they really ran is the timing: the `bin` suite takes 0.06 s without a database
and **~4 s** with one. **Committed (`b853afd`) before the mutation pass**, and the tree verified
clean after it.

### Completion Notes List

**494 → 502 tests (281 bin + 159 core + 62 xtask), +8.** Seven gates green, 28 fixtures, trap gate
still RED at 26/15/11. `opencmdb-core` behaviour unchanged; nothing under `templates/`.

#### A decision the story did not anticipate, taken and recorded

§4a said this story *"does not touch the ARP/ping connector"*. **It touches it**, and the reason is
that AC3 was otherwise unimplementable: both literals were anonymous expressions inside `poll`, and
`poll` opens an ICMP socket eagerly, so neither could be asserted without the network — and every
existing test that reaches a live poll is gated on `OPENCMDB_NET_TESTS`, which CI never sets.

What shipped is **two named functions, `declared_kinds()` and `emitted_facts()`, and no behaviour
change**: `poll` now calls them where it used to inline them. The prohibition was about *what the
connector emits* (giving it a MAC is a connector story with its own privilege question), and that is
untouched. 🔑 Naming them is also what let the two halves finally CROSS-CHECK each other — the
absence of that check is precisely what let the first draft's pin stay green.

#### The mutation table, with OBSERVED results

| id | mutation | predicted | OBSERVED | red | carrier |
|---|---|---|---|---|---|
| **M1** | delete the seam call inside `spawn_startup_scan` | **GREEN** | ✅ **0 red** — the three lines are uncarried, exactly as arbitration 6 established and arbitration 8 bounded | 0 | — |
| **M1b** | delete the `resolve` call inside the seam | RED ×1 | ⚠️ **3 red** — a divergence, in the good direction: three of the seam's tests consume `resolution`, where the validation's build had one. Recorded rather than smoothed | 3 | assertion |
| **M2** | run the pass in the SAME transaction as the ingest | RED | 🔴 **NOT EXECUTABLE.** There is no single ingest unit to join — the ingest is one `transact` PER OBSERVATION, which this story's own §2 established and its mutation table kept a row for anyway. Story 5.9's M4/M5 family. **M3 carries the boundary instead** | n/a | n/a |
| **M3** | hand the pass the UNFILTERED slice | RED | ✅ **exactly 1**, `a_refused_ingest_is_bounded_to_its_own_row`, on the database-count assertion placed first | 1 | assertion |
| **M4** | `FactKind::Mac` in the **DECLARATION** | RED | ✅ **2 red** — the declaration pin AND the emission pin's agreement assertion. The cross-check working | 2 | assertion |
| **M4b** | `Fact::Mac` in the **EMISSION** | RED | ✅ **1 red**, the pin that carries the structural zero. 🔴 **The first draft measured this combination GREEN**; that is the repair, measured | 1 | assertion |
| **M5** | `join` keys on an `IpV4` fact | RED | ✅ **11 red**, the structural-zero pin among them | 11 | assertion |
| **M6** | widen the vacate pass to close slots of unseen observations | RED | ✅ **4 red** — the accumulation pin **and three pre-existing `resolver` tests**. 🔑 Those three ARE the story's argument: over-vacating erases a host that missed a single scan, and it is now a measurement rather than prose | 4 | assertion |
| **M7** | drop `decided_by = 'ENGINE'` | RED | ✅ **1 red** | 1 | assertion |
| **M7b** | drop `current_subject IS NOT NULL` | RED | ✅ **1 red** | 1 | assertion |

**Ten rows: nine executable mutations, one NOT EXECUTABLE, one GREEN by construction (M1).**
Carriers read from each panic message; **all assertion-carried** among the reds. ⚠️ Two divergences
from prediction, both recorded above: M1b's count, and M2's executability.

#### What is NOT claimed

The counter's denominator is not decided (§4b), nothing is displayed, and `epics.md`'s four clauses
remain 5.14b's. The purge half of `:2407` is owned by neither story and is registered.

### File List

- `crates/opencmdb-bin/src/scan_pass.rs` — NEW. The seam, the read, and their tests.
- `crates/opencmdb-bin/src/main.rs` — MODIFIED. `mod scan_pass;` and the three uncarried lines.
- `crates/opencmdb-bin/src/arp_ping.rs` — MODIFIED. `declared_kinds()`, `emitted_facts()` and the
  two pins. No behaviour change.
- `_bmad-output/implementation-artifacts/5-14-*.md`, `sprint-status.yaml`, `deferred-work.md`,
  `CLAUDE.md`, `docs/project-context.md`.

---

## Change Log

| date | what |
|---|---|
| 2026-08-11 | Created as the display story; three arbitrations at contexting. |
| 2026-08-11 | **VALIDATED, then SPLIT** (arbitration 4): wiring replaces one structural zero with another, and the counter ages. 5.14b inserted. |
| 2026-08-11 | **VALIDATED A SECOND TIME on the rewritten story; arbitrations 7 and 8.** 🔴 Both layers found the same headline: **AC3 pinned the DECLARATION while its mutation muted the EMISSION** — two independent literals forty lines apart with no cross-check, measured GREEN, and it is the EMISSION that carries the structural zero. 🔴 **Two of the story's own ACs contradicted each other** (AC4 forbade a read, AC5 required one). 🔴 **A DB-refused observation cost every other observation its link** and AC2 could not see it: Guy replaced AC2's subject with the bounded blast radius, the reachable failure mode. 🔑 **And the cheap guard arbitration 6 dismissed exists**: a seam generic over `Connector` shrinks the uncarried region from the whole wiring to three lines — *recording an unavoidable GREEN is honest; recording it without measuring how much it covers is not.* Also: the read's every predicate was carried by nothing (fifth recurrence), the accumulation's honest mutation is a production one rather than a fixture edit, and `:2407`'s **purge half** — owned by neither story — is registered. |
