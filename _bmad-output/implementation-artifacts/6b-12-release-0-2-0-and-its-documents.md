# Story 6b.12: The release v0.2.0, and the documents that describe it

Status: review — ⚠️ **every task but T6 is done; the TAG is deliberately not cut.** It needs
Guy's approval, and the natural order is *review, then tag*: a tag is not revertible in the
ordinary sense, so nothing unreviewed should be made permanent by one.

### ⚠️ THE LIVE COUNT FOR THE PROJECT (AC6)

**729 tests** — 491 `opencmdb-bin` + 161 `opencmdb-core` + 77 `xtask` — run **both ways** with
`cargo test --workspace --locked`, warm: **7.6 s** against a live `mariadb:10.11.11` (container
`verif6b11`, port 13363, seeded by `a11y/seed.sql`) and **0.65 s** with `DATABASE_URL` unset. The
clock is the tell that the store-backed tests genuinely executed. 🔴 **The count read 728 until
the code review, on the sentence *"this story added no `#[test]`"* — which was true and was the
defect.** The blind layer found from the diff alone that the scratch registry shipped with no test
at all, against this project's own rule that a new guard needs one that reds when it is removed.
`a_scratch_tag_two_call_sites_claim_is_refused` is that test; ⚠️ **and the FIRST registry would
have passed it** while still being wrong, because it keyed on the helper's name — it reds only
because the owner is now the call site.

Nine `cargo xtask ci` gates green, `clippy --workspace --all-targets -- -D warnings` clean,
`cargo fmt --all --check` clean, `RUSTFLAGS="-D warnings" cargo test --workspace --locked` green,
both LaTeX manuals build. The two browser gates were last run at story 6b.11's close; ⚠️ **they
are not re-run here**, and CI runs them on the PR.

## Story

As the operator,
I want to install this,
So that the work that has been invisible for two epics becomes a thing I can run.

## Acceptance Criteria

*(`epics.md:2316`. §0 explains every divergence — there are four, and one of them changes a
count every other document repeats.)*

**AC1 — Given** the epic's preceding stories **When** the release is cut **Then** `v0.2.0` is
tagged and published to Docker Hub as `v0.1.1` was, from a green CI run.

**AC2 — Given** the release notes **When** they are written **Then** they exist somewhere an
operator will SEE them — 🔴 a **GitHub Release object** created by the workflow from a committed
`CHANGELOG.md` (arbitration 1: there is no such venue today, and neither `v0.1.0` nor `v0.1.1` has
one) — and they name **what this release does NOT do**: the example screens, the gestures Epic 7 owns, and 🔴 **every breaking
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
| *"the **eight** example screens"* | **six** of the ten in `Screen::ALL` | `screens.rs::nature()`: `Fed` = Triage, Sources, Diagnostic; `Mixed` = Dashboard; `Example` = Devices, **Device**, Apps, Ipam, Alerts, Commissioning. 🔴 **This row first said *five … plus `Screen::Device`, off the navigation*, and the validation refuted it**: `Screen::Device` carries `NavGroup::Inventory` and the nav is built by filtering `Screen::ALL`, so it renders with its label. What is off the generic dispatch is its ROUTE, in `router()`. ⚠️ `sprint-status.yaml:4578` had recorded that correction one story earlier and it did not carry. |
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

**So AC1 is a TAG on a green master plus a verification, and a WORKFLOW EDIT** — see arbitration
1 (§0i): `release.yml` builds and pushes an image and creates **no GitHub Release object**, which
is why `v0.1.0` and `v0.1.1` have none. Story 3.10's shape is otherwise the precedent: tag, watch
the run, then `docker pull` the published image and start it against a MariaDB.

⚠️ **Three version literals, not two**, and the validation found the third: `crates/opencmdb-bin/
Cargo.toml:6`, `crates/opencmdb-core/Cargo.toml:6` **and `xtask/Cargo.toml:9`** — which the glob
`crates/*/Cargo.toml` structurally cannot match. `xtask` never surfaces to an operator (the binary
reads its own `CARGO_PKG_VERSION`), so this is consistency rather than behaviour. **And
`docker-compose.yml:14` pins `gcorbaz/opencmdb:0.1.1`** — the deployment path both READMEs point
to, in AC3's list nowhere until now.

🔴 **`cargo build --locked` CANNOT update `Cargo.lock`, measured**: `error: cannot update the lock
file … because --locked was passed to prevent this`. A plain `cargo build` first, so the lock
follows; `--locked` afterwards.

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

