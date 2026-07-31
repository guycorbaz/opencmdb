//! The blocker — candidate generation, and the recall floor that proves it hides nothing.
//!
//! # Why a blocker exists at all, and it is not performance
//!
//! D13 states the failure this component prevents: *"if the candidate generator does not propose
//! the pair, no downstream logic can ever group. That is where false-splits are born silently, and
//! **nobody tests blockers**"* [architecture.md:1004-1007]. And it names what the component is for:
//! *"It is there for **SEMANTICS** — it defines the universe of plausible candidates, hence what
//! 'ambiguous' MEANS. **Without blocking, abstention has no denominator.**"*
//! [architecture.md:1009-1011].
//!
//! D13 also disposes of the performance argument with its own arithmetic: at the reference scale of
//! 300 hosts the universe is **90k pairs**, and *"the blocker is **not** there for performance (90k
//! pairs is noise on a NAS i5)"* [architecture.md:1009]. So [`candidates`] is TOTAL by decision: it
//! proposes every unordered pair of distinct observations, and the two exclusions it makes — the
//! self-pair and a repeated `obs_id` — are one rule, named and tested, not a narrowing.
//!
//! ⚠️ **A narrowing key is deliberately absent.** Blocking on the MAC, the hostname or the
//! `l2_domain` would each pass the whole committed corpus and each build a false split into the
//! universe: a device's interfaces are not confined to one L2 domain — a router, a firewall or a
//! dual-homed server has NICs in several VLANs — and D12 makes the device the level where the
//! product keeps its promise [architecture.md:919-928].
//!
//! # The floor is an INTEGER in per-mille, and that is D13's own corollary
//!
//! D13 gives the assertion as `blocking_recall >= 0.999` and, three paragraphs earlier, refuses the
//! type it is written in: *"`confidence` is an **INTEGER in milli-units (0..1000)**, never
//! `REAL`/`DOUBLE` — a threshold at 0.85 compared as a float on two engines = two different identity
//! decisions for the same input"* [architecture.md:988-993]. So the floor here is
//! [`BLOCKING_RECALL_FLOOR_PER_MILLE`], an integer, and [`blocking_recall_per_mille`] compares
//! integers. `cargo xtask ci`'s `float-free` gate holds that mechanically over this whole directory.
//!
//! # This is NOT the recall gate D18 refuses by name, and the difference is three-fold
//!
//! D18 puts **pairwise recall** in Tier 2, *"published per release with confidence intervals,
//! trended — blocking nothing"*, and says why: *"false-split is benign — so why would it block a
//! release? A loose threshold on a benign defect is a gate that can never fall, and a gate that
//! cannot fall is decoration"* [architecture.md:1246-1253]. Three things separate that from what is
//! asserted here:
//!
//! - **Different subject.** D18 measures the ENGINE'S OUTPUT — did it group what should group. This
//!   measures the CANDIDATE GENERATOR'S INPUT COVERAGE — did the pair even reach a rule.
//! - **Different venue.** D18 refuses a release gate over bulk statistics. This is an assertion in a
//!   unit test over the frozen corpus, which is where D13 puts it: *"a dedicated assertion:
//!   `blocking_recall >= 0.999`, measured in unit tests, before the scoring exists."*
//! - **Different arithmetic, and this is the honest half.** At the committed corpus's denominator
//!   the floor is **not** a tolerance: with 10 required pairs, one miss gives 900 per-mille and the
//!   floor reds. `>= 999` per-mille **IS zero-tolerance at this scale**, which is the binary form
//!   NFR4 demands. It becomes a real tolerance only if the required set ever exceeds 1000 pairs —
//!   and on that day NFR4's *"any fraction is theatre"* bites and the floor must be revisited rather
//!   than inherited. The per-mille dress must not be read as a statistical tolerance the corpus
//!   cannot support.
//!
//! ⚠️ **Nothing here advances NFR4.** NFR4 is at the DEVICE level; this adds no truth-table column
//! and gates no release.
//!
//! # What this module does not do
//!
//! It produces no verdict and builds no [`crate::identity::cascade::Decision`]: [`candidates`] calls
//! neither [`crate::identity::l1::join`] nor [`crate::identity::l1::decide_pair`]. Proposing is not
//! judging, and a blocker that consults a rule is that rule's echo — it would exclude exactly the
//! pairs the rule refuses, which is the false split this component exists to prevent. It consumes no
//! structural reading of a MAC (the U/L bit, the IANA prefixes, the I/G bit) and reads no
//! [`crate::observation::Fact`] at all. It writes nothing and reads nothing but its argument.
//!
//! The relation to the join is still pinned, in the other direction only: every pair sharing an L1
//! key is in the universe. That is a property of a TOTAL universe, and it is a superset property —
//! checkable only by importing the join, which this module's tests do on purpose.
//!
//! # The universe is quadratic in the slice the CALLER supplies
//!
//! D13's 90k figure is one poll of 300 hosts. The day a caller hands this function a retention
//! window instead of a single poll, the universe must be narrowed — and the recall assertion below
//! is precisely what would make that narrowing safe rather than silent. That day is registered, not
//! built for here.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use crate::observation::{ObsId, Observation};

