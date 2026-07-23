# Story 4.7a: A right verdict by the wrong rule fails

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author of the release gate,
I want a trap scored on `(verdict, rule)` and not the verdict alone,
so that an engine reaching the right answer by the wrong rule FAILS rather than passing.

## Acceptance Criteria

1. **Given** a trap whose expectation names a `rule_id`, **when** the answer reaches the expected
   outcome via a DIFFERENT rule, **then** the trap FAILS, and the failure names BOTH the expected and
   the actual rule.
2. **And** a trap whose answer reaches the expected outcome via the EXPECTED rule still PASSES — the
   rule comparison tightens the gate, it does not reject every correct answer.
3. **And** the rule comparison is LAYERED ON the 4.6a truth table, not folded into it: `score()`
   stays rule-blind (its module doc forbids the change), so the 9-cell table's meaning is unchanged
   and the wrong-rule failure is a distinct, separately-counted condition — not a tenth cell.
4. **And** the rule comparison applies only where a DECISION carries a rule on both sides: a
   `must-not-merge` answered by an `Abstained` (the load-bearing pass cell of 4.6a) has no rule to be
   wrong and still PASSES; an `Abstained` answer never triggers a rule mismatch.
5. **And** the gate's verdict (`passed()`) is FALSE on any wrong-rule failure — a wrong rule blocks a
   release exactly as a wrong verdict does — while the truth-table failure count and the wrong-rule
   count stay readable as two separate numbers.
6. **(The Epic-5 contract — no rule fires in Epic 4.)** **Given** a rule that fires, **when** it
   produces its verdict, **then** it must leave its `rule_id` and its evidence behind — a rule that
   fires without leaving its `rule_id` is undebuggable in production (D19). At v0.1 this is RECORDED
   as the contract the Epic-5 producer must honour and pinned by the uninhabited `verdict_vector`
   placeholder (4.6a); it is NOT built against an engine that does not exist.

## Tasks / Subtasks

- [x] **Task 1 — `Outcome::rule()` accessor** (AC: 1, 4) — pure `opencmdb-core`
  - [x] Add `pub fn rule(&self) -> Option<&RuleId>` on `Outcome` in `score.rs`, the mirror of the
        existing `Expectation::rule()`: `Some` for `Merged`/`Refused`, `None` for `Abstained`.
  - [x] Doc: an abstention names a cause, not a rule — the type says so, so the rule assertion cannot
        apply to it (this is what makes AC4 hold by construction, not by a runtime guard).
  - [x] Test: assert all three variants — `Merged`/`Refused` return `Some(rule)`, `Abstained` returns
        `None`. **Prove-to-red on the INHABITED direction**: collapsing a decision arm to `None` reds
        the `Merged`/`Refused` assertion. The `Abstained → None` direction is by-construction — an
        abstention carries no `RuleId`, so a "return `Some` for `Abstained`" mutation cannot even
        compile and would red nothing; assert it like `Expectation::rule()`'s existing test does
        (`abstain.rule() == None`, no mutation attached), not with a fake prove-to-red.

