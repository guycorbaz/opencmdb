//! The L1 join and the two rules the committed corpus names — the first producer of a [`Verdict`].
//!
//! D13 splits the cascade by level: **L1 = pure A** — *"a deterministic join on the scope-qualified
//! key `(l2_domain, mac) -> interface`. **It is not a probabilistic problem.**"*
//! [architecture.md:984-986]. "Pure A" is the ordered cascade D13 rejects *for L2* because it has no
//! native abstention and no representation for CONFLICT [architecture.md:934-937]; at L1 that
//! absence is correct, because the key either matches or it does not.
//!
//! So this file holds a **lookup, not an inference**: [`join`] groups observations by the key, and
//! [`decide_pair`] answers one candidate pair with exactly one [`RuleVerdict`] which it hands to
//! [`crate::identity::cascade::decide`]. Combining a verdict SET is `decide`'s job and is not
//! repeated here.
//!
//! # What this file does not do
//!
//! It does not generate candidate pairs and computes no recall floor — the blocker is an L2 organ
//! (D13 feeds it to *"the scoring"*, and grouping is L2 by definition [architecture.md:891]), and it
//! lives in [`crate::identity::blocking`]. The pair still arrives as an argument, and this file does
//! not call the blocker any more than the blocker calls this one: proposing and judging are
//! separate, and a blocker that consulted a rule would be that rule's echo. It does not map a
//! [`Decision`] onto [`crate::score::Outcome`]: no conversion exists in either direction,
//! deliberately.
//!
//! # Structural facts are read, never scored
//!
//! D13 calls the U/L bit and the IANA redundancy-protocol prefixes `Disqualifying` **"as grouping
//! anchors"** [architecture.md:1002] — and grouping is L2. **No rule here consumes either.** They are
//! readings available at ingestion; the refusal that does consume the prefix is the corpus's
//! `l2-virtual-mac-prefix`, which Epic 6 designs. Two committed traps require exactly this: a
//! `must-merge` pole answered by `l1-exact-mac` on a locally-administered MAC
//! (`fixtures/scenario/traps/randomized-mac.toml`, *"the U/L bit is no licence to abstain on an
//! exact-MAC match"*) and another on a VRRP-range MAC (`vrrp-virtual-mac.toml`, *"L1 is
//! deterministic on (l2_domain, mac)"*). A structural refusal at L1 would red both.
//!
//! The reading itself already exists — [`MacAddr::is_locally_administered`] — and is not re-derived
//! here. D17 refuses a level below `interface` because it *"would treat the U/L bit as a
//! judgement"* [architecture.md:1171-1173]; D16 refuses abstaining on a virtual MAC because *"this
//! is not an ambiguity, it is a topology fact"* [architecture.md:1106-1114].
//!
//! # ⚠️ A third structural reading exists, is NOT consumed here, and is NOT a decision
//!
//! The **I/G bit** — bit 0 of the first octet — marks a GROUP address (multicast or broadcast).
//! `opencmdb-bin`'s corpus privacy scan already states the reading, in this same epic: *"Set means
//! the address is a group (multicast or broadcast) address, **which names no interface**"*
//! (`fixtures.rs`, `is_multicast_mac`).
//!
//! **This file does not consume it either**, and unlike the U/L bit and the IANA prefixes that is
//! **not** backed by a committed trap — no trap exercises a group address, and refusing them here
//! would red none. So the honest statement is: `ff:ff:ff:ff:ff:ff`, `01:00:5e:…` and the all-zero
//! address are **currently treated as ordinary interface identities**, and three devices reporting
//! the broadcast address merge into one group. `a_group_address_merges_today_and_that_is_unresolved`
//! pins that behaviour so the class cannot change in silence.
//!
//! ⚠️ **That is a recorded gap, not a reasoned position.** Its failure mode is a FALSE MERGE, which
//! is what NFR4 exists to prevent, and the corpus can never catch it — the committed streams carry
//! no group address, and the privacy scan now refuses to let one be committed. It is registered
//! with an owner rather than decided by this story, because L1 consuming a structural reading is
//! exactly what this section says L1 does not do, and reversing that needs a decision, not a patch.
//!
//! # No float
//!
//! There is nothing to rank: the key matches or it does not. `cargo xtask ci`'s `float-free` gate
//! holds this mechanically over this whole directory.

use std::collections::{BTreeMap, BTreeSet};

use crate::identity::cascade::{Decision, RuleVerdict, RulesetVersion, Verdict, decide};
use crate::observation::{L2DomainId, MacAddr, ObsId, Observation};
use crate::trap::RuleId;

