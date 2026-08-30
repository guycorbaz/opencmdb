//! L2 — device grouping, and the rules that argue about it.
//!
//! L1 asks whether two observations are the same INTERFACE; L2 asks whether two interfaces are the
//! same DEVICE [architecture.md:891]. The trap matrix splits on exactly that line: *"multi-NIC
//! false-split = L1 correct, **L2 failed to group**"* [architecture.md:893].
//!
//! # What lives here, and what deliberately does not
//!
//! The RULES. The candidate generator stays in [`crate::identity::blocking`] — *a blocker that
//! consults a rule is that rule's echo* — and keeping the two in different files is what makes that
//! separation visible while reading. ⚠️ **It is visibility, not constraint**: story 6.6 measured
//! that adding a [`crate::identity::cascade::decide`] call inside the blocker leaves the whole
//! suite, clippy and every gate green. Read the separation as a convention, never as a guarantee.
//!
//! # 🔴 A rule needs FACTS, and an `L2CandidatePair` carries none
//!
//! [`crate::identity::blocking::L2CandidatePair`] holds two [`crate::identity::l1::L1Key`]s, and an
//! `L1Key` is `(L2DomainId, MacAddr)` — no hostname, no uplink, no [`Fact`] at all. That is
//! deliberate: it is what makes a narrowing key inexpressible inside the blocker. So a rule cannot
//! take a candidate pair; it takes an [`L2Side`] per side, which is the interface WITH the
//! observations that landed on it.
//!
//! ⚠️ **Resolving `ObsId`s back to observations is the CALLER's job, not this module's.**
//! [`crate::identity::l1::join`] returns `BTreeMap<L1Key, BTreeSet<ObsId>>`; mapping those ids back
//! against the original slice belongs to whoever holds that slice. Story 6.12 is the first such
//! caller; this module answers with what it is handed.
//!
//! # 🔴 A `decide` at this level must receive L2 verdicts ONLY
//!
//! Combining L1's verdict for the same pair with an L2 rule's is the obvious gesture and it
//! **erases this level**. Measured on the committed corpus: for any valid L2 pair the two `L1Key`s
//! differ by construction — equal keys are ONE interface and no pair — so
//! `l1::verdict_for_pair` on those observations always returns `Disqualifying` via
//! `l1-distinct-mac`, and `decide` gives `Disqualifying` absolute priority. The result is
//! `NoMatch { rule: "l1-distinct-mac" }`, in which no L2 rule can ever be named.
//!
//! `l1.rs` says the two organs do not consult each other; **nothing had ever said it about
//! `decide`'s ARGUMENT**, and this paragraph is that sentence. The invariant is registered against
//! story 6.12, which should consider closing it in a TYPE rather than in prose.
//!
//! # ⚠️ How a wrong rule id is caught here, and how it is NOT
//!
//! [`crate::trap::Expectation`] names a rule and the gate compares it — but **not for a verdict set
//! that only OPPOSES**. D13's ratified arbitration (GitHub issue #54) makes `>= 1 Opposes` with no
//! `Disqualifying` abstain on `AbsenceOfProof`, and an abstention carries **no rule**, so the
//! gate's rule comparison never fires. Measured end to end: corrupting this module's rule id leaves
//! the trap PASSING.
//!
//! The id is therefore pinned by a DOUBLE-LITERAL test — the constant against the corpus's own
//! spelling — which is L1's idiom and the only carrier that reds on a typo.

use std::collections::BTreeSet;

use crate::identity::cascade::{RuleVerdict, Verdict};
use crate::observation::{Fact, Observation};
use crate::trap::RuleId;

/// The rule id, spelled exactly as the committed corpus spells it.
///
/// Three committed traps name it. ⚠️ Only **two** of them can be answered at this level: the third,
/// `cloned-mac-must-not-merge`, carries the same MAC on both observations, so the L1 join collapses
/// them onto ONE interface and no pair exists to judge. Guy's arbitration of 2026-08-30 accepts
/// that; the structural reading it would need is registered against story 6.11.
pub const L2_DIFFERENT_HOSTNAME: &str = "l2-different-hostname";

