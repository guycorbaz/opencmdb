# Story 6b.5: The dashboard — the real reach section beside labelled example sections

Status: ready-for-dev

Epic: 6b — *L'interface de la maquette*. **Fifth numbered slot**, after 6b.4b closed the triage
screen's action bar. It is the story the epic has been pointing at since its first arbitration:
**the one screen that is MIXED by construction**, where the product's honest half sits beside a
demonstration and neither may be mistaken for the other.

## Story

As the operator,
I want the daily screen to carry what the product really knows,
so that the honest part is not diluted by the demonstration around it.

## Acceptance Criteria

Transcribed from `epics.md:2198-2206`, **unmodified** — divergences are raised in §0 below rather
than edited in (a story may not edit an AC; only a retrospective may).

1. **Given** the reach section story 5.14b shipped, **when** the dashboard renders, **then** it
   appears **unchanged in substance**: the same persisted counts, the same honest unit (**sightings**,
   arbitration 13), and **the two populations are never summed** (arbitration 10).
2. **Given** the stat cards, the sparklines and *"what grew since your last visit"*, **when** they
   render, **then** they are example surfaces and carry the marker — **the product has no notion of a
   last visit**, and the history a sparkline draws does not exist yet.
3. **And** ⚠️ the sparkline itself is **MVP by the arbitration of 2026-08-13** (the UX spec holds;
   `epics.md` E17's *"rich stat-card/sparkline/trend analytics"* clause is too broad and **must be
   narrowed by a retrospective or a correct-course, never by a story**). Here it renders over example
   data; its fed version waits for the history that feeds it.

---

## §0 — What contexting found

### §0a. 🔴 A MIXED SCREEN HAS NO NATURE, AND THE PARTITION REDS ON IT WHATEVER IT IS DECLARED

This is the story's structural question and it must be answered before any code.

`Screen::nature()` returns one of three (`screens.rs:222-239`), and **every one of them is a property
of the WHOLE screen**. The route-table partition story 6b.3 built asserts, verbatim
(`main.rs:920-931`):

```rust
Nature::Example(_) => assert!(carries,  "… shows the example dataset and must say so"),
Nature::Fed | Nature::Empty => assert!(!carries, "… carries the marker and must not"),
```

🔴 **A dashboard that is real on one section and example on the others reds this test under every
existing nature**: declared `Fed` or `Empty`, it carries a marker and must not; declared
`Example(_)`, it says the reach section is a demonstration, which is false. **Story 6b.3's AC2 said
this in advance** — *"a screen-level-only marker would either lie about the real half or hide the
example half"* — and this is the story where the sentence stops being a prediction.

🔴 **AND IT MUST LEAVE THE POOL-FREE ROUTER, which dissolves story 6b.2's structural guard for this
screen.** `screens::router` excludes only `Nature::Fed` (`screens.rs:262-270`) and returns a
`Router<()>` whose doc says *"change it to `Router<MySqlPool>` and the whole guard evaporates"*. The
reach section reads `count_engine_reach` — it **needs the pool**. So `/dashboard` moves to the main
router, and for that screen the compile-time refusal of `State<MySqlPool>` — which story 6b.2 chose
precisely because *"forbidding it by discipline is worth nothing"* — **stops holding**.

**→ PUT TO GUY. Three shapes, with what each costs:**

- **(a) A fourth nature, `Mixed`.** The partition gains an arm — a compiler-forced one, `E0004` — and
  the assertion for `Mixed` becomes *carries the marker AND is not wholly example*. ⚠️ It is a
  screen-level answer to a section-level fact, so the partition still cannot say WHICH section is
  which; it only stops lying.
- **(b) Natures per SECTION, the screen keeping none.** Truest to 6b.3's AC2 (*"the smallest unit that
  carries example content"*) and the biggest change: the partition stops being a route-table property,
  which is exactly what AC4 of 6b.3 demanded it be.
