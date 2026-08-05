# Story 5.11b: The arrival order of a scan cannot change what the product believes

Status: ready-for-dev

<!-- ⏳ NOT YET VALIDATED. This project requires a validation pass by **two fresh-context agents**
     (fact-check + gap-hunt) before `dev-story` — Guy's decision at the Epic 4 retrospective.

     🔑 **THE GAP-HUNT MUST COMPILE THIS STORY AGAINST A LIVE DATABASE.** `DATABASE_URL` is unset
     here, every DB-backed test passes by `return`ing, and the suite reports 429 either way. §8 has
     the `docker run`; host port **13306**, never 3306 (`kesh-mariadb` belongs to another project).

     🔑 **ASK IT EXPLICITLY:** *"does each prescribed mutation actually red, and WHAT carries the
     red — an assertion, an `expect` panic, or the compiler?"* Report the carrier PER TEST.

     🔴 **ASK IT SOMETHING ELSE, SPECIFIC TO THIS STORY:** *"is any test here capable of FAILING?"*
     A story whose subject is a property that already holds by construction is the ideal breeding
     ground for a test that passes because it measures nothing. §3 names the shape; every AC below
     carries a mutation whose job is to make the test red, and **a mutation that leaves the suite
     green is a HIGH finding, not a reassurance.**

     ⚠️ **Commit before the mutation pass** — the driver's first act is `git checkout -- crates/`.
     ⚠️ **A mutation must preserve the ARITY of a SQL statement's bind parameters** or the MySQL
     protocol desynchronises and the suite HANGS. Run every mutation under a timeout. -->

## Story

As the operator whose scanner sweeps a network in whatever order the wire answers,
I want the same observations to produce the same identity decisions regardless of arrival order,
So that a scan's timing cannot change what the product believes (NFR6).

**This story is a MEASUREMENT, and that is what makes it dangerous.** The property it tests is
already true by construction — `join` returns a `BTreeMap`, `candidates` a `BTreeSet`, the witness is
a `min` over a `BTreeSet`, the seen-window a `min`/`max` fold. **Nothing here is expected to red.**
That is exactly the condition under which a test that measures nothing looks like a success, and §3
is written against it.

**It also has TWO measured order-dependencies to answer**, both registered at story 5.11's name and
handed here explicitly rather than left to be rediscovered (§2).

**What this story does NOT do:**

- it does **not** change the engine. `identity::{l1,blocking,cascade}` are untouched. **A change
  there is a FINDING**;
