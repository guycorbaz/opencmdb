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
//! # `score` does not compare rules; [`run_trap`] layers that on top
//!
//! [`score`] is rule-blind: `(MustMerge { rule: A }, Merged { rule: B })` — the right answer reached
//! by the wrong rule — is a **PASS** to it. That is deliberate and it is D18's truth table, which is
//! about the verdict alone. D64 revoked D46b but kept its first criterion and *"it changes owner:
//! compare `(verdict, rule)`, never `verdict` alone… it becomes `assert_eq!(decision.rule,
//! case.expect_rule)` **in the trap runner**"* — story 4.7a. That assertion lives in [`run_trap`],
//! which calls [`score`] first and only then compares the rule, so the truth table's meaning is
//! unchanged and a wrong rule is a **distinct** failure beside it, not a tenth cell. Deriving
//! `PartialEq` between an expectation and an outcome would fold the two together and destroy that
//! separation — so it is not derived.

use std::collections::BTreeMap;

use crate::identity::cascade::{Conclusion, Decision, IdentityAbstentionCause};
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
/// **Not named `Decision`, and since story 5.4 that name is taken.**
/// [`crate::identity::cascade::Decision`] is the ENGINE's return — a conclusion, its verdict vector
/// and the ruleset version that produced it. This type is the HARNESS's record of an answer, which
/// may equally be hand-authored in a test. The two mirror each other's algebra (a decision names a
/// rule, an abstention names a cause) and differ in envelope. Since story 5.7 ONE function converts
/// between them — [`outcome_of`], in this module — and it is a named function rather than a `From`
/// impl precisely because the conversion is a decision about the release gate.
///
/// # The two sides of a trap speak different abstention vocabularies (story 5.3)
///
/// An outcome abstains in the ENGINE's vocabulary,
/// [`crate::identity::cascade::IdentityAbstentionCause`] — `Ambiguous`, which arises from the
/// verdict algebra (the cloned-MAC case), and `AbsenceOfProof`. The expectation abstains in the
/// CORPUS's, [`crate::gap::AbstentionCause`], which story 4.2 froze into the truth format and
/// which three committed trap files write as `must-abstain = { cause = "NoObservedValue" }`.
///
/// Story 4.6a used the corpus vocabulary on both sides and recorded that it cannot name
/// `Ambiguous`; story 5.3 took the second of the two branches it left open — a separate cause type
/// rather than a widened `AbstentionCause`, because a variant added there is one the corpus format
/// can express, that `cause_label` must label and that two locales must translate, for something
/// `reconcile` can never produce.
///
/// **Nothing compares the two, and that is structural rather than promised:** [`score`]'s 3×3
/// matches `Outcome::Abstained { .. }` and cannot reach the payload, and [`run_trap`] compares
/// rules only where both sides are `Some`, which an abstention never is ([`Outcome::rule`] returns
/// `None` for one by type). So two different types on the two sides cannot make the gate
/// asymmetric — there is no comparison to go asymmetric.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// These observations describe one device, by this rule.
    Merged { rule: RuleId },
    /// These observations describe different devices, and this rule opposes the merge.
    Refused { rule: RuleId },
    /// The signal was insufficient; no decision was taken, for this cause.
    Abstained { cause: IdentityAbstentionCause },
}

impl Outcome {
    /// The rule a DECISION fired, or `None` for an abstention — the mirror of
    /// [`Expectation::rule`].
    ///
    /// A merge names the rule that fired; a refusal names the rule that OPPOSED the merge. An
    /// abstention took no decision, so it carries a cause and no rule — the type says so, which is
    /// what lets [`run_trap`] leave an abstention out of the `(verdict, rule)` assertion by
    /// construction rather than by a runtime guard.
    pub fn rule(&self) -> Option<&RuleId> {
        match self {
            Outcome::Merged { rule } | Outcome::Refused { rule } => Some(rule),
            Outcome::Abstained { .. } => None,
        }
    }
}

