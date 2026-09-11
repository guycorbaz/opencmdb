# Story 14.2: The plan on screen, and in the operator's hands

Status: ready-for-dev

⚠️ **`ready-for-dev` is the workflow's status, not a statement that nothing is open.** §1 poses FOUR
arbitrations that are Guy's, and a dev agent that meets one of them mid-implementation will settle
it silently — which is the defect this section exists to prevent. They are taken before `dev-story`,
and the mandatory validation pass (two fresh-context agents) comes first either way.

Epic 14 (IPAM), decomposed 2026-09-11 in `epics.md` (`4b5db27`). Second story of four.
Baseline: `38da035` (master), **873 tests** (583 bin + 191 core + 99 xtask), ten gates.

## Story

As the operator,
I want to define a range and an address and see them drawn,
so that the product holds my addressing plan instead of an invented one.

## §0 — What is established and must not be re-opened

**The vocabulary is RATIFIED** (PR #166, `b466be7`). The PLAN axis is binding: `static` ·
`dhcp-pool` · `reserved` · `infrastructure`. **No story may extend it.** `structural` is retired as
a displayed word and **this story carries the removal** — `ipam/mod.rs:65-69` and
`ux-design-specification.md` both name 14.2 by name.

**Guy's six arbitrations** (2026-09-10) are `epics.md`'s Epic 14 preamble as six numbered
constraints. This story is downstream of all six and re-decides none. The three that bind hardest
here: **(1)** the write cost is front-loaded, so this story ships TWO routes and the second is
cheap; **(4)** IPAM and `declared_attribute` are two registers and the documenting gesture does not
touch IPAM; **(6)** no state is told apart by colour alone.

**14.1 shipped a domain, a schema and an adapter and NO producer.** `ip_subnet`, `ip_range`,
`ip_address` (`0007_addressing_plan.sql:91`, `:111`, `:145`); `IpPolicy` and `IpamError`
(`ipam/mod.rs:45`, `:128`); `canonical`/`from_canonical`/`Subnet`/`insert_subnet`/`load_subnet`/
`insert_range`/`insert_address`/`addresses_in` (`ipam_repo.rs`). **This story is that producer.**

**The audit is 14.3's and this story does not read `observation_record` at all.** A join written
here is scope, and it is the one the epic exists for — it deserves its own story.

## §1 — FOUR arbitrations this story cannot take alone

Each is written with the option refused and with what it costs. **A dev agent that meets one of
these mid-implementation settles it silently**, which is why they are here.

### (A) Do the two write routes sit behind a switch, and if so which one?

`POST /document-all` exists only when `OPENCMDB_DOCUMENT_ENABLED` is set (`main.rs:695-701`), and
the switch governs EXISTENCE, the route being merged ABOVE `auth_deny` so it is auth-gated like
every other path (`main.rs:696-700`).

- **(a) NO new switch — the IPAM routes always exist.** 🔑 The reason the documenting gesture has
  one is NFR5: it writes `origin='manual'`, `actor_id='operator'` — *a machine writing as a human*,
  which is the whole subject of the `authorship` gate. **IPAM has no such hazard**: the operator's
  own plan, written by the operator, in a register `declared_attribute` never touches (arbitration
  4). ⚠️ Cost: a fresh install gains a live write surface with no opt-in, on a product whose
  posture is closed-by-default (`is_public` is `/healthz` + `/assets/*` only, story 6.1).
- **(b) Reuse `OPENCMDB_DOCUMENT_ENABLED`.** Cheapest. ⚠️ **Refused on a measurement of meaning, not
  of cost**: that constant is named for the documenting gesture at four sites (`main.rs:113`,
  `page.rs:1229-1232`, `gesture.not_enabled`'s `%{switch}`, the diagnostic's security row), and
  arbitration (4) exists to keep the two registers apart. One switch for two registers re-fuses in
  configuration what the arbitration separated in the schema.
- **(c) A new `OPENCMDB_IPAM_ENABLED`.** Symmetrical and prudent. ⚠️ Cost measured on story 6.4:
  *"on the DEFAULT configuration there is still no gesture"* — a stock deployment would meet a
  seventh well-lit dead end, on the epic whose subject is the operator finally doing something.

**My recommendation: (a).** The switch on the documenting gesture guards an authorship hazard this
route does not have, and shipping a second dead-by-default surface is how this product got ten of
them. **Guy's to take, because it is a posture decision and not a technical one.**

### (B) What does `RepositoryError` gain, and what forces anyone to handle it?

🔴 **Two documents of the same commit contradict each other, and one describes code that does not
exist.** `ipam/mod.rs:122-123` states the adapter maps its refusals *"into
`RepositoryError::Constraint` with a name the caller can match on"*. It does not:
`ipam_repo.rs:332` is `RepositoryError::Backend(error.to_string())`, and that function's own doc
fifteen lines above says so (*"`RepositoryError` has no IPAM variant, so a refusal the domain states
precisely arrives at the caller as a sentence"*). **Three review layers did not catch it.**

🔑 And `Constraint` would be the WRONG target even if the code did it, by the doctrine the
`RepositoryError` file itself establishes for `InstantRegressed` and `ContradictoryObservation`:
*"`Constraint` means 'a database constraint was violated' by its own doc, and no database was
consulted here"* (`repo/mod.rs`). Six of the seven IPAM refusals are decided in Rust before any
statement runs.

`ipam_repo.rs:324-330` names this story: *"**Story 14.2 is the first story with a caller that must
DISTINGUISH these refusals to render them**, and that is the story where the variant earns itself."*

⚠️ **And NOTHING will force the handler to distinguish them.** Measured: `RepositoryError` is not
`#[non_exhaustive]` and no exhaustive `match` traverses it anywhere in `crates/`, so adding
`Ipam(IpamError)` produces **zero `E0004`** and the compiler names no site. This is the mirror of
story 6b.4b, where a one-variant enum forced the revisit; here the same gesture forces nothing.

- **(a) Add `RepositoryError::Ipam(IpamError)` and carry the obligation with a SET test** —
  `IpamError::ALL` (seven, `ipam/mod.rs:178`) against the statuses the handler serves, compared as a
  set in both directions, on `IpPolicy::ALL`'s own precedent (*a count is not a set*, story 6.5's
  M8). The test reds the day an eighth refusal exists.
- **(b) Keep `Backend(String)` and parse the sentence in the handler.** ⚠️ Refused in writing: it is
  string-matching a translated-adjacent sentence, and D47 exists against exactly that.
- **(c) Make `RepositoryError` `#[non_exhaustive]`.** ⚠️ Refused: it changes every downstream
  `match` in the crate for one story's benefit and `non_exhaustive` forces a `_` arm, which is the
  arm that swallows the new variant silently — *the opposite of what is wanted*.

**My recommendation: (a), and the SET test is the deliverable, not the variant.** ⚠️ Note it
changes one existing assertion: `ipam_repo.rs:907-914` asserts
`Err(RepositoryError::Backend(IpamError::RangeBoundsInverted.to_string()))` and becomes
`Err(RepositoryError::Ipam(IpamError::RangeBoundsInverted))` — which is the guard doing its job.

### (C) How is the overlap rule made to hold under concurrency?

🔴 Registered by 14.1 with **this story as owner**, and the register says it *"may not close it with
a `UNIQUE`"*. `insert_range` (`ipam_repo.rs:237-250`) reads every sibling, decides in Rust, then
inserts: no transaction, no lock, no index that could catch the loser. A `CHECK` referencing
siblings is `ERROR 1901` and a partial-overlap rule is not a `UNIQUE` key in any case. ⚠️ **The
first concurrent attempt did NOT reproduce and 14.1's review layer refused to read that as safety**:
with a 400 ms pause injected, two overlapping ranges both committed, both reporting success.

- **(a) Wrap in a transaction and take `SELECT … FROM ip_subnet WHERE id = ? FOR UPDATE` on the
  PARENT row first.** Serialises every range write within one subnet; different subnets stay
  parallel. The precedent is `repo::insert_device` (`repo.rs:1691-1708`), which does `conn.begin()`
  for exactly this class and becomes a SAVEPOINT when nested. ⚠️ Cost: the write path serialises per
  subnet — irrelevant at this product's write rate, and it must be SAID rather than discovered.
- **(b) `SELECT … FROM ip_range WHERE subnet_id = ? FOR UPDATE`** — lock the siblings. ⚠️ Weaker and
  subtler: it locks rows that EXIST, and the race is about a row that does not; it then depends on
  InnoDB gap locks on `ip_range_subnet` under REPEATABLE READ, which is an isolation-level
  behaviour, not a stated rule. *A guard whose correctness rests on an isolation level nobody
  states is a guard nobody can re-derive.*
- **(c) Register it again and ship the routes without it.** ⚠️ Refused: the register already says
  this story owns it, and *an action nobody carries is not an action*.

**My recommendation: (a).** ⚠️ **And the measurement is part of the deliverable**: the same 400 ms
injected pause must be re-run and measured to REFUSE the second writer — a fix whose proof is the
absence of the earlier symptom is not a proof, so the pause harness is what makes it one.

### (D) What does a cell show when the plan says nothing about it?

🔑 **`structural` is NOT an arbitration — the answer is already written.** The binding table defines
`infrastructure` as *"Gateway, equipment management, **and the network and broadcast addresses**"*,
and the UX spec says it in so many words: *"`.0` and `.255` are simply infrastructure the product
knows without being told; the arithmetic is how those rows get there, not a second concept."* So
`.0`/`.255` render as `infrastructure`, derived, with no second word.

**What IS open** is the rest of the grid. The example dataset had four states of one closed set
(`used`/`reserved`/`free`/`structural`, `example_data.rs:866`). A store-fed grid has a different
shape: an address may be **inside a range** (which carries one of four policies), **individually
defined** (`ip_address`), **both**, or **covered by nothing at all**.

- **(a) The uncovered cell is its own state, named and drawn.** The plan genuinely says nothing
  about it, and saying so is honest. ⚠️ It needs a word, and **the vocabulary may not be extended by
  a story** — so it must reuse one the binding table already holds, or be drawn without a noun.
- **(b) The uncovered cell renders as `free`.** ⚠️ Refused on the epic's own reason: `free` is a
  claim that the address may be allocated, and 14.3's whole point is that the product must not offer
  an address it has not checked. Rendering *unknown* as *free* pre-commits the screen to the
  sentence 14.3 exists to make false.
- **(c) An uncovered subnet is not drawn at all** — the grid only exists where a range does. ⚠️
  Simple and severe: it makes the empty-plan case the ordinary case and AC5's *"name the gesture
  that fills it"* the whole screen on day one, which may be exactly right.

**My recommendation: (a) drawn without a noun** — a cell the plan does not cover is left blank with
its `aria-label` saying so, and no legend entry claims a state. ⚠️ It must still be told apart from
`free` by something other than colour (constraint 6): blank-with-border vs. an explicit pattern.

## §2 — Five things the first draft would leave a dev agent to settle SILENTLY

1. **`/ipam` leaves the pool-free router and loses a compile-time guard.** Today the demonstration
   screens are merged AFTER `.with_state(pool)` so their state is `()` and `State<MySqlPool>` fails
   to compile (`main.rs:681-685`). A `Fed` `/ipam` must leave that router — story 6b.5 paid this for
   `/dashboard` and narrowed the promise in writing rather than dropping it. **Say the same sentence
   here**: the refusal holds for the screens that remain, and `/ipam` is the second exemption.
2. **Deleting `ExampleContent::IpamOccupancy` is what makes the compiler name the sites** — the
   `render` dispatch (`screens.rs:180`), the route-table witness (`main.rs:1393-1395`) and
   `example_contents.len() == 5` (`main.rs:1500-1508`). Delete the variant FIRST and let `E0004`
   drive the sweep; do not grep for `ipam` and hope.
3. **The stylesheet guard cannot see the grid's classes.** `every_class_a_template_names_is_defined_
   in_the_stylesheet` skips any `class="…"` containing `{` (`page.rs:5426-5428`), and the cell is
   `class="ipam-cell {{ cell.modifier }}"`. What covers the four modifiers today is
   `the_legend_names_every_cell_modifier_as_a_literal` (`example_screens.rs:1580`) — **a test over
   the example dataset that dies with it.** Its replacement is owed in the same commit, or the new
   modifiers ship covered by nothing.
4. **The axe gate measures no `/ipam` state.** Routes are scraped from the nav and the only
   query-string states it walks are `/triage`'s (`axe-gate.mjs:81`, `:86`). A store-fed `/ipam` has
   at least three states worth a pass — no plan, a plan with ranges, a refused form — and none is
   reachable today. ⚠️ And `a11y/seed.sql` seeds NO ipam row, so an axe pass over a real grid
   measures an empty one. **Widening the seed puts a new SQL writer in the `authorship` gate's
   perimeter** — `AUTHORSHIP_ROOTS` and `SANCTIONED_SITES` already name `a11y/seed.sql`, so this is
   free here, but the file's own header says the two are one act.
5. **The UX spec asks for a grid this product does not render, and the divergence must be settled
   rather than inherited.** `ux-design-specification.md:1634` wants `role="grid"` with keyboard
   navigation, `role="gridcell"`, a synthetic summary and an accessible *"jump to next free IP"*.
   Story 6b.7 shipped `<ul role="list">` and refused `role="img"` on a measured ARIA reason
   (`aria-label` on a bare `div` maps to `generic`, where ARIA 1.2 prohibits it). ⚠️ **This story
   rebuilds the grid**, so it either implements the spec's shape or records a divergence with its
   reason — it may not leave the two documents disagreeing silently a second time.

## Acceptance Criteria

**AC1 — `/ipam` is fed by the store.** `Screen::Ipam.nature()` is `Nature::Fed`; the route leaves
the pool-free router; `ExampleContent::IpamOccupancy` and the whole invented dataset
(`example_data.rs:839-1055`: `ExampleSubnet`, `CellState`, the six octet constants, `subnets()`,
`subnet_by_slug`, `address_conflict`) are DELETED, not bypassed. The route-table partition guard
passes with four `ExampleContent` variants where there were five.

**AC2 — the grid draws the plan and nothing else.** Every cell's state is derived from `ip_range`
and `ip_address` for the selected subnet; **no cell state reads `observation_record`** (that is
14.3). A test asserts the absence by mutation: making the view read an observation reds it.

**AC3 — a policy carries a pattern and a word, never a hue alone** (constraint 6). Each of the four
policies renders a distinct non-colour treatment, asserted over `IpPolicy::ALL` so a fifth policy
cannot compile without one. The `reserved` hatch already exists (`app.css:894-896`).

**AC4 — `structural` is gone as a displayed word.** No `ipam.state.structural` key, no
`.ipam-cell-structural` rule, no `CellState::Structural`; `.0`/`.255` render as `infrastructure`,
derived. ⚠️ **And the vocabulary gate gains `structural` in its denylist in the SAME commit** — the
register says adding it earlier simply reds the build, so adding it later is what nobody does.

**AC5 — two write routes, and the first carries the machinery.** A range and an address can each be
defined through `/ipam`. The FIRST route carries: the Origin check, a KEYED refusal body per status
in BOTH locales, the `authorship` sanction if it writes provenance (it does not — say so), and the
browser gates. The SECOND reuses them, and a test asserts the reuse rather than a comment claiming
it.

**AC6 — every refusal the operator can reach is a KEY, in both locales, naming the rule.** The seven
`IpamError` variants map to statuses and bodies as a SET compared in both directions. 🔴 Story 6.4's
finding is the reason: a success message shipped in English under a French UI with a raw UUID in it,
and three layers of guards were green over it.

**AC7 — the overlap rule holds under concurrency**, by arbitration (C), and the 400 ms injected
pause that made two overlapping ranges commit is re-run and measured to REFUSE the second writer.

**AC8 — an empty plan names the gesture that fills it and links to it.** Story 6b.4's finding: *a
door labelled with the room you are already in is not a door.*

**AC9 — `#![allow(dead_code)]` is narrowed or removed.** ⚠️ **The register's figure is wrong and the
correction belongs here**: it says *eleven items*; measured on the merged tree, removing the
attribute yields **TEN warnings covering FOURTEEN items** (one warning groups five associated
items), identically under `cargo build` and under `clippy --all-targets`. 🔑 What the attribute
hides is the absence of a PRODUCER, not the absence of a test — the six `#[tokio::test]`s do call
these functions. Correct the register row at its site.

**AC10 — the doc that describes code that does not exist is corrected.** `ipam/mod.rs:122-123`
claims a `Constraint` mapping that was never written. Correct it to what arbitration (B) decides,
and say in the commit that it was false rather than fixing it silently.

**AC11 — THE LIVE COUNT lives here**, with the command and both conditions named. Baseline: **873**
(583 bin + 191 core + 99 xtask) at `38da035`.

**AC12 — no regression**: ten `cargo xtask ci` gates, `clippy --workspace --all-targets -D
warnings`, `RUSTFLAGS="-D warnings" cargo test --workspace --locked`, `fmt`, `cargo deny`. ⚠️ **And
BOTH browser gates ARE claimed this time** — this story renders a screen and adds a control, which
is exactly the case story 6b.11's AC5 amendment covers: *a source-reading guard does not suffice
where the defect lives in the DOM.*

## Tasks / Subtasks

- [ ] **T0** Take the four arbitrations of §1 with Guy. Nothing below is stable until they are.
- [ ] **T1** (AC10, AC9) Correct `ipam/mod.rs:122-123`; measure the dead-code figure and correct the
  register row. These are cheap and they are what the next reader trips on.
- [ ] **T2** (AC1) Delete `ExampleContent::IpamOccupancy` FIRST and let the compiler name the sites.
- [ ] **T3** (AC2, AC3, AC4, AC8) The store-fed view: read `ip_range` + `ip_address`, derive each
  cell, the occupancy line, the next-free panel, the empty-plan sentence.
- [ ] **T4** (AC5, AC6) The two write routes on `document.rs`'s shape — a port, a state with no
  pool, the Origin check, keyed bodies, `hx-post` with the swap target carrying both handlers.
- [ ] **T5** (AC7) The transaction and the parent-row lock, with the pause harness as its proof.
- [ ] **T6** (§2.3, §2.4) Replace the legend guard that dies with the dataset; widen `a11y/seed.sql`
  and the axe gate's states.
- [ ] **T7** (§2.5) Settle the `role="grid"` divergence — implement or register, with the reason.
- [ ] **T8** (AC11, AC12) Measure. Both gates, both store conditions, the command named.

## Dev Notes

### Traps this project has already paid for, and which apply here

- 🔴 **A guard placed where the defect cannot occur reads as coverage and is none** — Epic 5's
  dominant class, and story 6b.5's specimen was an anti-sum guard over two pure builders that could
  not add each other's counts. The new view has the same shape: guard the COMPOSED route, not the
  pure builder alone.
- 🔴 **A source guard cannot see a cascade** (story 6.4): four correct `--accent-document`
  declarations painted nothing because a more specific selector won. The new policy treatments are
  four classes on one element — the same shape.
- 🔴 **A literal is not a key, and the locale guard can only see keys** (story 6b.3). The example
  dataset's French proper nouns rendered under an English UI with the whole suite green.
- 🔴 **`ascii_bin` is PAD SPACE and both obvious remedies fail** — `IN (...)` accepts `'static '`
  and so does `= TRIM(...)`. The carrier is `LENGTH(x) = LENGTH(TRIM(x))` (`0007:142`).
- ⚠️ **`cargo test` on a virgin database races itself on concurrent `migrate!`**; `DB_TEST_LOCK`
  is the remedy and every store-backed test takes it FIRST. Each test owns its own CIDR — a
  panicking test poisons its successor and inflates every mutation count taken in a full run.
- ⚠️ **The mutation driver refuses an anchor that matches twice**, and `cargo xtask mutate` is the
  artefact now (story 6.4b). Use it; it records the prediction beside the mutation and checks it.
- ⚠️ **`app.yml` is invisible to Cargo's incremental build** (story 6b.9): changing only the
  translation file rebuilds in 0.07 s and the new string is absent from the binary. Any mutation
  editing `app.yml` alone measures nothing.
- ⚠️ **Read the artefact you are about to believe, not the source you just edited** — a screenshot
  of a stale binary is not a look at your code (story 6b.4b, three times in one day).

### What the store must answer, and what it must not

**Must**: which ranges cover this subnet and with what policy · which addresses are individually
defined · what is the subnet's extent. **Must not, in this story**: anything about observations.
Containment runs in RUST, never in SQL (D10); `architecture.md:4847` (F57) asks that SQL-side
comparison be costed before any epic reintroduces it, and this story does not reintroduce it.

### Project Structure Notes

- New: a write-route module on `document.rs`'s shape; a store-fed ipam view module. ⚠️ `page.rs` is
  at 2033-ceiling risk — story 6.4 split `identity_view.rs` out for exactly this, and the answer is
  the split `CLAUDE.md` prescribes, never shorter prose.
- Deleted: the ipam half of `example_data.rs`, `_ipam_example.html`, `ipam.state.structural`,
  `.ipam-cell-structural`.
- Touched: `screens.rs` (nature, router skip), `main.rs` (route merge, the partition guard's
  counts), `app.yml`, `app.css`, `a11y/seed.sql`, `xtask` (the vocabulary denylist).
- **Not touched**: `epics.md`, the UX spec — a story may not edit either.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Epic 14` — the six constraints, 14.2's five ACs]
- [Source: `_bmad-output/planning-artifacts/prd.md#The PLAN axis` — the binding vocabulary]
- [Source: `_bmad-output/planning-artifacts/ux-design-specification.md:1634` — the grid's a11y shape]
- [Source: `crates/opencmdb-bin/src/document.rs` — the write-route machinery to reuse]
- [Source: `crates/opencmdb-bin/src/ipam_repo.rs:324-330` — the seam that names this story]
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md` — the two rows owned here]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log

- 2026-09-11 — contexted. Four arbitrations posed rather than taken; three defects found while
  contexting and carried as criteria (the false `Constraint` doc, the register's wrong dead-code
  figure, the UX-spec grid divergence).
