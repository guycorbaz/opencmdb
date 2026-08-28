# Story 6.4c: The interface on a telephone

Status: **DEFERRED to GitHub issue #131** — Guy, 2026-08-28: *the mobile interface is not needed at
the start*. Contexted 2026-08-28 against a booted binary and Chrome **152**, then VALIDATED the same
day by two fresh-context layers, and deferred **after** that validation rather than instead of it.

🔑 **The story file is kept, and `sprint-status.yaml` reads `backlog`, which is a STRETCH of that
word and is said here rather than glossed**: the definition is *"story only exists in epic file"*,
and this one exists in no epic file and carries several hundred lines of measured context. What
`backlog` means for 6.4c is *not scheduled; the work is issue #131; the measurements are in §0 and
§0g*.

⚠️ **The criteria below are the DRAFT the validation refuted in part.** §0g says which, and it is
deliberately NOT merged into them: a criterion and the measurement that broke it are worth more side
by side than a corrected criterion nobody can see was corrected. **Anyone re-opening this work starts
from §0g, not from the AC list.**

⚠️ **What deferring costs**: Epic 6b's retrospective sequenced this story *before* the engine work
because *"6.4 is the LAST story that touches the interface before 6.5-6.19 go into schema and the L2
cascade — fifteen engine stories would separate the decision from its application, and an interface
story written that late re-contexts entirely."* Taking it after Epic 6 accepts exactly that.

⚠️ **This story is in NO epic file.** It was created by Epic 6b's retrospective (decision 2,
2026-08-24) and sequenced by Guy the same day; `epics.md` defines stories 6.1–6.19 and not this one,
exactly as for 6.4b. There are therefore **no epic-level acceptance criteria to inherit** — the
criteria below are derived from **NFR24**, **UX-DR56**, the UX specification's *Responsive Strategy*,
and from what §0 MEASURED on the running product. §0 is that derivation shown rather than asserted.

## Story

As the operator woken at 11 p.m. by an alert,
I want the product to be usable on the telephone in my hand,
So that a deep link lands me on the object and I can read it, not fight it.

## Acceptance Criteria

*(Sources: `prd.md:1427` — NFR24, **"breakpoints 360 / 768 / 1280 px; no horizontal overflow; touch
targets ≥ 44 px"**; `epics.md:165`, which adds **"snapshot-verified; deep-linked object views usable
on a phone"**; `epics.md:313` — UX-DR56; `ux-design-specification.md:1562-1580` — *Responsive
Strategy* and *Breakpoint Strategy*; `epic-6b-retro-2026-08-24.md` §5 decision 2, which takes NFR24
and the `@media` gap **together** and refuses the alternative of amending NFR24 to match the mock.)*

**AC1 — No horizontal overflow at 360, 768 or 1280 px, on every route the navigation offers.**
🔴 Measured today: **nine of the ten screens overflow at 360 px** and **one still overflows at 768 px**
(§0b). The property is `documentElement.scrollWidth <= clientWidth`, measured in a browser on the
COMPUTED page — ⚠️ **a stylesheet-reading guard cannot establish it**, story 6.4's *a source guard
cannot see a cascade* and story 6b.11's amended AC5.

**AC2 — Wide content scrolls in its OWN container, and the page does not.** The UX spec's tablet
clause (`:1568`) is what makes AC1 satisfiable without deleting data: a table or a grid too wide for
the viewport gets a scrolling wrapper. ⚠️ **The wrapper must be reachable by keyboard** — a
`overflow-x: auto` region that no control can focus is a region a keyboard operator cannot scroll.

**AC3 — Touch targets are ≥ 44 px HIGH at the mobile and tablet breakpoints.** 🔴 Measured today at
360 px: **21 of 33 distinct control shapes are under 44 px**, the worst being `a.nav-entry` at
**183 × 34 px**, which is every navigation entry on every screen (§0c). ⚠️ **The desktop figure is an
ARBITRATION, not a criterion** — see §0e, arbitration 2: raising density everywhere changes the look
the mock defines, and the register has been waiting on that question since 2026-08-22.

**AC4 — The navigation stops eating the screen.** At 360 px the rail is a fixed **208 px** — **58 % of
the viewport**, leaving 152 px of content — and it is 208 px at every width because
`grid-template-columns: 208px minmax(0, 1fr)` carries no breakpoint. ⚠️ **What replaces it is
arbitration 1 and is NOT decided here**: the UX spec's bottom bar (`:841`, `:1570`) prescribes *"a
permanent search magnifier"*, and **the product has no search** — ten screens, no search route — so
the literal reading ships an eleventh well-lit dead end into the epic that spent itself counting them.

