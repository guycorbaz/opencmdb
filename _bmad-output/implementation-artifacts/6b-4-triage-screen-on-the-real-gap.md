# Story 6b.4: The triage screen, on the real gap

Status: review

Epic: 6b — *L'interface de la maquette*. **Fourth story**, after 6b.1 put the design system in the
binary, 6b.2 gave the product ten addresses, and 6b.3 gave the example screens their marker. It is
the story that finally dresses **the one screen the product actually feeds**.

## Story

As the operator,
I want the gap the product really computes shown the way the mock shows it,
so that the one screen that is fed is also the one that looks best.

## Acceptance Criteria

Transcribed from `epics.md:2172-2186`, **unmodified** — the divergences are raised in §0 below rather
than edited into the criteria (a story may not edit an AC; only a retrospective may).

1. **Given** the gap the product computes today, **when** `/triage` renders, **then** it is the
   mock's two panes — **the queue, and the two photos side by side** — each side carrying **its
   provenance and its freshness** (the spec: *neither side is "the truth"*).
2. **Given** the gestures, **when** the action bar renders, **then** **only the gestures that exist
   are live**, and the others are shown and labelled per 6b.3. ⚠️ *And that includes « Merger »
   until story 6.4 lands* — field-level documenting is **FR13(b), Epic 7's**; what 6.4 ships is
   **FR13(a)** on the abstention line. *A developer meeting a dead primary button on the product's
   signature screen will want to fix it: this paragraph is why it is not a bug.*
3. **And** sorting by age is available **and off by default** — the ban is not that age is hidden,
   it is that age is never brandished: *sorting by age is the operator's action, never a pushed
   label*.

---

## §0 — What contexting found, and what needs Guy's arbitration BEFORE any code

Seven findings. **Nothing below was taken from reading a summary** — each carries the command or the
file:line that established it. Two of them ask a question this project has learned to ask late: *what
can the operator DO with what we are about to ship?*

### §0a. 🔴 THE SCOPE: this story carries FOUR deliverables, and each has been a whole story elsewhere in this epic

Measured against what exists today, the three AC paragraphs decompose into four independent bodies of
work, three of which start from **nothing at all**:

| Deliverable | What exists today | Measured at |
|---|---|---|
| **The QUEUE** | **Nothing.** `build_view` hard-selects ONE entity by `ipv4` and returns; there is no list of subjects, no selection, no *next item* | `page.rs:469-561` |
| **The two photos with provenance + freshness** | The two columns exist; **every provenance and freshness column is dropped at the `SELECT`** | `repo.rs:446-480`, `page.rs:122-136` |
| **The ACTION BAR and the dead-gesture label** | No action-bar markup anywhere; the labelling mechanism **does not exist** (§0b) | `grep -rn "action-bar\|Merger" templates/ locales/` → nothing |
| **Sort by age** | Nothing — no parameter, no toggle, no notion of age | `page.rs`, `repo.rs`, `templates/` |

🔑 **This is the shape Epic 5's retrospective named**: its six insertions were *all* found while
contexting a story that carried two ideas. This one carries four, and the fourth — the action bar —
needs an arbitration and an invented surface before a line of it can be written.

**→ ✅ ARBITRATED (Guy, 2026-08-19): SPLIT TAKEN. 6b.4b is INSERTED — Epic 6b → 13 stories.**
⚠️ `epics.md` is **NOT edited** (a story may not; only a retrospective may); the divergence is
registered instead.

- **6b.4** keeps **AC1 and AC3** — the queue, the two photos with their provenance and freshness, and
  the age sort. One coherent deliverable: *the screen shows the real gap, honestly and completely.*
- **6b.4b** takes **AC2** — the action bar, and the dead-gesture labelling mechanism it needs.

The reason is not size but **dependency**: AC2 cannot be written until §0b's mechanism is invented and
§0c's contradiction is arbitrated, and neither of those is about the triage screen — both are
product-wide decisions that will govern every screen after this one. ⚠️ **This paragraph first added
*"and AC1 can be built as it stands"*, which the validation REFUTED: reading `origin` for display reds
the `authorship` gate (§0h), so AC1 needs an arbitration of its own.** The split still holds — it is
now *three* arbitrations across two stories rather than two across one — but its argument is weaker
than first written, and the honest form is that argument. ⚠️ **The alternative is
defensible and must be recorded if chosen**: keep all three, accept that the story opens with two
arbitrations, and take them in one sitting.

### §0b. 🔴 "LABELLED PER 6b.3" NAMES A MECHANISM THAT DOES NOT EXIST — measured, not supposed

Story 6b.3 shipped exactly two operator-visible mechanisms, and **both classify CONTENT**:

- `_example_marker.html` — *this content is a demonstration, it does not come from your network*;
- `_not_built_yet.html` — *this screen is not built yet*.

Both hang off `Screen::nature()`, whose three variants — `Fed`, `Example(ExampleContent)`, `Empty` —
are properties of **a screen or a section**, never of **a control**. And `Screen::Triage` is `Fed`,
which by that type's own doc *"owes NO marker, and carrying one would be a lie in the other
direction"* (`screens.rs:72-73`).

🔴 **A dead « Merger » button on a fed screen is not example data.** It is real UI whose backend does
not exist — a different axis entirely. Left unresolved, a developer will either force `Nature` onto a
button (which would be **false**) or improvise a one-off treatment, **which 6b.3's own AC1 forbids in
so many words** (*"never a per-screen improvisation"*).

