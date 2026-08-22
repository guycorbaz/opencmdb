# Story 6b.11: The keyboard layer and the focus contract

Status: ready-for-dev

✅ **Contexted, ARBITRATED (4 decisions by Guy) and VALIDATED by two fresh-context layers,
2026-08-22.** The validation produced **38 findings, 12 of them HIGH**, and **corrected the
contexting on eight points** — including three measurements of mine. Every correction is
carried below at the place it belongs, not appended.

🔑 **The gap-hunt layer BUILT rather than read**, and the developer inherits working code:
the axe harness (1.4 s over ten routes), the three-line patch that takes **237 violations
to 0**, and the DOM-level property test for AC2. See §0f.

## Story

As the operator,
I want to run the queue from the keyboard,
So that triage is as fast as clearing a mailbox.

## Acceptance Criteria

*(`epics.md:2298`, its criteria at `:2306`–`:2314`. §0 explains every divergence.)*

**AC1 — Given** the triage queue **When** the operator navigates **Then** `↑` and `↓`
move the selection, **the focus follows immediately and the URL catches up**, and the
layer **acts only while focus is inside the queue**.
🔴 **ARBITRATION 1: arrows ONLY** — `epics.md`'s `j`/`k` and `⏎` are not implemented
(§0a). 🔴 **ARBITRATION 4: the selection moves by shape (E)** — focus and highlight move
on the keypress, the URL follows after 250 ms of quiet (§0d). ⚠️ **The `INPUT`/`TEXTAREA`
exclusion the epic names is NOT the guard that matters here** — measured, this product
contains no text input at all (§0e).

**AC2 — Given** the gestures that do not exist yet **When** bindings are written **Then**
**no letters are assigned** (constraint 6, `epics.md:2106`) — not even the mock's `⌫`.
Met in FULL: the binding set contains no letter at all, so there is nothing to retire
when Epic 7 writes the six together against the corpus.

**AC3 — And the focus contract that is REACHABLE ships, and the half that is not is
STATED.** 🔴 **ARBITRATION 2.** There is no HTMX swap on any of the ten screens (§0b), so
*"focus is never lost across a swap"* is a property of an event that cannot occur. What
ships: focus follows the selection, **focus is VISIBLE — which today it is not** (§0e),
the tab order reaches every control, nothing traps it. The swap half → **story 6.4**.

**AC3b — axe-core runs on the ten routes and is GREEN.** 🔴 **ARBITRATION 3: in THIS
story.** ⚠️ **It closes NFR25's axe-core FLOOR and not NFR25** — `prd.md:1429` calls axe
*"a blocking floor, not a ceiling"* and makes two further gates blocking: a scripted
keyboard checklist, and a screen-reader pass per release. Those are registered.

### The criteria this story adds to itself

