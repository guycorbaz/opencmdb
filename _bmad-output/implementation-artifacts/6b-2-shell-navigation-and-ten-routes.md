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

## 1. 🔴 The mock's navigation contradicts a RECORDED UX decision, on two axes

Measured, by extracting the mock's `<nav>` and counting: **ten entries in three groups**, and
**"Topologie" appears ZERO times in the mock's 496 KB** (all four spellings checked).

| Mock's group | Entries (`data-screen`, label) |
|---|---|
| **Boucle** | `triage` *Triage* · `dashboard` *Tableau de bord* |
| **Inventaire** | `devices` *Appareils* · `device` *Fiche appareil* · `apps` *Applications* · `ipam` *IPAM* |
| **Machine** | `sources` *Sources* · `alerts` *Alertes* · `diagnostic` *Auto-diagnostic* · `onboarding` *Mise en service* |

Against that, **UX-DR33** — a recorded UX decision, `epics.md:278` — prescribes:

> *"Shallow left-nav (Inbox · Dashboard · Devices · IPAM · Applications · **Topology**)"*

🔴 **Six entries against ten, and the one UX-DR33 names that the mock does not have is Topology.**
The epic's AC says *"the navigation over the **ten** entries"*, so the mock wins by Guy's decision (3)
of 2026-08-13 — and the consequence is that **the topology loses its navigation entry while a
recorded decision still names it**. That is not this story's call to make silently:

- ⚠️ **REGISTER it** (a story may not edit `epics.md` or the UX spec), owner Epic 6b's retrospective;
- the five entries the mock adds and UX-DR33 does not name (`device`, `sources`, `alerts`,
  `diagnostic`, `onboarding`) are the same divergence in the other direction, and they are what
  stories 6b.6–6b.9 build. They need no arbitration — the epic's decomposition already assigns them.
- 🔑 *The interactive graphical topology is Growth by UX-DR33's own sentence*, so what is lost is a
  nav entry to a screen nothing builds in this epic. **State that, so nobody reads the omission as an
  oversight and nobody reads it as "topology is cut".**

---

## 2. 🔴 The AC's header asks for two facts the mock does not show, and one of them collides with the epic's OWN constraint 1

The AC: *"it carries the header (brand, tagline, **perimeter**, **last observation**)"*. The mock's
header, extracted verbatim:

```
opencmdb  ·  observé vs déclaré — l'écart est le produit  ·  v0.1.1 · maquette
```

So **brand and tagline are the mock's; perimeter and last observation are the AC's own addition**,
and the mock's third slot is a version string ending in the word *maquette* — which obviously does
not ship.

Where those two facts would come from, measured:

- **perimeter** — `OPENCMDB_SCAN_CIDR`, read at `main.rs:334` inside the startup path. It is **not**
  in `AppConfig` (story 6.1 put the document switch and the Basic pair there, not this) and it is
  read via `std::env::var` at the point of use. ⚠️ Story 6.1's rule is that **configuration enters as
  a PARAMETER** — *"not one new test mutates an env var"* — so putting `OPENCMDB_SCAN_CIDR` in the
  header means moving it into `AppConfig` first, which is a real change to the composition root.
- **last observation** — a `MAX(observed_at)` over `observation_record`. There is no such reader
  today; `repo.rs` reads facts and links, not a global maximum.

🔴 **AND HERE IS THE COLLISION.** Epic 6b's **constraint 1** says, in its own words:

> *"the example dataset lives in the handler/template layer, and **no demo screen opens a
> connection**"*

The header renders on **every** screen, eight of which are demonstrations in this epic. A header
carrying *last observation* is a database read. **So the AC and the constraint cannot both hold as
written**, and the three honest resolutions are:

| | Resolution | Cost |
|---|---|---|
| **(a)** | The header's dynamic half renders **only on fed screens**; demo screens carry brand + tagline + version | the frame is not identical across screens, which is a visible inconsistency — and it is arguably HONEST, since a demo screen has no last observation |
| **(b)** | The shell always reads, so every screen touches the database | **violates constraint 1 literally**, and constraint 1 exists because story 5.12 had to sanction `docker/seed-example.sql` as a write site — the promise is that a demo cannot be mistaken for real data |
| **(c)** | The header's dynamic half is **hoisted out of the shell** into the fed screens' own content | the AC's word *"header"* is then not met as written; the facts appear, one level down |

**Recommendation: (a)** — it keeps constraint 1 intact, and the difference is not a defect but the
truth (*a demonstration has no last observation*). ⚠️ **Guy's call, and it must be taken BEFORE dev**:
each option changes the shell's signature, and (b) would put a database read on the path of every
example screen for the rest of the epic.

---

## 3. `device` is a DETAIL screen with a navigation entry, and it has no address

The mock's nav carries `device` — *Fiche appareil* — as a peer of `devices`. In the mock that is an
artefact of a click-through demo: it shows the screen without needing a device to exist.

