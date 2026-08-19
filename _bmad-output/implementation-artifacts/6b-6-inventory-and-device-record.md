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

### §0h. 🔴 THE MOST EXPENSIVE THING — `Screen::href()` IS A ROUTE PATTERN *AND* A FETCHABLE URL, AND `/devices/{id}` CANNOT BE BOTH

The gap-hunt layer BUILT both placements. Neither is free, and **both leave the record route carried
by nothing**:

- **ON `Screen`** (`href() = "/devices/{id}"`) — **compiles cleanly, zero warnings**; only two count
  assertions red. Give it a real `Path<String>` handler that 404s on an unknown slug and the
  partition test reds `404 vs 200`, because it fetches `href()` **literally**. ⚠️ And before the
  lookup existed the same test **passed**: axum matched the param route with `id = "{id}"`. Worse, the
  navigation then ships `<a href="/devices/{id}">` on all eleven screens and
  `every_entry_is_offered_with_its_own_address` is **green** — its withheld-list is `href="#"`, `""`,
  `hidden`, `aria-disabled`, `display:none`, and a `{id}` placeholder is none of them.
- **OFF `Screen`** (`Screen::Device` → a real slug, `/devices/{id}` registered additionally) — axum
  0.8 accepts both, **no `Overlapping method route`**, and the **static route shadows the param route
  for exactly the URL every guard probes**. Measured on that green baseline: deleting the
  `/devices/{id}` registration leaves **636 green** (caught only by clippy's dead-code, which is
  **outside `cargo xtask ci`**); and making the handler **ignore its slug** — device #1 for every URL,
  `/devices/does-not-exist` included — leaves **636 green and `clippy -D warnings` clean**.

🔑 **A parameterised route is the first address in this product that `Screen::ALL` structurally cannot
represent**, and the whole inherited apparatus iterates `Screen::ALL`. Epic 5's dominant class through
a new door: *the guards are correct about what they test, and what they test is not the route.*

✅ Measured, so M4 can predict rather than guess: `/devices/` → **404**, `/devices/a/b` → **404**,
`/devices/%7Bid%7D` → **200** (axum percent-decodes before `Path`). **An unmatched slug 404s by
axum's own default; a 200 is only reachable by writing a handler that ignores its input.** And an
off-`Screen` route **is** auth-protected by the `is_public` property — carried by no test naming it,
story 6b.2's review finding reopened for the eleventh address.

**→ Guy arbitrates the placement. Neither is free.**

### §0i. 🔴 THE WITNESS SCREEN HAS NO PER-SECTION MARKER GUARD — MEASURED ON THE COMMITTED TREE

I ran this myself rather than believe it. Delete `{% include "_example_marker.html" %}` from the
**second** of `/devices`'s two example sections (`_devices_example.html:39`), change nothing else:

```
407 bin + 161 core + 66 xtask = 634 green
```

6b.5's `every_example_section_carries_its_own_marker` is anchored on `class="dashboard-example"` and
reads `rendered_dashboard()` alone. **The witness screen 6b.3 shipped is covered by the route-table
partition's `contains` only — which 6b.5 already measured cannot tell one marker from two.**

⚠️ **So M3's prediction in the first draft was FALSE: there is no such guard for these screens; the
story must WRITE one.** And there is no single anchor to write it over — `_devices_example.html` uses
`class="screen-section"`, `_dashboard.html` uses `dashboard-real`/`dashboard-example`, and a
four-block record would be a **third** section vocabulary. **Unify the anchor first, or the guard is a
third enumeration.**

### §0j. 🔴 AC2'S GUARD: FOUR MEASURED WAYS IT IS WORTH NOTHING

The gap-hunt BUILT it — drive `app()` over `Screen::ALL`, extract every `class="state"` text, check
membership.

1. **The population is EMPTY**: `found=0`. The only screen rendering state words today is `/triage`,
   skipped without `DATABASE_URL`; and with one, an empty store renders *"You are up to date."*
   ⚠️ **The guard is green and measures nothing, held up only by a `found >= N` floor — the exact
   shape 6b.5 shipped stale.** 🔑 *The example inventory is what gives AC2's guard a non-empty
   population on a machine with no database* — that is a design point, and the story must state it.