⚠️ `epics.md:2164` (6b.3's AC2) did anticipate the GRANULARITY — *"screen, section, **or control**"* —
but no text, badge or treatment for *"this button does not work yet"* exists anywhere: **not in the
mock** (every button there is live in the demo), not in the UX spec, not in 6b.3's shipped templates.

🔑 **This is 6b.3's own §0b, one story later and on a different axis**: *the mock cannot prevail,
because it has nothing to prevail with.*

**→ ✅ ARBITRATED (Guy, 2026-08-19): a GESTURE NATURE, closed in the TYPE.** An
`enum Gesture { Live { route }, Planned { owner } }`, one partial, one key pair — so adding a gesture
**forces declaring its state**, and `Live` **carries its route**, which makes *"a button that looks
live and calls nothing"* unrepresentable. Story 5.6's idiom and story 6b.3's, one axis over.

**The words**, a pair distinct from 6b.3's because they say a different thing to a different
population: **fr** — badge *« À venir »*, sentence *« Ce geste n'est pas encore construit. »*;
**en** — *"Not yet"*, *"This gesture is not built yet."*

**Refused, each with its reason:** a bare partial applied by hand (nothing obliges anyone to use it,
and a dead button can still pass for live); 🔴 **`<button disabled>` — refused on NFR25**: a disabled
button **leaves the tab order and disappears from a screen reader**, so the blind operator is not even
told the gesture exists; `aria-disabled="true"` on a non-activatable button keeps the announcement.
And a separate *"À venir"* group heading, which is honest but **diverges from the mock's form**, which
6b.2's *"the mock prevails"* forbids without an explicit decision.

⚠️ **And one thing deliberately NOT in the surface**: the owning story's number. *"Arrives in 6.4"* is
not information for the operator, and it turns the label into a **calendar, therefore a promise** —
exactly what story 5.14b refused. The owner lives in the type, never on the screen.

### §0c. 🔴 TODAY THE ACTION BAR WOULD BE ENTIRELY DEAD — and that collides with a recorded decision of Guy's

The mock's action bar carries **five** gestures: `Merger`/`Résoudre` (primary), `Accepter l'écart`,
`Mettre en veille`, `Rattacher`, `Exclure`. Measured against the product:

- **`Merger` is field-level and does not exist.** The only write surface is `POST /document-all`
  (`document.rs:167-176`), which adopts a **whole entity**, is behind `OPENCMDB_DOCUMENT_ENABLED`
  and **defaults to `false`** (`main.rs:246-253`), and **is called by no template** — no `<form>`, no
  button, nowhere.
- **`Résoudre`** needs FR16's ranked candidates. `link_candidate` is a table with no reader reaching
  `page.rs`. Epic 6's.
- **`Accepter l'écart`, `Mettre en veille`, `Rattacher`, `Exclure`** have **no state machine and no
  persistence at all** — no `snoozed`, `excluded` or `gap_accepted` column exists in any migration.
- And **story 6.4, which the AC names as Merger's unblocker, is sequenced AFTER Epic 6b**
  (`epics.md:2090`: *"6.1 · 6.2 · 6.3 → EPIC 6b → 6.4 → release v0.2.0"*).

**So AC2, applied literally today, ships an action bar of five buttons of which five are dead.**

🔴 **AND HALF OF THIS SECTION WAS NEVER AN OPEN QUESTION — it was put to Guy as one, and his own
premise had already answered it.** `epics.md:2092`, decision **(2)** of 2026-08-13, under the heading
*"this epic's premises and **not open questions**"*: *"the same rule applies INSIDE an implemented
screen — **the four gestures Epic 7 owns are visible and labelled**."* The count lands exactly:
*Accepter l'écart*, *Mettre en veille*, *Rattacher*, *Exclure* are **four**, and the fifth — *Merger* —
is covered by AC2's own sentence. **The *whether* is closed; only the form was open (§0b).**
🔑 *Reading a document as if it had not already decided is the same error as citing a line without
opening it* — this story's contexting made it twice in one day, once on a story number and once here.

🔴 **And that runs into a decision Guy already took, in the opposite direction.** Story 5.14b,
recorded in `deferred-work.md:3283`: the abstention section stays **DESCRIPTIVE** and does **not**
announce its gesture — *"because announcing an absent gesture is a promise"*. AC2 asks for five
announced absent gestures.

⚠️ **The two are reconcilable and Guy's premise (2) is the reconciliation** — it is later than 5.14b
and more specific: 5.14b governs a **descriptive section**, the premise governs **gestures inside an
implemented screen**. AC2's answer to *"a promise"* is *"labelled"*, and the label is what turns a
promise into a statement of intent. But
whether that holds for **five** dead buttons including the primary, on the product's signature screen,
is a judgement about the operator's experience and not a deduction. 🔑 **The question Epic 5's
retrospective found the method never asks, asked here on purpose: what does the operator DO with an
action bar in which nothing acts?** An honest answer may be *"show none of them yet"*, which the AC
does not permit and only Guy may authorise.

🔴 **AND THE VALIDATION MADE IT SHARPER THAN THE STORY HAD.** Verified gesture by gesture
(`grep -rn` over `templates/` and `locales/` → zero hits; `grep -rn "snoozed\|excluded\|gap_accepted"
migrations/*.sql` → zero hits): **the ONE gesture with a real live backend today —
`POST /document-all`, whole-entity adoption — has NO BUTTON IN THE MOCK'S ACTION BAR AT ALL.** The
mock's five buttons all sit at granularities that do not exist (field-level merge, ranked-candidate
resolve, snooze/exclude/attach state), while the one thing the operator could actually do is
**invisible to the mock's vocabulary**. 🔑 *So the bar is not merely all-dead: it does not even offer
the single live gesture.* That is the fact the arbitration should turn on.

### §0d. ⚠️ THE QUEUE IS THE LARGEST PIECE AND IT STARTS FROM NOTHING

`build_view` (`page.rs:469-561`) selects **one** perimeter entity — `OPENCMDB_ENTITY_IPV4`, or the
first declared entity carrying an `ipv4` — and reconciles it alone. There is no listing of subjects
awaiting triage, no selection state, no ordering, no count.

The mock's queue is one row per **gap**, carrying a kind (`Nouveau` / `Conflit` / `Écart` / `Absence` /
`Ambigu`), a title, an object id, the field diff, and a freshness on the right.

🔴 **THIS PARAGRAPH FIRST CLAIMED THREE OF THE FIVE KINDS HAVE NO PRODUCER. THE VALIDATION REFUTED IT
BY BUILDING ONE — FOUR OF FIVE ARE PRODUCIBLE TODAY.** The error was citing the wrong function:
`gap::project` (`gap/mod.rs:88`) is flat, but **`gap::reconcile`, which consumes it, already TYPES the
distinctions** — `AbstentionCause::NoObservedValue` and `AbstentionCause::ConflictingObservations`
(`gap/mod.rs:49-57`) are `Absence` and `Conflit`, and both already render on `/triage` today in the
abstentions section. Measured with four seeded scenarios against a live database:

```
PROBE queue rows = [("192.0.2.99", Nouveau), ("entity-absence", Absence),
                    ("entity-conflit", Conflit), ("entity-ecart", Ecart)]
```

