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
  and IPAM, sources and alerts, self-diagnostic and commissioning — `epics.md:2190-2278`). On this
  reading 6b.3 ships a MECHANISM and no content.
  ⚠️ **This bullet first read *"each titled `(example)`"* and the fact-check REFUTED it: two of the
  five do** — 6b.6 and 6b.7. 6b.5 reads *"beside labelled example sections"*, and 6b.8 and 6b.9 name
  no example in their titles at all. The substance holds — those stories carry the example content —
  but the sentence generalised five cases from two, **and it was a premise of the arbitration below**.
  Corrected here so nobody cites it as measured.
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

**→ ✅ ARBITRATED (Guy, 2026-08-19): the THIRD reading — the example DATASET, the marker, the gate,
and ONE witness screen filled from it.** The eight remaining demonstration screens keep an `empty`
nature until their own story fills them, so stories 6b.5–6b.9 keep their subject and `epics.md` is
NOT edited (a story may not; the divergence is registered instead).

🔑 **Three natures, therefore, not two** — `Fed`, `Example`, `Empty` — and that is a consequence of
the arbitration rather than a preference: with two natures, the eight unfilled screens would have to
be declared *example*, and the marker would then assert that an empty `<main>` is a demonstration.
⚠️ **`Empty` is a temporary nature and must be written as one**: it is a statement that the screen's
own story has not landed, and the day 6b.9 closes there should be no `Empty` left. Give it a doc
comment that says so, so it is retired rather than inherited.

### §0a-bis. ✅ WHICH screen is the witness, and what it costs — decided at contexting

**`/devices`.** It carries the richest example content (a list, with sections), which is what lets
AC2's *smallest unit* be demonstrated at SECTION granularity inside one screen rather than only at
screen level.

⚠️ **The cost, stated rather than discovered later**: `/devices` belongs to story **6b.6**
(*"Inventory and device record (example)"*). Filling it here takes part of that story's subject.
What 6b.6 keeps is the **device RECORD** (`/device`), the `/devices/{id}` routing debt already
registered against it, and the fidelity pass over the list this story roughs in. **Registered**, so
6b.6's author meets a narrowed scope rather than a surprise.

🔴 **And AC2's *mixed* case is still NOT met by this, which must not be blurred.** Demonstrating the
smallest unit at section granularity proves the MECHANISM attaches below screen level; it does not
produce the case AC2 actually names — *real content beside example content in one frame*. That case
needs the dashboard's reach section, which is story 6b.5's deliverable (§0e). **AC2 ships as: the
mechanism MET, its mixed specimen STATED and owned by 6b.5.** Recording it as fully met would be the
sentence this epic has now been caught writing five times.

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
cannot occur.

✅ **T0 IS ANSWERED — the validation MEASURED it rather than reasoning about it, and the answer comes
with three qualifications that matter more than the answer itself.** A `Screen` variant wired into
`href`, `label_key`, `group` and `nature()` but omitted from `ALL` produces `error: variant 'Probe'
is never constructed` under `cargo clippy --workspace --locked -- -D warnings` — CI's exact
invocation (`.github/workflows/ci.yml:59`). The hole IS narrower than it read.

⚠️ **(i) It holds by a property of the GATE, not of the language.** The obvious bypass was tried — a
`#[test]` constructing `Screen::Probe`, the gesture anyone writing a test for a new screen makes
first — and it **fails to disable the guard**, because `cargo clippy --workspace` *without*
`--all-targets` never compiles `#[cfg(test)]` code. ⚠️ But a future line of PRODUCTION code
constructing a `Screen` literal outside `ALL` would make the variant *constructed* and **silence the
guard for that variant**.

⚠️ **(ii) `dead_code` is caught by `cargo clippy`, NOT by `cargo xtask ci`** — different commands,
and the eight gates do not include clippy. **A developer running only `cargo xtask ci` locally never
sees this red.** Only CI, or a manual clippy, does.

🔴 **(iii) AC4's closure is therefore PARTIAL and must be recorded as such.** The carrier is an
EXTERNAL dependency on a lint, and **nothing in the suite pins it**: no test fails if clippy stops
covering this one day. Do not write *"AC4 met"* without that sentence beside it. Whether to pin it —
and a pin would have to be a gate, since you cannot measure the absence of code by running code
(story 5.12) — is a decision to put to Guy rather than take.

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
  belongs.** Its trailing test module holds **eleven** guards (the first draft said ten — counted, not
  estimated, after the fact-check); `the_perimeter_has_a_single_reader` and
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

- [x] **T0 — the two measurements that precede design** (§0c): ✅ **both answered by the validation,
      before any production code** — `dead_code` DOES catch an unlisted variant under
      `clippy -D warnings`, and a `nature()` match reds on a new variant. ⚠️ Read §0c's three
      qualifications: the closure is partial, it lives outside `cargo xtask ci`, and nothing pins it
- [ ] **T1 — the nature, in the type** (AC4): `Nature` — **`Fed` / `Example` / `Empty`** (§0a) — on
      `screens::Screen` through a `match`, so the compiler refuses a screen with no declared nature.
      ⚠️ `Empty` carries a doc comment saying it is TEMPORARY and must be gone when 6b.9 closes