/// Why a producer could not put a trap to its engine at all (story 5.8).
///
/// ⚠️ **This is not an abstention, and the distinction is the whole of story 5.8.** An abstention is
/// an ANSWER — the engine was asked, evaluated, and declined to decide; it is
/// [`Outcome::Abstained`] and it PASSES the `must-abstain` column. An unanswerable trap is one the
/// engine was **never asked about**. See [`Answer`] for the measurement that keeps them apart.
///
/// The three variants are the three classes measured over the committed corpus at story 5.8's
/// contexting — **8 / 2 / 1** of the eleven traps the L1 engine leaves unanswered. They are made
/// mutually exclusive by the producer consulting the PAIR condition first (story 5.8 §4):
/// `example-must-abstain` is in two classes at once — it names a cause and no rule AND it names one
/// observation — and pair-first files it under [`Self::NoPairUnderJudgement`], because *cannot be
/// asked* outranks *cannot be routed*.
///
/// Exhaustive with no `_` arm wherever it is matched: a fourth class must break the build and force
/// a decision about how the gate counts it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnanswerableCause {
    /// The trap's expectation names a rule at a cascade level this engine does not implement — the
    /// eight `l2-*` traps of the committed corpus.
    ///
    /// `expected` is the rule the trap's AUTHOR named, frozen in Epic 4 before any engine existed.
    /// It is never a rule the engine chose: the producer reads the expectation's level and nothing
    /// else, which is what keeps the exclusion from being *"we skipped the ones we fail"*.
    LevelNotImplemented {
        /// The rule id the trap's author said answers this case.
        expected: RuleId,
    },
    /// The trap's expectation names a CAUSE and no rule, so there is no level to route on — the two
    /// paired `must-abstain` traps (`hostname-absence-must-abstain`,
    /// `shared-hardware-vm-must-abstain`).
    ///
    /// [`Expectation::rule`] returns `None` for a `must-abstain`, which is why these are invisible
    /// to an `l2-*` selector and why story 5.7 measured the residue at eleven where `epics.md` had
    /// said eight.
    NoLevelToRouteOn,
    /// The trap does not put a PAIR under judgement, so there is no question to form at any level —
    /// `example-must-abstain`, which names one observation.
    ///
    /// Unanswerable at every level, present and future, which is why pair-first files it here
    /// rather than under a level it does not have.
    NoPairUnderJudgement,
}

/// What a producer says about one trap: it answered, or it declined and named why (story 5.8).
///
/// This is the value of the map the release gate's harness takes — `opencmdb_bin`'s
/// `trap_gate::score_corpus`, which lives outside this crate because it reads files (D47).
/// Until story 5.8 that map held a bare [`Outcome`] and **absence was the only way to say "not
/// answered"**. Absence cannot carry a reason, and a trap that leaves the denominator without one is
/// how a green gate comes to mean *"we did not ask the question"* — which `epics.md`'s story 5.8
/// forbids: the unanswerable traps *"never silently leave the denominator"*.
///
/// A trap that is both answered and declared unanswerable is **unrepresentable** rather than merely
/// invalid — `trap.rs`'s stated idiom for [`Expectation`], applied here. That is why this is one
/// enum and not two maps.
///
/// # 🔴 [`Self::Unanswerable`] is NOT an abstention
///
/// The tempting shortcut is to record a declined trap as `Outcome::Abstained` and let the existing
/// truth table deal with it. **Measured, and it is not hypothetical:** `example-must-abstain`'s
/// expectation is `must-abstain`, and `(must-abstain, Abstained)` is [`Score::Pass`] — so the trap
/// would PASS *because nothing was asked*, and put a 1 in the `must-abstain` column of a gate that
/// never ran. That is D18's cowardice, moved up one level from the engine to the harness.
///
/// So this variant **never becomes an [`Outcome`]**: there is no `From`, no `Default` and no helper
/// anywhere that converts it, and it never reaches [`Tally::record`]. The same refusal, for the same
/// reason, keeps [`outcome_of`] a named function rather than a `From` impl.
///
/// Story 5.7 refused the identical shortcut one layer down, in the runner
/// (`opencmdb_bin::l1_runner::answer_trap` gives a pairless trap NO answer rather than the
/// `decide(vec![], _)` that would have passed). This is that refusal at the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// The engine was asked and answered. Scored by the truth table exactly as before story 5.8.
    Answered(
        /// What the engine concluded, in the harness's vocabulary.
        Outcome,
    ),
    /// The engine was never asked, for this reason. Counted in a blocking bucket; never scored.
    Unanswerable {
        /// Why the producer could not put this trap to the engine.
        cause: UnanswerableCause,
    },
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

