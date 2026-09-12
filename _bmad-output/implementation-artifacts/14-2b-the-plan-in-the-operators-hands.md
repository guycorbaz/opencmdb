# Story 14.2b: The plan in the operator's hands

Status: ready-for-dev

⚠️ **`ready-for-dev` is the workflow's status, not a statement that nothing is open.** §1 poses
decisions that are Guy's, and the mandatory validation (two fresh-context agents) has not run.

Epic 14 (IPAM). 🔴 **INSERTED 2026-09-11 at story 14.2's implementation** — `epics.md` describes
FOUR stories and there are FIVE; a story may not edit it, and the divergence is registered with
Epic 14's retrospective. Baseline: `354283b` (master), **877 tests** (587 bin + 191 core + 99
xtask), ten gates.

## Story

As the operator,
I want to define a subnet, a range and an address through the product,
so that the plan it draws is one I can actually fill.

🔑 **Story 14.2 drew the plan; nothing can put anything in it.** `/ipam` today says the gesture is
not yet built, which was honest and is not a product. This story is the gesture.

## §0 — What is settled, and must not be re-opened

**The vocabulary is RATIFIED** (PR #166): `static` · `dhcp-pool` · `reserved` · `infrastructure`.
No story may extend it. `structural` is retired in the code AND in the `copy-vocabulary` gate's
denylist since 14.2 — reintroducing the key reds the gate, which names whatever line you put it on
(⚠️ the story first said `app.yml:810`, which was the retired key's own address before 14.2 deleted
it: *a line number is not a property*).

**Guy's six arbitrations** (2026-09-10) are `epics.md`'s Epic 14 preamble. Constraint (1) is this
story's shape: *the write cost is front-loaded — the FIRST new route carries the shared machinery
and the rest are cheap*, which is why this story ships three routes and not one.

**Guy's arbitration (A)** (2026-09-11): **NO switch on the IPAM routes.**
`OPENCMDB_DOCUMENT_ENABLED` guards an AUTHORSHIP hazard — a machine writing as a human,
`origin='manual'`/`actor_id='operator'`, NFR5's whole subject — that IPAM does not have: the
operator's own plan, in a register `declared_attribute` never touches (arbitration 4). ⚠️ **The
accepted cost is owed to the release notes**: a fresh install gains a live write surface with no
opt-in, the FIRST in this product.

**14.2 shipped the read path and narrowed the debt rather than paying it.** `RepositoryError::Ipam`
exists; `list_subnets`, `ranges_in`, `addresses_in` have a producer; six `#[allow(dead_code)]`
remain, one per write-path item, each naming this story.

## §1 — What this story inherits, each WITH the measurement that produced it

🔑 **None of these is an intention. Each was built and measured at 14.2's validation or code
review, and the measurement is what makes it actionable.**

### (a) THREE write routes, not two

`insert_subnet` has **NO PRODUCTION CALLER**, and `ip_range.subnet_id` / `ip_address.subnet_id`
are both `NOT NULL` foreign keys to it (`0007:120`, `:153`). With two routes an empty plan stays
empty for ever and every range write answers `Err(NotFound)` — measured, twice.

🔴 **This read *"no caller outside its own test module"* and story 14.2 itself had falsified it**:
`main.rs:1854` calls `insert_subnet` from `main.rs`'s own test module, added by 14.2's rewritten
selector test. The SUBSTANCE survives — nothing in production calls it — and the sentence did not.
*An inherited measurement is dated by the tree that produced it, and the tree moved underneath this
one.*

### (b) 🔴 The auth perimeter must be a PROPERTY, and today it is a literal

The gap-hunt mounted an always-on `POST /ipam/range` BELOW `auth_deny` and measured **873 tests and
TEN GATES GREEN over an unauthenticated write answering `201 Created`** — ⚠️ **873 is 14.2's
baseline at `38da035`, not this story's 877**, and it is dated here because a reader re-running it
on `354283b` will not reproduce the figure. The test carrying the
*route-above-the-layer* property is hardcoded to `/document-all` (`main.rs:2204`, re-derived), and
the perimeter guard iterates `Screen::ALL` (`main.rs:1586`, re-derived) — **a POST path is in
neither**, `Screen::href` yielding GET paths only. ⚠️ *"The ONE test"* is loose: the perimeter guard
catches the same property for GET screens, as its own doc records. The conclusion is unaffected. This is story 6b.2's
measured defect (`GET /dashboard` → 200 below the layer) in its POST form.

**What it owes**: a guard over EVERY route the router carries, derived and not listed.

### (c) 🔴 The keyed refusals are over what the HANDLER CAN RECEIVE, not over `IpamError`

Measured through the very adapters these routes call:

| gesture | what comes back | an `IpamError`? |
|---|---|---|
| a range in a subnet that does not exist | `Err(NotFound)` | **no** |
| the same CIDR defined twice | `Err(Constraint("unique"))` | **no** |
| **the same address defined twice** | `Err(Constraint("unique"))` | **no** |
| the loser of a lock wait, once (d) lands | `Err(Backend("…**1213**…"))` | **no** |
| **a label over 120 characters** | `Err(Backend("…1406: Data too long…"))` | **no** |

⚠️ **The story first wrote 1205 for row four; measured, it is 1213** — a deadlock, not a timeout.
And **row five was missing entirely**: `label` is `VARCHAR(120)` on all three tables, so an operator
typing a long name gets a driver sentence — ⚠️ and under a non-strict `sql_mode` the value would be
silently TRUNCATED instead, which is worse than a refusal.

🔑 *A SET over `IpamError::ALL` is correct about `IpamError` and silent about the four refusals the
operator meets first* — and the second row is **the likeliest refusal on this screen**, an operator
retyping an address. 🔴 **And *"carried by the SET test or by nothing"* is a FALSE DICHOTOMY, measured.** The gap-hunt
wrote the handler's mapping as an exhaustive `match &RepositoryError` with no `_` arm and then added
a variant: **`error[E0004]: non-exhaustive patterns`**. One design line decides whether the compiler
carries it. ⚠️ **The half that DOES hold**: `Constraint(&'static str)` forces a `_` inside, so a new
constraint NAME stays invisible to the compiler. **So AC4 owes both** — the exhaustive match for the
variants, and the SET test for the constraint names.

### (d) 🔴 The concurrency fix goes on the read that DECIDES, and the parent-row lock is defeated by one DRY line

`insert_range` reads every sibling (`ipam_repo.rs:300`, re-derived — the story first cited `:238`,
which is that line's address on 14.2's baseline and is `.bind(label)` today), decides in Rust, then
inserts — no transaction, no lock. With a 400 ms pause injected, **two overlapping ranges both committed, both
reporting success.**

Guy's arbitration (C) of 2026-09-11, re-aimed once already, was **RE-ARBITRATED A THIRD TIME on
2026-09-12 — and this time by a measurement that refuted the shape itself.**

🔴 **`SELECT … FROM ip_range … FOR UPDATE` alone does not refuse: it DEADLOCKS.** The gap-hunt built
five locking strategies and ran the matrix five times, in both sibling states:

| strategy | loser gets | rows |
|---|---|---|
| no lock | `Ok` | **2** |
| parent row `FOR UPDATE` | `Ipam(RangeOverlapsAnother)` | 1 |
| parent row + an early `load_subnet` (the DRY line) | `Ok` | **2** |
| **the deciding read `FOR UPDATE`** | **`Backend("…1213 (40001): Deadlock found…")`** | 1 |
| **parent row THEN the deciding read** | **`Ipam(RangeOverlapsAnother)`** | 1 |

**20 of 20 observations of the prescribed lock deadlocked.** `WHERE subnet_id = ?` is a non-unique
index scan, so both transactions take compatible gap locks and then each needs an insert-intention
lock. Correctness survives — one row every time — but the loser is told *deadlock, retry* where the
product knows the honest answer, and a raw driver sentence on a form field is what AC4 forbids.

✅ **Guy, 2026-09-12: lock the PARENT ROW first, THEN the deciding read.** Deterministic refusal,
5/5, in both sibling states, no deadlock. ⚠️ **This is the option refused on 2026-09-11 as
belt-and-braces, and the measurement overturned the refusal** — the two locks are not redundant: the
parent row serialises entry, and the range lock is what the DRY line cannot walk past.

⚠️ **AC5: the pause harness ships as a PERMANENT test**, and it must assert the REFUSAL, not merely
that one row survives — all four locking strategies leave one row, and three of them tell the
operator something false.

### (e) 🔴 `RepositoryError::Contention` is DEAD CODE, and this story is what makes it reachable

`classify` (`repo.rs:1613-1614`) matches `Some("1213") | Some("1205") => Contention`. It never
fires: **sqlx's `code()` returns the SQLSTATE**, and the MySQL number lives in a separate `number`
field. Measured on two errors — a lock-wait timeout arrives as `code() = Some("HY000"), number =
1205`, a duplicate key as `code() = Some("23000"), number = 1062`.

`Contention` has one producer site, no test and no consumer — **5 mentions, THREE of them prose**
(`repo/mod.rs:6`, `:21`, `:103`) and two of them code: the variant's own declaration (`:27`) and the
producer (`repo.rs:1614`). ⚠️ The story first wrote *four prose*, which counted the declaration as
narration.
⚠️ **Pre-existing and NOT this story's to have caused — but (d) is what first makes it reachable on
a route an operator presses**, and it would surface as a raw English driver sentence, which is
exactly what (c) forbids. **Decide here: repair `classify` or map the case in the handler, and say
which.** A register row is not enough when this story's own arbitration opens the path.

### (f) The last six `#![allow(dead_code)]`

`canonical`, `Subnet::contains`, `insert_subnet`, `load_subnet`, `insert_range`, `insert_address`
— at `ipam_repo.rs:67, 160, 212, 250, 278, 335`. Eleven warnings before 14.2, **six after**, both
re-measured by removing them. They go when these routes call them.

🔴 **And the spelling matters, because AC7 was satisfiable by doing NOTHING.** There is no
`#![allow(dead_code)]` in `ipam_repo.rs` at all — 14.2 removed the blanket one and left six
ITEM-level `#[allow(dead_code, reason = …)]`. AC7 said *"`#![allow(dead_code)]` is GONE"*, which the
committed tree already satisfies. *A criterion a tick can satisfy is a criterion that measures
nothing*, and this project has shipped that defect before (story 6b.8's ticked task that delivered
nothing).

## §2 — Decisions this story must take, and which are Guy's

✅ **ALL FIVE TAKEN, 2026-09-12.** Item 1 by precedent and stated rather than asked; items 2–5 are
**Guy's**, each recorded with the option refused and the cost accepted, so that none is re-opened in
silence by a dev agent meeting it mid-implementation.

1. ✅ **The form's shape and its mount point — settled by precedent, not asked.** `document.rs` is
   the model: a sub-router whose state holds a PORT and no pool, so `State<MySqlPool>` in the
   handler **fails to compile** (story 6.1's M4, re-measured by two review layers). ⚠️ `/ipam` is
   now on its own pool-bearing router (`ipam_page::router`), so the two must not be fused by
   accident — AC2 is what measures that they were not.
2. ✅ **Where the form lives on screen — IN LINE IN THE RAIL, COLLAPSED** (Guy, 2026-09-12). Three
   `<details>` in `ipam-rail`; the empty-plan branch links to the subnet one. **Refused: a dedicated
   address** (`/ipam/nouveau`) — an eleventh address outside `Screen::ALL`, whose cost this project
   has measured at story 6b.6 on `/devices/{id}` (the nature dispatch, the route-table partition and
   the navigation all have to be told about it), plus one round trip on a gesture the operator
   repeats; and **refused: always visible**, three permanent forms above a 256-cell grid on the one
   screen the axe gate walks under `AXE_REQUIRE_PLAN=1`.
   ⚠️ **THE ACCEPTED COST, WRITTEN RATHER THAN DISCOVERED: `epics.md`'s criterion 5 says the empty
   plan "names the gesture that fills it AND LINKS TO IT", and what this ships is an ANCHOR, not an
   href to a page.** It is a real link — `href="#…"` onto a `<details>` the browser must open — and
   it is NOT the same object the criterion's wording suggests. Say so at the site; do not let a tick
   stand for it. ⚠️ And `<details>` is a disclosure the anchor must OPEN: an anchor onto a collapsed
   element scrolls to a closed box, which is *a door that opens onto a door*. The `open` attribute,
   or a `:target` rule, is part of this decision and not an implementation detail.
3. ✅ **Who mints the id — THE SERVER, `Uuid::now_v7()`, AND THE NIL IS REFUSED AT THE ROUTE**
   (Guy, 2026-09-12), exactly `document.rs`'s two halves: `:120` mints v7 for the record being
   created, `:195` refuses a nil id ARRIVING from the client. Both halves are live here, and that is
   what the question was really about — a range and an address create a record (minted) **and**
   carry a `subnet_id` chosen in the browser (refused if nil). **Refused: a client-supplied id**,
   which puts the nil, the duplicate and the guessed id on the outside of the product; and
   **refused: refusing the nil in the adapter too** (`insert_device`'s shape), whose cost is three
   more refusals to test and a redundancy this codebase requires to be LABELLED deliberate or a DRY
   pass collapses it.
   ⚠️ **THE ACCEPTED COST: the adapter still accepts the nil**, measured at 14.2 on all three
   `insert_*`. So this is a TRIPWIRE at the route (story 5.12's precedent), never a barrier — write
   it as one, and do not let AC1 read as *the nil cannot be stored*.
4. ✅ **A label is REQUIRED, and refused AT THE ROUTE** (Guy, 2026-09-12): non-empty after `trim`, a
   keyed refusal in both locales, beside the over-120 refusal this story already carries — ⚠️ under
   a non-strict `sql_mode` a 121-character label TRUNCATES in silence, which is why that one is a
   refusal and not a lint. **Refused: optional**, which is a plan of numbers with no words on the
   screen whose whole subject is what was MEANT to be there (`_ipam.html` already renders the empty
   label away: `{% if row.label != "" %}`). **Refused: a `CHECK` in a migration `0008`** — a real
   barrier, at the price of an `ALTER` at boot on a published product, which is what story 6.5
   refused for `entity.state`.
   ⚠️ **THE ACCEPTED COST: `label` stays `NOT NULL` with no non-empty CHECK, so `""` still inserts
   cleanly through raw SQL** (measured at 14.1). Second tripwire of this story; the barrier is the
   same one story 5.12 registered — a privilege, not a matcher.
5. ✅ **An address inside an existing range is ACCEPTED, SILENTLY, HERE — and the warning belongs to
   14.3** (Guy, 2026-09-12). Measured at 14.2: the adapter accepts it in BOTH orders and the screen
   renders it as `Defined` carrying the range's policy. **Refused: refusing it** — `epics.md:2433`
   refused the refusal in the neighbouring case, on the ground that the most frequent LEGITIMATE
   gesture is entering what is already there, and a static reservation inside a `dhcp-pool` is
   ordinary and correct. **Refused: warning HERE**, which would build a second warning mechanism in
   the story of the write while 14.3 is the story that owns them all.
   🔑 **The two axes must not be confused, and that is why this was asked separately.**
   `epics.md:2433` settles PLAN-against-NETWORK (an observed address the audit highlights): *warn,
   name what is known, and still write*. This settles PLAN-against-PLAN, which no arbitration
   covered. ⚠️ **THE ACCEPTED COST: between this story and 14.3, a reservation inside a DHCP pool is
   written without a word** — legal, and not necessarily what the operator meant. Registered with
   14.3 by name, not left to be rediscovered.

## Acceptance Criteria

**AC1 — three write routes, and the FIRST carries the machinery.** Define a subnet, a range and an
address. The first carries: the Origin check (story 6.2), a KEYED refusal body per status in BOTH
locales, the `authorship` sanction if it writes provenance (it does not — say so), and both browser
gates. The others reuse it, and a TEST asserts the reuse rather than a comment claiming it.

**AC2 — the handler cannot extract `State<MySqlPool>`.** The sub-router's state holds a port. A
mutation adding the pool must fail to COMPILE, measured and recorded as `E0277`/`E0308`.

**AC3 — every route the router carries answers 401 without a credential, AND EXISTS.** ✅ Guy,
2026-09-12, on two measurements that refuted the criterion as first written:

🔴 **The shape it named does not exist.** axum 0.8's `Router` exposes `has_routes()` and nothing
else; `PathRouter` is `pub(crate)`. **A router cannot be enumerated.** And a source scan is
incomplete by construction: three of the fourteen `.route(` sites take a non-literal path
(`screens.rs:406`, `:424`, `document.rs:80`).

🔴 **And the guard as written PASSES OVER A ROUTE THAT DOES NOT EXIST.** `auth_deny` layers the
fallback, so 401 proves nothing about existence — measured: `/totally/made/up` answers 401 without a
credential exactly like a real route, and `/ipam/range` answered **404 with one** while unmounted. *A
misspelt path, an unmounted route and a typo all satisfied it.*

**The shape**: each sub-router declares `PATHS: &[&str]` and its `router()` is BUILT by iterating
that list, so the list and the mounts cannot drift; the guard walks `PATHS` and asserts BOTH halves
— 401 without a credential, and something **other than 404** with one. The positive control is what
makes the negative one mean anything.

**AC4 — every refusal the operator can reach is a KEY, in both locales, naming the rule** — and it
is carried TWICE, because neither carrier covers the other: an **exhaustive `match` with no `_`
arm** over `RepositoryError`, so a new VARIANT is an `E0004`; and a **SET test** over what the
handler can receive, because `Constraint(&'static str)` forces a `_` inside and a new constraint
NAME is invisible to the compiler. ⚠️ Six refusals are reachable today and `IpamError` contains
none of four of them — §1(c). ⚠️ Story 6.4's finding is the reason: a success message shipped in English
under a French UI with a raw UUID in it.

**AC5 — the overlap rule holds under concurrency**, by §1(d), with the lock on the read that
DECIDES — and **the 400 ms pause harness ships as a permanent test**, proven to red when the lock
moves back to the parent row alone.

**AC6 — `Contention` either fires or goes.** §1(e): decide, implement, and say which. If it fires,
a test drives a real lock wait and asserts the variant; if it goes, the arm is deleted rather than
left as a promise the driver cannot keep.

**AC7 — the SIX item-level `#[allow(dead_code, reason = …)]` are GONE from `ipam_repo.rs`**, each
because the item now has a producer, and removing them leaves **zero** warnings under
`clippy --workspace --all-targets`. 🔴 This criterion first said *"`#![allow(dead_code)]` is GONE"*,
which the committed tree already satisfied — 14.2 removed the blanket attribute and left six item
ones. **A criterion a tick can satisfy is a criterion that measures nothing.**

**AC8 — the empty plan LINKS to the gesture** (`epics.md` criterion 5), which 14.2 registered as
undeliverable and this story delivers.

**AC9 — THE LIVE COUNT lives here**, with the command and both conditions named. Baseline **877**
(587 + 191 + 99) at `354283b`. ⚠️ **Use `env -u DATABASE_URL`, never `DATABASE_URL=`**: an EMPTY
value is not an ABSENT one — sqlx tries to connect to it and **118 tests fail**, measured while
writing this story.

**AC9b — the write routes are BUDGETED, and the budget wraps the TRANSACTION.** ✅ Guy, 2026-09-12.
🔴 This story adds the product's **first blocking write**, and measured, the loser's wait tracks the
holder's: a 3 s hold gave a **6.005 s** round trip, bounded only by `innodb_lock_wait_timeout` —
**50 s** on a stock container. ⚠️ `document.rs` contains **zero** `store_within`; every budgeted
file in this product is a GET screen. Story 6b.10 put a per-handler budget on GETs after measuring
a 30 s hang, and this story is where a write can do it. 🔑 **The budget wraps the whole transaction,
not the first read** — story 14.2's denial of service was exactly a budget around the wrong half.

**AC10 — no regression**: ten gates, `clippy --workspace --all-targets -D warnings`,
`RUSTFLAGS="-D warnings"`, fmt, `cargo deny`. **BOTH browser gates ARE claimed**, and each needs work the story first
did not name: 🔴 **`kbd-probe.mjs` has NO `/ipam` coverage at all** — all sixteen of its `open(`
sites name `/triage` — while story 6b.11's focus contract and story 6.4's focus-after-swap contract
both apply to the three forms this story adds. And 🔴 **the axe gate never sees AC8's screen**:
`a11y/seed.sql` always seeds a subnet and `AXE_REQUIRE_PLAN=1` REFUSES a run that draws no cell, so
the empty-plan branch — AC8's whole deliverable — is the one `/ipam` state the gate is configured
never to reach. Both are criteria here, not notes.

## §3 — What the gap-hunt measured that this story must carry

Each was BUILT and run, not reasoned about.

### 🔴 `the_plan_reads_no_observation` is blind to the module this story creates

The guard is a hardcoded two-file list (`ipam_page.rs:1045-1056`). The layer planted, in a new
`ipam_write.rs`, **both** forms it exists to catch — a raw `SELECT COUNT(*) FROM observation_record`
and a `crate::repo::count_observations` call — and measured **the test green and all ten gates
green**. ⚠️ The story's own Dev Notes said the opposite (*"that guard will red if this story reaches
for anything else there"*). **The file list must be DERIVED**, not written: every `.rs` under the
plan's own perimeter, so a new module is covered the day it exists rather than the day someone
remembers.

### ⚠️ The machinery AC1 says to reuse is PRIVATE, and the story does not say where it should live

`same_origin`, `malformed`, `CSRF_REFUSED_BODY` and `refusal_body` are all private in `document.rs`.
*"The others reuse it"* needs either a widening of that file's surface or a shared module — and a
dev agent will settle it silently, **most likely by copying `same_origin`**, which is the one
function a CSRF check must never be duplicated for. **Decide it in T2 and say which.**

### ⚠️ `/ipam` does not use `Gesture`, so the type guarantee will not cover its new controls

`ipam_page.rs` never names `Gesture`; the empty-plan branch rolls its own strings, which 14.2 did
deliberately. `Gesture::Live { route: &'static str }` exists so that *a live gesture posting nowhere
is unrepresentable* (story 6b.4b's arbitration, made a compile obligation). AC8 makes IPAM the
product's SECOND live gesture. **Adopt the type or say why not** — silence here loses a guarantee
the product already bought.

### ⚠️ A virgin-store `cargo test --workspace` races itself on `migrate!`

Nineteen failures on a freshly created empty database; run one test first, then the same command is
green. Story 6.6's register row requires a dropped database per mutation pass, so **the dev agent
meets this on run one** rather than at the end.

### ✅ Refuted, with the check, so nobody re-chases them

The deadlock is not a gap-lock-on-empty artefact (it reproduces 10/10 with a pre-existing sibling) ·
the `classify` repair does not break the working `Constraint("unique")` path (duplicate CIDR still
`Constraint("unique")`, whole workspace green) · `page.rs` at 1954/2000 does **not** constrain this
story (a separate module costs nothing; `file-size` green at 57 files) · a new write module reds
none of `authorship`, `observed-immutable`, `entity-id-immutable` · and the form does have a key for
the subnet (`_ipam.html:49,79` already carries the store's own id).

## Tasks / Subtasks

- [x] **T0** Take §2's decisions with Guy; §1 is settled and is not re-opened. ✅ 2026-09-12 —
  all five recorded in §2 WITH the option refused and the cost accepted; items 2–5 are Guy's, item 1
  is settled by precedent and stated rather than asked.
- [x] **T1** (AC6) `classify`'s `Contention` arm: repair or remove, with the check either way. ⚠️ The
  repair was BUILT and measured safe (`try_downcast_ref` + `number()`, whole workspace green, the
  `Constraint("unique")` path intact) — **but under the chosen lock it buys less than it looks
  like**, the deadlock becoming `Contention` where the operator needs `RangeOverlapsAnother`.
- [x] **T2** (AC1, AC2) The sub-router and the first route, on `document.rs`'s shape. ✅ 2026-09-12
  — `POST /ipam/subnet` writes, eight mutations conform to predictions written first, and AC2's
  compile refusal is measured `E0277` on the `Handler` bound.
- [x] **T3** (AC4) The refusal set over what the handler can receive, keyed in both locales.
  ✅ 2026-09-12 — both carriers measured: `E0004` for a new VARIANT, and a SET test for a new
  constraint NAME. ⚠️ One prediction CONTRADICTED, and the guard it doubted turned out sound.
- [x] **T4** (AC5, AC9b) The parent row THEN the deciding read; the pause harness as a permanent
  test asserting the REFUSAL and not merely the row count; and the budget around the transaction.
  ✅ 2026-09-12 — 🔴 and the pass found that NEITHER LOCK is observable on its own.
- [ ] **T5** (AC1) The second and third routes, and the test that asserts the reuse.
- [ ] **T6** (AC3) `PATHS` per sub-router, the routers built by iterating it, and the guard with its
  POSITIVE control.
- [ ] **T6b** (§3) Derive `the_plan_reads_no_observation`'s file list; widen `kbd-probe.mjs` to
  `/ipam`; give the axe gate the empty-plan state.
- [ ] **T7** (AC7, AC8) Remove the last allows; link the empty plan to the gesture.
- [ ] **T8** (AC9, AC10) Measure. Both browser gates, both store conditions, the command named.

## Dev Notes

### Traps this project has paid for, and which apply here

- 🔴 **A guard that greps a file greps its prose too** (14.2): the stylesheet guard was satisfied by
  the comment narrating the defect. Strip comments, or assert on what is SERVED.
- 🔴 **A guard that splits on `#[cfg(test)]` may read six lines** (14.2): the first occurrence in
  `ipam_repo.rs` is on line 7, inside a sentence quoting the attribute. Anchor at a line start, and
  assert the guard REACHED what it promises to cover.
- 🔴 **A budget that bounds the wrong half of a handler reads as coverage and is none** (14.2):
  `store_within` wraps the reads; anything synchronous after them is unbounded.
- 🔴 **A test that pins the ugly thing is a test that demands it** (14.2): `offerable == 256`
  required the network address to be on offer.
- ⚠️ **`ascii_bin` is PAD SPACE**; the carrier is an INTEGER comparison, `LENGTH(x) = LENGTH(TRIM(x))`.
- ⚠️ **Store-backed tests take `DB_TEST_LOCK` FIRST and own their own CIDR** — and `a11y/seed.sql`
  owns two `/25`s precisely so it does not collide with the tests' three `/24`s (14.2 measured four
  tests red when it did, a failure CI could never see because its seed step runs after its tests).
- ⚠️ **`cargo fmt` can blind a source-scanning guard** by re-wrapping the list it anchors on (14.2).
- ⚠️ **`app.yml` is invisible to Cargo's incremental build** (6b.9): a mutation editing only the
  locale file measures nothing.

### What the store must answer, and what it must not

**Must**: accept a subnet, a range and an address, refusing what the adapter already refuses.
**Must not**: read `observation_record`, `identity_link` or `declared_attribute` — the audit is
14.3's, and `the_plan_reads_no_observation` now covers `ipam_page.rs` AND `ipam_repo.rs` and allows
exactly one item from `crate::repo` (`classify`). ⚠️ That guard will red if this story reaches for
anything else there — which is the point.

### Project Structure Notes

- New: a write-route module on `document.rs`'s shape. ⚠️ **`page.rs` is at 1954 of 2000** and `ipam_page.rs` at **631** (both computed as the gate
  does, from the first `#[cfg(test)]` at a line start — the story first guessed *~700*, an 11 %
  overstatement on the one figure the split rule is argued from); the rule applies BEFORE the
  growth.
- Touched: `ipam_repo.rs` (the transaction and the lock, the allows), `repo.rs` (`classify`),
  `main.rs` (the merge and the perimeter guard), `app.yml`, `_ipam.html`, `a11y/`.
- **Not touched**: `epics.md`, the UX spec.

### References

- [Source: `epics.md#Epic 14` — the six constraints; ⚠️ it describes FOUR stories and there are five]
- [Source: `14-2-the-plan-on-screen.md` §1, §3, §6 — every measurement this story inherits]
- [Source: `deferred-work.md` — the rows owned here, each re-owned from 14.2 by name]
- [Source: `crates/opencmdb-bin/src/document.rs` — the write-route machinery to reuse]

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context), 2026-09-12.

### Debug Log References

- **T1's prove-to-red carried its own cause.** With the arm still comparing `db.code()`, the failure
  message read `Backend("...1205 (HY000)...")` — the MariaDB number and the SQLSTATE side by side in
  one string, which is the whole defect in one line.

### Completion Notes List

- **T0 — §2's five decisions taken (2026-09-12).** Items 2–5 are Guy's, recorded in §2 with the
  option refused and the cost accepted. Three of the four ship a cost rather than a closure and each
  says so at its site: the empty plan's link is an ANCHOR onto a `<details>` and not an href to a
  page, so `epics.md`'s criterion 5 is met in substance and not in the shape its wording suggests;
  the nil refusal is at the ROUTE while the adapter still accepts it; and the label refusal is at
  the ROUTE while `""` still inserts through raw SQL. **Two tripwires and one divergence, written as
  such** — story 5.12's precedent, which this project has paid for twice.
- **T1 — `RepositoryError::Contention` was dead from the day it was written.** `classify` compared
  `db.code()`, the SQLSTATE, against a MariaDB NUMBER: a lock-wait timeout arrives as `HY000` and a
  deadlock as `40001`, while the number lives in a separate field. One producer, no consumer, no
  test. 🔑 **Why nobody had seen it: nothing in this product CONTENDED** — the identity pass runs
  alone and the screens only read. This story's routes take a row lock, so the loser is an ordinary
  operator. *A dead arm is a promise until something walks into it.*
- **T1 — the Origin check is SHARED, and that is a refusal to copy.** `document.rs`'s machinery is
  private, and the gap-hunt measured that a dev agent building a second write route would most
  likely COPY `same_origin` — the one function a CSRF check must never be duplicated for. It moves
  to `write_guard.rs` with a tripwire asserting the comparison exists in exactly one file.
  ⚠️ **What is NOT shared, by decision: the refusal BODIES.** `document.malformed` and the IPAM
  refusals are different sentences about different gestures; the DECISION is shared, the WORDING
  stays local. A shared body is the *one message for two surfaces* shape story 6.4 paid for.
- **T2 — the plan has a producer, and `POST /ipam/subnet` is the first of the three.** The subnet
  route comes first because the other two hang from it: with two routes an empty plan stays empty
  for ever (§1(a)). `ipam_write.rs` carries the shared machinery epic constraint (1) front-loads —
  the Origin check, keyed refusal bodies in both locales, and the exhaustive mapping of
  `RepositoryError` — and the second and third routes reuse it.
- **T2 — the refusal is DATA, and clippy and the design pointed the same way.** `Refusal { status,
  key }` replaced `Result<_, Response>` after clippy's `result_large_err` refused the latter
  outright (*the `Err`-variant is at least 128 bytes*). 🔑 The lint bought the property AC4 needs:
  the mappers are now PURE, so a test asserts WHICH rule a refusal names instead of scraping a
  rendered body. ⚠️ And the first draft of the distinctness test **restated the whole mapping in its
  own body** — a second table that agrees with itself, story 6.5's M8 one degree over. It reads
  `ipam_refusal(&error).key()` now.
- **T2 — AC2 is MEASURED, not asserted.** `IpamWriteState` holds a port and no pool, so adding
  `State<MySqlPool>` to the handler gives **`error[E0277]`: the trait bound … `Handler<…>` is not
  satisfied** (M8), which is `document.rs`'s M4 reproduced for this module. ⚠️ The sub-router is
  merged as its OWN router and never onto `ipam_page::router`, which BEARS the pool: fusing them
  retires the guarantee silently, and `main.rs` says so at the merge.
- **T2 — the label bound is in CHARACTERS and a test measures the difference.** `VARCHAR(120)`
  counts characters under `utf8mb4`, so a byte bound would refuse 120 accented characters the column
  accepts. M1 (`chars().count()` → `len()`) reds exactly that test. 🔑 The refusal is at the route
  rather than left to the store because the store's answer is `1406: Data too long` in the driver's
  English — and under a non-strict `sql_mode` it is not a refusal at all but a silent TRUNCATION.
- **T2 — two sentences for what looks like one mistake, and the distinction is the honest one.**
  `/999` does not fit a `u8` and is `malformed`; `/64` fits and cannot belong to IPv4, so it is
  `prefix_not_in_family`. The product names only a rule it can evaluate. M6 collapses the second
  into the first and reds 2.
- ⚠️ **`insert_subnet`'s `#[allow(dead_code)]` is removed here and not at T7**, because its stated
  reason — *"the write path has no producer until story 14.2b"* — became FALSE the moment this route
  called it. AC7's other five wait for the routes that call them. *An allow whose reason is untrue
  is a false doc, not a deferral.*
- ⚠️ **THE CLOCK IS NO LONGER THE TELL IT WAS, measured rather than assumed.** The bin suite runs in
  **5.02 s WITHOUT a store** and **8.66 s against a live `mariadb:10.11.11`** — a factor of 1.7,
  where earlier stories in this project relied on a gap nearer 25×. 🔴 **Two candidate causes were
  measured and REFUTED**: the 5 s page budget test alone is 0.33 s, and `arp_ping`'s overlap test
  0.13 s (it is env-gated). Serial and parallel differ by 1.3 s, so it is not one blocking test but
  603 tests' ordinary cost. **No cause is named** — the rule is this project's own — and what
  confirmed the store genuinely answered is the mutation driver's own `store: reachable at
  127.0.0.1:13450 (connected)` line, not the clock.

### T2's mutation pass

Every prediction was written BEFORE the run and every run went through `cargo xtask mutate
--baseline`, which measures the unmutated tree first — story 6.6's register row asks for exactly
that, and its own six mutations ran without it, *which is luck confirmed rather than rigour*. The
store was DROPPED and recreated before the pass (story 6.6: this suite is non-deterministic against
a reused database), and the migrations were warmed by one full run first (story 14.1: a virgin store
races itself on `migrate!`).

**Eight ids, eight conforming outcomes.** No *"every red assertion-carried"* headline is claimed;
the carrier is named per row.

| id | mutation | predicted | measured | carrier |
|---|---|---|---|---|
| M1 | the label bound in BYTES (`chars().count()` → `len()`) | red:1 | red:1 | the 120-accented-character test's own assertion |
| M2 | the Origin check deleted from the handler | red:1 | red:1 | the cross-origin test's status assertion — ⚠️ its port answers a SUCCESS, so without the check it would go green on a 201 |
| M3 | a re-entered subnet answers 500 instead of 409 | red:1 | red:1 | the conflict test's status assertion |
| M4 | the redirect drops the subnet it just defined | red:1 | red:1 | the created test's header assertion — ⚠️ **and clippy, which the driver reports separately**: `format!("/ipam")` is a useless format. The row is dual-carried and that is an artefact of the mutation's spelling, not a second guard |
| M5 | the label is not trimmed | red:1 | red:1 | the whitespace-label test (`label=%20%20`) |
| M6 | a bad prefix collapses into `malformed` | red:2 | red:2 | the `/64` and the `192.0.2.5/24` tests, each on its own key |
| M7 | two `IpamError` variants share one key | red:1 | red:1 | the distinctness property over `IpamError::ALL`, in both locales |
| M8 | **AC2** — the handler extracts `State<MySqlPool>` | compile-fail | compile-fail | `error[E0277]`: the trait bound `…{define_subnet}: Handler<…>` is not satisfied |

⚠️ **What is NOT covered and is said rather than left to be found**: the ORDER of the two shape
refusals — a request that is both unlabelled and not a CIDR — is asserted by nothing, and no test
distinguishes `checked_label` running first from `parse_cidr` running first. It is a real
under-determination and not a defect today, since each refusal names its own rule; T3 owns the
refusal set and is where it should either be pinned or declared indifferent.
- **T3 — AC4's two carriers are measured separately, because neither covers the other.** A new
  VARIANT of `RepositoryError` is `error[E0004]: non-exhaustive patterns … Planted not covered`
  (M9) — the first exhaustive `match` over that type in this codebase, `core`'s own doc having
  recorded at story 14.1 that adding a variant then produced **zero** `E0004`. A new constraint
  NAME is invisible to the compiler, because `Constraint` carries a `&'static str` and the match
  inside it needs a `_`; that half is `every_constraint_name_the_store_can_produce_is_mapped`,
  which reads the names `repo::classify` can emit and asserts each is mapped (M10b).
- 🔑 **`constraint_refusal` returns an `Option`, and that shape IS the guard.** `check` maps to the
  backend sentence, which is also what the fallthrough answers — so an explicit 500 and a
  fallthrough 500 are **the same answer and not the same statement**, and a test cannot tell them
  apart through the response. The `Option` makes the difference observable: M12 folds `check` into
  the `_` arm, the operator sees no change whatever, and the test reds.
- 🔴 **M10 CONTRADICTED ITS PREDICTION — 4 red where 1 was predicted — and the cause was the
  mutation, not the guard.** It was written as *"a new constraint name"* and applied as a RENAME,
  so it also broke the three `repo.rs` tests asserting the name it removed. **This project's
  four-time class, met a fifth time**: *a mutation named for one thing and applied to another
  measures the other thing.* Re-run as M10b, which ADDS an arm, it reds exactly one test.
  ⚠️ And the contradiction was worth its cost twice over: it forced a separate measurement of
  whether the SET test reds AT ALL under M10 — it does, quoting
  `["check", "unique", "range_overlap", "foreign_key"]` — which the driver's bare count could not
  have told me.
- 🔴 **THE FLOOR WAS MASKING THE ASSERTION WRITTEN FOR THE DEFECT.** The draft asserted
  `names.len() == 3` BEFORE the mapping loop, so a new name reddened on `left: 4, right: 3` — a
  count — and the sentence telling a reader what to do was never reached. Story 5.13's finding,
  and this project has now met it **five** times. The mapping is asserted first; the floor stays,
  catching the other direction (a name that disappears leaves a mapping nothing can reach).
- 🔴 **THE KEY-COVERAGE GUARD WAS DEFEATED TWICE BY ITS OWN SOURCE, THE SECOND TIME BY THE COMMENT
  EXPLAINING THE FIRST.** It scans this file for the module's own i18n keys; written with a literal
  needle it matched ITSELF and reported a key named after the bare prefix, and the repair — a
  comment narrating the trap — contained the sequence and reddened again. 🔑 *A guard that greps a
  file greps its prose, and the better the prose explains the defect, the more reliably it
  reproduces it* — story 14.2's finding, met twice in five minutes. The needle is assembled at
  runtime now and the comment names neither half adjacently. ⚠️ Fifteen keys found, and the floor
  is **fifteen**: equal to what is there, never under it (story 6b.7).
- ✅ **The `app.yml` build hazard is CLOSED and was verified rather than assumed.** Story 6b.9
  measured that a translation-only change leaves the string absent from the binary; `build.rs`
  closed it at 6b.10 with `rerun-if-changed`. M13 blanks a French value and reds **2** tests — so a
  mutation on the locale file does measure something on this tree, where the register's warning
  would have said it measures nothing.
- ⚠️ **T2's under-determination is SETTLED rather than carried**: the CIDR is read before the label.
  A form wrong in two places can only be told about one, and the honest one to name is the first
  field the operator filled — a sentence about the second while the first is unusable reads as
  though the first had been accepted. M11 swaps the order back and reds.

### T3's mutation pass

Six ids. **Five conform to predictions written first; M10 CONTRADICTS, and that row is the
deliverable.** Same conditions as T2's pass: `--baseline` throughout, the store dropped and
recreated first, the migrations warmed by one full run.

| id | mutation | predicted | measured | what it says |
|---|---|---|---|---|
| M9 | a new VARIANT on `RepositoryError` (core) | compile-fail | compile-fail | `error[E0004]: non-exhaustive patterns: &…::RepositoryError::Planted not covered` — AC4's first carrier, on the first exhaustive match over that type in this codebase |
| M10 | *"a new constraint name"* — applied as a RENAME of `foreign_key` | red:1 | **red:4** 🔴 | **the prediction was wrong and the mutation was the cause**: a rename also breaks the three `repo.rs` tests asserting the removed name. Re-measured in isolation, the SET test DOES red, quoting the four names it found |
| M10b | the same as an ADDED arm, `Constraint("range_overlap")` | red:1 | red:1 | the SET test alone, on the mapping assertion — AC4's second carrier |
| M11 | the label read before the CIDR | red:1 | red:1 | the order test; T2 shipped the opposite with nothing asserting either |
| M12 | `"check"` folded into the `_` arm | red:1 | red:1 | 🔑 the operator sees **no change at all** — same status, same sentence — and the `Option` is what makes the omission observable |
| M13 | a French value blanked in `app.yml` | red | red:2 | the locale file IS a build input on this tree (`build.rs`, story 6b.10); the blank check and one inherited guard |
- **T4 — `insert_range` owns its transaction, and that is what makes the locks mean anything.** In
  autocommit a `FOR UPDATE` is released at the end of its own statement, so read-decide-write would
  still race with every lock in place. The parent row is taken first, then the deciding read, per
  Guy's arbitration of 2026-09-12 on §1(d)'s measured matrix of five strategies.
- **T4 — AC5's harness ships as a PERMANENT test and asserts the REFUSAL, not the row count.** Four
  of the five strategies leave exactly one row and three of those tell the operator something false
  (`Ok`, or a raw `1213 (40001): Deadlock found`). The 400 ms pause is a FUTURE handed to the
  production function, not a flag inside it: production passes `std::future::ready(())`, so the
  seam has a live caller and cannot rot.
- 🔴 **THE PASS'S OWN FINDING: NEITHER LOCK IS MEASURABLE ALONE, BECAUSE EACH MASKS THE OTHER'S
  MUTATION.** M15 removes the deciding read's `FOR UPDATE` and the harness stays **GREEN** — the
  parent row alone serialises entry. M16 plants the DRY line with both locks present and it stays
  **GREEN** — the range lock catches it. Only the COMPOSITE reds, `[Ok(()), Ok(())]`. 🔑 *Two
  guards that each mask the other's mutation are two guards nothing measures* — a variant of this
  epic's dominant class in which both halves are individually correct. Closed by
  `both_reads_of_a_subnet_under_write_take_their_lock`, a SOURCE guard naming each half, after
  which M15b and M16c red one test each. ⚠️ Its limit is written: it measures what was WRITTEN, not
  what the server executes, and it is cheaper than the composite and NAMES THE CAUSE where the
  composite names only the symptom (story 6b.11's AC5 as amended — *the two cumulate*).
- **T4 — AC9b's budget wraps the whole transaction**, on story 6b.10's precedent and against story
  14.2's defect of budgeting the wrong half. ⚠️ A timeout is reported as `Contention`
  DELIBERATELY: *the store never answered* and *the store said deadlock* are different facts and
  the same action — nothing was written, try again — so the distinction is kept in the log, where
  whoever is debugging is.
- ⚠️ **M17 CANNOT GO THROUGH `cargo xtask mutate`, and that is a stated limit of the driver.**
  Remove the budget and the test does not fail, it **HANGS** — measured by hand under `timeout 60`,
  killed at 60 s with `running 1 test` and no result line. Which is the defect exactly: *a handler
  with no budget does not answer wrongly, it does not answer.* The driver would have hung with it.
- ⚠️ **The budget test's clock is PAUSED, and what that proves is narrower than it looks.**
  `start_paused` auto-advances to the next timer, so it measures that a budget is armed, wraps the
  work, and produces a sentence — **not** the five seconds. Under a paused clock a budget of ten
  thousand seconds would pass this test identically. Said rather than implied.
- ⚠️ **`tokio`'s `test-util` feature is added as a DEV edge**, on the `opencmdb-core`
  `test-support` precedent already in that file: `features = ["full"]` does not include it. **No new
  crate — `Cargo.lock` is byte-identical, verified by diff rather than assumed.**

### T4's mutation pass

**Eight ids. Two predictions CONTRADICTED and one mutation that no driver can run** — the three
rows that carry the finding.

| id | mutation | predicted | measured | what it says |
|---|---|---|---|---|
| M14 | the parent row's lock dropped | red:1 | red:1 (+clippy) | the harness reds; the loser gets the raw deadlock instead of the rule |
| M15 | the deciding read's `FOR UPDATE` dropped | red:1 | **GREEN** 🔴 | **the finding**: the parent row alone serialises entry, so the harness cannot see this lock |
| M16 | the DRY line, both locks present | green | green | the range lock is what it cannot walk past — §1(d)'s prose, now measured here |
| M16b | the DRY line **and** an unlocked deciding read | red | red, `[Ok(()), Ok(())]` | the composite is the only behavioural carrier of the pair |
| M15b | M15 again, after the source guard | red:1 | red:1 | each half now has an individual carrier that names it |
| M16c | M16 again, after the source guard | red:1 | red:1 | same |
| M17 | the write budget removed | the test HANGS | **hung; killed at 60 s** | ⚠️ not runnable through `cargo xtask mutate`, which would hang with it. Measured by hand under `timeout` |
| — | `Cargo.lock` after the `test-util` feature | unchanged | unchanged | verified by diff, not assumed |

⚠️ **Two runs before these were refused by the driver for a reason it named WRONGLY**, and the
cause was outside the repository: `/tmp` (a 16 GB tmpfs) hit a per-user quota, so `cargo` could not
write, and the driver reported **`THE BASELINE IS NOT CLEAN: Red { tests: 11 }`** over a tree that
was green. Re-measured once space was freed: **898 tests, 0 failed**. 🔑 *The driver refusing was
right; the cause it named was not* — and this project's rule is that a cause needs a check. The
check here was the re-run.

🔴 **AND `git checkout --` DESTROYED T4's UNCOMMITTED WORK — the FIFTH occurrence in this
project**, in the session that had already read the rule in `CLAUDE.md`: *"revert the MUTATION,
never the FILE" has an unstated precondition — a file revert equals a mutation revert only on a
COMMITTED baseline.* The composite mutation M16b was planted by hand, and the restore reached for
`git checkout --` out of habit; the baseline was the T3 commit, so `insert_range_pausing`,
`load_subnet_locked`, `subnet_from_row` and the whole harness went with the mutation. Rebuilt
verbatim from the session's own record and re-verified. **Every hand-planted mutation after it was
restored from a `cp` copy**, which is the only form that does not depend on what is committed.

### File List

- `crates/opencmdb-bin/src/write_guard.rs` — NEW; the shared Origin check and its two tests.
- `crates/opencmdb-bin/src/repo.rs` — `classify`'s `Contention` arm, and the test that makes it live.
- `crates/opencmdb-bin/src/document.rs` — the Origin check removed and taken from `write_guard`.
- `crates/opencmdb-bin/src/main.rs` — the module declaration.
- `crates/opencmdb-bin/src/ipam_write.rs` — NEW; the plan's write sub-router and its first route.
- `crates/opencmdb-bin/src/ipam_repo.rs` — `insert_subnet`'s dead-code allow removed; it has a
  producer now.
- `crates/opencmdb-bin/locales/app.yml` — fifteen keys in both locales: one confirmation and
  fourteen refusals, each naming its rule.
- `crates/opencmdb-bin/src/main.rs` — `mod ipam_write` and the unconditional merge, above
  `auth_deny` and beside `ipam_page::router` rather than inside it.
- `crates/opencmdb-bin/src/ipam_repo.rs` — `insert_range`'s transaction and its two locks in order,
  the pause seam, `load_subnet_locked`, `subnet_from_row`, AC5's harness and the source guard.
- `crates/opencmdb-bin/Cargo.toml` — `tokio`'s `test-util` as a dev feature on an existing edge.

### Change Log

- 2026-09-12 — **T4: the overlap rule holds under concurrency, and the pass found that neither lock
  could be seen alone.** The parent row then the deciding read, a permanent 400 ms harness asserting
  the named refusal, and a budget around the whole transaction. **897 → 900 tests** (610 bin + 191
  core + 99 xtask), ten gates, clippy `--all-targets`, fmt; **5.02 s without a store and 8.78 s
  against a live `mariadb:10.11.11`**. 🔴 Two predictions contradicted, one mutation no driver can
  run, one driver refusal whose stated cause was wrong, and `git checkout --` destroying uncommitted
  work for the fifth time in this project.

- 2026-09-12 — **T3: the refusal set, carried twice.** `E0004` for a variant, a SET test for a
  constraint name, and the shape that makes the second possible — `constraint_refusal` returning an
  `Option`, so an explicit 500 and a fallthrough 500 stop being indistinguishable. **893 → 897
  tests** (607 bin + 191 core + 99 xtask), ten gates, clippy `--all-targets`, fmt; **5.03 s without
  a store and 8.59 s against a live `mariadb:10.11.11`**. 🔴 Three findings against my own
  instruments: a mutation named for one thing and applied to another, a floor assertion masking the
  assertion written for the defect, and a source-scanning guard that matched itself and then matched
  the comment explaining that it had.

- 2026-09-12 — **T2: the addressing plan has a producer.** `POST /ipam/subnet` writes, on
  `document.rs`'s shape and with the machinery epic constraint (1) front-loads. **879 → 893 tests**
  (603 bin + 191 core + 99 xtask), ten gates green, clippy `--all-targets`, fmt; measured in both
  store conditions with `env -u DATABASE_URL` per AC9, **5.02 s without a store and 8.66 s against a
  live `mariadb:10.11.11`** — ⚠️ and that ratio is the finding: see the Completion Notes, where two
  candidate causes for the shrunken gap are measured and refuted and **no cause is named**.

- 2026-09-12 — **T0: §2's five decisions taken**, and **the record caught up with the tree**. T1 had
  shipped in `860d0e4` — `classify`'s dead `Contention` arm and the shared Origin check — with its
  task unticked and this whole section empty. ⚠️ *A commit message is not a story record*: the two
  live in different files and only one of them is read by the next agent. **879 tests** (589 bin +
  191 core + 99 xtask), re-measured on this tree with `cargo test --workspace --locked` rather than
  recalled.
- 2026-09-12 — contexted, then corrected by the mandatory FACT-CHECK layer, which found **a
  systematic class rather than a slip**: four of §1's six `file:line` citations were `38da035`-era,
  copied from story 14.2 without re-deriving them, and the tree had moved underneath them — 14.2
  itself being what moved three of the four. 🔑 **Copying a measurement with its coordinates copies
  a photograph with its date erased.** One substantive premise fell with them (`insert_subnet` does
  have a caller in a test module, added by 14.2), one criterion was satisfiable by doing nothing
  (AC7), and three counts were wrong. Every one is corrected in place with what it said.
- 2026-09-12 — contexted. Nothing in §1 is re-derived: each row carries the measurement that
  produced it at 14.2's validation or code review, so this story starts where that one stopped
  rather than where its plan did.
