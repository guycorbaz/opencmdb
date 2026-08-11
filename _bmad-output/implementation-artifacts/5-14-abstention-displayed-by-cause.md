# Story 5.14: Abstention is displayed, counted and grouped by cause — and never as a reproach

Status: ready-for-dev

<!-- ⚠️ VALIDATION IS MANDATORY (Guy, Epic 4 retrospective 2026-07-26): two fresh-context agents,
     fact-check + gap-hunt, BEFORE `dev-story`.

     🔴 THREE ARBITRATIONS BY GUY AT CONTEXTING — §2. The first is the one that changes what this
     story IS: **it wires the pass into `main.rs` AND displays.** Without the wiring the counter
     reads ZERO in production forever, which is a display that cannot fall.

     🔑 READ §4 BEFORE WRITING ANY UI. **The abstention panel ALREADY EXISTS** — story 3.8 shipped
     "Reach" for the DECLARED-side vocabulary. This story adds a SECOND, different vocabulary beside
     it, and `deferred-work.md` says in so many words that no `From` bridge between the two may
     appear silently.

     ⚠️ This is the LAST story of Epic 5. The epic does NOT close until `bmad-retrospective` has
     run, and §9's answered entries plus the items registered by 5.9b, 5.10, 5.12 and 5.13b are its
     input — carry them in, do not rediscover them. -->

## Story

As the operator,
I want to see how many interfaces the product could NOT place, broken down by why,
So that the number measures the product's REACH rather than my debt (FR16b).

**And the number must be REAL.** The engine has written identity links since story 5.9b — in tests.
`main.rs` has never called it. This story is the first to run the identity pass in the shipped
binary, and that is what makes the counter something other than a decorated zero.

**What this story does NOT do:**

- it does **not** build the `Ambiguous` disambiguation panel. `epics.md`'s third AC is unreachable
  at L1 and the code names **Epic 6** as the owner (§3b). Guy's arbitration 2: **re-owned, with the
  unreachability ASSERTED** so the day Epic 6 produces one, a test falls and names its successor;
- it does **not** implement an `l2-*` rule, and the trap gate must stay **`passed() == false`** with
  **11 unanswerable**. A green trap gate is a FINDING;
- it does **not** introduce a ranking. No milli-unit type, no `confidence` column, no widening of
  the `float-free` gate — §9 answers those three registered entries rather than building them;
- it does **not** add a `From`/`Into` bridge between `gap::AbstentionCause` and
  `IdentityAbstentionCause` (§4). Two vocabularies over two populations;
- it does **not** change BEHAVIOUR in `opencmdb-core`. ⚠️ The clause is scoped to behaviour on
  purpose — story 5.13b's review measured that a bare *"does not touch X"* becomes a reason not to
  look at X, and sheltered a false sentence in `score.rs` for exactly that reason;