/// One side of an L2 candidate pair — the observations that landed on ONE interface.
///
/// # Why a type rather than a bare slice
///
/// The rules below take two of these, and *which* side is which carries no meaning: naming the
/// shape once gives the invariant a home and stops a caller from passing a slice of something else.
/// It holds no key: no rule here reads one, and a field nothing reads is an invention (D45).
#[derive(Debug, Clone)]
pub struct L2Side<'a> {
    /// The observations that landed on this interface. May be empty — see [`hostnames_of`].
    pub observations: Vec<&'a Observation>,
}

impl<'a> L2Side<'a> {
    /// Build a side from the observations that landed on one interface.
    pub fn new(observations: Vec<&'a Observation>) -> Self {
        Self { observations }
    }
}

/// The hostnames this side actually offers, normalised — an empty set meaning *no name observed*.
///
/// # Absence and emptiness are ONE case, and the corpus says so
///
/// `fixtures/scenario/traps/hostname-absence.toml`'s header states the equivalence this implements:
/// *"MISSING and EMPTY are both the absence of a signal: an empty string is not a matchable value
/// (`"" == ""` is not hostname agreement), a byte-present empty name counts as NO observed value,
/// and a name that stops resolving opposes nothing."* A null hostname is unrepresentable — `Fact`'s
/// `name` is a `String`, not an `Option<String>` — so the format cannot even pose the non-case.
///
/// # ⚠️ Case is folded, in ASCII, and the decision is the point
///
/// Compared case-SENSITIVELY, `NAS-01` and `nas-01` argue that one machine is two devices — D20's
/// named bug produced by doing nothing in particular. D10 puts this comparison in Rust rather than
/// in SQL (*"comparison never descends into SQL"*), so the rule owns it.
///
/// **ASCII lowercasing, not [`str::to_lowercase`]**: full Unicode case folding has traps of its own
/// (the Turkish dotless ı, final sigma) and a DNS label is ASCII. ⚠️ **The limit is stated rather
/// than implied** — this is right for hostnames and would be wrong for arbitrary text. **No
/// committed trap exercises case in either direction**, so the behaviour ships with a synthetic
/// guard and this paragraph.
///
/// # ⚠️ Whitespace is trimmed, and that is a decision too
///
/// `"  "` counts as absent. Trimming is not obvious and not free: `"\u{200B}".trim().is_empty()` is
/// **`false`** in Rust — a measured fact this repository has met before — so a trim is a
/// convenience, never a proof of emptiness.
///
/// # ⚠️ [`crate::observation::HostnameSource`] is deliberately IGNORED
///
/// The rule does not care whether a name came from DHCP, DNS, mDNS or NetBIOS. Weighting by source
/// is exactly the invention D20 refuses, and a silence about a field is not a decision about it —
/// hence this sentence.
pub fn hostnames_of(side: &L2Side<'_>) -> BTreeSet<String> {
    side.observations
        .iter()
        .flat_map(|observation| observation.facts.iter())
        .filter_map(|fact| match fact {
            Fact::Hostname { name, .. } => {
                let trimmed = name.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_ascii_lowercase())
                }
            }
            _ => None,
        })
        .collect()
}