- [x] **Task 2 — `run_trap()` + the trap verdict type** (AC: 1, 2, 3, 4) — pure `opencmdb-core`
  - [x] Add `pub enum TrapVerdict { Pass, VerdictFail, WrongRule { expected: RuleId, actual: RuleId } }`
        in `score.rs`. **Not `Decision`** (reserved for Epic 5's engine), not `Score` (that binary
        stays 4.6a's). Doc each variant.
  - [x] Add `pub fn run_trap(expected: &Expectation, actual: &Outcome) -> TrapVerdict`. It CALLS
        `score()` first (AC3: layered, not folded): if `Fail` → `VerdictFail`. Only on a verdict Pass
        does it compare rules — `match (expected.rule(), actual.rule())`: two `Some` that DIFFER →
        `WrongRule`; anything else → `Pass`. This makes the wrong-rule condition fire ONLY on the two
        decision-pass cells (`must-merge`→`Merged`, `must-not-merge`→`Refused`), where both sides
        carry a rule.
  - [x] Doc the table extension explicitly, citing D46b AC1 / D19: `score()` is rule-blind on
        purpose; `run_trap` adds the `(verdict, rule)` assertion the trap runner owns.
  - [x] **Update `score.rs`'s MODULE doc** so it stays TRUE once `run_trap` lives beside `score()`:
        the "# What this module deliberately does NOT do — It does not compare rules" claim must
        become "**`score()`** does not compare rules; `run_trap` layers that assertion on top." A
        false module doc is the recurring defect this project has caught three times.
  - [x] Tests (prove-to-red), offending item SECOND per house rule:
        - a `must-merge` right verdict via the wrong rule → `WrongRule` naming both rules (headline);
        - a `must-not-merge` right verdict (`Refused`) via the wrong OPPOSING rule → `WrongRule` too,
          so BOTH decision-pass cells are pinned, not just `must-merge`;
        - a right verdict via the RIGHT rule → `Pass` (AC2);
        - a wrong verdict → `VerdictFail`, and the rule is NOT consulted (AC3 — a wrong-verdict answer
          whose rule ALSO differs must still read `VerdictFail`, not `WrongRule`);
        - `must-not-merge` → `Abstained` → `Pass`, no rule mismatch (AC4, the load-bearing cell);
        - a mutation that compares rules BEFORE `score()` reds the wrong-verdict test (valid because
          the wrong-verdict input uses a different rule, so mutated code would misread it `WrongRule`).

- [x] **Task 3 — surface wrong-rule failures in the harness** (AC: 1, 5) — `opencmdb-bin`
  - [x] In `trap_gate.rs`, feed each answered trap through `run_trap`. Collect wrong-rule failures
        into the `Report` as a `Vec<RuleMismatch>` where `RuleMismatch { trap: TrapId, column: Column,
        expected: RuleId, actual: RuleId }` — enough to NAME both rules (AC1), not just count.
  - [x] Keep the 4.6a `Tally` truth-table path AS-IS (AC3): a `VerdictFail` still records a
        truth-table failure via `tally.record`; a `WrongRule` is recorded in the new vector. A `Pass`
        touches neither failure bucket. (A trap is at most one of the two — `run_trap` returns one
        verdict.)
  - [x] `Report::rule_mismatches() -> &[RuleMismatch]`; `passed()` becomes `discovered > 0 &&
        failures() == 0 && rule_mismatches().is_empty()` (AC5). `Display` names each mismatch:
        "trap `X` (must-merge): expected rule `A`, got `B`". **Additive only** — the existing first
        line ("N trap(s) discovered, M scored, K truth-table failure(s)") and its exact substrings
        are asserted by `the_report_says_plainly_that_nothing_was_scored`; append mismatch lines,
        never rewrite that line, or the 4.6b test regresses.
  - [x] Tests (prove-to-red): a scratch corpus with a `must-merge` trap answered `Merged` by the
        WRONG rule → `passed()` false, one entry in `rule_mismatches()`, `failures()` (truth-table)
        still 0 (AC3 — it is not double-counted); the same trap answered by the RIGHT rule → green.
        Offending trap second. A mutation dropping `rule_mismatches` from `passed()` reds it.

- [x] **Task 4 — record the Epic-5 contract (AC6), do not build it** — docs only
  - [x] Append a `deferred-work.md` entry: the firing-rule contract (`rule_id` + evidence left
        behind) is Epic-5 producer work, pinned today by the uninhabited `verdict_vector` (4.6a). No
        code — there is no rule to fire.
  - [x] A short doc note on `run_trap` (or the `VerdictVectorEntry` placeholder) that AC6 is a
        forward contract, so a later reader does not mistake its absence for an oversight.

- [x] **Task 5 — gates** (all AC)
  - [x] `cargo fmt --all` · `cargo clippy --workspace --all-targets -- -D warnings` ·
        `cargo test --workspace` · `cargo run -p xtask -- ci` all green.
  - [x] `Cargo.lock` unchanged (no new dependency — pure domain + existing bin deps).
  - [x] Confirm `score.rs` stays well under the 2000-code-line `file-size` gate.

## Dev Notes

### What already exists — use it, do not rewrite it

- **`score()` and its 9-cell table** [Source: crates/opencmdb-core/src/score.rs:156] — pure, rule-blind
  BY DESIGN. Its module doc (score.rs:20-27) states the wrong-rule cell is a **PASS** here and that
  the `(verdict, rule)` assertion *"becomes `assert_eq!(decision.rule, case.expect_rule)` in the trap
  runner, which is story 4.7"*. **Do not touch `score()`** — AC3 exists to keep it rule-blind. Add
  beside it.
- **`Expectation::rule() -> Option<&RuleId>`** [Source: crates/opencmdb-core/src/trap.rs:79] — already
  returns the rule for the two decision columns and `None` for `must-abstain`. `Outcome::rule()`
  (Task 1) is its exact mirror; build it the same way.
- **`Outcome`** [Source: crates/opencmdb-core/src/score.rs:60] — `Merged{rule}` / `Refused{rule}` /
  `Abstained{cause}`. Its doc already says the payloads exist *"so that story 4.7 can add
  `(outcome, rule)` comparison without changing this type"* — this is that story. No change to
  `Outcome` beyond adding the `rule()` accessor.
- **`Tally`** [Source: crates/opencmdb-core/src/score.rs:271] — the truth-table failure count, per
  D18 column. Keep it exactly; the wrong-rule count is a NEW, separate surface (AC3/AC5), not a
  fourth column of this tally.
- **`score_corpus` / `Report`** [Source: crates/opencmdb-bin/src/trap_gate.rs:138] — the harness that
  feeds `answers: BTreeMap<TrapId, Outcome>` through `tally.record`. Task 3 routes each answer through
  `run_trap` instead and grows `Report` with the mismatch vector. The vacuity floor (`discovered >
  0`) and the unknown-answer / duplicate-id guards stay untouched.
- **`VerdictVectorEntry` (uninhabited)** [Source: crates/opencmdb-core/src/score.rs:200] — already the
  placeholder for the `(rule, verdict, evidence)` triple. It IS the pin for AC6; do not inhabit it.

### The design in one paragraph

`score()` answers "right verdict?" and stays rule-blind. `run_trap()` layers on top: right verdict
first (reuse `score()`), and only then "right rule?", but the rule question is asked ONLY where a
decision carries a rule on both sides. That is the two cells `must-merge → Merged` and
`must-not-merge → Refused`. Every other verdict-pass cell (`must-not-merge → Abstained`,
`must-abstain → Abstained`) has no `decision.rule` to compare and stays a pass. So `WrongRule` is,
by construction, "a decision that reached the right column by firing the wrong rule" — D46b's AC1,
verbatim, where D64 sent it to live.

### Traps

1. **Editing `score()` to compare rules.** AC3 forbids it, and score.rs's own module doc calls
   deriving `PartialEq` between expectation and outcome the thing that *"would silently fail that cell
   and steal 4.7's criterion."* The rule check is a separate function.
2. **Comparing the rule before the verdict.** A wrong-verdict answer whose rule also differs must read
   `VerdictFail`, not `WrongRule` — the verdict is the coarser, first question. Prove this ordering
   to red (Task 2).
3. **Flagging an abstention as a wrong rule.** `must-not-merge → Abstained` is 4.6a's load-bearing
   PASS (*"an engine that abstains on everything… gets demolished by the middle column"* — only true
   if abstention passes `must-not-merge`). An abstention has no rule; `Outcome::rule()` returns
   `None`, so the mismatch cannot fire. Keep it that way by construction (AC4).
4. **Double-counting.** A wrong-rule trap is a rule mismatch, NOT also a truth-table failure — its
   verdict passed. `run_trap` returns exactly one verdict; record accordingly (AC3). A test asserts
   `failures()` stays 0 while `rule_mismatches()` is 1.
5. **Building AC6.** There is no engine and no firing rule in Epic 4. AC6 is a recorded contract, not
   code. Inventing a rule producer to "satisfy" it is the *"metric written after the engine"* mistake
   (D19) in reverse — and would need a type Epic 5 owns.
6. **The `NoMatch → Refused` engine question is NOT this story.** Whether an engine that finds no
   merging rule should `Refuse` or `Abstain` is Epic-5 engine design; 4.7a only SCORES already-produced
   answers. Note it as inherited-by-Epic-5, do not decide it here.
7. **Claiming more than measured.** Name the command behind every count (three completion records
   over-claimed before). Prove every new guard to red and record the mutation.

### Project Structure Notes

- **Updated:** `crates/opencmdb-core/src/score.rs` (the `Outcome::rule()` accessor, `TrapVerdict`,
  `run_trap`), `crates/opencmdb-core/src/lib.rs` (exports), `crates/opencmdb-bin/src/trap_gate.rs`
  (route through `run_trap`, `RuleMismatch`, `Report` surface, `passed()`, `Display`),
  `_bmad-output/implementation-artifacts/deferred-work.md` (AC6 contract),
  `_bmad-output/implementation-artifacts/sprint-status.yaml`.
- **Unchanged, expected:** `score()` and `Tally` (semantics frozen by 4.6a), `fixtures/`, `Cargo.lock`.

### Latest technical specifics

No new crate, no version bump. Rust 1.96+, edition 2024. Pure domain code in core, plus existing bin
deps in the harness. **Never invent a version — pin from the committed `Cargo.lock`.**

### References

- [Source: _bmad-output/planning-artifacts/epics.md:1017 — Story 4.7 and the 4.7a/4.7b split note]
- [Source: _bmad-output/planning-artifacts/architecture.md:4457 — D46b AC1 survives D64 and changes
  owner: *"compare `(verdict, rule)`, never `verdict` alone… it becomes
  `assert_eq!(decision.rule, case.expect_rule)` in the trap runner"*]
- [Source: _bmad-output/planning-artifacts/architecture.md:2555-2582 — D46b in full; row 3 is the
  wrong-rule case: *"same output, different reason. BOTH JOBS GREEN… the worst kind"*]
- [Source: _bmad-output/planning-artifacts/architecture.md:1208 — D18: the release gate is one number,
  truth-table failures = 0; the rule assertion adds a second blocking number, not a fraction]
- [Source: _bmad-output/planning-artifacts/architecture.md:1267 — D19: the oracle is the author's
  reason, and a test that checks only the verdict *"goes green for the right answer reached by the
  wrong rule — and that engine will break on the next fixture"*]
- [Source: _bmad-output/planning-artifacts/architecture.md:1397 — the COMPLETE VERDICT VECTOR
  *requirement* ("the anti-drift is not discipline, it is a data requirement"). The element shape
  `(rule, verdict, evidence)` is the CODE's decomposition of it — score.rs:196, not the architecture —
  and AC6's contract plus the uninhabited `VerdictVectorEntry` anticipate it]
- [Source: crates/opencmdb-core/src/score.rs:20-27 — the module doc that assigns this criterion to 4.7]
- [Source: crates/opencmdb-core/src/trap.rs:79 — `Expectation::rule()`, the mirror for `Outcome::rule()`]
- [Source: _bmad-output/implementation-artifacts/4-6a-scoring-algebra.md — the algebra this layers on]
- [Source: _bmad-output/implementation-artifacts/4-6b-metrics-harness.md — the harness Task 3 extends]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.8 — `claude-opus-4-8[1m]`

### Debug Log References

Every claim below is a command that was run, not an inference.

- **Three guards proven to red**, each reverted from a backup:
  | Mutation | Test reddened |
  |---|---|
  | `run_trap` compares the rule BEFORE calling `score()` | `a_wrong_verdict_is_verdict_fail_even_when_the_rule_also_differs` — reads `WrongRule{l1-exact-mac, l2-different-switch}` instead of `VerdictFail` |
  | `Outcome::rule()` returns `None` on the `Merged`/`Refused` arms | `an_outcome_names_its_rule_only_when_it_is_a_decision` — `None` vs `Some(l2-uplink-agrees)` |
  | `Report::passed()` drops the `rule_mismatches.is_empty()` term | `a_right_verdict_by_the_wrong_rule_reddens_the_gate_without_a_truth_table_failure` — the `!passed()` assertion (AC5) |

  The first is the ordering guard AC3 rests on: a `must-merge` answered by a `Refused` whose rule ALSO
  differs is a WRONG VERDICT, and must read `VerdictFail`, not `WrongRule`. A "compare rules first"
  mutation misreads it — so the wrong-verdict input deliberately uses a different rule, and that is
  what makes the mutation observable.
- **`Outcome::rule()`'s `Abstained → None` direction is by construction, not prove-to-red**: an
  abstention carries no `RuleId`, so a "return `Some` for `Abstained`" mutation cannot compile and
  would red nothing (validation caught this as a fake prove-to-red before dev started). Asserted like
  `Expectation::rule()`'s own test — `abstained().rule() == None`, no mutation attached.
- **Gates:** `cargo fmt --all` clean · `cargo clippy --workspace --all-targets -- -D warnings` clean
  · `cargo test --workspace` → **104 (bin, +2) + 77 (core, +6) + 42 (xtask), 0 failed**.
- `cargo run -p xtask -- ci` → all gates green, `file-size` largest 884 (this story adds ~55 code
  lines to `score.rs` and ~40 to `trap_gate.rs`, both far under 2000). `architecture-views.md` NOT
  regenerated.
- **`Cargo.lock` did not move**; no dependency added. Pure domain code in core; existing bin deps in
  the harness. The MariaDB-backed tests did NOT run (`DATABASE_URL` unset) — the counts say nothing
  about the database.

### Completion Notes List

- **`score()` stays rule-blind; `run_trap` layers the `(verdict, rule)` assertion on top (AC3).** The
  9-cell truth table is untouched. `run_trap` calls `score()` first and only compares the rule on a
  verdict pass, so a wrong rule is a DISTINCT failure beside the table, not a tenth cell. The module
  doc's "does not compare rules" claim was re-scoped to "`score()` does not; `run_trap` layers it" so
  it stays true now that both live in one module.
- **`WrongRule` fires only on the two decision-pass cells, by construction (AC1, AC4).**
  `Outcome::rule()` (the new mirror of `Expectation::rule()`) is `Some` exactly on `Merged`/`Refused`;
  the `match (Some, Some) if differ` arm therefore fires only on `must-merge → Merged` and
  `must-not-merge → Refused`. Every abstention yields `None` and passes — so `must-not-merge →
  Abstained`, 4.6a's load-bearing pass cell, stays a pass here too. The failure names BOTH rules.
- **The gate blocks on a wrong rule, counted separately (AC5).** The harness routes each answer
  through `run_trap` BESIDE the unchanged `tally.record`: a `WrongRule` verdict passes the truth table
  (so `failures()` stays 0 and it is not double-counted) but is collected into `Report.rule_mismatches`,
  and `passed()` now requires that vector empty. `Display` appends one line per mismatch naming both
  rules, leaving the 4.6b first line and its asserted substrings intact.
- **AC6 is a recorded contract, not code.** No rule fires in Epic 4, so the requirement that a firing
  rule leave its `rule_id` + evidence is an Epic-5 obligation, pinned by the uninhabited
  `VerdictVectorEntry` (whose doc now says so) and appended to `deferred-work.md`. The `NoMatch →
  Refused` vs `Abstained` engine question is recorded there as inherited by Epic 5, not decided here.

### File List

- `crates/opencmdb-core/src/score.rs` (modified — `Outcome::rule()`, `TrapVerdict`, `run_trap`,
  module doc re-scoped, `VerdictVectorEntry` doc; 6 tests)
- `crates/opencmdb-core/src/lib.rs` (modified — exports `TrapVerdict`, `run_trap`)
- `crates/opencmdb-bin/src/trap_gate.rs` (modified — `RuleMismatch`, `Report.rule_mismatches`,
  `run_trap` routing, `passed()`, `Display`; 2 tests)
- `_bmad-output/implementation-artifacts/deferred-work.md` (modified — two story-4.7a entries)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modified)
- `_bmad-output/planning-artifacts/epics.md` (modified — the 4.7a/4.7b split, done at story prep)
- `Cargo.lock` — **unchanged**, measured.

### Change Log

- 2026-07-23 — The trap runner asserts the rule. `run_trap(expected, actual) -> TrapVerdict` scores
  the verdict via the frozen rule-blind `score()`, then — only on a verdict pass, only where a
  decision carries a rule on both sides — compares `(verdict, rule)`, D46b's surviving criterion. A
  right answer by the wrong rule FAILS, naming both rules; the harness carries these in a separate
  `rule_mismatches` bucket that blocks `passed()` without inflating the truth-table count. AC6 (a
  firing rule leaves its evidence) is recorded as the Epic-5 contract it is pre-engine. Three guards
  proven to red.

## Review Findings

_Code review 2026-07-23 (Blind Hunter + Edge Case Hunter + Acceptance Auditor). Verdict: clean pass on
AC1–AC6, none of the 7 traps fallen into, prove-to-red claims validated, core/bin tests re-run green
(77 core / 104 bin). No Critical/High. Findings below._

- [x] [Review][Decision→Patch APPLIED] `Display`'s first line read all-green while the gate was red on
      a wrong rule — [crates/opencmdb-bin/src/trap_gate.rs] — **Guy chose option 1 (fix).** The first
      line now appends `", K wrong-rule failure(s)"` when `rule_mismatches` is non-empty, so the line
      alone can never read as a pass while `passed()` is false — closing the 4.6b AC3 hazard for the new
      failure mode. Appended only when non-zero, so the nominal first line is byte-for-byte unchanged and
      the 4.6b substrings survive (verified: `the_report_says_plainly_that_nothing_was_scored` still
      green). Pinned by a new adjacency assertion in
      `a_right_verdict_by_the_wrong_rule_reddens_the_gate_without_a_truth_table_failure`, **proven to red**
      (dropping the suffix reds line 776).
- [x] [Review][Patch APPLIED] `run_trap` now gates on `!= Score::Pass` rather than `== Score::Fail`
      [crates/opencmdb-core/src/score.rs:226] — only a proven `Score::Pass` proceeds to the rule check;
      any verdict not provably right is `VerdictFail` and never falls through to a rule comparison.
      Semantics identical while `Score` is binary; future-proof against a third variant. Doc updated to
      state the positive-gate rationale. All 77 core tests green.
- [x] [Review][Defer] `(verdict, rule)` comparison is whitespace/case-sensitive with no normalization
      [crates/opencmdb-core/src/score.rs:373] — deferred, forward concern (Epic 5). `RuleId` on the
      `Outcome` side is never validated and the `Expectation` side is only emptiness-checked (not
      trimmed/lowercased like `TrapId`), so `"l1-exact-mac "` vs `"l1-exact-mac"` would be a false
      `WrongRule`. No real rule producer exists pre-engine, so it cannot bite until Epic 5 supplies one.
- Dismissed (2): `passed()` reads the `rule_mismatches` field rather than the accessor (functionally
  identical, internal); the harness scores each answered trap twice (once via `tally.record`, once via
  `run_trap`) — deliberate "layered, not folded" design per AC3, allowed by the DRY convention.