**AC5 — The measurement is a BROWSER GATE with the 0 / 1 / 2 contract**, on `a11y/axe-gate.mjs` and
`a11y/kbd-probe.mjs`'s precedent, deriving its routes from the rendered navigation and never from a
list. 🔴 **Neither existing gate sets a viewport at all** (§0d): both have run at Puppeteer's default
and **no accessibility instrument in this project has ever seen a phone**. ⚠️ A skipped check answers
**2**, never a green — story 6b.11's arbitration 1.

**AC6 — Relative units, and the fixed-pixel layouts named.** The UX spec (`:1577`) requires
`rem`/`%`/`vw` throughout and *"no fixed-pixel layouts"*. ⚠️ **This AC is met by NAMING what stays
fixed and why, not by a sweep**: a 1 px border and a 10 px grid cell are not layout. What must go is
every fixed dimension that decides how much room the content gets.

**AC7 — The deep-linked object view is usable on a phone**, which is UX-DR56's own words and the one
clause NFR24 spells out as a scenario. `/devices/nas-01` overflows by **374 px** at 360 px today.

**AC8 — Nothing regresses at 1280 px.** The desktop look is Guy's premise (3) of 2026-08-13 and only
the parts arbitration 2 names may change. The existing browser gates must stay green, and 🔴 **the
axe gate's own numbers are a carrier here**: 10 routes + 4 states, 0 violation nodes.

**AC9 — Every divergence from the UX spec is REGISTERED BY NAME with its owner** — the search
magnifier, the swipe gestures (`:1570`), the undo toast, the offline banner, and anything else the
spec prescribes for mobile that this story does not build. ⚠️ *A section that says "registered" is not
a registration* (story 6b.9): the row must exist in `deferred-work.md` and be diffed before the commit.

**AC10 — The live count lives in THIS file**, and every figure names the state it was taken against.
Baseline: **761 tests** (503 bin + 161 core + 97 xtask) at `master` = `3d8476f`, nine `cargo xtask ci`
gates, axe 10 routes + 4 states, kbd 30 checks.

---

## §0 — What contexting MEASURED, on a booted binary and a real Chrome 151

*Method: `master` at `3d8476f`, `cargo build --workspace --locked`, the binary booted against a
`mariadb:10.11.11` on port **13405** and seeded with `a11y/seed.sql`, driven by the `puppeteer-core`
already in `a11y/node_modules`. Routes DERIVED from the rendered navigation, as both gates do.*

### §0a. 🔴 THE PREMISE THIS STORY WAS HANDED IS FALSE, AND IT IS FALSE IN THREE DOCUMENTS

Epic 6b's retrospective (§5 decision 2), its action 2, and `sprint-status.yaml` all set this story's
subject as **"the zero `@media` rules"**. Measured on the tree they describe: `app.css` carries
**four** `@media` at-rules.

| rule | shipped by | date |
|---|---|---|
| `@media (max-width: 620px)` — `.cols` to one column | story **3.7** | 2026-07-20 |
| `@media (prefers-reduced-motion: reduce)` | story 6b.1's ramp | — |
| `@media (max-width: 720px)` — `.triage-panes` to one column | story **6b.4** | 2026-08-19 |
| `@media (max-width: 900px)` — `.ipam-layout` to one column | story **6b.7** | 2026-08-20 |

So the claim was false by **thirty-three days** when `deferred-work.md:4863` wrote it on 2026-08-22,
and by thirty-five when the retrospective repeated it on 2026-08-24.

🔑 **And the cause is visible in the file itself, which is what makes it worth a section.** `app.css:318`
says *"the mock has no `@media` rule"* — **scoped correctly, and TRUE**, written by story 6b.2 on
2026-08-18. What drifted is the SUBJECT: a measurement taken on **the mock** was restated about
**`app.css`**, and then inherited by two more documents. ***A measurement taken on one artefact and
applied to another*** is story 6b.4b's named class, met here on a premise rather than on a mutation.

⚠️ **The substantive gap is REAL and is sharper than the false claim.** Three layout breakpoints exist
at **620 / 720 / 900 px** — *none* of them is one of NFR24's **360 / 768 / 1280**, each was added
ad hoc by the story that needed it, and **not one touches the shell**. The story is not *"add media
queries"*; it is *"the breakpoints that exist are at the wrong widths, address one component each, and
leave the frame fixed."*

