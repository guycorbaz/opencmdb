//! The truth labelling of the adversarial trap corpus (Story 4.2).
//!
//! Story 4.1 froze what a source *saw*. This freezes what the right answer *is*, and why.
//! Together they make a trap.
//!
//! **There is no external truth to appeal to.** D19 settles the oracle question by naming it:
//! *"the oracle is the fixture's author, made explicit and versioned, with a mandatory `reason`
//! field on every expectation."* Everything in this module follows from that one sentence — the
//! format's job is to force the author to be explicit, and to let a later reader disagree with
//! them on the record.
//!
//! Two things are therefore not negotiable. The FIRST is enforced by the types — a column that
//! carries the wrong payload does not parse. The SECOND is enforced by [`Trap::validate`], which
//! every reader must call: `reason` is an ordinary `String`, and no type can require that a
//! sentence say something. Saying otherwise would be the overclaim this module exists to refuse.
//!
//! 1. **A decision names its RULE, not merely its outcome.** *"A test that checks only the
//!    verdict goes green for the right answer reached by the wrong rule — and that engine will
//!    break on the next fixture"* (D19). D46b later made this the sole survivor of the deleted
//!    verdict join: compare `(verdict, rule)`, never `verdict` alone.
//! 2. **A reason is mandatory.** If it cannot be written in one sentence, the case is genuinely
//!    ambiguous and becomes `must-abstain` — *"THE INABILITY TO STATE A REASON IS THE ABSTENTION
//!    LABEL"* (D19). That is not a weakening; it is the honest answer, and it is what keeps a
//!    debatable trap from being argued over instead of classified.

use serde::{Deserialize, Serialize};

use crate::gap::AbstentionCause;
use crate::observation::ObsId;

/// The identifier of an engine rule, as an expectation names it.
///
/// A `String` for now because no rule exists yet — Epic 5 names them. It closes into an enum
/// when it does: architecture.md:2652 requires *a decision on every variant*, and an
/// `Other(String)` cannot satisfy an `expect_rule`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RuleId(pub String);

/// A trap's stable name, used in failure messages and in the reality-debt register.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TrapId(pub String);

/// The stable name that groups a trap FAMILY — a class of identity scenario (`randomized-mac`,
/// `multi-nic`, …), each committed in both decision forms (stories 4.9 onward).
///
/// A trap declares its family (or none). Two traps in the same family are the positive and
/// negative sides of one identity problem; [`incomplete_families`] uses this to check that a
/// family was tested BOTH ways, never only one. The newtype sibling of [`TrapId`] / [`RuleId`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FamilyId(pub String);

/// One of D18's three columns, carrying exactly what that column needs.
///
/// Modelled as an enum rather than a struct of optional fields, so "a merge that also names an
/// abstention cause" is **unrepresentable** instead of merely invalid. D18's table is the whole
/// gate: `must-not-merge` guards the false merge (the operator loses trust and uninstalls),
/// `must-merge` guards cowardice (*"an engine that abstains on everything scores false-merge = 0
/// and gets demolished by the middle column"*), and `must-abstain` guards guessing on the
/// honestly ambiguous case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Expectation {
    /// These observations describe one device: the engine must merge them, by this rule.
    MustMerge { rule: RuleId },
    /// These observations describe different devices: the engine must not merge them, and must
    /// say WHY by naming the rule that OPPOSES the merge (`l1-distinct-mac`, not the rule that
    /// was tempting). Decided 2026-07-21: a refusal without a named rule cannot be told apart
    /// from an engine that simply found nothing, and that is undebuggable in production. Every
    /// decision names a rule; only an abstention names a cause.
    MustNotMerge { rule: RuleId },
    /// The signal is genuinely insufficient: the engine must refuse to decide, for this cause.
    MustAbstain { cause: AbstentionCause },
}