So **`Écart`, `Absence` and `Conflit` are already typed by the engine**; **`Nouveau`** is a small new
computation (~15 lines: declared-ipv4 set minus observed-ipv4 set); **only `Ambigu` genuinely has no
producer** — FR16, Epic 6's.

⚠️ **And the validation found the cost the story had not**: `reconcile` is written for ONE perimeter,
so looping it per entity re-classifies every *other* entity's observations as `OutOfPerimeter` noise
on each pass — O(N·M) reconciles over the corpus. The row vocabulary is real; **the queue's
implementation must discard that noise per entity, and no existing code does.**

### §0e. ✅ PROVENANCE AND FRESHNESS ARE SUPPLIED BY THE SCHEMA ON BOTH SIDES — and the mock is WEAKER than the AC

Measured in `migrations/0001_initial.sql`:

- declared: `origin` (`manual|adopted|imported`, line 14), `origin_obs_id` (15), `actor_id` (16),
  `updated_at` (17);
- observed: `connector_id` (27), `observed_at` (28).

**Everything the AC asks for is already persisted.** What is missing is only the plumbing: neither
`repo::load_declared_attributes` (`repo.rs:446-459`) nor `repo::load_observation_facts`
(`repo.rs:463-480`) selects any of it — `observed_at` is read solely to `ORDER BY` and never returned
— and `KeyValue`/`GapRow` carry neither.

🔴 **The mock must NOT prevail on this one, and the divergence is worth stating before someone copies
it.** In the mock's fixture, `observedMeta` is consistent — `{source} · vu il y a {temps}` — while
`declaredMeta` is **not a freshness field at all in five of its eight rows** (*"Rien de déclaré"*,
*"Deux VM clonées, même empreinte"*). That is fixture laziness, not a design decision: the UX spec
(`ux-design-specification.md:1215-1219`) requires *"each with a source-tagged, timestamped
meta-line"*, the AC requires *"each side carrying its provenance and its freshness"*, and **the schema
supplies it**. Follow the AC and the spec here; the mock prevails on FORM, never on a fact it simply
did not populate.

⚠️ **Nothing in the codebase renders a relative time** (*"il y a 4 min"*). That convention has to be
written, and it is a display concern: `chrono` is already a dependency with `default-features = false`,
which story 5.14b's review found is *the real carrier* of the no-clock-in-the-view-builder guard —
**do not reach for a clock inside a pure builder without re-reading that finding.**

✅ **A SUSPICION OF MINE, MEASURED AND REFUTED — recorded so nobody re-chases it.** I suspected AC1's
freshness collided with `screens.rs`'s guard `the_shell_shows_no_last_observation`. It does not: that
guard scans exactly `_shell.html` and `_nav.html` (`screens.rs:610-613`), and its own comment excludes
`_gap_card.html` **by name**, verbatim (`screens.rs:607-609`): *"`_gap_card.html` is deliberately NOT
here: it SHOWS observed values, which is its whole job. What is banned is the last-observation INSTANT
(a `MAX(observed_at)`), and it is banned from the frame, which renders on all ten screens."* Per-row
freshness is a different fact from a different query. ⚠️ **The ban does still bite one way**: a
freshness widget hoisted into the header or the nav footer WOULD collide, and that global figure is
registered to **story 6b.5** (`deferred-work.md:3724`).

### §0f. ⚠️ FIVE MOCK-versus-SPEC CONTRADICTIONS — "the mock prevails" resolves them, registering them is still owed

Story 6b.2's governing arbitration is *"the mock prevails"*. It decides these; it does not excuse
leaving them unrecorded.

1. **The column ORDER is reversed.** The spec's Gap Diff (`ux-design-specification.md:1217`) is
   **Observed → Declared**; the mock renders **Déclaré → Observé** in both the queue row and the
   detail pane. A direct ordering contradiction, not a wording nuance.
2. **Action-row weight.** The spec (`:1205`) wants *"amber Document + **ghost** Accept-gap/Snooze/Exclude/Attach"*; the
   mock makes three of the four secondary rather than ghost, reserving ghost for `Exclure` alone.
3. **`Résoudre`'s scope.** The spec (`:1206`) swaps Document for Resolve on `ambiguous` ONLY; the mock uses it
   for `Conflit` too.
4. **The Resolve Panel** the spec names as a distinct component (`:1248`) does not exist in the
   mock — ambiguous rows go through the same sidebar as every other row.
5. **`create`** is a named gesture in the spec's interaction table (`:1350`) and has no button in the
   mock; creation is a side effect of pressing *Merger* on a `Nouveau` row.

⚠️ **Also measured, and it is a fact about the mock rather than a contradiction**: `oldestLine`
(*"Le plus ancien : 10 juillet"*) is **hardcoded** in the fixture, independent of the data. Nothing
says what *oldest* means against real rows — `first_seen_at`? the link's `valid_from`? — so that is a
decision, not a transcription.

### §0g. ⚠️ TWO INHERITED DEBTS LAND HERE, one by name and one by nobody

- 🔴 **THE `--accent` GUARD IS **NOT** THIS STORY'S, and the first draft of this section said it was
  — refuted by the validation, whose refutation is worth more than the row.** Register row (e)
  (`deferred-work.md:3319`) reads *"**story 6.4**'s Document button in that section will be
  legitimately amber … 6.4 must re-examine the guard … **Owner: story 6.4, conditional on 6b.1's
  ordering**."* ⚠️ **`6.4` and `6b.4` are DIFFERENT STORIES**: story 6.4 is *"The abstention line
  carries the gesture"* (`epics.md:1794`), sequenced **after the whole of Epic 6b**
  (`epics.md:2090`). The register predates Epic 6b's insertion, and three further rows (`:3307`,
  `:3413`, `:3502`) use *"story 6.4"* the same way. **The row's own reason cannot apply here**: under
  §0a's split this story ships **no live Document button at all** — that button is precisely what is
  deferred to 6.4 — so there is no legitimate amber in 6b.4's scope to justify re-examining the guard.
  🔑 *The trap is not a wrong line number; it is a story NUMBER that reads like this one.* What
  remains true and much weaker: `the_identity_sections_own_rules_never_reach_for_the_accent`
  (`page.rs:1405`) still guards the identity region, and this story renders beside it — **do not break
  it, and do not narrow it either**, because narrowing it here would spend story 6.4's arbitration
  before its author arrives.