### §0b. 🔴 NINE OF TEN SCREENS OVERFLOW HORIZONTALLY AT 360 px — and one still does at 768

`documentElement.scrollWidth - clientWidth`, per route, with the widest offender named:

| route | 360 px | 768 px | 1280 px | worst offender at 360 |
|---|---|---|---|---|
| `/devices` | **+587** | **+179** | 0 | `td.mono` |
| `/apps` | **+389** | 0 | 0 | `table.grid` |
| `/devices/nas-01` | **+374** | 0 | 0 | `td` |
| `/ipam` | **+221** | 0 | 0 | `li.ipam-cell` |
| `/alerts` | **+131** | 0 | 0 | `table.grid` |
| `/triage` | **+68** | 0 | 0 | `span.entity.mono` |
| `/diagnostic` | **+59** | 0 | 0 | `span.diag-value` |
| `/dashboard` | **+42** | 0 | 0 | `span.count` |
| `/commissioning` | **+42** | 0 | 0 | — |
| `/sources` | **0** | 0 | 0 | — |

🔑 **`/sources` is the control that makes the table mean something**: one screen of prose and short
labels is clean at 360 px with no work, so the overflow is not *"the shell is 208 px"* alone — it is
the shell **plus** tables, monospace identifiers and a 256-cell grid. Both halves need an answer, and
`/devices` needs both at once (it is the only route that fails at 768 too).

⚠️ **`/devices` at +587 px is more than the viewport is wide.** That is a table, and AC2 is what it
needs; the register already carries *"a needle spelled `class="…"`"* about that file.

### §0c. 🔴 21 OF 33 DISTINCT CONTROL SHAPES ARE UNDER 44 px HIGH AT 360 px

Measured over `a[href], button, [role=button], input, select, summary` on all ten routes, deduplicated
by shape:

| height | shape | occurrences | screens |
|---|---|---|---|
| **16 px** | bare `a` | 2 | 2 |
| **27 px** | `a.btn-sort` (story 6b.4's age sort) | 1 | 1 |
| **30 px** | `a.filter` (seven of them) | 7 | 1 |
| **34 px** | **`a.nav-entry`** | **100** | **10** |
| 34–38 px | assorted bare `a` | 11 | — |

✅ **And what already passes is worth as much**: the five `span.btn-gesture` of story 6b.4b's action
bar measure **47 to 114 px** high, and the queue rows are 95 px. *The story that had to argue about a
`<span role="button">` got the size right; the shell never did.*

🔑 **`a.nav-entry` is the finding.** One shape, ten entries, ten screens — **100 of the failures are
one rule** (`app.css:352`, `padding: 7px 10px`), and it is inherited by every screen including the two
that are otherwise clean. ⚠️ Its 34 px CLEARS WCAG 2.2 AA (2.5.8, 24 × 24) and misses NFR24's own
stricter 44 (the 2.5.5 AAA figure) — which is exactly the tension arbitration 2 exists to settle, and
which `deferred-work.md:4722` measured at 1280 px on 2026-08-22 and routed to the retrospective.

⚠️ **The 1280 px half of that register row is now STALE in the product's favour**: it lists
`.btn-gesture` at 114 × **29** px, and story 6.4 filled that control — it measures 47 px at 360 px
today. Re-measure before quoting it.

### §0d. 🔴 NEITHER BROWSER GATE SETS A VIEWPORT — the apparatus has never seen a phone

`grep -n "viewport\|setViewport\|defaultViewport" a11y/*.mjs` returns **nothing**. Both gates run at
Puppeteer's default window, so:

- every axe result this project has ever recorded is a **desktop-width** result;
- story 6b.11's *"axe 10 routes, 0 violation nodes"* and story 6.4's *"10 routes + 4 states"* are true
  and say nothing about 360 px;
- 🔑 **adding a viewport to the EXISTING gates changes what they measure**, so it is not a free edit:
  axe at 360 px may surface violations that are real and were simply never looked for. Whether that
  lands in this story or is registered is a decision the validation should put to Guy with a number
  attached — *run axe once at 360 px and count, before deciding who owns the result.*

### §0e. ⚠️ THREE ARBITRATIONS FOR GUY, each with the option refused stated

**1 — What replaces the 208 px rail below 768 px?** The UX spec says *"bottom bar / drawer"* with **a
permanent search magnifier** (`:1310`, `:1329`, `:1570`). 🔴 **The product has no search**: ten routes,
no search screen, no search route. Options: **(a)** a bottom bar carrying the existing entries and
**no** magnifier, the divergence registered — *shipping only what exists*; **(b)** the rail collapses
to a disclosure/drawer above the content — smaller change, keeps ten entries reachable, further from
the spec; **(c)** the spec followed literally, magnifier included and dead. ⚠️ **(c) contradicts what
Epic 6b spent itself learning** — ten well-lit dead ends, counted at its retrospective — and story
6b.4b's own rule that *announcing an absent gesture is a promise*. **Recommendation: (a).**

**2 — Does the 44 px floor apply at 1280 px, or only below 768?** `deferred-work.md:4722` frames it
exactly: the product already clears WCAG 2.2 AA everywhere, and raising three control kinds to 44 px
**changes the density of the whole interface**, against Guy's premise (3) of 2026-08-13 (*the mock's
palette and typography are adopted*). Options: **(a)** 44 px at ≤ 768 only, desktop keeps the mock's
density and is stated to meet AA and not NFR24's AAA figure — *the divergence written, not hidden*;
**(b)** 44 px everywhere, the mock's density revised, which the retrospective's *"density revisited"*
can be read to authorise. ⚠️ **The retrospective already refused the third option** — amending NFR24
to match the mock — so it is not reopened here. **Recommendation: (a), with (b) cheap to take later
and expensive to undo.**