- it does **not** add a dependency, and it does **not** regenerate `architecture-views.md`
  (issue #50);
- it does **not** edit `epics.md`. §10 registers the corrections instead.

---

## 1. What this story inherits — and it is a lot

**About twenty entries in `deferred-work.md` name story 5.14 as owner**, most of them **by
CONDITION** rather than by name: *"the first story with a ranking surface"*, *"the first place a
consumer can justify the split"*, *"the FR16 surface"*. §9 answers every one of them, one at a time,
which is Guy's arbitration 3 and story 5.7's precedent (`Conclusion::rule()` was CLOSED by answering
its condition, not by building the accessor).

Plus, from the stories that raised them:

| inherited | from | state on arrival |
|---|---|---|
| *"may an operator override the engine?"* | 5.10 | **open**, and 5.10 measured the two natures MUTUALLY EXCLUSIVE on one placement: an operator row in the slot makes the whole replay roll back |
| the resolver is not wired into `main.rs` | 5.9b → 5.13 | **still true**, verified: nothing calls `resolver::resolve` outside tests |
| `Ambiguous` may need splitting into D13's three rows | 5.3, 5.4b | **no consumer yet**, and §3b explains why this story does not become one |
| the epic's RETROSPECTIVE | Guy, 2026-08-10 | **required**; this story is its last input |

## 2. 🔴 Guy's arbitrations at contexting (2026-08-11)

| # | question | decision |
|---|---|---|
| 1 | the pass is not wired; without it the counter reads zero forever | **5.14 WIRES the pass AND displays.** One story. The counter must measure something or it is decoration (D18). |
| 2 | `epics.md`'s AC3 (`Ambiguous` shows its candidates) is unreachable | **Re-owned to Epic 6, with the unreachability ASSERTED** — a test that reds the day an `Ambiguous` or a `link_candidate` row appears, so the successor is named by a falling test rather than by a sentence. |
| 3 | ~20 registered entries name 5.14, mostly by condition | **Answer the conditions one by one** — CLOSED where the condition is answered, RE-OWNED where it is not, each with the measurement that says which. |

## 3. 🔴 Two clauses of `epics.md` established as unimplementable by code

Re-verify these rather than quoting them.

### 3a. The counter would read ZERO in the shipped product

`main.rs:130-135` spawns a one-shot startup scan and `main.rs:249` writes each answered host as an
observation. **Nothing calls `resolver::resolve`** — verified over the whole tree; every call site is
under a `#[cfg(test)]`. So no `identity_link` row, and therefore no abstention, exists in a running
opencmdb. A display alone would be **structurally zero**, permanently, and green.

That is what arbitration 1 answers.

### 3b. The `Ambiguous` panel is unreachable, twice over — and the code names Epic 6

- **Nothing produces `Verdict::Supports` or `Verdict::Opposes`.** The only occurrence outside
  `cascade.rs`'s own definition is an exhaustive match arm at `l1.rs:475`. `resolver.rs:654-655`
  states the consequence: *"L1 emits no `Supports` and no `Opposes`, so it cannot conclude
  `Ambiguous` at all"*;
- **Nothing writes a `link_candidate` row in production.** `resolver.rs:669-671`, verbatim:
  *"Nothing fills `candidates_for_link` yet: the only call site passes `&[]` … **Whoever produces
  the first `Ambiguous` owns filling this slice, and that is Epic 6**"*.

⚠️ And there is a trap in the same doc worth reading before touching anything here: the writer
**refuses** an `Ambiguous` abstention carrying no candidates, and its author already recorded that
the guard *"would refuse a LEGITIMATE ambiguity rather than let it be written with its candidates"*
the day a producer arrives. **Do not build a renderer above a write path whose shape is still Epic
6's to decide.**

## 4. 🔑 What ALREADY EXISTS — read this before writing any UI

**Story 3.8 shipped an abstention panel.** `page.rs` carries `abstentions: Vec<AbstentionRow>`,
`abstention_count`, and `cause_label` (`page.rs:112`) routed through the `rust-i18n` `t!()` seam;
`_gap_card.html:47-48` renders it under *"Reach"* with the hint *"What we saw but could not place —
reach, not debt."* The locale file is `crates/opencmdb-bin/locales/app.yml`, EN + FR per key.

⚠️ **That panel is a DIFFERENT vocabulary over a DIFFERENT population.** It renders
`gap::AbstentionCause` (`OutOfPerimeter`, `NoObservedValue`, `ConflictingObservations`) over declared
attributes. This story renders `identity::cascade::IdentityAbstentionCause` (`Ambiguous`,
`AbsenceOfProof`) over **interfaces**. `deferred-work.md` is explicit: *"No `From`/`Into` bridge
between `gap::AbstentionCause` and `IdentityAbstentionCause`, and none should appear silently."*

**So: extend the page with a second section, reuse the `Strings` + `t!()` seam and the `cause_label`
IDIOM, and do not collapse the two enums.** A shared `cause_label` taking either enum is the
silent bridge that entry forbids.

## 5. The wiring (arbitration 1)

`resolver::resolve(conn, observations) -> Result<Resolution, RepositoryError>` (`resolver.rs:200`)
is the entry point; it computes the universe and delegates to `resolve_within`. It is **idempotent**
(5.11), **order-independent** (5.11b), and it **refuses** an `observed_at` that regresses
(`InstantRegressed`) or a repeated `obs_id` carrying different content (`ContradictoryObservation`).

Prescribed shape, and each clause has a reason:

- **run it after the startup scan's observations are written**, on the same slice the scan just
  ingested — not on a re-read of the whole table, which would grow without bound and would make the
  pass's cost a function of history rather than of the sweep;
- **best-effort, exactly like the scan itself** (`main.rs:163`): a failure logs and does not take the
  binary down. ⚠️ **But it must log at `error!` with the refusal named**, because
  `InstantRegressed` and `ContradictoryObservation` are the two refusals a real network can produce
  and a silent skip would make the counter lie by omission;
- **inside the same `transact` unit as the observation writes, or a separate one?** ⚠️ **This is a
  decision the story does not pre-empt and the dev must take and record**: one unit means a
  resolution failure rolls the observations back (FR11 says an observation is immutable and
  independently true), two units mean the observations survive a failed pass. The house reading —
  D34 §2, *"everything emitted before it is still true"* — points at TWO units. Measure it, do not
  assume it;
- **no new public API on the resolver.** If the wiring needs a signature change, that is a finding.

## 6. The read

Nothing counts abstentions today. `repo.rs` has `load_current_links_for_observation` (`:660`) and the
`abstention_cause` column (`:627`), written as a token by `cause_token` (`:485`) — and **nothing
parses that token back**.

🔑 **The parse-back is a design decision, not plumbing.** The display could map the raw string to a
label; it should instead **parse into `IdentityAbstentionCause` and refuse an unknown token**, so
that a cause the domain does not have cannot reach a screen. That gives the enum its first real
consumer, and it is what makes several of §9's registered entries answerable by measurement rather
than by opinion.

The reader owes:

- **evaluated vs not evaluated over CURRENT engine links** — the two numbers the UX mock puts side by
  side. "Evaluated" is a link carrying an interface; "not evaluated" is an abstention;
- **the not-evaluated count grouped BY CAUSE**, one row per cause, ordered deterministically;
- ⚠️ **current rows only** — `current_subject` is NULL on a superseded row (5.9's second
  arbitration), and counting history would make the number grow with every re-scan, which is exactly
  the ageing the bans forbid.

## 7. The display, and the bans are load-bearing

The UX spec's mock [`ux-design-specification.md:917-947`]:

```
┌────────────────────────────────────────────────────────┐
│  187 evaluated              ·        113 not evaluated │
│  ──────────────────────────────────────────────────    │
│  113 not evaluated, because:                           │
│    · 96  multi-interface — grouping unresolved         │
│    · 17  no live source on this scope                  │
└────────────────────────────────────────────────────────┘
```

**The rules, quoted because they are testable:**

- ***"I don't know" is a MOTIF, never N failures.*** One cause is one line, one question — never one
  row per interface;
- **the counter does not redden, does not grow bold, carries no gauge and no badge, and does not age
  visibly.** *"Six months of inaction and it still reads 113, in the same grey, with the same
  dignity."* ⚠️ **Assert this rather than style it**: a test over the rendered HTML that finds no
  alert class, no `<progress>`, no badge, and no elapsed-time string is a guard; a CSS class chosen
  carefully is not;
- **the floor is stated where the number is displayed** (FR9, NFR30): hostname is unusable on nearly
  half of known clients, so *"an abstention target that ignores this measures the network, not the
  product"*. One sentence, through the `t!()` seam like every other;
- 🔑 ***"Nobody finds a radar pathetic."*** The section's job is to say where the product sees and
  where it does not. That is the sentence to write the copy against.

⚠️ **Today the corpus can produce exactly ONE cause** (`AbsenceOfProof` — §3b), so the grouped list
has one line. **That is not a reason to skip the grouping**: the grouping is what Epic 6 fills, and a
display hard-wired to one cause would have to be rewritten. But **do not claim the grouping is
exercised**: assert the one line that exists, and assert that the second cause is absent (§9).

## 8. What the tests must not let pass

- **the counter cannot be green-by-emptiness.** A run with no observation gives 0 evaluated and 0 not
  evaluated, and every assertion about "the number is right" passes. Assert a NON-EMPTY population;
- **the wiring must be observable.** A test that calls `resolve` itself proves nothing about
  `main.rs`. The guard has to be that **the startup path** produced links — 5.9b's lesson, where a
  seam tested through the wrong door stayed green with the guard deleted;
- **prove-to-red on the bans.** Each ban needs a mutation: add an alert class, add a badge, render a
  per-interface row instead of a per-cause line. If the assertion does not red, it is decoration.

## 9. The registered entries, answered one by one (arbitration 3)

Every entry below names 5.14 in `deferred-work.md`. The dev must **re-read each one at its line** and
record CLOSED / RE-OWNED **with the measurement**, appending to the register and never rewriting a
bullet. The dispositions below are the story's PREDICTION, not its verdict — a divergence is a
finding.

| entry | predicted disposition |
|---|---|
| `:1066` `Ambiguous` must split into D13's three rows | **RE-OWNED to Epic 6** — the condition is *"the first place a consumer can justify one"*, and a display that can render only `AbsenceOfProof` is not that consumer |
| `:1077` no `Ord`/`Display`, the `cause_label` + `t!()` seam, two locale keys per variant | **CLOSED** — this story builds exactly that seam and those keys. ⚠️ `Display` is still NOT written: rendering goes through the label seam, and a `Display` would be the wrong one |
| `:1116` the exhaustiveness mechanism residue (both enums) | **CLOSED or RE-OWNED by measurement** — the parse-back (§6) is the first consumer that would break on a new variant. Measure whether it does |
| `:1257` `Decision::cause()` | **predicted RE-OWNED** — the display reads a persisted STRING, not a `Decision`, so the accessor may STILL have no caller. Story 5.7's shape: answer the condition |
| `:1449` `Abstained { Ambiguous }` does not record WHICH row | **RE-OWNED to Epic 6**, same reason as `:1066` |
| `:1467` widen the `float-free` gate beyond `identity/` | **CLOSED by answering** — no ranking here, so no float in `opencmdb-bin` either; the gate stays scoped |
| `:1471` no milli-unit type/constant/field | **CLOSED by answering** — a count is an integer and a ranking does not exist |
| `:2246` no `confidence` column | **CLOSED by answering**, same |
| `:2297` `link_candidate` attaches happily to a MATCH link | ⚠️ **must be handled, not deferred** — the entry warns that *"a renderer that shows a disambiguation UI whenever the list is non-empty would show it on a decisively-matched link"*. This story builds no such renderer (§3b), so record that it is answered BY ABSENCE and re-own the guard with the renderer |
| `:2300` one corrupt `evidence` blob blinds the whole observation | **must be decided** — the display's read path is the first that could hit it. `CHECK (json_valid(evidence))` exists in MariaDB 10.11 |
| `:2315` `PersistedLink.id`/`.interface_id` bare `String` | **the first reader drives it** — that is this story |
| 5.10's *"may an operator override the engine?"* | ⚠️ **must be answered in writing.** 5.10 measured the two natures mutually exclusive on one placement. The display shows engine rows; whether an operator row is shown beside them, or whether the question is re-owned, is a decision this story records |

## 10. Registered rather than fixed

- **`epics.md`'s story 5.14 AC3 is not implementable at L1** (§3b) — `epics.md` is **not edited**;
  the correction goes to Epic 5's retrospective with Epic 6 as the successor;
- **Epic 5's RETROSPECTIVE is REQUIRED and this story is its last input.** It already carries, by
  name: `epics.md`'s AC1 falsified by the code it describes (5.9b), the `TRUNCATE … WHERE`
  correction (5.10), 5.12's second-session lesson, and 5.13b's four — the reserved-prefix hole with
  no gate, *"a promise of non-modification shelters false sentences"*, *"a floor is only a guard
  while it equals what is there"*, and the mutation-driver family's fourth recurrence.

---

## Acceptance Criteria

**AC1 — the pass runs in the shipped binary.**
**Given** the startup scan has ingested its observations
**When** the binary starts with a scan configured
**Then** `resolver::resolve` runs over that slice and writes identity links, and a failure is logged
at `error!` **naming the refusal** rather than skipped in silence.
**And** the transaction boundary between the observation writes and the pass is **decided and
recorded with its reason** (§5), not inherited.
**And** the guard is on the STARTUP PATH, never on a test that calls `resolve` itself.

**AC2 — evaluated and not evaluated, over CURRENT links only.**
**Given** a population of interfaces, some of which the engine abstained on
**When** the page renders
**Then** it shows the evaluated count beside the NOT-evaluated count, read from current engine links
only — a superseded row must not be counted, or the number ages with every re-scan.
**And** the test asserts a **non-empty** population: 0 vs 0 satisfies every arithmetic claim.

**AC3 — grouped by cause, one line per cause.**
**Given** the not-evaluated population
**When** it renders
**Then** each cause is ONE line with its count — never one row per interface, which is the *"N
failures"* the UX spec forbids.
**And** the persisted token is **parsed into `IdentityAbstentionCause`**, an unknown token being
refused rather than rendered (§6).
⚠️ **And the test asserts what is true today**: exactly one cause is producible (`AbsenceOfProof`),
so it asserts that line AND asserts the absence of the other — never that "grouping works".

**AC4 — the bans, asserted rather than styled.**
**Given** the rendered abstention section
**When** the test inspects the HTML
**Then** it carries no alert/error class, no `<progress>` or gauge, no badge, and no elapsed-time
string; and the count renders identically whatever the age of the rows.
_Each ban carries a mutation that reds it (§8)._

**AC5 — the floor is stated where the number is displayed.**
FR9/NFR30's sentence — the abstention rate is bounded below by DATA AVAILABILITY, not by engine
quality — renders beside the counter, through the `t!()` seam, EN and FR.

**AC6 — no silent bridge between the two abstention vocabularies.**
`gap::AbstentionCause` and `IdentityAbstentionCause` keep separate label functions and separate
locale keys; **no `From`/`Into` and no shared label function** appears. The existing "Reach" panel
(story 3.8) is not altered.

**AC7 — `Ambiguous` is unreachable, and a test says so.**
**Given** the committed corpus and the shipped pass
**When** the assertion runs
**Then** it asserts that **no `Ambiguous` abstention and no `link_candidate` row exist**, with a
message naming **Epic 6** as the story that will make it fall.
⚠️ This is the ONLY thing this story does about `epics.md`'s third AC (§3b, arbitration 2).

**AC8 — every registered entry naming 5.14 is answered.**
Each entry in §9 is re-read at its line and recorded **CLOSED** or **RE-OWNED** with the measurement
that says which, appended to `deferred-work.md`. **A divergence from §9's prediction is a FINDING**,
written up with its measurement.
**And** 5.10's *"may an operator override the engine?"* is answered in writing.

**AC9 — the trap gate and the corpus are untouched.**
`cargo xtask ci` reports **28 fixture(s)**, seven gates green, and the trap gate still **26
discovered, 15 scored, 0 failures, 11 unanswerable, `passed() == false`**. A green trap gate is a
FINDING.

**AC10 — documents in the same commit.**
`CLAUDE.md`, `docs/project-context.md` and `sprint-status.yaml` carry the outcome, the live test
count and §3's findings. ⚠️ **One live count, in one place.** `epics.md` is not edited.

---

## Tasks / Subtasks

- [ ] **T1 — the read** (AC2, AC3): a repo function counting current engine links, evaluated vs
      abstained, the latter grouped by cause; the token parsed into `IdentityAbstentionCause` with an
      unknown token refused
- [ ] **T2 — the wiring** (AC1): `resolve` after the startup scan; the transaction boundary decided
      and recorded; the refusal logged by name; the guard on the startup path
- [ ] **T3 — the view** (AC2, AC3, AC5, AC6): a second section beside "Reach", its own label function
      and locale keys EN/FR, the floor sentence; the existing panel untouched
- [ ] **T4 — the bans** (AC4): the HTML assertions, each with a mutation that reds it
- [ ] **T5 — the unreachability** (AC7): the assertion naming Epic 6
- [ ] **T6 — the register** (AC8): §9's twelve entries, one at a time, each with its measurement;
      5.10's operator question answered
- [ ] **T7 — prove-to-red** (AC4, AC8): the mutation table with predictions written FIRST, carriers
      read from panic messages, and the command that carried each red named
- [ ] **T8 — gates and documents** (AC9, AC10)

---

## Dev Notes

### Shapes to follow, not reinvent

- **The string seam is story 3.8's** — `Strings`/`strings()` (`page.rs:56-92`) and `cause_label`
  (`:112`), both through `rust_i18n::t!`, locale keys in
  `crates/opencmdb-bin/locales/app.yml` (EN + FR per key). Add keys; do not add a second mechanism;
- **The pass is stories 5.9b–5.11b's.** `resolve` is idempotent, order-independent, and refuses a
  regressing instant and a contradictory `obs_id`. Do not re-implement any of that, and do not widen
  its signature;
- **The view builder is PURE and unit-tested without a database** (`page.rs`'s existing split). Keep
  that seam: the counting and grouping belong on the pure side, the SQL behind the repo.

### Traps, each one measured on this project

- ⚠️ **A green suite says NOTHING about the database.** `DATABASE_URL` is unset locally and DB-backed
  tests pass by `return`ing. This story is DB-heavy: run it against a real `mariadb:10.11.11`, on
  **your own container and port** — review layers sharing one schema fabricate a symptom
  indistinguishable from open issue #38, whose cause is still open. 13306/13307/13308/13311 are
  taken by earlier stories;
- ⚠️ **DB tests must take `crate::DB_TEST_LOCK`** (`main.rs:41`);
- ⚠️ **`cargo test --workspace A B` passes two filters where cargo accepts one**, so nothing runs and
  it reports 0 red for a sound mutation;
- ⚠️ **Commit before the mutation pass, and revert the MUTATION, never the FILE** — story 5.13b's
  repair pass reverted a file and ate a guard written minutes earlier, which is the fourth recurrence
  of that family;
- ⚠️ **Do not read a measurement through a truncation.** 5.13b recorded a divergence that never
  happened because `head -8` hid the evidence of its own full-suite run.

### The tree this story extends, to be RE-MEASURED

`master` at `6ceb284`: **494 tests** (273 bin + 159 core + 62 xtask), seven gates green, 28 fixtures,
26 traps across ten families, trap gate RED. Re-measure rather than quoting.

### What a reviewer will challenge

| challenge | the answer |
|---|---|
| *"the counter is zero"* | §3a — that is why arbitration 1 wires the pass; AC2 asserts a non-empty population |
| *"where is the `Ambiguous` panel?"* | §3b — unreachable, Epic 6 owns the producer, AC7 asserts it |
| *"one cause is not a grouping"* | §7 — the grouping is built, and AC3 asserts what is true today rather than claiming the grouping is exercised |
| *"reuse `cause_label`"* | §4, AC6 — that is the silent bridge `deferred-work.md` forbids |
| *"the bans are styling"* | AC4 — each is asserted over the HTML and carries a mutation |
| *"twenty entries is too many to answer"* | §9 — most are answered by their CONDITION, which is 5.7's precedent, not by building |

### References

- [Source: `epics.md#Story 5.14`] — the four ACs; **not edited**
- [Source: `ux-design-specification.md:912-947`] — the radar, the bans, the floor
- [Source: `crates/opencmdb-bin/src/page.rs:56-120`] — the string seam and `cause_label`
- [Source: `crates/opencmdb-bin/src/main.rs:130-135`, `:249`] — the startup scan and the observation write
- [Source: `crates/opencmdb-bin/src/resolver.rs:200`, `:654-671`] — `resolve`, and why `Ambiguous` is unreachable
- [Source: `crates/opencmdb-bin/src/repo.rs:485`, `:627`, `:660`] — `cause_token`, the column, the read
- [Source: `crates/opencmdb-core/src/identity/cascade.rs:615-643`] — the two variants
- [Source: `deferred-work.md`] — §9's twelve entries, at their own lines

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

---

## Change Log

| date | what |
|---|---|
| 2026-08-11 | Created. **Three arbitrations by Guy**: the story WIRES the pass as well as displaying (without it the counter is a permanent zero); `epics.md`'s AC3 is re-owned to Epic 6 with its unreachability ASSERTED; and the ~20 registered entries are answered one by one on story 5.7's precedent. Two clauses of `epics.md` established unimplementable by code (§3). 🔑 §4 records that the abstention panel ALREADY EXISTS (story 3.8) for a different vocabulary over a different population, and that no bridge between the two may appear silently. |
