//! The identity cascade — and, today, the vocabulary it speaks and the type it returns.
//!
//! D13 fixes the cascade's shape: **all rules are evaluated** (never first-match-wins), each yields
//! an enumerated verdict, and the verdicts combine by an **algebra, never a sum** — *"if the output
//! is a float, B has won in disguise"* [architecture.md:956-965]. This file holds the vocabulary
//! that algebra reads and writes — [`Verdict`], [`RuleVerdict`], [`Conclusion`], [`Decision`],
//! [`RulesetVersion`] and [`IdentityAbstentionCause`]. **It does not hold the algebra: nothing here
//! combines a verdict set into a conclusion. That is story 5.4b's**, and no rule produces a
//! [`Verdict`] yet.
//!
//! The vocabulary is chosen before the engine on purpose — the same order D19 imposed on the metrics
//! harness in Epic 4, *"a metric written after the engine is bent to fit the engine"* — and because
//! story 4.6a recorded the abstention half of the choice as Epic 5's to make.
//!
//! # Four types say "verdict", and they are four different judgements
//!
//! | type | whose judgement | of what | variants |
//! |---|---|---|---|
//! | [`Verdict`] | ONE rule | one candidate pair | 5 |
//! | [`Conclusion`] | the CASCADE | a whole verdict set | 3 |
//! | [`crate::score::Outcome`] | the trap harness | what something ANSWERED about a trap | 3 |
//! | [`crate::score::TrapVerdict`] | the trap RUNNER | whether one trap passed | 3 |
//!
//! [`Conclusion`] and [`crate::score::Outcome`] look redundant and are not. `Outcome` is what a
//! harness writes down about *any* answer, including a hand-authored one in a test; `Conclusion` is
//! what the engine concluded, and it never travels alone — it travels inside a [`Decision`], with
//! its verdict vector and its ruleset version. **Nothing converts between them**, and the day they
//! meet is story 5.7's; see [`Decision`] for why no `From` impl exists.

use crate::observation::ObsId;
use crate::trap::RuleId;

/// What ONE rule says about ONE candidate pair — D13's enumerated verdict.
///
/// D13 writes this enum out by name: `enum Verdict { Decisive, Supports, Neutral, Opposes,
/// Disqualifying }` [architecture.md:964]. The five variants and their spelling are the
/// architecture's, not a paraphrase.
///
/// # This type is the vocabulary; combining is story 5.4b's
///
/// D13's contract is that **all rules are evaluated** and that *"verdicts combine by an **algebra,
/// not a sum**"* [architecture.md:960-961], because *"REFUSED: `rule -> confidence: f64` … if the
/// output is a float, B has won in disguise"* [architecture.md:956-958]. **Nothing in this file
/// combines anything** — the six-row table at [architecture.md:967-974] is implemented by story
/// 5.4b, and no rule produces a `Verdict` until story 5.5.
///
/// ⚠️ **The six rows do not cover every input.** Enumerated over the PRESENCE of each variant, the
/// table leaves exactly one class unanswered: at least one `Opposes`, with no `Decisive`, no
/// `Supports` and no `Disqualifying`. It is not *"only `Neutral` / nothing"* and it is not
/// *"`Supports` AND `Opposes`"*. **Story 5.4b arbitrates it**; this doc names the gap so a reader
/// does not mistake five variants and six rows for a complete specification.
///
/// # Why there is no ordering
///
/// D20 refuses evidence STRENGTH as a default and specifies the shape of its return if it ever
/// comes back: *"if strength returns, it returns as an ORDINAL, not a weight: `Opposes(Weak) |
/// Opposes(Strong)`. The enum grows, the table grows, it stays finite and exhaustively enumerable.
/// No float decides."* [architecture.md:1374-1376], under a four-condition ADR
/// [architecture.md:1378-1394]. Deriving `Ord` here would let two verdicts be COMPARED by magnitude
/// today, which is the move that ADR exists to gate.
// Deliberately absent, each with a reason: `Serialize`/`Deserialize` (nothing persists a verdict —
// story 5.9 if it persists one), `PartialOrd`/`Ord` (see the doc above — D20's ADR owns it),
// `Display` (story 5.14 renders through the `t!()` seam), and `#[non_exhaustive]` (never:
// `opencmdb-bin` is another crate, so it would force a `_` arm downstream and destroy the
// `error[E0004]` that makes a sixth variant break story 5.4b's table).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// This rule alone settles the pair — the strongest thing a rule may say.
    ///
    /// It does not settle the DECISION: D13's table pairs a `Decisive` with `Opposes` to produce
    /// `Ambiguous`, the cloned-MAC case [architecture.md:971]. A rule is decisive about its own
    /// evidence, never about the outcome.
    Decisive,
    /// This rule argues FOR the pair being one interface, without settling it.
    Supports,
    /// This rule has nothing to say about this pair.
    ///
    /// The honest answer of a rule that does not KNOW — D20 names the opposite as the common bug:
    /// *"the rule that wrongly `Opposes` should return `Neutral`: it does not KNOW, it BELIEVES it
    /// knows… nine parasitic abstentions out of ten are that"* [architecture.md:1383-1387].
    Neutral,
    /// This rule argues AGAINST the pair, without disqualifying it.
    Opposes,
    /// This rule forbids the pair outright — D13's *"absolute priority, short-circuits
    /// everything"* [architecture.md:969].
    ///
    /// Its two committed instances are STRUCTURAL facts read at ingestion, never scored: the
    /// IANA-reserved VRRP/HSRP MAC prefixes and the U/L bit — *"**Both are** `Disqualifying` as
    /// grouping anchors, known at ingestion."* [architecture.md:1002]. D13 is explicit that
    /// *"confusing an IANA fact with scoring turns a fact into a probability — and that is how
    /// weights get invented"* [architecture.md:996-998].
    Disqualifying,
}

