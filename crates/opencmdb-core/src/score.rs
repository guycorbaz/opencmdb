//! Scoring a run against the trap corpus — the release gate's algebra (Story 4.6a).
//!
//! This module exists **before any engine does**, and that order is the point. D19:
//! *"the metrics harness BEFORE the engine — **a metric written after the engine is bent to fit
//! the engine**"*.
//!
//! It is pure. No file, no path, no clock, no I/O: it takes an [`Expectation`] (what the trap's
//! author said is right) and an [`Outcome`] (what something answered) and says pass or fail. The
//! harness that reads the corpus is story 4.6b; comparing two runs is 4.6c.
//!
//! # The gate is one number
//!
//! D18: *"THE GATE = Tier 1 only… **binary, zero tolerance, at the device level.** One number
//! blocks: **truth-table failures = 0**"*. There is no fraction here, no percentage, no threshold
//! and no score, because at n=300 *"the only measurable threshold is ZERO. Every fraction is
//! theatre"* — a `<= 0.01` threshold cannot distinguish 0.5% from 2%, so it is *"a coin toss
//! wearing a badge of authority"*. [`Tally`] breaks the failures down per column so a red gate is
//! readable; the number that blocks is still [`Tally::failures`].
//!
//! # What this module deliberately does NOT do
//!
//! **It does not compare rules.** `(MustMerge { rule: A }, Merged { rule: B })` — the right answer
//! reached by the wrong rule — is a **PASS** here. D64 revoked D46b but kept its first criterion
//! and *"it changes owner: compare `(verdict, rule)`, never `verdict` alone… it becomes
//! `assert_eq!(decision.rule, case.expect_rule)` **in the trap runner**"*, which is story 4.7.
//! Deriving `PartialEq` between an expectation and an outcome would silently fail that cell and
//! steal 4.7's criterion.

use std::collections::BTreeMap;

use crate::gap::AbstentionCause;
use crate::observation::Capabilities;
use crate::trap::{Expectation, RuleId, TrapId};

/// What something answered about a trap — the counterpart of [`Expectation`].
///
/// It mirrors the expectation's algebra on purpose: a merge names the rule that fired, a refusal
/// names the rule that OPPOSED the merge (not the one that was merely tempting), and an abstention
/// names a cause. **Scoring never reads those payloads** — totality comes from the exhaustive 3×3
/// in [`score`], and would hold if `Outcome` carried nothing at all. The payloads are here so that
/// story 4.7 can add `(outcome, rule)` comparison without changing this type.
///
/// **Not named `Decision`.** The architecture reserves that name for the engine's real return type
/// and never lists its fields; taking it here would squat a type Epic 5 has to define.
///
/// # The abstention cause is known to be inadequate, and that is recorded rather than fixed
///
/// [`AbstentionCause`] is the RECONCILIATION vocabulary — `OutOfPerimeter`, `NoObservedValue`,
/// `ConflictingObservations`. The identity cascade's abstention is `Ambiguous`, which arises from
/// the verdict algebra (the cloned-MAC case), and **none of the three names it.**
///
/// It is used anyway, on both sides, because story 4.2 froze the truth format on it: the committed
/// corpus already writes `must-abstain = { cause = "NoObservedValue" }`, and reusing it keeps one
/// vocabulary for one concept. **Nothing here compares causes** — [`score`] ignores them — so this
/// is about naming, not about a comparison that could go asymmetric. **Epic 5 builds the
/// cascade and should decide** whether to widen this enum or give the outcome its own cause type;
/// it is not widened here, because `reconcile` matches on it exhaustively and there is no producer
/// yet. Recorded in `deferred-work.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// These observations describe one device, by this rule.
    Merged { rule: RuleId },
    /// These observations describe different devices, and this rule opposes the merge.
    Refused { rule: RuleId },
    /// The signal was insufficient; no decision was taken, for this cause.
    Abstained { cause: AbstentionCause },
}

