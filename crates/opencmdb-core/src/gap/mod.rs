//! The gap engine — the product's core thesis, as a pure function.
//!
//! `gap := declared.value != current_observation.value` (D3). The computation reads no `origin`
//! and no history (a field adopted yesterday can drift again tomorrow), descends into no SQL
//! (D10), and touches no clock (D19): [`reconcile`] is a pure function of its inputs, which is
//! exactly what makes it deterministically testable. When it cannot conclude it ABSTAINS — it
//! never guesses and never merges (FR16); abstentions are counted and grouped by cause, and the
//! count measures REACH, not debt (FR16b).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::observation::{ConnectorId, Fact, ObsId, Observation, Timestamp};

/// A field on which the declared value and the observed value disagree — **a gap**.
///
/// ⚠️ It read *"a drift"* until story 6b.6. The canonical glossary binds `gap`/« écart » for this
/// concept and its preamble forbids synonyms in so many words, and the EN column governs *"docs,
/// API and code"*, not only the UI. 🔑 The story promised `opencmdb-core` would be *untouched*, and
/// that promise is what would have sheltered this line — story 5.13b's finding exactly: **a promise
/// of non-modification protects behaviour and shelters false sentences.** Narrowed there to *no
/// BEHAVIOUR change*, and the sentence corrected in place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gap {
    pub field: String,
    pub declared: String,
    pub observed: String,
}

/// Why the engine did not conclude. Never a guess, never a merge — reach, not debt (FR16b).
///
/// # This is the RECONCILIATION vocabulary; the identity cascade has its own
///
/// These three causes answer *why comparing a declared field against observations did not
/// conclude*. The identity cascade answers a different question — why a VERDICT SET did not
/// conclude — and since story 5.3 it names its own causes,
/// [`crate::identity::cascade::IdentityAbstentionCause`]. In the trap corpus this type lives on
/// the **expectation** side ([`crate::trap::Expectation::MustAbstain`], written by the trap's
/// author and frozen into the truth format by story 4.2); the cascade's type lives on the
/// **outcome** side ([`crate::score::Outcome::Abstained`]).
///
/// The asymmetry cannot make the release gate asymmetric, and the reason is a mechanism rather
/// than a promise: [`crate::score::score`]'s 3×3 matches `Outcome::Abstained { .. }` and cannot
/// reach the payload, and [`crate::score::run_trap`] compares rules only where both sides are
/// `Some`, which an abstention never is. Nothing compares the two vocabularies, and a test in
/// `score`'s module says so.
///
/// Widening this enum is not free, and the cost falls hardest on a variant [`reconcile`] cannot
/// produce. The three below are all produced here and all earn their label; adding the cascade's
/// `Ambiguous` — the case story 5.3 weighed — would add one the corpus format can express, that
/// `cause_label` must label and that two locales must translate, for something no code path here
/// ever returns. Measured: adding it yields exactly one `error[E0004]`, in `page.rs`'s
/// `cause_label`, and breaks nothing else in the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AbstentionCause {
    /// An observation that is not the perimeter entity (e.g. an undocumented device).
    OutOfPerimeter,
    /// A declared field for which no in-perimeter observation reported a value. Absence is NOT
    /// fabricated (NFR7) — the engine derives absence only when a source is live, which is later.
    NoObservedValue,
    /// Two in-perimeter observations disagree on a field; the engine refuses to pick (FR16).
    ConflictingObservations,
}

/// The outcome of reconciling one cardinality-1 perimeter.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Reconciliation {
    pub gaps: Vec<Gap>,
    pub abstentions: BTreeMap<AbstentionCause, usize>,
    /// Which sighting supplied the observed value of each compared field.
    ///
    /// 🔑 **This exists so a page can date a value by the sighting that carries it rather than by
    /// the row.** Since a source keeps its last value PER FIELD, the name shown for a host may
    /// come from an earlier sighting than the one that dates the row — and showing an old value
    /// with a fresh age is *two sightings in one line*, the defect story 6.4's code review found
    /// on this very screen. Empty for a field that abstained: nothing concluded, nothing to point
    /// at.
    pub observed_from: BTreeMap<String, ObsId>,
}

