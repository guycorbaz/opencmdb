# Story 6b.5: The dashboard — the real reach section beside labelled example sections

Status: done

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

**Story 6b.3's AC2 said this in advance** — *"a screen-level-only marker would either lie about the
real half or hide the example half"* — and this is the story where the sentence stops being a
prediction.

⚠️ **The first draft went on to name a mechanism, and the validation MEASURED that it is not what
fires.** It declared the dashboard all three ways and ran the suite: under `Example(_)` the test reds
on the **witness-count** assertion (`left: 2, right: 1`), under `Fed` on a **probe-count premise**
(*"at least the nine pool-free screens were probed (8)"*) or, with a database, on an **OK-status**
check — and under `Empty`, **the suite stays GREEN**, because content is dispatched from `nature()`
itself, so the *"real on one section, Empty on the rest"* case cannot even be constructed until §0a is
resolved. 🔑 *The structural conclusion is sound and independently reproducible; the sentence naming
which assertion notices was not.* **A red that lands on a different guard than the one you named is
the defect this project has caught four times** — including inside the very paragraph warning about
it.

🔴 **AND THE POOL QUESTION — where this section first said the dashboard *"MUST"* leave the pool-free
router. The validation REFUTED that word by building the alternative, and then measured why the
conclusion survives anyway.**

`screens::router` excludes only `Nature::Fed` (`screens.rs:262`) and returns a `Router<()>` whose doc
says *"change it to `Router<MySqlPool>` and the whole guard evaporates"* — the phrase is `screens.rs`'s
own module doc, attributed there to story 6b.2's validation. The reach section reads
`count_engine_reach` (`repo.rs:1325`, which takes an `Executor`), so the screen needs the pool.

✅ **A FOURTH SHAPE EXISTS AND WAS BUILT**: keep `/dashboard` on the pool-free router and fetch the
reach section as an **htmx fragment from a separate pool-bearing route** — the idiom
`_gap_card.html`'s refresh button already uses. The validation built it: it compiles, `/dashboard`'s
served HTML never touches the pool, `GET /dashboard-reach` returns the real section, and a headless
browser shows the placeholder swapped for *"Sightings placed 2 · not placed 0"*. 🔑 **And the register
had already pointed at it** — `deferred-work.md:3991` hands `/gap`'s orphaned fragment consumer to
**this story**, in one of the very rows §0c counts. *The option was sitting inside a row this section
cites.*

🔴 **But it costs something measured, and the cost is what decides.** The route-table partition
asserts on **one synchronous HTTP body**. Under the fragment design the real counts arrive in a
**second** request that the test's `oneshot` client cannot drive, so verifying AC1 needs either two
coordinated assertions or a real headless-browser test — a capability this project first used
yesterday and **which is not in CI**. The validation tried and **found no way to keep both the
pool-free router and the single-response test shape**.

⚠️ **So the conclusion stands and the REASON in the first draft was wrong.** It is not a compiler
necessity; it is a trade of compile-time safety against the testability of the composed page. *A
decision explained by a false premise is one nobody can re-derive*, which is why the premise is
replaced rather than quietly kept.

**→ ✅ ARBITRATED (Guy, 2026-08-19): shape (a) — a fourth nature, `Nature::Mixed`, on the
POOL-BEARING router.**

🔑 **Taken on the validation's measurement, not on a preference**: `Nature::Mixed` produces exactly
**three `E0004` sites** — one in `demonstration_screen`'s body dispatch and two in the partition test
— so the compiler forces both the marker decision AND the pending-badge decision, and nothing
silently defaults. It is also the codebase's own idiom, `ExampleContent` carrying its payload, closed
in the type on story 5.6's precedent.

🔴 **The fragment shape is REFUSED, and refused on the cost that was measured rather than on
taste.** It works — the validation built it, served it and screenshotted it — but the route-table
partition asserts on **one synchronous HTTP body**, and a fragment's counts arrive in a second
request a `oneshot` client cannot drive. Verifying AC1 would then need two coordinated assertions or
a headless-browser test **which is not in CI**. *The single-response shape is what keeps the guard
that story 6b.3 built, and a guard that has to grow a browser to keep working is a guard about to
stop working.*