/// The scope-qualified key D13 names: the L2 domain a MAC is unique in, and the MAC.
///
/// ⚠️ **`vantage` is deliberately NOT part of it**, although it sits beside `l2_domain` in
/// [`crate::observation::Scope`]. D13's key has two components [architecture.md:984-985], so two
/// vantages seeing the same `(l2_domain, mac)` resolve to the SAME interface.
///
/// **The architecture never writes that sentence — it is DERIVED here**, from D21's refusal of
/// connector precedence: *"NO connector precedence — deliberately. A precedence is a merge rule in
/// disguise… disagreement is an `Opposes`. A source that cannot know answers `Neutral` — it does not
/// LOSE an arbitration."* [architecture.md:1417-1428]. Putting `vantage` in the key would make one
/// interface into two per observer, which is a per-observer precedence by another name.
///
/// ⚠️ The scope dimension ships with **no contact with reality**, and D61 measured why: on the
/// developer's network `client.vlan` is missing 48/48 and `client.network_id` has one distinct
/// value, so *"`(l2_domain, mac)` degenerates to `(constant, mac)`"* and *"`network_id ≡ l2_domain`
/// is UNVERIFIED"* [architecture.md:4264-4288]. Every committed replay stream likewise carries one
/// `l2_domain`. The key is scope-qualified **in code**; that the derivation holds on a real network
/// is not claimed here, and the tests below are synthetic for that reason.
pub type L1Key = (L2DomainId, MacAddr);

/// The corpus's id for the rule that answers a `must-merge` pole at L1.
///
/// The two rule ids live in the committed corpus, **not in the architecture**: `grep -c "l1-"` over
/// `architecture.md` returns zero, and D13 names no rule at all. They are spelled here exactly as
/// `fixtures/scenario/traps/*.toml` spells them, and **since story 5.7 a test says so** rather than
/// this sentence asking to be believed: `opencmdb_bin`'s `l1_runner` walks every committed trap
/// file and asserts that every rule id beginning `l1-` is one of these two, that BOTH occur, and
/// that every id the corpus writes equals its own `trim()` and `to_lowercase()` — so `run_trap`'s
/// unnormalized string comparison cannot produce a wrong-rule failure on a correct answer. The test
/// is in `opencmdb-bin` because D47 forbids this crate to read a file.
///
/// ⚠️ A `&str` rather than a `const RuleId`: [`RuleId`] wraps a `String`, which is not
/// const-constructible (`error[E0015]`).
pub const L1_EXACT_MAC: &str = "l1-exact-mac";

/// The corpus's id for the rule that answers a `must-not-merge` pole at L1.
///
/// See [`L1_EXACT_MAC`] for why these are `&str` and why the corpus, not the architecture, is the
/// source of the spelling.
pub const L1_DISTINCT_MAC: &str = "l1-distinct-mac";

/// The version of the rule set this file's rules constitute (D14).
///
/// **One, not zero.** `RulesetVersion(0)` is constructible and means nothing — D14's "mandatory" is
/// about PRESENCE, and no value is refused — so one is the first value that names an actual ruleset:
/// the two rules below.
///
/// ⚠️ There is deliberately **no `Default`** on [`Decision`], and this constant does not add one.
/// The absent `Default` is what makes the version unforgettable: removing the field breaks every
/// construction site and every read.
pub const CURRENT_RULESET_VERSION: RulesetVersion = RulesetVersion(1);

/// Every L1 key an observation carries — one per [`crate::observation::Fact::Mac`] it holds.
///
/// A [`BTreeSet`], so an observation that repeats the same MAC in its `facts` contributes to its
/// group once rather than twice. An observation carrying no MAC has no L1 key and yields an empty
/// set; that is not an error, it is a source that did not see a MAC.
fn keys_of(observation: &Observation) -> BTreeSet<L1Key> {
    use crate::observation::Fact;

    observation
        .facts
        .iter()
        .filter_map(|fact| match fact {
            Fact::Mac { addr, .. } => Some((observation.scope.l2_domain, *addr)),
            // ⚠️ Exhaustive on purpose — NO `_` arm, in the idiom `fixtures.rs`'s
            // `assert_facts_are_synthetic` established. `Fact` is `#[non_exhaustive]`, but that is
            // inert inside its own crate, so a new variant CARRYING AN ADDRESS must break this
            // match and force a decision rather than defaulting to "not an identity key" in
            // silence. Measured: with a `_` arm, adding `Fact::Uplink { peer_mac }` as a key left
            // the whole workspace green while a switch merged with everything plugged into it.
            //
            // `Uplink` is the one to think hardest about: `peer_mac` is a real address, and it
            // identifies the NEIGHBOUR, never this device. It is a topology edge, which is L2's.
            Fact::Uplink { .. } => None,
            Fact::IpV4 { .. } => None,
            Fact::Hostname { .. } => None,
            Fact::DhcpLease { .. } => None,
            Fact::OuiVendor { .. } => None,
            Fact::Rtt { .. } => None,
        })
        .collect()
}

