# Story 6b.12: The release v0.2.0, and the documents that describe it

Status: ready-for-dev

## Story

As the operator,
I want to install this,
So that the work that has been invisible for two epics becomes a thing I can run.

## Acceptance Criteria

*(`epics.md:2316`. §0 explains every divergence — there are four, and one of them changes a
count every other document repeats.)*

**AC1 — Given** the epic's preceding stories **When** the release is cut **Then** `v0.2.0` is
tagged and published to Docker Hub as `v0.1.1` was, from a green CI run.

**AC2 — Given** the release notes **When** they are written **Then** they name **what this
release does NOT do**: the example screens, the gestures Epic 7 owns, and 🔴 **every breaking
change — of which there are FOUR, not the three this epic has been repeating** (§0c).
*Announcing an absent gesture is a promise; letting a change be discovered is a surprise. Neither
is acceptable and the notes are where both are settled.*

**AC3 — And** the docs-current-before-push rule is discharged in the same push: the **User
Manual**, the **Administrator Manual**, `README.md`, the `gh-pages` landing site,
`docker/README.dockerhub.md`, `docs/project-context.md` and `CLAUDE.md`.

### The criteria this story adds to itself

**AC4 — Every document claim this release falsifies is found by a CHECK, not by reading.** §0b
lists the ones contexting found; the story may not assume that list is complete, and must say how
it looked.

**AC5 — The visual sweep the epic deferred to this story is DONE or re-registered BY NAME**, with
what was looked at and in which browser. Sixteen register lines name this story; a sweep that
covers none of them is not a sweep.

**AC6 — The live count for the project lives in THIS file**, and every figure names the state it
was taken against.

**AC7 — No document is left dated forward.** `docker/README.dockerhub.md` already says *"Since
v0.2.0"* twice, about a release that does not exist yet (§0d).

---

## §0 — What contexting established

### §0a. 🔴 THE EPIC'S CRITERION IS STALE IN THREE COUNTS, AND ALL THREE WERE MEASURED

`epics.md:2316` was written before the epic ran. Measured on the merged tree at `477163b`:

| The criterion says | Measured | How |
|---|---|---|
| *"the epic's **eleven** preceding stories"* | **twelve** | 6b.1–6b.11 plus the **6b.4b** insertion (`sprint-status.yaml`, twelve `done` keys) |
| *"the **eight** example screens"* | **five** of the ten in `Screen::ALL` | `screens.rs::nature()`: `Fed` = Triage, Sources, Diagnostic; `Mixed` = Dashboard; `Example` = Devices, Apps, Ipam, Alerts, Commissioning. (`Screen::Device` is a sixth, off the navigation.) |
| *"the manuals, **whose screenshots** show a product that no longer looks like that"* | **there are no screenshots** | `grep -rn includegraphics docs/manuals/` → nothing |

⚠️ **The third correction is the useful one, and it is the *right in substance, wrong in its
stated reason* family this project keeps recording.** The manuals ARE stale about the interface —
but through a SENTENCE, not an image. `user-manual.tex:151` reads *"A dark theme is the
default"*, which story 6b.1 falsified on 2026-08-18. A story that went looking for screenshots
would have found none, concluded the manuals were fine, and shipped the false sentence.

### §0b. 🔴 WHAT THE RELEASE FALSIFIES, FOUND BY GREP AND LISTED WITH ITS SITE

| Document | Site | The claim | Falsified by |
|---|---|---|---|
| `docs/manuals/user-manual/user-manual.tex` | `:151` | *"A dark theme is the default"* | story 6b.1 (light base, the dark set selected by nothing) |
| `README.md` | `:16` | *"serves **one page** showing a real observed-vs-declared gap"* | story 6b.2 (ten addresses) |
| `README.md` | `:124` | *"an axum server serving a **single page** at `/`"* | 6b.2 — and `/` is now a **303** to `/triage` |
| `gh-pages` `index.html` | ×2 | *"one page"* | 6b.2 |
| `gh-pages` `index.html` | ×4 | *"dark"* | 6b.1 |
| `gh-pages` `index.html` | ×2 | `v0.1.1` | this release |
| `crates/*/Cargo.toml` | `:6` ×2 | `version = "0.1.1"` | this release |

⚠️ **AC4 exists because this table was produced by ONE pass of one reader.** The needles were
`dark`, `one page`, `single page`, `screenshot`, `includegraphics`, `0.1.1`. A claim phrased
differently — *"the gap card"*, *"the home page"*, a French value in `app.yml` — is not in this
list and is not thereby absent. **The story owes a check, and owes saying what the check could
not see.**

### §0c. 🔴 THERE ARE FOUR BREAKING CHANGES, NOT THREE — AND THE FOURTH IS THE ONE AN OPERATOR MEETS FIRST

Three have been repeated across the twins, `sprint-status.yaml` and several story files:

