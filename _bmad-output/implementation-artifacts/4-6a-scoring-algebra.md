# Story 4.6a: The scoring algebra and the scored record

Status: done

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

**Three cells come straight from D18's table** — `(must-not-merge, Merged)`, `(must-merge, Abstained)`
and the two `must-abstain` decisions counted as its single "a decision" condition. The rest need
their reason written down, and one of them needs two:

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

- [x] **Task 1 — the scored outcome** (AC: 1, 2)
  - [x] The type, mirroring `Expectation`, not named `Decision`. `RuleId` and `AbstentionCause` already exist; reuse both.
  - [x] The `AbstentionCause` inadequacy in the type's doc, with the Epic 5 question stated.

- [x] **Task 2 — the record** (AC: 7, 8, 9, 10, 11)
  - [x] Five fields, each doc'd with WHY it is there; the two substitutions doc'd with what D36 asked for instead.
  - [x] `source_state` empty by construction + the `size_of` witness.
  - [x] The verdict-vector field by the same standard, or a register entry instead.

- [x] **Task 3 — the algebra** (AC: 3, 5, 6)
  - [x] One exhaustive `match`, no `_` arm, the nine cells of the matrix.
  - [x] The tally as domain data; the single published number.

- [x] **Task 4 — the tests** (AC: 4, 12)
  - [x] All nine cells. `(must-not-merge, Abstained) → pass` carries D18's argument in its comment.
  - [x] `(must-merge{rule: A}, Merged{rule: B}) → pass` — the cell that proves 4.7's criterion was not stolen.
  - [x] **Prove-to-red**: flip one matrix cell and show exactly the matching test reds. Record the mutation.
  - [x] The `size_of` witness for AC8.

- [x] **Task 5 — the record and the gates** (AC: 13)
  - [x] Append to `deferred-work.md` under `## Deferred from: story-4.6a (2026-07-22)`: the `AbstentionCause`/`Ambiguous` question owned by Epic 5 · `fixture_seq` not implemented and why · the `TrapId` per-file uniqueness limit · `source_state` empty until Epic 13 with the mechanism · the verdict vector's absent producer. **Append; never rewrite a bullet.**
  - [x] Update `sprint-status.yaml`; put it in the File List.
  - [x] Run the four gates. **Name the command behind every claim in the completion record.**

### Review Findings

_Code review 2026-07-22 — three parallel layers (Blind Hunter, Edge Case Hunter, Acceptance Auditor). **All 13 ACs SATISFIED**, no violations; every measured claim in the Dev Agent Record independently reproduced, including all five mutations and the `size_of` probe. The auditor read D18 itself and judged the derivation of the five non-table cells **correct — and better supported than the story argued**: read literally, D18's table names one failure condition per column, which yields `(must-not-merge, Abstained) → pass` directly, with the anti-cowardice sentence as a second independent proof. The stricter reading was found and refuted from D18's own text. What follows is everything else._

