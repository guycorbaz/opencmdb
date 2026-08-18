# Story 6b.2: The shell — header, navigation, and ten routes

Status: ready-for-dev

Epic: 6b — *L'interface de la maquette*. **Second story**, after 6b.1 put the mock's design system
in the binary. This one gives the product its FRAME and, for the first time, more than one address.

## Story

As the operator,
I want each screen to have its own address,
so that I can link to one, bookmark it, and come back to it.

## What this story does NOT do

It fills **no screen**. Each of the ten routes renders the shell plus an empty main column — the
example dataset, the marker and the sentence that says *"this is a demonstration"* are story 6b.3's,
and the real gap moves into the triage screen in 6b.4. ⚠️ **This is the story most exposed to
building 6b.3's honesty by accident**: a screen that renders nothing looks broken, and the temptation
is to fill it. Do not.

It writes **no copy**: 6b.10 owns the ~100 mock strings and their English twins. This story's labels
are the ten navigation entries and the header, added to `locales/app.yml` as keys because a template
cannot carry a bare French string in a product whose NFR26 is EN + FR.

---

## 0. ✅ THE FIVE ARBITRATIONS, taken by Guy on 2026-08-18 before a line of code

**The governing sentence is Guy's own: *the mock prevails*.** It settles §1, §2 and §3 at a stroke,
and §2's resolution is the interesting one — it does not resolve the collision, it **dissolves** it.

| § | Question | Decision |
|---|---|---|
| 1 | mock's ten entries vs UX-DR33's six | **the mock.** Ten entries, three groups, **no Topology** |
| 2 | the header's *perimeter* and *last observation* | **the mock's header** — brand, tagline, version — with *"v0.1.1 · maquette"* replaced by the RELEASE version. **Neither of the AC's two extra facts ships.** |
| 3 | `device`, a detail screen with no address | **as the mock does, for now**; revisit later |
| 4 | what becomes of `/` | **`/` redirects to `/triage`**, and `/dashboard` exists in its own right |
| 5 | the Tailwind chain | **not now.** Hand-authored, as 6b.1 |
| 6 | responsive: the mock has none, the UX spec mandates three breakpoints | **not now**, we will see later |

### 0a. 🔴 THE REASON THIS STORY FIRST GAVE FOR ARBITRATION 2 WAS FALSE, and the validation refuted it

**What the first draft said:** that the collision between the epic's AC (*a header carrying "last
observation"*) and the epic's constraint 1 (*no demo screen opens a connection*) **dissolved**,
because neither fact was in the reference — *"the cheapest way to satisfy two contradictory
requirements was to notice that one of them was not in the reference."*

🔴 **Both facts ARE in the reference.** Not in the `<header>` — that part was right — but in the
**sticky footer of the `<nav>`**, which is part of the shell and therefore on all ten screens:

```html
<div style="margin-top: auto; padding: 16px 10px 0; border-top: 1px solid var(--line)">
  Périmètre 192.168.10.0/24<br>Dernière observation il y a 4 min
</div>
```

This story's §1 called its extraction *"verbatim"* and omitted that fourth nav block. **Guy's
arbitration stands — it is his — but its stated REASON was mine and it was wrong**: the collision is
dissolved by DECISION, not by the reference. *An extraction that misses a block is not a
verbatim extraction, and a decision explained by a false premise is a decision nobody can re-derive.*

### 0a-bis. ✅ RE-ARBITRATED on the corrected measurement (Guy, 2026-08-18)

🔑 **The two facts do not cost the same, and that is what the re-arbitration turns on**: the
**perimeter comes from configuration** (no database), while the **last observation is a
`MAX(observed_at)`** — only the second one touches constraint 1.

**Guy's decision: the perimeter ships, the last observation waits.**

- the nav footer carries **Périmètre {cidr}**, read from `AppConfig` — ⚠️ which means
  `OPENCMDB_SCAN_CIDR` moves from `std::env::var` at `main.rs:334` **into `AppConfig`**, exactly as
  story 6.1's rule requires (*configuration enters as a PARAMETER*; not one of its tests mutates an
  env var);
- **the last observation is registered**, owner story 6b.5 — the dashboard already carries the reach
  section, which is the same family of fact.

🔑 **And this is what makes AC4 TRUE rather than false** (see H1 below): a shell that needs the
perimeter needs a STATE, and that state holds the config and **not the pool** — which is precisely
story 6.1's shape, the one where the compiler refuses `State<MySqlPool>`. *The arbitration that
looked like extra work is what restores the structural carrier.*

### 0a-ter. 🔴 AC4's compiler carrier did NOT transfer, and the validation measured it

The first draft asserted that giving a shell handler `State<MySqlPool>` **fails to compile**, citing
story 6.1's `E0277`. **Measured on this tree: it compiles cleanly.** 6.1's red exists because the
document sub-router's state is `DocumentState`, which holds no pool; the ten screens were to land on
the **main** router, whose state IS the pool (`main.rs:364 .with_state(pool.clone())`), where the
blanket `FromRef` makes the extractor legal — `page::index` already does exactly that.