🔴 **The list of "the rest" this section first gave was wrong in 3 of its 5 items, and the
validation measured it**: the applications table (**Epic 15**), NFR24's touch targets (**Epic 6b's
retrospective**) and the axe gate's exit residual (**an unnamed future story**) carry no reference
to this story at all. *Naming someone else's row as your own is how a sweep grows a scope nobody
decided.* What genuinely belongs here, beyond the two above: the **OFL font attribution owed in
`README.md`**, the measured **zero `@media` rules** against the UX spec's mobile-first mandate
(with `/triage` unusable at 390×844), an orphan `.wrap` CSS rule, the dashboard's reach section
never seen populated in a browser, the missing locale-parameterised render test, the IPAM legend
at 10 px, and the `/gap` fragment no eye has seen.

### §0h. THE VALIDATION PASS — two fresh-context layers, 2026-08-24

**20 raw findings → 18 distinct defects, two of them reached by BOTH layers.** The fact-check
layer worked read-only over the tree; the gap-hunt layer had its own worktree, its own
`mariadb:10.11.11` (port 13370) and Chrome 151, and **built the release image and booted the
product three ways** rather than reading about it.

🔴 **THE CONTEXTING'S OWN CENTRAL DEVICE — §0b's six needles — IS MEASURED INSUFFICIENT THREE
WAYS, and AC4's hedge is what survives.** §0b listed the claims a grep for `dark`, `one page`,
`single page`, `screenshot`, `includegraphics` and `0.1.1` could find, and hedged: *"a claim
phrased differently is not in this list and is not thereby absent"*. The gap-hunt then found
three whole classes of falsified claim that no needle could reach:

1. 🔴 **THREE DOCUMENTS MAKE A SECURITY CLAIM THE PRODUCT'S OWN TEST FORBIDS AS FALSE.**
   `README.md:146`, `docker/README.dockerhub.md:144` and `admin-manual.tex:264` all assert, as
   current fact, that stored credentials are **encrypted at rest** with a master key outside the
   data volume. Measured: `diagnostic.rs:1038,1040` lists the strings `"encrypted at rest"` and
   `"encryption key"` in the set a test **forbids from ever rendering**, because the product has
   no credential store and no crypto call site — and `/diagnostic` says so on screen: *"stored
   credentials — none stored — Epics 10 and 19 build credential storage."* 🔑 **The exact
   sentence the codebase guards against is standing on the page Docker Hub renders as the
   product's storefront.** The admin manual goes further and claims API-key rotation for a
   connector that does not exist.
2. 🔴 **The project-status narrative is stale by roughly fifteen epics.** `README.md` says
   *"epics 1–4 of 23 are complete and epic 5 … is under way (10 of its 16 stories)"*; `gh-pages`
   says *"epics 1–3 … epic 4 is under way"*. Both then say *"no triage inbox, no IPAM, no
   alerts, no admin UI"* — **all four now exist**, three of them as screens this epic built.
3. 🔴 **Tailwind is claimed as the current stack in three places**, with `cargo xtask css` among
   the commands. Measured: the subcommand does not exist, and `CLAUDE.md` records the opposite
   decision taken **twice** (6b.1, re-confirmed at 6b.2) — *"there is NO Tailwind chain"*.

🔴 **AND THE STORY'S OWN §0 IS WRONG FOUR WAYS**, each measured by the fact-check layer:

- **`Screen::Device` IS in the navigation.** It carries `NavGroup::Inventory` and the nav is built
  by filtering `Screen::ALL`, so it renders with its label. What is true is that it is off
  `router()`'s generic dispatch loop. ⚠️ **And the correction already existed in this project's
  own record** — `sprint-status.yaml:4578`, written at story 6b.11's validation: *"`Screen::Device`
  IS in `Screen::ALL` — 6b.6 moved its route REGISTRATION, not the variant."* ***A correction
  established one story earlier did not carry into the next***, which is the defect rather than
  the confusion.
- **AC3 names the Administrator Manual and no task does** — the only document of the seven in that
  position. And it needs real work: a `\begin{planned}` block still says installation *"will be
  documented here once the first release image is published"* (published since **v0.1.0**), and
  the whole Security chapter is **silent** on `OPENCMDB_BASIC_USER`, on the closed-by-default
  posture, and on the 401 — the very two lines AC2 says the release notes owe the operator.