impl Verdict {
    /// Every variant, so a caller can exercise the whole vocabulary without listing it.
    ///
    /// # What the witness below guarantees, and what it does not
    ///
    /// Adding a variant makes the `match` non-exhaustive — **`error[E0004]`, which stops the
    /// build** and forces a human decision at this exact site. That is the guarantee.
    ///
    /// **It does not mechanically force the new variant into the array, and that was MEASURED on
    /// this idiom's first instance ([`IdentityAbstentionCause::all`]), not assumed.** Repairing only
    /// the `error[E0004]` — adding a bare arm — leaves this function returning the old list while
    /// the whole suite stays green. Widening the literal *without* widening the return type is a
    /// separate `error[E0308]`; the two errors are **alternatives along one repair path, never
    /// simultaneous**. The array length is pinned at `5` in the signature so the second error exists
    /// at all.
    ///
    /// The residue is registered with an owner (story 5.14) rather than closed here: the two
    /// candidate closures were built and measured on the sibling enum and both were rejected.
    pub fn all() -> [Self; 5] {
        let all = [
            Self::Decisive,
            Self::Supports,
            Self::Neutral,
            Self::Opposes,
            Self::Disqualifying,
        ];
        // ⚠️ If you arrived here from an error[E0004] after adding a variant: adding a bare
        // `Self::NewThing => {}` arm silences this and is the WRONG repair — `all()` would then
        // return a list missing your variant, and every test that loops over it would skip the
        // variant in silence (measured on the sibling enum: the suite stays green). Add it to the
        // literal above and widen the return type. And read D20 first: a sixth verdict is the
        // "evidence strength" question, which needs an ADR before any code.
        for v in all {
            match v {
                Self::Decisive => {}
                Self::Supports => {}
                Self::Neutral => {}
                Self::Opposes => {}
                Self::Disqualifying => {}
            }
        }
        all
    }
}

