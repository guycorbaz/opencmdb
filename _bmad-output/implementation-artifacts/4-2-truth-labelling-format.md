# Story 4.2: Freeze the truth-labelling format

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author of the trap corpus,
I want the expectation format frozen alongside the observation stream,
so that a trap states which RULE must fire and why, not merely what the answer was.

## Context

Story 4.1 froze the *observations* — what a source saw. This story freezes the other half: **what the right answer is, and why**. Together they make a trap.

The decision that governs everything here is D19's: **the oracle is the fixture's author, made explicit and versioned.** There is no external truth to appeal to, so the format's whole job is to force the author to be explicit and to make a later reader able to disagree with them. Hence the two non-negotiables: a mandatory one-sentence `reason` on every expectation, and an expected **rule**, not merely an expected outcome.

> *"A test that checks only the verdict goes green for the right answer reached by the wrong rule — and that engine will break on the next fixture."* (D19)

**Explicitly OUT of scope:**
- The **trap runner** that executes expectations against an engine — story 4.7. There is no engine yet; this story produces a format and its parser, not a test that runs traps.
- **`FixtureConnector`** (4.4), the metrics harness (4.6), any actual trap family (4.9+).
- The **MANIFEST.toml** migration and orphan detection — story 4.3.
- Any change to the reconciliation engine in `gap/mod.rs`.

## Acceptance Criteria

1. **Three columns, exactly one per expectation.** The label is one of `must-not-merge`, `must-merge`, `must-abstain` (D18). The type makes a second column unrepresentable rather than validating against it.

2. **A one-sentence `reason` is mandatory and enforced.** An expectation without one, or with an empty/whitespace-only one, is REFUSED at parse time with a message naming the trap. D19: *"if Guy cannot write the reason in one sentence, the case is genuinely ambiguous → it becomes a `MustAbstain`. THE INABILITY TO STATE A REASON IS THE ABSTENTION LABEL."* The format cannot check that a sentence is true, but it can make its absence impossible to commit.

3. **A decision names its rule; an abstention names its cause.**
   - `must-merge` / `must-not-merge` carry `expect_rule` — the `rule_id` the engine must have fired.
   - `must-abstain` carries the expected `AbstentionCause`, reusing the existing domain enum (`OutOfPerimeter`, `NoObservedValue`, `ConflictingObservations`) rather than inventing a parallel vocabulary.
   - Neither is optional for its variant, and the type forbids carrying both.

4. **Unknown or misspelled fields are refused**, as story 4.1 established for observations (`#[serde(deny_unknown_fields)]`). A trap file that silently means something other than it says is worse than one that does not parse.

5. **One committed example trap**, at `fixtures/scenario/traps/`, exercising all three columns, listed in `fixtures/MANIFEST` with its sha256 so `cargo xtask ci` locks it. It is an EXAMPLE of the format, not a trap family — no family lands before 4.9.

6. **The expectation references the observations it judges** by the `obs_id`s already frozen in 4.1, so truth points at a stable identifier rather than at a line number that a later edit would silently shift.

7. **Tests prove the refusals, not just the happy path**: a missing reason, an empty reason, an unknown field, both a rule and a cause on one expectation, and a reference to an `obs_id` absent from the replay stream — each is refused, each names what is wrong.

8. **All gates green:** `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo xtask ci`.

## Tasks / Subtasks

- [ ] **Task 1 — the label types, in the domain** (AC: 1, 3)
  - [ ] Add the expectation types to `crates/opencmdb-core` — pure data, `Serialize`/`Deserialize`, `#[serde(deny_unknown_fields)]`, no I/O. This mirrors 4.1 exactly: the TYPES live in core, the FILE READING lives in bin (D47).
  - [ ] Model the three columns as one enum whose variants carry what that column needs — a decision carries a rule, an abstention carries an `AbstentionCause`. Do not model it as a struct with three optional fields; that permits states the domain forbids.
  - [ ] `RuleId` as a newtype over `String` for now, with a comment recording that it closes into an enum when Epic 5 names the rules (architecture.md:2652 — *"an `Other(String)` cannot satisfy an `expect_rule`"*).

