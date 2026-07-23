# Story 4.6b: The harness runs the corpus, and cannot hide its own vacuity

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author of the release gate,
I want a harness that reads the committed trap corpus, scores it, and publishes the number — green when nothing scored, and visibly so,
so that the gate exists before the engine and cannot be mistaken for a passing gate when it has measured nothing.

## Context

**Depends on 4.6a**, which built the scored outcome, the record and the pure pass/fail algebra. This story gives the algebra a corpus to chew and a way to report. **4.6c** adds run comparability.

The vacuity problem is the whole story. With no engine, nothing produces an outcome, so nothing is scored, so failures = 0 — and *"0 failures"* is exactly what a fully-passing gate says. Story 4.1 exists because `no fixtures — skipped` was a vacuous green nobody could see; this story must not reintroduce it one storey up.

**Explicitly OUT of scope:** the algebra and the record (4.6a) · run comparability (4.6c) · any engine · asserting `(outcome, rule)` per trap (4.7) · the cross-stream `obs_id` guard (pulled out of the 4.6 family — corpus hygiene, not metrics) · wiring the number into `cargo xtask ci` (AC7).

## Decision — where the harness lives, and how it is reached

The architecture places it at `xtask/src/gen_metrics.rs` [architecture.md:3454]. Three facts make that placement unusable today:

1. **The corpus reader is in `bin`.** `read_traps` is `crates/opencmdb-bin/src/fixtures.rs:626`. `xtask` would have to depend on `opencmdb-bin`, dragging sqlx/axum/askama into the dev-tool runner. D56 makes xtask *"a dependency of NOBODY"*; it says nothing about the reverse, and the reverse has never been sanctioned.
2. **`xtask ci`'s own gate list does not contain the trap suite** [architecture.md:3530-3534]. The product's only gate is absent from the command that enumerates the gates.
3. **The whole corpus reader is dev-only by construction.** `fixtures.rs:32` is `#![allow(dead_code)]` and `FIXTURES_DIR` (`:48`) is baked from `CARGO_MANIFEST_DIR` at compile time — **a harness compiled into the shipped binary would read a path that exists on no deployed machine.**

⚠️ **And `walk_replay_streams` is `#[cfg(test)]`** — it lives at `fixtures.rs:1237`, inside the `mod tests` opened at `:667`. It is **not callable from a production module.** An earlier draft of this story told the dev to call it from one; that does not compile.

**Decision: the harness is a `pub` module in `opencmdb-bin` that takes its corpus root as a parameter, and its entry point is exercised by tests.** Taking the root as an argument — rather than reaching for `FIXTURES_DIR` — is what makes AC6's prove-to-red possible at all, and it is the shape every `xtask` gate already uses (`gate_fixture_manifest(root: &Path)`, `xtask/src/main.rs`, proven to red against scratch trees at `:1270`).

Publishing the number from `cargo xtask ci` is **not** this story: it needs either an xtask→bin dependency or moving the reader, and both are real decisions (AC7 records the obstacle).

## Acceptance Criteria

### Must / must not

- **MUST** report three numbers: traps **discovered**, traps **scored**, and **failures**.
- **MUST NOT** be satisfiable by a function that never opens the corpus.
- **MUST** take the corpus root as a parameter, not from the baked constant.
- **MUST NOT** require an engine, an engine parameter, or a stub engine to run.

---

1. **The harness runs against the committed corpus with no engine at all, and is green.** It discovers the traps, scores none of them (nothing produces an outcome), and reports failures = 0. It must not take an engine, an `Option<Engine>`, or a `NullEngine`: *"the metrics harness BEFORE the engine"* is the build order, and a harness shaped to accept one has already been shaped by the thing it judges.
   **A null engine that abstains on everything would be RED, not green** — D18: *"an engine that abstains on everything scores false-merge = 0 and gets **demolished by the middle column**"*. Vacuously green means **nothing was scored**, never "everything was scored by an abstainer".

2. **Three numbers, and `discovered` is the one that makes the other two honest.** `{discovered, scored, failures}`. Without `discovered`, `Report { scored: 0, failures: 0 }` — returned by a function with an empty body — satisfies every other criterion in this story. **A test must assert `discovered > 0` against the committed corpus** (the shape the two existing corpus walks already use: `assert!(checked > 0, …)`).
   This does not violate D18's *"one number"*. D18 forbids a **fraction as a quality score** — *"any fraction is theatre"*; it does not forbid stating what was looked at. The precedent is in this repo: the fixtures gate reports `5 fixture(s) match their recorded sha256`.

