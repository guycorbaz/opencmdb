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
denylist since 14.2 — reintroducing the key reds the gate naming `app.yml:810`.

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

`insert_subnet` has **no caller outside its own test module**, and `ip_range.subnet_id` /
`ip_address.subnet_id` are both `NOT NULL` foreign keys to it (`0007:120`, `:157`). With two routes
an empty plan stays empty for ever and every range write answers `Err(NotFound)` — measured.

### (b) 🔴 The auth perimeter must be a PROPERTY, and today it is a literal

The gap-hunt mounted an always-on `POST /ipam/range` BELOW `auth_deny` and measured **873 tests and
TEN GATES GREEN over an unauthenticated write answering `201 Created`.** The one test carrying the
*route-above-the-layer* property is hardcoded to `/document-all` (`main.rs:2147`), and the perimeter
guard iterates `Screen::ALL` (`main.rs:1570`) — **a POST path is in neither.** This is story 6b.2's
measured defect (`GET /dashboard` → 200 below the layer) in its POST form.

**What it owes**: a guard over EVERY route the router carries, derived and not listed.

### (c) 🔴 The keyed refusals are over what the HANDLER CAN RECEIVE, not over `IpamError`

Measured through the very adapters these routes call:

| gesture | what comes back | an `IpamError`? |
|---|---|---|
| a range in a subnet that does not exist | `Err(NotFound)` | **no** |
| the same CIDR defined twice | `Err(Constraint("unique"))` | **no** |
| **the same address defined twice** | `Err(Constraint("unique"))` | **no** |
| the loser of a lock wait, once (d) lands | `Err(Backend("…1205…"))` | **no** |

🔑 *A SET over `IpamError::ALL` is correct about `IpamError` and silent about the three refusals the
operator meets first* — and the second row is **the likeliest refusal on this screen**, an operator
retyping an address. ⚠️ And **nothing in the compiler forces a handler to distinguish any of them**:
`RepositoryError` is not `#[non_exhaustive]` and no exhaustive `match` traverses it, so the variant
14.2 added produced **zero `E0004`**. The obligation is carried by the SET test or by nothing.

### (d) 🔴 The concurrency fix goes on the read that DECIDES, and the parent-row lock is defeated by one DRY line

`insert_range` reads every sibling (`ipam_repo.rs:238`), decides in Rust, then inserts — no
transaction, no lock. With a 400 ms pause injected, **two overlapping ranges both committed, both
reporting success.**

Guy's arbitration (C) of 2026-09-11, **re-aimed the same day on the gap-hunt's measurement**: a
transaction plus `SELECT … FROM ip_subnet … FOR UPDATE` on the parent WORKS — the loser blocks,
re-reads and is refused on the domain rule, no deadlock and no 1205 — **but reusing `load_subnet`
inside the transaction, the DRY line anyone writes, BRINGS THE RACE BACK** with the lock still taken
and worthless: under REPEATABLE READ the snapshot is fixed by the first CONSISTENT read, and a
locking read does not fix it. **So the lock goes on `SELECT … FROM ip_range … FOR UPDATE`**, the
read whose result decides.

⚠️ **AC: the 400 ms pause harness ships as a PERMANENT test.** The ordering is load-bearing and
invisible; nothing else in the tree would catch its loss. *A fix whose evidence is the absence of
the earlier symptom is not evidence.*

### (e) 🔴 `RepositoryError::Contention` is DEAD CODE, and this story is what makes it reachable

`classify` (`repo.rs:1613-1614`) matches `Some("1213") | Some("1205") => Contention`. It never
fires: **sqlx's `code()` returns the SQLSTATE**, and the MySQL number lives in a separate `number`
field. Measured on two errors — a lock-wait timeout arrives as `code() = Some("HY000"), number =
1205`, a duplicate key as `code() = Some("23000"), number = 1062`.

`Contention` has one producer site, no test and no consumer (5 mentions, 4 of them prose).
⚠️ **Pre-existing and NOT this story's to have caused — but (d) is what first makes it reachable on
a route an operator presses**, and it would surface as a raw English driver sentence, which is
exactly what (c) forbids. **Decide here: repair `classify` or map the case in the handler, and say
which.** A register row is not enough when this story's own arbitration opens the path.

### (f) The last six `#![allow(dead_code)]`

`canonical`, `Subnet::contains`, `insert_subnet`, `load_subnet`, `insert_range`, `insert_address`.
Eleven warnings before 14.2, **six after**. They go when these routes call them; neither story may
leave one standing while claiming to have paid it off.

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

**AC3 — every route the router carries answers 401 without a credential, asserted as a PROPERTY**
derived from the route table and never from a list. 🔴 §1(b)'s measurement is the reason: 873 tests
and ten gates were green over an unauthenticated `201`.

**AC4 — every refusal the operator can reach is a KEY, in both locales, naming the rule.** The SET
is over **what the handler can RECEIVE** (§1(c)), compared in both directions, and it reds the day a
new refusal is reachable. ⚠️ Story 6.4's finding is the reason: a success message shipped in English
under a French UI with a raw UUID in it.

**AC5 — the overlap rule holds under concurrency**, by §1(d), with the lock on the read that
DECIDES — and **the 400 ms pause harness ships as a permanent test**, proven to red when the lock
moves back to the parent row alone.

**AC6 — `Contention` either fires or goes.** §1(e): decide, implement, and say which. If it fires,
a test drives a real lock wait and asserts the variant; if it goes, the arm is deleted rather than
left as a promise the driver cannot keep.

**AC7 — `#![allow(dead_code)]` is GONE from `ipam_repo.rs`**, not narrowed further. Six items, six
producers.

**AC8 — the empty plan LINKS to the gesture** (`epics.md` criterion 5), which 14.2 registered as
undeliverable and this story delivers.

**AC9 — THE LIVE COUNT lives here**, with the command and both conditions named. Baseline **877**
(587 + 191 + 99) at `354283b`. ⚠️ **Use `env -u DATABASE_URL`, never `DATABASE_URL=`**: an EMPTY
value is not an ABSENT one — sqlx tries to connect to it and **118 tests fail**, measured while
writing this story.

**AC10 — no regression**: ten gates, `clippy --workspace --all-targets -D warnings`,
`RUSTFLAGS="-D warnings"`, fmt, `cargo deny`. **BOTH browser gates ARE claimed** — this story adds
controls and a write; `a11y/seed.sql` and the axe gate's `/ipam` state may need widening, and
`AXE_REQUIRE_PLAN=1` is already in CI.

## Tasks / Subtasks

- [ ] **T0** Take §2's decisions with Guy; §1 is settled and is not re-opened.
- [ ] **T1** (AC6) `classify`'s `Contention` arm: repair or remove, with the check either way.
- [ ] **T2** (AC1, AC2) The sub-router and the first route, on `document.rs`'s shape.
- [ ] **T3** (AC4) The refusal set over what the handler can receive, keyed in both locales.
- [ ] **T4** (AC5) The lock on the deciding read, and the pause harness as a permanent test.
- [ ] **T5** (AC1) The second and third routes, and the test that asserts the reuse.
- [ ] **T6** (AC3) The auth perimeter as a derived property.
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

- New: a write-route module on `document.rs`'s shape. ⚠️ **`page.rs` is at 1954 of 2000** and
  `ipam_page.rs` at ~700; the split rule applies BEFORE the growth.
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

- 2026-09-12 — contexted. Nothing in §1 is re-derived: each row carries the measurement that
  produced it at 14.2's validation or code review, so this story starts where that one stopped
  rather than where its plan did.