/// One entry of the complete verdict vector: the `(rule, verdict, evidence)` triple.
///
/// D13: *"**explanation is free** (the list of `(rule, verdict, evidence)` IS the explanation)"*
/// [architecture.md:977-978]. D18 makes recording the whole list a requirement rather than a
/// courtesy: *"The harness records, for every case, the COMPLETE VERDICT VECTOR, not just the
/// outcome… **the anti-drift is not discipline, it is a data requirement.**"*
/// [architecture.md:1396-1399].
///
/// # Nothing produces one, and nothing validates one
///
/// There is no rule, so every `RuleVerdict` in the tree today is a test literal. The fields are
/// `pub` with no constructor — the [`crate::score::ScoredRecord`] precedent — and the two
/// validations a producer would want are **registered rather than invented**: that a verdict which
/// ARGUES leaves non-empty evidence, and that a [`Decision`]'s conclusion names a rule its own
/// vector contains. Both need story 5.4b or 5.5 to have something that could red.
///
/// # Its relationship to [`crate::score::VerdictVectorEntry`]
///
/// That type is the HARNESS-side placeholder for this same triple, and it is **uninhabited** so
/// that `ScoredRecord::verdict_vector` is provably empty until an engine fills it. This story does
/// not replace it: there is still no producer, and replacing it would falsify four places at once.
/// **Story 5.7 owns the unification**, when the trap runner first records a real run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleVerdict {
    /// The rule that spoke. A [`RuleId`] rather than an enum: five of the seven rule names the
    /// committed corpus writes are `l2-*`, which Epic 6 designs, so closing the enum here would
    /// enumerate rules nobody has specified.
    pub rule: RuleId,
    /// What it said.
    pub verdict: Verdict,
    /// The observations the rule read to say it, by the stable id stories 4.1/4.2 froze — never a
    /// line number, which *"would silently shift under the truth"*.
    ///
    /// This is the SMALLEST evidence that is not invented. The architecture requires a firing rule
    /// to leave evidence behind — *"a rule that fires without leaving its `rule_id` in the database
    /// is a rule we cannot debug in production"* [architecture.md:1309-1310] — and **shapes it
    /// nowhere**. A richer payload (the fact values, the candidate pair, a rendered sentence) has no
    /// producer until story 5.5 and would be a design taken without one.
    pub evidence: Vec<ObsId>,
}

/// The version of the rule set a [`Decision`] was produced under.
///
/// D14: *"**`ruleset_version` is mandatory:** without it, improving the engine is **a silent data
/// migration — the worst kind**."* [architecture.md:1044-1045]. D20 says what an increment means:
/// *"Any reintroduction increments `ruleset_version`; existing links are not recomputed (they carry
/// the version they were decided under)."* [architecture.md:1392-1393].
///
/// # No constant, no default, no ordering
///
/// **There is no `CURRENT_RULESET_VERSION` and no `Default`**, because there is no ruleset: no rule
/// exists. A constant would be a value asserting that the rules it versions are there. The version
/// arrives as a parameter when a producer builds a [`Decision`] — story 5.5 is the first story with
/// rules to version, and it owns the constant.
///
/// **No value is refused, including `RulesetVersion(0)`.** D14's "mandatory" is about PRESENCE, not
/// meaning; validating a number against nothing would be the same invention this story refuses for
/// evidence. Registered, owner story 5.5.
///
/// **No `PartialOrd`/`Ord`**: the first consumer that ORDERS two versions is persistence (D20's
/// "existing links are not recomputed" is a claim about which version a row carries, not a
/// comparison anything performs today). Story 5.9 adds it if it needs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RulesetVersion(
    /// The version number itself. Opaque: nothing derives meaning from its value yet.
    pub u32,
);

