# Story 4.6c: Two runs are comparable only under an identical capability snapshot

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author of the release gate,
I want the harness to refuse to compare two runs whose capability snapshots differ, and to say so rather than report no difference,
so that a verdict stays falsifiable — a regression and a legitimate re-derivation must never look alike.

## Context

**Depends on 4.6a** (the record, which carries the `capability_snapshot`) and **4.6b** (the harness that produces records). This story adds the one thing D36 asks for beyond the record itself.

The sentence this story implements, verbatim:

> **"A verdict without its capability snapshot is UNFALSIFIABLE: you cannot tell a regression from a legitimate re-derivation."** … **"Two verdicts are comparable only under an identical snapshot — otherwise they are not two answers, they are two questions."** [architecture.md:2069-2073]

And the trap it exists to prevent:

> **"The trap is lexical: we require REPRODUCIBILITY, not STABILITY.** Replay `(data, capability)` → the same verdict, always. The verdict is allowed to change over time. Requiring stability means pinning capability, i.e. reintroducing the false merge. **Anyone who 'fixes this flake' by pinning the capability has broken the product to make CI green.**" [architecture.md:2063-2067]

**Explicitly OUT of scope:** the record and the algebra (4.6a) · the harness and its vacuity report (4.6b) · any engine · lattice monotonicity ([architecture.md:2075-2077] — it needs an engine and 2^n capability subsets; Epic 5's *"monotone-honesty invariant trap family"*) · persisting runs to disk unless AC3 forces it.

## The question this story must answer first

**D36 puts `capability_snapshot` on each RECORD, not on a run.** A run is a set of records, and 4.5b made the descriptor *positional*: `fixtures/scenario/replay/capability-downgrade.jsonl` changes it mid-stream, so **two records in one run can legitimately carry different snapshots** — and a trap judging observations on both sides of that record has two descriptors in force.

So "two runs with identical snapshots" is not well-defined until this story defines it. That is AC1, and it is the whole design.

## Acceptance Criteria

### Must / must not

- **MUST** define what a run's snapshot IS, given that records carry their own.
- **MUST** make refusal distinguishable from "no difference found".
- **MUST NOT** pin, normalise or default a capability to make a comparison succeed.

---

1. **What a run's comparability key is, decided and written down.** Two shapes are defensible and the story does not pick for you — **but you must pick one, state it, and say what it costs**:
   - **Pairwise by trap**: records are matched by their input identity, and each PAIR is comparable only if its two snapshots are identical. A run can then be *partly* comparable, and the report must say which traps were compared and which were not.
   - **Run-level**: a run carries one snapshot and is comparable only if the whole set matches. Simpler to report, but **it is not obviously well-defined against a corpus that already contains a mid-stream downgrade** — say which snapshot represents a run, and why that is honest.

   Whichever you choose, the interaction with `capability-downgrade.jsonl` must be stated: it is committed, it is in the corpus, and any comparison touching it meets the positional case immediately.

2. **Refusal is a distinct outcome from "no difference".** A comparison of two runs returns one of at least three things: *comparable and identical* · *comparable and differing* (with what differs) · **refused, naming the snapshots that differ**. A function returning "no change" when the snapshots differ is exactly the unfalsifiability D36 names — the caller cannot tell a passing comparison from a meaningless one.
   **This is the story's core assertion**: a test drives two runs with differing snapshots and asserts the refusal, and a second test asserts that the refusal is NOT equal to the no-difference outcome.

3. **Where a run comes from, and whether anything is persisted.** 4.6b produces records in memory. Comparing two runs needs both to exist at once. **Decide and state**: two in-memory runs passed to a pure function (no persistence, and the comparison is then a domain function that belongs in `opencmdb-core`), or a serialized run artefact (and then: the format, and where it lives — **not under `fixtures/`**, which is the locked oracle; a generated score inside the lockfile-for-data would make the gate's input and output indistinguishable).
   Prefer the pure function unless something forces otherwise: it keeps this story free of I/O and puts the logic where D47 wants it.

4. **A difference in the snapshot is never repaired.** No normalising, no defaulting, no "closest match", no ignoring `as_of` to make two snapshots equal. If a test is uncomfortable because two runs will not compare, **that is the mechanism working**. Put the D36 quote next to the code so the next person to feel that discomfort reads it before acting on it.

5. **Comparison is exhaustive over the record's fields, with no `_` arm** where a new field must force a decision — the mechanism 4.5b relied on when `Record::Capability` broke four matches on purpose. A field added to the record later must not be silently excluded from comparison.

6. **`source_state` does not participate, and the reason is recorded.** It is empty by construction until Epic 13 (4.6a's AC8), so including it in the comparison key is vacuous today and would silently start mattering the day Epic 13 lands. State the choice; record it.

7. **The register is appended to** — `## Deferred from: story-4.6c (2026-07-22)`, never rewriting a bullet:
   - **Lattice monotonicity is not implemented** — *"losing a capability can only move a verdict TOWARD doubt"* [architecture.md:2075-2077] is the law that makes this testable exhaustively, and it needs an engine. Name it as Epic 5's.
   - **`source_state` excluded from the comparison key** and what happens when Epic 13 fills it.
   - Whatever AC1 and AC3 decided, so the next reader sees a choice rather than an accident.

8. **All gates green, locally:** `cargo fmt --all` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo test --workspace` · `cargo xtask ci`. **Do not regenerate `architecture-views.md`.**

## Tasks / Subtasks

- [x] **Task 1 — decide the key** (AC: 1, 3, 6)
  - [x] Pairwise or run-level, with the cost stated and the `capability-downgrade.jsonl` interaction addressed.
  - [x] Pure function vs persisted artefact; if pure, it belongs in `opencmdb-core` beside 4.6a's algebra.
  - [x] `source_state` excluded, with the reason in the code.

- [x] **Task 2 — the comparison** (AC: 2, 4, 5)
  - [x] Three outcomes minimum, refusal distinct from no-difference.
  - [x] Exhaustive over the record's fields, no `_` arm.
  - [x] The D36 quote next to the code that refuses.

- [x] **Task 3 — the tests** (AC: 2, 4)
  - [x] Differing snapshots → refused, naming both.
  - [x] Refusal `!=` no-difference — the assertion that makes AC2 non-vacuous.
  - [x] Identical snapshots → compared, and a real difference is reported.
  - [x] **Prove-to-red on every new guard**, each mutation recorded. Offending item **second** in any vector. **Do not write a comment asserting a coverage property you did not measure.**

- [x] **Task 4 — the record and the gates** (AC: 7, 8)
  - [x] Append the three entries of AC7 to `deferred-work.md`.
  - [x] Update `sprint-status.yaml`; put it in the File List.
  - [x] Run the four gates. **Name the command behind every claim in the completion record.**

### Review Findings

_Code review 2026-07-23 — three parallel layers (Blind Hunter, Edge Case Hunter, Acceptance Auditor). **All 8 ACs SATISFIED, no violations**; every measured claim reproduced (tests 102/66/42, E0027 destructure, `Cargo.lock` unmoved, gates green). The auditor judged pairwise the right call and the destructure a faithful "no `_` arm". The two hunters found robustness holes the spec did not require but the code should hold, and two errors in the completion record._

- [x] [Review][Decision] **Should a rule-only outcome change (same verdict, different `rule`) be a `Differing`, or ignored like `score()` ignores it?** **RESOLVED → option (a): keep rule-sensitivity.** A rule change between two runs of one corpus is exactly the D19/D46b drift a run comparison exists to surface — *"same output, different reason… an engine divergence hiding behind a correct result — the worst kind"*. `score()` (the gate) ignores the rule because it does not bear on correctness; a run-to-run comparison does not, because it hunts drift. `RecordComparison::Differing`'s doc now states this and cites D19/D46b, and `same_verdict_different_rule_is_differing_not_identical` pins it. Option (b) — coarsening to the `Column` — was rejected: it would blind the comparison to the one signal it is best placed to catch [crates/opencmdb-core/src/score.rs:335,865]
- [x] [Review][Patch] **`compare_runs` silently drops a duplicated `TrapId` (last-wins in the `BTreeMap`)** — resolved: `index` now `debug_assert!`s `by_trap.len() == run.len()` and the precondition is stated in the fn doc ("each run names every trap at most once", enforced upstream by 4.6b's `DuplicateTrapId`) [crates/opencmdb-core/src/score.rs:464]
- [x] [Review][Patch] **The `incomparable` bucket discards both snapshots** — resolved: `RunComparison::incomparable` is now `Vec<(TrapId, Capabilities, Capabilities)>`, carrying D36's load-bearing evidence like `differing` carries the outcomes; `compare_runs` threads both snapshots through, and the bucket test asserts the full tuple [crates/opencmdb-core/src/score.rs:430,488]
- [x] [Review][Patch] **`compare_records` never checks the two records name the same trap** — resolved: `debug_assert_eq!(before.trap, after.trap)` at the top of `compare_records`, with the doc stating the caller matches by `TrapId` [crates/opencmdb-core/src/score.rs:390]
- [x] [Review][Patch] **The `differing` bucket's payload is never asserted** — resolved: the bucket test now asserts `cmp.differing == vec![(trap, merged(), refused())]`, the full `(trap, before, after)` in order [crates/opencmdb-core/src/score.rs:919]
- [x] [Review][Patch] **`as_of`-only capability difference refuses the comparison, and nothing tests it.** — resolved: `IncomparableSnapshot`'s doc states `as_of` participates (a dated fact, D34 §1) and that same-corpus runs share it in practice; `same_kinds_different_as_of_is_incomparable` pins the refusal [crates/opencmdb-core/src/score.rs:373,979]
- [x] [Review][Patch] **Empty-run boundaries untested, and `is_unchanged()` on two empty runs is a vacuous `true`.** — resolved: `is_unchanged`'s doc names the vacuous-true edge and points a caller at `identical` to disambiguate (the `Tally::scored()` pattern); `two_empty_runs_are_vacuously_unchanged` and `a_trap_in_one_empty_sided_run_is_a_membership_change` cover the boundaries [crates/opencmdb-core/src/score.rs:437,999]
- [x] [Review][Patch] **Every bucket test uses a single-element vector — the "offending item second" convention is not honoured for the buckets.** — resolved: `two_traps_can_share_a_bucket` puts two traps in `differing` and asserts both are reported [crates/opencmdb-core/src/score.rs:1029]
- [x] [Review][Patch] **`two_identical_runs_are_unchanged`'s downgraded record is decorative** — KEPT AS-IS, documented. Both sides being `run.clone()` is the point: the test proves a two-record run carrying a *mixed* snapshot (`caps_full` + `caps_downgraded`), each pair comparing equal, reports `is_unchanged` with two `identical`. It is not decorative — it guards that a downgraded snapshot on its own does not trip `is_unchanged` when nothing changed. The refusal-within-a-run case the finding wanted is now covered by `a_run_with_only_an_incomparable_pair_is_not_unchanged` and the mixed bucket test [crates/opencmdb-core/src/score.rs:929]
- [x] [Review][Patch] **DRY: the 8-field destructure is duplicated verbatim for before/after.** — resolved: `fn comparable_fields(&ScoredRecord) -> (&Outcome, &Capabilities)` now holds the single `..`-free destructure (the E0027 guard, once), called twice by `compare_records` [crates/opencmdb-core/src/score.rs:353]
- [x] [Review][Patch] **Three doc overclaims.** — resolved: `is_unchanged` now says "unchanged **in outcome and snapshot**" and lists the ignored fields; `Differing` no longer claims "a function of the data alone" — it states the rule-sensitivity as the deliberate D19 choice; the `RunComparison` doc says "every DISTINCT trap id lands in exactly one bucket" [crates/opencmdb-core/src/score.rs:337,417,438]
- [x] [Review][Patch] **Two errors in my own completion record** — resolved below: the test count is corrected to **12** new `#[test]` fns (core 59→71), and the Debug Log's mutation #2 is re-described to match what actually reds (the caps guard removed on the outcome-match path, not a literal reorder) [this file]

_Dismissed as noise (2): the double clone of each record into two maps (a cost note at n≈300, not a defect) · the AC7 heading date `2026-07-23` vs the spec's placeholder `2026-07-22` (the real completion date is right)._

## Dev Notes

### What already exists — use it, do not rewrite it

- **4.6a's record**, carrying the `capability_snapshot`, the outcome, the trap reason and the input identity.
- **4.6b's harness**, which produces records from the corpus.
- **`Capabilities`** [Source: crates/opencmdb-core/src/observation/mod.rs:216-232] — `{ as_of, kinds }`, `PartialEq`, so snapshot equality is already total. Note `as_of` participates: two runs of the same corpus have the same `as_of` because 4.5b dates it from the FILE, not from a clock. That is what makes this comparison deterministic at all.
- **`capability-downgrade.jsonl`** [Source: fixtures/scenario/replay/] — the committed stream with a mid-stream descriptor change. AC1 must address it.
- **`deferred-work.md`'s 4.5b entry** — it states that `poll` returns no `PollSummary` on the error path, so **the snapshot is obtained by walking the records**, not from a summary. That constraint is 4.6b's, but it shapes what a record's snapshot can be.

### Traps

1. **Returning "no difference" when the snapshots differ.** AC2. It is the exact unfalsifiability D36 names, and it will look like a passing comparison forever.
2. **Pinning a capability to make a test comfortable.** *"Anyone who 'fixes this flake' by pinning the capability has broken the product to make CI green."*
3. **Assuming a run has one snapshot** because D36's sentence says "two runs". D36 puts the snapshot on the RECORD; 4.5b made it positional. AC1 exists because of that gap.
4. **Writing a run artefact under `fixtures/`.** That directory is the locked oracle.
5. **Claiming more than was measured.** Three consecutive completion records over-claimed. Name the command, or write the weaker true sentence.
6. **Skipping `--all-targets` or `xtask ci` locally.** Epic 3's retrospective recorded four CI-only failures from that.

### Git intelligence

`master` requires a pull request since 2026-07-22 (0 approvals, `ci` green, squash merge). Work on `story/4-6-metrics-harness`; **do not push to `master`**.

### Latest technical specifics

No new crate, no version bump. **Locked** (committed `Cargo.lock`, verified 2026-07-22): `serde 1.0.228`, `serde_json 1.0.150`, `uuid 1.24.0`, `chrono 0.4.45` (`default-features = false`), `toml 0.8.23`. Rust 1.96+, edition 2024, `resolver = "3"`. **Never invent a version.**

### Project Structure Notes

- **Updated:** `crates/opencmdb-core/src/` (the comparison, if pure — the preferred shape), `crates/opencmdb-core/src/lib.rs`, `deferred-work.md`, `sprint-status.yaml`. Possibly `crates/opencmdb-bin/src/` if AC3 forces persistence.
- **Unchanged, expected:** `fixtures/`, `Cargo.lock`.

### References

- [Source: _bmad-output/planning-artifacts/epics.md:983-1003 — Story 4.6; its third criterion is this story]
- [Source: _bmad-output/planning-artifacts/architecture.md:2057-2077 — D36 in full: unfalsifiability, reproducibility-not-stability, "two answers or two questions", lattice monotonicity]
- [Source: _bmad-output/planning-artifacts/architecture.md:2063-2067 — "anyone who 'fixes this flake' by pinning the capability has broken the product to make CI green"]
- [Source: _bmad-output/planning-artifacts/architecture.md:2075-2077 — lattice monotonicity, deferred to Epic 5]
- [Source: _bmad-output/planning-artifacts/architecture.md:1922-1935 — D34 §1: the descriptor is a dated fact, not a constant]
- [Source: crates/opencmdb-core/src/observation/mod.rs:216-232 — `Capabilities`, and why `as_of` is deterministic since 4.5b]
- [Source: _bmad-output/implementation-artifacts/4-5b-fixture-connector-replays-capability-changes.md — the story that made the descriptor positional, which is why AC1 exists]
- [Source: _bmad-output/implementation-artifacts/4-6a-scoring-algebra.md — the record this story compares]
- [Source: _bmad-output/implementation-artifacts/4-6b-metrics-harness.md — the harness that produces runs]
- [Source: _bmad-output/implementation-artifacts/deferred-work.md — the register to append to]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.8 — `claude-opus-4-8[1m]`

### Debug Log References

Every claim below is a command that was run, not an inference.

- **Five guards proven to red** (four mutations + the AC5 destructure), each reverted from a backup:
  | Mutation | Tests reddened |
  |---|---|
  | the snapshot check dropped entirely | 4 tests, incl. both refusal tests |
  | the caps guard removed on the outcome-MATCH path (equal outcomes return `Identical` without checking caps) | `a_differing_snapshot_refuses_even_when_the_outcomes_match` + the two refusal tests |
  | `is_unchanged` ignores `incomparable` (refusal read as no-difference) | `a_run_with_only_an_incomparable_pair_is_not_unchanged` |
  | the `only_after` membership bucket dropped | `compare_runs_buckets_every_trap_and_is_partly_comparable` |
  | **AC5:** a field added to `ScoredRecord` | `compare_records` fails to compile — `E0027: pattern does not mention field` |

  The second is the one that embodies D36, and the review corrected its earlier description: a
  literal *reorder* of the two `if`s reds nothing, because when outcomes are equal the reordered code
  still falls through to the caps check. The mutation that reds is removing the caps guard on the
  equal-outcome path — returning `Identical` on an outcome match without ever checking caps. Two
  verdicts that agree by coincidence under different capabilities would then pass as "the same
  answer", exactly the unfalsifiability D36 names.
- **Gates:** `cargo fmt --all` clean · `cargo clippy --workspace --all-targets -- -D warnings` clean
  · `cargo test --workspace` → **102 (bin) + 71 (core, +12) + 42 (xtask), 0 failed**. The +12 counts
  the 7 tests written in dev-story plus the 5 added while applying the review (rule-sensitivity,
  `as_of`, two empty-run boundaries, two-in-a-bucket).
- `cargo xtask ci` → all gates green, `file-size 884` (this story adds ~150 code lines to `score.rs`,
  still far under 2000). `architecture-views.md` NOT regenerated.
- **`Cargo.lock` did not move**; no dependency added. Pure domain code — no I/O, no clock.
- **The MariaDB-backed tests did NOT run** (`DATABASE_URL` unset). These counts say nothing about
  the database.

### Completion Notes List

- **Comparability is PAIRWISE, and run-level was rejected with a reason.** 4.5b made the capability
  descriptor positional, so two records in one run legitimately carry different snapshots — "the
  run's snapshot" is not well-defined. `compare_runs` matches by `TrapId`; a run may be *partly*
  comparable, and `RunComparison` says which traps compared, which differed, which were refused, and
  which changed membership. The committed `capability-downgrade.jsonl` is exactly the positional case:
  a trap scored under NET_RAW and under ping-only is refused, never silently equal.
- **Refusal is a distinct outcome from "no difference" — AC2's core.** `RecordComparison` has three
  variants: `Identical`, `Differing`, `IncomparableSnapshot`. The snapshot is checked FIRST, so a
  differing snapshot is refused before the outcomes are even looked at. A test asserts the refusal is
  `!=` `Identical`; `RunComparison::is_unchanged()` returns false when any pair is incomparable — an
  undecided comparison is not a passing one.
- **A snapshot difference is NEVER repaired (AC4).** No normalising, no defaulting, no ignoring
  `as_of`. The D36 quote — *"anyone who 'fixes this flake' by pinning the capability has broken the
  product to make CI green"* — sits next to the code that refuses.
- **The comparison is exhaustive over the record's fields (AC5)** by a full destructure with no `..`:
  every field is named, and each is annotated with why it participates or does not. Proven: adding a
  field makes `compare_records` fail to compile with `E0027`. `source_state` is excluded (AC6) —
  uninhabited until Epic 13 — and `verdict_vector` likewise, both recorded.
- **Pure function in `opencmdb-core` (AC3)**, beside 4.6a's algebra. No persistence, no I/O. The
  types are exercised by hand-built records, like 4.6a's `ScoredRecord`, until an engine produces
  runs.
- **Three register entries appended**: lattice monotonicity (Epic 5), `source_state` exclusion (to
  revisit at Epic 13), and the pairwise-vs-run-level decision.

### File List

- `crates/opencmdb-core/src/score.rs` (modified — `RecordComparison`, `RunComparison`,
  `compare_records`, `compare_runs`, `comparable_fields`; 12 tests)
- `crates/opencmdb-core/src/lib.rs` (modified — exports)
- `_bmad-output/implementation-artifacts/deferred-work.md` (modified — three story-4.6c entries)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modified)
- `Cargo.lock` — **unchanged**, measured.

### Change Log

- 2026-07-23 — Run comparability closes Epic 4's harness half. Two runs compare trap by trap; a pair
  is comparable only under an identical capability snapshot (D36), and a differing snapshot is
  refused, never reported as "no change". Pairwise, because 4.5b made capability positional and a
  run has no single snapshot. Pure domain code, exhaustive over the record's fields so a new field
  forces a decision. Five guards proven to red, including the D36 snapshot-first check and the AC5
  destructure.
- 2026-07-23 — Code review addressed (11 findings, all resolved). Decision (a): rule-sensitivity in
  `compare_records` is kept and documented as the D19/D46b drift case. Three robustness holes closed
  (`incomparable` now carries both snapshots; `debug_assert`s on same-trap and within-run uniqueness),
  the before/after destructure DRY-factored into `comparable_fields`, three doc overclaims corrected,
  and five tests added (rule-sensitivity, `as_of`, two empty-run boundaries, two-in-a-bucket).