/// Group observations by the scope-qualified key — the L1 join, as a pure function.
///
/// It reads **nothing** but its argument: no clock, no I/O, no SQL, no repository, and not `raw`,
/// which D3 keeps out of every decision. `observed_at` and `connector_id` are likewise unread.
///
/// # The map's key IS the interface identity at L1
///
/// No `InterfaceId` is minted. The architecture names an `EntityId` in a file that is not on disk
/// [architecture.md:3356], minting identity belongs to the composition root (D48), and a newtype
/// with no persistence would be surface with no consumer.
///
/// # Why the value is a [`BTreeSet`]
///
/// Order-independence is required of this function, and a `Vec` value would defeat it: the same
/// observations supplied in two orders would build two maps whose groups compare unequal, so the
/// property would hold only through a normalisation step a refactor can drop. The set carries it in
/// the type instead — and settles the repeated-MAC case in the same stroke.
pub fn join(observations: &[Observation]) -> BTreeMap<L1Key, BTreeSet<ObsId>> {
    let mut groups: BTreeMap<L1Key, BTreeSet<ObsId>> = BTreeMap::new();
    for observation in observations {
        for key in keys_of(observation) {
            groups.entry(key).or_default().insert(observation.obs_id);
        }
    }
    groups
}

/// What L1 says about one candidate pair — exactly one [`RuleVerdict`], with its evidence.
///
/// # The three cases, and the quantifier that makes them decidable
///
/// An observation may carry several MACs, so "the same MAC" needs a quantifier. It is
/// **existential**: the pair shares an interface when it shares AT LEAST ONE key. D12 is the ground
/// — *a MAC identifies an INTERFACE* [architecture.md:884] — so one shared key is one shared
/// interface whatever the other keys say. The universal reading would make a multi-NIC host oppose
/// itself, which the corpus's `multi-nic` family contradicts by expecting that pair to merge.
///
/// | the pair | rule | verdict |
/// |---|---|---|
/// | shares at least one key | [`L1_EXACT_MAC`] | [`Verdict::Decisive`] |
/// | both carry a MAC, shares none | [`L1_DISTINCT_MAC`] | [`Verdict::Disqualifying`] |
/// | either side carries no MAC | [`L1_EXACT_MAC`] | [`Verdict::Neutral`] |
///
/// # Why a key mismatch is `Disqualifying` and not `Opposes`
///
/// The chain is forced by the corpus, not chosen:
///
/// 1. the corpus expects a `must-not-merge` pole to be answered **by name** —
///    `rule = "l1-distinct-mac"` — and D19 is why: *"the fixture asserts the RULE, not just the
///    outcome … a test that checks only the verdict goes green for the right answer reached by the
///    wrong rule"* [architecture.md:1307-1310];
/// 2. [`crate::identity::cascade::Conclusion`] has three variants and `Abstained` carries no rule;
/// 3. so the only non-merge conclusion that NAMES a rule is `NoMatch { rule }`;
/// 4. `decide` reaches `NoMatch` from one input class only — a `Disqualifying` present. An `Opposes`
///    alone reaches `Abstained { AbsenceOfProof }`, which names no rule;
/// 5. therefore a key mismatch must be `Disqualifying`.
///
/// It is also defensible on its own terms: at L1 the interface **is** the scope-qualified key, so
/// two different keys are not a weak argument against a merge — they are a definitional refusal. A
/// reading, not an inference, which is the category D13 uses for its structural facts.
///
/// ⚠️ **The id says `distinct-mac`; the condition is distinct KEY, and the two are not the same.**
/// Two observations carrying the **identical** MAC in different `l2_domain`s share no key, so this
/// rule fires and names MAC-distinctness for a case where the MACs are equal. The id is the
/// corpus's and this file does not own it, so the overload is recorded rather than renamed — and it
/// is invisible on the committed corpus, where every replay stream carries one `l2_domain` (D61).
/// Story 5.7's corpus comparison now pins the SPELLING against those bytes — it says nothing about
/// the overload, and cannot: no committed trap separates "different MAC" from "same MAC, different
/// domain". The day one does, one of the two has to move.
///
/// # Why a missing MAC is `Neutral`
///
/// D20 names the bug class this avoids: *"the rule that wrongly `Opposes` should return `Neutral`:
/// it does not KNOW, it BELIEVES it knows… nine parasitic abstentions out of ten are that"*
/// [architecture.md:1383-1387]. A rule with no MAC to compare does not know, so it says nothing and
/// carries no evidence — `decide` then answers `Abstained { AbsenceOfProof }`.
///
/// The id such a verdict travels under is [`L1_EXACT_MAC`], the rule that TRIED to fire, rather than
/// a third id the corpus could never name. ⚠️ That choice is **recorded but not decision-bearing**:
/// a `Neutral` names no rule in the `Conclusion`, yet the id is still carried in
/// `Decision::verdict_vector` and printed by its `Debug`, so it is observable — it simply does not
/// change any decision.
///
/// # Evidence
///
/// A verdict that ARGUES carries both observations' [`ObsId`]s, **sorted**, so the pair is unordered
/// and `decide_pair(a, b) == decide_pair(b, a)` — D19: *"a rule that fires must leave
/// its `rule_id` and its evidence behind — a rule that fires without leaving its `rule_id` is
/// undebuggable in production"*. A `Neutral` legitimately carries none, so the invariant is not
/// "evidence is never empty". `decide` cannot enforce this: it never reads `evidence`, which is
/// precisely why the guard belongs on this side.
/// ⚠️ **`pub(crate)`, not `pub`, and that is load-bearing.** [`crate::identity::cascade`]'s doc
/// states that L1's two rules *"never appear in one vector"*, which is what lets its undesigned
/// byte-order tiebreak stay unconsulted at L1. That claim is true of [`decide_pair`] and would be
/// FALSE of an exposed intermediate: `decide(vec![verdict_for_pair(a,b), verdict_for_pair(c,d)], _)`
/// is a two-line call that puts both ids in one vector. Exposing the intermediate beside the
/// combinator, while documenting the combination as undesigned, is the defect this visibility
/// prevents. [`decide_pair`] is the entry point.
pub(crate) fn verdict_for_pair(a: &Observation, b: &Observation) -> RuleVerdict {
    let keys_a = keys_of(a);
    let keys_b = keys_of(b);

    if keys_a.is_empty() || keys_b.is_empty() {
        return RuleVerdict {
            rule: RuleId(L1_EXACT_MAC.to_string()),
            verdict: Verdict::Neutral,
            evidence: Vec::new(),
        };
    }

    let shares_a_key = keys_a.intersection(&keys_b).next().is_some();
    let (rule, verdict) = if shares_a_key {
        (L1_EXACT_MAC, Verdict::Decisive)
    } else {
        (L1_DISTINCT_MAC, Verdict::Disqualifying)
    };

    // Sorted, so the evidence of a pair does not depend on which side was the left argument.
    // Without this, `decide_pair(a, b) != decide_pair(b, a)` — same conclusion, unequal `Decision`s
    // — and `Decision` derives `PartialEq`, so any downstream dedup or "have we decided this pair
    // already" check would see one logical pair as two. Measured before it was fixed. Same
    // reasoning as [`join`]'s `BTreeSet`: the property holds by construction, not by a convention.
    let mut evidence = vec![a.obs_id, b.obs_id];
    evidence.sort();

    RuleVerdict {
        rule: RuleId(rule.to_string()),
        verdict,
        evidence,
    }
}