impl Reconciliation {
    fn abstain(&mut self, cause: AbstentionCause) {
        *self.abstentions.entry(cause).or_insert(0) += 1;
    }

    /// Total abstentions across all causes — the reach the engine could not place.
    pub fn abstention_count(&self) -> usize {
        self.abstentions.values().sum()
    }
}

/// Project an observation's facts into comparable `(field, value)` pairs — the vocabulary bridge
/// between observed [`Fact`]s and declared attribute keys. Kinds not reconciled as declared
/// fields in the walking skeleton (Rtt, DhcpLease, Uplink, OuiVendor) are ignored here.
///
/// **Two consumers, one source of truth** (story 6.2): the reconcile ([`reconcile`]) compares
/// observed against declared through these pairs, and the documenting gesture WRITES the
/// declared record through the SAME pairs. If the two drifted — a field documented under a key
/// the reconcile does not compare — the gap the operator just closed would stay open on the
/// page. It is `pub` for exactly that sharing; this is a VISIBILITY change with no behaviour
/// change. Fact order is preserved, so a caller that must pick one value per key can take the
/// first occurrence deterministically.
pub fn project(observation: &Observation) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for fact in &observation.facts {
        match fact {
            Fact::IpV4 { addr } => out.push(("ipv4".to_string(), addr.to_string())),
            Fact::Hostname { name, .. } => out.push(("hostname".to_string(), name.clone())),
            Fact::Mac { addr, .. } => out.push(("mac".to_string(), addr.to_string())),
            _ => {}
        }
    }
    out
}

