# Story 4.6b: The harness runs the corpus, and cannot hide its own vacuity

Status: ready-for-dev

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

- [ ] **Task 1 — the harness module** (AC: 1, 2, 3, 4, 8)
  - [ ] A new module in `crates/opencmdb-bin/src/`. ⚠️ **`metrics` is taken** — `crates/opencmdb-bin/src/metrics.rs` is the Prometheus handler (`main.rs:14`). Name it something that cannot be confused with it, and add the `mod` line alphabetically.
  - [ ] Entry point takes the corpus root; discovers trap files, reads them through `read_traps`, returns `{discovered, scored, failures}`.
  - [ ] `#![allow(dead_code)]` with a stated reason, as `fixtures.rs` and `arp_ping.rs` do.

- [ ] **Task 2 — discovery** (AC: 2, 4)
  - [ ] Walk the trap directory under the given root. ⚠️ `walk_replay_streams` and the trap walk are both `#[cfg(test)]` (`fixtures.rs:667` opens the test module). **Decide and state**: promote one to `pub(crate)` taking a root, or write the harness's own walk. If you promote, the two existing tests that call it move too — name them in the File List.
  - [ ] Recursive; refuses symlinks and foreign extensions; does not swallow read errors. *"Walks that quietly see less"* were the recurring defect of 4.1 and 4.3.

- [ ] **Task 3 — the tests** (AC: 1, 2, 3, 5, 6)
  - [ ] The committed corpus: `discovered > 0`, `scored == 0`, `failures == 0`, and the report says so.
  - [ ] AC6's three scratch-corpus failures, one per D18 column.
  - [ ] AC5's discovered-but-not-scored distinction.
  - [ ] **Prove-to-red on every new guard**, each mutation recorded. Offending item **second**, behind a valid one. **Do not write a comment asserting a coverage property you did not measure.**

- [ ] **Task 4 — the record and the gates** (AC: 7, 9, 10)
  - [ ] Append the three entries of AC7 to `deferred-work.md`.
  - [ ] Update `sprint-status.yaml`; put it in the File List.
  - [ ] Run the four gates. **Name the command behind every claim in the completion record.**

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

### Debug Log References

### Completion Notes List

### File List

### Change Log
