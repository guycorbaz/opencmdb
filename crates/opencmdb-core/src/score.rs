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

// ── Comparing two runs (story 4.6c) ─────────────────────────────────────────

/// Comparing one trap's [`ScoredRecord`] across two runs.
///
/// D36 is the whole reason this is not just `before == after`: *"A verdict without its capability
/// snapshot is UNFALSIFIABLE… Two verdicts are comparable only under an identical snapshot —
/// otherwise they are not two answers, they are two questions."* So a difference in the snapshot is
/// **refused**, never silently reported as "no change" — [`RecordComparison::IncomparableSnapshot`]
/// is a distinct outcome from [`RecordComparison::Identical`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordComparison {
    /// Same snapshot, same outcome — nothing changed.
    Identical,
    /// Same snapshot, DIFFERENT outcome — a change to investigate.
    ///
    /// This fires on a change of the RULE as well as the merge/no-merge/abstain answer, because
    /// [`Outcome`] carries the rule and equality compares it. That is deliberate and it is the
    /// D19/D46b case: *"same output, different reason… an engine divergence hiding behind a correct
    /// result — the worst kind"*. `score` (the release gate) ignores the rule; a run-to-run
    /// comparison does NOT, because a verdict reached by a different rule between two runs is
    /// exactly the drift a comparison exists to surface.
    Differing { before: Outcome, after: Outcome },
    /// The two records were reached under DIFFERENT capability snapshots, so they are not two
    /// answers to one question — they are two questions. Refused, and NEVER repaired: pinning or
    /// defaulting a capability to force a comparison is *"break[ing] the product to make CI green"*.
    ///
    /// `Capabilities` equality includes `as_of`, so two descriptors with the same `kinds` at a
    /// different `as_of` are also refused — a snapshot is a DATED fact (D34 §1), not a level. In
    /// practice two runs of one corpus share `as_of` (4.5b dates it from the file, not a clock), so
    /// this bites only a genuinely different descriptor.
    IncomparableSnapshot {
        before: Capabilities,
        after: Capabilities,
    },
}

/// The `(outcome, capability_snapshot)` a comparison looks at — extracted by ONE exhaustive
/// destructure so the field list is written once.
///
/// The destructure has no `..`: a field added to [`ScoredRecord`] must break THIS and force a
/// decision about whether it participates in a comparison — the mechanism 4.5b relied on. Every
/// other field is named and ignored on purpose:
/// - `trap`, `expected`, `replay` — identity; the caller matches on the trap, and the expectation
///   and stream come from the corpus, identical for one trap across two runs.
/// - `reason` — the trap author's sentence; same trap, same reason.
/// - `source_state` — excluded (AC6): uninhabited until Epic 13, so comparing it is vacuous today
///   and would silently start mattering the day it gains a type (this destructure forces that
///   decision then).
/// - `verdict_vector` — no producer until an engine; empty on both sides.
fn comparable_fields(record: &ScoredRecord) -> (&Outcome, &Capabilities) {
    let ScoredRecord {
        trap: _,
        expected: _,
        replay: _,
        outcome,
        reason: _,
        capability_snapshot,
        source_state: _,
        verdict_vector: _,
    } = record;
    (outcome, capability_snapshot)
}

/// Compare one trap's record across two runs — the primitive [`compare_runs`] is built from.
///
/// The two records must name the same trap; the caller matches them by [`TrapId`], and a
/// `debug_assert` catches a direct caller that does not. The snapshot is checked FIRST: a difference
/// there refuses the comparison before the outcomes are even looked at, because under different
/// capabilities the two verdicts answer different questions — so a coincidental outcome match must
/// NOT be reported as `Identical`.
pub fn compare_records(before: &ScoredRecord, after: &ScoredRecord) -> RecordComparison {
    debug_assert_eq!(
        before.trap, after.trap,
        "compare_records is for one trap across two runs; the caller matches by TrapId"
    );
    let (before_outcome, before_caps) = comparable_fields(before);
    let (after_outcome, after_caps) = comparable_fields(after);

    if before_caps != after_caps {
        return RecordComparison::IncomparableSnapshot {
            before: before_caps.clone(),
            after: after_caps.clone(),
        };
    }
    if before_outcome != after_outcome {
        RecordComparison::Differing {
            before: before_outcome.clone(),
            after: after_outcome.clone(),
        }
    } else {
        RecordComparison::Identical
    }
}

