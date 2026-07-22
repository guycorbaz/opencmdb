# Story 4.6a: The scoring algebra and the scored record

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author of the release gate,
I want the truth table that says whether an answer is right, and the record that makes a verdict falsifiable, to exist as pure domain data before any engine does,
so that the metric cannot be bent to fit the engine.

## Context

epics.md defines one story 4.6, *"the metrics harness, written before the engine"* [epics.md:983-1003]. **It is split into three**, because a validation pass found the single story bundled a pure algebra, a file-reading harness and a run-comparison surface — three different review problems — and because the gate's semantics deserve their own review:

- **4.6a (this story)** — the scored outcome, the record, and the pass/fail algebra. **`opencmdb-core` only. Zero I/O.**
- **4.6b** — the harness that reads the corpus, scores it, and reports without hiding vacuity.
- **4.6c** — run comparability under an identical capability snapshot.

This is the story the build order exists for. D19: *"the metrics harness BEFORE the engine — **a metric written after the engine is bent to fit the engine**"* [architecture.md:1341-1346].

**Explicitly OUT of scope:** any engine, rule or identity logic (Epic 5+) · reading the corpus, any file, any path (4.6b) · comparing two runs (4.6c) · asserting `(outcome, rule)` per trap (**4.7** — D64 moved that criterion to the trap runner, [architecture.md:4457]) · building `SourceState` (Epic 13) · the cross-stream `obs_id` guard (pulled out of this story entirely — it is corpus hygiene, not metrics).

## The pass/fail matrix — the gate itself

D18's table names **one** failure condition per column [architecture.md:1229-1233]. The 3×3 has nine cells. **All nine are decided here**, because whatever the implementation picks silently becomes the release gate's semantics and no review could call it wrong.

| expected \ scored | **Merged** | **Refused** | **Abstained** |
|---|---|---|---|
| **must-merge** | ✅ pass | 🔴 **fail** | 🔴 **fail** |
| **must-not-merge** | 🔴 **fail** | ✅ pass | ✅ **pass** |
| **must-abstain** | 🔴 **fail** | 🔴 **fail** | ✅ pass |

Four cells come straight from D18. The other five need their reason written down:

- **`(must-merge, Abstained)` → fail.** D18's named case: *"cowardice — an engine that abstains on everything scores false-merge = 0 and gets demolished by the middle column"*.
- **`(must-merge, Refused)` → fail.** Not named by D18 because it is not the subtle case: the trap says these ARE one device and the engine decided they are not. A wrong decision fails at least as hard as a refusal to decide.
- **`(must-not-merge, Abstained)` → PASS, and this is the load-bearing cell.** It looks lenient. It is **required by D18's own argument**: the sentence *"an engine that abstains on everything scores false-merge = 0 and gets demolished by the middle column"* is only TRUE if abstention passes `must-not-merge`. Make this cell a failure and the middle column becomes redundant — and D18's stated mechanism for catching cowardice would be a description of something else. The gate's strength comes from `must-merge`, not from tightening `must-not-merge`.
- **`(must-abstain, Merged)` and `(must-abstain, Refused)` → fail.** D18 says the column fails on *"a decision"*; both are decisions.

**The rule is NOT compared here.** `(must-merge{rule: A}, Merged{rule: B})` is a **PASS** in 4.6a. D64 kept D46b's first criterion and *"it changes owner: compare `(verdict, rule)`, never `verdict` alone… it becomes `assert_eq!(decision.rule, case.expect_rule)` **in the trap runner**"* [architecture.md:4457] — story 4.7. **Do not derive `PartialEq` between the expectation and the outcome and call it scoring**: that would silently fail the wrong-rule cell and steal 4.7's criterion.

## Acceptance Criteria

### Must / must not

- **MUST** decide all nine cells exactly as the matrix above.
- **MUST NOT** compare rules — that is 4.7's.
- **MUST NOT** name the scored-outcome type `Decision`.
- **MUST NOT** touch a file, a path, a clock or `anyhow`. This story is pure.
- **MUST** make `source_state` empty by construction, not by convention.

