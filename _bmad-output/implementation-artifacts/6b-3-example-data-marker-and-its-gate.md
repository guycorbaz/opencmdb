# Story 6b.3: The example-data marker, and the gate that keeps it honest

Status: done

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
- [x] **T1 — the nature, in the type** (AC4): `Nature` — **`Fed` / `Example` / `Empty`** (§0a) — on
      `screens::Screen` through a `match`, so the compiler refuses a screen with no declared nature.
      ⚠️ `Empty` carries a doc comment saying it is TEMPORARY and must be gone when 6b.9 closes
- [x] **T2 — the partition over the ROUTE TABLE** (AC4), not inside the templates: every demo surface
      carries the marker, every fed surface does not, driven through the real router and asserted on
      the **HTTP body**, never on the template source. ⚠️ **It is `DATABASE_URL`-gated and therefore
      INVISIBLE locally** — `/triage` is the only `Fed` screen and needs a real pool (measured ~5.9 s
      with a database against 0.06 s without). Say so where the test lives, or a local green reads as
      coverage it is not
- [x] **T3 — one partial, one key pair, one treatment** (AC1): `_example_marker.html`, `fr` + `en`,
      no `--accent-document`
- [x] **T3b — the example DATASET and the witness screen** (§0a, §0a-bis): the dataset, and
      `/devices` filled from it. ⚠️ It must not open a connection — the demo router's state is `()`
      and that is what keeps AC3 true (§0d). A dataset that needs a READ breaks the carrier
- [x] **T4 — the smallest unit** (AC2): the mechanism, demonstrated at SECTION granularity inside
      `/devices`. ⚠️ The MIXED specimen (real beside example) is 6b.5's — record AC2 as *mechanism
      met, mixed specimen stated*, never as simply met
- [x] **T5 — AC3 by CITATION** (§0d): the compile carrier already holds it; add no runtime test, and
      record why the absence is deliberate
- [x] **T6 — look at every screen in a browser**, `OPENCMDB_LOCALE=fr` exported first (the default is
      `en`, `main.rs:344`). *A status code is not a look* — 6b.1 and 6b.2 were both caught on this.
      ⚠️ That citation read `main.rs:291` in this file's first draft, copied from story 6b.2's own
      dossier where it was true; **verified and corrected at contexting**. Every line number here was
      re-measured against `932c570` — treat them as dated the day they are read, not as durable
- [x] **T7 — the register**, both directions: `grep -n "6b.3" deferred-work.md` **before starting and
      before finishing**. ⚠️ Four consecutive stories have missed a row naming them
- [x] **T8 — the count sweep** (§0f): recount, then correct `eight`/`nine` from the recount
- [x] **T9 — prove-to-red**, predictions written FIRST, **and every prescribed row executed**

### Review Findings — three layers, 2026-08-19

Blind Hunter (diff only, no repository access), Edge Case Hunter (own worktree, live
`mariadb:10.11.11`, every mutation re-executed), Acceptance Auditor (own worktree, `epics.md` and
the twins). All three on Sonnet 5 — a different model from the implementer. **3 decision-needed,
10 patch, 1 defer, 4 dismissed with the check that dismissed them — all three decisions arbitrated by
Guy and all ten patches APPLIED on 2026-08-19, each guard proven to red before it passed.**

🔑 **Three of the story's own recorded measurements were refuted by re-execution** (M3's carrier, M4's
count, M7's aftermath) — the mutation table was believed rather than replayed, in the story whose own
Dev Notes warn that its driver lied twice. 🔴 **And one finding was reachable by NO layer alone**: the
Blind Hunter could only suspect the CSS, the Edge Case Hunter verified the marker's classes and not
the content's, and it took a third recount to confirm it.

#### Decision-needed