- [ ] **Task 2 — the reason is mandatory** (AC: 2)
  - [ ] A constructor or `validate()` that refuses an absent, empty or whitespace-only reason.
  - [ ] Refuse a multi-line reason: "one sentence" is not checkable, but a paragraph is, and a reason that needs a paragraph is the `must-abstain` case D19 describes.

- [ ] **Task 3 — parse and validate a trap file** (AC: 4, 6, 7)
  - [ ] Extend `crates/opencmdb-bin/src/fixtures.rs` with a reader for the trap format, alongside `read_jsonl`. Reuse `FixtureError` and the single `FIXTURES_DIR` constant — do NOT add a second path expression (the story-4.1 review added a test that fails if you do).
  - [ ] Cross-check: every `obs_id` an expectation references must exist in the replay stream it names. A dangling reference is refused.

- [ ] **Task 4 — the committed example** (AC: 5)
  - [ ] Write `fixtures/scenario/traps/example.toml` exercising all three columns against the `obs_id`s already in `fixtures/scenario/replay/minimal.jsonl`.
  - [ ] Synthetic values only — same rule as 4.1, same reason (public repository).
  - [ ] `sha256sum` from inside `fixtures/` and append to `fixtures/MANIFEST`; confirm the gate reports 2 fixtures.

- [ ] **Task 5 — tests and gates** (AC: 7, 8)
  - [ ] One test per refusal in AC7, each asserting the message names the offending trap or field.
  - [ ] Round-trip the committed example and assert it re-serializes to the committed bytes, as 4.1 does for observations.
  - [ ] Run the full gate set.

## Dev Notes

### The format is TOML, not the YAML the architecture names

**Decided 2026-07-21 by Guy**, on the evidence below. `architecture.md:3327` and `:1342` specify `fixtures/scenario/traps/*.yaml`; that was written before the stack existed, and the ground has moved:

- **No YAML parser exists anywhere in the dependency tree** (`grep -E '^name = "(serde_yaml|serde_yml|serde_norway)"' Cargo.lock` → nothing).
- **`serde_yaml` is archived and unmaintained** by its author since 2024. This project runs `cargo deny check advisories` in CI with `version = 2` (`deny.toml`), so pulling in an unmaintained crate is at best noise in the audit and at worst a red gate.
- **`toml` is ALREADY in the shipped binary's normal dependency graph** (9 occurrences, via the `config` crate), so taking it as a direct dependency adds nothing to the supply chain — and D56 already chose TOML for `MANIFEST.toml`, so the corpus ends up with one metadata format instead of two.

The argument that lost: YAML's nested-list ergonomics are genuinely nicer for ~50 hand-written traps. TOML's array-of-tables (`[[trap]]`) is the shape to use; it stays readable, and a trap that becomes unreadable in TOML is usually a trap doing too much.

Nothing else in this story depends on the choice — the types, the validation and the tests go through serde and are format-agnostic. Add `toml` as a direct dependency of `opencmdb-bin` only (the domain crate parses nothing; D47).

### ⚠️ `fixtures/verdicts/expected.jsonl` in the architecture tree is DEAD — do not create it

The same directory diagram shows `fixtures/verdicts/expected.jsonl # D46b — the (verdict, rule) join`. **D46b killed that join**: *"the JSONL verdict join. It has nothing left to join … three of its four rows were engine-disagreement rows"* — it existed to compare two database backends, and D64 left only MariaDB. What survived is AC-1, and it changed owner:

> *"Compare `(verdict, rule)`, never `verdict` alone … it becomes `assert_eq!(decision.rule, case.expect_rule)` in the trap runner, where Murat says it should have lived from the start."*

So the surviving requirement is **this story's `expect_rule` field**, consumed by the trap runner in 4.7. Creating a `fixtures/verdicts/` directory would be building a dead decision.

### The domain vocabulary already exists — reuse it

`crates/opencmdb-core/src/gap/mod.rs:26-34` defines `AbstentionCause` with `OutOfPerimeter`, `NoObservedValue`, `ConflictingObservations`, and `Reconciliation` carries `abstentions: BTreeMap<AbstentionCause, usize>`. A `must-abstain` expectation must name one of these, not a new string. Inventing a parallel abstention vocabulary here would guarantee a translation layer later, and the trap runner would compare two enums that drifted apart.