/// Reconcile ONE entity (a cardinality-1 perimeter). `identity` is the `(field, value)` that
/// defines the entity (e.g. `("ipv4", "192.0.2.10")`); `declared` are its declared field/value
/// pairs; `observations` is a batch, some in perimeter and some not. Pure — no clock, no SQL.
pub fn reconcile(
    identity: (&str, &str),
    declared: &[(String, String)],
    observations: &[Observation],
) -> Reconciliation {
    let (id_field, id_value) = identity;
    let mut result = Reconciliation::default();

    // 🔴 **Per SOURCE and per FIELD, the last value that source gave for that field.** Guy's
    // arbitration, 2026-09-09, taken twice — the second time on a defect the first one caused.
    //
    // The rule below reads *two in-perimeter observations disagree* (FR16), and it was written
    // when a field could only ever come from two SOURCES. The reverse-DNS story made `hostname`
    // the first field on which one source can disagree WITH ITSELF, an hour apart — and the store
    // is append-only and never pruned, so comparing a declared value against every sighting ever
    // recorded meant that a host renamed once conflicted **for ever**.
    //
    // 🔴 **The first fix was per SOURCE — the newest SIGHTING — and all three review layers
    // measured what that costs**, one of them against a `master` binary on the identical store: a
    // sighting is not cumulative, so a single lost PTR answer WITHDREW a field the previous sweep
    // had supplied, and the page then said *"no source reported a value"* about a host it had
    // seen one minute earlier. That is the very sentence this rule was changed to abolish.
    //
    // 🔑 *A source that changes its mind is a HISTORY, not a disagreement; and a source that did
    // not learn something this time has not retracted what it said last time.* One source, one
    // current answer PER FIELD; a conflict is two SOURCES that cannot both be right, which is the
    // thing an operator has to arbitrate and the thing FR16 refuses to guess at.
    //
    // ⚠️ **The cost, stated rather than discovered**: a host that permanently loses its `PTR`
    // record keeps showing the last name it answered to. The product never fabricates absence
    // (NFR7/D35), so it cannot conclude *the name is gone* from *the name was not learnt* — and
    // [`Reconciliation::observed_from`] exists so the page can date the value by the sighting that
    // supplied it rather than by the row, which is what stops an old name wearing a fresh age.
    //
    // ⚠️ The out-of-perimeter count is unchanged: a sighting of something else is reach whether or
    // not a later sighting of it exists. Only superseded in-perimeter VALUES are set aside.
    let mut latest: BTreeMap<(String, ConnectorId), (Timestamp, ObsId, BTreeSet<String>)> =
        BTreeMap::new();
    for observation in observations {
        let projected = project(observation);
        let in_perimeter = projected
            .iter()
            .any(|(f, v)| f == id_field && v == id_value);
        if !in_perimeter {
            result.abstain(AbstentionCause::OutOfPerimeter);
            continue;
        }
        let stamp = (observation.observed_at, observation.obs_id);
        for (field, value) in projected {
            match latest.entry((field, observation.connector_id)) {
                std::collections::btree_map::Entry::Vacant(slot) => {
                    slot.insert((stamp.0, stamp.1, BTreeSet::from([value])));
                }
                std::collections::btree_map::Entry::Occupied(mut slot) => {
                    let kept = slot.get_mut();
                    match stamp.cmp(&(kept.0, kept.1)) {
                        std::cmp::Ordering::Greater => {
                            *kept = (stamp.0, stamp.1, BTreeSet::from([value]));
                        }
                        // The SAME sighting naming the field twice — one source contradicting
                        // itself in one breath, which is an ambiguity and not a history. Both
                        // values are kept so the conflict below sees them. Unreachable while a
                        // connector emits one name per host; `HostnameSource` has variants that
                        // make it reachable, and that is the next connector story's.
                        std::cmp::Ordering::Equal => {
                            kept.2.insert(value);
                        }
                        std::cmp::Ordering::Less => {}
                    }
                }
            }
        }
    }

    // Collect the observed values; flag fields two SOURCES disagree on.
    let mut observed: BTreeMap<String, String> = BTreeMap::new();
    let mut conflicting: BTreeSet<String> = BTreeSet::new();
    for ((field, _source), (instant, obs_id, values)) in &latest {
        for value in values {
            match observed.get(field) {
                Some(existing) if existing != value => {
                    conflicting.insert(field.clone());
                }
                _ => {
                    observed.insert(field.clone(), value.clone());
                    result.observed_from.insert(field.clone(), *obs_id);
                    let _ = instant;
                }
            }
        }
    }

    // A field two observations disagree on abstains — never picked, never merged (FR16).
    for field in &conflicting {
        observed.remove(field);
        // and its provenance with it: a field nothing concluded has no sighting to point at.
        result.observed_from.remove(field);
        result.abstain(AbstentionCause::ConflictingObservations);
    }

    // Compare declared against observed, field by field.
    for (field, declared_value) in declared {
        match observed.get(field) {
            Some(observed_value) if observed_value != declared_value => {
                result.gaps.push(Gap {
                    field: field.clone(),
                    declared: declared_value.clone(),
                    observed: observed_value.clone(),
                });
            }
            Some(_) => { /* clear: declared == observed */ }
            None => result.abstain(AbstentionCause::NoObservedValue),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::{
        ConnectorId, Fact, HostnameSource, L2DomainId, ObsId, Scope, Timestamp, VantageId,
    };
    use std::net::Ipv4Addr;
    use uuid::Uuid;

    fn ts() -> Timestamp {
        chrono::DateTime::from_timestamp(0, 0).unwrap()
    }

    fn obs(facts: Vec<Fact>) -> Observation {
        Observation {
            obs_id: ObsId::from_uuid(Uuid::nil()),
            connector_id: ConnectorId::from_uuid(Uuid::nil()),
            observed_at: ts(),
            scope: Scope {
                l2_domain: L2DomainId::from_uuid(Uuid::nil()),
                vantage: VantageId::from_uuid(Uuid::nil()),
            },
            facts,
            raw: None,
        }
    }

    /// The same observation, attributed to a named source and dated. Ids are derived from the two
    /// arguments so a test says *which source, when* and nothing else.
    fn seen_by(source: u128, at_secs: i64, facts: Vec<Fact>) -> Observation {
        Observation {
            obs_id: ObsId::from_uuid(Uuid::from_u128(
                u128::from(at_secs.unsigned_abs()) | (source << 64),
            )),
            connector_id: ConnectorId::from_uuid(Uuid::from_u128(source)),
            observed_at: chrono::DateTime::from_timestamp(at_secs, 0).expect("representable"),
            ..obs(facts)
        }
    }

    fn ip(a: u8, b: u8, c: u8, d: u8) -> Fact {
        Fact::IpV4 {
            addr: Ipv4Addr::new(a, b, c, d),
        }
    }

    fn host(name: &str) -> Fact {
        Fact::Hostname {
            name: name.to_string(),
            source: HostnameSource::Dns,
        }
    }

    fn declared(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(f, v)| (f.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn surfaces_exactly_the_disagreeing_field() {
        let d = declared(&[("ipv4", "192.0.2.10"), ("hostname", "nas")]);
        let obs = vec![obs(vec![ip(192, 0, 2, 10), host("intruder")])];
        let r = reconcile(("ipv4", "192.0.2.10"), &d, &obs);
        assert_eq!(
            r.gaps,
            vec![Gap {
                field: "hostname".into(),
                declared: "nas".into(),
                observed: "intruder".into(),
            }]
        );
        assert_eq!(r.abstention_count(), 0);
    }

    #[test]
    fn out_of_perimeter_observation_abstains_and_counts() {
        let d = declared(&[("ipv4", "192.0.2.10")]);
        let obs = vec![obs(vec![ip(192, 0, 2, 99), host("stranger")])];
        let r = reconcile(("ipv4", "192.0.2.10"), &d, &obs);
        assert!(r.gaps.is_empty());
        assert_eq!(
            r.abstentions.get(&AbstentionCause::OutOfPerimeter),
            Some(&1)
        );
    }

    /// TWO SOURCES that cannot both be right: the engine refuses to pick (FR16).
    ///
    /// ⚠️ **Both sightings are attributed to DIFFERENT sources, and that is now what the test is
    /// about.** It used to build them with the same (nil) connector at the same instant, which
    /// after 2026-09-09 is one source contradicting itself — a history, not a disagreement.
    #[test]
    fn conflicting_sources_abstain_never_pick() {
        let d = declared(&[("ipv4", "192.0.2.10"), ("hostname", "nas")]);
        let obs = vec![
            seen_by(0xA, 100, vec![ip(192, 0, 2, 10), host("alpha")]),
            seen_by(0xB, 100, vec![ip(192, 0, 2, 10), host("beta")]),
        ];
        let r = reconcile(("ipv4", "192.0.2.10"), &d, &obs);
        assert!(
            r.gaps.iter().all(|g| g.field != "hostname"),
            "a conflicting field must not become a gap"
        );
        assert!(
            r.abstentions
                .get(&AbstentionCause::ConflictingObservations)
                .copied()
                .unwrap_or(0)
                >= 1
        );
    }

    /// 🔴 **ONE source that changed its mind is a HISTORY, and the latest answer is the answer.**
    ///
    /// This is the case the reverse-DNS story made reachable and the reason the rule was narrowed.
    /// `observation_record` is append-only and nothing prunes it, so before 2026-09-09 a host
    /// whose PTR name changed once conflicted with its own past for ever: two queue rows, neither
    /// closable by any gesture. The renamed host must produce **one gap**, which the documenting
    /// gesture can then close.
    #[test]
    fn one_source_that_changed_its_mind_yields_a_gap_not_a_conflict() {
        let d = declared(&[("ipv4", "192.0.2.10"), ("hostname", "was-called-this")]);
        let obs = vec![
            seen_by(0xA, 100, vec![ip(192, 0, 2, 10), host("was-called-this")]),
            seen_by(0xA, 4_000, vec![ip(192, 0, 2, 10), host("renamed")]),
        ];
        let r = reconcile(("ipv4", "192.0.2.10"), &d, &obs);
        assert_eq!(
            r.gaps,
            vec![Gap {
                field: "hostname".into(),
                declared: "was-called-this".into(),
                observed: "renamed".into(),
            }],
            "the LATEST answer is what the network shows now"
        );
        assert_eq!(
            r.abstention_count(),
            0,
            "and nothing abstains: the superseded sighting is history, not reach"
        );
    }

    /// The order the rows arrive in must not decide the answer — the store's `ORDER BY` is not
    /// part of this function's contract.
    #[test]
    fn the_latest_sighting_wins_whichever_order_it_arrives_in() {
        let d = declared(&[("ipv4", "192.0.2.10"), ("hostname", "old")]);
        let early = seen_by(0xA, 100, vec![ip(192, 0, 2, 10), host("old")]);
        let late = seen_by(0xA, 4_000, vec![ip(192, 0, 2, 10), host("new")]);
        let forwards = reconcile(("ipv4", "192.0.2.10"), &d, &[early.clone(), late.clone()]);
        let backwards = reconcile(("ipv4", "192.0.2.10"), &d, &[late, early]);
        assert_eq!(forwards, backwards);
        assert_eq!(
            forwards.gaps.len(),
            1,
            "and it is the later value that shows"
        );
        assert_eq!(forwards.gaps[0].observed, "new");
    }

    /// 🔴 **A sweep that learned nothing about a field does not RETRACT what the last one learnt.**
    ///
    /// The defect all three review layers of 2026-09-09 measured, one of them against a `master`
    /// binary on the identical store. A `PTR` lookup has one attempt and a two-second timeout, and
    /// every failure — NXDOMAIN, SERVFAIL, a dropped datagram — collapses to *no name*. Under the
    /// first form of this rule the newest SIGHTING was authoritative for absence as well as for
    /// value, so one lost answer withdrew the hostname and the page said *"no source reported a
    /// value"* about a host it had seen a minute earlier.
    #[test]
    fn a_sweep_that_learned_no_name_does_not_withdraw_the_name() {
        let d = declared(&[("ipv4", "192.0.2.10"), ("hostname", "nas")]);
        let obs = vec![
            seen_by(0xA, 100, vec![ip(192, 0, 2, 10), host("nas")]),
            // the next sweep answered the ping and its PTR lookup timed out
            seen_by(0xA, 4_000, vec![ip(192, 0, 2, 10)]),
        ];
        let r = reconcile(("ipv4", "192.0.2.10"), &d, &obs);
        assert!(
            r.gaps.is_empty(),
            "the name still agrees with what was declared: {:?}",
            r.gaps
        );
        assert_eq!(
            r.abstention_count(),
            0,
            "and NOTHING abstains — a lookup that failed is not a source retracting a name: {:?}",
            r.abstentions
        );
        assert_eq!(
            r.observed_from.get("hostname"),
            Some(&ObsId::from_uuid(Uuid::from_u128(100 | (0xA << 64)))),
            "and the value is attributed to the sighting that CARRIES it — the earlier one — so \
             the page can date it by that sighting instead of by the row"
        );
    }

    /// The same shape, with the declared value now stale: the name that held must still produce
    /// the gap. Written because the test above passes for a product that compares nothing at all.
    #[test]
    fn a_name_that_held_across_a_failed_lookup_still_opens_its_gap() {
        let d = declared(&[("ipv4", "192.0.2.10"), ("hostname", "old-name")]);
        let obs = vec![
            seen_by(0xA, 100, vec![ip(192, 0, 2, 10), host("new-name")]),
            seen_by(0xA, 4_000, vec![ip(192, 0, 2, 10)]),
        ];
        let r = reconcile(("ipv4", "192.0.2.10"), &d, &obs);
        assert_eq!(
            r.gaps,
            vec![Gap {
                field: "hostname".into(),
                declared: "old-name".into(),
                observed: "new-name".into(),
            }]
        );
    }

    /// One source naming a field TWICE in one sighting is an ambiguity, not a history — it
    /// contradicts itself in one breath, and the engine refuses to pick (FR16).
    ///
    /// ⚠️ Unreachable while a connector emits one name per host. `HostnameSource` carries `Dhcp`,
    /// `Mdns` and `Netbios` beside `Dns`, so the next connector story reaches it; the edge-case
    /// layer of 2026-09-09 measured that `project` emits every hostname fact and this is what
    /// keeps that from silently picking the first.
    #[test]
    fn one_sighting_naming_a_field_twice_conflicts_rather_than_picking() {
        let d = declared(&[("ipv4", "192.0.2.10"), ("hostname", "nas")]);
        let obs = vec![seen_by(
            0xA,
            100,
            vec![ip(192, 0, 2, 10), host("dhcp-name"), host("mdns-name")],
        )];
        let r = reconcile(("ipv4", "192.0.2.10"), &d, &obs);
        assert!(r.gaps.iter().all(|g| g.field != "hostname"));
        assert_eq!(
            r.abstentions.get(&AbstentionCause::ConflictingObservations),
            Some(&1),
            "two names in one breath is a disagreement with itself: {:?}",
            r.abstentions
        );
        assert!(
            !r.observed_from.contains_key("hostname"),
            "and a field nothing concluded points at no sighting"
        );
    }

    /// A superseded sighting is set aside; a sighting of something ELSE is still reach.
    ///
    /// ⚠️ The two must not be confused: the out-of-perimeter count is what story 5.14b's section
    /// reports, and quietly shrinking it would change what the operator is told the product saw.
    ///
    /// ⚠️ **The two in-perimeter sightings DISAGREE on purpose.** The blind review layer of
    /// 2026-09-09 found this test's first form asserting nothing on its own subject: both
    /// sightings carried identical facts, so the *set aside* half passed identically against the
    /// old code, which projected every sighting. Only the out-of-perimeter half was measured.
    #[test]
    fn superseded_history_is_set_aside_but_out_of_perimeter_reach_is_not() {
        let d = declared(&[("ipv4", "192.0.2.10"), ("hostname", "current")]);
        let obs = vec![
            seen_by(0xA, 100, vec![ip(192, 0, 2, 10), host("superseded")]),
            seen_by(0xA, 200, vec![ip(192, 0, 2, 10), host("current")]),
            seen_by(0xA, 300, vec![ip(192, 0, 2, 99)]),
            seen_by(0xA, 400, vec![ip(192, 0, 2, 98)]),
        ];
        let r = reconcile(("ipv4", "192.0.2.10"), &d, &obs);
        assert!(
            r.gaps.is_empty(),
            "the older name was SET ASIDE — kept, it would conflict with the current one and \
             this assertion would red: {:?}",
            r.gaps
        );
        assert_eq!(
            r.abstentions.get(&AbstentionCause::ConflictingObservations),
            None,
            "and set aside is not the same as conflicting: {:?}",
            r.abstentions
        );
        assert_eq!(
            r.abstentions.get(&AbstentionCause::OutOfPerimeter),
            Some(&2),
            "two sightings of other addresses are two pieces of reach, superseding or not"
        );
    }

    #[test]
    fn declared_field_with_no_observation_abstains_never_fabricates() {
        let d = declared(&[("ipv4", "192.0.2.10"), ("mac", "0a:00:00:00:00:01")]);
        let obs = vec![obs(vec![ip(192, 0, 2, 10)])]; // ping-only: no mac observed
        let r = reconcile(("ipv4", "192.0.2.10"), &d, &obs);
        assert!(
            r.gaps.is_empty(),
            "no fabricated gap for an unobserved field (NFR7)"
        );
        assert_eq!(
            r.abstentions.get(&AbstentionCause::NoObservedValue),
            Some(&1)
        );
    }

    #[test]
    fn agreement_is_clear_no_gap_no_abstention() {
        let d = declared(&[("ipv4", "192.0.2.10"), ("hostname", "nas")]);
        let obs = vec![obs(vec![ip(192, 0, 2, 10), host("nas")])];
        let r = reconcile(("ipv4", "192.0.2.10"), &d, &obs);
        assert!(r.gaps.is_empty());
        assert_eq!(r.abstention_count(), 0);
    }
}