- [x] [Review][Decision] ✅ **ARBITRATED (Guy, 2026-08-19): option (e) — a source-scanning TEST, not a
      ninth gate and not clippy-in-`xtask`.** 🔑 The story's §0c(iii) reasoning was MISAPPLIED and
      that is what had closed the cheap door: story 5.12's *"you cannot measure the absence of code
      by running code"* governs an UNBOUNDED absence — no file in the tree, including files that do
      not exist yet. This property is bounded and present: **does `Screen::ALL` name every variant of
      `enum Screen`**, both constructs in one file, both existing now. A test reading the source
      measures it exactly, and **the idiom is already in this very file** — `every_key_carries_both_locales`
      reads `include_str!("../locales/app.yml")`, `the_perimeter_has_a_single_reader` walks `src/`;
      the story's own Dev Notes name both as the model. It runs under `cargo test --workspace`, so M2
      reds in the suite where none of the eight gates sees it. **Option (b) is refused on a
      MEASUREMENT, not a preference**: clippy inside `cargo xtask ci` does not survive the bypass
      line. **And `NavGroup::ALL` carries the identical hole** (a literal `[NavGroup; 3]`) — one
      property covers both enums where a gate written for `Screen` would cover one. ⚠️ **Its limit is
      to be WRITTEN, not implied**: a tripwire against the ordinary gesture, never a barrier — move the
      enum to another file and it goes blind (story 5.12's narrowing, third application).

      *Original finding:* **AC4's only carrier is defeated by one ordinary line** —
      `Screen::ALL` is a literal array; a variant wired into every `match` and omitted from it passes
      the build, all 613 tests and **all eight `cargo xtask ci` gates**, reddening only
      `clippy -D warnings` (M2, re-confirmed independently by two layers). 🔴 **New measurement: the
      Edge Case Hunter added one throw-away production line — `let _bypass = Screen::Probe;` inside
      `router()`, nothing to do with `ALL` — and `clippy --workspace --locked -- -D warnings` exits
      0 while the variant is still absent from `ALL`, from the navigation, from the routing and from
      the partition test itself.** So AC4's carrier is not merely external to the project's gates; it
      is silenced by a gesture nobody would recognise as dangerous. The story reserved this question
      for Guy; the measurement is what changed. Options: (a) a ninth `xtask` gate asserting the
      source property (`Screen::ALL` names every variant); (b) add `clippy -D warnings` to
      `cargo xtask ci`, which closes M2 but not the bypass; (c) accept it as a TRIPWIRE and narrow the
      promise in writing, on story 5.12's precedent; (d) defer to 6b.6.
- [x] [Review][Decision] ✅ **ARBITRATED (Guy, 2026-08-19): option (a) — the eight screens get the
      sentence NOW.** The mechanism exists (`Nature::Empty` already dispatches a body; it returns
      `String::new()`): one partial, one key pair in `fr`+`en`, and the partition test gains an
      assertion that every `Empty` screen carries it. 🔑 **The divergence from `epics.md:2092`
      DISAPPEARS rather than being registered** — no premise of Guy's is quietly revised, and the
      epic's own goal sentence (*"every screen states truthfully whether what it shows comes from the
      product or from an example dataset"*) becomes true for ten screens instead of one. It takes
      nothing from 6b.5–6b.9, which ship example CONTENT: a *"not built yet"* sentence is not example
      content and is REPLACED when the content lands — which is exactly what `Nature::Empty`'s doc
      already promises, a temporary nature that must be gone when 6b.9 closes. ⚠️ **Not to be
      overstated**: the sentence is not *"an example dataset with a text saying so"*. It meets the
      premise's spirit, not its letter; the dataset stays owed by 6b.5–6b.9.

      *Original finding:* **`epics.md:2092` is a premise of Guy's own and eight screens do not meet
      it** — *"all ten screens ship; those whose code is not implemented show an example dataset with
      a text saying so"*, written 2026-08-13 under the heading *"this epic's premises and not open
      questions"*. Measured: eight screens are `Nature::Empty => String::new()`, byte-identical to
      what they rendered before this story — no dataset, no text, nothing. ⚠️ **Neither contexting,
      nor either fresh-context validation layer, nor the arbitration itself surfaced that sentence**,
      and no register row records a divergence from it. **The bound, stated so the finding is not
      inflated**: *"all ten screens ship"* is plausibly a property of the EPIC at its close —
      `epics.md` itself places the example content in 6b.5–6b.9 — in which case nothing is violated
      today. What holds under BOTH readings is narrower and still true: this story's own motivation
      (*"nine screens read as broken, so 6b.3 is priority in the strict sense"*) is met for ONE
      screen; for the other eight the only change is a type no operator can see. Options: (a) the
      eight get the sentence now — the mechanism exists, it costs one partial and one key pair;
      (b) record the 2026-08-19 arbitration as a deliberate revision of the 2026-08-13 premise;
      (c) leave it and register the divergence against 6b.5–6b.9.
- [x] [Review][Decision] ✅ **ARBITRATED (Guy, 2026-08-19): option (c) — the nature CARRIES its
      content.** `Nature::Example(ExampleContent)`, one variant today, so declaring a screen `Example`
      obliges naming its content and **"declared `Example` with no content of its own" becomes
      unrepresentable**. This is story 5.6's gesture — the self-pair closed IN THE TYPE rather than in
      a test — and it is what this story's own AC3 argues for: a compile refusal beats an assertion,
      and the assertion standing here is the one M7 records as deletable by a future tidy-up. 🔴
      **Option (a) was WITHDRAWN at the arbitration, not chosen against**: a `match` on the screen
      already has an arm for every screen, so flipping an existing screen from `Empty` to `Example`
      forces nothing — *it does not close what it looks like it closes*. ⚠️ **The deflation, recorded
      because it is real**: the defect needs a SECOND `Example` screen, which story 6b.5 adds, and its
      author would see the device inventory appear under the dashboard by looking. Option (b) plus a
      referral to 6b.5 was defensible at zero cost; (c) was taken because the window opens at the very
      next story rather than someday, and forty lines close the class.

      *Original finding:* **`Nature::Example` dispatches a screen-agnostic body** —
      `screens.rs:250` is `Nature::Example => crate::page::devices_example_body()`, and
      `page.rs:615` `devices_example_body()` **takes no screen argument**. Declaring any second
      screen `Example` renders the device inventory under that screen's own heading, and the sole
      carrier is a bookkeeping count assertion the story's own M7 row records as deletable by a
      future tidy-up. The story saw half of this as a coverage question; it is also a production one —
      the compiler enforces that a nature is DECLARED, never that the content MATCHES the screen.
      Options: (a) key the body on the SCREEN so a second `Example` screen fails to compile until it
      declares its content — compiler-carried, which is the preference this story's own AC3 argues
      for; (b) keep the count assertion and its comment; (c) `Nature::Example(…)` carrying its body
      in the type.

#### Patch

- [x] [Review][Patch] The headline test count is false on its left half: `591 → 613` where the
      baseline is 611 — the story's own Dev Notes say so, and 386+161+66=613 makes the delta +2, not
      +22. `591` is story 6b.2's baseline reused, and `sprint-status.yaml:182` carries the correct
      `591 → 611` for 6b.2 twelve lines from the wrong figure [6b-3-example-data-marker-and-its-gate.md:412, :443; sprint-status.yaml:83]
- [x] [Review][Patch] The tracking file's contexting entry says the mutation table is *"deliberately
      SIX rows"*; the table delivered in the same commit has seven, and the Debug Log says *"seven
      prescribed, nine run"* [sprint-status.yaml:175]
- [x] [Review][Patch] **The i18n guard cannot catch a wrong key that resolves.** Measured live:
      setting `role_key: "example.badge"` on `nas-01` — a real key from the wrong namespace — leaves
      **all 613 tests green, all eight gates green, clippy clean**, and `/devices` renders
      *"Exemple"* in the Role column where *"Stockage"* belongs. M8/M8b check SHAPE (starts with
      `example.`) and RESOLVABILITY (`!= itself`), never WHICH key. Epic 5's dominant class again, in
      the guard this story added to close an operator-visible i18n defect. Fix: assert the namespace
      matches the field (`example.role.*`, `example.reason.*`), and prove it to red with the measured
      mutation [example_data.rs]
- [x] [Review][Patch] **`.screen-section` is used twice and defined nowhere, and `.rows` is a
      definition-list ruleset applied to a `<table>`.** `_devices_example.html:8,37` carry
      `class="screen-section"`, absent from `app.css`; `class="rows"` on both tables inherits
      `margin: 0` and nothing else, since every descendant rule (`.rows .row`, `.rows dt`,
      `.rows dd`) targets the `<dl>` shape `_gap_card.html` uses. **The sheet contains no `table`,
      `th` or `td` rule at all.** So the one witness screen this story exists to produce renders as
      browser-default tables inside unstyled sections. 🔑 **This is precisely what the missing browser
      hid**: the served TEXT is correct in every respect a layer measured — content, escaping, French
      — and the visual result is not. **The bound**: §0a-bis already registers *"the fidelity pass over
      the list this story roughs in"* to 6b.6, so the polish is owed elsewhere; what is a defect here
      is a class that matches nothing and a ruleset borrowed for a shape it does not fit — **reuse of
      the design system in appearance and none in fact** [_devices_example.html:8,11,37,40; app.css]
- [x] [Review][Patch] **The stale register row 6b.2's review left is now contradicted by this story's
      own measurement.** `deferred-work.md:3821-3825` still reads *"Not confirmed: it was not measured
      whether `dead_code` under `-D warnings` catches it"* and *"Owner: story 6b.6"* — while §0c
      quotes that row, MEASURES the thing it calls unconfirmed, and reassigns the question to Guy
      forty lines below. Two owners and a false *"not confirmed"* in one register. 🔑 **And T7's check
      structurally could not catch it**: it prescribes `grep -n "6b.3"`, and the row never names 6b.3
      — *a grep on your own name cannot find the row that speaks about you without naming you.* Fix
      the row, and record the lesson where T7 lives [deferred-work.md:3821]
- [x] [Review][Patch] **M3's carrier is misrecorded.** The table says it reds on the premise assertion
      `probed >= 9`; re-executed, it reds on the status assertion two lines earlier —
      `left: 404, right: 200` at `main.rs:903`. The cause is worth more than the correction: a screen
      flipped to `Fed` leaves the pool-free router (dynamic, keyed on `nature()`) but **nothing adds it
      to the main router, whose inclusion is hardcoded to `/triage`** — so it 404s instead of ever
      reaching the premise. Record the asymmetry, not just the carrier [6b-3…md, M3 row; main.rs:424-427, :903]
- [x] [Review][Patch] M4's count is wrong: **25 failed**, not 15 (`361 passed; 25 failed`, panic
      *"Overlapping method route… GET /triage"* at `main.rs:427`). The panic message matches; the
      figure does not [6b-3…md, M4 row]