⚠️ **What this costs, written rather than implied**: `/dashboard` leaves the `Router<()>`, so for that
one screen the compile-time refusal of `State<MySqlPool>` **stops holding** — the guard survives for
the eight screens that remain pool-free and is **narrowed in writing** for this one (story 5.12's
precedent). ⚠️ And **constraint 1 keeps its force where it matters**: no example data is written, and
the example sections still read nothing.

---

**The four shapes as they were put, kept with what each costs:**

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

⚠️ **SEVEN rows, not eight, and the correction is the lesson.** `grep -c "6b\.5" deferred-work.md`
returns **8** — and that counts matching **physical lines**, while the sentence claimed **register
rows**. One row wraps and names 6b.5 twice. 🔑 *The command was right and the unit was not* — story
5.8's `checked > 0` against `checked == 21`, one story family later. The load-bearing rows:

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
draws does not exist**. Both were measured by the validation.

✅ **The first is TRUE and now established rather than assumed**: `grep -rn "session\|cookie\|last_visit"`
over `crates/` returns nothing outside this story's own file. There is no per-operator state anywhere.

🔴 **The second is OVERSTATED, and its spirit is right for a reason the AC does not give.** **Four**
temporal columns are persisted, not the two the first draft named — `observation_record.observed_at`,
`identity_link.valid_from`/`valid_to`, `declared_attribute.updated_at`, and
`interface.first_seen_at`/`last_seen_at` — and nothing in the production path purges superseded rows.
So the **raw material exists today, with no migration needed**.

🔑 **But a sparkline drawn naively over it would draw the UX spec's BANNED GROWING COUNTER.** Stories
5.14/5.14b measured that **nothing supersedes an abstention across scans**: each scan mints a fresh
`obs_id`, so the raw row count rises with every scan whatever the network did. *A curve that climbs
because the product looked many times is the radar's range, not the operator's debt* — arbitration
13's own sentence, applied to a shape rather than a number. **What does not exist is a MEANINGFUL,
deduplicated history**, and building one is Epic 6's grouping work.

⚠️ **So the AC is right in substance and wrong in its stated reason.** Register the divergence; do not
repeat the flat absence, and do not build the naive sparkline over real rows to prove the point.

### §0f. ⚠️ E17's CLAUSE MAY NOT BE NARROWED HERE, and the AC says so itself

AC3 states that `epics.md` E17's *"rich stat-card/sparkline/trend analytics"* clause is too broad and
**must be narrowed by a retrospective or a correct-course, never by a story**. ⚠️ So this story
**registers** the narrowing it would want and does not perform it. Reading that sentence as
permission to tidy the epic would be the one thing it explicitly forbids.

### §0g. 🔴 THE EXISTING PARTITION CANNOT CATCH A PER-SECTION REGRESSION, and a sibling guard does not exist

Measured by building a two-example-section body and dropping the marker from **one** of them:

- the **screen-level** oracle — `body.contains("example-marker-badge")`, which is what the route-table
  partition asserts — **stays GREEN**, because the other section still carries the string;
- a **section-level** oracle — *count the sections, count the markers, compare* — reds `left: 2,
  right: 1`.

🔑 **So AC2 is not covered by any guard that exists.** The route-table partition cannot be extended
to it — it is a property of the route table and this is a property inside one body — so a **sibling**
guard is a deliverable of this story, not a refinement of an existing one. ⚠️ It was named in §0a's
prose and absent from the task list until now.

### §0h. 🔴 SEEN IN A BROWSER: the marker's SCOPE is ambiguous on a mixed screen

The validation assembled the prototype, rendered it against a live database and looked. The real
Identity block (*"Sightings placed 2 · not placed 0"*) sits **directly above** a bordered `EXAMPLE`
box with **no heading and no divider between them** — and at a glance the marker reads as though it
might be annotating the real numbers above it.

🔑 **The text is accurate about what follows it; the LAYOUT is not.** A bare reuse of
`_example_marker.html` is enough on a wholly-`Example` screen, where the marker unambiguously covers
the whole `<main>`, and **it is not enough on a mixed one**. ⚠️ **Whatever shape ships, each example
section needs its own heading or divider before the marker**, or the real/example boundary reads as
noise rather than as a boundary — which is the one thing this screen exists to draw.