1. **Every existing deployment changes colour** (6b.1 — dark to light).
2. **Every existing deployment changes address** (6b.2 — `/` becomes a 303 to `/triage`).
3. **`OPENCMDB_LOCALE=FR` stops the boot** (6b.10 — an unrecognised value is refused by name
   rather than falling back to English in silence).

🔴 **The fourth is registered and uncounted**: `deferred-work.md:3323`, row **(f)** —
***the product stops being publicly readable***, story 6.1's price for arbitration 2′. Measured:
`v0.1.1` was tagged **2026-07-21** and story 6.1 merged **2026-08-14**, so it is not in the
shipped release, and the register says in so many words *"the first release CONTAINING story 6.1
names it in its release notes; the obligation follows the release, not the story number. **Owner:
… story 6b.12**"*.

🔑 **And it is the one an operator meets FIRST** — before any colour, before any address: an
upgrade from `v0.1.1` with no `OPENCMDB_BASIC_USER` set answers **401 on every screen**, which is
the deliberate posture of a fresh instance and reads exactly like a broken deployment. *A change
nobody announced, met before the product renders anything, is the worst-shaped surprise this
release can ship.*

### §0d. ⚠️ ONE DOCUMENT IS ALREADY DATED FORWARD, WHICH IS A DEPENDENCY RATHER THAN A DEFECT

`docker/README.dockerhub.md:85` says *"**Since v0.2.0** the product is NOT publicly readable"* and
`:93` says *"**v0.2.0** an UNRECOGNISED value refuses to start"*. Both are FALSE today and become
true the moment this story ships. They were written by stories 6.1 and 6b.10 against a release
they assumed would happen.

**Consequence for this story: those two sentences are a test.** If the release is cut, they are
correct and need no edit. If the release slips, they are two false claims in the file Docker Hub
renders as the product's front page. AC7 exists so that outcome is decided rather than inherited.

### §0e. WHAT THE RELEASE MECHANISM ALREADY IS — LIFT IT, DO NOT REINVENT IT

`.github/workflows/release.yml` exists and shipped `v0.1.0` and `v0.1.1`: triggered on
`push: tags: ['v*.*.*']`, it does checkout → Buildx → Docker Hub login (secrets) →
`docker/metadata-action` → `docker/build-push-action`, then `peter-evans/dockerhub-description`
syncs `docker/README.dockerhub.md` to the Docker Hub page (best-effort, `continue-on-error`).

**So AC1 is a TAG on a green master plus a verification, not a workflow to write.** Story 3.10's
own shape is the precedent: tag, watch the run, then `docker pull` the published image and start
it against a MariaDB. ⚠️ **Do not tag before the version literals move**: `version = "0.1.1"` sits
in `crates/opencmdb-bin/Cargo.toml:6` and `crates/opencmdb-core/Cargo.toml:6`, and `Cargo.lock`
follows.

### §0f. THE VISUAL SWEEP IS SIXTEEN REGISTER LINES, AND ONE OF THEM IS NOT A DEFECT BUT A DESIGN QUESTION

`grep -n "6b.12" deferred-work.md` returns sixteen lines. They are not one task. The two that
carry the most weight:

- 🔴 **Story 6b.5's SALIENCE finding**: on `/dashboard` the fabricated example cards are
  *visually dominant* over the honest reach counts — invented figures at 22 px mono with
  sparklines against real counts in a body-size pill. It violates *"so that the honest part is
  not diluted by the demonstration around it"* **without breaking a single criterion**, because
  the criteria cover presence and marking and the defect is salience. **No text guard can measure
  it**; it is a look, and this story is where the look happens.
- ⚠️ **Four unpaid visual passes** (`:4062`) — stories 6b.1 through 6b.4 each deferred typography,
  spacing and layout to a browser check, on a sentence nobody had run `command -v` against until
  6b.4b found Chrome 151 installed all along.

The rest: the IPAM legend at 10 px (`libre` vs `réseau ou diffusion` hard to tell apart), the
`/gap` fragment seen by no eye, the applications table showing a divergence without naming it,
NFR24's touch targets, and the axe gate's own residual scope.

### §0g. WHAT THIS STORY MUST NOT DO

- **Not close Epic 6b.** The epic closes after the **retrospective** and then the **project
  review**, in that order, and the review is conducted on the project's own folder under its own
  `CLAUDE.md`. A release is not a closure.
- **Not fix what the sweep finds, beyond copy and CSS.** A salience repair that reshapes
  `/dashboard`'s data is a story, not a sweep. Find, fix what is cheap and visual, register the
  rest **by name**.
- **Not edit `epics.md`.** §0a's three corrections are registered with the retrospective, which
  may edit it; a story may not.
- **Not tag before CI is green ON THE HEAD COMMIT.** Story 6b.11 shipped with a green measured on
  the head commit deliberately — a green on an earlier commit is this project's own stale-green
  class, and a tag makes it permanent.

