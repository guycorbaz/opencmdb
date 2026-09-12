# Story 14.2: The plan on screen

Status: review

✅ **The four arbitrations of §1 were TAKEN by Guy on 2026-09-11**, the recommendation in all four.
⚠️ They were taken BEFORE the mandatory validation pass, which is the order this project uses and
which has already paid once: story 14.1's address representation was re-arbitrated after the
validation REFUTED the argument that chose the first answer. **A measurement that refutes one of
these four returns it to Guy; it does not get settled quietly in the implementation.**

Epic 14 (IPAM), decomposed 2026-09-11 in `epics.md` (`4b5db27`). Second story of four.
Baseline: `38da035` (master), **873 tests** (583 bin + 191 core + 99 xtask), ten gates.

## Story

As the operator,
I want the addressing plan I hold to be the one the product draws,
so that the screen stops showing an invented network and starts showing mine.

🔴 **SPLIT at implementation — 14.2b INSERTED, Epic 14 → FIVE stories** (Guy, 2026-09-11;
`epics.md` NOT edited, the divergence is registered). The mandatory validation had grown this story
from two write routes to three and added a `free` treatment, an auth-perimeter property, a permanent
concurrency harness and a `classify` repair. **14.2 is the SCREEN; 14.2b is the OPERATOR'S HANDS** —
the three write routes, the keyed refusals, the concurrency fix, AC5b and `classify`. Precedent:
5.9 (the schema) / 5.9b (the resolver that fills it), 5.14/5.14b, 6b.4/6b.4b, 6.4/6.4b.

⚠️ **The accepted cost is stated rather than discovered: between the two, `/ipam` draws a plan
nothing can fill.** So AC8 changes — the empty-plan sentence **names the gesture as NOT YET BUILT**,
through `Gesture::Planned { owner }`, which is the product's own shape for exactly this and which
story 6b.4b made a compile-time obligation. *A door labelled with a room that does not exist yet is
honest; a door labelled with the room you are already in is not.*

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
(`ipam/mod.rs:46`, `:129`); `canonical`/`from_canonical`/`Subnet`/`insert_subnet`/`load_subnet`/
`insert_range`/`insert_address`/`addresses_in` (`ipam_repo.rs`). **This story is that producer.**

**The audit is 14.3's and this story does not read `observation_record` at all.** A join written
here is scope, and it is the one the epic exists for — it deserves its own story.

## §1 — FOUR arbitrations, TAKEN by Guy on 2026-09-11

Each is recorded with the options refused and with what the choice costs. **A dev agent that meets
one of these mid-implementation settles it silently**, which is why they were posed rather than
assumed.

### (A) ✅ NO switch — the IPAM write routes always exist

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
  of cost**: that constant is named for the documenting gesture at `main.rs:113`, `:350`, `:432`,
  `page.rs:260`, `:760`, `:5772` and `app.yml:441,484-486`, and arbitration (4) exists to keep the
  two registers apart. One switch for two registers re-fuses in configuration what the arbitration
  separated in the schema. ⚠️ **The story first claimed a fourth site in the diagnostic's security
  row; the validation measured that `diagnostic.rs` never names the constant at all** — its four
  security rows are public paths, Basic, the metrics token and secrets. A refused option deserves a
  true reason as much as a chosen one.
- **(c) A new `OPENCMDB_IPAM_ENABLED`.** Symmetrical and prudent. ⚠️ Cost measured on story 6.4:
  *"on the DEFAULT configuration there is still no gesture"* — a stock deployment would meet a
  seventh well-lit dead end, on the epic whose subject is the operator finally doing something.

✅ **Guy, 2026-09-11: option (a).** The switch on the documenting gesture guards an authorship
hazard this route does not have, and shipping a second dead-by-default surface is how this product
reached ten well-lit dead ends. ⚠️ **The accepted cost is written rather than discovered**: a fresh
install gains a live write surface with no opt-in, and the release notes owe that sentence — the
first deployment where an operator can change stored state without setting anything.

### (B) ✅ `RepositoryError::Ipam(IpamError)`, and the SET test is the deliverable