/// One unordered candidate pair — two DISTINCT observations the blocker proposes to a rule.
///
/// # Unordered by construction
///
/// The two fields are private and ordered by [`Self::new`], so `new(a, b) == new(b, a)` holds
/// because the two calls build the same value, not because a caller remembered to normalise. A bare
/// tuple would carry no invariant and host no impl; this type carries the one property every
/// consumer depends on.
///
/// ⚠️ **The ordering carries NO meaning.** [`ObsId`] is a UUID, so low/high is a construction device
/// and nothing else — it is not "first seen", not chronology, not precedence. Reading an order into
/// it would be reading identity out of a byte comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CandidatePair {
    /// The smaller of the two ids, by [`ObsId`]'s own ordering.
    low: ObsId,
    /// The larger of the two ids, by [`ObsId`]'s own ordering.
    high: ObsId,
}

impl CandidatePair {
    /// Build the pair of two observation ids, or `None` when they are the same id.
    ///
    /// # Why the self-pair is refused here
    ///
    /// [`crate::identity::l1::decide_pair`] answers `(a, a)` with a merge — an observation is
    /// trivially its own interface — and its doc says the pair *"arrives as an argument"* without
    /// naming who guarantees `a != b`. This constructor is that holder: the generator is the first
    /// place in the engine where the precondition has an owner.
    ///
    /// `None` rather than a panic or a silent normalisation: refusing the pair is an ordinary
    /// outcome of asking for it, not a caller's bug, and a normalisation would let a self-pair enter
    /// the universe under another shape.
    ///
    /// # Returns
    ///
    /// `Some` with the two ids in canonical order, or `None` if `a == b`.
    pub fn new(a: ObsId, b: ObsId) -> Option<Self> {
        match a.cmp(&b) {
            Ordering::Less => Some(Self { low: a, high: b }),
            Ordering::Greater => Some(Self { low: b, high: a }),
            Ordering::Equal => None,
        }
    }

    /// The smaller of the two ids. See the type's doc: the order is a construction device.
    pub fn low(&self) -> ObsId {
        self.low
    }

    /// The larger of the two ids. See the type's doc: the order is a construction device.
    pub fn high(&self) -> ObsId {
        self.high
    }
}

/// The universe of candidate pairs over a slice of observations — every unordered pair of
/// DISTINCT observation ids.
///
/// It reads **nothing** but its argument: no clock, no I/O, no SQL, no repository, and not `raw`.
/// It reads no [`crate::observation::Fact`] either — only [`Observation::obs_id`].
///
/// # Total by decision
///
/// See this module's doc: at the reference scale the universe is noise, and every exclusion a
/// blocker makes is a false split it can never be talked out of. There are exactly two exclusions
/// and they are one rule — **distinct id**, not distinct index — so the self-pair is out and two
/// entries repeating one `obs_id` produce no pair.
///
/// # The count
///
/// `n * (n - 1) / 2` where `n` is the number of DISTINCT `obs_id`s in the slice — not
/// `observations.len()`. The two coincide until a duplicate id appears.
///
/// # Why a [`BTreeSet`]
///
/// Order-independence and de-duplication hold by CONSTRUCTION rather than through a sort a refactor
/// can drop — the same reasoning that made [`crate::identity::l1::join`]'s value a set.
pub fn candidates(observations: &[Observation]) -> BTreeSet<CandidatePair> {
    let mut universe = BTreeSet::new();
    for (i, left) in observations.iter().enumerate() {
        for right in &observations[i + 1..] {
            if let Some(pair) = CandidatePair::new(left.obs_id, right.obs_id) {
                universe.insert(pair);
            }
        }
    }
    universe
}

/// The scale per-mille is expressed in — 1000 parts, D13's milli-units.
const PER_MILLE: usize = 1000;

/// D13's `blocking_recall` floor, as an INTEGER in per-mille.
///
/// D13 writes the assertion as `blocking_recall >= 0.999`. The value is the same; the TYPE is not,
/// and the type is the decision — see this module's doc for the milli-units corollary
/// [architecture.md:988-993] that forbids the float, and for why at the committed corpus's
/// denominator this floor is zero-tolerance rather than a tolerance.
pub const BLOCKING_RECALL_FLOOR_PER_MILLE: u32 = 999;