**3 — Does this story TOUCH the two existing gates, or add a third?** A third file
(`a11y/responsive-probe.mjs`) keeps each gate's contract single-purpose and leaves the axe question of
§0d separable; extending `axe-gate.mjs` with a viewport loop is ×3 the runs and mixes *"this page has
an ARIA fault"* with *"this page is 587 px too wide"* under one exit code. ⚠️ **Whatever is chosen,
`AUTHORSHIP_ROOTS` and `SANCTIONED_SITES` do NOT need reopening** — a probe writes no SQL — but
`ci.yml` gains a step and `.gitignore`'s `node_modules` lesson (story 6b.11: `node_modules/foo` was
**not** ignored) applies. **Recommendation: a third gate.**

### §0f. WHAT THIS STORY MUST NOT BECOME

- **Not a Tailwind chain.** Refused twice by Guy on the measurement that it generates nothing this UI
  uses (`app.css` is hand-authored, story 6b.1). A responsive story is not a reason to reopen it.
- **Not a swipe layer.** `ux-design-specification.md:1570` prescribes swipe-to-defer with a visible
  button equivalent — those gestures belong to **Epic 7**, and four of the five action-bar controls
  are `Gesture::Planned { owner }` saying so on screen. Registering them is AC9's job.
- **Not a redesign.** AC8 is what bounds it: the desktop look is a decision, not this story's canvas.
- **Not a screenshot suite.** `epics.md:165`'s *"snapshot-verified"* would be a new apparatus with its
  own flakiness; the property NFR24 names is measurable directly and AC1/AC3 measure it. ⚠️ If the
  validation disagrees, that is a fourth arbitration and not a silent scope change.

---

## §0g — WHAT THE VALIDATION MEASURED, 2026-08-28 (two fresh-context layers, own worktree and own store each)

🔑 **This section exists because the story was deferred, not despite it.** The validation ran before
Guy's decision, and its yield is why issue #131 is a change request carrying numbers rather than a
line saying *"make it responsive"*. **Read this before the ACs above.**

### 🔴 Two criteria were NOT satisfiable as written, and two CONTRADICTED each other

| AC | verdict | the measurement |
|---|---|---|
| **AC2** | **NOT satisfiable** | *"Reachable by keyboard"* has no property, and axe is **measured** not to carry it: `scrollable-region-focusable` is in the gate's tags and **cannot fire on a region with focusable descendants** — i.e. on every table whose rows are links. On `/devices` the last column header sits at **x = 709 in a 360 px viewport**, unreachable by keyboard, while AC1's property reads **0**. |
| **AC3 vs AC4** | **CONTRADICT** | The action bar clears 44 px only because the rail squeezes its column to 74.4 px and the text wraps. Remove the rail as AC4 requires and the same controls fall to **28.8 px**. 🔴 **§0c's single ✅ was an artefact of the layout AC4 deletes.** |
| **AC6** | **not executable** | *"Relative units throughout"* over **297 `px` occurrences against 6 `rem`**. Naming what stays fixed *is* the sweep the AC says it is not. |
| **AC8** | satisfiable, **carries nothing** | measured — see the next block. |

### 🔴 NOTHING IN THE APPARATUS CAN SEE A RESPONSIVE CHANGE, and the Dev Notes said the opposite