---

1. **The scored outcome mirrors `Expectation`'s algebra, and is not called `Decision`.** `Expectation` (trap.rs:55-66) is `MustMerge { rule } | MustNotMerge { rule } | MustAbstain { cause }`. The outcome is its counterpart: a merge naming the rule that fired, a refusal naming the rule that opposed, an abstention naming the cause — so 4.7's `(outcome, rule)` comparison drops in with no redesign.
   `Decision` is **reserved**: the architecture names it as the engine's real return type [architecture.md:3306] and never lists its fields. Taking the name would squat a type Epic 5 must define.

2. **The abstention carries `AbstentionCause`, and its known inadequacy is recorded, not resolved.** `AbstentionCause` (gap/mod.rs:26-34) is the RECONCILIATION vocabulary — `OutOfPerimeter | NoObservedValue | ConflictingObservations`. The identity cascade's abstention is `Ambiguous`, arising from the verdict algebra [architecture.md:967-974], and **none of the three names it**.
   **Decision (Guy, 2026-07-22): use `AbstentionCause` on both sides and record the question.** The expectation side is already frozen on it (story 4.2, and the committed `example.toml` uses `NoObservedValue`), so a different outcome type would make comparison asymmetric against a locked format. Whoever builds the cascade decides whether to widen the enum or give the outcome its own cause type. **Do not widen it here** — `reconcile` (story 3.6) matches on it exhaustively, and there is no producer yet.

3. **The algebra is one exhaustive `match` over both sides, with no `_` arm**, implementing the nine cells above. A new column or a new outcome must break it and force a decision — the mechanism 4.5b relied on when `Record::Capability` broke four matches on purpose.

4. **Every cell is tested, and the five non-D18 cells cite their reason in the test.** Nine assertions minimum. The three failure cells D18 names are the gate's core; `(must-not-merge, Abstained) → pass` needs its own test with D18's argument in the comment, because it is the cell a reviewer will challenge.

5. **The tally is domain data, not strings.** `Expectation::column()` returns `&'static str` (trap.rs:70-76) — do **not** key the tally on it. D47: *"an error there is domain data, not a string"*. The precedent is `Reconciliation::abstentions: BTreeMap<AbstentionCause, usize>` (gap/mod.rs). A three-field struct or a `Column` enum; if you introduce a `Column` enum, `trap.rs` moves too — say so in the File List.

6. **The published number is one: failures = 0.** *"One number blocks: truth-table failures = 0"* [architecture.md:1224-1226]. The tally may break it down per column for a readable failure; **no fraction, no percentage, no threshold, no score.** D18 refuses them with the binomial argument: *"a `<= 0.01` threshold cannot distinguish 0.5% from 2%… not a gate, it is a coin toss wearing a badge of authority"* [architecture.md:1212-1217].

7. **The scored record carries five things, and two are substitutions D36 did not ask for.** Per D36 [architecture.md:2069-2073]:
   - the **outcome** (AC1);
   - the **`reason`** — the TRAP's authored sentence, carried so a failure is readable without opening the corpus. D19 licenses it: *"the oracle is the fixture's author, made explicit and versioned, with a mandatory `reason`"*. **The architecture never disambiguates trap-reason from engine-explanation** — state the choice in the type's doc;
   - the **`capability_snapshot`** — the `Capabilities` under which the outcome was reached;
   - **`source_state`** — present, empty by construction (AC8);
   - the **input's identity** — see AC9. **`fixture_seq` is not implemented**, see AC9.

