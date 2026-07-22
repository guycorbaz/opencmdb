# Story 4.6c: Two runs are comparable only under an identical capability snapshot

Status: ready-for-dev

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

- [ ] **Task 1 — decide the key** (AC: 1, 3, 6)
  - [ ] Pairwise or run-level, with the cost stated and the `capability-downgrade.jsonl` interaction addressed.
  - [ ] Pure function vs persisted artefact; if pure, it belongs in `opencmdb-core` beside 4.6a's algebra.
  - [ ] `source_state` excluded, with the reason in the code.

- [ ] **Task 2 — the comparison** (AC: 2, 4, 5)
  - [ ] Three outcomes minimum, refusal distinct from no-difference.
  - [ ] Exhaustive over the record's fields, no `_` arm.
  - [ ] The D36 quote next to the code that refuses.

- [ ] **Task 3 — the tests** (AC: 2, 4)
  - [ ] Differing snapshots → refused, naming both.
  - [ ] Refusal `!=` no-difference — the assertion that makes AC2 non-vacuous.
  - [ ] Identical snapshots → compared, and a real difference is reported.
  - [ ] **Prove-to-red on every new guard**, each mutation recorded. Offending item **second** in any vector. **Do not write a comment asserting a coverage property you did not measure.**

- [ ] **Task 4 — the record and the gates** (AC: 7, 8)
  - [ ] Append the three entries of AC7 to `deferred-work.md`.
  - [ ] Update `sprint-status.yaml`; put it in the File List.
  - [ ] Run the four gates. **Name the command behind every claim in the completion record.**

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

### Debug Log References

### Completion Notes List

### File List

### Change Log