A complete prototype pass — four `@media` blocks, a bottom bar, `min-height: 44px`, a table wrapper,
**plus a deliberately orphaned CSS class** — left **761 tests, all nine `cargo xtask ci` gates,
`axe-gate.mjs` and `kbd-probe.mjs` GREEN**. The guards are alive (an undefined class reds exactly one
test, `page.rs:4826`); they are **orthogonal to layout**.

⚠️ **The Dev Notes table below is therefore FRAMED BACKWARDS, and is kept with this correction rather
than quietly rewritten.** It is headed *"what ALREADY reds when you touch `app.css`"* and says *"these
are the ones that will red on a responsive edit"* — **not one of them does**. A developer would read
the absence of red as confirmation. ***The true sentence is the inverse: nothing cargo-side can see a
responsive change, and that is the whole argument for AC5.***

🔑 **And a SIXTH guard the table omits is hit FIRST**: AC2's keyboard-reachable wrapper needs an
accessible name, an accessible name needs an `app.yml` key in both locales, and `page.rs:2912` refuses
literal prose in a template. **No AC and no task budgets that key.**

### 🔴 §0's measurements reproduce EXACTLY — in English, and in no other locale

Every §0b figure and every worst-offender element name reproduces to the pixel in the **default
(English) locale**. **None of the six data-dependent ones reproduces in French** — `/devices` is
**+616 / +208** there against +587 / +179, `/triage` +83 against +68. ⚠️ French is what Guy reads and
what every browser look in Epic 6b used, and **§0 names no locale**. A gate floor pinned to these
numbers is pinned to one language.

Likewise **§0c's dedup rule is unstated**, and three rules give three answers on one tree: 21 of 34
(English, keyed on tag + class + width + height), 20 of 33 (French, same key), 9 of 18 (keyed on tag +
class + height). ⚠️ *"21 of 33"* is **one number from each of two runs**. What reproduces under every
rule is the finding that matters: **`a.nav-entry`, 183 x 34, x100, on ten screens**.

### 🔴 Corrections to §0 itself, each with what refuted it

- **§0c's *"the 1280 px half of that register row is now STALE"* is FALSE.** `deferred-work.md:4722`
  lists `.btn-gesture` at 114 x 29 as a **1280 px** measurement, and at 1280 px today one of the five
  is **exactly 114 x 29**. 🔑 A 1280 px figure was declared stale on the strength of a 360 px one —
  ***a measurement taken at one viewport and applied to another***, the very class §0a names,
  committed in the section that names it.
- **§0c's control paragraph is wrong on both numbers**: the action bar's five measure **47-64 px** at
  360, not 47-114 (the 114 px control is on `/commissioning`; eight elements across three screens were
  read as five on one), and the queue rows are **76-210 px**, not 95 — 95 being the register's
  **1280 px** figure, itself no longer current.
- **The dates**: both false-premise documents were written **2026-08-24**, so both were false by
  **thirty-five days**, not *"thirty-three and thirty-five"*. ⚠️ The false 33 is already propagated
  into `sprint-status.yaml`.
- **Chrome is 152.0.7977.64**, not 151 — a version copied from an earlier story's record instead of
  read from `--version`, in a story whose §0a subject is a claim copied instead of measured.
- **§0d UNDERSTATES**: the gates run at **800 x 600**, where `(max-width: 900px)` already matches — so
  `/ipam` has only ever been axe-tested **collapsed**, and **the 1280 px layout has been measured by
  no instrument either**. And `target-size` (WCAG 2.5.8) is run by **neither** gate: the tag list
  omits `wcag22aa`.
- **§0a's *"scoped correctly, and TRUE"* about the mock rests on no check that can be run here** — the
  reference mock is in no path under this repository. The defensible form is *"scoped correctly, and
  outside this repository's reach to verify."* ⚠️ *Under this project's own rule that is an assertion.*
- **The References omit the two decision records that actually govern AC2, AC4, AC6 and AC9** —
  **UX-DR55** (`epics.md:312`: breakpoints, relative units, *wide content scrolls in its own
  container*, no horizontal overflow) and **UX-DR54** (`epics.md:311`), which enumerates AC9's
  register list verbatim. ⚠️ AC4 cites `:841` for the search magnifier and `:841` does not carry it.

### ✅ THE NUMBER §0d ASKED FOR, and it settles arbitration 3

