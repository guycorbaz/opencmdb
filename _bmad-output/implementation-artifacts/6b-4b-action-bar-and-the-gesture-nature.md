# Story 6b.4b: The action bar, and the gesture nature it needs

Status: ready-for-dev

Epic: 6b — *L'interface de la maquette*. **INSERTED at story 6b.4's validation** (Guy, 2026-08-19),
taking Epic 6b from twelve stories to thirteen. It carries **AC2 of `epics.md`'s story 6b.4**, which
was split off because its mechanism is specified by no document this project has and governs every
screen after this one, not only triage.

🔑 **This story starts DESIGNED.** Its form and its words were arbitrated in advance, in story 6b.4's
§0b, precisely so that its author meets a decision rather than a question. What contexting adds is
one finding that the arbitrated shape **cannot be built as written today** — measured, not supposed —
and the options that follow from it.

## Story

As the operator,
I want the gestures the product does not have yet to be visible and labelled rather than absent,
so that a screen with nothing to click tells me why instead of looking broken.

## Acceptance Criteria

Transcribed from `epics.md:2184-2186`, **unmodified** — divergences are raised in §0 rather than
edited in (a story may not edit an AC; only a retrospective may).

1. **Given** the gestures, **when** the action bar renders, **then** **only the gestures that exist
   are live**, and the others are shown and labelled per 6b.3. ⚠️ *And that includes « Merger » until
   story 6.4 lands* — field-level documenting is **FR13(b), Epic 7's**; what 6.4 ships is **FR13(a)**
   on the abstention line. *A developer meeting a dead primary button on the product's signature
   screen will want to fix it: this paragraph is why it is not a bug.*

---

## §0 — What contexting found

### §0a. 🔴 THE ARBITRATED SHAPE DOES NOT COMPILE TODAY, and the reason is that nothing is live

Guy's arbitration (6b.4 §0b) is `enum Gesture { Live { route }, Planned { owner } }`, chosen so that
*"a button that looks live and calls nothing"* is unrepresentable.

**Measured at contexting** — the enum wired so that it is genuinely used, with only `Live` left
unconstructed:

```
cargo clippy --workspace --locked -- -D warnings
error: variant `Live` is never constructed
```

✅ **And the obvious escape hatch is closed, measured by the validation rather than assumed**: a
`#[cfg(test)]` test constructing `Live` does **not** silence it, because `cargo clippy --workspace`
without `--all-targets` never compiles `cfg(test)` code — story 6b.3's measurement, reproduced here
by writing the test.

🔴 **And that is not an accident of the probe — it is the state of the product.** Not one of the
mock's five action-bar gestures exists:

| The mock's button | What the product has |
|---|---|
| `Merger` / `Résoudre` | **Nothing.** Field-level documenting is FR13(b), Epic 7's; `Résoudre` needs FR16's ranked candidates, Epic 6's |
| `Accepter l'écart` · `Mettre en veille` · `Rattacher` · `Exclure` | **Nothing** — no `snoozed`, `excluded` or `gap_accepted` column in any migration |

⚠️ **And `POST /document-all` is not one of them.** It exists, but it adopts a **whole entity**, is
behind `OPENCMDB_DOCUMENT_ENABLED` which defaults to `false`, and is called by no template. It is not
the mock's field-level `Merger` wearing a different name; it is a different gesture at a different
granularity.

**→ PUT TO GUY. Four options, with what each costs:**