/// What the cascade concluded — D13's three-way decision.
///
/// D13: *"candidate generation (blocking) -> verdicts -> **three-way decision** (`match` /
/// `no-match` / **`ambiguous` -> abstain**)"* [architecture.md:931-932].
///
/// # A decision names a RULE; an abstention names a CAUSE
///
/// This mirrors [`crate::score::Outcome`]'s shape deliberately, so that [`Decision::rule`] can
/// return `None` for an abstention **by construction** — the property
/// [`crate::score::run_trap`] already relies on to leave an abstention out of its `(verdict, rule)`
/// assertion without a runtime guard. Story 4.2 fixed the rule half in the truth format:
/// *"Every decision names a rule; only an abstention names a cause"* (`trap.rs`), because *"a
/// refusal without a named rule cannot be told apart from an engine that simply found nothing, and
/// that is undebuggable in production"*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Conclusion {
    /// These observations describe one interface, and this rule settled it — D13's row *"a
    /// `Decisive`, no `Opposes`"* [architecture.md:970].
    Match {
        /// The rule that settled it.
        rule: RuleId,
    },
    /// These observations describe different interfaces, and this rule FORBADE the pair — D13's row
    /// *"any `Disqualifying`"* [architecture.md:969].
    ///
    /// ⚠️ **D13's `NoMatch` covers two situations and only ONE of them lands here.** The other —
    /// *"only `Neutral` / nothing → `NoMatch` (absence of proof)"* [architecture.md:974] — has **no
    /// rule to name**, so it cannot be represented by this variant and becomes
    /// [`Self::Abstained`] with [`IdentityAbstentionCause::AbsenceOfProof`]. That fork is why this
    /// variant carries a rule and cannot be given an optional one: an engine that mapped all of
    /// `NoMatch` onto a refusal *"would fail every honest `must-abstain` trap"*, the exact case D18
    /// says must NOT be gated. **This story builds the fork; story 5.4b decides which side an input
    /// falls on.**
    NoMatch {
        /// The rule that opposed the pair — never the rule that was merely tempting.
        rule: RuleId,
    },
    /// The cascade took no decision, for this cause.
    Abstained {
        /// Why it did not conclude.
        cause: IdentityAbstentionCause,
    },
}

/// What the identity cascade returns: the conclusion, the explanation, and the ruleset version.
///
/// A struct rather than an enum, and the reason is D14. `ruleset_version` is carried **once**, so
/// "mandatory" rests on one declaration instead of on three variants agreeing — and a variant added
/// to [`Conclusion`] later cannot forget it. [`crate::score::Outcome`] is an enum because it has no
/// common field to carry; the two types differ in envelope and agree in algebra, which is exactly
/// what [`Self::rule`] pins.
///
/// # Nothing produces one
///
/// There is no rule, no blocker and no join, so every `Decision` in the tree is a test literal.
/// **No `From<Decision> for Outcome` exists in either direction**: mapping the engine's return onto
/// the harness's record is a decision about the release gate, and it belongs to story 5.7 with a
/// story behind it — not to a silent conversion. The same refusal, for the same reason, kept the
/// two abstention vocabularies unbridged in story 5.3.
///
/// # No float, anywhere
///
/// D13's universal rule: *"floats may RANK, never DECIDE… The moment a float decides,
/// explainability and the truth table die the same day"* [architecture.md:988-990]. No field here
/// is a float and none is a magnitude: [`RulesetVersion`] is an identifier, [`Verdict`] is
/// enumerated and unordered. There is no ranking value at all, because L1 is a deterministic lookup
/// with nothing to rank; D13's milli-units corollary [architecture.md:991-993] binds the day one
/// appears. Story 5.4b adds the `cargo xtask ci` gate that holds this mechanically.
///
/// # What is representable and not refused
///
/// The fields are `pub` with no constructor ([`crate::score::ScoredRecord`]'s precedent), so a
/// `Decision` whose [`Self::conclusion`] names a rule **absent from its own
/// [`Self::verdict_vector`]** compiles, and so does a [`Conclusion::Match`] with an empty vector —
/// *"merged, with no explanation"*, which is what D13's *"the list … IS the explanation"* exists to
/// prevent. Nothing refuses either, and the reason is a missing producer rather than a preference:
/// story 5.4b's combining function is the first place a conclusion and a vector are built together,
/// hence the first place a test could red. Registered, owner story 5.4b.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    /// What the cascade concluded.
    pub conclusion: Conclusion,
    /// The complete verdict vector — D18's word, and D18's data requirement. Every rule that spoke,
    /// what it said, and what it read.
    pub verdict_vector: Vec<RuleVerdict>,
    /// The version of the ruleset that produced this decision (D14).
    pub ruleset_version: RulesetVersion,
}

