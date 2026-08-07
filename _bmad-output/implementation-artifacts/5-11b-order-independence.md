# Story 5.11b: The arrival order of a scan cannot change what the product believes

Status: done

<!-- ✅ VALIDATED 2026-08-05 by two fresh-context agents (fact-check + gap-hunt).
     **The gap-hunt BUILT THE WHOLE STORY** against its own live `mariadb:10.11.11` on 13311 —
     T1 through T6, **429 → 441 tests**, six gates green, both clippy forms clean, `cargo fmt`
     clean, and **no new dependency** (`Cargo.lock` diff empty). 5 HIGH, 5 MEDIUM, 4 LOW applied.

     🔴 **THE ANSWER TO THIS STORY'S OWN CENTRAL QUESTION IS BAD, AND THAT IS THE POINT OF HAVING
     ASKED IT: FOUR of the prescribed mutations were measured leaving the ENTIRE SUITE GREEN.**
       • **M1 as prescribed is a no-op** — the resolver consuming groups in slice order cannot reach
         a shape-A test that never calls the resolver. AC1 was reddenable by NO permitted mutation
         until Guy's arbitration opened `identity/` to a temporary mutation (§3).
       • **M5 is green**: reproducibility within one process is trivially true for any seed, so the
         seed's PROVENANCE was guarded by nothing. AC4 now demands a GOLDEN-VALUE test.
       • **M6 is green, and turns AC1 into a tautology** — `permutations()[0]` IS the identity.
       • **A degenerate ENUMERATOR leaves AC1–AC3 green** unless every consuming test asserts its
         own permutation count. The gap-hunt proved it by deleting the four count assertions it had
         happened to write and re-running.

     🔑 **WHAT THE STORY GOT RIGHT, ALSO MEASURED**: shape C is a no-op today AND is not a duplicate
     of story 5.11's idempotence test — under M2, 5.11's test stayed green while shape C reddened.
     Shape B's `interface_id` claim holds over 7 purge-and-replay samples. And the load-bearing
     sizing decision is exact: **720 permutations of the largest stream in 11.5 ms**.

     ⚠️ Also applied: `partial-then-failed.jsonl` ends in a `Failure` record so the obvious
     `.expect("poll")` PANICS, two streams carry their own connector/scope, the corpus context
     helpers are private, `main.rs` needs a `#[cfg(test)] mod` line so "untouched" was
     unsatisfiable, `resolver.rs:635` says "Nothing here reads `fixtures/`" which T5 falsifies, and
     the register bullet this story disposes of says "425 tests" where master carries 429.

     🔑 FOR THE NEXT VALIDATION: the gap-hunt found 9 of the 14, the fact-check the pure citation
     defects — including the one that started this, that AC1's mutation could not reach AC1's test.
     **Both layers found it independently**, one by reading and one by building.

     ⚠️ **Commit before the mutation pass** — the driver's first act is `git checkout -- crates/`.
     ⚠️ **A mutation must preserve the ARITY of a SQL statement's bind parameters** or the MySQL
     protocol desynchronises and the suite HANGS. Run every mutation under a timeout.
     🔑 `DATABASE_URL` is unset here and DB tests pass by `return`ing — `dev-story` must run its own
     database. §8 has the `docker run`; port **13306**, never 3306. -->

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

- it does **not** change the engine. `identity::{l1,blocking,cascade}` are untouched **in what
  ships**. **A change there in the diff is a FINDING** — a temporary mutation during T7 is not (§3);
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
copy that did NOT win — an observation can be placed on an interface derived from a MAC its winning
copy does not carry. ⚠️ That consequence follows from the code (`join` at `l1.rs:172` walks the
slice; `by_id` at `resolver.rs:234` keeps the last copy) and **is not recorded as MEASURED in any
artifact** — an earlier draft said it was. Under this project's *"a cause needs a named check"* rule
it is a reading, and T2's permutation sweep is what turns it into a measurement.

🔑 **The corpus cannot produce this shape**, and that is a fact in this story's favour rather than a
gap: `FixtureConnector::from_records` refuses a repeated `obs_id`
(`RepeatedObservationId` / `DuplicateObservationId`), so **no replay stream can carry one**. The
shape is reachable only by handing `resolve` a hand-built `Vec`.

**The prescribed answer is a REFUSAL, not a documented precondition.** An `ObsId` identifies an
immutable observation; two different contents under one id is a caller bug, and the pass should say
so rather than silently pick one. `resolve_within` returns `Err` on a repeated `obs_id` whose
content differs. _(A repeated IDENTICAL observation stays legal — story 5.9b's `a_repeated_obs_id_writes_one_link`
passes the same clone **twice** and 5.11's `a_repeated_obs_id_abstains_once_and_the_pass_says_so`
passes it **three times**; both must keep passing, and both were measured still passing when the
refusal was built at validation. **If either reds, the refusal is too broad.**)_

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