2. **It resolves in ENGLISH.** The test process never calls `set_locale`, so `rust_i18n` gives `en`:
   the probe reported `"Undeclared"`, not *"Non déclaré"*. A French-only glossary reds on every word.
3. **Wrapping the word defeats it**: `<span class="state"><strong>…</strong></span>` → `found=0`,
   green. The extractor stops at the first `<`.
4. 🔴 **AC1's own deliverable defeats it**: a filter bar naming three unknown state words with **no
   class** → `found=3, offenders=[]`, **full suite green**. *The filters-by-type bar is the control
   most likely to name state words, and the guard cannot see it.*

✅ The complementary form (derive from `state.*` / `triage.kind.*` namespaces, resolve both locales)
works and found §0k at once — but is blind to any literal. **Neither form alone is complete. AC2 needs
both, plus a written limit** on story 5.12's precedent: *a tripwire against the author who marks their
words, never a barrier.*

### §0k. 🔴 `Écart` WILL EXIST UNDER TWO KEYS, AND A GLOSSARY KEYED BY WORD CANNOT HOLD IT

This is §0c's operational cost, and it is a fork the story must take:

- **reuse `triage.kind.ecart`** — one word, one key ✅, but 6b.4's copy then serves 6b.6's screen and
  6b.4's screen is not this story's to change;
- **mint `state.ecart`** — two keys render one French word, and **UX-DR64's *"glossary uniqueness"*
  breaks on it**.

### §0l. 🔴 THE MOCK RENDERS SEVEN STATE STRINGS FOR DEVICES, NOT FIVE — I RECOUNTED IT

Over the mock's 18 device rows: `Ambigu` · `Concordant` · `Conflit` · `Non déclaré` ·
**`Écart · 1 champ`** · **`Écart · 2 champs`** · **`Écart · présence`** (plus two `Vivante · *` on the
sources screen, a third axis).

⚠️ **AC2's five words are a SIMPLIFICATION of what the mock actually renders.** An exact-membership
guard **reds on the mock's own copy**; a prefix match accepts anything after the separator. **Is
`Écart · 2 champs` the word `Écart`? That decision is AC2, not a rendering detail.**

### §0m. 🔴 A REPRODUCIBLE LOCALE RACE ALREADY EXISTS, AND T2'S GUARD IS THE SHAPE IT BREAKS

`page.rs:2886` (6b.4) calls `rust_i18n::set_locale("fr")` **process-wide**, while `page.rs:1848`'s own
comment says `set_locale` *"is NOT used, and must not be"*. With its control:

| run | result |
|---|---|
| that test + `build_view_empty_when_no_declared_entity`, `--test-threads=2`, ×60 | **18 failures** |
| same pair, `--test-threads=1`, ×30 — **CONTROL** | **0** |

⚠️ **This is NOT claimed as issue #38's cause** — `relative_time` landed 2026-08-19, #38 was recorded
at `d47631b` on 2026-08-02, and `CLAUDE.md` already records a `set_locale` hypothesis *refused* for
#38. *A cause needs a check, and this check settles a different question.* What it is: a **new**
reproducible race, with the check that settles it. 🔑 **It matters here because T2's guard is a render
assertion on translated words — precisely the shape this race breaks — and 6b.6 multiplies that
surface.**

### §0n. ⚠️ THE OBVIOUS 404 IS REFLECTED XSS, and 6b.4's clearance does not cover it

`format!` into `Html` (not through Askama):
`GET /devices/%3Cscript%3Ealert(1)%3C%2Fscript%3E` → `200`, body `<script>alert(1)</script>`.
6b.4's review verified *"no XSS"* **over templates**; the record's 404 is the first place a raw path
segment is echoed. **A prescribed mutation, not a note.**

### §0o. ⚠️ THE STATE PILL'S OWN IDIOM BLINDS THE STYLESHEET GUARD, with a control

`class="statepill statepill-{{ device.id }}"` (both undefined) → **GREEN**;
`class="statepill"` (undefined) — **control** → **RED**. `page.rs:3256` skips any attribute containing
`{`. Registered by 6b.4b as a general limit; **here it is live on this story's central widget**,
because a state pill wants a base class plus a per-state modifier.

### §0p. ⚠️ THE EXAMPLE DATASET IS THREE DEVICES AND THE MOCK'S SHAPE NEEDS NINE FIELDS

