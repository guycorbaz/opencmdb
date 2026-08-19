# Story 6b.6: Inventory and device record (example)

Status: review

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
- [x] **T1 — implement the state axis** (AC2), on Guy's ruling: five words, each with its EN pair, its meaning,
      and 🔴 **the reconciliation §0c found** — `Écart` and `Conflit` appear on TWO axes and nothing
      records that they mean the same state in both. *One term, one meaning* is the table's own rule
- [x] **T2 — the lint** (AC2, UX-DR64), in **BOTH forms** (§0j) and with its limit WRITTEN: a
      render-side extractor **and** a namespace-derived check resolving both locales. 🔴 Never parse
      `app.yml` — a key can be in the YAML and absent from the binary (6b.5). ⚠️ Four measured
      defeats to close first: an empty population, English resolution, a wrapping `<strong>`, and
      **AC1's own filter bar**
- [x] **T2b — one term, one key** (§0k): the glossary now makes it a REQUIREMENT, not a preference.
      ⚠️ Reuse the key that already renders *Écart* rather than minting a second — UX-DR64's *"glossary
      uniqueness"* breaks on two keys for one word. Registered if a rename is needed: the rendered
      bytes must be identical, and `/triage` is not this story's screen to re-copy
- [x] **T3 — withdraw the drift claim** (§0c) and register instead what is real: the product renders
      `Concordant`/`Non déclaré` **in other words already** (`app.yml:28`, `:301`). ⚠️ Do NOT touch
      6b.4's shipped copy — that screen is not this story's
- [x] **T4 — the inventory's shape** (AC1): the mock's eight filters by type, one row per object with
      its declared state and its last observation
- [x] **T4b — grow the dataset** (§0p): three devices cannot feed seven filters — **at least four
      render an empty table**, a state with no copy, no key and no marker decision. ⚠️ `role_key` is
      not the mock's `type` axis, and **`source` must NOT be invented here** (no connector registry;
      6b.8's)
- [x] **T5 — the device record** (AC1): field by field, *Hébergé ici*, the composite identity, the
      observation history — all example, all marked **per section**
- [x] **T5b — WRITE the per-section marker guard** (§0i): none exists for these screens — measured, a
      marker deleted from `/devices`'s second section leaves **634 green**. ⚠️ **Unify the section
      anchor first** (`screen-section` vs `dashboard-example` vs a third), or the guard is a third
      enumeration
- [x] **T6 — `/devices/{id}` OFF `Screen`** (T0b), on 6b.3's slugs. 🔴 **Covered by no inherited
      guard**: write a route test through `app()` fetching (a) a **non-canonical** slug — not the one
      the nav points at, which a static route shadows — and (b) a slug no device carries. ⚠️ And name
      its auth: an off-`Screen` route is protected by the `is_public` property, tested by nothing
- [x] **T7 — LOOK at both screens in a BROWSER**, `OPENCMDB_LOCALE=fr`. Chrome 151 / Firefox 153 are
      installed. ⚠️ **Rebuild first** — `cargo test` builds the test target, not `target/debug/opencmdb`
- [x] **T8 — the register, BOTH directions.** ⚠️ Five rows name 6b.6 (`deferred-work.md:3689, 3710,
      3825, 3860, 3898`), one struck through as CLOSED
- [x] **T9 — prove-to-red**, predictions FIRST, every row executed, ⚠️ **and the suite run BOTH ways**
      — 6b.5 shipped a red no-database suite behind a green CI
- [x] **T10 — SPLIT before writing** (§0g): `page.rs` 1575 / 2000, and 6b.4 alone added 533. Plan the
      module; do not discover the gate
- [x] **T11 — register, do not fix here** (§0q): a doc under `docs/` is outside `gate_vocabulary`'s
      hardcoded seven-path `DOCS` list (a planted one left all eight gates green); and
      `every_key_carries_both_locales` floors at 47 with a message saying *"48"* while `app.yml`
      carries **108**. ⚠️ And the `set_locale` race (§0m) — **registered with its control, NOT as
      issue #38's cause**

## Prove-to-red — executed