🔴 **The AC's promise is *"each screen has its own address … I can link to one, bookmark it"*, and a
device record without an identifier cannot satisfy it.** `/device` bookmarks nothing; `/devices/{id}`
bookmarks a device that does not exist yet (Epic 6's L2 grouping is what mints them, stories
6.5–6.19).

The three shapes, and only the third keeps both promises:

1. `/device` rendering a fixed example — the mock's own behaviour, and it makes the URL a lie the day
   real devices exist;
2. `/devices/{id}` with no id reachable — the nav entry then links to nothing, which the epic's AC
   forbids (*"the navigation shows all ten from the first day"*);
3. **`/devices/{id}` as the real address, and the nav entry points at an EXAMPLE id** — the example
   dataset is 6b.3's, so this story defines the route shape and 6b.3 supplies the id that makes the
   nav entry honest.

**Take (3), and state the dependency**: 6b.2 owes the route, 6b.3 owes the id and the marker. ⚠️ *A
nav entry that 404s is worse than one that says "example".*

---

## 4. What becomes of `/`, which is the product's only screen today

`main.rs:359-363` has five routes; `/` is `page::index`, the reconciliation card, and `/gap` is its
HTMX fragment. This story takes the router to **fifteen**.

🔴 **`/` cannot stay the reconciliation card and also be a shell screen.** Three options:

- **`/` redirects to `/triage`** (303), and the card's content moves to the triage screen in 6b.4;
- **`/` IS the triage screen** and `/triage` does not exist — but then the nav's current-entry
  marking has two addresses for one screen, and `aria-current` becomes ambiguous;
- `/` becomes the dashboard, per the mock's own default screen.

**Recommendation: `/` redirects to `/triage`.** The mock opens on `triage`, the epic's story order
puts the real gap in the triage screen (6b.4), and a redirect keeps every existing bookmark working —
⚠️ **including the one in `README.md`, the manuals and the landing site**, which is 6b.12's sweep and
must be registered here rather than discovered there.

⚠️ **`/gap` must keep working unchanged**: story 5.14b's reach sections and the HTMX refresh ride it,
and the AC says HTMX swaps fragments *within* a screen. Do not move it in this story.

---

## 5. The Tailwind chain: this story is its registered owner, and the question is now live

`deferred-work.md` names **story 6b.2** as the owner of 6b.1's withdrawn AC1/AC5/AC6, on the
criterion *"the first screen story that writes a utility class"*. Read the register entry **before
writing a line**: it carries four spellings D55 does not contain, each measured on v4.3.3
(`source(none)`, `@theme static`, the two narrow imports instead of the full one, and the
`@source inline()` htmx entries that can never be generated).

🔴 **But the criterion is a CHOICE this story makes, not a fact it inherits.** 6b.1 measured that the
intersection between the classes the templates carry and the utilities Tailwind emits is **empty**.
This story adds a shell — a header, a sticky sidebar, a two-column grid — and it can be written
either way:

- **hand-authored classes**, continuing 6b.1: the sheet grows by ~40 lines, no chain, no ninth gate;
- **utilities**, which finally gives the chain something to generate and makes 6b.1's withdrawn ACs
  land here.

⚠️ **Whichever is chosen, the four measurements above are load-bearing** — and note the one with the
sharpest consequence for THIS story: **preflight alone changes ten computed styles on the existing
page and collapses the first-boot `<h1>`.** If the chain lands, it lands with
`@import "tailwindcss/theme"` + `"tailwindcss/utilities"` and never the bare import.

**Recommendation: hand-authored, and move the chain to the first story that needs a utility the sheet
cannot express.** The shell is ten rules; a build step whose output is ten rules is not yet earning
its ninth gate. But this is Guy's to weigh, and it is the same shape as 6b.1's §7.

---

## 6. What the shell must NOT break

Measured on `master` at `b1ce1a5`:

- **`is_public` is `/healthz` + `/assets/*` only** (`auth.rs:72-74`), so all ten screens sit behind
  HTTP Basic. Story 6.1 shrank it deliberately; **this story must not widen it**, and the pin at
  `auth.rs:173` is what would catch that.
- **`page.rs`'s `reconcile_view`** feeds both `index` and `gap_fragment`; story 5.14b's identity
  reach pair and story 6.2's J3 end-to-end test read through them.
- **28 fixtures, the trap gate RED at 26/15/11**, eight gates. The shell touches none of it.
- `templates/` holds **two** files. This story adds the shell partials; the house convention is
  composable partials (UX spec), and `askama` resolves them from the crate manifest (D55).

---

## Acceptance Criteria

Derived from `epics.md`'s three bullets for 6b.2, each made measurable. **AC5–AC7 are this story's own
additions**, and §2/§3/§4 must be arbitrated before AC1 can be written in code.

