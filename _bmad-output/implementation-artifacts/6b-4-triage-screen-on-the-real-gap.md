# Story 6b.4: The triage screen, on the real gap

Status: ready-for-dev

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

**→ RECOMMENDED, and put to Guy rather than taken:** **SPLIT**, with **6b.4b INSERTED** (Epic 6b → 13
stories):

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
because it has nothing to prevail with.* It needs an arbitration on **the form and the words**,
recorded WITH the alternatives refused — a disabled button with a tooltip, a ghost button with an
inline note, a labelled group heading, or the gesture simply absent until it exists each say something
different about what the operator should expect.

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

🔴 **And that runs into a decision Guy already took, in the opposite direction.** Story 5.14b,
recorded in `deferred-work.md:3283`: the abstention section stays **DESCRIPTIVE** and does **not**
announce its gesture — *"because announcing an absent gesture is a promise"*. AC2 asks for five
announced absent gestures.

⚠️ **The two are reconcilable and the reconciliation is exactly what needs deciding**: AC2's answer to
*"a promise"* is *"labelled"* — the label is what turns a promise into a statement of intent. But
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

🔴 **The consequence for §0a: AC1 needs an arbitration too, and the split's stated rationale was
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

**Written for §0a's SPLIT — AC1 and AC3 only. Rescope on Guy's answer before starting.**

- [ ] **T0 — the THREE arbitrations, BEFORE any code** (§0b, §0c, §0h): the dead-gesture mechanism's
      form and words; whether an all-dead action bar ships at all; and **whether production may read
      `origin` for display, which needs a named `SANCTIONED_READS` entry on story 6.2's precedent**.
      ⚠️ The first two are AC2's and go to 6b.4b under the split; **the third is AC1's and stays
      here** — it was missing from the first draft and the validation found it by running the gate.
      **None may be decided by a developer**
- [ ] **T1 — the provenance and freshness plumbing** (AC1): widen `load_declared_attributes` and
      `load_observation_facts` to return `origin`/`updated_at` and `connector_id`/`observed_at`, and
      carry them into the view structs. ⚠️ **Not through `opencmdb-core`** — D47, and `gap::project`
      has no business holding a timestamp
- [ ] **T2 — the relative-time convention** (AC1): *"il y a 4 min"*, in both locales, as a display
      concern. ⚠️ Re-read story 5.14b's finding first: the real carrier of the clock-freedom guard is
      `chrono`'s `default-features = false`, **not** the test that claims to pin it
- [ ] **T3 — the queue** (AC1): its row vocabulary decided and RECORDED (§0d — only the kinds the
      engine can distinguish today), its query, its ordering, its selection, its empty state
- [ ] **T4 — the two photos** (AC1): the detail pane, each side with its meta-line. ⚠️ Follow the AC
      and the spec on the declared side, **not the mock's unpopulated fixture** (§0e)
- [ ] **T5 — sort by age, OFF by default** (AC3), with a guard that reds if the default flips —
      *the ban is not that age is hidden, it is that age is never brandished*
- [ ] **T6 — leave the `--accent` guard ALONE, and verify it still holds** (§0g). ⚠️ **Its narrowing
      belongs to story 6.4, not to this one** — the validation refuted the first draft's claim that
      the register assigned it here. This story renders beside the identity region: keep the guard
      green, do not widen it, do not narrow it
- [ ] **T7 — LOOK at the screen**, `OPENCMDB_LOCALE=fr`, against a live database with real rows.
      🔴 **And say whether a browser was used.** Three stories running have shipped without one
- [ ] **T8 — the register**, both directions (§0f, §0g). ⚠️ `grep -n "6b.4"` is **provably
      insufficient** — story 6b.3's review found a row its own contexting was quoting that a name-grep
      could not surface, *because the row never named the story*. Search the SUBJECTS too
- [ ] **T9 — prove-to-red**, predictions written FIRST, **and every prescribed row executed**

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

### Debug Log References

### Completion Notes List

### File List

### Change Log

| Date | Change |
|---|---|
| 2026-08-19 | Validated by two fresh-context layers. Fact-check: 61 assertions, 59 confirmed, **2 refuted, both mine** — the `--accent` row belongs to story **6.4**, a different story, and the trap was a number that reads like this one's. Gap-hunt: it BUILT the plumbing and found that **reading `origin` for display reds the `authorship` gate**, which adds a THIRD arbitration and refutes the split's first rationale; that **four of five row kinds are producible today, not two**; and that **M6's clock guard does not red** and never reaches `build_view` at all. |
| 2026-08-19 | Contexted: seven findings, three needing Guy's arbitration. The story carries four deliverables and a SPLIT is recommended; *"labelled per 6b.3"* names a mechanism that does not exist; and applied literally today the action bar would ship five buttons of which five are dead, against a recorded decision that *announcing an absent gesture is a promise*. |
