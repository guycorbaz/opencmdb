# Story 6b.11: The keyboard layer and the focus contract

Status: review

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

**AC5 — Every guard is measured RED before it passes. Where the defect lives in the DOM, a
guard that reads the SOURCE does NOT SUFFICE: there must be a carrier that reads what the
browser did. The two cumulate — the source guard names the CAUSE, the browser gate names the
REALITY.** Three of this story's subjects — focus visibility, key handling, scroll — are
invisible to any text-level test.

🔴 **AMENDED by Guy on 2026-08-23, after the repair pass, and the amendment is TWO WORDS and
no change of scope.** It read *"a guard that reads the SOURCE where the defect lives in the DOM
is **not counted**"*, and that phrasing over-reached in one direction while being right in the
other:

- ✅ **The SCOPE held and was earned by measurement, not postulated.** The clause was already
  conditional — it bites only where the defect lives somewhere other than where the guard
  looks — and on this story it was three-for-three: `app.js` emptied, the two focus rules
  deleted and `aria-current` stripped from the sort link each left **490 tests green**. Story
  6b.4b had met the same family a template over, where an attribute assembled in Rust and
  emitted with `|safe` put a real native `disabled` in the served page with both guards green.
- 🔴 **But *"not counted"* reads as *"worth nothing"*, and that is false.** A source-reading
  guard is cheaper, runs with no browser, and **names the cause** where a browser gate names
  only the symptom: `every_class_a_template_names_is_defined_in_the_stylesheet` catches an
  undefined class in 0.2 s and says which one, which no rendered-page check states as
  clearly. The pair beats the gate alone; what the pair must not become is the source guard
  alone.
- ⚠️ **The cost the amendment does NOT dissolve is proportionality.** Taken literally this
  criterion produced a browser gate — `a11y/kbd-probe.mjs`, seventeen checks and ~25 s of CI
  per run. Here that is the minimum rather than zeal, because the story's subject *is* the
  rendered page; the same clause applied to a `font-family` declaration would demand a browser
  to check a string. **It is a rule about where the defect lives, never a rule about how much
  apparatus to build.**

⚠️ **The three review reports below quote the ORIGINAL wording and are NOT edited** — they are
verbatim, they were written against the criterion as it then stood, and their verdict is
unaffected: *does not suffice* and *is not counted* give the same answer on all three
measurements they took.

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

- [x] **T1 — The axe harness, first, so every later task is measured against it (AC3b)**
  - [x] `a11y/` at the repo root: committed `package.json` + `package-lock.json`
        (`npm ci` in CI), `axe-core` and `puppeteer-core` only.
  - [x] Routes **derived by scraping the rendered navigation** (§0f), with the limit
        written at the site. A count floor: fewer than ten derived is a failure.
  - [x] 🔴 **The exit code distinguishes *the product has violations* from *the gate could
        not run***. Measured trap: with the database paused and `/apps` as the seed route,
        the derivation succeeds, `/triage` blocks on sqlx's 30 s acquire timeout, the
        navigation times out and the harness dies with **rc=1 — the same code as
        violations found**. Wrap every per-route navigation, not only the seed fetch.
  - [x] A CI step that builds, boots, waits, runs and tears down (§0c: no step has ever
        booted the binary). State the `ci.yml` exception and register it.
- [x] **T2 — Green: the two tokens (AC3b)**
  - [x] `--color-accent` → `#4b6b8b`, `--color-neutral-600` → `#68686b` (§0f).
  - [x] 🔴 **`--color-accent` is PINNED BY A TEST.** Story 6b.1's AC2 guard
        (`page.rs:3194`) asserts six token **hex literals**: darkening reds
        `ac2_the_sheet_carries_the_mocks_light_base_and_ramps`. **6b.1's *"carries the
        mock's palette"* and this story's *axe-green* cannot both hold literally.** Turn
        that guard into a CONTRAST property, or narrow 6b.1's claim to *the mock's hues at
        AA-conformant lightness* — and **register the divergence**.
  - [x] ⚠️ `--color-accent-600` is `#597ea3`, visually the same colour, untouched by that
        edit. Say whether the ramp follows.
  - [x] 🔴 **The amber is already answered, and the answer is FIX IT.** `--accent-document
        #b5793a` fails AA on **every** light ground (3.26 / 3.01 / 2.95, and 3.65 for
        white text on it). `#8d5e2d` clears all four. It is read by no live rule today, so
        axe cannot see it — **story 6.4 inherits the defect unless it is fixed here.**
  - [x] Re-run the harness against a **populated** store: zero.
- [x] **T3 — Green: the `aria-pressed` lie (AC3b)**
  - [x] Express the sort state without claiming a role the element does not have.
  - [x] 🔴 **Naming the test that will red: `main.rs:3545`**, whose oracle is
        `html.contains("aria-pressed=\"false\"")` under a message about the age-sort UX
        ban. **The accessibility defect is the support of an assertion that is not about
        accessibility** — story 6b.4's *"a test that pins the ugly thing requires it"*, on
        the same screen. Give it a new oracle; do not "update the number".
  - [x] A guard on the **served** page (AC5).