- **NFR25 names this screen as a key view and NOBODY owns its check.** The PRD (`prd.md:1397-1402`)
  makes axe-core 0-violations a *blocking floor* on the **inbox** — the pre-mock name for `/triage`, mapped
  by the spec's own interaction table (`ux-design-specification.md:1350`:
  *"Resolving inbox items | **triage** | **triage**"*) and by the retired six-entry nav
  (`deferred-work.md:3667-3672`) — plus a scripted keyboard checklist on any PR touching it. Epic 6b's
  DoD demands axe-core green on the ten routes, and `deferred-work.md:3748-3755` records that **no
  story in the twelve owns running it**. ⚠️ This story is the most exposed: it is the one adding
  interactive controls. **Registered here rather than silently inherited or silently skipped.**

- ⚠️ **And a third, smaller one, found by VERIFYING rather than by reading.** `epics.md:2108` sets
  Epic 6b's Definition of Done as *"…and `cargo xtask ci` green — **seven gates**"*. There are
  **eight** since story 6.3 added `observed-immutable`. The contexting report this story was built
  from asserted that the drift was already registered; **measured, it is not** —
  `grep -n "seven gates" deferred-work.md` returns nothing. 🔑 *A citation believed is a citation
  wrong*: **three of the four register line numbers that report supplied were also wrong**, and every
  one in this story was re-measured before being written down. The DoD line itself is a
  retrospective's to fix — a story may not edit an epic — and it is registered by this story.

### §0h. 🔴 READING `origin` FOR DISPLAY REDS THE `authorship` GATE — measured, and contexting missed it entirely

**This is the validation's headline, and it changes §0a's argument.** AC1 asks for *provenance* on the
declared side. The gap-hunt widened the `SELECT` to include `origin`, as the AC asks, and ran the real
gate:

```
cargo run -p xtask -- ci
🔴 authorship  1 unsanctioned access(es) to declared_attribute:
    crates/opencmdb-bin/src/repo.rs:458: a read of declared_attribute names `origin` — FR13
```

`xtask/src/main.rs:1196-1199` allows **exactly ONE** reader of
`PROVENANCE_COLUMNS = ["origin_obs_id", "actor_id", "origin"]` (`:1205`), and it is the **test-only**
`read_declared_provenance_for_test`. Removing `origin` and keeping `updated_at` turns the gate green
again — confirmed both through `cargo xtask ci` and through the gate's own
`the_authorship_gate_is_green_on_the_real_tree`.

🔑 **So freshness is free and provenance is not.** `updated_at` is not a guarded column;
`origin` — the source badge the mock's provenance line needs — is. **This story would be the FIRST
legitimate PRODUCTION reader of a provenance column**, where story 6.2 had to arbitrate a named
`SANCTIONED_READS` entry for a test-only one (its §6.5).

**→ ✅ ARBITRATED (Guy, 2026-08-19): TWO READERS, and the sanction names only the display one.**
`load_declared_attributes` stays **unchanged** — `(key, value)`, no provenance — and it is what feeds
the comparison; a **new** `load_declared_provenance_for_display` returns `(key, origin, updated_at)`
and goes **only to the view**; `SANCTIONED_READS` gains **one** entry naming that path **and** that
function (the `(path, Option<fn>)` shape already exists, and story 6b.3's review measured yesterday
that a name-only key lets a write through from elsewhere).

🔑 **Why a bare sanction on the existing reader was REFUSED, and it is not a style point**:
`load_declared_attributes` is consumed at `page.rs:568` by `reconcile_view` → `build_view`, **which IS
the divergence computation**. Sanctioning it would put provenance inside the scope of the very path
the gate exists to protect, and the gate's own doc says it guards against that *"BY ACCIDENT"*. With
two readers the divergence computation is **structurally unable** to see provenance, because it is
never passed it — and the gate goes back to being the tripwire it claims to be instead of the only
thing holding.

⚠️ **The cost, stated rather than discovered later**: two reads of `declared_attribute` instead of
one, joined at the view. ⚠️ **And the limit, written rather than implied**: this is a **TRIPWIRE, never
a barrier** — nothing stops a future story routing provenance into `build_view` another way; the real
closure is a database privilege, which story 5.12 already registered as its own voie B.

🔴 **The consequence for §0a: AC1 needed an arbitration too, and the split's stated rationale was
therefore incomplete.** The story first argued *"AC2 cannot be written until two things are settled,
and AC1 can"*. False. **T0 is THREE arbitrations, not two**, whichever scope Guy chooses — and the
third is on a gate six stories deep in this project's history that this story's contexting never
mentioned.

---

## Dev Notes

### What exists today (read, not assumed — `master` at `5075fe0`)

- **`crates/opencmdb-bin/src/page.rs`** (799 code lines, 1989 total) — `triage` (`:731`),
  `gap_fragment` (`:757`), `TriageState { pool, perimeter }` (`:696`), `triage_router` (`:710`),
  `reconcile_view` (`:567`), `build_view` (`:469`), `build_identity_view` (`:383`), and the view
  structs `ReconciledView` (`:209`), `KeyValue` (`:122`), `GapRow` (`:127`), `AbstentionRow` (`:133`),
  `IdentityView` (`:183`). 🔑 **`triage`'s own doc already names this story**: *"Story 6b.4 replaces
  this body with the mock's two-pane triage; the frame it renders into stays."*
- **`templates/_gap_card.html`** (98 lines) — the current body. One `<section id="gap-card">`, an
  entity heading with the HTMX refresh (`hx-get="/gap"`, `hx-target="#gap-card"`,
  `hx-swap="outerHTML"` — a manual button, never a poll), a two-column declared/observed `<dl>` pair,
  the gap list, the abstention list, and story 5.14b's identity section. ⚠️ **The identity section is
  OUTSIDE the `has_entity` gate on purpose** — a fresh install that has scanned and declared nothing
  is the default at first boot, and that is the deployment it is for. Do not fold it back in.
- **`crates/opencmdb-bin/src/repo.rs`** (181 code lines) — `load_declared_attributes` (`:446`),
  `load_observation_facts` (`:463`), `count_engine_reach`. **This is where the provenance and the
  freshness are lost**, and the columns are all there (§0e).