/// One of D18's three columns — the unit the gate counts in.
///
/// ⚠️ **"Column" is D18's word for the EXPECTATION axis**, and [`score`]'s doc table renders that
/// axis as rows because a 3×3 has to put one of them there. The vocabulary follows D18, not the
/// table's orientation: *"the middle column"* always means `must-merge`, never `Refused`.
///
/// A domain enum rather than [`Expectation::column`]'s `&'static str`: a tally keyed on a string
/// is stringly-typed domain data, and D47's rule is that *"an error there is domain data, not a
/// string"*. The precedent is `Reconciliation::abstentions: BTreeMap<AbstentionCause, usize>`.
/// [`Column::as_str`] still exists, for RENDERING a report — the objection is to keying domain
/// state on a string, not to ever printing one — and a test pins it against
/// [`Expectation::column`] so the two spellings cannot drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Column {
    /// These observations must be merged; failing to is the cowardice case.
    MustMerge,
    /// These observations must NOT be merged; merging them is the false merge.
    MustNotMerge,
    /// The case is honestly ambiguous; deciding either way is a guess.
    MustAbstain,
}

impl Column {
    /// The column an expectation belongs to.
    pub fn of(expectation: &Expectation) -> Self {
        match expectation {
            Expectation::MustMerge { .. } => Column::MustMerge,
            Expectation::MustNotMerge { .. } => Column::MustNotMerge,
            Expectation::MustAbstain { .. } => Column::MustAbstain,
        }
    }

    /// The column name as D18's table writes it, and as `Expectation::column()` already returns.
    pub fn as_str(self) -> &'static str {
        match self {
            Column::MustMerge => "must-merge",
            Column::MustNotMerge => "must-not-merge",
            Column::MustAbstain => "must-abstain",
        }
    }
}

/// Whether one trap was answered correctly. Binary, because the gate is (D18).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Score {
    Pass,
    Fail,
}

/// Score one trap: did this outcome satisfy this expectation?
///
/// # The full truth table
///
/// D18's table names **one** failure condition per column. The 3×3 has nine cells, so the other
/// five are derived below — and written down, because whatever this function decides silently
/// becomes the release gate's semantics.
///
/// | expected \ scored | `Merged` | `Refused` | `Abstained` |
/// |---|---|---|---|
/// | `must-merge`     | pass | **fail** | **fail** |
/// | `must-not-merge` | **fail** | pass | **pass** |
/// | `must-abstain`   | **fail** | **fail** | pass |
///
/// - `(must-not-merge, Merged)` — D18: the false merge, *"the operator loses trust and
///   uninstalls"*.
/// - `(must-merge, Abstained)` — D18: **cowardice**, the case the middle column was created for.
/// - `(must-abstain, Merged | Refused)` — D18: the column fails on *"a decision"*; both are.
/// - `(must-merge, Refused)` — not named by D18 because it is not the subtle case: the trap says
///   these ARE one device and the answer decided they are not. A wrong decision fails at least as
///   hard as a refusal to decide.
/// - **`(must-not-merge, Abstained)` → PASS, and this is the load-bearing cell.** It looks lenient
///   and it is REQUIRED by D18's own argument: *"an engine that abstains on everything scores
///   false-merge = 0 and gets **demolished by the middle column**"*. **That sentence is only true
///   if abstention passes `must-not-merge`** — make this a failure and an all-abstaining engine
///   scores n, not 0, and D18's own claim about its own gate is false. (It would NOT make the
///   middle column redundant: `must-merge` still uniquely catches an engine that REFUSES rather
///   than abstains. The narrower statement is the one that holds.)
///   Read literally, D18's table also gives this cell directly: it names `a merge` as the column's
///   failure condition **and nothing else**.
///   **The gate's strength comes from `must-merge`, not from tightening `must-not-merge`.**
///
/// Exhaustive with no `_` arm: a new [`Expectation`] or [`Outcome`] variant must break THIS
/// function and force a decision. (A new [`Column`] variant breaks [`Column::of`] and
/// [`Column::as_str`] instead — `score` never mentions `Column`.)
///
/// Note that `rule` and `cause` are ignored throughout — see the module doc. That is what makes
/// the wrong-rule cell a pass, and it is deliberate.
pub fn score(expected: &Expectation, actual: &Outcome) -> Score {
    match (expected, actual) {
        (Expectation::MustMerge { .. }, Outcome::Merged { .. }) => Score::Pass,
        (Expectation::MustMerge { .. }, Outcome::Refused { .. }) => Score::Fail,
        (Expectation::MustMerge { .. }, Outcome::Abstained { .. }) => Score::Fail,

        (Expectation::MustNotMerge { .. }, Outcome::Merged { .. }) => Score::Fail,
        (Expectation::MustNotMerge { .. }, Outcome::Refused { .. }) => Score::Pass,
        (Expectation::MustNotMerge { .. }, Outcome::Abstained { .. }) => Score::Pass,

        (Expectation::MustAbstain { .. }, Outcome::Merged { .. }) => Score::Fail,
        (Expectation::MustAbstain { .. }, Outcome::Refused { .. }) => Score::Fail,
        (Expectation::MustAbstain { .. }, Outcome::Abstained { .. }) => Score::Pass,
    }
}