**AC4 — The live count for the project lives in THIS file** (story 6.1's AC8); no figure
is written in flight, and **every figure names the store state it was taken against**
(§0c's correction).

**AC5 — Every guard is measured RED before it passes, and a guard that reads the SOURCE
where the defect lives in the DOM is not counted.** Three of this story's subjects —
focus visibility, key handling, scroll — are invisible to any text-level test.

**AC6 — Nothing shipped promises a gesture that does not exist.** The five controls stay
`Gesture::Planned`.

**AC7 — NFR24's touch targets are taken or re-registered BY NAME.** `deferred-work.md:3740`
names **this story** as their owner: the nav entries are ~30 px against NFR24's ≥44 px.
The story may not silently ignore a row that names it (§0g).

---

## §0 — What contexting and validation established

### §0a. 🔴 AC1 AND AC2 CONTRADICT EACH OTHER, AND THE EPIC SAYS SO IN ITS OWN VOICE

`epics.md:2306` prescribes **the arrows and `j`/`k`** and `:2308` prescribes `⏎`;
`:2312` says **"no letters are assigned"**. `j` and `k` are letters. And **constraint 6,
`epics.md:2106`**, says it in the epic's own voice:

> *"The spec assigns NO keyboard letters, by decision. They are written when all six
> gestures exist, at once, against the corpus — «a letter chosen in isolation is a letter
> whose neighbourhood nobody tested». **The mock's `⏎` and `⌫` must not be read as a
> specification**."*

✅ The fact-check confirmed the quotation verbatim and found the UX spec saying it
independently at `:726`: *"This spec fixes the GESTURES and the CRITERION. It deliberately
assigns NO letters (2026-07-17)."*

**ARBITRATED — option (a), arrows only** (Guy, 2026-08-22).

| | | |
|---|---|---|
| **(a) TAKEN** | `↑`/`↓`; no letter, no `⏎` | Constraint 6 met **in full**, not in spirit. **Nothing has to be retired** when Epic 7 writes the six together. |
| (b) refused | + `j`/`k`, narrowing constraint 6 | Narrowing a constraint is a planning act. |
| (c) refused | the AC verbatim | ⚠️ **The reason contexting gave was REFUTED by the validation** and is corrected here. It said *"`⏎` would have to activate a control whose every variant is `Gesture::Planned`"* — but `:2308` reads *"`⏎` performs the gesture **that exists** on the focused row"*, which is self-limiting and satisfied vacuously. The reason that holds: **a binding written now must be retired when Epic 7 writes the six against the corpus**, which is constraint 6's own argument. *A decision explained by a false premise is one nobody can re-derive* (story 6b.5). |

🔑 The validation also noticed that `:2308` says **the focused row**, distinguishing focus
from the selection the arrows move — which is why §0d's fork exists at all.

### §0b. 🔴 AC3 NAMES AN EVENT THAT CANNOT OCCUR — MEASURED, TWICE

Measured by contexting and **re-measured independently by both validation layers**, on a
booted binary against a live store, empty and seeded:

```
ten served screens, grep -o 'hx-get|hx-post|hx-put|hx-delete|hx-patch'  → 0 on all ten
grep -c 'id="gap-card"' on each of the ten                              → 0 on all ten
```

✅ And the fact-check made the argument FIRMER than contexting had: `/gap`, the one route
that *does* serve `#gap-card`, returns a bare fragment with **zero `<script>` tags and no
doctype** — so htmx and `app.js` load nowhere near it, under any condition.

**ARBITRATED — option (A), ship what is reachable** (Guy, 2026-08-22).

| | | |
|---|---|---|
| **(A) TAKEN** | Focus follows the selection; focus visible; tab order complete; no trap. The swap half **stated and registered**. | Testable today in a real browser. AC3 ships **partly met and says so** — preferred here to a green tick over an unreachable branch since story 5.13. |
| (B) refused | give the queue a real HTMX swap | ⚠️ **Contexting's reason was FALSE and is corrected**: it claimed this contradicts epic constraint 4, but `epics.md:2102` reads *"The target is **Askama partials + HTMX**, server-rendered, one URL per screen"* — constraint 4 is where HTMX comes from. The reason that holds is the second one alone: **it invents scope; nobody asked for a swap**, and arbitration 4 reaches the same speed without one. |
| (C) refused | hand all of AC3 to story 6.4 | Only the swap half moves. |

🔴 **THE DEAD CONTRACT HAS THREE ARTEFACTS, NOT ONE** — the validation's correction, and
it changes the task:

1. `assets/app.js`, 12 lines: the `htmx:afterSwap` handler on `#gap-card`.
2. `templates/_gap_card.html:5`: `<button class="refresh" hx-get="/gap" …>` — the **only**
   `hx-*` attribute in the entire template tree.
3. `assets/app.css:204`: `.card:focus { outline: … } /* visible focus on HTMX swap */` —
   a focus rule for an element measured unreachable.

⚠️ **Removing only (1) leaves an orphan trigger and an orphan focus rule that read as
live.** T5 decides all three or says which are left standing and why. `app.js`'s handler
is removed and the contract registered with story 6.4 (my call, §0's own record).

### §0c. 🔴 axe-core WAS RUN — AND CONTEXTING'S FIGURES WERE TAKEN ON AN EMPTY STORE

**ARBITRATED — in THIS story** (Guy: *"on finit les stories avant la rétrospective"*).

**The harness costs, measured by BUILDING it:** `npm install axe-core puppeteer-core` →
**26 packages, 3.4 s**, **no browser download** — `puppeteer-core` drives the system
Chrome (151.0.7922.173). The full run over ten routes takes **1.4–1.7 s**.

🔴 **CONTEXTING'S MEASUREMENT WAS TAKEN AGAINST AN EMPTY STORE, AND `/triage` WAS BLANK.**
The fact-check reproduced its figures **digit for digit** — which is the proof the store
was empty — and that exact match is the finding: with no gap rows, `/triage` renders *"You
are up to date"*, so **the queue, the detail pane and the five `Gesture::Planned` controls
— the exact surface T4/T5/T6 target — were in no measurement at all.**

**The figures, with the store state beside them, as AC4 now requires:**

| store | routes failing | `color-contrast` nodes | distinct pairs |
|---|---|---|---|
| empty | 10 / 10 | 202 | 3 |
| seeded (`seed-example.sql`) | 10 / 10 | 211 | **4** |
| 7 gap rows | 10 / 10 | **237** | **4** |

The fourth pair is `#7a7a7d on #f5f5f8 → 3.93:1` — `--color-neutral-100`, the **selected
queue row's** background. It appears only when a row is selected, which is why an empty
run misses it. **The node count scales with the number of gap rows**, so `202 → 0` is not
a reproducible criterion; the criterion is *zero*.

✅ **The conclusion survives, and the validation proved it where contexting had not.** The
fourth pair shares the `#7a7a7d` foreground, so it is still **two token values** — and the
gap-hunt applied the fix against a **populated** store, a 60-row queue, an empty queue,
French and English: **237 → 0.** *Contexting measured what fails; the validation measured
that the fix suffices, which is the claim that was actually load-bearing.*

🔴 **The critical node is one line**, on the primary screen:
`<a class="btn-sort" href="/triage?sort=age" aria-pressed="false">` (`_triage.html:15`).
`aria-pressed` is permitted only on `role=button`; an `<a href>` is `link`, so a screen
reader is told about a toggle state on an element that has none. Shipped by story 6b.4.
⚠️ The fix is NOT `role="button"` — it is a real link, and that would break middle-click,
copy-link and the back button.

⚠️ **`epics.md:2108`'s DoD says "seven gates"; there are NINE.** Registered.

⚠️ **The harness cannot live in `cargo xtask ci`, and the exception is bigger than
contexting framed it.** `ci.yml:1` says *"a THIN runner (D56). All gate logic lives in
`cargo xtask ci`, in Rust, never here"* — that is the **YAML's** header and it forbids
exactly what this story proposes: a gate step in the YAML. It needs a browser, and a
browser is not Rust. **State it as an exception and register it**, rather than justify it
with the sentence that prohibits it.

⚠️ **The CI cost is not `npm ci` + one script.** No step in `ci.yml` has ever run
`target/*/opencmdb`; the axe step must build, export the DB URL and credentials, boot,
poll until listening, run, tear down. ⚠️ *"GitHub's `ubuntu-latest` ships Node and
Chrome"* is an **assumption to verify on the first CI run**, not a measurement — neither
contexting nor either validation layer can see a GitHub runner from here. ✅ Measured
helpful: with `OPENCMDB_SCAN_CIDR` unset and a near-empty store, all ten routes pass in
1.4 s after the fixes — **no seeding is needed and the ARP sweep can stay off**.

### §0d. 🔴 THE STORY'S CENTRAL DESIGN DECISION WAS LEFT TO THE DEVELOPER, AND THE TWO READINGS DIFFER BY 20×

The selection IS `?sel=`; the rows are links; the detail pane is server-rendered. So
*"move the selection"* has three readings, and the gap-hunt **prototyped all three** in
Chrome against a 60-row queue:

| | 20 presses at 33 ms (a held arrow) | requests |
|---|---|---|
| **(A)** the arrow navigates | **10 rows of 20 — half the presses lost** | 161 |
| **(B)** the arrow moves focus only | 20 rows of focus, **URL unchanged: AC1 literally false** | 0 |
| **(E)** focus at once, URL after 250 ms of quiet | **20 of 20, at every cadence** | **8** |

One keypress under (A) is one full document = 8 requests, 72 ms; presses arriving during
the load are **lost**, because the old document is being torn down and the new `app.js`
has not run. Re-installing the handler on every `load` changes nothing — measured.

🔴 **ARBITRATION 4 (Guy, 2026-08-22): shape (E).** Focus and highlight move on the
keypress; the URL and the detail pane catch up after 250 ms of quiet. It is **5 lines
more than (A)** and it is the only shape under which the story's own user story —
*"as fast as clearing a mailbox"* — is true. ⚠️ **Its stated cost: for 250 ms the
highlighted row and the URL disagree.** That is written here so nobody rediscovers it as
a bug.

### §0e. 🔴 THREE THINGS THE STORY GOT BACKWARDS, EACH MEASURED

**1. The `INPUT`/`TEXTAREA` exclusion guards a case that cannot occur.** Measured across
all ten served pages and the whole template tree: **zero `<input>`, zero `<textarea>`,
zero `<select>`, zero `<form>`, zero `<button>`, zero `contenteditable`.** There is
nothing to type into. ⚠️ Contexting applied the *guard-where-the-defect-cannot-occur* rule
to AC3 and then committed it in AC1.

🔴 **The exclusion that DOES bite has no guard**: the listener is on `document`, so an
operator tabbing the navigation and pressing `↓` **navigates the page** — measured:
`focus after one Tab: Triage` → `ArrowDown` → `URL: /triage?sel=…`. AC1 now scopes the
layer to the queue, and T6 presses `↓` with focus on a nav entry and asserts nothing moved.

**2. `preventDefault` on `↓` kills page scrolling on the nine screens with no queue.**
`app.js` is loaded by `_shell.html` everywhere. Measured: `/diagnostic` scrolls 0 → 26 px
with an early return, **0 → 0 without one**.

**3. "Focus is visible" is delivered by Chrome, not by this product.** Computed styles:

```
queue row   : auto 1px rgb(16,16,16)         ← UA default
nav entry   : solid 2px rgb(75,107,139)      ← the product's own rule
sort link   : auto 1px rgb(16,16,16)         ← UA default
planned btn : auto 1px rgb(16,16,16)         ← UA default
```

`app.css` holds exactly two focus rules, and one of them is `.card:focus`, for the
element §0b measures unreachable. **axe has no focus-appearance rule, so AC3b goes green
while this is true** — the guard must read `getComputedStyle(document.activeElement)`.

### §0f. ✅ WHAT THE VALIDATION BUILT — LIFT IT, DO NOT REDISCOVER IT

- **The harness.** Routes derived by scraping the rendered navigation —
  `page.$$eval("nav.nav a.nav-entry", as => as.map(a => a.getAttribute("href")))` returns
  exactly ten, and `page.rs:79` builds every `NavEntry` from `screen.href()` over
  `Screen::ALL`, so it **is** a derivation. Auth in one line
  (`page.authenticate({username, password})`); axe injected from the npm package.
  ⚠️ **Its one limit: a screen hidden from the navigation is invisible to the gate.**
- **The three-line patch, verified 237 → 0** in both locales and at three queue sizes:
  `--color-accent: #5980a6` → **`#4b6b8b`**, `--color-neutral-600: #7a7a7d` → **`#68686b`**
  (both the original colour with HSL lightness scaled down at **constant hue and
  saturation** — the mock's hue is kept exactly), and `aria-pressed` → a conditional
  `aria-current="true"`.
- **The AC2 property test**, 12 lines in the DOM, dispatching every key and reading
  `defaultPrevented`. ⚠️ **It must run at a MIDDLE selection index**: at index 0 the `↑`
  bounds check returns before `preventDefault`, so `ArrowUp` reads `false` and the test
  would pass with the arrows entirely broken.
- **Seeds** that decode: the facts column shape is
  `[{"IpV4":{"addr":"…"}},{"Hostname":{"name":"…","source":"Dns"}}]` — the flat form
  silently 500s the page.

### §0g. WHAT THIS STORY MUST NOT DO, AND WHAT IT INHERITS

- **No letter bound to anything** (constraint 6). **No control becomes live** (AC6).
- **No `epics.md` edit** — six divergences are registered instead.
- ⚠️ **`deferred-work.md:3740` names THIS story as owner of NFR24's touch targets**
  (nav entries ~30 px against ≥44 px) and `:3748` hands it a second item: 6b.2's honesty
  guard misses `display:none` and `pointer-events:none`, and *"closing those needs
  computed styles, which is exactly what axe-core/Playwright provides"* — **the harness
  T1 builds is the instrument that row was waiting for.** Take both or re-register by name.

---

## Tasks / Subtasks

- [ ] **T1 — The axe harness, first, so every later task is measured against it (AC3b)**
  - [ ] `a11y/` at the repo root: committed `package.json` + `package-lock.json`
        (`npm ci` in CI), `axe-core` and `puppeteer-core` only.
  - [ ] Routes **derived by scraping the rendered navigation** (§0f), with the limit
        written at the site. A count floor: fewer than ten derived is a failure.
  - [ ] 🔴 **The exit code distinguishes *the product has violations* from *the gate could
        not run***. Measured trap: with the database paused and `/apps` as the seed route,
        the derivation succeeds, `/triage` blocks on sqlx's 30 s acquire timeout, the
        navigation times out and the harness dies with **rc=1 — the same code as
        violations found**. Wrap every per-route navigation, not only the seed fetch.
  - [ ] A CI step that builds, boots, waits, runs and tears down (§0c: no step has ever
        booted the binary). State the `ci.yml` exception and register it.
- [ ] **T2 — Green: the two tokens (AC3b)**
  - [ ] `--color-accent` → `#4b6b8b`, `--color-neutral-600` → `#68686b` (§0f).
  - [ ] 🔴 **`--color-accent` is PINNED BY A TEST.** Story 6b.1's AC2 guard
        (`page.rs:3194`) asserts six token **hex literals**: darkening reds
        `ac2_the_sheet_carries_the_mocks_light_base_and_ramps`. **6b.1's *"carries the
        mock's palette"* and this story's *axe-green* cannot both hold literally.** Turn
        that guard into a CONTRAST property, or narrow 6b.1's claim to *the mock's hues at
        AA-conformant lightness* — and **register the divergence**.
  - [ ] ⚠️ `--color-accent-600` is `#597ea3`, visually the same colour, untouched by that
        edit. Say whether the ramp follows.
  - [ ] 🔴 **The amber is already answered, and the answer is FIX IT.** `--accent-document
        #b5793a` fails AA on **every** light ground (3.26 / 3.01 / 2.95, and 3.65 for
        white text on it). `#8d5e2d` clears all four. It is read by no live rule today, so
        axe cannot see it — **story 6.4 inherits the defect unless it is fixed here.**
  - [ ] Re-run the harness against a **populated** store: zero.
- [ ] **T3 — Green: the `aria-pressed` lie (AC3b)**
  - [ ] Express the sort state without claiming a role the element does not have.
  - [ ] 🔴 **Naming the test that will red: `main.rs:3545`**, whose oracle is
        `html.contains("aria-pressed=\"false\"")` under a message about the age-sort UX
        ban. **The accessibility defect is the support of an assertion that is not about
        accessibility** — story 6b.4's *"a test that pins the ugly thing requires it"*, on
        the same screen. Give it a new oracle; do not "update the number".
  - [ ] A guard on the **served** page (AC5).
- [ ] **T4 — The keyboard layer, shape (E) (AC1, AC2)**
  - [ ] Focus and highlight move on the keypress; the URL follows after **250 ms of
        quiet**. Replace `app.js`'s dead handler.
  - [ ] 🔴 **Scope the layer to the queue** — the listener is on `document` and an arrow
        pressed with focus in the navigation currently navigates (§0e).
  - [ ] 🔴 **Inert where there is no queue** — an unconditional `preventDefault` kills page
        scrolling on the other nine screens (§0e).
  - [ ] ⚠️ **Read the row's own `href`**, never rebuild the URL: the hrefs already carry
        `?sort=age`, so the sort survives arrow navigation **for free** — and breaks the
        moment someone constructs the URL from the selector.
  - [ ] An idempotence guard (`if (window.__kbd) return;`): a double-registered listener
        moves two rows per press, which bites the day story 6.4 adds a swap.
  - [ ] ⚠️ No letter, no `⏎`, no `⌫`. The letter-free assertion is a **property** over
        dispatched keys, run at a MIDDLE index (§0f).
  - [ ] ⚠️ The `INPUT`/`TEXTAREA` clause: drop it, or ship it as a forward tripwire **with
        its emptiness stated** (story 5.12's narrowing precedent).
- [ ] **T5 — The focus contract that is reachable (AC3)**
  - [ ] 🔴 **Write the focus styles.** Three of four focusable kinds use Chrome's default
        (§0e); a future `outline: none` would retire AC3 with the gate green.
  - [ ] Preserve story 6b.4b's convention: `aria-disabled="true"` **and** `tabindex="0"`.
        ✅ Measured intact — all five controls report `tabIndex 0` and 40 dispatched Tabs
        reach every one; no screen traps focus (11–35 Tabs to escape). Do not disturb it.
  - [ ] Decide all **three** dead-contract artefacts (§0b), not just `app.js`.
  - [ ] Register the swap half with story 6.4.
- [ ] **T6 — Measured in a real browser, because none of this is visible to a text test**
  - [ ] Real key events; assert on `document.activeElement` and on `window.scrollY`.
  - [ ] Focus visibility read from `getComputedStyle`, never from a stylesheet grep.
  - [ ] The nav-focus case: `↓` with focus on a nav entry leaves the URL unchanged.
  - [ ] Bounds (✅ measured correct today), the **empty queue** (the reachable case), and
        ⚠️ **not** "no row selected" — measured unreachable: `/triage` always selects row 1,
        and an unknown `?sel=` answers 200 and selects row 1 silently.
- [ ] **T7 — NFR24's touch targets, or a named re-registration (AC7)**
- [ ] **T8 — Mutations, the documents, the register**
  - [ ] Every guard proven RED first; the greens recorded.
  - [ ] **Six divergences registered**: `j`/`k`+`⏎`; the *seven gates* DoD; the swap half
        → 6.4; `puppeteer-core` where the UX spec names `@axe-core/playwright` (and the
        spec's *"per theme"* ask, against a dark set selected by nothing); the `ci.yml`
        exception; and 6b.1's AC2 token-literal conflict.
  - [ ] Twins and `sprint-status.yaml` in the same push, verified identical.

## Dev Notes

### What the previous story leaves you

- 🔴 **The prescribed lint does not see test code.** Use
  `cargo clippy --workspace --locked --all-targets -- -D warnings`, and
  `RUSTFLAGS="-D warnings" cargo test --workspace --locked`. ⚠️ **The CAUSE recorded on
  master is wrong and is corrected here**: `RUSTFLAGS` appears **nowhere** in `.github/`.
  The mechanism is `actions-rust-lang/setup-rust-toolchain@v1` (`ci.yml:48`), which
  injects the flag itself. The commands stay right; the sentence explaining them did not.
- 🔴 **Never read a status from a pipeline** — `cargo test | grep` returns grep's code.
- 🔴 **Verify the artefact, not the source.** `grep -a` on the binary; the server listens
  on **8080**; kill it and measure the port dead. ⚠️ In the **dev** profile `rust-embed`
  reads `app.css` from disk, so a stylesheet edit is served without rebuilding — **this
  will not hold in release**, and a template edit does need a rebuild.

### The house rules that bite here

- **A guard that reads the DOM must read the DOM.** Story 6b.4b: an attribute assembled in
  Rust and emitted with `|safe` put a native `disabled` in the served page with both
  source-reading guards green.
- **An enumeration cannot claim the completeness of a property** (story 5.12).
- **Prove-to-red, and record the mutations that come back GREEN** — that is where this
  epic's findings have come from.

## References

- `epics.md:2094`–`:2106` the six constraints (**:2102** names HTMX, **:2106** is
  constraint 6), **:2108** the DoD, **:2298** this story, **:2306**–**:2314** its criteria.
- `prd.md:1429` — axe is a floor, not a ceiling; two manual gates are also blocking.
- `ux-design-specification.md:671` `@axe-core/playwright`; `:672` *"axe covers only part of
  a11y"*; `:726` no letters, by decision.
- `screens.rs:196` `Screen::ALL` (**`Screen::Device` IS in it** — 6b.6 moved its route
  registration, not the variant); `page.rs:79` the nav build; `page.rs:3194` 6b.1's token
  guard; `main.rs:3545` the oracle T3 must replace; `_triage.html:15` the `aria-pressed`.
- `deferred-work.md:3740` NFR24's touch targets, **owner: this story**; `:3748` the
  honesty-guard rows the harness can close.

## Dev Agent Record

*(Empty until `dev-story` runs.)*

## Change Log

| Date | Change |
|---|---|
| 2026-08-22 | Story created. Four contexting findings; the AC/constraint-6 contradiction; the HTMX swap measured absent. |
| 2026-08-22 | **Arbitrations 1–3 by Guy** — arrows only · ship the reachable focus contract · axe-core in this story. |
| 2026-08-22 | 🔴 **axe-core RUN**: all ten routes FAIL. |
| 2026-08-22 | **VALIDATED by two fresh-context layers: 38 findings, 12 HIGH.** Contexting corrected on eight points, three of them its own measurements — the axe figures were taken on an EMPTY store (the surface T4–T6 target was blank); the constraint-4 and `⏎` refusals rested on false premises; `Screen::Device` IS in `Screen::ALL`. The validation also proved what contexting had only assumed: the two-token fix takes **237 → 0** on a populated store. |
| 2026-08-22 | 🔴 **ARBITRATION 4 by Guy: shape (E)** — the three readings of *"move the selection"* were prototyped and differ by **20×**; under (A) a held arrow loses half its presses. |