**AC1 — the frame, on every screen.**
Every one of the ten routes renders the header (brand, tagline, version) and the navigation over the
**ten** entries **in the mock's three groups**, with the current entry marked `aria-current="page"`.
A test asserts the count is exactly ten and that exactly one entry carries `aria-current` per screen.
🔴 The header's *perimeter* and *last observation* halves depend on §2's arbitration.

**AC2 — one URL per screen, server-rendered, deep-linkable cold.**
Ten routes, each rendering its screen **server-side**. No client-side router, no screen chosen by
JavaScript — asserted by a test that renders each route and finds its own marker, and by the absence
of any screen-switching script in `app.js`. HTMX swaps fragments **within** a screen only: `/gap`
still serves the reconciliation fragment and no route swaps another screen in.

**AC3 — all ten are shown from the first day, and none of them lies.**
The navigation lists ten entries on every screen. ⚠️ **This story must not hide, disable or grey an
entry** — the honesty is 6b.3's, and hiding entries would silently satisfy this AC while destroying
the next story's subject. A test asserts all ten are present and that none carries `disabled`,
`hidden` or an empty `href`.

**AC4 — no demo route opens a database connection** (epic constraint 1).
The nine routes this story adds hold **no pool** — the same structural carrier story 6.1 used for the
document route, where adding `State<MySqlPool>` to the handler **fails to compile**. 🔑 That is a
guard the compiler holds, not a test that can rot.

**AC5 — `/` keeps working, and so does every existing bookmark.**
`/` resolves (redirect or screen, per §4) and `/gap` is byte-identical in behaviour. A test asserts
the reconciliation fragment still renders with story 5.14b's two reach sections.

**AC6 — every label is a key, in both locales.**
The ten entries, the three group headings and the header strings exist in `locales/app.yml` under
**both** `fr` and `en` (NFR26). A test asserts no template carries a bare non-ASCII string and that
every key used resolves in both locales. ⚠️ 6b.10 owns the ~100 mock strings; these ~15 are this
story's own and must not be left for it.

**AC7 — `is_public` is unchanged**, and the pin at `auth.rs:173` proves it: ten new screens, zero new
public paths.

**AC8 — the live test count lives HERE**, in this file (story 6.1's AC8 rule, F2), and is not copied
into `CLAUDE.md`, `docs/project-context.md` or `sprint-status.yaml`.

---

## Tasks / Subtasks

- [ ] **T0 — the four arbitrations of §2, §3, §4 and §5, with Guy, BEFORE code**
- [ ] **T1 — the shell** (AC1, AC6): `_shell.html` + `_nav.html` partials; the ten entries in three
      groups; `aria-current`; the header
- [ ] **T2 — the ten routes** (AC2, AC4): handlers holding no pool; the screen enum; `/devices/{id}`
      per §3
- [ ] **T3 — `/` and `/gap`** (AC5) per §4's arbitration
- [ ] **T4 — the copy** (AC6): ~15 keys, `fr` + `en`
- [ ] **T5 — the guards** (AC1–AC7), each written to red before it passes
- [ ] **T6 — look at all ten screens** in a browser. ⚠️ Story 6b.1's T6 logged HTTP statuses and
      called it looking; *a status code is not a look*
- [ ] **T7 — the register**: §1's UX-DR33 divergence, §4's bookmark sweep for 6b.12, and whatever
      §5 defers. 🔴 **Then VERIFY the rows exist by reading the register, not by reading this list** —
      story 6b.1 claimed two registrations that were never written, and its own §Traps had
      prescribed exactly that check
- [ ] **T8 — prove-to-red**, predictions written FIRST

---

## Prove-to-red — the mutations this story owes

| # | Mutation | Prediction |
|---|---|---|
| M1 | Drop one nav entry | AC1/AC3's count reds |
| M2 | Mark two entries `aria-current` | AC1 reds — *exactly one*, not *at least one* |
| M3 | Give a screen handler `State<MySqlPool>` | **fails to COMPILE** (story 6.1's measured carrier) |
| M4 | Add a screen-switching branch to `app.js` | AC2 reds |
| M5 | `hidden` on the entry whose screen is emptiest | AC3 reds — the mutation this story exists to forbid |
| M6 | Remove one `en` key, leaving `fr` | AC6 reds |
| M7 | Add `/devices` to `is_public` | AC7's pin reds |
| M8 | Break `/gap`'s fragment | AC5 reds |
| M9 | Serve a screen from JavaScript instead of the route | AC2 reds cold-load |
| M10 (control) | Reorder two entries within a group | green — order within a group is not pinned by anything, and that is deliberate |

⚠️ **M3 is the only compiler-carried red and it must be recorded as such** — story 6.1 measured
`E0277` on the `Handler` bound. **M5 is the one that matters**: it is the shape this story is most
likely to take by accident.

---

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

### Validation obligations (two fresh-context layers, MANDATORY here)

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