`ExampleDevice` is `{id, name, ipv4, mac, role_key}` × **3 rows**. The mock's row is
`{id, code, type, name, ip, observed, source, seen, state}` — **and carries no `mac`**, so the current
template's MAC column is itself a divergence. The record adds three nested collections
(`hosted`, `identity`, `history`).

- 🔴 **`source` is a connector NAME and the product has NO connector registry** — 6b.4 registered that
  the mock's *"UniFi"* was its fixture's invention (→ 6b.8). Rendering it here **invents it again, one
  story early.**
- ⚠️ `app`, `owner`, `criticality` are three more nouns with no producer **and not in the glossary
  either** — AC2's check applies to them the moment they render.
- 🔴 **Seven filters over three devices means at least four filters render an EMPTY table**, and the
  empty-filter state has no copy, no key and no marker decision. **Grow the dataset or the filter is a
  demo of nothing.**
- ⚠️ `role_key` (storage/network/peripheral) is **not** the mock's `type` axis: a second field, not a
  rename.

### §0q. ⚠️ TWO LIVE DEFECTS FOUND IN PASSING, both registered rather than fixed here

- A new `docs/state-vocabulary.md` sits **outside every gate**: `gate_vocabulary`'s `DOCS`
  (`xtask/src/main.rs:396`) is a hardcoded seven-path list. One was added and **all eight gates
  reported green**. *If T1 writes a document, add its path in the same commit.*
- `every_key_carries_both_locales` asserts `checked >= 47` on a message reading *"48 entries"*.
  **`app.yml` carries 108.** The floor is stale by 61 and its message states a false count — in the
  guard this story leans on while adding ~20 keys.

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

✅ **THE THREE ARBITRATIONS ARE TAKEN (Guy, 2026-08-19), each recorded with the option refused.**

- **T0 — the glossary gains a STATE axis, and Guy added it.** Five binding rows now sit under the
  eleven gesture rows in **both** `prd.md` and `ux-design-specification.md`. 🔑 **The justification was
  already in the document**: its own reason for retiring `ignore` is that *"every other verb describes
  the OBJECT'S STATE"* — and it carried no state noun at all. 🔑 **And the five ARE Guy's three-case
  taxonomy of 2026-08-12**, exactly: *software decides* (`concordant`, `gap`, `conflict`) · *operator
  lifts the doubt* (`ambiguous`) · *operator creates the entity* (`undeclared`). ⚠️ **`gap`/`écart` is
  the SAME pair as the gesture axis, not a second term.** Refused: deferring to the retrospective
  (AC2's check would rest on nothing) and shipping only words already in the product (visible
  divergence from the mock).
- **T0b — the record route sits OFF `Screen`.** Refused: a `Screen` variant, which compiles without a
  warning while shipping a literal `href="/devices/{id}"` in eleven navigations that every existing
  guard accepts. ⚠️ **The chosen cost is nameable and must therefore be paid**: a static route shadows
  the param route for the exact URL the guards probe, so **the story owes the route a test of its
  own** — fetch a NON-canonical slug and an unknown slug through `app()`.
- **T0c — a suffix is a rendering detail, not a term.** `Écart · 2 champs` **is** the word `écart`;
  the check matches before the separator. Refused: seven glossary rows (the table would describe
  rendering) and banning the suffix (the operator loses *how many fields diverge*).

- [x] **T0 / T0b / T0c — arbitrated above.** The glossary edit is **DONE** (both documents, eight
      gates re-run green); the other two bind the tasks below
- [ ] **T1 — implement the state axis** (AC2), on Guy's ruling: five words, each with its EN pair, its meaning,
      and 🔴 **the reconciliation §0c found** — `Écart` and `Conflit` appear on TWO axes and nothing
      records that they mean the same state in both. *One term, one meaning* is the table's own rule
- [ ] **T2 — the lint** (AC2, UX-DR64), in **BOTH forms** (§0j) and with its limit WRITTEN: a
      render-side extractor **and** a namespace-derived check resolving both locales. 🔴 Never parse
      `app.yml` — a key can be in the YAML and absent from the binary (6b.5). ⚠️ Four measured
      defeats to close first: an empty population, English resolution, a wrapping `<strong>`, and
      **AC1's own filter bar**