3. **The report says plainly that nothing was scored.** A human reading the output must not be able to mistake the vacuous state for a passing gate. `0 failures` and `0 scored` on the same line, in words.

4. **The harness takes its corpus root as a parameter.** No `FIXTURES_DIR`, no `fixture_path` inside the harness's own entry point. This is what lets AC6 point it at a scratch corpus; it is also the shape every existing gate uses.

5. **What the harness does with an unscored trap is stated, and tested.** With no outcome producer, a discovered trap is **discovered and not scored** — it produces no record. Do not emit a record with an absent outcome; do not silently drop the trap from `discovered`. One test pins the distinction.
   Consequence to state in the module doc: **4.6a's record type is therefore exercised only by hand-built values until an engine exists.** That is expected, not a gap.

6. **The gate can be shown to fail, and the demonstration is at the harness level.** 4.6a proves the algebra's cells. Here, drive the harness over a **scratch corpus** whose traps are paired with outcomes that contradict them — one per D18 column — and assert a failure is counted in each. *"A gate that cannot be shown to fail is decoration"* [architecture.md:5117].
   **Use what exists**: `scratch_dir(tag)` (`fixtures.rs:950`) and the scratch-corpus pattern `xtask` already uses for its own gates (`gate_reds_on_an_orphan_on_disk`, `xtask/src/main.rs:1270`). Do not invent a temp-dir helper.