### What story 4.1 established, and what its review changed

Read `_bmad-output/implementation-artifacts/4-1-jsonl-observation-stream.md` — its Dev Agent Record and Review Findings are the most relevant prior art. The load-bearing points:

- **Types in `opencmdb-core`, file reading in `opencmdb-bin`.** D47: the domain crate cannot read files. Follow the same split.
- **`#[serde(deny_unknown_fields)]` is now on `Observation`, `Fact`, `Scope`, `Capabilities`** (added by the 4.1 review, on Guy's explicit decision that the hole was not worth carrying into 19 more stories). Apply it to the new types from the start.
- **One path expression only.** `fixtures.rs` holds `FIXTURES_DIR`, and `the_fixtures_path_is_expressed_once` walks `crates/*/src` and `xtask/src` counting *occurrences*. A second copy makes it red — that is deliberate, and it is proven to red.
- **`fixture_path` returns `Result` and refuses absolute paths and `..`.** Use it; do not build paths by hand.
- **Tests get a private scratch directory** via `scratch_dir(tag)` (pid-suffixed, cleaned up). A shared constant temp path was a review finding — do not reintroduce one.
- **Synthetic data is enforced by a test that reads the committed file** and is exhaustive over `Fact`. If a trap file ever carries addresses, it needs the same treatment.

### Deferred items that touch this story — do NOT fix them here

`_bmad-output/implementation-artifacts/deferred-work.md` records six findings from the 4.1 review. Two are adjacent and will tempt you:
- **Orphan fixtures are still undetected** — a file added under `fixtures/` without a MANIFEST line is green. That is story 4.3. Add your MANIFEST line by hand and move on.
- **Duplicate `obs_id`s in a replay stream are not rejected.** Your cross-check (Task 3) reads the stream; it may be tempting to also dedupe it. Do not — corpus-integrity work is 4.3, and mixing it here makes both stories harder to review.

### Testing standards

- Unit tests in `#[cfg(test)] mod tests` at the foot of the module, the convention throughout.
- Tests that read committed fixtures are ordinary tests — no gate, no env var. The files are committed.
- **Prove the refusals.** A validation story whose tests only exercise the happy path has proved nothing: the entire value of this format is what it REFUSES. Each negative test should assert the message names the offending trap or field, not merely that an error occurred.
- Full local gate before done: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo xtask ci`.

### Project Structure Notes

- Trap files: `fixtures/scenario/traps/` (new directory; `fixtures/scenario/README.md` already describes `replay/` and should gain a line for `traps/`).
- Types: a new module in `crates/opencmdb-core/src/` — `trap.rs` or similar, declared in `lib.rs`. Do not put them in `observation/`; an expectation is not an observation.
- Reader: extend `crates/opencmdb-bin/src/fixtures.rs`. Do not create a second fixtures module.
- The `views-hash` gate reporting `ℹ STALE` is expected and does not fail the build. Do not "fix" it.

### References

- [Source: architecture.md#D19 — the oracle, the mandatory reason, `expect_rule`, the fixture as spec]
- [Source: architecture.md#D18 — the three columns and what each guards against]
- [Source: architecture.md:3327 — the fixtures directory diagram (partly stale, see above)]
- [Source: architecture.md:4443 — D46b, the dead verdict join and the surviving `(verdict, rule)` rule]
- [Source: architecture.md:2652 — why `expect_rule` argues for a closed rule enum]
- [Source: epics.md#Story 4.2 — the five acceptance criteria this story expands]
- [Source: crates/opencmdb-core/src/gap/mod.rs:26-34 — `AbstentionCause`, the vocabulary to reuse]
- [Source: crates/opencmdb-bin/src/fixtures.rs — `FIXTURES_DIR`, `fixture_path`, `FixtureError`, `scratch_dir`]
- [Source: _bmad-output/implementation-artifacts/4-1-jsonl-observation-stream.md — prior art and review findings]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