/// The state of a source when an outcome was reached — **not buildable in Epic 4.**
///
/// D32 specifies it as a struct: `{ liveness: Liveness, capabilities: Capabilities }`. Epic 13
/// builds it, together with liveness; the deferral is explicit in the epic list, and nothing in
/// Epic 4 produces a liveness at all.
///
/// This placeholder is **uninhabited**, so [`ScoredRecord::source_state`] is provably `None` until
/// Epic 13 — no value of this type can be constructed, and the compiler enforces it rather than a
/// comment asking politely.
///
/// ⚠️ **What survives Epic 13 is the field's NAME and its `Option`-ness, not this type.** Epic 13
/// will REPLACE this with D32's struct; it will not "add variants", because D32's `SourceState` is
/// not an enum. Saying otherwise would be a false claim in a doc comment, which is the defect this
/// project has caught three times.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceState {}

/// One entry of the complete verdict vector — **has no producer yet.**
///
/// D18's neighbour requirement, stated on the harness and absent from D36's five-field list:
/// *"The harness records, for every case, the COMPLETE VERDICT VECTOR, not just the outcome.
/// Without it the A-vs-B question is undecidable after the fact… **the anti-drift is not
/// discipline, it is a data requirement.**"*
///
/// The vector's element is `(rule, verdict, evidence)` and none of the three exists: rules arrive
/// in Epic 5. Rather than invent the type, this placeholder is **uninhabited** — so the field is
/// provably empty, by the same standard as [`SourceState`], instead of being empty by comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerdictVectorEntry {}

/// One scored trap, recorded so the verdict is FALSIFIABLE.
///
/// D36: *"A verdict without its capability snapshot is UNFALSIFIABLE: you cannot tell a regression
/// from a legitimate re-derivation."* → the harness records
/// `{verdict, reason, capability_snapshot, source_state, fixture_seq}`.
///
/// Two of those five are not what D36 wrote, and both substitutions are deliberate:
///
/// - **`fixture_seq` is not implemented.** It occurs exactly once in the whole architecture, inside
///   D36's list, and is defined nowhere — no type, no shape, no prose. The obvious reading, an
///   ordinal into the stream, **contradicts a locked decision**: stories 4.1/4.2 chose `obs_id`
///   *because* a line number *"would silently shift under the truth"*. This record instead carries
///   the names the corpus already froze: [`Self::trap`] and [`Self::replay`].
///   ⚠️ **That pair is not globally unique.** `TrapError::DuplicateId` is per FILE — *"two traps in
///   the same file share an id"* — so at ~50 traps across many files, two files could both define
///   `mac-randomized-01`. The key is provisional; a cross-file `TrapId` guard belongs with the
///   corpus-hygiene work.
/// - **`reason` is the TRAP AUTHOR's sentence**, not an engine explanation. D19 licenses it: *"the
///   oracle is the fixture's author, made explicit and versioned, with a mandatory `reason` field
///   on every expectation."* The architecture never disambiguates the two readings; this is the
///   choice, and it exists so a failure is readable without opening the corpus.
///
/// No `Serialize`: nothing persists these yet, and deriving a wire format for a domain type with no
/// consumer is a finding this project has already recorded once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoredRecord {
    /// The trap that was scored. Stable, authored, and never a line number.
    pub trap: TrapId,
    /// What the trap's author said is right.
    ///
    /// D36 justifies this record by post-hoc falsifiability — *"you cannot tell a regression from
    /// a legitimate re-derivation"* — and that analysis is impossible without the expectation: a
    /// record holding only an outcome cannot say whether it PASSED. Carrying it makes
    /// [`Self::score`] recomputable from the record alone, with no corpus in hand.
    pub expected: Expectation,
    /// The replay stream the trap judges, corpus-relative. A `String`, not a `PathBuf`: this is
    /// the domain crate, and `Trap::replay` is already a `String`.
    pub replay: String,
    /// What was answered.
    pub outcome: Outcome,
    /// The trap author's one-sentence reason — the oracle, carried so a failure reads on its own.
    pub reason: String,
    /// The descriptor under which the outcome was reached. The one D36 field with a real type, and
    /// since story 4.5b it is dated by the fixture rather than by a caller.
    pub capability_snapshot: Capabilities,
    /// Always `None` until Epic 13, and provably so — [`SourceState`] is uninhabited.
    pub source_state: Option<SourceState>,
    /// Always empty until an engine produces rules, and provably so —
    /// [`VerdictVectorEntry`] is uninhabited.
    pub verdict_vector: Vec<VerdictVectorEntry>,
}

