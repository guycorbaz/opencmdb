//! The blocker — candidate generation, and the recall floor that proves it hides nothing.
//!
//! # Why a blocker exists at all, and it is not performance
//!
//! D13 states the failure this component prevents: *"if the candidate generator does not propose
//! the pair, no downstream logic can ever group. That is where false-splits are born silently, and
//! **nobody tests blockers**"* [architecture.md:1029-1032]. And it names what the component is for:
//! *"It is there for **SEMANTICS** — it defines the universe of plausible candidates, hence what
//! 'ambiguous' MEANS. **Without blocking, abstention has no denominator.**"*
//! [architecture.md:1034-1036].
//!
//! D13 also disposes of the performance argument with its own arithmetic: at the reference scale of
//! 300 hosts the universe is **90k pairs**, and *"the blocker is **not** there for performance (90k
//! pairs is noise on a NAS i5)"* [architecture.md:1034]. So [`candidates`] is TOTAL by decision: it
//! proposes every unordered pair of distinct observations, and the two exclusions it makes — the
//! self-pair and a repeated `obs_id` — are one rule, named and tested, not a narrowing.
//!
//! ⚠️ **A narrowing key is deliberately absent**, and each candidate key builds a false split into
//! the universe: a device's interfaces are not confined to one L2 domain — a router, a firewall or a
//! dual-homed server has NICs in several VLANs — and D12 makes the device the level where the
//! product keeps its promise [architecture.md:919-923].
//!
//! What the committed corpus can and cannot see about that is **measured, not assumed**, over its
//! **eleven** `must-merge` pairs: a MAC-blocked universe scores **727 per-mille**, a
//! hostname-blocked one **363** (strict — an interface with no hostname agreeing with nobody) or
//! **818** under the loose reading a developer writes first, and an `l2_domain`-blocked one
//! **1000**. _(These read 700 / 400 / 1000 over **ten** pairs until story 6.6 re-measured them:
//! story 5.13b added a `must-merge` trap in 2026-08-10 and the figures were never re-run. Re-derived
//! 2026-08-30 over the committed corpus.)_ So only the `l2_domain`
//! narrowing passes the whole corpus, and it is the one the corpus is BLIND to — which is why the
//! synthetic `two_l2_domains_are_still_a_candidate_pair` exists and why it was written first. The
//! other two keys the corpus would catch on its own; they are refused here for the same reason
//! anyway, because being caught by today's corpus is not a property a key keeps.
//!
//! # The floor is an INTEGER in per-mille, and that is D13's own corollary
//!
//! D13 gives the assertion as `blocking_recall >= 0.999` and, three paragraphs earlier, refuses the
//! type it is written in: *"`confidence` is an **INTEGER in milli-units (0..1000)**, never
//! `REAL`/`DOUBLE` — a threshold at 0.85 compared as a float on two engines = two different identity
//! decisions for the same input"* [architecture.md:1013-1018]. So the floor here is
//! [`BLOCKING_RECALL_FLOOR_PER_MILLE`], an integer, and [`blocking_recall_per_mille`] compares
//! integers. `cargo xtask ci`'s `float-free` gate holds that mechanically over this whole directory.
//!
//! # This is NOT the recall gate D18 refuses by name, and the difference is three-fold
//!
//! D18 puts **pairwise recall** in Tier 2, *"published per release with confidence intervals,
//! trended — blocking nothing"*, and says why: *"false-split is benign — so why would it block a
//! release? A loose threshold on a benign defect is a gate that can never fall, and a gate that
//! cannot fall is decoration"* [architecture.md:1272-1279]. Three things separate that from what is
//! asserted here:
//!
//! - **Different subject.** D18 measures the ENGINE'S OUTPUT — did it group what should group. This
//!   measures the CANDIDATE GENERATOR'S INPUT COVERAGE — did the pair even reach a rule.
//! - **Different venue.** D18 refuses a release gate over bulk statistics. This is an assertion in a
//!   unit test over the frozen corpus, which is where D13 puts it: *"a dedicated assertion:
//!   `blocking_recall >= 0.999`, measured in unit tests, before the scoring exists."*
//! - **Different arithmetic, and this is the honest half.** At the committed corpus's denominator
//!   the floor is **not** a tolerance: with **11** required pairs, one miss gives **909** per-mille
//!   and the floor reds. `>= 999` per-mille **IS zero-tolerance at this scale**, which is the binary form
//!   NFR4 demands. It becomes a real tolerance the moment the required set REACHES 1000 pairs — at
//!   exactly 1000, one miss scores 999 and `999 >= 999` passes, so the boundary is `>= 1000` and not
//!   `> 1000` — and on that day NFR4's *"any fraction is theatre"* bites and the floor must be
//!   revisited rather than inherited. The per-mille dress must not be read as a statistical
//!   tolerance the corpus cannot support.
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