8. **`source_state` is empty by CONSTRUCTION, and the mechanism's rationale must be true.** A field merely documented as always-`None` drifts the first time someone has a plausible value.
   ⚠️ **D32's `SourceState` is a `struct`, not an enum** [architecture.md:1832-1835] — `{ liveness: Liveness, capabilities: Capabilities }`. So *"Epic 13 will add variants"* is **false**; Epic 13 will replace the type. What survives a replacement is the field's **name and its `Option`-ness**, not the placeholder's shape. Say that, and not the false thing.
   Suggested mechanism — take it or name your own, but **say which in the record**: an uninhabited placeholder makes `Option<_>` provably `None`. **The runtime witness is `assert_eq!(size_of::<Option<SourceState>>(), 0)`** — an `is_none()` assertion is exactly the "by convention" this AC rejects, since it passes for any inhabited type too.

9. **`fixture_seq` is not implemented, and the substitute is honest about its own limit.** `fixture_seq` occurs **once** in 5,123 lines [architecture.md:2071], zero times in the PRD, zero in code. The obvious reading — an ordinal into the stream — **contradicts a locked decision**: 4.1/4.2 chose `obs_id` *because* a line number *"would silently shift under the truth"* (trap.rs:94-95).
   The record instead carries the corpus's own names: the **`TrapId`** and the replay stream's **corpus-relative path as a `String`** (the precedent is `Trap.replay: String`; **`PathBuf` must not enter `opencmdb-core`**).
   ⚠️ **This pair is not globally unique**, and the story says so rather than pretending: `TrapError::DuplicateId` is documented *"Two traps in the same file share an id"* — uniqueness is **per file**. At ~50 traps across many files (4.9+), two files may both define `mac-randomized-01`. Record that the key is provisional and that a cross-file `TrapId` guard belongs with the corpus-hygiene work, not here.

10. **Room for the verdict vector, by the same standard as AC8.** *"The harness records, for every case, the COMPLETE VERDICT VECTOR, not just the outcome… **the anti-drift is not discipline, it is a data requirement**"* [architecture.md:1396-1399] — a requirement on the harness that D36's five-field list omits. No engine produces a vector, so **do not invent its element type**; use the AC8 mechanism so the field is provably empty rather than empty by comment. If that proves clumsy, drop the field and record it in the register instead — **what is forbidden is a doc comment standing in for a proof**, five lines after AC8 demanded one.

11. **Nothing here derives `Serialize` without a consumer.** The 4.5a review deferred exactly that finding against `ConnectorError`. This story persists nothing (4.6b/4.6c may). If you derive it anyway, say why in the record; if you do, `#[serde(deny_unknown_fields)]` applies as it does everywhere.

12. **The frontier holds.** No `anyhow`, no `axum`, no `sqlx`, no `askama`, no `std::path`, no clock in `opencmdb-core`. `cargo xtask ci`'s frontier gate must stay green.

13. **All gates green, locally:** `cargo fmt --all` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo test --workspace` · `cargo xtask ci`. The `ℹ views-hash STALE` line is informational; **do not regenerate `architecture-views.md`**.

## Tasks / Subtasks

- [ ] **Task 1 — the scored outcome** (AC: 1, 2)
  - [ ] The type, mirroring `Expectation`, not named `Decision`. `RuleId` and `AbstentionCause` already exist; reuse both.
  - [ ] The `AbstentionCause` inadequacy in the type's doc, with the Epic 5 question stated.

- [ ] **Task 2 — the record** (AC: 7, 8, 9, 10, 11)
  - [ ] Five fields, each doc'd with WHY it is there; the two substitutions doc'd with what D36 asked for instead.
  - [ ] `source_state` empty by construction + the `size_of` witness.
  - [ ] The verdict-vector field by the same standard, or a register entry instead.

- [ ] **Task 3 — the algebra** (AC: 3, 5, 6)
  - [ ] One exhaustive `match`, no `_` arm, the nine cells of the matrix.
  - [ ] The tally as domain data; the single published number.

- [ ] **Task 4 — the tests** (AC: 4, 12)
  - [ ] All nine cells. `(must-not-merge, Abstained) → pass` carries D18's argument in its comment.
  - [ ] `(must-merge{rule: A}, Merged{rule: B}) → pass` — the cell that proves 4.7's criterion was not stolen.
  - [ ] **Prove-to-red**: flip one matrix cell and show exactly the matching test reds. Record the mutation.
  - [ ] The `size_of` witness for AC8.

- [ ] **Task 5 — the record and the gates** (AC: 13)
  - [ ] Append to `deferred-work.md` under `## Deferred from: story-4.6a (2026-07-22)`: the `AbstentionCause`/`Ambiguous` question owned by Epic 5 · `fixture_seq` not implemented and why · the `TrapId` per-file uniqueness limit · `source_state` empty until Epic 13 with the mechanism · the verdict vector's absent producer. **Append; never rewrite a bullet.**
  - [ ] Update `sprint-status.yaml`; put it in the File List.
  - [ ] Run the four gates. **Name the command behind every claim in the completion record.**