- **`crates/opencmdb-core/src/gap/mod.rs`** — `project` (`:88`) returns `Vec<(String, String)>`. It is
  `pub` since story 6.2 so the documented fields ARE the compared fields. ⚠️ **It carries no
  provenance and no timestamp, and it is in `opencmdb-core`** — D47 forbids that crate `sqlx`, `axum`,
  `askama` and `anyhow`. Provenance is an ADAPTER concern; do not push a `DateTime` through the domain
  to save a struct.
- **`crates/opencmdb-bin/src/screens.rs`** (305 code lines) — `Screen::Triage` is `Nature::Fed`, which
  is why it stays on the pool-bearing router while the other nine are pool-free.
- **`locales/app.yml`** — **63** key pairs, `fr` + `en`, guarded by `every_key_carries_both_locales`.
- **`assets/app.css`** — the mock's tokens since 6b.1; `.grid`, `.screen-section`, `.example-marker`
  and `.not-yet` since 6b.3. ⚠️ `--accent-document` (amber) is **reserved for the documenting
  gesture** and guarded — see §0g, this story is the one that must re-examine that guard rather than
  merely satisfy it.

### The seams that fight back — measured by the validation, not predicted

- 🔴 **`sqlx` carries NO `chrono` feature** (`crates/opencmdb-bin/Cargo.toml:51`), so a `DATETIME(6)`
  column **cannot be decoded natively**. Every timestamp must go
  `DATE_FORMAT(col, '%Y-%m-%dT%H:%i:%s.%fZ')` then `chrono::DateTime::parse_from_rfc3339`. 🔑 **That
  pattern already exists, verbatim, at `repo.rs:371`** in `load_observation_by_id` — **reuse it or
  extract it, do not rediscover it** (the DRY rule, and the story's first draft cited neither).
- 🔑 **`read_declared_provenance_for_test` (`repo.rs:182-216`) is the closest existing analog** — a
  `#[cfg(test)]`-only reader already selecting all seven columns including `origin` and `updated_at`.
  ⚠️ **Copying its shape into production runs straight into §0h's gate collision.** It is the right
  model and the wrong sanction.
- ⚠️ **`uuid` is declared with `features = ["v7"]` only** (`Cargo.toml:61`): `Uuid::new_v4()` is
  `E0599`. The house convention is `now_v7()`, at 40+ call sites.
- ✅ **No D47 problem**: the whole plumbing lives in `opencmdb-bin`; `opencmdb-core` is untouched by
  the validation's spike, and it must stay that way — a `DateTime` has no business in the domain.

### The house rules this story will be judged against

- **Prove-to-red**: a guard is observed failing before it passes, and the mutation is recorded.
  🔴 **Write fewer rows and play every one.** Story 6b.2 prescribed eighteen and executed seven, and
  both holes that reached production were rows written at contexting and never played. Story 6b.3
  wrote seven and ran nine — and its code review still found **three of its recorded results false**,
  because the table had been believed rather than replayed. **A row you will not execute is worse than
  no row.**
- 🔴 **A guard placed where the defect cannot occur reads as coverage and is none.** Epic 5's dominant
  class, reproduced in every Epic 6b story so far. **Reading a guard cannot find it** — only running
  the mutation can.
- 🔴 **A status code is not a look, and neither is a green suite.** Story 6b.3's two defects were both
  found by reading the rendered screen; neither was reachable by any test. ⚠️ **And no browser has
  been available for three consecutive stories** — this is the story where that stops being cheap:
  a queue, a sticky sidebar and an action bar are layout, and layout is what a text dump cannot show.
  Say plainly what was and was not verified by eye.
- Doc comments must be TRUE; prefer the weaker true sentence. `#![deny(missing_docs)]` is live on
  `opencmdb-bin`.
- **No source file over 2000 lines of CODE** (`file-size`; tests do not count). `page.rs` is at **799**
  and this story is the largest addition it has ever had — **split into a module rather than growing
  it**, on story 6.3's precedent (`xtask/src/observed_immutable.rs`).
- **DRY with its exception**: mutualise the two panes rather than duplicating markup — but the
  declared/observed asymmetry is deliberate in places and pinned by tests; do not collapse a
  redundancy a comment labels as intentional.

### Testing

- `cargo test --workspace`, `cargo clippy --workspace --locked -- -D warnings`, `cargo fmt --all`,
  `cargo xtask ci` (**eight** gates — ⚠️ `epics.md:2108`'s DoD line still says *"seven gates"*
  and is stale, and **measured: it is registered NOWHERE** — see §0g; `views-hash` reports `ℹ STALE` by design and must NOT be
  regenerated inside a story).
- ⚠️ **`DATABASE_URL` is unset locally and database-backed tests then return early and pass in
  silence.** The tell is the clock: the bin suite runs in **0.06 s** locally against **5.5–5.9 s**
  with a live `mariadb:10.11.11`. 🔴 **This story is almost entirely database-backed** — every new
  `SELECT`, every view field, the queue itself — so a local green proves very nearly nothing here.
  **Run it both ways and record both figures**, as stories 6b.2 and 6b.3 did.
- Baseline to start from: **614 tests** (387 bin + 161 core + 66 xtask), eight gates green,
  `master` at `5075fe0`.

## Tasks / Subtasks

**✅ SCOPED BY GUY'S ARBITRATION (2026-08-19): AC1 and AC3 only. AC2 — the action bar and its gesture
nature — is story 6b.4b's**, whose form and words are already arbitrated in §0b so that story starts
with its design settled rather than with a question.

- [x] **T0 — the arbitrations, taken BEFORE any code** (§0a, §0b, §0h): ✅ **the split is TAKEN**
      (6b.4b inserted, AC2 moves); ✅ **the gesture nature is decided in the type with its words**
      (6b.4b's to build, settled here so it starts designed); ✅ **production may read `origin` for
      display through a SECOND reader**, with one named `SANCTIONED_READS` entry — the existing
      reader stays provenance-free because it feeds the divergence computation. ⚠️ §0c's other half
      was never open: Guy's premise (2) of 2026-08-13 had already decided that unbuilt gestures are
      *visible and labelled*
- [x] **T1 — the provenance and freshness plumbing** (AC1, §0h): ⚠️ **`load_declared_attributes` is
      NOT widened** — it feeds the divergence computation and stays `(key, value)`. Add
      `load_declared_provenance_for_display` returning `(key, origin, updated_at)`, and widen
      `load_observation_facts` to `(connector_id, observed_at, facts)`. Then **one named
      `SANCTIONED_READS` entry**, path AND function. ⚠️ **Not through `opencmdb-core`** — D47, and
      `gap::project` has no business holding a timestamp. 🔑 The `DATE_FORMAT` + `parse_from_rfc3339`
      pattern **already exists at `repo.rs:371`** — reuse it, `sqlx` has no `chrono` feature
- [x] **T2 — the relative-time convention** (AC1): *"il y a 4 min"*, in both locales, as a display
      concern. 🔴 **And WRITE the clock guard for `build_view`, do not inherit one**: the validation
      measured that `the_view_builder_has_no_clock_so_one_store_renders_identically` calls `build_view`
      with EMPTY data, so it proves clock-freedom for `build_identity_view` and, for the function this
      story fills, **nothing** — and `SystemTime::now()` compiles freely where `chrono::Utc::now()`
      does not
- [x] **T3 — the queue** (AC1): its row vocabulary is **measured, not chosen** (§0d) — `Écart`,
      `Absence` and `Conflit` come from `gap::reconcile`'s own types, `Nouveau` is a ~15-line set
      difference, and **`Ambigu` is omitted because it has no producer** (FR16, Epic 6's). Plus its
      query, ordering, selection and empty state. ⚠️ **Discard the `OutOfPerimeter` noise per entity**
      — `reconcile` is written for ONE perimeter and looping it is O(N·M)
- [x] **T4 — the two photos** (AC1): the detail pane, each side with its meta-line. ⚠️ Follow the AC
      and the spec on the declared side, **not the mock's unpopulated fixture** (§0e)
- [x] **T5 — sort by age, OFF by default** (AC3), with a guard that reds if the default flips —
      *the ban is not that age is hidden, it is that age is never brandished*
- [x] **T6 — leave the `--accent` guard ALONE, and verify it still holds** (§0g). ⚠️ **Its narrowing
      belongs to story 6.4, not to this one** — the validation refuted the first draft's claim that
      the register assigned it here. This story renders beside the identity region: keep the guard
      green, do not widen it, do not narrow it
- [x] **T7 — LOOK at the screen**, `OPENCMDB_LOCALE=fr`, against a live database with real rows.
      🔴 **And say whether a browser was used.** Three stories running have shipped without one
- [x] **T8 — the register**, both directions (§0f, §0g). ⚠️ `grep -n "6b.4"` is **provably
      insufficient** — story 6b.3's review found a row its own contexting was quoting that a name-grep
      could not surface, *because the row never named the story*. Search the SUBJECTS too
- [x] **T9 — prove-to-red**, predictions written FIRST, **and every prescribed row executed**

## Prove-to-red — deliberately short

🔑 Six rows, chosen so that every one of them WILL be played. Story 6b.2 wrote eighteen and played
seven; story 6b.3 wrote seven, played nine, and had three results refuted at review for having been
believed rather than replayed.

| # | Mutation | Prediction |
|---|---|---|
| M1 | Drop `observed_at` from the widened `SELECT` | the freshness guard reds. ⚠️ **Predict the CARRIER, not just the colour** — if it reds by `.expect()` on a missing column rather than on an assertion, say so |
| M2 | Flip the age sort's default to ON | ✅ **ALREADY MEASURED by the validation, on a spike, and the warning reproduced LIVE**: a guard pinning the default's exact ORDER reds (`left: [newest, middle, oldest]` / `right: [newest, oldest, middle]`); a guard asserting only that *the toggle changes something* **stays GREEN under the exact mutation it exists to catch**. Write the first shape; the second is the dominant defect class |
| M3 | Return an empty queue from the query | the empty state renders and the count guard reds — **and if the whole suite stays green, the queue is tested by nothing** |
| M4 | Give the declared side the observed side's meta-line | reds. This is AC1's *"neither side is the truth"* made measurable rather than asserted |
| M5 | Reach for `--accent-document` from a rule this story adds near the identity region | the existing guard reds — this story must not break it. ⚠️ **It must not NARROW it either**: the narrowing is story 6.4's, and §0g records why the first draft got that wrong |
| M6 | Render a relative time from `SystemTime::now()` inside the pure builder | 🔴 **MEASURED AND THE PREDICTION IS REFUTED — the guard does NOT red.** `chrono::Utc::now()` fails to COMPILE (`E0599`, no `clock` feature), but `SystemTime::now()` compiles freely, and a minute-bucketed clock wired into `build_identity_view` left `the_view_builder_has_no_clock_so_one_store_renders_identically` **GREEN** (two calls microseconds apart floor to the same minute). ⚠️ **And the stronger half**: that guard calls `build_view` with EMPTY declared/observations, so a clock in `build_view`'s POPULATED branch — where freshness will live — is **never reached at all**. The guard proves clock-freedom for `build_identity_view` and, for `build_view`, **nothing**. This story must write that guard, not inherit it |

## References

- `_bmad-output/planning-artifacts/epics.md:2172-2186` — the acceptance criteria, verbatim
- `_bmad-output/planning-artifacts/epics.md:2086-2109` — Epic 6b's premises and its six measured constraints
- `_bmad-output/planning-artifacts/ux-design-specification.md:250-255` — *"neither side is the truth"*, the sentence AC1 quotes
- `_bmad-output/planning-artifacts/ux-design-specification.md:1202-1219` — the Triage Card and Gap Diff anatomy (and §0f's contradictions)
- `_bmad-output/planning-artifacts/ux-design-specification.md:1442-1454` — the hard bans, including the age ban AC3 serves
- `_bmad-output/planning-artifacts/prd.md:881-897` — FR10, FR11, FR16, FR16b · `:1397-1404` — NFR25, NFR26
- `_bmad-output/implementation-artifacts/6b-3-example-data-marker-and-its-gate.md` — §0b's precedent (a marker no document specified), and the four lessons its review left
- `_bmad-output/implementation-artifacts/deferred-work.md:3283` — *"announcing an absent gesture is a promise"* (§0c) · `:3319` — the `--accent` narrowing · `:3724` — the last-observation deferral · `:3748-3755` — the unowned axe-core check
- `crates/opencmdb-bin/src/page.rs`, `repo.rs`, `screens.rs`, `templates/_gap_card.html` — what exists today
- The mock: `~/travail/Projets actuels/opencmdb/documents/opencmdb - maquette.html` — **outside this repository**, 496 KB in 390 lines of which **two** (376 and 388) carry ~96% of the bytes — a bundled export, not raw JSX. Search it by decoding the blob; `grep -n` degenerates to a single useless hit on those two lines

## Validation record — two fresh-context layers, 2026-08-19

**Mandatory here** (Guy, Epic 4 retrospective): every story is validated by two fresh-context agents
before `dev-story`. Both ran on a different model, each in its own git worktree.

**Layer 1, fact-check — 61 assertions verified one by one, 59 confirmed, 2 REFUTED, both mine.**
🔴 **The load-bearing refutation is a STORY NUMBER, not a line number**: register row (e)'s
*"Owner: story 6.4"* does **not** mean this story. Story 6.4 is *"The abstention line carries the
gesture"* (`epics.md:1794`), sequenced after the whole of Epic 6b, and three further rows use the name
the same way — the register predates Epic 6b's insertion. So the `--accent` narrowing is **not owed
here**, T6 and M5 were rewritten, and *the trap was a number that reads like this one's*. Also
refuted: 63 locale keys, not 62. Two claims were confirmed but weaker than written (the mock is 390
lines of which two carry 96% of the bytes; one bracketed "quotation" was a paraphrase, now verbatim
with its line).

**Layer 2, gap-hunt — it BUILT the plumbing** against a live `mariadb:10.11` and measured every seam.
🔴 **Its headline is the finding contexting missed entirely: reading `origin` for display REDS the
`authorship` gate** — `SANCTIONED_READS` allows one test-only reader of the three provenance columns,
so this story would be the first legitimate PRODUCTION one and needs its own arbitration (§0h).
**That refutes the split's first rationale**, which had said AC1 was arbitration-free. 🔴 **Second
refutation: four of the mock's five row kinds are producible today, not two** — the story cited
`gap::project` where `gap::reconcile` already types `Absence` and `Conflit`; proven with four seeded
scenarios. 🔴 **Third: M6's prediction is wrong** — the clock guard does not red on
`SystemTime::now()`, and it never reaches `build_view`'s populated branch at all. ✅ **And M2's
warning reproduced live**: the good guard reds, the *"does the toggle do something"* guard stays green
under the exact mutation it exists to catch.

🔑 **What this pass changed, and it is why the step is not optional**: one arbitration was ADDED
before a line of code, two of the story's own claims about the engine and the guards were replaced by
measurements, four undocumented seams were named (`sqlx` has no `chrono` feature, so timestamps go
through `DATE_FORMAT`; the pattern already exists at `repo.rs:371`; `read_declared_provenance_for_test`
is the right model with the wrong sanction; `uuid` has no `v4`), and the operator question was
answered more sharply than the story had asked it — **the one live gesture has no button in the mock
at all**.

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context), 2026-08-19. Built and mutated against a live `mariadb:10.11.11` on
port 13421.

### Debug Log References — the mutation pass, every prescribed row EXECUTED

🔑 **Six rows were prescribed and EIGHT were run** (M7 and M8 were added when the work created guards
the table had not imagined). **Carriers are named per row; *"every red assertion-carried"* is NOT
claimed** — M3's first red is a `.expect()` panic and M8's is a gate message.

| # | Mutation | Predicted | MEASURED | Carrier |
|---|---|---|---|---|
| M1 | freeze `observed_at` inside `load_observation_facts` | the freshness guard reds | 🔴 **GREEN on first measurement — and that is this story's finding.** Every unit test built `ObservedBatch` BY HAND, so the column round-trip T1 exists to add was carried by **nothing**. Closed by a route-level assertion computing the expected day count from the stored instant; re-run: **1 red**, on its named assertion | named assertion, *after* the hole was closed |
| M2 | the age sort defaults to ON | AC3's guard reds | ✅ **2 red** — the order-pinning guard and the route test. ⚠️ The validation had measured the OTHER shape (*"does the toggle change something"*) staying green under this exact mutation; the shape shipped is the one that reds | named assertion |
| M3 | the queue emptied before render | the empty state renders, the count guard reds | ✅ **9 red** — ⚠️ **carriers MIXED**: the first is `.expect("the first row is selected")`, a PANIC, not an assertion. Recorded rather than smoothed | panic (`.expect`) + named assertions |
| M4 | the declared side gets the observed side's meta-line | reds — AC1's *"neither side is the truth"* made measurable | ✅ **4 red** | named assertion |
| M5 | the new CSS reaches for `--accent-document` | story 6b.1's reservation guard reds | ✅ **1 red**, and its message confirms the count is still **zero** legitimate uses — this story neither broke the guard nor narrowed it, which was Guy's ruling | named assertion |
| M6 | a clock read INSIDE the pure builder | ⚠️ the validation predicted 5.14b's guard would NOT catch it | ✅ **4 red on the new guard** — and 🔴 **story 5.14b's guard, run alone on the SAME mutation, stays GREEN**. The validation's finding is now a measurement rather than an argument, and it is why the new guard was written | named assertion |
| M7 | delete the rule for a class a template names | *(not prescribed — written after the CSS work)* | ✅ **1 red**, naming the file and the class | named assertion |
| M8 | `origin` read by the COMPARISON's reader | *(not prescribed — written after §0h's arbitration)* | ✅ the `authorship` gate goes **RED**, which is the two-reader shape doing its job | gate message |

⚠️ **Every mutation ran on a scratchpad-restored base, never `git checkout --`** — the gesture that
destroyed an uncommitted fix in stories 6.1 and 6b.3. Each restore was verified by md5 before the
next row.

### Completion Notes List

🔴 **FOUR DEFECTS WERE FOUND BY LOOKING AT THE SCREEN, and no test could reach any of them.**

1. **The Absence pane contradicted itself.** It read *"Déclaré: 2 champ(s) déclaré(s)"* over a
   meta-line saying *"Rien de déclaré"* — two opposite sentences about one side, because a cause row
   was passed `None` provenance. Fixed: a cause row's meta-line is the entity's most recent declared
   write.
2. **The observed source was a raw UUID.** *"cccccccc-0000-0000-0000-00000000unif · il y a 4 min"* —
   it tells the operator nothing and pushes the freshness off the line. 🔴 **And my own route test
   REQUIRED it**: it asserted `contains(<full uuid>)`, so it passed on the defect. *A test that pins
   the ugly thing is a test that requires it.* Now a short labelled id, with the whole UUID asserted
   **absent**, and the missing connector registry registered against story 6b.8.
3. **A `Nouveau` row rendered an arrow pointing from nothing** — *"→ observé 192.0.2.99"*. The diff
   now needs BOTH sides.
4. **A cause row showed a bare count** beside an address — *"192.0.2.20  2"*. It carries its unit now.

🔴 **AND A FIFTH, FOUND BY RENAMING RATHER THAN BY LOOKING**: `page::tests::triage_html` was a helper
that rendered `GapFragment` directly and never touched the route. **The entire body of `/triage` was
replaced and all 387 bin tests stayed green.** Ten tests were built on it. *A helper named for a route
it does not serve is the dominant defect class wearing a filename, and reading it could not find that,
because it is correct about what it renders.* Renamed to `gap_card_html`; `/triage` now has a
route-level test that drives the real router against a real database.

**AC by AC:**

- **AC1 — MET.** The queue and the two photos, each side carrying its own provenance and its own
  freshness, on the gap the product really computes. ⚠️ **The declared side follows the AC and the UX
  spec, not the mock**, whose fixture leaves five of its eight declared meta-lines with no date at
  all — fixture laziness, not design (§0e).
- **AC2 — NOT IN SCOPE.** The action bar is story 6b.4b's by Guy's arbitration; its mechanism, form
  and words are already settled in §0b so that story starts designed.
- **AC3 — MET.** The sort is offered and **off by default**, and the guard pins the ORDER rather than
  the toggle's existence — the shape the validation measured green under its own mutation.

🔑 **Two guards were added that no acceptance criterion asked for, each because a defect had already
happened**: `build_triage_reads_no_clock_of_its_own` (5.14b's guard proves nothing about
`build_view`, measured under M6) and `every_class_a_template_names_is_defined_in_the_stylesheet`
(story 6b.3 shipped `.screen-section` defined nowhere, and only a recount found it). ⚠️ **The second
states its own limit**: it reads `class="…"` literals, so a class built in Rust is invisible to it,
and **it says nothing about whether an existing rule is the RIGHT one** — `.rows` on a `<table>` would
still pass. *Only a browser answers that.*

**611 → 625 tests** (398 bin + 161 core + 66 xtask). Eight gates green, fmt and clippy clean, and the
suite was run BOTH ways: **0.07 s without a database and 5.15 s against a live `mariadb:10.11.11`**,
which is the tell that the database-backed half really executed.

⚠️ **T7 was a real look**, in French, against a live database with seeded rows — a drift, an absence,
a conflict and an undeclared address — read as rendered text through the running server. **It is
still not a browser**: the CSS was recounted, never seen, so typography, spacing, contrast and the
sticky sidebar's behaviour remain unverified by eye, for the fourth consecutive story.

### File List

| File | Change |
|---|---|
| `crates/opencmdb-bin/src/repo.rs` | **`load_declared_provenance_for_display`** (the second reader, §0h), `DeclaredProvenance`, `ObservedBatch`, and `load_observation_facts` widened to carry the source and the instant |
| `crates/opencmdb-bin/src/page.rs` | `TriageView`/`QueueRow`/`DetailPane`/`MetaLine`, `build_triage`, `relative_time`, `source_label`, `declared_meta_line`, `origin_key`, `url_escape`, `now_utc`, `triage_view`, the rewired `triage` handler and `TriageQuery`; the renamed `gap_card_html`; nine new guards |
| `crates/opencmdb-bin/src/main.rs` | the route-level `/triage` test, through the real router against a live database |
| `crates/opencmdb-bin/templates/_triage.html` | **new** — the two panes |
| `crates/opencmdb-bin/templates/_identity_section.html` | **new** — story 5.14b's reach section, EXTRACTED (a pure move) so the body swap did not delete it |
| `crates/opencmdb-bin/templates/_gap_card.html` | includes the extracted partial; `/gap` renders as before |
| `crates/opencmdb-bin/assets/app.css` | the triage screen's rules — and every class it names is now defined, which a new guard asserts |
| `crates/opencmdb-bin/locales/app.yml` | 22 keys, both locales, the relative time INTERPOLATED because the word order differs |
| `xtask/src/main.rs` | `SANCTIONED_READS` 1 → 2 entries, with the shape's doctrine |
| `_bmad-output/implementation-artifacts/deferred-work.md` | five rows |

### Change Log

| Date | Change |
|---|---|
| 2026-08-19 | Implemented, scoped to AC1 + AC3. 611 → **625 tests**, eight gates green. Eight mutations run, eight measured — 🔴 **M1 came back GREEN and that is the finding** (the widened column's round-trip was carried by nothing), and 🔴 **M6 reds the new clock guard while story 5.14b's stays GREEN on the same mutation**. **Four defects found by LOOKING**, none reachable by any test — and a fifth found by renaming a helper: the whole body of `/triage` had been replaced with all 387 bin tests green. |
| 2026-08-19 | **Arbitrated by Guy (three rulings, before any code).** (1) The SPLIT is taken — **6b.4b INSERTED, Epic 6b → 13 stories**; this story keeps AC1 + AC3. (2) The dead-gesture mechanism is a **gesture nature closed in the TYPE**, with its words fixed here so 6b.4b starts designed — and 🔴 `<button disabled>` **refused on NFR25**, because a disabled button leaves the tab order and vanishes from a screen reader. (3) `origin` may be read for **display**, through a **SECOND reader**, so the divergence computation stays structurally unable to see provenance. 🔴 And one finding about the contexting itself: half of §0c was **never an open question** — Guy's premise (2) of 2026-08-13 had already decided that unbuilt gestures are *visible and labelled*. |
| 2026-08-19 | Validated by two fresh-context layers. Fact-check: 61 assertions, 59 confirmed, **2 refuted, both mine** — the `--accent` row belongs to story **6.4**, a different story, and the trap was a number that reads like this one's. Gap-hunt: it BUILT the plumbing and found that **reading `origin` for display reds the `authorship` gate**, which adds a THIRD arbitration and refutes the split's first rationale; that **four of five row kinds are producible today, not two**; and that **M6's clock guard does not red** and never reaches `build_view` at all. |
| 2026-08-19 | Contexted: seven findings, three needing Guy's arbitration. The story carries four deliverables and a SPLIT is recommended; *"labelled per 6b.3"* names a mechanism that does not exist; and applied literally today the action bar would ship five buttons of which five are dead, against a recorded decision that *announcing an absent gesture is a promise*. |