- [ ] **T2b — one term, one key** (§0k): the glossary now makes it a REQUIREMENT, not a preference.
      ⚠️ Reuse the key that already renders *Écart* rather than minting a second — UX-DR64's *"glossary
      uniqueness"* breaks on two keys for one word. Registered if a rename is needed: the rendered
      bytes must be identical, and `/triage` is not this story's screen to re-copy
- [ ] **T3 — withdraw the drift claim** (§0c) and register instead what is real: the product renders
      `Concordant`/`Non déclaré` **in other words already** (`app.yml:28`, `:301`). ⚠️ Do NOT touch
      6b.4's shipped copy — that screen is not this story's
- [ ] **T4 — the inventory's shape** (AC1): the mock's eight filters by type, one row per object with
      its declared state and its last observation
- [ ] **T4b — grow the dataset** (§0p): three devices cannot feed seven filters — **at least four
      render an empty table**, a state with no copy, no key and no marker decision. ⚠️ `role_key` is
      not the mock's `type` axis, and **`source` must NOT be invented here** (no connector registry;
      6b.8's)
- [ ] **T5 — the device record** (AC1): field by field, *Hébergé ici*, the composite identity, the
      observation history — all example, all marked **per section**
- [ ] **T5b — WRITE the per-section marker guard** (§0i): none exists for these screens — measured, a
      marker deleted from `/devices`'s second section leaves **634 green**. ⚠️ **Unify the section
      anchor first** (`screen-section` vs `dashboard-example` vs a third), or the guard is a third
      enumeration
- [ ] **T6 — `/devices/{id}` OFF `Screen`** (T0b), on 6b.3's slugs. 🔴 **Covered by no inherited
      guard**: write a route test through `app()` fetching (a) a **non-canonical** slug — not the one
      the nav points at, which a static route shadows — and (b) a slug no device carries. ⚠️ And name
      its auth: an off-`Screen` route is protected by the `is_public` property, tested by nothing
- [ ] **T7 — LOOK at both screens in a BROWSER**, `OPENCMDB_LOCALE=fr`. Chrome 151 / Firefox 153 are
      installed. ⚠️ **Rebuild first** — `cargo test` builds the test target, not `target/debug/opencmdb`
- [ ] **T8 — the register, BOTH directions.** ⚠️ Five rows name 6b.6 (`deferred-work.md:3689, 3710,
      3825, 3860, 3898`), one struck through as CLOSED
- [ ] **T9 — prove-to-red**, predictions FIRST, every row executed, ⚠️ **and the suite run BOTH ways**
      — 6b.5 shipped a red no-database suite behind a green CI
- [ ] **T10 — SPLIT before writing** (§0g): `page.rs` 1575 / 2000, and 6b.4 alone added 533. Plan the
      module; do not discover the gate
- [ ] **T11 — register, do not fix here** (§0q): a doc under `docs/` is outside `gate_vocabulary`'s
      hardcoded seven-path `DOCS` list (a planted one left all eight gates green); and
      `every_key_carries_both_locales` floors at 47 with a message saying *"48"* while `app.yml`
      carries **108**. ⚠️ And the `set_locale` race (§0m) — **registered with its control, NOT as
      issue #38's cause**

## Prove-to-red

