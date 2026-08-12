# Story 5.14b: The identity engine's abstention is displayed, grouped by cause — and the counter says what it cannot count

Status: review

<!-- ⚠️ NOT YET VALIDATED. `create-story validate` (two fresh-context agents, fact-check + gap-hunt)
     is MANDATORY on this project before `dev-story` (Guy's decision, Epic 4 retrospective
     2026-07-26 — it overrides the template's "validation is optional" banner).

     ✅ THE THREE OPEN ARBITRATIONS WERE TAKEN BY GUY ON 2026-08-12 (10, 11, 12 in §1). Each is
     recorded WITH the alternative that was refused and why, so a later reader meets a decision
     rather than a preference.

     ✅ VALIDATED BY BOTH LAYERS, 2026-08-12, and every finding is applied. Fact-check: 2 HIGH,
     7 MEDIUM, 5 LOW. Gap-hunt (which BUILT the story to 516 tests against a live mariadb on port
     13315): 5 HIGH, 7 MEDIUM, 4 LOW, and 8 of its own suspicions refuted by measurement.
     🔑 BOTH LAYERS INDEPENDENTLY FOUND `link_columns`, a function that exists nowhere.
     🔴 FOUR GUARDS THIS STORY PRESCRIBED WERE MEASURED GREEN, each with the control that shows it:
     AC3's anti-sum over a zero addend, AC5's own prescribed remedy for a missing locale key, AC8's
     assertion over a table the rollback left empty, and the colour check under an id selector.
     ✅ ARBITRATION 13 TAKEN (§5a): the number ships with an HONEST UNIT — *sightings*, not devices.
     NO ARBITRATION REMAINS OPEN. The story is ready for `dev-story`. -->

## Story

As the operator,
I want to see how many things the identity engine could NOT place, broken down by why,
So that the number measures the product's REACH rather than my debt (FR16b).

**And as the next developer, I want the counter to say, in the surface itself, what it is not
counting** — because story 5.14 measured that a count over current engine links measures *scan
iterations*, and a number whose meaning lives only in a story file is a number the next reader will
believe.

---

## What this story does NOT do

- it does **not** decide the denominator, and it does not pretend to. §5 states the limit in the
  surface; **Epic 6 owns the fix**, because collapsing sightings of one unplaceable thing means
  deciding what makes two sightings the same thing WITHOUT an identity, which is grouping;
- it does **not** implement an `l2-*` rule. The trap gate stays **`passed() == false`**, 26
  discovered / 15 scored / 11 unanswerable;
- it does **not** produce `IdentityAbstentionCause::Ambiguous`. Nothing does (§6), and the story
  ASSERTS that unreachability rather than leaving it as prose;
- it does **not** write a `link_candidate` row, and it does not build the candidate display. FR16's
  *"the abstention shows its candidates"* is **Epic 6's**, with its precondition asserted here;
- it does **not** add a purge. The accumulation (`:2407`'s second half) stays owned by nobody and
  stays registered — **a display story whose acceptance criteria cannot fail on a purge must not be
  the place a purge hides**;
- it does **not** change BEHAVIOUR in `opencmdb-core`. ⚠️ Scoped to behaviour ON PURPOSE: story
  5.13b measured that a bare *"does not touch X"* becomes a reason not to LOOK at X, and sheltered a
  false sentence in `score.rs` for a whole story. Correcting a false sentence in `opencmdb-core` is
  **inside** this scope;
- it does **not** add a dependency, and it does **not** edit `epics.md` (§11 registers instead).

---

## 1. 🔴 Arbitrations

Numbering continues story 5.14's, which ran to 9 — this story inherits four of them and proposes
three.

| # | when | question | decision |
|---|---|---|---|
| 2 | 5.14 contexting | `epics.md`'s `Ambiguous`-shows-its-candidates AC is unreachable | Re-owned to **Epic 6**, with the unreachability **ASSERTED here** (§6). Inherited. |
| 4 | 5.14 validation | the counter AGES: each scan writes a new current abstention link | 5.14 wires and PINS; **5.14b displays and STATES THE LIMIT**. Inherited. |
| 5 | 5.14 validation | the reach section inherits the declared side's visibility gate | **Hoist it out** (§7). A scan-only deployment is the DEFAULT at first boot and never sees it today. Inherited. |
| 9 | 5.14 code review | the `arp_ping` pins are a tripwire, not a barrier | Precedent for §8: **narrow the promise rather than widen the claim**. Inherited. |
| **13** | **validation** | the counter is the shape of two UX bans, not merely short of them | 🔴 **SHIP THE NUMBER WITH AN HONEST UNIT — *sightings*, not devices** (§5a). The ban becomes *stated*, never met. |
| **10** | contexting | there are TWO abstention vocabularies and the page already displays one. One section or two? | 🔴 **TWO SECTIONS, NEVER SUMMED** (§3). |
| **11** | contexting | what does the page do with a persisted `abstention_cause` token it does not recognise? | 🔴 **The reader never fails: count it, label it, render** (§9). |
| **12** | contexting | 5.14's doc says *"story 5.14b is `counted_current_engine_links`'s production consumer"*. Is it? | 🔴 **No — the grouped read subsumes it, and the sentence is corrected here** (§10). |

**10, 11 and 12 were taken by Guy on 2026-08-12.** Each section below records the alternative that
was refused and the reason, because a decision whose alternative is not written down reads as a
preference to the next person.

---

## 2. What is already measured, so nobody re-derives it

Everything here was measured by story 5.14, its two validation passes or its code review. **Do not
re-derive; a surprise is a FINDING.**

- **the pass runs in the shipped binary** since 5.14 (`scan_pass::poll_ingest_resolve`, called from
  `main.rs`'s `spawn_startup_scan`);
- **the shipped ARP/ping connector emits NO MAC, ever** (`arp_ping.rs`, `emitted_facts` /
  `declared_kinds`), and `join` keys on `(L2DomainId, MacAddr)` — so **every scanned observation
  abstains**, with cause `absence_of_proof`. A live deployment's reach section will therefore show
  **one line and one line only**, and its count will equal the number of scanned observations ever
  ingested;
- **the population ACCUMULATES per OBSERVATION, not per abstention**: five runs over ONE host leave
  five current links; two scans carrying the same MAC leave 2 links and 1 interface. ⚠️ **Giving the
  connector a MAC does NOT fix the counter** — this was measured at 5.14's code review and corrects
  the sentence a reader would guess;
- **the naive fix is measured WORSE than the defect**: widening `resolve_within`'s vacate pass to
  close slots of observations it never saw reds four tests, three of them pre-existing `resolver`
  ones, because it **erases a host that missed a single scan**;
- **`counted_current_engine_links` exists** (`scan_pass.rs:180`), is `#[allow(dead_code)]`, and both
  its predicates (`decided_by = 'ENGINE'`, `current_subject IS NOT NULL`) are carried by tests that
  plant an operator row and a superseded row. ⚠️ **Each of those predicates was measured droppable
  with the whole suite green before those rows existed** — fifth recurrence of that family. Any new
  read here inherits the obligation, not the credit;
- ⚠️ **`current_subject IS NOT NULL` is NOT equivalent to `valid_to = OPEN_END`** (5.14's code
  review, registered). A row with `valid_to = OPEN_END` and a NULL `current_subject` is ACCEPTED by
  the CHECK, and this story is the **second** to adopt `current_subject IS NOT NULL` as the
  definition of a human-facing population. Adopt it knowingly, cite the register entry;
- **nothing writes a `link_candidate` row in production** — the only call site passes `&[]`
  (`resolver.rs`, `guard_decision`'s doc), so `Ambiguous` is refused by `guard_decision` and cannot
  be persisted at all.

---

## 3. 🔴 The trap that would waste this story: there are TWO abstention vocabularies, and the page
already displays one

**`_gap_card.html:47-57` already renders a section titled "Reach", with a count and a per-cause
list.** A dev agent that reads the epic and opens the template will conclude the story is nearly
done. **It is not, and the reason is that the section shows a different engine's abstentions.**

| | the section that EXISTS (story 3.7) | what this story adds |
|---|---|---|
| type | `opencmdb_core::gap::AbstentionCause` | `opencmdb_core::identity::cascade::IdentityAbstentionCause` |
| variants | `OutOfPerimeter`, `NoObservedValue`, `ConflictingObservations` | `Ambiguous`, `AbsenceOfProof` |
| the question | *why did comparing a declared FIELD against observations not conclude?* | *why could the engine not PLACE this observation on an interface?* |
| population | declared fields of ONE perimeter entity | observations, across the whole store |
| source | computed per-request by the pure `reconcile` | **read from `identity_link` rows the pass persisted** |
| labels | `cause.out_of_perimeter` … in `locales/app.yml` | new keys — and they must not collide |

`gap/mod.rs:31` already says the two are different types and why. **The trap is not that they look
alike; it is that they are BOTH called "reach" and both counted.**

### 🔴 Arbitration 10 (Guy, 2026-08-12) — TWO SECTIONS, NEVER SUMMED

A second section, **distinctly titled and separately framed**, and **the two counts are never
added**. The shape Guy chose:

```
┌─ RECONCILIATION ────────────────────────────┐
│  Reach                                   2  │
│  What we saw but could not place            │
│    · Out of perimeter                    1  │
│    · No observed value                   1  │
└─────────────────────────────────────────────┘

┌─ IDENTITY ──────────────────────────────────┐
│  Sightings placed 0  ·  not placed     113  │
│  113 sightings not placed, because:         │
│    · No proof of identity              113  │
│                                             │
│  The floor is set by the data, not by the   │
│  engine. And this build counts SIGHTINGS,   │
│  not devices: one machine seen ten times    │
│  counts ten.                                │
└─────────────────────────────────────────────┘
```

🔴 **The unit is arbitration 13's** (§5a): *sightings*, on both sides of the pair, because each scan
mints fresh `obs_id`s and that is what the number is true about.

⚠️ **This is a LAYOUT sketch, not a string specification.** Every visible string goes through the
`t!()` seam in both locales; the wording above is indicative and §5 governs the two limit sentences.
The **structure** is what the arbitration fixes: two frames, two titles, no arithmetic between them.

**Why not one section:** the two counts range over different populations — declared *fields* on one
side, *observations* on the other. Adding them yields a number that denotes nothing. That is D13's
*"an algebra, not a sum"* in miniature, and this codebase already refuses the float version of it
with a gate.

**The alternative that was refused**, recorded because it is defensible: one fused frame with two
labelled sub-groups, closer to the UX spec's mock (§*What the Product Does Not Know*, lines 919-928),
which shows **one** panel — `187 evaluated · 113 not evaluated`. It was refused because two
populations inside one frame invite the reader to add them, and the invitation is the defect. ⚠️ **So
this story diverges from the spec's mock deliberately**, and the divergence is a FINDING to record
rather than a detail to absorb — the UX spec is not edited here, and §11 registers it.

⚠️ **`locales/app.yml` gains keys under a distinct prefix.** The existing keys are `page.reach`,
`page.reach_hint`, `page.nothing_unplaced`, `cause.*`. Reusing `cause.*` for identity causes would
put two vocabularies in one namespace and make a future rename silently mislabel a count.

---

## 4. The read, and the obligation it inherits

The grouped read is the story's only new SQL. Shape:

```sql
SELECT outcome, abstention_cause, COUNT(*)
FROM identity_link
WHERE decided_by = 'ENGINE' AND current_subject IS NOT NULL
GROUP BY outcome, abstention_cause
```

**Two `WHERE` predicates and two `GROUP BY` columns** — not "four predicates", and the distinction is
not pedantry: a `WHERE` no test can red is DECORATION, while a `GROUP BY` no test can red is **a line
the AC demands and nothing produces**. Different failure, same obligation. Each of the four must be
carried by a row a test CREATES, and the mutation table (§12) names each:

| clause | the row that carries it | what happens without it |
|---|---|---|
| `decided_by = 'ENGINE'` | an OPERATOR row | a human's decision is reported as the engine's reach |
| `current_subject IS NOT NULL` | a row whose `current_subject` has been NULLed — the existing idiom (`scan_pass.rs`'s `a_superseded_link_is_not_counted`). ⚠️ **It leaves `valid_to = OPEN_END`**, so it is not superseded by any `valid_to`-based definition; that is §2's registered non-equivalence, and **do not call the resulting row "superseded" in a doc comment** — the existing test does, and `deferred-work.md` records that its doc now stands in the way of the DDL repair | the number grows with every re-scan even where the engine settled |
| the `outcome` grouping | a `match` row **and** an `abstained` row | placed rows land in the not-evaluated column, or vice versa |
| `abstention_cause` grouping | **two rows with DIFFERENT causes** | one line where the AC demands one line PER CAUSE — and this is the AC's own subject, so it must not be carried by a single-cause fixture |

⚠️ **`sqlx 0.9` refuses a `format!`ed SQL string** (`SqlSafeStr` is `&'static str` only) — that part
stands. 🔴 **But do NOT copy `scan_pass.rs:181`'s `AssertSqlSafe`**: a `&'static str` literal compiles
without it, and `repo.rs`'s own reads — `count_identity_links` among them — use a plain literal.
Measured at validation. Follow the target file's idiom.

🔑 **The read's home is `repo.rs`, and the story's open question is ANSWERED**: its code half is
**1088 lines** (the first `#[cfg(test)]` is at `:1089`) against the 2000 ceiling. The 2626 total is
not the number the gate counts.

🔑 **`no_match` is UNPRODUCIBLE through `resolve`, and it was measured**: `placement_decision` only
asks `decide_pair` about two members of one `join` group, which share a key by construction, so
`l1-distinct-mac` never fires. **Pin it with a test** and let the read still handle the token — the
read is a display, not a proof.

🔑 **Which `outcome` values the production pass can write is ALREADY ANSWERED in the tree — cite it,
do not re-derive it.** `resolver.rs:962-965`: *"the RULE is knowable in advance (`l1-exact-mac` is the
only rule a placement can carry: `l1-distinct-mac` rides `Disqualifying`, which `decide` turns into
`NoMatch`, **which this pass never writes**)"*. So today the pass writes `match` and `abstained`, and
**`no_match` is unreachable through it**. ⚠️ **Confirm that by running it** before relying on it — the
sentence is a doc comment, and this project's rule is that a doc comment is a claim like any other —
but start from the citation rather than from a blank page.

**The read still handles `no_match`**: it is a display, not a proof, and the column's domain is the
DDL's. What must NOT be written is a doc sentence claiming the display is exhaustive over outcomes
the pass produces *when nothing measured which those are*.

⚠️ `repo::outcome_token` (`repo.rs:274-281`), bound by `insert_identity_link` (`repo.rs:501`), is what
maps a `Conclusion` to its persisted token — an exhaustive `match` with no `_` arm, so a new variant
is `error[E0004]` there.

⚠️ **`abstention_cause` and `outcome` are coupled by DDL**: `identity_link_rule_xor_cause`
guarantees `abstention_cause IS NOT NULL` exactly when `outcome = 'abstained'`, so a `NULL` cause on
an abstained row is impossible by DDL. **Do not add a defensive branch for it and then claim it is
guarded** — cite the constraint instead.

---

## 5. 🔴 The denominator, stated in the surface — this story's real deliverable

**The counter this story displays measures scan iterations, not reach.** That is measured (§2), it is
not repairable here (§2's fourth bullet), and Epic 6 owns it.

**So the surface says so.** Not the story file, not a code comment — the rendered page, in the
operator's language, through the `t!()` seam.

The sentence must be **descriptive, never apologetic**, and it must survive the UX spec's tone rules:
the product may state a limit of its own reach; it may not apologise for the operator's network and
it may not read as a defect report. The UX spec's own model sentence is the one to imitate:

> *"The floor is set by the DATA, not by the engine (PRD FR9): hostname — one of grouping's three
> signals — is unusable on nearly half of known clients. No amount of engine quality recovers a
> signal the source never sent."*

**Two facts belong in the surface, and they are different facts:**

1. **the measured floor** (`epics.md`'s fourth clause, FR9 + NFR30): the abstention rate is bounded
   below by DATA AVAILABILITY, not by engine quality;
2. **this build's own limit** (arbitration 4): the count is per OBSERVATION and does not collapse
   repeated sightings, so it grows with scan count.

⚠️ **Do not fuse them into one sentence.** The first is a permanent property of the problem; the
second is a property of *this build* that Epic 6 removes. A reader who meets them fused will carry
the temporary one as permanent — the exact failure this project has caught in six documents.

### 5a. 🔴 Arbitration 13 (Guy, 2026-08-12) — this counter is the shape two UX bans name, so its UNIT is made true

**Found by the validation pass, and it is sharper than "the denominator is undecided".** The UX
spec's first hard ban (`ux-design-specification.md:1280`) is **"No badge, no growing counter"** —
*"'47' is not information, it is a reproach disguised as a number"* — and `epics.md:1704` demands
*"after six months of inaction it reads the same number"*. **This build's number provably grows while
the operator does nothing**, because the scanner keeps scanning: five runs over one host leave five
links (§2).

So the ban is not merely unmet in the way a future story will meet it. **The counter this story
displays is the shape the ban names**, and §5's limit sentence explains that rather than removes it.

🔴 **DECIDED (Guy, 2026-08-12): ship the number with an HONEST UNIT.** The counter counts
**sightings** — *constats* — and says so, on **both** sides of the pair. Not devices, not interfaces:
each scan mints fresh `obs_id`s and the population is observations (§2), so *sightings* is what the
number is true about.

**What follows, and it is a naming decision that reaches the template, the locale keys and the
tests:**

- both columns carry the unit — *sightings placed* / *sightings not placed*, and the per-cause lines
  are counts of sightings;
- 🔑 **the growth then reads as a fact about scanning rather than as a backlog**: a number that rises
  because the product looked many times is the radar's range, not the operator's debt. **That is what
  makes the unit the arbitration and not a caption.**
- ⚠️ **The unit is TEMPORARY by construction.** Epic 6 gives the population an identity, at which
  point the honest unit changes and the locale keys change with it. Say so where the keys are
  defined, so the day it changes nobody wonders whether the old word was a mistake.

**The two alternatives that were refused**, recorded because both were defensible:

1. **the number as-is with only a limit sentence** — closest to the UX mock and to the epic's letter,
   refused because it leaves a growing, unqualified number on the page for as long as Epic 6 takes,
   and a sentence beside a number does not survive the reader's first glance;
2. **no total at all**, only the per-cause lines — the map without the score. Refused because it
   costs `epics.md`'s first clause its *"count NOT evaluated"* and would need that clause re-owned;
   ⚠️ **it would also have made AC3 simpler** (no total, nothing to sum), and that simplification is
   given up knowingly.

⚠️ **The ban does not become MET; it becomes STATED.** A true unit does not stop the number growing.
§11 registers it, and AC6 must not be read as covering it.

---

## 6. `epics.md`'s `Ambiguous` clause — discharged by an ASSERTION, not by prose

**The clause** (`epics.md:1706-1708`, quoted in full because a paraphrase typeset as a quote is how a
requirement quietly loses a line):

> **Given** an abstention whose cause is `Ambiguous`
> **When** it is displayed
> **Then** its candidates and their evidence are shown from the persisted `link_candidate` rows
> (FR16) — the abstention explains itself.

**It is unreachable, by three independent mechanisms**, and this story asserts the unreachability
rather than implementing a branch nothing can enter:

1. `Verdict::Supports` and `Verdict::Opposes` **have no producer** — L1 emits only `Decisive`,
   `Disqualifying`, `Neutral` (`cascade.rs`'s module doc). `Ambiguous` is what `decide` returns for
   verdict combinations that need them;
2. the resolver passes `&[]` as `candidates_for_link` at its only call site, so `guard_decision`
   **refuses** an `Ambiguous` decision with `Constraint("ambiguity_without_candidates")` — an
   ambiguity cannot be persisted at all today;
3. nothing calls `repo::insert_link_candidate` outside tests.

**What ships here:** a test that asserts (1) — that the production pass produces no `Ambiguous` — so
the day Epic 6 gives `Supports`/`Opposes` a producer, **something reds and names this clause as due**.

⚠️ **The assertion must be on the PRODUCTION path, not on the enum.** Asserting *"the enum has two
variants"* is a tautology a refactor breaks for the wrong reason; asserting *"a pass over a realistic
slice writes no row whose `abstention_cause` is `ambiguous`"* is a claim about the code. ⚠️ And it is
**weaker than it looks** — it is bounded by the slice the test hands it. Say which slice, and do not
write *"the engine never produces `Ambiguous`"* when what was measured is *"this slice did not"*.

---

## 7. The hoist (arbitration 5)

`_gap_card.html:2` opens `{% if view.has_entity %}`, `:58` is its `{% else %}` (the no-declared-record
empty state) and `:61` the `{% endif %}`. **The reach section is inside that gate.** So it is invisible in exactly the
deployment that most needs it: **a fresh install that has scanned and declared nothing** — the
default at first boot.

**The identity reach section renders whenever there is something to say about it**, independently of
whether a declared entity exists.

⚠️ **`build_view` returns EARLY** (`page.rs:188-198`) with a fully-zeroed `ReconciledView` when no
entity is chosen. Any field the hoisted section reads must be populated **before** that early return
or on both sides of it — a field filled only in the late path is a field the hoist silently zeroes,
and the test that catches it is *"a store with abstentions and NO declared row still shows them"*.

⚠️ **`build_view` is PURE and unit-tested without a database** (`page.rs`'s module doc says so, and
three tests depend on it). **Keep it pure**: the grouped counts arrive as an ARGUMENT, and
`reconcile_view` — the impure edge — does the read. A dev agent that reaches for the pool inside
`build_view` breaks the story's own testing surface.

---

## 8. The Dignity bans, as assertions rather than intentions

`epics.md`'s second clause: *"it does not redden, does not grow bold, carries no gauge and no badge,
and does not age visibly: after six months of inaction it reads the same number, in the same grey."*

🔴 **Five rows, five DIFFERENT strengths, and the strength column IS the deliverable.** An earlier
draft of this section said *"four of those are testable"* and then gave a mechanism for five without
ever naming the four — an instruction the story issued to itself and did not discharge. **Nothing
here is claimed as met beyond the strength written beside it.**

| ban | how it is carried | honest strength |
|---|---|---|
| **does not age** | the view builder takes **no clock**; render the same store at two instants → identical HTML | 🔴 **THIS IS NOT `epics.md`'s BAN, and the gap is measured in this same story.** The ban is *"after six months of inaction it reads the same number"* — a statement about the DISPLAYED NUMBER over calendar time. §2 and §5 measure that this build FAILS it: the scanner keeps scanning while the operator is inactive, so the store itself grows (five runs over one host → five links). What ships is the **clock-freedom of the view builder**, a strictly weaker property wearing the ban's name, plus §5's stated limit. ⚠️ M10 tests the weaker property — *a mutation named for one thing and applied to another measures the other thing*, and this row exists so that sentence is not discovered at code review for the fifth time. **The ban itself is OPEN, owned by Epic 6, and §11 registers it** |
| **no red** | `app.css`'s token set carries **no red at all** — `--attention: #f0f4fa`, commented *"severity by luminosity + weight, never hue (no red)"*. So *"does not redden"* is carried by **the token set**, not by anything this story writes | ⚠️ **Held by the palette, not by this story.** What this story CAN check is narrower and must be named as such: **this section's own rules resolve to `--muted`/`--text`/`--border` and never to `--accent`** (the amber reserved for the document action — amber, not red). 🔴 **And the mutation must target THIS story's selector**: `.abstentions .cause` (`app.css:134`) belongs to the story-3.7 reconciliation section, which arbitration 10 exists to keep separate — mutating it would test the section this story is not adding. 🔴 **Strength downgraded by measurement: this is a check on a RULE's text, not on a resolved colour.** `#gap-card .abstentions .cause { color: var(--accent) }` really does recolour this section (an id selector beats `.identity`'s 0-3-0) and leaves **all 515 tests green**. The *"an assertion over CSS is an enumeration"* caveat this story reserved for `font-weight` applies **identically to colour** |
| **no gauge, no percentage** | the rendered HTML contains no `%`, no `<progress>`, no `<meter>` in the section | real |
| **no badge, no growing counter** | the count is not attached to a nav item or icon | 🔴 **The ban's TEST is about badges; the ban's NAME is *"no growing counter"*, and this story's deliverable is a counter this project has measured as growing.** The badge half is met and is nearly vacuous (there IS no nav). ⚠️ **The growing-counter half is NOT met**, and whether it can be met before Epic 6 is arbitration 13 (§5a). Do not let AC6 be read as covering it |
| **does not grow bold** | — | ⚠️ **Weakest of the five.** `font-weight` is CSS, and a general assertion over CSS is an enumeration. Carry it as one specific check on the section's own rules, and **state that it is a check on those rules and not a property of the page** |

🔑 **The transferable rule, and it is why this table has a "strength" column at all**: story 5.12
spent a day discovering that a gate whose promise is wider than its mechanism is a false sentence
even while it passes, and story 5.13b that a promise of non-modification shelters false sentences.
**Write the narrow true form for each ban.**

---

## 9. 🔴 Arbitration 11 (Guy, 2026-08-12) — an unrecognised token must not kill the page

The read returns `abstention_cause` as a **string from the database**. Displaying it needs a
parse-back, and the gap-hunt of story 5.14 measured the shape of the hole:

> a new `IdentityAbstentionCause` variant breaks the LABEL (`error[E0004]` on an exhaustive match)
> and breaks the WRITER (`repo::cause_token`, also exhaustive), but **never** a
> `cause_from_token`-style reader, whose `_ => None` is STRUCTURAL. A variant added with the minimal
> repair therefore persists a token the reader refuses.

**And the consequence is disproportionate**: `page.rs`'s handlers turn any error into
`500 INTERNAL_SERVER_ERROR` for the WHOLE page. One unrecognised token in one row would take down
the gap display, the declared side and everything else.

**Decided: the reader never fails.**

- an unrecognised token is **counted** — the total must not silently shrink, which would be the
  counter lying by omission;
- it is **displayed on its own line**, labelled through the i18n seam as an unrecognised cause,
  carrying the raw token;
- the page **renders**.

**The guard:** a test plants a row with an invented `abstention_cause` and asserts the page renders
AND the count includes it. **This is the one test that turns the gap-hunt's structural finding into a
measurement**, and its mutation (make the reader return `Err` on an unknown token) reds it.

⚠️ **The raw token is displayed, so it goes through the privacy floor's world**: it is an ASCII
column constrained to `VARCHAR(32)` and written only by `cause_token`, so it carries no address — but
it is now operator-visible, and it must be **HTML-escaped** like any other value. Askama escapes by
default; do not reach for `|safe`.

**The alternative that was refused:** a DDL `CHECK (abstention_cause IN ('ambiguous',
'absence_of_proof'))`, which makes the reader total by construction because an unknown token cannot
exist. Refused for two reasons, and the second is the stronger: it is **DDL in a display story**; and
it **moves the failure from the display to the WRITE** — a variant added by a future story would then
be refused at insertion, i.e. the identity pass would start failing rather than the page rendering an
unfamiliar label. 🔑 *A display story may not be the place a write starts failing.*

⚠️ **The refusal is not free and the cost is stated rather than implied**: without the CHECK the
column's domain is enforced only by `cause_token` being the sole writer, which is a property of
today's code and not of the schema. **Registering it as the real closure** — on story 5.12's
precedent, where voie B's `GRANT` was registered rather than implied by the tripwire that shipped —
is §11's business, and the story does not claim the domain is guarded.

---

## 10. 🔴 Arbitration 12 (Guy, 2026-08-12) — 5.14's sentence about its own dead code is false, and is corrected here

`scan_pass.rs:178` reads: *"Story 5.14b is its production consumer."* It is a **prediction about this
story**, shipped as a statement, in the doc of a function marked `#[allow(dead_code)]`.

**Measure it rather than inherit it.** `counted_current_engine_links` is
`SELECT COUNT(*) … WHERE decided_by = 'ENGINE' AND current_subject IS NOT NULL` — an **UNGROUPED**
total over exactly the rows §4's read groups. ⚠️ **Not *"unfiltered"***: that word is spent, in
`scan_pass.rs:167` and in `deferred-work.md`, on `repo::count_identity_links`, the one read with no
`WHERE` at all — and the whole point of those sentences is to keep the three apart. §4's grouped read
returns the same rows partitioned, so **the total is a sum of the groups** and calling both would
issue two queries for one number.

**Decided: the grouped read subsumes it.** `counted_current_engine_links` stays as the instrument of
5.14's four pins, keeps its `#[allow(dead_code)]`, and **its doc sentence is corrected in this
commit** to say what is true — that its consumer is the pins that carry the two predicates, and that
the human-facing count is the grouped read's.

🔑 **This is the family this project has caught in six consecutive stories: a doc that describes a
state the code has passed.** The correction is cheap here and expensive in six months.

**The alternative that was refused:** keep it as a SECOND ORACLE — the page asserts that the groups
sum to the unfiltered total, which is a deliberate redundancy in this codebase's sense (`fixtures.rs`'s
`expected()`, `score.rs`'s two representations). Refused because the two reads would run **inside one
request against a live store**, so a disagreement between them means a concurrent write, not a
defect: the oracle would be **flaky by construction** rather than load-bearing. ⚠️ **The same
redundancy IS legitimate inside a single transaction in a test**, and the dev may write it there;
what was refused is putting it on the request path.

⚠️ **The correction is a MEASUREMENT, not a transcription.** Before editing the sentence, verify that
the grouped read really does return every row `counted_current_engine_links` counts — the two
`WHERE` clauses must be identical, and the grouping must not drop a row through a `NULL`
`abstention_cause` on a non-abstained row. **If they diverge, the divergence is the finding** and
this arbitration is re-opened rather than forced.

---

## 11. The register

Append to `deferred-work.md` (never rewrite a bullet):

| entry | disposition here |
|---|---|
| **the denominator** (`Owner: story 5.14b and Epic 6`) | 🔴 **Half-closed and the half is named**: 5.14b STATES the limit in the surface (§5); **Epic 6 fixes it**. The entry stays open with its owner reduced to Epic 6 |
| **`count_identity_links` has no production caller** (`Owner: story 5.14b`) | Answered by §10's measurement, whichever way it goes. ⚠️ **Record why all THREE exist so nobody unifies them, and get the word right**: §10's two reads are both FILTERED on `decided_by`/`current_subject` and differ only by grouping; `count_identity_links` (`repo.rs:885`) is the **only unfiltered one** — a bare `SELECT COUNT(*)` with no `WHERE`. `scan_pass.rs:167` uses *"unfiltered"* precisely to name that difference, so this story must not spend the same word on *"ungrouped"* |
| **`:2407`'s PURGE half** (`Owner: unassigned`) | ⚠️ **Untouched, and deliberately.** This story makes the accumulation VISIBLE, which raises the pressure without discharging it. Say so — a visible defect is not a fixed one |
| **`current_subject IS NOT NULL` ≠ `valid_to = OPEN_END`** (`Owner: unassigned`) | ⚠️ **RE-STATED, not closed.** This story is the second to adopt that predicate as a human-facing population's definition, which raises the cost of the DDL repair. Record the increment |
| **the `Ambiguous` clause** | Re-owned to Epic 6 **with §6's assertion as its tripwire**, so the day a producer arrives the clause is named by a red test rather than by memory |
| **NEW — two UX bans are NAMED by this counter, not merely unmet by it** (§5a) | *"No badge, no growing counter"* (`ux…:1280`) and *"after six months of inaction it reads the same number"* (`epics.md:1704`). The number grows while the operator is inactive, measured. **Arbitration 13 makes the unit TRUE (*sightings*) — it does not make the ban met**, and the entry says so in those words. **Owner: Epic 6** |
| **NEW — the unit *sightings* is TEMPORARY and its locale keys change with Epic 6** | The day grouping gives the population an identity, the honest unit is no longer a sighting. Registered so the rename is met as a scheduled consequence rather than as a correction of a mistake. **Owner: Epic 6** |
| **NEW — the `abstention_cause` column has no domain in the SCHEMA** | Arbitration 11 refused the DDL `CHECK` for this story (it moves the failure from the display to the WRITE). ⚠️ **Register it as the real closure rather than let the tolerant reader imply it** — story 5.12's precedent, where voie B's `GRANT` was registered rather than implied by the tripwire that shipped. Today the domain is held by `cause_token` being the sole writer, which is a property of the code and not of the schema. **Owner: unassigned** |
| **NEW — this story diverges from the UX spec's mock, deliberately** | The spec (lines 919-928) shows ONE panel; arbitration 10 ships TWO frames because two populations in one frame invite a sum. `ux-design-specification.md` is **not edited**. Register the divergence so it is met as a decision and not as drift. **Owner: Epic 6**, which will revisit this surface when the denominator becomes collapsible |

⚠️ **Story 5.14's §8 had two rows that were never appended** (`:2700` and the page-less deployment) —
caught by its code review. **Append this table by reading `deferred-work.md` afterwards and checking
each row landed**, rather than by trusting the edit.

---

## Acceptance Criteria

**AC1 — the identity engine's abstentions are displayed, grouped by cause, one line per cause.**
Read from the persisted `identity_link` rows (§4), not recomputed. **The fixture carries at least two
DIFFERENT causes**, so *"one line per cause"* is measured rather than coincidental.
_Reddened by: M1, M2._

**AC2 — the read's every predicate is carried by a row a test creates.**
An OPERATOR row, a SUPERSEDED row, a `match` row and two differing abstention causes — §4's table.
⚠️ Each of these was measured droppable-with-the-suite-green in story 5.14 before its rows existed.
_Reddened by: M3, M4, M5._

**AC3 — the evaluated and not-evaluated populations are shown side by side, and never summed
with the reconciliation section's counts** (§3, arbitration 10).
🔴 **The fixture must make BOTH populations non-zero, and the two counts and their sum must be THREE
DISTINCT NUMBERS.** Measured at validation: with the natural fixture for a story about the identity
section — reconciliation count **0** — the summing mutation applied in the direction where the
identity section absorbs the other left **all 515 tests green**. *An anti-sum guard over a zero
addend asserts nothing.*
⚠️ **And AC3 is only exercisable where a declared entity exists**: the reconciliation section stays
inside `{% if view.has_entity %}` (only the identity one is hoisted, §7), so on a fresh install there
is one frame and "never summed" has nothing to be false about. Write the test on a store that has
both.
🔴 **ONE vocabulary, and arbitration 13 fixes it: *sightings placed* / *sightings not placed*.**
Earlier drafts carried two — §3's sketch said *Placed / Not placed*, this AC said *evaluated /
not-evaluated* — and they diverge on `no_match`, which is evaluated AND carries an `interface_id`.
⚠️ **`no_match` is unproducible through `resolve`** (§4, measured), so the divergence is latent rather
than live today; fix the words anyway, in the template, the locale keys and the tests, because a
latent ambiguity is what Epic 6 will trip on.
_Reddened by: M6, M6b._

**AC4 — the section is visible without a declared entity** (§7, arbitration 5).
**Given** a store holding abstentions and NO declared row, **when** the page renders, **then** the
section is there. ⚠️ And `build_view` stays PURE — the counts arrive as an argument.
🔴 **AC4 also covers the EMPTY state, which the first draft left uncovered** — measured: deleting the
*"nothing seen yet"* key from both locales left **all 516 tests green**, because the branch renders
and nothing asserts on it. **That is the fresh-install case, i.e. the exact deployment the hoist
exists for**: a store with no abstentions AND no declared row must show the section saying so.
_Reddened by: M7, M8b._

**AC5 — the counter states its own limit, in the surface** (§5).
Two facts, **not fused**: the measured floor (data availability, FR9/NFR30) and this build's
per-observation counting. Through the `t!()` seam.

🔴 **A missing locale key is a SILENT KEY ECHO** — `t!` returns the literal `"identity.floor"`, no
compile error, no panic. **So reading the rendered HTML is a TAUTOLOGY**: measured at validation,
deleting the key from BOTH locales and asserting on the render left **all 515 tests green**, because
the echoed key is in the HTML. ⚠️ The assertion is `assert_ne!(resolved, key)` — an earlier draft
prescribed the render form as its remedy and the remedy was the defect.

🔴 **AC5 is SPLIT, and the reason is that its old wording drove the implementation into a
process-global.** `strings()` reads the global locale; the only legitimate `set_locale` caller is
`main.rs:95`, and **every test in this codebase uses the per-call `locale =` override**. A literal
*"both locales present in the surface"* needs `set_locale`, which is process-wide: measured across
four consecutive full runs, **2 or 3 red out of 290, varying** — and one casualty was the ageing
guard itself, reddened by a locale with no clock anywhere.

- **AC5a** — both locales carry both keys, asserted through the per-call override, `assert_ne!`
  against the key;
- **AC5b** — the default locale's text reaches the surface, asserted on the render.
- ⚠️ **`set_locale` is FORBIDDEN in tests.** A test that calls it makes the suite order-dependent.

_Reddened by: M8. ⚠️ M8b measures a SECOND key nothing asserts on — see AC4._

**AC6 — the Dignity bans are carried by the checks §8 names, at the strength §8 names.**
No red, no gauge, no percentage, no clock in the view builder. ⚠️ **The doc states which of the five
are true by construction and which are measured** — a tripwire is not a barrier (arbitration 9).
_Reddened by: M9, M10._

**AC7 — an unrecognised `abstention_cause` token is counted and displayed, and the page renders**
(§9, arbitration 11). The total does not silently shrink.
_Reddened by: M11._

**AC8 — `Ambiguous` is asserted unreachable on the PRODUCTION path** (§6), naming the slice the
assertion is bounded by, and re-owned to Epic 6.
🔴 **The PREMISE assertions are mandatory, not optional, and the measurement is why.** Under the
mutation that makes the production path emit `Ambiguous`, `guard_decision` refuses the decision, the
pass ROLLS BACK and writes nothing — so the prescribed assertion (*"no row whose cause is
`ambiguous`"*) passes **over an empty table**. Measured: 19 other tests red, **this one GREEN**.
The test must first assert **that the pass RAN and that `links_written > 0`**, then that no row
carries `ambiguous`. ⚠️ And once it does, the red is `.expect()`-carried on the premise and never
reaches §6's own assertion — **say so in the mutation row rather than claiming the assertion works**.
_Reddened by: M12, on its premise._

**AC9 — gates and corpus untouched.** `cargo xtask ci`: 28 fixtures, seven gates green; trap gate
**26 discovered, 15 scored, 0 failures, 0 wrong-rule, 11 unanswerable, `passed() == false`**.
`opencmdb-core` behaviour unchanged.

**AC10 — the register** (§11), appended and then **re-read to check each row landed**.

**AC11 — documents in the same commit, and ONE live count in ONE place.**
🔴 `docs/project-context.md`'s test count has now drifted two stories **three times** (commit
`5046cca`, issue registered for the retrospective). Do not add a second live count anywhere; update
the one that exists.

---

## Tasks / Subtasks

- [x] **T0 — arbitrations 10, 11 and 12** — taken by Guy, 2026-08-12. ⚠️ Arbitration 12 carries a
      *"re-open rather than force"* clause: §10's read-equivalence check runs BEFORE the doc sentence
      is edited, and a divergence re-opens the arbitration instead of being smoothed over.
- [x] **T1 — the grouped read** (AC1, AC2): the query, the four carrier rows, `AssertSqlSafe`
- [x] **T2 — the view and the hoist** (AC1, AC3, AC4): `build_view` stays pure, counts as an
      argument, section outside the `has_entity` gate, both sides of the early return
- [x] **T3 — the template, the CSS and the locales** (AC1, AC3, AC5, AC6): a distinct key prefix,
      `en` + `fr`, no `|safe`
- [x] **T4 — the limit sentence** (AC5): two facts, not fused
- [x] **T5 — the Dignity checks** (AC6): at §8's strengths, with the strengths WRITTEN
- [x] **T6 — the unrecognised token** (AC7): plant it, render, count it
- [x] **T7 — the `Ambiguous` tripwire** (AC8): on the production path, slice named
- [x] **T8 — prove-to-red** (AC1–AC8): M1–M12, predictions FIRST, carriers read from each panic
      message one by one, the command that carried each red named
- [x] **T9 — the register** (AC10), then re-read the file
- [x] **T10 — gates and documents** (AC9, AC11)

---

## 12. Prove-to-red

**Predict first, then measure, and record the divergence as a finding.** ⚠️ Story 5.14 recorded
*"two divergences"* where its own table showed four, twice because a mutation was named for one thing
and applied to another, and twice because a validation worktree's figures were copied in as
predictions about this tree. **A number inherited from another tree is not a prediction about this
one.**

🔴 **The rightmost column below is a VALIDATION WORKTREE's measurement, not a prediction about your
tree, and the distinction is not academic**: story 5.14 copied a validation worktree's figures into
its own table as predictions and shipped two false rows. **Re-measure every one.** The figures are
here because they carry the SHAPE of each red — which assertion, which carrier — not because they are
your counts.

⚠️ **A driver trap measured during validation**: matching `^error(\[|:)` to classify a red counts
cargo's own `error: test failed` trailer and reports a compiler-carried red on **every** mutation
that reds anything. Anchor on `^error\[E[0-9]+\]`.

| id | mutation | predicted | measured elsewhere (RE-MEASURE) |
|---|---|---|---|
| **M0** | control, no mutation | 0 | 0 / 515 |
| **M1** | collapse the grouping — return one total instead of per-cause rows | RED — AC1's *"one line per cause"* | **3**, all named assertions; the third is an end-to-end page test |
| **M2** | group by `outcome` only, dropping `abstention_cause` | RED — and this is the one a single-cause fixture would MISS, which is why AC1 demands two causes | 1, named assertion |
| **M3** | drop `decided_by = 'ENGINE'` | RED via the operator row | 1, named assertion |
| **M4** | drop `current_subject IS NOT NULL` | RED via the NULLed-`current_subject` row | 1, named assertion |
| **M5** | count `match` rows in the not-evaluated column | RED via the `match` row | 2, named assertions |
| **M6** | add the two sections' counts together | RED — AC3 | 1, named assertion |
| **M6b** | the same sum, **the other direction** (identity absorbs reconciliation) | RED — ⚠️ **and this is the direction that exposes a vacuous fixture** | 1 — but **0 / 515 GREEN** on a zero-reconciliation fixture. **That control is mandatory**, and AC3 now demands the fixture that defeats it |
| **M7** | put the section back inside `{% if view.has_entity %}` | RED — AC4 | **9**: 8 named assertions + 1 `.expect()`. §7's early-return trap is load-bearing, measured |
| **M8** | delete the limit sentence's locale key **from both locales** | RED — 🔴 **and the miss is a SILENT KEY ECHO**, measured: no compile error, no panic, `t!` returns the literal key | 1 — and **0 / 515 GREEN** if the assertion reads the rendered HTML instead of `assert_ne!(resolved, key)`. The first draft prescribed the green form as its remedy |
| **M8b** | delete the *"nothing seen yet"* key from both locales | RED — AC4's empty state | **0 / 516 GREEN** before the empty-state test existed. This mutation is why AC4 grew that clause |
| **M9** | point **THIS story's own section selector** at `var(--accent)` | RED. ⚠️ **Not `.abstentions .cause`** — that is the story-3.7 section's selector (`app.css:134`) and mutating it measures the section this story is not adding. And label the red honestly: it checks *"never `--accent`"*, not *"does not redden"*, which the palette carries | 1, named assertion |
| **M9b** | the generic `.abstentions .cause` → accent | GREEN, and **not a hole** — `.identity .abstentions .cause` (0-3-0) wins on specificity | 0, no-op. ⚠️ **Keep it SEPARATE from M9c** — fusing a refuted suspicion with a measured hole lets the first inherit the second's credit |
| **M9c** | `#gap-card .abstentions .cause` → accent | 🔴 **GREEN, and it IS a hole**: an id selector beats the rule the check reads | **0 / 515 GREEN.** This is what downgrades §8's "no red" strength |
| **M10** | give the view builder a clock and vary the output with it | RED. 🔴 **It carries the view builder's clock-freedom and NOT `epics.md`'s ageing ban** — §8's first row measures the difference, and §5a owns the ban | **3** — so §8's *"expected to be the only thing carrying that ban"* is **refuted**: two of the three are ordinary count assertions |
| **M10b** | a clock reaching a rendered string **no count assertion reads** | RED ×1 — this is the tripwire's real and narrower reach | 1, named assertion |
| **M11a** | make the reader DROP an unknown token silently | RED — AC7's *"the total must not shrink by omission"* | 3, all named assertions |
| **M11b** | make the reader PANIC on an unknown token | RED — reachability | 3, **all `panic!`-carried** (the mutation's own), so it proves the path is reached and **not** that the assertions work |
| ~~M11~~ | ~~return `Err` on an unknown token~~ | 🔴 **NOT EXECUTABLE against arbitration 11's design**, which forbids a `Result`-returning reader. Replaced by M11a and M11b — *a mutation that contradicts the design it tests measures nothing* | — |
| **M12** | make the production path emit an `Ambiguous` conclusion | RED — AC8. 🔴 **The caveat was right and incomplete**: `guard_decision` refuses it, the pass ROLLS BACK, and §6's assertion then passes over an EMPTY table. The premise assertions are mandatory | **20 red — and AC8's own test GREEN** without the premise. With it, the red is `.expect()`-carried on the premise and never reaches §6's assertion |

---

## Dev Notes

### Traps, each measured on this project

- ⚠️ **DB-heavy.** Your OWN `mariadb:10.11.11` container on your OWN host port. **13306 is held by an
  unrelated container** and 13307–13314 are earlier stories' and the validation worktrees'. Pick a
  free one and record it.
- ⚠️ **`DATABASE_URL` is unset locally**, so every DB-backed test passes by `return`ing and the
  suite reports the same counts either way. **The witness that they ran is the timing**: the `bin`
  suite takes ~0.06 s without a database and ~4 s with one.
- ⚠️ **DB tests take `crate::DB_TEST_LOCK`** (`main.rs:42-43`; `:41` is the `#[cfg(test)]` attribute).
- ⚠️ **`cargo test --workspace A B` passes two filters where cargo accepts one** — 0 red for a sound
  mutation, and story 5.13 filed it as a confirmation before catching it.
- ⚠️ **Never read a measurement through a truncation** (`head -8` on a mutation driver's output cost
  story 5.13b a false claim in five documents).
- ⚠️ **Commit before mutating; revert the MUTATION, never the FILE** — a `git checkout -- <file>`
  ate a guard written minutes earlier during 5.13b's repairs.
- ⚠️ **sqlx 0.9 refuses a `format!`ed SQL string** (`SqlSafeStr` is `&'static str` only).
- ⚠️ **Askama escapes by default; do not reach for `|safe`** on the raw token (§9).
- ⚠️ **`_gap_card.html` is included by `gap.html` AND rendered alone as the HTMX fragment**
  (`page.rs`'s `GapFragment`). A section added to the card appears in both; a section added to
  `gap.html` appears only in the full page and **vanishes on the first HTMX refresh** — a defect no
  full-page test can see.

### The tree this story extends, to be RE-MEASURED

`master` at `5046cca`: **502 tests** (281 bin + 159 core + 62 xtask), seven gates green, 28
fixtures, 26 traps across ten families, trap gate RED at 26/15/11. ⚠️ **Re-measure rather than quote**
— this figure has drifted three times in this project, most recently two stories deep.

### Files this story touches

| file | what |
|---|---|
| `crates/opencmdb-bin/src/page.rs` | the view models, the pure builder's new argument, the impure read call, the label |
| `crates/opencmdb-bin/src/repo.rs` **or** `scan_pass.rs` | the grouped read. ⚠️ **Decide and justify**: `repo.rs` holds every other persistence adapter (~2626 lines, ceiling 2000 for CODE — check the pre-`#[cfg(test)]` count before adding); `scan_pass.rs` holds the counter this one is a sibling of |
| `crates/opencmdb-bin/templates/_gap_card.html` | the section (in the CARD, not the page — see the trap above) |
| `crates/opencmdb-bin/assets/app.css` | the section's rules, if any |
| `crates/opencmdb-bin/locales/app.yml` | new keys under a distinct prefix, `en` + `fr` |
| `deferred-work.md`, `sprint-status.yaml`, `CLAUDE.md`, `docs/project-context.md` | AC10, AC11 |

⚠️ **The `file-size` gate counts lines BEFORE the first `#[cfg(test)]`.** `repo.rs` is 2626 lines
total; check its code half against the 2000 ceiling **before** choosing it, not after.

### References

- [Source: `epics.md:1690-1710`] — story 5.14's four clauses, **all four of them this story's**; not edited
- [Source: `ux-design-specification.md:910-945`] — the abstention MAP, the mock, the six-month test
- [Source: `ux-design-specification.md:1276-1291`] — the seven hard bans, each with its own test
- [Source: `prd.md:896-899`] — FR16, FR16b and the measured floor
- [Source: `crates/opencmdb-bin/src/page.rs:39-53`, `:110-120`, `:160-198`, `:258-263`]
- [Source: `crates/opencmdb-bin/templates/_gap_card.html:2`, `:47-57`, `:58`]
- [Source: `crates/opencmdb-bin/src/scan_pass.rs:151-188`] — the existing counter and its doc sentence
- [Source: `crates/opencmdb-bin/src/repo.rs:274-292`] — `outcome_token`, `cause_token`
- [Source: `crates/opencmdb-bin/src/resolver.rs:660-693`] — `guard_decision`, and Epic 6 named as owner
- [Source: `crates/opencmdb-core/src/gap/mod.rs:24-57`] — the two vocabularies, already distinguished
- [Source: `crates/opencmdb-bin/src/resolver.rs:962-965`] — which outcomes the pass can write
- [Source: `crates/opencmdb-bin/src/repo.rs:885`] — `count_identity_links`, the one UNFILTERED read
- [Source: `ux-design-specification.md:1280`] — *"No badge, no growing counter"*, §5a's ban
- [Source: `crates/opencmdb-bin/migrations/0002_interface_and_identity_link.sql:60-99`]
- [Source: `crates/opencmdb-bin/assets/app.css:5-24`, `:121-134`]
- [Source: `deferred-work.md:3015-3098`] — every entry naming this story
- [Source: `5-14-wire-the-pass-and-measure-what-it-cannot-say.md`] — arbitrations 1-9, the mutation table

---

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context) — `claude-opus-5[1m]`.

### Debug Log References

Built and mutated against a live `mariadb:10.11.11`, container `opencmdb-5-14b`, host port **13316**
(13315 was still held by the gap-hunt's container). ⚠️ **`DATABASE_URL` is unset locally and the
DB-backed tests pass by `return`ing** — the suite reports the same counts either way. The witness
that they really ran is the timing: the `bin` suite takes **0.05 s** without a database and
**~4.2 s** with one. Committed (`6e1a746`) **before** the mutation pass; the tree was verified clean
of every mutation afterwards by `grep` for each marker and by reading the diff.

### Completion Notes List

**502 → 519 tests (298 bin + 159 core + 62 xtask), +17.** Seven gates green, 28 fixtures unchanged,
trap gate still RED at 26 discovered / 15 scored / 11 unanswerable. `opencmdb-core` untouched.
`epics.md` and `ux-design-specification.md` not edited.

#### 🔴 The finding of the pass: an anti-sum guard placed where a sum cannot be written

**M6b came back GREEN**, and the reason was not the fixture the validation had warned about.
`the_two_engines_counts_are_never_added` builds both views and composes them **itself**, so it can
only prove that `build_view` and `build_identity_view` do not add each other's counts — and **neither
of them can**, because neither sees the other's numbers. The only place a sum can be written is
`reconcile_view`, the impure edge that assembles both, and no unit test reached it.

🔑 *A guard placed where the defect cannot occur reads as coverage and is none.* Closed by
`one_real_page_build_keeps_the_two_counts_apart`, a database-backed test that goes through
`reconcile_view` with both populations non-zero, after which M6b reds. ⚠️ **Reading the guard could
not have found this** — the guard is correct about what it tests. Only running the mutation did.

⚠️ **Its first run also caught a false premise of my own**: three out-of-perimeter sightings are
three reconciliation abstentions plus one `NoObservedValue`, so the count is **4**, not 2. The
expectation was corrected, not the fixture — 4, 3 and 7 are still three distinct numbers, which is
what the test needs.

#### ⚠️ A false sentence in my own assertion message, caught by its own mutation

`both_locales_carry_every_identity_key`'s message asserted that *"a test reading the rendered HTML
would pass here"*. M8 refuted it: deleting `identity.floor` from both locales reds **two** tests, the
second being `the_surface_states_both_limits_separately`, which reads the rendered HTML. The true
statement is narrower — a render assertion is vacuous when it checks that *something* appeared or
checks for the key's own text, **not** when it names a distinctive phrase of the translation. The
message and the doc comment were corrected; the two guards are independent and neither inherits the
other's reach.

#### The mutation table, with OBSERVED results and each carrier read from its own panic message

| id | mutation | predicted | OBSERVED | red | carrier |
|---|---|---|---|---|---|
| **M1** | collapse the per-cause lines in the view | RED | ✅ **2**, both named assertions (`left: 1`) | 2 | assertion ×2 |
| **M2** | `GROUP BY outcome` only, dropping the cause | RED | ✅ **1** — the message prints the collapse, 3 groups → 2 | 1 | assertion |
| **M3** | drop `decided_by = 'ENGINE'` | RED | ✅ **2** — the operator-exclusion test **and the subsumption test**, on its *"agreeing on the full set but not on the filtered one"* clause | 2 | assertion ×2 |
| **M4** | drop `current_subject IS NOT NULL` | RED | ✅ **2**, same pair | 2 | assertion ×2 |
| **M6b** | the identity count absorbs the reconciliation one, **in `reconcile_view`** | RED | 🔴 **0 — GREEN.** See above; this is the pass's finding. **1** after the composition test existed | 1 | assertion ⚠️ **on the PREMISE** (`left: 7, right: 3`), not on the `assert_ne!` below it, which never fires |
| **M7** | put the section back inside `{% if view.has_entity %}` | RED | ✅ **5**, all named assertions | 5 | assertion ×5 |
| **M8** | delete `identity.floor` from **both** locales | RED ×1 | ✅ **2** — and the second refutes my own message, above. `assert_ne!` prints `left: "identity.floor"`, the echo itself | 2 | assertion ×2 |
| **M9c** | `#gap-card .abstentions .cause` → `var(--accent)` | 🔴 **GREEN by stated limit** | ✅ **0 red**, exactly as §8 predicts: an id selector beats the rule the check reads | 0 | — |
| **M10** | a clock reaching the view builder | RED | ✅ **2** — the ageing tripwire **and an ordinary count assertion**, so §8's *"the only thing carrying that ban"* is refuted here too | 2 | assertion ×2 |
| **M11a** | drop the unrecognised cause LINE | RED | ✅ **1** (`left: 1, right: 2`). ⚠️ **The total did not move** — this mutation touches the line only | 1 | assertion |
| **M11a-bis** | drop the unrecognised row ENTIRELY — the silent shrink | RED | ✅ **1** (`left: 5, right: 8`), on the total. **Written because M11a left the total assertion unmeasured** | 1 | assertion |
| **M12** | the production path emits `Ambiguous` | RED | ✅ **21** | 21 | ⚠️ **`.expect()`-carried** on AC8's own test: it dies on *"the pass must have RUN"*, never reaching its `ambiguous` assertion — exactly as validation predicted, and the premise is what makes it red at all |

**Twelve rows: eleven executable mutations and one GREEN by stated limit (M9c).** Zero
compiler-carried reds. ⚠️ **Carriers are MIXED and named row by row** — M12 is `.expect()`-carried
and M6b reds on a premise rather than on the assertion written for it. *"Every red assertion-carried"*
is **not** claimed.

#### What is NOT claimed

The UX bans are **stated, not met** (§5a). The denominator is not decided. Nothing groups sightings.
`Ambiguous` has no producer and the tripwire is bounded by the slice it names. The CSS check reads
the text of its own rules and not a resolved colour. `epics.md` is not edited.

### File List

- `crates/opencmdb-bin/src/repo.rs` — MODIFIED. `EngineReachRow`, `count_engine_reach`, and the four
  carrier tests (two causes, an operator row, a row out of the current key, an unfamiliar token).
- `crates/opencmdb-bin/src/page.rs` — MODIFIED. `IdentityView`, `IdentityCauseRow`,
  `identity_cause_label`, `build_identity_view`, the second read at the impure edge, and ten tests.
- `crates/opencmdb-bin/src/scan_pass.rs` — MODIFIED. The `Ambiguous` tripwire, the subsumption
  measurement, and the correction of `counted_current_engine_links`'s false doc sentence.
- `crates/opencmdb-bin/templates/_gap_card.html` — MODIFIED. The section, outside the `has_entity` gate.
- `crates/opencmdb-bin/locales/app.yml` — MODIFIED. Eleven keys under `identity.`, `en` + `fr`.
- `crates/opencmdb-bin/assets/app.css` — MODIFIED. The section's rules.
- `_bmad-output/implementation-artifacts/5-14b-*.md`, `sprint-status.yaml`, `deferred-work.md`,
  `CLAUDE.md`, `docs/project-context.md`.

---

## Change Log

| date | what |
|---|---|
| 2026-08-12 | Created. Four arbitrations inherited from story 5.14 (2, 4, 5, 9), three proposed and left open (10, 11, 12). The trap named first: **a "Reach" section already exists on the page and shows a DIFFERENT engine's abstentions** — same word, different type, different population. |
| 2026-08-12 | **ARBITRATION 13 TAKEN (Guy): the number ships with an HONEST UNIT — *sightings*, not devices**, on both sides of the pair, because each scan mints fresh `obs_id`s and that is what the number is true about. 🔑 The growth then reads as a fact about scanning rather than as a backlog: *a number that rises because the product looked many times is the radar's range, not the operator's debt* — which is why the unit is the arbitration and not a caption. Refused, both defensible: the bare number with a limit sentence (a sentence beside a number does not survive the reader's first glance), and no total at all (it would have cost `epics.md` its *"count NOT evaluated"* clause — ⚠️ **and would have made AC3 simpler, a simplification given up knowingly**). ⚠️ **The ban does not become MET; it becomes STATED** — a true unit does not stop a number growing — and the unit itself is TEMPORARY: Epic 6 gives the population an identity and the locale keys change with it, registered so the rename is met as a scheduled consequence rather than as a correction. |
| 2026-08-12 | **VALIDATED — gap-hunt layer**, which BUILT the story (502 → **516 tests**, clippy clean, seven gates green) against its own `mariadb:10.11.11` on port 13315, and ran nineteen mutations with their controls. 5 HIGH, 7 MEDIUM, 4 LOW. 🔑 **It found `link_columns` independently of the fact-check layer** — the same non-existent function, by two agents that never spoke. 🔴 **FOUR GUARDS THIS STORY PRESCRIBED WERE MEASURED GREEN, and each control is the deliverable**: (a) AC3's anti-sum passes on a zero-reconciliation fixture — *an anti-sum guard over a zero addend asserts nothing*; (b) a missing locale key is a **silent key echo**, so AC5's own prescribed remedy — read the rendered HTML — is a **tautology**, the remedy being the defect; (c) under M12 `guard_decision` refuses the decision and the pass ROLLS BACK, so AC8's assertion passes over an EMPTY table — 19 tests red, that one green; (d) `#gap-card .abstentions .cause` beats the colour check on specificity. 🔴 **And AC5's old wording drove the implementation into `set_locale`, a process-global**: four consecutive full runs gave **2 or 3 red out of 290, varying**, one casualty being the ageing guard itself — *reddened by a locale, with no clock anywhere*. AC5 is split and `set_locale` is forbidden in tests. ⚠️ Also: M11 as written is **not executable against arbitration 11's own design**; §8's *"the tripwire is the only thing carrying that ban"* is refuted (M10 reds 3, two being ordinary count assertions); the fresh-install empty state was covered by nothing; `AssertSqlSafe` is unnecessary and conflicts with `repo.rs`'s idiom; `repo.rs`'s code half is **1088** lines, so it is the right home. 🔑 The layer **refuted eight of its own suspicions by running them** — including the `GROUP BY`-drops-NULLs worry, which makes arbitration 12's precondition **measured rather than transcribed** — and reported a defect in its OWN driver, caught only because a result contradicted a prediction. |
| 2026-08-12 | **VALIDATED — fact-check layer.** 2 HIGH, 7 MEDIUM, 5 LOW applied; a long confirmation list recorded, including all four register owners verified verbatim and the live counts re-measured green (502 tests, 28 fixtures, seven gates, trap gate 26/15/11 `passed()==false`). 🔴 **H1: `link_columns` exists nowhere** — `resolver.rs:641` is inside `same_decision`, story 5.11's idempotence COMPARISON, which writes nothing; the function that maps a `Conclusion` to its token is `repo::outcome_token`, which this story's own References cited correctly one page later. 🔴 **H2 opened arbitration 13** (§5a): §8 recorded *"does not age"* as true by construction via the view builder's clock-freedom, **a strictly weaker property wearing the ban's name**, while §2 and §5 of this same story measure that the displayed number grows with scan count — and the UX spec's FIRST hard ban is literally *"No badge, no growing counter"*, unaddressed in a story whose deliverable is a growing counter. ⚠️ Also caught: the mutation for *"no red"* targeted `.abstentions .cause`, **the OTHER section's selector**, which arbitration 10 exists to keep separate; *"four predicates"* for two predicates and two `GROUP BY` columns; *"a THIRD unfiltered counter"* when only `count_identity_links` is unfiltered, spending on *"ungrouped"* the very word the tree reserves to keep the three reads apart; and §4 sending the dev to re-derive which outcomes the pass can write, when `resolver.rs:962-965` already answers it — the omission §2 exists to prevent. 🔑 The layer also **refuted four of its own suspicions by measuring them** and recorded the checks. |
| 2026-08-12 | **The three open arbitrations TAKEN by Guy**, each recorded with the alternative refused and why. **10 — TWO sections, never summed**: two populations inside one frame invite the reader to add them, and the invitation is the defect; ⚠️ this **diverges from the UX spec's one-panel mock deliberately**, and the divergence is registered rather than absorbed. **11 — the reader never fails**: an unrecognised token is counted, labelled and rendered. The DDL `CHECK` was refused on a reason stronger than "DDL in a display story" — it **moves the failure from the display to the WRITE**, so a future variant would break the identity pass rather than show an unfamiliar label; 🔑 *a display story may not be the place a write starts failing*, and the schema-side domain is REGISTERED as the real closure rather than implied. **12 — the grouped read subsumes `counted_current_engine_links`** and 5.14's *"story 5.14b is its production consumer"* is corrected. The second-oracle alternative was refused because two reads on the REQUEST path against a live store disagree on concurrent writes — **flaky by construction**, not load-bearing; the same redundancy stays legitimate inside one transaction in a test. ⚠️ The correction is gated on a measurement: if the two reads' populations diverge, the arbitration re-opens rather than being forced. |