- [x] [Review][Patch] M7's aftermath is off by one: deleting the sole-carrier assertion leaves
      **386 green**, not 385 — deleting an assertion does not remove a test. The behavioural claim
      (the guard vanishes, a second `Example` screen goes unnoticed) is confirmed [6b-3…md, M7 row]
- [x] [Review][Patch] `ExampleSighting::mac`'s doc comment says *"when it gave one"* — an
      `Option<&str>` sentence over a field that is a hard `"—"` literal rendered straight into the
      HTML. Make the doc true or the type honest; a false doc is a defect and the weaker true
      sentence is preferred [example_data.rs]
- [x] [Review][Patch] Record the four refuted suspicions in the story, with the check that refuted
      each, so the next reader does not re-chase them: **(1)** an inlined stylesheet would make
      `body.contains("example-marker-badge")` match on every page and destroy the partition's
      negative half — refuted, `/dashboard` and `/triage` carry **zero** occurrences in the live body;
      **(2)** `.rows`/`.mono` missing from the sheet — refuted, both exist (`.screen-section` is the
      one that does not); **(3)** a `#[test]` constructing an unlisted variant would silence
      `dead_code` like production code — refuted, `clippy` without `--all-targets` never compiles
      `#[cfg(test)]`; **(4)** `epics.md:2330`'s *"the eight example screens"* looked like a fourth
      site of the count defect — refuted, it is **correct, and correct by accident**: the arbitration
      pulled `/devices` forward, leaving exactly eight for 6b.5–6b.9. Nothing records that its
      correctness is accidental [6b-3…md]