**So the carrier is not inherited, it is BUILT**: the nine demonstration screens go in their own
sub-router with a pool-free `ShellState`, merged into the main router on `document.rs`'s precedent.
⚠️ **Prove it by adding the extractor and reading the compiler error**, not by citing 6.1 — and note
that **the error CODE is shape-dependent**: on the pool-free sub-router the failure is **`E0308`**
(`expected Router<()>, found Router<Pool<MySql>>`), not 6.1's `E0277`, which arose because
`document_all` already carried a second `State`. *A mutation should predict "a compile failure" and
record the code it gets.*

🔑 **A second carrier of a different kind, measured and worth having**: point the pool at a dead port
and the ten screens still answer **200** while the CONTROL `/gap` answers **500** and `/` answers
**303**. The control is what makes it mean anything. ⚠️ It costs **30 s** on sqlx's default connect
timeout unless the test pool sets a short `acquire_timeout` — set one.

### 0a-quater. Why `/triage` is NOT in that sub-router, and what it saves

🔴 **A measurement the first draft missed entirely: after `/` becomes a redirect, nothing serves
`templates/gap.html`.** `/gap` serves the *fragment*; `gap.html` is the page that links the
stylesheet, loads htmx and hosts the swap target. **So the reconciliation card would VANISH from the
product between this story and 6b.4** — a functional regression, in an epic whose whole point is to
make the product more usable.

**Therefore `/triage` stays on the main router with the pool and renders today's card inside the new
shell.** It is the one fed screen of the ten (`epics.md` gives 6b.4 the real gap), so the split is
not a workaround: **nine demo screens with no pool, one fed screen with one.** Constraint 1 is about
demo screens, and it is satisfied exactly where it applies.

The story put three costed resolutions to Guy for a collision between the epic's own AC (*a header
carrying "last observation"*) and the epic's own constraint 1 (*no demo screen opens a connection*).
**Guy took none of the three: he took the mock's header, which carries neither fact.**

So there is **no dynamic half**, **no `MAX(observed_at)` reader**, **no `OPENCMDB_SCAN_CIDR` moved
into `AppConfig`** — and constraint 1 holds without an aménagement, because the shell reads nothing
at all. 🔑 *The cheapest way to satisfy two contradictory requirements was to notice that one of them
was not in the reference.*

⚠️ **What this costs, stated rather than buried: `epics.md`'s AC for this story is NOT met as
written.** It asks for four things — brand, tagline, perimeter, last observation — and **TWO ship in
the header** (brand, tagline), a **third arrives in the nav footer** (the perimeter, §0a-bis), the
fourth is registered, **and a fifth slot the AC never asked for is added** (the version). 🔴 The
first draft wrote *"four asked, three ship"*, wrong twice over: it counted the version among the
four, and it had not yet placed the perimeter. **Registered**, owner Epic 6b's retrospective.

### 0a-quinquies. ✅ ARBITRATED — no responsive in this story (Guy, 2026-08-18)

Raised by the validation as a **fourth collision of the same family** as the nav, the header and
`device`, and not seen at contexting. Measured: the mock has **zero `@media` rules** and **zero skip
links**, and is a fixed `grid-template-columns: 208px minmax(0, 1fr)`. The UX spec mandates
mobile-first with breakpoints at ≤360 / 768 / 1280 (`:1530-1545`), a mobile bottom nav with a
permanent search magnifier, skip links (`:1568`), and a left nav that *"collapses to a bottom bar /
drawer on mobile"* (`:841`).

**Guy: not now.** The shell ships desktop-only, as the mock has it. ⚠️ **Registered**, and the
register row's owner is no longer *"the retrospective unless Guy scopes it"* — it is **deferred by
decision**, to be scoped when someone opens the product on a telephone or when the release story
needs to describe it.

🔑 **What this changes in the numbers**: the gap-hunt's 18 rules included one breakpoint, so the
committed shell is nearer **15**, and its *"30–40 built to the spec"* is now a future story's figure
rather than this one's. §5's arbitration is unaffected either way.

⚠️ **What it does NOT change**: the ≥44px touch-target obligation (NFR24) against the mock's ~30px
nav entries is **not** a responsive question — it applies to a desktop pointer and a touch screen
alike. It stays registered with story 6b.11, untouched by this decision.

### 0b. The version string, and the one interpretation this story takes on its own

The mock reads `v0.1.1 · maquette`. Guy: *replace the mock's version number with the release's*.
**Taken as `env!("CARGO_PKG_VERSION")`, read at compile time** — `0.1.1` today, and `0.2.0` by itself
the day story 6b.12 bumps `Cargo.toml`. A number hardcoded in a template is a number that lies on a
release day, and `metrics.rs:22` already reads it that way, so the idiom exists in this crate.
The word *maquette* does not ship.

---

## 1. ✅ ARBITRATED — the mock's navigation, and what that retires

Extracted from the mock's `<nav>` and counted: **ten entries in three groups**, and **"Topologie"
appears ZERO times in its 496 KB** (all four spellings checked).

| Group | Entries (`data-screen` → label) |
|---|---|
| **Boucle** | `triage` → *Triage* · `dashboard` → *Tableau de bord* |
| **Inventaire** | `devices` → *Appareils* · `device` → *Fiche appareil* · `apps` → *Applications* · `ipam` → *IPAM* |
| **Machine** | `sources` → *Sources* · `alerts` → *Alertes* · `diagnostic` → *Auto-diagnostic* · `onboarding` → *Mise en service* |

