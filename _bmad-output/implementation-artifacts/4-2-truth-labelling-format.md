# Story 4.2: Freeze the truth-labelling format

Status: done

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

- [x] **Task 1 — the label types, in the domain** (AC: 1, 3)
  - [x] Add the expectation types to `crates/opencmdb-core` — pure data, `Serialize`/`Deserialize`, `#[serde(deny_unknown_fields)]`, no I/O. This mirrors 4.1 exactly: the TYPES live in core, the FILE READING lives in bin (D47).
  - [x] Model the three columns as one enum whose variants carry what that column needs — a decision carries a rule, an abstention carries an `AbstentionCause`. Do not model it as a struct with three optional fields; that permits states the domain forbids.
  - [x] `RuleId` as a newtype over `String` for now, with a comment recording that it closes into an enum when Epic 5 names the rules (architecture.md:2652 — *"an `Other(String)` cannot satisfy an `expect_rule`"*).

- [x] **Task 2 — the reason is mandatory** (AC: 2)
  - [x] A constructor or `validate()` that refuses an absent, empty or whitespace-only reason.
  - [x] Refuse a multi-line reason: "one sentence" is not checkable, but a paragraph is, and a reason that needs a paragraph is the `must-abstain` case D19 describes.

- [x] **Task 3 — parse and validate a trap file** (AC: 4, 6, 7)
  - [x] Extend `crates/opencmdb-bin/src/fixtures.rs` with a reader for the trap format, alongside `read_jsonl`. Reuse `FixtureError` and the single `FIXTURES_DIR` constant — do NOT add a second path expression (the story-4.1 review added a test that fails if you do).
  - [x] Cross-check: every `obs_id` an expectation references must exist in the replay stream it names. A dangling reference is refused.

- [x] **Task 4 — the committed example** (AC: 5)
  - [x] Write `fixtures/scenario/traps/example.toml` exercising all three columns against the `obs_id`s already in `fixtures/scenario/replay/minimal.jsonl`.
  - [x] Synthetic values only — same rule as 4.1, same reason (public repository).
  - [x] `sha256sum` from inside `fixtures/` and append to `fixtures/MANIFEST`; confirm the gate reports 2 fixtures.

- [x] **Task 5 — tests and gates** (AC: 7, 8)
  - [x] One test per refusal in AC7, each asserting the message names the offending trap or field.
  - [x] ~~Round-trip the committed example against its bytes~~ — **not possible and not done**: the trap file carries a comment header that `toml::to_string` cannot reproduce. Replaced by a SEMANTIC round-trip (`a_trap_file_survives_a_serde_round_trip`) plus a test pinning `AbstentionCause`'s committed spelling. This box was ticked before the test existed; the review caught it.
  - [x] Run the full gate set.

### Review Findings

_Code review 2026-07-21 — three parallel layers. 6 of 8 ACs satisfied, AC2 and AC7 partial. No scope violations; D47, D56, D19-mechanics and privacy all clean. Five Dev Agent Record claims verified TRUE._

- [x] [Review][Patch] **A `must-not-merge` names the rule that OPPOSES the merge** — decided 2026-07-21 (Guy), option 1 of three. Not the absence of a rule (a verdict without one is undebuggable in production, and "the engine found nothing" cannot be distinguished from a bug), and not the rule that was merely tempting. The engine must therefore be able to say *why* it refuses, and the three columns stay symmetric — every decision names a rule, only an abstention names a cause. Document the semantics on the variant, and rewrite the example accordingly. [crates/opencmdb-core/src/trap.rs:107]

