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

use crate::observation::{Fact, Observation};

/// A field on which the declared value and the observed value disagree — a drift.
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

    // Collect the observed values for in-perimeter observations; flag fields two of them disagree on.
    let mut observed: BTreeMap<String, String> = BTreeMap::new();
    let mut conflicting: BTreeSet<String> = BTreeSet::new();
    for observation in observations {
        let projected = project(observation);
        let in_perimeter = projected
            .iter()
            .any(|(f, v)| f == id_field && v == id_value);
        if !in_perimeter {
            result.abstain(AbstentionCause::OutOfPerimeter);
            continue;
        }
        for (field, value) in projected {
            match observed.get(&field) {
                Some(existing) if *existing != value => {
                    conflicting.insert(field);
                }
                _ => {
                    observed.insert(field, value);
                }
            }
        }
    }

    // A field two observations disagree on abstains — never picked, never merged (FR16).
    for field in &conflicting {
        observed.remove(field);
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
    fn surfaces_exactly_the_drift_gap() {
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

    #[test]
    fn conflicting_observations_abstain_never_pick() {
        let d = declared(&[("ipv4", "192.0.2.10"), ("hostname", "nas")]);
        let obs = vec![
            obs(vec![ip(192, 0, 2, 10), host("alpha")]),
            obs(vec![ip(192, 0, 2, 10), host("beta")]),
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