impl Expectation {
    /// The column name as the gate counts it — the vocabulary of D18's table.
    pub fn column(&self) -> &'static str {
        match self {
            Expectation::MustMerge { .. } => "must-merge",
            Expectation::MustNotMerge { .. } => "must-not-merge",
            Expectation::MustAbstain { .. } => "must-abstain",
        }
    }

    /// The rule a decision must have fired, or `None` for an abstention.
    pub fn rule(&self) -> Option<&RuleId> {
        match self {
            Expectation::MustMerge { rule } | Expectation::MustNotMerge { rule } => Some(rule),
            Expectation::MustAbstain { .. } => None,
        }
    }
}

/// One trap: the observations it judges, the expected outcome, and the author's reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trap {
    pub id: TrapId,
    /// The replay stream this trap judges, corpus-relative (e.g. `scenario/replay/minimal.jsonl`).
    pub replay: String,
    /// The observations under judgement, by the stable `obs_id`s frozen in story 4.1 — never by
    /// line number, which a later edit would silently shift under the truth.
    pub observations: Vec<ObsId>,
    /// **Mandatory.** One sentence, from the author, on the record. See [`Trap::validate`].
    ///
    /// `default` is deliberate and is NOT permissiveness: without it an absent key fails inside
    /// serde, which names the field but cannot name the trap. Routing absence through
    /// `validate` is what lets the refusal say WHICH trap has no reason.
    #[serde(default)]
    pub reason: String,
    pub expect: Expectation,
    /// The family this trap belongs to, or `None` for a format/example trap that is part of no
    /// family and is exempt from the completeness check ([`incomplete_families`]).
    ///
    /// A family groups the positive and negative decision forms of ONE identity scenario; both must
    /// be present or the corpus is incomplete (story 4.7b). `#[serde(default)]` — the same idiom as
    /// `reason` — so an absent key is `None` (exempt), never a serde error, which keeps every
    /// family-less `.toml` valid and byte-unchanged under `deny_unknown_fields`.
    #[serde(default)]
    pub family: Option<FamilyId>,
}

/// A trap file is a list of traps. TOML renders this as `[[trap]]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrapFile {
    #[serde(default)]
    pub trap: Vec<Trap>,
}

/// Why a trap is not admissible. Domain data, not a string (D47).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrapError {
    /// A trap has no id, so nothing else about it can be reported intelligibly.
    IdMissing,
    /// The author left no reason, or only whitespace.
    ReasonMissing { trap: TrapId },
    /// The reason spans more than one line, or contains a control character an editor renders
    /// as one.
    ReasonNotOneSentence { trap: TrapId },
    /// The reason is too short to state anything (`"."` passes every structural check).
    ReasonTooShort { trap: TrapId, chars: usize },
    /// The reason is a paragraph written without a newline — including via TOML's `\`
    /// line-continuation, which strips the newline before it is ever seen.
    ReasonTooLong { trap: TrapId, chars: usize },
    /// A decision does not name the rule it expects to fire.
    RuleMissing { trap: TrapId },
    /// A trap does not say which replay stream it judges.
    ReplayMissing { trap: TrapId },
    /// A trap judges nothing.
    NoObservations { trap: TrapId },
    /// A trap judges the same observation twice — it would assert that an observation merges
    /// with itself, which no engine can fail.
    DuplicateObservation { trap: TrapId },
    /// Two traps in the same file share an id, so a failure could not say which one failed.
    DuplicateId { trap: TrapId },
    /// A trap declares a `family` key that is blank or whitespace — a family with no name cannot
    /// group anything. Absence is legal (`None`, exempt); a present-but-empty name is not.
    FamilyEmpty { trap: TrapId },
    /// A trap declares a `family` name that is not a clean token — it carries surrounding whitespace
    /// or a control character. A control character (e.g. a newline) would inject into the gate's
    /// one-line-per-family report; surrounding whitespace groups under a folded key but renders
    /// padded. A family name must be a single clean token, the way `reason` refuses control chars.
    FamilyMalformed { trap: TrapId },
    /// A trap file declares no trap at all.
    NoTraps,
}