## Dev Notes

### What already exists — use it, do not rewrite it

- **`Expectation` / `column()` / `rule()`** [Source: crates/opencmdb-core/src/trap.rs:55-84] — `MustMerge{rule} | MustNotMerge{rule} | MustAbstain{cause}`; `column()` returns D18's three strings; `rule()` returns `None` only for an abstention.
- **`RuleId`** [Source: crates/opencmdb-core/src/trap.rs:36-38] — `pub struct RuleId(pub String)`, *"a `String` for now because no rule exists yet — Epic 5 names them"*.
- **`AbstentionCause`** [Source: crates/opencmdb-core/src/gap/mod.rs:26-34] — three variants, already reused by `Expectation::MustAbstain`.
- **`Capabilities`** [Source: crates/opencmdb-core/src/observation/mod.rs:216-232] — `{ as_of, kinds }`.
- **`Reconciliation::abstentions: BTreeMap<AbstentionCause, usize>`** [Source: crates/opencmdb-core/src/gap/mod.rs] — the precedent for a tally keyed on domain data, not strings.

### Traps

1. **Deriving `PartialEq` between expectation and outcome and calling it scoring.** It fails the wrong-rule cell, which is 4.7's criterion, not this story's.
2. **Making `(must-not-merge, Abstained)` a failure** because it feels stricter. It breaks D18's own anti-cowardice argument.
3. **Publishing a fraction.** Refused by name, with the binomial reasoning behind the refusal.
4. **Repeating a false rationale.** AC8's own history: the first draft of this story claimed Epic 13 would "add variants" to `SourceState`; it is a struct. Story 2.2 shipped a comment claiming a guardrail existed and it survived four epics until 4.5a's review added an enum variant and watched the build succeed.
5. **Claiming more than was measured.** **Three** consecutive completion records over-claimed (the figure is three — an earlier draft of this story said five, which nothing supports). Name the test or command behind every claim, or write the weaker true sentence.
6. **Skipping `--all-targets` or `xtask ci` locally.** Epic 3's retrospective recorded four CI-only failures from exactly that.

### Previous story intelligence (4.1 → 4.5b, all reviewed)

Exhaustive `match` with no `_` arm wherever a new variant must force a decision · offending item **second** in every test vector · the register must not lose an item · a comment asserting a checkable property gets checked · `#[serde(deny_unknown_fields)]` on anything parsed.

Still open and NOT this story's business: non-UTF-8 payloads passing the sha gate · a BOM diagnosed as a JSON error · `Observation.raw` inspected by no privacy rule · `Fact::Mac.locally_administered` denormalization (4.9+) · collecting all validation errors instead of the first (4.7) · two on-disk spellings for a unit `ConnectorError` variant · `fixture_path`'s lexical containment vs a symlink · `scopes_covered` still constructor-supplied · `PollSummary` absent on the error path (which is why **4.6b** must reconstruct the snapshot by walking records — `deferred-work.md` says so explicitly).

### Git intelligence

`master` requires a pull request since 2026-07-22 (0 approvals, `ci` must pass, squash merge). Work on `story/4-6-metrics-harness`; **do not push to `master`.** The branch already carries the `obs_id` collision fix (`6fb7f2c`).

