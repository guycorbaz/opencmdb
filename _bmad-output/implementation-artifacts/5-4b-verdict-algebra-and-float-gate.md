# Story 5.4b: The verdict algebra is a total function, and no float can reach it

Status: review

<!-- Validation is MANDATORY here (Guy's decision, Epic 4 retrospective 2026-07-26): two
     fresh-context agents (fact-check + gap-hunt) BEFORE dev-story. The template banner saying
     "Validation is optional" does not apply to this project. -->

## ⚠️ Prerequisite: PR #52 must be MERGED before `dev-story` runs

This story builds directly on story 5.4's five types **and on its code review's patches**, and both
live on branch `story-5.4-decision-and-ruleset-version` (commits `ab0d723` + `3d63544`), not on
`master`. Every `cascade.rs` line number in this file was measured on **`3d63544`**, the post-review
tree, which is what `master` will hold once PR #52 merges.

**The four previous Epic 5 stories were each merged before the next was contexted; this one was not**
— it was contexted while #52 was still open, at Guy's request, and the deviation is recorded rather
than smoothed. Consequence for the dev: **branch 5.4b off `master` only AFTER #52 is merged.** If the
line numbers below do not match the tree you have, that is the symptom — check `git log --oneline -3`
for `3d63544` before treating it as a finding.

## Story

As the identity engine,
I want the verdict algebra as a **total** pure function, and a gate that refuses a float in the
identity subtree,
so that no input class falls through D13's table unnoticed and no weight can enter through the back
door.

**This story writes the ALGEBRA and one gate. It writes no rule and no producer.** Nothing emits a
`Verdict`; nothing calls `decide` outside its own tests. Story 5.5 owns the L1 join and the first
firing rule, 5.6 the blocker, 5.7 the corpus wiring, 5.9 persistence, 5.14 the operator surface,
Epic 6 the `l2-*` rules. The build order, quoted as `epics.md:1317` groups it: *"the three debt
stories (5.1, 5.2, 5.2b) -> the engine's vocabulary (5.3, 5.4) -> the verdict algebra (5.4b) -> the
pure join (5.5) -> the blocker (5.6) -> wiring it to the corpus (5.7, 5.8) -> persistence (5.9,
5.10) -> the invariants (5.11, 5.12, 5.13) -> the operator-visible surface (5.14)"*.

**Nothing under `fixtures/` moves.** No artefact bytes, no `MANIFEST.toml`, no re-hash. This story
does not read the corpus at all. If any step appears to require re-authoring a committed artefact,
**STOP** — that is a finding, reported rather than absorbed. The procedure is in Dev Notes.

**`architecture.md` is NOT edited.** D13's table is short one row and this story proves it; the
correction to the decision body is a **milestone** act, never a story task. Same standing rule as
`architecture-views.md` (GitHub issue #50).

## What this story inherits, measured rather than assumed

Story 5.4 shipped the five types and **no algebra**; its code review then patched 22 findings. Three
of those findings were handed forward **to this story's contexting on purpose**, and this file is
where they get answered:

1. **`decide`'s RETURN type was never specified.** `epics.md:1455` does give the input — *"`decide`
   is a **pure function over a verdict set**"* — and says nothing about what comes back, nor who
   supplies `ruleset_version`. AC1 settles both, and the choice is not cosmetic — see *Why `decide`
   returns a `Decision`* in Dev Notes.
2. **Three documents name 5.4b owner of the conclusion↔`verdict_vector` coherence invariant, and no
   AC covered it.** AC5 covers it, and AC8 records which half this story can actually close and which
   half cannot.
3. **The workspace's first `f32`/`f64` token already exists, in the subtree this story's gate greps.**
   It is a *quotation* of D13 at `cascade.rs:52`. AC6 makes it the gate's committed negative test
   case rather than its first false positive.

## The finding this story FIXES

Enumerating D13's table [architecture.md:967-974] over the PRESENCE of each verdict, **exactly one
input class is covered by no row**: `≥1 Opposes`, with no `Decisive`, no `Supports` and no
`Disqualifying`. Re-derived independently three times (story 5.4's contexting and both of its
validation agents — the register credits exactly those; its code review's Edge Case Hunter found the
duplicate-`RuleId` entry, not this one), and a fourth time at this story's
contexting over the eight combinations of `(Decisive, Supports, Opposes)` that survive
`Disqualifying = absent`:

| `Decisive` | `Supports` | `Opposes` | D13 row | conclusion |
|---|---|---|---|---|
| ✔ | ✔ | ✔ | row 3 (and row 5) | `Ambiguous` |
| ✔ | ✔ | ✗ | row 2 | `Match` |
| ✔ | ✗ | ✔ | row 3 | `Ambiguous` |
| ✔ | ✗ | ✗ | row 2 | `Match` |
| ✗ | ✔ | ✔ | row 5 | `Ambiguous` |
| ✗ | ✔ | ✗ | row 4 | `Ambiguous` |
| **✗** | **✗** | **✔** | **NONE** | **← this story answers it** |
| ✗ | ✗ | ✗ | row 6 | `NoMatch` (absence of proof) |

It is not *"only `Neutral` / nothing"* (there IS an `Opposes`), it is not *"`Supports` AND
`Opposes`"* (there is no `Supports`), and every remaining row requires a `Decisive`.

**Guy's arbitration, 2026-07-29: that class concludes `Abstained { AbsenceOfProof }`.** Nothing
argues FOR the merge, so there is no merge to refuse, and D13 deliberately reserves the
refusal-that-names-a-rule for `Disqualifying`. **This story implements that arbitration; it does not
re-open it.**

## Acceptance Criteria

1. **AC1 — `decide` is a pure, total function whose SIGNATURE makes the coherence invariant
   provable.**
   **Given** story 5.4's `Verdict`, `RuleVerdict`, `Conclusion`, `Decision` and `RulesetVersion`, and
   D13's *"all rules are evaluated… verdicts combine by an algebra, not a sum"* [architecture.md:960-961]
   **when** the algebra is written
   **then** `crates/opencmdb-core/src/identity/cascade.rs` gains

   ```rust
   pub fn decide(verdict_vector: Vec<RuleVerdict>, ruleset_version: RulesetVersion) -> Decision
   ```

   — a free function in the module, **not** a method on `Decision` (nothing holds a `Decision` before
   `decide` produces one).

   Binding specifics, so they are not re-litigated at review:
   - **It returns `Decision`, NOT a bare `Conclusion`.** This is the story's central design choice and
     Dev Notes carries the argument. The consequence that matters: the returned `Decision`'s
     `verdict_vector` **is the input**, moved in, so *"a `Conclusion` naming a rule absent from its own
     vector"* and *"a `Match` with an empty vector"* become **unrepresentable by construction** rather
     than merely unenforced. AC5 tests both.
   - **`ruleset_version` is a PARAMETER.** Story 5.4's AC3 already bound this — *"the version arrives
     as a parameter at construction (5.4b's `decide` signature takes one)"*. It is passed through
     verbatim; `decide` neither defaults it, validates it, nor invents a constant. `RulesetVersion(0)`
     is accepted, as 5.4 registered.
   - **PURE: no clock, no I/O, no SQL, no allocation of ambient state.** *"the engine is a pure
     function: a `FixtureConnector` and nothing else — no database"* [architecture.md:3302] and
     *"the engine never touches the clock (D19)"* [architecture.md:3364]. `opencmdb-core` must not
     gain `anyhow`, `axum`, `sqlx` or `askama` (D47).
   - **TOTAL: it returns for every `Vec<RuleVerdict>`, including the empty one and including a vector
     that names the same `RuleId` twice.** No `panic!`, no `unwrap()`, **no `expect()`**, no
     `todo!()`, no `Result`. A `Result` here would be an error type with no error to carry.
   - ⚠️ **The rule-naming arms discharge their `Option` BY CONSTRUCTION, not by `expect()`.**
     Selecting the named rule yields an `Option<&RuleId>` (`.min()` over a filtered iterator) while
     the presence test that got you into the arm has already proved it `Some` — and the type system
     does not know that. **Do not bridge the gap with `expect()`**; make the presence test and the
     selection ONE act: `if let Some(rule) = min_rule_with(&verdict_vector, Verdict::Disqualifying)`.
     No arm then ever holds an `Option` it must prove `Some`.
     *(For the record, so nobody re-derives it: the `any Disqualifying` arm can never fire with no
     rule to name — the presence of the verdict IS the presence of a `RuleVerdict` carrying it, so
     the candidate set is non-empty by construction. The problem is expressive, not logical.)*
   - **`#[must_use]` is NOT added**, for the reason story 5.4's code review measured: the workspace
     carries exactly one `#[must_use]` in total (`opencmdb-bin/src/main.rs`), so adding one here is
     the deviation, not the convention. Registered, not applied.

2. **AC2 — every one of D13's six rows is implemented, and each arm cites the line it comes from.**
   **Given** D13's table [architecture.md:967-974] and `Conclusion`'s three variants
   **when** an arm is written
   **then** the mapping is exactly this, and each arm carries the architecture line in a comment:

   | D13 condition | D13 says | `Conclusion` produced | line |
   |---|---|---|---|
   | any `Disqualifying` | `NoMatch`, **absolute priority, short-circuits everything** | `NoMatch { rule }` | `:969` |
   | a `Decisive`, no `Opposes` | `Match` | `Match { rule }` | `:970` |
   | a `Decisive`, ≥1 `Opposes` | `Ambiguous` — *the cloned-MAC case* | `Abstained { Ambiguous }` | `:971` |
   | no `Decisive`, ≥1 `Supports`, no `Opposes` | `Ambiguous` (weak evidence) | `Abstained { Ambiguous }` | `:972` |
   | `Supports` AND `Opposes` | `Ambiguous` (conflict) | `Abstained { Ambiguous }` | `:973` |
   | only `Neutral` / nothing | `NoMatch` (absence of proof) | `Abstained { AbsenceOfProof }` | `:974` |

   - ⚠️ **D13's `NoMatch` splits two ways onto `Conclusion`, and that is the fork story 5.4 built at
     the type level.** The `any Disqualifying` half HAS a rule to name and lands on
     `Conclusion::NoMatch { rule }`; the `only Neutral / nothing` half has none and lands on
     `Abstained { AbsenceOfProof }` — `Conclusion::NoMatch` names a rule *always*, and there is no
     rule to name when nothing spoke. This closes the half of `deferred-work.md`'s two `NoMatch`
     entries that has been open since story 4.6a. AC8 carries the annotations.
   - **`Disqualifying` short-circuits and is checked FIRST**, before any other presence test. D13's
     word is *"absolute priority"*; an arm order that reaches it second would be correct by accident.
   - ⚠️ **BINDING, AND IT IS THE STORY'S LOAD-BEARING CONSTRUCT: the arms are a `match` on the
     presence TUPLE**
     `match (has_disqualifying, has_decisive, has_supports, has_opposes) { .. }`
     **— NOT an `if` / `else if` chain.** This was measured, not preferred: the gap-hunt validation
     agent compiled all three arm shapes the story otherwise permitted and found that with an
     `if`-chain ending in a permitted `else`, **deleting the arbitration arm compiles and changes
     ZERO of the 16 input classes** — *"SHAPE B: it compiles after M2, and 0 of 16 input classes
     changed answer"*. The suite stays green and Task 5's M2 has nothing to quote. With an `if`-chain
     whose final `else` IS the arbitration, deletion gives `error[E0317]: 'if' may be missing an
     'else' clause` — which proves only that an if-chain needs an else, and is compiler-carried.
     **Only the presence-tuple `match` turns the deletion into a truthful `error[E0004]`**, which is
     the whole point: the compiler, not a reviewer, is what notices a missing class.
     *Ordering is preserved inside the `match` by putting the `Disqualifying = true` arms first and
     letting them bind the other three positions with `_`.*
   - **The three `Ambiguous` rows collapse onto one `Conclusion` variant, and the doc says so** —
     `Abstained { Ambiguous }` carries no discriminator for *which* of the three produced it.
     `IdentityAbstentionCause` has exactly two variants (`cascade.rs:378-396`), and splitting
     `Ambiguous` into three would invent a vocabulary D13 does not have and story 5.3 deliberately
     did not write. **Registered with owner 5.14**, the first story that groups abstentions for an
     operator and therefore the first with a reason to want the distinction.

3. **AC3 — the uncovered input class concludes `Abstained { AbsenceOfProof }`, and the gap is
   documented at the function.**
   **Given** *The finding this story FIXES* above
   **when** `≥1 Opposes` arrives with no `Decisive`, no `Supports` and no `Disqualifying`
   **then** `decide` returns `Conclusion::Abstained { cause: AbsenceOfProof }`, in an arm that is
   **explicitly labelled as the arbitration and not as a D13 row** — it cites no architecture line,
   because there is none to cite; it cites *Guy's arbitration, 2026-07-29*.

   - The function's doc carries the enumeration table above (or its substance) so the next reader can
     re-derive the gap instead of trusting it.
   - ⚠️ **`architecture.md` is NOT patched.** The correction to D13 is registered as a **milestone**
     item, with the same standing as `architecture-views.md`'s regeneration (issue #50) — *"never to
     a story"*, `epics.md:1461`. AC8 carries the entry. **Opening a GitHub issue for the D13
     correction is REQUIRED** (CLAUDE.md: issues are the single source of truth for work items
     outside the story flow); a milestone item that lives only in the register is not tracked.

4. **AC4 — the rule named is chosen deterministically and independently of arrival order.**
   **Given** that a decision names ONE rule while several rules may be `Disqualifying` or `Decisive`
   at once, and D13's *"which rule fires first? The one written first. **That is not a decision, it
   is an accident of file order**"* [architecture.md:936-937]
   **when** more than one verdict qualifies
   **then** the rule named is the **lexicographically smallest `RuleId`** among the qualifying
   verdicts, and **a test PERMUTES the input to prove it** rather than prose asserting it.

   - **`RuleId` already derives `Ord`** — measured on the post-review tree, `trap.rs:39`:
     `#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]`. **No
     derive is added anywhere by this story.** If a derive appears to be needed, that is a finding.
   - **"Qualifying" is per-arm**: for the `any Disqualifying` arm the candidates are the
     `Disqualifying` verdicts; for the `Match` arm they are the `Decisive` ones. A `Supports` never
     gets named, because no row names a rule off a `Supports`.
   - ⚠️ **The doc must say this is a TIEBREAK WITH NO SEMANTIC CONTENT.** Lexicographic order on an
     identifier is not a priority: `l1-distinct-mac` is not "more disqualifying" than
     `l1-exact-mac`. It is chosen because it is the only order-independent rule available that
     invents nothing — no rule priority exists, because **no rule exists**. Say the weaker true
     sentence, and **register that a designed priority replaces it** when rules have one.
     **Owner: story 5.5** (the first firing rule) for L1, **Epic 6** for `l2-*`.
   - **The permutation test is the AC, not a nice-to-have.** For a vector of three qualifying
     verdicts, all **six** permutations must yield the same `Decision.conclusion`. Assert on the
     conclusion, **not** on the whole `Decision` — the `verdict_vector` field legitimately differs
     between permutations, since it carries the input in the order it arrived. A test asserting
     whole-`Decision` equality across permutations would red for the wrong reason, and that is the
     first trap this bullet disarms.
   - 🚨 **THE SECOND TRAP, AND IT MAKES THE TEST VACUOUS IF MISSED: all three verdicts must carry the
     SAME verdict, and it must be one whose arm NAMES A RULE.** Three `Disqualifying`, or three
     `Decisive` with no `Opposes`. **An `Abstained` names no rule**, so a "three qualifying verdicts"
     vector read as *"one `Decisive`, one `Supports`, one `Opposes`"* concludes
     `Abstained { Ambiguous }` under **every** permutation **regardless of the tiebreak** — the test
     then passes with "first in the vector", with "last in the vector", with any rule at all, and
     **Task 5's M3 fires no red**. The three `RuleId`s must additionally be chosen so their
     **lexicographic order differs from their vector order** (e.g. supply them as `"c", "a", "b"`),
     or the identity permutation is indistinguishable from first-wins.
     The assertion is `assert_eq!(d.conclusion, Conclusion::NoMatch { rule: rule("a") })` — the rule
     spelled out — for all six permutations.
     *(Measured by the gap-hunt validation agent: the 32-subset walk gives AC4 **zero** coverage,
     because it builds one `RuleVerdict` per present verdict, so no arm ever has two candidates and
     `min()` over a singleton is order-free. This test is the ONLY coverage the tiebreak has.)*

5. **AC5 — totality is proven over EVERY input class, not a sample, and the coherence invariant is
   tested.**
   **Given** `epics.md`'s *"the table's totality is proven by exercising **every** input class, not a
   sample"*
   **when** the tests are written
   **then**:
   - **All 32 subsets of the five `Verdict` variants are exercised** — `2^5`, built by iterating a
     `0..32` bitmask over `Verdict::all()` — plus the empty vector, which is the `0` case and is
     therefore already in the 32. Each subset builds one `RuleVerdict` per present verdict, with a
     distinct `RuleId`, and calls `decide`.
   - **The expectation is an INDEPENDENT second oracle, not the implementation restated.** Write the
     expected `Conclusion` for each subset from D13's table read directly, in the idiom this project
     protects by name: *"a test that restates the corpus bytes as a second independent oracle
     (`fixtures.rs`'s `expected()`)"* is DELIBERATE redundancy and must not be collapsed by a DRY
     pass (CLAUDE.md, engineering conventions). **A test that calls `decide` to compute its own
     expectation proves nothing**, and is the failure this bullet forbids.
   - 🚨 **The oracle returns a FULL `Conclusion` — rule included — and the assertion is `assert_eq!`,
     never `matches!`.** The rule is the half of a conclusion an arm can silently get wrong, and an
     oracle that predicts only the variant (or a `matches!` that ignores the payload) cannot see it.
   - 🚨 **A named rule must have QUALIFIED, and one test says so on its own.** Nothing in the ACs above
     forbids naming a rule that argued the other way: an implementation computing
     `verdict_vector.iter().map(|rv| &rv.rule).min()` **once** and reusing it in both rule-naming arms
     is deterministic, order-independent, and satisfies the coherence check trivially (the rule is
     always in the vector) — yet on `[("a", Decisive), ("z", Disqualifying)]` it returns
     `NoMatch { rule: "a" }`, **a refusal naming the rule that argued FOR the merge**, which D13
     reserves to `Disqualifying`. Add the test by name:
     `a_disqualifying_names_the_disqualifying_rule_not_the_smallest_one`, asserting
     `NoMatch { rule: rule("z") }`. *(Found by the gap-hunt validation agent, which constructed the
     passing-but-wrong implementation.)*
   - **The coherence invariant is tested, both halves that CAN be tested:** (a) for every subset whose
     conclusion names a rule, that rule **appears in the returned `Decision`'s own `verdict_vector`**;
     (b) `decide(Vec::new(), v)` returns `Abstained { AbsenceOfProof }` and **can never** return
     `Match` — so *"merged, with no explanation"* is unrepresentable through this function.
   - **A duplicated `RuleId` does not break totality.** `decide` is called with a vector naming the
     same `RuleId` twice with two different verdicts; it returns, deterministically. **This does NOT
     close the register's duplicate-rule entry** — see AC8, which states exactly which half moves and
     why.
   - Test names are sentences that say what they prove. Assertion messages name the **subset** and
     the claim: with 32 cases in one loop, *"expected Abstained"* is not actionable in a CI log.

6. **AC6 — `cargo xtask ci` gains a gate that reds on a float under `identity/`, and the committed
   D13 citation is its NEGATIVE test case.**
   **Given** D13's *"REFUSED: `rule -> confidence: f64`… if the output is a float, B has won in
   disguise"* [architecture.md:956-958], and that the rule is **currently true by accident** — measured
   at `505379e`: zero `f32`/`f64` in the whole Rust workspace
   **when** `cargo xtask ci` runs
   **then** a **sixth** gate (five exist today) reds on any `f32` or `f64` **in code** under
   `crates/opencmdb-core/src/identity/`.

   - **Idiom, exactly**: a `fn gate_float_free(root: &Path) -> Result<(bool, String)>` in
     `xtask/src/main.rs`, alongside `gate_ddl_collation` (`:289`) and `gate_vocabulary` (`:384`),
     registered in **`run_ci()`** — NOT in `main()`, which only dispatches — next to
     `let (g4, m4) = gate_file_size(&root)?;` (`:159`, inside `run_ci()` at `:138-166`), and printed
     through `fn report` (`:177-179`), whose glyphs are **`✅` / `🔴`**, not `❌`. **Rust, never YAML** (D56/D65). Its message names the offending file and
     line, as the other gates name their offender.
   - ⚠️ **IT MUST STRIP LINE COMMENTS, AND THIS IS NOT AN OPTIMISATION — the tree already contains the
     case that would red.** Measured on `3d63544`:
     `grep -rn "\bf32\b\|\bf64\b" crates xtask --include=*.rs` → **1 hit**, `identity/cascade.rs:52`,
     which is a **quotation of D13's own refusal** inside a `///` doc comment. A naïve line grep — the
     literal idiom of the DDL and vocabulary gates — reds on day one on a citation of the decision it
     enforces. **Strip from the first `//` to end of line before matching.** That one rule covers
     `//`, `///` and `//!` and a trailing comment after code.
   - 🚨 **A word-bounded `f32`/`f64` match is NOT ENOUGH, and this was MEASURED, not reasoned.** The
     gap-hunt validation agent built the gate exactly as specified — strip at `//`, then
     `contains_word("f64")` using the crate's own helper — and ran it:

     ```
     line                                              should    gate
     let _x: f64 = 0.0;                                  true    true
     /// REFUSED: `rule -> confidence: f64` …           false   false
     let confidence = 0.85f64;                           true   false  << MISSED
     let confidence = 1f32;                              true   false  << MISSED
     let confidence = 0.85;                              true   false  << MISSED
     /* let w: f64 = 0.5; */                            false    true  << FALSE POSITIVE
     ```

     A **suffixed** literal (`0.85f64`) has no word boundary before the `f`, and a **bare** literal
     (`let confidence = 0.85;`) contains no `f32`/`f64` token at all while still being an `f64` by
     inference — **that is a weight entering through the back door, which is the gate's entire
     purpose.** So the gate ALSO matches a float LITERAL: a `[0-9]+\.[0-9]` token, or an `f32`/`f64`
     suffix on a numeric literal.
   - **State the limits, and state the RIGHT ones.** Measured on the tree: neither `/* */` nor
     `#[doc = "…"]` occurs anywhere under `identity/` today, so the two limits the first draft named
     were the two that cannot bite. The real ones, in the honest direction:
     · a float inside a **block comment** `/* … */` **reds — a FALSE POSITIVE**, not a false negative
     (none exists today; if one ever does, that is the sentence to revisit);
     · a float after a `//` **inside a string literal** is missed (false negative, harmless — a float
     in a string is not a float);
     · `#[doc = "…"]` is not stripped and would red.
     **The doc says what the gate catches and what it does not** — the house rule is that a comment
     asserting a checkable property gets checked.
   - ⚠️ **The gate WALKS THE SUBTREE RECURSIVELY** (`walkdir`, already an `xtask` dependency and what
     `gate_ddl_collation` uses). `architecture.md:3370-3372` names a future
     `identity/field_decision/`; a non-recursive `read_dir` loop goes silently blind the day it is
     created. Tested with one nested subdirectory.
   - ⚠️ **It FAILS CLOSED if `crates/opencmdb-core/src/identity/` is absent**, and the file holds two
     contradicting precedents so this must be stated rather than inferred: `gate_ddl_collation`
     returns green with *"no migrations/ yet — nothing to check"*, while the fixture gate fails closed
     with its reason written out — *"reporting 'nothing to check' on the deletion of the thing being
     guarded is a guarantee the gate does not have."* **Follow the fixture gate**: the directory
     exists today, so its disappearance is a finding, not a skip. Tested.
   - **Prove-to-red, both directions, and both RUN:**
     (a) inserting a real float in code under `identity/` (e.g. `let _x: f64 = 0.0;`) reds the gate —
     quote the observed message;
     (b) the committed `cascade.rs:52` citation leaves it **green** — this is the regression the
     stripping exists for, and it is already in the tree, so it needs no fixture;
     (c) inline `#[cfg(test)]` tests in `xtask/src/main.rs`, **at TWO levels**:
     · the **line-level helper** in the idiom of `ddl_flags_bare_text_column_and_passes_a_collated_one`
     (`:941`) and `vocabulary_reds_on_a_stale_doc` (`:904`) — those two test a pure string function
     and build no temp tree;
     · **`gate_float_free` ITSELF against a temp tree**, built with the module's own
     `fn scratch(tag: &str) -> PathBuf` (`:1088`, whose doc explains why the per-test tag exists — *"a
     shared constant path races between concurrent runs"*), as the fixture-gate tests do (`:1330`,
     `:1404`): one file with a float in code, one with a float in a comment, **one nested
     subdirectory**, and **one run with the directory absent**.
     ⚠️ Testing only the string helper would leave the walk, the recursion and the missing-directory
     behaviour with no test at all while this AC read as satisfied.
   - ⚠️ **File-size headroom, so the gate's own message is not a surprise:** `xtask/src/main.rs` is
     **1484 lines total, first `#[cfg(test)]` at `:885` → 884 CODE lines**, and it is **the largest
     file in the workspace** — `cargo xtask ci` reports *"22 file(s) under 2000 code lines (largest:
     884)"*. The ceiling is 2000 and a gate is ~40–60 code lines, so there is an order of magnitude
     of room; the number is stated because this is the file that would hit the ceiling first, not
     because it is at risk.
   - **The gate scopes to `identity/` and no wider.** Widening it to the whole workspace is a
     different decision with a different blast radius (`opencmdb-bin` may legitimately want a float
     for a UI ranking one day — D13 permits *"floats may RANK, never DECIDE"*). Registered with owner
     **5.14**, the first story with a ranking surface.

7. **AC7 — no ranking value is invented, and the milli-units corollary is registered, not
   implemented.**
   **Given** D13's *"Corollary (portability): `confidence` is an **INTEGER in milli-units (0..1000)**,
   never `REAL`/`DOUBLE`"* [architecture.md:991-993]
   **when** this story ends
   **then** **no milli-unit type, constant or field exists** — Epic 5's L1 is a deterministic lookup
   with nothing to rank, so a `0..1000` integer here would be a value asserting that a ranking exists.
   The corollary **binds the day a float would otherwise appear**; the deferral is registered with its
   owner (**5.14**, the Resolve panel — *"a score may order candidates in the Resolve panel (UI
   comfort)"* [architecture.md:988-990]).

8. **AC8 — the register is closed by appending, never rewriting, and each entry says which HALF
   moved.**
   **Given** `deferred-work.md`'s **EIGHT** entries naming story 5.4b as owner, measured on
   `3d63544` and spread across **FOUR** sections — five in `## Deferred from: story-5.4`, one in
   `## Deferred from: code review of story-5.4`, and **two `NoMatch` entries carrying 5.4b
   annotations in `## Deferred from: code review of story-4.6a` and `## Deferred from: story-4.7a`**
   ⚠️ **Verify the set yourself before annotating: `grep -n '5\.4b' deferred-work.md`.** *(This AC
   said "six entries, all in `## Deferred from: story-5.4`" until both validation agents counted
   independently: only five are in that section, and four annotation lines across two older sections
   read* "Story 5.4b writes the function that chooses" *and* "which side an input falls on is still
   decided by nothing" *— sentences this story's own `decide` falsifies. Shipping them unannotated
   would put a document contradicting its own commit into the same push.)*
   **when** the story ends
   **then** each is annotated in the file's own `✅ **CLOSED by story X.** ~~struck~~` / `↺ PARTLY
   closed` idiom, and **never rewritten**:
   - **The D13 table gap** — ✅ **CLOSED**: the arbitration is implemented and every input class is
     exercised. The **milestone correction to D13 itself is NOT closed** and moves to a new entry
     naming the GitHub issue AC3 requires.
   - **The conclusion↔`verdict_vector` coherence** (a conclusion naming a rule absent from its own
     vector; `Match` with an empty vector) — ✅ **CLOSED**, and the closure names the MECHANISM:
     `decide` builds both together and returns the input vector, so the state is unrepresentable
     through the function. ⚠️ **It stays representable via a struct literal**, because `Decision`'s
     fields are `pub` with no constructor — say so. The remaining exposure moves to **5.9**, the
     first story that reconstructs a `Decision` from anywhere other than `decide` (persistence).
   - **A `verdict_vector` naming the same `RuleId` twice, and `Abstained { Ambiguous }` with an empty
     vector** — ↺ **PARTLY closed**: `decide` is TOTAL over both and its behaviour is now tested, so
     the *totality* half is closed. **REFUSING them is not**, and cannot be here: refusing needs a
     PRODUCER that emits one verdict per rule, and no rule exists. **Owner moves to story 5.5.** Not
     struck.
   - **The first `f32`/`f64` token under `identity/`** — ✅ **CLOSED** by AC6's gate and its
     comment-stripping, with the committed citation as the negative test case.
   - **5.4b's owner/spec gap** (three documents naming 5.4b owner of an invariant its criteria did not
     mention; what `decide` returns; who supplies `ruleset_version`) — ✅ **CLOSED**: AC1 and AC5
     answer all three, and this story's ACs are where the answers live.
   - **The `Decision` literals in `cascade.rs`'s tests that carry an empty `verdict_vector`**
     (`## Deferred from: code review of story-5.4`) — the entry predicted they *"must be rewritten the
     day 5.4b enforces the invariant"*. ⚠️ **Check whether that prediction held.** `decide` does not
     enforce anything about a hand-built literal, so those tests may well compile untouched — if they
     do, the entry is closed by **measurement refuting its own prediction**, and the record says so
     rather than quietly striking it. *(A register entry closed by "it turned out not to matter" is a
     legitimate closure; one closed silently is not.)*
     ⚠️ **The entry says "six"; it is FOUR literal sites** — `cascade.rs:537`, `:550`, `:564`, `:609`
     — two of them inside loops, giving **seven** constructions at runtime. Neither number is six.
     Correct the count in the same annotation, because a register entry is where counts are supposed
     to be right.
   - **The two `NoMatch` entries in the 4.6a and 4.7a sections** (`grep` locates them; they carry the
     ↺ annotations story 5.4 appended) — ✅ **the half they have been waiting for since story 4.6a is
     CLOSED**: `decide` is the function that chooses which side of D13's `NoMatch` an input falls on,
     and AC2's mapping is the answer. Annotate both; **strike neither** — the `Outcome` mapping they
     also mention still has no producer and belongs to **5.7**.
   - A new `## Deferred from: story-5.4b` section is **appended at the END** of the file (it is
     **chronological, not topical**), carrying **these items and no others** — gathered here because
     they are scattered across the ACs above and a section nobody enumerates cannot be counted:
     1. `#[must_use]` not added to `decide` (AC1) — *Owner: whoever revisits the workspace's
        `must_use` policy* — a CONDITION, not a story;
     2. `Abstained { Ambiguous }` does not record WHICH of D13's three rows produced it (AC2) —
        **Owner: story 5.14**;
     3. the lexicographic tiebreak is a placeholder for a designed rule priority (AC4) — **Owner:
        story 5.5** for L1 and **Epic 6** for `l2-*` — this is the item that names an EPIC;
     4. the float gate's documented limits: block comment = false positive, string literal = false
        negative, `#[doc = "…"]` unstripped (AC6) — *Owner: whoever meets one of them on a real tree*
        — a CONDITION;
     5. the gate scopes to `identity/` and not the workspace (AC6) — **Owner: story 5.14**;
     6. no milli-unit type, constant or field exists (AC7) — **Owner: story 5.14**;
     7. the incoherent `Decision` remains buildable by struct literal outside `decide` (AC8) —
        **Owner: story 5.9**;
     8. refusing a duplicated `RuleId` needs a producer (AC8) — **Owner: story 5.5**;
     9. D13's table is short one row in `architecture.md` itself (AC3) — **Owner: a milestone edit**,
        carrying the GitHub issue number AC3 requires — a CONDITION with an issue behind it.
     **Then count them mechanically after the last edit and state the split** — how many name a
     story, how many name an epic, how many name a condition. *(Story 5.4 shipped "thirteen name a
     story" when one of the thirteen named an epic, in the sentence forbidding exactly that; its
     review caught it. Item 3 above is this story's epic — do not let it be counted as a story.)*
   - ⚠️ **Cite entries by TITLE, not by line number.** Story 5.4 wrote stale citations into `deferred-work.md`
     at eight sites using pre-commit numbering that its own +21-line insertion invalidated; three
     review layers converged on it. This story appends to the same file. **A line citation written
     here will rot the same way.**

9. **AC9 — the gate is green, the docs are current, and the flow stops at the PR.**
   `cargo fmt --all` clean · `cargo clippy --workspace --locked --all-targets -- -D warnings` clean ·
   **`cargo clippy --workspace --locked -- -D warnings` clean** (the CI form, without `--all-targets`
   — the only invocation that catches an import kept alive solely by a test module or a `///` link) ·
   `cargo test --workspace --locked` green with the three per-crate counts **re-measured on the final
   tree** · `cargo xtask ci` green **with the new gate listed** (`ℹ views-hash STALE`, exit 0, is
   correct and **must not be regenerated in a story**) · `git status` under `fixtures/` empty ·
   `sprint-status.yaml`, `docs/project-context.md` and `CLAUDE.md`'s Epic 5 sentence updated ·
   `epics.md` **verified only — an edit there is a finding, not a task** ·
   **branch → PR → green CI. The story ends at status `review` and the PR open.** The merge is a
   separate act and it is what makes a story `done` in this project (5.1, 5.2, 5.2b, 5.3, 5.4); the
   `code-review` workflow's own default of setting `done` at the end of the review is WRONG here.

## Tasks / Subtasks

- [x] **Task 1 — Read before writing** (AC1–AC6)
  - [x] `crates/opencmdb-core/src/identity/cascade.rs` **in full** (690 lines, first `#[cfg(test)]` at
        `:457`). Specifically: the module doc and its four-judgement table (`:1-30`), `Verdict` and its
        five variant docs (`:39-105`), `Verdict::all()` and its witness (`:107-150`), `RuleVerdict`
        (`:174-212`), `RulesetVersion` (`:214-232`), `Conclusion` (`:234-296`), `Decision` and
        `Decision::rule()` (`:298-330`), `IdentityAbstentionCause` (`:377-397`) and its `all()` and witness
        (`:399-436`), and **the test module's placement convention** — *a test lives with the item
        whose CLAIM it pins* — which decides where every test in this story goes.
  - [x] `crates/opencmdb-core/src/trap.rs:31-45` — `RuleId`, its corrected doc, and **its derives at
        `:39` including `Ord`**, which AC4 depends on and which must not be re-derived.
  - [x] `crates/opencmdb-core/src/identity/mod.rs` (23 lines) — it says *"story 5.4b writes that
        algebra"*. After this story that sentence is false and Task 6 rewrites it.
  - [x] `xtask/src/main.rs`: the module doc listing the gates (`:1-40`), `main()`'s gate block
        (`run_ci()`'s, `:138-166` — NOT `main()`'s, which only dispatches), `fn report` and its
        `✅`/`🔴` glyphs (`:177-179`), `gate_ddl_collation` (`:289-327`), `gate_vocabulary`
        (`:384-441`),
        `gate_file_size` (`:87-136`), the gate tests at `:904-1003`, and **the temp-tree helper
        `fn scratch(tag: &str) -> PathBuf` (`:1088`)** with the fixture-gate tests that use it
        (`:1330`, `:1404`) — that, not the two string-helper tests, is the idiom for testing a gate
        that WALKS a directory. **Copy the idiom, including
        how a gate reports its offender.**
  - [x] `architecture.md` **D13** (`:929-1011`) — start from the Decision Index near the top (F56),
        not from a grep. Read **D20** (`:1348-1399`) for the ordinal clause, and **D14**
        (`:1013-1049`) for why the version is a parameter.
  - [x] `_bmad-output/implementation-artifacts/deferred-work.md` — **run `grep -n '5\.4b'` on it and
        read every hit.** There are **eight** entries across **four** sections (AC8 lists them), not
        six in one. **Locate them by title; the line numbers in this file will have moved by the time
        you read it.**
  - [x] Story 5.4's file, `### Review Findings` section — the 22 patches, and especially the three
        findings handed forward to this story.

- [x] **Task 2 — `decide`** (AC1, AC2, AC3, AC4)
  - [x] The signature of AC1, in `cascade.rs`, below `Decision`'s `impl` block.
  - [x] A `///` doc that: states the D13 contract and attributes it; carries the input-class
        enumeration of *The finding this story FIXES* so the gap is re-derivable; names the
        arbitration as Guy's and dates it; states the tiebreak IS a tiebreak with no semantic content
        and names who replaces it; and says **what `decide` does not do** — it does not refuse a
        duplicated rule, and it does not validate the version.
  - [x] 🚨 **A `match` on the presence tuple `(has_disqualifying, has_decisive, has_supports,
        has_opposes)` — NOT an `if`/`else if` chain.** AC2 carries the measurement behind this; it is
        what makes Task 5's M2 an `error[E0004]` instead of a silent no-op.
  - [x] Arms in D13's order with the `Disqualifying = true` arms **first** (binding the other three
        with `_`), each citing its architecture line, and the arbitration arm labelled as such and
        citing no line.
  - [x] The rule choice: lexicographic min `RuleId` **among the verdicts that QUALIFIED FOR THAT ARM**
        — the `Disqualifying` ones for the refusal, the `Decisive` ones for the match. **Not the min
        over the whole vector** (AC5 carries the failing case). **No new derive.**
  - [x] ⚠️ **No `_` catch-all arm that swallows an unhandled class.** Every one of the sixteen presence
        tuples is written out or covered by an arm a reader can check against the table. A wildcard
        would restore exactly the hole this story exists to close.

- [x] **Task 3 — The tests** (AC5, AC4) — inline in `cascade.rs`'s trailing `#[cfg(test)] mod tests`
      (D56b, one per file), placed by the convention that module already states.
  - [x] **The 32-subset totality walk.** Bitmask `0..32` over `Verdict::all()`; one `RuleVerdict` per
        present verdict with a distinct `RuleId`; assert the conclusion against an **independently
        written** expectation function. Assertion message names the subset.
  - [x] **The second oracle is written from D13's table, not from `decide`.** Deliberate redundancy —
        label it as such in a comment so a later DRY pass does not collapse it (CLAUDE.md protects
        exactly this).
  - [x] **The uncovered class, named as its own test** — `≥1 Opposes` alone → `Abstained {
        AbsenceOfProof }`. It is inside the 32-walk already; it gets its own named test anyway,
        because it is the one answer no architecture line backs and a reader must find it by name.
  - [x] **Order independence**: three verdicts **all carrying the SAME rule-naming verdict** (three
        `Disqualifying`, or three `Decisive` with no `Opposes`), with `RuleId`s supplied in an order
        that differs from their lexicographic order (e.g. `"c", "a", "b"`), **all six permutations**,
        same `conclusion` — asserted as `Conclusion::NoMatch { rule: rule("a") }`, the rule spelled
        out. ⚠️ Assert on `.conclusion`, **not** on the whole `Decision`. ⚠️ **A mixed vector whose
        conclusion is an `Abstained` makes this test VACUOUS** — an abstention names no rule, so no
        tiebreak can change it. Both traps are in AC4.
  - [x] **A named rule must have qualified**: `a_disqualifying_names_the_disqualifying_rule_not_the_smallest_one`
        on `[("a", Decisive), ("z", Disqualifying)]` → `NoMatch { rule: rule("z") }` (AC5).
  - [x] **Coherence**: for every subset whose conclusion names a rule, that `RuleId` is present in the
        returned `verdict_vector`. And `decide(Vec::new(), _)` → `Abstained { AbsenceOfProof }`,
        never `Match`.
  - [x] **Totality under a duplicated `RuleId`**: use `("a", Decisive)` + `("a", Opposes)` — the
        register's own example, and the pair that shows ONE rule fabricating D13's conflict row, so
        the expected conclusion is `Abstained { Ambiguous }`. *(`("a", Neutral)` + `("a", Supports)`
        is deterministic too and shows nothing — name the pair, do not leave it to taste.)* The test's
        doc says this pins TOTALITY, not refusal.
  - [x] ⚠️ **Minting an `ObsId` in a test: `ObsId::from_uuid(Uuid::from_u128(n))` with distinct `n`**
        — the crate's idiom (`trap.rs:416-418`), **already present in the very test module you are
        writing in, as `fn obs(n: u128) -> ObsId` at `cascade.rs:463-465` with `use uuid::Uuid;` at
        `:461`. REUSE it; re-adding it is the accidental duplication DRY forbids**. **`Uuid::new_v4()` does NOT compile**:
        `opencmdb-core` builds `uuid` with `features = ["v7", "serde"]`, so it yields `error[E0599]`.
        Evidence may be empty here — a `Neutral` legitimately has none — so prefer empty over
        inventing observations the algebra never reads.

- [x] **Task 4 — The float gate** (AC6) — `xtask/src/main.rs`
  - [x] `fn gate_float_free(root: &Path) -> Result<(bool, String)>` in the idiom of its two
        neighbours, **walking `crates/opencmdb-core/src/identity/` RECURSIVELY** (`walkdir`, as
        `gate_ddl_collation` does — already an `xtask` dependency), **stripping from the first `//`
        to end of line before matching**, and matching **both** a word-bounded `f32`/`f64` **and a
        float LITERAL** (`[0-9]+\.[0-9]`, or an `f32`/`f64` suffix on a numeric literal). AC6 carries
        the measured table showing why the word-boundary match alone misses `0.85f64` and
        `let confidence = 0.85;`.
  - [x] **Fail CLOSED if the directory is absent** — the fixture gate's precedent, not the DDL gate's.
        AC6 says why.
  - [x] Register it in **`run_ci()`** as `g5` and print it through `report`. ⚠️ **The module doc's gate
        list at `:1-21` is ALREADY missing `file-size`** — it names frontier, ddl-collation,
        vocabulary, fixtures and views-hash only. **Add BOTH bullets**, so the enumeration matches the
        six gates `run_ci()` prints. *"A doc that enumerates is a claim"*, and this one is already
        false before you touch it.
  - [x] Inline `#[cfg(test)]` tests at both levels (AC6): the line helper, **and `gate_float_free`
        itself against a `scratch(tag)` temp tree** — float in code, float in a comment, a nested
        subdirectory, and the directory absent.
  - [x] Verify no `#[doc = "…"]` attribute and no `/* */` block comment under `identity/` carries a
        float token before claiming the gate is clean on the real tree (both measured absent at
        contexting — re-verify, do not assume).
        `grep -rn '#\[doc\|/\*' crates/opencmdb-core/src/identity/`.

- [x] **Task 5 — Prove to red** (AC5, AC6; house rule, story 1.3). Run each, **quote the observed
      failure**, restore, re-run green. These are predictions to check against, **not a licence to
      skip running**: if the observed set differs, the DIFFERENCE is the finding and it goes in the
      Completion Notes. **A mutation with several reds is expected and is not a defect** — a record
      naming one red where four fired is the under-reporting this project's reviews keep catching.
  - [x] **M1 — swap one arm's CAUSE**: make the `Supports AND Opposes` row return
        `Abstained { AbsenceOfProof }` instead of `Abstained { Ambiguous }`. A one-token edit that
        needs no rule and reds purely by ASSERTION. Predicted: **2 of 32** subsets. **Name how many
        fired**, not just that it failed — that count is the walk's coverage, measured.
        ⚠️ *(Do NOT mutate this arm to `Match`: `Conclusion::Match` requires a `rule` and that arm has
        no `Decisive` to name, so the "mutation" becomes a rewrite and measures nothing clean.)*
  - [x] **M2 — delete the arbitration arm** for the uncovered class. Predicted: **`error[E0004]`,
        non-exhaustive `match`** — and that prediction holds **only because AC2 binds the arms to a
        `match` on the presence tuple**. ⚠️ **If you built an `if`/`else if` chain instead, this
        mutation is a silent no-op** (measured at validation: *"0 of 16 input classes changed
        answer"*) and the story's central claim ships unproven. **If M2 does not red, that is a
        finding about YOUR construct, not about the prediction** — go back to AC2.
  - [x] **M3 — replace the lexicographic tiebreak with "first in the vector"**. Predicted: the
        permutation test reds and the 32-walk does NOT (single-qualifier subsets are unaffected).
        **This is the mutation that proves the permutation test earns its place**; if the 32-walk also
        reds, say so.
  - [x] **M4 — remove the comment-stripping from the gate.** Predicted: the gate reds on the committed
        `cascade.rs:52` citation, and `cargo xtask ci` fails on a clean tree. Quote it.
  - [x] **M5 — a real float under `identity/`.** Predicted: the gate reds and names the file and line.
  - [x] ⚠️ **Say which reds are carried by the COMPILER and which by an ASSERTION.** Story 5.4's
        record claimed two of four were assertion-carried when only one was, and its review measured
        it. Classify honestly: a red that would fire on a test body of `assert_eq!(1, 1)` is
        compiler-carried.
  - [x] ⚠️ **Restore after every mutation and verify with `git status` before the next one.** Local
        flakiness (issue #38) and a forgotten revert look identical.

- [x] **Task 6 — Docs that this story falsifies** (AC2, AC9) — all doc-only; **re-read every rewritten
      sentence against the FINAL tree after the last edit**.
  - [x] `identity/mod.rs:10-11` — *"nothing combines a verdict set into a conclusion: story 5.4b
        writes that algebra"*. After this story it does. One sentence, naming what is still absent
        (no rule, no producer, no join — 5.5).
  - [x] `cascade.rs`'s module doc (`:1-13`) — it says *"It does not hold the algebra… That is story
        5.4b's"*. Same correction.
  - [x] `Verdict`'s doc (`:39-77`) — *"the six-row table… is implemented by story 5.4b, and no rule
        produces a `Verdict` until story 5.5"*. The first half is now false; the second is still true.
        ⚠️ The paragraph naming the uncovered class must now point at `decide` for the answer rather
        than saying 5.4b arbitrates it.
  - [x] `Conclusion::NoMatch`'s doc — it attributes the choice of side to 5.4b. It is made now.
  - [x] 🚨 **`IdentityAbstentionCause::AbsenceOfProof`'s doc (`cascade.rs:389-390`) becomes FALSE.** It
        reads *"Nothing in the verdict set argues either way — the row `only Neutral / nothing →
        NoMatch (absence of proof)`"*. AC3 routes the arbitration class onto that same variant, and an
        `Opposes` **does** argue — against. Rewrite it to name **BOTH** producing classes: D13's
        `only Neutral / nothing` row, and the arbitrated `Opposes`-alone class, where nothing argues
        **FOR** the merge. *(Found by the gap-hunt agent. Shipping the story without this ships a doc
        comment falsified by the function it documents.)*
  - [x] `Decision`'s doc (`cascade.rs:287-297`) — *"# What is representable and not refused… Nothing
        refuses either… Registered, owner story 5.4b"* and *"Story 5.4b **adds** the `cargo xtask ci`
        gate"*. Both are now shipped, not future.
  - [x] `RuleVerdict`'s doc (`cascade.rs:160-166`) — *"the two validations… Both need story 5.4b or
        5.5 to have something that could red"*. One of the two now has a place that reds.
  - [x] `IdentityAbstentionCause`'s type doc (`cascade.rs:331-345`) — *"Six rows, four of them
        abstentions, two variants"*. There are seven classes now, not six.
  - [x] The test module's doc (`cascade.rs:438-456`) — it is an explicit INVENTORY of which tests live
        there and why; Task 3 adds several.
  - [x] `xtask/src/main.rs`'s module doc — the gate list (**already missing `file-size`**, Task 4).
  - [x] 🚨 **The list above is the MINIMUM, not the set. Before counting, run
        `grep -rn '5\.4b' crates/ xtask/` on the FINAL tree and rewrite every hit** — every stale
        sentence names 5.4b, so that grep is exact. **Then count the doc locations and state the
        number ONCE.** *"A count in a doc is a claim"* — story 5.4 shipped "five doc blocks" where six
        changed, and its own review caught it. **This story's first draft listed five where eleven
        were falsified**, which is the same defect one level up; the validation pass caught it.

- [x] **Task 7 — The register** (AC8) — append-and-strike, never rewrite a bullet. Cite by TITLE.
  - [x] **`grep -n '5\.4b' deferred-work.md` FIRST**, then annotate all **eight** entries across the
        **four** sections per AC8, each saying which half moved and which did not — including the two
        `NoMatch` entries in the 4.6a and 4.7a sections, whose annotations still say nothing decides
        which side an input falls on.
  - [x] Check the empty-vector-literals prediction and record whether it held.
  - [x] Open `## Deferred from: story-5.4b` at the END with its items; **count them mechanically after
        the last edit** and state the story/epic/condition split.
  - [x] **Open the GitHub issue for D13's milestone correction** (AC3) and reference its number in the
        register entry and the PR.

- [x] **Task 8 — The full local gate, run WHOLE** (AC9; mirrors CI — Epic 3's retrospective recorded
      four CI-only failures from skipping exactly this)
  - [x] `cargo fmt --all` · `cargo clippy --workspace --locked --all-targets -- -D warnings` ·
        **`cargo clippy --workspace --locked -- -D warnings`** (the CI form) ·
        `cargo test --workspace --locked` · `cargo xtask ci`. **`--locked` everywhere.**
  - [x] Report the test count as three numbers (bin + core + xtask). **Baseline on `3d63544`:
        135 + 94 + 42 = 271, zero failures.** `core` and `xtask` should both move.
  - [x] `cargo xtask ci` must now print **six** gates plus the informational `views-hash`.
  - [x] `git status` under `fixtures/` **empty**; `MANIFEST.toml` untouched.
  - [x] Re-measure `cascade.rs` and `xtask/src/main.rs` code lines on the final tree (baselines:
        **456** and **884**, ceiling 2000).

- [x] **Task 9 — Docs current before push** (AC9; project rule)
  - [x] `sprint-status.yaml` — the `5-4b-…` entry and its narrative block.
  - [x] `docs/project-context.md` — the Epic 5 line and the test count.
  - [x] `CLAUDE.md` — the Epic 5 sentence. **Not conditional**: this story changes what "the engine
        proper" has shipped, so the sentence naming 5.4 as the last engine story goes stale on merge.
  - [x] `epics.md` — **verify only, do not edit.** Story 5.4b is present at `:1445`, `:22` and `:1313`
        read sixteen, `:1317`'s build order names 5.4b as its own step. **An edit here is a finding.**
  - [x] No manual, README or gh-pages change is expected: this story ships nothing a user can see.

- [x] **Task 10 — Branch → PR → green CI** (AC9). ⚠️ **Only after PR #52 is merged** (see the
      prerequisite at the top). Branch `story-5.4b-verdict-algebra-and-float-gate`. **The story ends
      at status `review` with the PR open and CI green.** The merge is a separate act.

### Review Findings (AI, 2026-07-30)

Three parallel layers on `master...HEAD` (9 files, +1150/−104; ~780 lines of code in `cascade.rs`
and `xtask/src/main.rs`): **Blind Hunter** (code diff only, no project access), **Edge Case Hunter**
(diff + repo; it extracted `line_has_float`/`has_float_literal`/`contains_word` into a standalone
binary and **executed** the cases, so its float-matcher verdicts are measured), **Acceptance
Auditor** (diff + this story + `project-context.md` + `CLAUDE.md`; it ran `cargo test --workspace`,
both clippy forms, `cargo xtask ci` and `gh`).

**1 decision-needed (resolved) · 22 actionable patch · 5 defer · 3 dismissed.** The patch section
carries 25 bullets, three of which are struck as `— absorbed` by the tokeniser item that came out of
the resolved decision; 25 − 3 = 22 is the number to act on. _(This line first said 23, a figure
carried over from before the decision was resolved and not recounted — corrected by counting the
bullets in this file, which is the same defect five of the findings below report.)_

Every count in the findings was re-measured by the orchestrating reviewer before being written here;
three layer claims were **refuted** in that pass and are recorded at the end rather than dropped
silently.

#### Decision needed — RESOLVED 2026-07-30 (Guy)

**Resolution: rewrite the matcher as a numeric-literal tokeniser** (option (c), expanded). A
delimited numeric literal, exactly one dot, not preceded by `.` or an identifier character; plus the
exponent and trailing-dot forms; plus the `f32`/`f64` suffix recognised **on the literal** rather
than as a bare substring. This closes three measured false positives *and* two measured false
negatives in one pass, and it absorbs three of the patch items below (marked `— absorbed`). No
escape-hatch comment and no `#[cfg(test)]` skip is added: the divergence with `file-size` stays, and
is now stated in the gate's doc rather than left implicit.

- [x] [Review][Decision] **The gate's false positives have no escape hatch, and story 5.5 is the
      first story likely to trip them** — Measured by execution: `fn a_f64_never_decides() {}` reds
      (labelled `"f32/f64 literal suffix"`, a wrong diagnosis), `let a = t.0.1;` reds (nested tuple
      field access, valid Rust, and the gate's doc at `xtask/src/main.rs:356` explicitly claims
      `x.0` is excluded), and a dotted-quad in a string literal reds — the last one IS documented as
      a known false positive on the grounds that "none exists under the guarded subtree today".
      5.5 writes the first L1 MAC/IP rule and its tests **under `identity/`**, where an IP literal
      or a test named after the rule is near-certain. There is no `#[allow]`, no `// gate: ok`, no
      allowlist, and no `#[cfg(test)]` skip — while the sibling `file-size` gate deliberately stops
      at the first `#[cfg(test)]` (`code_line_count`, `xtask/src/main.rs:80-88`), a divergence stated
      in neither gate's doc. The remedy is a genuine choice: (a) skip `#[cfg(test)]` like `file-size`,
      (b) an explicit allowlist/escape comment, (c) narrow the matcher (require a non-digit before
      the first digit, so `192.168.0.1` and `t.0.1` stop matching), (d) accept the friction and let
      5.5 work around it. Guy's call — the wrong choice here is the one that gets the gate weakened
      or deleted in its second week.

#### Patch

- [ ] [Review][Patch] **Rewrite `line_has_float`/`has_float_literal` as a numeric-literal
      tokeniser** (from the resolved decision above). Required behaviour, all five cases measured on
      the current tree: `"192.168.0.1"` → green (three dots), `t.0.1` → green (preceded by `.`),
      `fn a_f64_never_decides()` → green (not a literal), `1e-3` / `2E10` / `1.` → RED, `0.85` /
      `0.85f64` / `1f32` → RED. Recognise `f32`/`f64` as a word (type position) or as a suffix **on a
      literal**, and drop the bare `code.contains(token)` fallback that currently makes the
      word-boundary branch decorative. Add `f16`/`f128` to the token list while the matcher is open.
      Then rewrite the doc's "Known limits" list to match what the new matcher does — it currently
      states four limits, all about comments and strings, and its "three shapes" paragraph describes
      a gate this file does not implement [xtask/src/main.rs:335-370, doc at :312-326]
- [ ] [Review][Patch] ~~Two legal Rust float spellings are invisible to the gate~~ — **absorbed** by
      the tokeniser item above. Kept for the record because the measurement is the reason the
      tokeniser was chosen: `1e-3`, `2E10` and
      `1.` all measured as `None`; exponent form carries no `.` and trailing-dot has no digit after
      it, and both are `f64` by inference, i.e. exactly the "weight through the back door" shape AC6
      names as the gate's whole purpose. The doc's "Known limits" list is also incomplete: it states
      four limits, all about comments and strings, and neither literal form appears
      [xtask/src/main.rs:357-365, limits list at :319-326]
- [ ] [Review][Patch] A shipped doc comment asserts **47** offenders; the committed tree gives **45**
      — re-measured twice independently (a Python replication of the three matcher fns with stripping
      removed, and a raw grep): 44 in `cascade.rs` + 1 in `mod.rs` = 45, and 0 with stripping active.
      It was 47 at the WIP commit `1ced9e2` and was never re-measured after Task 6's doc pass
      shortened two prose lines — the story's own inherited lesson #1, committed by the story that
      quotes it. Stale in five places [xtask/src/main.rs:329; deferred-work.md:1311;
      sprint-status.yaml:730; this file :959 and :1039]
- [ ] [Review][Patch] `docs/project-context.md` was not brought current, yet Task 9's checkbox says
      it was — only the Epic 5 table row changed (`1 1` in `--numstat`). Left standing: the
      **271**-test count and the per-story inventory that never mentions 5.4b's nine tests
      [docs/project-context.md:62-64], and a sentence this branch falsifies — "no algebra, no rule
      and no producer, the combining function being story 5.4b's" [docs/project-context.md:72].
      Correct to 280 (135 + 100 + 45) and rewrite `:72` [this file :672]
- [ ] [Review][Patch] A shipped test doc claims the citation is "the workspace's only `f64` token"
      — there are **22** workspace-wide (3 under `identity/`), and this story measured the
      contradiction itself at :929-931 and left the doc standing. Second half: because this story
      added `f32`/`f64` at `cascade.rs:290-291`, the test's premise guard
      `assert!(source.contains("f64"), "the citation this gate must tolerate has moved or gone…")`
      now stays **green** when the D13 citation is deleted — the guard no longer detects what its
      message claims. Re-pin it to the citation text, not the substring
      [xtask/src/main.rs:1731 and :1743-1747]
- [ ] [Review][Patch] The register carries **TEN** 5.4b annotations, not eight — `grep -c 'by story
      5.4b, 2026-07-29'` → 10, at lines 253, 272, 365, 377, 1186, 1252, 1287, 1308, 1328, 1354. The
      same sentence that says "Eight" then enumerates ten (5 named + "the four `NoMatch` annotation
      lines" + the empty-vector entry). The 4.6a/4.7a sections carry **two** 5.4b-owned bullets each,
      not one. Stated as eight in three places [this file :982 and :1039; sprint-status.yaml:719]
- [ ] [Review][Patch] The File List claims the re-export cost "is recorded on the existing
      re-export entry"; that entry is untouched by this diff and still reads "grew by **five** names
      with no consumer", enumerating five — while `lib.rs` now re-exports a sixth un-consumed name,
      `decide`. The story's own binding Dev Note (:791-795) said to add the cost to the existing
      entry. Either annotate it or weaken the File List to say the cost is NOT recorded
      [deferred-work.md:1266-1271; this file :1012-1014]
- [ ] [Review][Patch] The `//`-truncation false negative is real and its stated justification is
      false — measured: `let url = "http://x"; let confidence = 0.85;`, `let s = "a // b"; let c =
      0.85;` and `/* see http://x */ let c = 0.85;` all return `None`. The doc calls this "harmless:
      a float inside a string is not a float", but the missed float is **not** inside the string, it
      is real code after it. Under this project's rule that a checkable comment gets checked, the
      false reassurance is the defect: a reader who consults the limits list concludes this case
      cannot hurt [xtask/src/main.rs:336-339, doc at :322-323]
- [ ] [Review][Patch] The last row of `decide`'s own doc table names `NoMatch` where `decide`
      returns `Abstained { AbsenceOfProof }` — the column is headed `conclusion` and the doc
      explicitly invites re-derivation ("so a reader can re-derive the gap rather than trust it"),
      so the row reads as the returned variant. It is a faithful quote of D13's row `:974` sitting in
      a column that means something else, and it contradicts the `NoMatch` variant's own doc 115
      lines above, which gets it exactly right ("has **no rule to name**, so it cannot be
      represented by this variant and becomes `Abstained`"). Split the column, or mark the row as
      D13's letter vs the engine's answer [cascade.rs:363 vs :453-455; correct text at :248-255]
- [ ] [Review][Patch] AC1's prescribed construct was not used, and it left two dead arms that answer
      "absence of proof" — AC1 required the presence test and the selection to be **one** act (`if
      let Some(rule) = …`), so that "no arm ever holds an `Option` it must prove `Some`". The
      presence test stayed in the tuple and the selection is a second act bridged by a fabricated
      `None` arm. Consequence: if `has(w)` and `smallest_rule_with` ever diverge, a `Disqualifying`
      refusal degrades into an honest-looking `Abstained` — the one answer D13/D18 care most about
      not fabricating — with nothing observable. No test can red on these arms, and the oracle takes
      the **opposite** decision on the same branch (`unreachable!()`), so implementation and oracle
      differ on a path neither exercises. The auditor supplied a compatible construct: match on
      `(smallest_rule_with(…Disqualifying), smallest_rule_with(…Decisive), has(Supports),
      has(Opposes))`, which is still exhaustiveness-checked, so M2 still yields `error[E0004]` and no
      arm holds an undischarged `Option` [cascade.rs:410-419 and :421-427; oracle at :892, :898]
- [ ] [Review][Patch] A sixth `Verdict` variant would silently halve the totality test's coverage —
      `decide`'s presence tuple stays 4-wide, so **no `error[E0004]` fires in `decide`** on variant
      addition (only on arm deletion), and the new verdict is treated exactly like `Neutral`.
      `subset()` is driven by a hard-coded `0..32`, so bit 5 is never set, `NAMES[5]` is never
      reached and nothing panics or reds; the test keeps the name
      `decide_is_total_over_every_one_of_d13s_input_classes` while checking 32 of 64 classes. Derive
      the bound from `Verdict::all().len()` and generate the names
      [cascade.rs:402-407, :925-937, :958]
- [ ] [Review][Patch] The gate greens on a guarded directory that exists but holds no `.rs` file —
      `checked` is computed and printed but never asserted `> 0`, so relocating `cascade.rs` to a new
      module while leaving `identity/` in place yields `no float in code across 0 file(s)` and a
      pass. `dir.exists()` is the only liveness check (and is true for a *file* named `identity`).
      AC6 demanded fail-closed only for the missing directory, so this is a residue rather than a
      violation — but the gate's doc claims the stronger property [xtask/src/main.rs:412-416,
      `checked` at :403, fail-closed block at :382-389]
- [ ] [Review][Patch] "TWELVE doc locations corrected" is not reproducible by any consistent method
      — the stated method (`grep -rn '5\.4b' crates/ xtask/` on the final tree) yields 10 rewritten
      mentions, or 9 doc blocks if two adjacent lines merge; counting doc *blocks* consistently gives
      **10** (8 in `cascade.rs` — which the File List itself states — plus `mod.rs` plus `xtask`);
      counting sentences gives 13+. Twelve is reachable only by excluding the `xtask` module doc,
      which Task 6 lists as a doc location and the File List records changing [this file :973, :639,
      :1010, :1015-1017]
- [ ] [Review][Patch] "Three sentences naming 5.4b were left standing" — exactly **one** was: the
      pre-existing `#[non_exhaustive]` note (two lines). The other three of the five surviving
      mentions are sentences this story **wrote** [this file :978-980; cascade.rs:77-78, :618, :865;
      xtask/src/main.rs:330]
- [ ] [Review][Patch] The register cites `cascade.rs:52` for the D13 citation; it is at **:53** (the
      module-doc rewrite added a line above it) — and AC8 forbade line citations in the register for
      exactly this reason ("a line citation written here will rot the same way"). The Completion
      Notes get `:53` right, so the two records now contradict each other
      [deferred-work.md:1309 vs this file :931]
- [ ] [Review][Patch] Task 5 required the observed failure quoted for each mutation; only M2 is
      quoted verbatim, and **M5's count is arithmetically impossible** — `line_has_float` `return`s
      on the first matching shape and the gate pushes at most one offender per line, so the
      classifications "type" and "literal" are mutually exclusive per line and a single inserted
      float line cannot produce "2 reds, the type AND the literal". The mutation text was not
      recorded, so the number cannot be reproduced. Quote the output or weaken the claim
      [this file :947-971; xtask/src/main.rs:335-352]
- [ ] [Review][Patch] `subset()`'s doc justifies its scrambled `NAMES` by a property the test cannot
      have — it claims the names are ordered so "lexicographic order does NOT follow `Verdict::all()`'s
      order — otherwise 'smallest rule' and 'first in the vector' would be indistinguishable here",
      but because `subset` emits at most one `RuleVerdict` per verdict kind, no rule-naming arm ever
      has two candidates, so `min()` and "first in the vector" are indistinguishable **whatever
      `NAMES` contains**. The file admits this 30 lines later ("What it does NOT cover: the
      tiebreak"). Replacing `NAMES` with `["a".."e"]` changes nothing observable
      [cascade.rs:925-937 vs :400-402]
- [ ] [Review][Patch] The tiebreak's "no semantic content" claim is false in the direction that
      matters — `RuleId(pub String)` derives `Ord`, so `min()` is **byte** order. Given this
      project's `l1-*`/`l2-*` naming, any decision where an L1 and an L2 rule both disqualify always
      names the L1 one, forever: a stable tier preference, not the absence of one. It is also
      case-sensitive (`"L1-exact"` beats `"l1-a"`) and **flips tiers the moment rule numbering
      reaches ten** (`"l10-x"` < `"l2-x"`). The doc's intra-tier example (`l1-distinct-mac` vs
      `l1-exact-mac`) is the one case where the claim holds. Weaken the sentence
      [cascade.rs:470-476, doc at :373-375]
- [ ] [Review][Patch] `Decision`'s doc closes "merged, with no explanation" "by construction"
      without saying **which** emptiness it closed — `decide` never inspects `evidence`, so a
      single `RuleVerdict { verdict: Decisive, evidence: vec![] }` still yields a `Match` whose
      explanation explains nothing. The empty-*vector* case is genuinely closed; the empty-*evidence*
      case is not, and the story defers it elsewhere. Qualify the sentence
      [cascade.rs:98-103 and :421-427]
- [ ] [Review][Patch] ~~The word-boundary branch can never change the verdict, only the label~~ —
      **absorbed** by the tokeniser item; kept because it is the measurement behind it.
      `code.contains(token)` is a strict superset of `contains_word(code, token)`, so the real rule
      is "any substring occurrence in the non-comment part". The doc describes "three shapes" as if
      the first were load-bearing detection and says the last two "were measured as escapes of a
      word-boundary-only match" — true of a hypothetical gate, not of this one. Any identifier
      containing the substring reds and is mislabelled `"f32/f64 literal suffix"`, sending the reader
      to the wrong fix [xtask/src/main.rs:340-348, doc at :312-317]
- [ ] [Review][Patch] Offenders are emitted in walkdir traversal order, so the same tree can print
      RED lines in different orders on two machines — the sibling `file-size` gate sorts before
      formatting [xtask/src/main.rs:391, :407 vs gate_file_size at :129]
- [ ] [Review][Patch] The determinism half of `decide`'s permutation test cannot fail — calling a
      `fn` with no state twice and asserting equal outputs is guaranteed by the language, not by the
      code under test; no mutation of `decide`'s body can red it. It inflates the apparent coverage
      of the doc's "answers the same way every time" claim [cascade.rs:587-601 region]
- [ ] [Review][Patch] ~~`f16`/`f128` are absent from the token list~~ — **absorbed** by the
      tokeniser item. D13's
      clause is "no float" and the gate enumerates two of the four float type names. Unreachable on
      stable today, which is why this is a doc/limits item rather than a hole
      [xtask/src/main.rs:340]
- [ ] [Review][Patch] The new gate breaks the file's banner numbering — order is now `Gate 0`,
      `Gate 5`, `Gate 1`, `Gate 2`, `Gate 3`, the new gate having been inserted above `Gate 1` rather
      than after the two neighbours AC6 named. Cosmetic, no behaviour affected
      [xtask/src/main.rs:298 vs :192, :429, :493, :623]
- [ ] [Review][Patch] PR **#55** (open, `ci` SUCCESS) is recorded nowhere — not in the
      `sprint-status.yaml` block, not in this story, not in `CLAUDE.md` — while every predecessor is
      recorded as "PR #41 / #44 / #46 / #48 / #52" [sprint-status.yaml:764]

#### Deferred

- [x] [Review][Defer] Four `cargo xtask ci` gates swallow every `walkdir` error via
      `filter_map(Result::ok)`, so an unreadable subdirectory silently shrinks the tree they claim to
      have checked [xtask/src/main.rs:105, :395, :439, :557] — deferred, pre-existing. **The Edge
      layer rated this HIGH against 5.4b on the grounds that the same file forbids it by name at
      `corpus_entries` (:753-806, "a walk whose failure mode is 'quietly saw less of the tree' is not
      a gate"). Re-measurement moved it**: three of the four sites predate this story
      (`gate_file_size`, `gate_ddl_collation`, `gate_vocabulary`), and AC6 told 5.4b to follow "the
      idiom of its two neighbours" — which it did, exactly. The weakness is repo-wide and fixing one
      gate would be the least useful version of the fix.
- [x] [Review][Defer] The gates do not follow symlinks and do not report them, so a symlinked
      subdirectory (or a module pulled in by `#[path]`, or an `include!`) is outside every walk
      [xtask/src/main.rs:393 and the three sibling gates] — deferred, pre-existing and repo-wide.
      Asymmetric with `corpus_entries`, which refuses to skip a symlink in silence
      (`CorpusEntry::Symlink`).
- [x] [Review][Defer] Nothing refuses a blank or whitespace `RuleId` on the `RuleVerdict` side, so
      `decide` can return `NoMatch { rule: RuleId("") }` on which `Decision::rule()` still answers
      `Some` — "every decision names a rule" degenerates to naming nothing. `Trap::validate` refuses
      a blank rule on the expectation side [trap.rs:302] — deferred, owner story 5.5, which is where
      a `RuleVerdict` first gets produced by something other than a test.
- [x] [Review][Defer] A named rule with EMPTY `evidence` yields a `Match` that explains nothing,
      defeating D13's "the list IS the explanation" one level below the empty-vector case this story
      closed [cascade.rs:421-427] — deferred, owner story 5.5; the invariant needs a firing rule to
      state, as the story already records.
- [x] [Review][Defer] The `xtask` module doc's list of gates is not itself gated and will drift again
      — evidence: this story had to add the `file-size` entry, meaning the previous story shipped a
      gate absent from that list [xtask/src/main.rs:646-648 region] — deferred, pre-existing.

#### Dismissed, with the reason (three layer claims that did not survive re-measurement)

1. **"The gate test leaks its scratch tree on failure and poisons the next run"** (Blind) —
   **refuted.** `scratch()` calls `remove_dir_all` on entry *and* namespaces the path with
   `std::process::id()`, with a doc comment saying exactly why ("a shared constant path races
   between concurrent runs and leaves a stale corpus behind when an assertion fails")
   [xtask/src/main.rs:1228-1235].
2. **"A sixth `Verdict` variant makes `NAMES[i]` panic with an index-out-of-bounds"** (Blind) —
   **refuted by the Edge layer**, and the real defect is worse than the claimed one: the hard-coded
   `0..32` means bit 5 is never set, so nothing panics, nothing reds, and the test silently covers
   half the space. Carried as a patch item above in its corrected form.
3. **"Every 'Registered' claim is unverifiable" and "what the diff fails to make self-evident"**
   (Blind) — dismissed as blindness artifacts. The two layers with repo access confirmed the
   register entries exist, that GitHub issue #54 is open, `RuleId`'s `Ord` derive, `contains_word`,
   `walkdir`'s presence and `Verdict::all()`'s arity. The one durable observation inside that
   finding (the module-doc gate list drifting) is carried as a defer above.
   **Blind also claimed the diff still says "`NoMatch` is therefore reached by two different rows"**
   — the tree says something different and correct; only the doc *table* row is wrong.

## Dev Notes

### Why `decide` returns a `Decision` and not a bare `Conclusion`

This is the story's one real design decision, and it was taken because of a measured defect rather
than by taste.

Story 5.4's code review found that three documents named 5.4b owner of the invariant *"a
`Conclusion`'s rule must appear in its own `verdict_vector`, and a `Match` may not carry an empty
vector"* while `epics.md`'s 5.4b criteria never mentioned it — and, separately, that those criteria
never said what `decide` returns.

`fn decide(v: Vec<RuleVerdict>, ..) -> Conclusion` would leave the caller to assemble the `Decision`,
which puts the conclusion and the vector back in two hands and makes the invariant **unenforceable at
exactly the point the register says is the only place a test could red**. `fn decide(v, version) ->
Decision` takes ownership of the vector and hands it back inside the result, so:

- the returned `verdict_vector` **is** the input — a conclusion naming a rule absent from it is
  impossible, because the rule is *selected from* it;
- `decide(vec![], v)` falls into *"only `Neutral` / nothing"* and returns
  `Abstained { AbsenceOfProof }` — so a `Match` with an empty vector **cannot be produced by this
  function at all**, and AC5 tests that;
- `ruleset_version` is carried once, by the struct, exactly as story 5.4's *"Why `Decision` is a
  struct"* argued.

⚠️ **What this does NOT do, and the doc must say so:** `Decision`'s fields are `pub` with no
constructor, so a struct literal can still build the incoherent state by hand. The invariant holds
**for everything `decide` produces**, which is the whole surface today because nothing else produces
a `Decision`. AC8 moves the residual exposure to 5.9, the first story that builds one from
elsewhere.

### The one thing that would make this story fail review

**Writing a rule.** The table is written, the types are in front of you, and `l1-exact-mac` is
sitting in the corpus with a name. It is story **5.5**'s, and writing it here would give the L1 join
no story of its own, invent a rule against a corpus this story never reads, and repeat one story
later the exact mistake 5.3 and 5.4 each refused for the same reason.

**The second thing: patching `architecture.md`.** D13's table is short one row and you can prove it.
The correction is a milestone act with a GitHub issue, never a story edit — `epics.md:1461` says so
in the AC that created this story.

**The deliverable is one function, one gate, their documentation, and the tests that prove the
function total and the gate honest.**

### Where the tests go, and why it is already decided

`cascade.rs`'s test module states the convention story 5.4's review closed: *a test lives with the
item whose CLAIM it pins; the items it merely READS are dependencies, imported and not owned.* Every
test in Task 3 pins a claim about `decide`, which lives in `cascade.rs` — so they all go there, by
the convention rather than by fiat. The gate's tests pin a claim about `gate_float_free` and go in
`xtask/src/main.rs`. There is no judgement call left to make; if one appears, say so rather than
inventing an exception.

### What was measured, before the story was written

All on `3d63544` (the post-review story-5.4 branch), so the dev re-derives nothing and a surprise
reads as a **finding**:

- **`cargo test --workspace --locked` → 135 (bin) + 94 (core) + 42 (xtask) = 271, zero failures.**
- **`RuleId` derives `Ord`** (`trap.rs:39`) — AC4's tiebreak needs no new derive.
- **`IdentityAbstentionCause` has exactly TWO variants**, `Ambiguous` and `AbsenceOfProof`
  (`cascade.rs:378-396`) — D13's three `Ambiguous` rows collapse onto one, and that is AC2's
  registered consequence, not an oversight.
- **`cascade.rs` = 690 lines, first `#[cfg(test)]` at `:457` → 456 CODE lines**; ceiling 2000.
- **`xtask/src/main.rs` = 1484 lines, first `#[cfg(test)]` at `:885` → 884 CODE lines** — **the
  largest file in the workspace**, and the one `cargo xtask ci` names.
- **`grep -rn "\bf32\b\|\bf64\b" crates xtask --include=*.rs` → exactly ONE hit**, `cascade.rs:52`,
  inside a `///` quotation of D13. Zero on `master`. **This is AC6's negative test case and it is
  already committed.**
- **Six `deferred-work.md` entries name story 5.4b as owner**, all in `## Deferred from: story-5.4`.
- **No ITEM named `decide` or `gate_float_free` exists**: `grep -rn "fn decide\|-> Decision"` and
  `grep -rn "gate_float_free"` over `crates` and `xtask` both return **0**. ⚠️ The weaker true
  sentence, because the strong one is falsified by a plain grep: the WORD `decide` appears **21**
  times in prose and doc comments, and `milli` **15** times — all of them milliseconds, including a
  real field `Fact::Rtt { millis: u32 }` (`observation/mod.rs:172`), never a confidence unit. So no
  name this story takes is contested, but `grep decide` is not how you check that.

### Inherited from stories 5.1–5.4 and their reviews — read before writing a doc comment

1. **A check that its own commit falsifies is worse than no check.** Story 5.4 wrote stale line citations into
   `deferred-work.md` at **eight sites** (four distinct coordinates) that its own +21-line insertion
   invalidated; **three review
   layers converged on it**. This story appends to the same file. **Cite by title.**
2. **A count in a doc is a claim.** 5.4 shipped *"five tests"* where four exist, *"five doc blocks"*
   where six changed, and *"thirteen name a story"* where one of the thirteen named an epic. Count
   mechanically, after the last edit.
3. **A red set is a count too.** Report every red a mutation fires, not the first.
4. **Classify your reds honestly.** 5.4's record said two of four mutations were assertion-carried;
   measurement showed only one was. A red that fires on a test body of `assert_eq!(1, 1)` is the
   compiler's, not the test's.
5. **An inventory in a doc comment has no guard behind it.** Say what THIS function does and what
   THIS test proves; let the register count what is open.
6. **Name the test behind every claim.** The temptation here is *"the identity engine now decides"*.
   What will hold is: *"`decide` maps a verdict set to a `Decision` over all 32 subsets of `Verdict`,
   a permutation test pins that the named rule does not depend on arrival order, and a gate refuses a
   float in code under `identity/`. No rule produces a verdict and nothing calls `decide`."*

### What this touches, and what it must not break

- **`crates/opencmdb-core/src/identity/cascade.rs`** (UPDATE) — `decide`, its doc, the new tests, and
  four doc corrections. *Must be preserved:* all five types and their derives **unchanged**; both
  `all()` implementations with their witnesses and corrected docs; `Decision::rule()`'s exhaustive
  match with no `_` arm; the six existing tests; the test module's placement convention.
- **`crates/opencmdb-core/src/identity/mod.rs`** (UPDATE, doc only).
- **`xtask/src/main.rs`** (UPDATE) — one gate, its registration, its module-doc line, its tests.
  *Must be preserved:* the four existing gates and `check_views_hash`'s informational, exit-0
  behaviour.
- **`crates/opencmdb-core/src/lib.rs`** — ⚠️ **decide whether `decide` joins the flat re-export
  block** (`:40`). The block already carries **six** identity names with no consumer (`Conclusion`, `Decision`,
  `IdentityAbstentionCause`, `RuleVerdict`, `RulesetVersion`, `Verdict` — `lib.rs:40-42`); story 5.4
  added five of them and registered the growing cost. Following the idiom is the default; **whichever you choose, say why**, and if you
  re-export, add the cost to the existing entry rather than opening a new one.
- **`crates/opencmdb-core/src/score.rs`, `trap.rs`, `gap/mod.rs`, `crates/opencmdb-bin/` (all of it)**
  — **NOT touched.** No `From<Decision> for Outcome` (5.7), `VerdictVectorEntry` stays uninhabited.
- **Under `fixtures/`: NOTHING.** **`architecture.md`: NOTHING.** **`epics.md`: verify only.**
- **`deferred-work.md`, `sprint-status.yaml`, `docs/project-context.md`, `CLAUDE.md`** (UPDATE).

### What STOP means, procedurally

If a step appears to require re-authoring a committed artefact, re-hashing `MANIFEST.toml`, or
editing `architecture.md`: **stop, do not edit.** Record what was attempted, what the tree says, and
the exact command that shows the conflict; report it as a finding in the Completion Notes, **open a
GitHub issue on `guycorbaz/opencmdb`**, and raise it with Guy.

### Testing standards

Tests live inline in a trailing `#[cfg(test)] mod tests` (D56b, one per file). Test names are
sentences that say what they prove. The identity engine's unit tests are pure — *"the engine is a
pure function: a `FixtureConnector` and nothing else — no database"* [architecture.md:3302], and
*"the engine never touches the clock (D19)"* [architecture.md:3364]. Nothing in this story needs
either. ⚠️ **Local flakiness is a known unexplained condition (GitHub issue #38)** — a failure that
does not reproduce is reported, never smoothed, and the "Synology Drive" explanation for it was
**refuted by measurement**. No test here reads a database, so `DATABASE_URL` is irrelevant.

### House rules that bind this story

- **Prove-to-red is not optional** (story 1.3). Task 5 names five mutations.
- **Document every public item** — `decide` and the gate both carry `///`, in idiomatic rustdoc prose
  (never `@param`/`@return`). **A doc comment must be TRUE**; prefer the weaker true sentence.
- **DRY, with deliberate redundancy protected.** AC5's second oracle is the protected kind: a test
  that restates D13's table independently is what makes the walk a test rather than a mirror. Do not
  collapse it.
- **File size:** ≤ 2000 CODE lines, tests excluded. Both touched files are far under; `xtask/main.rs`
  is the workspace's largest at 884.
- **Dependency frontier (D47):** `opencmdb-core` must not gain `anyhow`, `axum`, `sqlx` or `askama`.
  Nothing here needs anything outside `std` plus the crate's existing types. `xtask` is a dependency
  of nobody.

### Project Structure Notes

`identity/` is the architecture's own layout [architecture.md:3365-3373], and `cascade.rs` is
described there as *"the verdict algebra. No float decides. (D13)"* (`:3369`) — **so this story is
the one that makes that line true, in the file it already names.** No new module is created. The
tree also names `index.rs` (`:3367`), `blocking.rs` (`:3368`, story 5.6's), `field_decision/`
(`:3370-3372`) and `migration.rs` (`:3373`) — none of which this story creates. `mod.rs` (`:3366`) is
described as holding `IdentityError`; there is still no fallible operation, since `decide` is total
and returns no `Result`, so it keeps its module doc and its `pub mod cascade;`. D54 stands unchanged:
the FOLDER is not the frontier — visibility is.

### Git intelligence

Last five commits: `3d63544` (**story 5.4's code review**, 22 patches, PR #52) · `ab0d723` (**story
5.4**, PR #52) · `505379e` (issue-#50 bookkeeping, PR #51) · `d4151d1` (5.3 bookkeeping, PR #49) ·
`62f9c83` (**story 5.3**, PR #48).

Story 5.4 is the immediate predecessor and **the tree it left was reshaped twice** — once by the
story and once by its review, which touched `cascade.rs` again for doc corrections and one added
assertion. That is why every line number in this file was measured on `3d63544` rather than on
`ab0d723` or `master`. Every one of those commits went branch → PR → green CI → squash merge; hold
the same, and stop at the PR (AC9).

### Toolchain

No new dependency, no version to research. `uuid` is already a dependency of `opencmdb-core` with
`features = ["v7", "serde"]`. `xtask` already has everything the gate needs (`anyhow`, `std::fs`) —
**adding a crate for a grep would be the disproportion D47 makes a decision.** Rust 1.96+, edition
2024, `Cargo.lock` committed, every build `--locked`. **Never invent a version** — pin from the real
`Cargo.lock` if one is ever needed.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 5.4b] — `:1445-1474`. All six of its
  acceptance criteria are implemented here; AC1, AC5 and AC8 additionally answer the three gaps story
  5.4's code review handed forward. Build order at `:1317`.
- [Source: _bmad-output/planning-artifacts/architecture.md#D13] — `:929-1011`: the `Verdict` enum
  (`:964`), the six-row table (`:967-974`), the refusal of `rule -> confidence: f64` (`:956-958`),
  *"explanation is free"* (`:977-978`), the order-accident quote (`:936-937`), floats-may-rank
  (`:988-990`), the milli-units corollary (`:991-993`), `Disqualifying` as a structural fact
  (`:995-1002`).
- [Source: _bmad-output/planning-artifacts/architecture.md#D14] — `:1013-1049`: `ruleset_version` is
  mandatory (`:1044-1045`).
- [Source: _bmad-output/planning-artifacts/architecture.md#D20] — `:1348-1399`: strength returns as
  an ORDINAL, never a weight (`:1374-1376`); the complete verdict vector as a data requirement
  (`:1396-1399`).
- [Source: crates/opencmdb-core/src/identity/cascade.rs] — the five types (`:39-327`),
  `IdentityAbstentionCause` (`:377-430`), the test module's placement convention (`:457+`).
- [Source: crates/opencmdb-core/src/trap.rs] — `RuleId` and its derives (`:31-45`), the `ObsId` test
  helper (`:416-418`), and `cascade.rs`'s own copy of it (`:463-465`).
- [Source: xtask/src/main.rs] — the gate idiom (`:87-136`, `:289-327`, `:384-441`), `run_ci()`'s gate
  block (`:138-166`), `fn report` (`:177-179`), the temp-tree helper `scratch(tag)` (`:1088`), the
  gate tests (`:904-1003`).
- [Source: _bmad-output/implementation-artifacts/5-4-decision-and-ruleset-version.md] — the five
  types' ACs, and the `### Review Findings` section carrying the three findings handed forward.
- [Source: _bmad-output/implementation-artifacts/deferred-work.md] — the **eight** entries naming
  5.4b, by title, across **four** sections (`story-4.6a` review, `story-4.7a`, `story-5.4`, and
  `code review of story-5.4`). Find them with `grep -n '5\.4b'`, not by line number.
- [Source: docs/project-context.md] — the grounding rules for this repository.

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context) — `claude-opus-5[1m]`.

### Debug Log References

⚠️ **One process defect, recorded because it destroyed the implementation once.** The first mutation
pass ran against an **uncommitted** baseline and used `git checkout <file>` to restore M1 — which
reverts to `HEAD`, not to the pre-mutation state, so `decide` and its six tests were wiped. The
backup `cp` had also failed silently (its target directory did not exist and the exit code was never
checked). The code was re-applied from scratch, then **committed (`1ced9e2`) before the pass was
re-run**, after which every restore was verified by `md5sum` against the committed baseline
(`3958dd4bd844f5582fae55b9454eea16`) and by `git status`.

The story's own warning — *"restore after every mutation and verify with `git status` before the next
one"* — named the symptom and not the mechanism. **A mutation pass needs a committed baseline to
restore TO**; that is the sentence worth carrying forward.

### Completion Notes List

**In the weaker true sentence:** `identity/cascade.rs` gains `decide(Vec<RuleVerdict>,
RulesetVersion) -> Decision` — a total pure function implementing D13's six rows plus the one input
class the table leaves uncovered — and its private helper `smallest_rule_with`. `xtask` gains a sixth
gate, `float-free`. Six new core tests and three new xtask tests pin their claims. **Nothing produces
a `Verdict`, so nothing calls `decide` outside its own tests**: no rule, no join, no blocker, no
persistence, no corpus wiring. No byte moved under `fixtures/`.

**Counts re-measured on the FINAL tree** (a count in a doc is a claim):

- **271 → 280 tests: 135 bin + 100 core + 45 xtask**, zero failures. `core` +6, `xtask` +3.
- `cascade.rs`: **1154 lines, first `#[cfg(test)]` at `:631` → 630 CODE lines**, ceiling 2000.
- `xtask/src/main.rs`: **1748 lines, first `#[cfg(test)]` at `:1027` → 1026 CODE lines**; still the
  workspace's largest, and `file-size` reports `largest: 1026`.
- `cargo doc --workspace --no-deps` → **the same three pre-existing warnings** (`ing`,
  `comparable_fields`, `ScoredRecord`); none of the new intra-doc links is broken.
- ⚠️ **`f32`/`f64` workspace-wide is no longer 1 — it is 22**, because the gate's own implementation
  and tests name the tokens they hunt. The number that matters is **under `identity/`: 3, all in
  `///` doc comments** (D13's citation at `:53`, and two lines describing the gate at `:290-291`),
  all stripped, **zero in code**. The story's Dev Notes predicted 1 there; the two extra are its own
  doc.

**The design decision, and what it buys.** `decide` returns a `Decision`, not a bare `Conclusion`, so
the returned `verdict_vector` **is** the input and the named rule is selected **from** it. That makes
*"a conclusion naming a rule absent from its own vector"* and *"a `Match` with an empty vector"*
unreachable **through the function** rather than merely unenforced — `a_named_rule_is_always_present_in_the_vector_it_travels_with`
walks all 32 subsets and asserts it. A struct literal built elsewhere is still unconstrained; that
residue moved to story 5.9.

**The arms are a `match` on the presence tuple, and it is load-bearing.** M2 gives
`error[E0004]: non-exhaustive patterns: (false, false, false, true) not covered` — **the compiler
names the missing class**. Validation had measured that an `if`-chain swallows the same deletion with
all 16 classes keeping their answer, which is why the construct is binding rather than stylistic.

**FIVE mutations run. Two predictions were corrected BY MEASUREMENT, and both are recorded rather
than smoothed:**

- **M1** (the conflict arm's cause) → **exactly 2 of 32** subsets, `01010` and `01110`. ⚠️ That count
  is only observable because the 32-walk was made **cumulative**: a bare `assert_eq!` inside the loop
  aborts on the first mismatch, so the first run reported one subset where two had moved. The loop
  now collects every mismatch and reports them together — the difference between *"M1 reds"* and
  *"M1 reds on exactly 2 of 32"*, which is the walk's coverage measured.
- **M2** (delete the arbitration arm) → `error[E0004]`, naming `(false, false, false, true)`.
- **M3** (`min()` → `next()`, i.e. "first in the vector") → **only** the permutation test reds; the
  32-walk stays green. This confirms validation's finding that the walk gives the tiebreak **zero**
  coverage and that the permutation test is its only guard.
- **M4** (remove the comment-stripping) → **47 offenders on the real tree, not 1.** The prediction was
  *"reds on the committed citation"*. The other 46 are **story references in prose** — `5.4b`,
  `4.6a`, `4.7a` are literally digit-dot-digit — because the gate also matches **bare float
  literals**, `let confidence = 0.85;` carrying no `f32`/`f64` token at all. All three xtask gate
  tests red too, not just the one the story named. **The stripping and the literal rule are
  load-bearing together**, and the gate's doc now says so with the number.
- **M5** (a real float in code under `identity/`) → **2** reds naming file and line, the type AND the
  literal, where the story predicted one.

⚠️ **ONE of the five reds is compiler-carried, four are not.** M2 is `error[E0004]` and would fire on
a test body of `assert_eq!(1, 1)`. M1 and M3 red through assertions; M4 reds through the gate's three
xtask tests. **M5 was observed through `cargo xtask ci`'s own output rather than a test run** — that
is the gate reporting, not a test asserting, and saying so is the honest form.

**TWELVE doc locations corrected**, found by `grep -rn '5\.4b' crates/ xtask/` on the final tree
rather than from the story's list of five. Nine name 5.4b; **three do not and would have been missed
by that list**: `IdentityAbstentionCause::AbsenceOfProof`'s doc (it read *"nothing in the verdict set
argues either way"*, which the arbitrated class — carrying an `Opposes` — falsifies; the gap-hunt
agent caught this before dev), `IdentityAbstentionCause`'s *"six rows… two variants"* (seven input
classes now), and the test module's inventory. Three sentences naming 5.4b were left standing because
they are **still true**, chiefly the `#[non_exhaustive]` note whose *"5.4b's table lives in this
one"* is exactly right.

**The register.** Eight entries annotated across four sections — validation corrected the story's
*"six, all in one section"*. The coherence invariant ✅ **CLOSED by construction**, residue to 5.9;
the D13 table gap ✅ **CLOSED in code**, with the `architecture.md` correction moved to **GitHub issue
\#54** (a milestone act, never a story task); the duplicate-rule entry ↺ **PARTLY closed** (totality
yes, refusal → 5.5); the `f64`-token entry ✅ CLOSED; the owner/spec-gap entry ✅ CLOSED; and the four
`NoMatch` annotation lines in the 4.6a/4.7a sections ↺ **CLOSED IN PART** — their *"nothing decides
which side an input falls on"* is what `decide` falsifies, and the `Outcome`-mapping half moves to
5.7. The empty-vector-literals entry is closed **by its own prediction being refuted**: the four
literals compiled untouched, and its count was wrong (four sites, not six).

A new `## Deferred from: story-5.4b` section opens **nine** items — **six name a story, two name a
condition, one names a milestone (issue #54)**; one of the six also names **Epic 6** as a second owner.
⚠️ Counted after the last edit: a first draft of that preamble summed to nine only by counting the
epic as its own item and forgetting the milestone. The arithmetic was wrong before it was measured,
which is why the section states the split to that precision.

**Scope held.** No rule, no join, no blocker, no producer (5.5/5.6) · no `From<Decision> for Outcome`,
`VerdictVectorEntry` untouched and still uninhabited (5.7) · no serde, no persistence (5.9) · no
`Display`, no `cause()`, no grouping (5.14) · `RuleId` not closed into an enum (Epic 6) · no
milli-unit type, constant or field · `architecture.md` **not edited** · `epics.md` **verified and not
edited** · `page.rs`, `locales/app.yml`, `fixtures.rs`, `trap_gate.rs`, `score.rs`, `trap.rs`,
`gap/mod.rs` and everything under `fixtures/` untouched.

### File List

**Modified — code:**

- `crates/opencmdb-core/src/identity/cascade.rs` — `decide`, the private `smallest_rule_with`, six
  new tests with their independent oracle, and eight doc blocks corrected.
- `crates/opencmdb-core/src/identity/mod.rs` — module doc: the algebra now lives here. **Doc only.**
- `crates/opencmdb-core/src/lib.rs` — `decide` joins the flat `identity::cascade` re-export, in the
  idiom `gap` already uses for `reconcile` (a function, last in the braces). The cost is recorded on
  the existing re-export entry rather than in a new one.
- `xtask/src/main.rs` — `IDENTITY_DIR`, `line_has_float`, `has_float_literal`, `gate_float_free`,
  its registration in `run_ci()` as `g5`, three inline tests, and the module doc's gate list (which
  was **already** missing `file-size` before this story — both bullets added).

**Modified — documents:**

- `_bmad-output/implementation-artifacts/deferred-work.md` — eight entries annotated across four
  sections, plus a new `## Deferred from: story-5.4b` section of nine items.
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/project-context.md`, `CLAUDE.md`

**Untouched, verified:** `_bmad-output/planning-artifacts/epics.md` and `architecture.md`, everything
under `fixtures/` (including `MANIFEST.toml`), `Cargo.lock`, all of `crates/opencmdb-bin/`,
`crates/opencmdb-core/src/score.rs`, `trap.rs` and `gap/mod.rs`.

**Created outside the tree:** GitHub issue **#54** — D13's table is short one input class; the
correction to `architecture.md` is a milestone act (AC3).

## Change Log

| Date | Change |
|---|---|
| 2026-07-29 | Story contexted against `3d63544` (story-5.4 branch, post-code-review), PR #52 still open — the deviation from "merge before contexting" is recorded at the top. Answers the three findings story 5.4's review handed forward: `decide`'s signature, the coherence invariant's owner, and the pre-existing `f64` token in the gate's own subtree. |
| 2026-07-29 | Validated by two fresh-context agents (fact-check + gap-hunt); **9 HIGH + 12 MEDIUM/LOW applied**. Four findings changed the DESIGN, not the prose, and each was **measured rather than argued**: (1) the arms are now bound to a `match` on the presence tuple — the gap-hunt agent compiled all three permitted shapes and found that with an `if`-chain, deleting the arbitration arm *"compiles and changes 0 of 16 input classes"*, making M2 a silent no-op; (2) the permutation test now requires three verdicts of the SAME rule-naming kind, because an `Abstained` names no rule and the test was vacuous as written, giving AC4 zero coverage; (3) an implementation taking `min()` over the whole vector satisfied every specified test while naming the rule that argued FOR a refusal — a named test and a full-`Conclusion` oracle now close it; (4) the float gate was built and run, and missed `0.85f64` and `let confidence = 0.85;` — the likeliest shapes a weight takes. Bookkeeping: the register set is **eight entries across four sections**, not six in one; **eleven** doc blocks are falsified, not five; and `cascade.rs:42` was the pre-review coordinate for the `f64` citation — it is `:52`, an error inherited from the register without re-measuring, which is the story's own inherited lesson #1. |
| 2026-07-29 | Implemented: `decide` + `smallest_rule_with`, the `float-free` gate, 6 core tests + 3 xtask tests, 12 doc locations, 5 mutations run, 8 register entries annotated + 9 new items, GitHub issue #54 opened. 271 → 280 tests. Two mutation predictions corrected by measurement (M4 → 47 offenders not 1; M1's count only observable once the 32-walk was made cumulative). One process defect recorded: a mutation restored with `git checkout` against an uncommitted baseline destroyed the implementation once. Status → `review`. |