⚠️ **And AC2 does not ask for this**: it speaks to the marker's PRESENCE, never to its legibility
beside real content. Registered as a divergence rather than assumed.

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

- [x] **T0 — Guy's ruling on §0a**: ✅ **shape (a), `Nature::Mixed`, on the pool-bearing router** —
      the fragment shape refused on its measured cost to the single-response test. *(original)*: the nature of a mixed screen, and the pool question — now a
      choice between **three single-response shapes** and a **fourth, fragment-based one the
      validation BUILT** (§0a). ⚠️ **Neither may be decided by a developer.** 🔑 The validation
      recommends **(a) `Nature::Mixed`** on a measurement: it produces exactly **three `E0004`
      sites** — one production, two test — so the compiler forces both the marker AND the
      pending-badge decision and nothing silently defaults
- [x] **T1 — the reach section, unchanged in substance** (AC1): **INCLUDE `_identity_section.html`**,
      the same persisted counts, the unit **sightings**. 🔑 **Including it inherits arbitration 13's
      unit guard for free** (measured — `page.rs:1933` asserts the *"counts sightings, not devices"*
      note renders) ⚠️ **conditional on including the partial rather than hand-rolling the markup**;
      duplicate it and that guard goes blind
- [x] **T2 — the anti-sum guard AT THE COMPOSITION** (AC1, §0d), with the mutation that adds the two
      populations and proves it reds. ⚠️ A guard over the pure builders is measured worthless
- [x] **T3 — the example surfaces carry the marker** (AC2), at SECTION granularity — ⚠️ **and write
      the SIBLING guard that checks it** (§0g): the route-table partition is screen-level and was
      MEASURED green when one section of two lost its marker. It cannot be extended; a new shape is
      a deliverable
- [x] **T3b — a heading or divider before each example section** (§0h): seen in a browser, a bare
      marker under the real block reads as though it annotates the numbers ABOVE it. ⚠️ AC2 asks for
      the marker's presence, never its legibility beside real content — registered
- [x] **T3c — use STATIC class literals for the real and example halves**, or widen the guard first.
      ⚠️ **Measured**: `class="dashboard-section {% if … %}is-real{% else %}is-example{% endif %}"`
      with none of the three defined in `app.css` leaves
      `every_class_a_template_names_is_defined_in_the_stylesheet` **GREEN** — the brace-skipping hole,
      confirmed with the exact pattern a mixed screen invites
- [x] **T4 — the last observation** (§0c), in the BODY and never in the frame, with the divergence
      from the mock registered
- [x] **T5 — verify AC2's two absences** (§0e) rather than repeating them; register any overstatement
- [x] **T6 — LOOK at the screen in a BROWSER**, `OPENCMDB_LOCALE=fr`, against a live database. ⚠️
      **Rebuild the binary first** — `cargo test` does not
- [x] **T7 — the register, BOTH directions** (§0c). ⚠️ A name-grep is provably insufficient, and
      `6.4` / `6b.4` / `6b.5` are different stories
- [x] **T8 — prove-to-red**, predictions FIRST, every row executed

### Review Findings — three layers, 2026-08-19

Blind Hunter (diff only), Edge Case Hunter (own worktree, live `mariadb:10.11.11`, Chrome 151),
Acceptance Auditor (own worktree, the spec, the twins, its own renders). All three on Sonnet 5.
**0 decision-needed, 12 patch, 2 defer, 3 dismissed with the check that dismissed them.**

🔴 **THE COMMIT SHIPS A RED SUITE**, and the claim that it does not was never measured on this tree.
🔴 **AND THE STORY'S OWN DEFECT CLASS RECURRED A FOURTH TIME — inside the guard written to close it.**

🔑 **All twelve applied.** ⚠️ The suite is now green **both ways** — `407 passed` at **0.07 s**
without a database and **10.56 s** with one — and *that pair of figures is the one the Completion
Notes claimed and never measured*.

#### Patch