| # | Mutation | Prediction |
|---|---|---|
| M1 | render a state word absent from the glossary | T2's guard reds. ⚠️ **Predict GREEN for the four §0j shapes until each is closed** — empty population, `<strong>` wrap, unclassed filter bar, English resolution |
| M2 | a state word as a literal instead of a key | the example-copy guard reds on its **shape** half (`example_data.rs:146`), not the namespace one. ⚠️ **Predict GREEN on a NEW struct** — the guard iterates two functions only |
| M3 | drop the marker from ONE example section of the record | 🔴 **Predict GREEN before T5b exists** — measured on the committed tree, 634 green with `/devices`'s second marker deleted. After T5b, red |
| M4 | the record handler ignores its slug (device #1 for every URL) | 🔴 **Predict GREEN with clippy clean** — the gap-hunt measured 636 green. Red only after T6's own test fetches a **non-canonical** slug through `app()` |
| M4b | `/devices/does-not-exist` | **404** by axum's default. ⚠️ `/devices/%7Bid%7D` → **200** (percent-decoded before `Path`) |
| M5 | a `Screen` variant omitted from `Screen::ALL` | 6b.3's guard reds |
| M6 | `/devices/{id}` dropped from the auth perimeter | 🔴 **Predict GREEN before T6's guard** — the perimeter test iterates `Screen::ALL`, which cannot represent this route (§0h) |
| M7 | echo the raw path segment into the 404 body | 🔴 **XSS, and the suite stays green** — 6b.4's clearance was over templates (§0n) |
| M8 | `class="statepill statepill-{{ … }}"`, both undefined | **GREEN**; the control `class="statepill"` reds (§0o) |
| M9 | delete the `/devices/{id}` registration entirely | **GREEN** in `cargo xtask ci`; caught by clippy's dead-code alone, **which is outside the gates** |

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
| 2026-08-19 | **Gap-hunt folded in — it BUILT both designs and neither is free.** 🔴 **The most expensive thing: `Screen::href()` is a route pattern AND a fetchable URL, and `/devices/{id}` cannot be both.** ON `Screen`, the nav ships a literal `href="/devices/{id}"` that every existing guard accepts; OFF it, a static route shadows the param route for exactly the URL the guards probe — after which **the handler may ignore its slug and serve device #1 for `/devices/does-not-exist` with 636 tests and `clippy -D warnings` green**. A parameterised route is the first address `Screen::ALL` structurally cannot represent. 🔴 **I re-measured the marker hole myself on the committed tree**: deleting the marker from `/devices`'s second section leaves **634 green** — the witness screen has no per-section guard, so the draft's M3 prediction was false and the story must WRITE one. 🔴 **AC2's guard is worth nothing in four measured ways** — an empty population, English resolution, a `<strong>` wrap, and **AC1's own filter bar**. 🔴 **The mock renders SEVEN device states, three of them `Écart · <suffix>`**, so AC2's five words are a simplification and exact membership reds on the mock's own copy. Also: `Écart` under two keys breaks UX-DR64's uniqueness; a reproducible `set_locale` race (with its control, **not** claimed as issue #38's cause); reflected XSS on the obvious 404; the state pill's own class idiom blinds 6b.4's stylesheet guard; three devices cannot feed seven filters; and two live defects registered — a doc under `docs/` sits outside `gate_vocabulary`, and `every_key_carries_both_locales` floors at 47 while `app.yml` carries 108. |
| 2026-08-19 | **§0 REWRITTEN after the fact-check refuted its three central claims.** 🔴 The canonical glossary **EXISTS** (`ux-design-specification.md:1332`, `prd.md:985`) — the draft's absence was established over two directories, the very defect this project has a rule against. 🔑 **And the corrected finding is sharper**: the glossary is binding, carries **not one** of AC2's five words, and is a vocabulary of **GESTURES** where AC2 asks about **STATES** — its own preamble is AC2's warning with *gesture* changed to *state*, so the criterion is asking to extend the table to a second axis, which is a PLANNING act and not a story's. 🔴 The **reference mock was never opened** and carries every word the draft reported missing, plus the eight filters and the record's four blocks. 🔴 There is **no vocabulary drift**: the mock has three state axes, 6b.4 shipped one correctly, and the item against it is withdrawn. 🔴 `Ambigu` **has three producers** — the draft confused `identity::cascade::IdentityAbstentionCause` with `gap::AbstentionCause`, story 5.14b's own review finding, in the story about vocabulary. Also corrected: `/devices/{id}` **cannot** be a `Screen` variant (`href` returns `&'static str`), so it is covered by neither guard and this story owes both; `page.rs` is at 1575/2000 with two screens to add. |
| 2026-08-19 | Contexted (first draft, superseded above). 🔴 The story's centre is AC2, and **the canonical glossary it demands does not exist** — the `vocabulary` gate is a retired-term check over four pairs, not a list of live words. **Third AC in this epic pointing at an artefact the project lacks.** 🔴 **Three of the five state words appear nowhere but in the criterion naming them**, and **`Conflit` was already shipped by story 6b.4** — one story before the story meant to check it — leaving the product with **two state vocabularies** that overlap on two words and diverge on five. |