/// How much of `required` the blocker actually proposed, in per-mille — or `None` when there is
/// nothing to require.
///
/// # `None` for an empty requirement set
///
/// A recall with no denominator is **undefined**, not perfect. Returning the full 1000 would let the
/// floor pass over nothing, which is the guarantee the corpus lock already refuses to give
/// (*"reporting 'nothing to check' on the deletion of the thing being guarded is a guarantee the
/// gate does not have"*) and which D13 states outright: *"without blocking, abstention has no
/// denominator."*
///
/// # Truncation
///
/// Integer division truncates DOWN, which is the conservative direction for a floor: 2 of 3 is 666
/// per-mille, never 667. A borderline set therefore reds rather than rounds up into a pass.
///
/// # Arguments
///
/// `required` is a set, so a requirement stated twice cannot inflate the denominator.
///
/// # Returns
///
/// `Some(recall)` in `0..=1000`, or `None` if `required` is empty. Pairs proposed but not required
/// do not raise the value: only members of `required` are counted.
pub fn blocking_recall_per_mille(
    proposed: &BTreeSet<CandidatePair>,
    required: &BTreeSet<CandidatePair>,
) -> Option<u32> {
    if required.is_empty() {
        return None;
    }
    let hits = required
        .iter()
        .filter(|pair| proposed.contains(pair))
        .count();
    u32::try_from(hits * PER_MILLE / required.len()).ok()
}

/// Tests for the blocker, over SYNTHETIC inputs only.
///
/// Nothing here reads `fixtures/` — [`crate::observation`] is a domain type and this crate may not
/// touch the filesystem (D47). The corpus-driven half of the recall assertion lives in
/// `opencmdb-bin`'s `fixtures.rs`, which is where the corpus-wide walks already are.
///
/// # The corpus cannot see this module's central claim
///
/// Every committed trap pair sits in ONE scope, so a blocker that proposed only same-`l2_domain`
/// pairs would score a full 1000 per-mille on the corpus and be invisible to every corpus test.
/// [`two_l2_domains_are_still_a_candidate_pair`] is the only thing standing between that
/// implementation and green.
///
/// # A deliberate duplication, which a DRY pass may not collapse
///
/// The helpers below re-declare `l1.rs`'s spellings — `obs_id`, `l2`, `ts`, `mac`, `observation`.
/// They are private to that file's own test module and unreachable from here; the alternative is a
/// `pub(crate)` test-helper surface, which is a wider change than this story wants.
#[cfg(test)]
mod tests {
    use super::*;
    // The superset property below is not checkable without the thing it is a superset of. This
    // import is the point of `every_pair_inside_a_join_group_is_a_candidate`; the prohibition in
    // this module's doc binds `candidates`, not its tests.
    use crate::identity::l1::join;
    use crate::observation::{
        ConnectorId, Fact, HostnameSource, L2DomainId, MacAddr, Scope, Timestamp, VantageId,
    };
    use uuid::Uuid;

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
        chrono::DateTime::parse_from_rfc3339("2026-08-01T10:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc)
    }