- [x] [Review][Patch] 🔴 **`cargo test --workspace` FAILS without a database, and the Completion
      Notes claim otherwise.** `probed >= 9` (`main.rs:966`) is a hardcoded floor; making the
      dashboard `Mixed` added a SECOND nature the no-database branch skips, so the count is **8**.
      Reproduced: `405 passed; 1 failed`. ⚠️ **The edit that broke it is three lines above the
      assertion, in this story's own diff** — and *"0.05 s without a database"* was carried over from
      the previous session's habit, **never run on this tree**. 🔑 **CI would not have caught it**: CI
      supplies a database, so CI is green and only the local path reds. Fix the floor by DERIVING it
      from `Screen::ALL` and the natures, so it cannot go stale again
- [x] [Review][Patch] 🔴 **A FOURTH occurrence of the dominant defect class, in the sibling guard
      written to close the third.** `every_example_section_carries_its_own_marker` compares
      `markers == sections` **as totals**. Measured: two markers in the first example section and
      **zero in the second** — net 2 and 2 — leaves the **entire suite green**, including that guard.
      *It cannot tell "each section has exactly one" from "they happen to add up."* Count per section
- [x] [Review][Patch] 🔴 **A reachable contradiction on the dashboard's own real half, guarded by
      nothing.** An observation with no `identity_link` — the ordinary state between an ingest and the
      identity pass — renders *"Rien d'observé pour l'instant — lancez un scan"* **directly above**
      *"Dernière observation il y a 8 h"*, inside one `dashboard-real` div. 🔑 **This story is what
      co-located those two populations for the first time**, and every test feeds them consistently
      from one fixture, so no render-level test can see it. Seed them independently and reconcile the
      copy
- [x] [Review][Patch] 🔴 **`Nature`'s own doc is false, six lines above the variant this story
      added**: *"Three variants and not two"* — there are **four** — and *"the eight screens whose own
      story has not landed"* — there are **seven**
- [x] [Review][Patch] 🔴 **Six sites say *"nine demonstration screens"* and there are eight**:
      `screens.rs:1`, `:8`, `:282`, `main.rs:420`, `page.rs:1315`, `:1462`. A count sweep this story
      owed and skipped — the seventh site is the test floor above, which is why this one broke a test
      rather than only a sentence
- [x] [Review][Patch] 🔴 **M3's recorded mechanism is wrong, and the truth names a real hazard.** The
      row says *"3+ red … the route disappears"*; measured, **26 red, every one
      `Overlapping method route. Handler for GET /dashboard already exists`** — axum panics at
      construction because `screens::router` stops excluding the screen while `triage_router`'s
      **hand-written** `/dashboard` route still stands. 🔑 *The route is registered by hand and the
      exclusion by `nature()`; nothing ties them.* Record the mechanism and the coupling
- [x] [Review][Patch] 🔴 **`_dashboard.html`'s own comment says *"STATIC class literals throughout"*
      and line 46 renders `class="spark-bar spark-h{{ height }}"`** — the exact brace-containing shape
      the comment warns about, invisible to the stylesheet guard by its own admission. **And
      `.spark-h8` is missing** (defined: 1-7 and 9). No card uses 8 today, so nothing breaks — *an
      authoring bug behind a guard that cannot see it*
- [x] [Review][Patch] **The wrong outcome token recurs sixty lines below the comment explaining it.**
      `page.rs:3668` uses `reach("matched", …)` where the engine's token is `"match"` — harmless
      there (the test reads only the freshness) but it shows the fix was **patched at the one site the
      mutation hit**, not closed
- [x] [Review][Patch] **A dead `.clone()` in the handler** (`page.rs:1437`): `identity` is never read
      again, and the `Clone` derives were added for the duplicated field that no longer exists
- [x] [Review][Patch] **The anti-sum oracle is a bare `contains("17")` over the whole page** —
      fixture-coupled, and blind to a sum split across markup. Scope it to the two `dashboard-*`
      sections, as its sibling assertions already are
- [x] [Review][Patch] ⚠️ **§0b promised to register constraint 1's reading and did not.** Seven rows
      were added to the register and none carries it. The argument is judged sound by the Auditor —
      the clause's subject is example data being WRITTEN, and both queries are `SELECT`s — **the gap
      is the paper trail, which is the thing the section itself asked for**