#### Defer

- [x] [Review][Defer] **The look is still unverified by eye, for the third consecutive story** —
      no browser was available in this environment either, so typography, spacing, colour contrast
      and badge alignment remain unmeasured. This review found the CSS defect by RECOUNTING the
      sheet, not by seeing the page; a recount catches a class that matches nothing and cannot catch
      a page that is merely ugly. The epic's own DoD already names axe-core green on the ten routes.
      **Owner: Epic 6b's DoD, with story 6b.12's release sweep** — deferred, environmental, not
      caused by this change [_devices_example.html, app.css]

---

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

Claude Opus 5 (1M context), 2026-08-19.

### Debug Log References — the mutation pass, every prescribed row EXECUTED

🔑 **Seven rows were prescribed and nine were run** (M8/M8b were added when looking found a defect
the table had not imagined). Story 6b.2 prescribed eighteen and played seven; that is the failure
this table was sized against. **Carriers are named per row — *"every red assertion-carried"* is NOT
claimed, and two rows diverged from their prediction.**

| # | Mutation | Predicted | MEASURED | Carrier |
|---|---|---|---|---|
| M1 | `Screen` variant wired into `href`/`label_key`/`group`, no `nature()` arm | fails to compile | ✅ `error[E0004]: non-exhaustive patterns: 'Screen::Probe' not covered` | compiler |
| M2 | variant wired everywhere, omitted from `Screen::ALL` | unknown by design | 🔴 **build passes, 613 tests pass, `cargo xtask ci` ALL EIGHT GATES GREEN** — only `clippy -D warnings` reds (`variant 'Probe' is never constructed`) | lint, outside the project's own gates |
| M3 | `/devices` `Example` → `Fed` | partition reds | ✅ 1 red — 🔴 **and the recorded carrier was WRONG, corrected by the code review's re-execution**: it reds on the STATUS assertion two lines earlier, `left: 404, right: 200` at `main.rs:903`, never on the premise `probed >= 9`. 🔑 The cause is worth more than the correction: the demo router's exclusion is DYNAMIC (keyed on `nature()`) while **the main router's inclusion is hardcoded to `/triage` alone**, so a screen that becomes `Fed` is merged nowhere and 404s | named assertion (status), not the premise this table first claimed |
| M4 | `/triage` `Fed` → `Example` | partition reds | ⚠️ **25 red, not 1 and not the 15 this table first recorded** (`361 passed; 25 failed`, re-measured by the code review) — `/triage` is then merged onto BOTH routers and axum panics at construction (*"Overlapping method route… GET /triage"*, `main.rs:427`), so every test that builds `app()` dies. The panic message was right; the figure was not | panic at router construction |
| M5 | marker's `fr` half deleted | AC1 red | ✅ 1 red, `every_key_carries_both_locales` | named assertion |
| M6 | `--accent-document` in the marker | 6b.1's guard reds — *"verify before believing"* | ✅ 1 red — **my doubt was REFUTED**: the guard reads `templates/` at run time, so a brand-new partial is covered | named assertion |
| M7 | an `Empty` screen → `Example` | ⚠️ prediction already corrected at validation | ✅ 1 red, on the COUNT assertion — and with the count deleted, **386 green** (not the 385 first recorded: deleting an assertion does not remove a test). The behavioural claim is confirmed — the per-screen loop notices nothing. ⚠️ **The count is no longer the SOLE carrier since M12**: the nature now carries its content | named assertion (count); the type carries the other half |
| M8 | an English literal back in place of a `role_key` | *(not prescribed — written after looking)* | ✅ 1 red | named assertion |
| M8b | a key that looks right but does not resolve (`example.role.storag`) | *(not prescribed)* | ✅ 1 red | named assertion |