- [x] [Review][Patch] **The committed example is factually contradicted by the stream it references** — `…0001` carries MAC `…83:01`, `…0003` carries `…83:02` (different), yet the reason claims "the same interface"; and the `must-not-merge` names an exact-MAC rule against `…0002`, which carries no `Mac` fact at all. A sha-locked oracle that is wrong, and the template fifty traps will copy [fixtures/scenario/traps/example.toml]
- [x] [Review][Patch] The round-trip subtask is checked but no such test exists; `Serialize` is derived on five types and never exercised. Byte-for-byte is impossible (the file carries comments) — say so and assert a semantic round-trip instead of ticking the box [this file:70]
- [x] [Review][Patch] The module doc claims both non-negotiables are "enforced by the types rather than by discipline" — true for the columns, false for the reason, which is a `pub String` checked by a `validate()` a caller may never call [crates/opencmdb-core/src/trap.rs:12]
- [x] [Review][Patch] An ABSENT `reason` key fails in serde before `validate()` runs, so the message names the field but not the trap — AC2 requires it name the trap — and no test covers the absent case, only the empty one [crates/opencmdb-core/src/trap.rs:145]
- [x] [Review][Patch] `TrapId` is never validated: `id = ""` passes, which defeats the stated purpose of `DuplicateId` and renders failures as ``trap ``: …`` [crates/opencmdb-core/src/trap.rs:153]
- [x] [Review][Patch] `RuleId` is never validated: `rule = ""` satisfies "a decision names its rule", the premise the whole format rests on [crates/opencmdb-core/src/trap.rs:153]
- [x] [Review][Patch] An empty, comment-only or `trap = []` file reads as `Ok` with zero traps — indistinguishable from a file whose traps were deleted, while counting as a locked fixture [crates/opencmdb-core/src/trap.rs:100]
- [x] [Review][Patch] A trap may list the same `obs_id` twice; a `must-merge` over `[x, x]` asserts an observation merges with itself and can never distinguish a correct engine from a broken one [crates/opencmdb-core/src/trap.rs:166]
- [x] [Review][Patch] The one-sentence rule is bypassed three ways: `reason = "."` passes, a 5000-character single line passes, and TOML's `\`-continuation collapses a real paragraph into one line [crates/opencmdb-core/src/trap.rs:159]
- [x] [Review][Patch] A lone `\r` or U+2028 is not counted as a line break, so a reason an editor shows as two lines validates as one [crates/opencmdb-core/src/trap.rs:159]
- [x] [Review][Patch] Ids differing only in case or surrounding whitespace (`"T"` vs `"t "`) are accepted as distinct — exactly the harm `DuplicateId` exists to prevent [crates/opencmdb-core/src/trap.rs:181]
- [x] [Review][Patch] Two of AC7's five refusal tests assert only the error kind, never the message, contradicting the story's own Dev Notes [crates/opencmdb-bin/src/fixtures.rs:493]
- [x] [Review][Patch] `ReasonMissing`'s message tells the author to reclassify as `must-abstain` when the actual defect is a blank field; the test that "checks" it asserts a substring that is a constant of the message [crates/opencmdb-core/src/trap.rs:120]
- [x] [Review][Patch] `FixtureError::Toml`'s Display renders a bare path plus serde text, never saying a TRAP FILE failed to parse [crates/opencmdb-bin/src/fixtures.rs:104]
- [x] [Review][Patch] `replay` is never validated as non-empty or as a file: `replay = ""` surfaces as `Is a directory (os error 21)`, blaming I/O for a malformed trap field and naming no trap [crates/opencmdb-core/src/trap.rs:153]
- [x] [Review][Patch] A `./`-prefixed replay passes `fixture_path`, defeats the stream cache (keyed on the raw string) and never matches a MANIFEST path form [crates/opencmdb-bin/src/fixtures.rs:187]
- [x] [Review][Patch] `obs_id`s are compared as freshly-allocated `String`s on both sides, coupling correctness to `Uuid`'s `Display`; a `BTreeSet<Uuid>` compares 16 bytes [crates/opencmdb-bin/src/fixtures.rs:189]
- [x] [Review][Patch] `DuplicateId`'s message claims an id "must name exactly one trap" but uniqueness is file-scoped; two files may both declare the same id [crates/opencmdb-core/src/trap.rs:136]
- [x] [Review][Patch] The committed corpus pins `AbstentionCause`'s PascalCase representation with no test naming the coupling; a later `rename_all` on that enum would break the corpus and its hash [fixtures/scenario/traps/example.toml]
- [x] [Review][Patch] `the_committed_trap_file_reads_and_cross_checks` asserts an exact column ORDER while its message claims coverage, and asserts nothing about rules, reasons or counts [crates/opencmdb-bin/src/fixtures.rs:450]
- [x] [Review][Patch] The File List omits `Cargo.lock`, which the change modifies [this file]

- [x] [Review][Defer] No gate parses trap files: a trap file added without a MANIFEST line is neither hashed nor parsed nor cross-checked, and the gate should also run `read_traps` over every `scenario/traps/*.toml` [xtask/src/main.rs:403] — deferred to story 4.3, which reworks the gate; the story explicitly forbade touching it here
- [x] [Review][Defer] Duplicate `obs_id`s inside a replay stream are undetected, voiding the "never by line number" guarantee the labelling rests on [crates/opencmdb-bin/src/fixtures.rs:139] — deferred, corpus integrity is 4.3 and this story was told not to
- [x] [Review][Defer] Validation stops at the first offending trap; with fifty traps this is a fix-one-rerun loop, and per-trap errors shadow `DuplicateId` [crates/opencmdb-core/src/trap.rs:180] — deferred, an error-collecting pass belongs with the trap runner (4.7)

