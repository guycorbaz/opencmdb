# Story 6b.6: Inventory and device record (example)

Status: ready-for-dev

Epic: 6b — *L'interface de la maquette*. **Sixth numbered slot.** ⚠️ **Its scope was NARROWED before
it began**: story 6b.3 pulled `/devices` forward as its witness screen, so what remains here is the
device **RECORD**, the `/devices/{id}` routing debt, and the fidelity pass over the list 6b.3 roughed
in — recorded at 6b.3's contexting so this story's author meets a narrowed scope rather than a
surprise.

## Story

As the operator,
I want the two screens that will hold my devices to exist,
so that the grouping work has somewhere to land.

## Acceptance Criteria

Transcribed from `epics.md:2216-2224`, **unmodified** — divergences are raised in §0 (a story may not
edit an AC; only a retrospective may).

1. **Given** the example dataset, **when** the inventory and the device record render, **then** they
   carry the mock's shape — filters by type, one object per row with its declared state and its last
   observation; and on the record: field by field, *Hosted here*, the composite identity, the
   observation history.
2. **Given** the state vocabulary the mock introduces (*Concordant · Écart · Conflit · Ambigu · Non
   déclaré*), **when** it is written, **then** **every word is checked against the canonical
   glossary**, and any word the glossary does not carry is **registered rather than introduced**.
   ⚠️ *Vocabulary is architecture here: if a state is named after an operation we forbid, someone
   eventually implements the operation.*
3. **And** these two screens are what Epic 6's stories 6.5–6.19 turn real — **the frame is bought
   here, the content is not.**

---

## §0 — What contexting found

⚠️ **This section was REWRITTEN after its fact-check layer refuted its three central claims by
measurement.** The first draft said the canonical glossary does not exist, counted the five state
words over the planning artefacts alone, and told the developer `Ambigu` has no producer. All three
were errors of **PERIMETER**, and the corrected findings are sharper than the ones they replace. 🔑
*The draft's own §0a asserted an absence — and an absence established by looking in two directories is
the exact defect story 5.13b shipped and this project has a rule against.* The refutation is kept in
writing rather than quietly overwritten.

🔑 **This story's centre of gravity is AC2, not the pixels.** AC1 and AC3 are example surfaces on a
mechanism three stories have already built. AC2 asks for a check against a binding document — and
that document exists, is binding, and carries **not one of the five words**.

### §0a. 🔴 THE GLOSSARY EXISTS, IS BINDING, AND HAS NO STATE AXIS AT ALL

Measured — three documents, not zero:

- **`ux-design-specification.md:1332`** — *"### Terminology (**canonical glossary** — one term, one
  translation)"*, an eleven-row binding table (Concept | EN | FR | Meaning) plus a *"Retired, and not
  to be reintroduced"* list. **The heading is AC2's phrase verbatim.**
- **`prd.md:985`** — *"### Canonical Vocabulary (binding — one term, one translation)"*, its ten-row
  mirror; the UX spec says so in its own preamble.
- **`docs/manuals/user-manual/user-manual.tex:155`** — a `\chapter{Glossary}` appendix, seven entries.

🔑 **AND AC2'S WARNING IS THE GLOSSARY'S OWN PREAMBLE WITH ONE WORD CHANGED.** The table opens with
*"Vocabulary is architecture: if a **gesture** is named after an operation we forbid, someone
eventually implements the operation."* AC2 writes *"if a **state** is named after an operation we
forbid…"*. **That one-word edit IS the criterion**: it asks to extend the glossary's rule to a second
axis.

🔴 **And the second axis is empty.** The eleven binding rows are `observed`, `declared`, `gap`,
`reconcile`, `document`, `accept-gap`, `snooze`, `attach`, `exclude`, `triage`, `source` — a
vocabulary of **what the operator DOES**. AC2's five words name **what an object IS**. **The product
has a binding vocabulary for its verbs and none for its nouns**, and this story is the first to need
one.