**The code review's own pass — four mutations, four measured, every one on a committed-equivalent
base** (the files were copied to a scratchpad and restored from the copy, never `git checkout --`:
that gesture is what destroyed an uncommitted fix in this story and in story 6.1 before it). Run
against a live `mariadb:10.11.11`, so the database-backed half really executed (bin suite 5.6–5.8 s
against 0.06 s without).

| # | Mutation | Predicted | MEASURED | Carrier |
|---|---|---|---|---|
| M9 | a `Screen` variant wired into `href`/`label_key`/`group`/`nature`, omitted from `Screen::ALL` | the new guard reds **in the suite**, where M2 left all 613 green | ✅ **1 red**, `screens.rs:772` — *"Screen::Probe is declared but absent from Screen::ALL"*. Build succeeds, as under M2; the difference is that `cargo test --workspace` now notices | named assertion |
| M10 | `not_built_yet_body` returns `String::new()` — an `Empty` screen stops saying it is not built | the partition's new half reds | ✅ **1 red**, `main.rs:938` — *"/dashboard is not built yet and must say so"* | named assertion |
| M11 | `role_key: "example.badge"` — a real key that resolves, from the wrong namespace | reds, where the Edge Case Hunter measured it **GREEN across all 613 tests, all eight gates and clippy** | ✅ **1 red**, `example_data.rs:151` — *"is a real key that resolves, and it is the WRONG one"* | named assertion |
| M12 | a second screen declared `Nature::Example` without naming its content | fails to COMPILE | ✅ `error[E0308]: match arms have incompatible types`, and the compiler names the cause itself: *"`Example` defines an enum variant constructor here, which should be called"* | compiler |