- [x] [Review][Patch] **`ScoredRecord` cannot say whether the trap passed.** It carries neither a `Score` nor the `Expectation` it was judged against, so pass/fail is unrecoverable from a record — which is precisely the post-hoc analysis D36 justifies it with (*"you cannot tell a regression from a legitimate re-derivation"*). Found independently by two layers. Carrying the expectation makes the score recomputable and costs one field [crates/opencmdb-core/src/score.rs:212]
- [x] [Review][Patch] **`Tally` reproduces per column the vacuity `scored` closes globally.** Only failures are counted, so `failures_in(MustMerge) == 0` is ambiguous between "the column passed" and "the column was never exercised". Probed: 300 `must-not-merge` traps against an abstain-on-everything engine report `scored=300, failures=0` — a green gate over the exact cowardice D18 says the middle column exists to catch. The module's own argument is that the gate's strength lives in that column; the tally cannot prove it ran. Found independently by two layers [crates/opencmdb-core/src/score.rs:237]
- [x] [Review][Patch] **Three doc paragraphs claim `rule` and `cause` do work the code never does.** `score` matches `{ .. }` on every arm, so no payload influences any cell. Therefore *"that symmetry is what makes scoring total"* is false — totality comes from the exhaustive 3×3 and would hold with no payloads at all; and the long justification for reusing `AbstentionCause` (*"a different type would make comparison asymmetric"*) describes a comparison that is never performed [crates/opencmdb-core/src/score.rs:263]
- [x] [Review][Patch] **A test that cannot fail, using the assertion the module declares worthless ten lines above.** `a_scored_record_carries_the_snapshot_and_the_authored_reason` asserts `is_none()`, `is_empty()`, `can_emit` and a `contains` on a value the test itself constructed three lines earlier. Its only failure mode is a compile error — and AC8 explicitly rejects `is_none()` as proof [crates/opencmdb-core/src/score.rs:497]
- [x] [Review][Patch] **`assert_eq!(size_of::<SourceState>(), 0)` is vacuous, and the verdict-vector witness is for the wrong type.** Any inhabited ZST passes the first (`size_of::<()>() == 0`). The second asserts on `Option<VerdictVectorEntry>` while the field is `Vec<VerdictVectorEntry>`, whose size is constant regardless of element. **The `Option<SourceState>` witness itself is sound** — confirmed by the auditor: `size_of::<Option<T>>() == 0` cannot hold for inhabited `T`, since `Option<T>` would then have ≥2 values. Only the two neighbours are noise [crates/opencmdb-core/src/score.rs:473]
- [x] [Review][Patch] **"the middle column becomes redundant" does not follow.** Making the load-bearing cell a failure would still leave `must-merge` as the only column catching an engine that REFUSES rather than abstains. The sufficient and true statement is the one immediately before it — *"that sentence is only true if abstention passes `must-not-merge`"*. This project's standard is to write the weaker true sentence, and this is the comment that states the standard [crates/opencmdb-core/src/score.rs:133]
- [x] [Review][Patch] **All nine cell tests pair matching rules, so exactly ONE test defends the headline decision.** `must_merge()` and `merged()` both use `l1-exact-mac`; `must_abstain()` and `abstained()` share a cause. An implementation that compared rules passes all four cell tests — the mutation log confirms it reds one test "(only)". Using non-matching rules in the ordinary cells costs nothing and would give nine guards instead of one [crates/opencmdb-core/src/score.rs:514]
- [x] [Review][Patch] **Per-column accumulation past 1 is never tested.** The tally test records exactly one failure per column, so mutating `*entry += 1` to `*entry = 1` leaves every test green. For the type whose published number is a sum of these counters, that is the obvious mutation and it is undefended [crates/opencmdb-core/src/score.rs:634]
- [x] [Review][Patch] **`Column` names the expectation axis, while the doc table directly above puts expectations in ROWS.** A reader of `failures_in(Column::MustMerge)` who checks the table reads the wrong axis. For a module whose whole value is unambiguous written-down semantics, the vocabulary disagrees with its own diagram [crates/opencmdb-core/src/score.rs:300]
- [x] [Review][Patch] **"a new column or a new outcome must break this" is false for half of it.** Adding a `Column` variant breaks `Column::of` and `Column::as_str`, not `score` — which never mentions `Column`. Adding an `Expectation` or `Outcome` variant does break it [crates/opencmdb-core/src/score.rs:365]
- [x] [Review][Patch] **`Column`'s doc argues a `&'static str` is stringly-typed domain data, then publishes `as_str() -> &'static str`.** The argument that justifies the new type immediately re-exposes what it objected to. Either say what `as_str` is for (report rendering, and pinned against `Expectation::column()`), or drop it [crates/opencmdb-core/src/score.rs:301]
- [x] [Review][Patch] **AC5 required the `Column` choice in the File List; it is in the Completion Notes.** `Expectation::column()` still exists with a live consumer at `crates/opencmdb-bin/src/fixtures.rs`, so two types now claim D18's vocabulary. The drift is pinned by a test, and the deviation is stated — but not where the AC asked [this file]
- [x] [Review][Patch] **`(must-abstain, Merged)` and `(must-abstain, Refused)` cite their reason only through the test's NAME.** Every other derived cell carries an inline citation. AC4 asks for the reason *in the test* [crates/opencmdb-core/src/score.rs:371]
- [x] [Review][Patch] **The story file's own "four from D18 / five derived" split is internally inconsistent** — the first of the "five that need their reason written down" is labelled *"D18's named case"*. The shipped code gets this right; the story file does not, and its Change Log repeats the wrong framing [this file]