/// `l2-different-hostname` — two interfaces whose names cannot both be right argue against being
/// one device.
///
/// # The verdict
///
/// [`Verdict::Opposes`] when **both** sides offer a name and the two name sets are **disjoint**;
/// [`Verdict::Neutral`] otherwise. This is **the first producer of `Opposes` in this codebase**.
///
/// # 🔴 Why `Neutral` and not `Opposes` on absence — D20's lock
///
/// *"The rule that wrongly `Opposes` should return `Neutral` — it does not KNOW, it BELIEVES it
/// knows; nine parasitic abstentions out of ten are that"* [architecture.md:1409]. A missing name
/// is not a disagreement, and *absence is derived, never observed, so absence can never oppose* —
/// the `hostname-absence` family's own words. ⚠️ **No trap in that family names this rule**: it
/// constrains this rule by not letting it fire, which is a negative requirement carried by tests
/// here rather than by a trap turning green.
///
/// # ⚠️ Disjointness, on a side that offers several names
///
/// An interface is a group of observations, so a side may offer more than one name — a renamed
/// host, DHCP churn. `Opposes` requires the sets to share **nothing**: a partial overlap stays
/// `Neutral`, because a rule that opposes on a partial overlap claims to know which name is current,
/// and it does not.
///
/// ⚠️ **The committed corpus cannot exercise this.** Measured over every replay stream, exactly one
/// interface carries two different names — and it is the one `cloned-mac` collapses onto, which has
/// no L2 pair at all. So the multi-name behaviour is **reachable in production and unexercised by
/// the corpus**, and it ships with a synthetic guard rather than a claim.
///
/// # Evidence
///
/// On `Opposes`, both sides' [`crate::observation::ObsId`]s, **sorted** — so the evidence of a pair
/// does not depend on which side was the left argument, on `verdict_for_pair`'s measured precedent.
/// A `Neutral` legitimately carries none: D19's *"a rule that fires without leaving its `rule_id`
/// in the database is a rule we cannot debug"* is about a verdict that ARGUES.
pub fn verdict_for_hostname(a: &L2Side<'_>, b: &L2Side<'_>) -> RuleVerdict {
    let names_a = hostnames_of(a);
    let names_b = hostnames_of(b);

    // 🔴 D20'S LOCK, and it is one line that four tests were seen red without.
    //
    // `BTreeSet::is_disjoint` says TRUE of two empty sets, so a rule that only asks "are the names
    // disjoint?" OPPOSES on absence — the exact bug D20 names, reached by doing nothing in
    // particular. Both sides must actually OFFER a name before disagreement can mean anything.
    let both_sides_offer_a_name = !names_a.is_empty() && !names_b.is_empty();
    let opposes = both_sides_offer_a_name && names_a.is_disjoint(&names_b);

    if opposes {
        let mut evidence: Vec<_> = a
            .observations
            .iter()
            .chain(b.observations.iter())
            .map(|observation| observation.obs_id)
            .collect();
        evidence.sort();
        RuleVerdict {
            rule: RuleId(L2_DIFFERENT_HOSTNAME.to_string()),
            verdict: Verdict::Opposes,
            evidence,
        }
    } else {
        RuleVerdict {
            rule: RuleId(L2_DIFFERENT_HOSTNAME.to_string()),
            verdict: Verdict::Neutral,
            evidence: Vec::new(),
        }
    }
}

