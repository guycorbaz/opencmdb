# Story 6b.3: The example-data marker, and the gate that keeps it honest

Status: ready-for-dev

Epic: 6b — *L'interface de la maquette*. **Third story**, after 6b.1 put the design system in the
binary and 6b.2 gave the product ten addresses. It is the story the previous one deferred to BY
NAME, and it is **priority in the strict sense**: since `d8cc438`, nine of the ten screens render an
empty `<main>` with nothing saying so, so the product reads as nine broken screens rather than nine
deliberate placeholders.

## Story

As the operator,
I want to know at a glance whether what I am reading comes from my network,
so that a demonstration is never mistaken for my inventory.

## Acceptance Criteria

Transcribed from `epics.md:2150-2170`, **unmodified** — the divergences are raised in §0 below rather
than edited into the criteria (a story may not edit an AC; only a retrospective may).

1. **Given** a surface whose content comes from the example dataset, **when** it renders, **then** it
   carries **one marker and one sentence** — one partial, one key pair, one visual treatment — never
   a per-screen improvisation.
2. **Given** that the dashboard is **mixed by nature** (the real reach section beside example stat
   cards), **when** the marker is placed, **then** it attaches to **the smallest unit that carries
   example content** — screen, section, or control. A screen-level-only marker would either lie
   about the real half or hide the example half.
3. **Given** any demo surface, **when** it renders, **then** **no database connection is opened and
   no row is written** (constraint 1), and a test says so.
4. **And** the partition is asserted **over the route table, not inside each template**: every demo
   surface carries the marker, every fed surface does not, and **a route added without a declared
   nature must fail rather than default**. ⚠️ *Epic 5's dominant defect class, named at its
   retrospective: a guard placed where the defect cannot occur reads as coverage and is none.* A test
   that checks the marker inside the templates that already have it proves nothing about the
   eleventh screen.

---

## §0 — What contexting found, and what needs Guy's arbitration BEFORE any code

Six findings. Four are measurements; two are contradictions between documents the project already
has. **Nothing below was taken from reading a summary — each carries the command that established
it.**

### §0a. 🔴 THE SCOPE CONTRADICTION: does this story own the example DATASET, or only the marker?

Two documents disagree, and the disagreement decides whether this story is small or large.

- **`epics.md`** titles it *"The example-data marker, and the gate that keeps it honest"* and puts
  the example CONTENT in stories **6b.5–6b.9** (dashboard, inventory and device record, applications
  and IPAM, sources and alerts, self-diagnostic and commissioning — `epics.md:2190-2278`, each
  titled *"(example)"*). On this reading 6b.3 ships a MECHANISM and no content.
- **The code says the opposite.** `screens.rs:170`, written by story 6b.2 and merged in `d8cc438`:
  *"Story 6b.3 owns the example dataset and the marker that says so."*

🔴 **The consequence is not editorial.** On `epics.md`'s reading, this story marks nine screens that
contain nothing — and *a marker over an empty screen says "this emptiness is an example", which is
false*. On `screens.rs`'s reading, this story also fills nine screens, which is four stories' worth
of content and would leave 6b.5–6b.9 with their subject removed.

⚠️ **A third reading exists and may be the honest one**: this story ships the mechanism AND a
minimal specimen — one screen with example content — so the marker has something true to mark and
the partition has a real positive case. The remaining eight screens keep the *empty* nature until
their own story fills them, which requires a **third** nature (fed / example / **empty**) rather
than the two the AC names.

**→ ARBITRATION REQUIRED. The task list below is written for the third reading and must be rescoped
if Guy chooses another.**

### §0b. 🔴 THE MARKER IS SPECIFIED BY NO DOCUMENT THIS PROJECT HAS — measured, not supposed

Story 6b.2's governing arbitration was **"the mock prevails"**. It gives nothing here.

- **The mock carries no marker.** `~/travail/Projets actuels/opencmdb/documents/opencmdb - maquette.html`,
  390 lines, was searched for `exemple`, `démonstration`, `demo` and `sample`: the only hits are
  **base64 font payloads** (`DEmo5Fg2…`, `DEMouJLS…`) and one comment reading *"demonstrated in
  foundations/"*. There is no marker, no banner, no badge, no sentence.