🔴 **A MUTATION MAY EDIT `identity/`. The ban in AC7 is on the SHIPPED DIFF, not on the prove-to-red
pass.** Guy's arbitration at this story's validation, on a measurement: the validation built the
story and ran the originally-prescribed mutation — *"make the resolver consume `join`'s groups in
slice order"* — and **all 441 tests passed**. It could not do otherwise. Shape A is pure: its test
calls `join` and `candidates` directly and never enters `resolve_within`, so no mutation of the
resolver can reach it. And `join` returns a `BTreeMap`, whose equality is structural, so even a
varying *consumption* order is unobservable in the assertion.

**The mutations that DO red, measured:**

```
`join`'s group becomes order-dependent (e.g. `if position % 2 == 0`)  → AC1 reds INSIDE the loop
`placement_decision` gains `arrival: &[Observation]`, witness = first  → AC2 and AC3 red
```

⚠️ The second is **not the trivial swap** an earlier draft implied: `placement_decision` receives a
`BTreeSet<ObsId>` and a `BTreeMap`, **neither of which carries arrival order**, so `group.iter().next()`
*is* the smallest and swapping it changes nothing. The slice must be threaded down to it. State that
plumbing before running it.

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

**`rand` is in no manifest in this workspace** — verified over all four (`Cargo.toml`, `xtask/Cargo.toml`,
`crates/*/Cargo.toml`), not the narrower glob an earlier draft named. `Cargo.lock` carries it only
transitively, via `sqlx-postgres` and `surge-ping`. Taking it as a DIRECT dependency adds a crate to
a graph `deny.toml` audits, for a job that does not need it.

- **For the corpus streams (n ≤ 6): enumerate EXHAUSTIVELY.** No RNG, no seed, no flakiness, and
  strictly stronger than any sample. AC3's *"the seed is recorded so a failure is reproducible"* is
  satisfied a fortiori — the permutation index IS the reproduction.
- **For the reference-scale slice (n = 300): a seeded shuffle**, ~10 lines of a deterministic
  xorshift or LCG driving a Fisher-Yates, over a **fixed sweep of seeds** (`0..=K`) rather than a
  clock-derived one. A clock-derived seed makes a test that fails once a month and reproduces never
  — the anecdote AC3 exists to forbid. The seed is printed with the permutation on failure.

⚠️ **The generator is itself code that can be wrong**, and the validation measured that ONE guard is
not enough. Three are needed, each closing a hole the others leave:

1. **The shuffle gets its own test** — over the seed sweep it produces at least two DISTINCT
   permutations, and at least one that is not the identity. Measured: this reds when the shuffle
   returns its input, and **AC1–AC3 all stay green**, so it closes nothing else.
2. **The shuffle gets a GOLDEN-VALUE test** — `shuffled(&(0..8).collect::<Vec<_>>(), 7)` equals a
   literal vector, pinning the ALGORITHM. 🔴 **It does NOT pin the seed sweep's provenance, and this
   paragraph said it did until the code review**: it names seed 7 as a literal and never reads
   `SEED_SWEEP`, so a `SystemTime::now()`-derived sweep leaves it green along with everything else
   (**all 445 tests**, measured twice). Reproducibility *within one process* is trivially true for
   any seed. **A further guard is therefore required**: a test reading the constant's values.
3. **Every CONSUMING test asserts how many permutations it consumed.** Measured: with the enumerator
   replaced by *"return the input"*, AC1–AC3 reddened **only** on count assertions, and deleting
   those four lines left all three GREEN. A guard on the generator does not protect its callers.

🔴 **And `permutations()` yields lexicographic order, so element 0 IS the identity.** Sampling one
permutation turns AC1 into `join(o) == join(o)` — proven by combining that sample with a genuinely
order-dependent `join` and watching the test stay green. Any sampling must skip the identity or
assert its count.

### 6. The corpus can satisfy AC1's letter and cannot reach the interesting shapes

`epics.md` says *"given a corpus replay stream"*. Use one — and know what it cannot do. Measured and
already recorded in `resolver.rs`'s test module: **no committed observation carries more than one
MAC**, and every stream carries a single `l2_domain`. So the corpus cannot produce the multi-MAC
shape, the multi-scope shape, or the abstention/placement mix in one slice.

Synthetic slices are therefore **required, not optional** — the same conclusion story 5.5 reached
about its own claim, for the same reason. Cover at least: a multi-MAC observation, a MAC-less one
(abstention), two scopes, and a group of three (so the witness convention is exercised).