- [x] [Review][Patch] **M6's row omits a compile-carried half**: `chrono::Utc::now()` inside the
      builder is `E0599`, and only `SystemTime::now()` produces the runtime red. Record both, as
      story 6b.4 did

#### Defer

- [x] [Review][Defer] 🔴 **The example half is visually DOMINANT over the honest one, which inverts
      the user story's own purpose.** The fabricated cards render at **22 px mono with a sparkline**;
      the real counts render in a body-size pill. *"37 / 4 / 2"*, invented, are the loudest thing on
      the screen; *"2 sightings placed"*, real, is the quietest. ⚠️ **No AC covers relative visual
      weight** — they cover presence and marking — and **no guard can**: they assert text, never
      salience. 🔑 *This is exactly the risk the story exists to prevent* (*"so that the honest part
      is not diluted by the demonstration around it"*), reached without violating a single criterion.
      It needs a design pass and a criterion, not a patch. **Owner: Epic 6b's retrospective** for the
      criterion and **story 6b.12's release sweep** for the pass — deferred
- [x] [Review][Defer] ⚠️ **The handler's two reads are not transactional together**, so a resolver
      pass writing between them shows a reach snapshot and an instant from two moments. Display-only,
      self-correcting on refresh, and the same shape `reconcile_view` has carried since 5.14b.
      **Owner: the first story that needs a consistent read across both** — deferred, pre-existing

---

## Prove-to-red — deliberately short

| # | Mutation | Prediction |
|---|---|---|
| M1 | sum the two populations at the composition | ✅ **BOTH SHAPES BUILT AND MEASURED by the validation**: the guard over the pure builder alone stays **GREEN** under a sum planted three calls away; the same guard written AT the composition catches it (`left: 2, right: 3`). 4/4. *5.14b's lesson reproduces exactly* |
| M2 | drop the marker from one example section | ✅ **SETTLED BY THE VALIDATION, exactly as predicted**: the screen-level oracle stays **GREEN** (the other section still carries the string), a section-level one reds `left: 2, right: 1`. The sibling guard of §0g is what this row measures |
| M3 | declare the dashboard wholly `Example` | ⚠️ **UNMEASURED by the validation, with the command to settle it**: declare `Example(ExampleContent::DashboardStub)` whose content embeds the real fragment's `hx-get`, then run the partition test. By inspection **nothing scans inside an `Example` body for a live pool-backed reference**, so it would likely stay green — which would itself be the finding |
| M4 | render the last observation in `_nav.html` | ✅ **CONFIRMED** — reds at `screens.rs:658`. 🔑 **And the converse was measured too**: the same text in the screen's BODY leaves the suite green and **cannot trip the guard by accident**, because the body arrives as `{{ body\|safe }}` and the guard's self-widening loop follows only `{% include %}` directives inside the two frame files |
| M5 | swap the unit from *sightings* to *devices* | ✅ **The guard already exists** (`page.rs:1933`) and is inherited **for free** — ⚠️ **conditional on including `_identity_section.html`** rather than duplicating its markup. Hand-roll the reach markup and this row measures nothing |
| M6 | read the clock inside the new pure builder | ⚠️ **UNMEASURED — no builder exists yet.** The idiom to copy is `build_triage_reads_no_clock_of_its_own` (`page.rs:2849`, `now` as a parameter). ⚠️ AC2/AC3 keep the sparkline on static example data, so no live clock read is implied — **if the implementation needs none, say so and drop the row rather than writing a guard over nothing** |

## References

- `_bmad-output/planning-artifacts/epics.md:2198-2206` — the acceptance criteria, verbatim
- `_bmad-output/planning-artifacts/epics.md:2096` — constraint 1, whose last clause §0b examines
- `_bmad-output/implementation-artifacts/deferred-work.md` — the **eight** rows naming 6b.5, notably `:3692` (the `/` redirect) and `:3724` (the last observation)
- `_bmad-output/implementation-artifacts/6b-3-example-data-marker-and-its-gate.md` — AC2's mixed specimen, owed here
- `_bmad-output/implementation-artifacts/6b-4b-action-bar-and-the-gesture-nature.md` — assert on the render, and grep the artefact you believe
- `crates/opencmdb-bin/src/screens.rs`, `src/page.rs`, `templates/_identity_section.html`

## Validation record — two fresh-context layers, 2026-08-19

**Mandatory here** (Guy, Epic 4 retrospective). Both on a different model, each in its own worktree.

**Layer 1, fact-check — 27 assertions verified, 25 confirmed, 1 REFUTED, 1 true-but-weaker.**
🔴 The refutation is a UNIT error of mine: *"eight register rows"* is **eight matching LINES from
SEVEN rows** — one wraps and names 6b.5 twice. The command was right; the unit was not. ⚠️ And §0a's
described mechanism is weaker than written: the layer declared the dashboard **all three ways and ran
the suite**, and every red lands on a different guard than the prose names — a witness count, a probe
premise, an OK status — while the `Empty` case **does not red at all**. ✅ It also refuted **its own**
suspicion that a quotation was fabricated: *"forbidding it by discipline is worth nothing"* is
genuine, in `screens.rs`'s module doc rather than in story 6b.2's markdown.

**Layer 2, gap-hunt — it BUILT the alternative and refuted the story's central word.**
🔴 *"It MUST leave the pool-free router"* is **false as an absolute**: a pool-free `/dashboard` whose
reach section arrives by `hx-get` from a separate pool-bearing route was built, served and
screenshotted working. 🔑 **And the register had already pointed at it** — `/gap`'s orphaned fragment
consumer is handed to this story in one of the rows §0c counts. ⚠️ **But the cost was measured and it
is what decides**: the route-table partition asserts on ONE synchronous body, and the fragment's
counts arrive in a second request a `oneshot` client cannot drive, so AC1 would need two coordinated
assertions or a headless-browser test **that is not in CI**. The layer found **no way to keep both**.
*So the conclusion survives and the reason given for it did not — and a decision explained by a false
premise is one nobody can re-derive.*

✅ **Shape (a) recommended on a measurement**: `Nature::Mixed` produces exactly **three `E0004`
sites** — one production, two test — forcing both the marker and the pending-badge decision, with
nothing silently defaulting; the suite is green after patching all three.

✅ **§0d confirmed by building**: the guard over the pure builder alone stays green under a planted
sum, the same guard at the composition catches it. 5.14b's lesson reproduces exactly.

🔴 **And the two findings only a browser and a build could give**: the marker's SCOPE is ambiguous
beside real content (§0h), and the CSS guard's brace-skipping hole was confirmed **with the exact
conditional class a mixed screen invites** (§0g/T3c).

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context), 2026-08-19. Built and mutated against a live `mariadb:10.11.11` on port
13430, and rendered in Google Chrome 151.