/// A reason shorter than this states nothing; `"."` and `"n/a"` pass every structural check.
const REASON_MIN_CHARS: usize = 20;
/// Beyond this it is a paragraph, whatever the newlines say — and a case needing a paragraph is
/// the ambiguous one D19 sends to `must-abstain`.
const REASON_MAX_CHARS: usize = 300;

impl core::fmt::Display for TrapError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TrapError::IdMissing => f.write_str("a trap has no id"),
            TrapError::ReasonMissing { trap } => write!(
                f,
                "trap `{}`: no reason — the oracle IS the author's stated reason, so a trap \
                 without one asserts nothing (D19)",
                trap.0
            ),
            TrapError::ReasonNotOneSentence { trap } => write!(
                f,
                "trap `{}`: the reason spans more than one line — one sentence, or the case is \
                 ambiguous enough to belong in must-abstain (D19)",
                trap.0
            ),
            TrapError::ReasonTooShort { trap, chars } => write!(
                f,
                "trap `{}`: the reason is {chars} characters — too short to state why this is \
                 the right answer",
                trap.0
            ),
            TrapError::ReasonTooLong { trap, chars } => write!(
                f,
                "trap `{}`: the reason is {chars} characters — that is a paragraph, and a case \
                 needing a paragraph belongs in must-abstain (D19)",
                trap.0
            ),
            TrapError::RuleMissing { trap } => write!(
                f,
                "trap `{}`: a decision must name the rule it expects to fire — comparing the \
                 verdict alone goes green for the right answer reached by the wrong rule (D19)",
                trap.0
            ),
            TrapError::ReplayMissing { trap } => {
                write!(f, "trap `{}`: names no replay stream", trap.0)
            }
            TrapError::NoObservations { trap } => {
                write!(f, "trap `{}`: judges no observation", trap.0)
            }
            TrapError::DuplicateObservation { trap } => write!(
                f,
                "trap `{}`: judges the same observation twice — an observation merging with \
                 itself is a trap no engine can fail",
                trap.0
            ),
            TrapError::DuplicateId { trap } => write!(
                f,
                "trap `{}`: declared twice in this file — an id must name exactly one trap, or \
                 a failure cannot say which one failed",
                trap.0
            ),
            TrapError::FamilyEmpty { trap } => write!(
                f,
                "trap `{}`: declares an empty family — omit the key to be family-exempt, or name \
                 the family it belongs to",
                trap.0
            ),
            TrapError::FamilyMalformed { trap } => write!(
                f,
                "trap `{}`: the family name carries surrounding whitespace or a control character \
                 — a family name must be a single clean token",
                trap.0
            ),
            TrapError::NoTraps => f.write_str("trap file declares no trap"),
        }
    }
}

impl core::error::Error for TrapError {}