use crate::identity::l1::L1Key;
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
    /// [`crate::identity::l1::decide_pair`] does not refuse `(a, a)` — it answers it like any other
    /// pair, which is a merge when the observation carries a MAC (an observation trivially shares
    /// every key with itself) and `Abstained { AbsenceOfProof }` when it carries none, since
    /// `verdict_for_pair` is `Neutral` on a MAC-less side. Either way nothing there names who
    /// guarantees `a != b`; its doc says only that the pair *"arrives as an argument"*. This
    /// constructor is that holder: the generator is the first place in the engine where the
    /// precondition has an owner.
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

/// One unordered candidate pair at L2 — two DISTINCT interfaces the blocker proposes to a rule.
///
/// # An interface IS an [`L1Key`], and that is the level's whole vocabulary
///
/// [`crate::identity::l1::join`] returns `BTreeMap<L1Key, BTreeSet<ObsId>>` and each entry IS one
/// interface — `resolver.rs`'s own doc says *"`join` NAMES the interface"*. So L2 blocking is over
/// `(l2_domain, mac)` keys, not over database ids: `InterfaceId` exists in this crate and could be
/// taken, but the committed corpus has no store and therefore no `InterfaceId` to supply, and the
/// recall floor below is measured against that corpus.
///
/// # Unordered by construction, exactly as [`CandidatePair`] is
///
/// The fields are private and ordered by [`Self::new`], so `new(a, b) == new(b, a)` holds because
/// the two calls build the same value. ⚠️ **The ordering carries NO meaning**: an [`L1Key`] sorts by
/// a UUID and then by six bytes, which is a construction device and not precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct L2CandidatePair {
    /// The smaller of the two interface keys, by [`L1Key`]'s own ordering.
    low: L1Key,
    /// The larger of the two interface keys, by [`L1Key`]'s own ordering.
    high: L1Key,
}

impl L2CandidatePair {
    /// Build the pair of two interfaces, or `None` when they are the same interface.
    ///
    /// # Why `None` rather than a panic
    ///
    /// Same reasoning as [`CandidatePair::new`]: refusing the pair is an ordinary outcome of asking
    /// for it, not a caller's bug.
    ///
    /// # The duplicate rule is a COROLLARY of this, not a rule of its own
    ///
    /// [`l2_candidates`] takes a slice, which admits a repeated key where `join`'s map cannot. No
    /// code handles that case: the repeated key meets itself and this constructor returns `None`.
    /// ⚠️ **So "duplicates collapse" and "the self-pair is refused" have ONE carrier between them** —
    /// a mutation of this `Equal` arm reds both tests, and neither can red alone. Do not record them
    /// as two guards.
    ///
    /// # Returns
    ///
    /// `Some` with the two keys in canonical order, or `None` if `a == b`.
    pub fn new(a: L1Key, b: L1Key) -> Option<Self> {
        match a.cmp(&b) {
            Ordering::Less => Some(Self { low: a, high: b }),
            Ordering::Greater => Some(Self { low: b, high: a }),
            Ordering::Equal => None,
        }
    }