impl Decision {
    /// The rule a DECISION named, or `None` for an abstention — the mirror of
    /// [`crate::score::Outcome::rule`].
    ///
    /// A match names the rule that settled the pair; a refusal names the rule that FORBADE it. An
    /// abstention took no decision, so it carries a cause and no rule — **the type says so**, which
    /// is what lets a future consumer leave an abstention out of a `(verdict, rule)` comparison by
    /// construction rather than by a runtime guard.
    ///
    /// It lives on `Decision` and not on [`Conclusion`], and there is no `Conclusion::rule()`: the
    /// consumer that needs it holds a decision, so an accessor on the inner enum would have no
    /// caller. Same no-consumer argument that keeps serde off these types.
    pub fn rule(&self) -> Option<&RuleId> {
        match &self.conclusion {
            Conclusion::Match { rule } | Conclusion::NoMatch { rule } => Some(rule),
            Conclusion::Abstained { .. } => None,
        }
    }
}

/// Why the identity cascade did not conclude — the engine's own abstention vocabulary.
///
/// # Two variants, and the two D13 rows that produce none
///
/// D13's algebra is a table of six conditions over the verdict set [architecture.md:967-974]. Four
/// of those rows produce an abstention and are covered by the two variants below — three by
/// [`Self::Ambiguous`], one by [`Self::AbsenceOfProof`]. The remaining **two** are accounted for
/// here, so a reader does not find an uncovered row and add a variant for it:
///
/// - `any Disqualifying → NoMatch` [architecture.md:969] is an **active opposition**: a rule
///   opposes, so the answer names that rule and is a [`crate::score::Outcome::Refused`], never an
///   abstention.
/// - `a Decisive, no Opposes → Match` [architecture.md:970] is a [`crate::score::Outcome::Merged`].
///
/// `NoMatch` is therefore reached by two different rows — the `Disqualifying` one just named, which
/// is a decision, and the absence-of-proof one below — and **exactly one of them has no rule to
/// name**: that row is [`Self::AbsenceOfProof`]. Six rows, four of them abstentions, two variants.
///
/// A third variant is a **finding**, not a tidy-up. In particular this enum does not reproduce
/// `OutOfPerimeter`: the cascade's table has no such row, and D16 names the failure mode of
/// carrying causes across — *"if `Ambiguous` means both 'real conflict' and 'unmodelled case', it
/// means nothing, and the operator learns to ignore it"* [architecture.md:1112-1115].
///
/// # The two sides of a trap now speak different vocabularies, and that is safe by construction
///
/// [`crate::gap::AbstentionCause`] is the RECONCILIATION vocabulary (`OutOfPerimeter`,
/// `NoObservedValue`, `ConflictingObservations`). It says why comparing a *declared field* against
/// observations did not conclude, and in the trap corpus it lives on the **expectation** side —
/// [`crate::trap::Expectation::MustAbstain`] carries it, and three committed trap files write it.
/// This enum lives on the **outcome** side, in
/// [`crate::score::Outcome::Abstained`], and says why *the cascade* did not conclude: a question
/// about a verdict set, not about a field.
///
/// The two are never compared, and the reason is a mechanism rather than a promise:
/// [`crate::score::score`]'s 3×3 matches `Outcome::Abstained { .. }` with `..` and cannot read the
/// payload at all, and [`crate::score::run_trap`] compares rules only where BOTH sides return
/// `Some`, which an abstention never does — [`crate::score::Outcome::rule`] returns `None` for one
/// **by type**. There is no comparison here to go asymmetric. A test says so
/// (`the_two_abstention_vocabularies_are_never_compared`, in `score`'s test module).
///
/// If some future story wants to map one onto the other, it needs a decision and a story — not a
/// silent `impl From`, which is why none exists.
// Deliberately absent, each with an owner: `Serialize`/`Deserialize` (nothing persists a cause yet
// — story 5.9 if it persists one), `PartialOrd`/`Ord` (nothing orders or keys one — story 5.14 if
// it groups by cause), `Display` (story 5.14 renders through the `t!()` seam, not through
// `Display`), and `#[non_exhaustive]` (never: `opencmdb-bin` is another crate, so it would force a
// `_` arm on every downstream match and destroy the `error[E0004]` that makes a new variant break
// its consumers).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityAbstentionCause {
    /// The verdict set points both ways, or too weakly to point at all.
    ///
    /// Three of D13's rows produce it: *"a `Decisive`, >=1 `Opposes`"* — **the cloned-MAC case**
    /// [architecture.md:971] — *"no `Decisive`, >=1 `Supports`, no `Opposes`"* (weak evidence,
    /// `:972`), and *"`Supports` AND `Opposes`"* (conflict, `:973`).
    ///
    /// Whether those three should later become three variants is **open, with an owner**: story
    /// 5.14 owns the operator-facing grouping and is the first place a split can be justified by a
    /// consumer rather than by symmetry. Registered in `deferred-work.md`.
    Ambiguous,
    /// Nothing in the verdict set argues either way — the row *"only `Neutral` / nothing →
    /// `NoMatch` (absence of proof)"* [architecture.md:974].
    ///
    /// It is a separate variant because it is the half of `NoMatch` with no rule to name, and D18
    /// is explicit that this case must NOT be gated: *"an engine that abstains because there is NOT
    /// ENOUGH SIGNAL is being honest… We do not gate that"*. Mapping it to a refusal instead would
    /// fail every honest `must-abstain` trap.
    AbsenceOfProof,
}

