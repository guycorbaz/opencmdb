# Story 6.4: The abstention line carries the gesture

Status: **ready-for-dev** — the arbitration is taken (§0h, Guy, 2026-08-24).

⚠️ **The story's title and its *so that* clause are left as the plan wrote them and are now
INACCURATE**: the gesture lands on the triage queue's `Nouveau` row, not on the abstention line,
and the reach section stays a number. A story may not edit the plan; the divergence is registered
and Epic 6's retrospective may correct it.

## Story

As the operator,
I want to document an unplaced sighting from where I see it,
So that the reach section stops being a number and becomes a door.

## Acceptance Criteria

*(`epics.md:1794`. §0 explains every divergence. ⚠️ **AC1 cannot be implemented as written** — §0a.)*

**AC1 — REWRITTEN on the arbitration (§0h). Given** a triage queue row whose kind is `Nouveau`
— *an observed address no declared entity claims*, which is the glossary's `undeclared` and FR13's
own population — **When** the operator acts on it **Then** it carries the documenting action, and
**the whole record is documented at once** (**FR13(a)**, *the day-one case*). ⚠️ **One act, one
answer, never N failures** (FR16b): whatever the row's count, the operator asks once and is
answered once.

**AC2 — WIDENED, and the principle is what widened it. Given** any cause that is not `undeclared`
**When** it is displayed **Then** it carries **no** documenting gesture — and the surface says why
rather than staying silent. `Ambiguous` is a doubt to LIFT, not an entity to CREATE. 🔴 **And
`AbsenceOfProof` carries none either**: it says *the engine could not place this sighting*, whose
answer is **a better source**, not a documented entity — the shipped scanner reads no hardware
address, which `/sources` already states. ⚠️ **The operator's three cases (Guy, 2026-08-12)**: no
ambiguity → the software decides; ambiguity → the operator lifts the doubt; **unknown → the
operator creates the entity**. **The gesture belongs to the CAUSE** — applied correctly, to
`undeclared` and to nothing else.

**AC3 — And** the UX bans still hold: the section does not redden, carries no gauge, and does not
age. ⚠️ Story 5.14b left them **stated, not met** — the number still grows with scan count — and
this story does not claim otherwise.

### The criteria this story adds to itself

**AC4 — The gesture is LIVE, and `Gesture::Planned`'s single variant stops being single.** Story
6b.4b built that enum with one variant so that adding `Live` gives `error[E0004]` on every `match`
— *"forcing the revisit at exactly the moment story 6.4 needs it"*. **This is that moment.** If the
enum is not touched, the compiler was never the guard it was sold as.