### Debug Log References — the mutation pass, and M1's THREE false greens

🔑 **Six rows prescribed, seven run — and M1 took FOUR attempts to red, for THREE different
reasons.** Each false green was a distinct way for a guard to be worthless, and none was visible by
reading.

| # | Mutation | MEASURED | Carrier |
|---|---|---|---|
| M1 | sum the two populations at the composition | 🔴 **green ×3, then red.** (1) The driver had **no assert**, so the pattern never matched and nothing was mutated — *the mutation-driver-lies class, mine*. (2) The fixture used `"matched"` where the engine's token is **`"match"`**, so `placed` rendered as **0**, and the oracle's `>2<` was satisfied by an **example stat card**. (3) The test helper built `DashboardBody` from the **un-composed** identity while the handler builds it from the view — *the guard rendered a shape production does not use*. Fixed at the root: the duplicated field is **gone**, the template reads `view.identity`, and the fixture's 11/6 collide with no card | named assertion |
| M1b | the same sum, guarded over the pure builder alone | ✅ **GREEN by design** — the shape story 5.14b shipped, reproduced. *Neither builder can add what it never sees* | — (control) |
| M2 | drop the marker from one example section of two | ✅ **1 red**, the new sibling guard. ⚠️ The route-table partition stays green, exactly as the validation measured | named assertion |
| M3 | declare the dashboard wholly `Example` | 🔴 **THE RECORDED MECHANISM WAS WRONG, and the truth names a real coupling.** Not *"3+ red, the route disappears"* — measured at the code review: **26 red, every one `panicked … Overlapping method route. Handler for GET /dashboard already exists`** at router construction. `screens::router` stops excluding the screen while `triage_router`'s **hand-written** `/dashboard` route still stands, so axum sees two handlers for one path. ⚠️ *The route is registered by hand and the exclusion by `nature()`, and nothing ties them* — the Blind Hunter suspected it from the diff alone and the Edge Case Hunter measured it | panic at router construction |
| M4 | the last observation planted in `_nav.html` | ✅ **1 red**, `the_shell_shows_no_last_observation` | named assertion |
| M5 | the unit *sightings* → *devices* | ✅ **2 red** — the inheritance from `_identity_section.html` is real | named assertions |
| M6 | a clock read inside `build_dashboard` | ✅ **2 red** with `SystemTime::now()`, the new clock guard first. ⚠️ **And the row omitted a compile-carried half**, added at the review: `chrono::Utc::now()` is `E0599` here (`default-features = false`), so only the `std` spelling produces a runtime red — *a feature flag guards a name, never the act of reading a clock*, story 6b.4's sentence, and this row should have carried it | named assertion; compiler for the `chrono` spelling |

