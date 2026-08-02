# Story 5.7: The trap runner stops scoring nothing

Status: done

<!-- Validation is MANDATORY here (Guy's decision, Epic 4 retrospective 2026-07-26): two
     fresh-context agents (fact-check + gap-hunt) BEFORE dev-story. The template banner saying
     "Validation is optional" does not apply to this project. Measured on 5.5 AND 5.6: EVERY HIGH
     finding came from the agent that COMPILED the story, none from the agent that checked its
     claims. Point the gap-hunt agent at AC2, AC5 and AC9 — the selector, the wrong-rule
     demonstration, and the doc sweep.

     ✅ DONE 2026-08-01 — and the pattern held a THIRD time: 3 HIGH from the compiling agent, 0 from
     the fact-checker. 13 findings applied (3 HIGH, 5 MEDIUM, 5 LOW), 1 dismissed. The prediction
     above was right about AC2 and AC9 and wrong about AC5, which validated clean; the third HIGH
     landed on AC7, which this note did not name. See the Change Log at the end of this file. -->

## Story

As the release gate,
I want the L1 engine wired into the corpus harness,
so that the committed traps become **a gate that runs** instead of data that is merely discovered
and parsed.

**This story crosses the seam.** Since story 4.6b, `score_corpus` has walked the committed corpus,
read 24 traps, validated them — and scored **zero**, because its `answers` map has always been
empty. Story 5.5 built the producer; story 5.6 built the blocker; neither has a production caller.
This story fills the map, and after it `scored` is **13** where it has been **0** for nine stories.

**What this story does NOT do**, so the boundary is explicit and not discovered in review:

- it does **not** build a bucket for the traps L1 cannot answer — that is **story 5.8**, and §3
  below hands it a measured correction to its own premise;
- it does **not** persist anything — story 5.9;
- it does **not** produce a `ScoredRecord`, and §6 gives the measured obstacle and re-owns the
  `VerdictVectorEntry` unification that four doc sites currently pin on this story;
- it does **not** design an `l2-*` rule — Epic 6;
- it does **not** touch the blocker. `candidates` still has no production caller after this story,
  and that is stated rather than quietly fixed: the trap runner is handed a PAIR by the trap
  (`Trap::observations`), so it has nothing to generate. See §2.4.

**Nothing under `fixtures/` moves.** No artefact bytes, no `MANIFEST.toml`, no re-hash. This story
**reads** the corpus and never writes it. If any step appears to require re-authoring a committed
artefact, **STOP** — that is a finding, reported rather than absorbed.

**`architecture.md` is NOT edited** (issue #54 for D13's short table; a milestone act).
**`architecture-views.md` is NOT regenerated** (issue #50). **`epics.md` is verify-only — an edit
there is a finding**, including the correction §3 measures.

⚠️ **Branch from `master`.** Measured at contexting: `master` is at **`6cc137b`**, the working tree
is **clean**, and the workspace reports **333 tests** (139 bin + 148 core + 46 xtask) under
`cargo test --workspace --locked`.

---

## What this story inherits, measured rather than assumed

Everything below was measured at contexting on `6cc137b`, most of it by a temporary probe test
appended to `fixtures.rs`'s test module which ran the **real** engine (`decide_pair`) over **every
committed trap**. The probe was deleted afterwards and the file's md5 verified back to
`3616d8e489c931747f3722effda641f9`, `git status` clean. **The dev re-derives none of this; a
surprise reads as a FINDING.**

### 1. 🔴 The corpus answer table — the measurement that decides this story's shape

The L1 engine's answer for all 24 committed traps, produced by `decide_pair` on the two
observations each trap names, mapped onto `Outcome`, then run through `score` and `run_trap`:

| trap | column | expected rule | L1 answers | `run_trap` |
|---|---|---|---|---|
| `cloned-mac-must-merge` | must-merge | `l1-exact-mac` | `Merged { l1-exact-mac }` | ✅ Pass |
| `dhcp-churn-must-merge` | must-merge | `l1-exact-mac` | `Merged { l1-exact-mac }` | ✅ Pass |
| `example-must-merge` | must-merge | `l1-exact-mac` | `Merged { l1-exact-mac }` | ✅ Pass |
| `hostname-absence-must-merge` | must-merge | `l1-exact-mac` | `Merged { l1-exact-mac }` | ✅ Pass |
| `hostname-collision-must-merge` | must-merge | `l1-exact-mac` | `Merged { l1-exact-mac }` | ✅ Pass |
| `randomized-mac-must-merge` | must-merge | `l1-exact-mac` | `Merged { l1-exact-mac }` | ✅ Pass |
| `vrrp-virtual-mac-must-merge` | must-merge | `l1-exact-mac` | `Merged { l1-exact-mac }` | ✅ Pass |
| `dhcp-churn-must-not-merge` | must-not-merge | `l1-distinct-mac` | `Refused { l1-distinct-mac }` | ✅ Pass |
| `docker-veth-must-not-merge` | must-not-merge | `l1-distinct-mac` | `Refused { l1-distinct-mac }` | ✅ Pass |
| `example-must-not-merge` | must-not-merge | `l1-distinct-mac` | `Refused { l1-distinct-mac }` | ✅ Pass |
| `hostname-absence-must-not-merge` | must-not-merge | `l1-distinct-mac` | `Refused { l1-distinct-mac }` | ✅ Pass |
| `hostname-collision-must-not-merge` | must-not-merge | `l1-distinct-mac` | `Refused { l1-distinct-mac }` | ✅ Pass |
| `randomized-mac-must-not-merge` | must-not-merge | `l1-distinct-mac` | `Refused { l1-distinct-mac }` | ✅ Pass |
| `cloned-mac-must-not-merge` | must-not-merge | `l2-different-hostname` | `Merged { l1-exact-mac }` | 🔴 **VerdictFail** |
| `docker-veth-must-merge` | must-merge | `l2-uplink-agrees` | `Refused { l1-distinct-mac }` | 🔴 **VerdictFail** |
| `multi-nic-must-merge` | must-merge | `l2-uplink-agrees` | `Refused { l1-distinct-mac }` | 🔴 **VerdictFail** |
| `shared-hardware-vm-must-merge` | must-merge | `l2-hostname-agrees` | `Refused { l1-distinct-mac }` | 🔴 **VerdictFail** |
| `hostname-absence-must-abstain` | must-abstain | *(a cause)* | `Refused { l1-distinct-mac }` | 🔴 **VerdictFail** |
| `shared-hardware-vm-must-abstain` | must-abstain | *(a cause)* | `Refused { l1-distinct-mac }` | 🔴 **VerdictFail** |
| `multi-nic-must-not-merge` | must-not-merge | `l2-different-switch` | `Refused { l1-distinct-mac }` | 🟠 **WrongRule** |
| `shared-hardware-vm-must-not-merge` | must-not-merge | `l2-different-hostname` | `Refused { l1-distinct-mac }` | 🟠 **WrongRule** |
| `vrrp-virtual-mac-must-not-merge-bearers` | must-not-merge | `l2-different-hostname` | `Refused { l1-distinct-mac }` | 🟠 **WrongRule** |
| `vrrp-virtual-mac-must-not-merge-master` | must-not-merge | `l2-virtual-mac-prefix` | `Refused { l1-distinct-mac }` | 🟠 **WrongRule** |
| `example-must-abstain` | must-abstain | *(a cause)* | **no pair — one observation** | — |

**Counts, asserted rather than quoted: 24 traps · 13 name an `l1-*` rule · 8 name an `l2-*` rule ·
3 name a cause and no rule · 23 name two observations · 1 names one.**

⇒ **Answering all 24 makes the gate RED: 6 truth-table failures and 4 wrong-rule failures.**
⇒ **Answering the 13 whose expected rule is `l1-*` makes it GREEN: 13 scored, 0 failures, 0
wrong-rule.** Every one of the thirteen passes, including its rule.

That is the whole shape of the story. It is not a prediction. **Reproduced row for row during
validation** by a second, independent implementation, which also measured the report itself:
`discovered=24, scored=13, failures=0, rule_mismatches=[], incomplete_families=[], passed=true`, and
`scored_in` = 7 / 6 / 0. The `Display` render is
`24 trap(s) discovered, 13 scored, 0 truth-table failure(s)`.

### 2. The selector is the EXPECTED RULE's level — and a reviewer will challenge it, rightly

`epics.md:1525`: *"each trap whose expected rule is `l1-*` is answered by the real engine"*. Taken
literally that is a selector **on the expectation**, and the obvious objection is that it looks like
*"answer only the traps we already pass"* — scoring theatre, which this project refuses by name.

**Four things answer it, and all four belong in the code's doc, not only here:**

1. **It selects by LEVEL, never by outcome.** `expect.rule()` says which cascade level the trap's
   AUTHOR said answers the case — frozen in Epic 4, before any engine existed, precisely so the
   metric could not be bent to the engine (D19: *"a metric written after the engine is bent to fit
   the engine"*). The selector reads the level prefix and nothing else. It does not read the
   column, the outcome, the family or the reason.
2. **The unanswered traps do not leave the denominator.** `Report::discovered()` stays **24** while
   `scored()` is **13**, and `Tally::scored_in` reports the per-column split. Story 5.8 then turns
   the residue into a bucket that BLOCKS. The exclusion is visible in this story and blocking in
   the next; it is never silent in either.
3. **A prefix, not a whitelist of the two implemented ids.** `starts_with("l1-")`, so a trap
   expecting an `l1-*` rule this engine does NOT implement is answered anyway and reds as a
   `WrongRule`. A whitelist would let a future L1 rule slip out of the denominator in silence,
   which is exactly what story 5.8 exists to prevent. Measured: the corpus writes exactly two
   `l1-*` ids today, so the two selectors agree on the committed bytes and only the prefix keeps
   agreeing tomorrow. **AC7 requires a test that pins this**, on a scratch corpus.
4. **The counter-factual is recorded, not hidden.** §1's table is the answer to *"what if you
   answered everything"*, measured rather than argued, and AC5 makes the wrong-rule half of it a
   live test rather than a table row.

### 3. 🔴 The residue is **11**, not 8 — and that is a correction owed to story 5.8

`epics.md:1545` gives story 5.8 the premise *"the **8** traps whose expected rule is `l2-*`, which
the L1 engine cannot answer"*. Measured: **11 committed traps are unanswerable at L1**, in three
distinct classes, and only the first is the one 5.8's premise names.

| class | n | why L1 cannot answer it | what happens if it is answered anyway |
|---|---|---|---|
| expected rule is `l2-*` | 8 | the level is not implemented | 4 VerdictFail + 4 WrongRule |
| `must-abstain`, names a pair | 2 | the expectation names **no rule**, so there is no level to route on — and the question is a device-level one (shared uplink, absent hostname) | 2 VerdictFail |
| `must-abstain`, names ONE observation | 1 | there is no pair at all | see §4 — it would **pass**, for the wrong reason |

The three `must-abstain` traps are `hostname-absence-must-abstain`, `shared-hardware-vm-must-abstain`
and `example-must-abstain`. They are invisible to a `l2-*` selector because `Expectation::MustAbstain`
carries a **cause**, not a rule: `Expectation::rule()` returns `None` for all three
[`crates/opencmdb-core/src/trap.rs:107-113`].

**The residue, as the literal set AC8 asserts** — measured at contexting and again during validation,
written here so AC8 does not send the dev back to §1's table to transcribe it:

```
cloned-mac-must-not-merge            docker-veth-must-merge
example-must-abstain                 hostname-absence-must-abstain
multi-nic-must-merge                 multi-nic-must-not-merge
shared-hardware-vm-must-abstain      shared-hardware-vm-must-merge
shared-hardware-vm-must-not-merge    vrrp-virtual-mac-must-not-merge-bearers
vrrp-virtual-mac-must-not-merge-master
```

Eleven ids: the eight `l2-*` of the first class, plus the three `must-abstain`.

**This story does not fix `epics.md`** (verify-only — an edit there is a finding). It **registers**
the correction with story 5.8 named as owner, in `deferred-work.md`, and flags it FORWARD the same
way story 5.5 flagged the float forward to 5.6 — *"so it is a decision and not a surprise"*. **AC8.**

⚠️ **A consequence for AC4, and it is the kind of sentence this project's reviews keep catching.**
`epics.md:1527` calls randomized-mac, dhcp-churn, hostname-collision and hostname-absence *"the four
pure-L1 families"* and `:1529` asks that *"their traps pass in BOTH poles"*. Measured: **hostname-absence
holds THREE traps, and its third is a `must-abstain` the L1 engine gets wrong.** The sentence is
true under the reading *"both DECISION poles"* and **false** under the reading *"all their traps"*.
AC4 takes the first reading, states it, and asserts the third trap is unanswered — so the narrower
claim is written down instead of the wider one being assumed.

### 4. The single-observation trap **would pass**, and that is precisely why it must not be answered

`example-must-abstain` names ONE observation (`minimal.jsonl:2`), so there is no pair. The tempting
implementation is one line — `decide(vec![], CURRENT_RULESET_VERSION)` — and it returns
`Abstained { AbsenceOfProof }`, which maps to `Outcome::Abstained` and **PASSES** the `must-abstain`
column [`decide`'s last row, `identity/cascade.rs`].

**Refused.** The pass would come from calling the algebra with an empty verdict vector — the engine
evaluating *nothing* — not from L1 reasoning about the observation. A trap that passes because no
rule was asked is the "right answer for the wrong reason" D19 and D46b exist to catch, and it would
put a **1** in the `must-abstain` column of a gate that never asked the question. A trap the engine
cannot form a pair for gets **no answer**. **AC2 requires this to be a named condition with its own
assertion**, not an incidental consequence of an `if let [a, b]`.

### 5. The `Decision -> Outcome` mapping: total, named, and **not** a `From`

Four sites currently say this story owns it
[`score.rs:51`, `identity/cascade.rs:38-39`, `:296-299`, `deferred-work.md`]. The mapping is total
and information-preserving on the answer, and lossy on the envelope:

| `Conclusion` | `Outcome` | note |
|---|---|---|
| `Match { rule }` | `Merged { rule }` | |
| `NoMatch { rule }` | `Refused { rule }` | |
| `Abstained { cause }` | `Abstained { cause }` | **same type** — `IdentityAbstentionCause` since story 5.3 |

`verdict_vector` and `ruleset_version` are **dropped**, because `Outcome` has nowhere to put them —
that loss is §6's subject and must be documented at the function rather than left for a reader to
notice.

**Decided at contexting, so it is not re-litigated at review:**

- **a named free function `outcome_of(&Decision) -> Outcome`, in `crates/opencmdb-core/src/score.rs`**
  — beside `score` and `run_trap`, the two other functions the release gate is made of;
- **`impl From<Decision> for Outcome` is REFUSED.** `cascade.rs:296-299` states the reason and this
  story honours it rather than overturning it: *"mapping the engine's return onto the harness's
  record is a decision about the release gate… not a silent conversion"*. A `From` makes the
  conversion free at every call site — `.into()` — which is exactly the invisibility the refusal
  was about. The same refusal, for the same reason, keeps the two abstention vocabularies
  unbridged (story 5.3);
- **in `score.rs`, not in `identity/`** — the mapping is knowledge about the RELEASE GATE, and the
  engine must not acquire it. `score.rs` already names `identity::cascade::IdentityAbstentionCause`
  in a field type, so the dependency direction is unchanged;
- **in core, not in bin** — both types are domain types, and an exhaustive match that must break on
  a new `Conclusion` variant belongs where the variant lives (D47);
- **exhaustive, no `_` arm** — a fourth `Conclusion` variant must produce `error[E0004]` here and
  force a decision. This is the mechanism `score`'s own 3×3 and `keys_of`'s `Fact` match already use.

⚠️ `outcome_of` is in `score.rs`, which is **not** under `identity/`, so the `float-free` gate does
not walk it. See §8 for the trap that applies to the `identity/` doc edits.

### 6. The `VerdictVectorEntry` unification is **DECIDED here, not done** — with a measured obstacle

**Five** sites name story 5.7 for the unification, and — corrected during validation — only **two**
of them state the condition; the other three name the story without one:

| site | wording, verbatim |
|---|---|
| `score.rs:289-290` | *"Story 5.7 owns the unification, when the trap runner first records a run a real engine produced"* — names it **and** states the condition |
| `deferred-work.md` | *"Owner: story 5.7, when **the harness** first records a run a real engine produced"* — condition stated, but the subject is the harness, not the runner |
| `cascade.rs:186` | *"story 5.7 owns the unification."* — **no condition** |
| `cascade.rs:33` | *"uninhabited until story 5.7 unifies the two."* — no condition, and it does not say "owns" |
| `score.rs:467` | *"Story 5.7 crosses that seam."* — no condition, and it does not say "owns" |

The condition, where it is stated, is *"when the trap runner first records **a run** a real engine
produced"*. AC9 narrows **all five**, not only the two that carry a condition — a site that names
this story without a condition is the easier one to leave behind.

**That condition is not met by this story, and the reason is measurable rather than a matter of
appetite.** A "run" in `score.rs`'s vocabulary is a `Vec<ScoredRecord>`, and `ScoredRecord` carries
`capability_snapshot: Capabilities` — D36's whole point: *"a verdict without its capability snapshot
is UNFALSIFIABLE"*. Measured at contexting:

- **11 replay streams are referenced by a trap**, and **not one of them carries a `capability`
  control record.** Only `capability-downgrade.jsonl` and `partial-then-failed.jsonl` carry control
  records at all, and **no trap names either**;
- the only `Capabilities` value **a trap run can reach** today is `corpus_caps()`, a **hand-authored
  value inside `fixture_connector.rs`'s TEST module** [`:382-395`]. ⚠️ Narrowed during validation:
  it is **not** true that no production code constructs a `Capabilities` — `arp_ping.rs:183-187`
  builds one for its `PollSummary`, above that file's `#[cfg(test)]` at `:193`, and
  `capability-downgrade.jsonl:3` is a committed corpus byte carrying a real one. Neither is on the
  trap-run path, which is what makes the conclusion below hold; the wider sentence was false;
- `read_jsonl`, the reader this story uses, **discards control records by construction**
  [`fixtures.rs:647-657`].

⇒ Producing a `ScoredRecord` here would mean **inventing a capability snapshot for all 24 traps**,
which is D36's unfalsifiability in reverse and D45's *"a gate on a false truth"*.

**AC9 therefore requires a DECISION, in writing, and forbids inheriting the claim:** each of the
sites above is narrowed to the sentence that is true after this story, and the unification is
re-owned — **owner: the story that gives a trap run a real capability snapshot**, which is the
`FixtureConnector` read path, not `read_jsonl`. Recorded in `deferred-work.md` with that condition
spelled out. **Writing "story 5.7 owns it" a sixth time, in a story that did not do it, is the
defect five consecutive code reviews have caught.**

### 7. Rule-id canonicality — both sides measured canonical, and nothing checks it

Two open items converge here and both name this story as owner:

- `l1.rs:94-96` claims the two constants are *"spelled here exactly as `fixtures/scenario/traps/*.toml`
  spells them, because story 5.7 compares this producer's id against those bytes"*, and
  `deferred-work.md` records that **nothing in this crate checks it** — that is the register's own
  wording; *"nothing in the tree"* happens to be true as well (`L1_EXACT_MAC` and `CORPUS_EXACT_MAC`
  occur only in `l1.rs`), but it is not what is written down: the test-side redundancy
  (`CORPUS_EXACT_MAC` / `CORPUS_DISTINCT_MAC`) catches a rename of one constant but cannot catch
  both literals being wrong relative to the TOML;
- `deferred-work.md` §*code review of story-4.7a*: `run_trap` compares raw `RuleId` strings with **no
  normalization**, so a trailing space or a casing difference on either side would be a
  false-positive `WrongRule` — *"a red gate on a correct answer"*. Owner: *"when Epic 5 supplies a
  producer"*. **This story is that supply.**

Measured on the committed bytes: the corpus writes **seven** distinct rule ids —
`l1-exact-mac` (7 occurrences), `l1-distinct-mac` (6), `l2-different-hostname` (3),
`l2-uplink-agrees` (2), `l2-different-switch` (1), `l2-hostname-agrees` (1),
`l2-virtual-mac-prefix` (1) — and **all seven equal their own `trim()` and their own
`to_lowercase()`**. Story 5.5 already pins the producer's side
(`the_producers_rule_ids_are_canonical`). **AC7** closes the corpus side and the cross-comparison.

### 8. Baseline, gates and one trap that will cost an hour if it is not read

- **master `6cc137b`, clean tree, 333 tests** (139 bin + 148 core + 46 xtask).
- `trap_gate.rs` is **384 code lines** (first `#[cfg(test)]` at `:385`); `fixtures.rs` is **728**
  (first `#[cfg(test)]` at `:729`); ceiling **2000** (`file-size` gate, tests excluded).
- Six gates must stay green: `cargo xtask ci` — frontier (D47), DDL collation (D64), retired
  vocabulary (D65), corpus lock (both directions), `file-size`, `float-free`. The informational
  `views-hash` reports `ℹ STALE` and exits 0 **by design** — do not regenerate (issue #50).
- ⚠️ **The CI form of clippy is `cargo clippy --workspace -- -D warnings`, WITHOUT `--all-targets`.**
  Run **both** forms. An import kept alive only by a test module is an `unused_imports` error in the
  form CI runs and is invisible in the other — this has reddened CI twice in this epic.
- 🔴 **The `float-free` trap, measured in story 5.6 and still live.** Under
  `crates/opencmdb-core/src/identity/`, the gate strips `//`, `///` and `//!` but **not** block
  comments and **not** string literals on a code line. A bare story number without a letter suffix
  is a **float literal**: `assert!(true, "story 5.7 answers it")` **REDS**. The doc edits AC9
  requires under `identity/` are all in `//!` / `///` comments and are safe; **a "5.7" inside a
  string literal, a `/* … */` block or a `#[doc = "…"]` under `identity/` will red the gate.**
  `score.rs` is outside the guarded subtree and unaffected.
- ⚠️ `DATABASE_URL` is usually unset locally and the MariaDB-backed tests `return` early — a green
  suite says nothing about the database. Irrelevant to this story, stated so it is not re-derived.

---

## Decisions taken at contexting

Recorded so they are not re-litigated at review. Each carries the measurement or the quote behind
it, above.

1. **Selector = `expect.rule()` starts with `"l1-"`**, plus "the trap names exactly two
   observations". Prefix, not whitelist (§2.3). A trap failing the second condition is a **named
   condition with its own assertion**, not a fall-through (§4).
2. **`outcome_of(&Decision) -> Outcome`, a free function in `score.rs`.** No `From` impl in either
   direction, and the existing refusal is honoured rather than overturned (§5).
3. **The producer lives in a NEW module, `crates/opencmdb-bin/src/l1_runner.rs`** — not in
   `trap_gate.rs`. `trap_gate.rs`'s structural guarantee is *"the harness never calls a producer"*
   [`:8-19`]; putting the producer in the same file would weaken that from a FILE-level property to
   a per-function promise, on the very day it first has something to promise about. The seam
   between them stays the `BTreeMap<TrapId, Outcome>`, which is data.
4. **`score_corpus`'s signature and body are UNCHANGED.** It gains no engine parameter, no callback
   and no `answerer`. 4.6b's AC1 is not spent here.
5. **`read_jsonl`, not `FixtureConnector`,** to load a trap's stream — the same reader
   `read_traps`' own cross-check and story 5.6's `corpus_pairs()` use. The consequence is §6's
   obstacle and it is documented, not worked around.
6. **The blocker is not called.** A trap hands the runner a pair; there is nothing to generate.
   Story 5.6's registered entry (*"nothing calls the blocker and the engine in sequence… Owner:
   story 5.7"*) is answered with the **narrow true sentence and stays open**: this story is not the
   place, because a trap runner that generated its own pairs would ignore `Trap::observations` —
   the corpus's own statement of what is under judgement. Re-owned to the first caller that has a
   set of observations and no trap: story 5.9 or Epic 6.
7. **No `ScoredRecord`, no `compare_runs`, no `VerdictVectorEntry` replacement** (§6). AC6's
   reproducibility is asserted on `Report`, which is what `epics.md:1535` asks for.
8. **No `Decision::cause()`, no `Conclusion::rule()`.** `outcome_of` matches the conclusion
   directly; neither accessor gains a caller. The register entry naming 5.7 for `Conclusion::rule()`
   is answered *"no consumer holds a bare conclusion"* and closed.

---

## Acceptance Criteria

**AC1 — the mapping exists, is total, and is not a convenience.**
**Given** `identity::cascade::Decision` and `score::Outcome`, which nothing has ever converted
between — *"the day they meet is story 5.7's"* [`cascade.rs:39`]
**When** the mapping is written
**Then** `score::outcome_of(&Decision) -> Outcome` exists in `crates/opencmdb-core/src/score.rs`,
matches `Conclusion` **exhaustively with no `_` arm** (a fourth variant must give `error[E0004]`
here), maps the three rows of §5's table, and its doc states plainly that `verdict_vector` and
`ruleset_version` are dropped and why. **No `impl From<Decision> for Outcome` is added**, and
`cascade.rs`'s sentence refusing it is kept and updated to say the mapping now exists as a named
function.

**AC2 — the producer fills the map, and the harness still runs no producer.**
**Given** `score_corpus(traps_root, answers)`, whose `answers` map has been empty since story 4.6b
**When** the L1 runner is built
**Then** a new module `crates/opencmdb-bin/src/l1_runner.rs` exposes a function that walks a trap
corpus and returns `BTreeMap<TrapId, Outcome>`, containing an entry for **exactly** those traps
whose `expect.rule()` is `Some(r)` with `r.0.starts_with("l1-")` **and** which name exactly two
observations. It reads each replay stream through `fixtures::read_jsonl` (one read per distinct
stream, not one per trap), resolves the two `obs_id`s, calls `identity::l1::decide_pair` and maps
the result through `outcome_of`.
**And** `score_corpus`'s signature and body are unchanged — it takes no engine, no callback and no
closure; the map is the only seam.
**And** a trap that names fewer or more than two observations gets **no entry**, as a NAMED
condition carrying its own assertion (§4): the pass `decide(vec![], _)` would manufacture is
refused, and the refusal is documented with the measurement that it *would* have passed.
🔴 **The assertion must be `answer_trap(&example_must_abstain) == Ok(None)`, NOT a property of
`l1_answers`'s output.** Measured during validation: **of the 13 traps whose expected rule is
`l1-*`, ZERO name other than two observations** — the corpus's only pairless trap,
`example-must-abstain`, carries a cause and is excluded by the **level** selector before the pair
condition is ever consulted. So through `l1_answers` the pair guard is unreachable on the committed
corpus and any assertion on the walk's output is **vacuous**; through `answer_trap`, which is
level-blind by design (see *Two functions, not one*), it is the only thing standing between the
runner and an index-out-of-bounds panic. This is story 5.6's M2 idiom exactly: a guard the committed
corpus cannot exercise through the production path needs a test that reaches it directly.

**AC3 — the committed corpus is scored, and the numbers are asserted rather than quoted.**
**Given** the committed corpus, whose report has read `24 discovered, 0 scored` since story 4.6b
**When** the runner's answers are fed to `score_corpus`
**Then** a test asserts `discovered() == 24`, `scored() == 13`, `failures() == 0`,
`rule_mismatches().is_empty()`, `incomplete_families().is_empty()` and `passed() == true`.
**And** the rendered `Display` line contains `"13 scored"` — the sentence *"0 scored"* has been the
honest state for nine stories and stops being it here.

**AC4 — the per-column split is asserted, including the column that is EMPTY.**
**Given** `Tally::scored_in`, which exists so a reader can tell *"the column held"* from *"the
column was empty"* [`score.rs:406-415`]
**When** the corpus is scored by the engine
**Then** a test asserts `scored_in(MustMerge) == 7`, `scored_in(MustNotMerge) == 6`,
`scored_in(MustAbstain) == 0` and `failures_in(..) == 0` for all three.
**And** the **zero** is named in the test's own message as the vacuity `scored_in` was built to
make visible — after this story the `must-abstain` column is measured by nothing, and story 5.14
and Epic 6 are where it stops being.
**And** the four families `epics.md:1527` calls pure-L1 are asserted to pass in **both decision
poles** — with the narrower claim written down (§3): `hostname-absence` holds a **third** trap, a
`must-abstain`, and the test asserts it is **absent from the answers map**.

**AC5 — a right verdict by the WRONG rule fails separately, and it is demonstrated on the real corpus.**
**Given** story 4.7a's separation — a wrong rule is `Report::rule_mismatches`, never
`Report::failures`
**When** the engine answers a trap whose expected rule is `l2-*`
**Then** a test builds a map holding ONLY those four `must-not-merge` traps
(`multi-nic-must-not-merge`, `shared-hardware-vm-must-not-merge`,
`vrrp-virtual-mac-must-not-merge-bearers`, `vrrp-virtual-mac-must-not-merge-master`), each answered
through `answer_trap` (§ *Two functions, not one* below), asserts `failures() == 0` and
`rule_mismatches().len() == 4`, each entry naming the expected `l2-*` id and the actual
`l1-distinct-mac`, and asserts `passed() == false`.
**And** the test's doc states that this is §1's measured counter-factual made live: the separation
is exercised by the **real** engine against the **committed** corpus, not by a hand-authored
`Outcome`.

**AC6 — replaying the corpus twice yields an identical `Report` (D36).**
**Given** D36's reproducibility requirement
**When** the runner and the harness are both run twice over the committed corpus
**Then** the two `Report`s compare equal AND render to the same string, and the test asserts
`scored() == 13` on the first so the equality compares two real runs and not two vacuities — the
shape `replaying_the_same_corpus_twice_yields_identical_verdicts` already established.

**AC7 — the producer's rule ids are compared against the corpus BYTES.**
**Given** `l1.rs:94-96`'s claim that the two constants are spelled exactly as the corpus spells
them, which nothing in the tree checks, and story 4.7a's registered normalization gap
**When** the comparison is written
**Then** a test in `opencmdb-bin` (D47: the domain crate may not read files) walks every committed
trap file and asserts: every rule id beginning `l1-` is **either** `L1_EXACT_MAC` **or**
`L1_DISTINCT_MAC`; **both** constants occur (so the assertion cannot pass by finding none); and
every rule id the corpus writes — `l2-*` included — equals its own `trim()` and its own
`to_lowercase()`, so `run_trap`'s unnormalized comparison is trustworthy on the committed bytes.
🔴 **The walk must be `trap_gate::discover_trap_files(root)`, NOT `fixtures::walk_trap_files`.**
Measured during validation: `walk_trap_files(visit: &mut dyn FnMut(&Path)) -> usize`
[`fixtures.rs:835`] takes **no root** — it hardcodes `fixture_path("scenario/traps")` — so a test
built on it **cannot be reddened by M5** (a rename in a scratch copy is invisible to it; measured:
the scratch copy was renamed and the walk still reported 21 canonical ids). `discover_trap_files` is
root-parameterised (`(root: &Path) -> Result<Vec<PathBuf>, FixtureError>`) and this story promotes
it to `pub(crate)` anyway, so the byte test takes a root and M5 becomes runnable.
**And** a scratch-corpus test pins §2.3: a trap expecting an **unimplemented** `l1-*` rule IS
answered and lands in `rule_mismatches`, so the prefix selector is proven not to be a whitelist.
🔴 **That scratch trap must be `must-not-merge`, not `must-merge`** — measured during validation,
where the naive `must-merge` form landed in `failures()` and left `rule_mismatches()` **empty**:

```toml
# scratch only — never under fixtures/
expect = { must-not-merge = { rule = "l1-not-yet-implemented" } }
observations = [ "aaaaaaaa-0000-4000-8000-000000000001",
                 "aaaaaaaa-0000-4000-8000-000000000003" ]   # scenario/replay/minimal.jsonl
```

The reason is a corpus fact worth stating once: **`minimal.jsonl` contains no pair the L1 engine
merges.** Its three observations carry MAC `…:53:01`, *no MAC*, and MAC `…:53:02` — no two share
one. Since `run_trap` raises `WrongRule` **only on a verdict PASS**, a `must-merge` scratch trap over
that stream is a truth-table failure and can never reach `rule_mismatches` at all. Measured on the
corrected form: `scored=1, failures=0, rule_mismatches=[{ expected: "l1-not-yet-implemented",
actual: "l1-distinct-mac" }], passed=false`.

**AC8 — the residue is asserted, enumerated, and flagged FORWARD to story 5.8.**
**Given** that 11 committed traps are unanswerable at L1 while `epics.md:1545` gives story 5.8 the
premise that there are 8
**When** the runner runs
**Then** a test asserts the answered set has 13 members and that the 11 unanswered trap ids are
**exactly** the eleven literals listed in §3 (the list is written out there; do not transcribe it
off §1's table) — a residue that can grow in silence is how a gate quietly stops testing (story
5.6's idiom).
**And** `deferred-work.md` gains a `## Deferred from: story-5.7` section recording the 8→11
correction with **story 5.8 named as owner**, the three classes named, and the note that
`epics.md` is NOT edited by this story.

**AC9 — every doc site claiming this story is honoured or narrowed, and none is inherited.**
**Given** the **19** sites listed in Dev Notes — **14 matched by the grep, across 5 files**, plus
**5** the grep does not match (4 in `trap_gate.rs`, 1 in `score.rs`). *(Corrected during validation:
the story said "14 sites across 6 files", which is neither figure — the grep spans 5 files, and the
sixth file's sites are among the 5 unmatched. A dev could not tell which scope AC9 governs. It
governs all 19.)*
**When** the story lands
**Then** each is either made TRUE by this story's code or rewritten to the **weaker true sentence**
with a **new owner** — specifically: the `VerdictVectorEntry` unification is re-owned with §6's
measured obstacle recorded in `deferred-work.md`; `trap_gate.rs`'s module doc claims *"The map is
empty today"* and *"no answer producer exists"* are corrected; the test named
`the_committed_corpus_is_discovered_and_scored_by_nothing` is renamed so its name states what its
CALL does (it supplies no answers) rather than a claim about the corpus that this story falsifies.
**And** no site is left saying "story 5.7 owns it" — a promise re-made by the story that was
supposed to keep it is the defect five consecutive reviews have caught.

**AC10 — every new guard is proven to red before it passes, and nothing under `fixtures/` moves.**
**Given** the house rule (story 1.3)
**When** the work is done
**Then** at least the following mutations are run and **recorded with their observed red set**,
each on a committed baseline (`git stash`/commit first — `git checkout <file>` restores to HEAD and
has destroyed an implementation mid-run before):
- **M1** — `outcome_of` maps `Conclusion::NoMatch` to `Outcome::Merged`;
- **M2** — the selector drops the `l1-` prefix test and answers every trap with a rule (predict:
  4 VerdictFail + 4 WrongRule, per §1);
- **M3** — the selector answers `must-abstain` traps too, via `decide(vec![], _)` for the
  pairless one (predict: 2 VerdictFail, and `example-must-abstain` PASSES — §4's measured trap);
- **M4** — the two-observation condition is dropped **from `answer_trap`** (measured during
  validation: an index-out-of-bounds panic on `example-must-abstain`). ⚠️ Applying M4 to
  `l1_answers` instead reds **nothing at all** — the level selector excludes the corpus's only
  pairless trap first, so 13 of 13 `l1-*` traps name exactly two observations and the output is
  unchanged. **Record that invisibility as the measurement**, in story 5.6's M2 idiom: it is why
  AC2's assertion is on `answer_trap` and not on the walk;
- **M5** — one `l1-` rule id is renamed in a **scratch** copy of the trap corpus, never in
  `fixtures/`, and AC7's byte test — walking that scratch root through `discover_trap_files` — must
  red. ⚠️ Built over `walk_trap_files` the mutation is **invisible** (measured: 21 ids, all still
  canonical), because that walk takes no root. If AC7's walk is not root-parameterised, M5 is not a
  mutation, it is a no-op;
- **M6** — `AC3`'s `scored() == 13` is confronted with a selector that answers nothing, to prove
  the assertion is not vacuous.
**And** `git status fixtures/` is empty and `MANIFEST.toml` is untouched at the end — verified, not
assumed.
**And** the full local gate is run before pushing: `cargo fmt --all`, **both** clippy forms,
`cargo test --workspace --locked`, `cargo xtask ci`.

---

## Tasks / Subtasks

- [x] **Task 1 — the mapping (AC1)**
  - [x] Add `pub fn outcome_of(decision: &Decision) -> Outcome` to
        `crates/opencmdb-core/src/score.rs`, exhaustive match on `&decision.conclusion`, no `_` arm.
  - [x] Doc: the three rows, the two dropped fields and why, the refusal of `From` and who it
        belongs to, and a pointer to `Decision::rule()`'s mirror.
  - [x] Tests in `score.rs`: one per row, plus `outcome_of(&d).rule() == d.rule()` for every row —
        the mirror `run_trap` depends on.
  - [x] Update `cascade.rs:296-299` and `:38-39` and `score.rs:46-51`: the mapping now EXISTS and is
        a named function; the `From` refusal stands, with its reason.
- [x] **Task 2 — the runner (AC2, AC4's absent trap, AC7's prefix pin)**
  - [x] New file `crates/opencmdb-bin/src/l1_runner.rs`, `#![allow(dead_code)]`, module doc stating
        why it is NOT in `trap_gate.rs` (decision 3) and what the seam is.
  - [x] `mod l1_runner;` in `main.rs`, alphabetically between `fixtures` and `metrics`.
        ⚠️ `#![deny(missing_docs)]` is ON for this crate — every `pub` item needs a `///`.
  - [x] Make `trap_gate::discover_trap_files` `pub(crate)` and reuse it — **no third walk**.
        Cache one `read_jsonl` per distinct `replay`.
  - [x] Both items: `answer_trap` and `l1_answers` (see *Two functions, not one*).
  - [x] The selector, with the two conditions as separate named predicates so a mutation can hit
        each one alone.
  - [x] The pairless case: an explicit arm in **`answer_trap`** with the §4 argument in its comment,
        and its assertion on `answer_trap`, not on `l1_answers`' output (AC2 — the walk cannot reach
        it).
  - [x] `answer_pair(stream, trap, a, b) -> Outcome` as the shared core; `contains_key` + `insert`
        for the stream cache (`or_insert_with` + `?` does not compile).
- [x] **Task 3 — the corpus is scored (AC3, AC4, AC6)**
  - [x] Tests in `l1_runner.rs`'s own test module (subject = the runner) for the answers map and the
        residue; tests in `trap_gate.rs`'s test module (subject = the report) for the counts, the
        per-column tally and the reproducibility. Follow `cascade.rs`'s stated convention: *a test
        lives with the item whose CLAIM it pins.*
  - [x] Assert the `Display` line contains `"13 scored"`.
- [x] **Task 4 — wrong rule, on the real corpus (AC5)**
  - [x] The widened-selector test, with its four expected/actual pairs asserted by name.
- [x] **Task 5 — the corpus byte comparison (AC7)**
  - [x] The rule-id test over **`trap_gate::discover_trap_files(root)`** — root-parameterised, so M5
        can red it. **Not `walk_trap_files`**, which takes no root (AC7). Assert both constants occur
        and all seven ids are canonical.
  - [x] The scratch-corpus test for an unimplemented `l1-*` rule — **`must-not-merge`**, over
        `minimal.jsonl`'s obs `…0001` / `…0003` (AC7).
- [x] **Task 6 — the residue and the register (AC8, AC9)**
  - [x] The 11-id literal assertion.
  - [x] `deferred-work.md`: a new `## Deferred from: story-5.7` section, **appended, never
        rewriting an existing bullet**. It records: the 8→11 correction (owner **5.8**); the
        `VerdictVectorEntry` re-ownership with §6's measurement; the blocker's still-absent
        production caller with decision 6's narrow sentence; and closes by append-and-strike the
        entries this story genuinely closes (the `Decision -> Outcome` mapping half; the rule-id
        corpus comparison; `Conclusion::rule()`).
  - [x] The **19** doc sites (Dev Notes list: 14 grep-matched across 5 files + 5 unmatched) — each
        honoured or narrowed; the test rename.
- [x] **Task 7 — mutations and the gate (AC10)**
  - [x] Commit first, then M1–M6, each red set recorded verbatim in the Debug Log.
  - [x] `git status fixtures/` empty; `MANIFEST.toml` untouched; full local gate; then branch → PR →
        green CI → squash merge. **`done` is the MERGE's business**, not the review's.

### Review Findings (code review, 2026-08-02 — three layers)

Blind Hunter (diff only) · Edge Case Hunter (diff + tree) · Acceptance Auditor (diff + spec + context
docs). 20 unique findings after dedup: 1 decision, 10 patches, 2 deferred, 7 dismissed. **Every
finding below was re-measured before being written down** — two were refuted by that re-measurement
and are recorded in the dismissed list with the measurement that killed them.

- [x] [Review][Decision] **The self-pair is re-opened on the `answer_trap` path** — `named_pair`
      never compares `a` and `b`, so a hand-built `Trap { observations: vec![x, x] }` gives
      `decide_pair(o, o)`: `keys_of(o)` intersects itself, `shares_a_key` is trivially true, and the
      trap PASSES as `Merged { l1-exact-mac }` — a pass no rule reasoned about, the same *right
      answer for the wrong reason* this story's pairless arm refuses in prose. Story 5.6 closed the
      self-pair **in the type** (`CandidatePair::new(a, a) -> None`); on this path it is held only by
      `Trap::validate`'s `DuplicateObservation`, which does not run through `answer_trap` (that
      function calls `read_jsonl`, never `read_traps`). Unreachable from the committed corpus.
      **Choice: guard it in `named_pair` (`a == b` → `None`, with a test), or state the precondition
      in `answer_trap`'s doc and leave the behaviour.** [`crates/opencmdb-bin/src/l1_runner.rs:112`]

- [x] [Review][Patch] `CLAUDE.md` still asserts *"Nothing feeds the corpus harness (5.7)"* in the
      present tense, falsified ~400 words later in the same paragraph by *"The seam is crossed and
      the trap corpus is a gate that RUNS"*. `docs/project-context.md` corrected the identical
      sentence in this commit (`-Nothing feeds the corpus harness (5.7)…` → `+**The corpus harness
      is now fed**…`); `CLAUDE.md` was left. Measured: `grep -c` → `CLAUDE.md:1`,
      `project-context.md:0`. Violates docs-current-before-push. [`CLAUDE.md:7`]
- [x] [Review][Patch] The Debug Log records a measurement that measurement contradicts:
      *"`trap_gate.rs` **311**"* code lines. Measured on the shipped tree: first `#[cfg(test)]` at
      `:406`, so **405**. 311 is not merely wrong, it is *below* the story's own `6cc137b` baseline
      of 384, which the commit's +21 non-test lines make impossible. Harmless for the `file-size`
      gate; it is a fabricated number in the record.
      [`_bmad-output/implementation-artifacts/5-7-trap-runner-stops-scoring-nothing.md:773`]
- [x] [Review][Patch] `the_committed_corpus_is_scored_by_the_l1_engine`'s doc claims *"every one of
      the thirteen passes: the truth table, the rule, and **the family completeness**"*. Measured:
      `incomplete_families` is computed over `all_traps` [`trap_gate.rs:301`, `trap.rs:384-408`], not
      over the scored ones — it is a corpus-shape property, independent of `answers`, and the
      assertion's own message already says so (*"the corpus shape is unchanged by this story"*). It
      must be: the runner answers 2 of `hostname-absence`'s 3 traps and 1 of `vrrp-virtual-mac`'s 3,
      so a completeness check over *scored* traps would be non-empty. Narrow the sentence.
      [`crates/opencmdb-bin/src/trap_gate.rs:496`]
- [x] [Review][Patch] *"all **seven** ids the corpus writes"* is quoted, never asserted.
      `assert_rule_ids_are_canonical`'s only cardinality guard is `checked > 0`, so a truncated walk
      or an eighth rule id is caught by nothing in that test — it would pass over a corpus reduced to
      `dhcp-churn.toml` alone. Measured on the committed bytes: **21** rule-naming traps, **7**
      distinct ids. ⚠️ The count must go in the COMMITTED-corpus test, **not** in the shared
      root-parameterised helper: put it in the helper and M5c (a scratch corpus of `multi-nic.toml`
      alone) reds on the count instead of on the both-occur guard, masking the very guard the story
      measured M5c to prove load-bearing. [`crates/opencmdb-bin/src/l1_runner.rs:460-522, 532`]
- [x] [Review][Patch] **No test covers a trap naming three or more observations**, and the gap is
      measured invisible: rewriting `named_pair`'s `[a, b]` arm to `[a, b, ..]` — which would answer
      such a trap on its first two ids and ignore the third — leaves all 350 tests green. Every
      committed trap names ≤ 2 and every scratch TOML in the tree names 2, and `Trap::validate`
      rejects empty and duplicate observation lists but **not** a third id [`trap.rs:311-317`]. An
      `l1-*` trap with 3 observations would leave the denominator in silence — the exact harm the
      module doc argues a whitelist would cause, through a different door. This is the upper half of
      the guard whose lower half AC2 closed. Add the `answer_trap` test.
      [`crates/opencmdb-bin/src/l1_runner.rs:112-117`]
- [x] [Review][Patch] `expected_answered()`'s doc claims to be *"the third independent statement of
      the corpus's L1 surface, beside `l1.rs`'s two constants and its test-side restatement"*. It is
      a list of thirteen **trap ids**; those are two **rule-id** spellings — no DRY pass could
      collapse one into the other, so the anti-collapse warning protects nothing. `l1.rs:326-329`
      points its *"THIRD, independent statement"* at `the_producers_rule_ids_are_the_corpus_spelling`,
      which makes the same claim correctly 260 lines later in this same file. Two claimants, one
      title. The literal list IS right, for its own simpler reason (already stated in the sentence
      above it: an expectation computed by the predicate under test proves nothing) — keep that and
      drop the borrowed lineage. [`crates/opencmdb-bin/src/l1_runner.rs:269-275`]
- [x] [Review][Patch] `answer_trap` is `pub`, documents only `# Errors`, and can **panic**: it calls
      `read_jsonl` directly and never `read_traps`, so the `DanglingObservation` cross-check that
      `resolve`'s `# Panics` note leans on does not run on this path. `Trap` is fully public with
      public fields and no `#[non_exhaustive]` [`trap.rs:118-142`], so any caller can build one. The
      `# Panics` note lives on a private helper while the `pub` entry point states no precondition —
      against the house rule *"`# Panics` where relevant"*.
      [`crates/opencmdb-bin/src/l1_runner.rs:176-182`]
- [x] [Review][Patch] The scratch test removes the FILE and never the DIRECTORY, and only on the
      success path, where every sibling scratch test in `trap_gate.rs` uses `remove_dir_all`
      [`:747, :759, :781, :811, :871`]. Measured: three leaked directories in `/tmp`
      (`opencmdb-l1-runner-{104584,112245,150444}-unimplemented-l1-rule`), one per prior run.
      Hygiene, not a flake — but `scratch_dir`'s own doc sells the per-test directory as what keeps
      runs from racing, and cleanup that only runs on green does not support that claim.
      [`crates/opencmdb-bin/src/l1_runner.rs:604`]
- [x] [Review][Patch] The module doc's counter-factual — *"Answering **all 24** traps makes the gate
      red: 6 truth-table failures and 4 wrong-rule failures"* — describes a state this code forbids:
      `answer_trap` returns `None` for `example-must-abstain`, so at most 23 are answerable without
      the shortcut the same doc refuses. The **arithmetic is correct** (verified: M2's 4 + 4 over the
      eight `l2-*`, plus M3's 2 over the two paired `must-abstain`, with `example-must-abstain`
      passing = 6 + 4), and it is pinned by no live test — its oracle was a contexting probe since
      deleted. Narrow the wording to the traps the runner can answer plus the refused shortcut.
      [`crates/opencmdb-bin/src/l1_runner.rs:44-45`]
- [x] [Review][Patch] `the_verdict_vector_and_the_ruleset_version_are_dropped` cannot red under any
      realistic mutation of `outcome_of`: `Outcome` has no field able to carry either dropped value,
      so every implementation that does not branch on `ruleset_version` satisfies it, and it appears
      in none of the six recorded red sets. It is a documentation test and worth keeping — but the
      story made exactly this honesty note for the rule-mirror test (*"does NOT red under M1 …
      recorded rather than smoothed over"*) and did not make it here, while counting it among the
      five tests pinning AC1. [`crates/opencmdb-core/src/score.rs`, the AC1 test block]

- [x] [Review][Defer] `l1_answers` silently overwrites on a duplicate `TrapId` across trap files
      (`answers.insert`, no cross-file uniqueness check). Composed with the harness nothing ships
      wrong — `score_corpus` raises `DuplicateTrapId` [`trap_gate.rs:259-265`] before scoring — but
      `l1_answers` is `pub` and its doc calls the map *"exactly the `answers` map `score_corpus`
      takes"*; a caller reading `answers.len()` alone gets a silently short count, and story 5.8's
      residue arithmetic is the obvious such caller. Owner: **story 5.8**.
      [`crates/opencmdb-bin/src/l1_runner.rs:223`]
- [x] [Review][Defer] `outcome_of`'s abstaining row has **no end-to-end path through the runner**:
      all 13 answered traps carry a MAC on both sides, so `verdict_for_pair`'s `Neutral` branch
      [`l1.rs:257-263`] is never taken through `l1_answers`/`answer_trap`. The row is proved by
      `score.rs`'s unit tests and by a test that calls `decide(vec![], _)` directly — nothing in the
      runner's own tests would notice if `answer_pair` mishandled a MAC-less observation. This is the
      `scored_in(MustAbstain) == 0` vacuity, seen from the mapping's side. Owner: **story 5.14 /
      Epic 6**, when the `must-abstain` column stops being empty.
      [`crates/opencmdb-core/src/score.rs`, `Conclusion::Abstained` arm]

**Dismissed, each with the measurement that killed it** (7):

1. *"`!report.passed()` in the scratch test may be green for family incompleteness, not for the rule
   mismatch"* — **refuted**: the scratch trap declares no `family` key, `family` is
   `Option<FamilyId>` with `#[serde(default)]`, and `incomplete_families` skips `None`
   [`trap.rs:389`]. `passed()` is false because of the mismatch, exactly as the message says.
2. *"nothing pins that the answer is independent of the order a trap lists its pair"* — **refuted**:
   `decide_pair(a, b) == decide_pair(b, a)` including evidence, pinned by
   `a_pair_decides_the_same_whichever_side_is_the_left_argument` [`l1.rs:277-278`].
3. *"`resolve`'s `.find()` picks the first of two duplicate `obs_id`s, so the answer depends on file
   order"* — **refuted**: `read_records` refuses a duplicate `obs_id` in a stream
   [`fixtures.rs:585-594`].
4. *"the two `len` assertions after a set equality cannot fail"* — they pin the CARDINALITY of the
   literal expectation sets: if the corpus gained a fourteenth `l1-*` trap and `expected_answered()`
   were updated to match, the equality would pass and `len == 13` would red. Not vacuous.
5. *"the commit message's M2 (`21 scored, 4 truth-table`) contradicts the doc's 6 truth-table
   failures"* — M2 keeps the three `must-abstain` traps out because they name no rule at all, so
   21 = 13 + 8 and 4 + 4 is consistent; the story states this at `:807-809`. Only the *"all 24"*
   wording survives, as a patch above.
6. *"`l1_runner` is `#![allow(dead_code)]`, so calling it a production caller while denying the
   blocker one is misleading"* — this project's established meaning of *production caller* is a
   caller outside `#[cfg(test)]`, which `l1_runner` is and the blocker's callers are not; and
   `l1_runner.rs:76` states plainly that it is *"wired into no runtime path"*. Both halves are on
   the page.
7. *"nine `Owner: story 5.7` sentences survive in `deferred-work.md`"* — each carries an adjacent
   ✅ CLOSED / ↺ / ⚠️ RE-OWNED annotation, which is precisely what Task 6 prescribed (*"appended,
   never rewriting an existing bullet"*).

#### Prove-to-red on the review's three new guards (commit `6f958dd`, baseline committed first)

Every red assertion-carried; the tree was restored and re-verified green after each.

| mutation | what it changes | observed |
|---|---|---|
| **MR1** | `named_pair`'s arm back to `[a, b]` — the self-pair guard dropped | RED, **1** test: `a_trap_that_names_the_same_observation_twice_gets_no_answer`, *"two ids, one observation"* |
| **MR2** | `[a, b, ..] if a != b` — three observations accepted, third ignored | RED, **1** test: `a_trap_that_names_three_observations_gets_no_answer`; the answer it would have given is `Some(Abstained { AbsenceOfProof })` |
| **MR3a** | the canonicality walk truncated to **one** file | RED — but on the PRE-EXISTING both-occur guard (*"no trap … names `l1-distinct-mac`"*), which **masks** the new counts. `cloned-mac.toml` sorts first and writes no `l1-distinct-mac`. |
| **MR3b** | truncated to **two** files — `dhcp-churn.toml` writes both `l1-*` ids, so both-occur stays green | RED on the new assertion alone: `checked` **4**, expected **21** |

⚠️ **MR3a is recorded rather than dropped**: it is the same masking the story met on M5b/M5c, met
again on the first try, and it is why MR3b had to choose a prefix that keeps the older guard
satisfied. A mutation that reds the wrong assertion proves nothing about the new one.

**352 tests** (153 bin + 153 core + 46 xtask) — 350 → 352. Six gates green, both clippy forms clean,
`cargo fmt --all --check` clean, `git status fixtures/` and `git diff HEAD -- fixtures/` both empty.

**Also verified clean, by measurement rather than by reading the claim:** AC1–AC10 all MET; 24 traps
/ 13 `l1-*` / 8 `l2-*` / 3 causes / 23 pairs / 1 single; the residue of 11 literal ids matches the
corpus exactly; 7 distinct rule ids, all `trim()`/`to_lowercase()` fixed points; 11 trap-named replay
streams, zero `capability` control records among them; **350 tests** (151 + 153 + 46); `grep "5.7
owns"` over `crates`/`xtask` returns **nothing**, and all 19 doc sites were in fact updated;
`score_corpus`'s signature and body provably unchanged (the only non-doc, non-test change to
`trap_gate.rs` is `fn` → `pub(crate) fn`); `git diff 6cc137b b555712 -- fixtures/` empty; File List
matches the diff exactly. M6's red set of 8 and M1's of 7 check out arm by arm.

---

## Dev Notes

### The 19 doc sites that name story 5.7 (AC9) — 14 grep-matched across **5** files, plus 5 unmatched

Measured with `grep -rn "5\.7" --include=*.rs crates xtask` on `6cc137b`: **14 hits, across 5
files** — `core/src/lib.rs` (1), `core/src/identity/mod.rs` (1), `core/src/score.rs` (4),
`core/src/identity/l1.rs` (3), `core/src/identity/cascade.rs` (5). `trap_gate.rs` contributes **no**
grep hit; its four sites are in the unmatched list below.

| file | line | what it claims |
|---|---|---|
| `core/src/lib.rs` | 50 | `join`'s intended consumer *"has not crossed the crate frontier"* |
| `core/src/identity/mod.rs` | 21 | the blocker's *"INTENDED consumer is the trap runner (story 5.7), which does not reach the engine today"* |
| `core/src/score.rs` | 51 | *"mapping one onto the other… story 5.7 owns it"* → AC1 |
| `core/src/score.rs` | 289 | *"Story 5.7 owns the unification"* → §6, re-owned |
| `core/src/score.rs` | 297 | *"which is what story 5.7 changes"* → §6 |
| `core/src/score.rs` | 467 | *"Story 5.7 crosses that seam"* → §6 |
| `core/src/identity/l1.rs` | 95 | the constants are the corpus's spelling → AC7 |
| `core/src/identity/l1.rs` | 217 | *"Story 5.7 compares this id against the corpus bytes"* → AC7 |
| `core/src/identity/l1.rs` | 321 | *"story 5.7's corpus comparison would fail"* → AC7 |
| `core/src/identity/cascade.rs` | 33 | *"uninhabited until story 5.7 unifies the two"* → §6 |
| `core/src/identity/cascade.rs` | 39 | *"the day they meet is story 5.7's"* → AC1 |
| `core/src/identity/cascade.rs` | 186 | *"story 5.7 owns the unification"* → §6 |
| `core/src/identity/cascade.rs` | 295 | the blocker's intended consumer |
| `core/src/identity/cascade.rs` | 298 | no `From` impl, *"belongs to story 5.7"* → AC1 |

Plus **five** sites not matched by that grep, falsified or narrowed all the same — 14 + 5 = **19**,
which is the set AC9 governs:

- `trap_gate.rs:15-20` — *"The map is empty today; the vacuous run below is what that emptiness
  looks like"*;
- `trap_gate.rs:8-19` — *"It scores answers; it never runs a producer"*. **Still true of
  `score_corpus` and now load-bearing** — narrow it to the function, and name `l1_runner` as the
  producer it does not call (decision 3);
- `trap_gate.rs:104-108` — `Report::scored`'s *"Zero with a non-zero `discovered` is the honest
  state before any engine exists"*;
- `trap_gate.rs:418-428` — the test comment *"no answer producer exists, so nothing is scored"*, in
  `the_committed_corpus_is_discovered_and_scored_by_nothing` → renamed by AC9;
- `score.rs:346-350` — `ScoredRecord`'s two *"provably so"* fields. **Unchanged in substance** —
  no `ScoredRecord` is produced here (§6) — but re-read before claiming so.

### Two functions, not one — forced by AC5, and decided here

`l1_runner` exposes **two** items, and the split is not stylistic:

```rust
/// Answer ONE trap, whatever level its expectation names. `None` when the trap does not name
/// exactly two observations (§4).
pub fn answer_trap(trap: &Trap) -> Result<Option<Outcome>, FixtureError>

/// Walk a trap corpus and answer only the traps whose expected rule is `l1-*`.
pub fn l1_answers(traps_root: &Path) -> Result<BTreeMap<TrapId, Outcome>, FixtureError>
```

With the level selector buried inside a single function, **AC5 is not testable**: there would be no
way to ask the engine about an `l2-*` trap without duplicating the runner in the test, and a test
that reimplements its subject proves nothing. Splitting also puts the selector in exactly one named
predicate, which is what makes mutation **M2** hit one thing.

Validated: **both signatures compile verbatim**, `answer_trap` needs **no** root parameter, and AC5's
four-trap test is constructible through `answer_trap` from `l1_runner.rs`'s test module.

⚠️ `answer_trap` takes a `&Trap` and reads its stream itself, so it does its own `read_jsonl` —
acceptable for the four-trap test, wasteful for the walk. `l1_answers` must **cache one read per
distinct `replay`** and therefore cannot simply loop over `answer_trap`. Three corrections from
validation, each measured under `rustc`:

- **The core to factor is NOT `(&Observation, &Observation) -> Outcome`.** That signature is
  literally `outcome_of(&decide_pair(a, b))` — a one-liner, and factoring it shares nothing. What is
  actually duplicated is the `ObsId -> &Observation` resolution **plus the panic that must name the
  trap**. Prescribe:
  `fn answer_pair(stream: &[Observation], trap: &Trap, a: ObsId, b: ObsId) -> Outcome`.
- **The caching idiom `streams.entry(k).or_insert_with(|| read_jsonl(..)?)` does not compile** — `?`
  is not allowed in that closure. Use `contains_key` + `insert`, which is what `read_traps` itself
  does.
- **`answer_trap` resolves `trap.replay` against the BAKED corpus root** (via `fixture_path`), not
  against any root passed to `l1_answers`. So `l1_answers(scratch_root)` reads trap **files** from
  the scratch root and **streams** from `fixtures/`. That is correct and load-bearing for AC7's
  scratch tests, and the story records the limit for `read_traps` only — say it about the runner.
- Minor: *"one read per distinct stream, not one per trap"* is true of `l1_answers`'s **own** cache.
  `read_traps` already reads every stream once per trap **file** for its `obs_id` cross-check, so the
  walk touches each stream at least twice. Do not chase a saving that is not there.

State the shared core and the baked-root limit in the module doc.

### Compile-level facts — measured, and each one costs an hour if it is discovered under `rustc`

- 🔴 **`trap_gate::discover_trap_files` is PRIVATE** (`fn`, `trap_gate.rs:306`). `l1_runner` cannot
  call it. **Make it `pub(crate)`** and say why in its doc. **Do NOT write a third walk** — the tree
  already carries two (`discover_trap_files` and the `#[cfg(test)] walk_trap_files`) whose
  divergence on three points is a registered defect from story 5.2's review; a third would be the
  same mistake a third time.
- 🔴 **`fixtures::walk_trap_files` is `#[cfg(test)] pub(crate)`** (`fixtures.rs:834-835`) — it does
  **not exist** in a non-test build. Production code may not use it. AC7's corpus-byte test is a
  test and may.
- 🔴 **`trap_gate::committed_traps_root()` lives INSIDE `mod tests`** (`trap_gate.rs:393`).
  `l1_runner`'s tests cannot import it; write the one-liner locally —
  `fixtures::fixtures_dir().join("scenario/traps")`. Do **not** reach for a relative path:
  `the_fixtures_path_is_expressed_once` [`fixtures.rs:2204-2250`] counts occurrences of the literal
  `"/../../fixtures"` across `crates/*/src` + `xtask/src` and asserts **exactly 2**, both in
  `fixtures.rs` itself. *(Corrected during validation: the test does not police the substring
  `fixtures/`, so the prescription is right but its stated reason was stronger than the test.)*
- 🔴 **`identity::l1::verdict_for_pair` is `pub(crate)` in `opencmdb-core`** and therefore
  unreachable from `opencmdb-bin`. The entry point is `decide_pair`.
- **`Trap`, `TrapId`, `Expectation`, `RuleId` come from `opencmdb_core::trap`**;
  `Outcome`, `Tally`, `Column`, `run_trap`, `TrapVerdict` from `opencmdb_core::score`;
  `Decision`, `Conclusion` from `opencmdb_core::identity::cascade`; `decide_pair`, `L1_EXACT_MAC`,
  `L1_DISTINCT_MAC` from `opencmdb_core::identity::l1`. `lib.rs` re-exports `Decision`/`Conclusion`
  flat at the crate root — either path compiles; pick one and be consistent with `trap_gate.rs`,
  which uses the full module paths.
- **`FixtureError` is the crate's one error type** (`fixtures.rs:154`) and `score_corpus` already
  returns it. `l1_runner` returns it too — do **not** introduce a second error type, and do **not**
  reach for `anyhow` (D47 permits it only in `main.rs`'s composition root).
- **`ObsId` is `Copy` + `Ord`**; `Observation` is not `Copy`. Look the two observations up by
  `o.obs_id == *id` over the `Vec<Observation>`, or index a `BTreeMap<ObsId, &Observation>` if the
  stream is large — it is not: the longest committed stream is 6 lines.
  ⚠️ **This is NOT an existing idiom to copy.** Measured during validation: `grep -rn "obs_id ==" --include=*.rs crates`
  returns **zero hits**, and `corpus_pairs()` never resolves an `ObsId` to an `&Observation` — it
  builds `CandidatePair`s of ids and hands whole streams to `candidates`. The id → `&Observation`
  resolution is **new code**; `corpus_pairs()` supplies the walk-and-cache shape only.
- **`incomplete_families()` is empty on the committed corpus today** — verified, since
  `passed_is_the_failures_gate_with_a_discovered_floor` asserts `vacuous.passed()`, and `passed()`
  requires it. AC3's assertion is therefore not new information; it is there so a corpus change
  cannot make AC3 pass for a different reason.

### Existing shapes to follow, not reinvent

- **`corpus_pairs()`** [`fixtures.rs:4489-4560`, story 5.6] already does trap → stream → **candidate
  pairs of `ObsId`s**, caching one read per `replay` and using `match trap.observations.as_slice()`
  with a `[a, b]` arm and named residue buckets. **`l1_runner` is its production sibling** — same
  walk-and-cache shape, and it may not call it (a `#[cfg(test)]` helper in another module).
  ⚠️ It stops at ids: it does **not** resolve them to `&Observation`s, so that part is new (above).
- **`decide_pair(&Observation, &Observation) -> Decision`** [`l1.rs:300-302`] is the entry point.
  `verdict_for_pair` is `pub(crate)` **in `opencmdb-core`** and therefore unreachable from
  `opencmdb-bin`; do not try.
- **`Trap::observations: Vec<ObsId>`** and `read_traps` guarantees every id **exists** in the named
  stream [`fixtures.rs:665-703`], so a lookup that finds nothing is a broken invariant, not an
  ordinary outcome — panic with a message that names the trap, in `corpus_pairs()`'s idiom.
- **`read_traps` resolves `replay` against the BAKED corpus root, not against `traps_root`** —
  `trap_gate.rs:218-222` records this limit, and it applies to the runner too: a scratch trap corpus
  may only reference committed replay streams. AC5's and AC7's scratch tests must respect it.
- **`score_corpus` errors with `AnswerForUnknownTrap`** if the map holds an id no trap has
  [`trap_gate.rs:274-281`]. Feeding it answers from a **different** root is caught, not silent.
- **The deliberate redundancies must survive a DRY pass**: `l1.rs`'s `CORPUS_EXACT_MAC` /
  `CORPUS_DISTINCT_MAC` restate the ids as independent literals — measured load-bearing (mutation
  M6 of story 5.5 reds ten tests). AC7's new test is a **third**, independent statement, from the
  TOML side. Do not collapse any of them into a shared constant.

### Test placement

`cascade.rs`'s test-module doc states the convention this project decided once: **a test lives with
the item whose CLAIM it pins; the items it merely READS are dependencies.** So: `outcome_of`'s row
tests in `score.rs`; the answers map and the residue in `l1_runner.rs`; the report counts, the
tally and the reproducibility in `trap_gate.rs`; the corpus byte comparison in `fixtures.rs` or
`l1_runner.rs` — decide it and state the reason in the test's doc.

### References

- `_bmad-output/planning-artifacts/epics.md:1515-1535` — story 5.7's ACs;
  `:1537-1557` — story 5.8, whose premise §3 corrects; `:1317` — the build order.
- `architecture.md:967-974` — D13's table; `:984-986` — L1 is pure A; `:1004-1011` — the blocker;
  `:1246-1253` — D18 on recall; `:1307-1310` — D19, the fixture asserts the RULE.
- `crates/opencmdb-core/src/score.rs` — `Outcome`, `score`, `run_trap`, `Tally`, `ScoredRecord`.
- `crates/opencmdb-core/src/identity/cascade.rs` — `Decision`, `Conclusion`, `decide`.
- `crates/opencmdb-core/src/identity/l1.rs` — `decide_pair`, `L1_EXACT_MAC`, `L1_DISTINCT_MAC`,
  `CURRENT_RULESET_VERSION`.
- `crates/opencmdb-bin/src/trap_gate.rs` — `score_corpus`, `Report`, `RuleMismatch`.
- `crates/opencmdb-bin/src/fixtures.rs` — `read_traps`, `read_jsonl`, `fixture_path`,
  `walk_trap_files`, `corpus_pairs()`.
- `_bmad-output/implementation-artifacts/deferred-work.md` — the entries naming story 5.7.
- `docs/project-context.md`, `CLAUDE.md` — update both in the same push (docs-current-before-push).

---

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context) — `claude-opus-5[1m]`, via `bmad-dev-story`, 2026-08-01.

### Debug Log References

**Baseline for the mutation pass: commit `50470d1` on branch
`story-5.7-trap-runner-stops-scoring-nothing`, off `master` at `6cc137b`.** Committed FIRST, per the
house rule — `git checkout <file>` restores to HEAD, and it destroyed an implementation mid-run once.
Every restore below is `git checkout <file>` against that commit, and the tree was re-verified clean
after each.

**Baseline numbers, measured not quoted:** `cargo test --workspace --locked` →
**151 bin + 153 core + 46 xtask = 350** (333 before this story: 139 + 148 + 46), zero failures.
`l1_runner.rs` is **233 code lines** (first `#[cfg(test)]` at `:234`), `trap_gate.rs` **405**,
`score.rs` **681** — ceiling 2000. _(The `trap_gate.rs` figure read **311** until 5.7's code review
re-measured it: `#[cfg(test)]` is at `:406`. 311 was not merely wrong, it was BELOW this story's own
`6cc137b` baseline of 384, which the commit's +21 non-test lines make impossible — a fabricated
number in a Debug Log whose whole purpose is measured ones.)_

#### M1 — `outcome_of` maps `Conclusion::NoMatch` to `Outcome::Merged`

**RED, 7 tests, every one assertion-carried:**

```
score::tests::a_no_match_becomes_a_refusal_naming_the_same_rule            (core)
l1_runner::tests::an_unimplemented_l1_rule_is_answered_and_reported_as_a_wrong_rule
l1_runner::tests::the_four_pure_l1_families_pass_in_both_decision_poles
trap_gate::tests::a_right_verdict_by_an_l2_rule_is_a_wrong_rule_failure_not_a_truth_table_one
trap_gate::tests::the_committed_corpus_is_scored_by_the_l1_engine
trap_gate::tests::the_per_column_tally_names_the_empty_column
trap_gate::tests::the_report_line_says_thirteen_scored
```

⚠️ **`the_mapping_preserves_the_rule_mirror_on_every_row` does NOT red under M1, and that is
recorded rather than smoothed over.** `Merged` carries a rule too, so `outcome_of(&d).rule()` is
still `Some(the same rule)` — the mirror survives a wrong ROW. The mirror test and the three row
tests are therefore not redundant: each closes what the other cannot.

#### M2 — the selector drops the `l1-` prefix test (`rule().is_some()`)

**RED, 6 tests.** Observed report, verbatim:

```
24 trap(s) discovered, 21 scored, 4 truth-table failure(s), 4 wrong-rule failure(s)
  wrong rule: trap `multi-nic-must-not-merge` (must-not-merge): expected rule `l2-different-switch`, got `l1-distinct-mac`
  wrong rule: trap `shared-hardware-vm-must-not-merge` (must-not-merge): expected rule `l2-different-hostname`, got `l1-distinct-mac`
  wrong rule: trap `vrrp-virtual-mac-must-not-merge-master` (must-not-merge): expected rule `l2-virtual-mac-prefix`, got `l1-distinct-mac`
  wrong rule: trap `vrrp-virtual-mac-must-not-merge-bearers` (must-not-merge): expected rule `l2-different-hostname`, got `l1-distinct-mac`
```

**4 VerdictFail + 4 WrongRule, exactly §1's prediction**, and the four wrong-rule pairs are the four
AC5 names. (§1's *"6 truth-table failures"* counts all 24 answered; this mutation keeps the
`must-abstain` traps out, because they name no rule at all — the two figures agree.)

#### M3 — the selector answers `must-abstain` traps too, `decide(vec![], _)` for the pairless one

**RED, 7 tests.** Observed: `24 trap(s) discovered, 16 scored, 2 truth-table failure(s)`.

**16 = 13 + 3, and only 2 of the three fail — so `example-must-abstain` PASSED.** That is §4's
measured trap, reproduced: the pairless shortcut manufactures a pass out of the engine evaluating
*nothing*, and it would have put a **1** in a `must-abstain` column the gate never asked about.
(A test also asserts the pass is real on its own: `score(&must_abstain, &manufactured) == Pass`.)

#### M4 — the two-observation condition dropped. **Run on BOTH sides, and the difference is the point.**

**M4a, dropped from `answer_trap` → RED, 2 tests, panic-carried:**

```
l1_runner::tests::a_trap_that_names_no_pair_gets_no_answer
l1_runner::tests::the_pass_the_pairless_shortcut_would_have_manufactured_is_real_and_refused
panicked at crates/opencmdb-bin/src/l1_runner.rs:177: index out of bounds: the len is 1 but the index is 1
```

**M4b, dropped from `l1_answers` → the ENTIRE SUITE STAYS GREEN: 151 + 153 + 46, zero failures.**
The level selector removes the corpus's only pairless trap first — `example-must-abstain` carries a
cause and no rule — so all 13 `l1-*` traps name exactly two observations and the walk's output is
byte-identical. **That invisibility IS the measurement**, in story 5.6's M2 idiom, and it is why
AC2's assertion is on `answer_trap` and not on the walk's output: an assertion there would have been
vacuous on the committed corpus.

#### M5 — a renamed `l1-` rule id in a **scratch copy** of the trap corpus (`fixtures/` never touched)

Three variants, each RED and each naming its own guard:

| variant | scratch mutation | observed |
|---|---|---|
| M5a | `l1-exact-mac` → `L1-Exact-MAC ` in one file | ``rule id `L1-Exact-MAC ` in …/randomized-mac.toml is not trimmed`` |
| M5b | `l1-exact-mac` → `l1-exact-mac-v2` in one file | ``the corpus writes `l1-exact-mac-v2` in …/randomized-mac.toml, which `identity::l1` does not implement`` |
| M5c | a corpus of `multi-nic.toml` alone (no `l1-` id at all) | ``no trap in … names `l1-exact-mac` `` |

M5c is what proves the **both-occur** guard load-bearing: without it the "either constant"
assertion passes vacuously over a corpus containing no `l1-*` id, and both constants could then be
renamed to anything. M5b is masked by the earlier assertion when it fires first, which is why M5c
had to remove the ids rather than rename them.

⚠️ **The walk had to be root-parameterised for any of this to be a mutation at all.**
`fixtures::walk_trap_files(visit: &mut dyn FnMut(&Path)) -> usize` takes **no root** — it hardcodes
`fixture_path("scenario/traps")` — so a test built on it reads the committed bytes whatever the
scratch copy says. The byte test walks `trap_gate::discover_trap_files(root)` instead, which this
story promotes to `pub(crate)`.

#### M6 — a selector that answers nothing

**RED, 8 tests.** Observed: `24 trap(s) discovered, 0 scored, 0 truth-table failure(s)` — the state
this story ends. `replaying_the_corpus_twice_yields_an_identical_report` is among the eight, which
is what its `scored() == 13` precondition exists for: without it the equality would have compared
two vacuities and stayed green.

#### Gate, run whole on the restored tree

`cargo fmt --all --check` clean · `cargo clippy --workspace --all-targets -- -D warnings` clean ·
**`cargo clippy --workspace -- -D warnings` (the CI form, without `--all-targets`) clean** ·
`cargo test --workspace --locked` → 151 + 153 + 46, zero failures · `cargo xtask ci` → six gates
green (`float-free` still over **4** files; `views-hash` `ℹ STALE`, exit 0, by design — issue #50).
`git status fixtures/` **empty**, `git diff HEAD -- fixtures/` **empty**, `MANIFEST.toml` md5
`1db9798116a0942eb65f4153ec7b22b6`.

#### One measurement re-derived rather than inherited

§6's stream/capability count was re-run on the tree before it was written into the register:
**11 replay streams are named by a trap and not one carries a `capability` control record**; the
only two streams that carry control records at all — `capability-downgrade.jsonl` and
`partial-then-failed.jsonl` — are named by **no** trap. Reproduced exactly.

### Completion Notes List

**In the weaker true sentence:** the committed trap corpus is now scored by the real L1 engine.
`score_corpus` reports `discovered=24, scored=13, failures=0, rule_mismatches=[],
incomplete_families=[], passed=true`, where `scored` has read **0** since story 4.6b — nine
stories. Nothing under `fixtures/` moved, `epics.md` and `architecture.md` were not edited, and
`architecture-views.md` was not regenerated.

- **AC1 — MET.** `score::outcome_of(&Decision) -> Outcome` in `crates/opencmdb-core/src/score.rs`,
  an exhaustive match on `&decision.conclusion` with **no `_` arm**. Five tests: one per row, the
  `outcome_of(&d).rule() == d.rule()` mirror on every row, and one asserting the two DROPPED fields
  leave no trace (two decisions differing only in `verdict_vector`/`ruleset_version` map to the same
  outcome). **No `impl From<Decision> for Outcome`** — `cascade.rs`'s refusal was kept and updated
  to say the mapping now exists as a named function.
- **AC2 — MET.** `crates/opencmdb-bin/src/l1_runner.rs`, a new module, exposes `answer_trap` and
  `l1_answers`. `score_corpus`'s signature and body are **unchanged** — no engine, no callback, no
  closure; the seam is the `BTreeMap<TrapId, Outcome>`. The two selector conditions are separate
  named predicates (`expects_an_l1_rule`, `named_pair`) so a mutation hits one at a time, and the
  pairless assertion is on `answer_trap`, where M4 shows it is reachable.
- **AC3 — MET**, asserted rather than quoted, including `passed() == true` and the `"13 scored"`
  substring of the rendered line. Full render:
  `24 trap(s) discovered, 13 scored, 0 truth-table failure(s)`.
- **AC4 — MET.** `scored_in` = **7 / 6 / 0**, `failures_in` = 0 in all three columns, the zero named
  in the assertion's own message as the vacuity `scored_in` was built to expose. The four pure-L1
  families pass in both **decision** poles — asserted through `run_trap`, so the RULE is asserted
  too, not only the verdict — and `hostname-absence-must-abstain`, the family's third trap, is
  asserted **absent** from the answers map.
- **AC5 — MET.** `failures() == 0`, `rule_mismatches().len() == 4`, each entry's expected `l2-*` id
  and actual `l1-distinct-mac` asserted by name, `column == MustNotMerge`, `passed() == false`.
  Reached through `answer_trap`, which is level-blind, so the test does not reimplement its subject.
- **AC6 — MET.** Two full runs compare equal AND render identically, with `scored() == 13` asserted
  on the first so the equality is not comparing two vacuities (M6 proves that precondition
  load-bearing).
- **AC7 — MET.** `the_producers_rule_ids_are_the_corpus_spelling` walks
  `trap_gate::discover_trap_files(root)` — **root-parameterised**, which is what makes M5 a mutation
  rather than a no-op — and asserts: every `l1-` id is `L1_EXACT_MAC` or `L1_DISTINCT_MAC`; **both**
  occur; the walk found at least one id at all; and all **seven** ids the corpus writes, `l2-*`
  included, equal their own `trim()` and `to_lowercase()`. The prefix-not-whitelist pin is a
  separate scratch test, **`must-not-merge`** over `minimal.jsonl`'s `…0001`/`…0003` — measured
  `scored=1, failures=0, one mismatch (l1-not-yet-implemented / l1-distinct-mac), passed=false`,
  the story's figure exactly.
- **AC8 — MET.** The residue is asserted as the literal set of **eleven** ids, not as a count, and
  `deferred-work.md` gains `## Deferred from: story-5.7` recording the 8→11 correction with **story
  5.8 named as owner**, the three classes tabulated, and the note that `epics.md` is not edited.
- **AC9 — MET, all 19 sites.** Each is now true or narrowed with a new owner, and **no site says
  "story 5.7 owns it"** — verified by re-grepping `5\.7` over `crates`/`xtask` on the final tree:
  every remaining hit is either a statement of what this story DID or a pointer to where the
  residue now lives. The `VerdictVectorEntry` unification is re-owned with §6's measured obstacle;
  `trap_gate.rs`'s *"the map is empty today"* and *"no answer producer exists"* are corrected; and
  `the_committed_corpus_is_discovered_and_scored_by_nothing` is renamed
  `an_empty_answers_map_scores_nothing_over_the_committed_corpus` — its name now states what its
  CALL does, not a claim about the corpus this story falsifies.
- **AC10 — MET.** Six mutations run on a committed baseline, each red set recorded verbatim above.
  **Every red is assertion- or panic-carried; zero are compiler-carried.** `fixtures/` verified
  untouched two ways (`git status`, `git diff HEAD`) and `MANIFEST.toml` md5-verified.

**333 → 350 tests** (139 → 151 bin, 148 → 153 core, 46 xtask unchanged).

**What this story did NOT do, stated rather than left to be discovered:**

- **no `ScoredRecord`, no `VerdictVectorEntry` unification** — the obstacle is measured, not a
  matter of appetite: a `ScoredRecord` needs a `capability_snapshot`, and **11 of 11** trap-named
  replay streams carry no `capability` control record while `read_jsonl` discards control records by
  construction. Re-owned to the story that gives a trap run a real snapshot;
- **no production caller for the blocker.** `candidates` is still reached only from its own tests
  and `fixtures.rs`'s test module. This was a decision, not an omission: a trap NAMES its pair, so
  the runner has nothing to generate. Story 5.6's entry is answered with that narrow sentence and
  **stays open**, re-owned to the first caller holding observations and no trap;
- **no bucket for the 11 unanswered traps** — story 5.8, which now has a corrected premise;
- **the `must-abstain` column is measured by nothing** (`scored_in == 0`), asserted and named;
- **no `l2-*` rule** (Epic 6), **no persistence** (5.9), **no `Decision::cause()`/`Conclusion::rule()`**.

**Two divergences from the story's own predictions, recorded rather than absorbed:**

1. **M1 does not red the rule-mirror test**, because `Merged` also carries a rule. The story implied
   the mirror was the mapping's main guard; measured, the three row tests are what catch a wrong row
   and the mirror catches a different defect. Both were kept and the reason is now in the test docs.
2. **M5 needed THREE variants, not one.** A single rename cannot exercise the both-occur guard,
   because the "either constant" assertion fires first and masks it. M5c (a corpus with no `l1-` id
   at all) is the only shape that reds it — and it is the shape that would let both constants be
   renamed to anything and stay green.

**One AC7 detail worth carrying forward:** the byte test asserts `checked > 0` as well as the two
occurrence flags. That is not belt-and-braces — a corpus of `must-abstain` traps only would satisfy
neither `l1-` branch and the loop would never run, and `checked` is the only thing that would say so.

### File List

Modified:

- `crates/opencmdb-core/src/score.rs` — `outcome_of` + its doc; imports `Conclusion`/`Decision`;
  `Outcome`'s doc; `VerdictVectorEntry`'s doc (re-owned, with the measurement);
  `ScoredRecord::verdict_vector`'s field doc; `comparable_fields`' `verdict_vector` bullet;
  five new tests.
- `crates/opencmdb-core/src/identity/cascade.rs` — module doc (`:33`, `:38-39`), `RuleVerdict`'s
  relationship section, `Decision`'s *"one producer, and — since story 5.7 — one consumer"* section
  with the `From` refusal restated.
- `crates/opencmdb-core/src/identity/l1.rs` — `L1_EXACT_MAC`'s doc (a test holds the claim now),
  `verdict_for_pair`'s overload paragraph, the test module's deliberate-redundancy paragraph
  (now naming a THIRD statement).
- `crates/opencmdb-core/src/identity/mod.rs` — the blocker's intended consumer, corrected: the trap
  runner reaches the engine and deliberately not the blocker.
- `crates/opencmdb-core/src/lib.rs` — the `join` re-export comment: a consumer HAS crossed the crate
  frontier, and it does not consume `join`.
- `crates/opencmdb-bin/src/main.rs` — `mod l1_runner;`.
- `crates/opencmdb-bin/src/trap_gate.rs` — `discover_trap_files` → `pub(crate)` with the reason;
  module doc narrowed to `score_corpus` and naming `l1_runner`; `Report::scored`'s doc; the renamed
  test and its message; five new tests.
- `_bmad-output/implementation-artifacts/deferred-work.md` — six entries closed by
  append-and-strike, five annotated `↺`, one new `## Deferred from: story-5.7` section.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — status and record.
- `_bmad-output/implementation-artifacts/5-7-trap-runner-stops-scoring-nothing.md` — this file.
- `docs/project-context.md`, `CLAUDE.md` — docs-current-before-push.

Added:

- `crates/opencmdb-bin/src/l1_runner.rs` — the producer (233 code lines + its test module).

**Nothing under `fixtures/` was added, modified or deleted.**

---

## Change Log

| Date | Change |
|---|---|
| 2026-08-02 | **CODE-REVIEWED (three layers), 11 patches applied, 2 deferrals registered, 7 findings dismissed — commit `6f958dd`. Status stays `review`: `done` is the MERGE's business in this project's flow.** AC1–AC10 were re-verified MET by independent measurement (24/13/8/3, the residue of 11 literal ids, 7 distinct rule ids, 11 trap-named replay streams with zero `capability` records, `grep "5.7 owns"` empty, File List matching the diff exactly). **6 of the 11 patches were SENTENCES, not behaviour — the sixth consecutive story with that shape.** Two behavioural gaps, both measured invisible before the tests existed: **the SELF-PAIR was re-opened on the `answer_trap` path** — a hand-built `Trap { observations: vec![x, x] }` gave `decide_pair(o, o)`, which intersects `keys_of(o)` with itself, so `l1-exact-mac` fired and the trap MERGED, a pass no rule reasoned about; story 5.6 had closed that in the TYPE (`CandidatePair::new(a, a)` is `None`) but `answer_trap` never calls `read_traps`, so `Trap::validate`'s `DuplicateObservation` does not hold the precondition there. **Guy's call: close it in `named_pair`** (`[a, b] if a != b`), not in a doc sentence. And **no test covered a trap naming THREE observations** — relaxing the arm to `[a, b, ..]` left all 350 tests green, because every committed trap names ≤ 2 and `Trap::validate` rejects an empty or duplicated list but not a third id. Story 5.6's M2 idiom, on the upper half of the guard AC2 closed for the lower. One assertion strengthened: *"all seven ids"* was QUOTED in three documents and guarded by `checked > 0` alone, so the test would have passed over a corpus reduced to one file — `checked == 21` and `distinct == 7` are now asserted, **in the committed-corpus test and deliberately NOT in the root-parameterised helper**, where they would red M5c on the count and mask the both-occur guard M5c was measured to prove load-bearing. The five claim corrections: `CLAUDE.md` still said *"Nothing feeds the corpus harness (5.7)"* in the present tense, falsified 400 words below in the same file while `project-context.md` had corrected the identical sentence in `b555712`; the Debug Log recorded `trap_gate.rs` at **311** code lines where it is **405**, a figure BELOW the story's own 384 baseline and therefore impossible; `the_committed_corpus_is_scored_by_the_l1_engine` credited the thirteen answers with *"the family completeness"*, which `incomplete_families` computes over ALL discovered traps and cannot be; `expected_answered()` claimed to be the *"third independent statement"* `l1.rs` points at, when that is a RULE-id test 260 lines below and this is a list of TRAP ids; and the module doc's *"answering all 24 traps"* describes a state the code forbids (23 are answerable), with its correct 6 + 4 arithmetic now marked as pinned by no live test. `answer_trap` gains the `# Panics` section its `pub` signature owed. **Prove-to-red on all three new guards, each assertion-carried, on a committed baseline** — and **MR3a is recorded rather than dropped**: truncating the canonicality walk to one file reds the OLDER both-occur guard and masks the new counts, the same masking the story met on M5b/M5c, met again on the first try. **350 → 352 tests** (153 + 153 + 46); six gates green, both clippy forms clean, `fixtures/` verified untouched two ways. Two deferrals registered with owners: `l1_answers` has no cross-file `TrapId` uniqueness check (owner **5.8**; `score_corpus` catches it today, a `len()`-only caller would not), and `outcome_of`'s abstaining row has no end-to-end path through the runner (owner **5.14 / Epic 6**). Seven findings dismissed each with the measurement that killed it — **two of them refuted sub-agent claims**: the scratch test's `!passed()` is not green for family incompleteness (that trap declares no `family` key, and `incomplete_families` skips `None`), and pair order IS pinned (`a_pair_decides_the_same_whichever_side_is_the_left_argument`). |
| 2026-08-01 | **IMPLEMENTED → `review`.** The committed corpus is scored by the real L1 engine: `discovered=24, scored=13, failures=0, rule_mismatches=[], incomplete_families=[], passed=true`, where `scored` had read **0** since story 4.6b. **333 → 350 tests** (151 bin + 153 core + 46 xtask); six gates green including `float-free` over 4 files, both clippy forms clean, `fixtures/` verified untouched two ways and `MANIFEST.toml` md5-verified. AC1–AC10 all met. **Six mutations run on a committed baseline (`50470d1`), every red assertion- or panic-carried, zero compiler-carried** — and each of the story's own numbers reproduced: M2 gives `21 scored, 4 truth-table failure(s), 4 wrong-rule failure(s)` with the four expected/actual pairs verbatim, M3 gives `16 scored, 2 truth-table failure(s)` (so `example-must-abstain` PASSED — §4's manufactured pass, live), M6 gives `24 discovered, 0 scored`. **M4 was run on BOTH sides and the difference is the finding the validation predicted**: dropped from `answer_trap` it panics with `index out of bounds: the len is 1 but the index is 1`; dropped from `l1_answers` the **entire suite stays green**, because the level selector removes the only pairless trap first. **M5 needed THREE variants, not the one specified** — a single rename is masked by the "either constant" assertion, and only a corpus with no `l1-` id at all (M5c) reds the both-occur guard, which is the guard that would otherwise let both constants be renamed to anything. **Two divergences from the story's predictions recorded rather than absorbed**: that, and the fact that M1 does NOT red `the_mapping_preserves_the_rule_mirror_on_every_row` (a wrong ROW keeps the rule, so the mirror and the three row tests each close what the other cannot). §6's obstacle was re-measured on the tree before being written into the register — **11 trap-named replay streams, zero `capability` control records among them**, and the only two streams that carry control records are named by no trap — so the `VerdictVectorEntry` unification is **re-owned, not done**, to the story that gives a trap run a real capability snapshot. The blocker still has **no production caller** and that was a decision, not an omission: a trap NAMES its pair. All **19** doc sites honoured or narrowed, verified by re-grepping the final tree: **zero remaining sites claim story 5.7 owns anything**. `epics.md`, `architecture.md` and `architecture-views.md` untouched. |
| 2026-08-01 | **Validation pass, two fresh-context agents (fact-check + gap-hunt), MANDATORY per Guy's Epic 4 retrospective decision.** The gap-hunt agent **implemented the story end to end** in an isolated worktree off `6cc137b` — AC1–AC8 built and run, mutations M1/M3/M4/M5/M6 executed, all six gates green, `git status fixtures/` empty. **The pattern held for the third consecutive story: all 3 HIGH findings came from the agent that COMPILED the story, 0 from the agent that checked its claims.** Every number the story predicts was reproduced exactly — the 24-row §1 table row for row, `discovered=24 / scored=13 / failures=0 / mismatches=empty / passed=true`, `scored_in` 7 / 6 / 0, AC5's `failures=0` + `4` mismatches, AC6's identical replay, AC8's residue of 11 — and §5's load-bearing *"same type"* claim (`Outcome::Abstained` and `Conclusion::Abstained` both carry `IdentityAbstentionCause`, which is `Copy`) is TRUE, so AC1 is implementable verbatim. The `Display` render is `24 trap(s) discovered, 13 scored, 0 truth-table failure(s)` — AC3's `"13 scored"` substring holds. **13 findings applied: 3 HIGH, 5 MEDIUM, 5 LOW.** **The three HIGH, each a prescription that only broke under a compiler:** **(H1)** AC7's scratch trap was unspecified as to **column**, and the obvious `must-merge` form lands in `failures()` with `rule_mismatches()` **empty** — because `minimal.jsonl` holds **no pair the L1 engine merges** (MACs `…:53:01`, none, `…:53:02`) and `run_trap` raises `WrongRule` only on a verdict PASS. AC7 now prescribes `must-not-merge` over obs `…0001`/`…0003`, with the corpus fact stated. **(H2)** **M5 was a no-op**: `fixtures::walk_trap_files` takes **no root** (it hardcodes `fixture_path("scenario/traps")`), so a rename in a scratch copy is invisible to it — measured, the walk still reported 21 canonical ids. AC7's byte test is re-prescribed over `discover_trap_files(root)`, which this story promotes to `pub(crate)` anyway. **(H3)** **M4 reds nothing and AC2's named condition was vacuous**: of the 13 traps whose expected rule is `l1-*`, **zero** name other than two observations — the corpus's only pairless trap carries a cause and is excluded by the **level** selector first. The pair guard is unreachable through `l1_answers`; AC2's assertion is now on `answer_trap(&example_must_abstain) == Ok(None)` and M4 is applied there (measured: index-out-of-bounds panic). Story 5.6's M2 idiom, again. **The five MEDIUM:** *"14 sites across 6 files"* is neither figure — the grep spans **5** files, and with the 5 unmatched sites the set AC9 governs is **19**; AC8 asserted *"exactly the literal set measured in §3"* and **§3 contained no such set** (the 11 ids are now written out there); §6's *"four sites … and each states the condition"* is **five** sites of which only **two** state it; the prescribed shared core `(&Observation, &Observation) -> Outcome` is a one-liner that factors nothing — the real duplication is the id→`&Observation` resolution plus the trap-naming panic, so `answer_pair(stream, trap, a, b) -> Outcome` is prescribed instead; and the `obs_id ==` *"`corpus_pairs()` idiom"* **does not exist** (`grep` → **0** hits; `corpus_pairs()` stops at pairs of ids), so that resolution is new code, not a copy. **The five LOW:** the caching idiom `or_insert_with(\|\| read_jsonl(..)?)` does not compile (`?` in a closure — use `contains_key` + `insert`, as `read_traps` does); `answer_trap` resolves `replay` against the **baked** root, so `l1_answers(scratch)` reads traps from the scratch root and streams from `fixtures/` — stated for `read_traps` only, now stated for the runner; *"one read per distinct stream"* is true of the runner's own cache but `read_traps` already reads each stream once per trap file; `the_fixtures_path_is_expressed_once` counts `"/../../fixtures"` and asserts **2**, it does not police the substring `fixtures/` (prescription right, reason overstated); §6's *"there is no production source"* of `Capabilities` is **false** — `arp_ping.rs:183-187` builds one above its `#[cfg(test)]`, and `capability-downgrade.jsonl:3` carries one — narrowed to *"none a trap run can reach"*, which is what §6's conclusion actually needs. Also corrected: *"their traps pass in BOTH poles"* is `epics.md:1529`, not `:1527`; §7's *"nothing in the tree checks it"* is the register's *"nothing in **this crate**"* (the wider sentence happens to be true, but is not what is written down). **One gap-hunt finding was DISMISSED after re-measurement**: `Expectation::rule()` cited as `trap.rs:107-113` — the doc line is 106 and the fn spans 107-112, so the citation resolves; the agent's "106-111" is no more exact. All 14 grep rows, every other line citation, §6's stream/capability counts, §7's seven ids and their canonicality, §8's 333 tests / `#[cfg(test)]` offsets / six gates, and the float-free trap (`assert!(true, "story 5.7 …")` → red; `///`, `//!` → safe) reproduced exactly. The prototype implementation was **discarded** — `dev-story` rewrites from the corrected story. |
