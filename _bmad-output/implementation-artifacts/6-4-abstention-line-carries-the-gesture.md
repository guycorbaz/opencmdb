# Story 6.4: The abstention line carries the gesture

Status: **done** — PR #127 squash-merged 2026-08-26 as `b6dfd69`, after a CI run green on the head commit itself (2m57s: nine gates, 741 tests, axe 10 routes + 4 states 0 nodes, kbd 30 checks).

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
forces **no `E0004`** in `diagnostic.rs` — ⚠️ **corrected at the code review, where the SENTENCE
over-reached and the measurement did not**: the story edits that file and its template anyway,
because renaming `not_built` to `nature` reopens the template's own match, and both now carry a
`Live` arm and a class named after it. *Touching no site* and *forcing no compile error* are
different claims, and the ✅ was on the wrong one; and a 401 seen during prototyping was a **leftover server
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
- [x] **T3b — The population must EXIST where the gates run** (AC5). 🔴 `a11y/seed.sql` produces
      **zero** `Nouveau` rows — measured — because all three seeded observations land on declared
      addresses. And 🔴 **the axe gate structurally cannot reach a selected `Nouveau` row**: it
      derives its state route from the **first** queue row, and `build_triage` pushes every gap,
      absence and conflict row **before** the `Nouveau` loop. Seed the population **and** give the
      gate a state that names it, or the story's only DOM-level carrier in CI never sees the
      gesture.
- [x] **T4 — One act, one answer** (FR16b) — ⚠️ **and within this story's arbitrated scope there is
      no N>1 case to exercise**, measured: a `Nouveau` row is one address and one subject, and its
      `counted` is `false` with an empty `count`. FR16b's *"96 devices is ONE question"* bites on
      the `Absence`/`Conflict` cause rows, which this story does not touch. **Say so rather than
      leaving a task that reads as needing bulk-failure handling nothing in scope produces.**
- [x] **T5 — The bans, and the honest sentence** (AC3, §0e) — the count is of SIGHTINGS and the
      observations are never modified, so **it will not fall when the operator acts**. The surface
      must not imply it will.
- [x] **T6 — Prove-to-red in the DOM** (AC5) — ⚠️ on the **unrepaired** mutation driver, by Guy's
      sequencing of 2026-08-24: write the prediction BEFORE the mutation, which is what caught all
      seven driver defects in Epic 6b.
- [x] **T7 — Register the divergence** — the title, the *so that* clause, and AC1's population.
      `epics.md` is NOT edited; Epic 6's retrospective may.
- [x] **T8 — The record** (AC6, AC7) — the live count here; what the operator can DO, plainly. ⚠️
      **This is the first story in three epics whose honest answer is not *nothing*** — and it must
      say precisely which population gained a door and which did not.

### Review Findings — three-layer code review, 2026-08-25

⚠️ **All three layers ran at this session's capability, in isolation but NOT on a different
model.** The house rule's "different model" half is not met by this pass and is not claimed.
Isolation was real: the Blind Hunter was forbidden the repository, the Edge Case Hunter had its
own worktree and its own `mariadb:10.11.11` on port 13411, the Acceptance Auditor read only.

**Convergence, which is what the count measures:** 6 findings were reached by two layers
independently, 1 by all three. 🔑 **The Blind Hunter — the diff and nothing else — found the
headline again, for the sixth story running**, and it found it by arithmetic the two sighted
layers had every means to do and did not.

🔑 **And the Edge Case Hunter replayed all twelve mutation rows and every one conformed** — the
first table in this project to survive re-execution unchanged, the M4 divergence included. That is
recorded as a result, not as an absence of findings.

#### Decisions (the fix is not unambiguous)