impl IdentityAbstentionCause {
    /// Every variant, so a caller can exercise the whole vocabulary without listing it.
    ///
    /// # What the witness below guarantees, and what it does not
    ///
    /// Adding a variant makes the `match` non-exhaustive — **`error[E0004]`, which stops the
    /// build** and forces a human decision at this exact site. That is the guarantee, and it is the
    /// one that matters: given that a third variant is a finding rather than a routine addition
    /// (see this type's own doc), an error that refuses to compile IS the mechanism.
    ///
    /// **It does not mechanically force the new variant into the array, and that was measured, not
    /// assumed.** Repairing only the `error[E0004]` — adding a bare arm — leaves this function
    /// returning the old list while the whole suite stays green. Widening the literal *without*
    /// widening the return type is a separate `error[E0308]`; the two errors are alternatives along
    /// one repair path, never simultaneous. The array length is still pinned at `2` in the
    /// signature so the second error exists at all.
    ///
    /// Whether to close that path with a single-source construction is registered with an owner
    /// (story 5.14) rather than decided here: the alternatives measured for it either make adding a
    /// variant *frictionless*, which is the opposite of what this vocabulary wants, or cost a
    /// dependency in the domain crate.
    pub fn all() -> [Self; 2] {
        let all = [Self::Ambiguous, Self::AbsenceOfProof];
        // ⚠️ If you arrived here from an error[E0004] after adding a variant: adding a bare
        // `Self::NewThing => {}` arm silences this and is the WRONG repair — `all()` would then
        // return a list missing your variant, and every test that loops over it would skip the
        // variant in silence (measured: the suite stays green). Add it to the literal above and
        // widen the return type. Better still: re-read this type's doc first — a third variant is
        // a finding, not a routine addition.
        for c in all {
            match c {
                Self::Ambiguous => {}
                Self::AbsenceOfProof => {}
            }
        }
        all
    }
}