- **(c) `Fed`, with the marker asserted per section instead of per screen.** Smallest diff; ⚠️ it makes
  `Fed` mean *"reads the store"* rather than *"owes no marker"*, which is **not** what its doc says —
  and a type whose doc goes false is this project's most-caught defect.

⚠️ **Whichever is chosen, the pool question is separate and also Guy's**: does `/dashboard` move to
the main router (and the guard is narrowed in writing, story 5.12's precedent), or does the reach
section arrive by some other route?

### §0b. ⚠️ CONSTRAINT 1'S LAST CLAUSE MEETS A SCREEN IT DID NOT ANTICIPATE

Epic 6b's constraint 1 (`epics.md:2096`) ends: *"The example dataset lives in code, in the
handler/template layer, and **no demo screen opens a connection**."* Its subject is that **example
data must never be WRITTEN to the database** — a mixed screen reading its own real half is not that.
🔑 **And the register already half-says so**: the last-observation row (`deferred-work.md:3724`) calls
that figure *"the only one of the two that touches epic constraint 1"* and hands it to this story.

⚠️ So the clause is not violated in substance, and it **is** violated in letter if *"demo screen"*
includes a screen with demo sections. **Register the reading; do not quietly pick one.**

### §0c. ✅ EIGHT REGISTER ROWS NAME THIS STORY — read them before starting, not after

Measured: `grep -c "6b\.5" deferred-work.md` → **8**. The load-bearing ones:

- 🔴 **The last observation is this story's** (`:3724`) — a `MAX(observed_at)`, re-arbitrated by Guy on
  2026-08-18 as *"the perimeter ships and the last observation waits"*. ⚠️ **And it is BANNED from the
  frame**: `screens.rs`'s `the_shell_shows_no_last_observation` scans `_shell.html` and `_nav.html`
  for *"observ"* and reds. **The mock puts it in the nav footer**; this story may not. It goes in the
  dashboard's body, and **that is a divergence from the mock that must be registered**, since 6b.2's
  governing rule is *"the mock prevails"*.
- **The `/` redirect target** (`:3692`) — *"to be re-examined when the dashboard stops being mixed"*,
  owner this story, and 🔑 *"cheap because the redirect is separate from the screen: its target is one
  line"*. ⚠️ The dashboard does **not** stop being mixed here, so the honest answer is probably *not
  yet* — but the row asks for the re-examination, not for the change.
- **`/gap`'s orphaned fragment** — no page has embedded it since 6b.4 replaced the triage body.
- **AC2's MIXED specimen**, which story 6b.3 recorded as *stated, not met*, and owed here.

### §0d. 🔴 THE ANTI-SUM GUARD MUST BE AT THE COMPOSITION, AND 5.14b LEARNED THAT THE HARD WAY

Arbitration 10 forbids summing the two populations. Story 5.14b wrote a guard for it and **its own
mutation pass measured the guard GREEN**: it asserted that two pure builders do not add each other's
counts, and **neither of them can**, since neither sees the other's numbers. *A guard placed where
the defect cannot occur reads as coverage and is none* — the epic's dominant class, and its cleanest
specimen.

🔑 **The only place a sum can be written is the impure edge that assembles both**, and **this story
creates a second one**. The guard belongs there, and it must be proven to red by a mutation that
actually adds the two.

⚠️ **And arbitration 13's unit is part of AC1**: *sightings*, not devices, on both sides — *a figure
that rises because the product looked many times is the radar's range, not the operator's debt*.

### §0e. ⚠️ TWO THINGS THE AC SAYS DO NOT EXIST — verify before believing either

AC2 asserts that **the product has no notion of a last visit** and that **the history a sparkline
draws does not exist**. The first is almost certainly true (no session, no per-operator state
anywhere). ⚠️ **The second deserves a measurement rather than a repetition**: `observation_record`
carries `observed_at` per row and `identity_link` carries `valid_from`/`valid_to`, so *some* history
is persisted. **Establish precisely what a sparkline would need and whether it exists**, and if part
of it does, say so — an AC that overstates an absence is still an AC, and the divergence is
registered rather than argued.

### §0f. ⚠️ E17's CLAUSE MAY NOT BE NARROWED HERE, and the AC says so itself

AC3 states that `epics.md` E17's *"rich stat-card/sparkline/trend analytics"* clause is too broad and
**must be narrowed by a retrospective or a correct-course, never by a story**. ⚠️ So this story
**registers** the narrowing it would want and does not perform it. Reading that sentence as
permission to tidy the epic would be the one thing it explicitly forbids.

---

## Dev Notes

### What exists today (read, not assumed — `master` at `1843066`)

- **`crates/opencmdb-bin/src/screens.rs`** (**305** code lines) — `Screen`, `Nature` (three variants),
  `router()` returning `Router<()>`, `demonstration_screen`, and the shell guards including
  `the_shell_shows_no_last_observation`.
- **`crates/opencmdb-bin/src/page.rs`** (**1432** code lines — the largest in `crates/`) — `build_identity_view`,
  `count_engine_reach`'s consumer, `TriageView`/`build_triage`, `reconcile_view`, `triage_view`, and
  `now_utc`, **the only clock read in the crate**. ⚠️ Story 6b.4's review measured that
  `the_view_builder_has_no_clock_so_one_store_renders_identically` proves nothing about a POPULATED
  `build_view`; a new builder needs its own clock guard, written not inherited.
- **`templates/_identity_section.html`** — story 5.14b's reach section, extracted to a partial by
  6b.4 precisely so a second page could include it. **This story is that second page.**
- **`templates/_example_marker.html`** — 6b.3's marker, one partial, one key pair.
- **`assets/app.css`** — ⚠️ every class a template names must be defined
  (`every_class_a_template_names_is_defined_in_the_stylesheet`, recursive since 6b.4's review).
  🔴 **And that guard has two known holes** (registered): it scans Askama comments, and it **silently
  skips any `class="…"` containing a brace** — so a conditional class goes unchecked. Use static
  class literals, or widen the guard.
- **`locales/app.yml`** — 98 key pairs, `fr` + `en`, guarded.

### The house rules this story will be judged against

- 🔴 **Assert on the RENDER, not the source.** Story 6b.4b's four HIGH findings were one mistake made
  four times: every guard it wrote scanned a template or a YAML file, and every defect lived in the
  served HTML or the resolved string.
- 🔴 **Grep the artefact you are about to believe.** Three times in one day a measurement landed on
  the wrong artefact — a `<button>` measured for a `<span>`, a screenshot of a stale binary
  (`cargo test` builds the test target, **not** `target/debug/opencmdb`), and a mutation named for one
  thing applied to another.
- 🔴 **A guard placed where the defect cannot occur reads as coverage and is none** — and §0d is this
  epic's cleanest specimen, in this story's own subject.
- **Prove-to-red**, predictions FIRST, **every prescribed row executed**, carriers named per row.
- Doc comments must be TRUE. `#![deny(missing_docs)]` is on. No file over 2000 code lines
  (⚠️ `xtask/src/main.rs` is at **1908** — split, do not grow).

### Testing

- `cargo test --workspace`, `cargo clippy --workspace --locked -- -D warnings`, `cargo fmt --all`,
  `cargo xtask ci` (eight gates; `views-hash` is `ℹ STALE` by design).
- ⚠️ `DATABASE_URL` unset ⇒ DB-backed tests return early and pass in silence (0.05 s vs ~6 s).
  **This story's real half is database-backed. Run both ways and record both figures.**
- ✅ **A BROWSER IS AVAILABLE** — `google-chrome` 151 and `firefox`, discovered at 6b.4b's validation
  after four stories had deferred the visual check on an unmeasured sentence. **T6 is a real browser
  check**, at desktop width; ⚠️ 390 px is knowingly broken (responsive deferred by Guy, 2026-08-18).
- Baseline: **629 tests** (402 bin + 161 core + 66 xtask), eight gates green, `master` at `1843066`.

## Tasks / Subtasks

**Written for §0a option (a). Rescope on Guy's answer before starting.**

- [ ] **T0 — Guy's ruling on §0a**: the nature of a mixed screen, and whether `/dashboard` moves to
      the pool-bearing router. ⚠️ **Neither may be decided by a developer**
- [ ] **T1 — the reach section, unchanged in substance** (AC1): include
      `_identity_section.html`, the same persisted counts, the unit **sightings**
- [ ] **T2 — the anti-sum guard AT THE COMPOSITION** (AC1, §0d), with the mutation that adds the two
      populations and proves it reds. ⚠️ A guard over the pure builders is measured worthless
- [ ] **T3 — the example surfaces carry the marker** (AC2), at SECTION granularity
- [ ] **T4 — the last observation** (§0c), in the BODY and never in the frame, with the divergence
      from the mock registered
- [ ] **T5 — verify AC2's two absences** (§0e) rather than repeating them; register any overstatement
- [ ] **T6 — LOOK at the screen in a BROWSER**, `OPENCMDB_LOCALE=fr`, against a live database. ⚠️
      **Rebuild the binary first** — `cargo test` does not
- [ ] **T7 — the register, BOTH directions** (§0c). ⚠️ A name-grep is provably insufficient, and
      `6.4` / `6b.4` / `6b.5` are different stories
- [ ] **T8 — prove-to-red**, predictions FIRST, every row executed

## Prove-to-red — deliberately short

| # | Mutation | Prediction |
|---|---|---|
| M1 | sum the two populations at the composition | AC1's guard reds. ⚠️ **If a guard over the pure builders is written instead, it stays GREEN** — 5.14b measured exactly that |
| M2 | drop the marker from one example section | the section-level partition reds — ⚠️ **not** the route-table one, which is screen-level and would stay green |
| M3 | declare the dashboard wholly `Example` | the reach section is called a demonstration; predict WHICH assertion notices, and whether any does |
| M4 | render the last observation in `_nav.html` | `the_shell_shows_no_last_observation` reds |
| M5 | swap the unit from *sightings* to *devices* | AC1's unit guard reds — arbitration 13 |
| M6 | read the clock inside the new pure builder | the new clock guard reds. ⚠️ **Write it; 5.14b's proves nothing about a populated builder** |

## References

- `_bmad-output/planning-artifacts/epics.md:2198-2206` — the acceptance criteria, verbatim
- `_bmad-output/planning-artifacts/epics.md:2096` — constraint 1, whose last clause §0b examines
- `_bmad-output/implementation-artifacts/deferred-work.md` — the **eight** rows naming 6b.5, notably `:3692` (the `/` redirect) and `:3724` (the last observation)
- `_bmad-output/implementation-artifacts/6b-3-example-data-marker-and-its-gate.md` — AC2's mixed specimen, owed here
- `_bmad-output/implementation-artifacts/6b-4b-action-bar-and-the-gesture-nature.md` — assert on the render, and grep the artefact you believe
- `crates/opencmdb-bin/src/screens.rs`, `src/page.rs`, `templates/_identity_section.html`

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log

| Date | Change |
|---|---|
| 2026-08-19 | Contexted. 🔴 ONE structural question, and it touches story 6b.2's central guard: a MIXED screen has no nature, the route-table partition reds on it under all three, and the reach section needs the POOL — so `/dashboard` must leave the `Router<()>` whose type refusal was the guard. Three shapes put to Guy. Plus: constraint 1's *"no demo screen opens a connection"* meets a screen it did not anticipate; the last observation is banned from the frame while the mock puts it there; and the anti-sum guard must sit at the composition, which is where 5.14b's measured green. |