    /// The smaller of the two interface keys. See the type's doc: the order is a construction device.
    pub fn low(&self) -> L1Key {
        self.low
    }

    /// The larger of the two interface keys. See the type's doc: the order is a construction device.
    pub fn high(&self) -> L1Key {
        self.high
    }
}

/// The universe of candidate pairs over a population of interfaces — every unordered pair of
/// DISTINCT [`L1Key`]s.
///
/// It reads **nothing** but its argument. It calls no `l2-*` rule and no
/// [`crate::identity::cascade::decide`], on this module's founding rule: *a blocker that consults a
/// rule is that rule's echo.*
///
/// # ⚠️ How much of that refusal is CARRIED, measured rather than asserted
///
/// [`crate::identity::l1::join`] and [`crate::identity::l1::decide_pair`] are **unreachable from
/// here by the TYPE** — both demand [`Observation`]s and this function has none. That half is
/// structural. But [`crate::identity::cascade::decide`] takes a `Vec<RuleVerdict>` and could be
/// called: story 6.6's validation planted exactly that call and measured **the whole suite, clippy
/// and all ten gates GREEN**. So that half is a **TRIPWIRE, not a barrier** — read it as *a future
/// story will not add such a call by accident*, never as *such a call cannot exist*.
///
/// # Total by decision, and what the corpus cannot see
///
/// Every exclusion a blocker makes is a false split it can never be talked out of, so there is no
/// narrowing key. Measured over the committed corpus's **three** L2 `must-merge` pairs: a
/// `l2_domain`-narrowed universe scores **1000** per-mille and an uplink-`peer_mac`-narrowed one
/// **1000** as well — *the corpus is blind to both*, and the second is the most tempting L2
/// narrowing there is, being the very signal `l2-uplink-agrees` scores on. **TWO synthetic tests
/// stand between a `l2_domain` narrowing and green** — `l2_two_domains_are_still_a_candidate_pair`,
/// written for it, and `l2_the_universe_is_total_over_distinct_interfaces`, which happens to pin a
/// cross-domain pair as well. _(This sentence named ONE until story 6.6's own mutation M1 reddened
/// two; a claim of sole carriership is worth exactly the mutation that checked it.)_
///
/// ⚠️ **The uplink narrowing has no guard here and cannot have one**: [`L1Key`] carries no
/// [`crate::observation::Fact`], so the narrowing is inexpressible in this function — the TYPE
/// carries what a guard would have claimed to. It IS expressible at the call site, where it was
/// measured leaving the whole suite green; that is registered against the first caller (story 6.12).
///
/// # The count
///
/// `n * (n - 1) / 2` where `n` is the number of DISTINCT keys in the slice — not `interfaces.len()`.
pub fn l2_candidates(interfaces: &[L1Key]) -> BTreeSet<L2CandidatePair> {
    let mut universe = BTreeSet::new();
    for (i, left) in interfaces.iter().enumerate() {
        for right in &interfaces[i + 1..] {
            if let Some(pair) = L2CandidatePair::new(*left, *right) {
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
/// [architecture.md:1013-1018] that forbids the float, and for why at the committed corpus's
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
/// `Some(recall)` in `0..=1000`, or `None` if `required` is empty — **and `None` has that one
/// meaning only.** The empty set is the sole reason this returns nothing; the arithmetic below
/// cannot add a second. Pairs proposed but not required do not raise the value: only members of
/// `required` are counted.
///
/// # Panics
///
/// Never in practice, and the bound is stated rather than hoped for: `hits` counts a subset of
/// `required`, so `hits <= required.len()` and the quotient is at most `PER_MILLE`. The `expect`
/// below is that invariant written down — if it ever fired it would mean the filter counted a pair
/// outside the set it iterates, which is not a recall to report as `None` but a broken function.
pub fn blocking_recall_per_mille<T: Ord>(
    proposed: &BTreeSet<T>,
    required: &BTreeSet<T>,
) -> Option<u32> {
    if required.is_empty() {
        return None;
    }
    let hits = required
        .iter()
        .filter(|pair| proposed.contains(pair))
        .count();
    let per_mille = hits * PER_MILLE / required.len();
    Some(u32::try_from(per_mille).expect("hits <= required.len() bounds the quotient at PER_MILLE"))
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

    /// An observation carrying NO fact at all is still a candidate — the falsifiable half of the
    /// module doc's *"reads no [`crate::observation::Fact`] at all"*.
    ///
    /// The test above varies the KIND of fact; this one removes them entirely, and the difference is
    /// load-bearing. Measured at this story's code review: a `facts.is_empty()` narrowing inside
    /// [`candidates`] left the whole workspace green, because no test fed the generator an empty
    /// `facts` vector and no committed observation has one (0 of 51). Same mutation class as M2,
    /// but M2 has `two_l2_domains_are_still_a_candidate_pair` and this one had nothing.
    #[test]
    fn an_observation_with_no_facts_at_all_is_still_a_candidate() {
        let mut factless = simple(2, mac(0x02));
        factless.facts = Vec::new();

        assert!(
            factless.facts.is_empty(),
            "the fixture must actually carry no fact, or the narrowing this test exists to red \
             stays green"
        );

        assert_eq!(
            candidates(&[simple(1, mac(0x01)), factless]),
            BTreeSet::from([pair(1, 2)]),
            "the generator reads only `obs_id`; an observation with nothing to say is still a \
             pair's other half"
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
        // The EXACT count, not `> 0`: the fixture's one multi-member group is
        // `(l2(10), mac(0x01))` = {1, 2, 3}, so three pairs are checked and the two singleton
        // groups contribute none. `> 0` would still pass if the join degraded to a single 2-member
        // group — a guard an order of magnitude weaker than the fixture it guards.
        assert_eq!(
            checked, 3,
            "the fixture's grouped pairs are exactly three, or this test no longer proves what it \
             names"
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

    // ---- L2: the interface-level blocker (story 6.6) ----

    /// One interface key, in domain `d`, with `last` as the MAC's final byte.
    fn iface(d: u128, last: u8) -> L1Key {
        (l2(d), mac(last))
    }

    /// AC5's guard, and **the only one of its kind this story ships**.
    ///
    /// The committed corpus is BLIND to an `l2_domain` narrowing: measured over its three L2
    /// `must-merge` pairs, such a universe scores a full 1000 per-mille, because every one of them
    /// sits in a single domain. This test is the only thing between that narrowing and green — the
    /// L1 twin `two_l2_domains_are_still_a_candidate_pair` exists for exactly the same reason and
    /// was likewise written first.
    ///
    /// ⚠️ **It is not the SOLE carrier, measured**: mutation M1 reds this test AND
    /// [`l2_the_universe_is_total_over_distinct_interfaces`], whose fixture happens to include a
    /// cross-domain pair. Two carriers, deliberately kept — but neither may be described as the only
    /// thing standing between the narrowing and green.
    ///
    /// ⚠️ **Its uplink counterpart is deliberately absent and must not be added**: [`L1Key`] carries
    /// no [`crate::observation::Fact`], so an uplink narrowing cannot be written inside
    /// [`l2_candidates`] at all — the mutation that would red such a guard does not compile. The
    /// type carries what the guard would have claimed to; see [`l2_candidates`]' doc.
    #[test]
    fn l2_two_domains_are_still_a_candidate_pair() {
        // A router, a firewall or a dual-homed server has NICs in several VLANs, and D12 makes the
        // DEVICE the level where the product keeps its promise.
        let universe = l2_candidates(&[iface(1, 0x01), iface(2, 0x02)]);

        assert_eq!(
            universe.len(),
            1,
            "two interfaces in DIFFERENT L2 domains are still one device's NICs; a blocker that \
             narrows on the domain builds that false split into the universe and no committed trap \
             can see it"
        );
    }

    #[test]
    fn l2_the_self_pair_is_refused_in_the_type() {
        assert_eq!(
            L2CandidatePair::new(iface(1, 0x01), iface(1, 0x01)),
            None,
            "an interface is not a candidate against itself"
        );
    }

    /// ⚠️ This shares its ONE carrier with the test above — see [`L2CandidatePair::new`]'s doc.
    /// Both red under a single mutation of the `Equal` arm and neither can red alone, so this is a
    /// corollary of the type and not a second guard.
    #[test]
    fn l2_a_repeated_interface_yields_no_pair() {
        let universe = l2_candidates(&[iface(1, 0x01), iface(1, 0x01)]);

        assert!(
            universe.is_empty(),
            "a slice admits a repeated key where `join`'s map cannot; the pair type absorbs it"
        );
    }

    #[test]
    fn l2_the_pair_is_unordered_by_construction() {
        let (a, b) = (iface(1, 0x01), iface(1, 0x02));

        assert_eq!(
            L2CandidatePair::new(a, b),
            L2CandidatePair::new(b, a),
            "the two calls build the same value; no caller has to remember to normalise"
        );
    }

    #[test]
    fn l2_an_empty_population_has_an_empty_universe() {
        assert!(l2_candidates(&[]).is_empty(), "no interface, no pair");
    }

    #[test]
    fn l2_a_single_interface_has_an_empty_universe() {
        assert!(
            l2_candidates(&[iface(1, 0x01)]).is_empty(),
            "one interface cannot pair with anything"
        );
    }

    /// TOTAL by decision: `n * (n - 1) / 2` over DISTINCT keys, and the count is pinned exactly.
    ///
    /// A `>= 1` oracle would pass under a blocker that collapsed the whole population onto one
    /// pair — the weak-oracle defect story 5.11b's code review measured on the L1 corpus test.
    #[test]
    fn l2_the_universe_is_total_over_distinct_interfaces() {
        let universe = l2_candidates(&[iface(1, 0x01), iface(1, 0x02), iface(2, 0x03)]);

        assert_eq!(
            universe.len(),
            3,
            "three interfaces give three unordered pairs"
        );
        assert!(
            universe.contains(&L2CandidatePair::new(iface(1, 0x01), iface(2, 0x03)).unwrap()),
            "including the cross-domain one"
        );
    }

    /// The truncation guard AC2 needs and `float-free` CANNOT give it.
    ///
    /// The gate forbids a float TYPE under `identity/`; it says nothing about the ORDER of an
    /// integer computation. Story 6.6's validation measured that reordering
    /// `hits * PER_MILLE / len` into `hits / len * PER_MILLE` leaves all ten gates green — the gate
    /// forbids the TYPE and says nothing about the ORDER of an integer computation.
    ///
    /// ⚠️ **It is not the only carrier, measured**: mutation M3 reds **three** tests — this one and
    /// the two L1 truncation tests above, which exercise the same arithmetic. _(The draft called it
    /// the only one. Two of three must be 666, and under the reordering it is 0.)_
    #[test]
    fn the_per_mille_scale_is_applied_before_the_division() {
        let required = BTreeSet::from([
            L2CandidatePair::new(iface(1, 0x01), iface(1, 0x02)).unwrap(),
            L2CandidatePair::new(iface(1, 0x01), iface(1, 0x03)).unwrap(),
            L2CandidatePair::new(iface(1, 0x02), iface(1, 0x03)).unwrap(),
        ]);
        let proposed: BTreeSet<_> = required.iter().take(2).copied().collect();

        assert_eq!(
            blocking_recall_per_mille(&proposed, &required),
            Some(666),
            "two of three is 666 per-mille; dividing before scaling would give 0 and no gate can \
             see the difference"
        );
    }
}