impl ScoredRecord {
    /// Whether this trap passed — recomputed from the record, without the corpus.
    pub fn score(&self) -> Score {
        score(&self.expected, &self.outcome)
    }

    /// The D18 column this record belongs to.
    pub fn column(&self) -> Column {
        Column::of(&self.expected)
    }
}

/// Truth-table failures, per D18 column.
///
/// The number that blocks a release is [`Self::failures`] and it must be zero. The per-column
/// breakdown exists so a red gate says WHICH guard fell, not to be turned into a ratio: D18 refuses
/// fractions by name, and the three columns guard three different disasters.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tally {
    failures: BTreeMap<Column, usize>,
    scored: BTreeMap<Column, usize>,
}

impl Tally {
    /// Record one scored trap.
    pub fn record(&mut self, expected: &Expectation, actual: &Outcome) {
        let column = Column::of(expected);
        *self.scored.entry(column).or_insert(0) += 1;
        if score(expected, actual) == Score::Fail {
            *self.failures.entry(column).or_insert(0) += 1;
        }
    }

    /// **The number the gate publishes. It must be zero.**
    pub fn failures(&self) -> usize {
        self.failures.values().sum()
    }

    /// Failures in one column.
    ///
    /// **Read it with [`Self::scored_in`], never alone.** Zero failures in a column means the
    /// column passed OR that it was never exercised, and those are not the same news.
    pub fn failures_in(&self, column: Column) -> usize {
        self.failures.get(&column).copied().unwrap_or(0)
    }

    /// How many traps were scored. **Not a denominator for a rate** — it exists so a caller can
    /// tell "zero failures over three hundred traps" from "zero failures because nothing ran",
    /// which is the vacuity story 4.1 removed from the fixtures gate.
    pub fn scored(&self) -> usize {
        self.scored.values().sum()
    }