### Latest technical specifics

No new crate, no version bump. **Locked** (committed `Cargo.lock`, verified 2026-07-22): `serde 1.0.228`, `serde_json 1.0.150`, `uuid 1.24.0`, `chrono 0.4.45` with `default-features = false`, `toml 0.8.23`, `tokio 1.53.0`, `tokio-util 0.7.18`. Rust 1.96+, edition 2024, `resolver = "3"`. **Never invent a version.**

### Project Structure Notes

- **New:** one module under `crates/opencmdb-core/src/`. ⚠️ **`metrics` is taken** — `crates/opencmdb-bin/src/metrics.rs` is the Prometheus handler (`main.rs:14`). Pick a name that cannot be confused with it.
- **Updated:** `crates/opencmdb-core/src/lib.rs` (exports), possibly `trap.rs` (only if AC5's `Column` enum is chosen), `deferred-work.md`, `sprint-status.yaml`.
- **Unchanged, expected:** `fixtures/`, `crates/opencmdb-bin/`, `Cargo.lock`.

### References

- [Source: _bmad-output/planning-artifacts/epics.md:983-1003 — Story 4.6, which 4.6a/b/c jointly implement]
- [Source: _bmad-output/planning-artifacts/epics.md:1005-1021 — Story 4.7: the `(outcome, rule)` assertion this story deliberately does not make]
- [Source: _bmad-output/planning-artifacts/architecture.md:1208-1265 — D18 in full: the binomial argument, the three columns, what was refused as a gate]
- [Source: _bmad-output/planning-artifacts/architecture.md:1224-1226 — "One number blocks: truth-table failures = 0"]
- [Source: _bmad-output/planning-artifacts/architecture.md:1229-1233 — the three columns, their failure conditions, and the anti-cowardice sentence the matrix is derived from]
- [Source: _bmad-output/planning-artifacts/architecture.md:1341-1346 — D19's build order: "a metric written after the engine is bent to fit the engine"]
- [Source: _bmad-output/planning-artifacts/architecture.md:1396-1399 — the COMPLETE VERDICT VECTOR: "not discipline, a data requirement"]
- [Source: _bmad-output/planning-artifacts/architecture.md:2057-2077 — D36: the five-field record; "two answers or two questions"]
- [Source: _bmad-output/planning-artifacts/architecture.md:1832-1835 — D32's `SourceState`, a STRUCT, built in Epic 13]
- [Source: _bmad-output/planning-artifacts/architecture.md:967-974 — the three-way outcome and where `Ambiguous` comes from (AC2's problem)]
- [Source: _bmad-output/planning-artifacts/architecture.md:3306 — `Decision` is the engine's return type; its fields are never listed]
- [Source: _bmad-output/planning-artifacts/architecture.md:4457 — D64 keeps "compare `(verdict, rule)`, never `verdict` alone" and moves it to the trap runner]
- [Source: _bmad-output/planning-artifacts/architecture.md:986-993 — floats may RANK, never DECIDE; `confidence` as an integer]
- [Source: _bmad-output/planning-artifacts/prd.md:1160-1188 — NFR4: any fraction is theatre; bulk metrics gate nothing]
- [Source: _bmad-output/planning-artifacts/epics.md:611 — the `source_state`/Epic 13 deferral. NOTE: this line is in **Story 2.1**, not in Epic 4 — an earlier draft misattributed it]
- [Source: crates/opencmdb-core/src/trap.rs:36-113 — `RuleId`, `TrapId`, `Expectation`, `Trap`, `TrapFile`, `DuplicateId`'s per-file scope]
- [Source: crates/opencmdb-core/src/gap/mod.rs:26-34 — `AbstentionCause` and the `BTreeMap` tally precedent]
- [Source: _bmad-output/implementation-artifacts/deferred-work.md — the register; the 4.5b entry naming 4.6 by name]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