⚠️ **M12's refusal is real and its MESSAGE is not the crisp one**: `E0308` speaks of arm types rather
than of a missing content, because a payload-carrying variant used bare is a constructor function.
Recorded as measured rather than as hoped — the refusal is what closes the class, the diagnostic is
merely adequate.

⚠️ **The driver lied twice, and both are recorded because both cost real work.** M8b first came back
GREEN: it ran against a tree that **did not compile**, and the driver grepped for `FAILED` test
lines, which a compile failure does not produce. And the tree did not compile because
`git checkout -- <file>` had reverted that file to the last COMMIT — which predated the i18n fix,
silently discarding it. *Story 6.1's incident, reproduced by me: a file revert equals a mutation
revert only on a committed baseline.* Both mutations were re-run on a committed base and both red.

### Completion Notes List

🔴 **Two defects were found by LOOKING at the screen, and neither was reachable by any test.**

1. **The example data rendered in English under a French interface** — *Storage*, *Network*,
   *No declared record matches this address* — with the whole suite green, because a literal is not
   a key and `every_key_carries_both_locales` can only see keys. An NFR26 violation with no possible
   carrier on the locale side. Fixed by making the copy i18n KEYS, and closed by a new guard
   asserting both that each value IS a key and that it RESOLVES (M8, M8b). ⚠️ **This is what T6
   exists for**: *a status code is not a look*, and neither is a green suite.

2. **The partition's `Fed` half was gated on a fact that does not govern the code under test.** It
   skipped unless `DATABASE_URL` was set — but `lazy_pool()` is a hardcoded dead URL and ignores that
   variable entirely, so the `Fed` half could never have passed and was never even attempted. Found
   by running the suite WITH a database, where it reddened at once (`left: 500, right: 200`). It now
   connects and migrates like every other database-backed test. *A gate keyed on a fact that does not
   govern the code under test is not a gate.*

**AC by AC, and two are deliberately NOT ticked as fully met:**

- **AC1 — MET.** One partial, one key pair, one treatment. The neutral ramp, never `--accent-document`.
- **AC2 — MECHANISM MET, MIXED SPECIMEN STATED.** Two sections on `/devices`, each carrying the
  marker, proves placement below screen level. The case AC2 names — real beside example in one frame
  — needs the dashboard and is story 6b.5's. Registered.
- **AC3 — MET BY CITATION, and no runtime test was added.** The `Router<()>` shape refuses
  `State<MySqlPool>` at compile time; a run-time assertion would be strictly weaker and would be the
  epic's own dominant defect. The reasoning is in `screens.rs`'s module doc, not only here.
- **AC4 — PARTIALLY MET AT THE STORY'S CLOSE; MET AFTER THE CODE REVIEW, with its limit stated.**
  The nature is a compiler-checked `match` and the partition runs over the route table on the real
  HTTP body — both true as shipped. What was missing was `Screen::ALL`'s blind half: closed by
  `clippy` alone, **outside `cargo xtask ci`**, with nothing in the suite pinning it (M2) — and the
  review measured that carrier **defeated by one throw-away production line** constructing the
  variant. Guy's arbitration (2026-08-19): a source-scanning TEST, not a ninth gate and not
  clippy-in-`xtask`. 🔑 The story's reasoning that it *"would have to be a gate"* was misapplied —
  story 5.12's rule governs an unbounded absence, and this property is bounded and present in one
  file. ⚠️ **The limit is written, not implied**: a tripwire against the ordinary gesture, never a
  barrier — move either enum to another module and the guard goes blind. And `NavGroup::ALL` had the
  identical hole, which is why the guard is a property over both enums rather than an enumeration
  for one.

**611 → 613 tests** (386 bin + 161 core + 66 xtask) at the story's close, **611 → 614** after the
code review. ⚠️ This read **591 → 613** in two places here and one in `sprint-status.yaml` until the
review recounted it: `591` is story 6b.2's baseline, reused. The Dev Notes above state 611, and
386+161+66 makes the delta **+2**, not +22 — a dated figure and a living one in one sentence, and it
is always the living one that rots. Eight gates green, fmt and clippy clean, and the
suite was run BOTH ways: 0.07 s without a database and 5.71 s against a live `mariadb:10.11.11`,
which is the tell that the database-backed half really executed.