- [x] [Review][Defer] **`Tally::record` takes no `TrapId`, so the same trap can be scored twice and inflate both `scored` and the failure count** [crates/opencmdb-core/src/score.rs:245] — deferred to **4.6b**, which owns the join between records and the tally. Reachable for real, not theoretical: `TrapError::DuplicateId` is enforced PER FILE, so one id defined in two corpus files is legal today. Probed: two identical `record` calls give `scored=2, failures=2`.
- [x] [Review][Defer] **`ScoredRecord`'s `reason`, `replay` and `trap` are unvalidated `String`s that bypass `Trap::validate`'s contract** [crates/opencmdb-core/src/score.rs:213] — deferred. The corpus refuses an empty, multi-line or >300-character reason (`REASON_MIN_CHARS = 20`); the record accepts all of them, and every field is `pub` with no constructor. In practice the harness builds records from an already-validated `Trap`, so the value arrives validated — but nothing enforces it. A constructor belongs with 4.6b, which is the first real producer.
- [x] [Review][Defer] **The `size_of` witness rests on a layout optimisation, not a language guarantee** [crates/opencmdb-core/src/score.rs:473] — deferred, and verified benign. The Reference specifies `Option<T>` layout only for null-pointer-optimisation cases; an `Option` of an uninhabited type collapsing to zero bytes is the compiler's choice. Confirmed on rustc 1.97.1. Also verified: replacing `SourceState` with D32's struct still compiles everywhere and fails **usefully** (`left: 48, right: 0`), and deriving serde later does not break on an uninhabited field.
- [x] [Review][Defer] **The cascade's `NoMatch` maps two ways onto `Outcome`, and only half is recorded** [crates/opencmdb-core/src/score.rs:58] — deferred to Epic 5. `architecture.md:967-974` makes `NoMatch` cover both an active opposition and a mere absence of proof; `Outcome::Refused` requires a rule to name, so absence-of-proof must map to `Abstained`. If Epic 5 maps `NoMatch → Refused` uniformly, every honest `must-abstain` trap fails — the case D18 says must NOT be gated. The register currently records only the `Ambiguous`-has-no-cause half.
- [x] [Review][Defer] **`ScoredRecord` is not `#[non_exhaustive]` despite being designed to change shape** [crates/opencmdb-core/src/score.rs:212] — deferred. `fixture_seq` may return, `SourceState` will be replaced by Epic 13, `VerdictVectorEntry` will gain a real type. Every field is `pub` with no constructor, so each struct literal is a breaking-change site. Pairs naturally with the constructor deferred above.