7. **The register records what this story did not do** — `## Deferred from: story-4.6b (2026-07-22)`, appended, never rewriting a bullet:
   - **The number is not published by `cargo xtask ci`.** State the obstacle (xtask cannot reach `bin`'s reader) and name the two candidate resolutions — an xtask→bin dependency, or moving the corpus reader — choosing neither.
   - **The corpus reader is dev-only by construction** (`#![allow(dead_code)]`, a compile-time `FIXTURES_DIR`), so the harness cannot ship in the binary as things stand. Say so plainly.
   - **Two committed streams are judged by no trap** (`partial-then-failed.jsonl`, `capability-downgrade.jsonl`). Expected, owned by 4.7; recorded so nobody "fixes" it here.

8. **`clippy --all-targets -- -D warnings` stays green.** A `pub` item in a **binary** crate that nothing calls trips `dead_code`. `fixtures.rs` survives only because of its module-level `#![allow(dead_code)]` with a stated reason — mirror that, with a reason, or arrange a caller.

9. **Nothing is written under `fixtures/`.** That directory is the ORACLE, locked as data. A generated score inside the lockfile-for-data would make the gate's input and output indistinguishable. If a scratch corpus is needed, it goes in a scratch dir.

10. **All gates green, locally:** `cargo fmt --all` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo test --workspace` · `cargo xtask ci`. **Do not regenerate `architecture-views.md`.**

## Tasks / Subtasks

- [x] **Task 1 — the harness module** (AC: 1, 2, 3, 4, 8)
  - [x] A new module in `crates/opencmdb-bin/src/`. ⚠️ **`metrics` is taken** — `crates/opencmdb-bin/src/metrics.rs` is the Prometheus handler (`main.rs:14`). Name it something that cannot be confused with it, and add the `mod` line alphabetically.
  - [x] Entry point takes the corpus root; discovers trap files, reads them through `read_traps`, returns `{discovered, scored, failures}`.
  - [x] `#![allow(dead_code)]` with a stated reason, as `fixtures.rs` and `arp_ping.rs` do.

- [x] **Task 2 — discovery** (AC: 2, 4)
  - [x] Walk the trap directory under the given root. ⚠️ `walk_replay_streams` and the trap walk are both `#[cfg(test)]` (`fixtures.rs:667` opens the test module). **Decide and state**: promote one to `pub(crate)` taking a root, or write the harness's own walk. If you promote, the two existing tests that call it move too — name them in the File List.
  - [x] Recursive; refuses symlinks and foreign extensions; does not swallow read errors. *"Walks that quietly see less"* were the recurring defect of 4.1 and 4.3.

- [x] **Task 3 — the tests** (AC: 1, 2, 3, 5, 6)
  - [x] The committed corpus: `discovered > 0`, `scored == 0`, `failures == 0`, and the report says so.
  - [x] AC6's three scratch-corpus failures, one per D18 column.
  - [x] AC5's discovered-but-not-scored distinction.
  - [x] **Prove-to-red on every new guard**, each mutation recorded. Offending item **second**, behind a valid one. **Do not write a comment asserting a coverage property you did not measure.**

- [x] **Task 4 — the record and the gates** (AC: 7, 9, 10)
  - [x] Append the three entries of AC7 to `deferred-work.md`.
  - [x] Update `sprint-status.yaml`; put it in the File List.
  - [x] Run the four gates. **Name the command behind every claim in the completion record.**

### Review Findings

_Code review 2026-07-22 — three parallel layers (Blind Hunter, Edge Case Hunter, Acceptance Auditor). **All 10 ACs SATISFIED**, no violations; every measured claim in the completion record independently reproduced, including all four mutations. The auditor read D19 and judged the "answers map is not an engine" reasoning **sound, not a rationalisation**: the harness's shape is fixed by 4.6a's algebra, so a future engine must conform to `Outcome` — the metric is not bent to fit the engine, the engine must fit the metric. The Edge Case Hunter reproduced all four symlink cases (file, directory, loop, broken) and confirmed the walk errors before any recursion — no infinite loop, no escape. What follows is everything else._

- [x] [Review][Patch] **The symlink guard — the one the module doc shouts about — has NO test.** It is CORRECT (the Edge Case Hunter reproduced all four cases), but "prove-to-red on every new guard" was claimed and this guard was not proven. A `return Err` no test exercises can silently become `continue`. Add a scratch-tree test with a symlink and assert the walk errors [crates/opencmdb-bin/src/trap_gate.rs:169]
- [x] [Review][Patch] **Cross-file duplicate `TrapId` scores one answer against two traps.** Found independently by two layers. `read_traps` dedups ids per FILE only; two files under one root both defining `id = "X"`, with an answer for `X`, score both — potentially in two different columns from one outcome. This is the exact mirror of the cross-stream `obs_id` guard 4.6a already made me record for observations, unseen here for traps. `score_corpus` must detect a duplicate `TrapId` across the discovered corpus and error [crates/opencmdb-bin/src/trap_gate.rs:129]
- [x] [Review][Patch] **An empty-but-present trap directory returns a vacuously-green `{0,0,0}`.** A MISSING root errors; an existing empty (or trap-less) directory returns `Ok` with `failures == 0`. `discovered` only annotates `Display` — nothing in `score_corpus` reds a run that discovered nothing, so the vacuity is caught solely by a test of the corpus, not by a property of the gate. `score_corpus` should refuse to report a run that discovered zero traps, or expose a `passed()` predicate that is false when `discovered == 0` [crates/opencmdb-bin/src/trap_gate.rs:122]
- [x] [Review][Patch] **Answers with no matching discovered trap are silently dropped.** A producer id that is renamed, typo'd or stale emits an outcome the gate ignores and never reports — which contradicts the module's own "swallows nothing" ethos. `score_corpus` should surface unused answer keys (count them, or error) [crates/opencmdb-bin/src/trap_gate.rs:131]
- [x] [Review][Patch] **`Report` has no gate predicate; "the number that blocks is `failures()`" lives only in prose.** The next caller must reconstruct the pass rule and could gate on `scored() == 0` or a fraction just as easily. Add `passed()` (false when `discovered == 0` OR `failures > 0`, closing the empty-corpus hole above) so the gate's verdict is a method, not a comment [crates/opencmdb-bin/src/trap_gate.rs:69]
- [x] [Review][Patch] **The "answers map cannot be bent by an engine" claim is TEMPORAL, stated as STRUCTURAL.** It holds because the map is empty today; when Epic 5 populates it from engine output, that is the "metric computed from engine output" D19 warns against. The real structural guarantee is narrower and true — the harness never CALLS a producer, and `Outcome`'s shape is fixed by 4.6a so the engine conforms to the metric, not the reverse. State that, not "cannot be bent" [crates/opencmdb-bin/src/trap_gate.rs:8]
- [x] [Review][Patch] **No test mixes a passing and a failing answer in one run.** One test is all-green, one all-red; the discriminating case — a correct answer stays out of `failures` while a wrong one enters it, in the same corpus — is never exercised, and the story's own "offending item second, behind a valid one" convention is not applied to the red test [crates/opencmdb-bin/src/trap_gate.rs:181]
- [x] [Review][Patch] **`the_walk_exempts_readme_at_any_depth` tests depth 0; recursion is tested nowhere.** No test creates a nested subdirectory, so the recursive descent and the "at any depth" / "recursive" claims are unexercised. The recursion could break and every test stay green [crates/opencmdb-bin/src/trap_gate.rs:210]
- [x] [Review][Patch] **`a_missing_root_is_an_error_not_an_empty_result` asserts only `matches!(err, Io { .. })`, which nearly every error satisfies.** It also matches a symlink refusal and a foreign-extension refusal. A read error and an authoring mistake are indistinguishable to a caller — worth a distinct signal, or at least an assertion on the message [crates/opencmdb-bin/src/trap_gate.rs:230]

- [x] [Review][Defer] **A `traps_root` that is ITSELF a symlink is followed.** `read_dir(root)` follows a symlink at the path; the guard only inspects entries [crates/opencmdb-bin/src/trap_gate.rs:154] — deferred, low reachability: the root is supplied by the harness's caller (a test or, later, CI config), not discovered from the corpus, so it is not an author-controlled attack surface the way a corpus entry is. Revisit if the root ever comes from untrusted config.
- [x] [Review][Defer] **A stray dot-file (`.DS_Store`) or `readme.md` (wrong case) reds the whole gate.** `.toml` is matched case-insensitively but `README.md` exactly, and an extensionless file falls into the foreign-extension error [crates/opencmdb-bin/src/trap_gate.rs:210] — deferred. The strictness is defensible for a spec corpus (an unexplained file IS a finding), but the asymmetry should be a deliberate, stated choice; fold it into the duplicate/empty-corpus patch rather than a separate pass.
- [x] [Review][Defer] **Scratch dirs leak on a failing assertion, and are not cleared before use** [crates/opencmdb-bin/src/trap_gate.rs:203] — deferred, pre-existing pattern. Every test in the repo cleans up only on the success path (`fixtures.rs` does the same); a `Drop` guard would fix the whole codebase at once and is not this story's to introduce. PID-keyed tags differ per test, so no in-run collision today.

_Dismissed as noise (2): `found.sort()` buys determinism for a value no caller observes (harmless, and cheap insurance if a future caller does consume the order) · unused `#[derive(Clone, PartialEq, Eq)]` on `Report` (a value type that will be compared/cloned the moment 4.6c consumes it; keeping it is cheaper than re-adding it)._

## Dev Notes

### What already exists — use it, do not rewrite it

- **`read_traps`** [Source: crates/opencmdb-bin/src/fixtures.rs:626] — parses, validates, and cross-checks every trap's `obs_id`s against the stream it names. It calls `fixture_path` internally, so a trap's `replay` is resolved against the **baked** corpus root; if the harness is pointed at a scratch root, that interaction needs thought — say what you found.
- **`scratch_dir(tag)`** [Source: crates/opencmdb-bin/src/fixtures.rs:950] and **`read_scratch`** [:960] — the existing scratch-corpus helpers, both inside `mod tests`.
- **`gate_fixture_manifest(root: &Path)` and its prove-to-red tests** [Source: xtask/src/main.rs:408, :1270] — the house pattern for a gate that takes a root and is proven red against a scratch tree. This is the model.
- **The report line format** [Source: xtask/src/main.rs:93] — `report(name, ok, msg)`; the fixtures gate's message is the model for AC2's counts.
- **4.6a's algebra and record** — the scoring is done; this story feeds it and reports.

### Traps

1. **A harness that never opens the corpus.** AC2. The three-number report with a `discovered > 0` test is the only thing standing between this story and an empty function.
2. **Calling `walk_replay_streams` from production code.** It is `#[cfg(test)]`. An earlier draft of this story instructed exactly that.
3. **Naming the module `metrics`.** Taken.
4. **A null engine "so there is something to score".** RED by D18's middle column, and it makes the vacuous case unreachable.
5. **Claiming more than was measured.** Three consecutive completion records over-claimed. Name the command, or write the weaker true sentence.
6. **Skipping `--all-targets` or `xtask ci` locally.** Epic 3's retrospective recorded four CI-only failures from that.

### Git intelligence

`master` requires a pull request since 2026-07-22 (0 approvals, `ci` green, squash merge). Work on `story/4-6-metrics-harness`; **do not push to `master`**.

### Latest technical specifics

No new crate, no version bump. **Locked** (committed `Cargo.lock`, verified 2026-07-22): `serde 1.0.228`, `serde_json 1.0.150`, `toml 0.8.23`, `uuid 1.24.0`, `chrono 0.4.45` (`default-features = false`), `tokio 1.53.0`. Rust 1.96+, edition 2024, `resolver = "3"`. **Never invent a version.**

### Project Structure Notes

- **New:** one module under `crates/opencmdb-bin/src/` (not `metrics`).
- **Updated:** `crates/opencmdb-bin/src/main.rs` (one `mod` line), possibly `crates/opencmdb-bin/src/fixtures.rs` (only if a walk is promoted out of `mod tests` — and then its two existing callers), `deferred-work.md`, `sprint-status.yaml`.
- **Unchanged, expected:** `fixtures/` (AC9), `crates/opencmdb-core/`, `Cargo.lock`.
- **Variance from the architecture:** not at `xtask/src/gen_metrics.rs`. See the Decision; state it in the record rather than taking it quietly.

### References

- [Source: _bmad-output/planning-artifacts/epics.md:983-1003 — Story 4.6, which 4.6a/b/c jointly implement]
- [Source: _bmad-output/planning-artifacts/architecture.md:1224-1226 — "One number blocks: truth-table failures = 0"]
- [Source: _bmad-output/planning-artifacts/architecture.md:1229-1233 — the three columns and the anti-cowardice sentence AC1 rests on]
- [Source: _bmad-output/planning-artifacts/architecture.md:1212-1217 — the binomial argument: why any fraction is theatre]
- [Source: _bmad-output/planning-artifacts/architecture.md:1341-1346 — D19's build order: the harness BEFORE the engine]
- [Source: _bmad-output/planning-artifacts/architecture.md:3454, :3530-3534 — the `gen_metrics.rs` placement, and the gate list that omits the trap suite]
- [Source: _bmad-output/planning-artifacts/architecture.md:5117 — "a gate that cannot be shown to fail is decoration"]
- [Source: _bmad-output/planning-artifacts/prd.md:1160-1188 — NFR4 at product level]
- [Source: crates/opencmdb-bin/src/fixtures.rs:32, :48, :626, :667, :950, :1237 — the dev-only reader, the baked root, `read_traps`, the test-module boundary, `scratch_dir`, the test-only walk]
- [Source: xtask/src/main.rs:93, :408, :1270 — the report shape, a gate taking a root, and its prove-to-red]
- [Source: _bmad-output/implementation-artifacts/4-6a-scoring-algebra.md — the algebra and record this story drives]
- [Source: _bmad-output/implementation-artifacts/deferred-work.md — the register to append to]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.8 — `claude-opus-4-8[1m]`

### Debug Log References

Every claim below is a command that was run, not an inference.

- **Four guards proven to red**, each mutation applied and reverted from a pre-mutation backup:
  | Mutation | Tests reddened |
  |---|---|
  | `discovered += 1` → `+= 0` (the empty-body vacuity) | 5 tests, incl. every committed-corpus assertion |
  | the harness ignores `answers` (never scores) | `each_column_can_be_driven_red`, `a_trap_with_no_answer_is_discovered_but_not_scored` |
  | the walk swallows a foreign extension instead of refusing it | `the_walk_refuses_a_foreign_extension` |
  | the walk swallows a `read_dir` error (quietly sees less) | `a_missing_root_is_an_error_not_an_empty_result` |

  The second is the central guard: without it, an empty-body `score_corpus` would report `{0,0,0}`
  and pass — the vacuity AC2 exists to forbid. `each_column_can_be_driven_red` is what fails.
- **Gates:** `cargo fmt --all` clean · `cargo clippy --workspace --all-targets -- -D warnings` clean
  · `cargo build --workspace` clean (no `dead_code` warning — the module-level `#![allow(dead_code)]`
  carries it, as `fixtures.rs` does) · `cargo test --workspace` → **93 (bin, +7) + 59 (core) + 38
  (xtask), 0 failed**.
- `cargo xtask ci` → all gates green, `✅ fixtures 5/5`; `architecture-views.md` NOT regenerated.
- **`Cargo.lock` did not move** — `git diff --stat Cargo.lock` empty; no dependency added.
- **The MariaDB-backed tests did NOT run.** `DATABASE_URL` is unset; the four DB-backed tests return
  early and pass either way. These counts say nothing about the database.

### Completion Notes List

- **The harness scores answers, and never runs a producer — which is how AC1 and AC6 both hold.**
  `score_corpus(traps_root, answers: &BTreeMap<TrapId, Outcome>)`. A map of outcomes is DATA, not an
  engine: no `poll`, no trait to stub. It is empty for a real run today (nothing produces answers),
  so every discovered trap is discovered and not scored; a test supplies contradicting answers to
  drive the gate red. The metric therefore cannot be bent by an engine it never calls — D19's
  reason for building it first.
- **`discovered` is what makes the zeros honest.** The committed corpus has three traps; a real run
  reports `discovered 3, scored 0, failures 0`, and `Display` puts all three on one line so "0
  failures" can never read as a passing gate. An empty-body function reports `{0,0,0}` and fails
  `the_committed_corpus_is_discovered_and_scored_by_nothing` on `discovered > 0`.
- **The demonstration that the gate is red-able is at the harness level**, not just the algebra's: a
  scratch trap corpus, one trap per D18 column, each paired with a contradicting answer, produces one
  failure in each column. The scratch traps reference the committed `minimal.jsonl` so `read_traps`'
  obs_id cross-check still validates.
- **The walk is the harness's own, and it swallows nothing.** Recursive over `.toml`, refuses a
  symlink and a foreign extension, exempts `README.md` at any depth (matching the corpus lock's
  orphan rule so documenting a directory does not red the gate), and turns a `read_dir` failure into
  an error rather than a smaller result — the *"walks that quietly see less"* defect of 4.1/4.3. I
  wrote it rather than promote `fixtures.rs`'s `#[cfg(test)]` `walk_replay_streams`, which scans
  replay streams (not traps) and whose promotion would move two existing tests for no gain.
- **Naming:** module `trap_gate` (not `metrics`, taken by the Prometheus handler `metrics.rs`),
  placed between `fixtures` and `metrics` in `main.rs`; `score_corpus`, `discover_trap_files`,
  `Report`. A local `#[cfg(test)] scratch_dir` rather than promoting `fixtures.rs`'s — each test
  module keeps its own, the house convention.
- **`ScoredRecord` (4.6a) is not produced here**, and the module doc says so: the harness tallies;
  it does not persist a record per trap. That join is later work.
- **After the review, the harness refuses two corpus-integrity mismatches it silently tolerated:**
  a `TrapId` repeated across files (`DuplicateTrapId`, the mirror of the cross-stream `obs_id` rule
  4.6a already recorded for observations), and an answer for a trap that does not exist
  (`AnswerForUnknownTrap`). And `Report::passed()` requires `discovered > 0`, so an empty directory
  is vacuity rather than a green gate — while a real corpus with no engine yet still passes, which
  is AC1's defined green. The symlink guard and the recursion, correct but previously unproven, now
  have prove-to-red tests.
- **Four findings recorded in `deferred-work.md`**: the number not yet published by `xtask ci` with
  the two candidate resolutions; the reader being dev-only by construction; `read_traps` resolving
  `replay` against the baked root (so a scratch corpus references committed streams); and the two
  committed streams judged by no trap (owned by 4.7).

### File List

- `crates/opencmdb-bin/src/trap_gate.rs` (new — `score_corpus`, `discover_trap_files`, `Report`
  with a `passed()` gate predicate; 14 tests after the review's +7)
- `crates/opencmdb-bin/src/fixtures.rs` (modified by the review — two `FixtureError` variants,
  `DuplicateTrapId` and `AnswerForUnknownTrap`, with their `Display` and `source()` arms)
- `crates/opencmdb-bin/src/main.rs` (modified — one `mod trap_gate;` line)
- `_bmad-output/implementation-artifacts/deferred-work.md` (modified — four story-4.6b entries)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modified)
- `Cargo.lock` — **unchanged**, measured.

### Change Log

- 2026-07-22 — The metrics harness exists before the engine. `score_corpus` walks the trap corpus
  under a given root, reads each trap through `read_traps`, and scores it against a map of
  already-produced outcomes — DATA, not an engine, so the metric cannot be bent by a producer it
  never calls (D19). It reports `{discovered, scored, failures}`: with no answers it is green but
  visibly vacuous, and a scratch corpus with contradicting answers drives one failure into each of
  D18's three columns, proving the gate is red-able. The harness's own walk swallows nothing. Not
  wired into `cargo xtask ci` — that needs `xtask` to reach `bin`'s reader, recorded rather than
  forced.