/// The result of comparing two whole runs, trap by trap.
///
/// A run is a set of [`ScoredRecord`]s. Comparability is **pairwise, not run-level**: 4.5b made the
/// capability descriptor positional, so two records in one run legitimately carry different
/// snapshots and "the run's snapshot" is not well-defined. A run may therefore be *partly*
/// comparable — some traps compared, others refused — and this report says which.
///
/// Every DISTINCT trap id lands in exactly one bucket. `incomparable` and `differing` are what a
/// reader acts on; `only_before`/`only_after` mean the run MEMBERSHIP changed, itself a difference.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunComparison {
    /// Traps whose record was identical in both runs.
    pub identical: Vec<TrapId>,
    /// Traps whose verdict changed under an identical snapshot: `(trap, before, after)`.
    pub differing: Vec<(TrapId, Outcome, Outcome)>,
    /// Traps refused because their snapshots differ, carrying BOTH snapshots so the report names the
    /// evidence D36 says is load-bearing — `(trap, before, after)`, not just the trap.
    pub incomparable: Vec<(TrapId, Capabilities, Capabilities)>,
    /// Traps present in the BEFORE run only — the membership changed.
    pub only_before: Vec<TrapId>,
    /// Traps present in the AFTER run only — the membership changed.
    pub only_after: Vec<TrapId>,
}

impl RunComparison {
    /// Whether the two runs are unchanged **in outcome and snapshot** — no differing verdict, no
    /// refused pair, no membership change.
    ///
    /// It does NOT assert the runs are byte-identical: the comparison ignores `reason`, `expected`,
    /// `replay`, `source_state` and `verdict_vector` (see [`comparable_fields`]), so two runs that
    /// differ only there report unchanged. A refusal is NOT "no difference" (D36), so a run with any
    /// `incomparable` pair is UNDECIDED, not unchanged, and this returns false.
    ///
    /// Note it is vacuously true for two EMPTY runs — "nothing to compare" reads as "unchanged". A
    /// caller that needs to tell that apart from "compared 300 identical traps" reads `identical`,
    /// the same way the harness reads `Tally::scored()`.
    pub fn is_unchanged(&self) -> bool {
        self.differing.is_empty()
            && self.incomparable.is_empty()
            && self.only_before.is_empty()
            && self.only_after.is_empty()
    }
}