**Fifteen rows after the code review: thirteen reds, one compiler-carried, one GREEN by measurement.** (Thirteen rows at implementation; M2b and M13 were added by the review.) Carriers are named
per row and *"every red assertion-carried"* is **not** claimed. One restore mechanism only — a
scratchpad snapshot, never `git checkout --`, the gesture that destroyed uncommitted work three
times in this project.

| # | Mutation | Result | Carrier |
|---|---|---|---|
| M1 | a state renders a key outside the glossary | **RED ×2** | both glossary guards, each by its own named assertion — the enum-side and the render-side |
| M2 | `rust_i18n::t!("record.hosted")` → `"record.hosted_bogus"` | **RED ×3** | `every_literal_key_…`, `no_i18n_key_reaches_the_screen`, and `the_record_carries_the_four_blocks_ac1_names`. ⚠️ **The KEY had to be named.** The row first read *"a literal key that does not exist"*, and the acceptance auditor could not reproduce ×3 — it tried other keys and got 2 or 4. *The count is a property of the key, not of the mutation class* |
| M2b | `rust_i18n::t!("devices.none")` → bogus — a key **no block assertion reads** | **RED ×1** | 🔑 `every_literal_key_…` **alone**, which is the honest statement: the redundancy M2 shows is an artefact of the key chosen, and for most keys the literal-key guard is the **sole** carrier. M2 without M2b overstates the coverage |
| M3 | a template re-includes the marker beside the dispatch's | 🔑 **COMPILER** (`E0609`) | ⚠️ Once the marker left the templates, `ExampleStrings` stopped carrying `example_badge` — **a double marker became unrepresentable in the type**, which is stronger than the assertion written for it. That assertion is therefore carried by the compiler for these two templates, and by itself only for a struct that still has the field |
| M4 | the record handler ignores its slug | **RED ×2** | `the_record_route_answers_a_non_canonical_slug` **and** `an_unknown_slug_is_answered_without_echoing_it` — the row named only the first, which the acceptance auditor caught. 🔑 Exactly the mutation the gap-hunt measured leaving **636 tests and clippy green** under the shadowed-route design |
| M5 | the route renders with `ScreenQuery::default()` | **RED ×1** | the route-level filter test. ⚠️ **This mutation was the shipped code** until a browser showed it; the pure filter test stayed green through it |
| M6 | the unknown page echoes the slug (and drops the sentence) | **RED ×1** | ⚠️ on *"an unknown slug must be SAID"* — **the FIRST assertion, not the anti-XSS one it was named for.** Story 5.13's assertion-order family, fifth occurrence |
| M6b | the same, sentence KEPT | **RED ×1** | 🔑 the anti-XSS assertion itself (`main.rs:1230`). **M6b is what proves it load-bearing; M6 alone credited the wrong assertion** |
| M7 | a `statepill-*` rule deleted from `app.css` | **RED ×1** | the pill-modifier guard — the one the generic stylesheet guard structurally cannot see |
| M8 | a variant dropped from `ObjectState::ALL` | **RED ×3** | 🔑 the variant-in-`ALL` guard fires, which is what the extension from one file to three bought |
| M9 | a filter offered with no device behind it | **RED ×1** | `no_filter_the_bar_offers_is_empty` |
| M10 | a key-valued field printed unresolved | **RED ×1** | the render-side key guard — the second of the two defects a browser found |
| M11 | a marker deleted from the dashboard's second example section | **RED ×2** | both per-section guards |
| M12 | `values_are_keys` set on a FACTUAL field | ✅ **GREEN, by measurement** | ⚠️ `t!` renders an unknown key verbatim, so resolving `"192.0.2.10"` yields `"192.0.2.10"`. **The flag protects one direction only**, and its doc now says so |
| M13 | `Screen::Device.href()` → a slug no device carries | **RED ×1** | `the_navigations_device_address_names_a_device_that_exists`, added at the code review — before it, the product's own primary link to the record degraded to the *unknown device* page with **no red anywhere** |

⚠️ **Two rows first came back GREEN and both were MY DRIVER, not a weak guard** — a `sed` that
replaced a string with itself, and a `sed` with `\n` in the pattern, which GNU sed does not match
across lines. Caught because `RED=0` contradicted a prediction; had the prediction been *"green"*,
both would have been filed as confirmations. *The mutation driver lies*, fourth epic running — and
the driver now `exit 9`s when the mutation does not apply.