🔴 **THREE documents prescribe the six-entry nav, not one** — `epics.md:278` (UX-DR33) and the UX
spec **twice**, at `:836-838` and `:1308`. The first draft cited only the first, which would have
left a retrospective correcting one document of three. **The mock retires Topology in all three.**
The decision is Guy's and it is recorded; what must not happen is the retirement being *silent*.
**Registered**, owner Epic 6b's retrospective. 🔑 State it precisely so it is misread in neither
direction: UX-DR33's own sentence already calls interactive graphical topology **Growth**, so what
is lost is a nav entry to a screen no epic in this plan builds — not a feature, and not an oversight.

The five entries the mock adds that UX-DR33 does not name (`device`, `sources`, `alerts`,
`diagnostic`, `onboarding`) need no arbitration: stories 6b.6–6b.9 already build them.

---

## 2. ✅ ARBITRATED — the header is the mock's, and it reads nothing

```
opencmdb   ·   observé vs déclaré — l'écart est le produit   ·   v{CARGO_PKG_VERSION}
```

Three slots, all static. See §0a for what this dissolves and §0b for the version.

⚠️ The perimeter and the last observation are **not lost, they are unplaced**: the perimeter is a
commissioning fact (story 6b.9, *Mise en service*) and the last observation is a dashboard fact
(6b.5, which already carries the reach section). **Registered so neither is rediscovered as missing.**

---

## 3. ✅ ARBITRATED — `device` behaves as the mock does, and the debt is named

The nav carries `device` (*Fiche appareil*) as a peer of `devices`, which in the mock is a
click-through artefact: it shows the screen without a device existing. Guy: **do as the mock does for
now.** So the route is `/device`, rendering the shell like the other nine.

🔴 **What that owes, and it must be written down rather than felt later**: this story's own AC
promises *"each screen has its own address … I can link to one, bookmark it"*, and **`/device` is the
one entry for which that promise is hollow** — it addresses no device. The honest shape is
`/devices/{id}`, and it needs an id, which needs either 6b.3's example dataset or Epic 6's real
devices. **Registered**, owner story 6b.6 (*Inventory and device record*), which is where the screen
gains content and the choice becomes concrete.

---

## 4. ✅ ARBITRATED — `/` redirects to `/triage`

Guy's shape, and it is better than the one this story first recommended: **the redirect is separate
from the screen**, so every one of the ten keeps exactly one address and `aria-current` has one case
rather than two.

**The target is `/triage` rather than `/dashboard`**, on a measurement rather than a taste: `epics.md`
makes **6b.4's triage the screen fed by the REAL gap** (the phrase *"Today's card becomes the mock's two-pane
triage"* is the change proposal's, `sprint-change-proposal-2026-08-13.md:143`, not `epics.md`'s; the
substance is at `epics.md:2172-2182`) while **6b.5's dashboard is mixed by construction** — the real reach section beside example
stat cards and sparklines. Redirecting `/` at the dashboard would land every visitor, and every
existing link in the README, the manuals, the landing site and Docker Hub, on the product's **most
half-demonstration screen** — which is exactly the risk the change proposal names: *"ten screens of
which eight are examples is a product that looks far more finished than it is"*, whose only defence
is a marker aimed at **the person who installs it**. That person arrives at `/`.

🔑 **And the arbitration is cheap to revisit precisely because of Guy's shape**: a redirect's target
is one line. **Registered** — owner story 6b.5 — to re-examine the target when the dashboard stops
being mixed.

⚠️ `/gap` keeps working unchanged: story 5.14b's reach sections and the HTMX refresh ride it, and the
AC says HTMX swaps fragments *within* a screen. **The bookmark sweep** (README, the two manuals, the
`gh-pages` landing site, `docker/README.dockerhub.md`) is **registered for 6b.12**, not discovered.

---

## 5. ✅ ARBITRATED — no Tailwind chain, again, and the criterion is now explicit

`deferred-work.md` names this story as the owner of 6b.1's withdrawn AC1/AC5/AC6, on the criterion
*"the first screen story that writes a utility class"*. **This story writes none: Guy's decision is
hand-authored, as 6b.1.** ⚠️ **The first draft said "of the order of ten rules" and the gap-hunt
built it: 18 rules, 60 declarations, 115 lines — `app.css` 297 → 413, +39%.** And 18 is the FLOOR:
it has one breakpoint where the UX spec asks for three, no skip link, no mobile bottom nav and no
nav footer, so built to the spec it is comfortably 30–40. 🔑 **The arbitration survives the
correction and that is why the figure is worth fixing**: a build chain whose output is 18
hand-written rules is still not earning a ninth gate — but the next Tailwind conversation must start
from a measured number rather than from ten.

⚠️ **The register row is re-owned rather than discharged**, and its criterion is sharpened from *"the
first screen story"* (which this one is, and which turned out not to be the deciding property) to
**"the first story that needs a utility the hand-authored sheet cannot express"** — plausibly 6b.4 or
6b.6. Its four measured spellings stay attached, and the sharpest one for whoever lands it:
**preflight ALONE changes ten computed styles and collapses the first-boot `<h1>`.**