⚠️ **T6 was a real look**, not a status sweep: the server was run against a live database with
`OPENCMDB_LOCALE=fr`, and `/devices`, an `Empty` screen and `/triage` were read as rendered text.
That is what found defect 1. **It is still not a visual check in a browser** — no browser was
available in this environment — so typography, spacing and colour remain unverified by eye.

### File List

| File | Change |
|---|---|
| `crates/opencmdb-bin/src/example_data.rs` | **new** — the example dataset, i18n keys for its copy, and the guard that keeps it translated |
| `crates/opencmdb-bin/templates/_example_marker.html` | **new** — the marker: one partial, one key pair |
| `crates/opencmdb-bin/templates/_devices_example.html` | **new** — the witness screen's two sections |
| `crates/opencmdb-bin/src/screens.rs` | `Nature`, `Screen::nature()`, the body dispatch, AC3's citation, the count sweep |
| `crates/opencmdb-bin/src/page.rs` | the resolved view structs, `devices_example_body()`, the new strings, the count sweep |
| `crates/opencmdb-bin/src/main.rs` | `mod example_data`, and the route-table partition test |
| `crates/opencmdb-bin/locales/app.yml` | 14 keys, both locales |
| `crates/opencmdb-bin/assets/app.css` | the marker's treatment, on the neutral ramp |
| `crates/opencmdb-bin/templates/_nav.html` | the count sweep |
| `_bmad-output/implementation-artifacts/deferred-work.md` | five rows, plus one from the code review |

**Added by the code review (2026-08-19):**

| File | Change |
|---|---|
| `crates/opencmdb-bin/templates/_not_built_yet.html` | **new** — the line an `Empty` screen carries, on its own classes so the partition asserts both directions for both markers |
| `crates/opencmdb-bin/src/screens.rs` | `Nature::Example(ExampleContent)`, the new `ExampleContent` enum, the `Empty` dispatch, and `every_variant_of_a_navigated_enum_is_listed_in_all` |
| `crates/opencmdb-bin/src/page.rs` | `not_built_yet_body`, the `NotBuiltYet` template struct, two `Strings` fields, and the sighting MAC resolved from an `Option` |
| `crates/opencmdb-bin/src/example_data.rs` | `mac: Option<&'static str>` so its doc is true, and the i18n guard's THIRD half — the key's namespace |
| `crates/opencmdb-bin/src/main.rs` | the partition's second half (`Empty` says so) and the count assertion's corrected comment |
| `crates/opencmdb-bin/locales/app.yml` | `pending.badge` and `pending.sentence`, both locales |
| `crates/opencmdb-bin/assets/app.css` | `.not-yet`, `.not-yet-badge`, and **`.screen-section` / `.grid`, which were used by the template and defined nowhere** |
| `crates/opencmdb-bin/templates/_devices_example.html` | `class="rows"` → `class="grid"`: `.rows` is a `<dl>` ruleset and matched nothing on a `<table>` |

### Change Log

| Date | Change |
|---|---|
| 2026-08-19 | Contexted: six findings, four needing arbitration. The marker is specified by no document the project has — measured against the mock and the UX spec. |
| 2026-08-19 | Arbitrated by Guy: the dataset ships with ONE witness screen, which forces a third nature (`Empty`). |
| 2026-08-19 | Validated by two fresh-context layers: 30 assertions checked, 2 refuted (both mine, one a premise of the arbitration); the gap-hunt BUILT the mechanism and found that my own M7 does not guard what I claimed. |
| 2026-08-19 | Implemented. 611 → 613 tests, eight gates green. Nine mutations run, nine measured, two diverging from prediction. Two defects found by LOOKING that no test could reach. |
| 2026-08-19 | **Code-reviewed (three layers, Sonnet 5 — a different model from the implementer) and REPAIRED.** 3 decisions taken by Guy, 10 patches, 1 deferral, 4 dismissed with the check that dismissed them. 611 → **614 tests**, eight gates green, fmt and clippy clean, measured against a live `mariadb:10.11.11` (bin suite 5.80 s against 0.06 s without). Four review mutations, four measured. 🔴 Three of the story's OWN recorded measurements were refuted by re-execution (M3's carrier, M4's count, M7's aftermath) — the table had been believed rather than replayed, in the story whose Dev Notes warn that its driver lied twice. 🔴 One finding was reachable by NO layer alone: the CSS the witness screen depends on. |