**The suite was run BOTH ways**, story 6b.5's lesson: **0.04 s** without `DATABASE_URL` and
**4.77 s** with one against a live `mariadb:10.11` on port **13316** — the clock is the tell that
the database-backed tests genuinely executed. ⚠️ Port **3306 belongs to another project's
container**, which the context names as a trap and which was avoided rather than discovered.

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

Claude Opus 5 (1M context).

### Completion Notes

⚠️ **THE LIVE COUNT FOR THE PROJECT LIVES HERE**: **634 → 653 tests** (426 bin + 161 core + 66
xtask), eight gates green, clippy clean, 28 fixtures, trap gate still RED at 26/15/11, no migration,
`epics.md` **not** edited. `page.rs` **1575 → 1524** code lines (measured with the `file-size`
gate's own rule, lines before the first `#[cfg(test)]`); the largest file in the workspace is still
`xtask/src/main.rs` at 1908.

🔴 **The figure read *"1482"* until the code review, and the BLIND layer caught it from the diff's
own hunk arithmetic while two layers had the tree.** 1482 was true for about twenty minutes and
went stale the moment `example_marker()` was added; it was never re-measured. *A number written in
flight, inside the story whose own §0 warns about numbers written in flight* — fourth occurrence in
this epic.

⚠️ **`opencmdb-core` is NOT byte-identical, and the claim was narrowed rather than defended.** One
doc line there defined a `Gap` as *"a drift"* — the retired synonym, as the DEFINITION, in the
domain crate — and the promise of non-modification is exactly what would have sheltered it. Story
5.13b's finding, live: **a promise of non-modification protects behaviour and shelters false
sentences.** Narrowed to *no BEHAVIOUR change in `opencmdb-core`*; the sentence is corrected in
place and a test renamed.

🔴 **THE STORY'S CENTRE WAS AC2, AND ITS FIRST DRAFT WAS WRONG ABOUT IT IN THREE WAYS.** The
fact-check layer refuted them and the corrected findings are sharper than the ones they replaced —
the refutation is kept in §0 rather than quietly overwritten. The canonical glossary **exists**, in
three places; the draft established its absence by looking in two directories, which is the exact
defect story 5.13b shipped. What replaced it: **the glossary is binding, and carries not one of
AC2's five words, because its eleven rows are all GESTURES** — and its own preamble is AC2's warning
with *gesture* changed to *state*. Guy added the state axis to both `prd.md` and
`ux-design-specification.md` on 2026-08-19; **a story may not edit a binding artefact, so it is his
act, recorded as one.**

🔑 **The five states ARE Guy's three-case taxonomy of 2026-08-12, exactly and without forcing** —
*software decides* (`concordant`, `gap`, `conflict`) · *operator lifts the doubt* (`ambiguous`) ·
*operator creates the entity* (`undeclared`) — and `gap`/`écart` is the **same binding pair as the
gesture axis**, which settles the story's open fork: one word, one meaning, **one key**, where a
second would have broken UX-DR64's glossary uniqueness.

🔴 **THE GLOSSARY CHECK'S FIRST RUN REDDENED ON THE PRODUCT'S CORE TERM.** The binding table says
`gap` / `écart`; the English UI said **"Drift"**, "No drift", "Open drifts", "one drift closed" —
four sites, on the one term the table calls *"the core object; the product"*, under a preamble that
forbids synonyms in so many words. The French was already right everywhere and the code already said
`gap`: the sole test pinning it asserted `contains("Drift")` under a message reading *"a **Gap** must
be a row"*. **A guard written to satisfy a criterion found a two-story-old defect in its first
second of life.**

🔴 **FOUR MORE DEFECTS WERE FOUND BY LOOKING AT THE PAGE IN A BROWSER, AND NO TEST COULD REACH ANY
OF THEM.** Each passed the whole suite, eight gates and clippy first:

1. two headings resolved `devices.unplaced_*`, keys that do not exist, so the page rendered **its own
   key names**. `rust-i18n` renders an unknown key verbatim; `every_key_carries_both_locales` asks
   whether keys *in* `app.yml` have two languages, and a key absent from the file is not in its
   population at all;
2. the record's *Rôle* row printed `example.role.storage` **as a value**, invisible even to the guard
   just written (the key is data, not a `t!` argument);
3. `/devices?kind=printer` served **all eight devices** — the route's closure took no argument while
   the filter test called the pure builder. Epic 5's dominant class, and story 6b.4's `triage_html`
   was the same shape. ⚠️ **And the fix itself missed**: the parameter was threaded through three
   signatures and the arm still passed `Default::default()`. **Rust does not lint an unused function
   PARAMETER**, so nothing warned;
4. the record shipped **four identical marker banners down one page** — story 6b.4b's finding
   reproduced, invisible because every marker guard asks whether the marker is PRESENT.

🔑 **The fourth resolved into the story's best structural change, by reading story 6b.3's own rule
precisely**: the marker goes on the smallest unit that is **entirely** example — the SECTION on a
`Mixed` screen, the SCREEN on an `Example` one. It is now emitted by the **dispatch** rather than
included by each template, which is strictly stronger: it comes from the same `match` arm as the
body, so a screen declared `Example` cannot render content without it. ⚠️ **That immediately exposed
the record route**, which is off `Screen` and bypasses the dispatch — it served four sections with no
marker at all until the partition test named it. *The skip in `router` buys structural safety on one
axis and costs it on another*, and the marker call there is explicit rather than implied.

🔴 **`/devices/{id}` is the first address `Screen::ALL` structurally cannot represent**, because
`Screen::href` returns a `&'static str` used as a route PATTERN and FETCHED as a URL. Guy chose
**off `Screen`** over a variant that compiles without a warning while shipping a literal
`href="/devices/{id}"` in eleven navigations. ⚠️ The cost is nameable and was therefore paid: no
inherited guard covers the route, so it carries two of its own — a **non-canonical** slug and an
unknown one, both through `app()`. M4 is exactly the mutation the gap-hunt measured leaving 636 tests
and clippy green under the alternative.

⚠️ **The section anchor is unified for real, and the unification had a cost worth recording.**
`_devices_example.html` said `screen-section` while `_dashboard.html` said `dashboard-example`, so
6b.5's per-section guard covered the dashboard and **nothing covered the witness screen** — measured
on the committed tree, a marker deleted from `/devices`'s second section left **all 634 green**.
Widening the dashboard's class then silently unmatched four needles spelled `class="dashboard-example"`:
*an anchor that includes the attribute opening is defeated by the ordinary gesture of adding a class.*

⚠️ **What this story does NOT close, stated plainly**: `Ambigu` is rendered but **unreachable in the
engine** (`Supports`/`Opposes` have no producer until Epic 6's `l2-*` rules); there is **no
containment data of any kind**, so *Hosted here* is example in the strongest sense; `criticality`,
`app` and `owner` are in the mock and were **deliberately not rendered**, being three more nouns with
no glossary row; and the queue's `kindLabel` axis (`Absence`, `Nouveau`) is **still outside the
binding table** — reconciling it changes story 6b.4's shipped copy, which is not this story's.

✅ **The suite was run BOTH ways** (0.04 s without a database, 4.77 s against a live
`mariadb:10.11` on port 13316 — the clock is the tell), and **T7 was a real browser look**, in
French, at 1280 px, on two rebuilt-binary captures. ⚠️ 390 px remains knowingly broken.

### File List

- `crates/opencmdb-bin/src/state_vocabulary.rs` — NEW
- `crates/opencmdb-bin/src/example_screens.rs` — NEW
- `crates/opencmdb-bin/templates/_device_record.html` — NEW
- `crates/opencmdb-bin/templates/_device_unknown.html` — NEW
- `crates/opencmdb-bin/src/example_data.rs` · `screens.rs` · `page.rs` · `main.rs`
- `crates/opencmdb-bin/templates/_devices_example.html` · `_dashboard.html`
- `crates/opencmdb-bin/locales/app.yml` · `assets/app.css`
- `_bmad-output/planning-artifacts/prd.md` · `ux-design-specification.md` — **Guy's arbitration**
- `_bmad-output/implementation-artifacts/deferred-work.md` · `sprint-status.yaml`

### Change Log

| Date | Change |
|---|---|
| 2026-08-19 | **Developed.** 634 → 652 tests, eight gates green, `page.rs` 1575 → 1482 (the split came first, not after the gate said so). 🔴 The glossary check's **first run** reddened on the product's core term: the binding table says `gap`/`écart` and the English UI said *"Drift"* at four sites. 🔴 **Four more defects found by looking at the page in a browser**, each green through the whole suite first — two headings rendering their own key names, a value printed as a key, `?kind=printer` serving all eight devices (and the fix threading a parameter Rust does not lint as unused), and **four identical marker banners** on the record. 🔑 The last resolved into the story's best change: the marker follows the screen's NATURE and is emitted by the DISPATCH, so a screen declared `Example` cannot render content without it — which then exposed the record route, off `Screen` and bypassing the dispatch. **Thirteen mutation rows: eleven red, one compiler-carried, one GREEN by measurement**; ⚠️ two first came back green and **both were my driver, not a weak guard**. |
| 2026-08-19 | **Gap-hunt folded in — it BUILT both designs and neither is free.** 🔴 **The most expensive thing: `Screen::href()` is a route pattern AND a fetchable URL, and `/devices/{id}` cannot be both.** ON `Screen`, the nav ships a literal `href="/devices/{id}"` that every existing guard accepts; OFF it, a static route shadows the param route for exactly the URL the guards probe — after which **the handler may ignore its slug and serve device #1 for `/devices/does-not-exist` with 636 tests and `clippy -D warnings` green**. A parameterised route is the first address `Screen::ALL` structurally cannot represent. 🔴 **I re-measured the marker hole myself on the committed tree**: deleting the marker from `/devices`'s second section leaves **634 green** — the witness screen has no per-section guard, so the draft's M3 prediction was false and the story must WRITE one. 🔴 **AC2's guard is worth nothing in four measured ways** — an empty population, English resolution, a `<strong>` wrap, and **AC1's own filter bar**. 🔴 **The mock renders SEVEN device states, three of them `Écart · <suffix>`**, so AC2's five words are a simplification and exact membership reds on the mock's own copy. Also: `Écart` under two keys breaks UX-DR64's uniqueness; a reproducible `set_locale` race (with its control, **not** claimed as issue #38's cause); reflected XSS on the obvious 404; the state pill's own class idiom blinds 6b.4's stylesheet guard; three devices cannot feed seven filters; and two live defects registered — a doc under `docs/` sits outside `gate_vocabulary`, and `every_key_carries_both_locales` floors at 47 while `app.yml` carries 108. |
| 2026-08-19 | **§0 REWRITTEN after the fact-check refuted its three central claims.** 🔴 The canonical glossary **EXISTS** (`ux-design-specification.md:1332`, `prd.md:985`) — the draft's absence was established over two directories, the very defect this project has a rule against. 🔑 **And the corrected finding is sharper**: the glossary is binding, carries **not one** of AC2's five words, and is a vocabulary of **GESTURES** where AC2 asks about **STATES** — its own preamble is AC2's warning with *gesture* changed to *state*, so the criterion is asking to extend the table to a second axis, which is a PLANNING act and not a story's. 🔴 The **reference mock was never opened** and carries every word the draft reported missing, plus the eight filters and the record's four blocks. 🔴 There is **no vocabulary drift**: the mock has three state axes, 6b.4 shipped one correctly, and the item against it is withdrawn. 🔴 `Ambigu` **has three producers** — the draft confused `identity::cascade::IdentityAbstentionCause` with `gap::AbstentionCause`, story 5.14b's own review finding, in the story about vocabulary. Also corrected: `/devices/{id}` **cannot** be a `Screen` variant (`href` returns `&'static str`), so it is covered by neither guard and this story owes both; `page.rs` is at 1575/2000 with two screens to add. |
| 2026-08-19 | Contexted (first draft, superseded above). 🔴 The story's centre is AC2, and **the canonical glossary it demands does not exist** — the `vocabulary` gate is a retired-term check over four pairs, not a list of live words. **Third AC in this epic pointing at an artefact the project lacks.** 🔴 **Three of the five state words appear nowhere but in the criterion naming them**, and **`Conflit` was already shipped by story 6b.4** — one story before the story meant to check it — leaving the product with **two state vocabularies** that overlap on two words and diverge on five. |