/// Compare two runs trap by trap. Pure: no I/O, no clock — two in-memory sets of records in, one
/// [`RunComparison`] out.
///
/// **Precondition: each run names every trap at most once.** Story 4.6b's `DuplicateTrapId` guard
/// enforces that on a corpus-produced run; a `debug_assert` catches a malformed run here rather than
/// silently keeping the last record and erasing the earlier one — the "never silent" rule this
/// module lives by.
pub fn compare_runs(before: &[ScoredRecord], after: &[ScoredRecord]) -> RunComparison {
    let index = |run: &[ScoredRecord]| -> BTreeMap<TrapId, ScoredRecord> {
        let by_trap: BTreeMap<TrapId, ScoredRecord> =
            run.iter().map(|r| (r.trap.clone(), r.clone())).collect();
        debug_assert_eq!(
            by_trap.len(),
            run.len(),
            "a run names a trap more than once — the corpus guard (4.6b) should prevent this"
        );
        by_trap
    };
    let before_by_trap = index(before);
    let after_by_trap = index(after);

    let mut out = RunComparison::default();
    for (trap, before_record) in &before_by_trap {
        match after_by_trap.get(trap) {
            None => out.only_before.push(trap.clone()),
            Some(after_record) => match compare_records(before_record, after_record) {
                RecordComparison::Identical => out.identical.push(trap.clone()),
                RecordComparison::Differing { before, after } => {
                    out.differing.push((trap.clone(), before, after))
                }
                RecordComparison::IncomparableSnapshot { before, after } => {
                    out.incomparable.push((trap.clone(), before, after))
                }
            },
        }
    }
    for trap in after_by_trap.keys() {
        if !before_by_trap.contains_key(trap) {
            out.only_after.push(trap.clone());
        }
    }
    out
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

    // ── Comparing two runs (story 4.6c) ──────────────────────────────────────

    /// A record with a chosen trap id, snapshot and outcome, built from `a_record`.
    fn record_with(trap: &str, caps: Capabilities, outcome: Outcome) -> ScoredRecord {
        ScoredRecord {
            trap: TrapId(trap.into()),
            capability_snapshot: caps,
            outcome,
            ..a_record()
        }
    }

    fn caps_full() -> Capabilities {
        Capabilities {
            as_of: ts(),
            kinds: BTreeSet::from([FactKind::Mac, FactKind::IpV4, FactKind::Rtt]),
        }
    }

    /// The NET_RAW-lost descriptor — the positional-downgrade case 4.5b introduced and
    /// `capability-downgrade.jsonl` commits. A verdict reached under this is not comparable to one
    /// reached under `caps_full`.
    fn caps_downgraded() -> Capabilities {
        Capabilities {
            as_of: ts(),
            kinds: BTreeSet::from([FactKind::Mac, FactKind::IpV4]),
        }
    }

    #[test]
    fn same_snapshot_same_outcome_is_identical() {
        let a = record_with("t", caps_full(), merged());
        let b = record_with("t", caps_full(), merged());
        assert_eq!(compare_records(&a, &b), RecordComparison::Identical);
    }

    #[test]
    fn same_snapshot_different_outcome_is_a_real_difference() {
        let a = record_with("t", caps_full(), merged());
        let b = record_with("t", caps_full(), refused());
        assert_eq!(
            compare_records(&a, &b),
            RecordComparison::Differing {
                before: merged(),
                after: refused()
            }
        );
    }

    /// AC2's core: a differing snapshot is REFUSED, not reported as "no change".
    ///
    /// D36: two verdicts under different capabilities are two questions, not two answers. This is
    /// the exact case `capability-downgrade.jsonl` produces — the same trap scored under NET_RAW and
    /// under ping-only.
    #[test]
    fn a_differing_snapshot_is_refused_not_silently_equal() {
        let a = record_with("t", caps_full(), merged());
        let b = record_with("t", caps_downgraded(), merged());
        let c = compare_records(&a, &b);
        assert_eq!(
            c,
            RecordComparison::IncomparableSnapshot {
                before: caps_full(),
                after: caps_downgraded(),
            }
        );
        // The assertion that makes AC2 non-vacuous: refusal is a DISTINCT outcome from "identical".
        // A caller must be able to tell "the same answer" from "not comparable".
        assert_ne!(c, RecordComparison::Identical);
    }

    /// Even when the OUTCOMES agree, a differing snapshot still refuses — the snapshot is checked
    /// first, because under different capabilities the agreement is a coincidence, not a re-derivation.
    #[test]
    fn a_differing_snapshot_refuses_even_when_the_outcomes_match() {
        let a = record_with("t", caps_full(), merged());
        let b = record_with("t", caps_downgraded(), merged());
        assert!(matches!(
            compare_records(&a, &b),
            RecordComparison::IncomparableSnapshot { .. }
        ));
    }

    #[test]
    fn compare_runs_buckets_every_trap_and_is_partly_comparable() {
        // BEFORE: three traps under the full descriptor.
        let before = vec![
            record_with("identical", caps_full(), merged()),
            record_with("differing", caps_full(), merged()),
            record_with("downgraded", caps_full(), merged()),
            record_with("gone", caps_full(), merged()),
        ];
        // AFTER: one identical, one with a changed verdict, one under a downgraded descriptor
        // (refused), one new trap, and "gone" absent.
        let after = vec![
            record_with("identical", caps_full(), merged()),
            record_with("differing", caps_full(), refused()),
            record_with("downgraded", caps_downgraded(), merged()),
            record_with("new", caps_full(), merged()),
        ];
        let cmp = compare_runs(&before, &after);
        assert_eq!(cmp.identical, vec![TrapId("identical".into())]);
        // The differing bucket carries the BEFORE and AFTER verdicts, not just the trap id — a
        // reader must see what changed, so assert the whole `(trap, before, after)` payload.
        assert_eq!(
            cmp.differing,
            vec![(TrapId("differing".into()), merged(), refused())]
        );
        // The incomparable bucket carries BOTH snapshots (D36's evidence), not just the trap id.
        assert_eq!(
            cmp.incomparable,
            vec![(TrapId("downgraded".into()), caps_full(), caps_downgraded())]
        );
        assert_eq!(cmp.only_before, vec![TrapId("gone".into())]);
        assert_eq!(cmp.only_after, vec![TrapId("new".into())]);
        // A run with an incomparable pair is UNDECIDED, never "unchanged" (D36).
        assert!(!cmp.is_unchanged());
    }

    #[test]
    fn two_identical_runs_are_unchanged() {
        let run = vec![
            record_with("a", caps_full(), merged()),
            record_with("b", caps_downgraded(), refused()),
        ];
        let cmp = compare_runs(&run, &run.clone());
        assert!(cmp.is_unchanged(), "identical runs, every pair comparable");
        assert_eq!(cmp.identical.len(), 2);
    }

    /// A run whose ONLY change is an incomparable pair is not unchanged — the refusal is a distinct
    /// state from "no difference", one level up from the record comparison.
    #[test]
    fn a_run_with_only_an_incomparable_pair_is_not_unchanged() {
        let before = vec![record_with("t", caps_full(), merged())];
        let after = vec![record_with("t", caps_downgraded(), merged())];
        let cmp = compare_runs(&before, &after);
        assert_eq!(cmp.incomparable.len(), 1);
        assert!(cmp.differing.is_empty());
        assert!(
            !cmp.is_unchanged(),
            "an undecided comparison is not a passing one"
        );
    }

    /// Same verdict COLUMN, different RULE, identical snapshot → `Differing`.
    ///
    /// This is the D19/D46b drift the review chose (option a) to keep sensitive to: `score` (the
    /// gate) collapses these to one Pass because the rule is irrelevant to correctness, but a
    /// run-to-run comparison surfaces it — *"same output, different reason… the worst kind"*. Two
    /// `Merged` outcomes differing only in `RuleId` must NOT read as `Identical`.
    #[test]
    fn same_verdict_different_rule_is_differing_not_identical() {
        let by_rule_a = Outcome::Merged { rule: rule("l2-a") };
        let by_rule_b = Outcome::Merged { rule: rule("l2-b") };
        let a = record_with("t", caps_full(), by_rule_a.clone());
        let b = record_with("t", caps_full(), by_rule_b.clone());
        assert_eq!(
            compare_records(&a, &b),
            RecordComparison::Differing {
                before: by_rule_a,
                after: by_rule_b,
            },
            "a verdict reached by a different rule between two runs is drift, not sameness"
        );
    }

    /// Two snapshots with identical `kinds` at a DIFFERENT `as_of` are refused — a snapshot is a
    /// dated fact (D34 §1), so `Capabilities` equality includes `as_of` and the comparison inherits
    /// it. Guards the `IncomparableSnapshot` doc claim that `as_of` participates.
    #[test]
    fn same_kinds_different_as_of_is_incomparable() {
        let later = ts() + chrono::Duration::seconds(1);
        let a = record_with("t", caps_full(), merged());
        let b_caps = Capabilities {
            as_of: later,
            kinds: caps_full().kinds,
        };
        let b = record_with("t", b_caps, merged());
        assert!(
            matches!(
                compare_records(&a, &b),
                RecordComparison::IncomparableSnapshot { .. }
            ),
            "a different as_of is a different dated fact, so the pair is two questions"
        );
    }

    /// Two EMPTY runs compare as unchanged (vacuously): no differing pair, no refusal, no
    /// membership change, and every bucket empty. Documents the `is_unchanged` edge the doc warns
    /// a caller to disambiguate via `identical`.
    #[test]
    fn two_empty_runs_are_vacuously_unchanged() {
        let cmp = compare_runs(&[], &[]);
        assert!(cmp.is_unchanged());
        assert!(cmp.identical.is_empty());
        assert_eq!(cmp, RunComparison::default());
    }

    /// A trap present in only one of two runs lands in the right membership bucket even when the
    /// other run is empty — the two halves of `compare_runs` (before-loop, after-loop) each stand
    /// alone.
    #[test]
    fn a_trap_in_one_empty_sided_run_is_a_membership_change() {
        let only = vec![record_with("t", caps_full(), merged())];
        let forward = compare_runs(&only, &[]);
        assert_eq!(forward.only_before, vec![TrapId("t".into())]);
        assert!(!forward.is_unchanged());
        let backward = compare_runs(&[], &only);
        assert_eq!(backward.only_after, vec![TrapId("t".into())]);
        assert!(!backward.is_unchanged());
    }

    /// Two traps can share one bucket — the buckets are `Vec`s, not single slots. A regression that
    /// kept only the last trap per bucket (an easy indexing mistake) would red this.
    #[test]
    fn two_traps_can_share_a_bucket() {
        let before = vec![
            record_with("x", caps_full(), merged()),
            record_with("y", caps_full(), merged()),
        ];
        let after = vec![
            record_with("x", caps_full(), refused()),
            record_with("y", caps_full(), refused()),
        ];
        let cmp = compare_runs(&before, &after);
        assert_eq!(cmp.differing.len(), 2, "both traps changed, both reported");
        let traps: BTreeSet<_> = cmp.differing.iter().map(|(t, _, _)| t.clone()).collect();
        assert_eq!(
            traps,
            BTreeSet::from([TrapId("x".into()), TrapId("y".into())])
        );
    }
}