    fn mac(last: u8) -> MacAddr {
        MacAddr([0x00, 0x11, 0x22, 0x33, 0x44, last])
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

    /// The pair of two ids, for a test that has already decided they are distinct.
    fn pair(a: u128, b: u128) -> CandidatePair {
        CandidatePair::new(obs_id(a), obs_id(b)).expect("the two ids are distinct")
    }

    // ---- AC2: the pair type is unordered, and refuses the self-pair ----

    #[test]
    fn the_self_pair_is_refused() {
        assert_eq!(
            CandidatePair::new(obs_id(1), obs_id(1)),
            None,
            "an observation is not a candidate against itself; the generator is where that \
             precondition has a holder"
        );
    }

    #[test]
    fn a_pair_is_unordered() {
        assert_eq!(
            CandidatePair::new(obs_id(1), obs_id(2)),
            CandidatePair::new(obs_id(2), obs_id(1)),
            "the same two ids build the same pair whichever side they arrive on"
        );
    }

    #[test]
    fn the_accessors_report_the_canonical_order() {
        let forward = pair(1, 2);
        let backward = CandidatePair::new(obs_id(2), obs_id(1)).expect("distinct");

        assert_eq!(forward.low(), backward.low());
        assert_eq!(forward.high(), backward.high());
        assert!(
            forward.low() < forward.high(),
            "the constructor orders the two ids, so the accessors cannot disagree"
        );
        assert_eq!(forward.low(), obs_id(1));
        assert_eq!(forward.high(), obs_id(2));
    }

    // ---- AC1: the universe is total ----

    #[test]
    fn no_observation_yields_no_candidate() {
        assert!(
            candidates(&[]).is_empty(),
            "an empty slice proposes nothing, and that is not an error"
        );
    }

    #[test]
    fn one_observation_yields_no_candidate() {
        assert!(
            candidates(&[simple(1, mac(0x01))]).is_empty(),
            "one observation has nothing to be paired with"
        );
    }

    #[test]
    fn four_observations_yield_six_pairs() {
        let universe = candidates(&[
            simple(1, mac(0x01)),
            simple(2, mac(0x02)),
            simple(3, mac(0x03)),
            simple(4, mac(0x04)),
        ]);

        assert_eq!(universe.len(), 6);
        assert_eq!(
            universe,
            BTreeSet::from([
                pair(1, 2),
                pair(1, 3),
                pair(1, 4),
                pair(2, 3),
                pair(2, 4),
                pair(3, 4),
            ]),
            "the universe is every unordered pair, named rather than merely counted"
        );
    }

    #[test]
    fn the_count_is_quadratic_in_the_number_of_distinct_ids() {
        for n in 0u128..=6 {
            let observations: Vec<Observation> = (1..=n).map(|i| simple(i, mac(i as u8))).collect();

            // The DISTINCT id count, not `observations.len()`. The two coincide here and diverge
            // the day a duplicate id appears, which is what `a_repeated_obs_id_yields_no_pair`
            // covers — a count asserted from `len()` is green today and wrong that day.
            let distinct: BTreeSet<ObsId> = observations.iter().map(|o| o.obs_id).collect();
            let expected = distinct.len() * distinct.len().saturating_sub(1) / 2;

            assert_eq!(
                candidates(&observations).len(),
                expected,
                "with {} distinct ids the universe holds {expected} pair(s)",
                distinct.len()
            );
        }
    }

    #[test]
    fn a_repeated_obs_id_yields_no_pair() {
        let once = simple(1, mac(0x01));
        let again = once.clone();

        assert_eq!(once.obs_id, again.obs_id, "the fixture must repeat the id");
        assert!(
            candidates(&[once, again]).is_empty(),
            "the rule is distinct ID, not distinct index"
        );
    }

    #[test]
    fn the_universe_is_input_order_independent() {
        let a = simple(1, mac(0x01));
        let b = simple(2, mac(0x02));
        let c = simple(3, mac(0x03));

        assert_eq!(
            candidates(&[a.clone(), b.clone(), c.clone()]),
            candidates(&[c, b, a]),
            "the same observations in any input order propose the same universe"
        );
    }

    #[test]
    fn the_generator_reads_neither_raw_nor_observed_at_nor_the_connector() {
        let plain = [simple(1, mac(0x01)), simple(2, mac(0x02))];

        let mut decorated = plain.clone();
        decorated[0].raw = Some("{\"whatever\": true}".to_string());
        decorated[1].connector_id = ConnectorId::from_uuid(Uuid::from_u128(999));
        decorated[1].observed_at = chrono::DateTime::parse_from_rfc3339("2019-01-02T03:04:05Z")
            .unwrap()
            .with_timezone(&chrono::Utc);

        assert_ne!(plain[0].raw, decorated[0].raw, "the test must vary raw");
        assert_ne!(plain[1].connector_id, decorated[1].connector_id);
        assert_ne!(plain[1].observed_at, decorated[1].observed_at);

        assert_eq!(
            candidates(&plain),
            candidates(&decorated),
            "provenance and time are not read by the blocker"
        );
        // ⚠️ The clock/SQL/repository half of purity is UNREACHABLE from a `&[Observation]`, so no
        // test can red on it. This asserts the falsifiable half only.
    }

    #[test]
    fn two_l2_domains_are_still_a_candidate_pair() {
        let here = observation(1, l2(10), vantage(100), &[mac(0x01)]);
        let there = observation(2, l2(20), vantage(100), &[mac(0x02)]);

        assert_ne!(
            here.scope.l2_domain, there.scope.l2_domain,
            "the test must actually vary the domain, or it degrades into a single-domain test and \
             a domain-blocked universe passes it"
        );

        assert_eq!(
            candidates(&[here, there]),
            BTreeSet::from([pair(1, 2)]),
            "a device's interfaces are not confined to one L2 domain; excluding the pair would \
             build a false split into the universe"
        );
    }

    #[test]
    fn an_observation_with_no_mac_is_still_a_candidate() {
        let mut hostname_only = simple(2, mac(0x02));
        hostname_only.facts = vec![Fact::Hostname {
            name: "kitchen-pi".to_string(),
            source: HostnameSource::Dhcp,
        }];

        assert!(
            !hostname_only
                .facts
                .iter()
                .any(|f| matches!(f, Fact::Mac { .. })),
            "the fixture must actually carry no MAC"
        );

        assert_eq!(
            candidates(&[simple(1, mac(0x01)), hostname_only]),
            BTreeSet::from([pair(1, 2)]),
            "the blocker proposes; whether a rule can answer is the rule's business"
        );
    }

    #[test]
    fn every_pair_inside_a_join_group_is_a_candidate() {
        let observations = [
            observation(1, l2(10), vantage(100), &[mac(0x01)]),
            observation(2, l2(10), vantage(200), &[mac(0x01)]),
            observation(3, l2(10), vantage(100), &[mac(0x01), mac(0x02)]),
            observation(4, l2(20), vantage(100), &[mac(0x02)]),
        ];

        let universe = candidates(&observations);
        let groups = join(&observations);

        let mut checked = 0usize;
        for members in groups.values() {
            let members: Vec<ObsId> = members.iter().copied().collect();
            for (i, left) in members.iter().enumerate() {
                for right in &members[i + 1..] {
                    let inside =
                        CandidatePair::new(*left, *right).expect("a group holds distinct ids");
                    assert!(
                        universe.contains(&inside),
                        "the join grouped {left} with {right}, so the blocker must have proposed \
                         that pair"
                    );
                    checked += 1;
                }
            }
        }
        assert!(
            checked > 0,
            "the fixture must produce at least one grouped pair, or this proves nothing"
        );
    }

    // ---- AC3: the floor and the recall arithmetic ----

    #[test]
    fn the_floor_is_nine_hundred_and_ninety_nine_per_mille() {
        // An INDEPENDENT literal, not a read of the constant: every other assertion in this story
        // compares against `BLOCKING_RECALL_FLOOR_PER_MILLE` and would move with it, so weakening
        // the floor would otherwise red nothing. The value is D13's, expressed in milli-units.
        assert_eq!(BLOCKING_RECALL_FLOOR_PER_MILLE, 999);
    }

    #[test]
    fn every_required_pair_proposed_is_full_recall() {
        let required = BTreeSet::from([pair(1, 2), pair(1, 3), pair(2, 3)]);
        let proposed = required.clone();

        assert_eq!(blocking_recall_per_mille(&proposed, &required), Some(1000));
    }

    #[test]
    fn one_miss_in_ten_is_nine_hundred_per_mille() {
        let required: BTreeSet<CandidatePair> = (2..=11).map(|n| pair(1, n)).collect();
        assert_eq!(required.len(), 10, "the denominator must actually be ten");

        let mut proposed = required.clone();
        let dropped = pair(1, 11);
        assert!(
            proposed.remove(&dropped),
            "the fixture must drop a required pair"
        );

        assert_eq!(
            blocking_recall_per_mille(&proposed, &required),
            Some(900),
            "one miss out of ten reds the floor — at this denominator it is zero-tolerance"
        );
    }

    #[test]
    fn integer_division_truncates_down() {
        let required = BTreeSet::from([pair(1, 2), pair(1, 3), pair(1, 4)]);
        let proposed = BTreeSet::from([pair(1, 2), pair(1, 3)]);

        assert_eq!(
            blocking_recall_per_mille(&proposed, &required),
            Some(666),
            "two of three truncates DOWN, which is the conservative direction for a floor"
        );
    }

    #[test]
    fn an_empty_requirement_set_has_no_recall() {
        let proposed = BTreeSet::from([pair(1, 2)]);

        assert_eq!(
            blocking_recall_per_mille(&proposed, &BTreeSet::new()),
            None,
            "a recall with no denominator is undefined, not perfect: a full score here would let \
             the floor pass over nothing"
        );
    }

    #[test]
    fn a_pair_proposed_but_not_required_does_not_inflate_recall() {
        let required = BTreeSet::from([pair(1, 2), pair(1, 3)]);
        let proposed = BTreeSet::from([pair(1, 2), pair(1, 3), pair(2, 3), pair(3, 4)]);

        assert!(
            proposed.len() > required.len(),
            "the fixture must over-propose"
        );
        assert_eq!(
            blocking_recall_per_mille(&proposed, &required),
            Some(1000),
            "recall counts the required set, so a larger universe cannot exceed the full value"
        );
    }
}