- [x] **[Review][Decision] The answer to the act is stale, silent on refusal, and in English** —
      `blind+edge+auditor`, all three, MEASURED in Chrome. `hx-swap="innerHTML"` on
      `#gesture-result` replaces one paragraph, so after a 201 the queue row is still there, the
      declared pane still reads *« Rien de déclaré à cette adresse »* over a success message, and
      the amber button is still live. A **second press answers 409 and swaps nothing** — the first
      press's success sentence stays on screen asserting the opposite, focus does not move, and
      `aria-live` announces nothing. Measured the same on a 500: total silence. And the body itself
      is `format!("documented {} field(s) as entity {}")` — an English literal under a French UI,
      carrying `field(s)` (the parenthetical plural story 6b.10 fixed **as a class on this screen**)
      and a raw UUID (the defect 6b.4's review removed from this pane). 🔴 **AC7 says *"the queue's
      question disappears"*; it does not.**
- [x] **[Review][Decision] With the route unmounted, the product says the gesture is NOT BUILT** —
      `blind`, extended by reading `action_bar`. `document_enabled = false` renders
      `Gesture::Planned { owner: "6.4" }`, so the control carries the *À venir* badge and *"les
      gestes marqués À venir ne sont pas encore construits"*. It IS built; it is disabled. On the
      default configuration — which the story itself calls what almost every deployment runs — the
      product's own copy is false about its only live gesture.

#### Patches

- [x] [Review][Patch] The live count is the SESSION's delta, not the story's: `master` is **729**,
      HEAD is 738, so **+9 and not +3** — and `c4c187d`, the baseline it names, is story-commit 4
      of 8. `blind+auditor`, three independent routes, and it is story 6b.1's recorded defect
      verbatim: *an intermediate state read as a baseline*. [story file §Dev Agent Record]
- [x] [Review][Patch] The `Live` doc block was inserted INSIDE `Planned`'s, so `Live` opens *"The
      product does not have this gesture yet"* and documents an `owner` it has no field for, while
      `Planned` — which has one — is left undocumented. `blind+auditor` [page.rs:526]
- [x] [Review][Patch] `enum Gesture`'s header still reads *"# One variant today"* and *"the day
      story 6.4 adds `Live`"*. [page.rs:507]
- [x] [Review][Patch] `GestureView::nature`'s doc still says *"what makes story 6.4's `Live` a
      compile error here"*. `blind+auditor` [page.rs:557]
- [x] [Review][Patch] Three shipped sentences saying **none of the controls is live**, in the three
      files this story edited to make one live — including *"`POST /document-all` is not one of them
      either"* in the very file whose `Live` arm this story wrote. `auditor`
      [_action_bar.html:3, page.rs:561, app.yml:356]
- [x] [Review][Patch] `identity_view.rs` — the file this story created — carries a taxonomy table
      mapping **`AbsenceOfProof` → the documenting gesture**, the exact attachment §0a/§0f/§0h/AC2
      and three locale keys exist to refute, plus *"the documenting gesture needs a write surface
      the product does not have"* in the commit whose subject is that surface. `blind` — prose
      carried unchanged by a pure move, *and the move is what made nobody re-read it*.
- [x] [Review][Patch] The pane's `observed_meta.source` is NOT updated by the last-wins overwrite,
      so a repeated address shows the FIRST sighting's provenance beside the LATEST one's freshness
      while the button posts the latest — and `_action_bar.html` states in writing that the two
      *"cannot name two different observations"*. `blind+edge+auditor`, **measured on the wire**
      (`Source aaaaaaaa · just now` beside `subject: …bbbb`). Reachable: `connector_id` is minted
      fresh at every boot, so scan → restart → scan produces it. [page.rs:951]
- [x] [Review][Patch] `a11y/kbd-probe.mjs` inverts the 0/1/2 contract in BOTH directions —
      `edge`, measured. A real **500** on the gesture is reported as *"the gate could not run"* with
      a message asserting 409 as the cause (a cause with no check behind it for four of five
      reachable statuses); and an unseeded reach section — a HARNESS shortfall — is reported as
      *"the keyboard layer has regressed"*. Split: 409 → `cannotRun`, every other non-2xx →
      `check(false, …)`; empty reach → `cannotRun`.
- [x] [Review][Patch] The gesture block waits a fixed 900 ms and then reads `posted`; a slow-but-
      correct POST leaves `posted === null`, skips the guard, and reds as *the product broke* —
      the confusion the block's own comment says cannot happen. `blind`. Wait for the response.
- [x] [Review][Patch] Seven new probe checks, recorded as **three** (`MIN_CHECKS` 20 → 27, seven
      `check(` sites). `blind+auditor` — in the file whose own comment says *"if a check is added
      this number moves deliberately"*. [story file]
- [x] [Review][Patch] The amber count's decomposition is **2 + 2**, not the *"three declarations
      plus its hover"* written in the comment AND in the assertion's failure message — the message
      a future developer reads when the guard reds. `blind+auditor` [page.rs:3794]
- [x] [Review][Patch] `ORDER BY observed_at` has no tiebreaker, so arbitration 1's *"most recent
      sighting"* is undefined on equal instants — stable today by the storage engine's habit, not
      by the query. `edge`, reported deflated with its own measurement. `ORDER BY observed_at, id`.
- [x] [Review][Patch] `.btn-gesture.live`'s comment says the emphasis question is *"registered
      rather than taken here"* — it was taken twenty lines below, by Guy, and it is in no register
      row. `auditor` [app.css:600]
- [x] [Review][Patch] `identity.no_gesture.ambiguous` says *"choosing among the candidates is still
      to come"* — an announcement of a future gesture, against `identity_view.rs`'s own doctrine
      sentence in the same commit: *"announcing an absent gesture is a promise; this section stays
      descriptive until the gesture is there"*. `auditor`
- [x] [Review][Patch] `a11y/seed.sql`'s justification for seeding both causes (*"a seed with only
      one would let a gate pass over a section where the two had been fused"*) is refuted by the
      probe check's own stated limit — it reads that the lines are there, never which sentence.
      The seeding is right; the reason given is false. `blind`
- [x] [Review][Patch] The identity guard's doc claims a section-level premise it does not carry:
      `_identity_section.html` has no branch that can emit a control, so no change to this story's
      code can red those four negative assertions. It is a tripwire for a future story and reads as
      a measurement of this one. `blind`
- [x] [Review][Patch] The ✅ *"`Gesture::Live` touches no site in `diagnostic.rs`"*, recorded as
      refuted-with-its-check, sits in a commit that edits `diagnostic.rs`, adds a `Live` arm to its
      template and a CSS class named after that arm. The E0004 claim was true; the sentence as
      written is not. `blind` [story §0i, sprint-status.yaml]
- [x] [Review][Patch] `sprint-status.yaml` and the 2026-08-24 Change Log entry still carry
      *"computed by `gap::reconcile`"* — the sentence §0i certifies FALSE nine lines below, warning
      that a developer who reads it *"would look in the wrong file"*. Only §0h was corrected.
      `blind+auditor`. `sprint-status.yaml` also still ends `# NEXT: dev-story.` under a `review`
      status.
- [x] [Review][Patch] EN renders *"The gestures marked Not yet are not built yet"*; the badge is
      interpolated unquoted, so the reader has no cue that *Not yet* is a mark. FR reads correctly.
      `blind`
- [x] [Review][Patch] `both_locales_carry_every_identity_key`'s `KEYS` is 15 while `app.yml` holds
      **17** `identity.*` keys — the two `unrecognised` ones are absent. Pre-existing, and this
      story is what re-sized that list. `auditor`
- [x] [Review][Patch] Record hygiene: two wall-clock figures for one measurement (4.66 s / 4.85 s);
      *"twelve mutations"* in the commit over a table whose M3 is a CONTROL and M8 a green — the
      story file wisely states no total and the commit reintroduces one; and *"the ceiling at
      2033"* appears in no commit (max committed is 1981) — plausible as an uncommitted
      intermediate, stated as a measurement. A fourteen-space gap in an assertion message.
      `blind+auditor`
- [x] [Review][Patch] AC5's *"every guard is measured RED"* covers **4 of the 9** new Rust guards.
      The uncovered set includes the carrier for **Guy's arbitration 1** and the carrier for
      **T2b** — the hole that would put a pressable 404 on the default configuration. `blind+auditor`
- [x] [Review][Patch] AC7's three stated limits are incomplete — they do not name what the act
      leaves on screen (see the first decision). `auditor`

#### Deferred

- [x] [Review][Defer] **A multi-homed sighting documents the address the operator was not looking
      at, and that row then becomes permanently un-documentable** — `edge`, MEASURED: two `Nouveau`
      rows from one observation, selecting `.202` writes `.201`, and a second press answers 409
      silently. Not reachable on the shipped ARP/ping connector (one `IpV4` per observation); it
      goes live on the first connector reporting a second address, and `multi-nic` is a committed
      trap family. AC1's *"the whole record at once"* reads as covering it and does not.
- [x] [Review][Defer] `hx-vals='{"subject": "{{ pane.subject }}"}'` builds JSON by interpolation
      and Askama's escaper is not a JSON escaper. Latent: the value is `ObsId::to_string()` at every
      call site — a property of the call sites, not of the type, exactly as 6b.4's review recorded
      for the missing URL escape. `blind`
- [x] [Review][Defer] Focus is moved INTO an `aria-live` region; several screen readers then
      announce twice or suppress the live announcement. Neither browser gate can see it. The
      template argues both are needed and neither substitutes — that argument is defensible and is
      now a stated limit rather than a settled fact. `blind`
- [x] [Review][Defer] `CLAUDE.md` and `docs/project-context.md` carry no story 6.4 paragraph. The
      merge-time convention here, but the branch is pushed and the repo's own
      *docs-current-before-push* rule names both files. `auditor`

#### Dismissed with the check that dismissed them

- **`identity_view.rs`'s `#[cfg(test)]` mention truncating the `file-size` gate** — the gate matches
  `trim_start().starts_with(...)` and the mention is prefixed `//!`. Refuted by `edge` and by me
  independently, on the same reading. My own first count had used a whole-string search: *the
  sloppy measurement was mine, not the gate's.*
- **`identity_section_of`'s `depth -= 1` underflow** — unreachable, the slice starts at the opening
  `<div>`. Refuted by `blind` and `edge` independently.
- **XSS through the new cause/why lines, and hostile subjects at the route** — `edge` ran the whole
  battery: nine malformed subjects all answer 422/404, uppercase and braced UUIDs are canonicalised
  before the write, and an `onerror` payload in `abstention_cause` renders entity-escaped.
- **`MIN_CHECKS = 27` sitting under what is there** — the clean run reports exactly 27.
- **Both gates over an empty store** — kbd exits 2, axe exits 2; neither passes over a surface it
  could not reach.
- **Whitespace-sensitive needles** (`"  border-color: …"`, the exact class order) — real, but
  `cargo fmt` does not touch CSS and the class order is pinned deliberately.

### The repair pass (2026-08-26) — what the review changed, and what it measured

**Guy's two arbitrations**, each with the option refused:

🔑 **(2) `Gesture::Disabled` — a THIRD state.** With the route unmounted the documenting control
rendered `Planned`, so the product said *"not built yet"* about the gesture it had just built, on
the configuration nearly every deployment runs. **Refused:** hiding the control (contradicts Guy's
premise (2) of 2026-08-13 — *show and label rather than hide* — and a fresh install would never
learn the product can document at all), and leaving it with a note in AC7. The variant carries the
SWITCH, interpolated from `main::DOCUMENT_ENABLED_ENV`, the constant the boot reads, so the screen
cannot send an operator to a variable the binary does not consult.

🔴 **And the boolean it replaced was conflating two facts.** `action_bar(key, primary_is_live)`
meant *the route is mounted* at one call site and *this kind of row is eligible* at two others —
harmless while every `false` rendered the same control, and false the moment the switched-off state
got its own words: an `Écart` pane would have told the operator to set `OPENCMDB_DOCUMENT_ENABLED`
for a gesture that is enabled and simply does not apply there. `PrimaryState { Acts, SwitchedOff,
NotBuilt }`, and the decision stays inside `action_bar` so no caller can hand `Live` to `Résoudre`.

🔑 **(1) Arbitration 2′ — the answer.** `HX-Redirect` on success, keyed bodies for all five
statuses, and `hx-on::before-swap` so a refusal is swapped at all. **Refused: re-rendering the
screen from the route**, and the type is what refuses it — the document sub-router's state holds no
pool (story 6.1's M4: adding one fails to compile), so a handler that rebuilt the triage body would
demolish that guarantee for a display convenience. ⚠️ **Story 6.2's 201-with-a-body contract is
untouched**: same status, same kind of body; only a browser running htmx reads the header.

🔴 **The entity id left the operator's screen and landed in the LOG**, where an administrator
correlating a write actually needs it. Two end-to-end tests were parsing it out of the sentence —
*a test that pins the ugly thing is a test that requires it* — and now read it from the store,
through a second `SANCTIONED_READS` entry added in the same act rather than left outside the
perimeter.

#### The repair's own mutations — predictions written BEFORE any was applied

| id | mutation | predicted | measured |
|---|---|---|---|
| M-R1 | `Disabled` reverted to `Planned` | RED 2 rust | ✅ RED 2 |
| M-R2 | `observed_meta` not updated on the overwrite | RED 1 rust | ✅ RED 1 |
| M-R3 | the confirmation answers for a count of 0 | RED 1 rust | ✅ RED 1 |
| M-R4 | the `HX-Redirect` header dropped | rust GREEN + kbd RED 2 | ✅ 503 green, kbd RED 2 — *"5 row(s) left, 1 live control(s)"*, the defect verbatim |
| M-R5 | `hx-on::before-swap` back on the button | rust RED 1 + kbd RED 2 | ✅ both |
| M-R6a | the first sighting wins again | RED ≥1 | ✅ RED 2 |
| M-R6b | the route flag ignored | RED ≥1 | ✅ RED 3 |
| M-R6c | the test wrapper offers the gesture | RED 1 | ✅ RED 1 |
| M-R6d | a `Nouveau` row carries a count | RED 1 | ✅ RED 1 |
| M-R6e | any primary may go `Live` | RED ≥1 | ✅ RED 1 |

**Ten mutations, ten conforming to prediction.** M-R6a–e are the five guards AC5 claimed and had no
recorded red for — including the carrier for Guy's arbitration 1 and the one for T2b.

🔴 **AND THE HANDLER WAS ON THE WRONG ELEMENT A SECOND TIME.** `hx-on::before-swap` was written on
the button, exactly as the focus handler had been, and the probe reported `status=409` with an
empty answer region: **`htmx:beforeSwap` is dispatched on the SWAP TARGET**, like `htmx:afterSwap`.
Two handlers, the same mistake, the same afternoon — and nothing but a browser could see either.

🔴 **My own attribute slicer then broke on the fix.** `split_once('>')` cut inside
`hx-on::before-swap="… status >= 400 …"`, so the guard reported the focus handler missing — story
5.12's `statement_after` defect, where a quote inside a SQL literal truncated the statement. *A
check that fails for the wrong reason is worth nothing.* The slicer now tracks quotes.

⚠️ **Three insertions landed before the wrong item in one session**, and each was named by a
compiler rather than by reading: a `#[cfg(test)]` that slipped off its function (**`cargo test`
stayed green — only `cargo build` reds, because in the test build the cfg is active**), a doc block
that merged into `AppConfig`'s and left it undocumented, and a `let` that landed inside a function
signature. The rule stands: *a repetitive edit must let the compiler name each site.*

⚠️ **And my mutation driver lied once, in the way this project has recorded five times**: M-R4's
anchor missed after rustfmt reflowed it, the script printed the traceback and CARRIED ON, and I
read a green from the unmutated tree. Re-run under `set -e` with the corrected anchor: RED 2. *A
driver that cannot stop on a failed apply is a driver that reports on the wrong tree.*

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

### The live count (AC6), and every figure names the state it was taken against

**729 → 741 tests** — 503 `opencmdb-bin` + 161 `opencmdb-core` + 77 `xtask`, `cargo test
--workspace --locked` against a live `mariadb:10.11.11` on port 13405, **6.19 s** wall clock
(**~0.2 s** with `DATABASE_URL` unset — the clock is the tell that the store-backed tests ran).
Nine `cargo xtask ci` gates green; clippy `--all-targets`, `cargo fmt --check`, each status read
from `$?` rather than through a pipe.

🔴 **THE BASELINE WAS WRONG TWICE, AND THE SECOND TIME IS THIS STORY'S OWN.** Three commits
carried *"737"* over parts summing to 735; the correction written here then read **735 → 738**,
which is the delta of ONE SESSION measured against `c4c187d` — story-commit **4 of 8**. The
story's baseline is `master`: **729** (491 + 161 + 77), measured in a worktree, so the story is
**+12**. ⚠️ *An intermediate state read as a baseline* is story 6b.1's recorded defect, verbatim,
and it was committed inside a paragraph invoking *verify both of its terms*. Found by the code
review's blind layer — from the diff alone, by counting the added `#[test]` attributes — and
independently by its acceptance layer.

**The browser gates**, which are where this story's centre of gravity is: `a11y/axe-gate.mjs`
walks 10 routes plus **4** states (the selected `Nouveau` pane and the post-gesture confirmation,
neither of which any href carries) for **0 violation nodes**; `a11y/kbd-probe.mjs` runs **30**
checks, 0 failed, **ten** of them new. ⚠️ *"27 checks, three of them new"* stood here and was
wrong on the decomposition: `MIN_CHECKS` had moved 20 → 27, and 20 + 7 = 27 is arithmetic inside
the diff.

### What the operator can DO (AC7), stated plainly

**On `/triage`, select a `Nouveau` row and press one amber control that WRITES.** The address and
its hostname become a declared record in one transaction, the queue's question disappears, and the
answer is swapped in and focused where a keyboard operator lands on it. That is the first gesture
in three epics whose honest answer is not *nothing*, and it is measured end to end by a browser
that presses it.

🔴 **AND THE FIRST VERSION OF THIS PARAGRAPH OVERSTATED THE ACT, which is what AC7 exists to
prevent.** It said *"the queue's question disappears"*; measured in Chrome by two review layers,
nothing on the screen moved at all. `hx-swap="innerHTML"` replaced one paragraph, so the row, the
amber button and the pane reading *« Rien de déclaré à cette adresse »* all stayed exactly as they
were — over a message saying the record had just been written — and a second press answered 409
and swapped **nothing**, leaving the first press's success sentence on screen asserting the
opposite. The paragraph above describes the product after Guy's arbitration 2′ of 2026-08-26; what
shipped before it did not do that.

⚠️ **And three limits, because AC7 exists so the route is not counted as the gesture:**

1. **On the DEFAULT configuration there is still no gesture.** `POST /document-all` is registered
   only under `OPENCMDB_DOCUMENT_ENABLED`, off by default since story 6.1, and the page renders the
   control only where the route is mounted. A stock deployment sees five labelled controls, none
   live — and, since the code review, the documenting one now says it is **built and switched
   off**, naming the variable, rather than *"not built yet"*. *The gesture exists; it is not yet
   what a fresh install meets.*
2. **The four other controls remain planned**, and the sentence under them names the badge it
   scopes itself to rather than claiming the whole bar is unbuilt.
3. **The identity reach section offers no gesture at all**, and now says why on every line. The
   answer there is a better SOURCE — Epic 11 — not a record the operator writes.

⚠️ **Two more, found by the review and stated rather than fixed:** the queue behind the
confirmation is correct because the whole screen is re-rendered, but **another tab is not** — it
learns of the write only by pressing and reading the refusal; and **a multi-homed sighting
documents the address the operator was not looking at** and then cannot be documented at all
(registered, unreachable on the shipped connector).

### The mutation pass (T6) — predictions written BEFORE any mutation was applied

⚠️ On the **unrepaired** driver, by Guy's sequencing of 2026-08-24. Every mutation was reverted from
a scratchpad copy taken beforehand, **never with `git checkout --`** — the gesture that destroyed
uncommitted work four times in this project. Carriers are MIXED and named per row; no *"every red
assertion-carried"* headline is claimed.

| id | mutation | predicted | measured | carrier |
|---|---|---|---|---|
| M-A1 | server booted without `OPENCMDB_DOCUMENT_ENABLED` | axe exit 2 | ✅ exit 2 | the gate's named refusal |
| M-A2 | the seeded undeclared sighting removed, flag kept | axe exit 2 | ✅ exit 2, same refusal | idem — **both halves are load-bearing** |
| M1 | `<p class="why">` deleted from the partial | RED 1 | ✅ RED 1 | its own assertion, the `why` needle |
| M2 | the tolerant arm returns the `ambiguous` sentence | RED 1 | ✅ RED 1 | idem, third needle only |
| M3 | **CONTROL** — the same guard's page with the route unmounted | RED 1, on the PREMISE | ✅ RED 1, on the premise's own message | assertion |
| M4 | `document_all` also deletes the subject's identity links | RED 1 | ⚠️ **RED 3** | mine on the reach equality, **plus story 6.3's two NFR5 guards** |
| M5 | the `Nouveau` loop stops filtering against `claimed` | RED ≥2 | ✅ RED 2 | assertions |
| M6 | `hx-on::after-swap` back on the button | RED 1 rust **and** RED 1 browser | ✅ both | a Rust assertion naming the CAUSE, a probe check naming the SYMPTOM |
| M7 | `.btn-gesture.btn-document` → `.btn-document` | rust GREEN + kbd RED 1 | ✅ 738 green, kbd RED 1 | the probe alone |
| M8 | the seed's `identity_link` block removed | axe GREEN | 🔴 ✅ axe exit **0**, and **zero** sentences served | *nothing* — the finding |
| M8-bis | the same, against the check M8 earned | kbd RED 1 | ✅ RED 1 | the new probe check |
| M9 | the axe gate loses its `REQUIRE_GESTURE` refusal, no flag | exit 0 | ✅ exit 0, **2 states where 3 are due** | *nothing* — which is why the refusal exists |

🔴 **M-K0 is not in the table because it was not planted.** The keyboard probe's first run found the
focus-after-swap contract BROKEN — `⏎ → status=201`, the answer swapped in, `activeElement.id`
**empty** — while the previous commit's record claimed focus moves and every Rust assertion about
that bar was green. `htmx:afterSwap` fires on the TARGET of the swap, a sibling of the button, so
the handler on the control never ran. *The render assertion pinned the attribute; the attribute
named a behaviour the product did not have.*

🔴 **M7 is the story's clean specimen of story 6b.11's amended AC5.** The whole Rust suite, the nine
gates and the sheet's own amber guard — which counts `var(--accent-document)` reads and found all
four, correct — stayed green over a control that computed to plain grey. *A declaration that loses
is still a declaration, and no source guard can see a cascade.*

🔴 **M8 is the pass's own finding and it cost a check.** The seed was widened so the reach section
would render its new sentences where the gates run; measured, the axe gate walks that page and
**asserts nothing about it**, so the copy was carried by no browser at all. *A gate that walks a
page is not a gate that reads it.* Closed by the probe check M8-bis reds.

⚠️ **M4 diverged from its prediction and the divergence is the useful part**: 3 red, not 1, because
story 6.3's NFR5 guards already pin the link itself. What this story's guard adds is the DISPLAYED
aggregate — the number the operator reads — and that is a different claim from *the row is intact*.


## Change Log

| Date | Change |
|---|---|
| 2026-08-26 | **CODE-REVIEWED (three isolated layers) and REPAIRED — 2 arbitrations by Guy, 23 patches, 4 deferrals, 6 dismissed with their check.** 🔴 **All three layers measured the same thing in a browser: the act left the screen unchanged.** The narrow swap replaced one paragraph, so the queue row, the amber button and the pane reading *« Rien de déclaré à cette adresse »* stayed put over a success message — and a second press answered 409 and swapped **nothing**, leaving the first press's sentence asserting the opposite. The body was a Rust `format!`: English under a French UI, `field(s)`, a raw UUID. **AC7's *"the queue's question disappears"* was FALSE.** ✅ **Guy 2′:** `HX-Redirect` + keyed bodies + `hx-on::before-swap`; re-rendering from the route was REFUSED by the type (story 6.1's pool-free state). ✅ **Guy (2):** `Gesture::Disabled` — with the route unmounted the product said *"not built yet"* about the gesture it had just built; hiding the control was refused as contradicting *show and label rather than hide*. 🔴 The boolean it replaced conflated *the route is mounted* with *this row is eligible*. 🔴 **The blind layer found the headline from the diff alone**, sixth story running: the live count was a SESSION delta — `master` is **729**, so the story is **+12**, not +3 — *an intermediate state read as a baseline*, story 6b.1's defect verbatim, committed inside a paragraph invoking *verify both terms*. 🔴 `htmx:beforeSwap` fires on the TARGET too: the second handler was on the button, the same mistake the same afternoon. ⚠️ **Ten repair mutations, ten conforming**, five of them the guards AC5 claimed and had never proved red. **729 → 741 tests**, nine gates, axe 10 routes + 4 states 0 nodes, kbd **30** checks. |
| 2026-08-25 | **IMPLEMENTED — T1 through T8, and the three sharpest findings came from a BROWSER rather than from a test.** 🔴 The focus-after-swap contract was BROKEN: `hx-on::after-swap` sat on the button, `htmx:afterSwap` fires on the swap TARGET, and the handler never ran — `⏎ → 201`, the answer swapped in, `activeElement.id` **empty**, with every Rust assertion about that bar green. 🔴 The amber **painted nothing**: `.btn-gesture.live` (0-2-0) beat `.btn-document` (0-1-0) on all four declarations while the guard that counts `var(--accent-document)` reads found all four and passed — *a declaration that loses is still a declaration*. 🔴 And this story made a shipped sentence FALSE — *"This gesture is not built yet."* under a bar whose loudest control now IS — caught by looking, fixed by interpolating the badge. ✅ **Guy's arbitration (2026-08-25): FILL the control with the amber** (contrast **5.57**) over darkening the token (**4.56**) or dropping the tint (**4.59**) — the other two pass by 0.06 and 0.09, a margin the next palette adjustment erases. 🔑 `page.rs` hit the `file-size` ceiling at 2033 and the answer was the SPLIT (`identity_view.rs`), never shorter prose. ⚠️ **M8 is the mutation pass's own finding**: the axe gate walks the reach section and asserts nothing about it, so T3's new copy was carried by no browser — closed by a 27th probe check. **735 → 738 tests** (the 737 three commits carried was wrong: 497+161+77 = 735), nine gates, axe 13 routes/states 0 nodes, kbd 27 checks 0 failed. |
| 2026-08-25 | **VALIDATED by two layers — nine findings, two reached by both, and the second layer BUILT the gesture and pressed it.** 🔴 Four holes the story did not know it had: `OPENCMDB_DOCUMENT_ENABLED` is invisible to the page builder, so a naive wiring shows a pressable control pointing at a **404** on the default configuration; the axe gate cannot reach a selected `Nouveau` row; `a11y/seed.sql` produces **zero** of them; and a plain `<form>` navigates to the raw text body. ⚠️ **`Gesture::Live` forces ONE site, not *every match*, and satisfying it naively ships an inert `<span>`** — 6b.4b's own warning, measured. 🔴 And two false citations of mine, one already propagated to the register: `epics.md:2118` is **`:1794`**, and §0h's *"produced by `gap::reconcile`"* is false — that enum has no `undeclared` variant. **✅ Two arbitrations (Guy):** the subject is the **most recent** sighting, not the oldest; and the gesture is **`hx-post`** with a target and a swap — ⚠️ which makes it the product's first live swap, inheriting 6b.11's focus-after-swap contract by name. |
| 2026-08-24 | ✅ **ARBITRATION TAKEN (Guy): the gesture goes on the triage queue's `Nouveau` row, and the abstention line carries none.** 🔑 The measurement that decided it: `undeclared` is NOT typeless — `page.rs:1005` already labels a queue row *"an observed address no declared entity claims"*, the glossary's own words, on screen since 6b.4 — ⚠️ **NOT** *"computed by `gap::reconcile`"*, which this entry asserted until story 6.4's code review and which §0i had certified false in the same commit: that enum has no `undeclared` variant. **The glossary left it typeless because it was looking at the identity engine; it lives on the reconciliation side.** *Refused:* AC1's letter (offers to create an entity for a machine already declared) and the intersection (safe, two reads and a join, and most of the population anyway on the shipped connector). 🔴 **The cost, stated:** the story's *so that* clause — *"the reach section becomes a door"* — is not met, and **the premise is what is wrong**: an identity abstention's answer is a better SOURCE, not a documented entity. *The reach section may have no door, and that is not this story's failure.* |
| 2026-08-24 | Story created and CONTEXTED. 🔴 **AC1 cannot be implemented as written**: it attaches FR13's gesture to `AbsenceOfProof`, an identity verdict about whether two sightings could be JOINED, while FR13's own population is `undeclared` — *observed, and no declared record claims it* — which the PRD's binding glossary gives a row and **no type**. ⚠️ A sighting can be `AbsenceOfProof` **and already declared**, a state `a11y/seed.sql` produces today. **This is story 5.14b's defect one layer down**, and that story's own record names the shape: *same word, different type, different population*. 🔴 Second measurement: `IdentityCauseRow { cause, count }` carries **no subject** and the route takes exactly **one** — so AC1 is not a wiring job but a read the page does not have. ⚠️ And on the shipped connector *"one line"* is the WHOLE unplaced population, since the scanner emits no hardware address ever. **One arbitration open (§0f).** |