- **The UX specification is silent.** `grep -i "exemple|example data|démonstration|demonstration"`
  over `ux-design-specification.md` returns **nothing**.

🔑 So the mock cannot prevail, because it has nothing to prevail with. **This story INVENTS an
operator-visible surface**, which is the first time in Epic 6b that a story does so. It needs an
arbitration on the form and the words, and that arbitration should be recorded WITH the alternatives
refused — a banner per screen, an inline chip per section, a persistent header slot, or a treatment
of the content itself (dimming, hatching) each say something different about how much the operator
should trust what is under it.

⚠️ **And the wording is a translation obligation, not a label**: `nav.*` keys landed in `fr` + `en`
in 6b.2, and NFR26 makes a French UI silently rendering English a defect — story 6b.2's **M6-bis**
was measured GREEN before its guard existed. The guard now exists
(`every_key_carries_both_locales`, `screens.rs`), so new keys are covered the day they land.

### §0c. 🔴 AC4's carrier EXISTS but is HALF-BLIND, and the register gave the blind half to the wrong story

AC4 demands that *"a route added without a declared nature must fail rather than default"*. The
right shape is a `match` on `screens::Screen` returning the nature — and **the compiler really does
enforce that half**: the code review measured a new `Screen` variant producing **three `E0004`
errors** on `href`, `label_key` and `group`, which a `nature()` would join.

⚠️ **But `Screen::ALL` is a literal `[Screen; 10]` and the compiler does NOT check it for
exhaustiveness.** A variant wired into `href`, `label_key`, `group` and `nature` yet left out of
`ALL` compiles cleanly and **disappears from the navigation, from the routing, and from every test
that iterates `ALL`** — including the partition test AC4 asks for. That is precisely *the eleventh
screen* the AC names.

🔴 **The register assigned that finding to story 6b.6** (`deferred-work.md`, code review of 6b.2,
*"Owner: story 6b.6, the next story to add a screen"*). **AC4 makes it this story's**, because a
partition asserted over a table that can silently lose a row is a guard placed where the defect
cannot occur. ⚠️ **Not confirmed and worth one measurement before designing around it**: nobody has
checked whether `dead_code` under `-D warnings` catches an unlisted variant. Measure it first — if
it does, the hole is narrower than it reads.

### §0d. ⚠️ AC3 IS ALREADY DISCHARGED, and satisfying it with a runtime test would WEAKEN it

*"No database connection is opened and no row is written"* is carried **by the type** since
`d8cc438`: the nine demonstration screens sit on a `Router<()>` merged after `.with_state(pool)`, so
a handler taking `State<MySqlPool>` **fails to compile** — `E0308`, re-measured independently by two
code-review layers on the committed tree.

🔑 **A runtime test asserting "no query ran" would be strictly weaker than a compile failure, and
writing one is the very defect class AC4 quotes.** The right discharge is to CITE the existing
carrier and add nothing — and to say so in the story record, so a reviewer does not read the absence
of a new test as an omission. ⚠️ If Guy's answer to §0a gives a demo screen real content, re-check
this: content that needs a *shape* is fine, content that needs a *read* breaks the carrier.

### §0e. ⚠️ AC2's specimen DOES NOT EXIST YET, and cannot be manufactured honestly

