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

1. **The form's shape and its mount point.** `document.rs` is the model: a sub-router whose state
   holds a PORT and no pool, so `State<MySqlPool>` in the handler **fails to compile** (story 6.1's
   M4, re-measured by two review layers). ⚠️ `/ipam` is now on its own pool-bearing router
   (`ipam_page::router`), so the two must not be fused by accident.
2. **Where the form lives on screen.** `/ipam` has a rail (`ipam-rail`) and an empty-plan branch
   whose control currently says NOT YET BUILT through `Gesture::Planned`'s badge. ⚠️ **`epics.md`'s
   criterion 5 says the empty plan "names the gesture that fills it AND LINKS TO IT"** — 14.2 could
   not, and registered it; this story owes the link.
3. **What a range's id is.** ⚠️ Measured at 14.2: `insert_subnet`/`insert_range`/`insert_address`
   all accept the **nil UUID**, where story 6.1's review added a nil refusal AT THE ROUTE (D21/D48)
   and `insert_device` refuses it outright. Who mints the id, and is it v7?
4. **Whether a label is required.** `label` is `NOT NULL` with no non-empty CHECK; `""` inserts
   cleanly — measured.
5. **Whether the routes refuse an address inside an existing range.** Measured: the adapter accepts
   it in BOTH orders, and 14.2 renders it as `Defined` carrying the range's policy. Legal today; the
   form may still want to warn.

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

- [ ] **T0** Take §2's decisions with Guy; §1 is settled and is not re-opened.
- [ ] **T1** (AC6) `classify`'s `Contention` arm: repair or remove, with the check either way. ⚠️ The
  repair was BUILT and measured safe (`try_downcast_ref` + `number()`, whole workspace green, the
  `Constraint("unique")` path intact) — **but under the chosen lock it buys less than it looks
  like**, the deadlock becoming `Contention` where the operator needs `RangeOverlapsAnother`.
- [ ] **T2** (AC1, AC2) The sub-router and the first route, on `document.rs`'s shape.
- [ ] **T3** (AC4) The refusal set over what the handler can receive, keyed in both locales.
- [ ] **T4** (AC5, AC9b) The parent row THEN the deciding read; the pause harness as a permanent
  test asserting the REFUSAL and not merely the row count; and the budget around the transaction.
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

### Debug Log References

### Completion Notes List

### File List

### Change Log

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