/// The verdict of RUNNING one trap: [`score`]'s truth table, plus the `(verdict, rule)` assertion.
///
/// ⚠️ **Not [`crate::identity::cascade::Verdict`]**, which is what ONE RULE says about one candidate
/// pair (D13's five-variant enum). This type is what the RUNNER says about one trap. The four
/// judgements in play are tabulated in `identity::cascade`'s module doc.
///
/// D46b's surviving criterion (D64): *"compare `(verdict, rule)`, never `verdict` alone"*. [`score`]
/// answers the verdict question and stays rule-blind; this adds the rule question ON TOP, so the two
/// failure modes stay distinct — a wrong verdict and a right verdict by the wrong rule are not the
/// same news, and D46b's whole point is that the second *"survives until the data changes"* and is
/// *"the worst kind"*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrapVerdict {
    /// The verdict is right AND (where a decision carries a rule) it fired the expected rule.
    Pass,
    /// The verdict itself is wrong — [`score`] failed. The rule is not consulted: a wrong answer
    /// fails at the coarser question first, whatever rule it named.
    VerdictFail,
    /// The verdict is right but the DECISION fired the wrong rule. The right answer by the wrong
    /// rule — D46b's *"same output, different reason… the worst kind"*. Names both rules so the
    /// failure is debuggable without opening the corpus.
    WrongRule {
        /// The rule the trap's author said must fire (or oppose the merge).
        expected: RuleId,
        /// The rule the answer actually fired.
        actual: RuleId,
    },
}

/// Run one trap: score the verdict, then assert the rule — the trap runner's `(verdict, rule)` check.
///
/// The order is load-bearing. [`score`] is asked FIRST: a verdict that is not a [`Score::Pass`] is
/// [`TrapVerdict::VerdictFail`] and the rule is never looked at, because the verdict is the coarser
/// question and a wrong answer fails at it regardless of the rule it named. The gate is written on the
/// POSITIVE (`!= Score::Pass`), not on `== Score::Fail`: only a proven pass may proceed to the rule
/// check, so a verdict this function cannot prove right never falls through to a rule comparison.
/// Only on a verdict pass is the rule compared, and only where a DECISION carries one on both sides:
/// [`Expectation::rule`] and [`Outcome::rule`] are both `Some` exactly on the two decision cells
/// (`must-merge → Merged`, `must-not-merge → Refused`), so [`TrapVerdict::WrongRule`] fires there and
/// nowhere else. An abstention has no rule and passes — which keeps `must-not-merge → Abstained`,
/// [`score`]'s load-bearing pass cell, a pass here too.
pub fn run_trap(expected: &Expectation, actual: &Outcome) -> TrapVerdict {
    if score(expected, actual) != Score::Pass {
        return TrapVerdict::VerdictFail;
    }
    // Verdict is right. Now the rule, but only where a decision names one on both sides. An
    // abstention on either side yields `None` and is therefore a pass — an abstention has no rule to
    // be wrong (AC4).
    match (expected.rule(), actual.rule()) {
        (Some(expected_rule), Some(actual_rule)) if expected_rule != actual_rule => {
            TrapVerdict::WrongRule {
                expected: expected_rule.clone(),
                actual: actual_rule.clone(),
            }
        }
        _ => TrapVerdict::Pass,
    }
}

