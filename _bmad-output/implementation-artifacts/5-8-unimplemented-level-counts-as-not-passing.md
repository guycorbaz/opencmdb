# Story 5.8: A trap whose level the engine does not implement counts as NOT PASSING

Status: review

<!-- Validation is MANDATORY here (Guy's decision, Epic 4 retrospective 2026-07-26): two
     fresh-context agents (fact-check + gap-hunt) BEFORE dev-story. The template banner saying
     "Validation is optional" does not apply to this project.

     🔑 Measured on 5.5, 5.6 AND 5.7: EVERY HIGH finding came from the agent that COMPILED the
     story, none from the agent that checked its claims. On 5.7 all three HIGH were **no-ops** — a
     prescribed mutation that reddened nothing, an assertion placed where it was unreachable, a walk
     that could not see the mutation aimed at it. So the gap-hunt agent must be told explicitly:
     **implement the story, then ask of every prescribed mutation "does this actually red, and
     WHICH assertion carries the red?"**

     Point the gap-hunt agent at **AC2 (does the Answer type change really leave every existing
     `passed()` assertion green except the one this story flips?), AC5 (is the per-column arithmetic
     assertion reachable, or does an earlier assertion panic first?) and AC9's M3** — the mutation
     that maps an unanswerable trap onto `Outcome::Abstained`, which this story claims makes
     `example-must-abstain` PASS. If M3 does not red, AC1's central refusal is decoration.

     ✅ DONE 2026-08-02. **20 findings applied**, 0 dismissed. The pattern held a FOURTH time: the
     gap-hunt agent implemented AC1-AC10 end to end (352 -> 364 tests, six gates green) and produced
     every finding that a compiler was needed for. §11 records the end state it measured.

     The prediction above was RIGHT on AC5 (M5 does not reach the arithmetic loop — M5b added) and
     on M3 (which reds 7 tests, so AC1's refusal is NOT decoration), and WRONG on AC2: the seam
     change behaved exactly as §7 predicted, and the real breakage was a class §7 did not cover at
     all — decision 7's TOTALITY silently inverting three `l1_runner.rs` assertions. Both agents
     found that class independently, which is why it is now a second table in §7 rather than a
     sentence. Three of the four HIGH were about sites/mutations being AIMED WRONG, not about facts
     being false — the fact-check found the story's own "eight" vs "eleven" contradiction, which was
     the seventh consecutive story to carry a number falsified inside its own file. -->

## Story

As the release gate,
I want traps expecting a rule level the current engine does not implement to be counted in a named,
**blocking** bucket,
so that a green gate can never mean *"we did not ask the question"*.

**This story turns the committed gate RED, on purpose.** After story 5.7 the corpus reports
`24 discovered, 13 scored, 0 failures, passed = true` — green while **eleven** of its twenty-four
traps were never put to any engine. That is the decoration D18 refuses by name: *"a gate that cannot
fall is decoration"*. After this story the same corpus reports **11 unanswerable** and
`passed() == false`, and it stays false until Epic 6 implements `l2-*`. **That is the deliverable,
not a regression.**

**What this story does NOT do**, so the boundary is explicit and not discovered in review:

- it does **not** design or implement any `l2-*` rule — Epic 6. This story makes the absence
  countable, never smaller;
- it does **not** persist anything — story 5.9;
- it does **not** produce a `ScoredRecord` or fill `VerdictVectorEntry` — re-owned by story 5.7 to
  *"the story that gives a trap run a real capability snapshot"*, with the obstacle measured
  (`deferred-work.md`, 5.7's section);
- it does **not** call the blocker. `identity::blocking::candidates` still has no production caller
  after this story, by the same decision 5.7 recorded: a trap NAMES the pair it puts under
  judgement, so a runner has nothing to generate. Owner stays story 5.9 / Epic 6;
- it does **not** make an *absent* map entry blocking (§4). 4.6b's AC1 — *"it reports truth-table
  failures per D18 column and is GREEN vacuously — it must not require an engine to exist"*
  [`epics.md:1055`] — is preserved, and the residual question is registered rather than silently
  widened.

**Nothing under `fixtures/` moves.** No artefact bytes, no `MANIFEST.toml`, no re-hash. This story
**reads** the corpus and never writes it. If any step appears to require re-authoring a committed
artefact, **STOP** — that is a finding, reported rather than absorbed.

**`architecture.md` is NOT edited** (issue #54 for D13's short table; a milestone act).
**`architecture-views.md` is NOT regenerated** (issue #50).
**`epics.md` IS edited by this story, and only at `:1545`** — see AC8. It is the one place where
this project's *"verify-only"* rule is lifted, because 5.7 registered the correction with **this
story as owner** and leaving a false premise standing in the epic file is the defect six consecutive
reviews have caught.

⚠️ **Branch from `master`.** Measured at contexting: `master` is at **`60107fa`**, the working tree
is **clean**, and the workspace reports **352 tests** (153 bin + 153 core + 46 xtask) under
`cargo test --workspace --locked`.

---

## What this story inherits, measured rather than assumed

Everything below was measured at contexting on `60107fa`, by reading the committed TOML directly and
by reading the tree. **The dev re-derives none of it; a surprise reads as a FINDING.**

### 1. 🔴 The premise `epics.md:1545` gives this story is WRONG, and the correction is owed to it

`epics.md:1545`: *"the **8** traps whose expected rule is `l2-*`, which the L1 engine cannot
answer"*. Measured by story 5.7 and re-measured here: **eleven** committed traps are unanswerable at
L1, in **three** classes. Only the first is the one that premise names.

| # | class | n | why L1 cannot answer it |
|---|---|---|---|
| 1 | the expectation names a rule at an unimplemented level (`l2-*`) | **8** | the level is not implemented |
| 2 | the expectation names a **cause and no rule**, and the trap names a pair | **2** | there is no level to route on |
| 3 | the trap does not put a **pair** under judgement | **1** | there is nothing `decide_pair` can be asked |

⚠️ The *"and the trap names a pair"* clause in class 2 is **§4's pair-first decision, already
applied** — it is what makes these three classes mutually exclusive and gives 8 / 2 / 1. §4 records
why, and mutation M4 measures the alternative.

The three `must-abstain` traps (`hostname-absence-must-abstain`, `shared-hardware-vm-must-abstain`,
`example-must-abstain`) are invisible to an `l2-*` selector because `Expectation::MustAbstain`
carries a **cause**, so `Expectation::rule()` returns `None` for all three
[`crates/opencmdb-core/src/trap.rs:107-113`].

**The eleven, as literals** — already asserted by name in
`l1_runner`'s `expected_unanswered()` [`crates/opencmdb-bin/src/l1_runner.rs:326-347`]:

```
cloned-mac-must-not-merge            docker-veth-must-merge
example-must-abstain                 hostname-absence-must-abstain
multi-nic-must-merge                 multi-nic-must-not-merge
shared-hardware-vm-must-abstain      shared-hardware-vm-must-merge
shared-hardware-vm-must-not-merge    vrrp-virtual-mac-must-not-merge-bearers
vrrp-virtual-mac-must-not-merge-master
```

### 2. The whole committed corpus, measured trap by trap — the table every count below comes from

Read from the ten committed `.toml` files at contexting. **24 traps, 10 files, 9 families + 3
family-less example traps.**

| trap | family | column | expected rule | obs | class |
|---|---|---|---|---|---|
| `cloned-mac-must-merge` | cloned-mac | must-merge | `l1-exact-mac` | 2 | **answered** |
| `cloned-mac-must-not-merge` | cloned-mac | must-not-merge | `l2-different-hostname` | 2 | 1 |
| `dhcp-churn-must-merge` | dhcp-churn | must-merge | `l1-exact-mac` | 2 | **answered** |
| `dhcp-churn-must-not-merge` | dhcp-churn | must-not-merge | `l1-distinct-mac` | 2 | **answered** |
| `docker-veth-must-merge` | docker-veth | must-merge | `l2-uplink-agrees` | 2 | 1 |
| `docker-veth-must-not-merge` | docker-veth | must-not-merge | `l1-distinct-mac` | 2 | **answered** |
| `hostname-absence-must-abstain` | hostname-absence | must-abstain | *(a cause)* | 2 | 2 |
| `hostname-absence-must-merge` | hostname-absence | must-merge | `l1-exact-mac` | 2 | **answered** |
| `hostname-absence-must-not-merge` | hostname-absence | must-not-merge | `l1-distinct-mac` | 2 | **answered** |
| `hostname-collision-must-merge` | hostname-collision | must-merge | `l1-exact-mac` | 2 | **answered** |
| `hostname-collision-must-not-merge` | hostname-collision | must-not-merge | `l1-distinct-mac` | 2 | **answered** |
| `multi-nic-must-merge` | multi-nic | must-merge | `l2-uplink-agrees` | 2 | 1 |
| `multi-nic-must-not-merge` | multi-nic | must-not-merge | `l2-different-switch` | 2 | 1 |
| `randomized-mac-must-merge` | randomized-mac | must-merge | `l1-exact-mac` | 2 | **answered** |
| `randomized-mac-must-not-merge` | randomized-mac | must-not-merge | `l1-distinct-mac` | 2 | **answered** |
| `shared-hardware-vm-must-abstain` | shared-hardware-vm | must-abstain | *(a cause)* | 2 | 2 |
| `shared-hardware-vm-must-merge` | shared-hardware-vm | must-merge | `l2-hostname-agrees` | 2 | 1 |
| `shared-hardware-vm-must-not-merge` | shared-hardware-vm | must-not-merge | `l2-different-hostname` | 2 | 1 |
| `vrrp-virtual-mac-must-merge` | vrrp-virtual-mac | must-merge | `l1-exact-mac` | 2 | **answered** |
| `vrrp-virtual-mac-must-not-merge-bearers` | vrrp-virtual-mac | must-not-merge | `l2-different-hostname` | 2 | 1 |
| `vrrp-virtual-mac-must-not-merge-master` | vrrp-virtual-mac | must-not-merge | `l2-virtual-mac-prefix` | 2 | 1 |
| `example-must-abstain` | *(none)* | must-abstain | *(a cause)* | **1** | 3 |
| `example-must-merge` | *(none)* | must-merge | `l1-exact-mac` | 2 | **answered** |
| `example-must-not-merge` | *(none)* | must-not-merge | `l1-distinct-mac` | 2 | **answered** |

**Exactly one trap in the corpus names other than two observations** (`example-must-abstain`, one).
Every other trap names exactly two.

### 3. 🔴 The per-column arithmetic — the assertion that makes *"never leaves the denominator"* checkable

Derived from §2 and re-checked by hand. **This is the strongest single guard the story ships**: it
is the only assertion that fails if a trap silently disappears from BOTH the scored set and the
bucket.

| column | in the corpus | scored (5.7) | unanswerable (this story) |
|---|---|---|---|
| `must-merge` | 10 | 7 | **3** |
| `must-not-merge` | 11 | 6 | **5** |
| `must-abstain` | 3 | 0 | **3** |
| **total** | **24** | **13** | **11** |

`scored_in(c) + unanswered_in(c) == corpus_total_in(c)` for all three columns, and `13 + 11 == 24`.

⚠️ The `must-abstain` **3** decomposes as 2 of class 2 and 1 of class 3, which is what §4's ordering
decision fixes. Under the other ordering it would be 3 / 0 and the class counts published to the
register by story 5.7 would become false.

### 4. The classification ORDER is a decision with two defensible answers, and it is taken here

`example-must-abstain` is in **two** classes at once: it names a cause and no rule (class 2) **and**
it names one observation (class 3). Whichever predicate the runner consults first wins.

- **Level-first** (the order `l1_answers` uses today, `l1_runner.rs:235-243`) → **8 / 3 / 0**.
- **Pair-first** → **8 / 2 / 1**.

**Decision: PAIR-FIRST.** Two reasons, both checkable:

1. story 5.7 measured, published and registered **8 / 2 / 1** in `deferred-work.md:1942-1946` and in
   `l1_runner.rs:326-332`. Shipping 8 / 3 / 0 would falsify a register entry inside the story that
   inherits it;
2. *"there is no pair at all"* is the more fundamental impossibility: class 2 says the engine cannot
   be **routed**, class 3 says the engine cannot be **asked**. A trap with no pair is unanswerable
   at every level, present and future, so it must not be filed under a level.

⚠️ **The ANSWERED SET is invariant under the ordering** — a trap that fails both predicates is
unanswerable either way, so only the CAUSE moves. `l1_answers`'s thirteen keys and 5.7's
`expected_answered()` are untouched by this reordering. Say so in the code, because a reviewer will
otherwise read the reorder as a behaviour change.

### 5. 🔴 `Unanswerable` is NOT an abstention, and the difference is the whole story

An **abstention** is an answer the engine gave; it can pass the `must-abstain` column and it is what
`Outcome::Abstained` records. **Unanswerable** means the engine was never asked. Collapse the two and
the gate passes by declining — which is precisely D18's cowardice, moved up one level from the engine
to the harness.

Measured, and it is not hypothetical: `example-must-abstain`'s expectation is `must-abstain`. Map its
unanswerable state onto `Outcome::Abstained` and `score`'s bottom-right cell returns **`Pass`**
[`score.rs:198`] — a trap that passes because nothing was asked, put into the `must-abstain` column
of a gate that never ran. Story 5.7 refused exactly this shortcut at the runner
[`l1_runner.rs:171-186`]; this story must refuse it again at the report, where the temptation is
`Answer::Unanswerable => Outcome::Abstained`.

⇒ **`Answer::Unanswerable` never reaches `Tally::record`, never produces a `Score`, and never
touches `scored()`.** AC9's **M3** is the mutation that proves it.

### 6. What changes in the seam, and why the signature change is legitimate

`score_corpus(traps_root, answers: &BTreeMap<TrapId, Outcome>)` cannot express *"the producer ran
and declined, for this reason"*. Absence cannot carry a reason, and `Report::passed()` — which
`epics.md:1551` requires the bucket to block — reads only `Report`'s own fields
[`trap_gate.rs:158-163`], and the `Report` is CONSTRUCTED inside `score_corpus` [`:297-302`], so the
bucket must arrive through that parameter.

**The value type widens; the arity does not.** `&BTreeMap<TrapId, Answer>` where

```rust
pub enum Answer {
    Answered(Outcome),
    Unanswerable { cause: UnanswerableCause },
}
```

Three things make this the right shape rather than a second parameter:

- **a trap answered AND declared unanswerable is unrepresentable**, not merely invalid — `trap.rs`'s
  own stated idiom for `Expectation` (*"so 'a merge that also names an abstention cause' is
  unrepresentable instead of merely invalid"*). A second map would need a runtime guard;
- **4.6b's AC1 survives literally.** Its words are *"it must not require an engine to exist"*
  [`epics.md:1055`], and the implementation's own restatement of it is *"must not take an engine
  parameter"* [`trap_gate.rs:28`] — that phrase occurs nowhere in `epics.md`, so quote it from the
  code. An `Answer` is **data** — no trait, no callback, no producer — so the structural guarantee
  `trap_gate.rs:8-30` states is unchanged and must be re-stated, not quietly relied on;
- **it is the smaller edit.** Measured: **26** `score_corpus(&…)` call sites (25 in `trap_gate.rs`,
  1 in `l1_runner.rs`). **13 pass `&BTreeMap::new()`** and need no change — the element type is
  inferred. The other **13** pass a named `answers` binding: those produced by `l1_answers` need no
  call-site edit either (that function's own return type changes), and the hand-built ones need one
  `Answer::Answered(..)` wrap per `insert` — **13 `insert` sites** measured in `trap_gate.rs`. A
  second parameter would touch all 26.

### 7. Exactly ONE existing assertion flips, and it is measured rather than predicted

`grep -rn "passed()" crates/` gives **eleven** live `passed()` assertions — **four** assert TRUE and
**seven** assert FALSE. (Seven further hits are comments and one, `score.rs:978`, is a test *name*.)
Under §6's design — where the bucket is filled only by an explicit `Answer::Unanswerable`, never by
absence — **four** tests build a report over the committed corpus from `l1_answers` and will each
see an 11-entry bucket [`trap_gate.rs:510`, `:540`, `:558`, `:657`/`:662`]. Only the FIRST asserts
`passed()`; the other three assert `scored_in`, render substrings and report equality, all of which
the widening leaves unchanged (AC6 appends to the render, never rewrites). **No test outside those
four constructs an `Answer::Unanswerable`.**

| site | asserts | after this story |
|---|---|---|
| `trap_gate.rs:534` `the_committed_corpus_is_scored_by_the_l1_engine` | **true** | 🔴 **flips to false** — the deliverable |
| `trap_gate.rs:830` `passed_is_the_failures_gate_with_a_discovered_floor` | **true**, on a vacuous run over the committed corpus | unchanged (empty map ⇒ empty bucket) — this is 4.6b's AC1, live |
| `trap_gate.rs:1109` `a_right_verdict_by_the_right_rule_leaves_the_gate_green` | **true** | unchanged (scratch corpus, every trap answered) |
| `trap_gate.rs:1227` `a_two_sided_family_leaves_the_gate_green` | **true** | unchanged (scratch corpus, empty map) |
| `trap_gate.rs:642` · `:815` · `:882` · `:1057` · `:1175` · `:1253` · `l1_runner.rs:744` | `!passed()` | unchanged — already red for another reason |

**If more than one `passed()` assertion flips, that is a FINDING** — it means the bucket is being
filled by absence somewhere, and decision 4's 4.6b guarantee has been broken without saying so.
`trap_gate.rs:830` is the canary: it is the only site that asserts a green gate over the COMMITTED
corpus with an empty map.

⚠️ **`passed()` is the only class §7 scopes, and it is NOT the only class that moves.** Decision 7
(totality: `l1_answers` returns 24 entries, not 13) inverts **three further assertions in
`l1_runner.rs`, by construction**. They are rewritten by AC7 — they are *not* the FINDING condition
above, and a dev who reads only the table will diagnose them as one. Both validation agents found
this class independently; the gap-hunt agent hit all three under a compiler.

| site | what breaks | how it breaks |
|---|---|---|
| `:352` `the_committed_corpus_yields_thirteen_l1_answers` | `assert_eq!(answers.len(), 13)` [`:360`] | **silently** — 24 now |
| `:367` `the_eleven_unanswered_traps_are_named_one_by_one` | derives the residue as `all.difference(answers.keys())` [`:372-375`], which is now **EMPTY** | **silently** — re-express as a filter on `Answer::Unanswerable`, which is what AC4 actually wants |
| `:534` `the_four_pure_l1_families_pass_in_both_decision_poles` | `run_trap(&…expect, outcome)` [`:549`] and `!answers.contains_key("hostname-absence-must-abstain")` [`:556`] | the first is **compiler-carried**, the second **silent** — it becomes `matches!(…, Answer::Unanswerable { .. })` |

🔴 **Two of the four are SILENT** — they compile and fail at runtime with a message that totality
makes misleading. The middle one matters beyond bookkeeping: a set difference against the map's KEYS
is the wrong derivation once the map is total, and it is the very *"eleven by name"* assertion AC4
must preserve.

### 8. The three MIXED families, named trap by trap (`epics.md:1553-1555`)

A family does **not** move as a block, and its completeness check is **not** a failure of Epic 5.

| family | scored at L1 | bucketed |
|---|---|---|
| `cloned-mac` | `cloned-mac-must-merge` | `cloned-mac-must-not-merge` (`l2-different-hostname`) |
| `docker-veth` | `docker-veth-must-not-merge` | `docker-veth-must-merge` (`l2-uplink-agrees`) |
| `vrrp-virtual-mac` | `vrrp-virtual-mac-must-merge` | `-must-not-merge-bearers`, `-must-not-merge-master` |

⚠️ **`vrrp-virtual-mac` holds THREE traps, not two** — and so does `hostname-absence` (which is
*"pure L1"* per `epics.md:1527` yet carries a `must-abstain`). Do not assume two traps per family.

⚠️ **`incomplete_families()` must stay EMPTY and that is not a claim about the answers.**
`incomplete_families` is computed over ALL discovered traps [`trap_gate.rs:301`], so it reports
corpus SHAPE and is answer-independent by construction. It is already asserted empty at
`trap_gate.rs:530-533` with that reasoning spelled out; this story asserts it again **beside** a
non-empty unanswerable bucket, which is the only way to show the two buckets are orthogonal.
`epics.md:1555`: *"the family does not move as a block, and its completeness check is not read as a
failure of Epic 5."*

### 9. The inherited debt this story owns — `l1_answers` has no cross-file id guard

Registered by 5.7's code review with **story 5.8 named as owner**
[`deferred-work.md:2026-2036`]: `answers.insert(trap.id.clone(), …)`
[`crates/opencmdb-bin/src/l1_runner.rs:249`] walks every discovered trap file and blindly inserts;
`TrapFile::validate` enforces uniqueness only **within** a file. Composed with `score_corpus` nothing
ships wrong today — `FixtureError::DuplicateTrapId` [`trap_gate.rs:259-265`] fires first — but the
register names the reason this story owns it: *"the residue arithmetic story 5.8 is about to write
is precisely that shape of caller"*, one that reads `answers.len()` alone.

After this story `l1_answers` is **TOTAL over the corpus** (24 entries, not 13), so a duplicate id
silently shortens the map by one and the residue arithmetic in §3 reads a wrong denominator with no
diagnostic. Measured: the committed corpus has **24 distinct ids across 10 files**, so nothing reds
today — the guard needs a scratch corpus.

Reuse `FixtureError::DuplicateTrapId { trap, first, second }` — the variant already exists and
already renders the right sentence [`fixtures.rs:275-279`, `:429-439`]. **Do not add a variant.**

### 10. Baseline, gates and the traps that cost an hour if they are not read

- **master `60107fa`, clean tree, 352 tests** (153 bin + 153 core + 46 xtask), measured under
  `cargo test --workspace --locked`.
- Code lines (before the first `#[cfg(test)]`, which is what the `file-size` gate counts; ceiling
  **2000**): `score.rs` **681**, `trap_gate.rs` **405**, `l1_runner.rs` **259**, `trap.rs` **409**,
  `fixtures.rs` **728**.
- Six gates must stay green: `cargo xtask ci` — frontier (D47), DDL collation (D64), retired
  vocabulary (D65), corpus lock (both directions), `file-size`, `float-free`. The informational
  `views-hash` reports `ℹ STALE` and exits 0 **by design** — do not regenerate (issue #50).
- ⚠️ **The CI form of clippy is `cargo clippy --workspace -- -D warnings`, WITHOUT `--all-targets`.**
  Run **both** forms. An import kept alive only by a test module is an `unused_imports` error in the
  form CI runs and invisible in the other — this has reddened CI twice in this epic.
- 🔴 **The `float-free` trap.** Under `crates/opencmdb-core/src/identity/` the gate strips `//`,
  `///` and `//!` but **not** block comments and **not** string literals on a code line. A bare
  story number is a float literal: `assert!(true, "story 5.8 …")` under `identity/` **REDS**.
  `score.rs`, `trap_gate.rs` and `l1_runner.rs` are all **outside** the guarded subtree, so this
  story's own files are unaffected — it bites only if a doc pointer is added under `identity/`.
- ⚠️ `DATABASE_URL` is usually unset locally and the MariaDB-backed tests `return` early — a green
  suite says nothing about the database. Irrelevant here, stated so it is not re-derived.
- ⚠️ **Issue #38 (unexplained local test non-determinism) RECURRED on `master` at `d47631b`**: 2 red
  runs out of ~11, on two DIFFERENT tests, then 8 consecutive green; clean tree, all 25 fixture
  sha256s green during the red runs, CI green on the same commit. **It recurred AGAIN during this
  story's validation**, in a clean isolated worktree: 1 red in ~11 runs, on
  `fixtures::tests::a_decision_carrying_an_abstention_cause_is_refused` — a **third distinct test**,
  not reproduced in 10 subsequent runs, six gates green. That the failure keeps MOVING between tests
  is what the recurrences add, and it argues against any single test being the defective one.
  **The cause is OPEN.** If a red appears that does not reproduce, re-run before diagnosing — and
  **do not adopt a cause without naming the check that would have failed if it were wrong.**

### 11. The END STATE, measured by an agent that implemented this story before you did

The validation's gap-hunt agent built AC1–AC10 in an isolated worktree and ran the full gate. These
are **targets, not predictions** — a divergence is a FINDING, not a variation.

| | before | after |
|---|---|---|
| tests | 352 (153 bin + 153 core + 46 xtask) | **364** (163 bin + 155 core + 46 xtask) |
| `trap_gate.rs` code lines | 405 | **552** |
| `l1_runner.rs` code lines | 259 | **323** |
| `score.rs` code lines | 681 | **751** |
| `file-size` largest file | — | **1136** (ceiling 2000) |
| `float-free` | 4 files under `identity/` | **4** — unchanged, none of the three touched files is there |

`cargo fmt --check` clean · **both** clippy forms clean · six `xtask ci` gates green ·
`git status fixtures/` **empty**.

**Every numeric prediction in §1–§8 reproduced exactly**, with no divergence: 24 / 13 / 11, causes
8 / 2 / 1, per-column unanswerable 3 / 5 / 3, corpus per-column 10 / 11 / 3, `scored_in` 7 / 6 / 0,
`passed() == false`, the render containing `"11 unanswerable"`, 24 distinct ids across 10 files,
26 call sites, 13 empty-map, 13 `insert`.

**§7's table is this story's strongest verified claim**: after the seam change and *before any new
test existed*, the suite produced three failures and **exactly one** was a `passed()` site —
`trap_gate.rs:534`. The other ten kept their value, the canary `trap_gate.rs:830` included.

⚠️ **What implementing it found that reading it could not** — and this is the third consecutive
story with that shape: M5 never reached the guard it was written for (§AC5, AC9's M5b), totality
inverts three `l1_runner.rs` assertions **silently** (§7's second table), and three `l1_runner.rs`
doc sites are falsified by §4's own decision rather than by the bucket (AC8). None of the three was
visible without a compiler.

---

## Decisions taken at contexting

Recorded so they are not re-litigated at review. Each carries the measurement or the quote behind it,
above.

1. **The seam widens in VALUE, not in arity**: `&BTreeMap<TrapId, Answer>`, `Answer` in
   `opencmdb-core/src/score.rs` beside `Outcome` (§6). `score_corpus` still takes no engine, no
   callback, no closure, and `trap_gate.rs`'s file-level guarantee is restated rather than assumed.
2. **`UnanswerableCause` has three variants**, matching §1's three measured classes, and it is
   exhaustive with no `_` arm anywhere it is matched — a fourth class must break the build.
3. **Classification is PAIR-FIRST** ⇒ 8 / 2 / 1 (§4), because 8/2/1 is what 5.7 registered and
   because "cannot be asked" outranks "cannot be routed".
4. **An ABSENT map entry is not bucketed and does not block** — 4.6b's AC1 stands (§6).
   `Report::unaccounted()` (`discovered − scored − unanswered.len()`) exists as an **accessor only**:
   it is **NOT rendered**, and AC6's render list is exhaustive. _(Corrected at validation: this
   decision originally said "and rendered when non-zero", which AC6 contradicted. The agent that
   implemented it confirmed the accessor has **no production and no render consumer** — it is
   reachable from tests, where AC2's empty-map assertion gives it one and mutation **M8** proves that
   assertion load-bearing. A public accessor nothing consumes would otherwise be dead surface, so it
   is kept deliberately and the reason is written down.)_ The residual question — *"should a
   non-empty but partial map block?"* —
   is **registered in `deferred-work.md`, not decided here**: deciding it would overturn an
   epic-level AC inside a story that was not given it.
5. **`Unanswerable` never becomes an `Outcome`.** No `From`, no helper, no default. It never reaches
   `Tally::record` (§5).
6. **The NFR4 line is CONDITIONAL on the bucket being non-empty.** An unconditional
   *"NFR4 NOT MET"* becomes a false sentence the day Epic 6 lands, and this project has caught a
   doc falsified by its own commit in six consecutive reviews. Tied to the bucket, the sentence
   deletes itself when the bucket empties.
7. **`l1_answers` becomes TOTAL over the corpus** — 24 entries, 13 `Answered` + 11 `Unanswerable`.
   Totality by construction is what makes a producer unable to leave a trap out silently; the guard
   of §9 protects the count that totality now carries.
8. **`answer_trap` keeps its signature** (`Result<Option<Outcome>, FixtureError>`) and stays
   level-blind. It is what the wrong-rule demonstration reaches
   [`trap_gate.rs:594`], and changing it would move a test that is not this story's subject.
9. **No new `FixtureError` variant.** `DuplicateTrapId` already exists and already says the right
   thing (§9).
10. **`epics.md:1545` IS corrected by this story** (AC8) — the one exception to the verify-only
    rule, because 5.7 registered the correction with this story as owner.

---

## Acceptance Criteria

**AC1 — the vocabulary exists, is domain data, and cannot be mistaken for an abstention.**
**Given** that *"an error there is domain data, not a string"* (D47) and that an abstention is an
ANSWER while unanswerable means the engine was never asked (§5)
**When** the vocabulary is written
**Then** `crates/opencmdb-core/src/score.rs` gains

```rust
pub enum UnanswerableCause {
    LevelNotImplemented { expected: RuleId },
    NoLevelToRouteOn,
    NoPairUnderJudgement,
}

pub enum Answer {
    Answered(Outcome),
    Unanswerable { cause: UnanswerableCause },
}
```

each `pub` item, variant and field carrying a `///` (house rule), with docs that state: the three
classes of §1 with their measured counts; that `LevelNotImplemented` carries the rule the trap's
AUTHOR named, never one the engine chose; and — **on `Answer` itself** — that
`Unanswerable` is **not** an abstention, with §5's measurement that mapping it to
`Outcome::Abstained` makes `example-must-abstain` pass.
**And** there is **no** `From<UnanswerableCause> for Outcome`, no `Default`, and no function
anywhere that turns an `Answer::Unanswerable` into an `Outcome` — the same refusal, for the same
reason, that keeps `outcome_of` a named function rather than a `From` impl [`score.rs:299-304`].

**AC2 — the seam carries the bucket, and the harness still runs no producer.**
**Given** `score_corpus(traps_root, answers: &BTreeMap<TrapId, Outcome>)`, whose `answers` map
cannot express a declined trap (§6)
**When** the seam is widened
**Then** its parameter becomes `&BTreeMap<TrapId, Answer>`; its **arity is unchanged**; it takes no
engine, no callback and no closure; and `trap_gate.rs`'s module doc is updated to say **why the
widening does not spend 4.6b's AC1** — an `Answer` is data.
**And** an `Answer::Unanswerable` key that names no discovered trap is refused by the existing
`FixtureError::AnswerForUnknownTrap` path exactly as an `Answered` one is (the check is on keys; do
not let the `Unanswerable` arm skip `used.insert`).
🔴 **And a test asserts that the ten surviving `passed()` assertions of §7 keep their value** —
concretely: `score_corpus(committed_root, &BTreeMap::new()).passed()` is still **true**, which is
4.6b's AC1 in one line. If it is false, decision 4 has been broken.

**AC3 — the fourth bucket exists on `Report`, is named, and BLOCKS.**
**Given** `epics.md:1547` — *"counted as NOT PASSING in a fourth named bucket, beside truth-table
failures, rule mismatches and incomplete families — they never silently leave the denominator"*
**When** the report is built
**Then** `Report` gains a fourth field with an accessor `unanswered() -> &[Unanswered]`, where

```rust
pub struct Unanswered {
    pub trap: TrapId,
    pub column: Column,
    pub cause: UnanswerableCause,
}
```

**And** `Report::passed()` requires `self.unanswered.is_empty()` **in addition to** the three
existing conditions — added, never replacing one.
**And** the doc on `passed()` is corrected: it currently says *"A real corpus with no engine yet
(discovered > 0, scored == 0) DOES pass"* [`trap_gate.rs:156-157`] — still true for an EMPTY map, and
now false for a run that declined, so the sentence must distinguish the two rather than be left
standing. **The same sentence appears a second time** on
`passed_is_the_failures_gate_with_a_discovered_floor`'s doc [`trap_gate.rs:822-824`] — correct both.
One twin updated and the other missed is the HIGH finding of 5.7's review.

**AC4 — the committed corpus goes RED, with the eleven named one by one.**
**Given** the committed corpus, whose report has read `passed = true` since story 5.7 while eleven
traps were never asked
**When** it is scored through `l1_answers`
**Then** a test asserts `discovered() == 24`, `scored() == 13`, `failures() == 0`,
`rule_mismatches().is_empty()`, `incomplete_families().is_empty()`, `unanswered().len() == 11`
and **`passed() == false`**.
**And** the eleven `TrapId`s in `unanswered()` are asserted to be **exactly** §1's eleven literals —
by NAME, not by count: *a residue that can grow in silence is how a gate quietly stops testing*
(story 5.6's idiom).
**And** the three causes are asserted by class: **8** `LevelNotImplemented`, **2** `NoLevelToRouteOn`,
**1** `NoPairUnderJudgement` (§4's ordering decision) — and each `LevelNotImplemented` carries the
`l2-*` id §2's table gives it.
**And** `the_committed_corpus_is_scored_by_the_l1_engine` [`trap_gate.rs:509`] has its
`assert!(report.passed())` **flipped with the reason written in the assertion's own message**, not
deleted.

**AC5 — the per-column arithmetic proves nothing left the denominator.**
**Given** §3's table and D18's per-column reading
**When** the corpus is scored
**Then** `Report` exposes `unanswered_in(column) -> usize`, and a test asserts — **in this order,
which is load-bearing** — first that for every column `scored_in(c) + unanswered_in(c)` equals the
number of traps the corpus carries in that column (**10 / 11 / 3**, read from the discovered traps
rather than hard-coded twice), and only THEN the three literals `unanswered_in(MustMerge) == 3`,
`unanswered_in(MustNotMerge) == 5`, `unanswered_in(MustAbstain) == 3`.
🔴 **The order is not style.** Measured by the validation agent: with the literals first, mutation
M5 panics on `unanswered_in(MustMerge)` (`left: 0 right: 3`) and **never reaches the arithmetic
loop at all** — the guard the story calls its strongest is then protected by nothing, and M5 gives
a green-looking red. Putting the loop first makes both reachable, and **M5b** (AC9) is the mutation
that proves the loop specifically.
**And** the test's own message states what the equality means: a trap that vanished from **both**
sets is the only thing it can catch, and it is the failure `discovered` alone cannot see.

**AC6 — the report says plainly how many, why, and that NFR4 is NOT MET.**
**Given** `epics.md:1551` — *"its output states plainly how many traps were unanswerable at this
level and why"* — and `:1557` — *"the gate's own report names NFR4 as NOT MET at this epic, at the
device level, closed by Epic 6"*
**When** `Report` is rendered
**Then** the first line gains a **third** count suffix, `", N unanswerable"`, appended **after**
`wrong-rule` and `incomplete-famil{y|ies}` so the existing substrings stay byte-stable and every
4.6b/4.7a/4.7b assertion on them keeps passing; each unanswered trap then gets its own line naming
the trap, its column and its cause **in words** (the `l2-*` id for `LevelNotImplemented`).
**And** a final line names NFR4 as NOT MET, at the DEVICE level, closed by Epic 6 — **rendered only
when the bucket is non-empty** (decision 6).
**And** two tests pin both halves: the committed corpus's render contains `"11 unanswerable"` and the
NFR4 line; a report with an EMPTY bucket contains **neither** — so the claim deletes itself when
Epic 6 empties the bucket, instead of relying on someone remembering.

**AC7 — the runner is TOTAL over the corpus, and the mixed families are asserted trap by trap.**
**Given** decision 7 and §8
**When** `l1_answers` is rewritten
**Then** it returns `BTreeMap<TrapId, Answer>` with an entry for **every** discovered trap — 24 over
the committed corpus, 13 `Answered` and 11 `Unanswerable` — classified pair-first (§4), with
`expects_an_l1_rule` and `named_pair` kept as **separate named predicates** so a mutation can hit
either alone (5.7's AC2, preserved).
🔴 **Two predicates cannot produce three causes, and the bridge is prescribed rather than left to
taste.** `expects_an_l1_rule(&Trap) -> bool` collapses `NoLevelToRouteOn` and `LevelNotImplemented`.
Do **not** add a third predicate — it would either duplicate the `l1-` prefix test or leave
`expects_an_l1_rule` dead. Re-consult `trap.expect.rule()` **inside that predicate's false arm**:

```rust
match named_pair(trap) {
    None => Answer::Unanswerable { cause: NoPairUnderJudgement },
    Some((a, b)) if !expects_an_l1_rule(trap) => Answer::Unanswerable {
        cause: match trap.expect.rule() {
            None => NoLevelToRouteOn,
            Some(r) => LevelNotImplemented { expected: r.clone() },
        },
    },
    Some((a, b)) => Answer::Answered(answer_pair(…)),
}
```

Measured by the validation agent: this shape compiles, keeps both predicates live and mutable, and
yields 8 / 2 / 1.
**And** a test asserts the answered thirteen are **unchanged** from 5.7's `expected_answered()` —
the reordering moves causes, never keys (§4).
**And** a test asserts the three MIXED families §8 names, trap by trap: which id is `Answered`, which
is `Unanswerable` and with which `l2-*` rule — **and asserts `incomplete_families()` is empty in the
same report**, with the message stating that completeness is corpus SHAPE and answer-independent, so
a bucketed pole is not a failure of Epic 5.
**And** the two pure-L2 families (`multi-nic`, `shared-hardware-vm`) are asserted **fully** bucketed,
and `hostname-absence` — called *"pure L1"* by `epics.md:1527` — is asserted **2 answered + 1
bucketed**, the narrower true claim written down rather than the wider one assumed.

**AC8 — the 8→11 correction lands where the premise lives.**
**Given** that `epics.md:1545` still says **8** and that story 5.7 registered the correction with
**this story as owner** [`deferred-work.md:1937-1960`]
**When** the story lands
**Then** `epics.md`'s story-5.8 block is corrected to **eleven**, in three classes, with a dated
parenthetical in the file's own idiom naming what it replaced and why — the shape
`epics.md:416` and `:1315` already use. Nothing else in `epics.md` is touched.
**And** `deferred-work.md` gains a `## Deferred from: story-5.8` section, **appended, never
rewriting an existing bullet**, which: strikes the 8→11 entry as CLOSED with the commit that closed
it; strikes the `l1_answers` cross-file-id entry as CLOSED (AC10); registers decision 4's residual
question (*"should a non-empty but partial answers map block?"*) with its measurement and an owner;
and re-states what this story did **not** do — no `l2-*` rule, no `ScoredRecord`, no blocker caller.
**And** every doc site that says the residue is 8, that the committed gate passes, or that story 5.8
owns something, is made true or narrowed. **The list is measured, not "at minimum"** — `grep -rn
'5\.8' crates/ --include=*.rs` gives **exactly seven** code sites:

| site | what is false after this story |
|---|---|
| `trap_gate.rs:38` | *"story 5.8 is what turns that residue into a bucket"* — future tense |
| `trap_gate.rs:118` | same, on `scored()`'s doc |
| `trap_gate.rs:555` | *"the state story 5.8 turns into a blocking bucket"* |
| `l1_runner.rs:36` | *"Story 5.8 turns the residue into a bucket that BLOCKS"* |
| `l1_runner.rs:329` | *"`epics.md:1545` hands story 5.8 the premise that there are **8**"* |
| `l1_runner.rs:332` | *"registered in `deferred-work.md` with story 5.8 as owner"* |
| `l1_runner.rs:694` | *"which is exactly what story 5.8 exists to prevent"* |

plus `docs/project-context.md` and `CLAUDE.md`.
🔴 **`score.rs` carries NO such site** — measured: `grep -c "Report" crates/opencmdb-core/src/score.rs`
is **0**, and `Report` lives in `trap_gate.rs` (`opencmdb-bin`). Do not go looking for one.
🔴 **And three `l1_runner.rs` sites are falsified by §4's own pair-first decision, not by the
bucket** — the validation agent found them only after implementing:
- `:19-21` — *"the seam stays a `BTreeMap<TrapId, Outcome>` … `score_corpus`'s signature and body are
  unchanged"*; AC2 changes both;
- `:25-29` — the numbered selector list gives the `l1-` prefix as **1** and the pair as **2**; §4
  reverses them;
- `:106-110` — *"the three committed `must-abstain` traps are excluded here, before the pair
  condition is ever consulted"*; under pair-first only **two** are, because `named_pair` removes
  `example-must-abstain` first. This is the sentence §4's decision most directly inverts.

**No site is left saying "story 5.8 owns it"** — a promise re-made by the story that was supposed to
keep it is the defect six consecutive reviews have caught.
🔑 **Update BOTH twins of every duplicated sentence.** `CLAUDE.md` and `docs/project-context.md`
carry the same paragraphs; one updated and the other missed was the HIGH of 5.7's review.

**AC9 — every new guard is proven to red before it passes.**
**Given** the house rule (story 1.3)
**When** the work is done
**Then** at least the following mutations are run and **recorded with their observed red set**, each
on a **committed** baseline (`git stash`/commit first — `git checkout <file>` restores to HEAD and
has destroyed an implementation mid-run before):
- **M1** — `passed()` drops the `unanswered.is_empty()` conjunct. Measured: **2** tests red, not one
  — AC4's new test and the flipped `the_committed_corpus_is_scored_by_the_l1_engine`;
- **M2** — `l1_answers` stops emitting `Unanswerable` entries and returns only the thirteen
  `Answered` ones (predict: the committed report is green again with `unanswered() == 0`; this is
  the exact pre-5.8 behaviour, so it measures whether the story's central claim is load-bearing);
- **M3** — 🔴 an `Answer::Unanswerable` is mapped to `Outcome::Abstained` and recorded in the
  `Tally`. Predicted per §5 and **confirmed by the validation agent, which ran it**: `scored()`
  becomes **24**, `must-abstain` scores **3** where it scored 0, and all three `must-abstain`
  traps — `example-must-abstain` included — **PASS**. ⚠️ It also takes `failures()` from **0 → 3**:
  three of the eleven bucketed traps are `must-merge` (`docker-veth-must-merge`,
  `multi-nic-must-merge`, `shared-hardware-vm-must-merge`) and `(must-merge, Abstained)` is D18's
  cowardice cell [`score.rs:190`]. Measured red set: **7 tests**. What M3 proves is that the three
  `must-abstain` traps would have passed — not that the whole report would have. **AC1's refusal is
  therefore NOT decoration**, and that is measured rather than argued;
- **M4** — the classification is flipped to **level-first** (predict per §4: causes become
  8 / 3 / 0, AC4's class counts red, and the answered thirteen stay green — record BOTH halves, the
  green half is the measurement that the reorder is cause-only);
- **M5** — `unanswered_in` returns the failures count instead. Measured: it reds **one** test and
  panics on the **first literal** (`unanswered_in(MustMerge)`, `left: 0 right: 3`), so it never
  reaches the arithmetic loop. It proves the literals, nothing more. Run it, and record that
  boundary rather than claiming the loop;
- **M5b** — 🔴 **the mutation that proves the arithmetic**, and the one M5 was mistakenly believed
  to be: `l1_answers` silently skips ONE trap (`dhcp-churn-must-merge`) so it leaves **both** the
  scored set and the bucket. Predict: the totals loop reds with `column must-merge … left: 9 right:
  10`, carrying its own message; measured red set **9 tests**. This is the only mutation that
  reaches the guard §3 calls the story's strongest;
- **M6** — the NFR4 line is rendered unconditionally (predict: AC6's empty-bucket test reds — the
  test that exists so the sentence deletes itself);
- **M7** — the `l1_answers` cross-file-id guard is removed and a scratch corpus defines one id in
  two files. 🔴 **Measured: `score_corpus` STILL refuses that corpus with `DuplicateTrapId`, so a
  test written through the harness stays GREEN and the new guard is untested.** This is why AC10
  *requires* the call to be `l1_answers` directly. With the direct call the map silently shortens to
  one entry and the test reds;
- **M8** — `unaccounted()` returns 0 unconditionally (predict: AC2's empty-map test reds — the one
  assertion that gives that accessor a consumer).
**And** `git status fixtures/` is empty and `MANIFEST.toml` is untouched at the end — verified, not
assumed.
**And** the full local gate is run before pushing: `cargo fmt --all`, **both** clippy forms,
`cargo test --workspace --locked`, `cargo xtask ci`.

**AC10 — the inherited cross-file id guard is closed where it was registered.**
**Given** `deferred-work.md:2026-2036`, which names this story as owner because it is *"the first
consumer of this map that counts rather than scores"* (§9)
**When** `l1_answers` becomes total
**Then** it refuses a `TrapId` seen in a second file with `FixtureError::DuplicateTrapId { trap,
first, second }` — the existing variant, **no new one** — and a test over a **scratch** trap corpus
(two files, one shared id, streams resolved against the committed corpus per `answer_trap`'s
baked-root limit) asserts the refusal names both paths.
🔴 **The test MUST call `l1_answers` directly, never through `score_corpus`.** Measured under M9's
predecessor: with the runner's guard removed, `score_corpus` **still** refuses that same corpus with
`DuplicateTrapId` [`trap_gate.rs:259-265`], so a test written through the harness stays **green** and
the new guard is untested — it would be measuring `score_corpus`, which already worked. Assert
`score_corpus`'s behaviour in the same test, as the stated reason the direct call is required.
**And** a test asserts the committed corpus has **24 distinct ids across 10 files**, so the guard is
recorded as unreachable-today rather than assumed so.

---

## Tasks / Subtasks

- [x] **Task 1 — the vocabulary (AC1)**
  - [x] `UnanswerableCause` and `Answer` in `crates/opencmdb-core/src/score.rs`, beside `Outcome`.
        Every `pub` item/variant/field documented; `Debug, Clone, PartialEq, Eq` to match `Outcome`.
  - [x] The doc on `Answer` carries §5's measurement — that mapping an unanswerable trap to
        `Outcome::Abstained` makes `example-must-abstain` pass — and refuses the conversion by name.
  - [x] Unit tests in `score.rs`: the type is inert (it produces no `Score`), and no conversion
        exists. **No `From`, no `Default`, no helper.**
- [x] **Task 2 — the seam and the bucket (AC2, AC3)**
  - [x] `score_corpus`'s parameter becomes `&BTreeMap<TrapId, Answer>`; the `Answered` arm keeps the
        existing tally + `run_trap` path **byte-identical**; the `Unanswerable` arm pushes an
        `Unanswered` and **calls neither** `Tally::record` nor `run_trap`.
  - [x] Both arms `used.insert(trap.id)` — the `AnswerForUnknownTrap` check must see an
        `Unanswerable` key too.
  - [x] `Unanswered` struct + `Report::unanswered()` + `Report::unanswered_in()` +
        `Report::unaccounted()` (decision 4: reported, non-blocking).
  - [x] `passed()` gains the fourth conjunct; its doc distinguishes "no producer ran" (still green)
        from "a producer declined" (red).
  - [x] `trap_gate.rs`'s module doc: why the widening does not spend 4.6b's AC1 (§6).
  - [x] Update the **13** map-building call sites with `Answer::Answered(..)`; the **13** that pass
        `&BTreeMap::new()` need no change.
- [x] **Task 3 — the runner becomes total (AC7, AC10)**
  - [x] `l1_answers -> BTreeMap<TrapId, Answer>`, pair-first classification (§4), predicates kept
        separate and named.
  - [x] The cross-file `TrapId` guard, `DuplicateTrapId`, mirroring `score_corpus`'s `seen` map.
  - [x] `answer_trap` **unchanged** (decision 8).
  - [x] Tests in `l1_runner.rs` (subject = the runner): totality (24 entries), the thirteen answered
        unchanged vs `expected_answered()`, the eleven unanswered vs `expected_unanswered()`, the
        three cause classes 8/2/1, the scratch duplicate-id refusal, the 24-distinct-ids assertion.
- [x] **Task 4 — the report tests (AC4, AC5, AC6)**
  - [x] Tests in `trap_gate.rs` (subject = the report): the RED committed corpus with the eleven by
        name and by cause; the per-column arithmetic; the two render tests (bucket non-empty →
        `"11 unanswerable"` + the NFR4 line; bucket empty → neither).
  - [x] Flip `the_committed_corpus_is_scored_by_the_l1_engine`'s `passed()` assertion, message
        first.
  - [x] The mixed/pure family assertions (AC7's family half may live here — put each test with the
        item whose CLAIM it pins, `cascade.rs`'s stated convention).
- [x] **Task 5 — the docs and the register (AC8)**
  - [x] `epics.md`: the story-5.8 block, 8 → eleven, three classes, dated parenthetical. **Nothing
        else in that file.**
  - [x] `deferred-work.md`: a new `## Deferred from: story-5.8` section, appended.
  - [x] Sweep every site claiming the residue is 8 or that the committed gate passes — at minimum
        `trap_gate.rs`, `l1_runner.rs`, `score.rs`, `docs/project-context.md`, `CLAUDE.md`.
        🔑 **Update BOTH twins.** 5.7's HIGH was `CLAUDE.md` and `project-context.md` carrying the
        same sentence with only one corrected.
- [x] **Task 6 — mutations and the gate (AC9)**
  - [x] Commit first, then M1–M7, each red set recorded verbatim in the Debug Log, each red labelled
        **assertion-carried** or **compiler-carried**.
  - [x] `git status fixtures/` empty; `MANIFEST.toml` untouched; full local gate; then branch → PR →
        green CI → squash merge. **`done` is the MERGE's business**, not the review's.

---

## Dev Notes

### Existing shapes to follow, not reinvent

- **The bucket's shape** is `RuleMismatch` [`trap_gate.rs:71-87`]: a `pub struct` beside `Report`,
  every field documented, named so a red gate is debuggable without opening the corpus. `Unanswered`
  is its sibling — do not invent a different idiom.
- **The cause enum's shape** is `IdentityAbstentionCause` / `AbstentionCause`: a closed domain enum,
  exhaustively matched with no `_` arm. The house rule that a new variant must break the build
  applies (`score`'s and `outcome_of`'s docs both state it).
- **The `Display` idiom** is `trap_gate.rs:175-219`: count suffixes on the first line in a **fixed**
  order, appended only when non-zero, then one line per entry. Add third; never reorder.
- **The cross-file id guard** already exists, twenty lines above where you need it —
  `score_corpus`'s `seen: BTreeMap<TrapId, PathBuf>` [`trap_gate.rs:251`, `:259-265`]. Mirror it.
- **Test placement** follows `cascade.rs`'s stated convention: *a test lives with the item whose
  CLAIM it pins*. WHICH traps the runner classifies → `l1_runner.rs`. What the REPORT makes of them
  → `trap_gate.rs`. The type's own inertness → `score.rs`.
- **Scratch corpora**: `scratch_dir(tag)` embeds `std::process::id()` [`trap_gate.rs:421-427`] —
  copy that helper's shape, never a shared constant path.
- **`answer_trap` resolves `replay` against the BAKED corpus root**, never the scratch root
  [`l1_runner.rs:71-78`]. A scratch trap file may only reference committed streams — e.g.
  `scenario/replay/minimal.jsonl`. This is load-bearing, not incidental: it is what lets a scratch
  trap vary an expectation while judging real committed observations.
- **`minimal.jsonl` contains no pair L1 merges** — its three observations carry MAC `…:53:01`, *no
  MAC*, and MAC `…:53:02`. A scratch `must-merge` trap over it is a truth-table failure and can never
  reach `rule_mismatches`. Story 5.7 lost an hour to this; use `must-not-merge` (obs `…0001` /
  `…0003`) if a scratch trap must reach the rule comparison.

### Compile-level facts — each costs an hour if it is discovered under `rustc`

- `#![deny(missing_docs)]` is **ON** for `opencmdb-bin` and `xtask`. Every `pub` item, field and
  variant added to `trap_gate.rs` needs a `///`. It is **OFF** for `opencmdb-core` (~70 outstanding
  field docs) — but the house rule still requires the docs, and clippy runs `-D warnings`.
- An import kept alive **only** by a `#[cfg(test)]` module is an `unused_imports` **error** in
  `cargo clippy --workspace -- -D warnings` (the form CI runs) and invisible with `--all-targets`.
  Put such imports inside `mod tests` — `l1_runner.rs:266-271` is the worked example, with the
  reason in a comment.
- `?` is not allowed in an `or_insert_with` closure; `l1_answers` uses `contains_key` + `insert` for
  exactly that reason [`l1_runner.rs:226-228`]. Keep the shape.
- `RuleId`, `TrapId`, `FamilyId` are `#[serde(transparent)]` newtypes over `String`; `.0` is the
  string. `Column` is `Copy`; `RuleId` is not.
- 🔴 **clippy's `type_complexity` is a HARD ERROR under `-D warnings`** on the natural literal table
  for AC7's mixed-family test — `[(&str, &[&str], &[(&str, &str)]); 3]` fails. Use a small named
  struct. Found by the validation agent under a compiler; it is not in any earlier story's notes.
- **`Expectation::MustAbstain` carries only `cause`. There is no `reason` on it** — `reason` is a
  field of `Trap` [`trap.rs:118+`]. This cost the validation agent one compile error.
- Doc comments on a **tuple-variant field** (`Answered(/// … Outcome)`) are legal under edition 2024
  with `#![deny(missing_docs)]` — verified by probe, so AC1's "every variant and field carries a
  `///`" is satisfiable for `Answer::Answered(Outcome)`.
- `opencmdb-core` **must not** depend on `anyhow`, `axum`, `sqlx` or `askama` (D47, gated). It also
  must not read files — which is why every corpus assertion lives in `opencmdb-bin`.

### What a reviewer will challenge, and the answer that is already measured

1. *"You widened `score_corpus`'s signature — 5.7 kept it unchanged on purpose."* → 5.7's decision 4
   says *"4.6b's AC1 is not spent here"*, which is a deferral, not a prohibition. AC1's words are
   *"must not require an engine to exist"*; an `Answer` is data. §6, and it must be in the module
   doc, not only here.
2. *"The gate is red — you broke the build."* → `epics.md:1551` requires exactly this:
   *"`passed()` is blocked by that bucket exactly as it is by the other three"*. And
   `epics.md:416`: *"Epic 5's commitment is the 13 L1-ruled traps; **NFR4 stays RED and is closed by
   Epic 6**"*. The red is the deliverable.
3. *"An unfed corpus is now red too."* → It is not; §7's table is the measurement, and AC2 asserts
   `score_corpus(root, &BTreeMap::new()).passed() == true` in one line.
4. *"The classification order is arbitrary."* → §4: two defensible answers, one taken, both counts
   published, and M4 measures the difference.
5. *"`incomplete_families()` should be non-empty — three families lost a pole."* → It must not be.
   `incomplete_families` runs over ALL discovered traps [`trap_gate.rs:301`]; it is corpus SHAPE and
   answer-independent. `epics.md:1555`: *"its completeness check is not read as a failure of Epic
   5."* AC7 asserts it empty **beside** a non-empty bucket, which is the only way to show the two are
   orthogonal.

### References

- `_bmad-output/planning-artifacts/epics.md:1537-1557` — story 5.8's ACs (⚠️ `:1545` says **8**; §1).
- `_bmad-output/planning-artifacts/epics.md:416` — the Epic 5 NFR4 arbitration: 13 L1 / 8 L2 / 3
  must-abstain, four pure-L1 families, two pure-L2, **three MIXED**, *"NFR4 stays RED, closed by
  Epic 6"*.
- `_bmad-output/planning-artifacts/epics.md:1055` — 4.6b's AC1, the vacuous-green guarantee.
- `_bmad-output/planning-artifacts/prd.md:1179` — NFR4 as the binary adversarial trap suite.
- `_bmad-output/implementation-artifacts/deferred-work.md:1937-1960` — the 8→11 correction, owner
  5.8.
- `_bmad-output/implementation-artifacts/deferred-work.md:2026-2036` — the `l1_answers` cross-file id
  gap, owner 5.8.
- `_bmad-output/implementation-artifacts/5-7-trap-runner-stops-scoring-nothing.md` — §1's answer
  table, §3's residue, §4's pairless refusal.
- `crates/opencmdb-core/src/score.rs:99-200` — `Column`, `score`, the 3×3 and its load-bearing cell.
- `crates/opencmdb-core/src/score.rs:243-318` — `run_trap`, `outcome_of` and the `From` refusal.
- `crates/opencmdb-core/src/score.rs:444-495` — `Tally`, `failures_in`, `scored_in`.
- `crates/opencmdb-bin/src/trap_gate.rs:98-219` — `Report`, `passed()`, `Display`.
- `crates/opencmdb-bin/src/trap_gate.rs:237-303` — `score_corpus`, `seen`, `used`.
- `crates/opencmdb-bin/src/l1_runner.rs:95-253` — the selector, `named_pair`, `answer_trap`,
  `l1_answers`.
- `crates/opencmdb-core/src/trap.rs:69-140` — `Expectation` (`:69`), `Expectation::rule()` (`:107`),
  `Trap` (`:118`).
- `crates/opencmdb-core/src/trap.rs:362-409` — `IncompleteFamily` (`:362`), `incomplete_families`
  (`:384`).
- `docs/project-context.md` — grounding; `CLAUDE.md` — the four engineering rules and the gates.

---

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context) — `claude-opus-5[1m]`.

### Debug Log References

Baseline committed as `2871ebe` on branch `story-5.8-unanswerable-bucket` before any mutation, per
AC9 (`git checkout <file>` restores to HEAD, and that has destroyed an implementation mid-run in
this project before).

#### 🔴 The §7 prediction, verified BEFORE any new test existed

After the seam change and the runner rewrite, with **no story-5.8 test yet written**, the suite
produced **exactly one** failure:

```
test trap_gate::tests::the_committed_corpus_is_scored_by_the_l1_engine ... FAILED
   panicked at crates/opencmdb-bin/src/trap_gate.rs:690
test result: FAILED. 153 passed; 1 failed
```

§7 predicted one `passed()` assertion would flip and named it. Ten kept their value, the canary
`trap_gate.rs:830` (4.6b's AC1) included. **No divergence.**

#### Mutations — every red assertion-carried, ZERO compiler-carried

| # | mutation | red set | carried by |
|---|---|---|---|
| M1 | `passed()` drops the `unanswered.is_empty()` conjunct | **2** — `…is_red_with_eleven_unanswerable_traps`, `…is_scored_by_the_l1_engine` | assertion |
| M2 | `l1_answers` emits only the 13 `Answered` | **9** | assertion |
| M3 | an `Unanswerable` is recorded in the `Tally` as `Outcome::Abstained` | **6** | assertion |
| M4 | level-first classification | **3** — all three CAUSE assertions | assertion |
| M5 | `unanswered_in` returns the failures count | **1**, *on the arithmetic loop* | assertion |
| M5b | one trap leaves BOTH the scored set and the bucket | **8**, incl. the loop at `9 != 10` | assertion |
| M5c | a REDISTRIBUTION inside one column (added, see below) | **11**, incl. the literal at `4 != 3` | assertion |
| M6 | the NFR4 line rendered unconditionally | **1** — `an_empty_bucket_renders_neither…` | assertion |
| M7 | the runner's cross-file id guard removed | **1** — `one_trap_id_in_two_files_is_refused_by_the_runner_itself` | assertion |
| M8 | `unaccounted()` returns 0 | **1** — `an_absent_answer_is_not_a_decline_and_does_not_block` | assertion |

**M3, probed directly** — the story's central claim, and it holds exactly:

```
M3 scored=24 failures=3 must_abstain_scored=3 must_abstain_failures=0 must_merge_failures=3
```

`scored` 13 → **24**; the `must-abstain` column goes 0 → 3 scored with **0 failures**, i.e. **all
three `must-abstain` traps PASS, `example-must-abstain` included** — a trap passing because nothing
was asked. The three new `failures` are all `must-merge` (D18's cowardice cell). **AC1's refusal is
NOT decoration**, measured rather than argued.

**M4's green half, recorded because it is the measurement**: the answered thirteen, the eleven names
and the per-column arithmetic all stayed GREEN. Only the three cause assertions red
(`no_level: left 3, right 2` — i.e. 8 / 3 / 0). The reorder moves a CAUSE and never a key, exactly
as §4 claims.

#### 🔴 Two corrections to the story's own mutation prescriptions, found by running them

1. **M5 reds on the arithmetic LOOP, not on a literal.** The story says it *"panics on the first
   literal … and never reaches the arithmetic loop"* and that it *"proves the literals, nothing
   more"*. That was true of the ordering the validation agent had; **AC5 changed the ordering, and
   the sentence was not re-derived against it.** Measured here at `trap_gate.rs:807`:
   `column must-merge … left: 7 right: 10` — the loop. So AC5's reordering did what it was for, and
   M5's stated boundary is inherited from a tree that no longer exists.
2. **That left the three literals proven by nothing**, since the loop panics first. **M5c was added
   for exactly that**: it moves one answered `must-merge` trap into the bucket, so the column TOTAL
   is unchanged (10) and the loop stays green, while `unanswered_in(MustMerge)` goes 3 → 4 and the
   literal reds at `trap_gate.rs:819` (`left: 4, right: 3`). M5 and M5c prove different assertions
   and neither masks the other.

#### Divergences from the story's predicted figures, stated rather than smoothed

- **M3 reds 6 tests where the story predicted 7.** The story's figure came from the validation
  agent's implementation; this one keeps the bucket filled *and* records in the tally, which
  isolates the tally effect. The direction and every named consequence reproduced exactly.
- **364 tests, but distributed 162 bin + 156 core** where §11 predicted 163 + 155. Same total; three
  vocabulary tests went to `score.rs` rather than two. Not a divergence in coverage.
- Code lines (measured): `score.rs` 681 → **766** (§11 predicted 751), `trap_gate.rs` 405 → **560**
  (predicted 552), `l1_runner.rs` 259 → **330** (predicted 323). All three run a little longer than
  the validation's tree — longer docs — and all are far under the 2000 ceiling; the `file-size`
  gate's largest file is unchanged at **1136**, which is `xtask/src/main.rs` — not one of these.
  `score.rs` at 766 is now the workspace's **second** largest by code lines, ahead of `fixtures.rs`
  (728); worth knowing, nowhere near the ceiling.

#### Gate, run whole on the restored tree

`cargo fmt --all --check` clean · `cargo clippy --workspace --all-targets -- -D warnings` clean ·
`cargo clippy --workspace -- -D warnings` (**the form CI runs**) clean, exit 0 ·
`cargo test --workspace --locked` **162 + 156 + 46 = 364**, 0 failed ·
`cargo xtask ci` **six gates green** (`file-size` largest 1136; `float-free` still 4 files under
`identity/`) · `git status fixtures/` **empty**, `MANIFEST.toml` untouched.

### Completion Notes List

- **The committed gate is now RED, and that is the deliverable.** `24 discovered, 13 scored,
  0 failures, 0 wrong-rule, 0 incomplete families, **11 unanswerable**, `passed() == false`` — and it
  stays false until Epic 6 implements `l2-*`, which is what `epics.md:416` has always said.
- **AC1–AC10 all MET.** Every numeric prediction in §1–§8 reproduced exactly: 24 / 13 / 11, causes
  8 / 2 / 1, per-column unanswerable 3 / 5 / 3, corpus per-column 10 / 11 / 3, `scored_in` 7 / 6 / 0.
- **The seam widened in VALUE, not arity.** `score_corpus` takes `&BTreeMap<TrapId, Answer>` and
  still takes no engine, no callback, no closure. 4.6b's AC1 survives literally and is asserted in
  one line (`an_absent_answer_is_not_a_decline_and_does_not_block`): an empty map over the committed
  corpus still passes, with `unaccounted() == 24` naming the state.
- **Three assertions that totality inverts were rewritten, not diagnosed** (§7's second table), and
  a **fourth was found during implementation and is not in the story**: the scratch prefix-selector
  test asserted `answers.len() == 1`, which a total map satisfies whether the trap is answered *or*
  bucketed — the exact behaviour that test exists to refuse. It now asserts the VARIANT.
- **`epics.md:1545` corrected** (8 → eleven, three classes, dated parenthetical). The one lifting of
  the verify-only rule, and nothing else in that file was touched.
- **Both inherited register entries closed**, with the `l1_answers` cross-file id test calling the
  runner **directly** — M7 confirms `score_corpus` would otherwise have kept it green.
- **Both twins updated** (`CLAUDE.md` and `docs/project-context.md`) — one updated and the other
  missed was the HIGH of 5.7's review.
- **What this story did NOT do**, unchanged and registered: no `l2-*` rule (Epic 6); no
  `ScoredRecord`/`VerdictVectorEntry`; no production caller for the blocker; no decision on whether
  a non-empty but PARTIAL map should block — measured and owned forward instead.
- ⚠️ **Issue #38 did not recur** during this implementation: ~14 full-suite runs across the
  mutations, no unexplained red. Recorded as an observation, not as evidence about the cause.

### File List

| file | change |
|---|---|
| `crates/opencmdb-core/src/score.rs` | UPDATED — `UnanswerableCause`, `Answer`, 3 tests |
| `crates/opencmdb-bin/src/trap_gate.rs` | UPDATED — `Unanswered`, the fourth bucket, `unanswered()`/`unanswered_in()`/`unaccounted()`, `passed()`, `Display`, `score_corpus`'s seam, module + `passed()` docs, 6 tests, 13 call sites wrapped |
| `crates/opencmdb-bin/src/l1_runner.rs` | UPDATED — `l1_answers` total + pair-first + cross-file id guard, module/predicate docs, 3 tests, 4 assertions strengthened |
| `_bmad-output/planning-artifacts/epics.md` | UPDATED — story 5.8's first criterion, 8 → eleven (AC8; the only edit) |
| `_bmad-output/implementation-artifacts/deferred-work.md` | UPDATED — appended `## Deferred from: story-5.8` |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | UPDATED — status + the story's record |
| `_bmad-output/implementation-artifacts/5-8-…-not-passing.md` | UPDATED — this file |
| `CLAUDE.md` · `docs/project-context.md` | UPDATED — both twins |

**Nothing under `fixtures/` changed** — verified with `git status fixtures/` (empty).

## Change Log

| Date | Change |
|---|---|
| 2026-08-02 | Story contexted from `epics.md:1537-1557` against `master` `60107fa` (352 tests, clean tree). The corpus was re-measured trap by trap: the premise at `epics.md:1545` is **8** and the measured residue is **11** in three classes, so AC8 makes this story the one that corrects the epic file — the correction 5.7 registered with this story as owner. Per-column arithmetic (10/11/3 = 7/6/0 scored + 3/5/3 unanswerable) was derived and is the story's strongest guard. The **eleven** live `passed()` assertions were enumerated (4 true, 7 false) and exactly **one** flips. |
| 2026-08-02 | **Validated by two fresh-context agents** (fact-check + gap-hunt), 20 findings applied. The gap-hunt agent IMPLEMENTED the story end to end in an isolated worktree: **352 → 364 tests**, six gates green, both clippy forms clean, `fixtures/` untouched, and **every numeric prediction reproduced exactly** — 24/13/11, causes 8/2/1, per-column 3/5/3, `passed() == false`. §7's table was verified byte-exact and only `trap_gate.rs:534` flipped, as predicted. What the compiler found that reading did not: **M5 never reaches the arithmetic loop** it was written to protect (replaced by M5b), **totality inverts three `l1_runner.rs` assertions silently** (a class §7 did not cover), and **three `l1_runner.rs` doc sites are falsified by §4's own pair-first decision**. Both agents independently found the totality class — the convergence is why it is now a table rather than a sentence. |
| 2026-08-02 | **IMPLEMENTED → `review`.** AC1–AC10 all MET. **352 → 364 tests** (162 bin + 156 core + 46 xtask), six `xtask ci` gates green, both clippy forms clean, `fixtures/` untouched. 🔴 **The committed gate is now RED on purpose**: `24 discovered, 13 scored, 0 failures, 0 wrong-rule, 11 unanswerable, passed = false`. §7's prediction verified **before any new test existed** — exactly one `passed()` assertion flipped, the one it named. Ten mutations, **every red assertion-carried, ZERO compiler-carried**; M3 probed directly gives `scored=24 must_abstain_scored=3 must_abstain_failures=0`, so all three `must-abstain` traps PASS under it and AC1's refusal is measured, not asserted. **Two of the story's own mutation prescriptions were corrected by running them**: M5 reds on the arithmetic LOOP (not on a literal, as the story inherited from a tree AC5's reordering replaced), which left the three literals proven by nothing — so **M5c was added**, a redistribution inside one column that keeps the loop green and reds the literal at `4 != 3`. A **fourth** silently-inverted assertion was found during implementation and is not in §7's table: the scratch prefix-selector test asserted `answers.len() == 1`, satisfied by a total map whether the trap is answered or bucketed. Divergences stated rather than smoothed: M3 reds 6 where the story predicted 7; the 364 split 162/156 where §11 predicted 163/155; all three files run slightly longer than the validation's tree. |