    /// How many traps were scored IN ONE COLUMN.
    ///
    /// A global count closes global vacuity and stops one level short of where this module's own
    /// argument says the risk lives. D18 localises the anti-cowardice guard to `must-merge`; a run
    /// containing no `must-merge` trap at all reports zero failures everywhere and is green, while
    /// an engine that abstains on everything walks through it. Only a per-column denominator can
    /// tell "the middle column held" from "the middle column was empty".
    pub fn scored_in(&self, column: Column) -> usize {
        self.scored.get(&column).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::{FactKind, Timestamp};
    use std::collections::BTreeSet;

    fn ts() -> Timestamp {
        chrono::DateTime::parse_from_rfc3339("2026-03-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc)
    }

    fn rule(name: &str) -> RuleId {
        RuleId(name.to_string())
    }

    fn must_merge() -> Expectation {
        Expectation::MustMerge {
            rule: rule("l1-exact-mac"),
        }
    }
    fn must_not_merge() -> Expectation {
        Expectation::MustNotMerge {
            rule: rule("l1-distinct-mac"),
        }
    }
    fn must_abstain() -> Expectation {
        Expectation::MustAbstain {
            cause: AbstentionCause::NoObservedValue,
        }
    }

    /// The outcomes deliberately carry rules and a cause that DO NOT match the expectations above.
    ///
    /// If they matched, an implementation that compared `(outcome, rule)` — story 4.7's criterion,
    /// not this module's — would pass every cell test, and the module's headline design decision
    /// would rest on a single guard. With them mismatched, all nine cell assertions defend it.
    fn merged() -> Outcome {
        Outcome::Merged {
            rule: rule("l2-uplink-agrees"),
        }
    }
    fn refused() -> Outcome {
        Outcome::Refused {
            rule: rule("l2-different-switch"),
        }
    }
    fn abstained() -> Outcome {
        Outcome::Abstained {
            cause: AbstentionCause::ConflictingObservations,
        }
    }

    // ── The nine cells (AC4) ─────────────────────────────────────────────────

    #[test]
    fn must_merge_passes_only_on_a_merge() {
        assert_eq!(score(&must_merge(), &merged()), Score::Pass);
        // A wrong decision fails at least as hard as a refusal to decide.
        assert_eq!(score(&must_merge(), &refused()), Score::Fail);
        // D18's named case: cowardice. This is the column the anti-cowardice guard lives in.
        assert_eq!(score(&must_merge(), &abstained()), Score::Fail);
    }

    #[test]
    fn must_not_merge_fails_only_on_a_merge() {
        // The false merge — "the operator loses trust and uninstalls".
        assert_eq!(score(&must_not_merge(), &merged()), Score::Fail);
        assert_eq!(score(&must_not_merge(), &refused()), Score::Pass);
        assert_eq!(score(&must_not_merge(), &abstained()), Score::Pass);
    }

    /// The load-bearing cell, with its own test because a reviewer will challenge it.
    ///
    /// It looks lenient. It is REQUIRED by D18's own argument: *"an engine that abstains on
    /// everything scores false-merge = 0 and gets demolished by the middle column."* That sentence
    /// is only TRUE if abstention passes `must-not-merge`. Make this cell a failure and the middle
    /// column is redundant — and D18's stated mechanism for catching cowardice describes nothing.
    ///
    /// The test below proves the mechanism itself, not just the cell.
    #[test]
    fn an_engine_that_abstains_on_everything_is_demolished_by_the_middle_column() {
        let mut tally = Tally::default();
        tally.record(&must_not_merge(), &abstained());
        tally.record(&must_merge(), &abstained());
        tally.record(&must_abstain(), &abstained());

        assert_eq!(
            tally.failures_in(Column::MustNotMerge),
            0,
            "an abstainer scores false-merge = 0 — that is exactly D18's premise"
        );
        assert_eq!(
            tally.failures_in(Column::MustMerge),
            1,
            "…and the middle column is what demolishes it"
        );
        assert_eq!(tally.failures_in(Column::MustAbstain), 0);
        assert_eq!(
            tally.failures(),
            1,
            "the gate is red, and the reason is cowardice"
        );
    }

    #[test]
    fn must_abstain_fails_on_any_decision() {
        // D18: the column fails on "a decision", and a merge is one — guessing on the honestly
        // ambiguous case is what this column guards (FR16).
        assert_eq!(score(&must_abstain(), &merged()), Score::Fail);
        // A refusal is equally a decision. D18 names no direction, and neither does this.
        assert_eq!(score(&must_abstain(), &refused()), Score::Fail);
        assert_eq!(score(&must_abstain(), &abstained()), Score::Pass);
    }

    /// The right answer reached by the WRONG rule is a PASS here — 4.7 owns that comparison.
    ///
    /// This test exists to catch the tempting implementation: deriving `PartialEq` between the
    /// expectation and the outcome, or comparing `rule` fields, would fail this cell and silently
    /// steal story 4.7's criterion (D64: `assert_eq!(decision.rule, case.expect_rule)` belongs
    /// "in the trap runner").
    #[test]
    fn scoring_ignores_the_rule_because_the_trap_runner_owns_it() {
        let expected = Expectation::MustMerge {
            rule: rule("l1-exact-mac"),
        };
        let by_another_rule = Outcome::Merged {
            rule: rule("l2-uplink-agrees"),
        };
        assert_eq!(score(&expected, &by_another_rule), Score::Pass);

        // Same on the refusal side, and same for a mismatched abstention cause.
        let refused_other = Outcome::Refused {
            rule: rule("something-else"),
        };
        assert_eq!(score(&must_not_merge(), &refused_other), Score::Pass);
        let other_cause = Outcome::Abstained {
            cause: AbstentionCause::ConflictingObservations,
        };
        assert_eq!(score(&must_abstain(), &other_cause), Score::Pass);
    }

    // ── The tally (AC5, AC6) ─────────────────────────────────────────────────

    #[test]
    fn the_tally_counts_failures_per_column_and_publishes_one_number() {
        let mut tally = Tally::default();
        assert_eq!(tally.failures(), 0);
        assert_eq!(
            tally.scored(),
            0,
            "an empty tally scored nothing, and says so"
        );

        tally.record(&must_merge(), &merged()); // pass
        tally.record(&must_not_merge(), &merged()); // fail: the false merge
        tally.record(&must_merge(), &abstained()); // fail: cowardice
        tally.record(&must_abstain(), &refused()); // fail: a guess

        assert_eq!(tally.scored(), 4);
        assert_eq!(tally.failures(), 3);
        assert_eq!(tally.failures_in(Column::MustNotMerge), 1);
        assert_eq!(tally.failures_in(Column::MustMerge), 1);
        assert_eq!(tally.failures_in(Column::MustAbstain), 1);
    }

    /// `scored` is not a denominator — it exists so a caller can tell a passing gate from one that
    /// measured nothing. This is the vacuity story 4.1 removed from the fixtures gate.
    #[test]
    fn zero_failures_over_nothing_is_distinguishable_from_zero_failures_over_something() {
        let vacuous = Tally::default();
        let mut real = Tally::default();
        real.record(&must_merge(), &merged());

        assert_eq!(
            vacuous.failures(),
            real.failures(),
            "both report zero failures"
        );
        assert_ne!(
            vacuous.scored(),
            real.scored(),
            "…and only `scored` tells them apart"
        );
    }

    /// The published number is a SUM of per-column counters, so accumulation past one must be
    /// pinned: mutating `*entry += 1` to `*entry = 1` would otherwise leave every test green.
    #[test]
    fn failures_accumulate_within_a_column() {
        let mut tally = Tally::default();
        for _ in 0..3 {
            tally.record(&must_merge(), &abstained());
        }
        assert_eq!(tally.failures_in(Column::MustMerge), 3);
        assert_eq!(tally.scored_in(Column::MustMerge), 3);
        assert_eq!(tally.failures(), 3);
    }

    /// Zero failures in a column means the column PASSED or that it was never exercised, and a
    /// caller must be able to tell those apart. This is the same vacuity `scored()` closes
    /// globally, one level down — and it is where this module's own argument says the risk lives,
    /// since D18 localises the anti-cowardice guard to `must-merge`.
    #[test]
    fn a_column_that_never_ran_is_distinguishable_from_one_that_passed() {
        let mut only_must_not_merge = Tally::default();
        only_must_not_merge.record(&must_not_merge(), &abstained());

        let mut middle_column_held = Tally::default();
        middle_column_held.record(&must_merge(), &merged());

        // Both are green, and both report zero failures in the middle column…
        assert_eq!(only_must_not_merge.failures(), 0);
        assert_eq!(middle_column_held.failures(), 0);
        assert_eq!(only_must_not_merge.failures_in(Column::MustMerge), 0);
        assert_eq!(middle_column_held.failures_in(Column::MustMerge), 0);
        // …and only the per-column denominator says one of them never tested for cowardice.
        assert_eq!(only_must_not_merge.scored_in(Column::MustMerge), 0);
        assert_eq!(middle_column_held.scored_in(Column::MustMerge), 1);
    }

    #[test]
    fn a_column_knows_its_expectation_and_its_d18_name() {
        assert_eq!(Column::of(&must_merge()), Column::MustMerge);
        assert_eq!(Column::of(&must_not_merge()), Column::MustNotMerge);
        assert_eq!(Column::of(&must_abstain()), Column::MustAbstain);
        // The vocabulary matches what `Expectation::column()` already returns, so a report and a
        // trap file speak the same words.
        assert_eq!(Column::of(&must_merge()).as_str(), must_merge().column());
        assert_eq!(
            Column::of(&must_not_merge()).as_str(),
            must_not_merge().column()
        );
        assert_eq!(
            Column::of(&must_abstain()).as_str(),
            must_abstain().column()
        );
    }

    // ── The two deferred fields, proven empty by CONSTRUCTION (AC8, AC10) ────

    /// `is_none()` would pass for any inhabited type, so it proves nothing. The witness that
    /// `SourceState` is UNINHABITED — and therefore that Epic 4 cannot populate the field however
    /// hard it tries — is that `Option<SourceState>` occupies no space at all.
    #[test]
    fn source_state_cannot_be_populated_in_epic_4() {
        // `size_of::<Option<T>>() == 0` cannot hold for an INHABITED `T`: `Option<T>` would then
        // have at least two distinct values (`None` and one `Some`) and need somewhere to put the
        // discriminant. So this witnesses uninhabitedness, which `is_none()` never could —
        // `is_none()` passes for any type at all.
        //
        // NOT asserted here, deliberately: `size_of::<SourceState>() == 0`, which is vacuous —
        // every zero-sized type passes it, including inhabited ones like `()`.
        //
        // This rests on a layout OPTIMISATION rather than a language guarantee (the Reference
        // specifies `Option<T>`'s layout only for the null-pointer cases), which is recorded in
        // `deferred-work.md`. Verified on rustc 1.97.1.
        assert_eq!(std::mem::size_of::<Option<SourceState>>(), 0);
        assert_eq!(std::mem::size_of::<Option<VerdictVectorEntry>>(), 0);
    }

    /// The verdict vector's emptiness needs its own witness: the field is a `Vec`, and
    /// `size_of::<Vec<T>>()` is the same three words whatever `T` is. What proves it is that no
    /// value of the element type can be constructed, so no `push` can ever compile — witnessed
    /// here by the element being uninhabited.
    #[test]
    fn the_verdict_vector_can_never_be_pushed_to() {
        let record = a_record();
        assert!(record.verdict_vector.is_empty());
        assert_eq!(std::mem::size_of::<Option<VerdictVectorEntry>>(), 0);
    }

    fn a_record() -> ScoredRecord {
        ScoredRecord {
            trap: TrapId("example-must-merge".into()),
            expected: must_merge(),
            replay: "scenario/replay/example-traps.jsonl".into(),
            outcome: merged(),
            reason: "both carry the identical MAC an hour apart, so only the lease moved.".into(),
            capability_snapshot: Capabilities {
                as_of: ts(),
                kinds: BTreeSet::from([FactKind::Mac, FactKind::IpV4]),
            },
            source_state: None,
            verdict_vector: Vec::new(),
        }
    }

    /// A record can say whether its trap PASSED, without the corpus in hand.
    ///
    /// That is the post-hoc analysis D36 justifies the record with — *"you cannot tell a regression
    /// from a legitimate re-derivation"* — and it is impossible unless the record carries what it
    /// was judged against. An earlier version of this test asserted `is_none()` and `contains()` on
    /// values it had constructed three lines above, and could only ever have failed to compile.
    #[test]
    fn a_record_can_recompute_its_own_score_without_the_corpus() {
        let mut record = a_record();
        assert_eq!(record.expected, must_merge());
        assert_eq!(record.score(), Score::Pass);
        assert_eq!(record.column(), Column::MustMerge);

        // Change only the ANSWER, and the record's own verdict flips — the record is not merely
        // carrying a label, it is recomputing the truth table.
        record.outcome = abstained();
        assert_eq!(
            record.score(),
            Score::Fail,
            "cowardice, recovered from the record alone"
        );
        assert_eq!(
            record.column(),
            Column::MustMerge,
            "the column follows the expectation"
        );
    }
}