impl Trap {
    /// Admissibility of a single trap.
    ///
    /// "One sentence" is not checkable and this does not pretend to check it. What IS checkable:
    /// that a reason exists, that it is long enough to state something, that it has not grown
    /// into a paragraph — by length OR by line count OR by a control character, because TOML's
    /// `\` continuation and a lone `\r` both defeat a naive line count — and that a decision
    /// names its rule. Everything beyond that is the author's judgement, on the record, which is
    /// exactly what D19 says the oracle is.
    pub fn validate(&self) -> Result<(), TrapError> {
        // The id first: every other message quotes it, and `trap ``:` names nothing.
        if self.id.0.trim().is_empty() {
            return Err(TrapError::IdMissing);
        }
        let id = || self.id.clone();

        let reason = self.reason.trim();
        if reason.is_empty() {
            return Err(TrapError::ReasonMissing { trap: id() });
        }
        // `lines()` splits on `\n` only, so a lone `\r` or U+2028 would pass as one line while
        // an editor shows two. Any control character is therefore refused outright.
        if reason.lines().count() > 1 || reason.chars().any(|c| c.is_control()) {
            return Err(TrapError::ReasonNotOneSentence { trap: id() });
        }
        let chars = reason.chars().count();
        if chars < REASON_MIN_CHARS {
            return Err(TrapError::ReasonTooShort { trap: id(), chars });
        }
        if chars > REASON_MAX_CHARS {
            return Err(TrapError::ReasonTooLong { trap: id(), chars });
        }

        // A decision names the rule that fires (must-merge) or the rule that opposes
        // (must-not-merge). An abstention names a cause instead, and the type already says so.
        if let Some(rule) = self.expect.rule()
            && rule.0.trim().is_empty()
        {
            return Err(TrapError::RuleMissing { trap: id() });
        }

        if self.replay.trim().is_empty() {
            return Err(TrapError::ReplayMissing { trap: id() });
        }
        if self.observations.is_empty() {
            return Err(TrapError::NoObservations { trap: id() });
        }
        let distinct: std::collections::BTreeSet<_> = self.observations.iter().collect();
        if distinct.len() != self.observations.len() {
            return Err(TrapError::DuplicateObservation { trap: id() });
        }
        // A family is optional (None = exempt), but a PRESENT family must be a clean token: not
        // blank (it would group nothing), and free of surrounding whitespace or control characters
        // (a control char injects into the gate's one-line-per-family report; padding renders dirty).
        if let Some(family) = &self.family {
            if family.0.trim().is_empty() {
                return Err(TrapError::FamilyEmpty { trap: id() });
            }
            if family.0.as_str() != family.0.trim() || family.0.chars().any(|c| c.is_control()) {
                return Err(TrapError::FamilyMalformed { trap: id() });
            }
        }
        Ok(())
    }
}

impl TrapFile {
    /// Admissibility of every trap, plus the file-level rules: at least one trap, and ids that
    /// are distinct even after trimming and case-folding — `"T"` and `"t "` are indistinguishable
    /// in a failure message, which is the whole harm [`TrapError::DuplicateId`] exists to prevent.
    pub fn validate(&self) -> Result<(), TrapError> {
        if self.trap.is_empty() {
            return Err(TrapError::NoTraps);
        }
        let mut seen = std::collections::BTreeSet::new();
        for trap in &self.trap {
            trap.validate()?;
            if !seen.insert(trap.id.0.trim().to_lowercase()) {
                return Err(TrapError::DuplicateId {
                    trap: trap.id.clone(),
                });
            }
        }
        Ok(())
    }
}

/// One family that was tested in only one decision form — or in neither (story 4.7b).
///
/// A family is complete only when exercised BOTH ways: at least one `must-merge` AND one
/// `must-not-merge`. This record names which pole is present so a red gate is debuggable without
/// opening the corpus. Constructed ONLY for a one-sided (or pole-less) family — the invariant
/// `!(has_merge && has_not_merge)` holds by [`incomplete_families`]'s construction; a complete family
/// is never reported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncompleteFamily {
    /// The family's name in the author's ORIGINAL casing (the case-fold is only a grouping key).
    pub family: FamilyId,
    /// Whether a `must-merge` trap was seen in the family — the merge pole.
    pub has_merge: bool,
    /// Whether a `must-not-merge` trap was seen in the family — the not-merge pole.
    pub has_not_merge: bool,
}