**axe-core at 360 x 640 over the ten derived routes + four states: `0 violation nodes, 0 failing
routes`.** The stated fear — *"axe at 360 px may surface violations that are real and were simply
never looked for"* — is **REFUTED at those tags**. The gate question is a cost question.

### ⚠️ Holes the ten ACs do not cover at all

- **A page with no `<meta name="viewport">` lays out at Chrome's 980 px default**, so AC1's property
  is **0 and meaningless** there. `/gap` (a real 200 route) and the **401 body** are both such pages.
- **`getBoundingClientRect().height` is not what WCAG 2.5.5/2.5.8 mean.** A drawer spelled
  `visibility: hidden` measures 63 px and **passes AC3 over an invisible control**; hit-testability
  (`elementFromPoint` at the target's centre) is the property.
- **AC5's floor counts CHECKS and is blind to an empty population**: point an assertion's selector at
  a class that does not exist and the count stays intact while nothing is sized — *Epic 5's dominant
  class written into a criterion.*
- **AC1 cannot tell a scroll from a CLIP**, and neither can a `scrollLeft` oracle: with
  `overflow-x: hidden` on the wrapper, 373 px are unreachable, AC1 reads 0, and `scrollLeft` is still
  settable. The oracle is computed `overflow-x` **plus** `scrollWidth > clientWidth`.
- **320 px, not 360, is the WCAG AA figure** (1.4.10 Reflow). Four screens still overflow at 320 with
  the prototype in place. NFR24 says 360; nothing names the gap.
- **Landscape and the 401 body are named by no criterion**, and no browser gate can reach the 401 at
  all — Chrome yields no document for a challenge.

### ⚠️ The bottom bar, measured — and it changes arbitration 1 rather than answering it

Ten entries are **797 px of content in a 360 px bar, four of ten visible**. All ten stay
keyboard-reachable and the bar is correctly announced (`<nav aria-label="Screens">`), so the objection
is **not** accessibility: six destinations sit behind a horizontal scroll with no touch affordance.
🔑 **The mock's bottom bar assumes four or five entries; this product has ten.**

### ⚠️ Two process findings, from the validation's own conduct

- **`cargo test` and the browser sweep must not share a store.** One layer's two passes diverged; the
  cause is established, not guessed — a `cargo test --workspace` ran against the same `DATABASE_URL`
  between them, and **`kbd-probe.mjs` WRITES** (it presses the documenting gesture for real). *Story
  6b.11's "green on residue", live inside a validation session.*
- **A broad `pkill -f 'target/debug/opencmdb'` killed the sibling layer's server.** Two isolated
  layers are isolated by worktree and by store, **not by process table**. Kill by port.

---

## Tasks / Subtasks

- [ ] **T1 — Re-measure §0 on the tree you start from** (AC10). ⚠️ Do not inherit §0's numbers: they
      were taken at `3d8476f` and the point of this project's method is that a figure names its state.
      - [ ] Boot against a store, seed with `a11y/seed.sql`, and re-run the 360/768/1280 sweep.
      - [ ] Record the baseline in this file, with the command that produced it.
- [ ] **T2 — Put §0e's three arbitrations to Guy BEFORE writing CSS** (AC4, AC3, AC5). Record each
      answer with the option refused, in this file.
- [ ] **T3 — The gate first, red, before the product is touched** (AC5). Prove-to-red is the house
      rule: the probe must FAIL on today's stylesheet, naming a route and a number, before any fix.
      - [ ] Routes derived from the rendered navigation, never a list.
      - [ ] 0 / 1 / 2 contract, whole body inside one `try`; a skipped check answers 2.
      - [ ] A floor EQUAL to what is there, not under it (story 6b.11).
- [ ] **T4 — The shell** (AC4, arbitration 1). The rail below the chosen breakpoint.
- [ ] **T5 — The wide content** (AC1, AC2): tables and the occupancy grid in scrolling containers,
      keyboard-reachable.
- [ ] **T6 — The targets** (AC3, arbitration 2), starting with `a.nav-entry`, which is 100 of the
      failures in one rule.
- [ ] **T7 — Relative units, and the fixed dimensions NAMED** (AC6) — a written list with a reason
      each, not a sweep.
- [ ] **T8 — The two existing gates stay green** (AC8), and §0d's axe-at-360 question is answered with
      a number rather than a sentence.
- [ ] **T9 — Mutation pass with predictions written FIRST**, through `cargo xtask mutate` where the
      carrier is cargo-side. ⚠️ **The browser gates are NOT driven by it** (story 6.4b, AC10) — and
      *this story's carrier is a browser gate*, so most of its mutations must be run by hand with the
      recipe `--help` carries. Say so rather than letting the driver's green stand for the whole.
- [ ] **T10 — A real look on a real narrow viewport**, in both locales. ⚠️ Four stories in this epic
      deferred a browser look on a sentence nobody had run `command -v` against; Chrome **151** and
      Firefox are installed.
- [ ] **T11 — Register every unbuilt spec prescription BY NAME** (AC9) and diff `deferred-work.md`
      before committing.

---

## Dev Notes

### What the previous story leaves you

Story **6.4b** (`8c5ecb3`, merged 2026-08-28) shipped `cargo xtask mutate`. Use it — and read its
AC10 first: **it does not drive the two browser gates**, and this story's central carrier is one.
Its record also carries the measurement that rewrites how every earlier figure reads: `cargo test
--workspace` **stops at the first failing crate**, so any *"N red"* in this project's history where
`opencmdb-bin` reddened is a **lower bound**.

Story **6.4** (`b6dfd69`) is the source of AC1's method: *a source guard cannot see a cascade* — four
correct `--accent-document` declarations painted nothing because a 0-2-0 selector beat a 0-1-0 one,
and the guard counting declarations found all four and passed. **A layout is the same shape of
problem, one axis over.**

### The house rules that bite here

- **A guard placed where the defect cannot occur reads as coverage and is none** — Epic 5's dominant
  class, counted in at least nine of its twenty stories, and it recurred four times inside Epic 6b.
  A guard that reads `app.css` measures what was written; the operator gets what was **computed**.
- **An enumeration cannot claim the completeness of a property** (story 5.12). A list of selectors to
  check is an enumeration; *every interactive element on every derived route* is a property.
- **Prove-to-red, and write the prediction before the mutation** (Epic 6b retrospective, action 4).
- **`git checkout --` has destroyed uncommitted work four times in this project.** Restore from a
  snapshot; `cargo xtask mutate` does it for you and refuses if a leftover is on disk.
- **Read the exit status from `$?` on a file, never through a pipe** — two commits went in over a red
  tree because `cargo … | grep` takes grep's status.
- **`cargo clippy --workspace -- -D warnings` does NOT cover test targets.** CI does. Run
  `cargo clippy --workspace --all-targets -- -D warnings` and
  `RUSTFLAGS="-D warnings" cargo test --workspace --locked` before pushing.
- **`app.yml` is invisible to Cargo's incremental build** (story 6b.9): editing only the translation
  file rebuilds in 0.07 s and the new string is absent from the binary. If this story adds a key,
  touch a `.rs` file or `cargo clean -p opencmdb-bin`.
- **`grep -a`, not `grep`** — GNU `strings`/`grep` break their run on multibyte characters, and most
  French values carry one.

### 🔴 What ALREADY reds when you touch `app.css` — read this before the first edit

The stylesheet is not unguarded. **Thirteen sites under `crates/opencmdb-bin/src/` name it**, and
these are the ones that will red on a responsive edit — each is CORRECT about what it tests, so a red
here is information, not an obstacle:

| guard | file | what it refuses |
|---|---|---|
| `every_class_a_template_names_is_defined_in_the_stylesheet` | `page.rs:4826` | a class in any template with no rule — ⚠️ **it silently skips any `class="…"` containing a brace** (registered, unfixed) |
| `the_identity_sections_own_rules_never_reach_for_the_accent` | `page.rs:2616` | `--accent-document` outside the documenting gesture; it scans **to the closing brace**, not the selector line |
| `every_text_token_clears_aa_on_every_ground_it_can_sit_on` | `page.rs:3500` | a token that stops clearing AA; ⚠️ it reads the **light `:root` BLOCK with comments stripped** — story 6b.11 measured a CSS comment carrying the old value turning the property off in silence |
| the pill-state rule guard | `example_screens.rs:1059` | a rendered state whose rule `app.css` does not define |
| `referenced_urls` | `page.rs` (same module) | any external `url(…)`, protocol-relative forms included — the fonts are embedded and stay embedded |

⚠️ **The theme structure is a property, not a convention**: the light palette lives in an
UNCONDITIONAL `:root`, the dark set is selected by nothing, and story 6b.1's review measured five
ordinary gestures defeating the first version of these guards — an OS dark-mode `@media` above the
real `:root` among them. **A responsive story adds `@media` blocks; adding one above `:root` is
exactly the shape that broke the scan.** Add breakpoints *after* the token block, and re-read that
review's five specimens before assuming a guard is wrong.

⚠️ `main.rs:1875`/`:3316` serve the sheet and assert its route; `diagnostic.rs:196` lists it; and in
a **debug build `rust-embed` reads from disk**, so *served* and *source* are the same bytes by
construction — a served-versus-source check is unreddenable locally (story 6b.11, recorded
green-by-construction rather than counted).

### Project Structure Notes

- The stylesheet is **`crates/opencmdb-bin/assets/app.css`**, 1044 lines, hand-authored on the mock's
  tokens. **There is no Tailwind chain and no `cargo xtask css`.**
- The frame is `crates/opencmdb-bin/templates/_shell.html` (which already carries the correct
  `<meta name="viewport">`) and `_nav.html`.
- `page.rs` is at **1978** of the 2000-line `file-size` ceiling — 22 lines of headroom. If this story
  needs Rust there, it **splits** rather than grows (`CLAUDE.md`'s engineering convention, and story
  6.4 already split `identity_view.rs` out for exactly this reason).
- The gates live in `a11y/` and are CI steps, **not** `cargo xtask ci` gates — that count stays at
  **nine**, and the exception is deliberate: they need a browser and a browser is not Rust.
- ⚠️ A stylesheet guard added by story 6b.4 uses a shared `templates()` helper whose flat `read_dir`
  was fixed once already; and it **scans Askama comments and silently skips any `class="…"`
  containing a brace** (registered, unfixed).

### References

- [Source: `_bmad-output/planning-artifacts/prd.md#L1427`] — NFR24, verbatim.
- [Source: `_bmad-output/planning-artifacts/epics.md#L165`] — NFR24 with *snapshot-verified* and the
  deep-linked phone clause.
- [Source: `_bmad-output/planning-artifacts/epics.md#L313`] — UX-DR56.
- [Source: `_bmad-output/planning-artifacts/ux-design-specification.md#Responsive-Design--Accessibility`]
  — `:1562-1580`, the strategy and the three breakpoints; `:841` and `:1310`/`:1329`/`:1570`, the
  bottom bar and its magnifier.
- [Source: `_bmad-output/implementation-artifacts/epic-6b-retro-2026-08-24.md#5`] — decision 2, and §7
  action 2 with its owner.
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md#L3759`] — the deferral of 2026-08-18
  (*"not now, we will see later"*), which this story discharges; and `#L4722`, the 44 px table.
- [Source: `crates/opencmdb-bin/assets/app.css#L318`] — the comment whose correct scope §0a shows
  drifting.

### Project context reference

`docs/project-context.md` and `CLAUDE.md` are the twins; `_bmad-output/implementation-artifacts/sprint-status.yaml`
is the live status. Both twins must be updated in the same push as any behaviour change
(`CLAUDE.md`'s *docs-current-before-push*).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
|---|---|
| 2026-08-28 | Story created and CONTEXTED **against a booted binary and Chrome 151**, not against the documents alone. 🔴 **The premise the retrospective handed it is FALSE in three documents**: *"the zero `@media` rules"* — `app.css` carries **four**, three of them layout, the oldest shipped by story 3.7 on 2026-07-20, so the claim was false by thirty-three days when it was written. 🔑 The cause is in the file: `app.css:318` says *"the **mock** has no `@media` rule"*, which is TRUE, and the subject drifted from the mock to the stylesheet — *a measurement taken on one artefact and applied to another*. ⚠️ The real gap is sharper: the three breakpoints that exist are at **620 / 720 / 900 px**, none of them NFR24's 360 / 768 / 1280, each added ad hoc, and **not one touches the shell**. 🔴 Measured: **nine of ten screens overflow at 360 px** (`/devices` by **+587 px**, more than the viewport is wide) and `/devices` still does at 768; **21 of 33 distinct control shapes are under 44 px high**, of which **100 occurrences are one rule** — `a.nav-entry` at 183 × 34, inherited by every screen; and the rail is a fixed **208 px** at every width, **58 % of a 360 px viewport**. 🔴 **Neither browser gate sets a viewport**, so no accessibility instrument in this project has ever seen a phone. ✅ `/sources` is clean at 360 px and is kept as the control that shows the overflow is not the shell alone. ⚠️ **Three arbitrations are OPEN** (§0e): what replaces the rail — the spec's bottom bar prescribes a **search magnifier and the product has no search**; whether 44 px applies at 1280 or only below 768; and a third browser gate versus a viewport loop in the existing one. ⚠️ NEXT: `create-story validate` (two fresh-context agents) is MANDATORY before `dev-story`. |