/// Decide one candidate pair at L1 — the first caller of
/// [`crate::identity::cascade::decide`] outside its own tests.
///
/// It builds the one-element verdict vector, calls `decide` with
/// [`CURRENT_RULESET_VERSION`], and returns the [`Decision`] unchanged. There is no `Result`:
/// `decide` is total and this function adds no failure of its own.
///
/// The pair travels **by reference**, not by [`ObsId`]: a pair of ids would force [`join`]'s map in
/// here and make this function a candidate generator, which is
/// [`crate::identity::blocking::candidates`]'s organ and not this one's.
///
/// ⚠️ **`a != b` is the CALLER's precondition, and it now has a holder.** This function does not
/// refuse the self-pair `(a, a)`: it answers it like any other pair, which is a merge when the
/// observation carries a MAC — it trivially shares every key with itself — and
/// `Abstained { AbsenceOfProof }` when it carries none, since [`verdict_for_pair`] is `Neutral` on a
/// MAC-less side. Neither answer is a refusal.
/// [`crate::identity::blocking::CandidatePair::new`] is where the exclusion lives: it returns `None`
/// for two equal ids, so a pair that comes out of the blocker can never be one.
pub fn decide_pair(a: &Observation, b: &Observation) -> Decision {
    decide(vec![verdict_for_pair(a, b)], CURRENT_RULESET_VERSION)
}