---

## Tasks / Subtasks

- [ ] **T1 — The version literals and the lockfile** (AC1)
  - [ ] `0.1.1` → `0.2.0` in both `crates/*/Cargo.toml`; `cargo build --locked` so `Cargo.lock`
        follows; verify `/diagnostic` renders the new version (it reads the build's own).
- [ ] **T2 — Find every falsified claim by a CHECK** (AC4)
  - [ ] Run the §0b needles plus any others the developer chooses, over `README.md`,
        `docker/README.dockerhub.md`, `docs/manuals/`, the `gh-pages` worktree and `app.yml`.
  - [ ] **Write what the check could NOT see.** An enumeration cannot claim the completeness of
        a property — the sixth application of that rule in this project.
- [ ] **T3 — The documents** (AC3, AC7)
  - [ ] `user-manual.tex:151`, `README.md:16` and `:124`, `gh-pages/index.html`,
        `docker/README.dockerhub.md`, the twins.
  - [ ] ⚠️ Decide `docker/README.dockerhub.md`'s two forward-dated sentences: correct if the
        release ships, and say so if it does not.
- [ ] **T4 — The release notes** (AC2)
  - [ ] What it DOES: ten addresses, the mock's light design, the triage screen on the real gap,
        the keyboard layer, two browser gates, the copy in FR and EN.
  - [ ] What it does NOT: five example screens (six with `/device`), the four gestures Epic 7
        owns, no write surface beyond `POST /document-all`, which no template calls.
  - [ ] 🔴 **The FOUR breaking changes**, the closed-by-default one FIRST, with the two lines an
        operator needs to get back in (`OPENCMDB_BASIC_USER` / `OPENCMDB_BASIC_PASSWORD`).
- [ ] **T5 — The visual sweep** (AC5)
  - [ ] A real browser, both locales, the ten addresses plus `/devices/{id}`. Name the browser
        and the width.
  - [ ] Work the sixteen register lines: fix what is copy or CSS, re-register the rest BY NAME
        with an owner.
  - [ ] 🔴 Look at `/dashboard`'s salience specifically — it is the one finding a criterion
        cannot express.
- [ ] **T6 — Cut it** (AC1) — Guy's approval required before the tag
  - [ ] Green CI **on the head commit**, checked. Tag `v0.2.0`, push, watch the release run.
  - [ ] `docker pull gcorbaz/opencmdb:0.2.0`, start it against a MariaDB, `/healthz` → 200,
        `/` → 303 `/triage`, a screen renders.
- [ ] **T7 — The record** (AC6) — live count here; every figure names its state; twins and
      `sprint-status.yaml` in the same push.

---

## Dev Notes

### What the previous story leaves you

Story 6b.11 shipped **two browser gates** and they now run in CI: `a11y/axe-gate.mjs` (ten routes
plus two query-string states) and `a11y/kbd-probe.mjs` (twenty checks). ⚠️ **Both are 0/1/2 — but
only once `node` is running them**; the shell before them keeps its own codes. `cargo xtask ci`
still carries **nine** gates, not eleven.

⚠️ **CI seeds `a11y/seed.sql`, which TRUNCATES the store first.** If you point CI at a database
you care about, it will be emptied. That is deliberate: the previous seeding was green on the test
step's residue.

### The house rules that bite here

- **Docs-current-before-push** is an AC here rather than a convention: this is the first release
  where a manual describing the previous interface is a live risk.
- **A cause needs a check, not a plausible story** — §0a's third row is that rule catching the
  epic's own criterion.
- **The live count lives in the current story's file** (story 6.1's AC8), never in the twins.

---

## References

- `epics.md:2316` — the epic's criterion, stale in three counts (§0a).
- `deferred-work.md:3323` (row f), and fifteen more lines naming this story (§0f).
- `_bmad-output/implementation-artifacts/3-10-release-0-1-0.md` — the release shape to lift.
- `.github/workflows/release.yml` — tag-triggered, already proven twice.

---

## Dev Agent Record

*(to be filled by the dev agent)*

## Change Log

| Date | Change |
|---|---|
| 2026-08-24 | Story created and CONTEXTED. 🔴 Four divergences from `epics.md:2316`, each measured: **twelve** preceding stories not eleven; **five** example screens not eight; the manuals carry **no screenshots** at all — their staleness is a SENTENCE (`user-manual.tex:151`, *"A dark theme is the default"*), so a story hunting images would have shipped it; and 🔴 **FOUR breaking changes, not three** — the product stopping being publicly readable is registered at `deferred-work.md:3323` and counted nowhere, **and it is the one an operator meets first**, before any colour or address. ⚠️ Also found: `docker/README.dockerhub.md` is already dated *"Since v0.2.0"* twice, so the release is a dependency of a document rather than the other way round. |