🔴 **A consequence worth stating rather than discovering: the corpus AC1 test is reddened by nothing
in the permitted set.** Measured — under a `join` mutated to *first-key-wins*, the corpus test stays
GREEN while the synthetic one reds, because first-key-wins is a no-op where every observation
carries exactly one key. AC6 is satisfiable in letter, and the synthetic slices are what carry the
measurement.

⚠️ **Two committed streams do not load with the corpus context, and one PANICS on the obvious
idiom** — all measured at validation, through the exact path §7 prescribes:

| stream | what happens |
|---|---|
| `partial-then-failed.jsonl` | ends in `Record::Failure`, so `poll` returns `Err(ConnectorError::Unreachable)` with 4 observations already in the sink. **`.expect("poll")` panics.** By design (`fixture_connector.rs:321`) |
| `capability-downgrade.jsonl`, `partial-then-failed.jsonl` | carry their OWN `connector_id` and `scope`; loading them with the corpus context is refused `ForeignConnectorId`, then `UncoveredScope` |
| the other **11** | stream cleanly — 3, 4 or 6 observations, 1–5 groups, every permutation order-stable |

⚠️ `corpus_id()`, `corpus_scope()` and `corpus_caps()` are **private to `fixture_connector`'s own
test module**. T5 must restate them or make them `pub(crate)`; the choice is the implementer's, but
it is not free and the story names it here rather than leaving it to be hit.

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
`n!` permutations, then all results are equal — **and the test asserts HOW MANY permutations it
consumed** (§5.3), because a degenerate enumerator otherwise leaves it green.
**Mutation:** make `join`'s grouping order-dependent — a temporary edit to `identity/l1.rs`, which
AC7 permits for a mutation and forbids in the shipped diff (§3). Measured to red inside the
permutation loop. 🔴 The originally-prescribed mutation (the resolver consuming groups in slice
order) was **built and left all 441 tests green**; it cannot reach a test that never calls the
resolver.

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

**AC4 — the fuzzing is reproducible, and the generator is measured THREE ways.** (§5)
Given the permutation source, when it runs, then corpus-scale slices are enumerated EXHAUSTIVELY (no
RNG) and the reference-scale slice uses a fixed seed sweep whose seed is printed with any failure.
**And** all three guards of §5 exist: the shuffle's own distinctness test, a **golden-value** test
pinning one `(seed, input) → output` against a literal, and a permutation-count assertion in every
consuming test.
🔴 **REFUTED AT THE CODE REVIEW, 2026-08-06 — and the refutation is what this AC now requires.**
~~The golden-value test is not decoration. Without it, replacing the fixed sweep with a clock-derived
one leaves all 441 tests green over three consecutive runs — measured.~~ The PREMISE (a clock-derived
sweep is invisible) is true; the CONCLUSION (the golden-value test is what sees it) is false. That
test pins `shuffled` at a HARDCODED seed and never reads `SEED_SWEEP`, so it stays green under a
clock-derived sweep like everything else. **The guard that actually closes it reads the constant's
VALUES** (`the_seed_sweep_is_the_fixed_range_it_claims_to_be`), and with it M5 reds exactly one test.
Two properties had been conflated: *reproducible WITHIN one process* is trivially true for every
seed; *reproducible ACROSS runs* is what a fixed seed buys, and only the second was ever at stake.

**AC5 — a repeated `obs_id` whose DECISION-BEARING content differs is refused.** (§2a)
Given a slice carrying one `obs_id` twice with differing facts, instants, scope or vantage, when it
is resolved, then the pass returns an error that NAMES the condition rather than silently keeping
the last copy.
🔑 **`raw` is excluded from that comparison** — Guy's arbitration. `Observation` derives `PartialEq`,
so `!=` would compile, but `raw` is *"opaque provenance … that NO decision ever reads (D19)"*
[`observation/mod.rs:255-258`], and refusing on it would red a case where nothing was ever at stake.
The comparison is therefore explicit, field by field — **and it is guarded at COMPILE time, not by
a test**, which is the honest statement and not what this AC first asked for. ⚠️ Corrected at the
code review: `contradicts` destructures all six fields with no `..`, so adding a field to
`Observation` yields `error[E0027]` at that site (measured, with a `vlan` field). That is stronger
than a test — but it is compiler-carried, in a story whose headline is *"zero compiler-carried
reds"*, and it has a hole nothing sees: **replacing the pattern with `..` leaves the entire suite
green** (measured). The residue is registered rather than papered over; a test cannot practically
assert that a pattern is exhaustive.
🔑 **The error gets its OWN `RepositoryError` variant**, following story 5.11's precedent with
`InstantRegressed`: `Constraint(_)` means *"a database constraint was violated"* by its own doc, and
a self-contradictory input from the caller is not that.
**And** a repeated IDENTICAL observation stays legal — story 5.9b's `a_repeated_obs_id_writes_one_link`
(same clone **twice**) and story 5.11's `a_repeated_obs_id_abstains_once_and_the_pass_says_so` (same
clone **three times**) must both still pass. **If either reds, the refusal is too broad.**
_(Both were measured still passing when the refusal was built at validation.)_

