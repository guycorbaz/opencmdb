//! The identity cascade — and, today, only the vocabulary its abstentions speak.
//!
//! D13 fixes the cascade's shape: **all rules are evaluated** (never first-match-wins), each yields
//! an enumerated verdict, and the verdicts combine by an **algebra, never a sum** — *"if the output
//! is a float, B has won in disguise"* [architecture.md:956-965]. **Story 5.4 writes that algebra
//! and the `Verdict` enum it combines; this file holds neither.**
//!
//! What it holds today is [`IdentityAbstentionCause`]: the two names the cascade may give for not
//! concluding. The vocabulary is chosen before the engine on purpose — the same order D19 imposed
//! on the metrics harness in Epic 4, *"a metric written after the engine is bent to fit the
//! engine"* — and because story 4.6a recorded the choice as Epic 5's to make.

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::Outcome;

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
}