/// Tests for this file's types.
///
/// # Where a test lives, decided once
///
/// **A test lives with the item whose CLAIM it pins; the items it merely READS are dependencies,
/// imported and not owned.** Story 5.3's review left this convention to "whoever next adds a test
/// to `cascade.rs`", and these are those tests. It decides two cases:
///
/// - `a_decision_names_a_rule_and_an_abstention_does_not` and
///   `the_conclusion_mirrors_the_outcomes_rule_shape` pin claims about [`Decision`]'s shape.
///   [`crate::score::Outcome`] is the dependency the second one reads to express the mirror, not
///   its subject. Both belong here.
/// - `an_abstention_names_no_rule_whatever_its_cause` (story 5.3) pins a claim about the abstention
///   VOCABULARY — *"an abstention has no rule to name, for EVERY cause"* — and reads
///   `Outcome::rule()` as the mechanism that expresses it. Its subject is
///   [`IdentityAbstentionCause`], so it stays here too, by the convention rather than by fiat.
///
/// The truth-table tests of `score()` stay in `score.rs` under the same rule: their subject is
/// `score`, and this module's types are what they read.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::Outcome;
    use uuid::Uuid;

    fn obs(n: u128) -> ObsId {
        ObsId::from_uuid(Uuid::from_u128(n))
    }

    fn rule(name: &str) -> RuleId {
        RuleId(name.into())
    }

    /// The variant SET, not its length: on `[Self; 2]` the length is a tautology the return type
    /// already guarantees, so asserting it would prove nothing. What this pins is *which* two —
    /// renaming a variant, or swapping one for another, reds here.
    ///
    /// ⚠️ **What it does NOT catch, stated because the name reads stronger than the body:** a
    /// third variant added to the enum leaves `Ambiguous` and `AbsenceOfProof` in `all()`, so this
    /// test stays green. What stops that is [`IdentityAbstentionCause::all`]'s `error[E0004]`, and
    /// its doc says exactly how far that goes. "Exactly" in this name is carried by the `[Self; 2]`
    /// return type, not by the two assertions below.
    #[test]
    fn the_vocabulary_is_exactly_ambiguous_and_absence_of_proof() {
        let all = IdentityAbstentionCause::all();
        assert!(
            all.contains(&IdentityAbstentionCause::Ambiguous),
            "Ambiguous is D13's three-row abstention and must be in the witness"
        );
        assert!(
            all.contains(&IdentityAbstentionCause::AbsenceOfProof),
            "AbsenceOfProof is D13's no-rule-to-name row and must be in the witness"
        );
    }

    /// An abstention has no rule to name — for EVERY cause, by construction rather than by
    /// convention. This is what lets `run_trap` leave an abstention out of the `(verdict, rule)`
    /// assertion, and it must not depend on which cause was chosen.
    #[test]
    fn an_abstention_names_no_rule_whatever_its_cause() {
        for cause in IdentityAbstentionCause::all() {
            assert_eq!(
                Outcome::Abstained { cause }.rule(),
                None,
                "an abstention with cause {cause:?} must name no rule"
            );
        }
    }

    /// D13 names five verdicts [architecture.md:964] and this pins that set — renaming one, or
    /// swapping one for another, reds here.
    ///
    /// ⚠️ **What it does NOT catch, stated because the name reads stronger than the body:** a SIXTH
    /// variant added to the enum leaves the five in `all()`, so this test stays green. What stops
    /// that is [`Verdict::all`]'s `error[E0004]`, and its doc says exactly how far that goes. The
    /// length is carried by the `[Self; 5]` return type, not by the assertions below, so asserting
    /// it here would prove nothing.
    #[test]
    fn the_five_verdicts_of_d13_are_present() {
        let all = Verdict::all();
        for expected in [
            Verdict::Decisive,
            Verdict::Supports,
            Verdict::Neutral,
            Verdict::Opposes,
            Verdict::Disqualifying,
        ] {
            assert!(
                all.contains(&expected),
                "{expected:?} is one of D13's five verdicts and must be in the witness"
            );
        }
    }

    /// A decision names a rule; an abstention names a cause and no rule — for EVERY cause, not one
    /// hand-picked variant. This is the property that lets a consumer skip an abstention in a
    /// `(verdict, rule)` comparison by construction.
    #[test]
    fn a_decision_names_a_rule_and_an_abstention_does_not() {
        let matched = Decision {
            conclusion: Conclusion::Match {
                rule: rule("l1-exact-mac"),
            },
            verdict_vector: Vec::new(),
            ruleset_version: RulesetVersion(1),
        };
        assert_eq!(
            matched.rule(),
            Some(&rule("l1-exact-mac")),
            "a Match names the rule that settled the pair"
        );

        let refused = Decision {
            conclusion: Conclusion::NoMatch {
                rule: rule("l1-distinct-mac"),
            },
            verdict_vector: Vec::new(),
            ruleset_version: RulesetVersion(1),
        };
        assert_eq!(
            refused.rule(),
            Some(&rule("l1-distinct-mac")),
            "a NoMatch names the rule that forbade the pair"
        );

        for cause in IdentityAbstentionCause::all() {
            let abstained = Decision {
                conclusion: Conclusion::Abstained { cause },
                verdict_vector: Vec::new(),
                ruleset_version: RulesetVersion(1),
            };
            assert_eq!(
                abstained.rule(),
                None,
                "an abstention with cause {cause:?} took no decision, so it names no rule"
            );
        }
    }

    /// The claim the epic makes about `Decision` — *"the same shape `Outcome` mirrors, so
    /// `run_trap`'s existing assertion needs no runtime guard"* — made executable.
    ///
    /// Conclusion by counterpart outcome: a rule is named on exactly the same two of three. The
    /// ENVELOPES differ deliberately (`Decision` is a struct carrying the ruleset version,
    /// `Outcome` an enum), so what is asserted is the algebra, not the layout.
    #[test]
    fn the_conclusion_mirrors_the_outcomes_rule_shape() {
        let r = rule("l1-exact-mac");
        let pairs: [(&str, Conclusion, Outcome); 3] = [
            (
                "match / merged",
                Conclusion::Match { rule: r.clone() },
                Outcome::Merged { rule: r.clone() },
            ),
            (
                "no-match / refused",
                Conclusion::NoMatch { rule: r.clone() },
                Outcome::Refused { rule: r.clone() },
            ),
            (
                "abstained / abstained",
                Conclusion::Abstained {
                    cause: IdentityAbstentionCause::Ambiguous,
                },
                Outcome::Abstained {
                    cause: IdentityAbstentionCause::Ambiguous,
                },
            ),
        ];

        for (label, conclusion, outcome) in pairs {
            let decision = Decision {
                conclusion,
                verdict_vector: Vec::new(),
                ruleset_version: RulesetVersion(1),
            };
            assert_eq!(
                decision.rule().is_some(),
                outcome.rule().is_some(),
                "{label}: Decision and Outcome must agree on whether a rule is named"
            );
        }
    }

    /// The `(rule, verdict, evidence)` triple survives the trip in full — D13's *"the list of
    /// `(rule, verdict, evidence)` IS the explanation"*. Three entries, three different verdicts,
    /// three distinct non-empty evidence lists, order preserved.
    #[test]
    fn the_verdict_vector_carries_the_whole_triple_in_order() {
        let vector = vec![
            RuleVerdict {
                rule: rule("l1-exact-mac"),
                verdict: Verdict::Decisive,
                evidence: vec![obs(1), obs(2)],
            },
            RuleVerdict {
                rule: rule("l2-hostname-agrees"),
                verdict: Verdict::Supports,
                evidence: vec![obs(3)],
            },
            RuleVerdict {
                rule: rule("l2-different-switch"),
                verdict: Verdict::Opposes,
                evidence: vec![obs(4), obs(5)],
            },
        ];
        let decision = Decision {
            conclusion: Conclusion::Abstained {
                cause: IdentityAbstentionCause::Ambiguous,
            },
            verdict_vector: vector.clone(),
            ruleset_version: RulesetVersion(7),
        };

        assert_eq!(
            decision.verdict_vector, vector,
            "the verdict vector is carried verbatim, in order, with every field intact"
        );
        assert_eq!(
            decision.verdict_vector[1].verdict,
            Verdict::Supports,
            "each entry keeps its own verdict"
        );
        assert_eq!(
            decision.verdict_vector[2].evidence,
            vec![obs(4), obs(5)],
            "each entry keeps its own evidence, and the three lists are distinct"
        );
        assert_eq!(
            decision.ruleset_version,
            RulesetVersion(7),
            "the ruleset version is carried alongside, not derived"
        );
    }
}