**AC6 — the corpus is used, and its limits are stated.**
Given at least one committed replay stream, when it is loaded through `FixtureConnector` and
resolved, then AC1–AC3 hold over it. **And** the story records that the corpus carries no multi-MAC
observation and no second `l2_domain`, so the synthetic slices of §6 are what cover those.

**AC7 — nothing else moves, in the SHIPPED DIFF.**
`identity::{l1,blocking,cascade}` untouched **in what ships** — a temporary mutation during T7 is
permitted and is how AC1 is measured at all (§3, Guy's arbitration) · `fixtures/` untouched · trap
corpus still 11 unanswerable and `passed() == false` · six gates green · both clippy forms clean ·
**no new dependency** in any `Cargo.toml` or in `Cargo.lock`.
⚠️ **`main.rs` gains no BEHAVIOUR — a `#[cfg(test)] mod` declaration excepted.** Measured: T1's new
module needs one, so *"`main.rs` untouched"* was unsatisfiable as first written. Putting the
generator inside an existing test module avoids even that; either is acceptable, silence is not.

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

Every row below was **executed at this story's validation**, so the predictions are measurements
rather than guesses. Reproduce them; a divergence is a finding.

| | mutation | measured at validation | carrier |
|---|---|---|---|
| M1 | `join`'s grouping made order-dependent inside `identity/l1.rs` (permitted for a mutation — AC7) | **AC1 reds inside the permutation loop** | assertion |
| M1-noop | the resolver consumes `join`'s groups in slice order — *the originally prescribed one* | 🔴 **ENTIRE SUITE GREEN, 441/441.** Shape A never calls the resolver | none |
| M2 | `placement_decision` gains `arrival: &[Observation]`, witness = first-by-arrival | **AC2 reds** (a link is written), **AC3 reds** (the `evidence` column diverges), + `the_write_amplification_…` | assertion ×3 |
| M3a | the SHUFFLE returns its input | AC4's distinctness tests red — and **AC1, AC2, AC3 all stay GREEN** | assertion ×2 |
| M3b | the ENUMERATOR returns its input | AC1×2, AC2, AC3 red — but **every red lands on a count assertion**; delete those four lines and all three go GREEN | assertion |
| M4 | drop the repeated-`obs_id` refusal | **AC5 reds**, and only AC5 | assertion |
| M5 | seed the sweep from the clock | 🔴 **ENTIRE SUITE GREEN**, three runs — unless AC4's golden-value test exists | none |
| M6 | sample one permutation instead of enumerating | 🔴 **suite GREEN**, and `permutations()[0]` IS the identity, so AC1 becomes a tautology. **M6 + M1 together still leave the corpus AC1 test green** | none |

🔴 **Four of these were measured GREEN, and that is this story's whole subject.** A mutation that
leaves the suite green is a HIGH finding here, not a reassurance — and the reason each is green is
recorded above so it cannot be re-discovered as a surprise.

### Review Findings (2026-08-06)

All patches applied in the review commit; defers are registered with owners in `deferred-work.md`.

- [x] [Review][Patch] `SEED_SWEEP`'s doc asserted what a test 80 lines below it refutes [`permute.rs`]
- [x] [Review][Patch] `XorShift64`'s *"state is never zero"* is false for `0x9E37_79B9_7F4A_7C15` — silent degeneration [`permute.rs`]
- [x] [Review][Patch] the golden test's doc pointed at *"the test above"* positionally and repeated the refuted claim [`permute.rs`]
- [x] [Review][Patch] `factorial`'s `.max(1)` was an unreachable correction inside an oracle [`permute.rs`]
- [x] [Review][Patch] *"the refusal precedes every write"* was guarded by nothing — `transact` rolls back and hid it [`resolver.rs`]
- [x] [Review][Patch] no order test touched the `interface` table's derived instants — all five stayed green [`resolver.rs`]
- [x] [Review][Patch] the `facts.len()` term in `contradicts` was droppable with the suite green [`resolver.rs`]
- [x] [Review][Patch] the corpus test's `join` oracle was `!is_empty()` where its twin pins an exact count [`resolver.rs`]
- [x] [Review][Patch] the `seeds == 8` message claimed to catch a clock-derived sweep, which it cannot [`resolver.rs`]
- [x] [Review][Patch] the `raw` carve-out's order-independence was an argument, now a measurement [`resolver.rs`]
- [x] [Review][Patch] `sampled_permutations`' constants are tuned to `n == 6` with no stated precondition [`resolver.rs`]
- [x] [Review][Patch] `M3b+` was wrong in five documents — six count assertions, not five
- [x] [Review][Patch] `M1-fkw` was 10 red, not 11 — the driver's raw line count was copied
- [x] [Review][Patch] three timing figures rested on no check
- [x] [Review][Patch] AC4 and §5 still carried the M5 prediction the story's own record labels refuted
- [x] [Review][Patch] AC5 promised *"a test that reds"*; what shipped is a compile-time guard
- [x] [Review][Defer] a `..` in `contradicts`' pattern retires the compile guard with the suite green — registered, owner named
- [x] [Review][Defer] the dead seed and the `SEED_SWEEP` fold — registered, owner is whoever widens the sweep
- [x] [Review][Defer] `permutations` materialises all `n!` vectors eagerly — registered at implementation
- [x] [Review][Defer] `ContradictoryObservation` names the condition, not the instance — consistent with 5.11's `InstantRegressed`
- [x] [Review][Dismissed] *"DB tests may not run in CI"* — refuted: `ci.yml:24` exports `DATABASE_URL` against a `mariadb:10.11.11` service
- [x] [Review][Dismissed] *"`rand`'s transitive provenance is unverified"* — verified: only `sqlx-postgres` and `surge-ping`
- [x] [Review][Dismissed] `expected_pairs.len() == 15` is derivable — deliberate redundancy, house idiom

**T8 — docs and register.** The register carries **ONE** bullet for both §2 dependencies
(`deferred-work.md:2704-2711`, owner 5.11b) and none for §5 — dispose of that bullet and open what
§5 and §6 raise. ⚠️ That bullet also says *"invisible to all **425** tests"* where `master` carries
**429**; correct it while you are in the file.
⚠️ `resolver.rs:635` states *"Nothing here reads `fixtures/`"* as a property — **T5 falsifies it**.
Update that sentence in the same commit, or put the corpus tests elsewhere. Then the twins (AC8).

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

Claude Opus 5 (1M context) — `claude-opus-5[1m]`, via `dev-story`, 2026-08-06.

### Debug Log References

**Environment.** Built and mutated against a live `mariadb:10.11.11` on host port **13306**
(container `opencmdb-5-11b`). Verified load-bearing rather than assumed: the same test runs with
`DATABASE_URL` set and prints `skipping resolver test: DATABASE_URL unset` without it.

**Counts.** `master` 429 (224 bin + 159 core + 46 xtask) → 446 at implementation → **450
(245 + 159 + 46)** after the code review.

**Permutation counts and sample sizes, as actually consumed.**

| shape | slice | permutations | note |
|---|---|---|---|
| A, synthetic | `order_fixture()`, 6 obs | **720**, exhaustive | pure, no database; no test times it, so no figure is claimed |
| A, corpus | `hostname-absence.jsonl`, 6 obs | **720**, exhaustive | the largest committed stream; derives 5 interfaces |
| B, purge-and-replay | `order_fixture()` | **12**, sampled | `skip(1).step_by(60)` — never the identity |
| C, no-op | `order_fixture()` | **12**, sampled | same sample |
| C, reference scale | 300 obs over 100 MACs | **8 seeds** (`SEED_SWEEP`) | 692 ms here, 561 ms on the auditor's machine — environment-dependent, asserted by nothing |

**The mutation table, measured. Ten rows, not eight: two sub-measurements were required to tell
what a red was actually carried by, and one mutation was added to check a claim this story's own
doc makes.**

| | mutation | measured HERE | predicted | carrier |
|---|---|---|---|---|
| M1 | `join`'s grouping keyed on SLICE POSITION (`identity/l1.rs`) | **23 red**, incl. both AC1 tests | AC1 reds *inside the loop* | assertion |
| M1-loop | M1 with AC1's two pre-loop oracles neutralised | **red at permutation 1** | — | assertion |
| M1-fkw | `join` keeps only the first key (first-key-wins) | **10 red; the CORPUS AC1 test stays GREEN** | §6 said exactly this | assertion |
| M1-noop | resolver consumes `join`'s groups in slice order | 🔴 **ENTIRE SUITE GREEN, 446/446** | green | none |
| M2 | witness = first-by-arrival (slice plumbed down 3 levels) | **4 red**: AC2, AC3, reference-scale, `the_write_amplification_…` | 3 red | assertion ×4 |
| M3a | the SHUFFLE returns its input | **3 red**: 2 shuffle guards + the reference-scale consumer | 2 red, AC1–AC3 green | assertion |
| M3b | the ENUMERATOR returns its input | **7 red**: 3 enumerator guards + 4 consumers | 4 consumers red | assertion |
| M3b+ | M3b with **all six** count assertions deleted | the 4 consumers go GREEN; only the 3 guards red | exactly this | assertion |
| M3b+4 | M3b with only the **four `consumed`** assertions deleted | 🔴 **the 2 DATABASE consumers still red** — `sampled_permutations` guards them a second time | — | assertion |
| M4 | drop the repeated-`obs_id` refusal | **1 red**, AC5's database test only | AC5 only | assertion |
| M5 | seed the sweep from the clock | 🔴 **first run: ENTIRE SUITE GREEN** → **1 red** after the new guard | said the golden test caught it | assertion |
| M6 | sample one permutation instead of enumerating | **7 red** (same edit as M3b) | 🔴 green | assertion |

**Zero compiler-carried reds.** One compile failure occurred and was an artefact of the mutation
DRIVER, not a measurement: M5's blanket rename hit a `use` statement. Repaired and re-run.

`COUNTS_DELETED` alone, with no other mutation: **entire suite green** — the count assertions
constrain nothing on correct code, which is what makes them cheap and what makes M3b/M6 visible.

🔴 **There are SIX count assertions, not five, and the difference is load-bearing** — found by the
Acceptance Auditor and re-measured on both variants. Four are the `consumed` counters in the
consuming tests; the fifth is `seeds == 8` in the reference-scale sweep; the sixth lives inside the
helper `sampled_permutations` (`sample.len() == DB_PERMUTATION_SAMPLE`) and guards shapes B and C a
SECOND time. So *"delete the count assertions and the four consumers go green"* is true only if the
helper's is among them: deleting just the four `consumed` lines leaves the two database consumers
**red**, at `left: 0, right: 12`. The code is better than the claim was; the claim is what was
wrong, and it stood in five documents.

### Completion Notes List

- **Three measurement shapes, all built.** A (pure, exhaustive) is the only one that covers the
  derived interface SET; B and C both start from a store an in-order pass built. C is the strongest
  and exists only because story 5.11 shipped idempotence — and it is **not** a duplicate of 5.11's
  test: under M2, 5.11's `a_second_identical_pass_writes_nothing_at_all` stayed GREEN while shape C
  reddened. Measured, as the story predicted.
- 🔴 **M5's prediction was REFUTED, and the refutation is the story's best finding.** The
  golden-value test does not guard the seed sweep's provenance: it pins `shuffled` at a hardcoded
  seed and never reads `SEED_SWEEP`. A clock-derived sweep left **all 445 tests green**. Closed by
  `the_seed_sweep_is_the_fixed_range_it_claims_to_be`, which reads the constant's values. *Two
  different properties had been conflated: reproducible WITHIN one process is trivially true for
  every seed; reproducible ACROSS runs is what a fixed seed buys.*
- 🔴 **M2 exposed a gap in this story's own code.** `shuffled` and `SEED_SWEEP` had **no consumer
  outside their own tests**, so AC4's *"the reference-scale slice uses a fixed seed sweep"* was
  satisfied by nothing. Added `the_reference_scale_pass_is_order_independent_across_the_seed_sweep`:
  300 observations over 100 MACs — groups of THREE, because the existing reference-scale test gives
  every observation its own MAC and a singleton group never exercises the witness convention.
- **M1 diverges from its prediction in WHERE it reds.** The story says *"inside the permutation
  loop"*; it reds first on the pre-loop oracle (`4` groups expected, `1` obtained). The loop was
  measured separately with the oracles neutralised and reds at **permutation 1**, so both are
  load-bearing and the oracle simply fires first. Stated because a divergence is a finding.
- **M6 reds here where the story measured it green** — 7 tests, all on count assertions. That is
  §5.3 working as designed, and `M3b+COUNTS_DELETED` proves it: delete the five count assertions and
  the four consumers go green again.
- **AC5's refusal is narrow, and both exclusions are measured.** `raw` and the ORDER of `facts` are
  excluded, each with a test asserting BOTH that `contradicts` accepts it AND that a bare `a != b`
  would have refused it — without that second half, nothing would notice `contradicts` being
  replaced by `!=`. The two existing repeated-`obs_id` tests (5.9b's, 5.11's) both still pass, which
  is the story's own test that the refusal is not too broad.
- **`contradicts` destructures all six `Observation` fields with no `..`**, so a new field is a
  COMPILE error at that site rather than a silent omission.
- **AC7 verified by inspection, not intention.** `identity/` and `fixtures/` carry no change in the
  shipped diff (`git status` over both is empty); `Cargo.toml`/`Cargo.lock` have an empty diff; the
  trap corpus is still **11 unanswerable with `passed() == false`**; `float-free` still walks 4 files
  under `identity/`. `main.rs` gains the one `#[cfg(test)] mod permute;` line the AC names.
- **`resolver.rs`'s module doc said *"Nothing here reads `fixtures/`"*** — falsified by T5 and
  corrected in the same commit that falsified it, with the old sentence quoted.

### File List

| file | change |
|---|---|
| `crates/opencmdb-bin/src/permute.rs` | **new** — `permutations`, `shuffled`, `SEED_SWEEP`, and their eight guards |
| `crates/opencmdb-bin/src/main.rs` | one `#[cfg(test)] mod permute;` declaration, no behaviour |
| `crates/opencmdb-bin/src/resolver.rs` | the refusal + `contradicts`; shapes A/B/C; the corpus test; the module-doc correction |
| `crates/opencmdb-core/src/repo/mod.rs` | `RepositoryError::ContradictoryObservation` |
| `_bmad-output/implementation-artifacts/deferred-work.md` | the 5.11b bullet CLOSED; six new entries |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | status |
| `_bmad-output/implementation-artifacts/5-11b-order-independence.md` | this record |
| `CLAUDE.md`, `docs/project-context.md` | the doc twins (AC8) |

---

## Senior Developer Review (AI)

**Date:** 2026-08-06 · **Outcome:** Changes Requested → **applied** · **Layers:** three, in parallel
(Blind Hunter diff-only, Edge Case Hunter with a live `mariadb:10.11.11`, Acceptance Auditor against
the spec). **446 → 450 tests.** Six gates green, both clippy forms clean.

### What the review was for, and what it found

🔑 **Two layers independently found the same two defects** — the false `XorShift64` doc and the
corpus test's weak oracle. Independent convergence is the strongest signal this process produces,
and neither was reachable by reading the story alone.

🔴 **The story's own headline defect recurred inside the story.** `SEED_SWEEP`'s doc comment asserted
that the golden-value test guarded its provenance and that a clock-derived sweep left the suite
green — both refuted by a test **eighty lines below it in the same commit**. This is the sixth
consecutive story in this project where a doc was falsified by a measurement in its own commit, and
the first where the falsifying measurement was the story's own deliverable.

### The four defects that were behaviour, not prose — each measured before its patch

| # | defect | how it was found | now |
|---|---|---|---|
| 1 | *"the refusal happens before anything is written"* was guarded by **nothing** — moving the `return Err` to the END of `resolve_within` left **446/446 green**, because every test runs inside `transact`, which rolls back on `Err` | Edge Case Hunter, by mutation | `the_refusal_precedes_every_write_even_without_a_transaction` runs the pass on a POOLED connection under autocommit. Mutation R1 reds it, and only it |
| 2 | no order test touched the `interface` table's derived instants — making `seen_window` follow arrival order left **all five** order tests green | Edge Case Hunter, by mutation | shape C now compares `interface_windows` on every permutation. R4 reds it |
| 3 | the `facts.len()` term in `contradicts` was **droppable with the suite green** — it is the only term catching `[x]` vs `[x, x]`, and nothing tested that | Edge Case Hunter, by mutation | the multiplicity case is asserted. R2 reds it |
| 4 | the corpus test's `join` oracle was `!is_empty()` where its synthetic twin pins an exact count — a `join` collapsing everything onto one interface left it **green** | Blind Hunter + Edge Case Hunter | pinned at **5** interfaces (measured). R3 reds it |

### Three claims of mine that measurement refuted

- 🔴 **`M3b+` was wrong in five documents.** *"Delete the five count assertions and the four consumers
  go green"* is true only if the fifth is the one inside `sampled_permutations`. Deleting just the
  four `consumed` lines leaves the two database consumers **red** at `left: 0, right: 12` — there are
  **six** count assertions, and shapes B and C are guarded twice. Found by the Auditor, reproduced on
  both variants. *The code was better than the claim; the claim was the defect.*
- **`M1-fkw` was 10 red, not 11.** I transcribed the driver's raw line count, which includes
  `result: FAILED`, for that one row while subtracting correctly for the other nine.
- **The timing figures rested on no check.** 692 ms became 561 ms on the auditor's machine; the
  "~20 ms" and "11.5 ms" figures are timed by no test. Now stated as environment-dependent, or
  removed.

### Also patched

`XorShift64`'s doc claimed *"state is never zero"*, false for seed `0x9E37_79B9_7F4A_7C15` — the
golden-ratio constant, which is exactly what someone reaching for "a good seed" reaches for; the
degeneration is SILENT (it becomes a fixed rotation, so the multiset and reproducibility guards both
pass). Now pinned, with the load-bearing half asserting that **no seed in the sweep** reaches the
fixed point — R5 reds it. The `seeds == 8` message claimed to catch a clock-derived sweep, which it
cannot. The golden test's doc pointed at *"the test above"* positionally. `factorial`'s `.max(1)` was
unreachable. `sampled_permutations`' constants are tuned to `n == 6` and now say so. The `raw`
carve-out's order-independence was an argument and is now a measurement.

### Dismissed, with the check that dismissed it

- *"The database-gated tests may not run in CI"* (Blind Hunter, HIGH) — **refuted**: `ci.yml:24`
  exports `DATABASE_URL` against a `mariadb:10.11.11` service.
- *"`rand`'s transitive provenance is unverified"* (Blind Hunter, LOW) — **verified**: only
  `sqlx-postgres` and `surge-ping` depend on it in `Cargo.lock`.
- The Edge Case Hunter refuted **eight** of its own suspicions by running them, including silent
  sample truncation, the refusal's transitivity over three copies, and whether fact ORDER reaches a
  stored column. Recorded because a refuted suspicion is a measurement too.

### Per-AC verdict after patching

AC1–AC4, AC6, AC7, AC8 **MET**. **AC5 PARTLY MET and corrected rather than ticked**: the refusal,
its naming, the `raw` exclusion and both pre-existing repeated-`obs_id` tests are verified, but the
"test that reds when a new field is added" is a COMPILE error, and a `..` in the pattern retires it
with the suite green. The AC text now says what shipped, and the residue is registered.

---

## Change Log

| Date | Change |
|---|---|
| 2026-08-07 | **DONE** — PR #73 squash-merged as `f597882` after a green CI run (1m10s). Epic 5 is now 15/18; `master` carries **450 tests**. ⚠️ CI did not trigger on the first push — no run was created for the branch or the PR, and none anywhere in the repo for ~15h, while the workflow was `active` and GitHub reported Actions `operational`; closing and reopening the PR re-triggered it and it passed first time. **The cause is not guessed**, per this project's own rule. |
| 2026-08-06 | **Code-reviewed** — three layers, **446 → 450 tests**, six gates green; stays `review` because `done` is the MERGE's business. 🔑 Two layers independently found the same two defects. 🔴 **The story's own headline defect recurred inside it**: `SEED_SWEEP`'s doc was refuted by a test eighty lines below it in the same commit. 🔴 **Four behavioural defects, all found by MUTATION**: the *"refusal precedes every write"* claim was guarded by nothing (moving the `return Err` to the end left 446/446 green — `transact` rolls back); no order test touched the `interface` table's derived instants (all five stayed green under an arrival-ordered `seen_window`); the `facts.len()` term was droppable; the corpus oracle was `!is_empty()`. 🔴 **Three of my own claims refuted**: `M3b+` was wrong in five documents (there are SIX count assertions, not five), `M1-fkw` was 10 not 11, and the timings rested on no check. AC5 corrected to PARTLY MET rather than ticked. Two findings dismissed with the check that dismissed them; the Edge Case Hunter refuted eight of its own. |
| 2026-08-06 | Implemented by `dev-story` against a live `mariadb:10.11.11` on 13306. **429 → 446 tests**, six gates green, both clippy forms clean, `Cargo.lock` diff empty. Three measurement shapes plus a reference-scale seeded sweep; the one production change is the repeated-`obs_id` REFUSAL (`RepositoryError::ContradictoryObservation`). 🔴 **M5's prediction was refuted by measurement**: the golden-value test does NOT guard the seed sweep's provenance — a clock-derived sweep left all 445 tests green — and a new guard reading the constant's VALUES closes it. 🔴 **M2 exposed that `shuffled` had no consumer at all**, so AC4's reference-scale half was satisfied by nothing; a 300-observation, 100-MAC, eight-seed test now consumes it. M6 reds here where the story measured it green, which is §5.3's count assertions working. Ten mutation rows measured, **zero compiler-carried reds**. |
| 2026-08-05 | Validated by two fresh-context agents; **14 findings applied, 5 HIGH**. The gap-hunt BUILT the story (429 → 441 tests, six gates green, no new dependency). 🔴 **Four prescribed mutations were measured leaving the entire suite GREEN** — M1 could not reach a pure test, M5 could not see the seed's provenance, M6 made AC1 a tautology, and a degenerate enumerator slipped past every consumer. Guy's arbitrations: a mutation MAY edit `identity/` (the ban is on the shipped diff), and AC5's comparison EXCLUDES `raw`, which no decision reads. What held: shape C is a no-op and is not a duplicate of 5.11's test, shape B's `interface_id` claim holds, and 720 permutations run in 11.5 ms. |
| 2026-08-05 | Created by `create-story`. Five decisions at contexting, the load-bearing one being that **the corpus streams carry 3–6 observations, so permutations can be enumerated EXHAUSTIVELY rather than sampled** — no `rand`, no seed, no flakiness, and strictly stronger than the fuzz `epics.md` asks for. The story is designed around one failure mode: a test that cannot fail. |