_Dismissed as noise (2): the claim that `toml` was not already a dependency — verified false, `toml v0.8.23` was in `opencmdb-bin`'s normal graph before this change; and suspected `deny_unknown_fields` blind spots on an externally-tagged enum — empirically probed in five positions, all refused._

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

Claude Opus 4.8 (1M context) — `claude-opus-4-8[1m]`

### Debug Log References

- `cargo xtask ci` now reports `2 fixture(s) match their recorded sha256` — the corpus has an
  observation stream and a labelling, both locked.
- 14 tests in `fixtures.rs` (was 8) and 6 in `trap.rs`; workspace totals 38 + 36 + 23.

### Completion Notes List

- **The three columns are an enum, not a struct of optional fields.** "A merge that also names an
  abstention cause" is therefore *unrepresentable* rather than merely invalid — the parse fails,
  and no validator has to remember to check it. A test pins that: `a_decision_carrying_an_abstention_cause_is_refused`.
- **`must-abstain` reuses the domain's `AbstentionCause`** rather than a parallel string
  vocabulary. Had it invented one, the trap runner in 4.7 would eventually compare two enums that
  had drifted apart, and the drift would look like an engine bug.
- **Externally-tagged serde, deliberately.** Internally-tagged and `flatten` both break
  `deny_unknown_fields` in serde; external tagging keeps the strictness story 4.1 established, and
  TOML renders it readably as `expect = { must-merge = { rule = "…" } }`.
- **The dangling-reference check is the load-bearing part of the reader.** A trap pointing at an
  `obs_id` its stream does not contain can never fire, and would sit in the corpus *looking like
  coverage* — while the gate counts traps. It resolves each replay stream once, not once per trap.
- **`replay` goes through `fixture_path`**, so a trap file cannot reach outside the corpus through
  its own field either. That containment came from the story-4.1 review; this story would have
  reopened the hole without it.
- **"One sentence" is not checkable and the code does not pretend it is.** What is checkable is an
  absent reason and a reason that has become a paragraph — and D19 says a case needing a paragraph
  is the ambiguous one that belongs in `must-abstain`. The error message says so.
- **TOML, not the architecture's YAML** — decided by Guy on the evidence recorded in the Dev
  Notes. The architecture diagram was corrected in the same session (commit `da23f9f`).

### File List

- `crates/opencmdb-core/src/trap.rs` (new)
- `crates/opencmdb-core/src/lib.rs` (modified — module declaration)
- `crates/opencmdb-bin/src/fixtures.rs` (modified — `read_traps`, three error variants, six tests)
- `crates/opencmdb-bin/Cargo.toml` (modified — `toml` dependency)
- `fixtures/scenario/traps/example.toml` (new)
- `fixtures/scenario/README.md` (modified — documents `traps/`)
- `fixtures/MANIFEST` (modified — the trap file is locked)
- `fixtures/scenario/replay/example-traps.jsonl` (new — review patch: the example needed a stream containing a genuinely mergeable pair)
- `Cargo.lock` (modified)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modified)

### Change Log

- 2026-07-21 — Froze the truth-labelling format: the three D18 columns as an enum carrying exactly
  what each needs, a mandatory one-sentence reason enforced at parse time, `expect_rule` on
  decisions and `AbstentionCause` on abstentions, a committed TOML example exercising all three
  columns, and a reader that refuses a trap judging an observation its stream does not contain.
- 2026-07-21 — Code review (three parallel layers): 22 patches applied, 3 deferred, 2 dismissed on
  evidence. The one that mattered: **the committed example was factually contradicted by the
  stream it referenced** — it claimed two observations shared a MAC when the bytes showed
  `…83:01` and `…83:02`, and named an exact-MAC rule against an observation carrying no MAC at
  all. A sha-locked oracle that was wrong, and the template fifty traps would have copied. Fixed
  by giving the example its own purpose-built stream containing a genuinely mergeable pair, and
  by rewriting all three reasons to be true of the bytes. Guy decided that a `must-not-merge`
  names the rule that OPPOSES the merge, which is what made the second trap writable at all.
  Validation gained id, rule, replay, duplicate-observation, empty-file and reason-quality
  checks; `./` paths are refused; `obs_id`s compare as 16 bytes rather than formatted strings.
