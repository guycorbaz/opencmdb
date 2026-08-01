# Story 5.6: The blocker, and the recall assertion nobody writes

Status: done

<!-- Validation is MANDATORY here (Guy's decision, Epic 4 retrospective 2026-07-26): two
     fresh-context agents (fact-check + gap-hunt) BEFORE dev-story. The template banner saying
     "Validation is optional" does not apply to this project. Story 5.5 measured the payoff again:
     6 HIGH findings, and EVERY ONE came from the agent that COMPILED the story, none from the
     agent that checked its claims. Point the gap-hunt agent at AC1, AC3 and AC4. -->

## Story

As the identity engine,
I want candidate generation to be an **explicit component** with a **measured recall floor**,
so that a false split cannot be born silently before any rule has had a chance to speak — and so
that abstention finally has a denominator.

**This story writes the blocker and the recall assertion. It answers no pair and scores no trap.**
Story 5.7 owns the corpus wiring (`score_corpus`, `run_trap`, `Decision -> Outcome`); 5.8 owns the
unanswerable-level bucket; 5.9 persistence; 5.14 the operator surface; Epic 6 the `l2-*` rules. The
build order, quoted from `epics.md:1317`: *"the three debt stories (5.1, 5.2, 5.2b) -> the engine's
vocabulary (5.3, 5.4) -> the verdict algebra (5.4b) -> the pure join (5.5) -> **the blocker (5.6)**
-> wiring it to the corpus (5.7, 5.8) -> persistence (5.9, 5.10) -> the invariants (5.11, 5.12,
5.13) -> the operator-visible surface (5.14)"*.

**Nothing under `fixtures/` moves.** No artefact bytes, no `MANIFEST.toml`, no re-hash. This story
**reads** the corpus (that is AC4) and never writes it. If any step appears to require re-authoring
a committed artefact, **STOP** — that is a finding, reported rather than absorbed.

**`architecture.md` is NOT edited** (issue #54 for D13's short table; a milestone act).
**`architecture-views.md` is NOT regenerated** (issue #50). **`epics.md` is verify-only — an edit
there is a finding.**

⚠️ **Branch from `master` only after PR #58 merges.** Measured at contexting: the working branch is
`bookkeeping-5.5-done`, one commit ahead of `master`, and **PR #58 is OPEN, not merged**. `master`
is at `0ebd50f` with **309 tests**.

## What this story inherits, measured rather than assumed

### 1. 🔴 The epic's own assertion reds the `float-free` gate — and this was measured, not reasoned

`epics.md:1507` gives this story the assertion `blocking_recall >= 0.999`. Story 5.5 flagged it
forward on purpose (`deferred-work.md` §*Deferred from: story-5.5*, *"Owner: story 5.6, at
contexting, so it is a decision and not a surprise"*). **This is that decision.**

Measured at contexting by writing a probe file under `crates/opencmdb-core/src/identity/` and
running `cargo xtask ci` — the probe was deleted and `git status` verified clean afterwards:

| line, as written in the probe | gate |
|---|---|
| `let x = 0.999;` | 🔴 **bare float literal** |
| `assert!(true, "blocking_recall >= 0.999");` | 🔴 **bare float literal** — quoting the epic's AC *in an assertion message* reds |
| `assert!(true, "story 5.6 owns the blocker");` | 🔴 **bare float literal** — a story number with no letter suffix, inside a string, on a code line |
| `assert!(true, "story 5.7 answers it");` | 🔴 **the same** — the rule is *any* story number without a letter suffix, **not** just this story's |
| `/* … 0.999 … */` · `#[doc = "… 0.999 …"]` | 🔴 — block comments are **not** stripped, and `#[doc]` is a code line |
| `/// blocking_recall >= 0.999 in a doc comment` · `//! …` | ✅ green — `//`, `///` and `//!` are stripped |
| `assert!(true, "story 5.6b owns the blocker");` | ✅ green — **only** because of the trailing `b`; do not generalise |
| `const A: usize = 999;` · `const B: usize = 1000;` | ✅ green |
| `let recall_per_mille = 1000 * 10 / 10;` and `assert!(recall_per_mille >= 999, "…")` | ✅ green |
| `fn blocking_recall_above_999() {}` | ✅ green — `999` is preceded by `_`, an identifier character |
| `"architecture.md:1004-1007"` · `":988-993"` · `":1246-1252"` · `"epics.md:1317"` · `"l1.rs:16-18"` | ✅ green **even on a code line** — a hyphenated range is no numeric literal |
| `"n*(n-1)/2"` · `"2 of 3 is 666"` · `"v0.1.1"` | ✅ green |

The first probe reported exactly three offenders, all three the predicted ones; the rows added above
were measured by a second probe during validation, which wrote AC6's required module doc in full and
found it **green** — every citation it must carry survives, and the prose lives in `//!` anyway.

⚠️ **`"v0.1"` — one dot, no third segment — was NOT measured.** Do not infer it from `"v0.1.1"`.

⇒ **The floor is expressed as an INTEGER in per-mille**, and the gate is not touched, not weakened,
not skipped and not given an escape hatch. Two independent grounds, neither invented here:

1. **D13's own corollary** [architecture.md:988-993]: *"`confidence` is an **INTEGER in milli-units
   (0..1000)**, never `REAL`/`DOUBLE` — a threshold at 0.85 compared as a float on two engines = two
   different identity decisions for the same input."* Story 5.4b registered this as the corollary
   that *"binds the day a float would otherwise appear"*. **This is that day.**
2. **The architecture already named the test**, and its name carries no float:
   `blocking_recall_above_999` [architecture.md:2954], listed among the ratified test names with the
   rule *"an invariant reads as a claim, so a red test names a broken claim."* **Use that exact
   name.** It is the architecture's, not this story's.

⚠️ **Do not weaken, disable, `#[allow]` or narrow the gate to make room for the assertion.** Story
5.4b's review named this exact failure mode: *"the wrong choice here is the one that gets the gate
weakened or deleted in its second week."* The gate walks **3** `.rs` files under `identity/` today
and must walk **4** after this story.

### 2. 🔴 D18 refuses a "recall gate" by name — and this assertion is not one. The story owes the distinction.

A reviewer will find this and it must be answered in the doc, not discovered in review.
[architecture.md:1246-1253] puts **pairwise recall** in Tier 2, *"published per release with
confidence intervals, trended — **blocking nothing**"*, with the reason spelled out:

> pairwise recall — *"false-split is benign — so why would it block a release? **A loose threshold
> on a benign defect is a gate that can never fall, and a gate that cannot fall is decoration.**"*

And NFR4 [prd.md:1179-1207] generalises it: at n=300, *"**any fraction is theatre**"* (`:1182`), so the
release gate is **truth-table failures = 0**, binary, at the **device** level.

**Three differences make D13's `blocking_recall` legitimate where D18's pairwise recall is not, and
all three must be stated at the function:**

- **Different subject.** D18 measures the ENGINE'S OUTPUT (did it group what should group). This
  measures the CANDIDATE GENERATOR'S INPUT COVERAGE (did the pair even reach a rule). D13 names the
  gap: *"if the candidate generator does not propose the pair, no downstream logic can ever group.
  That is where false-splits are born silently, and **nobody tests blockers**"*
  [architecture.md:1004-1007].
- **Different venue.** D18 refuses a *release gate over bulk statistics*. This is an assertion in a
  unit test over the frozen corpus — D13 says so in the same breath: *"a dedicated assertion:
  `blocking_recall >= 0.999`, **measured in unit tests, before the scoring exists**."*
- **Different arithmetic, and this is the honest half.** At the committed corpus's denominator the
  floor is **not** a tolerance: with 10 required pairs, one miss gives 900‰ and the floor reds.
  **`>= 999‰` IS zero-tolerance at this scale**, which is exactly the binary form NFR4 demands. It
  only becomes a 0.1% tolerance if the required set ever exceeds 1000 pairs — **and on that day
  NFR4's *"any fraction is theatre"* bites and the floor must be revisited rather than inherited.**
  Say that; do not let the per-mille dress imply a statistical tolerance the corpus cannot support.

⚠️ **Do not claim this advances NFR4.** NFR4 is at the **device** level and story 5.8 reports it NOT
MET for this epic. This story adds no truth-table column and gates no release.

### 3. The corpus is the truth set, and it was measured — 24 traps, 23 pairs, 10 must-merge, 7 sharing a MAC

Counted at contexting by parsing `fixtures/scenario/traps/*.toml` against the streams they name:

| quantity | measured |
|---|---|
| committed traps | **24** across 10 files |
| traps naming exactly **2** observations | **23** |
| traps naming **1** observation | **1** — `example-must-abstain` (`example.toml:40`, `replay = "scenario/replay/minimal.jsonl"`) |
| `must-merge` traps | **10** — one per family for the **nine** named families, plus `example-must-merge`, whose `family` is `None` (`trap.rs:133-141`: *"a format/example trap that is part of no family"*) |
| `must-merge` pairs whose two observations **share a MAC** | **7** |
| `must-merge` pairs that share **no** MAC | **3** — `multi-nic`, `shared-hardware-vm`, `docker-veth`, exactly the three whose expected rule is `l2-*` |
| trap pairs whose two observations are in the **same** `Scope` | **23 of 23** |
| replay streams named by a trap that carry a control record (`failure`/`capability`) | **0** — so `read_jsonl` is the right reader |
| trap pairs listing their two ids in an order the stream does not | **0 of 23** — and 0 out of ascending-UUID order either; **this is what makes M4 corpus-invisible** (AC7) |
| distinct `obs_id`s across the **11** streams a trap names | **39, zero collision** — the property AC4's union depends on |

Two consequences the dev must carry:

- **The 3 `l2-*` pairs belong in the recall denominator.** *Proposing* is not *answering*: story 5.8
  buckets the `l2-*` traps as unanswerable at this level, but a pair the blocker never proposes can
  never be answered by Epic 6 either — that is precisely the false split D13 names. **The recall
  truth set is all 10 `must-merge` pairs, not the 7 an L1 engine can answer.**
- **`sameScope` = 23/23 means the corpus cannot judge the scope question at all.** See §5.

### 4. 🔴 The corpus can only be read from `opencmdb-bin` — the frontier decides where each test lives

D47 is a gate: `opencmdb-core` may not touch the filesystem, and `read_traps`/`read_jsonl` live in
`crates/opencmdb-bin/src/fixtures.rs` (`:665`, `:647`). The blocker itself belongs in **core** — the
architecture names the file [architecture.md:3368]. So the story splits, and the split is forced:

| where | what | why |
|---|---|---|
| `crates/opencmdb-core/src/identity/blocking.rs` | the pair type, the generator, the recall function, the floor constant, and **synthetic** tests | the architecture's own file; core cannot read `fixtures/` |
| `crates/opencmdb-bin/src/fixtures.rs`, **test module only** | `blocking_recall_above_999` and the universe-coverage test, over the committed corpus | the corpus-wide walks already live there (`walk_trap_files`, `#[cfg(test)] pub(crate)`, `:834`), added by stories 5.1/5.2/5.2b |

⚠️ **Not `trap_gate.rs`.** Its `score_corpus` is story 5.7's seam and its code is off limits.
⚠️ `fixtures.rs` is the largest file in the tree, but the `file-size` gate counts only the lines
**before the first `#[cfg(test)]`** (`:729`), and this story adds none of those.

**Why the truth set is the corpus and not synthetic pairs**, in one sentence the doc should carry:
Epic 4 froze the corpus **before** the engine on purpose — *"a metric written after the engine is
bent to fit the engine"* — so a recall floor measured against a truth set the engine's own author
writes today is the mirror D13 refuses for weights, applied to blocking.

### 5. 🔴 The wrong blocker that passes the entire corpus

Story 5.5's equivalent was the bare-MAC key. This story's is **blocking on `l2_domain`**: every
committed trap pair is in one scope (23/23, measured), so a generator that proposes only same-domain
pairs scores **1000‰** on the corpus and is invisible to every corpus test.

It is wrong because a device's interfaces are not confined to one L2 domain — a router, a firewall
or a dual-homed server has NICs in several VLANs, and D12 makes the device the level where the
product keeps its promise [architecture.md:919-928]. Excluding cross-domain pairs would **build a
false split into the universe**, which is the one thing the blocker exists to prevent.

⇒ **The universe must contain cross-domain pairs, and the only thing standing between that and green
is a synthetic two-domain test. Write it first.**

⚠️ **Do not resolve this by consulting L1.** L1 will answer such a pair `l1-distinct-mac` ->
`Disqualifying` -> `NoMatch`, and that is correct **at L1**, about *interface* identity. The blocker
proposes; it does not judge.

### 6. 🔴 L1 already contradicts a committed `must-merge` trap, and that is NOT a bug to fix here

`multi-nic-must-merge` expects a merge via `l2-uplink-agrees`, and the two observations carry
different MACs — so `decide_pair` on that pair yields `l1-distinct-mac` -> `Disqualifying` ->
`NoMatch`. The same holds for `shared-hardware-vm-must-merge` and `docker-veth-must-merge`.

This is **by design**: D12 splits the levels — *"multi-NIC false-split = L1 correct, L2 failed to
group"* [architecture.md:893] — and `multi-nic.toml`'s own header says it, in the committed bytes:
*"This family lives at L2 (device grouping) … never at L1, which is right to keep two distinct MACs
apart."* Story **5.8** owns the bucket that counts an `l2-*` trap as NOT PASSING.

⇒ **If you find yourself changing an L1 verdict, widening `decide`, or reconciling a level, STOP.**
This story neither calls `decide_pair` nor compares a verdict to a trap.

### 7. `grep '5\.6'` over the code returns **ZERO** — the doc worklist is by MEANING, not by grep

Measured: `grep -rn '5\.6' crates/ xtask/ --include=*.rs` returns **one** hit and it is unrelated
(`arp_ping.rs:272`, *"128 * 200ms = 25.6s"*). The idiom story 5.5 used — start from the grep — yields
an **empty worklist here** and would ship four falsified claims. The sites were found by reading:

| site | the claim this story falsifies |
|---|---|
| `identity/mod.rs:15-16` | *"**There is still no candidate pair generator**… the blocker that would propose pairs is the next story's"* |
| `identity/cascade.rs:292-293` | *"There is still **no blocker**: the pair it answers arrives from its caller, and candidate generation is the next story's"* |
| `identity/l1.rs:16-18` | *"the blocker is an L2 organ… and it **is the next story's**"* — the first half stays TRUE (l1 still generates nothing); only the ownership clause moves |
| `identity/l1.rs:287` | *"which is the **next story's** organ"* |
| `lib.rs:46-47` | *"its consumer (the candidate generator) **does not exist yet**"* — the generator exists after this story, **and does not consume [`join`]**. The reason changes; the conclusion (no flat re-export) survives. ⚠️ Getting this one right means saying what is actually true: `join`'s consumer is still story 5.7's harness. |

⚠️ **This is a floor, not the set.** Re-read the module doc of every file you touch. Story 5.4b's
review measured that a grep-based enumeration of falsified doc sites is not reproducible.

### 8. Two register entries name this story as owner, and both are real

Enumerate with `grep -n '5\.6' _bmad-output/implementation-artifacts/deferred-work.md` — measured
**4 lines** (`:1629`, `:1632`, `:1633`, `:1684`), in **two** entries. **Do not use `grep 'story 5\.6'`**: owner strings wrap across newlines
and that is exactly how story 5.4b came to claim eight register entries where ten existed.

- **(R1) the float** — §1 above. Closed by this story's per-mille decision.
- **(R2) the self-pair** — §*code review of story-5.5*: *"`verdict_for_pair(a, a)` — the self-pair is
  answered but undocumented… `decide_pair`'s doc tells a future candidate generator that the pair
  'arrives as an argument' without telling it that excluding `i == j` is the generator's
  responsibility. **Owner: story 5.6**, which writes that generator and is the first place the
  precondition has a holder."* ⇒ **the exclusion is this story's, and it belongs in the type**
  (AC2), not in a comment.

Two further entries mention the blocker without owning it — read them, do not close them:
`&str` rule-id constants allocate *"on a function a blocker will call O(pairs) times"* (owner: a
condition), and story 5.5's `L1Key` bare-tuple criticism (owner: 5.9).

### 9. What already exists, so that nothing is re-created

- `join(&[Observation]) -> BTreeMap<L1Key, BTreeSet<ObsId>>` (`l1.rs:165`) — **the blocker does not
  call it**; see AC1. `L1Key = (L2DomainId, MacAddr)` (`:86`).
- `decide_pair(a, b) -> Decision` (`l1.rs:288`), `verdict_for_pair` is `pub(crate)` — **neither is
  called by this story**.
- `Observation { obs_id, scope: Scope { l2_domain, vantage }, facts, observed_at, connector_id, raw }`
  (`observation/mod.rs`). `ObsId` is `Ord` (the join's `BTreeSet<ObsId>` proves it).
- `read_jsonl(&Path) -> Result<Vec<Observation>, FixtureError>` (`fixtures.rs:647`),
  `read_traps(&Path) -> Result<TrapFile, FixtureError>` (`:665`, and it already cross-checks that
  every `obs_id` a trap names exists in its stream), `walk_trap_files` (`:834`, `#[cfg(test)]`).
- `Trap { id, replay, observations: Vec<ObsId>, reason, expect, family }` and
  `Expectation::{MustMerge{rule}, MustNotMerge{rule}, MustAbstain{cause}}` with
  `Expectation::column()` (`trap.rs:69-141`).
- **There is no `dormant` anywhere**: `grep -rn 'dormant\|Dormant' crates/ xtask/ --include=*.rs`
  returns nothing. So [architecture.md:1205-1206] — *"the blocker excludes `dormant` from automatic
  candidate generation"* — **cannot be implemented here**; it is registered with an owner (AC8), not
  written from belief (D45).

## Acceptance Criteria

### AC1 — The blocker is an explicit component, and its universe is TOTAL by decision

**Given** a slice of `Observation`s
**When** the blocker runs
**Then** it returns every unordered pair of observations with **distinct** `ObsId`s, deterministically
and reading nothing but its argument: no clock, no I/O, no SQL, no repository, and not `raw`.

```rust
pub fn candidates(observations: &[Observation]) -> BTreeSet<CandidatePair>
```

- **Total by DECISION, not by omission — and the doc says so.** D13: at 300 hosts *"the blocker is
  **not** there for performance (90k pairs is noise on a NAS i5)"* [architecture.md:1009]. Every
  exclusion the component makes is named and tested; there are exactly two, both in AC2. **A
  narrowing key (MAC, hostname, domain) is NOT added** — §5.
- **It does not call [`join`], `verdict_for_pair` or `decide_pair`.** Proposing is not judging, and a
  blocker that consults a rule is the rule's echo. ⚠️ The relation between the two organs is still
  pinned, in the other direction: **every pair sharing an L1 key is in the universe** (AC7), which is
  a property of the total universe and would red the day someone narrows it.
  🔴 **The prohibition binds `candidates`, NOT the module.** AC7's test 9 imports `join` on purpose —
  `use crate::identity::l1::join;` inside `blocking.rs`'s test module, same crate, `join` is already
  `pub`, no circular import and no visibility change (measured: clean under both clippy forms). A
  superset property is not checkable without the thing it is a superset of. Do not drop test 9, and
  do not re-implement the join to avoid the import.
- **`BTreeSet`, so order-independence and de-duplication hold by CONSTRUCTION**, not by a `sort()` a
  refactor can drop — the reasoning that made `join`'s value a `BTreeSet` and `l1`'s evidence sorted.
- **The count is exactly `n*(n-1)/2` where `n` is the number of DISTINCT `obs_id`s in the slice** —
  not `observations.len()`. The two coincide until a duplicate id appears (AC2), and a test that
  asserts from `len()` is green today and wrong that day. Assert it from the distinct count.
- ⚠️ **The caller supplies the universe.** The doc states the growth is quadratic in the slice it is
  handed, that D13's 90k figure is for one poll of 300 hosts, and that **the day a caller hands it a
  retention window instead of a poll, the universe must be narrowed — and the recall assertion is
  what makes that narrowing safe.** Register it (AC8); do not build it.

### AC2 — `CandidatePair` is unordered by construction, and refuses the self-pair

**Given** two `ObsId`s
**When** a pair is built
**Then** `CandidatePair::new(a, b) == CandidatePair::new(b, a)`, and `CandidatePair::new(a, a)` is
`None`.

- **The fields are PRIVATE**, ordered internally, with accessors. This is deliberate and it answers
  a live register criticism of `L1Key` — *"a bare tuple alias creates no distinct type, carries no
  invariant, hosts no impl"*. A pair whose order is enforced by its constructor cannot be built
  wrong; a tuple can. (⚠️ That register entry is **owned by 5.9** and is about `L1Key`. Do not report
  it closed — this story neither renames nor wraps `L1Key`.)
- **`Option`, not a panic and not a silent normalisation.** The self-pair is the register's R2, owned
  here: `verdict_for_pair(a, a)` today answers `Decisive` with `evidence = [x, x]` — an observation
  is trivially its own interface — and the generator is where the precondition finally has a holder.
- **The ordering carries NO meaning.** `ObsId` is a UUID; low/high is a construction device, not
  "first seen". Say so, or a later reader will infer chronology from it.
- **Duplicate ids in the input slice are excluded by the same rule**: two entries carrying the same
  `obs_id` produce no pair, because the rule is *distinct id*, not *distinct index*.

### AC3 — The floor is an integer in per-mille, and the gate is untouched

**Given** D13's `blocking_recall >= 0.999` and the `float-free` gate
**When** the floor is written
**Then** it is `999` per-mille as an integer constant, the assertion compares integers, and
`cargo xtask ci` reports **six green gates** walking **4** files under `identity/`.

```rust
pub const BLOCKING_RECALL_FLOOR_PER_MILLE: u32 = 999;
pub fn blocking_recall_per_mille(
    proposed: &BTreeSet<CandidatePair>,
    required: &BTreeSet<CandidatePair>,
) -> Option<u32>
```

- **`Option`, and `None` for an empty `required` set.** A recall with no denominator is **undefined**,
  not perfect: returning `1000` would let the floor pass over nothing, which is the reasoning the
  fixture gate already carries (*"reporting 'nothing to check' on the deletion of the thing being
  guarded is a guarantee the gate does not have"*) and which D13 states outright — *"without
  blocking, abstention has no denominator."* **A test reds if this returns `Some`.**
- **Integer division truncates DOWN**, which is conservative for a floor; state it and pin it with a
  case that truncates (e.g. 2 of 3 -> `666`).
- **`required` is a `BTreeSet`**, so a duplicated requirement cannot inflate the denominator.
- 🔴 **The constant is pinned by an INDEPENDENT literal.** A test asserting
  `BLOCKING_RECALL_FLOOR_PER_MILLE == 999` written as a literal, citing D13's `0.999`. Without it,
  weakening the floor reds nothing — the corpus scores 1000‰, so every assertion that reads the
  constant moves with it. **This is story 5.5's M6 lesson, and it is worth quoting accurately**:
  non-canonical rule ids left the *validation prototype* green **296/296**, because every expectation
  was built from the constant it was checking; the tests 5.5 actually shipped restate the two ids as
  independent literals and **red 10** on the same mutation (`5-5-l1-join-pure.md:836`). The prototype
  is the warning, the shipped form is the remedy — and the floor takes the remedy.
- ⚠️ **No float anywhere, including in assertion messages and `#[doc = "…"]`.** §1's table is the
  measured list. `//` and `///` comments ARE stripped, so D13 may be quoted verbatim in the doc.

### AC4 — The truth set is the committed corpus, and the assertion carries the architecture's name

**Given** the 10 committed `must-merge` traps
**When** `blocking_recall_above_999` runs, in `crates/opencmdb-bin/src/fixtures.rs`'s test module
**Then** for each trap, `candidates()` over the observations of the stream that trap names contains
the pair the trap names, and `blocking_recall_per_mille` over the whole truth set is `Some(1000)`,
which is `>= BLOCKING_RECALL_FLOOR_PER_MILLE`.

- **The test name is `blocking_recall_above_999`** — the architecture's ratified name
  [architecture.md:2954], not a paraphrase.
- 🔴 **PAIRS are formed per stream; the recall is computed over a UNION — and these are two separate
  tests.** `candidates()` never sees two streams at once (a cross-stream pair is meaningless), but
  `required` is 10 pairs drawn from 10 different streams, so the only call that typechecks passes
  `proposed` = the union of the per-stream universes. Two obligations follow, and neither is
  optional:
  - **The per-trap containment assertion is what makes the union honest.** It proves each required
    pair is in *its own* stream's universe, so the union can only add pairs, never explain a miss
    away. Measured backstop: the **39** `obs_id`s across the **11** trap-named streams are all
    distinct (§3), so no coincidental cross-stream hit exists today — but that is a property of the
    committed bytes, not of the code, and the per-trap assertion is the one that would still hold if
    it changed.
  - **Put the containment assertion and the recall value in DIFFERENT tests** — AC7's own rule
    (*"split an assertion that pins two properties into two tests"*) applied here, and it is
    load-bearing: measured during validation, a single test panics on the first missing pair
    (`docker-veth-must-merge: the blocker never proposes the pair this trap requires`) and **never
    computes the recall at all**, so M1's 700‰ is never observed. Two tests, or M1's prediction is
    unobservable by construction.
- **The counts are ASSERTED, not quoted**: the truth set has **10** pairs; **23** of the **24** traps
  name a pair; **exactly one** names fewer than two observations (`example-must-abstain`). Assert
  that last count — a skip that can grow silently is how a gate quietly stops testing.
- **A second, separate assertion — do not call it recall:** *every* trap pair (all 23, the
  `must-not-merge` and `must-abstain` ones included) is in the universe. Story 5.7 must be able to
  answer every trap, and a pair outside the universe can never be answered. D13's recall metric is
  about the merge pairs; this is coverage. **Two names, two assertions.**
- **Nothing in `fixtures.rs` above `#[cfg(test)]` (`:729`) changes.** No new `pub` item in bin.
- ⚠️ **`CandidatePair::new` returns `Option`**; a trap naming the same id twice must fail the test
  loudly (`.expect("a trap names two distinct observations")`), not vanish.

### AC5 — What this story must NOT do

**Given** the seams around it
**When** the work is done
**Then** none of the following has happened, and if any looked necessary it was reported as a
finding rather than absorbed:

- no verdict is produced, no `Decision` is built, `decide`/`decide_pair`/`verdict_for_pair` are not
  called (5.7);
- `score_corpus`, `run_trap`, `Tally`, `Report`, `SourceState`, `Outcome`, `VerdictVectorEntry` and
  `trap_gate.rs` are untouched, and no `From<Decision> for Outcome` appears (5.7);
- no `l2-*` rule, no `InterfaceId`/`EntityId`, no persistence, no `Default` impl anywhere;
- no structural reading is consumed (the U/L bit, the IANA prefixes, the I/G bit) — the blocker
  proposes; the group-address gap stays registered with Epic 6 as owner;
- nothing under `fixtures/` is written.

### AC6 — The doc states WHY the blocker exists, and it is semantics, not performance

**Given** the blocker
**When** its module doc is read
**Then** it states, with D13's words and its citation:

> *"If the candidate generator does not propose the pair, no downstream logic can ever group. **That
> is where false-splits are born silently, and nobody tests blockers.**"* [architecture.md:1004-1007]
>
> *"It is there for **SEMANTICS** — it defines the universe of plausible candidates, hence what
> 'ambiguous' MEANS. **Without blocking, abstention has no denominator.**"* [architecture.md:1009-1011]

- and it names the 90k-pairs-at-300-hosts figure as D13's, with the conclusion D13 draws from it —
  the blocker is **not** a performance device at this scale;
- and it carries §2's three-way distinction from D18's refused pairwise-recall gate, including the
  honest sentence that **at this denominator the floor is zero-tolerance, not a tolerance**;
- and it says what the component does not do (AC5), in the weaker true form.

⚠️ **Do not write an inventory of the epic in this doc.** Say what THIS module does and what THIS
test proves; let the register carry what is open.

### AC7 — Tests, and prove-to-red with the predictions measured at contexting

**Core (`blocking.rs`), synthetic inputs only, inline trailing `#[cfg(test)] mod tests` (D56b).**
⚠️ **The eleven items below are REQUIREMENTS, not a target `#[test]` count** — applying this AC's own
split rule to them yields more functions than eleven (a validation prototype that did so landed on
19 core + 3 bin, `309 -> 331`; that is an order of magnitude, **not a number to hit**). Writing
literally eleven functions is how a red gets lost. AC8's mechanical re-count is the authority.

1. 0 and 1 observation -> **empty** universe (not an error).
2. `n = 4` distinct -> **6** pairs; the `n*(n-1)/2` count asserted for at least two values of n.
3. the same observation twice in the slice -> **0** pairs (distinct **id**, not index).
4. `CandidatePair::new(a, a)` -> `None`; `new(a, b) == new(b, a)`; and the **accessors** are
   exercised on their own — `new(a, b)` and `new(b, a)` yield the same `low()` and the same `high()`,
   with `low() < high()`. (House rule: test every function. The accessors are the only `pub` items
   the eleven items would otherwise leave to incidental coverage.)
5. input-order independence: the same observations shuffled -> the same set.
6. the falsifiable half of purity: varying `raw`, `observed_at` and `connector_id` across otherwise
   identical observations leaves the set identical (the clock/SQL half is unreachable from
   `&[Observation]`, so no test can red on it — say that rather than claiming purity is tested).
7. 🔴 **two observations in DIFFERENT `l2_domain`s are still a candidate pair** (§5) — with an
   assertion that the test data actually varied the domain, so it cannot degrade into a
   single-domain test unnoticed (story 5.4b's measured hole, story 5.5's AC2 idiom).
8. an observation carrying **no** `Fact::Mac` is still a candidate. ⚠️ **Keep this test, drop the
   reason you may have inherited**: the `hostname-absence` family does *not* depend on it — all six
   of its observations carry a `Fact::Mac` (it encodes an absent/empty **hostname**). Measured: the
   only MAC-less observation any trap names is `minimal.jsonl`'s `…-02`, judged by
   `example-must-abstain`. The test is synthetic and stands on the blocker's totality, not on a
   family.
9. **superset of the join**: for a set of observations, every pair inside a `join` group is in
   `candidates`. The test module imports `join` (AC1) — that import is the point of the test.
10. recall arithmetic: full hit -> `Some(1000)`; 9 of 10 -> `Some(900)`; 2 of 3 -> `Some(666)`
    (truncation); empty `required` -> **`None`**; a proposed pair not in `required` does not raise
    the value above 1000.
11. the floor constant equals the literal `999`.

**Bin (`fixtures.rs` test module), over the committed corpus:** AC4's two assertions plus the
one-trap-with-one-observation residue count.

**Mutations — every red reported, not the first, and each classified compiler-carried vs
assertion-carried.** Predictions marked *(measured)* were computed against the committed corpus at
contexting; the others are estimates and a divergence is expected to be REPORTED, not hidden:

- **(M1)** narrow the universe to pairs sharing an L1 key (an "exact-MAC blocker") — corpus recall
  falls to **700‰** *(measured: 7 of the 10 `must-merge` pairs share a MAC)*, so
  `blocking_recall_above_999` reds, and so do the universe-coverage test and several core tests.
  ⚠️ **The 700‰ is only observable if AC4's two assertions are two tests.** Measured during
  validation: with the containment assertion in the same test, it panics on `docker-veth-must-merge`
  before any recall is computed, and the number you would report is the panic, not 700.
- **(M2)** narrow the universe to same-`l2_domain` pairs — 🔴 **the whole corpus stays GREEN**
  *(measured: 23/23 trap pairs are same-scope)*; only the synthetic cross-domain test reds. **This is
  the mutation that proves test 7 is load-bearing**, exactly as M6 was for story 5.5.
- **(M3)** admit the self-pair -> tests 3 and 4 red. ⚠️ **The `n*(n-1)/2` count test does NOT red**
  *(measured)*, and predicting it is a trap: `candidates` only offers index pairs `i < j`, so on test
  2's *distinct* input the `Ordering::Equal` branch is unreachable and the count is unchanged. It
  could only red on a fixture carrying a duplicate id — which test 2's own wording excludes.
- **(M4)** order the pair by argument order instead of canonically -> **the core tests red and the
  corpus stays entirely GREEN** *(measured: `138 passed; 0 failed` on `opencmdb-bin`)*. 🔴 **The
  corpus cannot see this mutation at all** — of the 23 trap pairs, **0** list their two ids in an
  order the stream does not, and **0** out of ascending-UUID order (§3). Do not go looking for a
  corpus red here; there is none to find, and reporting one would be the defect lesson 9 names.
- **(M5)** `blocking_recall_per_mille` returns `Some(1000)` for an empty `required` set -> test 10's
  `None` case reds.
- **(M6)** weaken `BLOCKING_RECALL_FLOOR_PER_MILLE` to `900` -> **only** test 11 reds, because the
  corpus scores 1000‰. That single red is the whole reason test 11 exists; report it as such.

⚠️ **COMMIT the implementation before mutating.** `git checkout <file>` restores to `HEAD`; story
5.4b lost work to that twice inside one story. Verify each restore with `md5sum` against the
committed baseline **and** `git status`.

⚠️ **Split an assertion that pins two properties into two tests** — story 5.5 measured a single test
reporting 1 red where the mutation broke 2, because `assert_eq!` aborts on the first mismatch.

### AC8 — Register, docs, gates

- **Annotate the register by requirement, appending only, citing entries by TITLE** (never by line
  number — a check its own commit falsifies is worse than no check). Dispose of **R1** (the float)
  and **R2** (the self-pair); do **not** close what belongs to others: the `L1Key` tuple (5.9), the
  `&str` rule-id allocation (a condition), the group-address gap (Epic 6), `RuleId` -> enum (Epic 6).
- **New entries this story owes:**
  - **F17/D17 dormancy** — *"the blocker excludes `dormant` from automatic candidate generation"*
    [architecture.md:1205-1206] is **not implemented**, because no lifecycle state exists (measured:
    zero occurrences of `dormant` in `crates/`). Owner: the lifecycle epic (FR40-42).
  - **the quadratic universe** — the day a caller supplies a retention window rather than a poll, a
    narrowing key is required; the recall assertion is what makes it safe. Owner: 5.9/5.7, whichever
    first hands the blocker something other than one poll.
  - **the floor's own arithmetic** — `>= 999‰` is zero-tolerance below 1000 required pairs and
    becomes a real tolerance above it, where NFR4's *"any fraction is theatre"* applies. Owner: the
    story that first grows the truth set past that size (Tier 2, Epic 11+).
- **Correct the five falsified doc sites of §7 in the same commit**, then re-read the module docs of
  every file touched. ⚠️ `grep '5\.6'` gives you **nothing** — this list is by meaning.
- **Re-count mechanically after the last edit and state each number once.** Baseline measured at
  contexting: **309 = 135 bin + 128 core + 46 xtask** (`cargo test --workspace --locked`).
- `sprint-status.yaml`, `CLAUDE.md`, `docs/project-context.md` — docs-current-before-push.
  `epics.md` — **verify only**; an edit is a finding.
- **Full local gate before push**: `cargo fmt --all` · `cargo clippy --workspace --locked
  --all-targets -- -D warnings` · **`cargo clippy --workspace --locked -- -D warnings`** (the CI
  form, the only one that catches an import kept alive by a test module or an intra-doc link) ·
  `cargo test --workspace --locked` · `cargo xtask ci` printing **six** gates — `float-free` over
  **4** files — plus `ℹ views-hash STALE` (exit 0 by design). `git status` under `fixtures/` empty.
- **Branch -> PR -> green CI. The story ends at status `review` with the PR open.** `done` is the
  merge's business; the `code-review` workflow's step-6 default would set it early and has been
  deliberately not followed on any Epic 5 story.

## Tasks / Subtasks

- [x] **Task 1 — Enumerate the obligations before writing code** (AC8)
  - [x] `grep -n '5\.6' _bmad-output/implementation-artifacts/deferred-work.md` — **4 lines, two
        entries** (R1 the float, R2 the self-pair). Do **not** use `grep 'story 5\.6'`.
  - [x] Read the two entries in full, plus the two that mention the blocker without owning it.
  - [x] ⚠️ **Check that R1 and R2 have citable TITLES before AC8 asks you to cite them by title** —
        this was NOT verified at contexting, and AC8's no-line-numbers rule needs a title to exist.
        If one of them has none, say so and cite the smallest stable anchor instead.
  - [x] ⚠️ `grep -rn '5\.6' crates/ xtask/ --include=*.rs` returns **one unrelated hit**
        (`arp_ping.rs:272`). The doc worklist is §7's five sites, found by reading.

- [x] **Task 2 — Read before writing** (the project's primary named cause of review cycles)
  - [x] `crates/opencmdb-core/src/identity/l1.rs` in full — the join, `decide_pair`, and the module
        doc you are about to falsify.
  - [x] `crates/opencmdb-core/src/identity/cascade.rs:280-300` — the `Decision` doc's *"no blocker"*
        claim; and `identity/mod.rs`, `lib.rs`.
  - [x] `crates/opencmdb-bin/src/fixtures.rs` — `read_jsonl` (`:647`), `read_traps` (`:665`),
        `walk_trap_files` (`:834`), and the test module's helper idiom (`:921`).
  - [x] `crates/opencmdb-core/src/trap.rs:69-150` — `Expectation`, `Trap`.
  - [x] `xtask/src/main.rs`'s `gate_float_free`, `line_has_float`, `float_literal_kind`,
        `strip_line_comment` — you are writing under the directory it guards. §1 is the measured
        behaviour; re-run the probe yourself if you doubt a line.

- [x] **Task 3 — The pair type** (AC2)
  - [x] New file `crates/opencmdb-core/src/identity/blocking.rs`; `pub mod blocking;` in
        `identity/mod.rs`.
  - [x] `CandidatePair` with **private** fields, ordered by the constructor;
        `new(ObsId, ObsId) -> Option<Self>` returning `None` on the self-pair; accessors; derive
        `Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash`.
  - [x] Tests 3 and 4 of AC7 — **4 includes the accessors on their own**.
  - [x] Declare the test helpers you need here (`obs_id`, `l2`, `ts`, `mac`, `observation`), copied
        from `l1.rs`'s spellings. They are private there; this duplication is sanctioned (Dev Notes).

- [x] **Task 4 — The generator** (AC1)
  - [x] `pub fn candidates(observations: &[Observation]) -> BTreeSet<CandidatePair>`.
  - [x] It calls neither `join` nor `decide_pair` nor `verdict_for_pair`.
  - [x] Tests 1, 2, 5, 6, 7, 8, 9 of AC7 — **write test 7 (two `l2_domain`s) FIRST**; it is the only
        thing standing between a domain-blocked universe and green (§5). Test 9's module imports
        `join` deliberately (AC1); test 2 asserts from the **distinct-id** count, not `len()`.

- [x] **Task 5 — The floor and the recall function** (AC3)
  - [x] `BLOCKING_RECALL_FLOOR_PER_MILLE: u32 = 999` and `blocking_recall_per_mille(...) -> Option<u32>`.
  - [x] Tests 10 and 11 — **11 with an independent literal**, not the constant.
  - [x] ⚠️ No float in any assertion message; no story number without a letter suffix inside a string
        literal on a code line (§1, measured).

- [x] **Task 6 — The corpus assertion** (AC4)
  - [x] In `crates/opencmdb-bin/src/fixtures.rs`'s **test module only**: `blocking_recall_above_999`
        over the 10 `must-merge` pairs, plus the universe-coverage assertion over all 23 pairs, plus
        the *exactly one trap names fewer than two observations* residue count.
  - [x] 🔴 **The per-trap containment assertion and the recall value go in SEPARATE tests** (AC4).
        In one test they cannot both be observed, and M1's 700‰ becomes unmeasurable.
  - [x] Pairs are formed **per stream** via `Trap::replay` + `read_jsonl`; `proposed` is the **union**
        of those per-stream universes, which is what lets one recall call cover 10 streams (AC4).
        Nothing above `#[cfg(test)]` changes.
  - [x] ⚠️ `trap_gate.rs` is not opened.

- [x] **Task 7 — Prove to red** (AC7)
  - [x] **COMMIT first.** Then M1–M6, every red reported and classified.
  - [x] Report divergence from the predictions explicitly — story 5.5's review caught a table that
        tabulated six counts and commented only on the one that matched.

- [x] **Task 8 — Docs, register, gate, PR** (AC8)
  - [x] The five doc sites of §7; then re-read the module docs of every file touched.
  - [x] Register: dispose R1 and R2; append the three new entries; close nothing that is not yours.
  - [x] `lib.rs` — decide the re-export and **state the reason**: the recommendation is to follow
        `join`'s precedent (reach it through `identity::blocking::…`, since `candidates` and
        `recall` are very generic root-level names), and to correct `lib.rs:46-47`, whose stated
        reason this story falsifies.
  - [x] `sprint-status.yaml`, `CLAUDE.md`, `docs/project-context.md`; `epics.md` verify-only.
  - [x] Full local gate, both clippy forms; six gates, `float-free` over **4** files.
  - [x] Branch `story-5.6-blocker-and-recall-assertion` (from `master`, **after PR #58 merges**) ->
        PR -> green CI. **Ends at `review`, PR open.**

### Review Findings

**All nine patches applied 2026-08-01.** `332 → 333 tests` (139 bin + **148** core + 46 xtask) — the
one new test is `an_observation_with_no_facts_at_all_is_still_a_candidate`. Full local gate re-run
after the patches: `cargo fmt --all`, **both** clippy forms with `-D warnings`,
`cargo test --workspace --locked`, `cargo xtask ci` — six gates green, `float-free` still over 4
files, `file-size` unmoved at 24 files / largest 1136, `fixtures` 25/25.

**Three prove-to-red measurements on what the patches changed** — each restore verified by `md5sum`
against a pre-mutation copy, and `git status` shows no stray file:

| # | Mutation | Predicted | Observed | Reds | Carried by |
|---|---|---|---|---|---|
| MV1 | `if facts.is_empty() { continue; }` inside `candidates` — the narrowing that was green before the patch | the new test reds, alone | confirmed | **1** core | assertion |
| MV2 | the superset fixture degraded so the join yields one 2-member group instead of `{1,2,3}` | `assert_eq!(checked, 3)` reds where `checked > 0` would not | confirmed, and `left: 1` — **1 is greater than 0**, so the old guard was measurably green here | **1** core | assertion |
| MV3 | one required pair removed from the proposed universe, recall 900‰ | the FLOOR is the assertion that speaks, not the bare equality | confirmed — `recall is 900 per-mille, below the floor of 999` | **1** bin | assertion |

**Zero compiler-carried reds.** MV2 is the one that mattered: it turns "this assertion looks weak"
into a measured fact, since `checked` was **1** under the degraded fixture and `> 0` admits it.

_`bmad-code-review`, 2026-08-01, three layers (Blind Hunter · Edge Case Hunter · Acceptance
Auditor). Every finding below was RE-MEASURED by the reviewer before being written down; the two
that the layers reported but measurement refuted or found already-disclosed are not here. The
negative requirements all hold: `fixtures/` and `_bmad-output/planning-artifacts/` are byte-unchanged,
so `architecture.md`, `architecture-views.md` and `epics.md` are untouched; 332 tests; six green
gates with `float-free` over 4 files; no production caller; no `Default`._

- [x] [Review][Patch] **HIGH — the module doc's headline rationale is false for two of its three
      keys, and the story's own M1 refutes one of them** [crates/opencmdb-core/src/identity/blocking.rs:18-19].
      The doc reads *"Blocking on the MAC, the hostname or the `l2_domain` would each pass the whole
      committed corpus"*. Measured over the 10 committed `must-merge` pairs by parsing the TOML +
      JSONL directly: **MAC 7/10 = 700‰**, **hostname 4/10 = 400‰**, **`l2_domain` 10/10 = 1000‰**.
      Only the `l2_domain` narrowing is corpus-invisible. The MAC figure is the story's OWN M1 result
      (`| M1 | … | **700‰ exactly** |`, story:787) — the sentence is refuted by a measurement inside
      the same commit. The module's test-module doc already states the correct, narrower claim
      (*"a blocker that proposed only same-`l2_domain` pairs would score a full 1000"*), so the fix
      is to bring the head doc down to it and name the two measured figures.
- [x] [Review][Patch] **MED — two new doc comments claim `decide_pair(a, a)` answers with a merge;
      it abstains whenever the observation carries no MAC**
      [crates/opencmdb-core/src/identity/blocking.rs:105-107, crates/opencmdb-core/src/identity/l1.rs:297-298].
      `verdict_for_pair` returns `Neutral` when either side has no MAC key (`l1.rs:252-258`), and
      `decide` maps `(None, None, false, false)` onto `Abstained { AbsenceOfProof }`
      (`cascade.rs:511-513`) — not a merge. The diff's own
      `an_observation_with_no_mac_is_still_a_candidate` proves MAC-less observations are in scope.
      `deferred-work.md`'s R2 entry repeats it (*"it still answers `Decisive`"*). Weaken all three to
      the true sentence.
- [x] [Review][Patch] **MED — three doc sites name a consumer in the present tense that this same
      commit says does not exist** [crates/opencmdb-core/src/identity/cascade.rs:294-295,
      crates/opencmdb-core/src/identity/mod.rs:20-21, crates/opencmdb-core/src/lib.rs:49-50].
      *"The blocker's consumer **is** the trap runner"*, while `deferred-work.md`'s new entry states
      *"the blocker has **no production caller at all**"*. `grep -rn "blocking::\|candidates(\|CandidatePair"
      crates/ xtask/ --include=*.rs` returns zero hits outside the two test modules. This is the
      shape of the story's own inherited lesson 10 (*"a rationale that names a future story is a
      claim with an expiry date"*): say "intended consumer (story 5.7)", not "is".
- [x] [Review][Patch] **MED — the register says three new entries were opened; four were**
      [_bmad-output/implementation-artifacts/deferred-work.md:1730]. Under `### New, raised by this
      story` there are four bullets: D17's `dormant` exclusion, the quadratic universe, the floor's
      arithmetic past 1000 pairs, and "nothing calls the blocker and the engine in sequence". The
      fourth is legitimate and carries an owner; only the count is stale. The same "three" is
      repeated at story:857, story:891 (Change Log, which enumerates only the first three) and
      `sprint-status.yaml`. Falls under the story's own inherited lesson 2 (*"a count in a doc is a
      claim; count mechanically, after the last edit"*).
- [x] [Review][Patch] **MED — nothing feeds `candidates` an observation with empty `facts`, so the
      "reads no `Fact` at all" claim has no red** [crates/opencmdb-core/src/identity/blocking.rs:140].
      Measured: inserting `if left.facts.is_empty() { continue; }` into the loop leaves the whole
      workspace green (332 passed, 0 compile errors) — the same mutation class as M2, but M2 is
      caught by `two_l2_domains_are_still_a_candidate_pair` and this one is caught by nothing. The
      corpus cannot help: 0 of the 51 committed observations has empty facts, and the only empty
      `facts` vectors in the tree are in `l1.rs`'s tests. Give one observation in a `candidates` test
      `facts: vec![]`.
- [x] [Review][Patch] **MED — the floor comparison can never be the assertion that fails**
      [crates/opencmdb-bin/src/fixtures.rs:4582-4589]. `assert_eq!(recall, 1000)` is strictly
      stronger and runs before `assert!(recall >= BLOCKING_RECALL_FLOOR_PER_MILLE)`. Measured:
      changing `>=` to `>` leaves 332 green, so the operator choice — whether exactly 999‰ is meant
      to pass — is untested at its boundary. The module doc quotes D18's *"a gate that cannot fall is
      decoration"* to justify this very assertion. Put the floor assertion first, so a narrowed
      blocker reds with D13's message rather than with a bare equality.
- [x] [Review][Patch] **LOW — `blocking_recall_per_mille` has a second `None` with a different
      meaning, and its only caller misreads it**
      [crates/opencmdb-core/src/identity/blocking.rs:218]. `u32::try_from(hits * PER_MILLE /
      required.len()).ok()` synthesises a `None` on conversion failure, indistinguishable from the
      documented empty-denominator `None`; `fixtures.rs:4580` reads it as `.expect("the truth set is
      not empty, so the recall is defined")`. Unreachable today (`hits <= required.len()` bounds the
      quotient at 1000), so the doc is not yet false — but the `# Returns` section states a contract
      the code does not enforce, and the same site multiplies in `usize` with no `checked_mul`. Make
      the conversion infallible-by-statement and say why.
- [x] [Review][Patch] **LOW — the tolerance boundary in the module doc is off by one**
      [crates/opencmdb-core/src/identity/blocking.rs:49]. *"It becomes a real tolerance only if the
      required set ever exceeds 1000 pairs"* — at exactly 1000 required pairs one miss scores
      999/1000 = 999‰ and `999 >= 999` passes, so the tolerance opens **at** 1000, not above it. The
      same off-by-one is in `deferred-work.md`'s new entry. Doc-only today
      (`assert_eq!(corpus.required.len(), 10)` reds long before), but it is the sentence that gets
      inherited "on that day".
- [x] [Review][Patch] **LOW — the superset test's own guard is an order of magnitude weaker than its
      fixture** [crates/opencmdb-core/src/identity/blocking.rs:527]. `assert!(checked > 0)` where the
      fixture deterministically yields exactly 3 grouped pairs (group `(l2(10), mac(0x01))` =
      {1,2,3}); it would still pass if the join degraded to one 2-member group. `assert_eq!(checked,
      3)` is the idiom this very story uses at `fixtures.rs:4613`.
- [x] [Review][Defer] **LOW — `checked == 10` counts required-pair occurrences, not `must-merge`
      traps** [crates/opencmdb-bin/src/fixtures.rs:4601-4614] — deferred, latent. The filter is
      `corpus.required.contains(pair)`, so a second trap of any expectation naming a pair already in
      `required` increments `checked` again and the test reds with a message that does not describe
      the cause. Not live: 23 traps, 23 distinct pairs today.
- [x] [Review][Defer] **LOW — the residue assertion compares an order-dependent `Vec`**
      [crates/opencmdb-bin/src/fixtures.rs:4653] — deferred, latent. `without_a_pair` is pushed in
      `walk_trap_files` order and compared to a one-element `vec![]`; green today because there is
      exactly one residue trap, order-dependent the day a second one-observation trap is committed.
      A `BTreeSet` (as `required` and `universes` already use) removes the dependency.

## Dev Notes

### The float gate, in one paragraph

It walks `crates/opencmdb-core/src/identity/` **recursively**, strips `//` and `///` comments, and
reds on a word-bounded `f32`/`f64`/`f16`/`f128` or on a numeric literal that tokenises as a float
(one dot with an empty suffix, an exponent, or an `f32`/`f64` suffix). It **fails closed** if the
directory is missing or holds no `.rs` file. There is no `#[allow]`, no allowlist, no `#[cfg(test)]`
skip. **Green**: `999`, `1000`, `"192.168.0.1"` (three dots), `t.0.1`, `0xFF`, `1..32`,
`fn blocking_recall_above_999()`, a float quoted in a `///` comment. **Red**: `0.999`, `1.`, `1e-3`,
`0.85f64`, a one-dot decimal inside a **string literal on a code line** (`"story 5.6"`,
`"blocking_recall >= 0.999"`), a float in a `/* … */` block comment, a decimal in `#[doc = "…"]`.
All of §1's rows were measured on this tree at contexting.

### Why `identity/blocking.rs`, and why the recall test is not there

The architecture names the file — `blocking.rs # candidate generator + blocking_recall >= 0.999`
[architecture.md:3368] — so unlike story 5.5, this story does **not** choose its location. What it
must choose is where the *corpus-driven* assertion lives, and D47 settles it: core cannot read
files. §4 is the split. Do not try to route the corpus into core through a feature flag, a
`test-support` reader or an `include_str!` — an `include_str!` of a committed artefact would create a
second copy of a sha256-locked spec, which the fixture gate exists to prevent.

### The 5.6 / 5.7 boundary

5.6 **proposes**; 5.7 **answers** and 5.8 **buckets**. The seam is `score_corpus`'s `answers:
&BTreeMap<TrapId, Outcome>` (`trap_gate.rs:223-226`), which **has no production caller at all**:
measured, all **10** call sites sit at `:410` and below, inside the `#[cfg(test)] mod tests` that
opens at `:385`, and every one of them passes an empty map.
Three things keep it out of reach here: the crate frontier, the deliberate absence of a
`Decision <-> Outcome` mapping, and `VerdictVectorEntry`'s deliberate uninhabitedness (pinned by two
`size_of::<Option<T>>() == 0` tests). **If you find yourself comparing a verdict to a trap, you have
squatted 5.7.**

### Deliberate redundancy you must not collapse

- `cascade.rs`'s `expected_conclusion` and `l1.rs`'s `expected_l1_conclusion` — D13's table restated
  independently of the code under test. `l1.rs`'s `CORPUS_EXACT_MAC`/`CORPUS_DISTINCT_MAC` — the rule
  ids as independent literals. **AC3's floor literal joins that family.**
- `Verdict::all()` / `IdentityAbstentionCause::all()` — the exhaustive-match witnesses.
- `keys_of`'s match over `Fact` is **exhaustive on purpose, no `_` arm**; if the blocker ever reads a
  `Fact` (it should not), the same rule applies.
- `fixtures.rs`'s `expected()`; `score.rs`'s `Column::as_str()` vs `Expectation::column()`.

### House rules that bind this story

- **`opencmdb-core` is the domain.** No `anyhow`, `axum`, `sqlx`, `askama` (D47, gated). No clock:
  `chrono` is built with `default-features = false`, so `Utc::now()` does not compile here.
- **Document every `pub` item** — struct, enum, **field**, **variant**, fn. ⚠️ `opencmdb-core` does
  not carry `#![deny(missing_docs)]`; nothing checks you but the review. **A doc comment must be
  TRUE**; prefer the weaker true sentence.
- **A comment asserting a checkable property gets checked.** Do not quote a number in a comment when
  you can assert the property in a test.
- **`deferred-work.md` is append-only.** Never rewrite a bullet.
- **Test helpers: an idiom to COPY, not items to import.** Measured: `l1.rs`'s helpers are
  `obs_id(n: u128)` (`:325`), `l2(n)` (`:329`), `ts()` (`:337`), `mac(last)` (`:343`) and
  `observation(…)` (`:358`) — **all private, inside its own `#[cfg(test)] mod tests`**, so none is
  reachable from `blocking.rs`. (`fn obs(n)` exists too, but in `trap.rs:417` and `cascade.rs:697` —
  a *third* spelling, and not the one in the file this story tells you to read.) Re-declare what you
  need in `blocking.rs`'s test module and **name them after `l1.rs`'s spellings**. This duplication
  is sanctioned here: the alternative is a `pub(crate)` test-helper surface this story does not want,
  and three copies already coexist. Do **not** report it as a DRY violation, and do not extract a
  shared helper.
- **The two forms that do compile**, both already used in `l1.rs`, because the obvious ones do not:
  `Uuid::from_u128(n)` (⚠️ `Uuid::new_v4()` does not compile — core builds `uuid` with
  `features = ["v7","serde"]`, no `v4`) and
  `DateTime::parse_from_rfc3339(…).unwrap().with_timezone(&Utc)` (⚠️ `Utc::now()` does not compile —
  `chrono` is built `default-features = false, features = ["serde","std"]`, so no `clock`). AC7's
  test 6 requires *varying* `observed_at`, so you need the second one.

### Inherited lessons — read before writing a doc comment or a number

Cumulative; **ten**, as of story 5.5's code review:

1. **A check that its own commit falsifies is worse than no check.** Cite register entries by TITLE.
2. **A count in a doc is a claim.** Count mechanically, after the last edit.
3. **A red set is a count too.** Report every red a mutation fires, not the first.
4. **Classify your reds honestly.** A red that fires on `assert_eq!(1, 1)` is the compiler's.
5. **An inventory in a doc comment has no guard behind it.**
6. **Name the test behind every claim**, or write the weaker true sentence.
7. **A mutation pass needs a committed baseline to restore TO.**
8. **Do not quote a number in code — assert the property instead.**
9. 🔴 **The completion record is the most defect-prone artefact you will write.** Story 5.5's review
   found 6 HIGH of which **three were about claims rather than code, and all three were the
   implementer's — the fourth consecutive story with that defect.** Re-read your own record as if it
   were someone else's, and reconcile every number you state against a command you can re-run.
10. **A green-case rationale that names a future story is a claim with an expiry date.**
    `xtask/src/main.rs:1821` predicted story 5.5 would write IP literals under `identity/`; it wrote
    none, and verifying is what found it false. ⚠️ **The message was corrected during 5.5**, so
    grepping `:1821` today shows the replacement rationale, not the prediction — the lesson survives,
    the evidence for it is in `deferred-work.md:1622-1628`. Do not write a new prediction about story
    5.7 into an assertion message — and if you must name a story in code, remember §1: **any** story
    number without a letter suffix reds the gate, 5.7 included.

### The self-referential test, one more time

Story 5.5's validation agent measured it: renaming the two rule-id constants to non-canonical
spellings left its **prototype** suite green 296/296, because every expectation was built from the
constant it was checking. That is why 5.5 shipped the two ids as independent literals — after which
the same mutation reds **10**. **This story's equivalent is the floor**: every assertion that reads
`BLOCKING_RECALL_FLOOR_PER_MILLE` moves with it, so a weakened floor is invisible unless one test
pins the literal. AC7's M6 is the proof, and its expected red count is **one** — confirmed exactly
during validation.

### What this touches, and what it must not break

| file | NEW / UPDATE | what |
|---|---|---|
| `crates/opencmdb-core/src/identity/blocking.rs` | **NEW** | `CandidatePair`, `candidates`, `blocking_recall_per_mille`, the floor, and its tests. **Subject to `float-free`.** |
| `crates/opencmdb-core/src/identity/mod.rs` | UPDATE | `pub mod blocking;` + the module doc at `:15-16` (*"There is still no candidate pair generator"*) |
| `crates/opencmdb-core/src/identity/l1.rs` | UPDATE (**docs only**) | `:16-18` and `:287` — the ownership clauses only; the join, the rules and `decide_pair` are **untouched** |
| `crates/opencmdb-core/src/identity/cascade.rs` | UPDATE (**docs only**) | `:292-293` — *"There is still no blocker"* |
| `crates/opencmdb-core/src/lib.rs` | UPDATE | the re-export decision + `:46-47`'s stated reason, which this story falsifies |
| `crates/opencmdb-bin/src/fixtures.rs` | UPDATE (**tests only**) | AC4's assertions, below `#[cfg(test)]` (`:729`). Nothing above it moves |
| `trap_gate.rs`, `score.rs`, `trap.rs`, `observation/mod.rs` | **LEAVE ALONE** | 5.7's seam / no change needed |
| `fixtures/**` | **LEAVE ALONE, READ ONLY** | locked spec; the gate checks both directions |
| `deferred-work.md`, `sprint-status.yaml`, `CLAUDE.md`, `docs/project-context.md` | UPDATE | annotations and docs-current |
| `epics.md`, `architecture.md`, `architecture-views.md` | **NEVER** | verify-only / issue #54 / issue #50 |

### What STOP means, procedurally

If a step appears to require editing `fixtures/`, `architecture.md` or `epics.md`; or calling
`decide_pair`; or filling `score_corpus`'s `answers`; or inhabiting `VerdictVectorEntry`; or
weakening the `float-free` gate; or "fixing" L1's `Disqualifying` on the `multi-nic` pair — **stop
and report it as a finding.** Do not absorb it. Every one of those is another story's, and three of
them are load-bearing claims in files this story does not own.

### Project Structure Notes

`crates/opencmdb-core/src/identity/` today: `mod.rs`, `cascade.rs`, `l1.rs`. The new file is the
**fourth**, and the `float-free` gate's file count moves from 3 to 4 — a number worth checking in the
gate's own output rather than asserting in prose.

D54: **the folder is not the frontier — visibility is.** `identity/mod.rs:23-26` currently says
*"nothing yet is"* restricted to this subtree; `verdict_for_pair` became `pub(crate)` in story 5.5,
so **verify that sentence rather than trust it**, and correct it if this story adds a
`pub(in crate::identity)` item.

### References

- **D13** the blocker [architecture.md:1004-1011] · decision `:931-932` · float refusal `:956-958` ·
  the six-row table `:967-974` · **the milli-units corollary `:988-993`** · level split `:984-986` ·
  structural facts `:995-1002`
- **D12** one engine instantiated twice, *"two rule sets and two blocking keys"* `:917` · a MAC
  identifies an INTERFACE `:884` · the L1/L2 table `:888-893` · the device is non-negotiable `:919-928`
- **D14** `AMBIGUOUS` is a LINK, not an absence `:1031-1034`
- **D17** `dormant` excluded from automatic candidate generation `:1205-1206` (**not implementable
  today**) · no `presence` level `:1171-1173`
- **D18** the gate is Tier 1, binary, at the device level `:1224-1226` · the three columns
  `:1230-1234` · honesty vs cowardice `:1241-1244` · **Tier 2 blocks nothing, and why a loose
  threshold is decoration `:1246-1253`**
- **D19** the fixture asserts the RULE `:1307-1310` · ATDD order `:1341-1346`
- **D47** frontier `:2584` · **D56b** identity tests inline, no database `:3302-3306`
- **Ratified test naming**, incl. `blocking_recall_above_999` `:2954`
- **The target source tree**, incl. `identity/blocking.rs` `:3368`
- **NFR4** — read from `prd.md:1179-1207` (it runs to `:1207`; `NFR5` starts at `:1208`), **not**
  architecture.md's stale F-tables. *"any fraction is theatre"* is at `:1182`, the **device** level at
  `:1183`, and the closing oracle sentence at `:1205-1207`
- Corpus: `fixtures/scenario/traps/*.toml` (24 traps), `fixtures/scenario/replay/*.jsonl`
- GitHub issues: **#54** (D13's table is short one row), **#50** (`architecture-views.md` stale)

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context), `claude-opus-5[1m]`, as Amelia (`bmad-dev-story`).

### Debug Log References

**Baseline, re-measured before writing anything** (`cargo test --workspace --locked`):
**309 = 135 bin + 128 core + 46 xtask**, matching the contexting figure exactly.

**Red phase, before the implementation existed.** The two functions and the constructor were written
with degenerate bodies (`candidates` → empty set, `new` → always `Some` in argument order,
`blocking_recall_per_mille` → always `Some(1000)`) and the suite run: **11 of the 19 core tests
FAILED, every one of them assertion-carried — zero compile errors.** The 8 that passed under the
degenerate implementation are named here rather than glossed, because a test that passes against a
stub is a test whose red comes from somewhere else: `no_observation_yields_no_candidate`,
`one_observation_yields_no_candidate`, `a_repeated_obs_id_yields_no_pair`,
`the_universe_is_input_order_independent`,
`the_generator_reads_neither_raw_nor_observed_at_nor_the_connector`,
`every_required_pair_proposed_is_full_recall`,
`a_pair_proposed_but_not_required_does_not_inflate_recall`,
`the_floor_is_nine_hundred_and_ninety_nine_per_mille`. Each of those is redded by a mutation below
instead (M3, M4, M5, M6), which is where their guarantee actually comes from.

**Mutation pass (AC7), run against the committed baseline `eea3ef1`.** Every restore was verified
with `md5sum` against the commit **and** `git status` — `blocking.rs` back to
`8bbceb7b35d05996b7b9536d8d7529d2`, `fixtures.rs` unchanged at
`100d2a3323d9e959f577d51143c95c17`, working tree clean — after each of the six.

| # | mutation | predicted | OBSERVED | reds | carried by |
|---|---|---|---|---|---|
| M1 | universe narrowed to pairs sharing an L1 key | corpus recall **700‰**; recall + coverage + several core tests red | **700‰ exactly** | **7** — 3 bin, 4 core | assertions |
| M2 | universe narrowed to same-`l2_domain` pairs | whole corpus GREEN; only the synthetic cross-domain test reds | **corpus 139/139 green** | **1** core | assertion |
| M3 | the self-pair admitted | tests 3 and 4 red; the `n*(n-1)/2` count test does **NOT** | confirmed on both halves | **2** core | assertions |
| M4 | pair ordered by argument order | core reds; **corpus entirely green** | corpus green | **3** core | assertions |
| M5 | `Some(1000)` for an empty `required` | the `None` case reds | confirmed | **1** core | assertion |
| M6 | floor weakened to `900` | **only** the independent-literal test reds | confirmed | **1** core | assertion |

**Zero compiler-carried reds across all six.**

**Divergences from the predictions, stated because the fourth-consecutive-story defect this project
records is tabulating counts and commenting only on the ones that matched:**

- **M4's bin figure.** The story predicted `138 passed; 0 failed` on `opencmdb-bin`. Observed
  **`139 passed; 0 failed`** — the *verdict* (entirely green, the corpus cannot see the mutation) is
  exactly right; the *count* is one higher because this story ships **4** bin tests where the
  validation prototype shipped 3. AC4's split rule is what added the fourth.
- **M1's bin red set is 3, and the story could only name 2.** The third is
  `the_blocker_proposes_every_required_pair_within_its_own_stream`, which did not exist as a separate
  test at contexting — it is the test AC4's split rule created. Its message names the first trap that
  misses (`docker-veth-must-merge`), and because it is a *different* test, `blocking_recall_above_999`
  still runs and still reports **700**. That is the MEDIUM finding from validation, confirmed by
  measurement rather than inherited: in one test the 700 would have been unobservable.
- **M1's core red set is 4**, not the "several" the story estimated:
  `four_observations_yield_six_pairs`, `the_count_is_quadratic_in_the_number_of_distinct_ids`,
  `two_l2_domains_are_still_a_candidate_pair`, `an_observation_with_no_mac_is_still_a_candidate`.
- **M2, M3, M5 and M6 matched their predictions with no divergence at all**, including M3's negative
  half (the count test stays green because `candidates` only offers index pairs `i < j`, so the
  `Ordering::Equal` branch is unreachable on distinct input).

**Corpus counts, measured by the shipped assertions rather than quoted from the story:** 24 traps,
23 naming a pair, 1 naming fewer (`example-must-abstain`), **0 naming more than two**, 10 `must-merge`
pairs, recall **1000‰**. All five are `assert_eq!`s, not comments.

**Final gate**, all green in one pass: `cargo fmt --all` · `cargo clippy --workspace --locked
--all-targets -- -D warnings` (clean) · `cargo clippy --workspace --locked -- -D warnings` (the CI
form, clean) · `cargo test --workspace --locked` · `cargo xtask ci` — **six green gates**, with
`float-free` reporting **4 file(s)** where it reported 3, `file-size` unmoved at 24 files / largest
1136, plus `ℹ views-hash STALE` (exit 0 by design, issue #50). `git status --porcelain fixtures/`
returns nothing, and so does `git status --porcelain _bmad-output/planning-artifacts/`.

### Completion Notes List

- **AC1 — met.** `candidates(&[Observation]) -> BTreeSet<CandidatePair>` returns every unordered pair
  of distinct `obs_id`s. It calls neither `join`, `verdict_for_pair` nor `decide_pair`; it reads no
  `Fact` at all, only `obs_id`. The count is asserted from the **distinct-id** count for n = 0..=6,
  not from `len()`. The superset property is pinned by
  `every_pair_inside_a_join_group_is_a_candidate`, whose test module imports `join` on purpose — that
  import is the test, and it is clean under both clippy forms.
- **AC2 — met.** Private fields ordered by the constructor; `new(a, a)` is `None`; the accessors are
  exercised on their own. The doc states that the ordering carries **no** meaning.
- **AC3 — met.** `BLOCKING_RECALL_FLOOR_PER_MILLE: u32 = 999`, `blocking_recall_per_mille` returns
  `Option<u32>` and `None` on an empty denominator, truncation pinned at 2-of-3 → 666, and the floor
  pinned by an **independent literal**. The gate is untouched and now walks 4 files.
- **AC4 — met.** `blocking_recall_above_999` carries the architecture's ratified name and computes
  **only** the recall; containment lives in its own test; universe coverage over all 23 pairs is a
  third, deliberately not called recall; the residue count is a fourth. Pairs are formed per stream,
  `proposed` is the union, and the per-trap containment assertion is what makes the union honest.
  Nothing above `fixtures.rs`'s `#[cfg(test)]` changed and no new `pub` item appeared in bin.
- **AC5 — met, and nothing was absorbed.** No verdict, no `Decision`, no call to
  `decide`/`decide_pair`/`verdict_for_pair`; `trap_gate.rs`, `score.rs`, `trap.rs` and
  `observation/` were not opened for editing; no `l2-*` rule, no `InterfaceId`/`EntityId`, no
  persistence, no `Default`; no structural MAC reading consumed; nothing under `fixtures/` written.
  **No step required a STOP** — no finding of the kind §*What STOP means* describes arose.
- **AC6 — met.** The module doc carries both D13 quotations with their citations, the 90k-at-300-hosts
  figure with D13's own conclusion that the blocker is not a performance device, the three-way
  distinction from D18's refused pairwise-recall gate including the honest sentence that at this
  denominator the floor is zero-tolerance, the explicit *"nothing here advances NFR4"*, and what the
  component does not do. It is not an inventory of the epic.
- **AC7 — met.** 19 core tests + 4 bin, all six mutations run, every red reported and classified, and
  three divergences from the predictions stated above rather than left implicit.
- **AC8 — met.** R1 and R2 disposed of by title; **four** new entries opened with owners (the
  count read "three" until this story's code review counted the bullets); four entries
  belonging to others explicitly read and left open. The five falsified doc sites corrected, plus a
  sixth found by re-reading rather than by grep (below). Counts re-measured mechanically after the
  last edit. `epics.md` verified unmodified.
- **A sixth falsified site, found by re-reading and not on the story's list.** `identity/mod.rs`'s
  D54 paragraph says nothing here is restricted to this subtree. That is still **true**
  (`verdict_for_pair` is `pub(crate)`, which is the crate and not the subtree, and `blocking` adds no
  `pub(in ...)` item) — so it was **verified and annotated rather than rewritten**, which is what the
  story asked for. The related sentence in `lib.rs`'s crate doc *was* stale and was corrected.
- **What is claimed here and what is not.** The blocker has **no production caller**: it is reached
  from its own tests and from `fixtures.rs`'s test module only. The purity claim is tested on its
  falsifiable half only — varying `raw`, `observed_at` and `connector_id` — because the clock/SQL
  half is unreachable from a `&[Observation]` and no test could red on it; the test says so in a
  comment. Nothing here advances NFR4.

### File List

| file | change |
|---|---|
| `crates/opencmdb-core/src/identity/blocking.rs` | **NEW** — module doc, `CandidatePair`, `candidates`, `BLOCKING_RECALL_FLOOR_PER_MILLE`, `blocking_recall_per_mille`, 19 tests |
| `crates/opencmdb-core/src/identity/mod.rs` | `pub mod blocking;` + the module doc's *"no candidate pair generator"* claim + the D54 paragraph annotated |
| `crates/opencmdb-core/src/identity/l1.rs` | **docs only** — the *"next story's"* ownership clauses at the module doc and at `decide_pair`, plus the self-pair precondition's holder |
| `crates/opencmdb-core/src/identity/cascade.rs` | **docs only** — `Decision`'s *"there is still no blocker"* claim |
| `crates/opencmdb-core/src/lib.rs` | the crate doc, the retired re-export reason, and the `blocking` re-export decision |
| `crates/opencmdb-bin/src/fixtures.rs` | **tests only** — one import line in the test module, `CorpusPairs`, `corpus_pairs`, 4 tests. Nothing above `#[cfg(test)]` |
| `_bmad-output/implementation-artifacts/deferred-work.md` | appended §*Deferred from: story-5.6* |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | `5-6` → `review`, with the measurements |
| `_bmad-output/implementation-artifacts/5-6-blocker-and-recall-assertion.md` | this record |
| `CLAUDE.md`, `docs/project-context.md` | docs-current-before-push |

## Change Log

| Date | Change |
|---|---|
| 2026-08-01 | **Implemented (`dev-story`) → status `review`.** Branched `story-5.6-blocker-and-recall-assertion` from `master` at `440b30e`, after PR #58 merged — the condition the contexting note set. **309 → 332 tests (139 bin + 147 core + 46 xtask)**, re-counted mechanically after the last edit. Six green gates, `float-free` over **4** files where it walked 3, `file-size` unmoved. **A genuine red phase preceded the implementation**: with degenerate bodies, 11 of the 19 core tests failed and **every red was assertion-carried, zero compile errors**; the 8 that passed against the stub are named in the Debug Log rather than glossed, and each is redded by a mutation instead. **All six mutations run against the committed baseline `eea3ef1`**, each restore verified by `md5sum` **and** `git status`: M1 **700‰ exactly as measured at contexting** (7 reds), M2 **the whole corpus stays green** and only the synthetic cross-domain test reds (1) — the mutation that proves that test is load-bearing, M3 2 reds with the count test correctly NOT redding, M4 3 core reds and **the corpus entirely green**, M5 1, M6 1. **Zero compiler-carried reds.** Three divergences from the predictions are stated explicitly rather than left implicit: M4's bin count is **139**, not the predicted 138 (same verdict, one more test, because AC4's split rule added a fourth bin test); M1's bin red set is **3**, not 2, the extra one being the containment test that did not exist at contexting; and M1's core red set is **4**, where the story estimated "several". The corpus counts are **asserted, not quoted** — 24 traps, 23 pairs, 1 residue, 0 beyond a pair, 10 required, recall 1000‰. AC8's five doc sites corrected, plus **a sixth found by re-reading and not by grep**: `identity/mod.rs`'s D54 paragraph, which turned out to be **true** (`verdict_for_pair` is `pub(crate)` — the crate, not the subtree) and was therefore verified and annotated rather than rewritten. Register: **R1 (the float) and R2 (the self-pair) disposed of by TITLE**, **four** new entries opened (D17's unimplementable `dormant` exclusion, the quadratic universe, the floor's own arithmetic past 1000 pairs, and the blocker's absent production caller — this record said "three" and enumerated three until the code review counted four bullets), and four entries belonging to others read and left open — including the `&str` allocation one, whose condition is **measured NOT met**, since `candidates` calls no rule. `fixtures/` and the planning artifacts are byte-unchanged. No STOP condition arose. |
| 2026-08-01 | **Validation pass, two fresh-context agents (fact-check + gap-hunt), MANDATORY per Guy's Epic 4 retrospective decision.** Coverage: **128 factual claims measured (120 true, 6 false, 2 unverifiable)** and the story **implemented end to end** in an isolated worktree — it compiled first try, reached **331 tests (309 + 22)**, six green gates with `float-free` over **4** files and `file-size` unmoved, and all six mutations were run. **16 findings applied: 3 HIGH, 3 MEDIUM, 4 LOW, plus 6 citation corrections.** As on story 5.5, **every HIGH came from the agent that COMPILED the story, none from the agent that checked its claims** — the citations, greps, corpus counts, register enumeration, the 309 baseline and all eight rows of the float probe reproduced exactly. **The three HIGH:** **(H1)** **M4's second half is FALSE by measurement** — *"the corpus test reds wherever a trap lists its two ids in an order the stream does not"*: **0 of 23** trap pairs do, and 0 are out of ascending-UUID order either, so `opencmdb-bin` stays **138/138 green** under M4; the prediction is now corpus-invisible by statement, and the measurement is a new row in §3. **(H2)** **AC4 was self-contradictory** — *"per stream, never across streams"* cannot coexist with *"recall over the whole truth set"* except through a **union** of the per-stream universes, which the story never named; the union is only honest because the per-trap containment assertion proves each required pair is in its OWN stream (backstopped by a new measurement: **39** distinct `obs_id`s across the **11** trap-named streams, zero collision). **(H3)** **AC1 forbade calling `join` while AC7 test 9 requires it** — the prohibition binds `candidates`, not the module; the test module imports `join` deliberately, which is the only way a superset property is checkable. **The three MEDIUM:** M3's predicted red set named a test that **cannot** red (`candidates` offers only `i < j`, so the `Equal` branch is unreachable on distinct input); **M1's 700‰ is unobservable unless AC4's two assertions are two tests** — measured, a single test panics on `docker-veth-must-merge` before any recall is computed, so AC7's own split rule is now imposed on AC4; and **`fn obs(n: u128)` is not importable** — `l1.rs` spells its five helpers `obs_id`/`l2`/`ts`/`mac`/`observation`, all private, so the copy is sanctioned explicitly rather than left to collide with DRY. Also corrected: `n*(n-1)/2` now says **n = distinct ids**; *"eleven core tests"* is restated as eleven **requirements**; the accessors got a test; `Uuid::from_u128` and `parse_from_rfc3339` are named as the forms that compile; **test 8's justification was refuted** (all six `hostname-absence` observations carry a MAC — the family encodes an absent *hostname*); the **296/296** figure was 5.5's *validation prototype*, whose shipped tests red **10**; *"one per family"* omitted `example-must-merge`, which has no family; `xtask:1821`'s prediction **was corrected during 5.5**; `score_corpus` has **no production caller** (all 10 sites are below `:385`); NFR4 runs to `prd.md:1207`; D18's quote to `:1253`. Two rows added to §1's float table, both measured: the citations AC6 demands are **green even on a code line**, and **`"story 5.7"` reds too** — the rule is any story number without a letter suffix. Two things left explicitly unmeasured rather than assumed: `"v0.1"` (one dot), and whether R1/R2 carry citable TITLES, now a task. The prototype implementation was **discarded** (Guy's call) — dev-story rewrites from the corrected story. |
| 2026-08-01 | Story contexted against `master` at `0ebd50f` (5.5 merged; bookkeeping PR #58 still OPEN). **Four findings changed the story rather than decorating it, each measured at contexting.** (1) **The float was resolved by measurement, not by reading the gate**: a probe file under `identity/` run through `cargo xtask ci` reported exactly three offenders — `0.999`, the epic's own assertion text quoted in an assertion message, and `"story 5.6"` in a string on a code line — while `999`, `1000`, `blocking_recall_above_999` and a `///`-quoted `0.999` were green. The floor becomes an INTEGER in per-mille, on D13's own milli-units corollary and on the architecture's ratified test name, which already contains no float. (2) **D18 refuses a pairwise-recall gate by name** (*"a gate that cannot fall is decoration"*), so the story owes the distinction — different subject, different venue, and the honest arithmetic that `>= 999‰` at a 10-pair denominator IS zero tolerance. (3) **The truth set is the committed corpus**: 24 traps, 23 name a pair, 10 `must-merge`, of which **7 share a MAC and 3 do not** — so an exact-MAC blocker scores **700‰** and the recall assertion has a real red. Because core cannot read files (D47), the corpus assertion lives in `fixtures.rs`'s test module and the blocker in `identity/blocking.rs`, the file the architecture already names. (4) **The wrong blocker that passes the whole corpus is the same-`l2_domain` one** — all 23 trap pairs are same-scope, measured — so only a synthetic cross-domain test stands between it and green; it is required to be written first. Also measured: `grep '5\.6'` over the code returns **zero** relevant hits, so the doc worklist is by meaning (five sites, listed); there is no `dormant` anywhere, so D17's blocker clause cannot be implemented and is registered instead; and the baseline is **309 tests = 135 + 128 + 46**. |