AC2 turns on the dashboard being *"mixed by nature — the real reach section beside example stat
cards"*. Measured: `/dashboard` renders `empty_screen`, an empty `<main>`; the real reach section
lives on `/triage` (story 5.14b's two sections, inside `_gap_card.html`), and the dashboard's own
mixed shape is **story 6b.5's deliverable** (`epics.md:2190`).

So at this story's time there is **no mixed surface to place a marker on**. Either the smallest-unit
rule ships as a MECHANISM with its specimen deferred to 6b.5 — and then AC2 is *stated, not met*, and
must be recorded that way rather than ticked — or §0a's third reading supplies one deliberately.
🔑 **Do not tick AC2 on a hand-built fixture that no route serves**: that is a guard placed where the
defect cannot occur, and this epic has now shipped that shape five times.

### §0f. ⚠️ A false sentence is live in THREE files, and this story is the one that touches them

Ten screens minus `/triage` is **nine** demonstrations. Measured by `grep`:

- **"nine"** — `screens.rs:1`, `screens.rs:8`, `screens.rs:147`, `main.rs:419`
- **"eight"** — `page.rs:53`, `screens.rs:510`, `_nav.html:16`

The three *"eight of which are demonstrations"* are **false**, they shipped in `d8cc438`, and **all
three code-review layers missed them**. This story edits exactly these files and their nature
vocabulary; correcting the count here costs nothing and leaving it means a fourth reviewer meets a
contradiction inside one file. ⚠️ Registered rather than assumed: if Guy's §0a answer changes how
many screens are demonstrations, the true number changes with it — **fix the sentence at the end of
the story, from a recount, not from this paragraph**.

---

## Dev Notes

### What exists today (read, not assumed — `master` at `932c570`)

- **`crates/opencmdb-bin/src/screens.rs`** — `Screen` (10 variants), `NavGroup` (3), `Screen::ALL`,
  `href()`, `label_key()`, `group()`, `title_key()`, `router(perimeter)` returning `Router<()>`, and
  `empty_screen()` which renders the frame around `String::new()`. **This is where `nature()`
  belongs.** Its trailing test module holds ten guards; `the_perimeter_has_a_single_reader` and
  `every_key_carries_both_locales` are source-scanning properties whose idiom the new partition test
  should follow.
- **`crates/opencmdb-bin/src/page.rs`** — `Shell`, `render_shell(shell, body)`, `ShellPage`,
  `TriageState`, `triage_router()`. The marker partial will be included by `_shell.html` or by the
  screen body; `render_shell` is the one place that sees every screen.
- **`templates/_shell.html`** (the frame), **`_nav.html`** (ten entries, three groups),
  **`_gap_card.html`** (the fed content, untouched since 5.14b).
- **`locales/app.yml`** — 48 entries, every key in `fr` + `en`, guarded.
- **`assets/app.css`** — 356 lines on the mock's tokens. ⚠️ `--accent-document` (amber) is
  **RESERVED for the documenting gesture** (6b.1's arbitration, guarded by a test): the marker must
  NOT use it, or the reservation stops meaning anything.

### The house rules this story will be judged against

- **Prove-to-red**: a guard is observed failing before it passes, and the mutation is recorded.
  🔴 **And the mutation table is the thing to get right in this story specifically.** Story 6b.2
  prescribed **eighteen** mutations and executed **seven**; the two holes that reached production —
  M12 and M7 — were both rows written at contexting and never played. **Write fewer rows and play
  every one.** A row you will not execute is worse than no row: it reads as coverage.
- **A guard placed where the defect cannot occur reads as coverage and is none.** Epic 5's dominant
  class, and Epic 6b has reproduced it in every story so far.
- Doc comments must be TRUE; a false one is a defect. Prefer the weaker true sentence.
- No source file over 2000 code lines (`file-size` gate; largest today is 1894 — **check before
  growing `page.rs` or `main.rs`**, and split rather than grow).
- Every `pub` item documented, rustdoc idiom.

### Testing

- `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all`,
  `cargo xtask ci` (eight gates; `views-hash` reports `ℹ STALE` by design and must NOT be
  regenerated inside a story).
- ⚠️ **`DATABASE_URL` is unset locally**: database-backed tests return early and pass in silence.
  The tell is the clock — the bin suite runs in **0.06 s** locally against **5.38 s** in CI. Say what
  a local green does and does not prove.
- Baseline to start from: **611 tests** (384 bin + 161 core + 66 xtask), eight gates green.

## Tasks / Subtasks

**Written for §0a's third reading.** Rescope on Guy's answer before starting.

- [ ] **T0 — the two measurements that precede design** (§0c): does `dead_code` under `-D warnings`
      catch a `Screen` variant absent from `ALL`? And does a `nature()` `match` red on a new variant?
      Both answers change T2's shape
- [ ] **T1 — the nature, in the type** (AC4): `Nature` on `screens::Screen` through a `match`, so the
      compiler refuses a screen with no declared nature. ⚠️ Two variants or three — §0a decides
- [ ] **T2 — the partition over the ROUTE TABLE** (AC4), not inside the templates: every demo surface
      carries the marker, every fed surface does not, driven through the real router. ⚠️ Close
      `Screen::ALL`'s blind half first or the test can silently lose a row
- [ ] **T3 — one partial, one key pair, one treatment** (AC1): `_example_marker.html`, `fr` + `en`,
      no `--accent-document`
- [ ] **T4 — the smallest unit** (AC2): the mechanism; ⚠️ its specimen is §0a's business and AC2 is
      **stated, not met**, unless one exists
- [ ] **T5 — AC3 by CITATION** (§0d): the compile carrier already holds it; add no runtime test, and
      record why the absence is deliberate
- [ ] **T6 — look at every screen in a browser**, `OPENCMDB_LOCALE=fr` exported first (the default is
      `en`, `main.rs:344`). *A status code is not a look* — 6b.1 and 6b.2 were both caught on this.
      ⚠️ That citation read `main.rs:291` in this file's first draft, copied from story 6b.2's own
      dossier where it was true; **verified and corrected at contexting**. Every line number here was
      re-measured against `932c570` — treat them as dated the day they are read, not as durable
- [ ] **T7 — the register**, both directions: `grep -n "6b.3" deferred-work.md` **before starting and
      before finishing**. ⚠️ Four consecutive stories have missed a row naming them
- [ ] **T8 — the count sweep** (§0f): recount, then correct `eight`/`nine` from the recount
- [ ] **T9 — prove-to-red**, predictions written FIRST, **and every prescribed row executed**

## Prove-to-red — deliberately short

🔑 Six rows, not eighteen. Story 6b.2's lesson is that an unplayed row is a lie about coverage; the
number below is chosen so that every one of them WILL be played.

| # | Mutation | Prediction |
|---|---|---|
| M1 | Add a `Screen` variant, wire `href`/`label_key`/`group`, omit `nature()` | **fails to COMPILE** — read the real error and record it, do not cite `E0004` from this file |
| M2 | Add a `Screen` variant, wire everything, omit it from `Screen::ALL` | ⚠️ **prediction unknown on purpose** — §0c/T0 measures it. If GREEN, that is the story's finding and AC4 is not met until it is closed |
| M3 | Flip one demo screen's nature to *fed* | the partition test reds — the marker vanishes where it is owed |
| M4 | Flip `/triage`'s nature to *example* | the partition test reds in the OTHER direction — a fed surface must never carry the marker |
| M5 | Delete the marker partial's `fr` half, keep `en` | AC1 red via `every_key_carries_both_locales` — the NFR26 direction 6b.2 measured green before its guard existed |
| M6 | Give the marker `--accent-document` | 6b.1's reservation guard reds — ⚠️ **verify this before believing it**: the guard may scan only `app.css` and the templates it knew about |

## References

- `_bmad-output/planning-artifacts/epics.md:2150-2170` — the acceptance criteria, verbatim
- `_bmad-output/planning-artifacts/epics.md:2086-2109` — Epic 6b's six measured constraints
- `_bmad-output/planning-artifacts/epics.md:2190-2278` — stories 6b.5–6b.9, which carry the example content (§0a)
- `_bmad-output/implementation-artifacts/6b-2-shell-navigation-and-ten-routes.md` — the shell, its guards, and the review that found four of them wanting
- `_bmad-output/implementation-artifacts/deferred-work.md` — the `Screen::ALL` row (§0c) and the nine-empty-screens row
- `crates/opencmdb-bin/src/screens.rs` — `Screen`, `ALL`, `router`, `empty_screen`, and the ten guards
- `docs/project-context.md` / `CLAUDE.md` — the twins; both carry 6b.2's paragraph as of `932c570`
- The mock: `~/travail/Projets actuels/opencmdb/documents/opencmdb - maquette.html` — **outside this repository**, 390 lines, and it carries NO marker (§0b)

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