🔴 **AND THE PASS DESTROYED WORK, which is recorded because it cost a full debugging cycle.** The
mutation script restored some files from a scratchpad copy and others with `git checkout --`.
`locales/app.yml` was in the second set and **not** in the snapshot, so restoring it after M5 reverted
it to the last COMMIT and **silently deleted this story's nine new keys**. The symptom was
`dash.last_observed` rendering as its own name, and the guard that should have caught it — 
`every_key_carries_both_locales` — reads the **file**, while `t!()` reads the **embedded copy**:
source against artefact, one more time. 🔑 *Story 6.1's incident, reproduced here for the third time
in this project: **a file revert equals a mutation revert only on a committed baseline**, and mixing
two restore mechanisms in one script guarantees the boundary is crossed.*

⚠️ **And one hypothesis of mine was refuted before it could be written down as a cause.** The failure
looked like the flake of issue #38, so the suite was run three times in parallel and three times
**single-threaded**: 1 failure in all six. Deterministic, not a race — and the `set_locale` story I
had ready was wrong.

### Completion Notes List

**AC by AC:**

- **AC1 — MET.** The reach section is **included**, not re-drawn, so it is unchanged in substance and
  arbitration 13's unit guard is inherited rather than restated (M5 proves the inheritance real).
  🔴 The anti-sum guard sits **at the composition**, and M1b measures why: the same guard over the
  pure builder alone stays green under the same mutation.
- **AC2 — MET.** Each example section carries its own heading and its own marker, and a **sibling
  guard** counts sections against markers — the route-table partition cannot, measured.
- **AC3 — MET as scoped.** The sparkline renders over example heights, as bars. ⚠️ **E17's clause is
  NOT narrowed here**, by its own words; the wording is registered for a retrospective.

🔑 **`Nature::Mixed` behaved exactly as the validation measured**: adding it produced three `E0004`
sites and nothing defaulted silently.

⚠️ **Seen in a browser** (Chrome 151, French, live database): heading → identity → *"Dernière
observation il y a 7 min"* → rule → **EN UN COUP D'ŒIL** with its marker and three cards → rule →
*"ce qui a grandi"* with its own marker. **The real/example boundary is legible**, which is §0h's
finding addressed rather than asserted. ⚠️ **An honest limit**: the reach section rendered its EMPTY
state, because the seeded link did not reach `count_engine_reach`'s filter — the populated case is
covered by unit tests and **was not seen by eye**.

**629 → 634 tests** (407 bin + 161 core + 66 xtask) after the code review. Eight gates green, fmt and
clippy clean, run both ways: **0.07 s** without a database and **10.56 s** against a live
`mariadb:10.11.11`.

🔴 **This paragraph claimed *"0.05 s without a database"* and the suite was RED there.** The figure
was carried over from the previous session's habit and **never run on this tree**: `probed >= 9` was
a hardcoded floor, making the dashboard `Mixed` added a second nature the no-database branch skips,
and the count fell to 8 — with the breaking edit three lines above the assertion, in this story's own
diff. ⚠️ **CI supplies a database, so CI was green and only the local path reddened**: a floor CI
cannot check is a floor nobody re-reads. The floor is now DERIVED from `Screen::ALL`.