⚠️ **This changes T1 in kind.** *Writing* a descriptive glossary is a story's business; **extending a
table both the PRD and the UX spec call binding is a planning act** — and `epics.md:319` (UX-DR61,
*"Canonical bilingual glossary — binding EN/FR pairs"*) and `epics.md:324` (UX-DR64, *"forbidden-word
lint over templates + i18n; glossary uniqueness + retired-words denylist"*) show **both the table and
the gate are already outstanding obligations**, not new ideas. **→ T0 goes to Guy on these facts.**

### §0b. 🔴 THE REFERENCE MOCK CARRIES EVERY WORD, AND THE FIRST DRAFT NEVER OPENED IT

The mock is cited by `ux-design-specification.md:7` and lives at
`~/travail/Projets actuels/opencmdb/documents/opencmdb - maquette.html` (496 kB, readable). Measured
inside it: **Concordant 7 · Écart 9 · Conflit 5 · Non déclaré 3 · Ambigu 2**, plus
`const FILTERS = ['Tous','Serveurs','Machines virtuelles','Conteneurs','Appliances','Réseau','Imprimantes','Postes']`,
an *"Hébergé ici"* heading, an *"Identité composite"* block and an *"Historique d'observation"* block.

⚠️ **Everything the draft reported as absent or unspecified is specified in the artefact the criterion
points at.** The draft's word count — *"three of five appear nowhere but in the AC"* — was taken over
`_bmad-output/planning-artifacts/` alone. Over the right perimeter the true figure is different and
duller: **all five are the mock's, and none is the glossary's.**

🔑 **The transferable rule, and it is this project's own**: *an absence is established over the whole
perimeter or not at all* — story 5.13b shipped a reserved UUID prefix by enumerating one directory and
concluding about the tree. **Here the missing perimeter was the design source itself.**

### §0c. 🔴 THERE IS NO VOCABULARY DRIFT — THE MOCK HAS THREE STATE AXES AND THE PRODUCT HAS ONE

The draft claimed story 6b.4 introduced `Conflit` in divergence, leaving two vocabularies. Measured in
the mock, they are **two axes of one document**:

| axis | words | status |
|---|---|---|
| triage queue (`kindLabel`) | `Écart` `Absence` `Conflit` `Nouveau` `Ambigu` | **shipped by 6b.4**, minus `Ambigu` — omitted with its reason |
| inventory (`state`) | `Concordant` `Écart` `Conflit` `Ambigu` `Non déclaré` | **this story's** |
| sources | `Vivante`, `Vivante · portée réduite`, … | unowned |

✅ **Story 6b.4 did exactly the right thing** and the draft's retrospective item against it is
withdrawn. ⚠️ What is REAL and stays: **`Écart` and `Conflit` now appear on two axes, and nothing
records that they mean the same state in both** — which is precisely the *one term, one meaning* rule
the glossary exists to enforce. **That is the finding, and it belongs in T1.**

### §0d. 🔴 `Ambigu` HAS A PRODUCER, AND THE DRAFT CONFUSED TWO TYPES SHARING A WORD

`IdentityAbstentionCause::Ambiguous` exists (`cascade.rs:625`), **three arms of `decide` return it**
(`:507`, `:512`, `:517`, on `architecture.md:971-973`), both locales carry it (`app.yml:99`), and
**story 5.14b renders it on the dashboard today** (`page.rs:415`).

⚠️ **The draft's reason — *"`link_candidate` is a table nothing reads"* — is about
`gap::AbstentionCause`, a different enum over a different population.** 🔑 **That is story 5.14b's
own review finding reproduced — two types sharing a word — inside the story whose subject is
vocabulary.** The true statement is narrower and has a different cause: `Ambiguous` is **unreachable**
because `Verdict::Supports` and `Verdict::Opposes` have no producer until Epic 6's `l2-*` rules —
`cascade.rs:13-14` says so in the code.

Same correction for the other two: `Concordant` is not absent from the product — `app.yml:28` already
ships *"le déclaré et l'observé **concordent**"* — and `Non déclaré` is `app.yml:301`'s *"Rien de
déclaré"*. ⚠️ **So the five words are not new; they are UNRECONCILED with words the product already
renders.** T1's real job.

### §0e. ⚠️ THE RECORD'S FOUR ELEMENTS, and what each rests on

- **field by field** — `declared_attribute` is per `(entity, field)`; the shape exists.
- **"Hosted here"** — **FR29** (`epics.md:85`), one hop, never *Impact*; **ARCH-38** (`:235`) makes
  `hosts` lookup-only. ⚠️ **There is no containment data of any kind** — the schema has five tables
  and not one of them relates two objects. Pure example.
- **the composite identity** — `prd.md:783`, *"composite identity, not raw MAC"*. ⚠️ `ExampleDevice`
  carries `id, name, ipv4, mac, role_key`: **no L2 domain and no second interface**, i.e. nothing that
  makes an identity composite. The dataset grows or the concept is shown over invented values.
- **the observation history** — **FR37** (`:97`), and **story 6.19 owns it** (`backlog`). ⚠️ `6.19`,
  `6.6` and `6b.6` are three different stories.

### §0f. 🔴 `/devices/{id}` CANNOT BE A `Screen` VARIANT, AND THAT OPENS A HOLE THIS STORY CREATES

`Screen::href()` returns **`&'static str`** (`screens.rs:189`) and `router()` iterates `Screen::ALL`
(`:297`). A parameterised path has no static href, so the route this story ships **lives outside
`Screen` by construction** — the draft offered *"a new variant **or** a parameterised path"* as though
both were open, and only one is.

⚠️ **Consequence the draft had backwards**: story 6b.3's `Screen::ALL` source-scanning guard, and the
auth-perimeter test that iterates it, therefore **do NOT cover `/devices/{id}`**. The hole registered
at `deferred-work.md:3825` is struck through as CLOSED *for screens*; this story re-opens it *for
parameterised routes* and owes it a guard of its own.

✅ What does hold: story 6b.3 minted stable slugs (`nas-01`, `switch-core`, `printer-hall`) and emits
`data-device-id` in the markup, **precisely so this story can route on them**
(`deferred-work.md:3898-3902`).

### §0g. ⚠️ TWO SIZE CONSTRAINTS THE DRAFT DID NOT STATE

`page.rs` is at **1575** code lines of the 2000 the `file-size` gate allows, and this story adds two
screens. `screens.rs` is **344**, not the *"~330"* the draft wrote. **Plan for a module, not for
growth** — `CLAUDE.md`'s *"split, not grown"*.

## Dev Notes

### What exists today (read, not assumed — `master` at `301ef1c`)

- **`crates/opencmdb-bin/src/screens.rs`** (**344** code lines) — `Screen` (10 variants), `Nature`
  (**four** since 6b.5), `Screen::ALL` with its source-scanning guard, `router()` excluding
  `Fed | Mixed`.
- **`crates/opencmdb-bin/src/example_data.rs`** — `ExampleDevice { id, name, ipv4, mac, role_key }`
  and `ExampleSighting`. 🔴 **Its copy is KEYS, never literals**, guarded by
  `the_example_copy_is_translated_rather_than_typed`, whose third half checks the key's **namespace**
  — story 6b.4b measured that a real key from the wrong namespace renders a plausible wrong word.
- **`templates/_devices_example.html`** — the list 6b.3 roughed in, two sections, `.grid` tables.
- **`templates/_example_marker.html`** — one partial, one key pair.
- **`locales/app.yml`** — 108 key pairs, `fr` + `en`, guarded.

### The house rules this story will be judged against

- 🔴 **A guard must read the ARTEFACT, not the SOURCE.** Story 6b.4b's four HIGH findings were one
  mistake four times, and 6b.5 hit it from the other side: **a locale key can be in `app.yml` and
  absent from the binary**, and the guard that reads the file cannot tell. **Resolve through `t!()`;
  assert on the rendered HTML.**
- 🔴 **Count per unit, never in aggregate.** 6b.5's section guard compared totals and two markers in
  one section with none in the other left the whole suite green.
- 🔴 **Grep the artefact you are about to believe.** ⚠️ `cargo test` builds the TEST target, **not**
  `target/debug/opencmdb` — rebuild before looking at a running server.
- 🔴 **One restore mechanism per mutation script.** Mixing a scratchpad copy with `git checkout --`
  destroyed uncommitted work three times in this project, most recently in 6b.5.
- 🔴 **A floor CI cannot check is a floor nobody re-reads** — 6b.5 shipped a red no-database suite
  behind a green CI. **Run the suite BOTH ways and record both figures.**
- **Prove-to-red**, predictions FIRST, every prescribed row executed, carriers named per row.
- No file over 2000 code lines (⚠️ `xtask/src/main.rs` at **1908**). Doc comments must be TRUE.

### Testing

- `cargo test --workspace`, `cargo clippy --workspace --locked -- -D warnings`, `cargo fmt --all`,
  `cargo xtask ci` (eight gates; `views-hash` `ℹ STALE` by design).
- ✅ **A browser is available** — `google-chrome` 151 and `firefox`. **T6 is a real browser check.**
  ⚠️ 390 px is knowingly broken (responsive deferred by Guy, 2026-08-18).
- Baseline: **634 tests** (407 bin + 161 core + 66 xtask), eight gates green, `master` at `301ef1c`.

## Tasks / Subtasks

**T0 is a PLANNING act and gates T1–T3. T4–T7 are unaffected and their facts all check out.**

- [ ] **T0 — Guy's ruling** (§0a): the canonical glossary is **binding** and has **no state axis**.
      Extending it is not a story's business. Three shapes:
      **(a) — RECOMMENDED: Guy adds the state axis to the binding table now**, in the PRD and the UX
      spec, and this story implements it. 🔑 *It is the smallest act that lets AC2 be satisfied as
      written*, and UX-DR61/UX-DR64 (`epics.md:319,324`) already owe both the table and its lint.
      **(b)** The story ships the five words and **registers the axis to the retrospective**. ⚠️ Then
      AC2's *"checked against"* is satisfied by nothing, which is what it exists to prevent.
      **(c)** The story ships **only words already in the product** (`écart`, `concordent`, *"Rien de
      déclaré"*) and registers the rest. Honest; ⚠️ diverges visibly from the mock.
- [ ] **T1 — the state axis** (AC2), on Guy's ruling: five words, each with its EN pair, its meaning,
      and 🔴 **the reconciliation §0c found** — `Écart` and `Conflit` appear on TWO axes and nothing
      records that they mean the same state in both. *One term, one meaning* is the table's own rule
- [ ] **T2 — the lint** (AC2, UX-DR64): a state word that reaches the operator and is not in the axis
      reds. 🔴 Resolve through `t!()` and assert on the RENDERED page — **never parse `app.yml`**: a
      key can be in the YAML and absent from the binary, measured in story 6b.5
- [ ] **T3 — withdraw the drift claim** (§0c) and register instead what is real: the product renders
      `Concordant`/`Non déclaré` **in other words already** (`app.yml:28`, `:301`). ⚠️ Do NOT touch
      6b.4's shipped copy — that screen is not this story's
- [ ] **T4 — the inventory's shape** (AC1): the mock's eight filters by type, one row per object with
      its declared state and its last observation
- [ ] **T5 — the device record** (AC1): field by field, *Hébergé ici*, the composite identity, the
      observation history — all example, all marked **per section** (6b.5's rule: a totals guard is
      worthless)
- [ ] **T6 — `/devices/{id}`** (§0f), on 6b.3's slugs. 🔴 **It cannot be a `Screen` variant**, so it
      is covered by neither the `Screen::ALL` guard nor the auth-perimeter test — **give it both**
- [ ] **T7 — LOOK at both screens in a BROWSER**, `OPENCMDB_LOCALE=fr`. Chrome 151 / Firefox 153 are
      installed. ⚠️ **Rebuild first** — `cargo test` builds the test target, not `target/debug/opencmdb`
- [ ] **T8 — the register, BOTH directions.** ⚠️ Five rows name 6b.6 (`deferred-work.md:3689, 3710,
      3825, 3860, 3898`), one struck through as CLOSED
- [ ] **T9 — prove-to-red**, predictions FIRST, every row executed, ⚠️ **and the suite run BOTH ways**
      — 6b.5 shipped a red no-database suite behind a green CI
- [ ] **T10 — watch the ceiling** (§0g): `page.rs` 1575 / 2000 and two screens to add. Split

## Prove-to-red — deliberately short

| # | Mutation | Prediction |
|---|---|---|
| M1 | render a state word absent from the glossary | T2's guard reds. ⚠️ **If it stays green because it reads the source rather than the render, that is the finding** |
| M2 | a state word rendered as a literal instead of a key | the example-copy guard reds — its **shape** half (`example_data.rs:146`), not the namespace one. ⚠️ **Predict GREEN for a word on a new struct**: the guard iterates two functions only |
| M3 | drop the marker from ONE example section of the record | the per-section guard reds. ⚠️ **Predict per-SECTION, not totals** — 6b.5's totals form was measured worthless |
| M4 | give `/devices/{id}` a slug no device carries | predict the answer: a 404, or the list? **Whichever it is, assert it** |
| M5 | a `Screen` variant wired everywhere but omitted from `Screen::ALL` | 6b.3's guard reds |
| M6 | `/devices/{id}` removed from the auth perimeter | 🔴 **predict GREEN before T6's guard exists** — the perimeter test iterates `Screen::ALL` and this route is not in it (§0f) |

## References

- `_bmad-output/planning-artifacts/epics.md:2216-2224` — the acceptance criteria, verbatim
- `_bmad-output/planning-artifacts/epics.md:85` — FR29, *"Hosted here"*, one hop, never *Impact* · `:97` — FR37, the observation history · `:235` — ARCH-38
- `_bmad-output/planning-artifacts/prd.md:783` — *"composite identity, not raw MAC"*
- 🔑 `_bmad-output/planning-artifacts/ux-design-specification.md:1332` — **the canonical glossary**, eleven binding rows, all of them GESTURES · `prd.md:985` — its mirror
- `_bmad-output/planning-artifacts/epics.md:319` — UX-DR61, the binding glossary · `:324` — UX-DR64, the forbidden-word lint this story's T2 is
- 🔑 `~/travail/Projets actuels/opencmdb/documents/opencmdb - maquette.html` — **the reference mock**, cited at `ux-design-specification.md:7`; it carries all five state words, the eight filters, and the record's four blocks
- `xtask/src/main.rs:426` — the `vocabulary` gate: Volet B is a retired-term co-presence check over four pairs, Volet A (`:419`) a denylist over `crates/`. Neither is a glossary
- `crates/opencmdb-core/src/identity/cascade.rs:13,507,512,517` — `Ambiguous` HAS three producers, and is unreachable only because `Supports`/`Opposes` have none
- `crates/opencmdb-bin/src/example_data.rs` — the stable slugs story 6b.3 minted for `/devices/{id}`
- `_bmad-output/implementation-artifacts/6b-3-…md` — the narrowing that gave this story its scope
- `_bmad-output/implementation-artifacts/6b-4b-…md`, `6b-5-…md` — assert on the artefact; count per unit

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log

| Date | Change |
|---|---|
| 2026-08-19 | **§0 REWRITTEN after the fact-check refuted its three central claims.** 🔴 The canonical glossary **EXISTS** (`ux-design-specification.md:1332`, `prd.md:985`) — the draft's absence was established over two directories, the very defect this project has a rule against. 🔑 **And the corrected finding is sharper**: the glossary is binding, carries **not one** of AC2's five words, and is a vocabulary of **GESTURES** where AC2 asks about **STATES** — its own preamble is AC2's warning with *gesture* changed to *state*, so the criterion is asking to extend the table to a second axis, which is a PLANNING act and not a story's. 🔴 The **reference mock was never opened** and carries every word the draft reported missing, plus the eight filters and the record's four blocks. 🔴 There is **no vocabulary drift**: the mock has three state axes, 6b.4 shipped one correctly, and the item against it is withdrawn. 🔴 `Ambigu` **has three producers** — the draft confused `identity::cascade::IdentityAbstentionCause` with `gap::AbstentionCause`, story 5.14b's own review finding, in the story about vocabulary. Also corrected: `/devices/{id}` **cannot** be a `Screen` variant (`href` returns `&'static str`), so it is covered by neither guard and this story owes both; `page.rs` is at 1575/2000 with two screens to add. |
| 2026-08-19 | Contexted (first draft, superseded above). 🔴 The story's centre is AC2, and **the canonical glossary it demands does not exist** — the `vocabulary` gate is a retired-term check over four pairs, not a list of live words. **Third AC in this epic pointing at an artefact the project lacks.** 🔴 **Three of the five state words appear nowhere but in the criterion naming them**, and **`Conflit` was already shipped by story 6b.4** — one story before the story meant to check it — leaving the product with **two state vocabularies** that overlap on two words and diverge on five. |