/// The families tested in only one decision form — the corpus-completeness check (story 4.7b).
///
/// Groups the traps that DECLARE a family by their case-folded name — the same `trim().to_lowercase()`
/// normalization [`TrapFile::validate`] uses for [`TrapError::DuplicateId`], so a family split across
/// files or casings is seen as one. Per family it records whether a `must-merge` and a `must-not-merge`
/// were seen; a `must-abstain` is D18's orthogonal third column and counts for NEITHER pole (DR1). A
/// family missing either pole is returned as an [`IncompleteFamily`], sorted by the folded key for a
/// deterministic order. Traps with no family are skipped — they are format/example traps, exempt.
///
/// # Assumes validated input
///
/// Callers pass traps that already passed [`Trap::validate`], so no blank-after-trim family reaches
/// here; this function does not re-guard the empty-name case.
pub fn incomplete_families<'a>(traps: impl IntoIterator<Item = &'a Trap>) -> Vec<IncompleteFamily> {
    // folded key -> (first-seen original name, saw must-merge, saw must-not-merge).
    let mut families: std::collections::BTreeMap<String, (FamilyId, bool, bool)> =
        std::collections::BTreeMap::new();
    for trap in traps {
        let Some(family) = &trap.family else { continue };
        let entry = families
            .entry(family.0.trim().to_lowercase())
            .or_insert_with(|| (family.clone(), false, false));
        match trap.expect {
            Expectation::MustMerge { .. } => entry.1 = true,
            Expectation::MustNotMerge { .. } => entry.2 = true,
            Expectation::MustAbstain { .. } => {}
        }
    }
    families
        .into_values()
        .filter(|(_, has_merge, has_not_merge)| !(*has_merge && *has_not_merge))
        .map(|(family, has_merge, has_not_merge)| IncompleteFamily {
            family,
            has_merge,
            has_not_merge,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    const GOOD: &str = "the two observations carry the same locally-administered MAC";

    fn obs(n: u128) -> ObsId {
        ObsId::from_uuid(Uuid::from_u128(n))
    }

    fn trap(reason: &str) -> Trap {
        Trap {
            id: TrapId("t".into()),
            replay: "scenario/replay/minimal.jsonl".into(),
            observations: vec![obs(1)],
            reason: reason.into(),
            expect: Expectation::MustAbstain {
                cause: AbstentionCause::NoObservedValue,
            },
            family: None,
        }
    }

    #[test]
    fn a_reason_is_mandatory() {
        assert_eq!(
            trap("").validate(),
            Err(TrapError::ReasonMissing {
                trap: TrapId("t".into())
            })
        );
        assert_eq!(
            trap("   \n  ").validate(),
            Err(TrapError::ReasonMissing {
                trap: TrapId("t".into())
            })
        );
        assert!(trap(GOOD).validate().is_ok());
    }

    /// Every structural check passes on `"."` — which states nothing. A floor is the only
    /// checkable proxy left once "does this sentence mean something" is out of reach.
    #[test]
    fn a_reason_too_short_to_state_anything_is_refused() {
        assert!(matches!(
            trap(".").validate(),
            Err(TrapError::ReasonTooShort { .. })
        ));
        assert!(matches!(
            trap("n/a").validate(),
            Err(TrapError::ReasonTooShort { .. })
        ));
    }

    /// A paragraph is refused three ways, because it can arrive three ways: as real newlines,
    /// as sheer length (TOML's `\` continuation strips the newlines before serde sees them),
    /// and as a control character that `lines()` does not split on but an editor renders.
    #[test]
    fn a_paragraph_is_refused_however_it_arrives() {
        assert!(matches!(
            trap("first line of a reason that is long enough\nsecond line").validate(),
            Err(TrapError::ReasonNotOneSentence { .. })
        ));
        let collapsed = "x".repeat(REASON_MAX_CHARS + 1);
        assert!(matches!(
            trap(&collapsed).validate(),
            Err(TrapError::ReasonTooLong { .. })
        ));
        // A lone CR: `str::lines()` does not split on it, an editor does.
        assert!(matches!(
            trap("a reason long enough to pass\rand a second line").validate(),
            Err(TrapError::ReasonNotOneSentence { .. })
        ));
    }

    /// The premise of the whole format is `(verdict, rule)`. An unnamed rule guts it, and no
    /// type can require a non-empty `String`.
    #[test]
    fn a_decision_must_actually_name_its_rule() {
        let mut t = trap(GOOD);
        t.expect = Expectation::MustMerge {
            rule: RuleId("  ".into()),
        };
        assert!(matches!(t.validate(), Err(TrapError::RuleMissing { .. })));

        t.expect = Expectation::MustNotMerge {
            rule: RuleId("l1-distinct-mac".into()),
        };
        assert!(t.validate().is_ok());
    }

    #[test]
    fn a_trap_without_an_id_is_refused_before_anything_else() {
        let mut t = trap("");
        t.id = TrapId("  ".into());
        // Not ReasonMissing: with no id, no other message could name the offender.
        assert_eq!(t.validate(), Err(TrapError::IdMissing));
    }

    #[test]
    fn a_trap_must_judge_something_and_not_twice() {
        let mut t = trap(GOOD);
        t.observations.clear();
        assert!(matches!(
            t.validate(),
            Err(TrapError::NoObservations { .. })
        ));

        // An observation merging with itself is a trap no engine can fail.
        t.observations = vec![obs(1), obs(1)];
        assert!(matches!(
            t.validate(),
            Err(TrapError::DuplicateObservation { .. })
        ));
    }

    #[test]
    fn a_trap_must_name_a_replay_stream() {
        let mut t = trap(GOOD);
        t.replay = "  ".into();
        assert!(matches!(t.validate(), Err(TrapError::ReplayMissing { .. })));
    }

    #[test]
    fn a_file_must_declare_at_least_one_trap() {
        assert_eq!(
            TrapFile { trap: vec![] }.validate(),
            Err(TrapError::NoTraps)
        );
    }

    /// Ids differing only by case or surrounding whitespace are indistinguishable in a failure
    /// message — which is exactly the harm the rule exists to prevent.
    #[test]
    fn near_duplicate_ids_are_duplicates() {
        let mut a = trap(GOOD);
        let mut b = trap(GOOD);
        a.id = TrapId("T".into());
        b.id = TrapId("t ".into());
        b.observations = vec![obs(2)];
        assert!(matches!(
            TrapFile { trap: vec![a, b] }.validate(),
            Err(TrapError::DuplicateId { .. })
        ));
    }

    #[test]
    fn the_three_columns_carry_what_they_need_and_nothing_else() {
        let merge = Expectation::MustMerge {
            rule: RuleId("l1-exact-mac".into()),
        };
        assert_eq!(merge.column(), "must-merge");
        assert_eq!(merge.rule(), Some(&RuleId("l1-exact-mac".into())));

        // A refusal names the rule that OPPOSES the merge, not the one that was tempting.
        let split = Expectation::MustNotMerge {
            rule: RuleId("l1-distinct-mac".into()),
        };
        assert_eq!(split.column(), "must-not-merge");
        assert_eq!(split.rule(), Some(&RuleId("l1-distinct-mac".into())));

        let abstain = Expectation::MustAbstain {
            cause: AbstentionCause::ConflictingObservations,
        };
        assert_eq!(abstain.column(), "must-abstain");
        // An abstention has no rule to name — the type says so rather than a runtime check.
        assert_eq!(abstain.rule(), None);
    }

    #[test]
    fn an_unknown_field_is_refused() {
        let json = r#"{"id":"t","replay":"r","observations":[],"reason":"r","expect":{"must-abstain":{"cause":"NoObservedValue"}},"extra":1}"#;
        assert!(serde_json::from_str::<Trap>(json).is_err());

        let bad_variant = r#"{"must-merge":{"rule":"r","cause":"NoObservedValue"}}"#;
        assert!(
            serde_json::from_str::<Expectation>(bad_variant).is_err(),
            "a decision must not be able to carry an abstention cause"
        );
    }

    /// The committed corpus spells `AbstentionCause` in PascalCase. That representation is owned
    /// by another module, so a `rename_all` added there would silently invalidate every trap file
    /// and its recorded sha256. This test is the coupling, made visible.
    #[test]
    fn the_committed_abstention_vocabulary_is_pinned() {
        let json = serde_json::to_string(&AbstentionCause::NoObservedValue).unwrap();
        assert_eq!(
            json, r#""NoObservedValue""#,
            "the trap corpus commits this spelling — changing it rewrites every trap file"
        );
    }

    /// A semantic round-trip. It is NOT byte-for-byte: the committed trap file carries comments,
    /// and `toml::to_string` does not reproduce them. Claiming a byte round-trip here would be
    /// the kind of overclaim this module refuses elsewhere.
    #[test]
    fn a_trap_file_survives_a_serde_round_trip() {
        let file = TrapFile {
            trap: vec![trap(GOOD)],
        };
        let rendered = serde_json::to_string(&file).unwrap();
        let back: TrapFile = serde_json::from_str(&rendered).unwrap();
        assert_eq!(back, file);
    }

    // ── The family field and the completeness check (story 4.7b) ─────────────

    /// A valid trap with a chosen id, family and expectation — the builder for the family tests.
    fn fam(id: &str, family: Option<&str>, expect: Expectation) -> Trap {
        Trap {
            id: TrapId(id.into()),
            replay: "scenario/replay/minimal.jsonl".into(),
            observations: vec![obs(1)],
            reason: GOOD.into(),
            expect,
            family: family.map(|f| FamilyId(f.into())),
        }
    }
    fn merge() -> Expectation {
        Expectation::MustMerge {
            rule: RuleId("l1-exact-mac".into()),
        }
    }
    fn not_merge() -> Expectation {
        Expectation::MustNotMerge {
            rule: RuleId("l1-distinct-mac".into()),
        }
    }
    fn abstain() -> Expectation {
        Expectation::MustAbstain {
            cause: AbstentionCause::NoObservedValue,
        }
    }

    /// A family is optional: absent validates (exempt), present-and-named validates, present-but-blank
    /// is `FamilyEmpty`. Prove-to-red on the blank case — dropping the guard lets `"  "` through.
    #[test]
    fn a_present_but_blank_family_is_refused_while_absent_or_named_is_ok() {
        assert!(
            fam("t", None, abstain()).validate().is_ok(),
            "absent = exempt"
        );
        assert!(
            fam("t", Some("randomized-mac"), abstain())
                .validate()
                .is_ok(),
            "named = ok"
        );
        assert_eq!(
            fam("t", Some("  "), abstain()).validate(),
            Err(TrapError::FamilyEmpty {
                trap: TrapId("t".into())
            })
        );
    }

    /// A family name must be a clean token: surrounding whitespace and any control character are
    /// refused (a control char would inject into the gate's one-line-per-family report). A clean name
    /// with an internal space is still fine. Prove-to-red — dropping the guard lets `"multi\nnic"` and
    /// `" padded "` validate.
    #[test]
    fn a_family_name_with_whitespace_or_a_control_char_is_refused() {
        let malformed = TrapError::FamilyMalformed {
            trap: TrapId("t".into()),
        };
        assert_eq!(
            fam("t", Some("multi\nnic"), abstain()).validate(),
            Err(malformed.clone()),
            "a newline (control char) is refused"
        );
        assert_eq!(
            fam("t", Some(" padded "), abstain()).validate(),
            Err(malformed),
            "surrounding whitespace is refused"
        );
        assert!(
            fam("t", Some("multi nic"), abstain()).validate().is_ok(),
            "an internal space in an otherwise clean token is fine"
        );
    }

    /// A family present in BOTH decision forms is complete (AC2): nothing reported.
    #[test]
    fn a_two_sided_family_is_complete() {
        let traps = vec![
            fam("a", Some("randomized-mac"), merge()),
            fam("b", Some("randomized-mac"), not_merge()),
        ];
        assert!(incomplete_families(&traps).is_empty());
    }

    /// A one-sided family is reported, naming which pole is present (AC1). Both directions are pinned —
    /// merge-only misses not-merge, and the mirror not-merge-only misses merge.
    #[test]
    fn a_one_sided_family_is_reported_with_the_missing_pole() {
        assert_eq!(
            incomplete_families(&[fam("a", Some("randomized-mac"), merge())]),
            vec![IncompleteFamily {
                family: FamilyId("randomized-mac".into()),
                has_merge: true,
                has_not_merge: false,
            }]
        );
        assert_eq!(
            incomplete_families(&[fam("a", Some("multi-nic"), not_merge())]),
            vec![IncompleteFamily {
                family: FamilyId("multi-nic".into()),
                has_merge: false,
                has_not_merge: true,
            }]
        );
    }

    /// DR1, load-bearing: a `must-abstain` counts for NEITHER pole, so `{must-merge, must-abstain}` is
    /// still incomplete, missing `must-not-merge`. Prove-to-red — a mutation letting an abstention
    /// satisfy the not-merge pole turns this green.
    #[test]
    fn an_abstention_in_a_family_satisfies_no_pole() {
        let traps = vec![
            fam("a", Some("randomized-mac"), merge()),
            fam("b", Some("randomized-mac"), abstain()),
        ];
        assert_eq!(
            incomplete_families(&traps),
            vec![IncompleteFamily {
                family: FamilyId("randomized-mac".into()),
                has_merge: true,
                has_not_merge: false,
            }]
        );
    }

    /// DR1 consequence: an abstain-only family has NEITHER pole — reported with both false.
    #[test]
    fn an_abstain_only_family_has_neither_pole() {
        assert_eq!(
            incomplete_families(&[fam("a", Some("ambiguous-cases"), abstain())]),
            vec![IncompleteFamily {
                family: FamilyId("ambiguous-cases".into()),
                has_merge: false,
                has_not_merge: false,
            }]
        );
    }

    /// A family-less trap is skipped entirely (AC4, the exemption): a corpus of only `None`-family
    /// traps reports nothing, even when they are one-sided in aggregate.
    #[test]
    fn family_less_traps_are_exempt() {
        let traps = vec![fam("a", None, merge()), fam("b", None, merge())];
        assert!(
            incomplete_families(&traps).is_empty(),
            "no family declared -> nothing to check"
        );
    }

    /// Grouping is case-folded and cross-input, the same fold `DuplicateId` uses: `Randomized-MAC` and
    /// `randomized-mac` are ONE family, complete when one carries each pole. Prove-to-red — a raw-string
    /// group makes two one-sided families.
    #[test]
    fn family_grouping_is_case_folded() {
        let traps = vec![
            fam("a", Some("Randomized-MAC"), merge()),
            fam("b", Some("randomized-mac"), not_merge()),
        ];
        assert!(
            incomplete_families(&traps).is_empty(),
            "one family across two casings, both poles present"
        );
    }

    /// A one-sided family split across two casings is reported ONCE (folded), and the reported name
    /// keeps the FIRST-SEEN author casing — the fold is a grouping key, not the reported value. Here
    /// both sides are `must-merge`, so the family is one-sided; `Randomized-MAC` is seen first.
    #[test]
    fn a_one_sided_family_keeps_the_first_seen_original_casing() {
        let traps = vec![
            fam("a", Some("Randomized-MAC"), merge()),
            fam("b", Some("randomized-mac"), merge()),
        ];
        assert_eq!(
            incomplete_families(&traps),
            vec![IncompleteFamily {
                family: FamilyId("Randomized-MAC".into()),
                has_merge: true,
                has_not_merge: false,
            }],
            "one folded family, reported with the first-seen casing"
        );
    }
}