**AC5 — Every guard is measured RED before it passes, and where the defect lives in the DOM a
source-reading guard does NOT SUFFICE** (story 6b.11's amended AC5). A gesture that acts is a DOM
behaviour before it is a route.

**AC6 — The live count lives in THIS file**, and every figure names the state it was taken against.

**AC7 — What the operator can DO is stated plainly**, and this is the first story in three epics
where the answer is not *"nothing"*. If any part of the gesture is still not reachable, the story
says so rather than counting the route as the gesture.

---

## §0 — What contexting established

### §0a. 🔴 AC1 ATTACHES FR13'S GESTURE TO A POPULATION FR13 DOES NOT NAME — AND IT IS 5.14b's DEFECT, ONE LAYER DOWN

AC1 puts the documenting gesture on the **`AbsenceOfProof`** cause line. FR13's own population is
something else, and the PRD's binding glossary says so in a row of its own:

| Glossary state (`prd.md:1018`) | Meaning | Who acts | Type named |
|---|---|---|---|
| `ambiguous` | several possible identities for one object | the operator **lifts the doubt** | `identity::cascade::IdentityAbstentionCause::Ambiguous` |
| **`undeclared`** | **observed, and NO DECLARED RECORD CLAIMS IT** | the operator **creates the entity** — **the documenting gesture (FR13)** | **none** |

🔴 **`AbsenceOfProof` is neither of those.** Measured in `cascade.rs`: it is the verdict for
*"only `Neutral` / nothing"* — **nothing proved the PAIR**. It is a statement about whether two
sightings could be joined, not about whether a declared record claims the thing. On the shipped
connector it means *the scanner saw no hardware address*, which says nothing at all about the
declared side.

⚠️ **So a sighting can be `AbsenceOfProof` AND already fully declared** — `a11y/seed.sql` produces
exactly that state: four declared entities, observations that the identity pass cannot place.
Documenting there would create a second entity for a machine the operator has already documented.

🔑 **This is story 5.14b's defect one layer down, and that story's own record names the shape**:
*"Same word, different type, different population"* — it found two sections both titled *Reach*,
one showing `gap::AbstentionCause` and the other `identity::…::IdentityAbstentionCause`. AC1 now
attaches a gesture defined for the RECONCILIATION population to a line counting the IDENTITY
population. ⚠️ The glossary noticed: it gave `ambiguous` a type and left `undeclared` without one,
**because no type for it exists**.

**→ ARBITRATION FOR GUY, and it decides what this story builds. See §0f.**

### §0b. 🔴 A CAUSE LINE CARRIES NO SUBJECT, AND THE ROUTE TAKES EXACTLY ONE

Measured:

- `page.rs:143` — `struct IdentityCauseRow { cause: String, count: i64 }`. **No subject, no id.**
  It is an aggregate: `COUNT(DISTINCT observation_id)` grouped by cause.
- `document.rs:183` — `document_all(subject: ObsId)`, **one** observation, and the nil UUID refused
  as a shape error before the store sees it.

**So AC1 is not a wiring job.** Rendering a form on a cause line requires a read the page does not
have — which observations sit under that cause — and either a bulk route or a fan-out. ⚠️ **And
FR16b makes the bulk reading the right one**, in its own words: *"Each cause is one line and one
gesture, not N failures. **96 multi-interface devices is not 96 failures — it is ONE question.**"*
So the gesture is one act over N subjects, and *"never N failures"* is a constraint on the
**answer**: one line asked, one answer given, never N error messages to work through.

### §0c. ⚠️ ON THE SHIPPED CONNECTOR, "ONE LINE" IS THE WHOLE NETWORK

The ARP/ping scanner emits an IPv4 address and a round-trip time and **no hardware address ever** —
`/sources` says so on screen. `join` keys on `(l2_domain, mac)`, so **every scanned observation
abstains, and every abstention carries the same cause**. The `AbsenceOfProof` line is therefore
**one line holding the entire unplaced population**, and its single gesture would document all of
it at once.

🔑 That is not an argument against the bulk gesture — FR16b asks for exactly one question per cause
— but it sizes the act, and the surface must size it too. *A button whose blast radius is the whole
network may not look like a button whose blast radius is one row.*

### §0d. WHAT STORY 6b.4b LEFT, AND IT WAS BUILT FOR THIS STORY

`enum Gesture { Planned { owner } }` — **one variant, by Guy's arbitration taken OVER the
contexting's own recommendation of a struct**, and mutation M7 measured why: adding `Live` gives
`error[E0004]: non-exhaustive patterns` on every `match`, *"forcing the revisit at exactly the
moment story 6.4 needs it, where the struct would have forced none ever."*

⚠️ **And 6b.4b narrowed its own promise in writing**: a labelling and typing DISCIPLINE, never a
compiler-enforced guarantee — the validation built both shapes and measured that **a route pointing
nowhere reds nothing under either**. The real closure — *a route typed as a member of a CLOSED
SET* — was registered **to this story**, where the set stops being empty. AC4 is that debt.

### §0e. THE UX BANS, AND WHAT THIS STORY MAY NOT CLAIM

Story 5.14b left the bans **stated, not met**: the number grows with scan count, because nothing
supersedes an abstention across passes and every scan mints fresh `obs_id`s. AC3 says this story
does not claim otherwise — ⚠️ **and it must not accidentally make it worse**: a gesture that
documents N sightings does not remove them from the count, since the count is of SIGHTINGS and the
observations are never modified (FR13's invariant). *The number will not go down when the operator
acts, and the surface must not imply it will.*

### §0f. THE ARBITRATION, WITH WHAT EACH OPTION COSTS

Which population does the gesture attach to?

1. **AC1's letter — the `AbsenceOfProof` line.** Cheapest, and it is what the epic says. ⚠️ But it
   offers to create entities for sightings that may already be declared, which is the one thing
   FR13's invariant is written to prevent.
2. **FR13's own population — `undeclared`.** Correct by the glossary, and it is the operator's
   third case. ⚠️ But `undeclared` has **no type**: it must be computed — an observed address that
   no `declared_attribute` claims — and the identity section does not read the declared side at all.
3. **The intersection** — unplaced AND undeclared. Honest, and the smallest population. ⚠️ Two reads
   and a join the page does not have; and on the shipped connector it may be most of the network
   anyway.

⚠️ **Whatever is chosen, `epics.md` is NOT edited by this story** — a story may not. The divergence
is registered, and Epic 6's retrospective may correct the criterion.

### §0h. THE ARBITRATION (Guy, 2026-08-24) — the gesture goes where the population already is

🔑 **The measurement that decided it: `undeclared` is NOT typeless — it is already computed and
already on screen.** `page.rs:1005` labels a triage queue row `Nouveau`, *"an observed address no
declared entity claims"*, which is the glossary's `undeclared` word for word, rendered since story
6b.4.

🔴 **⚠️ AND THE MECHANISM THIS PARAGRAPH FIRST NAMED WAS FALSE — the validation refuted it and the
correction matters for whoever implements.** It said *"produced by `gap::reconcile`"*. Measured:
`gap::AbstentionCause` has **no `undeclared` variant at all** (`OutOfPerimeter`,
`NoObservedValue`, `ConflictingObservations`), and `reconcile()` is called **once per
already-declared entity** inside `build_triage`'s entity loop — it structurally cannot report on an
address no entity claims. `Nouveau` is a **second, independent loop** over the observations,
filtering against the `claimed` set the entity loop collected. *The conclusion holds and the
sentence pointing at it did not* — and a developer told *"`gap::reconcile` already produces the
row"* would look in the wrong file.

**So the gesture lands on the `Nouveau` row**, and the abstention line carries none:

- **The population is already there, correct and computed** — no new read, no invented join, and
  none of option 1's risk of offering to create an entity for a machine already declared.
- **It is where the operator looks.** The triage queue IS the product's loop, and FR13 is the third
  of Guy's three cases — *unknown → the operator creates the entity* — displayed there by name.
- **AC2's principle is preserved rather than bypassed**: *the gesture belongs to the CAUSE*.
  Applied correctly, that cause is `undeclared`.

⚠️ **Refused, and recorded because both were defensible:** AC1's letter — the `AbsenceOfProof` line
— because it offers to create entities for sightings that may already be declared, the one thing
FR13's invariant exists to prevent; and the **intersection** (unplaced AND undeclared), which is
safe but costs two reads and a join, and on the shipped connector is most of the population anyway.

🔴 **What the arbitration costs, stated rather than glossed.** The story's own *so that* clause says
*"the reach section stops being a number and becomes a door"*. Under this decision it stays a
number. 🔑 **And the premise is what is wrong, not the decision**: an identity abstention says *the
engine could not place this sighting*, and its answer is **a better source** — the shipped scanner
reads no hardware address, which `/sources` already tells the operator, and Epic 11 is what opens
it. ***The reach section may have no door, and that is not this story's failure*** — it is the
honesty the product claims everywhere else.

### §0i. THE VALIDATION — two layers, 2026-08-25, and the second one BUILT the gesture

The fact-check layer worked read-only; the gap-hunt layer had its own worktree and store, **wired
a real control to `/document-all`, pressed it, and reported what broke**. Nine findings, **two
reached by both**.

🔴 **THE TWO CONVERGENCES.** *(a)* **The `Nouveau` row carries no `ObsId`** — the fact-check by
reading `ObservedBatch` and its `SELECT`, the gap-hunt **on the wire**, finding only declared
entity UUIDs anywhere in the rendered page. *(b)* **`Gesture::Live` forces exactly ONE `E0004`
site** — the fact-check by counting match sites, the gap-hunt by **adding the variant and
rebuilding**. ⚠️ And the gap-hunt went one step further, which is the finding: **satisfying that
error naively ships an inert `<span class="btn-gesture live">`** — no route, no method, no subject
— *a control that looks live and does nothing*, which is precisely what story 6b.4b's own doc said
the compiler could not prevent.

🔴 **AND FOUR HOLES THIS STORY DID NOT KNOW IT HAD, each measured by building:**

1. **`OPENCMDB_DOCUMENT_ENABLED` is invisible to the page builder** — zero references in
   `page.rs`; the flag is read in `main.rs` at router-merge time only. A naive wiring therefore
   renders, **on the default configuration**, a pressable control pointing at a **404**. → T2b.
2. **The axe gate cannot reach a selected `Nouveau` row.** It derives its state route from the
   **first** queue row, and `build_triage` emits every gap, absence and conflict row before the
   `Nouveau` loop. AC5's DOM-level carrier has a hole in CI's only DOM-level gate. → T3b.
3. **`a11y/seed.sql` produces ZERO `Nouveau` rows** — the target population does not exist in the
   corpus the gates run against. The layer had to hand-insert an observation to make one. → T3b.
4. **A plain `<form>` full-navigates to the raw text body** — `documented 1 field(s) as entity …`,
   `text/plain`. → arbitration 2.

⚠️ **Three more worth carrying:** the row keeps the **oldest** sighting of a repeated address
(→ arbitration 1); AC2's *"says why"* is **new copy**, not a rewire, and no such key exists; and
FR16b's *"never N failures"* has **no N>1 case** in the arbitrated scope, a `Nouveau` row being one
address and one subject with `counted: false`.

🔴 **And two false citations of mine, one of which had already propagated.** `epics.md:2118` is a
line inside story **6b.1** about the amber; the criterion is at **`:1794`** — and the wrong number
was already carried into `deferred-work.md` in the same commit. ⚠️ **And §0h's *"produced by
`gap::reconcile`"* is false**: that enum has no `undeclared` variant and `reconcile()` runs per
already-declared entity. *The conclusion held; the sentence pointing at it did not, and it would
have sent the implementer to the wrong file.*

✅ **Refuted by the gap-hunt, each with its check** — recorded so nobody re-chases them:
documenting does **not** make the reach count fall (that section reads `identity_link`, which the
scan pass writes; the queue's own total falls, correctly, being the reconciliation backlog);
a repeated address already collapses to **one** row, confirmed on a live render; `Gesture::Live`
touches **no** site in `diagnostic.rs`; and a 401 seen during prototyping was a **leftover server
process** booted without credentials, not a defect.

### §0j. THE TWO ARBITRATIONS (Guy, 2026-08-25)

🔑 **Arbitration 1 — the subject is the MOST RECENT sighting.** The row keeps the oldest today
(`ORDER BY observed_at` ascending, then first-wins), measured on two observations ten minutes
apart. **A gesture that writes *the whole record at once* may not write the one the network has
already moved past** — if a later scan saw a `hostname` the first did not, the letter of AC1 would
document the less complete record. ⚠️ The defect is pre-existing and becomes **visible** only when
a gesture writes from it: *a story that lays a gesture on a choice it knows to be wrong inherits
the choice.* **Refused:** keeping the oldest and stating it (the operator would document a stale
record with nothing on screen saying so), and merging the facts of the N sightings — which would
invent an observation that never happened, and **D19 forbids fabricating an observed**.

🔑 **Arbitration 2 — `hx-post` with a target and a swap.** Measured: a plain `<form>` takes the
browser to `documented 1 field(s) as entity …` in `text/plain`. htmx is already vendored and
served (D37), and `_gap_card.html` already uses it. **Refused:** a plain form plus a real response
page (work this story does not budget), and a 303 redirect — which would mean changing the route's
201-with-a-body contract, breaking story 6.2's tests and its record.
⚠️ **This is the product's FIRST live swap, so it inherits a debt by name**: story 6b.11 registered
the *focus-after-swap* contract to story 6.4 because **no swap existed to attach it to**. It exists
now. Focus must land inside the swapped region, and the guard reads `document.activeElement` after
the swap — never the template.

### §0g. WHAT THIS STORY MUST NOT DO

- **Not touch the observed record.** FR13's invariant, both cases: the observation is never
  modified, the link is preserved, only the declared record is written. Story 6.3's `observed-immutable`
  gate enforces it and its allowlist is EMPTY.
- **Not build `document-field`.** FR13(b) is Epic 7's, registered.
- **Not claim the UX bans are met** (§0e).
- **Not count the route as the gesture.** `POST /document-all` has existed since story 6.2 and is
  reached from no template. AC7 exists because *a route nobody can press is not a gesture*.

---

## Tasks / Subtasks

- [x] **T1 — The subject the row does NOT know** (AC1). 🔴 **The open question is SETTLED and the
      answer is the harder one: a new read is needed.** Measured by both validation layers —
      `ObservedBatch` has three fields (`connector_id`, `observed_at`, `facts`) and **no id**, and
      `load_observation_facts`'s `SELECT` does not name the `id` column at all. Confirmed on the
      wire: the observation's UUID appears **nowhere** on the rendered page. ⚠️ **T1's first draft
      said *"no new read"* and contradicted itself one sentence later** — the read is: add `id` to
      the query and to `ObservedBatch`, then plumb it to the row. **The blast radius is every
      `ObservedBatch` test constructor**, `page.rs`'s `fn batch(...)` helper first.
- [x] **T1b — WHICH sighting supplies the subject** (arbitration 1, §0i). 🔴 The row keeps the
      **OLDEST** sighting of a repeated address today (`ORDER BY observed_at` ascending, then
      first-wins in `seen_new`). **Guy: take the most recent.** A gesture that writes *the whole
      record at once* may not write the one the network has already moved past.
- [x] **T2 — The gesture, LIVE** (AC1, AC4) — `Gesture::Live`, as `hx-post` with a target and a
      swap (arbitration 2, §0i). ⚠️ **The `E0004` is ONE site, not *"every match"*, and satisfying
      it naively ships an inert control** — measured: the variant added and rebuilt gives exactly
      one error, at `page.rs:751`; and `GestureView` carries only `label` and `not_built`, so the
      template's live arm renders a bare `<span class="btn-gesture live">` with **no route, no
      method, no subject**. *A compile error that can be silenced by a span is not the guard 6b.4b
      sold — its own doc said so, and this is the measurement.*
- [x] **T2b — The page must know whether the route EXISTS** (§0i). 🔴 `page.rs` has **zero**
      references to `document_enabled`; the flag is read only in `main.rs` at router-merge time. So
      a naive wiring renders, **on the default configuration**, a pressable control pointing at a
      **404** — measured. The page builder reads the flag and renders `Planned` when the route is
      not mounted, which is what the rest of the action bar already does.
- [x] **T3 — Every other cause carries none, and SAYS so** (AC2) — `Ambiguous` and `AbsenceOfProof`
      alike. ⚠️ The guard measures the **absence on the rendered page**, not in the template
      (AC5 as amended). ⚠️ **And *"says why" is NEW COPY, not a rewire**: `_identity_section.html`
      renders cause and count and nothing else, and no `identity_no_gesture`-shaped key exists.
      Two new keys, both languages, under the `copy-vocabulary` gate.
- [ ] **T3b — The population must EXIST where the gates run** (AC5). 🔴 `a11y/seed.sql` produces
      **zero** `Nouveau` rows — measured — because all three seeded observations land on declared
      addresses. And 🔴 **the axe gate structurally cannot reach a selected `Nouveau` row**: it
      derives its state route from the **first** queue row, and `build_triage` pushes every gap,
      absence and conflict row **before** the `Nouveau` loop. Seed the population **and** give the
      gate a state that names it, or the story's only DOM-level carrier in CI never sees the
      gesture.
- [ ] **T4 — One act, one answer** (FR16b) — ⚠️ **and within this story's arbitrated scope there is
      no N>1 case to exercise**, measured: a `Nouveau` row is one address and one subject, and its
      `counted` is `false` with an empty `count`. FR16b's *"96 devices is ONE question"* bites on
      the `Absence`/`Conflict` cause rows, which this story does not touch. **Say so rather than
      leaving a task that reads as needing bulk-failure handling nothing in scope produces.**
- [ ] **T5 — The bans, and the honest sentence** (AC3, §0e) — the count is of SIGHTINGS and the
      observations are never modified, so **it will not fall when the operator acts**. The surface
      must not imply it will.
- [ ] **T6 — Prove-to-red in the DOM** (AC5) — ⚠️ on the **unrepaired** mutation driver, by Guy's
      sequencing of 2026-08-24: write the prediction BEFORE the mutation, which is what caught all
      seven driver defects in Epic 6b.
- [ ] **T7 — Register the divergence** — the title, the *so that* clause, and AC1's population.
      `epics.md` is NOT edited; Epic 6's retrospective may.
- [ ] **T8 — The record** (AC6, AC7) — the live count here; what the operator can DO, plainly. ⚠️
      **This is the first story in three epics whose honest answer is not *nothing*** — and it must
      say precisely which population gained a door and which did not.

---

## Dev Notes

### What the previous story leaves you

Story 6.3 closed NFR5 on all three assertions and shipped the **eighth** gate, `observed-immutable`,
whose allowlist is EMPTY — *the one form of allowlist nobody can quietly widen*. Any write this
story adds to `observation_record` reds it.

⚠️ **Epic 6b sits between 6.3 and this story**: ten screens, two browser gates in CI, `v0.2.0`
published — and **not one gesture that acts**. The count of *well-lit dead ends* ran from four to
ten. This story is what ends that, and it is the first in three epics whose honest answer to
*"what can the operator DO?"* is not *"nothing"*.

### The house rules that bite here

- **AC5 as amended (6b.11)**: where the defect lives in the DOM, a source guard does not suffice.
  A gesture is a DOM behaviour.
- **`a11y/kbd-probe.mjs` and `a11y/axe-gate.mjs` run in CI** and the store is seeded by
  `a11y/seed.sql`, which TRUNCATES first. A new control on `/triage` or the identity section is
  measured by axe on every run.
- **The live count lives in the current story's file** (story 6.1's AC8).

---

## References

- `epics.md:1794` — the criterion, and §0a's divergence.
- `prd.md:884` (FR13), `:897` (FR16b), `:1018` (the binding glossary row for `undeclared`).
- `cascade.rs` — `AbsenceOfProof`'s definition, which is about the PAIR.
- `page.rs:143` — `IdentityCauseRow`, which carries no subject.
- `document.rs:183` — the route, which takes exactly one.

---

## Dev Agent Record

*(to be filled by the dev agent)*

## Change Log

| Date | Change |
|---|---|
| 2026-08-25 | **VALIDATED by two layers — nine findings, two reached by both, and the second layer BUILT the gesture and pressed it.** 🔴 Four holes the story did not know it had: `OPENCMDB_DOCUMENT_ENABLED` is invisible to the page builder, so a naive wiring shows a pressable control pointing at a **404** on the default configuration; the axe gate cannot reach a selected `Nouveau` row; `a11y/seed.sql` produces **zero** of them; and a plain `<form>` navigates to the raw text body. ⚠️ **`Gesture::Live` forces ONE site, not *every match*, and satisfying it naively ships an inert `<span>`** — 6b.4b's own warning, measured. 🔴 And two false citations of mine, one already propagated to the register: `epics.md:2118` is **`:1794`**, and §0h's *"produced by `gap::reconcile`"* is false — that enum has no `undeclared` variant. **✅ Two arbitrations (Guy):** the subject is the **most recent** sighting, not the oldest; and the gesture is **`hx-post`** with a target and a swap — ⚠️ which makes it the product's first live swap, inheriting 6b.11's focus-after-swap contract by name. |
| 2026-08-24 | ✅ **ARBITRATION TAKEN (Guy): the gesture goes on the triage queue's `Nouveau` row, and the abstention line carries none.** 🔑 The measurement that decided it: `undeclared` is NOT typeless — `page.rs:1005` already labels a queue row *"an observed address no declared entity claims"*, the glossary's own words, computed by `gap::reconcile` and on screen since 6b.4. **The glossary left it typeless because it was looking at the identity engine; it lives on the reconciliation side.** *Refused:* AC1's letter (offers to create an entity for a machine already declared) and the intersection (safe, two reads and a join, and most of the population anyway on the shipped connector). 🔴 **The cost, stated:** the story's *so that* clause — *"the reach section becomes a door"* — is not met, and **the premise is what is wrong**: an identity abstention's answer is a better SOURCE, not a documented entity. *The reach section may have no door, and that is not this story's failure.* |
| 2026-08-24 | Story created and CONTEXTED. 🔴 **AC1 cannot be implemented as written**: it attaches FR13's gesture to `AbsenceOfProof`, an identity verdict about whether two sightings could be JOINED, while FR13's own population is `undeclared` — *observed, and no declared record claims it* — which the PRD's binding glossary gives a row and **no type**. ⚠️ A sighting can be `AbsenceOfProof` **and already declared**, a state `a11y/seed.sql` produces today. **This is story 5.14b's defect one layer down**, and that story's own record names the shape: *same word, different type, different population*. 🔴 Second measurement: `IdentityCauseRow { cause, count }` carries **no subject** and the route takes exactly **one** — so AC1 is not a wiring job but a read the page does not have. ⚠️ And on the shipped connector *"one line"* is the WHOLE unplaced population, since the scanner emits no hardware address ever. **One arbitration open (§0f).** |