- **§0f's worked example is wrong in 3 of its 5 items.** The applications table (Epic 15), NFR24
  (Epic 6b's retrospective) and the axe gate's exit residual (an unnamed story) contain no
  reference to this story at all; meanwhile at least six genuine 6b.12 rows go unnamed, among
  them the **OFL font attribution owed in `README.md`** and the measured fact that the sheet
  carries **zero `@media` rules** against the UX spec's mobile-first mandate.
- ⚠️ **`cargo build --locked` cannot do what T1 asks of it** — measured: `error: cannot update the
  lock file … because --locked was passed to prevent this`. A plain build first, then `--locked`.

**Reached by BOTH layers, independently:**

- **The manuals present unbuilt features as CURRENT, unmarked** — the fact-check found the
  *Triage gestures* chapter (six gestures described as working, while all five controls ship
  labelled *À venir*); the gap-hunt found **four** such sections (*Sources and liveness*, *Triage
  gestures*, *IP address management*, *Alerts and notifications*), each sitting a paragraph away
  from another section that IS wrapped in `\begin{planned}`. 🔑 The gap-hunt also supplies the
  CHECK AC4 demands: for every `Screen::ALL` entry whose `nature()` is `Example`, grep the
  matching manual chapter for `\begin{planned}` — **four chapters fail it**.
- **A third version literal**, `xtask/Cargo.toml:9`, outside T1's `crates/*` glob. Never
  user-visible (the binary reads its own `CARGO_PKG_VERSION`), and a real inconsistency left
  behind.

🔴 **AND ONE FINDING REACHES PAST THIS STORY: a reproduced candidate cause for ISSUE #38.**
`read_scratch("both", …)` at `fixtures.rs:1749` and `write_traps("both", …)` at `:2138` resolve to
the **same** `scratch_dir("both")` — same pid, same tag, two different helpers — and the first
one's cleanup does `remove_dir_all` on it. The gap-hunt reproduced the failure once in ten
full-suite runs (`NotFound` at `fixtures.rs:2032`), which matches issue #38's own recorded rate.
⚠️ **`CLAUDE.md` records that hypothesis as *"raised and refuted"*, and the refutation checked
only that *the six `write_traps` tags are distinct from each other*** — it never compared a
`write_traps` tag with a `read_scratch` tag. ***The refutation measured one half of the
population.*** The fix is a one-line rename. Open since Epic 4.

**Registered by the validation, not fixed:** `docker-compose.yml:14` pins
`gcorbaz/opencmdb:0.1.1` and is the deployment path both READMEs point to, absent from AC3 and
T3; the Docker Hub *Tags* table has no `0.2.0` row and its one-clause-per-tag shape was never
built to carry four breaking changes; `OPENCMDB_DOCUMENT_ENABLED` is documented **nowhere**,
including the admin manual's Configuration chapter which lists every other variable; `favicon.ico`
404s on every route.

**Refuted by the gap-hunt, each with its check** — recorded so nobody re-chases them: the missing
`WWW-Authenticate` on the unconfigured 401 is **arbitration 6**, deliberate and tested; no test
pins the crate version literal, so the bump breaks nothing; **the release Dockerfile still
builds** (`docker build` succeeded, the image booted against a live MariaDB, `/healthz` 200, `/`
401); the manuals still compile with `make`; and §0b's own counts (`v0.1.1` ×2, `dark` ×4, `one
page` ×2 on `gh-pages`) are **numerically exact** — ⚠️ though the four `dark` hits are the landing
page's own theme toggle, not prose about the product, so that row of §0b names a design question
rather than a false claim.

### §0i. THE TWO ARBITRATIONS (Guy, 2026-08-24)

Both were raised by a measurement the validation took, and in both Guy took the option that closes
the property rather than the one that keeps the story narrow. Recorded with the option refused.

🔴 **Arbitration 1 · The release notes get a VENUE: a GitHub Release object, created by the
workflow, fed by a committed `CHANGELOG.md`.** The gap-hunt measured that there is none —
`gh release list` is **empty**, neither `v0.1.0` nor `v0.1.1` has a Release object, there is no
changelog anywhere, and `release.yml` creates neither. Story 3.10 recorded the v0.1.0 cut **in its
own story file's Change Log**, which no operator will ever read. 🔑 *So AC2 was satisfiable in
letter while leaving zero operators informed — which is exactly the outcome §0c calls the
worst-shaped surprise this release can ship.* The Docker Hub *Tags* table gains its `0.2.0` row
and points at the Release. **Refused:** the Tags table alone (one clause per version, never built
to carry four breaking changes) and a `CHANGELOG.md` alone (invisible to anyone pulling the image).
⚠️ This touches `release.yml`, which no story has edited since 3.10 — and it must be exercised
before the tag, since a tag is not revertible in the ordinary sense.

🔴 **Arbitration 2 · Issue #38's reproduced candidate cause is FIXED HERE, with the control the
original refutation never ran.** `read_scratch("both")` and `write_traps("both")` resolve to one
`scratch_dir`, and the first one's cleanup deletes it; reproduced once in ten full-suite runs, the
rate the issue itself records. ⚠️ **`CLAUDE.md` files that hypothesis as *"raised and refuted"* on
a check that compared only `write_traps` tags with each other** — *the refutation measured one half
of the population.* The fix is a rename plus a guard that reds when any tag is shared **between**
the two helpers. 🔑 The reason it lands in a release story rather than waiting: **a suite known to
be unstable on the very commit about to be tagged is a release risk**, and §0g already forbids
tagging on anything but a green measured on the head commit. **Refused:** registering it (leaves
the tag on a tree we know is flaky) and *closing issue #38* — ⚠️ **one reproduced occurrence
establishes *a* cause, never *the* cause**, and this project forbids naming a cause without the
check that would refute it. The issue stays open, with the measurement and the control attached.

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

*(Rewritten on what the validation measured, 2026-08-24. Every task that changed carries why.)*

- [ ] **T0 — Issue #38's reproduced cause** (arbitration 2)
  - [ ] Rename one of the two colliding `"both"` tags in `fixtures.rs` (`:1749` `read_scratch`
        against `:2138` `write_traps`).
  - [ ] 🔑 **The guard the original refutation never ran**: a test that reds when ANY tag is
        shared *between* the two helpers, not merely when `write_traps`' own tags collide. Prove
        it red by restoring the collision.
  - [ ] Attach the measurement and the control to **issue #38**; ⚠️ do NOT close it — one
        reproduced occurrence is *a* cause, not *the* cause.

- [ ] **T1 — The version, in FOUR places, and the lockfile** (AC1)
  - [ ] `0.1.1` → `0.2.0` in `crates/opencmdb-bin/Cargo.toml:6`, `crates/opencmdb-core/
        Cargo.toml:6` and ⚠️ **`xtask/Cargo.toml:9`** (outside the `crates/*` glob), plus
        `docker/docker-compose.yml:14`'s pinned image.
  - [ ] ⚠️ A plain `cargo build` FIRST so `Cargo.lock` follows — `--locked` refuses to update it,
        measured. Then `--locked` for everything else.
  - [ ] `/diagnostic` renders the new version (it reads the build's own).

- [ ] **T2 — Find falsified claims by a CHECK, and say what the check cannot see** (AC4)
  - [ ] 🔴 **The six needles of §0b were measured insufficient THREE ways** — start from what the
        validation found, then go wider: `README.md`, `docker/README.dockerhub.md`, both LaTeX
        manuals, `origin/gh-pages`, `app.yml`, `.env.example`, `docker/docker-compose.yml`, `docs/`.
  - [x] 🔴 **THIS CHECK WAS NOT WHAT RAN, and the code review measured it.** As written — *for
        every `Screen::ALL` entry whose `nature()` is `Example`, the matching manual chapter must
        carry `\begin{planned}`* — it finds almost nothing: three of the six `Example` screens
        have no chapter at all, and two of the three that do already carried the marker. What ran
        was a **structural scan of every chapter, judged by reading**, and the six it corrected
        describe connectors and gestures on screens that are `Fed`. *The work is right and its
        description was a check that was a reading* — the exact pattern AC4 exists to catch,
        inside AC4's own delivery. Registered with its owner; the record is corrected rather than
        the criterion rewritten to fit. Originally: **four chapters fail it today** (*Sources and liveness*, *Triage gestures*, *IP address
        management*, *Alerts and notifications*).
  - [ ] Write what the sweep could NOT reach. *An enumeration cannot claim the completeness of a
        property* — and §0b is this project's newest proof of it.

- [ ] **T3 — The documents** (AC3, AC7)
  - [ ] 🔴 **The security claim first**: `README.md:146`, `docker/README.dockerhub.md:144`,
        `admin-manual.tex:264` — *"encrypted at rest"*, which `diagnostic.rs:1038` forbids the
        product from rendering because it is false. Delete or mark planned; the admin manual's
        API-key rotation claim goes with it (no connector exists).
  - [ ] 🔴 **The status narrative**: `README.md` (*"epics 1–4 of 23"*) and `gh-pages`
        (*"epics 1–3"*), and the *"no triage inbox, no IPAM, no alerts, no admin UI"* line in both
        — all four exist now.
  - [ ] 🔴 **Tailwind**, claimed as the stack in `README.md` ×2 and `gh-pages` ×1, with a
        `cargo xtask css` that does not exist. `CLAUDE.md` carries the opposite decision, twice.
  - [ ] **The Administrator Manual** — ⚠️ *named by AC3 and by no task until now*: the stale
        `\begin{planned}` installation block (the image has been published since **v0.1.0**), and
        a Security chapter **silent** on `OPENCMDB_BASIC_USER`, the closed-by-default posture and
        the 401. Add `OPENCMDB_DOCUMENT_ENABLED` to its Configuration chapter, documented nowhere.
  - [ ] The User Manual's *"A dark theme is the default"* (`:151`), and the four unmarked chapters
        from T2's check.
  - [ ] `README.md:16`/`:124`, `gh-pages/index.html`, the twins, and the OFL font attribution the
        register says `README.md` owes.
  - [ ] ⚠️ Decide `docker/README.dockerhub.md`'s two *"Since v0.2.0"* sentences — correct once the
        release ships, false if it slips (AC7).
  - [ ] ⚠️ The `gh-pages` *"dark"* ×4 are the landing page's OWN theme toggle, not claims about the
        product: a design question (should the site's default follow the product's light-first
        identity?), not a correction. Decide it, do not silently edit it.

- [ ] **T4 — The release notes, and the VENUE they did not have** (AC2, arbitration 1)
  - [ ] `CHANGELOG.md`, committed, with a `0.2.0` section.
  - [ ] `release.yml` gains a step creating a **GitHub Release** from it. ⚠️ Exercise it before
        the tag — a tag is not revertible in the ordinary sense, and no story has touched that
        workflow since 3.10.
  - [ ] The Docker Hub *Tags* table gains its `0.2.0` row, pointing at the Release.
  - [ ] Content — what it DOES: ten addresses, the mock's light design, `/triage` on the real gap,
        the keyboard layer, two browser gates, the copy in FR and EN. What it does NOT: **six**
        example screens, the four gestures Epic 7 owns, no write surface a template calls.
  - [ ] 🔴 **The FOUR breaking changes, the closed-by-default one FIRST**, with the two variables
        that let an operator back in.

- [ ] **T5 — The visual sweep** (AC5)
  - [ ] A real browser, both locales, the ten addresses plus `/devices/{id}`. Name browser and
        width. ⚠️ **The gap-hunt has already produced this sweep's screenshots**; lift them rather
        than re-taking them, and say which are lifted.
  - [ ] Work the rows §0f now lists correctly. ⚠️ **Three items the first draft named belong to
        Epic 15, the retrospective and an unnamed story — do not work them here.**
  - [ ] 🔴 `/dashboard`'s salience: the fabricated cards against the honest counts. Say whether
        copy-and-CSS reaches it or whether it needs a design decision, and register accordingly.

- [ ] **T6 — Cut it** (AC1) — **Guy's approval required before the tag**
  - [ ] Green CI **on the head commit**, checked rather than assumed.
  - [ ] Tag `v0.2.0`, push, watch the release run, confirm the Release object appeared.
  - [ ] `docker pull gcorbaz/opencmdb:0.2.0`, start against a MariaDB, `/healthz` → 200,
        `/` → 303 `/triage`, a screen renders. ✅ The gap-hunt has already proven the Dockerfile
        still builds and boots from this tree.

- [ ] **T7 — The record** (AC6) — live count here; every figure names its state; twins and
      `sprint-status.yaml` in the same push; the register rows this story does not take,
      re-registered BY NAME with an owner.

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

### What was built, and what the building found

**Everything below was measured against a live `mariadb:10.11.11` (container `verif6b11`, port
13363), a booted binary and Chrome 151. T6 — the tag — is NOT done: it needs Guy's approval, and
a tag is not revertible in the ordinary sense.**

#### 🔴 T0 — issue #38: the validation named two claimants and there are SIX

The scratch namespace is keyed on `(pid, tag)` and `read_scratch`'s callers finish with
`remove_dir_all`. The validation found `read_scratch("both")` against `write_traps("both")`. ⚠️
**Four more tests call `scratch_dir` DIRECTLY**, outside both helpers — a population neither the
original refutation nor the validation had in its perimeter. So the fix is not the rename: it is
that **`scratch_dir` now takes its caller and refuses a tag a different one has claimed**.

- Restoring the collision reds **exactly one test**, naming both claimants.
- ⚠️ **The guard's first form turned one defect into EIGHTEEN failures** — the panic poisons the
  registry's mutex and every later lock inherits it. Poison is recovered rather than propagated:
  *a guard that multiplies one finding by ten is noise around it.*
- **Issue #38 stays OPEN**, with the measurement and the control attached. One reproduced
  occurrence establishes *a* cause, never *the* cause.

#### T1 — four version literals, and `--locked` cannot do what the plan asked

`crates/opencmdb-bin`, `crates/opencmdb-core`, **`xtask/Cargo.toml`** (outside the `crates/*`
glob) and **`docker/docker-compose.yml`**'s pinned image. A plain `cargo build` first so
`Cargo.lock` follows — `--locked` refuses to update it, by design and measured.

#### 🔴 T2/T3 — the check found more than the reading did

The prescribed check (every `Example` screen's manual chapter must carry `\begin{planned}`) found
**six** unmarked sections where the validation had read four: it added ***The UniFi source*** —
there is no UniFi connector at all — and ***Documenting a discovery***. 🔑 *That is what AC4
bought: a check sees what a reading misses, including a reading by a layer whose job was to look.*

The false security claim was corrected in all three documents. The README's status narrative said
*"one page, one connector, no triage inbox, no IPAM, no alerts, no admin UI"* — **four of those six
had stopped being true**. Tailwind was named as the stack in three places against a decision taken
twice the other way. The admin manual — named by AC3 and by no task until the validation caught it
— gained the closed-by-default posture its Security chapter was silent on, and every variable its
Configuration chapter did not list.

#### T4 — the notes get a venue, and the venue has a guard

`CHANGELOG.md` committed; `release.yml` extracts the section by version and creates a GitHub
Release. ⚠️ **A missing or empty section FAILS the release** rather than publishing silence —
proven both ways locally: `0.2.0` extracts 79 lines, `0.9.9` is refused. The Docker Hub *Tags*
table carries the four breaking changes in the order an operator meets them.

#### 🔴 T5 — the sweep, and what only a look could give

The screenshots are **lifted from the validation's gap-hunt layer** rather than re-taken, on a
check that the templates, assets and locales had not moved since. ⚠️ **That check was
insufficient and the screenshots say so**: the version pill reads `v0.1.1`, because T1 moved a
literal the shell renders and my check did not cover it. *The verification that authorised the
lift did not cover everything the lift shows.* Everything else in them is current.

- 🔴 **The salience finding is confirmed by eye and is sharper than its description.** The honest
  section is three lines of small grey prose; the fabricated one is three cards with 22 px figures
  and sparklines. The invented `37 / 4 / 2` is the loudest thing on the page.
- 🔴 **And a finding no report named: story 6b.5's repair is PARTIAL.** In the state
  `a11y/seed.sql` creates — observations present, none placed, which is **CI's own state on every
  gate run** — the page says *"Nothing observed yet — run a scan and this fills in."* immediately above *"A scan has
  landed; the identity pass has not placed any of it yet."* The second sentence is 6b.5's fix and
  **it explains the first without removing it**. ⚠️ On `/triage` the false line renders *below
  four queue rows dated one minute ago*: the claim and its refutation in one viewport.

Both are registered with owners; neither is copy-and-CSS, and §0g forbids this story from
reshaping a screen.

### What was decided rather than built

- **The landing page's dark default is left alone**, with its reason written: it is the site's own
  toggle, not a claim about the product, and the story said decide it rather than silently edit it.
- **`gh-pages` is committed on its branch and deliberately NOT pushed** — publishing it would
  announce `v0.2.0` before the tag exists. It follows T6.
- **The `\begin{planned}` check is a one-off, not a tenth gate.** Registered for the retrospective
  to decide, since a gate over LaTeX is a real cost for a documentation property.

### File List

- `crates/opencmdb-bin/src/fixtures.rs` — the scratch-tag ownership registry (T0)
- `crates/opencmdb-bin/Cargo.toml`, `crates/opencmdb-core/Cargo.toml`, `xtask/Cargo.toml`,
  `Cargo.lock`, `docker/docker-compose.yml` — the version (T1)
- `README.md`, `docker/README.dockerhub.md`, `docs/manuals/user-manual/user-manual.tex`,
  `docs/manuals/admin-manual/admin-manual.tex` — the documents (T3)
- `CHANGELOG.md` (new), `.github/workflows/release.yml` — the notes and their venue (T4)
- `_bmad-output/implementation-artifacts/deferred-work.md` — five rows (T5)
- `gh-pages` branch, `index.html` — committed, unpushed

## §T11 — The three-layer code review, and its repair (2026-08-24)

Three isolated layers, a different model, one worktree and one live store each. **24 raw findings
→ 19 distinct, three of them reached by more than one layer.**

### 🔴 The two that would have hurt an operator

**1 · `docker/.env.example` — the file the documents tell an operator to COPY — never got the
variables this release makes mandatory.** Only the README's *inline copy* of it did. `README.md`
says, literally, `cp docker/.env.example docker/.env    # set DATABASE_PASSWORD,
OPENCMDB_SCAN_CIDR, …` — an instruction that enumerates what to fill in and **does not name the
auth pair**, over a file that carries neither. 🔑 *So an operator doing exactly what the document
says boots `v0.2.0`, meets `401` on every screen, and the two variables that would rescue them are
absent from both the file and the instruction.* That is this release's headline breaking change,
and the file that protects against it is the one the story did not touch. Found by the edge layer,
by following the instructions rather than reading them.

**2 · The image was published BEFORE the changelog check ran.** `Build and push` — which also
overwrites `latest` — sat several steps above `Extract this version's changelog section`, the only
step that can `exit 1`. So *"a missing section FAILS the release"* meant *"fails to create the
Release OBJECT"*: the artefact was already on Docker Hub with no notes, which is the precise
outcome arbitration 1 built that step to prevent. The extraction now runs **immediately after
checkout**, before anything is published. *Validating an input before doing irreversible work is
not a preference.*

### 🔴 What the review found about the story's own claims

**3 · The "prescribed check" was a reading.** T2 defines it as *every `Screen::ALL` entry whose
`nature()` is `Example` must have its manual chapter marked*. Taken literally it finds almost
nothing — three of the six `Example` screens have no chapter, and two of the three that do already
carried the marker. The six chapters actually corrected describe **connectors and gestures**, on
screens that are `Fed`. ⚠️ **The work is right and its description was not**, and *a check that is
a reading* is the exact pattern AC4 exists to catch, found inside AC4's own delivery. **Guy's
arbitration: correct the record and register the real check** — over rewriting T2 to describe what
happened, which is the quietest form of the false sentence.

**4 · The scratch registry shipped with NO test, and the blind layer found it from the diff
alone.** A prove-to-red run during development is not a guard in the suite. ⚠️ **And the edge layer
then REPRODUCED a race the registry could not see**: its owner was the *helper's name*, so two
tests reusing one tag through one helper shared the directory and the bare `NotFound` came back
**with the registry silent** — while its doc promised *"a tag may be claimed by exactly ONE"*.
**Guy's arbitration: close it at the CALL SITE** (`#[track_caller]`), so the written promise
becomes true rather than narrowed. 🔑 **The two findings compose**: the missing test now exists,
and **the first registry would have passed it** — it reds only because the owner is a file and a
line. *A test written against the old mechanism would have vouched for the hole.*

**5 · Four documents quoted one on-screen sentence and not one quoted it correctly.** The blind
layer found the inconsistency from the diff and mis-assigned which was which; measured, the served
string is *"Nothing observed yet — run a scan and this fills in."* and every document truncated it,
each differently. All six occurrences normalised.

**6 · AC5 required *fixed or re-registered BY NAME*, and six of nine items got neither** — no CSS
file is touched anywhere in the diff, and none of the five register rows named them. ⚠️ **Including
the OFL font attribution, which the register and this story's own T3 assigned to it by name**:
`grep` for *OFL*, *Barlow*, *SIL* over the tree returned **zero**. A ticked task that delivered
nothing. The attribution is written now; the other five are registered with owners.

### ⚠️ Measured, and not visible any other way

- **Three raw ⚠️ glyphs added to the admin manual by this story rendered as nothing.** The edge
  layer built the PDF and read it back: `Missing character: There is no ⚠ (U+26A0) in font "Noto
  Sans"`, twelve warnings, `make` exiting **0**, and `pdftotext` showing the glyph dropped. Two of
  the three were plain body text with no admonition box to fall back on. Removed; the build now
  reports **zero** missing characters.
- **The extracted release notes ended with a dangling `---`** — the separator between two
  changelog sections. Trimmed by a visible loop rather than a `sed` incantation, *because a release
  step nobody can re-derive is a release step nobody checks*.
- **The version was interpolated into an awk regex.** `0.2.[0]` matched the `0.2.0` section; a tag
  containing `[` killed the step. It compares strings now. Re-proven across the edge layer's whole
  battery: `0.2.0` → 76 lines, and `0.9.9`, `0.2`, `0.2.0-rc1`, `0.2.[0]` all → 0.
- **`docker pull gcorbaz/opencmdb:0.1.1`** stood at the top of the Docker Hub page — the first
  command a visitor copies — and again in its embedded compose sketch. **All three layers found
  it**; two found the second site. It was diff CONTEXT, never a changed line.
- **The README and the admin manual asserted the tag was already published**, in the commit whose
  own status says it is deliberately not cut. Both now read true before the tag and after it.
- **The admin manual's security fix was the weakest of the three**: the `planned` box covered the
  correction while the paragraph below slid into the present tense describing a mechanism that
  does not exist. The whole passage is inside the box now.

### Refuted, each with the check

- **"`release.yml` will fail on every push to `master`"** (blind, HIGH-if-true, self-labelled a
  suspicion and naming its own check) — refuted: the workflow triggers on `push: tags: v*.*.*`
  and nothing else; `latest` is set by `metadata-action` on each tag, not by a master publish.
- The edge layer additionally ran and refuted **fourteen** suspicions, among them that the six
  `\begin{planned}` blocks attach to the wrong section (they do not — verified in source order and
  in the rendered PDF), that an upgrade from a `v0.1.1`-shaped database breaks (migrations 1–5
  apply cleanly), that the documented refusals are not real (`OPENCMDB_LOCALE=FR`, a half pair and
  a colon in the username all refuse to boot, by name), and that the registry made the suite
  order-dependent (25 runs at 64 threads, clean).

---

## Change Log

| Date | Change |
|---|---|
| 2026-08-24 | 🔴 **CODE-REVIEWED (three layers) and REPAIRED.** 24 raw findings → 19 distinct. **The two that would have hurt an operator**: `docker/.env.example` — the file the documents say to COPY — never got the variables this release makes mandatory, so following the instructions literally lands on 401 everywhere with the rescue absent from both file and instruction; and the image was published **before** the changelog check, so *fails the release* meant *fails to create the Release object*. **Two arbitrations (Guy)**: correct the record on the check that was a reading, and close the scratch registry at the CALL SITE. 🔑 The registry's missing test and the edge layer's reproduced race COMPOSE — **the first registry would have passed the new test**. 728 → **729**. |
| 2026-08-24 | **DEVELOPED, except the tag.** T0–T5 and T7 done; **T6 deliberately not cut** — a tag is not revertible and the order is review-then-tag. 🔴 The validation named two claimants of the scratch namespace and there are **six**, so issue #38 is closed as a CLASS (an ownership registry) rather than as an instance; ⚠️ the guard's first form turned one defect into eighteen failures, the panic poisoning its own mutex. 🔴 The prescribed check found **six** unmarked manual chapters where the validation read four — *a check sees what a reading misses, including a reading by a layer whose job was to look*. 🔴 And the sweep found what no report had: **story 6b.5's repair is PARTIAL**, its true sentence standing beside the false one it was meant to replace — visible on `/triage` below four rows dated one minute ago. ⚠️ The lifted screenshots exposed my own check as insufficient: it did not cover the version the shell renders. **728 tests both ways (7.9 s / 0.57 s), nine gates, both manuals build.** Five register rows. |
| 2026-08-24 | **VALIDATED by two fresh-context layers — 20 raw findings, 18 distinct, two reached by both.** 🔴 §0b's six needles are measured insufficient THREE ways, and AC4's own hedge is what survives: three documents claim credentials are *encrypted at rest*, a string `diagnostic.rs:1038` **forbids the product from rendering** because it is false; the status narrative is stale by ~15 epics; Tailwind is claimed as the stack where `CLAUDE.md` records the opposite decision, taken twice. 🔴 And §0 was wrong four ways — `Screen::Device` IS in the navigation (⚠️ *a correction `sprint-status.yaml:4578` had already made one story earlier*), AC3's Administrator Manual was named by no task, §0f claimed 3 of 5 rows that belong to others, and `cargo build --locked` cannot update `Cargo.lock`. 🔑 **Both arbitrations taken (Guy)**: the release notes get a VENUE (a GitHub Release from a committed `CHANGELOG.md`) because there is none today; and issue #38's REPRODUCED candidate cause is fixed here with the control the original refutation never ran — ⚠️ it had compared only `write_traps` tags with each other, measuring one half of the population. The issue stays OPEN: one occurrence is *a* cause, not *the* cause. |
| 2026-08-24 | Story created and CONTEXTED. 🔴 Four divergences from `epics.md:2316`, each measured: **twelve** preceding stories not eleven; **five** example screens not eight; the manuals carry **no screenshots** at all — their staleness is a SENTENCE (`user-manual.tex:151`, *"A dark theme is the default"*), so a story hunting images would have shipped it; and 🔴 **FOUR breaking changes, not three** — the product stopping being publicly readable is registered at `deferred-work.md:3323` and counted nowhere, **and it is the one an operator meets first**, before any colour or address. ⚠️ Also found: `docker/README.dockerhub.md` is already dated *"Since v0.2.0"* twice, so the release is a dependency of a document rather than the other way round. |
