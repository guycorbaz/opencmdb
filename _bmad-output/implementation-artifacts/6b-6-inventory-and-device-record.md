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

🔑 **This story's centre of gravity is AC2, not the pixels.** AC1 and AC3 are example surfaces on a
mechanism three stories have already built. AC2 asks for a check against a document, and the document
does not exist.

### §0a. 🔴 THERE IS NO CANONICAL GLOSSARY — measured, and this is the THIRD time in this epic

AC2 says *"every word is checked against the **canonical glossary**"*. Measured:

- **No glossary document exists.** No file under `_bmad-output/planning-artifacts/` or `docs/` is one;
  the word appears only inside prose in `epics.md`, the PRD, the UX spec and the architecture.
- ⚠️ **The `vocabulary` gate is NOT one.** `cargo xtask ci`'s gate (`xtask/src/main.rs:426`) is a
  **retired-term co-presence check** over exactly four pairs — `pending_accept → pending_commit`,
  `reverting → failed/in_queue`, `accept-as-declared → accept-gap/document`, `ignore → exclude`
  (D45/D65). It forbids four dead words. It carries no list of live ones and cannot answer *"is
  `Concordant` a word this product uses?"*

🔑 **This is the same shape as story 6b.3's §0b** (*"the marker is specified by no document this
project has"*) **and story 6b.4b's** (*"labelled per 6b.3 names a mechanism that does not exist"*).
**Three ACs in one epic pointing at an artefact the project lacks** — a pattern for the retrospective,
not a coincidence.

**→ PUT TO GUY. Three shapes:**

- **(a) — RECOMMENDED: this story writes a DESCRIPTIVE glossary** under `docs/`, listing every state
  word the product **renders** with the code that produces it and the locale key that carries it.
  🔑 *It is a record of what exists, not a planning decision, which is exactly what "checked against"
  needs and what a story may write.* ⚠️ **And it is worth a gate on story 6b.4b's lesson — assert on
  the RENDERED string, not on the source** — so a word that reaches the operator and is not in the
  glossary reds.
- **(b) Register the absence and use only words already shipped**, deferring the glossary. Cheapest;
  ⚠️ leaves AC2 satisfiable by no check at all, which is the shape *"registered rather than
  introduced"* exists to prevent.
- **(c) A retrospective writes it.** Correct if the glossary is to be PRESCRIPTIVE; ⚠️ but then this
  story ships state words with nothing to check them against, and §0c shows what that costs.

### §0b. 🔴 THREE OF THE FIVE WORDS APPEAR NOWHERE BUT IN THE CRITERION THAT NAMES THEM

Measured across `_bmad-output/planning-artifacts/*.md`:

| Word | Occurrences outside AC2's own sentence |
|---|---|
| **Concordant** | **0** |
| **Conflit** | **0** |
| **Non déclaré** | **0** |
| *Écart* | 13 |
| *Ambigu* | a handful — most apparent hits are the English *ambiguous*/*ambiguity* |

⚠️ So AC2's *"the state vocabulary **the mock introduces**"* is exact, and its instruction —
*"registered rather than introduced"* — applies to **three of the five words**, not to an edge case.

### §0c. 🔴 AND `Conflit` WAS ALREADY INTRODUCED — ONE STORY BEFORE THE STORY MEANT TO CHECK IT

Story 6b.4 shipped four French state words for the triage queue (`locales/app.yml:243-254`):
**`Écart` · `Absence` · `Conflit` · `Nouveau`**.

🔴 **`Conflit` is one of the three the glossary does not carry, and it reached the operator's screen
before AC2's check existed.** And the two vocabularies are not the same set:

| | triage queue (shipped, 6b.4) | inventory (prescribed, this AC) |
|---|---|---|
| shared | **Écart**, **Conflit** | **Écart**, **Conflit** |
| only there | `Absence`, `Nouveau` | `Concordant`, `Ambigu`, `Non déclaré` |

🔑 **The product therefore has TWO state vocabularies, overlapping on two words and diverging on
five** — and *"vocabulary is architecture"* is the AC's own warning, biting one story early. ⚠️ **This
story cannot resolve it alone**: reconciling them changes a shipped screen's copy, which is 6b.4's,
and naming the union is a planning act. **Register it, and put the reconciliation to the
retrospective.**

### §0d. ⚠️ `Ambigu` HAS NO PRODUCER, and the same omission was already taken once

`Ambigu` needs FR16's ranked candidates; `link_candidate` is a table nothing reads into the view
layer. Story 6b.4 **omitted it from the queue for exactly this reason** and registered it to Epic 6.
⚠️ Here it is example content, so it MAY be rendered — but the row it labels must not imply the engine
can produce it. **Say which of the five the engine can produce today** (measured: `Écart` and
`Conflit` come from `gap::reconcile`; the other three do not exist as engine outputs at all).

### §0e. ⚠️ THE RECORD'S FOUR ELEMENTS, and what each rests on

AC1 names four things for the device record. Measured:

- **field by field** — `declared_attribute` is per `(entity, field)`, so the shape exists.
- **"Hosted here"** — **FR29** (`epics.md:85`), *"one containment hop, no traversal, never called
  Impact"*, and **ARCH-38** confirms `hosts` is lookup-only. ⚠️ **There is no containment data of any
  kind** — no applications, no host relation, nothing. Pure example.
- **the composite identity** — the PRD's own term (`prd.md:783`: *"composite identity, not raw MAC"*),
  which the identity engine holds as its L1 key. ⚠️ **The example dataset carries no composite
  identity today** — `ExampleDevice` has a MAC and nothing else — so either the dataset grows one or
  the record shows the concept over invented values.
- **the observation history** — **FR37**, and **story 6.19 owns it** (`6-19-observation-history-per-device`,
  `backlog`). Example here.

### §0f. ✅ THE ROUTING DEBT IS READY TO CLOSE, and story 6b.3 prepared it deliberately

`/device` addresses no particular device — story 6b.2's review called it *"the sharpest case"* of the
nine empty screens. 🔑 **Story 6b.3 minted a STABLE slug on every example device precisely for this**
(`example_data.rs`: *"story 6b.6 routes on it… change a slug here and you break a URL that story will
ship"*), and emitted it as `data-device-id` in the markup. **The id exists; the route does not.**
⚠️ A new route means a new `Screen` variant or a parameterised path — and `Screen::ALL`'s
source-scanning guard (story 6b.3) now covers the first, so the hole that named 6b.6 as its owner is
**already closed**.

---

## Dev Notes

### What exists today (read, not assumed — `master` at `301ef1c`)

- **`crates/opencmdb-bin/src/screens.rs`** (~330 code lines) — `Screen` (10 variants), `Nature`
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

**Written for §0a option (a). Rescope on Guy's answer before starting.**

- [ ] **T0 — Guy's ruling on §0a**, the glossary. ⚠️ Not a developer's call
- [ ] **T1 — the glossary** (AC2): every state word the product RENDERS, with its producer and its
      locale key. ⚠️ **Descriptive, not prescriptive** — a record of what exists
- [ ] **T2 — a guard over the RENDERED word** (AC2): a state word that reaches the operator and is
      not in the glossary reds. 🔴 Resolve through `t!()`, never parse the file
- [ ] **T3 — register the two vocabularies** (§0c) and put their reconciliation to the retrospective.
      ⚠️ Do NOT change 6b.4's shipped copy: that screen is not this story's
- [ ] **T4 — the inventory's shape** (AC1): filters by type, one row per object with its declared
      state and its last observation
- [ ] **T5 — the device record** (AC1): field by field, *Hosted here*, the composite identity, the
      observation history — all example, all marked, **per section** (§0d, and 6b.5's per-unit rule)
- [ ] **T6 — `/devices/{id}`** (§0f), on the slugs story 6b.3 minted for it
- [ ] **T7 — LOOK at both screens in a BROWSER**, `OPENCMDB_LOCALE=fr`. **Rebuild first**
- [ ] **T8 — the register, BOTH directions.** ⚠️ A name-grep is provably insufficient, and `6.6`,
      `6b.6` and `6b.4b` are different stories
- [ ] **T9 — prove-to-red**, predictions FIRST, every row executed, **and the suite run both ways**

## Prove-to-red — deliberately short

| # | Mutation | Prediction |
|---|---|---|
| M1 | render a state word absent from the glossary | T2's guard reds. ⚠️ **If it stays green because it reads the source rather than the render, that is the finding** |
| M2 | a state word rendered as a literal instead of a key | the example-copy guard reds — its namespace half, story 6b.4b's |
| M3 | drop the marker from ONE example section of the record | the per-section guard reds. ⚠️ **Predict per-SECTION, not totals** — 6b.5's totals form was measured worthless |
| M4 | give `/devices/{id}` a slug no device carries | predict the answer: a 404, or the list? **Whichever it is, assert it** |
| M5 | a `Screen` variant wired everywhere but omitted from `Screen::ALL` | 6b.3's source-scanning guard reds — the hole this story was once going to own |

## References

- `_bmad-output/planning-artifacts/epics.md:2216-2224` — the acceptance criteria, verbatim
- `_bmad-output/planning-artifacts/epics.md:85` — FR29, *"Hosted here"*, one hop, never *Impact* · `:97` — FR37, the observation history · `:235` — ARCH-38
- `_bmad-output/planning-artifacts/prd.md:783` — *"composite identity, not raw MAC"*
- `xtask/src/main.rs:426` — the `vocabulary` gate, which is a RETIRED-term check and not a glossary
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
| 2026-08-19 | Contexted. 🔴 The story's centre is AC2, and **the canonical glossary it demands does not exist** — the `vocabulary` gate is a retired-term check over four pairs, not a list of live words. **Third AC in this epic pointing at an artefact the project lacks.** 🔴 **Three of the five state words appear nowhere but in the criterion naming them**, and **`Conflit` was already shipped by story 6b.4** — one story before the story meant to check it — leaving the product with **two state vocabularies** that overlap on two words and diverge on five. |