---


## 6. What the shell must NOT break

Measured on `master` at `b1ce1a5`:

- **`is_public` is `/healthz` + `/assets/*` only** (`auth.rs:72-74`), so all ten screens sit behind
  HTTP Basic. Story 6.1 shrank it deliberately; **this story must not widen it**, and the pin at
  `auth.rs:173` is what would catch that.
- **`page.rs`'s `reconcile_view`** feeds both `index` and `gap_fragment`, and story 5.14b's identity
  reach pair reads through them. ⚠️ The first draft added *"and story 6.2's J3 end-to-end test"* —
  **false**: that test (`main.rs:1282`) hits only `/document-all` and asserts through
  `gap::project`/`gap::reconcile` in-process. It never requests `/` or `/gap`.
- **28 fixtures, the trap gate RED at 26/15/11**, eight gates. The shell touches none of it.
- `templates/` holds **two** files. This story adds the shell partials; the house convention is
  composable partials (UX spec), and `askama` resolves them from the crate manifest (D55).

---

## Acceptance Criteria

Derived from `epics.md`'s three bullets and **scoped by §0's five arbitrations**. AC5–AC9 are this
story's own additions.

⚠️ **Before AC1: the shell may NOT use the amber.** 6b.1 pins `var(--accent-document)` at **exactly
zero** uses until story **6.4**, which lands after this whole epic. The natural focus ring and the
natural `aria-current` marking both reach for the accent — and the mock uses its blue for exactly
that. **Mark the current entry and the focus ring with the neutrals** (`--color-neutral-200` was
measured working) or with `--color-accent`, the mock's structural blue. *Meeting a red guard and
widening it is the failure mode; this sentence exists to prevent it.*

**AC1 — the frame, identical on all ten screens.**
Every route renders the mock's header — **brand · tagline · `v{CARGO_PKG_VERSION}`**, three static
slots, the word *maquette* absent — and the navigation over the **ten** entries in the mock's **three
groups**, with the current entry marked `aria-current="page"`.
Tests: the entry count is **exactly ten**; the group count is **exactly three**; **exactly one**
entry carries `aria-current` on each of the ten screens (not *at least one*); the version rendered
equals `env!("CARGO_PKG_VERSION")` and the template contains no literal version number.
🔴 **The header carries no perimeter and no last observation; the NAV FOOTER carries the perimeter**
(§0a-bis). ⚠️ **Do not write the absence guard over the `<header>` element**: the gap-hunt measured
that a test asserting *"the header carries neither fact"* passes over a shell that carries **both** in
the sidebar — *the guard placed exactly where the defect cannot occur*. The assertion that means
something is over the WHOLE shell: **`MAX(observed_at)` appears nowhere and no demo handler can
reach the database** (AC4), while the perimeter is present and comes from `AppConfig`.

**AC2 — one URL per screen, server-rendered, deep-linkable cold.**
Ten routes render server-side. **No client-side router and no screen chosen by JavaScript**: a test
renders each route in isolation and finds that screen's own marker, and `app.js` gains no
screen-switching code — measured baseline, **zero occurrences of `router` or `screen` in its 482
bytes**. HTMX swaps fragments **within** a screen only.

**AC3 — all ten are offered from the first day, and none is quietly withheld.**
⚠️ **This story must not hide, disable, grey or empty an entry.** The honesty of showing ten screens
that are mostly empty is story 6b.3's subject, and pre-empting it here would satisfy this AC while
destroying the next story's.
🔑 **Write this guard as a PROPERTY, and state its blind spots — which the gap-hunt MEASURED by
attacking a real implementation ten ways.** Eight gestures were caught (`hidden`, `aria-disabled`,
inline `display:none`, `href="#"` on all ten, an entry behind a false conditional, an entry dropped
from the route table, `<a>` → `<span>`, an href pointing at no route). **Two came back GREEN:**

| | gesture | why the guard cannot see it |
|---|---|---|
| **A4** | `.nav-entry[href="/apps"] { display: none }` **in `app.css`** | it is in the STYLESHEET, not the markup — and in a hand-authored-CSS story that is exactly where anyone would write it |
| **A9** | inline `pointer-events: none; opacity: .4` | the entry is visible, correctly href'd and route-backed; it simply cannot be clicked |

🔴 **A guard covering all ten is NOT writable at the template-text level**, and the honest sentence
belongs in the story rather than in a dev's head: *the guard covers the markup; the stylesheet and
interaction-blocking are outside it, and closing them needs computed styles — axe-core or a headless
browser over the ten routes.* Anything short of that loses to `opacity:0`, `font-size:0`,
`height:0;overflow:hidden`, `clip-path:inset(100%)` or a `@media` block.
⚠️ Two corrections to the first draft's list of six: **`disabled` is not valid on `<a>`** — the real
spelling is `aria-disabled` — and `display:none` / `visibility:hidden` are **one** class, while the
CSS-file location is a genuine second class the draft had merged away.