/// Tests for the L1 join and its two rules.
///
/// # Synthetic inputs only
///
/// Nothing here reads `fixtures/`. The corpus cannot exercise this file's central claim: every
/// committed replay stream carries exactly one `l2_domain`, so a join keyed on the bare MAC would
/// pass the whole corpus. The two-scope test below is the only thing standing between that
/// implementation and green, which is why the epic demands synthetic inputs by name.
///
/// # Two deliberate redundancies, neither of which a DRY pass may collapse
///
/// - [`expected_l1_conclusion`] restates the decision from D13's text rather than by calling
///   [`decide_pair`]. An expectation computed by calling the code under test proves nothing at all —
///   the idiom is `cascade.rs`'s `expected_conclusion`.
/// - [`CORPUS_EXACT_MAC`] and [`CORPUS_DISTINCT_MAC`] spell the two rule ids again, as literals, so
///   the assertions do not read the constants they are checking. **This was measured to be
///   load-bearing**: with every expectation built from [`L1_EXACT_MAC`], renaming that constant to a
///   non-canonical spelling leaves the entire suite green. Story 5.7 added a THIRD, independent
///   statement — from the TOML side, in `opencmdb_bin`'s `l1_runner` — which is what catches BOTH
///   literals being wrong relative to the corpus, something these two cannot. None of the three may
///   be collapsed into the others.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::cascade::{Conclusion, IdentityAbstentionCause};
    use crate::observation::{ConnectorId, Fact, HostnameSource, Scope, Timestamp, VantageId};
    use uuid::Uuid;

    /// The rule id the committed corpus writes for a `must-merge` pole, restated independently of
    /// [`L1_EXACT_MAC`]. Deliberate redundancy — see this module's doc.
    const CORPUS_EXACT_MAC: &str = "l1-exact-mac";
    /// The rule id the committed corpus writes for a `must-not-merge` pole, restated independently
    /// of [`L1_DISTINCT_MAC`]. Deliberate redundancy — see this module's doc.
    const CORPUS_DISTINCT_MAC: &str = "l1-distinct-mac";

    fn obs_id(n: u128) -> ObsId {
        ObsId::from_uuid(Uuid::from_u128(n))
    }

    fn l2(n: u128) -> L2DomainId {
        L2DomainId::from_uuid(Uuid::from_u128(n))
    }

    fn vantage(n: u128) -> VantageId {
        VantageId::from_uuid(Uuid::from_u128(n))
    }

    fn ts() -> Timestamp {
        chrono::DateTime::parse_from_rfc3339("2026-07-31T10:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc)
    }

    fn mac(last: u8) -> MacAddr {
        MacAddr([0x00, 0x11, 0x22, 0x33, 0x44, last])
    }

    /// A locally-administered address: bit 1 of the first octet is set.
    fn locally_administered_mac() -> MacAddr {
        MacAddr([0x02, 0x00, 0x5e, 0x00, 0x53, 0x20])
    }

    /// An address in the IANA VRRP range.
    fn vrrp_mac() -> MacAddr {
        MacAddr([0x00, 0x00, 0x5e, 0x00, 0x01, 0x0a])
    }

    /// An observation carrying the given MACs, in the given L2 domain, seen from the given vantage.
    fn observation(
        id: u128,
        domain: L2DomainId,
        seen_by: VantageId,
        macs: &[MacAddr],
    ) -> Observation {
        Observation {
            obs_id: obs_id(id),
            connector_id: ConnectorId::from_uuid(Uuid::from_u128(7)),
            observed_at: ts(),
            scope: Scope {
                l2_domain: domain,
                vantage: seen_by,
            },
            facts: macs
                .iter()
                .map(|addr| Fact::Mac {
                    addr: *addr,
                    locally_administered: addr.is_locally_administered(),
                })
                .collect(),
            raw: None,
        }
    }

    /// The common shape: one MAC, domain 10, vantage 100.
    fn simple(id: u128, addr: MacAddr) -> Observation {
        observation(id, l2(10), vantage(100), &[addr])
    }

    /// D13's table, restated for the L1 verdict set, INDEPENDENTLY of [`decide_pair`].
    ///
    /// One rule speaks at L1, so only three of D13's rows are reachable: a `Decisive` with no
    /// `Opposes` is a `Match` [architecture.md:970]; any `Disqualifying` is a `NoMatch` naming its
    /// rule [architecture.md:969]; only `Neutral` is an absence of proof [architecture.md:974],
    /// which this engine answers as an abstention because `NoMatch` carries a rule and that class
    /// has none to name.
    ///
    /// ⚠️ **Protected deliberate redundancy.** It must not be rewritten to call [`decide_pair`] or
    /// [`crate::identity::cascade::decide`]; that would make every assertion below a tautology.
    fn expected_l1_conclusion(rule: &str, verdict: Verdict) -> Conclusion {
        match verdict {
            Verdict::Decisive => Conclusion::Match {
                rule: RuleId(rule.to_string()),
            },
            Verdict::Disqualifying => Conclusion::NoMatch {
                rule: RuleId(rule.to_string()),
            },
            Verdict::Neutral => Conclusion::Abstained {
                cause: IdentityAbstentionCause::AbsenceOfProof,
            },
            Verdict::Supports | Verdict::Opposes => {
                panic!("no L1 rule emits this verdict; if one does, this oracle owes it a row")
            }
        }
    }

    // ---- AC1: the join is pure, and the key is (l2_domain, mac) ----

    #[test]
    fn the_join_is_order_independent() {
        let a = simple(1, mac(0x01));
        let b = simple(2, mac(0x01));
        let c = simple(3, mac(0x02));

        let forward = join(&[a.clone(), b.clone(), c.clone()]);
        let backward = join(&[c, b, a]);

        assert_eq!(
            forward, backward,
            "the same observations in any input order give the same grouping"
        );
    }

    #[test]
    fn two_vantages_seeing_one_key_land_in_one_group() {
        let here = observation(1, l2(10), vantage(100), &[mac(0x01)]);
        let there = observation(2, l2(10), vantage(200), &[mac(0x01)]);

        assert_ne!(
            here.scope.vantage, there.scope.vantage,
            "the test must actually vary the vantage, or it proves nothing"
        );

        let groups = join(&[here, there]);

        assert_eq!(groups.len(), 1, "vantage is not part of the key");
        assert_eq!(
            groups[&(l2(10), mac(0x01))],
            BTreeSet::from([obs_id(1), obs_id(2)])
        );
    }

    #[test]
    fn an_observation_carrying_two_macs_joins_two_groups() {
        let dual = observation(1, l2(10), vantage(100), &[mac(0x01), mac(0x02)]);

        let groups = join(&[dual]);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[&(l2(10), mac(0x01))], BTreeSet::from([obs_id(1)]));
        assert_eq!(groups[&(l2(10), mac(0x02))], BTreeSet::from([obs_id(1)]));
    }

    #[test]
    fn an_observation_repeating_one_mac_contributes_once() {
        let repeated = observation(1, l2(10), vantage(100), &[mac(0x01), mac(0x01)]);

        let groups = join(&[repeated]);

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[&(l2(10), mac(0x01))], BTreeSet::from([obs_id(1)]));
    }

    #[test]
    fn an_observation_with_no_mac_has_no_l1_key() {
        let mut hostname_only = simple(1, mac(0x01));
        hostname_only.facts = vec![Fact::Hostname {
            name: "kitchen-pi".to_string(),
            source: HostnameSource::Dhcp,
        }];

        let groups = join(&[hostname_only]);

        assert!(
            groups.is_empty(),
            "an observation with no MAC appears in no group, and that is not an error"
        );
    }

    #[test]
    fn the_join_reads_neither_raw_nor_observed_at_nor_the_connector() {
        let plain = simple(1, mac(0x01));

        let mut decorated = plain.clone();
        decorated.raw = Some("{\"whatever\": true}".to_string());
        decorated.connector_id = ConnectorId::from_uuid(Uuid::from_u128(999));
        decorated.observed_at = chrono::DateTime::parse_from_rfc3339("2019-01-02T03:04:05Z")
            .unwrap()
            .with_timezone(&chrono::Utc);

        assert_ne!(plain.raw, decorated.raw, "the test must actually vary raw");
        assert_ne!(plain.observed_at, decorated.observed_at);
        assert_ne!(plain.connector_id, decorated.connector_id);

        assert_eq!(
            join(&[plain]),
            join(&[decorated]),
            "provenance and time are not read by the join"
        );
    }

    // ---- AC2: the same MAC in two l2_domains is two interfaces ----

    #[test]
    fn the_same_mac_in_two_l2_domains_is_two_groups() {
        let here = observation(1, l2(10), vantage(100), &[mac(0x01)]);
        let there = observation(2, l2(20), vantage(100), &[mac(0x01)]);

        assert_ne!(
            here.scope.l2_domain, there.scope.l2_domain,
            "the test must actually vary the scope dimension, or it degrades into a single-scope \
             test and a bare-MAC key passes it"
        );
        assert_eq!(
            keys_of(&here).len(),
            1,
            "one key per side, so the grouping below cannot be explained by a second MAC"
        );
        // ⚠️ Without this, editing `there` to carry a different MAC leaves both assertions green
        // and `groups.len()` still 2 — and the test would prove nothing about scope qualification,
        // which is the only thing standing between a bare-MAC key and green.
        assert_eq!(
            keys_of(&here).iter().map(|(_, m)| *m).collect::<Vec<_>>(),
            keys_of(&there).iter().map(|(_, m)| *m).collect::<Vec<_>>(),
            "both sides must carry the IDENTICAL MAC; only the domain may differ"
        );

        let groups = join(&[here, there]);

        assert_eq!(
            groups.len(),
            2,
            "the key is scope-qualified, not the bare address"
        );
    }

    #[test]
    fn no_rule_reports_two_l2_domains_as_the_same_interface() {
        let here = observation(1, l2(10), vantage(100), &[mac(0x01)]);
        let there = observation(2, l2(20), vantage(100), &[mac(0x01)]);

        assert_ne!(here.scope.l2_domain, there.scope.l2_domain);

        let decision = decide_pair(&here, &there);

        assert_eq!(
            decision.conclusion,
            expected_l1_conclusion(CORPUS_DISTINCT_MAC, Verdict::Disqualifying)
        );
    }

    // ---- AC3: structural facts are read, never scored ----

    #[test]
    fn an_exact_match_fires_on_a_locally_administered_mac() {
        let addr = locally_administered_mac();
        assert!(
            addr.is_locally_administered(),
            "the fixture must actually set the U/L bit"
        );

        let decision = decide_pair(&simple(1, addr), &simple(2, addr));

        assert_eq!(
            decision.conclusion,
            expected_l1_conclusion(CORPUS_EXACT_MAC, Verdict::Decisive),
            "the U/L bit is no licence to abstain on an exact-MAC match"
        );
    }

    #[test]
    fn an_exact_match_fires_on_a_redundancy_protocol_mac() {
        let addr = vrrp_mac();
        assert_eq!(
            [addr.0[0], addr.0[1], addr.0[2], addr.0[3], addr.0[4]],
            [0x00, 0x00, 0x5e, 0x00, 0x01],
            "the fixture must actually sit in the IANA VRRP range"
        );

        let decision = decide_pair(&simple(1, addr), &simple(2, addr));

        assert_eq!(
            decision.conclusion,
            expected_l1_conclusion(CORPUS_EXACT_MAC, Verdict::Decisive),
            "L1 is deterministic on the key, so the exact-MAC rule fires for a virtual MAC \
             exactly as for any other"
        );
    }

    /// A group address is treated as an ordinary identity TODAY — pinned so it cannot change in
    /// silence, and named as unresolved rather than as a position.
    ///
    /// ⚠️ **This test asserts the CURRENT behaviour, and that behaviour is a recorded gap.** The
    /// I/G bit marks a multicast or broadcast address, which `opencmdb-bin`'s privacy scan already
    /// says *"names no interface"*. Here three devices reporting the broadcast address merge into
    /// one group. Its failure mode is a FALSE MERGE. If a later story decides to refuse group
    /// addresses at L1, **this test is the one that must red and be rewritten** — that is its job.
    #[test]
    fn a_group_address_merges_today_and_that_is_unresolved() {
        let broadcast = MacAddr([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
        assert_eq!(broadcast.0[0] & 1, 1, "the fixture must set the I/G bit");

        let groups = join(&[
            simple(1, broadcast),
            simple(2, broadcast),
            simple(3, broadcast),
        ]);

        assert_eq!(
            groups.len(),
            1,
            "three devices reporting the broadcast address are ONE group today — the class is \
             registered, not decided"
        );
        assert_eq!(
            decide_pair(&simple(1, broadcast), &simple(2, broadcast)).conclusion,
            expected_l1_conclusion(CORPUS_EXACT_MAC, Verdict::Decisive)
        );
    }

    #[test]
    fn a_peers_mac_is_not_this_devices_identity() {
        let mut with_uplink = simple(1, mac(0x01));
        with_uplink.facts.push(Fact::Uplink {
            peer_mac: mac(0x99),
            peer_port: "gi0/1".to_string(),
        });

        let groups = join(&[with_uplink]);

        assert_eq!(
            groups.keys().collect::<Vec<_>>(),
            vec![&(l2(10), mac(0x01))],
            "an Uplink's peer_mac identifies the NEIGHBOUR; treating it as a key would merge a \
             switch with everything plugged into it"
        );
    }

    // ---- AC4: two rules fire, with the corpus's names, each leaving evidence ----

    #[test]
    fn a_shared_key_is_decisive_and_carries_both_obs_ids() {
        let verdict = verdict_for_pair(&simple(1, mac(0x01)), &simple(2, mac(0x01)));

        assert_eq!(verdict.rule, RuleId(CORPUS_EXACT_MAC.to_string()));
        assert_eq!(verdict.verdict, Verdict::Decisive);
        assert_eq!(verdict.evidence, vec![obs_id(1), obs_id(2)]);
    }

    #[test]
    fn distinct_keys_are_disqualifying_and_carry_both_obs_ids() {
        let verdict = verdict_for_pair(&simple(1, mac(0x01)), &simple(2, mac(0x02)));

        assert_eq!(verdict.rule, RuleId(CORPUS_DISTINCT_MAC.to_string()));
        assert_eq!(verdict.verdict, Verdict::Disqualifying);
        assert_eq!(verdict.evidence, vec![obs_id(1), obs_id(2)]);
    }

    #[test]
    fn a_pair_with_no_mac_is_neutral_and_carries_no_evidence() {
        let mut blind = simple(2, mac(0x01));
        blind.facts = vec![];

        let verdict = verdict_for_pair(&simple(1, mac(0x01)), &blind);

        assert_eq!(verdict.verdict, Verdict::Neutral, "the rule does not KNOW");
        assert!(
            verdict.evidence.is_empty(),
            "a rule that says nothing read nothing"
        );
    }

    #[test]
    fn overlapping_mac_sets_share_an_interface() {
        let left = observation(1, l2(10), vantage(100), &[mac(0x01), mac(0x02)]);
        let right = observation(2, l2(10), vantage(100), &[mac(0x02), mac(0x03)]);

        let verdict = verdict_for_pair(&left, &right);

        assert_eq!(
            verdict.rule,
            RuleId(CORPUS_EXACT_MAC.to_string()),
            "one shared key is one shared interface, whatever the other keys say"
        );
        assert_eq!(verdict.verdict, Verdict::Decisive);
    }

    #[test]
    fn disjoint_mac_sets_do_not_share_an_interface() {
        let left = observation(1, l2(10), vantage(100), &[mac(0x01), mac(0x02)]);
        let right = observation(2, l2(10), vantage(100), &[mac(0x03), mac(0x04)]);

        let verdict = verdict_for_pair(&left, &right);

        assert_eq!(verdict.rule, RuleId(CORPUS_DISTINCT_MAC.to_string()));
        assert_eq!(verdict.verdict, Verdict::Disqualifying);
    }

    // ---- AC5: a key mismatch names its rule through NoMatch ----

    #[test]
    fn a_key_mismatch_concludes_no_match_naming_the_rule() {
        let decision = decide_pair(&simple(1, mac(0x01)), &simple(2, mac(0x02)));

        assert_eq!(
            decision.conclusion,
            Conclusion::NoMatch {
                rule: RuleId(CORPUS_DISTINCT_MAC.to_string())
            },
            "only NoMatch names a rule for a non-merge; an Opposes would abstain and name nothing"
        );
        assert_eq!(
            decision.rule(),
            Some(&RuleId(CORPUS_DISTINCT_MAC.to_string()))
        );
    }

    // ---- AC6: decide is called from outside its own tests ----

    #[test]
    fn a_decision_carries_the_current_ruleset_version() {
        let decision = decide_pair(&simple(1, mac(0x01)), &simple(2, mac(0x01)));

        assert_eq!(decision.ruleset_version, RulesetVersion(1));
        assert_eq!(decision.ruleset_version, CURRENT_RULESET_VERSION);
    }

    #[test]
    fn the_ruleset_version_is_not_the_meaningless_zero() {
        assert_ne!(
            CURRENT_RULESET_VERSION,
            RulesetVersion(0),
            "zero is constructible and means nothing; one names an actual ruleset"
        );
    }

    #[test]
    fn the_decision_carries_the_verdict_that_produced_it() {
        let left = simple(1, mac(0x01));
        let right = simple(2, mac(0x01));

        let decision = decide_pair(&left, &right);

        // ⚠️ The expectation is a LITERAL, not `verdict_for_pair(&left, &right)`. Computing it by
        // calling the code under test would make this test unable to fail for any change to the
        // producer — both sides would move together — which is the standard this module's own doc
        // sets for `expected_l1_conclusion`.
        assert_eq!(
            decision.verdict_vector,
            vec![RuleVerdict {
                rule: RuleId(CORPUS_EXACT_MAC.to_string()),
                verdict: Verdict::Decisive,
                evidence: vec![obs_id(1), obs_id(2)],
            }],
            "the vector decide returns is the one this producer built"
        );
    }

    #[test]
    fn a_pair_decides_the_same_whichever_side_is_the_left_argument() {
        let a = simple(1, mac(0x01));
        let b = simple(2, mac(0x01));
        let c = simple(3, mac(0x02));

        assert_eq!(
            decide_pair(&a, &b),
            decide_pair(&b, &a),
            "a candidate pair is unordered: the whole Decision, evidence included, must agree"
        );
        assert_eq!(
            decide_pair(&a, &c),
            decide_pair(&c, &a),
            "and that holds for the refusing rule too, not only the merging one"
        );
    }

    // ---- AC7: producer-side validation ----

    #[test]
    fn the_producers_rule_ids_are_canonical() {
        for id in [L1_EXACT_MAC, L1_DISTINCT_MAC] {
            assert!(!id.is_empty(), "a blank rule id names nothing");
            assert_eq!(id, id.trim(), "a rule id carries no surrounding whitespace");
            assert_eq!(
                id,
                id.to_lowercase(),
                "the corpus spells its rule ids in lower case"
            );
        }
        assert_eq!(L1_EXACT_MAC, CORPUS_EXACT_MAC);
        assert_eq!(L1_DISTINCT_MAC, CORPUS_DISTINCT_MAC);
    }

    #[test]
    fn every_emitted_rule_id_is_non_blank() {
        let pairs = [
            (simple(1, mac(0x01)), simple(2, mac(0x01))),
            (simple(1, mac(0x01)), simple(2, mac(0x02))),
        ];
        for (left, right) in pairs {
            let verdict = verdict_for_pair(&left, &right);
            assert!(
                !verdict.rule.0.trim().is_empty(),
                "mirrors the refusal Trap::validate already makes on the expectation side"
            );
        }
    }

    #[test]
    fn an_arguing_verdict_never_ships_empty_evidence() {
        let arguing = [
            verdict_for_pair(&simple(1, mac(0x01)), &simple(2, mac(0x01))),
            verdict_for_pair(&simple(1, mac(0x01)), &simple(2, mac(0x02))),
        ];
        for verdict in &arguing {
            assert_ne!(verdict.verdict, Verdict::Neutral, "this fixture must argue");
            assert!(
                !verdict.evidence.is_empty(),
                "decide never reads evidence, so this guard belongs on the producer side"
            );
        }
        // The two fixtures must be the two ARGUING rules, not the same one twice — otherwise this
        // test could pass while one of the two producing branches ships empty evidence.
        assert_ne!(arguing[0].rule, arguing[1].rule);
    }

    #[test]
    fn the_producer_emits_exactly_one_verdict_for_one_pair() {
        let decision = decide_pair(&simple(1, mac(0x01)), &simple(2, mac(0x01)));

        assert_eq!(decision.verdict_vector.len(), 1);
    }

    // ---- An abstention reached from a producer, not from a literal ----

    #[test]
    fn a_produced_abstention_names_no_rule() {
        let mut blind = simple(2, mac(0x01));
        blind.facts = vec![];

        let decision = decide_pair(&simple(1, mac(0x01)), &blind);

        assert_eq!(
            decision.conclusion,
            Conclusion::Abstained {
                cause: IdentityAbstentionCause::AbsenceOfProof
            }
        );
        assert_eq!(
            decision.rule(),
            None,
            "every decision names a rule; only an abstention names a cause"
        );
    }

    #[test]
    fn a_neutral_verdict_still_records_the_rule_that_tried() {
        let mut blind = simple(2, mac(0x01));
        blind.facts = vec![];

        let decision = decide_pair(&simple(1, mac(0x01)), &blind);

        assert_eq!(decision.rule(), None, "the conclusion names no rule");
        assert_eq!(
            decision.verdict_vector[0].rule,
            RuleId(CORPUS_EXACT_MAC.to_string()),
            "the id is still recorded in the vector — observable, but not decision-bearing"
        );
    }
}