- [ ] **T2 — the partition over the ROUTE TABLE** (AC4), not inside the templates: every demo surface
      carries the marker, every fed surface does not, driven through the real router and asserted on
      the **HTTP body**, never on the template source. ⚠️ **It is `DATABASE_URL`-gated and therefore
      INVISIBLE locally** — `/triage` is the only `Fed` screen and needs a real pool (measured ~5.9 s
      with a database against 0.06 s without). Say so where the test lives, or a local green reads as
      coverage it is not
- [ ] **T3 — one partial, one key pair, one treatment** (AC1): `_example_marker.html`, `fr` + `en`,
      no `--accent-document`
- [ ] **T3b — the example DATASET and the witness screen** (§0a, §0a-bis): the dataset, and
      `/devices` filled from it. ⚠️ It must not open a connection — the demo router's state is `()`
      and that is what keeps AC3 true (§0d). A dataset that needs a READ breaks the carrier
- [ ] **T4 — the smallest unit** (AC2): the mechanism, demonstrated at SECTION granularity inside
      `/devices`. ⚠️ The MIXED specimen (real beside example) is 6b.5's — record AC2 as *mechanism
      met, mixed specimen stated*, never as simply met
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

🔑 Seven rows, not eighteen. Story 6b.2's lesson is that an unplayed row is a lie about coverage; the
number below is chosen so that every one of them WILL be played.

| # | Mutation | Prediction |
|---|---|---|
| M1 | Add a `Screen` variant, wire `href`/`label_key`/`group`, omit `nature()` | **fails to COMPILE** — read the real error and record it, do not cite `E0004` from this file |
| M2 | Add a `Screen` variant, wire everything, omit it from `Screen::ALL` | ⚠️ **prediction unknown on purpose** — §0c/T0 measures it. If GREEN, that is the story's finding and AC4 is not met until it is closed |
| M3 | Flip one demo screen's nature to *fed* | the partition test reds. ⚠️ **Predicted PANIC-carried, not assertion-carried** — the validation measured it dying on `unreachable!("Fed screens are not merged onto this router")` before reaching its own assertion. Record the carrier per row; never claim *"every red assertion-carried"* |
| M4 | Flip `/triage`'s nature to *example* | the partition test reds in the OTHER direction — a fed surface must never carry the marker |
| M7 | Flip an `Empty` screen's nature to `Example` | 🔴 **MY PREDICTION WAS WRONG AND THE VALIDATION MEASURED WHY — read this before writing the guard.** It predicted *"the product asserts a blank screen is a demonstration"*. In a design that dispatches CONTENT from `nature()`, an `Empty` screen promoted to `Example` receives the real dataset **with its marker**, so the lying-marker scenario is **structurally impossible** and cannot be what reds. M7 still reds — but only through a bookkeeping assertion (*exactly one witness screen*), and that assertion was measured to be its **SOLE carrier**: delete it and the test goes GREEN with a second screen declared `Example`, because *"this screen is declared Example and carries the marker"* is a true sentence. ⚠️ **A future tidy-up would remove that assertion as redundant without knowing it carries everything** — give it a comment saying so |
| M5 | Delete the marker partial's `fr` half, keep `en` | AC1 red via `every_key_carries_both_locales` — the NFR26 direction 6b.2 measured green before its guard existed |
| M6 | Give the marker `--accent-document` | 6b.1's reservation guard reds. ✅ **MEASURED, and my suspicion was REFUTED**: the guard scans `std::fs::read_dir("templates/")` at run time, not a frozen list, so it reds on the colour planted in a template that did not exist when it was written. *The guard survives the ordinary gesture; I doubted it and was wrong.* |

## Validation record — two fresh-context layers, 2026-08-19

**Mandatory here** (Guy, Epic 4 retrospective): every story is validated by two fresh-context agents
before `dev-story`. Both ran on a different model, each in its own git worktree.

**Layer 1, fact-check — 30 assertions verified one by one, 28 confirmed, 2 REFUTED, both mine.**
The two most load-bearing claims were reproduced BY COMPILATION rather than read: the triple `E0004`
on an unwired variant, and the `E0308` on `State<MySqlPool>` in the pool-free sub-router. Refuted:
*"each titled `(example)`"* (two of five — corrected in §0a, **and it was a premise of the
arbitration**), and *"ten guards"* in `screens.rs` (eleven).

**Layer 2, gap-hunt — it BUILT the whole mechanism** against a live `mariadb:10.11.11`: `Nature`,
the marker partial, the example dataset, `/devices` filled, and the route-table partition asserted
on the real HTTP body. It compiles, 611 → 612 tests, clippy and the eight gates green, `file-size`
unmoved at 1894.

🔴 **Its central finding is that MY OWN mutation M7 does not guard what I said it guards**, and only
building could reach it: content dispatched from `nature()` makes the lying-marker scenario
structurally impossible, so M7's real carrier is a bookkeeping assertion that a future tidy-up would
delete as redundant. ✅ **Two of my suspicions were REFUTED by measurement** — the
`--accent-document` guard survives a brand-new template (it reads the directory, not a list), and
AC3's compile carrier holds exactly as claimed, so T5's *"cite, add no test"* is the right
prescription. ⚠️ **And T2 is `DATABASE_URL`-gated and invisible locally**, which the story had said
for AC3 and not for T2 itself.

🔑 **What this pass changed, and it is the reason the step is not optional**: T0 is answered before a
line of production code, one arbitration premise was corrected, one mutation prediction was replaced
by a measurement, and two doubts of mine were closed as unfounded. **Three of the four open questions
I flagged at contexting were settled by someone other than me** — which is what the rule exists for.

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