_Dismissed as noise (3): the module being named `score` while its prose says "no score" (it means no numeric grade, and the sentence says so) · `SourceState`/`VerdictVectorEntry` re-exported at the crate root (they are part of `ScoredRecord`'s public shape; a caller matching the struct needs them nameable) · `usize` overflow in the tally (unreachable at n≈300, and a debug build panics)._

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

Claude Opus 4.8 — `claude-opus-4-8[1m]`

### Debug Log References

Every claim below is a command that was run, not an inference.

- **The `size_of` witness was verified BEFORE the story relied on it.** A standalone `rustc -O`
  probe outside the repo printed `size_of::<Option<Never>>() = 0` and `size_of::<Never>() = 0`.
  Without that check, AC8's chosen witness would have been an assumption about niche optimisation.
- **Five guards proven to red**, each mutation applied and reverted from a pre-mutation backup:
  | Mutation | Tests reddened |
  |---|---|
  | `(must-not-merge, Abstained)` Pass → Fail — the load-bearing cell | `an_engine_that_abstains_on_everything_is_demolished_by_the_middle_column`, `must_not_merge_fails_only_on_a_merge` |
  | `(must-merge, Abstained)` Fail → Pass — the cowardice cell | the same D18-mechanism test, `must_merge_passes_only_on_a_merge`, `the_tally_counts_failures_per_column_and_publishes_one_number` |
  | `(must-not-merge, Merged)` Fail → Pass — the false merge | `must_not_merge_fails_only_on_a_merge`, the tally test |
  | scoring starts comparing rules | `scoring_ignores_the_rule_because_the_trap_runner_owns_it` (only) |
  | `SourceState` made inhabited | `source_state_cannot_be_populated_in_epic_4` (only) |

  The first mutation is the one worth reading: making the load-bearing cell stricter reds **the test
  that proves D18's own anti-cowardice mechanism**, which is the argument the cell is derived from.
- **Gates:** `cargo fmt --all` clean · `cargo clippy --workspace --all-targets -- -D warnings` clean
  · `cargo test --workspace` → **86 (bin) + 56 (core) + 38 (xtask), 0 failed** (core gained 10).
- `cargo xtask ci` → all gates green, `✅ fixtures 5/5`; `architecture-views.md` NOT regenerated.
- **The frontier is untouched.** `cargo tree -p opencmdb-core -e normal --depth 1` still lists only
  `chrono`, `serde`, `thiserror`, `tokio-util`, `uuid`. No `anyhow`, no `std::path`, no clock.
- **`Cargo.lock` did not move** — `git diff --stat Cargo.lock` empty.
- **The MariaDB-backed tests did NOT run.** `DATABASE_URL` is unset; the four DB-backed tests return
  early and pass either way. These counts say nothing about the database.

### Completion Notes List

- **The gate's semantics are now written down, all nine cells.** D18 names one failure condition per
  column; the other five are derived in the `score` doc comment with their reason, so a reviewer can
  disagree with a specific cell rather than with a black box.
- **The load-bearing cell is `(must-not-merge, Abstained) → PASS`**, and it has its own test that
  proves the mechanism rather than the cell: an engine abstaining on everything scores
  `must-not-merge` failures = 0 and is caught by the middle column, which is D18's stated argument.
  Had the cell been a failure, that sentence would describe nothing.
- **The rule is deliberately ignored**, so the right answer reached by the wrong rule PASSES here.
  A test exists solely to catch the tempting implementation — a `PartialEq` between expectation and
  outcome — which would silently steal story 4.7's criterion (D64 moved it to the trap runner).
- **Two deferred fields are empty by CONSTRUCTION, not by comment.** `SourceState` and
  `VerdictVectorEntry` are uninhabited, so `Option<SourceState>` is provably `None` and
  `Vec<VerdictVectorEntry>` provably empty. The witness is `size_of == 0`; an `is_none()` assertion
  would have passed for any inhabited type and proved nothing.
- **The story's own false claim was corrected before it reached the code.** An earlier draft said
  Epic 13 would "add variants" to `SourceState`; D32's `SourceState` is a struct, so it will be
  REPLACED. The type doc now says what survives — the field's name and its `Option`-ness — and not
  the false thing.
- **The tally is keyed on a `Column` enum, not on `Expectation::column()`'s `&'static str`.** D47:
  domain data, not strings. `Column::as_str()` still returns D18's exact vocabulary, and a test pins
  it against `Expectation::column()` so a report and a trap file cannot drift apart.
- **`scored` is carried but is not a denominator.** It exists so a caller can tell "zero failures
  over three hundred traps" from "zero failures because nothing ran" — the vacuity story 4.1 removed
  from the fixtures gate. A test asserts the two tallies agree on `failures()` and differ only on
  `scored()`. No fraction is computed anywhere.
- **Naming, as the story allowed:** module `score` (not `metrics`, which is taken by
  `opencmdb-bin/src/metrics.rs`, the Prometheus handler); `Outcome` (not `Decision`, reserved);
  `Column`; `Score { Pass, Fail }`; `Tally`; `ScoredRecord`; `SourceState`; `VerdictVectorEntry`.
  `trap.rs` was NOT modified — `Column::of` derives the column here rather than changing
  `Expectation::column()`'s return type.
- **No `Serialize` anywhere.** Nothing persists these records yet, and deriving a wire format for a
  domain type with no consumer is a finding already recorded against `ConnectorError`.

### File List

- `crates/opencmdb-core/src/score.rs` (new — `Outcome`, `Column`, `Score`, `score`, `SourceState`,
  `VerdictVectorEntry`, `ScoredRecord`, `Tally`; 10 tests)
- `crates/opencmdb-core/src/lib.rs` (modified — `pub mod score;` and the re-exports)
- `crates/opencmdb-core/src/trap.rs` — **NOT modified, and that is the AC5 decision.** A `Column`
  enum was introduced in `score.rs` rather than changing `Expectation::column()`'s return type, so
  `trap.rs` and its consumer at `crates/opencmdb-bin/src/fixtures.rs` keep their `&'static str`.
  Two types therefore name D18's vocabulary; the drift is pinned by a test asserting
  `Column::as_str()` equals `Expectation::column()` for all three variants. AC5 asked for this
  choice to appear here, and an earlier version put it only in the Completion Notes.
- `_bmad-output/implementation-artifacts/deferred-work.md` (modified — five story-4.6a entries)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modified)
- `Cargo.lock` — **unchanged**, measured.

### Change Log

- 2026-07-22 — The release gate's algebra exists before the engine. All nine cells of D18's truth
  table are decided and justified, the five D18 does not name being derived from its own
  anti-cowardice argument rather than invented. Scoring ignores the rule, leaving `(verdict, rule)`
  to story 4.7. The scored record carries the capability snapshot that makes a verdict falsifiable
  (D36), plus two fields that are provably empty by construction until Epic 13 and Epic 5 give them
  producers. `fixture_seq` is not implemented, and the substitution is recorded rather than passed
  off as compliance. Pure domain code: no I/O, no clock, no new dependency.
- 2026-07-22 — Code review (three parallel layers), 14 findings applied. Two were design holes found
  independently by two layers: `ScoredRecord` could not say whether its trap passed (it now carries
  the `Expectation` and recomputes its own `score()`), and `Tally` reproduced per column the vacuity
  `scored()` closes globally (it now counts what RAN in each column, so "the middle column held" is
  distinguishable from "the middle column was empty" — the exact case D18 says the gate turns on).
  Six false or overstated claims in my own comments were corrected, including *"that symmetry is
  what makes scoring total"* (totality comes from the exhaustive match) and *"the middle column
  becomes redundant"* (it would not be). The nine cell tests now pair MISMATCHED rules: the
  "scoring compares rules" mutation reddened one test before and reddens six after.