**AC4 — the shell reads NOTHING, and the compiler holds it.**
None of the ten handlers takes `State<MySqlPool>`. 🔑 Story 6.1 measured this carrier: adding the
extractor **fails to compile** (`E0277` on the `Handler` bound). That is stronger than a test, and
after §0a it covers the whole shell rather than the demo screens alone — **epic constraint 1 is met
structurally, not by discipline.**

**AC5 — `/` redirects to `/triage`, the card does not disappear, and every bookmark resolves.**
`/` answers a 303 to `/triage`; `/triage` renders **today's reconciliation card inside the new
shell** (§0a-quater); `/gap` is unchanged and still serves the fragment with story 5.14b's two reach
sections.
🔴 **One existing test WILL red, and it is invisible locally**: `main.rs:1173`
`index_renders_the_real_gap` does `GET /` and asserts `200` plus rendered content. It is
`DATABASE_URL`-gated, so on a dev machine it passes **by returning** and only CI would catch it —
the exact trap this story's own Dev Notes warn about. **Update it deliberately**; do not discover it
in CI.
🔴 **And a second CI red the first draft did not see at all**: with `/` redirecting and `/gap`
serving only the FRAGMENT, nothing routes to `page::index` / `GapPage` / `gap.html` any more, so
`cargo clippy -- -D warnings` fails with *"function `index` is never used"* — measured. §0a-quater's
decision (`/triage` renders today's card inside the shell) is what keeps `index` alive **and** keeps
the product's only fed screen reachable; without it this story would silently empty the one screen
that was full, in an epic whose purpose is to make the product more usable.

**AC6 — every label is a key, in both locales.**
The ten entries, the three group headings and the nav footer's perimeter label live in
`locales/app.yml` under **both** `fr` and `en` — **13 new keys, not the ~15 first estimated**:
⚠️ `page.tagline` **already exists** with the mock's French string byte-identical
(`locales/app.yml:5-7`), and `gap.html` already renders brand + tagline. **Two of the header's three
slots ship today** — reuse them; a new `nav.tagline` would duplicate an existing key. Baseline measured: **32 top-level entries, and not one is missing a locale**
— so the guard is *"no key regresses to a single locale"*, asserted over the whole file rather than
over the new keys. 🔴 **And a third guard the first draft did not have, measured: a TYPO'D KEY SHIPS AS VISIBLE PAGE
TEXT.** `rust-i18n` renders a missing key as **its own name** — no panic, not empty — so
`nav.apps` → `nav.appz` put the literal string `nav.appz` in the navigation with **all guards
green**, confirmed on the wire. AC6 needs *"every key a handler references resolves"*, which neither
YAML completeness nor template ASCII-ness expresses.
⚠️ **The ASCII half must be narrowed to non-ASCII LETTERS**: `gap.html` and `_gap_card.html` already
carry `—` and `·`, so *"no bare non-ASCII"* reds on the committed tree, over two files this story may
not touch. *Typography is not copy, and `is_ascii()` cannot draw that line.*

**AC7 — `is_public` is unchanged**: ten new screens, zero new public paths.
🔴 **The pin at `auth.rs:173` does NOT carry this, measured**: its gated set is an ENUMERATION —
`["/", "/gap", "/metrics", "/document-all", "/healthzz", "/assets"]` — and none of the ten new paths
is in it. Making `/devices` public leaves **all 371 tests green**. The project's dominant defect
class, found in a story that names it twice. **The ten paths must be added to that enumeration**, or
better, the pin re-shaped as a property over the router's own path list, so the eleventh screen is
covered without anyone remembering.

**AC8 — the register, in BOTH directions.**

*Rows this story writes*: UX-DR33's retired Topology entry (in **three** documents, not one);
`epics.md`'s header, of which two ship in the header, one moves to the nav footer, one is deferred
and a fifth slot is added; `/device`'s hollow bookmark promise (owner 6b.6); the redirect target to
re-examine (owner 6b.5); the bookmark sweep (owner 6b.12); the last observation (owner 6b.5); the
Tailwind row re-owned with its sharpened criterion.

🔴 *Rows the register ALREADY NAMES THIS STORY AS OWNER OF, and the first draft addressed one of
four* — found by the gap-hunt reading the register rather than the story:
- **`:3608`** — the **four documents** that describe a Tailwind chain which does not exist
  (`.gitignore:40`, `CLAUDE.md`'s stack line, `docs/project-context.md:279`, `xtask/Cargo.toml:2-3`,
  which also announces a `recapture` subcommand that does not exist). §5 defers the chain **again**,
  so these are **due now**;
- **`:3616`** — *"`assets/` is a public unauthenticated namespace … decide it rather than inherit
  it"*;
- **`:3641`** — **the RADIUS divergence**: *"either it uses the mock's steps and the spec sentence is
  corrected, or it keeps 3px and the three tokens are deleted rather than carried for ever."* A
  shell that writes no `border-radius` takes **neither** branch, silently.

🔑 **THIS IS THE THIRD CONSECUTIVE STORY to miss a register row that names it by number**, and the
register says so at `:3629` — *"the question is not the row but why a register searched by hand keeps
missing the rows that name the searcher."* ⚠️ **AC8's instruction was right and its SCOPE was wrong**:
it told the dev to verify what they *wrote*, when the failure mode is what the register *already says
they owe*. **`grep -n "6b.2" deferred-work.md` before starting, and again before finishing.**

**AC8b — 🔴 story 6b.1's two repaired guards must SEE the templates this story adds.**
`page.rs:1562` hardcodes `fn templates() -> [&'static str; 2]`, and 6b.1's review rewrote the
`data-theme` and `--accent-document` guards as properties *"over the sheet AND both templates"*.
**Measured by the gap-hunt: plant `data-theme="dark"` in `_shell.html` and
`style="color: var(--accent-document)"` on all ten nav entries, and 607 tests stay GREEN** — confirmed
on the wire with `curl`. A typed literal array does not fail to compile when a template is added, so
**nothing forces the update**. This story doubles the template count and must therefore make that
helper enumerate `templates/` rather than list it — and a test must red when a template is added and
not scanned. 🔑 *A guard repaired yesterday is undone by the ordinary act of adding a file.*

**AC9 — the live test count lives HERE**, in this file (story 6.1's AC8 rule, F2), and is not copied
into `CLAUDE.md`, `docs/project-context.md` or `sprint-status.yaml`.


## Tasks / Subtasks

**Scoped by §0.** No Tailwind, no chain, no `xtask` subcommand; no database read anywhere in the shell.

- [ ] **T1 — the shell** (AC1, AC6): `_shell.html` + `_nav.html` partials; ten entries in three
      groups; `aria-current`; the header's three static slots with `CARGO_PKG_VERSION`
- [ ] **T2 — the ten routes** (AC2, AC4): 🔴 **nine demo screens in a `Router<()>` sub-router merged
      AFTER `.with_state(pool)`** — the shape IS the carrier, and forbidding the extractor without it
      is not enforceable (measured). `/triage` stays on the main router with the pool and renders
      today's card (§0a-quater). `/device` as the mock has it
- [ ] **T3 — `/` and `/gap`** (AC5): 303 to `/triage`; `/gap` untouched
- [ ] **T4 — the copy** (AC6): **13** keys (`page.tagline` exists and is reused), `fr` + `en`, plus
      the guard that a referenced key RESOLVES — a typo renders as visible page text
- [ ] **T4b — the perimeter into `AppConfig`** (§0a-bis): `OPENCMDB_SCAN_CIDR` moves out of
      `std::env::var` at `main.rs:334`; no test may mutate an env var (story 6.1's rule)
- [ ] **T4c — `page.rs`'s `templates()` must enumerate `templates/`, not list two files** (AC8b), or
      6b.1's two repaired guards go blind on every partial this story adds
- [ ] **T5 — the guards** (AC1–AC7), each written to red before it passes, and AC3's written as a
      PROPERTY with its blind spots stated
- [ ] **T6 — look at all ten screens** in a browser. ⚠️ *A status code is not a look* — 6b.1's T6
      logged HTTP statuses and called it looking, and the review said so. 🔑 **Export
      `OPENCMDB_LOCALE=fr` first**: the default locale is `en` (`main.rs:291`), so an unset locale
      compares *Triage / Dashboard / Devices* against a French mock — the wrong comparison
- [ ] **T7 — the register** (AC8), in BOTH directions: the rows this story writes, **and the three
      the register already owes it** — the four stale chain documents (due now), the `assets/`
      namespace decision, and the radius branch. 🔴 **`grep -n "6b.2" deferred-work.md` before
      starting and before finishing** — third consecutive story to miss a row naming it
- [ ] **T8 — prove-to-red**, predictions written FIRST

## Prove-to-red — the mutations this story owes

| # | Mutation | Prediction |
|---|---|---|
| M1 | Drop one nav entry | AC1/AC3 red — the count is exactly ten |
| M2 | Mark two entries `aria-current` | AC1 red — *exactly one*, never *at least one* |
| M3 | Give a **demo** screen handler `State<MySqlPool>` | **fails to COMPILE** — but only once the nine demo routes sit in a pool-free sub-router (§0a-ter). 🔴 **Measured GREEN on the main router**, where the state IS the pool: the carrier is BUILT by this story, not inherited from 6.1. Read the compiler error rather than citing 6.1 |
| M4 | Add a screen-switching branch to `app.js` | AC2 red |
| M5 | `hidden` on the entry whose screen is emptiest | AC3 red — **the shape this story is most likely to take by accident** |
| M5b | `href="#"` on one entry (the mock's own spelling!) | AC3 red — ⚠️ the mock writes `href="#"` on all ten, so this is the mutation the mock itself would introduce |
| M6 | Remove one `en` key, leaving `fr` | AC6 red |
| M6-bis | Remove one **`fr`** key, leaving `en` | 🔴 **the dangerous direction, and the one the first draft missed**: `rust-i18n` falls back to `en`, so the French UI silently renders English — an NFR26 violation the renderer never betrays |
| M6-ter | Typo a key name in a handler (`nav.apps` → `nav.appz`) | AC6 red — **measured GREEN before this guard existed**, with `nav.appz` rendered as the entry's visible label |
| M14 | Add a template under `templates/` and leave `templates()` listing two | AC8b red — otherwise 6b.1's `data-theme` and `--accent-document` guards go blind (measured: 607 tests green with both violations planted) |
| M15 | `.nav-entry[href="/apps"] { display: none }` in `app.css` | ⚠️ **GREEN by stated limit** — AC3 covers the markup, not the cascade. Recorded as a limit rather than hidden |
| M7 | Add `/devices` to `is_public` | 🔴 **measured GREEN today** — the pin enumerates six gated paths and none of the ten is among them, so all 371 tests pass with `/devices` public. AC7's guard must be widened before this row can be believed |
| M8 | Break `/gap`'s fragment | AC5 red |
| M9 | Hardcode `v0.2.0` in the header template | AC1 red — the version must come from the crate |
| M10 | Put a `MAX(observed_at)` read into a demo screen | AC4 red at compile time, for the same reason as M3 — and note it is the LAST OBSERVATION specifically: the perimeter is config and legal (§0a-bis) |
| M12 | Read the perimeter from `std::env::var` in the handler instead of `AppConfig` | a test reds — story 6.1's rule is that configuration enters as a PARAMETER, and no test may mutate an env var |
| M13 | Serve `/triage` without the reconciliation card | AC5 reds — §0a-quater's regression, which the first draft did not see |
| M11 (control) | Reorder two entries within a group | green — order within a group is pinned by nothing, deliberately |

⚠️ **M5b is the one to design carefully**: the mock's own markup is `<a href="#" data-screen="…">`,
because its navigation is JavaScript. Transcribing the mock faithfully therefore PRODUCES the defect
AC3 forbids — *the reference itself is the adversary here*, which is why M5b is a row rather than a
footnote.

## Dev Notes

### The tree this story extends (measured 2026-08-18, `master` at `b1ce1a5`)

- `main.rs:359-363` — five routes. `/` → `page::index`, `/gap` → `page::gap_fragment`,
  `/assets/{*path}`, `/metrics`, `/healthz`.
- `auth.rs:72-74` — `is_public` = `/healthz` + `/assets/*`; the pin is at `:173`.
- `page.rs` — 516 code lines; `reconcile_view` feeds both the page and the fragment.
- `templates/` — `gap.html`, `_gap_card.html`. **Two files.**
- `assets/app.css` — 297 lines, hand-authored on the mock's tokens (story 6b.1). `assets/app.js` —
  482 B, focus management, and **zero occurrences of `router` or `screen`** (measured) — so AC2's
  *"no screen chosen by JavaScript"* is TRUE today and the guard's job is to keep it true.
- `locales/app.yml` — **32 top-level entries** (31 keys plus a `_version` marker), and **`fr` and
  `en` are both complete: zero key is missing a locale**, measured. That is AC6's baseline and it
  means the guard to write is *"no key regresses to one locale"*, not *"add the missing ones"*.
- Eight gates, 28 fixtures, trap gate RED 26/15/11.

### Traps, each measured on this project

- 🔴 **A guard placed where the defect cannot occur reads as coverage and is none** — Epic 5's
  dominant class, and story 6b.1 shipped **five** of them, each defeated by an *ordinary* gesture.
  For this story the exposed guards are AC3's (an entry can be hidden six ways: `hidden`, `disabled`,
  `display:none`, an empty `href`, a comment, or simply not being in the list) and AC2's (a "no
  client router" check that greps for the word `router` proves nothing). **Write properties, not
  enumerations.**
- 🔴 **The mutation driver lies** — `cargo test --workspace A B` passes two filters where cargo
  accepts one, and nothing runs. Read every red from its own panic message.
- ⚠️ **Verify `fmt` AFTER any hand-edit made for clippy.** Story 6b.1 shipped a tree that would have
  reddened CI because clippy was fixed by hand after `cargo fmt` and only clippy was re-run.
- ⚠️ **`cargo build` does not see a NEW file under `assets/`** — it ships silently stale; `touch
  src/page.rs` first. Applies to any new partial or asset.
- ⚠️ **Without `DATABASE_URL` the suite passes by RETURNING** — the tell is elapsed time (~5 s vs
  ~0.05 s), never the count.
- 🔴 **A prescribed check nobody runs is worth what no check is worth** (story 6b.1, T7 above).

### Stack — from the tree, not from memory

axum 0.8 · askama 0.16 (templates resolve from the crate manifest, D55) · `rust-i18n` (YAML) ·
htmx 2.0.4 at `assets/vendor/htmx-2.0.4.min.js` · Barlow / Barlow Condensed embedded (6b.1).
**Do not invent a version — read `Cargo.lock`.**

### ✅ VALIDATION, layer 1 (fact-check) — 2026-08-18, 37 claims re-measured, **8 refuted**

Four HIGH, and **two of them were established by running a mutation rather than by reading** — both
against guards this story had named as its strongest carriers:

- 🔴 **AC4's compiler carrier did not transfer** (§0a-ter): giving a shell handler `State<MySqlPool>`
  **compiles cleanly** on the main router. Story 6.1's `E0277` depended on a pool-free state, and
  this story restated the conclusion without its precondition. The carrier is now BUILT.
- 🔴 **AC7's named pin does not carry** (see AC7): `/devices` made public leaves **371 tests green**,
  because the pin enumerates six gated paths and none of the ten new ones is among them.
- 🔴 **The premise of arbitration 2 was false** (§0a): both facts ARE in the mock, in the nav footer.
  Re-arbitrated by Guy on the corrected measurement.
- 🔴 **The redirect reds a test that is invisible locally** (AC5), and would have surfaced only in CI.

Four MEDIUM: *"four asked, three ship"* was wrong twice over; the six-entry nav is prescribed in
**three** documents, not one; the claim that story 6.2's J3 test reads through `reconcile_view` is
false; and `page.tagline` **already exists**, byte-identical to the mock, so the copy is 13 new keys
rather than ~15. Plus four LOW, including a quote attributed to `epics.md` that belongs to the change
proposal.

✅ **What held**: the ten entries, their keys, labels, order and three group headings — character for
character; *"Topologie"* absent from the whole file and all 19 decoded resources; `href="#"` on all
ten (M5b's premise); every figure in §6; and **all six register rows exist with the owners claimed —
story 6b.1's defect is not repeated.**

### ✅ VALIDATION, layer 2 (gap-hunt) — 2026-08-18, the shell was BUILT and attacked

It wrote the whole thing — two partials, ten handlers, the redirect, 13 locale keys, 115 lines of
CSS, nine guards — and ran it against a live `mariadb:10.11.11`. **Seven HIGH, five MEDIUM**, and
four of them were invisible to the reading layer:

- 🔴 **AC4 measured false**, independently of the fact-check, **plus the shape that restores it** and
  the correction that the error code is `E0308`, not `E0277` (§0a-ter);
- 🔴 **`page::index` becomes dead code and `clippy -D warnings` FAILS** — a CI red on day one, behind
  which the product's only fed screen would have become unreachable until 6b.4 (AC5);
- 🔴 **story 6b.1's two repaired guards go blind on the new templates**: both violations planted,
  **607 tests green**, confirmed on the wire (AC8b);
- 🔴 **AC3 attacked ten ways: eight caught, two green**, and neither is on the draft's list of six
  (AC3);
- 🔴 **a typo'd i18n key ships as visible page text** with every guard green, and **M6 pointed the
  wrong way** — the fallback is `en`, so it is the FRENCH half whose loss is silent (AC6);
- 🔴 **the shell may not use the amber**, pinned at zero uses until story 6.4 (before AC1);
- ⚠️ the CSS is **18 rules, not ten** — 30–40 built to the spec — which corrects §5's figure **without
  overturning its arbitration**;
- ⚠️ **three register rows already name this story as owner** and the draft addressed one (AC8);
- ⚠️ **nobody owns axe-core** on the ten routes the epic's DoD requires it for — registered, and it is
  the same gap as AC3's two blind spots.

✅ **What it confirmed by measurement**: M5b's premise (`href="#"` on all ten); the ten entries, three
groups and absent Topology; **every figure in §6** — *"story 6b.1's record was wrong four ways; this
one is clean"*; that `aria-current` survives a real render, with the trap that computing `current` in
the template puts it where no unit test reaches; and that the redirect behaves for a browser GET
(303, `location: /triage`, `curl -L` lands on 200).

🔑 **One thing it could not do**: no headless browser in the worktree, so AC3's *"closing the two
blind spots needs computed styles"* rests on the structure of the problem rather than on a checker
measured failing. Stated as such rather than as a measurement.

### Validation obligations for dev

1. **§2's collision is the one to attack first.** Verify independently that the header's *last
   observation* cannot be rendered on a demo screen without violating constraint 1 — and if a fourth
   resolution exists, that is the finding.
2. **BUILD the shell before judging it.** Five stories running, the layer that built found what the
   layer that read did not.
3. **Attack AC3 as an adversary would**: how many ways can an entry be present-but-not-offered? The
   guard must catch all of them or state which it does not.
4. **Re-measure every figure in §6** — story 6b.1's record was wrong four ways and the audit layer
   found all four.
5. Confirm §1's UX-DR33 divergence is real by reading `epics.md:278` and searching the mock yourself.

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 6b's premises and constraints; story 6b.2's three
  criteria; **`:278` UX-DR33**, the six-entry nav naming Topology
- `_bmad-output/implementation-artifacts/deferred-work.md` — the Tailwind chain with its four
  measured spellings (owner: this story), the radius row, the OFL row
- `_bmad-output/implementation-artifacts/6b-1-design-system-tokens-and-accent.md` — the design system,
  and the five hollow guards its review found
- `_bmad-output/planning-artifacts/ux-design-specification.md` — the shallow-IA nav, the responsive
  collapse, composable partials
- The mock: `~/travail/Projets actuels/opencmdb/documents/opencmdb - maquette.html` — **outside this
  repository**, 496 068 B; its `<nav>` and `<header>` are extracted verbatim in §1 and §2

---

## Dev Agent Record

### Agent Model Used

_(filled by dev-story)_

### Debug Log References

### Completion Notes List

### File List

### Change Log

| Date | Change |
|---|---|
| 2026-08-18 | Story contexted. §1 found the mock's nav contradicts UX-DR33 and drops Topology; §2 found the AC's header collides with the epic's own constraint 1; §3 found a nav entry with no addressable URL; §4 and §5 put `/` and the Tailwind chain to Guy. |