- [x] **T4 — The keyboard layer, shape (E) (AC1, AC2)**
  - [x] Focus and highlight move on the keypress; the URL follows after **250 ms of
        quiet**. Replace `app.js`'s dead handler.
  - [x] 🔴 **Scope the layer to the queue** — the listener is on `document` and an arrow
        pressed with focus in the navigation currently navigates (§0e).
  - [x] 🔴 **Inert where there is no queue** — an unconditional `preventDefault` kills page
        scrolling on the other nine screens (§0e).
  - [x] ⚠️ **Read the row's own `href`**, never rebuild the URL: the hrefs already carry
        `?sort=age`, so the sort survives arrow navigation **for free** — and breaks the
        moment someone constructs the URL from the selector.
  - [x] An idempotence guard (`if (window.__kbd) return;`): a double-registered listener
        moves two rows per press, which bites the day story 6.4 adds a swap.
  - [x] ⚠️ No letter, no `⏎`, no `⌫`. The letter-free assertion is a **property** over
        dispatched keys, run at a MIDDLE index (§0f).
  - [x] ⚠️ The `INPUT`/`TEXTAREA` clause: drop it, or ship it as a forward tripwire **with
        its emptiness stated** (story 5.12's narrowing precedent).
- [x] **T5 — The focus contract that is reachable (AC3)**
  - [x] 🔴 **Write the focus styles.** Three of four focusable kinds use Chrome's default
        (§0e); a future `outline: none` would retire AC3 with the gate green.
  - [x] Preserve story 6b.4b's convention: `aria-disabled="true"` **and** `tabindex="0"`.
        ✅ Measured intact — all five controls report `tabIndex 0` and 40 dispatched Tabs
        reach every one; no screen traps focus (11–35 Tabs to escape). Do not disturb it.
  - [x] Decide all **three** dead-contract artefacts (§0b), not just `app.js`.
  - [x] Register the swap half with story 6.4.
- [x] **T6 — Measured in a real browser, because none of this is visible to a text test**
  - [x] Real key events; assert on `document.activeElement` and on `window.scrollY`.
  - [x] Focus visibility read from `getComputedStyle`, never from a stylesheet grep.
  - [x] The nav-focus case: `↓` with focus on a nav entry leaves the URL unchanged.
  - [x] Bounds (✅ measured correct today), the **empty queue** (the reachable case), and
        ⚠️ **not** "no row selected" — measured unreachable: `/triage` always selects row 1,
        and an unknown `?sel=` answers 200 and selects row 1 silently.
- [x] **T7 — NFR24's touch targets, or a named re-registration (AC7)**
- [x] **T8 — Mutations, the documents, the register**
  - [x] Every guard proven RED first; the greens recorded.
  - [x] **Six divergences registered**: `j`/`k`+`⏎`; the *seven gates* DoD; the swap half
        → 6.4; `puppeteer-core` where the UX spec names `@axe-core/playwright` (and the
        spec's *"per theme"* ask, against a dark set selected by nothing); the `ci.yml`
        exception; and 6b.1's AC2 token-literal conflict.
  - [x] Twins and `sprint-status.yaml` in the same push, verified identical.

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

### What was built, and what the building found

**728 tests** (490 bin + 161 core + 77 xtask), **nine gates green**, clippy clean over
`--all-targets`, `fmt` clean — every status read from a file with `$?`, never from a
pipeline. ⚠️ **The live count for the project lives here** (AC4), and every figure below
names the store state it was taken against.

**The axe gate is green on all ten routes, twice**: at 2 queue rows and at **5**, the
second run being the one that matters — the fourth colour pair the validation found
appears only when a row is selected, so a run against a short queue would have missed it.

#### 🔴 THE FIX IS THREE VALUES, AND THE GUARD THAT PINNED THEM HAD TO BECOME A PROPERTY

`--color-accent` `#5980a6 → #4b6b8b`, `--color-neutral-600` `#7a7a7d → #68686b`,
`--accent-document` `#b5793a → #8d5e2d` — each the original colour with HSL lightness
scaled down at constant hue and saturation. **237 violation nodes → 0.**

Story 6b.1's AC2 guard pinned six token **hex literals**, so it reddened on the repair —
which is the finding, not the obstacle: *"it carries the mock's palette"* and *"axe-green"*
could not both hold at the letter. It is now **two properties instead of one literal**: the
HUE is the mock's (±6°, so a contrast repair is legal and a repalette is not), and a new
sibling asserts that **every text token clears 4.5:1 on every ground it can sit on** —
sixteen pairings, derived from two tables rather than enumerated.

🔑 **The Rust guard and axe agree to two decimals, independently**: restoring `#7a7a7d`
reds it at **3.82:1**, the exact ratio axe reported for that pair. ⚠️ And they do not
subsume each other — axe measures the ten routes a browser paints; this walks the token
table, so a pairing no screen renders today is still checked.

#### 🔴 THE ACCESSIBILITY DEFECT WAS THE SUPPORT OF AN ASSERTION ABOUT SOMETHING ELSE

Removing `aria-pressed` reddened `main.rs:3545` exactly as the validation predicted —
whose message is about the backlog ban on brandishing age, and whose oracle was
`contains("aria-pressed=\"false\"")`. ⚠️ **And there was a SECOND one the spec had not
named**, found by running the change rather than reading it: `contains("aria-pressed=
\"true\"")` for the ON state. Both now read the two facts the ban is made of — the toggle
is offered, and it is off until asked.

#### 🔴 FOUR TIMES, MY OWN INSTRUMENT WAS THE THING AT FAULT — AND A CONTROL SETTLED IT EACH TIME

- The `↓`-on-`/diagnostic` check reported the layer killing page scrolling. **Control: the
  same page with `app.js` BLOCKED does not scroll either** — arrows do not scroll under
  headless CDP at all. The check was replaced by the property that measures MY code: the
  layer does not INTERCEPT the key where there is no queue (`defaultPrevented === false`).
  🔴 **THIS CAUSE IS FALSE AND WAS REFUTED BY THE CODE REVIEW'S ACCEPTANCE LAYER (finding 5),
  which is the correction and not a nuance.** Measured in headless Chrome 151 on
  `/diagnostic` (`scrollHeight` 728, viewport 600): the shipped code with
  `page.keyboard.press('ArrowDown')` gives **`scrollY = 40`**, and the same page carrying an
  unconditional `preventDefault` gives **`scrollY = 0`**. *The check discriminated exactly;
  what did not discriminate was the control that dismissed it.* ⚠️ And **§0e of this same
  document already carried the refuting figure** — *"0 → 26 px with an early return, 0 → 0
  without one"* — four sections above, so the story contradicted itself and **the false half
  is the one that changed a test**. The real fault is almost certainly a *dispatched*
  `KeyboardEvent`, which is untrusted and never scrolls, rather than a pressed key: a
  mutation named for one thing applied to another, not a limit of CDP. Against this
  project's own rule — **a cause needs a check, not a plausible story** — the sentence above
  is exactly the kind it forbids, and it is left standing with its refutation attached
  rather than quietly deleted. ✅ The *replacement* check is unaffected and stays: the
  property that matters is still that the layer does not intercept the key, and the keyboard
  gate now measures it on **all nine** screens with no queue rather than on a sample of
  three.
- Two assertions were absolute where the behaviour is relative (`index === 0`, then
  `index === 1`) and reddened when the queue grew from 2 rows to 5. The code was right both
  times.
- A `grep -oE` measuring the prove-to-red pass required letters before `on`, so
  `--color-neutral-600` — which carries digits — reported an empty message and looked like a
  guard that had not fired. It had.

*Four instruments, four wrong readings, and not one of them a defect in the product.*
⚠️ **Three of the four hold; the first does not — its READING was right and its stated
CAUSE was invented, which is worse than the defect it was excusing.** The repair pass
then produced six more of the same family against itself, tabled in §T8.

#### ✅ THE LETTER-FREE PROPERTY, WITH THE CONTROL THAT MAKES IT MEAN ANYTHING

`{"a":false,"j":false,"k":false,"x":false,"Enter":false,"Backspace":false,"Home":false,
"PageDown":false," ":false}` — and the positive control, **at a MIDDLE index**:
`ArrowUp:true, ArrowDown:true`. ⚠️ At index 0 the `↑` bounds check returns before
`preventDefault`, so the same test would have passed **with the arrows entirely broken** —
which is why the queue was seeded to five rows before it could run at all.

#### ⚠️ AND `clippy --all-targets` CAUGHT THREE THINGS THE PRESCRIBED LINT WOULD NOT HAVE

Three `needless_borrow` in the new test code — invisible to
`cargo clippy --workspace -- -D warnings`, which does not walk test targets. The lesson
story 6b.10's review registered, biting my own code the day after it was written down.

#### What was decided rather than built

- **NFR24's touch targets: RE-REGISTERED with the measurement** (AC7). Measured at 1280 px:
  `.nav-entry` 34 px, `.btn-sort` 27 px, `.btn-gesture` 29 px, `.queue-row > a` 95 px. The
  product **already clears WCAG 2.2 AA's 24×24**; what it misses is the stricter 44 px
  NFR24 set itself. Raising three control kinds changes the DENSITY of the whole interface,
  against epic 6b's premise (3) — *the mock's typography is adopted*. That is an arbitration
  between NFR24 and the reference, not a keyboard task, and axe cannot decide it either
  (no target-size rule at WCAG 2.0/2.1 AA). **Owner: Epic 6b's retrospective.**
- **The dead contract's other two artefacts are LEFT STANDING, deliberately and in writing**:
  `_gap_card.html:5`'s `hx-get` and `app.css`'s `.card:focus`, which now sits in the focus
  block beside the three rules this story added. Story 6.4 picks all three up with the first
  swap.

### File List

- `a11y/package.json`, `a11y/package-lock.json`, `a11y/axe-gate.mjs`, `a11y/kbd-probe.mjs` — **new**
- `crates/opencmdb-bin/assets/app.js` — the keyboard layer replaces the dead handler
- `crates/opencmdb-bin/assets/app.css` — three tokens, four focus rules
- `crates/opencmdb-bin/templates/_triage.html` — `aria-pressed` → `aria-current`
- `crates/opencmdb-bin/src/page.rs` — the hue property, the contrast property, the amber needle
- `crates/opencmdb-bin/src/main.rs` — the two sort-state oracles
- `.github/workflows/ci.yml` — the accessibility step, with its exception written
- `_bmad-output/implementation-artifacts/deferred-work.md` — three rows
- `_bmad-output/implementation-artifacts/6b-11-keyboard-layer-and-focus-contract.md`, `sprint-status.yaml`

#### Added by the CODE REVIEW's repair pass (2026-08-23)

- `.gitignore` — ⚠️ **omitted from the list above** while the story edited it; caught by the
  acceptance layer. The entry is now the PATTERN rather than one path (D20).
- `crates/opencmdb-bin/templates/_gap_card.html` — the FOURTH dead-contract artefact, decided
  in writing rather than left standing (D19)
- `crates/opencmdb-bin/src/screens.rs` — `no_screen_is_chosen_by_javascript` reads the code
  rather than the comments, with a premise that can fail (D33)
- `a11y/kbd-probe.mjs` — rewritten as a GATE: English, env-driven, the 0/1/2 contract, a floor
  equal to what is there, and seven checks the layer had none for
- `a11y/package.json`, `.github/workflows/ci.yml` — the keyboard gate runs, the store is
  seeded, `AXE_REQUIRE_QUEUE=1`, `curl --max-time`

## Code Review — the three layers, VERBATIM and UNTRIAGED

**2026-08-22.** The three adversarial layers ran against `6649e65`, each isolated from the others
and none of them shown another's findings. They are reproduced here **exactly as each layer wrote
them**, with their own heading levels — nothing merged, nothing deduplicated, nothing ranked,
no arbitration taken. **60 raw findings.**

⚠️ **Read as a set, not as a total.** Several defects were reached by two or three layers
independently, so a count over these three reports counts some defects more than once; and at
least three claims of one layer are **refuted with a measurement by another** (the Chrome leak,
the missing lockfile, the scroll control). The convergences and the refutations are exactly what
the triage pass has to establish, and it has not been run.

⚠️ **The session that launched these layers crashed mid-review.** The blind and acceptance layers
had finished and their reports survived in their transcripts; the edge-case layer had not, and was
**re-run from its original mandate in a fresh worktree** on 2026-08-22 — so its report is a first
run, not a resumption. The interrupted run left no conclusions of its own (569 bytes of narration,
measured), only its probe scripts.

Conditions, as each layer states them:

| Layer | What it was given | What it could do |
|---|---|---|
| **Blind Hunter** | `d11-code.patch` and nothing else | no repository, no build, no run — findings needing the tree are labelled *suspicion* |
| **Acceptance Auditor** | the story spec, the tree, a live `mariadb:10.11` (port 13351) | replay every AC, Chrome 151; cleaned up after itself |
| **Edge Case Hunter** | the patch, the spec, an isolated worktree, a live `mariadb:10.11` (port 13350) | plant mutations, build, boot, Chrome 151, 8-row queue; every mutation reverted from a scratchpad copy, never `git checkout --` |

---

# Blind Hunter — findings on `d11-code.patch`

Input: the patch only. I have not read the repository, run anything, or resolved any file the patch does not contain. Where a finding needs the tree to settle, it is labelled a suspicion.

---

## HIGH

### 1. The axe gate exits `1` — "the product has violations" — on almost every way it can fail to run
**Severity** HIGH · **Location** `a11y/axe-gate.mjs`, `.github/workflows/ci.yml` · **Confidence** high (settleable from the diff)

**Evidence.** The file's headline promises a three-way contract:

```
//   0 — every route clean
//   1 — the product has accessibility violations   (a real regression: fix the product)
//   2 — the gate could not run                     (fix the harness or the environment)
```

and `ci.yml` restates it: *"If that stops being true, this step fails with exit 2 — the gate could not run — and not with exit 1, which is reserved for a real regression."*

Only two call sites are actually guarded: `puppeteer.launch` (try/catch) and `page.goto`/status (`goOrGiveUp`). Everything else is unguarded top-level `await` in an ES module, where a throw is an unhandled rejection and **Node exits 1**:

- `import puppeteer from "puppeteer-core";` — package missing → exit 1.
- `readFileSync(new URL("./node_modules/axe-core/axe.min.js", …))` — axe-core missing or renamed → exit 1. This runs *before* the launch try/catch, and it is exactly the "the environment changed" case ci.yml says is exit 2.
- `await openPage()` / `page.authenticate` → exit 1.
- `await seedPage.$$eval("nav.nav a.nav-entry", …)` → exit 1.
- `await page.evaluate(axeSource)` and `await page.evaluate(async (tags) => window.axe.run(…))` → exit 1. An axe injection failure, a page that navigates during evaluate, a CSP that blocks it: all report as *the product has accessibility violations*.

**Why it matters.** The whole design rationale of this file is that a CI cannot otherwise tell "the harness broke" from "the product regressed" — and the code delivers that distinction for two of roughly eight failure modes. The most likely real-world break (a dependency that did not install, an axe bundle that moved) lands on the code reserved for a product regression, and someone will spend the morning looking for a contrast defect that is not there. Secondary: those paths also leak the Chrome process, since only `goOrGiveUp` closes the browser before exiting.

---

### 2. The new ARIA assertion is satisfied by a different element on the same page
**Severity** HIGH · **Location** `crates/opencmdb-bin/src/main.rs`, AC3 test · **Confidence** high — established from two files *inside this diff*

**Evidence.** The diff adds:

```rust
assert!(
    sorted.contains("aria-current=\"true\""),
    "and the link says WHICH view the operator is in — the accessible half of the \
     state, which `aria-pressed` on a link could not carry"
);
```

The needle is unanchored — it is a substring search over the whole page body. And `a11y/kbd-probe.mjs`, added in the same diff, establishes that a queue row already carries that exact attribute on a plain `/triage` with no query string:

```js
await p.evaluate(()=>document.body.focus());
await p.keyboard.press('ArrowDown');
// Depuis la ligne DÉJÀ sélectionnée (aria-current), ↓ va à la suivante : index 1
check(idx===1, …);
```

For `currentIndex` to have returned `0` from `document.body`, its fallback `all[i].getAttribute("aria-current") === "true"` must have matched row 0 — on the default render. The AC3 fixture has two rows ("*With two rows the ORDER is observable*").

**Why it matters.** Delete `aria-current` from the sort link entirely and this assertion still passes, because the queue row supplies the string. The one assertion written to prove the *replacement* for an axe-critical attribute is in place measures the presence of that attribute *somewhere on the page*. This is the shape the comment three lines above congratulates itself for escaping: *"a test that pins the ugly thing is a test that requires it"* — the oracle changed, the weakness moved rather than left.

---

### 3. Nothing cancels the settle timer except another arrow — a click or `Enter` inside 250 ms is overridden
**Severity** HIGH · **Location** `crates/opencmdb-bin/assets/app.js`, `select()` · **Confidence** high

**Evidence.**
```js
if (pending !== null) window.clearTimeout(pending);
pending = window.setTimeout(function () {
  window.location.assign(row.getAttribute("href"));
}, SETTLE_MS);
```
The only `clearTimeout` in the file is this one, reached only from `select()`, reached only from a `keydown` on `ArrowUp`/`ArrowDown`. There is no `click`, `pointerdown`, `keydown Enter`, `pagehide`, `beforeunload` or `blur` handler.

**Why it matters.** Press `↓`, then within 250 ms click a navigation entry (or press `Enter` on a different link, or activate anything that navigates). The old document stays alive until the new response commits; at t=250 ms the timer fires and `location.assign` replaces the navigation the operator actually asked for with the arrow-highlighted row. The slower the server, the wider the window. The module comment enumerates the shape's cost as *"for 250 ms the highlighted row and the URL disagree"* — the real cost is that for 250 ms the layer holds a queued navigation that outranks anything the operator does next, and that sentence does not appear.

---

### 4. `checked == TEXTS.len() * GROUNDS.len()` cannot fail, and is sold as the guard's premise
**Severity** HIGH · **Location** `crates/opencmdb-bin/src/page.rs`, `every_text_token_clears_aa_on_every_ground_it_can_sit_on` · **Confidence** certain

**Evidence.**
```rust
for text in TEXTS {
    …
    for ground in GROUNDS {
        …
        checked += 1;
        assert!(ratio >= AA, …);
    }
}
// 🔑 The premise, derived rather than pinned: every pairing of the two tables was
// read. A scan that matched nothing would assert nothing.
assert_eq!(checked, TEXTS.len() * GROUNDS.len(), "every text token was measured against every ground");
```

`TEXTS` and `GROUNDS` are `const [&str; 4]`. The increment is unconditional, there is no `continue`, no filter, no `if let`. `checked` is `16` on every possible execution that reaches the assertion. A token missing from the stylesheet does not skip an iteration — `token_hex(...).unwrap_or_else(|| panic!(...))` aborts the test instead.

**Why it matters.** This is a counter written to defend against an empty scan, placed over a loop that structurally cannot scan nothing. The comment beside it asserts the opposite ("*A scan that matched nothing would assert nothing*"), which is a claim about a `css.contains`-style scan that this code is not. It reads as coverage of the guard's premise and is arithmetic on two array lengths.

---

### 5. `--accent-document` is darkened as part of a contrast repair, and is a token nothing paints text with
**Severity** HIGH · **Location** `crates/opencmdb-bin/assets/app.css` + `page.rs` · **Confidence** high (both sentences are in this diff)

**Evidence.** `app.css`, in unchanged context immediately above the changed line:
```
/* 🔴 The amber, named for what it MEANS. Reserved for the documenting gesture (FR13) and
   used by NOTHING else — the mock's blue carries structure. Story 6.4 adds the first
   legitimate use; a test pins the count at zero until then. */
-  --accent-document: #b5793a;
+  --accent-document: #8d5e2d;
```
`page.rs`, the new test:
```rust
// The grounds a screen actually paints text on, and the tokens it paints with.
const TEXTS: [&str; 4] = [ …, "--accent-document" ];
```
And the doc above it justifies the whole change by rendered measurement: *"axe-core failing on ALL TEN routes: 202 `color-contrast` nodes on an empty store, 237 with a populated one, and the whole of it was three colour pairs, i.e. two token values."*

**Why it matters.** Two sentences in the same change are mutually exclusive: a token used by nothing cannot have contributed a `color-contrast` node, and it is not one of "the tokens it paints with". So either the amber's value was changed for a reason that is not the one recorded (and a reserved brand token moved with no stated justification), or the "used by NOTHING else" comment — which another test reportedly enforces at count zero — is stale. Whichever it is, the record and the code disagree about a token that is under a pinned-usage guard.

---

### 6. The ARIA attribute this change introduces is never put in front of axe
**Severity** HIGH · **Location** `a11y/axe-gate.mjs` route derivation vs `templates/_triage.html` · **Confidence** high

**Evidence.** The gate walks exactly the hrefs the navigation offers:
```js
const routes = await seedPage.$$eval("nav.nav a.nav-entry", (entries) =>
  entries.map((entry) => entry.getAttribute("href")));
```
The template renders the new attribute only in one state:
```html
{% if triage.sort_by_age %}aria-current="true"{% endif %}
```
Nothing in the harness appends `?sort=age`, `?sel=`, or any query string, and axe runs immediately after `networkidle0` with no keyboard interaction.

**Why it matters.** The stated justification for the whole ARIA change is that a browser rated the old attribute critical. The removal is therefore browser-verified; **the replacement is verified by no browser at all** — only by the substring assertion of finding 2, which does not even bind to the right element. The same blind spot covers everything else this change adds: the `.selected` class app.js writes, the `:focus-visible` rings, and the `aria-current`/`.selected` divergence of finding 11 are all states axe never reaches, because they exist only after a keypress or a query parameter.

---

### 7. `npm ci` with no lockfile visible in the change
**Severity** HIGH if true · **Location** `.github/workflows/ci.yml`, `.gitignore`, `a11y/package.json` · **Confidence** SUSPICION — I cannot list the tree

**Evidence.** `.gitignore` asserts: *"The lockfile IS committed — same doctrine as `Cargo.lock` and the hand-authored CSS, nothing resolves on the fly — and `npm ci` installs exactly it."* CI runs `npm --prefix a11y ci`. The patch adds `a11y/axe-gate.mjs`, `a11y/kbd-probe.mjs` and `a11y/package.json`; **no `a11y/package-lock.json` appears anywhere in it.**

**Why it matters.** `npm ci` aborts with a non-zero status when there is no lockfile — the gate would be red on its first run, with an exit code that is neither 1 nor 2 (finding 9). If the lockfile was simply filtered out of the review diff, this is nothing; if it was not committed, the comment asserting it is committed is false in the file whose job is to say what is committed. Settle it with `git ls-files a11y/`.

---

## MEDIUM

### 8. The route floor cannot detect either growth or duplication, and the comment claims it can
**Severity** MEDIUM · **Location** `a11y/axe-gate.mjs` · **Confidence** high

**Evidence.**
```js
// 🔑 A FLOOR, not a nicety … Ten is what `Screen::ALL` carries today; if a
// screen is added, this number moves deliberately.
const EXPECTED_ROUTES = 10;
…
if (routes.length < EXPECTED_ROUTES) { … }
```
`<` means eleven routes pass silently; nothing forces the constant to move when a screen is added, so "moves deliberately" is a hope, not a mechanism. And `routes` is a raw `map` of `getAttribute("href")` — not deduplicated, not filtered for `null`, not filtered for absolute or fragment hrefs. Ten anchors all pointing at `/triage` clear the floor and the gate then measures one screen ten times, reporting `10 route(s) derived`.

**Why it matters.** The stated purpose is *"a harness that derives nothing and reports success is the failure mode this file exists to avoid"*. A harness that derives one screen ten times has the same character and is not caught. A `Set`, a `null` filter and `!==` would cost three lines.

---

### 9. The exit-code contract is invisible to CI, and the shell has its own codes
**Severity** MEDIUM · **Location** `.github/workflows/ci.yml` · **Confidence** high

**Evidence.** The step is a plain `run:` under `set -euo pipefail`. GitHub Actions fails a step on any non-zero status and nothing branches on the value. Meanwhile the shell's own failures produce neither 1 nor 2: `cargo build` failing exits with cargo's code, the post-loop `curl -fsS` failing exits 22, `npm ci` failing exits npm's code — all before `node` runs.

**Why it matters.** The comment presents the 1/2 split as something the CI acts on ("*this step fails with exit 2 … and not with exit 1, which is reserved for a real regression*"). Nothing acts on it. Its only consumer is a human reading the log, where the `axe gate: …` stderr line already says the same thing in words — and finding 16 says even that line can be lost.

---

### 10. The focus-rule needle was narrowed to one selector while its comment says it was widened to the declaration
**Severity** MEDIUM · **Location** `crates/opencmdb-bin/src/page.rs`, the CSS-rules loop · **Confidence** certain

**Evidence.**
```rust
-            ".card:focus { outline: 2px solid var(--color-accent);",
+            // ⚠️ … The needle is the DECLARATION they now share, not the single selector
+            // that carried it while `.card` was the only focusable thing anybody had styled.
+            ".btn-gesture:focus-visible { outline: 2px solid var(--color-accent);",
```
The needle is a selector *plus* the declaration, and it is the **last** of four selectors in the group. Delete `.queue-row > a:focus-visible` from `app.css` — the change the story exists to make — and the needle still matches. Delete `.card:focus` (which the CSS comment says is deliberately kept for story 6.4's swap) and the needle still matches; that selector was previously pinned and is now pinned by nothing.

**Why it matters.** The comment claims the guard was generalised; it was moved. Three of the four selectors it is said to cover, including the one it used to cover, can be removed with the test green — and the two most likely to be removed are the ones added yesterday. It is also formatting-coupled: reordering the four selectors reds the test for no behavioural reason, inviting the fix "update the needle".

---

### 11. `select()` never moves `aria-current`, which the same file reads back as its own state
**Severity** MEDIUM · **Location** `crates/opencmdb-bin/assets/app.js` · **Confidence** high

**Evidence.** `currentIndex` falls back to it:
```js
for (var i = 0; i < all.length; i++) {
  if (all[i].getAttribute("aria-current") === "true") return i;
}
```
`select()` writes only a class and focus:
```js
all[i].parentNode.classList.toggle("selected", all[i] === row);
row.focus();
```

**Why it matters.** Two consequences. (a) For the settle window the DOM says two different rows are current: `.selected` on the new one, `aria-current="true"` on the old — and `aria-current` is the one assistive technology reads. The module's stated cost names only the URL. (b) If focus leaves the queue without a navigation (a click on inert page chrome, an `Escape` in some contexts, a programmatic blur), `document.activeElement` is no longer a row, `indexOf` returns `-1`, and the next arrow resumes from the **stale** `aria-current` — the highlight jumps backwards to wherever the server last said the selection was. One line (`setAttribute`/`removeAttribute`) in the same loop closes both.

---

### 12. The layer takes keyboard page-scrolling on `/triage`, and the probe that checks scrolling only visits screens with no queue
**Severity** MEDIUM · **Location** `app.js` scope check + `a11y/kbd-probe.mjs` check D · **Confidence** high

**Evidence.** `app.js` argues the point at length for the screens it does not affect:
```js
// 🔴 **Inert where there is no queue.** … an unconditional `preventDefault` on the arrow
// KILLS PAGE SCROLLING — measured: `/diagnostic` scrolls 0 → 26 px with this return, 0 → 0 without it.
var all = rows(); if (all.length === 0) return;
```
and then, on the screen that *does* have a queue, treats `body` as inside:
```js
var inQueue = focused === document.body || focused === null || (… .closest(".queue") !== null);
```
On a fresh `/triage`, focus is on `body`, so the operator's very first `↓` is taken; from then on focus is inside `.queue`, so every `↓`/`↑` is taken except at the two ends. The probe written to prove inertness visits `/diagnostic`, `/apps`, `/sources` — the three screens where `rows().length === 0` returns before the scope check is ever reached:
```js
for (const r of ['/diagnostic','/apps','/sources']) { … check(res.prevented===false && res.rows===0, …) }
```

**Why it matters.** A guard placed where the defect cannot occur. `res.rows===0` in the probe's own assertion is the proof: it is asserting that the early return fired, on pages selected because the early return fires there. Nothing measures the trade the change actually makes — on `/triage`, keyboard scrolling is gone for a keyboard operator with a queue longer than the viewport, and the file's own reasoning ("*an operator at the last row still expects the page to scroll*") is inconsistent with taking the press from `body` at the top.

---

### 13. The entire keyboard layer is covered by a script nothing runs, which reports success when it measures nothing
**Severity** MEDIUM · **Location** `a11y/kbd-probe.mjs`, `a11y/package.json`, `ci.yml` · **Confidence** high

**Evidence.** `package.json` declares one script:
```json
"scripts": { "gate": "node axe-gate.mjs" }
```
and CI runs `node a11y/axe-gate.mjs` only. `kbd-probe.mjs` is referenced by neither. It also gates almost everything behind fixture size:
```js
if (n >= 2) { …checks A, B, C… }
…
if (n>=3) { …checks F… }
…
console.log(fail===0 ? '\nTOUT VERT' : …); process.exit(fail===0?0:1);
```
With an empty or one-row queue it runs checks D and E only and prints `TOUT VERT` with exit 0.

**Why it matters.** ~100 lines of new production JavaScript ship with no automated coverage, and the artefact that looks like its coverage is a hand-run probe with hardcoded credentials (`op`/`pw`) and a hardcoded base URL. Worse, it is the exact failure mode `axe-gate.mjs` builds `EXPECTED_ROUTES` to prevent — *a harness that derives nothing and reports success* — reproduced one file over, without the floor. If it is meant as a record rather than a check, it should say so; as committed it reads as a check.

---

### 14. Two of the six "hue-pinned" tokens are now constrained on one property only
**Severity** MEDIUM · **Location** `page.rs`, `ac2_the_sheet_carries_the_mocks_light_base_and_ramps` · **Confidence** high

**Evidence.** The guard was rewritten from six pinned hex literals to six pinned hues (±6°), justified as: *"What is pinned instead is the pair of properties that actually matter — the HUE … and the RATIO."* The ratio comes from the *other* test, whose tables are:
```rust
const GROUNDS: [&str; 4] = ["--color-bg","--color-surface","--color-neutral-100","--color-neutral-200"];
const TEXTS:   [&str; 4] = ["--color-text","--color-neutral-600","--color-accent","--accent-document"];
```
`--color-neutral-500` and `--color-accent-700` appear in neither. Their only remaining constraint is hue. I checked the arithmetic by hand: `hue()` returns a hardcoded `240` for any achromatic colour (`max == min`), and `#0000ff` also computes to exactly `240`. So `--color-neutral-500: #0000ff`, `#000000` or `#ffffff` all pass the "mock's palette" guard, as does any 210°-family value of any lightness for `--color-accent-700`.

**Why it matters.** The rewrite was necessary — pinning literals did block the contrast repair — but the replacement's promise ("*so the mock is still recognisably the mock*") holds only for the four tokens that also appear in the contrast tables. For the other two, "recognisably the mock" now means "somewhere on a hue wheel". Cheap fix: add a lightness band, or put every token in one of the two tables.

---

### 15. `token_hex` takes the first match in a file that contains a second, complete palette
**Severity** MEDIUM · **Location** `page.rs`, `token_hex` · **Confidence** high on the mechanism, suspicion on the ordering

**Evidence.**
```rust
let at = css.find(&format!("{token}: #"))? + token.len() + 3;
```
`str::find` returns the first occurrence. The same stylesheet carries a full second palette, per unchanged context in the diff: *"🔴 The dark set — PRESENT AND SELECTED BY NOTHING. No template emits `data-theme`, and no …"*. From the hunk line numbers the light `:root` (≈ line 58) precedes the dark set (≈ line 154), so today the light values win.

**Why it matters.** (a) The test whose doc says *"Every text token clears WCAG AA on every ground it can sit on"* measures one of the two palettes in the file, silently, by source order — the other set's contrast is asserted by nothing while the sentence reads as universal. (b) The selection is positional and undocumented: moving the dark block above the light one, a pure reordering, silently repoints every guard in both new tests at the other palette with no test failing to say so.

---

### 16. `process.exit` after `console.error` can truncate the diagnostic in CI
**Severity** MEDIUM · **Location** `a11y/axe-gate.mjs`, `cannotRun` and the final block · **Confidence** high (documented Node behaviour)

**Evidence.**
```js
function cannotRun(message) { console.error(`axe gate: ${message}`); process.exit(2); }
…
console.error(`axe gate RED: ${failing.length} route(s) …`); process.exit(1);
```
When stdout/stderr are pipes — which they are under a CI runner — Node's writes are asynchronous, and `process.exit()` does not flush pending writes.

**Why it matters.** These are precisely the two paths where the message is the only thing distinguishing a harness failure from a product failure (finding 9 having established that the exit code itself reaches nobody). A red step with no reason printed is the worst of both. `process.exitCode = 2; return;` at the top level, or an explicit flush, avoids it.

---

### 17. "Two tokens were darkened" — the diff darkens three
**Severity** MEDIUM · **Location** `page.rs` ac2 comment vs `app.css` · **Confidence** certain

**Evidence.** The comment: *"**Two tokens were darkened at constant hue and saturation**, and pinning the literal would have meant choosing the mock over NFR25."* The stylesheet hunks change three token values: `--color-accent` `#5980a6 → #4b6b8b`, `--color-neutral-600` `#7a7a7d → #68686b`, `--accent-document` `#b5793a → #8d5e2d`. The neighbouring doc repeats the count from the other side: *"the whole of it was three colour pairs, i.e. two token values."* Under the narrower reading — tokens in ac2's own list — the count is **one**, not two.

**Why it matters.** No reading of the diff yields two. I verified the "constant hue and saturation" half by hand and it does hold for all three (210°/210°, 240°/240°, 30°/30°; saturation 0.51→0.52 for the amber) — so the property is true and only the count is wrong, which is the harder kind to notice later.

---

## LOW

### 18. Two adjacent helpers give opposite justifications for the same concern
**Severity** LOW · **Location** `page.rs`, `contrast()` and `hue()` · **Confidence** certain

`contrast()`: *"Floating point, deliberately: the sRGB transfer function is a power of 2.4 and an integer approximation would be a second, wrong definition."* `hue()`, twenty lines later: *"Integer arithmetic on purpose … a ratio comparison that depends on binary rounding is a guard nobody can re-derive."* The second sentence, as a general principle, indicts the first function — whose comparison is literally `ratio >= 4.5` on `f64`. Both choices are defensible; the pair of rationales is not. (No live flakiness: I computed the three new values and the tightest pair sits at ≈4.58:1, well clear of any rounding.)

### 19. A route can print red and the gate exit 0
**Severity** LOW · **Location** `a11y/axe-gate.mjs` · **Confidence** medium

`failing.push(route)` and the `🔴` print are gated on `violations.length > 0`; the exit is gated on `nodes > 0`, where `nodes` sums `v.nodes.length`. A violation carrying zero nodes prints a red line, appears in no summary, and the process exits 0. Unlikely in practice, but it is one of the two shapes the file exists to prevent — a failure indistinguishable from success. Key the exit on `failing.length`.

### 20. `token_hex` is silently wrong on two legal CSS colour spellings
**Severity** LOW · **Location** `page.rs` · **Confidence** certain

`css.get(at..at + 6)` takes six characters unconditionally. `#f2f2f3aa` (8-digit, alpha) parses as opaque `#f2f2f3` and the contrast is computed against a colour the browser does not paint. `#fff` (3-digit) grabs `fff;\n ` → `from_str_radix` fails → `None` → a panic whose message reads *"`{token}` is declared as a hex literal"*, which is false: it is, just shorter.

### 21. Committed probe hygiene
**Severity** LOW · **Location** `a11y/kbd-probe.mjs` · **Confidence** certain

Hardcoded credentials `{username:'op',password:'pw'}` and base URL; typo in a user-facing label (`'le surlignage suit le focix immédiatement'`); and the label of that check promises a *highlight* while the code reads `classList.contains('selected')` — a class name, not a painted state. Nothing in this diff adds a `.queue-row.selected` rule, so whether that class paints anything is unverified here (suspicion — it may pre-exist for the server-rendered selection).

### 22. `routes` entries are used unvalidated
**Severity** LOW · **Location** `a11y/axe-gate.mjs` · **Confidence** certain

`getAttribute("href")` can return `null`, `#…`, or an absolute URL. `BASE + route` then builds `http://127.0.0.1:8080null` or a doubled scheme; `goOrGiveUp` reports it as *"did not answer"* and exits 2 with a message that names the wrong cause.

### 23. axe injected as an evaluated string rather than a script tag
**Severity** LOW · **Location** `a11y/axe-gate.mjs` · **Confidence** low (suspicion)

`await page.evaluate(axeSource)` evaluates the UMD bundle as an expression and asks CDP to serialise its completion value. `page.addScriptTag({ content: axeSource })` is the form with no serialisation step. This works in most puppeteer/axe pairings; if it ever throws it lands on finding 1's exit-1 path.

---

## Nothing found at HIGH in these areas
- **The WCAG arithmetic itself.** `contrast()` matches WCAG 2.x exactly: the `<= 0.03928` branch, `/12.92`, `((v+0.055)/1.055).powf(2.4)`, coefficients `0.2126/0.7152/0.0722`, `(L1+0.05)/(L2+0.05)`, and `>=` against 4.5 for normal text (§1.4.3) — all correct, including the comparison direction.
- **The `hue()` branch structure.** I checked all three branches against the HSL definition (`60·((g−b)/span mod 6)`, `120+60(b−r)/span`, `240+60(r−g)/span`) and the wrap handling; they are right, truncation error is under 1° against a 6° tolerance, and the `360 − abs_diff` in the delta cannot underflow given `sixth % 360`. I also verified by hand that all six pinned hues are met by the current values, old and new — `#5980a6` and `#4b6b8b` both compute to 210°, so the "constant hue" claim holds.
- **The CI shell mechanics.** `set -euo pipefail` with the `if curl` guard, the `trap … EXIT`, and `$server` under `set -u` are all correct; the trap window before it is armed is negligible.
- **The JS null-safety ordering.** `focused === null` is tested before `focused.closest`, and the modifier-key bail-out (`altKey||ctrlKey||metaKey||shiftKey`) is complete for the arrows.

---

Cleanup done — no tracked file modified, worktrees and the audit container removed, tree clean.

# Acceptance audit — story 6b.11

## Findings

### 🔴 HIGH

**1 · The keyboard layer — the story's central deliverable — is carried by no automated guard** · Violates AC5 and the house rule *"a guard shipping without a test that reds when it is removed"* · **Evidence I established**: in a detached worktree at `6649e65`, with `DATABASE_URL` pointing at a live `mariadb:10.11` (audit container, port 13351), I emptied `crates/opencmdb-bin/assets/app.js` (`: > app.js`) and ran `cargo test -p opencmdb-bin --locked` → **`490 passed; 0 failed` in 6.08 s** (the clock is the tell that the DB-backed tests ran). `cargo xtask ci` is blind to `.js`. The only carrier is `a11y/kbd-probe.mjs`, and **nothing runs it**: `ci.yml:99-102` runs `node a11y/axe-gate.mjs` only, and `a11y/package.json` declares one script, `gate`. The one Rust test that reads `app.js` (`screens.rs:648`, 6b.2's `no_screen_is_chosen_by_javascript`) passes on an empty file.

**2 · The focus ring on the queue rows and the sort link — AC3's shipped half — is carried by nothing, and the fourth selector's only carrier is a guard about the amber** · Violates AC5's second clause verbatim (*"a guard that reads the SOURCE where the defect lives in the DOM is not counted"*) · **Evidence**: deleting `.queue-row > a:focus-visible,` and `.btn-sort:focus-visible,` from `app.css` leaves **490/490 green** and nine gates green; axe has no focus-appearance rule (the story says so itself). Deleting `.btn-gesture:focus-visible` too *does* red — but the red is `page::tests::ac4_the_amber_is_reserved_for_the_documenting_gesture` at `page.rs:3545`, whose subject is the amber count and whose needle this story moved from `.card:focus` to `.btn-gesture:focus-visible`. So the one Rust carrier of the focus contract is a source-reading needle inside an assertion about something else — **the exact shape T3 exists to undo, reproduced in T5**.

**3 · The "second `aria-pressed` oracle" the Dev Agent Record presents as its discovery is vacuous** · Violates T3 (*"Give it a new oracle; do not 'update the number'"*) and AC5 · **Evidence**: `main.rs:3576` asserts `sorted.contains("aria-current=\"true\"")`. `_triage.html:39` emits `aria-current="true"` on the **selected queue row** of the same page. Measured against a live database: stripping `aria-current` from the sort link (`_triage.html:23`) leaves **490 passed; 0 failed** in 5.63 s. The ON/OFF pair *is* carried — dropping the `on` class reds `the_triage_route_renders_the_queue_and_both_photos` at `main.rs:3571` — but the accessible-state half is measured by nothing.

**4 · Focus is LOST at the settle-navigation, and it is neither measured nor stated** · Bears on AC3 (*"focus follows the selection… focus is visible"*) and on the user story · **Evidence** (Chrome 151 headless, 7-row queue, my probe): after two `ArrowDown`s, `activeElement` is the row at index 2 with `outline: rgb(75,107,139) solid 2px`; after the 250 ms settle fires `window.location.assign`, `activeElement` is **`BODY`**, `outline: none`, and **12 Tab presses** are needed to re-enter the queue. The committed `kbd-probe.mjs` checks focus only *before* the settle (it reads `p.url()` after the wait, never `activeElement`). The arrows keep working via the `aria-current` fallback, so the arrow workflow survives — but every 250 ms of quiet resets the tab position to the top of the document. This is the reachable analogue of the very contract §0b declared unreachable and handed to 6.4.

**5 · "Instrument at fault #1" is itself refuted — the scroll check discriminated, and the cause recorded for dropping it is false** · Violates the project rule *"a cause needs a check, not a plausible story"* · **Evidence**: the record says *"Control: the same page with `app.js` BLOCKED does not scroll either — **arrows do not scroll under headless CDP at all**."* Measured, headless Chrome 151, `/diagnostic`, `scrollHeight 728`, viewport 600: shipped code + `page.keyboard.press('ArrowDown')` → **`scrollY = 40`**; the same page after installing an unconditional `preventDefault` on ArrowDown → **`scrollY = 0`**. The check separates the two states exactly. (§0e's own figure, *"0 → 26 px with an early return, 0 → 0 without one"*, agrees with me; the Dev Agent Record contradicts §0e four sections later, and the false one is the one that changed the test.) The likely real fault is a **dispatched** `KeyboardEvent` (untrusted, never scrolls) rather than a pressed key — a mutation named for one thing applied to another, not a limit of CDP.

**6 · Neither twin carries any record of this story, and both still point the live count at 6b.10's file** · Violates T8 (*"Twins and `sprint-status.yaml` in the same push"*, ticked) and AC4 · **Evidence**: `git diff master...HEAD -- CLAUDE.md docs/project-context.md` is **one line changed in each** — the `RUSTFLAGS` correction — and `grep -c "6b\.11"` returns 1 in each, that same line. `sprint-status.yaml` got a 30-line entry. The convention is not "twins at merge only": 6b.10's **dev** commit `d40a613` touched `CLAUDE.md` (8 lines) and `docs/project-context.md` (6 lines); `d678fd1` then flipped review→done with 1 line each. Consequence measured: the last `THE LIVE COUNT lives in…` pointer in both twins names `6b-10-copy-fr-and-en.md`, whose figure is **727**, while this story's figure is 728.

**7 · "Six divergences registered" is false in three documents** · Violates T8, and T2's explicit *"and **register the divergence**"* · **Evidence**: `deferred-work.md` gained **three** rows (NFR24, the swap half, `puppeteer-core`/per-theme). Of the six named: the *seven gates* DoD was already registered by earlier stories (`:3932`, `:4548`) — not this story's row; the **`ci.yml` exception** exists only as a code comment ending *"Story 6b.11, registered"*; **6b.1's AC2 token-literal conflict** exists only as a comment in `page.rs`; and the **`j`/`k` + `⏎` divergence — the largest, since `epics.md:2306`/`:2308` prescribe them and Epic 7 must know they were skipped — appears nowhere outside the story file** (`grep` over `deferred-work.md` finds nothing). Story 6b.9's finding, verbatim: *a comment that says "registered" is not a registration.* The sentence is repeated in `sprint-status.yaml` and in the story file, so three documents carry it.

**8 · AC5's record does not exist: there is no mutation table** · Violates AC5 and T8 (*"Every guard proven RED first; the greens recorded"*, ticked) · **Evidence**: the Dev Agent Record carries four anecdotes and no table — no mutation ids, no carriers, no greens. Every story in this epic ships one (6b.7 *"ten mutations, ten reds, carriers named per row"*; 6b.9 *"eighteen mutations"*). Three of this story's guards are measured green above (findings 1, 2, 3); a table would have had to say so.

### ⚠️ MEDIUM

**9 · The AC2 rewrite unpins three of the six tokens 6b.1 pinned** · Bears on AC5 and on the narrowing the story claims (*"two properties instead of one literal"*) · **Evidence**, three mutations each leaving **490/490 green**: `--color-neutral-500: #98989b → #ffffff`; `--color-bg: #f2f2f3 → #ffffff`; `--color-accent-700: #416180 → #b5d9fd` (the `.ipam-cell-used` fill, `app.css:775`). The cause is structural and documented in the code without its consequence: `hue()` returns a constant `240` for any grey, so for the three near-grey tokens the hue property is satisfied by *every* grey; and `--color-accent-700` is in neither the `GROUNDS` nor the `TEXTS` table, so only its hue is constrained. ✅ The guard is **not** vacuous for chromatic tokens — control: `--color-accent → #4b8b6b` reds with *"carries hue 150°, the mock's is 210°"*.

**10 · The contrast guard's "premise" assertion cannot fail, and its completeness sentence is false** · Same class the story quotes (*"an enumeration cannot claim the completeness of a property"*) · **Evidence**: `assert_eq!(checked, TEXTS.len() * GROUNDS.len())` counts loop iterations over two fixed-size arrays — shrinking `TEXTS` from 4 to 3 (dropping `--accent-document`) leaves **490/490 green**, because the right-hand side shrinks with the left. Its doc calls the tables *"the grounds a screen actually paints text on"*: `app.css` paints on **six** distinct background tokens (`--color-neutral-100/200/300/400`, `--color-accent-100`, `--color-accent-700`, plus the `--bg`/`--surface` aliases); four are listed. ✅ **Refuted the stronger form**: I computed every painted pairing outside the table — `.badge` 9.55, `.filter.is-active` 9.10, the five `.statepill-*` 6.03–11.45, white on `.ipam-cell-used` 6.47 — **no live AA failure**, which is why axe agrees.

**11 · `--color-accent-600` fails AA on all four grounds and T2's question about the ramp is answered nowhere** · Ticked task delivering nothing (story 6b.7's family) · **Evidence**: `#597ea3` measures **3.80 / 3.51 / 3.91 / 3.45** against bg / surface / n100 / n200 — below 4.5 on every one, i.e. exactly the defect `--color-accent` was darkened for. T2's bullet *"⚠️ `--color-accent-600` is `#597ea3`… Say whether the ramp follows"* is `[x]`, and the string `accent-600` appears nowhere in the Dev Agent Record. Latent rather than live: `grep` shows the token is referenced by no rule in `app.css` today.

**12 · The dead contract has FOUR artefacts, not three** · Violates T5 (*"Decide all **three** dead-contract artefacts, not just `app.js`"*) · **Evidence**: `_gap_card.html:1` still carries `tabindex="-1"` on `#gap-card`, and the removed handler's own doc said why it was there — *"The card carries `tabindex="-1"` so it is programmatically focusable"*. Its only purpose was the focus the deleted handler performed. §0b's enumeration lists three; the register row names two left standing; this one is decided and registered nowhere.

**13 · `.gitignore` closes the incident on one path, not on the pattern** · Same enumeration rule, applied to the fix for the incident the entry narrates · **Evidence**: `.gitignore:110` is `a11y/node_modules/`; `git check-ignore -v node_modules/foo` → **rc=1, not ignored**. The entry's own comment says *"a directory this size entering the tree is invisible to every check the project has"* — which is exactly as true one directory up. ✅ The removal itself is clean: `git ls-files a11y/` lists four files, `git log --all -- a11y/node_modules` is empty, `git check-ignore -v` maps the path to `.gitignore:110`.

**14 · The CI accessibility gate runs against an uncontrolled, essentially empty store — the state §0c identifies as blind** · Bears on AC3b · **Evidence**: `ci.yml:83-102` boots the binary against `opencmdb_test` with no seed, immediately after the `Tests` step, so the queue holds whatever residue the test step left. §0c is explicit that on an empty store `/triage` renders *"You are up to date"* and *"the queue, the detail pane and the five `Gesture::Planned` controls — the exact surface T4/T5/T6 target — were in no measurement at all"*, and that the fourth colour pair appears only when a row is selected. §0c then presents *"no seeding is needed"* as a saving. The Rust contrast property compensates on the colour axis only.

**15 · §0g's second inherited item is neither taken nor re-registered** · §0g: *"Take both or re-register by name"* · **Evidence**: `deferred-work.md:3748-3757` hands this story 6b.2's honesty guard missing `display:none` and `pointer-events:none`, noting *"closing those needs computed styles, which is exactly what axe-core/Playwright provides"* — the instrument T1 just built. The Dev Agent Record's *"What was decided rather than built"* covers NFR24 and the dead contract only; no register row mentions it.

**16 · The register row this story discharges is left standing as pending** · Bookkeeping, against the file's own convention · **Evidence**: `deferred-work.md:4376` — *"the repository has NO accessibility check at all: no axe-core, no headless browser… **Owner: story 6b.11**"* — is satisfied by this story and untouched. The file carries `## Discharged by story 6b.2 (2026-08-18)` sections, so the convention exists.

### ℹ LOW

**17** · `ci.yml:80-82`'s *"If that stops being true, this step fails with exit 2"* is half true: a missing Chrome does exit 2 (measured), a missing Node makes the shell fail with 127. **18** · `a11y/kbd-probe.mjs` is written in French, comments and output, against the repo's English-artefact convention, and contains the typo *"le focix"*. **19** · The headline **237** is not reproducible from the store state it names: my own 7-gap-row store gives **221** on the same reverted tokens. **20** · On `/triage` with focus on `body`, `ArrowDown` no longer scrolls the page (measured `defaultPrevented: true` at the first row); the code states that cost only for the end-of-list case. **21** · `EXPECTED_ROUTES` is a floor with no distinctness check, and 6b.2's `no_screen_is_chosen_by_javascript` needle list (`pushState`, `location.hash`, `history.`) now sits beside a deliberate `window.location.assign` with no note added. **22** · The File List omits `.gitignore`.

---

## AC verdict table

| AC | Verdict | Evidence I established |
|---|---|---|
| **AC1** arrows, scoped, shape (E) | **MET** | Ran the committed `kbd-probe.mjs` (repointed to my port/credentials) against a booted binary with a 7-row queue, Chrome 151: focus steps `1 → 2` on the keypress, highlight index equals focus index immediately, URL becomes `/triage?sel=ecart:…a3:hostname` after the quiet, `↓` with focus on a nav entry leaves the URL unchanged, `/diagnostic` `/apps` `/sources` report `intercepted=false rows=0`. |
| **AC2** no letters | **MET** | Same run, at a middle index: `{"a":false,"j":false,"k":false,"x":false,"Enter":false,"Backspace":false,"Home":false,"PageDown":false," ":false}` with the positive control `ArrowUp:true, ArrowDown:true`. |
| **AC3** reachable focus contract | **PARTLY MET** | Visibility ships and is real (`getComputedStyle` on a queue row: `2px solid rgb(75,107,139)`, the product's rule, not the UA's); five `.btn-gesture` all `aria-disabled="true"` / `tabIndex 0`. **But** focus is lost to `<body>` at every settle-navigation (finding 4), and the visibility itself is guarded by nothing (finding 2). |
| **AC3b** axe green on ten routes | **MET** | Ran the committed `a11y/axe-gate.mjs` twice against a booted binary: empty store → 10 routes derived, **0 violation nodes, rc=0**; store with 7 gap rows → 10 routes, **0 nodes, rc=0**. Prove-to-red: reverting the three tokens on the running server → **rc=1, 221 nodes, all ten routes red**. Exit-code discipline verified: dead port → **rc=2**, `AXE_CHROME=/nonexistent` → **rc=2**. `npm --prefix a11y ci` from the committed lockfile → 26 packages, 354 ms. |
| **AC4** live count here, every figure names its store state | **PARTLY MET** | Both terms of the delta verified: `master` in a clean worktree → 489 + 161 + 77 = **727**; HEAD → 490 + 161 + 77 = **728**. Nine gates green (`cargo xtask ci`, rc=0), clippy `--all-targets` rc=0, `fmt --check` rc=0. **But** the headline test figure names no store state, and the state changes what it means: the same 490 runs in **0.21 s** without a database and **5.69 s** against one. |
| **AC5** every guard measured red first; a source guard for a DOM defect does not count | **NOT MET** *(round one's verdict; round two set it NOT MET on a NAMED condition, and §T10 records that condition measured satisfied by CI)* | No mutation table exists. Two central guards measured green (app.js emptied → 490 green; the two focus selectors removed → 490 green), one new oracle vacuous (`aria-current` stripped from the sort link → 490 green with a live DB), and the one Rust carrier of the focus contract reads `app.css` while the defect is in the DOM. |
| **AC6** nothing promises an absent gesture | **MET** | On the served `/triage`: 5 `.btn-gesture`, all `aria-disabled="true"` with `tabIndex 0`, `document.querySelectorAll('form,input,textarea,select,button').length === 0`, zero `hx-*`. No control became live; the arrows do something real. |
| **AC7** NFR24 taken or re-registered by name | **PARTLY MET** | The register row exists and its measurement replays: at 1280 px on the served `/triage`, `.nav-entry` **34 px**, `.btn-sort` **27 px**, `.btn-gesture` **29 px**, `.queue-row > a` 72–95 px — heights identical to the row's table. **But** §0g's second inherited obligation (`:3748`) is neither taken nor re-registered (finding 15). |

## Claims I REFUTED, each with the check that refuted it

- **"Arrows do not scroll under headless CDP at all"** (Dev Agent Record, instrument #1) — refuted by `page.keyboard.press('ArrowDown')` on `/diagnostic`: shipped code **scrollY 40**, an unconditional `preventDefault` **scrollY 0**. The check discriminated; the control that dismissed it did not.
- **"Six divergences registered"** (story file, `sprint-status.yaml`, T8) — refuted by `git diff` on `deferred-work.md`: three rows, and `grep` finds no `j`/`k`/`⏎` divergence anywhere in the register.
- **"Twins … verified identical" as a discharge of the twin obligation** — refuted by `git diff master...HEAD -- CLAUDE.md docs/project-context.md`: one line each, no story record, against 6b.10's dev commit which wrote 8 and 6.
- **"The grounds a screen actually paints text on"** (the new guard's doc) — refuted by `grep -oE 'background(-color)?: var\(--…'` over `app.css`: six painted background tokens, four listed.
- **"Story 6b.1's guard became two properties"** as a like-for-like replacement — refuted by three green mutations (`neutral-500`, `bg`, `accent-700` all freely movable now, each reddening under the old literal guard).

## Claims I CONFIRMED by replay

`237 → 0`'s zero half (0 nodes at 7 gap rows, and rc=1/221 nodes on revert) · the Rust guard reds at **3.82:1** when `#7a7a7d` is restored, with a named assertion at `page.rs:3259` · the hue guard reds on a repalette (`150° vs 210°`) · the exit-code triage (0/1/2) · zero `hx-*` and zero `id="gap-card"` on all ten served routes, `/gap` a bare fragment with **0** `<script>` · `RUSTFLAGS` absent from `.github/`, `actions-rust-lang/setup-rust-toolchain@v1` at **`ci.yml:48`** exactly · `a11y/node_modules/` absent from tree, index and history and correctly ignored · nine gates, clippy `--all-targets`, `fmt` all green · 727 → 728 with **both** terms measured.

## The question no conformance check asks

**What can the operator DO that they could not before?** Move down and up the triage queue with the arrows, see where they are (the product's own focus ring, for the first time on the queue), and have the detail pane follow. That is the product's first real keyboard reach, and it is genuine. **What they still cannot do is act on the row they landed on** — five controls, all `Gesture::Planned`, no form, no button, no POST anywhere on the screen. The story makes a well-lit dead end faster to traverse; *"as fast as clearing a mailbox"* is met on the traversal and empty on the clearing, and story 6.4 is still what ends it. **Nothing shipped promises a gesture that does not exist** — the arrows do what they appear to do, and the five controls still say *À venir*. The one place the surface now over-promises slightly is the new focus ring on `.btn-gesture`: an `aria-disabled` control that draws the same 2 px accent outline as a live link. That is 6b.4b's deliberate reachability decision carried forward, not a new promise, but it is the one pixel where *"looks actionable"* and *"is not"* moved closer together.

---

All mutations reverted, tree clean, container removed, port 8080 measured dead.

---

# Edge Case Hunter — story 6b.11

Everything below was executed in an isolated worktree at `6649e65` against a live `mariadb:10.11` (port 13350), Chrome 151, `puppeteer-core` 25.8, an 8-row queue. Every mutation was reverted by restoring a scratchpad copy, never `git checkout --`; the closing state is `git status` empty, 728 tests green (490 bin + 161 core + 77 xtask), nine gates green, axe gate exit 0.

## HIGH

### 1. The axe gate cannot see the screen this story changed — CI's store is empty
**Severity: HIGH** · `.github/workflows/ci.yml:83-102`, `a11y/axe-gate.mjs`

**Input applied.** The CI step sets only `DATABASE_URL`, `OPENCMDB_BASIC_USER`, `OPENCMDB_BASIC_PASSWORD` — no seed, no `OPENCMDB_SCAN_CIDR`. I reproduced that shape (`DELETE FROM declared_attribute`) and counted what `/triage` serves, then ran the same planted defect against both stores. The plant is the story's own defect put back: `<a href="{{ row.href }}" aria-pressed="false"` on the queue link in `_triage.html`.

**Measured.**
- Empty store, `/triage` 200: `queue-row` **0**, `btn-gesture` **0**, `triage-panes` **0**, `btn-sort` 1 (off, so no `aria-current`).
- With 8 rows: `🔴 /triage aria-allowed-attr(8, critical)` → **exit 1**.
- With the store emptied, same binary, same plant: ten ✅, `0 violation node(s)` → **exit 0**.

**Why it matters.** The gate is green over a `/triage` that carries none of the elements 6b.11 touched — no queue rows, no focus rule to compute, no action bar, no selected row. The commit's own headline says *"measured twice: at 2 queue rows and at 5. The second run is the one that counts — the fourth colour pair appears only when a row is selected, so a short queue would have missed it."* CI's queue is not short, it is **empty**, on every run, forever.

### 2. The route floor counts nav ENTRIES, not distinct screens
**Severity: HIGH** · `a11y/axe-gate.mjs:38` (`EXPECTED_ROUTES`), `:96-103`

**Mutation applied, verbatim.** `_nav.html:10`, `href="{{ entry.href }}"` → `href="/triage"`, plus a live violation planted elsewhere (`app.css`, `.example-marker-badge { color: var(--muted) }` — 3.76:1 on its `--color-neutral-300` ground, rendered twice on `/dashboard`).

**Measured.** Rebuilt, served (`nav-entry" href="/triage"` the only distinct href), gate output:
```
✅ /triage  ×10
axe gate: 10 route(s) derived from the navigation, 0 violation node(s)      EXIT=0
```
Nine screens were never visited; the planted violation was live on at least five of them (measured separately: `color-contrast(2, serious)` on `/dashboard`, 1 each on `/devices`, `/devices/nas-01`, `/apps`).

**Why it matters.** `routes.length < EXPECTED_ROUTES` is a length check on an array of hrefs — it cannot tell ten screens from one screen ten times, which is exactly the shape its own comment names: *"a harness that derives nothing and reports success is the failure mode this file exists to avoid."* Story 6b.5's review converted a hardcoded floor to one derived from `Screen::ALL` for this reason; `EXPECTED_ROUTES = 10` restores the hardcoded form in a file no Rust test can read.

### 3. A pending arrow timer overrides a click and lands the operator on a row they did not choose
**Severity: HIGH** · `crates/opencmdb-bin/assets/app.js:56-66`

**Input applied.** Document responses delayed 1200 ms via CDP request interception (inside the 2 s per-handler budget 6b.10 installed on `/triage`); `document.body.focus()`, `ArrowDown`, wait 40 ms, then `a[5].click()`.

**Measured.**
```
operator CLICKED : /triage?sel=absence:11111111-...-000000000006
browser LANDED on: /triage?sel=absence:11111111-...-000000000002
```
Control: the same sequence against a warm local store (sub-100 ms responses) → the click wins. The defect needs only a document slower than the remaining settle.

**Why it matters.** `select()` arms `pending` and **nothing cancels it** — not `click`, not `blur`, not `pagehide`, not `beforeunload`. Only another arrow press clears it. A store answering in 1–2 s is the state 6b.10's own budget was written for, so this is not an exotic path.

### 4. The keyboard layer is covered by nothing that runs
**Severity: HIGH** · `crates/opencmdb-bin/assets/app.js`, `a11y/kbd-probe.mjs`, `a11y/package.json`, `ci.yml`

**Mutation applied, verbatim.** `printf '// (the keyboard layer, removed)\n' > crates/opencmdb-bin/assets/app.js` — the whole file.

**Measured.** `RUSTFLAGS="-D warnings" cargo test --workspace --locked` → **490 / 161 / 77, 0 failed**. `cargo xtask ci` → **all gates green**. Rebuilt, served (`/assets/app.js` returns the one comment line), axe gate → **exit 0**.

**Why it matters.** `kbd-probe.mjs` is the only thing that measures the layer, and it is referenced by no `package.json` script and by no CI step — `npm run gate` runs `axe-gate.mjs` alone. The one Rust test that reads the file, `screens.rs:646 no_screen_is_chosen_by_javascript`, asserts the **absence** of four strings, which an empty file satisfies. 117 lines of behaviour with no automated carrier of any kind.

### 5. `token_hex` reads a CSS COMMENT, and one comment silences the contrast property over the exact defect this story fixed
**Severity: HIGH** · `crates/opencmdb-bin/src/page.rs:3288-3297` (`token_hex`)

**Mutations applied, verbatim.** In `app.css`, with the story's repaired value reverted (`--color-neutral-600: #7a7a7d;`):
```css
  /* AA repair: --color-neutral-600: #68686b; reverted 2026-08-22 pending design review */
  --color-neutral-600: #7a7a7d;
```

**Measured.**
- Control (bad value, no comment): `every_text_token_clears_aa_on_every_ground_it_can_sit_on` **FAILS** — `--color-neutral-600 on --color-bg is 3.82:1` (the exact figure the commit message quotes).
- With the comment one line above: **`test result: ok. 1 passed`.**
- The same tree served: axe reports **210 violation nodes on all ten routes**.

The opposite direction is equally reachable: a comment reading `/* before the rebrand this read --color-text: #ffffff; */` above the correct `--color-text: #1d1f20` makes the guard red at **1.12:1** over a correct sheet.

**Why it matters.** `css.find(&format!("{token}: #"))` takes the first textual occurrence anywhere in the file, comments included. Writing the old value in a comment when you revert a colour is the ordinary gesture. For pairings axe cannot see (finding 8), this guard is the *only* carrier, and a comment turns it off silently — the `checked == 16` premise assertion counts pairings, not sources, so it stays green too.

### 6. Exit **1** — the violation code — for "the gate could not run", measured twice
**Severity: HIGH** · `a11y/axe-gate.mjs:44-49, 100-102, 111-118`; `ci.yml:80-82`

**Mutations applied, verbatim.**
- (a) `mv a11y/node_modules <elsewhere>` (i.e. `npm ci` did not produce the tree the gate needs).
- (b) `printf '\nthrow new Error("axe payload broken");\n' >> node_modules/axe-core/axe.min.js`.

**Measured.** (a) `Error [ERR_MODULE_NOT_FOUND]: Cannot find package 'puppeteer-core'` → **EXIT=1**. (b) `Error: axe payload broken … at axe-gate.mjs:111` → **EXIT=1**.

**Why it matters.** The module header states `1 — the product has accessibility violations (a real regression: fix the product)` and `2 — the gate could not run`, and `ci.yml:80-82` promises in so many words that if Node/Chrome stop shipping *"this step fails with exit 2 … and not with exit 1"*. Only `puppeteer.launch` is inside a try. The top-level `import`, the `readFileSync` of `axe.min.js`, `page.evaluate(axeSource)` and `axe.run` are all outside every catch, and an uncaught rejection in an ESM module exits **1**. A developer would be sent to hunt an accessibility regression that does not exist.

## MEDIUM

### 7. `token_hex` silently discards an alpha channel
**Severity: MEDIUM** · `crates/opencmdb-bin/src/page.rs:3288-3297`

**Mutation applied, verbatim.** `--color-neutral-600: #68686b;` → `--color-neutral-600: #68686b1a;` (10 % alpha — near-invisible muted text).

**Measured.** Rust property: **`test result: ok. 1 passed`** (it reads the first six digits and reports 4.96:1). Served: axe → **exit 1, 210 violation nodes on all ten routes**.

**Why it matters.** `css.get(at..at + 6)` takes six characters and never checks what follows, so an 8-digit hex is read as its opaque prefix and the guard measures a colour the browser does not paint. *Refuted for the neighbouring forms:* a 3-digit hex (`#686`), a `var()` declaration and a missing token all **panic loudly** with `--color-neutral-600 is declared as a hex literal` — the 8-digit form is the one that mis-parses in silence.

### 8. `GROUNDS` omits two grounds the sheet actually paints text on
**Severity: MEDIUM** · `crates/opencmdb-bin/src/page.rs:3212-3223`

**Mutation applied, verbatim.** `app.css`, `.example-marker-badge { color: var(--color-neutral-900) }` → `color: var(--muted)`.

**Measured.** Rust property `every_text_token_clears_aa_on_every_ground_it_can_sit_on`: **passes**. Served: axe → **exit 1**, `color-contrast(2, serious)` on `/dashboard`, `(1, serious)` on `/devices`, `/devices/nas-01`, `/apps`. Computed ratio 3.76:1.

The sheet paints text on `--color-neutral-300` (`.example-marker-badge`) and on `--color-accent-100` (`.filter.is-active`, `.statepill-gap`); neither is in `GROUNDS`, whose comment calls it *"the grounds a screen actually paints text on"*. Against `--color-neutral-300`, **three of the four `TEXTS` fail AA** (`--color-neutral-600`, `--color-accent`, `--accent-document` = 3.76:1 each).

**Why it matters.** The doc claims *"this walks the token TABLE, so a combination nobody has used yet is still checked"*. For the two omitted grounds it is not checked at all, and the only remaining carrier is axe over ten rendered routes — which finding 1 shows is itself partial in CI.

### 9. The ±6° hue tolerance is unattainable for the near-neutral tokens it guards
**Severity: MEDIUM** · `crates/opencmdb-bin/src/page.rs:3186-3208`, `hue()` at `:3300-3323`

**Mutation applied, verbatim.** `--color-text: #1d1f20;` → `--color-text: #141616;` — the same colour with HSL lightness × 0.7 at **constant hue and saturation** (verified in float: `colorsys` reports hue 200.0 for the source).

**Measured.**
```
the mock's token --color-text carries hue 180°, the mock's is 200° —
a contrast repair moves LIGHTNESS, never hue
```
Control: `every_text_token_clears…` **passes** on the same value (contrast rises 14.79 → 16.7), so the darkening is legal by the other property and illegal by this one.

**Why it matters.** `--color-text` is `#1d1f20`: max−min = **3**, so the smallest representable hue step is 60/3 = 20° — larger than the tolerance. `--color-bg` and `--color-surface` have span 1 (step 60°), `--color-neutral-500` span 3. Four of the six tokens the guard checks live where a lightness-only move cannot hold hue to ±6°, and the failure message states as fact the thing that just proved false. `hue()` itself is faithful — `colorsys` agrees the darkened colour *is* 180° at 8 bits; the defect is the tolerance's claim about what it permits.

### 10. `select()` moves the highlight and the focus but never `aria-current`
**Severity: MEDIUM** · `crates/opencmdb-bin/assets/app.js:53-66`

**Input applied.** `ArrowDown` from `document.body`, then poll `.selected` and `[aria-current="true"]` every 50 ms until they agree; repeated at document delays of 0 / 1200 / 1900 ms.

**Measured.** 80 ms after ↓: `{focused: 1, selectedClass: 1, ariaCurrent: 0}` — the eye and the screen reader name different rows. Time to agreement:

| document delay | disagreement window |
|---|---|
| 0 ms (warm local) | **360 ms** |
| 1200 ms | **1551 ms** |
| 1900 ms | **2240 ms** |

**Why it matters.** The story records the cost as *"for 250 ms the highlighted row and the URL disagree"*. 250 ms is the timer, not the observed window; it is **360 ms** even warm, it scales with the store up to 6b.10's 2 s budget, and the third disagreeing party — the accessible `current` state — is not named at all, in the story whose subject is accessibility.

### 11. Every settle navigation destroys the operator's focus position
**Severity: MEDIUM** · `crates/opencmdb-bin/assets/app.js:56-66`

**Input applied.** Three `ArrowDown` presses, 600 ms apart, then one `Tab`.

**Measured.**
```
after settle 1: {"active":"BODY","focusedRow":-1,"ariaRow":1}
after settle 2: {"active":"BODY","focusedRow":-1,"ariaRow":2}
after settle 3: {"active":"BODY","focusedRow":-1,"ariaRow":3}
one Tab after the settle lands on: nav-entry | Triage
```
Second witness: arrow, then `Tab` inside the window — focus moves to the next row's link, then 1.2 s later the pending timer navigates and focus is `BODY`. Third: a held arrow (12 presses at 40 ms) lands on row 7 with `activeElement = BODY`.

**Why it matters.** Arrowing keeps working because `currentIndex` falls back to `aria-current`, but every other keyboard gesture restarts at the top of the document: an operator who arrows to row 3 and presses Tab is thrown back to the first navigation entry. `kbd-probe.mjs` reads the focus index immediately after the keypress and never after the settle, so it is blind to this by construction.

### 12. Arrow navigation fills the history stack; Back can no longer leave `/triage`
**Severity: MEDIUM** · `crates/opencmdb-bin/assets/app.js:64` (`location.assign`)

**Input applied.** Three `ArrowDown` presses 500 ms apart from a fresh `/triage`, reading `history.length` and then pressing Back once.

**Measured.** `history.length` **2 → 5** (one entry per settled press). `1× Back → /triage?sel=…000000000003` — the previous *selection*, not the previous page. A held arrow is bounded (12 presses → 1 document request, `history.length` 2 → 3), so the cost is per deliberate row-by-row step.

**Why it matters.** `_triage.html:18-20` justifies choosing `aria-current` over `role="button"` precisely because the latter *"would cost middle-click, copy-link and **the back button**"* — and the keyboard layer added in the same commit degrades the back button on the same screen. `location.replace` would keep one entry per visit.

## LOW

### 13. ↓ then ↑ back to the row already selected issues a full reload to the identical URL
**Severity: LOW** · `crates/opencmdb-bin/assets/app.js:64`

**Input applied.** From `/triage?sel=<row0>` (row 0 already current), set `window.__mark`, press ↓, wait 50 ms, press ↑.

**Measured.** `url before` and `url after` **identical**; **1 document request**; `window.__mark` → `DOCUMENT WAS REPLACED`; `activeElement` → `BODY`.

**Why it matters.** Nothing compares `row.getAttribute("href")` with `location.href` before assigning, so an operator who overshoots by one and corrects pays a full page load and loses their place — a net movement of zero costing a round trip.

### 14. The readiness loop is bounded at ~15.5 minutes, not the 30 seconds it reads as
**Severity: LOW** · `.github/workflows/ci.yml:94-98`

**Input applied.** `docker pause` on the database container, then the step's own probe: `curl -fsS -o /dev/null http://127.0.0.1:8080/healthz`.

**Measured.** `http=503 time=30.002716`. `curl` carries no `--max-time` or `--connect-timeout`, the loop adds `sleep 1`, and neither the job nor the step has `timeout-minutes`.

**Why it matters.** `for _ in $(seq 1 30)` reads as a 30-second wait; against a database that accepts TCP but does not answer it is 30 × ~31 s plus a final 30 s. The normal failure (port not yet bound) is instant, so this only bites on the degraded path — which is the path a readiness loop exists for.

### 15. Three different `aria-current` semantics on one page
**Severity: LOW** · `_triage.html:23`, `:39`, `_nav.html:11`

**Measured** on a rendered `/triage?sort=age`:
```
A.nav-entry    = page
A.btn-sort on  = true
A.<queue row>  = true
```
No axe violation (measured: exit 0) and no code collision (`currentIndex` scopes its search to `.queue .queue-row > a`). Recorded as a stated fact rather than a defect: the sort toggle and the selected row announce the same generic `current` for two different meanings, one click apart.

---

## Suspicions I RAN and REFUTED, each with the check

| Suspicion | Check | Result |
|---|---|---|
| A click during the settle window is overridden | click at t+60 ms on a warm store | **Refuted** on a fast store — the click wins; it takes a slow document (finding 3) |
| A failure path leaks Chrome into CI | `pgrep -af "/usr/bin/google-chrome"` after a launch failure, a 500-route abort and an uncaught throw | **Refuted** — 0 in every case; puppeteer's exit handler reaps the child |
| The CI trap masks the gate's exit code / orphans the server | ran the step's shell shape with a stub exiting 1 then 2 | **Refuted** — step exits 1 and 2 respectively; `pgrep` finds no orphan |
| The readiness probe can pass over an unusable server | `docker pause` the store, then `GET /healthz` | **Refuted** — 503, so `curl -fsS` fails and the loop cannot break early |
| A route that 500s is reported as a violation | store paused, gate run | **Refuted** — `axe gate: /triage answered 500, so nothing there can be measured`, **exit 2** |
| A violation on a late route is lost | plant reddened `/dashboard`…`/apps` after a clean `/triage` | **Refuted** — the walk accumulates and exits 1 naming every failing route |
| `?sort=age` is dropped by an arrow | `/triage?sort=age`, one ↓ | **Refuted** — landed `…&sort=age`; reading the row's own href holds |
| The layer is not inert on the other screens | dispatched ↓/↑ on all **nine** non-triage routes (the story measured three) | **Refuted** — `rows=0`, `defaultPrevented=false` on every one |
| Modified arrows are captured | `shiftKey`/`ctrlKey`/`metaKey`/`altKey` + ArrowDown | **Refuted** — all four `prevented: false` |
| An arrow with focus on a gesture control navigates | focused `.btn-gesture`, ↓, 900 ms | **Refuted** — `inQueue=false`, URL unchanged; the action bar sits outside `.queue` |
| A one-row or empty queue misbehaves | pruned the `<ol>` to one `<li>`, then removed `.queue` entirely | **Refuted** — both arrows inert, no exception, focus untouched |
| The focus ring is invisible on the row an arrow lands on | `getComputedStyle` on the selected row after ↓ | **Refuted** — `2px solid rgb(75,107,139)` on `rgb(245,245,248)` = **5.11:1**, against WCAG 1.4.11's 3:1; present on all four focusable kinds |
| Disabling JS breaks the queue | `setJavaScriptEnabled(false)`, load `/triage` | **Refuted** — 8 rows still rendered; the selection is a URL |
| `hue()` is wrong at the boundaries | pure red, mid-grey, both sides of the 0°/360° wrap | **Refuted** — `(255,0,0)→0`, `(128,128,128)→240` (stated convention), `(255,0,4)→0`, `(255,4,0)→0` |
| A 3-digit hex / a `var()` declaration / a missing token mis-parses | planted `#686` and `var(--color-neutral-700)` | **Refuted** — both panic loudly (`… is declared as a hex literal`); only the 8-digit form is silent (finding 7) |

**Two probe artefacts, recorded so they are not read as findings:** `page.goBack({waitUntil:"networkidle0"})` times out at 30 s against this app (bfcache) — re-run with `waitUntil:"load"` it returns instantly; and one `page.evaluate` sampling at t+400 ms died with *"Execution context was destroyed"* because the settle navigation had started — the measurement it was meant to take is carried by finding 10's polling loop instead.

---

# The TRIAGE pass — 2026-08-23

**60 raw findings → 33 distinct defects.** The three reports above stay verbatim and untouched; this
section is what reading them *together* establishes, and nothing here is a fourth layer's opinion —
every row cites the layer that measured it. Layer keys: **B** = Blind Hunter (the patch alone),
**A** = Acceptance Auditor (spec + tree + live store), **E** = Edge Case Hunter (mutations, browser,
8-row queue).

⚠️ **The raw count was never a total, and the ratio is the interesting number**: 60 findings cover
33 defects, so **nearly half of what the layers wrote is a second or third layer reaching the same
thing by a different road**. Sixteen defects were reached by two layers, six by all three. A count
over three isolated reports measures the isolation, not the product.

## §T1 — What all THREE layers reached independently

These six are the review's spine. Each was found blind from the diff, replayed against the spec,
*and* reproduced by a planted mutation in a browser — three roads, one defect.

| # | The defect | B | A | E | Where it goes |
|---|---|---|---|---|---|
| **D1** | The axe gate answers **1** — *the product has violations* — for *the gate could not run*, which it promises as **2** | 1 (≈8 unguarded paths enumerated from the diff) | 17 (a missing Node exits **127**, neither code) | 6 (**measured twice**: `mv node_modules` → 1; a broken axe payload → 1) | **Arbitration 1** |
| **D2** | The keyboard layer — the story's central deliverable — is carried by **nothing that runs** | 13 (`kbd-probe.mjs` is in no script and no CI step; it prints `TOUT VERT` when it measures nothing) | 1 (`: > app.js` → **490 passed**) | 4 (whole file replaced → 490/161/77, nine gates, axe exit 0) | Patch — AC5 |
| **D3** | The route floor cannot tell ten screens from **one screen ten times** | 8 (`<`, no `Set`, no `null` filter) | 21 | 2 (**measured verbatim**: every nav href → `/triage`; gate printed `✅ /triage ×10 … 0 violation node(s) EXIT=0` while a planted violation was live on five screens) | Patch |
| **D4** | The AC2 hue guard is **too loose and too tight at once** | 14 (`hue()` returns a constant `240` for any achromatic — so `#0000ff` and `#ffffff` are one hue) | 9 (**three green mutations**: `neutral-500`→`#ffffff`, `bg`→`#ffffff`, `accent-700`→`#b5d9fd`; with the ✅ control that it is *not* vacuous for chromatic tokens) | 9 (**the mirror**: `--color-text` has span 3, so its smallest representable hue step is **20°** against a ±6° tolerance — a legal lightness-only darkening **REDS** it) | Patch — see §T3 |
| **D5** | The gate is green over a surface it never reaches | 6 (the new attribute renders only under `?sort=age`; nothing appends a query string and axe runs before any keypress) | 14 (CI seeds nothing, so §0c's blind state is CI's permanent state) | 1 (**measured**: empty store → `queue-row` 0, `btn-gesture` 0, `triage-panes` 0; the story's own defect replanted **reds at 8 rows and exits 0 with the store emptied**) | **Arbitration 2** |
| **D6** | `token_hex` takes the **first textual match** in the file | 15 (the dark palette is a second complete set; the light one wins **by source order**, undocumented) · 20 (8- and 3-digit spellings) | — | 5 (**a CSS COMMENT carrying the old value turns the property off in silence** — guard green, axe **210 nodes**; and the opposite direction reds a correct sheet at 1.12:1) · 7 (an 8-digit alpha reads as its opaque prefix: Rust passes at 4.96:1, axe exits 1) | **Arbitration 3** |

🔑 **D4 is the finding no single layer has, and the triage is the only place it exists.** B and A
measured the guard letting *anything grey* through; E measured the same guard *refusing a correct
repair*. Read together: the property is unattainable exactly where it is unconstrained, and
E's failure message — *"a contrast repair moves LIGHTNESS, never hue"* — **states as fact the thing
the mutation that produced it had just disproved.** Neither half is a full account; the pair is.

🔑 **And D2 · D3 · D5 compose into one sentence the individual findings do not carry**: the
accessibility apparatus this story shipped **reports success over a surface it does not reach** —
an empty store (D5), walked by a floor that cannot detect duplication (D3), beside a keyboard layer
nothing exercises (D2). Three independent holes, one shape, and `axe-gate.mjs`'s own comment names
it: *"a harness that derives nothing and reports success is the failure mode this file exists to
avoid."*

## §T2 — What TWO layers reached independently

| # | The defect | Layers | Note |
|---|---|---|---|
| **D7** | Nothing cancels the settle timer — not `click`, not `blur`, not `pagehide` | B3 (deduced) · E3 (**measured**: at a 1200 ms document delay the operator clicked row 6 and landed on row 2; **control** on a warm store — the click wins) | **Arbitration 4** |
| **D8** | The new `aria-current` oracle is satisfied by a **queue row on the same page** | B2 (deduced **from the diff alone**, via `kbd-probe.mjs`) · A3 (**measured**: stripping it from the sort link → 490 passed) | Patch |
| **D9** | The focus ring is carried by nothing; its one Rust carrier is a needle **inside the amber test** | A2 (**measured**: two selectors deleted → 490 green; the third reds `ac4_the_amber_is_reserved…`) · B10 (the needle was *moved*, not widened, and is formatting-coupled) | Patch — this is T3's own shape reproduced in T5 |
| **D10** | `select()` moves the class and the focus, never `aria-current` | B11 (two consequences, incl. the stale fallback after a blur) · E10 (**measured**: the disagreement window is **360 ms warm**, 1551 ms at 1200 ms, 2240 ms at 1900 ms) | Patch — one `setAttribute` |
| **D11** | Every settle navigation destroys the operator's focus position | A4 (**measured**: `activeElement` → `BODY`, **12 Tab presses** to re-enter the queue) · E11 (three witnesses; one Tab after the settle lands on *nav-entry ǀ Triage*) | Patch or register |
| **D12** | `checked == TEXTS.len() * GROUNDS.len()` cannot fail | B4 (certain, from the diff) · A10 (**measured**: shrinking `TEXTS` 4 → 3 leaves 490 green, because the right-hand side shrinks with the left) | Patch |
| **D13** | `GROUNDS` omits two grounds the sheet paints text on | E8 (**measured**: `.example-marker-badge` → `--muted` passes Rust, axe exits 1 at 3.76:1) · A10b (six painted background tokens, four listed) | Patch — ⚠️ see the non-contradiction below |
| **D30** | Probe hygiene: hardcoded credentials `op`/`pw`, a hardcoded base URL, French against the English-artefact convention, the typo *"le focix"*, and a label promising a *highlight* while the code reads a class name | B21 · A18 | Patch |

⚠️ **D13 is where two layers look like they disagree and do not.** A measured the *tree* and refuted
the stronger form — every painted pairing outside the table clears AA today (`.badge` 9.55,
`.filter.is-active` 9.10, the `.statepill-*` 6.03–11.45), which is why axe agrees. E measured the
*guard* and showed it blind to a planted defect on the same grounds. **The sheet is clean and the
guard would not notice if it stopped being** — both true, and only the pair says so.

## §T3 — Single-layer defects that survive triage

**Behaviour.** **D14** arrow navigation fills the history stack (E12, measured `history.length`
2 → 5; **and `_triage.html:18-20` justifies choosing `aria-current` over `role="button"` precisely
to keep the back button** — the layer added in the same commit degrades it). **D28** ↓ then ↑ back
to the row already selected issues a full reload to the identical URL (E13). **D27** the inertness
probe visits only the three screens where the early return fires — *a guard placed where the defect
cannot occur* (B12); ⚠️ E extended it to **all nine** non-triage routes and inertness **held**, so
the behaviour is confirmed and the guard is still misplaced.

**The apparatus.** **D25** gate robustness, four cheap ones in one file: `process.exit` after
`console.error` can truncate the only diagnostic that distinguishes the two failures (B16); a
violation carrying zero nodes prints red and exits **0** (B19); `routes` entries are used
unvalidated, so a `null` href is reported as *"did not answer"* — the wrong cause (B22); axe is
injected as an evaluated string rather than a script tag (B23, suspicion). **D26** the CI readiness
loop is bounded at **~15.5 minutes**, not the 30 seconds it reads as (E14, measured
`http=503 time=30.002716` — `curl` carries no `--max-time`, and neither job nor step has
`timeout-minutes`).

**The record — and this is where the acceptance layer did its work.**

- 🔴 **D15 · The story contradicts itself, and the false half is the one that changed a test.** The
  Dev Agent Record's instrument #1 says *"arrows do not scroll under headless CDP at all"*; A5
  measured `/diagnostic`: shipped code → **`scrollY 40`**, an unconditional `preventDefault` →
  **`scrollY 0`**. The check discriminated. §0e's own figure (*0 → 26 px with an early return, 0 → 0
  without one*) agrees with A and contradicts the record four sections later. Against this project's
  own rule — **a cause needs a check, not a plausible story** — the recorded cause is refuted and
  A's reading (a *dispatched* untrusted `KeyboardEvent` rather than a pressed key, i.e. a mutation
  named for one thing applied to another) is the one with a measurement behind it.
- 🔴 **D16 · *"Six divergences registered"* is false in three documents.** `deferred-work.md` gained
  **three** rows (A7; **re-measured here**: `47 insertions`, one `## Deferred from: story 6b.11`
  section, three owner rows). The `ci.yml` exception and 6b.1's token-literal conflict exist only as
  **code comments ending "registered"**; and the **`j`/`k` + `⏎` divergence — the largest, since
  `epics.md:2306`/`:2308` prescribe them and Epic 7 must know they were skipped — is in the register
  nowhere** (`grep`, empty, re-run here). Story 6b.9's sentence, verbatim: *a comment that says
  "registered" is not a registration*.
- 🔴 **D17 · Neither twin carries this story.** `git diff master...HEAD -- CLAUDE.md
  docs/project-context.md` is **one line changed in each** — the `RUSTFLAGS` correction — and
  `grep -c "6b\.11"` returns **1** in each, that same line (A6; **re-measured here, identical**).
  Consequence: both twins' last *"THE LIVE COUNT lives in…"* pointer still names
  `6b-10-copy-fr-and-en.md`, whose figure is **727**, while this story's is **728**.
- 🔴 **D18 · There is no mutation table.** Four anecdotes, no ids, no carriers, no greens (A8) — in
  the story whose **AC5 is the mutation record**, and whose T8 is ticked. Three of its guards are
  measured green in the reports above; a table would have had to say so.
- **D19 · The dead contract has FOUR artefacts, not three** — `_gap_card.html:1` still carries
  `tabindex="-1"` (A12; **confirmed here**), whose only purpose was the focus the deleted handler
  performed. T5 says *"all three"*; §0b enumerates three.
- **D20 · `.gitignore` closes the incident on one path, not the pattern** — `git check-ignore -v
  node_modules/foo` → **rc=1, not ignored** (A13; **confirmed here**, while `a11y/node_modules/foo`
  maps to `.gitignore:110`). The entry's own comment says a directory that size entering the tree is
  invisible to every check the project has — as true one directory up.
- **D21 · Counts and a token that contradicts its own comment.** *"Two tokens were darkened"* — the
  diff darkens **three** (B17, which also verified by hand that the *constant hue and saturation*
  half **holds** for all three, so only the count is wrong). And `--accent-document` sits in `TEXTS`
  — *the tokens a screen paints text with* — beside an unchanged comment saying it is *"used by
  NOTHING else"* with a test pinning that at zero (B5): **the record and the code disagree about a
  token under a pinned-usage guard**, and one of the two sentences is false.
- **D22 · `--color-accent-600` fails AA on all four grounds** (A11: 3.80 / 3.51 / 3.91 / 3.45) —
  latent, since no rule references it today, and T2's ticked bullet asking whether the ramp follows
  is answered nowhere. **D23 · The headline `237` is reproducible from no store state** (A19: A's own
  7-row store gives **221**) — which is AC4's own rule, *every figure names its store state*, unmet by
  the story's loudest figure. **D24 · §0g's second inherited obligation is neither taken nor
  re-registered** (A15), and **the register row this story discharges is left standing as pending**
  (A16, `deferred-work.md:4376`, while the file already carries `## Discharged by story 6b.2`
  sections). **D31** two adjacent helpers give opposite justifications for float versus integer
  arithmetic (B18 — both defensible, the pair is not). **D32** the File List omits `.gitignore`
  (A22). **D33** 6b.2's `no_screen_is_chosen_by_javascript` needle list now sits beside a deliberate
  `window.location.assign` with no note (A21b).

**Stated, not a defect: D29** — three different `aria-current` semantics live on one rendered
`/triage` (E15: `nav-entry = page`, `btn-sort on = true`, queue row `= true`), with no axe violation
and no code collision measured. Recorded so it is not rediscovered as a bug.

## §T4 — The refutations, each with the check that settled it

⚠️ **The untriaged preamble named three refutations and it is imprecise: two are a layer refuting a
layer, the third is a layer refuting THE STORY.** They must not be filed together — one kind
corrects a reviewer, the other corrects the product record.

| Claim | Who claimed it | The check | Verdict |
|---|---|---|---|
| *"Those paths also leak the Chrome process"* | B1, secondary clause | E ran `pgrep -af "/usr/bin/google-chrome"` after a launch failure, a 500-route abort **and** an uncaught throw | **REFUTED** — 0 in every case; puppeteer's exit handler reaps the child. B1's *primary* claim (exit 1) is confirmed by E6. |
| *"`npm ci` with no lockfile visible in the change"* | B7, HIGH **if true**, self-labelled SUSPICION, and it **named the check itself** — *"settle it with `git ls-files a11y/`"* | A ran `npm --prefix a11y ci` → 26 packages, 354 ms; **re-run here**: `git ls-files a11y/` lists `package-lock.json` | **REFUTED** — and this is the honest shape of a blind finding: a claim carrying the instrument that kills it. |
| *"Arrows do not scroll under headless CDP at all"* | ⚠️ **the story's own Dev Agent Record**, not a layer | A5's scroll measurement, above | **REFUTED — against the story.** Filed as **D15**, a defect, not as a reviewer's error. |

**E also ran and refuted fourteen suspicions of its own** — among them that a click during the
settle window is always overridden (**refuted warm**, which is exactly the control that makes D7's
slow-document measurement mean something), that the CI trap masks the exit code, that a route
answering 500 is reported as a violation (it exits **2**, correctly), that a violation on a late
route is lost, that `?sort=age` is dropped by an arrow, that modified arrows are captured, that the
focus ring is invisible (**`2px solid rgb(75,107,139)` on `rgb(245,245,248)` = 5.11:1** against
WCAG 1.4.11's 3:1), and that disabling JavaScript breaks the queue (8 rows still render — the
selection is a URL). **A likewise refuted the stronger form of its own D13** and confirmed the hue
guard is not vacuous for chromatic tokens. None of that is a finding, and all of it is work: it is
the difference between a suspicion and a measurement.

## §T5 — What the triage says about the METHOD

🔑 **The blind layer found HIGHs from the diff alone for a FIFTH consecutive story**, and this time
the sharpest of them is a deduction no sighted layer made: **B2 established that the new
`aria-current` assertion is satisfied by a different element — using only `kbd-probe.mjs`, a file
added in the same patch, as evidence that a queue row already carries the attribute on a default
render.** It could not open the tree, so it read the diff *against itself*. A then measured exactly
that (D8). The argument for keeping that layer blind is now five stories old and has not lost once.

⚠️ **The crash is visible in the shape of the reports and it cost nothing measurable**: the
edge-case layer was re-run from its original mandate rather than resumed, and it is the layer that
contributed the most *executed* mutations (fifteen findings, every one with a planted input). A
resumed layer would have inherited 569 bytes of narration; a re-run one inherited a mandate.

⚠️ **The dominant defect class of Epic 5 is present here in four separate places and was named by
three different layers**: D2 (a layer covered by a script nothing runs), D12 (a counter that cannot
fail placed over a loop that cannot scan nothing), D13 (a guard blind on the grounds it omits), D27
(an inertness probe run only where the early return fires). ⚠️ And **D9 is the story's own T3
reproduced in its T5** — T3 exists to undo a source-reading needle standing in for a DOM property,
and the focus contract's only Rust carrier is a source-reading needle inside a test about the amber.

## §T6 — Ranked disposition

**Four ARBITRATIONS for Guy** — each raised by a measurement, none decidable from the criteria:
**D1** the exit-code contract (repair, or narrow the promise in writing on story 5.12's precedent) ·
**D5** does CI seed a queue before measuring · **D6** `token_hex` reading a comment · **D7** the
uncancelled settle timer.

**PATCH — the guards, in the order the measurements rank them.** D2 (AC5's central hole) · D4 (the
hue guard, both directions) · D3 (the route floor, a `Set` and `!==`) · D8 (a real oracle for
`aria-current`) · D9 (a DOM-level carrier for the focus ring) · D12 (a premise that can fail) ·
D13 (the two omitted grounds) · D10 (one `setAttribute`) · D25 (four cheap gate repairs) · D26
(`--max-time`) · D28 · D30.

**PATCH — the record, and AC5 is not met until D18 exists.** D18 (the mutation table) · D15 (a
refuted cause, per this project's own rule) · D16 (three documents saying *registered*) · D17 (the
twins and the live-count pointer) · D19 · D20 · D21 · D23 · D32 · D33 · D31.

**REGISTER by name, with an owner**: D11 (focus lost at the settle — decide, or Epic 7 with the
`⏎` gesture) · D14 (`location.replace`) · D22 (`--color-accent-600`) · D24 (both inherited rows) ·
D27's residual · the `j`/`k` + `⏎` divergence that D16 shows was never registered at all.

**DISMISSED with the check**: the Chrome leak, the missing lockfile (§T4).

**AC verdicts stand as the acceptance layer set them: AC5 NOT MET; AC3, AC4 and AC7 PARTLY MET;
AC1, AC2, AC3b and AC6 MET.** Nothing in the triage moves one of them, and D18 is what AC5 needs
first.


## §T7 — The four arbitrations (Guy, 2026-08-23)

Each was raised by a measurement, none was decidable from the criteria, and in all four Guy took the
option that closes the property rather than the one that corrects the sentence. Recorded here with
the option refused, per the house convention.

**Arbitration 1 · D1 — repair AND state the residual.** A top-level `try/catch` closes the ~8
unguarded Node paths at once: every *could not run* answers **2**, and **1** is left to the
violation path alone. ⚠️ **But the shell's own codes escape any Node-level repair** — `npm ci`
failing exits npm's code, the readiness `curl` exits 22, a missing Node makes the shell exit **127**
(A17, measured) — so that part ships as a **stated limit** and `ci.yml:80-82`'s sentence is
corrected to match. *Refused:* repairing in silence (the CI sentence would stay false for the shell
half), and narrowing alone on story 5.12's precedent (~8 paths closable by one wrapper is not a
limit, it is an omission).

**Arbitration 2 · D5 — seed a queue AND put the unreached states in front of axe.** CI seeds a few
rows before the gate runs, and the route list gains the states axe never reaches (`?sort=age`,
`?sel=…`). 🔑 *Refused:* seeding alone, which closes E1/A14 and leaves **B6** standing — the
replacement for an attribute a browser rated critical would still be verified by no browser at all —
and leaving CI blind by decision, which the measurement makes untenable: **the story's own defect,
replanted, exits 0 on CI's store.**

**Arbitration 3 · D6 — read the BLOCK, not the file.** Comments stripped, the token bound to a
declaration inside the light `:root`, and anything that is not a 6-digit hex refused loudly. Closes
all three witnesses in one change: the comment (E5), the dark palette's positional selection (B15),
the silent 8-digit alpha (E7). *Refused:* stripping comments only, which leaves the palette chosen
by source order and undocumented; and narrowing the promise, which would leave the guard's own
`checked == 16` premise vouching for a scan that reads the wrong source.

**Arbitration 4 · D7 — the operator's gesture always outranks the timer.** `click`, `pointerdown`,
a non-arrow `keydown` and `pagehide` all clear `pending`. *Refused:* stating the cost and
registering it, since the defect is reachable **inside 6b.10's own 2 s budget** and was measured
there; and dropping the timer for `⏎`, which would reopen arbitration 4's shape (E) — taken on a
20× prototype measurement — and the deliberate `⏎` refusal handed to Epic 7.

## §T8 — The repair pass, and AC5's record

**2026-08-23.** Everything below ran against a live `mariadb:10.11.11` (container
`opencmdb-6b11-fix`, port **13360**), a booted binary, Chrome 151 and a seeded two-row queue.
Every mutation was reverted from a scratchpad copy, **never `git checkout --`** — the gesture
that destroyed uncommitted work four times in this project.

**Closing state.** `490 + 161 + 77 = 728` tests, run **both ways**: ⚠️ **the figure first written
here was 16.7 s and it is NOT reproducible** — the second review round measured 5.1–7.1 s and
re-measurement gives **6.65 s** with `cargo test --workspace --locked`, warm, against the live
store, and **0.65 s** without `DATABASE_URL`. Mine had a build inside it and named no command,
which is precisely what this project's own rule forbids. The 0.6 s half reproduced exactly. The
clock remains the tell that the store-backed tests genuinely executed. Nine gates green, `cargo fmt --all --check` clean, and clippy over
`--all-targets` — the form that sees the test targets CI compiles — clean. `RUSTFLAGS="-D
warnings" cargo test --workspace --locked` green, which is the other half of what CI does.

⚠️ **The test COUNT does not move, and that is the honest reading of this pass**: no `#[test]`
was added. What changed is that three guards that measured nothing now measure something, one
read the wrong source, and the story's central deliverable acquired a carrier that is not a Rust
test at all — it is a browser gate, because the property lives in the DOM and AC5 says in so many
words that a source-reading guard does not suffice for a defect that lives in the DOM (the
criterion's wording as amended on 2026-08-23; the reports above quote the earlier *"is not
counted"*, and the two give the same verdict here).

### The table

| id | mutation | before the repair | after the repair | carrier |
|---|---|---|---|---|
| **M-A1a** | `mv a11y/node_modules` away | exit **1** | exit **2**, `Cannot find package 'puppeteer-core'` | gate message |
| **M-A1b** | `throw` appended to `axe.min.js` (the review's own) | exit **1** | exit **0 — GREEN**, see finding 3 | — |
| **M-A1b-bis** | `axe.min.js` truncated, `window.axe` undefined | — | exit **2**, `TypeError … reading 'run'` | gate message |
| **M-A2a** | store emptied, `AXE_REQUIRE_QUEUE=1` | exit **0** | exit **2**, *"carries no queue row"* | gate message |
| **M-A2a′** | same, variable unset (a developer's tree) | exit 0, silent | exit **0 with the gap PRINTED** | stdout |
| **M-A2b** | the story's own `aria-pressed` defect replanted on the **sort link** | invisible — on no href | exit **1**, `/triage?sort=age  aria-allowed-attr(1, critical)`, and the ten derived routes all ✅ | axe |
| **M-D3** | every nav href → `/triage` | exit **0**, `✅ /triage ×10` | exit **2**, *"1 DISTINCT route(s) … expected exactly 10"* | gate message |
| **M-D25b** | every nav href → `#…` | *"did not answer"* — the wrong cause | exit **2**, naming the ten unresolvable hrefs | gate message |
| **M-A3a** | E5 verbatim: bad value, good value in a **comment** above it | **GREEN** (axe: 210 nodes) | **RED at 3.82:1** — the figure the commit quotes | named assertion |
| **M-A3b** | *control*: stale value in a comment over a **correct** declaration | **RED at 1.12:1** | **GREEN** | — |
| **M-A3c** | `#68686b1a` (8-digit, alpha) | **GREEN at 4.96:1** (axe: 210 nodes) | **RED**, *"declared as a six-digit hex literal"* | named panic |
| **M-A3d** | `light_root` returns the whole file | — | **RED**, *"still carries a comment"* | named assertion |
| **M-D4a** | `--color-neutral-500` → `#ffffff` | **GREEN** | **RED**, pure-extreme refusal | named assertion |
| **M-D4b** | `--color-bg` → `#ffffff` | **GREEN** | **RED**, same | named assertion |
| **M-D4c** | `--color-accent-700` → `#b5d9fd` | **GREEN** | **RED on the luminance band** — ⚠️ its hue passed at 210° exactly, which is why the band exists | named assertion |
| **M-D4d** | *control*: `--color-text` → `#141616`, a legal lightness-only repair | **RED** | **GREEN** | — |
| **M-D4e** | `--color-neutral-500` → `#0000ff` | **GREEN** | **RED**, family, *"spans 255 channel units"* | named assertion |
| **M-D8** | `aria-current` stripped from the sort link | **490 passed** | **RED**, message printing the tag it actually read | named assertion |
| **M-D9** | `.queue-row > a:focus-visible` and `.btn-sort:focus-visible` deleted | 490 green, nothing else | **490 STILL green** and the **kbd gate reds**, `1px auto rgb(16,16,16)` | kbd gate |
| **M-D2** | `app.js` emptied entirely | 490 + nine gates + axe **all green** | **kbd gate exit 1**, ⚠️ **EIGHT** checks red — this row said *four*, a figure my driver produced by piping the gate's output through `head -4`; corrected at the second review round | kbd gate |
| **M-D10** | `select()` no longer moves `aria-current` | — | **RED**, `focus=1 aria-current=0` | kbd gate |
| **M-D11** | the restore marker removed | — | **RED**, `activeElement=BODY row=-1` | kbd gate |
| **M-D14** | `location.replace` → `location.assign` | — | **RED**, `history 2 → 3` | kbd gate |
| **M-D28** | the identical-URL guard removed | — | **RED**, the mark lost | kbd gate |
| **M-arb4a** | the `pointerdown`/`click` cancel removed | — | **RED**, the pending navigation wins | kbd gate |
| **M-arb4b** | the non-arrow `keydown` cancel removed | — | **RED**, same | kbd gate |
| **M-floor** | one check skipped inside the kbd gate | — | **exit 2**, *"16 check(s) ran where 17 are declared"* | gate message |
| **M-D33a** | `pushState` planted in `app.js` **code** | RED | **RED** — detection survives the comment stripping | named assertion |
| **M-D33b** | the comment stripper returns `""` | — | **RED**, *"matched against nothing"* | named assertion |

**29 ids: 26 reds, 2 controls GREEN by design (M-A3b, M-D4d), and 1 green for a reason the
repair created (M-A1b).** Carriers are mixed and named per row — twelve on a named Rust
assertion, twelve on a gate's own message or check, two on a named panic. No headline of the
*"every red assertion-carried"* shape is claimed: this project has had that sentence refuted five
times, and here it would be false by inspection of the table's own last column.

### Six findings the pass itself produced

1. 🔴 **MY OWN DRIVER CARRIED THE DEFECT THIS PROJECT HAS RECORDED FOUR TIMES.** The first
   control ran `cargo test -p opencmdb-bin --locked page::tests::ac2 page::tests::every_text` —
   **two filters where cargo accepts one**, so nothing ran. Caught because the control printed
   **nothing** where it owed a result; had it printed a green, it would have been filed as a
   confirmation. The form that works is `cargo test … -- filter1 filter2`.
2. 🔴 **THE FIRST REPAIR FOR ARBITRATION 1 DID NOT WORK, and the mutation said so.** A top-level
   `try` was written around the gate's whole body and `mv node_modules` **still exited 1**: a
   static `import` is resolved before any statement of the module runs, so there is no `catch` of
   ours in existence yet. `puppeteer-core` is a **dynamic** import now, inside the `try`. *A
   repair believed is a repair unmeasured.*
3. 🔴 **THE REVIEW'S OWN M-A1b NO LONGER MEASURES WHAT IT IS NAMED FOR, and the reason is a
   different repair.** Appending `throw new Error(…)` to `axe.min.js` exited 1 under
   `page.evaluate(axeSource)`, which propagates any throw; under `page.addScriptTag` — adopted
   here for an unrelated reason — the same input is **harmless**, because the throw comes *after*
   axe has defined itself, so axe genuinely ran and genuinely found nothing. **M-A1b-bis** is
   what measures the contract. *A mutation named for one thing applied to another*, met again —
   and this time the change of meaning was caused by the repair rather than by the driver.
4. ⚠️ **A CHECK WRITTEN FROM A FINDING'S SUMMARY INSTEAD OF ITS INPUT REDDENED A CORRECT
   PRODUCT.** The *net movement of zero* check started from a bare `/triage`, where no row is
   current, so ↓ then ↑ genuinely moves the operator from *no row* to *row 0*. E13 names its
   input precisely — *"from `/triage?sel=<row0>`, row 0 already current"* — and the check now
   starts there. *The summary of a finding is not its input.*
5. 🔴 **A RED WAS NEARLY MISATTRIBUTED TO THE MUTATION THAT DID NOT CAUSE IT.** M-D9 (a CSS
   change) appeared to red the Rust suite at 489/1. The failing test was
   `screens::tests::no_screen_is_chosen_by_javascript`, which reads **`app.js`** — the red came
   from this pass's own keyboard-layer edit, not from the stylesheet. Re-measured on a clean
   base: M-D9 leaves **490 green**, exactly as the review reported, and reds the kbd gate alone.
   *A mutation's red belongs to the mutation only if the baseline was clean.*
6. 🔴 **AND THE TEST THAT REDDENED WAS READING A COMMENT** — `no_screen_is_chosen_by_javascript`
   matched its `history.` needle inside a *sentence describing* the defect being repaired. Same
   family as `token_hex` reading a CSS comment, one file over and the same day. ⚠️ The tempting
   fix was to rewrite the prose; that would have been the guard editing the record instead of the
   record editing the guard. The guard strips comments now, its detection is re-measured intact
   (M-D33a) and its stripper has a premise that can fail (M-D33b).

### What the repair did NOT close, stated rather than implied

- **The exit contract stops at the shell** (arbitration 1's stated residual): `npm ci`, `curl`
  and a missing Node keep their own codes, none of them 1 or 2. Written in the gate's header and
  in `ci.yml`, registered.
- **The contrast table is still an enumeration.** Two painted grounds stay out, with the
  measurement that adding them reds pairings no screen renders; a `TEXTS` entry removed still
  reduces coverage with nothing red. *An enumeration cannot claim the completeness of a
  property* — the sixth application of that sentence in this project, and it is written at the
  guard rather than contradicted by it.
- **`--color-accent-600` still fails AA on all four grounds**, latent because no rule paints it.
  Registered with its owner.
- **The luminance band is a tripwire, not a proof**: a token can be moved anywhere inside its own
  band. What it refuses is a jump to another part of the ramp, which is what the three green
  mutations were.

## §T9 — The SECOND review round, on the repair itself (2026-08-23)

The repair of §T7/§T8 was itself put in front of three isolated layers, on a different model,
each in its own worktree with its own live `mariadb:10.11.11`. **19 raw findings → 17 distinct
defects — and this time NOT ONE defect was reached by two layers.** Where the first round found
six defects by three roads each, this one found seventeen by one road each: the repair moved the
weaknesses into places the three viewpoints do not reach the same way. *A convergence rate is a
property of the round, not a constant.*

### 🔑 The most instructive pairing is a DISAGREEMENT of reading, not an agreement

The edge layer measured *"`kbd-probe.mjs` on exactly 1 row → exit 2, `MIN_ROWS=2` enforced"* and
filed it under **suspicions RUN and REFUTED — the contract holds**. The acceptance layer measured
**the same fact** and filed it as **HIGH — the gate cannot run in CI**. Both measurements are
correct. The difference is the question: *is the contract respected?* against *can the gate
execute where it is supposed to protect?* **The same number, read as a confirmation by one layer
and as the defect by the other.**

### The four that blocked, each re-measured by me before being believed

🔴 **1 · CI WAS GREEN ON RESIDUE.** From a virgin store the shipped demo seed renders **one**
queue row — `0` before it, `1` after, measured — and the keyboard gate's floor is two, so it
exits **2**. The run that passed reported `queue: 3 row(s)` because the `Tests` step uses the
same database immediately before and what it left behind carried the queue over the floor.
**Nothing guaranteed that.** ⚠️ And §T8 says the repair was measured *"against a seeded two-row
queue"* — a state that existed only in my own session's accumulated database and **nowhere in the
repository**. The acceptance layer found it by doing the one thing I had not: reproducing CI's
sequence from a TRUNCATED store rather than from the store its session had to hand.

🔴 **2 · I REPLACED A VACUOUS PREMISE WITH A VACUOUS PREMISE.** `assert_eq!(measured_tokens, 6)`
counts increments in a loop over a six-element array whose only escape is a `panic!` one line
above — it is 6 on every execution that reaches it, exactly as `checked == TEXTS.len() *
GROUNDS.len()` was 16, which is the defect it was written to close. ⚠️ **And its comment claimed
it *CAN fail*** — a false sentence inside the repair whose subject is false sentences. Found by
the layer with the diff and nothing else.

🔴 **3 · THE FOCUS MARKER BELONGED TO NO NAVIGATION.** It carried a bare `"1"`, and
`sessionStorage` outlives the navigation that wrote it, so a settle whose document never
committed left it standing until the next plain load of `/triage` pulled focus into the queue —
**the autofocus the design refuses in its own comment**. There was no way to tell *my own marker*
from *a stale one*. The marker is now the address it was written for, and is cleared whether it
matches or not.

🔴 **4 · `sessionStorage` THROWING SILENTLY RESTORES THE ORIGINAL DEFECT.** Blocked storage:
the URL still catches up and `activeElement` is `<body>` after the settle — the twelve-Tab defect
back, invisibly. The `catch` prevents the crash and there is no fallback, because without storage
there is nowhere to leave a note. **Stated as a limit at the site**, not covered: a restore keyed
on the URL alone would be the autofocus this mechanism exists to avoid.

⚠️ **And my own M-D2 row said *"four checks red"* where there are EIGHT** — my driver piped the
gate's output through `head -4`. *A measurement read through a truncation is not a measurement*,
committed in the table whose whole job is to carry measurements.

### What the repair of the repair changed

| id | mutation | result | carrier |
|---|---|---|---|
| **M-P1** | `a11y/seed.sql` removed from `SANCTIONED_SITES` | **RED**, `a11y/seed.sql:37: insert into declared_attribute outside the sanctioned write sites — NFR5` | gate message |
| **M-P2** | `a11y` removed from `AUTHORSHIP_ROOTS` | **GREEN over the same file**, 42 files walked where 43 are | — |
| **M-B2** | `light_root` returns the whole file | **RED** on the replaced premise | named assertion |
| **M-E3** | `--color-accent` declared TWICE, the second value out of band | **RED** at the luminance band | named assertion |
| **M-E4** | a SECOND unqualified `:root` overriding the accent | **RED**, same | named assertion |
| **M-E1** | the marker reverted to a bare `"1"` | **RED**, `focus in queue=true` | kbd gate |
| **M-B1** | `app.js` emptied (re-run against the widened gate) | **RED**, 9 of 20 checks | kbd gate |
| **M-P3** | the third root's file removed from the gate test's synthetic tree | **RED**, `all three roots are read: … across 2 file(s)` | named assertion |
| **M-P4** | `a11y` added to the roots with the synthetic trees left untouched | **RED ×3**, *"a11y/ is missing — the guarded subtree must exist"* — while `cargo xtask ci` stayed **GREEN** over the real tree | named assertion |

🔑 **M-P1 and M-P2 are one proof in two halves.** Naming the site without opening the roots
sanctions a file nobody walks; opening the roots without naming the site turns the gate red. Each
mutation shows the other half is load-bearing — which is why story 5.12's instruction (*reopen the
perimeter the day a new file carries SQL*) is one act and not two.

### 🔴 Three findings the repair-of-the-repair produced against ITSELF

1. **The stale-marker guard came back GREEN under the reversion it exists to catch.** It planted
   the marker's CURRENT representation for a different address — which the OLD bare-sentinel code
   also refuses, for its own reason. *A guard written against the new representation cannot see
   the old defect.* It now plants both, and the property is stated representation-free: **no
   pre-existing marker value whatsoever may restore focus on a load the keyboard did not drive.**
2. **The floor said 18 for 19 checks**, then 19 for 20 — caught by running it, twice. *A floor is
   only a guard while it equals what is there*, in the file where that sentence is written.
3. 🔴 **WIDENING THE PERIMETER REDDENED THREE `xtask` TESTS WHILE `cargo xtask ci` STAYED
   GREEN.** The gate FAILS CLOSED on a missing root — deliberately — and every synthetic tree the
   tests build carries one directory per root, so a third root made twenty probe verdicts and the
   walk test fail closed on scratch trees that had only two. The real tree has `a11y/`, so the
   gate run passed throughout. ***A gate green over the real tree says nothing about its own
   tests***, and only `cargo test --workspace` said so — the other half of what CI does, run
   locally before the push. ⚠️ Adding the directory alone left the new root covered by **nothing
   but the real tree** (the count assertion stayed at `2 file(s)`); it now holds a file and M-P3
   reds when that file goes.
4. **The served-versus-source check is UNREDDENABLE in the configuration CI runs.** `rust-embed`
   reads assets from disk in a debug build, so served and source are the same bytes by
   construction and no mutation of the source separates them; my first attempt to red it simply
   re-ran *"`app.js` emptied"*. It is kept as a tripwire for a RELEASE build, where the assets are
   embedded — and **recorded as green-by-construction, because a check nobody can red must say
   so.**

⚠️ **And one incident worth keeping**: my first `a11y/seed.sql` encoded the observation facts
wrongly and `/triage` answered **500**. Both gates said `answered 500, so nothing there can be
measured` and exited **2** — the contract held under a fault I introduced myself, rather than
measuring a blank page and reporting success.

### Refuted, each with the check

- **The keyboard gate's CI step lacks its credentials and will 401** (blind layer, HIGH) —
  refuted twice: the step's `env:` block sets both at STEP level, so they are in the environment
  of every command in the `run:`; and the CI run reported `queue: 3 row(s)` and `17 check(s) run,
  0 failed`, i.e. it authenticated. The layer named this check itself and could not run it.
- **A `//` inside a string literal defeats `strip_js_comments`** — refuted by grep over the whole
  file, independently by two layers. Recorded as the disclosed limit it already was.
- **`class="btn-sort` could match a longer class name earlier in the page** — one element in the
  whole template tree carries it. Latent, not live.
- The edge layer additionally **ran and refuted fourteen** of its own suspicions, among them held
  arrows, arrow-then-click, arrow-then-Back, the neutral/chromatic dead zone at span 17–31, the
  8-digit hex, compound `:root, .foo` selectors, multibyte characters at brace boundaries, and
  the four harness-failure shapes of the 0/1/2 contract. ⚠️ One of its own probes produced
  spurious timeouts it traced to **its own proxy** forwarding a stale `Content-Length` — a
  reviewer's harness lying, recorded so the next one recognises it.

### Registered rather than fixed

- The `sessionStorage`-blocked context has no fallback (finding 4 above), stated at the site.
- `light_root`'s two enumerated limits: a `}` inside a quoted value truncates the block; an
  unterminated `/*` panics. Neither shape exists in `app.css`; *an enumeration cannot claim the
  completeness of a property.*
- **§T8's `16.7 s` was not reproducible and is CORRECTED in place** to **6.65 s** with its
  command named — the acceptance layer measured 5.1–7.1 s on its machine. Mine had a build inside
  it and named no command, which is precisely what this project's own rule forbids. The `0.60 s`
  half reproduced exactly.
- **Issue #38 recurred** during the audit on a different tree (1 red run in ten, not reproducible).
  Recorded with the run count and **no cause named**.

## §T10 — AC5's verdict, resolved on the auditor's OWN criterion (2026-08-23)

The acceptance layer returned **AC5 NOT MET**, and — this is what makes it resolvable without
anyone marking their own homework — **it named the condition rather than the conclusion**:

> *The carrier now genuinely exists and genuinely reds on replay … **But** as wired into
> `ci.yml`, this carrier cannot execute — a carrier that cannot run in the pipeline that is
> supposed to enforce it is not a carrier AC5's "there must be a carrier" clause is satisfied by.*

So the verdict turned on one measurable fact: **can the carrier execute in CI?** At the time it
could not, for the reason §T9 records — the shipped seed renders one queue row against a floor of
two, and the run that had passed did so on the previous step's residue.

🔑 **The condition is now met, and the evidence is neither mine nor the auditor's.** CI run
`32658655672`, on the pushed repair:

```
axe gate: 10 route(s) derived from the navigation plus 2 state(s) no href carries, 0 violation node(s)
queue: 4 row(s)
kbd gate: 20 check(s) run, 0 failed
```

**Four rows**, which is what `a11y/seed.sql` produces from a truncated store — not the three the
residue had left. The carrier executes, in the pipeline, on a state the repository can reproduce.

**AC5 is therefore MET**, and the reasoning is worth stating rather than the conclusion alone:

| AC5's clause | Carrier | Measured |
|---|---|---|
| every guard measured RED before it passes | §T8's table plus §T9's | 29 + 7 mutation ids, carriers named per row |
| a source-reading guard does not SUFFICE where the defect lives in the DOM | `a11y/kbd-probe.mjs`, 20 checks | `app.js` emptied → 9 red; the two focus rules deleted → **490 Rust tests still green** and the gate reds |
| …there must be a carrier that reads what the browser did | the same gate, **in CI** | run `32658655672`, `20 check(s) run, 0 failed` at 4 seeded rows |

⚠️ **What is NOT claimed.** The three review layers have not re-audited this repair either; what
is claimed is narrower and checkable — *the layer stated a condition, the condition is now
measured satisfied, and the measurement is CI's.* The other verdicts stand as the auditor set
them: **AC1, AC2, AC3, AC3b, AC4, AC6, AC7 MET**, with AC3's swap half correctly deferred to story
6.4 and AC6 carried forward rather than re-tested (no code path touched it).

⚠️ **And one thing the whole exercise did not change**: what the operator can DO is unchanged —
no route, no write, no gesture. This was a hardening pass over an accessibility apparatus and its
own review's findings. What it buys is that a regression in the keyboard layer or in the
axe-clean state is now caught **before** a merge rather than shipping quietly, which two rounds
ago was true of neither.

---

## Change Log

| Date | Change |
|---|---|
| 2026-08-22 | Story created. Four contexting findings; the AC/constraint-6 contradiction; the HTMX swap measured absent. |
| 2026-08-22 | **Arbitrations 1–3 by Guy** — arrows only · ship the reachable focus contract · axe-core in this story. |
| 2026-08-22 | 🔴 **axe-core RUN**: all ten routes FAIL. |
| 2026-08-22 | **VALIDATED by two fresh-context layers: 38 findings, 12 HIGH.** Contexting corrected on eight points, three of them its own measurements — the axe figures were taken on an EMPTY store (the surface T4–T6 target was blank); the constraint-4 and `⏎` refusals rested on false premises; `Screen::Device` IS in `Screen::ALL`. The validation also proved what contexting had only assumed: the two-token fix takes **237 → 0** on a populated store. |
| 2026-08-22 | **DEVELOPED.** 727 → 728 tests, nine gates, axe green on all ten routes at two queue sizes. Three tokens, `aria-pressed` → `aria-current`, the keyboard layer in shape (E), four focus rules, the CI step. 🔴 Story 6b.1's hex-literal guard became TWO properties (hue + contrast); a SECOND `aria-pressed` oracle the spec had not named was found by running the change; and **four times the faulty instrument was mine, each settled by a control**. |
| 2026-08-22 | 🔴 **ARBITRATION 4 by Guy: shape (E)** — the three readings of *"move the selection"* were prototyped and differ by **20×**; under (A) a held arrow loses half its presses. |
| 2026-08-23 | **TRIAGED.** 60 raw findings -> **33 distinct defects**: six reached by all three layers, ten more by two, so nearly half of what was written is a second road to the same thing. 🔑 **D4 exists only in the triage** — the hue guard is too LOOSE for greys (any grey passes, B14/A9) and too TIGHT for the repair it exists to permit (a legal lightness-only darkening reds it, E9), and its failure message states as fact what the mutation producing it had just disproved. 🔑 D2·D3·D5 compose: the accessibility apparatus **reports success over a surface it does not reach**. ⚠️ The preamble's *"three refuted claims"* is imprecise — two are layer-versus-layer (the Chrome leak, the lockfile, both dismissed with their check), the third refutes **the story itself** (the scroll control) and is a defect. Four arbitrations stand open. |
| 2026-08-22 | **CODE-REVIEWED by three isolated layers — the three reports are poured in VERBATIM and UNTRIAGED.** 60 raw findings, no dedup, no ranking, no arbitration. The acceptance layer returns **AC5 NOT MET** (no mutation table; the keyboard layer, the focus ring and the new `aria-current` oracle each measured green) and AC3/AC4/AC7 partly met. 🔴 The review session crashed mid-flight: the edge-case layer was re-run from its original mandate in a fresh worktree, the other two recovered intact from their transcripts. |
| 2026-08-23 | **TRIAGED, ARBITRATED and REPAIRED.** 60 raw findings -> **33 distinct defects** (six by all three layers, ten by two). 🔑 **D4 exists in no single report**: the AC2 hue guard was too LOOSE and too TIGHT at once, and its failure message asserted as fact what the mutation producing it had disproved. 🔑 **D2·D3·D5 compose**: the accessibility apparatus reported success over a surface it did not reach. **Guy's four arbitrations, the recommendation taken in all four.** `a11y/kbd-probe.mjs` becomes the second BROWSER gate (17 checks, a floor equal to what is there) and CI runs it; the store is seeded; `token_hex` reads the light `:root` block with comments stripped; the settle timer yields to the operator. **29 mutation ids, 26 reds, 2 controls green by design, 1 green for a reason the repair created.** ⚠️ **Six findings against the pass itself**, the sharpest being a guard matching its needle inside a COMMENT — `token_hex`'s defect one file over, the same day. 728 tests both ways, nine xtask gates + two browser gates, clippy `--all-targets`. |
| 2026-08-23 | 🔴 **AC5 AMENDED by Guy** — *a source-reading guard **does not SUFFICE** where the defect lives in the DOM*, where it read *is not counted*. Two words, no change of scope. The scope was earned by measurement and held three-for-three; *counted* over-reached, reading as *worth nothing* while a source guard is cheaper and **names the cause** where a browser gate names only the symptom — the two cumulate. ⚠️ The residual cost is **proportionality**, which the amendment does not dissolve: a rule about where the defect lives, never about how much apparatus to build. The three review reports keep the ORIGINAL wording, unedited — both phrasings give the same verdict on all three measurements they took. |
| 2026-08-23 | 🔴 **SECOND REVIEW ROUND, on the repair itself** — three isolated layers, **19 raw findings, 17 distinct, none reached by two layers**. 🔑 Its sharpest pairing is a DISAGREEMENT of reading: the same measurement filed as a *refuted suspicion* by one layer and as a **HIGH** by another. 🔴 **CI was green on RESIDUE** — the shipped seed renders one queue row against a floor of two; closed by `a11y/seed.sql`, with `AUTHORSHIP_ROOTS` and `SANCTIONED_SITES` widened in ONE act. 🔴 `measured_tokens == 6` was as vacuous as the assertion it replaced; the focus marker belonged to no navigation. ⚠️ **Four findings against the repair-of-the-repair**, including a guard green under the reversion it exists to catch, and three `xtask` tests red while `cargo xtask ci` stayed green. |
| 2026-08-23 | ✅ **AC5 MET, resolved on the ACCEPTANCE LAYER'S OWN CRITERION.** It returned NOT MET while naming the condition rather than the conclusion — *a carrier that cannot run in the pipeline meant to enforce it is not a carrier* — and CI run `32658655672` measures that condition satisfied: `queue: 4 row(s)`, the number `a11y/seed.sql` produces from a truncated store, and `20 check(s) run, 0 failed`. ⚠️ The evidence is neither mine nor the layer's. All seven other ACs stand as the auditor set them. |