/// Map the ENGINE's return onto the HARNESS's record of an answer — the seam story 5.7 crosses.
///
/// [`crate::identity::cascade::Decision`] is what the identity cascade concluded; [`Outcome`] is
/// what the release gate writes down about an answer, from any source including a hand-authored
/// test value. Until this function existed **nothing converted between them**, and the conversion
/// was withheld on purpose: *"mapping the engine's return onto the harness's record is a decision
/// about the release gate… not a silent conversion"* [`Decision`'s doc]. This is that decision,
/// taken and named.
///
/// # The three rows
///
/// | [`Conclusion`] | [`Outcome`] |
/// |---|---|
/// | `Match { rule }` | `Merged { rule }` |
/// | `NoMatch { rule }` | `Refused { rule }` |
/// | `Abstained { cause }` | `Abstained { cause }` |
///
/// The two algebras mirror each other — a decision names a rule, an abstention names a cause — so
/// the mapping is total on the ANSWER and needs no fallback. The abstaining row carries the SAME
/// type on both sides: [`IdentityAbstentionCause`] since story 5.3. It is not a translation between
/// the two abstention vocabularies, which stay unbridged; [`Expectation::MustAbstain`]'s
/// [`crate::gap::AbstentionCause`] is never touched here.
///
/// # What is DROPPED, and why that is not a bug to fix here
///
/// [`Decision::verdict_vector`] and [`Decision::ruleset_version`] have **nowhere to go**: `Outcome`
/// is a three-variant enum with no envelope. [`ScoredRecord`] is where a run would carry them —
/// its [`ScoredRecord::verdict_vector`] is exactly D18's *"COMPLETE VERDICT VECTOR"* requirement —
/// and it is uninhabited ([`VerdictVectorEntry`]) because no run in this crate is produced by an
/// engine. So the loss is real and it is bounded by what the destination can hold, not chosen
/// here: the day a trap run records a [`ScoredRecord`], the vector and the version are what that
/// story must carry, and this function is where a reader finds out they were dropped.
///
/// [`Decision::rule`] and [`Outcome::rule`] mirror each other, and this mapping preserves that
/// mirror: `outcome_of(&d).rule() == d.rule()` on every row. That is what makes [`run_trap`]'s
/// `(verdict, rule)` assertion mean the same thing about an engine answer as about a hand-authored
/// one, and a test pins it.
///
/// # Why a free function and not `impl From<Decision> for Outcome`
///
/// A `From` makes the conversion free at every call site — `.into()` — which is precisely the
/// invisibility [`Decision`]'s doc refused. A named function has to be typed out, so a reader of a
/// call site sees that a gate decision was taken. The same refusal, for the same reason, keeps the
/// two abstention vocabularies unbridged (story 5.3).
///
/// It lives HERE and not in `identity/`: this is knowledge about the release gate, and the engine
/// must not acquire it. `Outcome` already names [`IdentityAbstentionCause`] in a field type, so the
/// dependency direction is unchanged.
///
/// Exhaustive with no `_` arm: a fourth [`Conclusion`] variant must break THIS function with
/// `error[E0004]` and force a decision about how the gate records it.
pub fn outcome_of(decision: &Decision) -> Outcome {
    match &decision.conclusion {
        Conclusion::Match { rule } => Outcome::Merged { rule: rule.clone() },
        Conclusion::NoMatch { rule } => Outcome::Refused { rule: rule.clone() },
        Conclusion::Abstained { cause } => Outcome::Abstained { cause: *cause },
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
/// The vector's element is `(rule, verdict, evidence)`, and since story 5.4 that triple HAS a type
/// — [`crate::identity::cascade::RuleVerdict`], on the engine's side. A rule now speaks and a
/// verdict vector IS built ([`crate::identity::l1`]), but **nothing feeds it to the harness**: no
/// run here is produced by that engine. This placeholder therefore stays **uninhabited** rather than
/// being replaced, so the field is provably empty by the same standard as [`SourceState`], instead
/// of being empty by comment.
///
/// # Story 5.7 did NOT unify the two, and the obstacle is measured rather than a matter of appetite
///
/// An engine answer now reaches the harness — `opencmdb_bin`'s `l1_runner` produces one per trap
/// and [`outcome_of`] maps it — but an [`Outcome`] is where it lands, and an `Outcome` carries no
/// vector. Unifying this type means producing a [`ScoredRecord`], and a `ScoredRecord` carries
/// [`ScoredRecord::capability_snapshot`], which is D36's whole point: *"a verdict without its
/// capability snapshot is UNFALSIFIABLE"*. Measured on the committed corpus: **eleven replay
/// streams are named by a trap and not one of them carries a `capability` control record**, and the
/// reader the trap runner uses discards control records by construction. Producing a record here
/// would mean **inventing a capability snapshot for all 24 traps** — D36's unfalsifiability in
/// reverse, and D45's *"a gate on a false truth"*.
///
/// **Owner: the story that gives a trap run a real capability snapshot** — the `FixtureConnector`
/// read path, which replays control records, rather than the observations-only reader. Recorded in
/// `deferred-work.md` with that condition spelled out.
///
/// This is also what PINS story 4.7a's AC6 forward contract: [`run_trap`] asserts `(verdict, rule)`
/// today, but the requirement that *a firing rule leave its `rule_id` and evidence behind* needs an
/// engine to enforce. It now IS a mechanism, on the engine's side:
/// [`crate::identity::l1::verdict_for_pair`] carries both `ObsId`s on every verdict that argues, and
/// a test reds if it stops. ⚠️ **That mechanism still has not reached HERE**: the evidence is
/// carried on the [`crate::identity::cascade::Decision`] and dropped by [`outcome_of`], because the
/// destination has nowhere to put it.
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
    /// Always empty, and provably so — [`VerdictVectorEntry`] is uninhabited.
    ///
    /// ⚠️ The condition this used to name — *"until an engine produces rules"* — has been MET since
    /// story 5.5 and the vector is still empty, so it was not the condition. The one that holds is
    /// on [`VerdictVectorEntry`]'s own doc: a trap run that can reach a real capability snapshot.
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
/// - `verdict_vector` — an engine now produces verdicts (`identity::l1`) and story 5.7 brought its
///   ANSWER to the harness, but not its vector: [`outcome_of`] drops it, because [`Outcome`] has
///   nowhere to put it. So this stays empty on both sides, and the story that gives a trap run a
///   real capability snapshot is the one that can fill it (see [`VerdictVectorEntry`]).
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
    // The CORPUS's abstention vocabulary, which lives on the `Expectation` side only (story 5.3).
    // `use super::*` no longer brings it in: production `score` names the ENGINE's type, and an
    // import kept alive solely by this module would be an unused import in the lib build CI
    // compiles (`cargo clippy --workspace` without `--all-targets`).
    use crate::gap::AbstentionCause;
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

    /// The outcomes deliberately carry rules that DO NOT match the expectations above — and, since
    /// story 5.3, a cause that CANNOT match, because the two sides no longer share a type.
    ///
    /// If the rules matched, an implementation that compared `(outcome, rule)` — story 4.7's
    /// criterion, not this module's — would pass every cell test, and the module's headline design
    /// decision would rest on a single guard. With them mismatched, all nine cell assertions defend
    /// it. On the cause the mismatch was a choice until 5.3 retyped
    /// [`Outcome::Abstained`]; it is now a type-level fact, and the deliberate part is which
    /// variant each helper carries — see `other_cause` below.
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
            cause: IdentityAbstentionCause::Ambiguous,
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

        // Same on the refusal side. And the abstention leg, which since story 5.3 proves something
        // stronger than it used to: the mismatch is no longer between two values of one enum but
        // ACROSS the two vocabularies — `must_abstain()` carries the corpus's
        // `AbstentionCause::NoObservedValue`, this outcome carries the engine's
        // `IdentityAbstentionCause`, and scoring still passes because it reads neither.
        // The variant DIFFERS from `abstained()`'s so the two helpers stay distinguishable at a
        // glance. It is no longer the load-bearing reason it once was: since 5.3,
        // `scoring_is_blind_to_the_abstention_cause_whatever_it_is` asserts this same pair inside
        // its loop, so the sub-claim survives either way. Kept different because two helpers that
        // are value-identical but differently named are a trap for the next reader.
        let refused_other = Outcome::Refused {
            rule: rule("something-else"),
        };
        assert_eq!(score(&must_not_merge(), &refused_other), Score::Pass);
        let other_cause = Outcome::Abstained {
            cause: IdentityAbstentionCause::AbsenceOfProof,
        };
        assert_eq!(score(&must_abstain(), &other_cause), Score::Pass);
    }

    // ── The two abstention vocabularies (story 5.3) ──────────────────────────

    /// Scoring reads no cause, for EVERY cause the engine can name.
    ///
    /// The three abstention cells are asserted per variant rather than once: `score`'s blindness is
    /// structural (it matches `Outcome::Abstained { .. }`), and this is what says so at the value
    /// level — including the load-bearing `(must-not-merge, Abstained) → Pass` cell, which D18's own
    /// anti-cowardice argument rests on.
    ///
    /// The loop is over `all()` rather than two hand-written blocks: one behaviour, one source of
    /// truth. `all()` itself is the deliberate redundancy — its exhaustive match is what makes a new
    /// variant a compile error — and it must not be collapsed into a bare literal.
    #[test]
    fn scoring_is_blind_to_the_abstention_cause_whatever_it_is() {
        for cause in IdentityAbstentionCause::all() {
            let abstained = Outcome::Abstained { cause };
            assert_eq!(
                score(&must_abstain(), &abstained),
                Score::Pass,
                "(must-abstain, Abstained{{{cause:?}}}) is the column's passing cell"
            );
            assert_eq!(
                score(&must_not_merge(), &abstained),
                Score::Pass,
                "(must-not-merge, Abstained{{{cause:?}}}) is the load-bearing pass cell (D18)"
            );
            assert_eq!(
                score(&must_merge(), &abstained),
                Score::Fail,
                "(must-merge, Abstained{{{cause:?}}}) is cowardice, and it fails"
            );
        }
    }

    /// The claim of story 5.3, made executable: the two sides of a trap carry DIFFERENT cause types
    /// and nothing compares them.
    ///
    /// The expectation carries a `AbstentionCause` — including `NoObservedValue`, the spelling the
    /// three committed `must-abstain` trap files write — and the outcome carries an
    /// `IdentityAbstentionCause`, which no `AbstentionCause` can even name. Every pair passes,
    /// because [`score`] reads neither payload and [`run_trap`] finds no rule on either side.
    ///
    /// **All SIX pairs, not one.** The corpus side is enumerated by hand because `AbstentionCause`
    /// belongs to another module and has no `all()`; that literal is the redundancy — if a variant
    /// is added there, this list does not follow, which is a gap the reconciliation enum's own
    /// exhaustive consumers (`page.rs`'s `cause_label`) catch first.
    #[test]
    fn the_two_abstention_vocabularies_are_never_compared() {
        let corpus_causes = [
            AbstentionCause::OutOfPerimeter,
            AbstentionCause::NoObservedValue,
            AbstentionCause::ConflictingObservations,
        ];
        for corpus_cause in corpus_causes {
            let corpus_side = Expectation::MustAbstain {
                cause: corpus_cause,
            };
            for cause in IdentityAbstentionCause::all() {
                let engine_side = Outcome::Abstained { cause };
                assert_eq!(
                    score(&corpus_side, &engine_side),
                    Score::Pass,
                    "a corpus {corpus_cause:?} expectation answered by the engine's {cause:?} passes"
                );
                assert_eq!(
                    run_trap(&corpus_side, &engine_side),
                    TrapVerdict::Pass,
                    "and ({corpus_cause:?}, {cause:?}) is a plain Pass, never a WrongRule — \
                     neither side names a rule"
                );
            }
        }
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

    // ── The (verdict, rule) assertion the trap runner owns (story 4.7a) ───────

    /// [`Outcome::rule`] mirrors [`Expectation::rule`]: a decision names a rule, an abstention does
    /// not. The `Merged`/`Refused` arms are proven-to-red (collapse either to `None` and the
    /// `Some` assertions fail); the `Abstained → None` direction is by construction — an abstention
    /// carries no `RuleId`, so no mutation could make it `Some` and compile.
    #[test]
    fn an_outcome_names_its_rule_only_when_it_is_a_decision() {
        assert_eq!(merged().rule(), Some(&rule("l2-uplink-agrees")));
        assert_eq!(refused().rule(), Some(&rule("l2-different-switch")));
        assert_eq!(abstained().rule(), None);
    }

    /// The headline (AC1): a right verdict reached by the WRONG rule fails, naming both rules.
    /// `must_merge` expects `l1-exact-mac`; `merged()` fires `l2-uplink-agrees` — right column,
    /// wrong rule.
    #[test]
    fn a_right_verdict_by_the_wrong_rule_is_wrong_rule_naming_both() {
        assert_eq!(
            score(&must_merge(), &merged()),
            Score::Pass,
            "verdict is right"
        );
        assert_eq!(
            run_trap(&must_merge(), &merged()),
            TrapVerdict::WrongRule {
                expected: rule("l1-exact-mac"),
                actual: rule("l2-uplink-agrees"),
            }
        );
    }

    /// Both decision cells are pinned, not only `must-merge`: a `must-not-merge` refused by the
    /// wrong OPPOSING rule is also `WrongRule`. `must_not_merge` expects `l1-distinct-mac`;
    /// `refused()` cites `l2-different-switch`.
    #[test]
    fn a_wrong_opposing_rule_on_a_refusal_is_also_wrong_rule() {
        assert_eq!(
            score(&must_not_merge(), &refused()),
            Score::Pass,
            "a refusal is the right verdict for must-not-merge"
        );
        assert_eq!(
            run_trap(&must_not_merge(), &refused()),
            TrapVerdict::WrongRule {
                expected: rule("l1-distinct-mac"),
                actual: rule("l2-different-switch"),
            }
        );
    }

    /// The right verdict via the RIGHT rule PASSES (AC2) — the assertion tightens the gate, it does
    /// not reject every correct answer. Built with matching rules on both sides.
    #[test]
    fn a_right_verdict_by_the_right_rule_passes() {
        let expected = Expectation::MustMerge {
            rule: rule("l1-exact-mac"),
        };
        let actual = Outcome::Merged {
            rule: rule("l1-exact-mac"),
        };
        assert_eq!(run_trap(&expected, &actual), TrapVerdict::Pass);
    }

    /// AC3: a WRONG verdict is `VerdictFail`, and the rule is NOT consulted — even when the rule
    /// ALSO differs. `must_merge` answered by `refused()` (wrong verdict) whose rule differs too
    /// must read `VerdictFail`, never `WrongRule`. This is the case a "compare rules first" mutation
    /// gets wrong, so it is the prove-to-red for the ordering.
    #[test]
    fn a_wrong_verdict_is_verdict_fail_even_when_the_rule_also_differs() {
        assert_eq!(
            score(&must_merge(), &refused()),
            Score::Fail,
            "must-merge answered by a refusal is a wrong verdict"
        );
        assert_ne!(
            must_merge().rule(),
            refused().rule(),
            "and the rules differ too — so only the ORDER keeps this VerdictFail"
        );
        assert_eq!(
            run_trap(&must_merge(), &refused()),
            TrapVerdict::VerdictFail
        );
    }

    /// AC4, the load-bearing cell: `must-not-merge` answered by an `Abstained` passes `score` and
    /// has no rule to be wrong, so `run_trap` returns `Pass` — never `WrongRule`. An abstention on
    /// either side is never a rule mismatch.
    #[test]
    fn an_abstention_on_a_passing_cell_is_never_a_wrong_rule() {
        assert_eq!(
            score(&must_not_merge(), &abstained()),
            Score::Pass,
            "abstaining on must-not-merge is a pass (the anti-cowardice argument rests on it)"
        );
        assert_eq!(run_trap(&must_not_merge(), &abstained()), TrapVerdict::Pass);
        // And must-abstain answered correctly: no rule on either side, a plain pass.
        assert_eq!(run_trap(&must_abstain(), &abstained()), TrapVerdict::Pass);
    }

    // ── An answer, or a named refusal to ask (story 5.8) ─────────────────────

    /// [`UnanswerableCause::LevelNotImplemented`] is compared by the rule it CARRIES, not only by
    /// its discriminant — so two traps declined for different levels are two different records.
    ///
    /// _(This replaced a pair of assertions story 5.8's code review measured as unfailable: one
    /// destructured a value it had just constructed, the other `assert_ne!`d across variants of a
    /// derived `PartialEq`, where cross-variant equality is unrepresentable. What survives is the
    /// one comparison the derive does NOT make trivially true — the payload.)_
    #[test]
    fn two_unimplemented_levels_are_two_different_causes() {
        let uplink = UnanswerableCause::LevelNotImplemented {
            expected: rule("l2-uplink-agrees"),
        };
        let hostname = UnanswerableCause::LevelNotImplemented {
            expected: rule("l2-different-hostname"),
        };
        assert_ne!(
            uplink, hostname,
            "the rule the trap's AUTHOR named is part of the cause, not decoration — a bucket that \
             compared only the discriminant could not report WHICH level a trap waits on"
        );
        assert_eq!(
            uplink,
            UnanswerableCause::LevelNotImplemented {
                expected: rule("l2-uplink-agrees")
            }
        );
    }

    /// 🔴 **The pass that collapsing `Unanswerable` into `Outcome::Abstained` would manufacture is
    /// REAL** — measured here rather than asserted in prose, in story 5.7's idiom.
    ///
    /// This is the premise behind [`Answer`]'s refusal to convert. `example-must-abstain`'s
    /// expectation is `must-abstain`; record its unanswerable state as an abstention and the truth
    /// table returns [`Score::Pass`] — a trap passing *because nothing was asked*, in the column of
    /// a gate that never ran. The refusal itself is proven where it can be: nothing in this crate
    /// converts the two, and story 5.8's mutation M3 measures what happens downstream when
    /// something does.
    #[test]
    fn the_pass_a_declined_trap_would_manufacture_as_an_abstention_is_real() {
        assert_eq!(
            score(&must_abstain(), &abstained()),
            Score::Pass,
            "this is why `Answer::Unanswerable` must never become an `Outcome::Abstained`: the \
             must-abstain column would pass on a trap the engine was never asked about"
        );
        // And the same shortcut on a decision column would NOT pass — which is why the danger is
        // specific to `must-abstain` and why only that column can hide it.
        assert_eq!(score(&must_merge(), &abstained()), Score::Fail);
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

    // ── The engine's return, mapped onto the harness's record (story 5.7) ─────
    //
    // These tests live HERE, with `outcome_of`, because the claim they pin is `outcome_of`'s:
    // `Decision` and `Conclusion` are dependencies they merely read. That is `cascade.rs`'s stated
    // convention — *a test lives with the item whose CLAIM it pins*.

    use crate::identity::cascade::{RuleVerdict, RulesetVersion, Verdict};
    use crate::observation::ObsId;

    /// A decision with a NON-EMPTY verdict vector and a non-trivial ruleset version, so the two
    /// fields the mapping drops are present to be dropped. A vector that was empty on both sides
    /// would make the loss unobservable.
    fn decision_with(conclusion: Conclusion) -> Decision {
        Decision {
            conclusion,
            verdict_vector: vec![RuleVerdict {
                rule: rule("l1-exact-mac"),
                verdict: Verdict::Decisive,
                evidence: vec![ObsId::from_uuid(uuid::Uuid::nil())],
            }],
            ruleset_version: RulesetVersion(1),
        }
    }

    #[test]
    fn a_match_becomes_a_merge_naming_the_same_rule() {
        let decision = decision_with(Conclusion::Match {
            rule: rule("l1-exact-mac"),
        });
        assert_eq!(
            outcome_of(&decision),
            Outcome::Merged {
                rule: rule("l1-exact-mac")
            }
        );
    }

    #[test]
    fn a_no_match_becomes_a_refusal_naming_the_same_rule() {
        let decision = decision_with(Conclusion::NoMatch {
            rule: rule("l1-distinct-mac"),
        });
        assert_eq!(
            outcome_of(&decision),
            Outcome::Refused {
                rule: rule("l1-distinct-mac")
            }
        );
    }

    /// The abstaining row is the one that carries the SAME type on both sides
    /// ([`IdentityAbstentionCause`], since story 5.3), so the cause travels unchanged. Both
    /// variants are checked: a mapping that collapsed them onto one cause would still satisfy a
    /// single-variant test.
    #[test]
    fn an_abstention_keeps_its_cause_variant() {
        for cause in IdentityAbstentionCause::all() {
            let decision = decision_with(Conclusion::Abstained { cause });
            assert_eq!(
                outcome_of(&decision),
                Outcome::Abstained { cause },
                "the engine's cause must survive the mapping unchanged"
            );
        }
    }

    /// The mirror `run_trap` depends on: `outcome_of(&d).rule() == d.rule()` on EVERY row.
    ///
    /// `run_trap` compares `(expected.rule(), actual.rule())` and fires `WrongRule` only where both
    /// are `Some`. If the mapping lost or changed the rule, a trap answered by the engine would
    /// report a wrong-rule failure that the engine never committed — *"a red gate on a correct
    /// answer"*, which is the failure mode the register already names for the unnormalized string
    /// comparison.
    #[test]
    fn the_mapping_preserves_the_rule_mirror_on_every_row() {
        let rows = [
            Conclusion::Match {
                rule: rule("l1-exact-mac"),
            },
            Conclusion::NoMatch {
                rule: rule("l1-distinct-mac"),
            },
            Conclusion::Abstained {
                cause: IdentityAbstentionCause::AbsenceOfProof,
            },
        ];
        for conclusion in rows {
            let decision = decision_with(conclusion);
            assert_eq!(
                outcome_of(&decision).rule(),
                decision.rule(),
                "Decision::rule and Outcome::rule must agree after the mapping: {decision:?}"
            );
        }
    }

    /// What the mapping DROPS, asserted rather than left to the doc comment.
    ///
    /// `Outcome` has nowhere to put a verdict vector or a ruleset version, and the doc says so.
    /// This pins the consequence a reader has to be able to rely on: two decisions that differ ONLY
    /// in those two fields map to the SAME outcome. The day `ScoredRecord` carries them, this test
    /// is what says the loss was at the `Outcome` boundary and not further up.
    ///
    /// ⚠️ **It cannot red under any realistic mutation of [`outcome_of`], and that is recorded
    /// rather than smoothed over** — the same honesty the rule-mirror test above carries about M1.
    /// [`Outcome`] has no field able to hold either value, so every implementation that does not
    /// branch on `ruleset_version` satisfies the second assertion; it appears in none of the six
    /// mutation red sets. The only assertion here that can fail is `assert_ne!`, which guards
    /// [`Decision`]'s derived `PartialEq`, not the mapping. It is kept as a DOCUMENTATION test: what
    /// it states is a property of the destination type, and the day that type changes it is what
    /// says the loss used to be at this boundary.
    #[test]
    fn the_verdict_vector_and_the_ruleset_version_are_dropped() {
        let conclusion = Conclusion::Match {
            rule: rule("l1-exact-mac"),
        };
        let rich = decision_with(conclusion.clone());
        let bare = Decision {
            conclusion,
            verdict_vector: Vec::new(),
            ruleset_version: RulesetVersion(9999),
        };
        assert_ne!(rich, bare, "the two decisions really do differ");
        assert_eq!(
            outcome_of(&rich),
            outcome_of(&bare),
            "the two dropped fields leave no trace in the outcome"
        );
    }
}