- **(d) — RECOMMENDED, but on a NARROWER promise than this section first wrote.**
  `Gesture { label_key, route: Option<&'static str> }` compiles clean, needs no `#[allow]`, and the
  render branch cannot disagree with the route because there is only one field to read.
  🔴 **But *"the arbitrated PROPERTY survives; only its spelling changes"* is FALSE, and the
  validation refuted it by BUILDING it.** M1 was run for real — `route: Some("/does-not-exist")` on
  the primary gesture — and **nothing reds**: 398 tests, clippy and all eight gates green, and the
  rendered control is a genuine `<a class="btn-gesture live" href="/does-not-exist">Merger</a>`,
  **visually indistinguishable in a screenshot** from what a wired gesture will look like. *Not merely
  representable — unguarded.* §0a's own prediction (*"if nothing reds, the type is not forcing what
  this claims"*) resolves in the negative.
  🔑 **And the enum was never much better, which is the finding that decides this.** Rebuilt and
  measured: constructing **one** `Live { route: "/bogus" }` beside four `Planned`s makes the
  *"never constructed"* error **disappear**. Clippy asks *"was this variant instantiated anywhere,
  with any value"* — never whether the value means anything. So the enum's forcing power was a
  **one-shot, whole-crate check that goes dark the day story 6.4 constructs its first `Live`**,
  correctly wired or not.
  ⚠️ **So no option ships the property as literally stated, and the honest closing sentence is that
  this is a LABELLING AND TYPING DISCIPLINE, not a compiler-enforced guarantee** (story 5.12's
  precedent: narrow the promise in writing rather than imply one). What (d) really buys: the rendered
  state cannot contradict the field. What it really costs against (a): **the one compiler-forced
  moment of attention available today**, while nothing is live.
  🔑 **And the real closure is registered rather than implied**: a route that must be a member of a
  CLOSED SET — not a `&'static str` — makes a nonexistent route unrepresentable. It cannot be built
  today because the set would be empty, which is the same wall from the other side. **Owner: story
  6.4**, which wires the first live gesture.
- **(a) ship `Planned` alone now, `Live` arrives with story 6.4.** Honest, no `#[allow]`. ⚠️ **Its
  one advantage over (d) is real and was found by measurement**: it keeps a compiler-forced moment —
  the day someone adds `Live`, the lint makes them notice that nothing was live before. Its cost is
  that the type carries one variant, so *"declaring is forced"* is true of nothing until 6.4.
- **(b) `#[allow(dead_code)]` on the variant.** 🔴 **Refuse-worthy**: that lint is the only carrier of
  `Screen::ALL`'s exhaustiveness outside `cargo xtask ci`, and story 6b.3 built a test because it is silenceable — ⚠️ at its
  CODE REVIEW, after a one-line probe measured the lint-carried version defeated, not from the start. Silencing it deliberately, here, teaches the next reader
  that it may be silenced.
- **(c) wire `/document-all` to the bar so something is `Live`.** 🔴 **Refuse**: it would put a
  whole-entity adoption under a label the mock uses for a field-level merge, and it is off by
  default — *a button that does something other than what it says is worse than a button that does
  nothing and says so.*

### §0b. ✅ THE FORM AND THE WORDS ARE ALREADY ARBITRATED — do not reopen them

From story 6b.4's §0b, Guy, 2026-08-19, reproduced here so this story is self-contained:

- **The words**, distinct from story 6b.3's because they address a different population: **fr** —
  badge *« À venir »*, sentence *« Ce geste n'est pas encore construit. »*; **en** — *"Not yet"*,
  *"This gesture is not built yet."*
- 🔴 **`<button disabled>` is REFUSED on NFR25**: a disabled button **leaves the tab order and
  disappears from a screen reader**, so the blind operator is not even told the gesture exists.
  `aria-disabled="true"` on a non-activatable control keeps the announcement.
- ⚠️ **The owning story's number stays OUT of the surface.** *"Arrives in 6.4"* is not information for
  the operator and turns the label into a **calendar, therefore a promise** — what story 5.14b
  refused. The owner lives in the type, never on the screen.
- A separate *"À venir"* group heading was refused: honest, but it diverges from the mock's form,
  which 6b.2's *"the mock prevails"* forbids without an explicit decision.

### §0c. ✅ THE *WHETHER* IS NOT OPEN EITHER — Guy's premise decided it

`epics.md:2092`, decision **(2)** of 2026-08-13, under a heading reading *"this epic's premises and
**not open questions**"*: *"the same rule applies INSIDE an implemented screen — **the four gestures
Epic 7 owns are visible and labelled**."* The count lands exactly on the mock's four secondary
buttons; the fifth, `Merger`, is covered by the AC's own sentence.