### File List

| File | Change |
|---|---|
| `crates/opencmdb-bin/src/screens.rs` | **`Nature::Mixed`**, `Screen::Dashboard` declared with it, and `router` excluding it beside `Fed` |
| `crates/opencmdb-bin/src/page.rs` | `DashboardView`, `StatCardView`, `DashboardBody`, `build_dashboard`, `example_cards`, the `dashboard` handler, the `/dashboard` route, and four guards |
| `crates/opencmdb-bin/src/repo.rs` | `last_observed_at` — the `MAX(observed_at)` the register handed this story |
| `crates/opencmdb-bin/src/main.rs` | the partition's `Mixed` arms, forced by `E0004` |
| `crates/opencmdb-bin/templates/_dashboard.html` | **new** — the real half, then two example sections each with a heading and a marker |
| `crates/opencmdb-bin/locales/app.yml` | 9 keys, both locales |
| `crates/opencmdb-bin/assets/app.css` | the dashboard's rules, the stat cards and the bar sparkline |

### Change Log

| Date | Change |
|---|---|
| 2026-08-19 | **Code-reviewed (three layers, Sonnet 5) and REPAIRED.** 0 decisions, 12 patches, 2 deferrals, 3 dismissed. 🔴 **The commit shipped a RED SUITE without a database** — a hardcoded `probed >= 9` floor this story's own edit invalidated three lines above it — while the Completion Notes claimed 0.05 s green there, a figure never run on this tree. CI could not catch it: CI has a database. 🔴 **And the dominant defect class recurred a FOURTH time, inside the guard written to close the third**: the section-marker guard compared totals, so two markers here and none there left the whole suite green. 🔴 A reachable contradiction this story created by co-locating two populations — *"nothing observed yet"* above a real instant — invisible to every test because they all fed both from one fixture. Plus six false *"nine screens"* sites, a doc saying *"three variants"* for four, a comment falsified by line 46 of its own file, and M3's mechanism wrong (26 red, `Overlapping method route`). ⚠️ Deferred: **the example half is visually dominant over the honest one** — the story's own stated risk, reached without violating a criterion. |
| 2026-08-19 | Implemented, shape (a). 629 → **633 tests**, eight gates green. 🔴 **M1 took FOUR attempts to red, for THREE different reasons** — a driver without an assert, a fixture whose wrong token made `placed` render 0 while an example card satisfied the oracle, and a test helper that built the body differently from the handler. Each is a distinct way for a guard to be worthless and none was visible by reading. 🔴 **And the pass destroyed work**: mixing a scratchpad restore with `git checkout --` reverted `app.yml` to its last commit and deleted nine uncommitted keys — story 6.1's incident, third occurrence. ⚠️ One flake hypothesis of mine was refuted by six runs before it could be written down as a cause. |
| 2026-08-19 | Validated by two fresh-context layers. Fact-check: 27 assertions, 25 confirmed, 🔴 **1 refuted — *"eight register rows"* is eight LINES from seven rows**, the right command on the wrong unit — and §0a's described mechanism measured to fire on other guards than it names. Gap-hunt: it **BUILT the fragment alternative** and refuted *"must leave the pool-free router"* as an absolute, then **measured the cost that keeps the conclusion** — the partition asserts on one synchronous body and the fragment's counts arrive in a second request. 🔴 Plus: the existing partition **cannot** catch a per-section regression (measured green), the marker's scope is **ambiguous beside real content** (seen in a browser), and the CSS brace hole is confirmed with the exact pattern this screen invites. |
| 2026-08-19 | Contexted. 🔴 ONE structural question, and it touches story 6b.2's central guard: a MIXED screen has no nature, the route-table partition reds on it under all three, and the reach section needs the POOL — so `/dashboard` must leave the `Router<()>` whose type refusal was the guard. Three shapes put to Guy. Plus: constraint 1's *"no demo screen opens a connection"* meets a screen it did not anticipate; the last observation is banned from the frame while the mock puts it there; and the anti-sum guard must sit at the composition, which is where 5.14b's measured green. |