- it does **not** wire the resolver into `main.rs` (still story 5.14's), and does **not** implement
  an `l2-*` rule — the trap corpus stays **11 unanswerable, `passed() == false`**. **If it turns
  green, that is a FINDING**;
- it does **not** add a dependency. §5 is why `rand` is not taken;
- **one production change only**, and it is small and stated: `resolve` gains a refusal (§2a). If
  that is unwanted, strike it and the story is pure test.

**Nothing under `fixtures/` moves.** No artefact bytes, no `MANIFEST.toml`, no re-hash.

---

## What this story inherits, measured rather than assumed

### 1. The four mechanisms that make the property true, and where they are

| mechanism | why order cannot reach it | site |
|---|---|---|
| `join` | returns `BTreeMap<L1Key, BTreeSet<ObsId>>` — key order, not arrival order | `l1.rs:172` |
| `candidates` | returns a `BTreeSet<CandidatePair>` over unordered pairs | `blocking.rs:171` |
| `placement_decision` | the witness is the **smallest other** `ObsId` in a `BTreeSet` | `resolver.rs:358` |
| `seen_window` | a `min`/`max` fold, commutative | `resolver.rs:405` |

Since story 5.11 there is a fifth, and it is the one that makes the strongest test in this story
possible: **the pass is idempotent**, so a fuzzed order run into an already-populated store must be
a NO-OP. Any order-dependence surfaces as a write, a supersede or a vacate (§4, AC2).

### 2. 🔴 Two measured order-dependencies, both in `resolve_within`, both this story's

Story 5.11's validation found them and named 5.11b as owner. Neither is in the four mechanisms
above; both are in the function that encloses them.

**2a. `by_id` is LAST-DUPLICATE-WINS.** `resolver.rs:234`:

```rust
let by_id: BTreeMap<ObsId, &Observation> = observations.iter().map(|o| (o.obs_id, o)).collect();
```

A slice carrying one `obs_id` **twice with different content** resolves to whichever copy arrives
last — and that copy's `observed_at` is what `write_link` stores as `valid_from` and what
`seen_window` folds into the interface window. **A fuzzed order over such a slice changes a STORED
column.** Worse, `join` walks the whole slice, so the observation is grouped under keys from the
copy that did NOT win: story 5.11's validation measured a slice `[obs1 with mac01, obs1 MAC-less]`
placing `obs 1` on `interface(mac01)` while the winning copy carries no MAC at all.

🔑 **The corpus cannot produce this shape**, and that is a fact in this story's favour rather than a
gap: `FixtureConnector::from_records` refuses a repeated `obs_id`
(`RepeatedObservationId` / `DuplicateObservationId`), so **no replay stream can carry one**. The
shape is reachable only by handing `resolve` a hand-built `Vec`.

**The prescribed answer is a REFUSAL, not a documented precondition.** An `ObsId` identifies an
immutable observation; two different contents under one id is a caller bug, and the pass should say
so rather than silently pick one. `resolve_within` returns `Err` on a repeated `obs_id` whose
content differs. _(A repeated IDENTICAL observation stays legal — story 5.9b's
`a_repeated_obs_id_writes_one_link` and 5.11's `a_repeated_obs_id_abstains_once_and_the_pass_says_so`
both pass the same clone twice and must keep passing. **If either reds, the refusal is too broad.**)_

**2b. The tail abstention loop iterates the RAW SLICE.** `resolver.rs:329`. The row VALUES are
invariant; only the INSERT order moves, and `snapshot_links` sorts. So this one is benign — but it
is benign by an argument, and AC1's permutation sweep is what turns the argument into a measurement.

### 3. 🔴 The failure mode this story must design against: a test that cannot fail

Six consecutive reviews in this project caught a claim outrunning its measurement, and story 5.11's
own review caught **two mutations that left the whole suite green**. A story that asserts a property
already true by construction is where that defect is easiest to ship and hardest to see: the test is
green on day one, and stays green whether or not it measures anything.

**Every AC below therefore names the mutation that must red it**, and the mutation is chosen to
break the ORDER-INDEPENDENCE specifically — not the pass in general. A mutation that reds a test by
breaking something else has not shown that test measures order.

The canonical shape, and the one to prescribe:

```
replace `join`'s BTreeMap with a HashMap seeded per-instance   → group ITERATION order varies
replace the witness `min` with `first()` over the slice order  → the witness follows arrival
```

⚠️ Neither is a compile-level change to the engine — **`identity/` is not to be edited**. Apply both
by mutating the RESOLVER's use of them where possible, and where it is not, say so and mark the AC
as measured by the permutation sweep alone.

### 4. The three measurement shapes, and why all three

**Shape A — pure, exhaustive, no database.** `join(&obs)` and `candidates(&obs)` over **every**
permutation of a small slice. The corpus streams carry **3 to 6 observations**, so `n!` is at most
720 — exhaustive enumeration is cheap and strictly stronger than sampling. This covers the SET of
interfaces, which the database shapes below cannot: they start from a store where the interfaces
already exist.

**Shape B — purge-and-replay in a fuzzed order.** Run in-order, snapshot, `purge_engine_links`
(interfaces are NOT purged), run the PERMUTED slice, snapshot, compare. 🔑 **`interface_id` is
literally comparable here** because the replay FINDS its interfaces rather than minting them — this
is story 5.10's apparatus with a permuted input, and it needs no new adapter code. It is also the
exact statement D14 wants: *the engine's output depends only on the observations and on the
interfaces*.

**Shape C — the fuzzed order is a NO-OP.** Run in-order, then run the permuted slice into the SAME
populated store. If the order is irrelevant, the pass reports
`links_written = 0, superseded = 0, vacated = 0, unchanged = N`. **Only possible since story 5.11**,
and it is the strongest of the three: any difference in a derived decision must surface as a write,
a supersede or a vacate. It needs no snapshot comparison at all.

⚠️ **Shape B alone would be a weaker claim than it looks** — both runs start from a store the
in-order pass built. Shape A is what covers *"the same observations in any order derive the same
interfaces"*, and it is the cheapest of the three.

### 5. The permutation generator: no new dependency, and mostly no randomness

**`rand` is not in the workspace** (checked: `crates/*/Cargo.toml`), and taking it adds a crate to a
graph `cargo-deny` audits, for a job that does not need it.

- **For the corpus streams (n ≤ 6): enumerate EXHAUSTIVELY.** No RNG, no seed, no flakiness, and
  strictly stronger than any sample. AC3's *"the seed is recorded so a failure is reproducible"* is
  satisfied a fortiori — the permutation index IS the reproduction.
- **For the reference-scale slice (n = 300): a seeded shuffle**, ~10 lines of a deterministic
  xorshift or LCG driving a Fisher-Yates, over a **fixed sweep of seeds** (`0..=K`) rather than a
  clock-derived one. A clock-derived seed makes a test that fails once a month and reproduces never
  — the anecdote AC3 exists to forbid. The seed is printed with the permutation on failure.

⚠️ **The generator is itself code that can be wrong.** A shuffle that returns the identity
permutation makes every test below green and measures nothing — the §3 failure mode, arriving
through the tool rather than the subject. **It gets its own test**: over a fixed seed sweep the
generator must produce at least two DISTINCT permutations, and must produce a permutation that is
not the identity.

### 6. The corpus can satisfy AC1's letter and cannot reach the interesting shapes

`epics.md` says *"given a corpus replay stream"*. Use one — and know what it cannot do. Measured and
already recorded in `resolver.rs`'s test module: **no committed observation carries more than one
MAC**, and every stream carries a single `l2_domain`. So the corpus cannot produce the multi-MAC
shape, the multi-scope shape, or the abstention/placement mix in one slice.

Synthetic slices are therefore **required, not optional** — the same conclusion story 5.5 reached
about its own claim, for the same reason. Cover at least: a multi-MAC observation, a MAC-less one
(abstention), two scopes, and a group of three (so the witness convention is exercised).

### 7. The tree this story extends, measured on `798799d`

- **`crates/opencmdb-bin/src/resolver.rs`** — `resolve` [`:207`], `resolve_within` [`:228`],
  `write_link` [`:424`], `same_decision` [`:569`], `Resolution` with its **eight** counters
  [`:130-155`], and the test harness `fixture()` / `pass()` / `try_pass()` / `within()` /
  `current_links()` / `versions()` / `all_link_ids()`.
- **`crates/opencmdb-bin/src/repo.rs`** — `snapshot_links`, `purge_engine_links`,
  `count_identity_links`, `load_current_engine_slots`. **Nothing new is needed here**: shape B
  reuses `snapshot_links` unchanged, which is the point of choosing it.
- **`crates/opencmdb-bin/src/fixture_connector.rs`** — `FixtureConnector::load(id, caps, scopes,
  "scenario/replay/<name>.jsonl")`, then `poll` into a `VecSink` to get `Vec<Observation>`.
- **`master` is at 429 tests** (224 bin + 159 core + 46 xtask), six gates green.

### 8. 🔴 A green suite says NOTHING about the database

`DATABASE_URL` is unset locally; every DB-backed test begins `let Some(pool) = fixture(…) else {
return; }` and PASSES by returning. **The suite reports 429 either way.**

```
docker run -d --rm --name opencmdb-5-11b -p 13306:3306 \
  -e MARIADB_ROOT_PASSWORD=<choose> -e MARIADB_DATABASE=opencmdb mariadb:10.11.11
export DATABASE_URL='mysql://root:<choose>@127.0.0.1:13306/opencmdb'
```

⚠️ **Port 13306, never 3306.** Tests serialise on `crate::DB_TEST_LOCK`.
⚠️ **A permutation sweep multiplies database round-trips.** Shape A is pure and can be exhaustive;
shapes B and C must be SAMPLED, and the sample size stated rather than left implicit.

### 9. Gates

`cargo xtask ci` (six gates) · `cargo clippy --workspace -- -D warnings` **and**
`--all-targets` · `#![deny(missing_docs)]` is on for `opencmdb-bin` · the `file-size` gate counts
only lines before the first `#[cfg(test)]`, and `resolver.rs` is well under it.

---

## Decisions taken at contexting

1. **Exhaustive over the corpus, seeded-sweep over the reference scale** (§5). No `rand`.
2. **Three measurement shapes, not one** (§4) — A pure, B purge-and-replay, C no-op.
3. **`by_id`'s last-duplicate-wins becomes a REFUSAL** (§2a). The one production change. A repeated
   IDENTICAL observation stays legal.
4. **The permutation generator gets its own test** (§5), because a broken shuffle makes every other
   test in this story vacuously green.
5. **Synthetic slices are required** (§6); the corpus alone cannot reach the shapes that matter.

---

## Acceptance Criteria

**AC1 — the derived interfaces and pairs are identical under EVERY permutation.** (shape A)
Given a slice of up to six observations, when `join` and `candidates` are called on every one of its
`n!` permutations, then all results are equal.
**Mutation:** make the resolver consume `join`'s groups in slice order rather than key order. The
test must red.

**AC2 — a fuzzed order run into a populated store writes NOTHING.** (shape C)
Given a store built by an in-order pass, when the permuted slice is resolved into it, then
`links_written = 0`, `links_superseded = 0`, `links_vacated = 0`, and `links_unchanged` equals the
number of current engine links.
🔴 This is the strongest statement in the story and it exists only because story 5.11 shipped
idempotence. **Mutation:** make the witness `first()` over slice order instead of the smallest
other `ObsId` — the evidence then follows arrival and the pass supersedes.

**AC3 — a purge-and-replay in a fuzzed order reproduces every decision-bearing column.** (shape B)
Given an in-order pass and its snapshot, when the engine's links are purged and the PERMUTED slice
replayed, then `snapshot_links` compares equal — `interface_id` included, which is comparable
because the replay finds its interfaces rather than minting them.

**AC4 — the fuzzing is reproducible, and the generator is itself measured.**
Given the permutation source, when it runs, then corpus-scale slices are enumerated EXHAUSTIVELY
(no RNG) and the reference-scale slice uses a fixed seed sweep whose seed is printed with any
failure. **And** the generator has its own test: over that sweep it produces at least two distinct
permutations, and at least one that is not the identity.
🔴 Without this AC a shuffle that returns its input makes AC1–AC3 green and meaningless.

**AC5 — a repeated `obs_id` with DIFFERENT content is refused.** (§2a)
Given a slice carrying one `obs_id` twice with differing facts or instants, when it is resolved,
then the pass returns `Err` rather than silently keeping the last copy.
**And** a repeated IDENTICAL observation stays legal — story 5.9b's and story 5.11's tests for it
must both still pass. **If either reds, the refusal is too broad.**

**AC6 — the corpus is used, and its limits are stated.**
Given at least one committed replay stream, when it is loaded through `FixtureConnector` and
resolved, then AC1–AC3 hold over it. **And** the story records that the corpus carries no multi-MAC
observation and no second `l2_domain`, so the synthetic slices of §6 are what cover those.

**AC7 — nothing else moves.**
`identity::{l1,blocking,cascade}` untouched · `fixtures/` untouched · `main.rs` untouched · trap
corpus still 11 unanswerable and `passed() == false` · six gates green · both clippy forms clean ·
no new dependency in any `Cargo.toml`.

**AC8 — the doc twins say the same thing.**
`CLAUDE.md`, `docs/project-context.md`, `sprint-status.yaml` and this file agree on status and
counts. 🔴 **This has now failed on SEVEN consecutive stories**, most recently inside story 5.11's
own AC9 — which was ticked while neither twin had been touched. Check the twins by opening them, not
by intending to.

---

## Tasks / Subtasks

**T1 — the permutation source.** (AC4)
A `permutations(&[T]) -> impl Iterator` for exhaustive enumeration, and a seeded Fisher-Yates for
the large slice. Test the generator FIRST (AC4's second half), because everything downstream is
vacuous if it is wrong.

**T2 — shape A, pure and exhaustive.** (AC1)
No database. `join` and `candidates` over every permutation of the corpus stream and of the
synthetic slices of §6.

**T3 — shape C, the no-op.** (AC2)
The strongest test; write it before shape B, because it needs no snapshot machinery.

**T4 — shape B, purge-and-replay permuted.** (AC3)
Story 5.10's apparatus with a permuted input. Reuse `snapshot_links` unchanged.

**T5 — the corpus stream.** (AC6)
`FixtureConnector::load` → `poll` into a `VecSink` → `Vec<Observation>`. Record which stream, and
its limits.

**T6 — the refusal.** (AC5)
`resolve_within` refuses a repeated `obs_id` with differing content. Verify the two existing
repeated-`obs_id` tests still pass.

**T7 — prove-to-red.** Commit first. Each mutation under a timeout, carrier recorded PER TEST:

| | mutation | prediction |
|---|---|---|
| M1 | consume `join`'s groups in slice order | AC1 reds |
| M2 | witness = `first()` over slice order instead of smallest-other | AC2 reds — a supersede appears |
| M3 | the shuffle returns its input unchanged | AC4's generator test reds; ⚠️ **predict whether AC1–AC3 stay green, and say so before running** |
| M4 | drop the repeated-`obs_id` refusal | AC5 reds |
| M5 | seed the sweep from the clock | AC4 reds on reproducibility |
| M6 | sample one permutation instead of enumerating | AC1 reds only if the sample is the identity — ⚠️ **say what this measures before running it** |

🔴 **A mutation that leaves the suite green is a HIGH finding**, and on this story it is the expected
failure mode rather than a surprise. Record it as such.

**T8 — docs and register.** The three §2/§5 entries in `deferred-work.md`; then the twins (AC8).

---

## Dev Notes

### Shapes to follow, not reinvent

- The DB test harness at `resolver.rs`'s test module: `fixture()`, `pass()`, `try_pass()`,
  `within()`, `versions()`, `all_link_ids()`, `current_links()`. Every DB test takes `DB_TEST_LOCK`
  and returns early without `DATABASE_URL`.
- One pass runs inside ONE `transact` (D21). `resolve` opens no transaction — that is the caller's
  precondition, and story 5.9b measured 2 interfaces committed under autocommit when a caller did
  not cooperate.
- Deliberate redundancies a DRY pass may NOT collapse: `expected_l1_conclusion` restating D13's
  text, `fixtures.rs`'s `expected()`, the per-module `scratch_dir`.

### Compile-level facts

- `sqlx` is built without its `chrono` feature: a `DATETIME(6)` has no Rust type to decode into.
  Instants come back as strings via `CAST(… AS CHAR)`.
- `Observation` derives `Clone`; permuting a slice means permuting clones or indices, not
  references into a moved `Vec`.
- `FixtureConnector::from_records` **refuses a repeated `obs_id`** — so a corpus-derived slice can
  never exercise AC5, and AC5's fixture must be hand-built.

### What a reviewer will challenge, and the answer that is already measured

- *"Can any of these tests fail?"* → §3, and every AC names its mutation. That is the question this
  story is designed around.
- *"Why not `rand`?"* → §5. Exhaustive enumeration is stronger at corpus scale and needs no crate.
- *"Isn't shape B enough?"* → No. Both its runs start from a store the in-order pass built; shape A
  is what covers the derived interface SET.

### References

- `epics.md:1636-1652` (story 5.11 as written; AC1 and AC3 are this story's), `:136` (NFR6),
  `prd.md:1224-1225` (NFR6).
- `architecture.md:931` (D13's order), `:3364` (the engine never touches the clock).
- `deferred-work.md` — the two order-dependencies registered at 5.11's close, owner 5.11b.

---

## Dev Agent Record

### Agent Model Used

_(to be filled by `dev-story`)_

### Debug Log References

_(to be filled — the permutation counts, the sample sizes, and the mutation table's measured column)_

### Completion Notes List

_(to be filled)_

### File List

_(to be filled)_

---

## Change Log

| Date | Change |
|---|---|
| 2026-08-05 | Created by `create-story`. Five decisions at contexting, the load-bearing one being that **the corpus streams carry 3–6 observations, so permutations can be enumerated EXHAUSTIVELY rather than sampled** — no `rand`, no seed, no flakiness, and strictly stronger than the fuzz `epics.md` asks for. The story is designed around one failure mode: a test that cannot fail. |