🔴 **Two documents of the same commit contradict each other, and one describes code that does not
exist.** `ipam/mod.rs:122-123` states the adapter maps its refusals *"into
`RepositoryError::Constraint` with a name the caller can match on"*. It does not:
`ipam_repo.rs:332` is `RepositoryError::Backend(error.to_string())`, and that function's own doc
six lines above (`:325-326`) says so (*"`RepositoryError` has no IPAM variant, so a refusal the domain states
precisely arrives at the caller as a sentence"*). **Three review layers did not catch it.**

🔑 And `Constraint` would be the WRONG target even if the code did it, by the doctrine the
`RepositoryError` file itself establishes for `InstantRegressed` and `ContradictoryObservation`:
*"`Constraint` means 'a database constraint was violated' by its own doc, and no database was
consulted here"* (`repo/mod.rs:53-56`). ⚠️ **The story first wrote *"six of the seven are decided in
Rust before any statement runs"*; measured, it is FOUR** — `MalformedAddress`,
`PrefixLengthNotInFamily`, `BaseIsNotTheNetworkAddress` and `RangeBoundsInverted`; the other three
follow a `load_subnet`. The conclusion survives on the doctrine's own terms, which are about whether
a DATABASE CONSTRAINT was violated and not about whether a statement ran: **none of the seven is
refused by the DDL** (`ipam/mod.rs:130-133`). *The conclusion was right and its stated premise
matched no partition of the seven.*

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

✅ **Guy, 2026-09-11: option (a), and the SET test is the deliverable, not the variant.** Adding the
variant is half an hour; what earns the story is the test that makes an unhandled refusal impossible
to ship, since the compiler will not. ✅ The gap-hunt confirmed both halves by building it: the
variant composes with `#[from]`, `Clone`/`PartialEq`/`Eq` survive, **zero compiler errors**, and
exactly one existing assertion breaks (`ipam_repo.rs:907-914`) — the guard doing its job.

🔴 **RE-SCOPED the same day, because as first written the SET test was this epic's dominant defect
one type over. Guy: the SET is over what the HANDLER CAN RECEIVE, not over `IpamError::ALL`.**
Measured through the very adapters AC5's routes call:

| gesture | what comes back | in `IpamError`? |
|---|---|---|
| a range in a subnet that does not exist | `Err(NotFound)` | **no** |
| the same CIDR defined twice | `Err(Constraint("unique"))` | **no** |
| **the same address defined twice** | `Err(Constraint("unique"))` | **no** |
| the loser of a lock wait, once (C) lands | `Err(Backend("…1205…"))` | **no** |

🔑 *A SET over `IpamError::ALL` is correct about `IpamError` and silent about the three refusals the
operator meets first* — and the second row is **the single most likely refusal on this screen**, an
operator typing an address they already entered. The first is the state of **every empty plan**. As
first written, *"makes an unhandled refusal impossible to ship"* was false.

### (C) ✅ A transaction and a parent-row `FOR UPDATE`, with the pause harness as its proof

🔴 Registered by 14.1 with **this story as owner**, and the register says it *"may not close it with
a `UNIQUE`"*. `insert_range` reads every sibling (`ipam_repo.rs:238`), decides in Rust (`:243-250`), then
inserts (`:251`): no transaction, no lock, no index that could catch the loser. A `CHECK` referencing
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

🔴 **RE-ARBITRATED the same day, on the gap-hunt's measurement. Guy: LOCK THE READ THAT DECIDES** —
`SELECT first_addr, last_addr FROM ip_range WHERE subnet_id = ? FOR UPDATE`.

**What the layer measured, in order.** First the instrument: a `#[cfg(test)]` pause inside the REAL
`insert_range`, two concurrent callers — `a = Ok`, `b = Ok`, two overlapping ranges committed,
`left: 2 right: 1`. *The harness can open the window*, which is what makes everything after it mean
something. Then option (a): the loser blocks, re-reads, and is refused on the DOMAIN rule
(`RangeOverlapsAnother`) — **no deadlock, no 1205, and a better answer than `Contention`**. Twelve
ipam tests green.

🔴 **Then the ordinary DRY gesture brought the race back.** Reusing `load_subnet` inside the
transaction before taking the lock — the helper already exists, already takes an `Executor`, and is
what `insert_range` calls today — gives `a = Ok`, `b = Ok`, **two overlapping ranges committed
again, with the lock still taken and worthless.** Under REPEATABLE READ the snapshot is fixed by the
first CONSISTENT read, and a locking read does not fix it; the parent lock only worked because the
sibling `SELECT` happened to be that first read.

🔑 **So the story's own reason for refusing (b) is REFUTED**: it said (b) *"depends on InnoDB gap
locks under REPEATABLE READ … a guard whose correctness rests on an isolation level nobody states is
a guard nobody can re-derive"* — and (a) rested on an isolation-level behaviour just as much, while
stating none of it. *The refusal was right about the danger and wrong about which option carried
it.* Measured, the locking sibling read refuses the second writer and survives the DRY line.

⚠️ **AC7's harness ships as a PERMANENT test, not as a measurement taken once** — the statement
ordering is load-bearing and invisible, and nothing else in the tree would catch its loss.
⚠️ The serialisation is per subnet and is SAID: two operators defining ranges in one subnet queue;
in two subnets they do not. **And there is no retry path**: `insert_range` takes a bare connection,
not `WriteRepository::transact`, so NFR15's *"the caller replays the closure"* does not exist here.

### (D) ✅ A cell the plan does not cover is drawn without a noun

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

✅ **Guy, 2026-09-11: option (a), drawn without a noun** — the cell is left blank, its `aria-label`
says the plan does not cover it, and **no legend entry claims a state**, because a story may not
extend the binding vocabulary and *not covered* is not one of its four words. 🔑 `free` was refused
because it asserts the address may be allocated, and 14.3 exists so the product never offers one it
has not checked.

🔴 **And the gap-hunt measured that this is UNBUILDABLE as first written, so Guy re-aimed it the
same day: `free` GAINS AN EXPLICIT TREATMENT and the bare cell keeps the blank.** In Chrome, against
the shipped stylesheet, a `free` cell and a bare `.ipam-cell` are **byte-identical in every computed
property** — same `rgba(0,0,0,0)`, same `none` background image, same border, same box:
`.ipam-cell-free` is `background: transparent` (`app.css:897`) over a base that already draws the
border. *Not hard to tell apart by colour — indistinguishable in every property there is.*

⚠️ **Two consequences the first draft did not carry.** AC4 deletes only `structural`, so **`free`'s
new treatment is a criterion this story did not have** — it is added to AC3. And **AC3's guard
cannot reach it**: AC3 asserts a treatment per `IpPolicy::ALL` so a fifth policy cannot compile
without one, but *not covered* is **not a policy** — it sits outside that enum, and the guard is
structurally blind to the one state this arbitration invents. 🔑 *A guard scoped to an enum cannot
see a state that is not in it* — this epic's dominant class, reached through the type system rather
than through a test. The pair `free` / *not covered* needs its own assertion.

⚠️ And arbitration (D) removes the mechanism that makes a cell class visible to the stylesheet
guard at all: the guard skips any `class="…"` containing `{` (`page.rs:5426`), so what covers the
four modifiers today is the LEGEND's literals, pinned on purpose by
`the_legend_names_every_cell_modifier_as_a_literal`. *"No legend entry claims a state"* means a
not-covered class would be seen by nothing.

## §2 — Five things the first draft would leave a dev agent to settle SILENTLY

1. **`/ipam` leaves the pool-free router and loses a compile-time guard.** Today the demonstration
   screens are merged AFTER `.with_state(pool.clone())` (`main.rs:680`) so their state is `()` and
   `State<MySqlPool>` fails to compile. A `Fed` `/ipam` must leave that router — story 6b.5 paid
   this for `/dashboard` and narrowed the promise in writing rather than dropping it. ⚠️ **This
   paragraph called `/ipam` the SECOND exemption; the validation measured FIFTH; the blind review
   layer then measured SIXTH — and the blind layer is right.** `page::triage_router`
   (`page.rs:1552`) already carries FIVE screens — `/triage`, `/dashboard`, `/diagnostic`,
   `/sources`, `/devices` — so `/ipam` is the SIXTH screen served with a pool. 🔑 *Two review layers
   gave two numbers and the tie was broken by counting, not by seniority*: the fact-check said
   "fifth exemption" while listing five predecessors, which is the same sentence refuting its own
   count that the blind layer caught in the code. 🔑 *The narrowed promise is worth less each time it
   is narrowed, and saying "the second" would have hidden that.* Say the true count.
2. **Deleting `ExampleContent::IpamOccupancy` makes the compiler name THREE sites, and one the
   story listed is not among them.** Measured by deleting it and running `cargo check --workspace
   --all-targets`: three errors, all **`E0599`** — the `render` dispatch (`screens.rs:180`), the
   `nature()` arm (`screens.rs:304`, **which the story's first draft did not list**) and the
   route-table witness (`main.rs:1393-1395`). ⚠️ **`example_contents.len() == 5` (`main.rs:1500`) is
   NOT compiler-named** — it compiles and fails at run time, so it is carried by the test suite and
   not by the build. And the sweep is driven by `E0599`, not `E0004`: **`E0004` is what ADDING a
   variant produces**, which is arbitration (B)'s subject and the opposite gesture. Delete the
   variant FIRST all the same; do not grep for `ipam` and hope.
3. **The stylesheet guard cannot see the grid's classes.** `every_class_a_template_names_is_defined_
   in_the_stylesheet` skips any `class="…"` containing `{` (`page.rs:5426-5428`), and the cell is
   `class="ipam-cell {{ cell.modifier }}"`. What covers the four modifiers today is
   `the_legend_names_every_cell_modifier_as_a_literal` (`example_screens.rs:1580`) — **a test over
   the example dataset that dies with it.** Its replacement is owed in the same commit, or the new
   modifiers ship covered by nothing.
4. **The axe gate measures no `/ipam` state.** Routes are scraped from the nav and the only
   query-string states it walks are `/triage`'s (`axe-gate.mjs:81`, `:87`). A store-fed `/ipam` has
   at least three states worth a pass — no plan, a plan with ranges, a refused form — and none is
   reachable today. ⚠️ And `a11y/seed.sql` seeds NO ipam row, so an axe pass over a real grid
   measures an empty one. **Widening the seed puts a new SQL writer in the `authorship` gate's
   perimeter** — `AUTHORSHIP_ROOTS` and `SANCTIONED_SITES` already name `a11y/seed.sql`, so this is
   free here, but the file's own header says the two are one act.
5. **The UX spec asks for a grid this product does not render, and the divergence must be settled
   rather than inherited.** `ux-design-specification.md:1632-1634` wants `role="grid"` with keyboard
   navigation (`:1632`), `role="gridcell"` and a synthetic summary (`:1633`), and an accessible
   *"jump to next free IP"* (`:1634`).
   Story 6b.7 shipped `<ul role="list">` and refused `role="img"` on a measured ARIA reason
   (`aria-label` on a bare `div` maps to `generic`, where ARIA 1.2 prohibits it). ⚠️ **This story
   rebuilds the grid**, so it either implements the spec's shape or records a divergence with its
   reason — it may not leave the two documents disagreeing silently a second time.

## §3 — What the gap-hunt MEASURED, beyond the four re-arbitrations

Each was built and run against a live `mariadb:10.11.11`, not reasoned about.

### 🔴 `RepositoryError::Contention` is DEAD CODE, and arbitration (C) is what makes it reachable

`classify` (`repo.rs:1613-1614`) matches `Some("1213") | Some("1205") => Contention`. It never
fires. **The cause is named by a check rather than by a story**: sqlx's `code()` returns the
**SQLSTATE**, and the MySQL number lives in a separate `number` field. Measured twice, on two
different errors — a lock-wait timeout surfaces as `code() = Some("HY000"), number = 1205`, and a
duplicate key as `code() = Some("23000"), number = 1062`. So the arm compares a SQLSTATE against a
MySQL number and can never match.

`Contention` has **one producer site, no test and no consumer**. ⚠️ It is pre-existing and NOT this
story's to have caused — but **(C) is what first makes it reachable on a route an operator
presses**, and it surfaces as a raw English driver sentence (`Backend("error returned from
database: 1205 (HY000): Lock wait timeout exceeded…")`), which is exactly what AC6 forbids. **Decide
here: fix `classify` or map the case in the handler — and say which.** A register row is not enough
when the story's own arbitration is what opens the path.

### ⚠️ The deletion sweep is smaller in the compiler and larger outside it than §2.2 says

Three `E0599`, and **one of them needs `--all-targets`** — a plain `cargo build` names only two, so
*"let the compiler drive it"* silently misses `main.rs:1393` unless the flag is there. What the
compiler names **nothing** about: **18** `ipam.*` keys in `app.yml`, **13** `.ipam*` rules in
`app.css`, `_ipam_example.html`, **41** `ipam` references in `example_screens.rs` (the story first said 33, which was the lowercase count and under-reported what the sweep actually removed) (`IpamStrings`,
the `Ipam` template struct, `ipam_body` and its tests), and `main.rs:1822-1840`'s `/ipam?subnet=`
route test. ⚠️ And `example_contents.len() == 5`'s failure message reads *"a sixth is a screen that
grew example content without a story deciding it should"* — **an invitation to edit the digit**,
which is story 6b.6's recorded defect verbatim (*three bookkeeping counts whose messages read
"update this number", which a developer follows*).

### ⚠️ The split is FORCED, not optional

`page.rs` is at **1954 of 2000** — 46 lines. `/ipam` must be mounted with a pool, and
`triage_router` lives in `page.rs`. **Name where the route registration goes before a dev discovers
it at line 2001.**

### ⚠️ Six decisions the story does not take, each measured rather than supposed

1. **A label is not required.** `label` is `NOT NULL` with no non-empty CHECK; `""` inserts cleanly.
   The form's required-ness is undecided.
2. **The nil UUID is accepted as an id.** Story 6.1's review added a nil-UUID refusal AT THE ROUTE
   (D21/D48, *"before 6.2 can forget it"*) and `insert_device` refuses it — **none of
   `insert_subnet`/`insert_range`/`insert_address` does.** Who mints a range's id, and is it v7?
3. **An address inside an existing range is silently accepted, in BOTH orders.** Nothing objects, so
   the grid must pick a **rendering priority** — and story 6b.7's own finding is this exact trap:
   *"`state_of` tests `used` before `reserved`, so an octet in both lists renders silently as used —
   a priority order is a rendering decision and must not double as a repair."* Now reachable against
   real data.
4. **The subnet selector has no key.** `ScreenQuery.subnet` is a SLUG from `ExampleSubnet::slug`,
   which this story deletes; `ip_subnet` has **no slug column** — `id CHAR(36)`, `base`,
   `prefix_len`, `label`. `/ipam?subnet=…` must change meaning and the story does not say to what.
5. **`free` has no definition in a store-fed grid.** `ipam.state.free` and the *next free address*
   panel survive AC4, but `free` is not one of the four binding PLAN words. 14.3 owns the
   EXCLUSION; **14.2 owns the word.**
6. **The axe gate has no floor for `/ipam`.** It walks a 256-cell grid today; after this story it
   would walk an **empty page and report success** — the `AXE_REQUIRE_QUEUE` shape exactly. Widening
   `a11y/seed.sql` is named in §2.4; **the missing floor is not**, and it is what turns the existing
   pass into a pass over nothing.

## Acceptance Criteria

**AC1 — `/ipam` is fed by the store.** `Screen::Ipam.nature()` is `Nature::Fed`; the route leaves
the pool-free router; `ExampleContent::IpamOccupancy` and the whole invented dataset
(`example_data.rs:839-1055`: `ExampleSubnet`, `CellState`, the six octet constants, `subnets()`,
`subnet_by_slug`, `address_conflict`) are DELETED, not bypassed. ⚠️ **Two counts, and the story
first conflated them**: `ExampleContent` has **SIX** variants (`screens.rs:134-158`) and keeps five;
what goes **5 → 4** is `example_contents` (`main.rs:1501`), which collects only screens whose nature
is `Example` — `DevicesInventory` belongs to `Screen::Devices`, which is `Mixed`. **And
`main.rs:1497-1499`'s sentence must change in the same commit**: *"five are wholly example, three
are fed by the store and two are mixed"* becomes four / four / two.

**AC2 — the grid draws the plan and nothing else.** Every cell's state is derived from `ip_range`
and `ip_address` for the selected subnet; **no cell state reads `observation_record`** (that is
14.3). A test asserts the absence by mutation: making the view read an observation reds it.

**AC3 — a policy carries a pattern and a word, never a hue alone** (constraint 6). Each of the four
policies renders a distinct non-colour treatment, asserted over `IpPolicy::ALL` so a fifth policy
cannot compile without one. The `reserved` hatch already exists (`app.css:894-896`). ⚠️ **And
`free` gains a treatment of its own**, by the re-arbitration of §1(D): measured in Chrome, a `free`
cell and a bare cell are identical in every computed property, so *not covered* cannot be told from
*free* on today's sheet. 🔑 **That pair needs its OWN assertion**, because the `IpPolicy::ALL` guard
is structurally blind to a state that is not a policy.

**AC4 — `structural` is gone as a displayed word.** No `ipam.state.structural` key, no
`.ipam-cell-structural` rule, no `CellState::Structural`; `.0`/`.255` render as `infrastructure`,
derived. ⚠️ **And the vocabulary gate gains `structural` in its denylist in the SAME commit** — `epics.md`'s
constraint (5) and `ux-design-specification.md:1404-1405` say adding it earlier simply reds the
build, so adding it later is what nobody does. (Not `deferred-work.md`: there is no such row there,
and in this project *the register* means that file.)

**AC5 · AC5b · AC6 · AC7 → STORY 14.2b.** The three write routes and the machinery the first one
carries; the auth perimeter as a property; the keyed refusal bodies over what the handler can
RECEIVE; the concurrency fix and its permanent pause harness. 🔑 They are moved WITH their
measurements, not re-derived there: `insert_subnet` has no caller outside its own tests (so an empty
plan stays empty and AC5 is three routes, not two); an always-on `POST` below `auth_deny` left 873
tests and ten gates green over an unauthenticated `201`; the parent-row lock is defeated by one
ordinary DRY line, so the lock goes on the read that DECIDES; and `RepositoryError::Contention` is
dead code because `classify` compares sqlx's SQLSTATE against a MySQL number.

**AC8 — an empty plan says the gesture is NOT YET BUILT and names its owner.** ⚠️ Changed by the
split: the gesture arrives in 14.2b, so promising a door here would be the defect AC8 exists to
prevent, pointed the other way. It renders through `Gesture::Planned { owner }` — the product's
existing shape, which story 6b.4b made a compile-time obligation by giving the enum one variant, so
14.2b cannot ship the route without meeting this site.

**AC9 — `#![allow(dead_code)]` is narrowed or removed.** Measured on `38da035` with the attribute
removed: **ELEVEN warnings covering FIFTEEN items**, identically under `cargo build --workspace` and
`clippy --workspace --all-targets` (one warning groups `Subnet`'s five associated items). 🔴 **This
criterion first said TEN and FOURTEEN, and the validation refuted it — the error was the
INSTRUMENT**: the figure was taken with `grep "never used"`, which cannot see ``struct `Subnet` is
never constructed``. *A measurement whose instrument cannot see the answer is not a measurement.*
`deferred-work.md`'s *eleven items* was right about the number and wrong about the noun. Corrected
in both twins and the register by PR #171 before this story starts. 🔑 What the attribute hides is
the absence of a PRODUCER, not the absence of a test — the six `#[tokio::test]`s do call these
functions. ⚠️ **Narrowed by the split**: this story gives the READ functions a producer, so the
attribute shrinks to what remains dead; **14.2b removes it**, and neither story may leave it
module-wide while claiming to have paid it off.

**AC10 — the doc that describes code that does not exist is corrected.** `ipam/mod.rs:122-123`
claims a `Constraint` mapping that was never written. Correct it to what arbitration (B) decides,
and say in the commit that it was false rather than fixing it silently.

**AC11 — THE LIVE COUNT lives here**, with the command and both conditions named. Baseline: **873**
(583 bin + 191 core + 99 xtask) at `38da035`, re-measured by the validation. ⚠️ **And check what the
"without a store" run actually had**: the validation measured the bin suite at **5.06 s** with
`DATABASE_URL` unset, where this project's notes record ~0.65 s — so either a `.env` is supplying a
store or the floor has moved, and recording a figure under the wrong label is how *"the clock is the
tell"* stops being true.

**AC12 — no regression**: ten `cargo xtask ci` gates, `clippy --workspace --all-targets -D
warnings`, `RUSTFLAGS="-D warnings" cargo test --workspace --locked`, `fmt`, `cargo deny`. ⚠️ **And
BOTH browser gates ARE claimed this time** — this story renders a screen and adds a control, which
is exactly the case story 6b.11's AC5 amendment covers: *a source-reading guard does not suffice
where the defect lives in the DOM.*

## Tasks / Subtasks

- [x] **T0** ✅ The four arbitrations of §1 were taken by Guy on 2026-09-11, then ALL FOUR came back
  from the validation and were re-arbitrated the same day. ⚠️ Then the story was SPLIT at
  implementation: 14.2 the screen, **14.2b the operator's hands**.
- [x] **T1** (AC10) `ipam/mod.rs`'s false `Constraint` doc corrected, and the variant that makes the
  corrected sentence TRUE shipped with it: `RepositoryError::Ipam(IpamError)`. Proved to red first —
  the assertion was moved to the variant and the build named `E0599` **only under `--all-targets`**,
  which is §3's finding met in its first minute.
- [x] **T2** (AC1) Delete `ExampleContent::IpamOccupancy` and let the compiler name its sites; sweep
  what it cannot name — 18 `ipam.*` keys, 13 `.ipam*` rules, the template, 33 references in
  `example_screens.rs`, and `main.rs`'s `/ipam?subnet=` route test.
- [x] **T3** (AC2, AC3, AC4) The store-fed grid: the cell vocabulary, the policy treatments, `free`'s
  own treatment, `.0`/`.255` as derived `infrastructure`, the occupancy list, the next-offerable
  panel, and the subnet selector keyed on the id rather than on a slug that no longer exists.
- [x] **T4** (AC8) The empty-plan sentence, through `Gesture::Planned { owner }`.
- [x] **T5** (AC9) Narrow `#![allow(dead_code)]` to what this story leaves dead, and say what remains.
- [x] **T6** (§2.3) Replace the legend guard that dies with the dataset, and cover the one modifier
  no legend entry can carry — `not-covered`, which has none by decision.
- [x] **T7** (§2.4) Widen `a11y/seed.sql` with a plan, and give the axe gate an `/ipam` state with a
  FLOOR — without one it would walk an empty page and report success.
- [x] **T8** (AC11, AC12) Measure. Both browser gates ARE claimed: this story changes a screen.

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

- New: a write-route module on `document.rs`'s shape; a store-fed ipam view module. ⚠️ **`page.rs`
  is at 1954 code lines of the 2000 ceiling — 46 of headroom** (its first `#[cfg(test)]` is at 1955,
  which is how the gate counts; `cargo xtask ci` reports `largest: 1954`). The story first wrote
  *2033*, which is story 6.4's PRE-SPLIT figure — a number that was true before a split and false
  after it. The view code does not fit, and the answer is the split `CLAUDE.md` prescribes, never
  shorter prose.
- Deleted: the ipam half of `example_data.rs`, `_ipam_example.html`, `ipam.state.structural`,
  `.ipam-cell-structural`.
- Touched: `screens.rs` (nature, router skip), `main.rs` (route merge, the partition guard's
  counts), `app.yml`, `app.css`, `a11y/seed.sql`, `xtask` (the vocabulary denylist).
- **Not touched**: `epics.md`, the UX spec — a story may not edit either.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Epic 14` — the six constraints, 14.2's five ACs]
- [Source: `_bmad-output/planning-artifacts/prd.md#The PLAN axis` — the binding vocabulary]
- [Source: `_bmad-output/planning-artifacts/ux-design-specification.md:1632-1634` — the grid's a11y
  shape; `role="grid"` is on :1632 and *"jump to next free IP"* on :1634]
- [Source: `crates/opencmdb-bin/src/document.rs` — the write-route machinery to reuse]
- [Source: `crates/opencmdb-bin/src/ipam_repo.rs:324-330` — the seam that names this story]
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md` — the two rows owned here]

## §4 — Mutation record (predictions written FIRST)

| # | mutation | predicted | measured | carrier |
|---|---|---|---|---|
| Q1 | assert the `Ipam` variant before it exists | compile failure | ✅ `E0599` — **and only under `--all-targets`**; a plain `cargo build` printed nothing | compiler |
| Q2 | `offerable` as `matches!(self, Self::Free(_))` | *(shipped as the first draft)* | 🔴 RED on the first run, `left: Some(192.0.2.0)` — the network address offered | assertion |
| Q3 | `/ipam` becomes `Fed` with no budget | green, per the story's first reading | 🔴 RED, *"must refuse rather than hang"* — the guard is DERIVED from `Screen::ALL` | assertion |
| Q4 | delete `ExampleContent::IpamOccupancy` | 3 sites named | ✅ 3 × `E0599`, one only under `--all-targets`; `example_contents.len()` is NOT among them | compiler |
| Q5 | remove `ScreenQuery.subnet` | 3 sites named | ✅ 3 × `E0560`, each an explicit field list — the house idiom against `..Default::default()` paying for itself | compiler |
| Q6 | `.ipam-cell-free` back to `background: transparent` | the axe gate reds | ✅ **exit 1**, naming *"free and not-covered render IDENTICALLY"* — ⚠️ and **axe itself reported 0 violations** under the same mutation: this check is the sole carrier | browser gate |
| Q7 | the a11y seed on a `/24` | green | 🔴 **FOUR tests RED**, `the subnet: Constraint("unique")` — the seed took a CIDR the tests own. ⚠️ CI could never see it: its seed step runs AFTER the tests | assertion |
| Q8 | `AXE_REQUIRE_PLAN=1` against an unseeded store | exit 2 | ✅ exit 2 — measured incidentally when the gesture row was consumed: the gate refuses rather than passes | gate contract |

🔑 **Q2 and Q7 are the pass's own findings, and neither was in any analysis.** Q2 is the argument for
two axes rather than one flattened enum, made by a red rather than by prose. Q7 is a defect this
story INTRODUCED and the measurement caught — *a fixture and a test that share a namespace are one
collision apart, and the store does not forget between them.*

## §5 — Registered rather than fixed

- ⚠️ **The UX spec's `role="grid"` divergence** (`:1632-1634`): the spec asks for a two-dimensional
  focus model with `role="gridcell"`; the grid ships as story 6b.7's `<ul role="list">`, whose ARIA
  reason still holds, and the spec's real requirement — *find a free IP without sight* — is met by
  the next-address panel in text. **Owner: Epic 14's retrospective.**
- ⚠️ **`#![allow(dead_code)]` is narrowed, not removed**: eleven warnings before this story, SEVEN
  after, each naming story 14.2b. It goes when the write path has a producer.
- ⚠️ **`page.rs` is still at 1954 of 2000** — this story added no line to it, by putting `/ipam` on
  its own router. 14.2b's routes must not undo that.

## §6 — The three-layer code review, and what it changed

Three isolated layers, 2026-09-11/12. ⚠️ **All three ran at this session's capability — isolated by
context and by worktree, NOT on a different model, and that half is not claimed** (story 6.5's own
wording). **40 raw findings; 5 arbitration-grade, 2 sent to Guy and re-arbitrated the same day.**

### 🔴 The screen was a DENIAL OF SERVICE, and the page budget could not see it

A `10.0.0.0/8` in the plan made `/ipam` serve **2.08 GB in 44 s with status 200**; a `/16` — an
ordinary corporate subnet — shipped 8.1 MB and 65 536 `<li>`. ⚠️ **On the PLAIN navigation link**,
the default view being the numerically-lowest subnet. Forty concurrent requests took `/healthz`
from 1 ms to **10.1 s** and the process to 2.6 GB.

🔑 **`store_within` wraps the READS; `PlanView::derive` and `render_plan` are synchronous**, so
`tokio::time::timeout` had nothing to preempt. *A budget that bounds the wrong half of a handler is
a budget that reads as coverage and is none* — this epic's dominant class, met at the level of a
whole request rather than of a guard.

✅ **Guy, 2026-09-12**: beyond `MAX_DRAWN_ADDRESSES` (1024, a `/22`) the grid is not built AT ALL and
the plan is shown as the **ranges the operator declared** — what the plan actually holds; the grid is
what does not scale. Refused: a bare refusal sentence (an operator with a `/16` would get nothing
while the product holds their ranges) and paginating by `/24` (a pagination to design, to make
keyboard-accessible and to walk with both gates — a story of its own). **The ceiling is tested
BEFORE the cells are materialised**, and the boundary is asserted on both sides so the VALUE is
pinned and not merely its existence. Measured after: **2 295 bytes in 8.9 ms**.

### 🔴 One schema-legal row removed the whole screen, healthy subnets included

`0007` admits `prefix_len` up to 128 (IPv6 forward-compat) and cannot check that a base is its own
network address — both are adapter rules. So one row with `prefix_len = 64` made `list_subnets`
return `Err` and `/ipam` answer **500 for every subnet**. Three aggravations, each measured: the log
named **no id**; the 500 body read *"the data behind it is intact; the fault is in the display"*,
which is backwards here; and it pointed at `/diagnostic`, which keeps no error history. *A row
nobody can find, breaking a page nobody can read.* The row is now SKIPPED and NAMED, with the trade
written: a silent skip would hide a real defect, so it is a `warn` carrying the id and the stored
spelling.

### 🔴 Two of my own guards passed over a broken product, and the reasons are worth more than the fixes

- **The stylesheet guard was satisfied by its own COMMENT.** `app.css`'s paragraph narrating the
  free-versus-blank defect contains `.ipam-cell-free`, so deleting the RULE left 875 tests, clippy
  and ten gates green — measured with `cargo xtask mutate`. 🔑 *A guard that greps a file greps its
  prose too, and the better the prose explains the defect, the more reliably it hides it.* Comments
  are stripped now — ⚠️ **and a badly-chosen mutation of mine then exposed a second hole**: I renamed
  the rule instead of deleting it, `contains` accepted the PREFIX, and the guard stayed green for a
  different reason. *A mutation named for one thing and applied to another sometimes measures a
  third.* The oracle now knows where a selector ends; both mutations red.
- **The AC2 guard was blind to a CALL.** It forbade three table-name literals;
  `crate::repo::count_observations` carries none, so an `/ipam` hitting `observation_record` on
  every request was green under ten gates. It now allows exactly ONE item from `crate::repo` —
  `classify`, this crate's single backend-error translation — stated as an allowlist rather than an
  exception nobody can audit.
- 🔑 **And its earlier widening had been worthless for a reason that is the story's best sentence**:
  it split on `#[cfg(test)]`, whose first occurrence in `ipam_repo.rs` is **line 7, inside the
  sentence explaining that the `file-size` gate stops at the first one and therefore reads 183 lines
  of `repo.rs` where 1743 are**. It read **382 bytes of 47 324**. *The defect the file documents,
  committed by the guard written while reading that documentation* — and found only because a
  mutation that should have reddened came back green and was disbelieved.

### 🔴 `structural` was in no denylist, and the task was ticked

`git diff master...HEAD --stat -- xtask/` was **empty**. The Project Structure Notes listed `xtask`
under *Touched*, T3 was `[x]`, and `prd.md:1055` says in writing that this story is the one that adds
it. Added to three columns (`<key>`, `en`, `fr`), proved red — reintroducing the retired key reds the
gate naming `app.yml:810`. ⚠️ **And `cargo fmt` then broke a neighbouring guard**: lengthening the
French column pushed it past rustfmt's width, the formatter split it, and
`the_two_carriers_agree_on_what_is_retired`'s `("fr",` anchor vanished. It PANICKED rather than
passing — the right failure — but *a guard an automatic formatter can blind depends on something
nobody is deciding*, and the anchor is now the literal.

### 🔴 `next_offerable` offered the network address, and the story's own test demanded it

Found by the blind layer from the diff alone, reproduced by the edge layer through the sanctioned
`insert_range`. `derive` reached `is_edge` only in the `else` branch, so any range over `.0` made it
`Free(_)` and offerable. 🔑 **My first fix that morning had named a POLICY; the defect was an
ORDER** — the edge is decided before any range now, and carries the declared policy so the plan is
drawn as written while only the OFFER is refused. ⚠️ The guard asserted `offerable == 256` on a
`/24`: *a test that pins the ugly thing is a test that demands it*, and this one demanded it for a
day, in a file whose header quotes that exact trap.

### ✅ The policy axis is a STATED LIMIT (Guy, 2026-09-12)

`border-style: double` at `border-width: 1px` collapses to a solid line, so `static` and
`infrastructure` came back **IDENTICAL PIXELS** in Chrome 151. Not *"told apart by colour alone"*,
which 1.4.1 would already forbid — not told apart. 🔑 **The channel is exhausted**: a 14 px cell
offers three border styles that render at 1 px and the fill is spoken for by the four STATES. So the
grid separates the STATES, the policy is carried by each cell's accessible name and by the legend
— constraint 6 in its own terms, *a pattern and a word, never a hue alone* — and the limit is
written in the stylesheet with a test that reds if the sentence goes. Refused: a thicker border for
one policy (the cell changes apparent size), and composing state × policy into one fill (16 patterns
at 14 px, which would need measuring rather than hoping).

### ⚠️ Also repaired, each from a layer's measurement

The empty plan served `/triage`'s sentence — *"resolve an ambiguity, accept a gap, snooze, attach"*,
four gestures unrelated to an addressing plan — and now has its own key plus `aria-describedby`, so
it is announced once as well as seen once; its verb is **Define**, not *Declare*, because `prd.md:992`
binds `declared` to a STATE. `IpamQuery::subnet`'s doc promised the fallback the handler refuses. A
comment said FIFTH over a list of five predecessors — ⚠️ **and two layers gave two numbers here**
(the validation said fifth, the blind layer sixth); **the tie was broken by counting, not by
seniority**: `triage_router` carries five screens, so `/ipam` is the sixth. Two modifiers, not one,
lacked a legend entry. AC9's *SEVEN* was six. The File List's *eleven guards* was twelve, *five
items* was six, *33 references* was 41. And **§5 said "registered" over a `deferred-work.md`
identical to master** — story 6b.9's finding verbatim; five rows are written now and two inherited
rows re-owned to 14.2b.

### ✅ Refuted by the layers, with the check — so nobody re-chases them

Fifteen suspicions were RUN and refuted by the edge layer: hostile `?subnet=` (15 probes — all 200,
**nothing reflected**), duplicate `?subnet=` (**400** from axum's own extractor), XSS through a label
or an id (escaped, `grep -c "<script>alert"` → 0), the budget on store failure (**500 in 5.004 s**),
a mid-page failure (**500 in 5.002 s**), poisoned policy tokens and addresses (**all refused by the
DDL**), `Subnet::addresses()` panicking (`/0`, `/31`, `/32`, `/129` — none), `AXE_REQUIRE_PLAN`
satisfied by something that is not a grid (**exit 2**, the contract holds), and axe over five `/ipam`
states including the first-boot empty plan (**0 violation nodes on all five**). ⚠️ Three of the
eight guard mutations reddened **compiler-carried** rather than on an assertion, and that is named
per row rather than folded into a headline.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Agent Model Used

Claude Opus 5 (1M context)

### Completion Notes List

- **AC1** `/ipam` is `Nature::Fed`, on its own pool-bearing router; `ExampleContent::IpamOccupancy`,
  `_ipam_example.html` and `example_data`'s whole ipam half are DELETED. The compiler named three
  sites; 18 keys, 13 CSS rules and 33 references were swept by hand because it could not.
- **AC2** the grid reads `ip_subnet`, `ip_range` and `ip_address` and nothing else. The guard is a
  SOURCE scan (`the_plan_reads_no_observation`), because the defect is an ADDED read and *you cannot
  measure the absence of code by running code*.
- **AC3** four policies × four line styles, four states × four fills, every pair asserted distinct
  as a SET rather than counted. ⚠️ **`free` gained its own treatment**, and the browser says the pair
  now differs in `background-image` where before it differed in nothing.
- **AC4** no `ipam.state.structural`, no `.ipam-cell-structural`, no `CellState::Structural`;
  `.0`/`.255` render as `infrastructure` **through the POLICY key**, because a second word for one
  concept is the synonym problem the binding table exists to prevent.
- **AC5 · AC5b · AC6 · AC7** → story 14.2b, with their measurements.
- **AC8** an empty plan names the gesture as NOT YET BUILT, through the badge `Gesture::Planned`
  renders — 14.2b cannot ship its route without meeting that site.
- **AC9** `#![allow(dead_code)]` narrowed from module-wide to item-by-item: **eleven warnings
  before, SIX after** — one per attribute, each naming 14.2b. ⚠️ **This read *SEVEN* in four places
  until the acceptance layer recounted**, in the criterion whose own subject is a wrong count of
  this very figure. *A unit-sensitive sentence is where an unqualified number does the most damage.* 🔴 The register said *eleven items* and a correction of it said
  *ten warnings covering fourteen*; both were wrong, and the error was the INSTRUMENT (`grep "never
  used"` cannot see ``struct `Subnet` is never constructed``). ⚠️ **PR #171 carries the correction to the twins and the register and is OPEN, not merged** — the criterion first said *corrected before this story starts*, which was false while that PR waited.
- **AC10** `ipam/mod.rs`'s false `Constraint` doc corrected, **and the variant that makes the
  corrected sentence true shipped with it** rather than leaving a doc describing future code.
- **AC11** THE LIVE COUNT: **873 → 877 tests** (587 bin + 191 core + 99 xtask, after the code
  review's two new guards), the sum re-added
  rather than recalled. `cargo test --workspace --locked`, wall clock, warm: **8.69 s** against a
  live `mariadb:10.11.11`, **5.61 s** with `DATABASE_URL` unset.
  ✅ **The 5.6 s is NOT a store leaking in, and that is settled by a check rather than by a story**:
  there is no `.env`, `DATABASE_URL` is absent from the environment, and `--skip budget` runs in
  **0.74 s** while the three `budget` tests alone take **5.15 s**. Story 14.1's §6 recorded the same
  floor. *The clock is a weaker tell than this project's notes claim.*
- **AC12** ten `cargo xtask ci` gates ✅ · `clippy --workspace --all-targets` ✅ (zero warnings) ·
  `RUSTFLAGS="-D warnings" cargo test` ✅ · `fmt --check` ✅. **BOTH BROWSER GATES ARE CLAIMED AND
  WERE RUN**: axe **0 violation nodes** over 10 routes + 4 states with `AXE_REQUIRE_PLAN=1`,
  `AXE_REQUIRE_QUEUE=1` and `AXE_REQUIRE_GESTURE=1`; kbd-probe **30 checks, 0 failed**.

### File List

- `crates/opencmdb-bin/src/ipam_page.rs` — NEW (525 code lines): the cell vocabulary, the pure
  `PlanView`, the router, the budgeted handler, the render, and TWELVE guards.
- `crates/opencmdb-bin/templates/_ipam.html` — NEW.
- `crates/opencmdb-bin/templates/_ipam_example.html` — DELETED.
- `crates/opencmdb-bin/src/ipam_repo.rs` — `list_subnets`, `ranges_in`, `policy_from_token`,
  `Subnet::addresses`/`is_edge`/`cidr`; `ipam()` now produces the named variant; the blanket
  `allow` narrowed to SIX items.
- `crates/opencmdb-core/src/repo/mod.rs` — `RepositoryError::Ipam(IpamError)`.
- `crates/opencmdb-core/src/ipam/mod.rs` — the false `Constraint` paragraph corrected.
- `crates/opencmdb-bin/src/screens.rs` — `Ipam` is `Fed`; `ExampleContent::IpamOccupancy` deleted.
- `crates/opencmdb-bin/src/main.rs` — the ipam router merged; the partition count and its sentence;
  the selector test rewritten against the store.
- `crates/opencmdb-bin/src/page.rs` — `store_within` is `pub(crate)`; the attribute-scan floor.
- `crates/opencmdb-bin/src/example_data.rs`, `example_screens.rs` — the ipam halves deleted.
- `crates/opencmdb-bin/locales/app.yml` — the ipam block rewritten: 20 keys, both locales.
- `crates/opencmdb-bin/assets/app.css` — the cell vocabulary; `.panel-caveat`, `.ipam-planned-note`.
- `a11y/seed.sql` — a plan on two `/25`s, and the plan's tables truncated in dependency order.
- `a11y/axe-gate.mjs` — the `/ipam` state, its floor, and the browser-only distinguishability check.
- `.github/workflows/ci.yml` — `AXE_REQUIRE_PLAN=1`.

### Change Log

- 2026-09-11 — contexted; the four arbitrations TAKEN by Guy the same day (the recommendation in
  all four); then the mandatory validation ran both layers. **The FACT-CHECK layer refuted SEVEN
  claims of the story's own, four of them numbers.** **The GAP-HUNT layer, which builds, REFUTED TWO
  OF THE FOUR ARBITRATIONS and re-scoped a third** — (C)'s stated reason for refusing its
  alternative was wrong and one ordinary DRY line defeats the chosen fix; (D) is unbuildable because
  `free` already IS the blank cell, measured in a browser; (B)'s SET was scoped to a type that does
  not contain three of the refusals the operator meets first. All four went back to Guy and were
  re-arbitrated the same day. **And it found that nothing in this product can create a subnet**, so
  AC5 became three routes. Every correction is recorded in place with what it replaced. Three defects found while contexting and carried as criteria rather than filed
  elsewhere: the false `Constraint` doc, the register's wrong dead-code figure, the UX-spec grid
  divergence. No code changed.
- 2026-09-11 — implemented 14.2 after the split. Eight mutation ids, eight conforming outcomes,
  **two of them the pass's own findings**: `offerable` offered the network address inside a declared
  `infrastructure` range (which is the argument for two axes rather than one enum, made by a red),
  and the a11y seed took a CIDR the tests own — four tests red, on a defect CI could never have seen
  because its seed step runs after the tests.