/// Tests for the L2 rules, over SYNTHETIC inputs only.
///
/// Nothing here reads `fixtures/` — this crate may not touch the filesystem (D47). The
/// corpus-driven half lives in `opencmdb-bin`'s `fixtures.rs`, beside the corpus walks.
///
/// # A deliberate duplication, which a DRY pass may not collapse
///
/// The helpers below re-declare the spellings `l1.rs` and `blocking.rs` use in their own test
/// modules. Those are private to their files and unreachable from here; the alternative is a
/// `pub(crate)` test-helper surface, which is a wider change than this story wants.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::{
        ConnectorId, HostnameSource, L2DomainId, MacAddr, ObsId, Scope, Timestamp, VantageId,
    };
    use uuid::Uuid;

    fn ts() -> Timestamp {
        chrono::DateTime::parse_from_rfc3339("2026-08-30T10:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc)
    }

    /// One observation carrying a MAC and, when `hostname` is `Some`, that name.
    fn observation(n: u128, last: u8, hostname: Option<&str>) -> Observation {
        let mut facts = vec![Fact::Mac {
            addr: MacAddr([0x00, 0x11, 0x22, 0x33, 0x44, last]),
            locally_administered: false,
        }];
        if let Some(name) = hostname {
            facts.push(Fact::Hostname {
                name: name.to_string(),
                source: HostnameSource::Dhcp,
            });
        }
        Observation {
            obs_id: ObsId::from_uuid(Uuid::from_u128(n)),
            connector_id: ConnectorId::from_uuid(Uuid::from_u128(0xC0)),
            observed_at: ts(),
            scope: Scope {
                l2_domain: L2DomainId::from_uuid(Uuid::from_u128(1)),
                vantage: VantageId::from_uuid(Uuid::from_u128(2)),
            },
            facts,
            raw: None,
        }
    }

    fn side(observations: &[Observation]) -> L2Side<'_> {
        L2Side::new(observations.iter().collect())
    }

    // ---- D20's lock: the rule must STAY QUIET where it does not know ----

    /// 🔴 **The story's centre.** A missing name is not a disagreement.
    ///
    /// D20: *"the rule that wrongly `Opposes` should return `Neutral` — it does not KNOW, it
    /// BELIEVES it knows; nine parasitic abstentions out of ten are that."* And the
    /// `hostname-absence` family's own words: *absence is derived, never observed, so absence can
    /// never oppose.*
    #[test]
    fn a_side_with_no_hostname_is_neutral_never_opposes() {
        let left = [observation(1, 0x01, Some("doc-nas-01"))];
        let right = [observation(2, 0x02, None)];

        let verdict = verdict_for_hostname(&side(&left), &side(&right));

        assert_eq!(
            verdict.verdict,
            Verdict::Neutral,
            "a side that offers no name cannot disagree with one; opposing here is D20's named bug"
        );
        assert!(
            verdict.evidence.is_empty(),
            "a Neutral argues nothing and owes no evidence"
        );
    }

    /// An EMPTY name is the same case as a missing one — the corpus pins that equivalence.
    #[test]
    fn an_empty_hostname_is_an_absence_not_a_value() {
        let left = [observation(1, 0x01, Some("doc-nas-01"))];
        let right = [observation(2, 0x02, Some(""))];

        assert_eq!(
            verdict_for_hostname(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "an empty name is the absence of a signal wearing a value's clothes"
        );
    }

    /// Both sides empty is still absence — `"" == ""` is not hostname agreement, and two absences
    /// are not a disagreement either.
    #[test]
    fn two_empty_hostnames_oppose_nothing() {
        let left = [observation(1, 0x01, Some("   "))];
        let right = [observation(2, 0x02, None)];

        assert_eq!(
            verdict_for_hostname(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "neither side observed a name, so nothing was compared"
        );
    }

    // ---- What the rule DOES say ----

    #[test]
    fn two_present_and_different_hostnames_oppose() {
        let left = [observation(1, 0x01, Some("doc-vm-alpha"))];
        let right = [observation(2, 0x02, Some("doc-vm-beta"))];

        let verdict = verdict_for_hostname(&side(&left), &side(&right));

        assert_eq!(
            verdict.verdict,
            Verdict::Opposes,
            "two names that cannot both be right argue against one device"
        );
        assert_eq!(
            verdict.rule.0, L2_DIFFERENT_HOSTNAME,
            "the verdict names the rule that produced it"
        );
        assert_eq!(
            verdict.evidence.len(),
            2,
            "a verdict that ARGUES leaves both sides' observations behind (D19)"
        );
    }

    #[test]
    fn the_same_hostname_opposes_nothing() {
        let left = [observation(1, 0x01, Some("doc-nas-01"))];
        let right = [observation(2, 0x02, Some("doc-nas-01"))];

        assert_eq!(
            verdict_for_hostname(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "agreement is not this rule's business; it only ever opposes or stays quiet"
        );
    }

    /// The evidence is SORTED, so it does not depend on which side was the left argument.
    #[test]
    fn the_evidence_does_not_depend_on_the_argument_order() {
        let left = [observation(9, 0x09, Some("doc-a"))];
        let right = [observation(1, 0x01, Some("doc-b"))];

        assert_eq!(
            verdict_for_hostname(&side(&left), &side(&right)).evidence,
            verdict_for_hostname(&side(&right), &side(&left)).evidence,
            "sorted evidence, or one logical pair produces two unequal verdicts"
        );
    }

    // ---- The two decisions the committed corpus CANNOT exercise ----

    /// ⚠️ Case is folded. **No committed trap exercises this in either direction** — see
    /// [`hostnames_of`]'s doc for why the decision is ASCII folding and what its limit is.
    #[test]
    fn hostnames_differing_only_in_case_are_the_same_name() {
        let left = [observation(1, 0x01, Some("NAS-01"))];
        let right = [observation(2, 0x02, Some("nas-01"))];

        assert_eq!(
            verdict_for_hostname(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "two sources reporting one machine with different capitalisation must not be argued \
             into two devices — D20's bug reached by doing nothing in particular"
        );
    }

    /// ⚠️ A side may offer SEVERAL names. A partial overlap stays quiet: opposing there would claim
    /// to know which name is current. **Unexercised by the corpus** — the only multi-named interface
    /// it holds is the one `cloned-mac` collapses onto, which has no L2 pair.
    #[test]
    fn a_partial_overlap_of_name_sets_stays_neutral() {
        let left = [
            observation(1, 0x01, Some("doc-old-name")),
            observation(2, 0x01, Some("doc-shared")),
        ];
        let right = [observation(3, 0x02, Some("doc-shared"))];

        assert_eq!(
            verdict_for_hostname(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "the sets share a name, so the rule does not know which is current and says so"
        );
    }

    /// Disjoint MULTI-name sets still oppose — the overlap rule is not an excuse to go quiet.
    #[test]
    fn disjoint_multi_name_sets_still_oppose() {
        let left = [
            observation(1, 0x01, Some("doc-a")),
            observation(2, 0x01, Some("doc-b")),
        ];
        let right = [observation(3, 0x02, Some("doc-c"))];

        assert_eq!(
            verdict_for_hostname(&side(&left), &side(&right)).verdict,
            Verdict::Opposes,
            "nothing is shared, so nothing suggests one device"
        );
    }

    /// 🔴 **Why AC3 cannot be carried by the trap gate, made executable rather than quoted.**
    ///
    /// This is the reason the rule id is pinned by a double-literal test in `opencmdb-bin` instead
    /// of by `run_trap`: an `Opposes`-only verdict set abstains, and **an abstention names no
    /// rule**, so nothing downstream can compare the id the trap expects against the id the rule
    /// carried. Misspell [`L2_DIFFERENT_HOSTNAME`] and the gate stays silent.
    ///
    /// D13's ratified arbitration, GitHub issue #54 — not a defect, a decision. It has been in the
    /// algebra since story 5.4b and was invisible for want of a producer; this module is the first
    /// producer, so this is where it becomes observable.
    #[test]
    fn an_opposes_only_verdict_abstains_and_names_no_rule() {
        use crate::identity::cascade::{Conclusion, IdentityAbstentionCause, decide};
        use crate::identity::l1::CURRENT_RULESET_VERSION;

        let left = [observation(1, 0x01, Some("doc-vm-alpha"))];
        let right = [observation(2, 0x02, Some("doc-vm-beta"))];
        let verdict = verdict_for_hostname(&side(&left), &side(&right));
        assert_eq!(
            verdict.verdict,
            Verdict::Opposes,
            "the premise of this test"
        );

        let decision = decide(vec![verdict], CURRENT_RULESET_VERSION);

        assert!(
            matches!(
                decision.conclusion,
                Conclusion::Abstained {
                    cause: IdentityAbstentionCause::AbsenceOfProof
                }
            ),
            "`>= 1 Opposes` with no `Disqualifying` abstains on AbsenceOfProof (D13, issue #54), \
             got {:?}",
            decision.conclusion
        );
        assert_eq!(
            decision.rule(),
            None,
            "and an abstention names NO rule — which is exactly why a misspelled id is invisible \
             to the trap gate, and why AC3 ships on a double literal instead"
        );
    }

    #[test]
    fn a_side_with_no_observations_at_all_is_neutral() {
        let right = [observation(1, 0x01, Some("doc-nas-01"))];

        assert_eq!(
            verdict_for_hostname(&L2Side::new(Vec::new()), &side(&right)).verdict,
            Verdict::Neutral,
            "an empty side observed nothing; it cannot disagree"
        );
    }
}