⚠️ **Story 6b.4's contexting put this to Guy as an open question and it was not one** — recorded so
this story does not repeat it. The apparent tension with story 5.14b (*"announcing an absent gesture
is a promise"*) is resolved by the premise, which is later and more specific: 5.14b governs a
**descriptive section**, the premise governs **gestures inside an implemented screen**, and the label
is what turns a promise into a statement of intent.

### §0d. 🔴 A PLANNED BUTTON MUST NOT BE AMBER, and the guard is watching

`--accent-document` is reserved for the documenting gesture (story 6b.1's arbitration), and
`page.rs`'s `ac4_the_amber_is_reserved_for_the_documenting_gesture` asserts the count of legitimate
uses is **ZERO**, with the message *"story 6.4 adds the first legitimate use"*.

🔑 **A `Planned` Merger button must therefore be NEUTRAL, not amber** — and this is a reason, not a
convenience: **the amber means *"this is the gesture"*, and it is not one yet.** A dead button in the
gesture's colour would spend the reservation's meaning before the gesture arrives.

⚠️ **So this story must leave the guard at zero**, and register row (e)'s narrowing stays **story
6.4's** — story 6b.4's contexting mistook that row for its own (*"story 6.4"* and *"story 6b.4"* are
different stories) and the validation refuted it. Do not repeat that either.

### §0e. ⚠️ NO KEYBOARD LETTERS, by a decision that predates this story

The mock's detail pane carries `⏎ merger · ⌫ exclure`. Epic 6b's measured constraint **(6)**
(`epics.md:2106`) says the spec assigns **no keyboard letters** by decision — *"a letter chosen in
isolation is a letter whose neighbourhood nobody tested"* — and that **the mock's `⏎` and `⌫` must
not be read as a specification**. ⚠️ Rendering a keyboard hint for a gesture that does not exist would
be the promise §0b just refused, twice over. **No key bindings, and no hint line.**

### §0f. ⚠️ WHAT THIS STORY DOES NOT CHANGE

- **The bar is a display.** No route, no handler, no migration, no write. If this story adds a POST,
  it has left its scope.
- **`/document-all` stays uncalled by any template.** Wiring it is story 6.4's (FR13(a)).
- ⚠️ **The operator still cannot DO anything after this story.** It converts *"a screen with nothing
  to click"* into *"a screen that says why there is nothing to click"* — an improvement in honesty,
  not in reach. The phrase *a better-lit dead end* is worth carrying rather than
  blurring — ⚠️ **and it lives in `docs/project-context.md`, not in story 6b.4's own review**, which
  is where the validation found it; the substance is confirmed by the code (`_triage.html` has no
  `<form>`, no button and no POST, and `document-all` is called by no template). **6.4 is the story that ends it.**

### §0g. 🔴 THE STYLESHEET GUARD HAS TWO HOLES, AND THE NATURAL WAY TO WRITE THIS MARKUP FALLS IN ONE

Story 6b.4's `every_class_a_template_names_is_defined_in_the_stylesheet` — the guard written because
6b.3 shipped `.screen-section` defined nowhere — was measured by the validation to have two gaps, and
**the second is this story's trap specifically**:

1. ⚠️ It scans **raw template bytes, Askama comments included**. Writing the pattern `class="…"` inside
   a `{#- … -#}` comment to explain the markup **reds the guard**. The validation tripped it doing
   exactly that.
2. 🔴 **It silently skips any `class="…"` containing `{`** (`if literal.contains('{') { continue; }`).
   And the obvious way to write a live/planned toggle is one element with
   `class="btn-gesture{% if gesture.route.is_some() %} live{% else %} planned{% endif %}"` — **that
   whole literal is skipped, so `live` and `planned` go unchecked for stylesheet coverage.** *Story
   6b.3's defect, silently reintroduced by the natural way to write this story's own markup.*

⚠️ **Both are pre-existing holes in a story-6b.4 guard, not defects of this one** — but this is the
story that walks into the second. The validation avoided it by using **two full elements with two
static class literals**. Either do the same, or widen the guard; do not write the conditional class
and assume it is covered.

### §0h. ⚠️ THREE MORE THINGS THE TASK LIST DOES NOT WARN ABOUT

- 🔴 **T3's *"following the row's kind"* has a trap with a precedent.** The only machine-readable
  signal on `DetailPane`/`QueueRow` at render time is `kind: String` — **already translated**. Branch
  on that and you reproduce story 6b.3's `role_key: "example.badge"` defect: a real, resolving, wrong
  value that every check passes. **Branch on the cause enum, where the row is built**, as the
  validation did (`Conflit` → *Résoudre*, the other three → *Merger*, which is what the mock does and
  what 6b.4's §0f recorded).
- ⚠️ **M2 and M3 presuppose guards that DO NOT EXIST.** The table says *"the NFR25 guard reds"* and
  *"the guard reds"*; neither is on `master`. **Writing them is uncredited scope inside T1/T2** and is
  now named as a deliverable. (M4 and M5 do use existing guards, and the validation confirmed both
  red as predicted.)
- 🔴 **`Option<&'static str>` says nothing about the HTTP METHOD, and that is a landmine for 6.4.**
  These are actions, not navigations: story 6.4 will need a POST, presumably `hx-post` on story 6.2's
  CSRF-Origin precedent. ⚠️ **And `aria-disabled` does NOT stop htmx from firing** — htmx respects the
  native `disabled` attribute or `hx-disabled-elt`, nothing else. Harmless today (no `hx-*` on a
  planned gesture) and **registered**, because the shape recommended here will have to grow before it
  can host a live gesture safely.

### §0i. ✅ TWO THINGS THE VALIDATION MEASURED THAT NO ONE HAD

- 🔴 **A BROWSER EXISTS, and four consecutive stories have deferred the visual check on a premise
  nobody tested.** Measured: `google-chrome --version` → **151.0.7922.169**, and `firefox` is
  installed too. The validation rendered the bar in headless Chrome, took screenshots at **1200 px and
  390 px**, and reports the row reads as *deliberately not built* rather than broken, wrapping 3+2 and
  then stacking, with no overlap and no unstyled fallback. 🔑 *A limit believed is a limit unmeasured*
  — nobody had run `command -v`. **T6 is therefore a real browser check, not a text read.**
- ✅ **Guy's NFR25 arbitration is confirmed by a real DOM rather than by prose.** With
  `aria-disabled="true"`: `tabIndex 0`, focus succeeds, the click event fires. With the native
  `disabled`: focus is **refused** and **no event fires**. *The distinction the arbitration turns on
  is measured, not asserted* — and it is why the label must be `aria-disabled`.

---

## Dev Notes

### What exists today (read, not assumed — `master` at `6605a3c`)

- **`crates/opencmdb-bin/templates/_triage.html`** — the two panes. The action bar belongs in the
  **detail pane** (`<aside class="photos">`), under the two photos, which is where the mock puts it.
- **`crates/opencmdb-bin/src/page.rs`** (1332 code lines) — `TriageView`, `QueueRow`, `DetailPane`,
  `build_triage`, `Strings`, and the guards. A gesture list belongs on `DetailPane` or beside it.
- **`crates/opencmdb-bin/templates/_example_marker.html`, `_not_built_yet.html`** — story 6b.3's two
  partials, both classifying CONTENT. ⚠️ **Neither is reusable here**: a dead control is a different
  axis, `Screen::Triage` is `Fed` and *"owes NO marker"*.
- **`locales/app.yml`** — 90 key pairs, `fr` + `en`, guarded by `every_key_carries_both_locales`.
- **`assets/app.css`** — the triage screen's rules. ⚠️ **Every class a template names must be defined**
  (`every_class_a_template_names_is_defined_in_the_stylesheet`, story 6b.4), and that guard walks
  `templates/` **recursively** since 6b.4's review.

### The house rules this story will be judged against

- **Prove-to-red**, predictions written FIRST, **and every prescribed row executed**. 🔴 And read
  6b.4's two lessons before writing the table: **a mutation that changes two things attributes its
  red to whichever you were looking at**, and **a fixture of one cannot witness a property of many**.
- 🔴 **A guard placed where the defect cannot occur reads as coverage and is none.** Reading it cannot
  find that; only running the mutation can.
- 🔴 **A status code is not a look.** Story 6b.4 found four defects by reading the rendered screen and
  a fifth by renaming a helper, none reachable by any test. ⚠️ **And no browser has been available for
  four consecutive stories** — this story ships *five controls in a row*, which is layout. Say plainly
  what was and was not verified by eye.
- Doc comments must be TRUE; prefer the weaker true sentence. `#![deny(missing_docs)]` is on.
- No source file over 2000 code lines. ⚠️ `xtask/src/main.rs` is at **1908** — split, do not grow it.

### Testing

- `cargo test --workspace`, `cargo clippy --workspace --locked -- -D warnings`, `cargo fmt --all`,
  `cargo xtask ci` (eight gates; `views-hash` is `ℹ STALE` by design and must NOT be regenerated).
- ⚠️ **`DATABASE_URL` unset means database-backed tests return early and pass in silence** — 0.07 s
  against ~5 s. This story is mostly pure, but its route-level guard is not: **run it both ways and
  record both figures.**
- Baseline: **625 tests** (398 bin + 161 core + 66 xtask), eight gates green, `master` at `6605a3c`.

## Tasks / Subtasks

**Written for §0a's option (d). Rescope on Guy's answer before starting.**

- [ ] **T0 — Guy's ruling on §0a**, the only open question. ⚠️ Everything else is already decided
      (§0b, §0c) and must not be reopened
- [ ] **T1 — the gesture type** (AC1): every gesture declares whether it has a route, so a control
      that looks live and calls nothing cannot be written. ⚠️ **No `#[allow(dead_code)]`**
- [ ] **T2 — one partial, one key pair** (AC1, §0b): `aria-disabled="true"` and **never** `disabled`;
      the words as arbitrated; **no story number in the surface**. ⚠️ **And WRITE the two guards M2
      and M3 name** — neither exists on `master`, and the mutation table read as though they did
      (§0h)
- [ ] **T3 — the five gestures in the detail pane**, in the mock's order, with `Merger`/`Résoudre`
      following the row's kind as the mock does. 🔴 **Branch on the CAUSE, never on the translated
      `kind` string** — that is story 6b.3's wrong-namespace defect waiting (§0h). ⚠️ And use two
      static class literals rather than one conditional one, or the stylesheet guard skips them
      silently (§0g)
- [ ] **T4 — NEUTRAL, not amber** (§0d): leave `ac4_the_amber_is_reserved_for_the_documenting_gesture`
      at **zero** and do not narrow it — that is story 6.4's
- [ ] **T5 — no keyboard hint, no key binding** (§0e), and the absence recorded as a decision
- [ ] **T6 — LOOK at the screen IN A BROWSER**, `OPENCMDB_LOCALE=fr`, against a live database.
      🔴 **`google-chrome` 151 and `firefox` are installed** (§0i) — the *"no browser available"*
      sentence four stories carried was never measured. Screenshot at desktop AND mobile width; a row
      of five controls is layout, and layout is what a text dump cannot show
- [ ] **T7 — the register, BOTH directions.** ⚠️ `grep -n "6b.4b"` is **provably insufficient**: story
      6b.3's review found a row its own contexting was quoting that a name-grep could not surface,
      and story 6b.4's contexting **misread `story 6.4` as itself**. Search the SUBJECTS, and check
      the story NUMBER twice
- [ ] **T8 — prove-to-red**, predictions FIRST, every prescribed row executed

## Prove-to-red — deliberately short

🔑 Five rows, sized so that every one WILL be played.

| # | Mutation | Prediction |
|---|---|---|
| M1 | give a gesture a route it does not have | 🔴 **ALREADY MEASURED BY THE VALIDATION: nothing reds** — 398 tests, clippy and eight gates green, and the control renders as a genuine live link, indistinguishable in a screenshot. The prediction resolved in the negative, so **§0a's promise is narrowed rather than the type strengthened**. Re-run it to confirm on the shipped shape, and record it as a KNOWN GREEN with its reason — not as a guard |
| M2 | render a planned gesture with `disabled` instead of `aria-disabled` | reds — ⚠️ **but only once T2 writes the guard**, which does not exist today. The validation wrote one (every occurrence of `disabled` must be part of `aria-disabled`) and measured it green then red |
| M3 | put the owning story's number in the rendered label | reds — ⚠️ **also a guard T2 must write**; the validation's form scans the new locale values for a digit |
| M4 | give a planned gesture `--accent-document` | ✅ **MEASURED by the validation on the EXISTING guard**: `left: 1, right: 0`. No new guard needed (§0d) |
| M5 | delete the `fr` half of the new key pair | ✅ **MEASURED by the validation on the EXISTING guard** — `every_key_carries_both_locales` catches it |

## References

- `_bmad-output/planning-artifacts/epics.md:2184-2186` — the acceptance criterion, verbatim
- `_bmad-output/planning-artifacts/epics.md:2092` — Guy's premise (2): unbuilt gestures are *visible and labelled* (§0c)
- `_bmad-output/planning-artifacts/epics.md:2106` — constraint (6): no keyboard letters (§0e)
- `_bmad-output/planning-artifacts/ux-design-specification.md:1205` — the action row's weights
- `_bmad-output/implementation-artifacts/6b-4-triage-screen-on-the-real-gap.md` — §0b (this story's arbitrated design), §0c (the premise), and its **four** defects found by looking plus **a fifth found by renaming a helper** — ⚠️ that story draws the distinction itself and this reference collapsed it until the validation caught it
- `_bmad-output/implementation-artifacts/deferred-work.md` — the 6b.4b rows, and row (e) which belongs to story **6.4** and not to this one
- `crates/opencmdb-bin/templates/_triage.html`, `src/page.rs` — where the bar goes

## Validation record — two fresh-context layers, 2026-08-19

**Mandatory here** (Guy, Epic 4 retrospective). Both on a different model, each in its own worktree.

**Layer 1, fact-check — 36 assertions verified, 34 confirmed, 1 REFUTED and 1 true-but-weaker, both
mine.** ✅ It reproduced the headline measurement live **and closed the escape hatch I had only
inferred**: a `#[cfg(test)]` test constructing `Live` does not silence the lint, because
`clippy --workspace` without `--all-targets` never compiles `cfg(test)` code — verified by writing the
test, not by citing story 6b.3. Refuted: my References line said *"the five defects only looking
found"* where story 6b.4 records **four by looking and a fifth by renaming**, a distinction that story
insists on. Weaker: *"a better-lit dead end"* lives in `docs/project-context.md`, not in 6b.4's own
review — the substance holds, the attribution did not.

**Layer 2, gap-hunt — it BUILT both options** against a live `mariadb:10.11.11`, rendered the bar, and
took screenshots. 🔴 **Its central finding refutes my recommendation's justification by running it**:
under option (d), `route: Some("/does-not-exist")` reds **nothing** and the control renders as a
genuine live link. *"The arbitrated property survives, only its spelling changes"* is FALSE — the
property is not merely representable, it is unguarded. 🔑 **And the enum was never much better**:
constructing one `Live` with a bogus route makes the *"never constructed"* error vanish, so its
forcing power was a one-shot whole-crate check that goes dark the day 6.4 lands. **No option ships the
property as stated**, which is why §0a now narrows the promise instead of claiming one.

🔑 **And it measured two things nobody had.** `google-chrome` **151** and `firefox` are installed —
**the *"no browser available"* sentence four consecutive stories carried was never checked** — so the
bar was rendered and read at 1200 px and 390 px, and it reads as *deliberately not built* rather than
broken. And Guy's NFR25 arbitration is confirmed by a real DOM: `aria-disabled` keeps focus and fires
its click, the native `disabled` refuses both.

⚠️ It also found **two holes in story 6b.4's stylesheet guard** — it scans Askama comments, and it
**silently skips any `class="…"` containing `{`**, which is exactly how one would naturally write this
story's live/planned toggle — plus three traps the task list did not warn about (§0g, §0h).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log

| Date | Change |
|---|---|
| 2026-08-19 | Validated by two fresh-context layers. Fact-check: 36 assertions, 34 confirmed, 2 flagged (both mine) — and it CLOSED the escape hatch I had only inferred, by writing the test. Gap-hunt: it BUILT both options and 🔴 **refuted my recommendation's justification** — under option (d) a route that goes nowhere reds nothing and renders as a genuine live link, and **the enum was never much better** (one bogus `Live` silences its lint). **No option ships the property as stated**, so §0a narrows the promise. 🔴 **And a BROWSER EXISTS** — Chrome 151 and Firefox — so four stories deferred the visual check on a premise nobody measured. |
| 2026-08-19 | Contexted. 🔴 ONE open question and it is a measurement: the arbitrated `enum Gesture { Live, Planned }` **does not compile today** — `error: variant 'Live' is never constructed` under `clippy -D warnings`, because not one of the mock's five gestures exists and `/document-all` is not one of them. Four options put to Guy, with a struct carrying an `Option` route recommended: the arbitrated PROPERTY survives, only its spelling changes. Everything else was already decided — the form, the words, and the *whether*, which Guy's own premise (2) settled on 2026-08-13. |
