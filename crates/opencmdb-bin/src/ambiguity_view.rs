//! The L2 ambiguity on `/triage` — the question the engine refused to guess at, shown with its
//! candidates and their evidence (story 6.14, FR16).
//!
//! # One row per GROUP, not per pair — Guy's decision D, on a measurement
//!
//! Story 6.12 persists a decision per PAIR of interfaces. Shown per pair, three NICs sharing a name are
//! three rows and four are six — the validation measured 10 queue rows for 4 NICs — and a vendor
//! default name on 100 devices would be 4950: FR16b's *"not N failures — ONE question"* inverted. So the
//! interfaces linked by current `Ambiguous` pairs are gathered into GROUPS, and a group is one row.
//!
//! 🔑 **This is exact today, and not the transitive fusion L1 refused.** An L2 `Ambiguous` arises only
//! from `l2-hostname-agrees`, whose agreement is SET EQUALITY of names — an equivalence relation — so
//! the pairs of one group form a clique: every member is ambiguous with every other. ⚠️ The day a rule
//! whose agreement is NOT transitive can produce an `Ambiguous`, a group can join two interfaces the
//! engine never compared, and this must be re-examined — registered with that rule's story.
//!
//! # *As seen now* — Guy's decision F, and its cost said on the screen
//!
//! `l2_pair_decision` stores the verdict VECTOR, not what the rules read (story 6.12, decision F). The
//! candidates' addresses and names are therefore read from each interface's LATEST placed observation at
//! render time, and the pane says so: they may differ from the ones the decision was reached on.
//!
//! # What this module does NOT show
//!
//! An OPERATOR row (an answer, not a question) and a `NoMatch` (the software decided — case one of
//! Guy's taxonomy): [`crate::l2_repo::load_current_ambiguous_pairs`] reads neither, and a test in
//! `page.rs` pins both.

use std::collections::{BTreeMap, BTreeSet};

use rust_i18n::t;

use crate::page::{counted_fields, hostname_of};
use crate::repo::ObservedBatch;
use crate::triage_view::{
    DetailPane, Gesture, GestureView, MetaLine, QueueRow, relative_time, row_href,
};

/// What `/triage` reads to show L2 questions: the current ENGINE `Ambiguous` pairs, and for each of
/// their interfaces its hardware address and its latest placed observation.
#[derive(Debug, Clone, Default)]
pub(crate) struct AmbiguityInput {
    /// `(interface_low, interface_high, verdicts)` per current ENGINE `Ambiguous` pair.
    pub(crate) pairs: Vec<(String, String, String)>,
    /// `(interface_id, mac_canon, observation_id)` — the latest placed observation of each interface.
    pub(crate) sightings: Vec<(String, String, String)>,
}

/// One group of interfaces the engine will not say are one machine or several.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AmbiguityGroup {
    /// The queue row's id, `ambigu:{smallest interface id}` — stable while the group is.
    pub(crate) id: String,
    /// The interfaces, sorted.
    pub(crate) interfaces: Vec<String>,
    /// The distinct stored verdict vectors of the group's pairs, sorted.
    pub(crate) verdicts: Vec<String>,
}

/// Gather the pairs into groups: the connected components of the graph whose edges are the pairs.
/// Deterministic — groups are ordered by their smallest interface id.
pub(crate) fn groups(pairs: &[(String, String, String)]) -> Vec<AmbiguityGroup> {
    let mut parent: BTreeMap<String, String> = BTreeMap::new();
    fn root(parent: &BTreeMap<String, String>, id: &str) -> String {
        let mut current = id.to_string();
        while let Some(next) = parent
            .get(&current)
            .filter(|next| **next != current)
            .cloned()
        {
            current = next;
        }
        current
    }
    for (low, high, _) in pairs {
        for id in [low, high] {
            parent.entry(id.clone()).or_insert_with(|| id.clone());
        }
        let (a, b) = (root(&parent, low), root(&parent, high));
        // The smaller id becomes the root, so a group's root IS its smallest member.
        let (keep, fold) = if a <= b { (a, b) } else { (b, a) };
        parent.insert(fold, keep);
    }
    let mut members: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let ids: Vec<String> = parent.keys().cloned().collect();
    for id in ids {
        let r = root(&parent, &id);
        members.entry(r).or_default().insert(id);
    }
    members
        .into_iter()
        .map(|(smallest, interfaces)| {
            let verdicts: BTreeSet<String> = pairs
                .iter()
                .filter(|(low, _, _)| interfaces.contains(low))
                .map(|(_, _, verdicts)| verdicts.clone())
                .collect();
            AmbiguityGroup {
                id: format!("ambigu:{smallest}"),
                interfaces: interfaces.into_iter().collect(),
                verdicts: verdicts.into_iter().collect(),
            }
        })
        .collect()
}

/// One sentence per verdict that ARGUED, in the operator's language — never a raw token.
///
/// ⚠️ **Total over tokens it does not know**: `verdicts` carries only a non-empty CHECK, so an
/// unfamiliar `rule=verdict` renders a generic sentence rather than itself (story 5.14b's arbitration
/// 11: the reader never fails). `Neutral` verdicts are left out — a rule that did not speak is not
/// evidence — which is why a vector can yield fewer sentences than it has entries.
pub(crate) fn evidence_sentences(verdicts: &[String]) -> Vec<String> {
    let mut sentences: BTreeSet<String> = BTreeSet::new();
    for vector in verdicts {
        for entry in vector.split(';') {
            let key = match entry {
                "l2-hostname-agrees=supports" => "triage.ambiguous.evidence.same_name",
                "l2-different-hostname=opposes" => "triage.ambiguous.evidence.different_names",
                entry if entry.ends_with("=neutral") => continue,
                _ => "triage.ambiguous.evidence.unfamiliar",
            };
            sentences.insert(t!(key).to_string());
        }
    }
    sentences.into_iter().collect()
}

/// One candidate as the pane shows it — as seen NOW (decision F).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Candidate {
    /// The interface's hardware address.
    pub(crate) mac: String,
    /// The addresses its latest sighting carried, joined; the words for *none* otherwise.
    pub(crate) addresses: String,
    /// The names its latest sighting carried, joined; empty when it carried none.
    pub(crate) names: String,
    /// When that sighting was taken — each candidate its own freshness (story 6b.4's rule).
    pub(crate) freshness: String,
}

/// What [`ambiguity_rows`] returns: the queue rows, their panes keyed by row id, and for every address a
/// candidate currently shows, its group's row id.
pub(crate) type AmbiguityRows = (
    Vec<QueueRow>,
    Vec<(String, DetailPane)>,
    BTreeMap<String, String>,
);

/// The Ambigu rows and panes, and for every address a candidate currently shows, the row id of its
/// group — which is how a `Nouveau` row says it belongs to an open question (Guy's decision C).
pub(crate) fn ambiguity_rows(
    input: &AmbiguityInput,
    observations: &[ObservedBatch],
    now: chrono::DateTime<chrono::Utc>,
    sort_by_age: bool,
) -> AmbiguityRows {
    let by_id: BTreeMap<String, &ObservedBatch> = observations
        .iter()
        .map(|batch| (batch.id.to_string(), batch))
        .collect();
    // One sighting per interface: the first by observation id among ties (the reader orders them).
    let mut sighting: BTreeMap<&str, (&str, Option<&ObservedBatch>)> = BTreeMap::new();
    for (interface, mac, observation) in &input.sightings {
        sighting
            .entry(interface.as_str())
            .or_insert((mac.as_str(), by_id.get(observation).copied()));
    }

    let mut rows = Vec::new();
    let mut panes = Vec::new();
    let mut address_to_group = BTreeMap::new();
    for group in groups(&input.pairs) {
        let mut candidates = Vec::new();
        let mut all_addresses = Vec::new();
        let mut short_name = String::new();
        let mut newest: Option<chrono::DateTime<chrono::Utc>> = None;
        for interface in &group.interfaces {
            let (mac, batch) = sighting
                .get(interface.as_str())
                .copied()
                .unwrap_or(("", None));
            let addresses: Vec<String> = batch
                .map(|b| {
                    b.facts
                        .iter()
                        .filter_map(|fact| match fact {
                            opencmdb_core::observation::Fact::IpV4 { addr } => {
                                Some(addr.to_string())
                            }
                            _ => None,
                        })
                        .collect()
                })
                .unwrap_or_default();
            let names: Vec<String> = batch
                .map(|b| {
                    b.facts
                        .iter()
                        .filter_map(|fact| match fact {
                            opencmdb_core::observation::Fact::Hostname { name, .. } => {
                                Some(name.clone())
                            }
                            _ => None,
                        })
                        .collect()
                })
                .unwrap_or_default();
            if short_name.is_empty() {
                short_name = batch
                    .and_then(|b| hostname_of(&b.facts))
                    .unwrap_or_default();
            }
            for address in &addresses {
                address_to_group.insert(address.clone(), group.id.clone());
            }
            all_addresses.extend(addresses.iter().cloned());
            if let Some(b) = batch {
                newest = Some(newest.map_or(b.observed_at, |n| n.max(b.observed_at)));
            }
            candidates.push(Candidate {
                mac: mac.to_string(),
                addresses: if addresses.is_empty() {
                    t!("triage.ambiguous.no_address").to_string()
                } else {
                    addresses.join(" · ")
                },
                names: names.join(" · "),
                freshness: batch.map_or_else(
                    || t!("meta.never_seen").to_string(),
                    |b| relative_time(now, b.observed_at),
                ),
            });
        }
        let seen = newest.map_or_else(
            || t!("meta.never_seen").to_string(),
            |at| relative_time(now, at),
        );
        let kind = t!("state.ambiguous").to_string();
        let entity = if all_addresses.is_empty() {
            candidates
                .iter()
                .map(|c| c.mac.clone())
                .collect::<Vec<_>>()
                .join(" · ")
        } else {
            all_addresses.join(" · ")
        };
        rows.push(QueueRow {
            href: row_href(&group.id, sort_by_age),
            id: group.id.clone(),
            kind: kind.clone(),
            entity: entity.clone(),
            observed_at: newest.unwrap_or(chrono::DateTime::UNIX_EPOCH),
            name: short_name,
            field: String::new(),
            declared: String::new(),
            observed: String::new(),
            count: counted_fields("triage.ambiguous.n_candidates", candidates.len()),
            counted: true,
            seen: seen.clone(),
            age_seconds: newest.map_or(i64::MAX, |at| (now - at).num_seconds().max(0)),
            selected: false,
        });
        panes.push((
            group.id.clone(),
            DetailPane {
                subject: String::new(),
                gestures: resolve_bar(),
                kind,
                entity,
                field: String::new(),
                declared: String::new(),
                declared_meta: MetaLine {
                    source: String::new(),
                    freshness: String::new(),
                },
                observed: String::new(),
                observed_meta: MetaLine {
                    source: String::new(),
                    freshness: seen,
                },
                candidates,
                evidence: evidence_sentences(&group.verdicts),
                open_question: None,
                not_built: t!("gesture.not_built_resolve", badge = t!("gesture.badge")).to_string(),
            },
        ));
    }
    (rows, panes, address_to_group)
}

/// The action bar of an Ambigu pane: *Résoudre* ALONE, labelled and not acting (Guy's decisions A and
/// E). Epic 7's four gap gestures answer a GAP and not a doubt, so they are not offered here; what an
/// answer writes is story 6.14b's, which is why the control is `Planned`.
pub(crate) fn resolve_bar() -> Vec<GestureView> {
    vec![GestureView::of(
        Gesture::Planned { owner: "6.14b" },
        t!("gesture.resolve").to_string(),
    )]
}

/// Tests for the grouping and the evidence sentences — pure; the rendered pane is tested in `page.rs`.
#[cfg(test)]
mod tests {
    use super::*;

    const AGREES: &str =
        "l2-different-hostname=neutral;l2-hostname-agrees=supports;l2-virtual-mac-prefix=neutral";

    fn pair(a: &str, b: &str) -> (String, String, String) {
        (a.to_string(), b.to_string(), AGREES.to_string())
    }

    /// Decision D: three NICs sharing a name are THREE pairs and ONE question. The validation measured
    /// the per-pair shape at 10 rows for 4 NICs.
    #[test]
    fn a_clique_of_three_interfaces_is_one_group() {
        let groups = groups(&[pair("i1", "i2"), pair("i1", "i3"), pair("i2", "i3")]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].interfaces, vec!["i1", "i2", "i3"]);
        assert_eq!(
            groups[0].id, "ambigu:i1",
            "the id is the smallest member's — stable"
        );
        assert_eq!(groups[0].verdicts, vec![AGREES.to_string()]);
    }

    /// Two unrelated questions stay two, in a deterministic order.
    #[test]
    fn two_disjoint_pairs_are_two_groups_ordered_by_their_smallest_member() {
        let groups = groups(&[pair("i7", "i9"), pair("i2", "i4")]);
        let ids: Vec<&str> = groups.iter().map(|g| g.id.as_str()).collect();
        assert_eq!(ids, vec!["ambigu:i2", "ambigu:i7"]);
    }

    /// A chain whose pairs arrive in an order that folds a root under a later one still lands in ONE
    /// group rooted at its smallest member — the union must follow roots, not the ids it was handed.
    #[test]
    fn a_chain_arriving_out_of_order_is_still_one_group() {
        let groups = groups(&[pair("i3", "i4"), pair("i1", "i2"), pair("i2", "i3")]);
        assert_eq!(groups.len(), 1, "{groups:?}");
        assert_eq!(groups[0].interfaces, vec!["i1", "i2", "i3", "i4"]);
    }

    /// The sentences are TOTAL: an unfamiliar `rule=verdict` renders a generic sentence, never itself,
    /// and a `Neutral` — a rule that did not speak — renders nothing.
    #[test]
    fn the_evidence_is_said_in_words_and_never_as_a_token() {
        let sentences = evidence_sentences(&[AGREES.to_string()]);
        assert_eq!(
            sentences,
            vec![t!("triage.ambiguous.evidence.same_name").to_string()],
            "the one verdict that argued, and no neutral one"
        );
        let unknown = evidence_sentences(&["l2-future-rule=supports".to_string()]);
        assert_eq!(
            unknown,
            vec![t!("triage.ambiguous.evidence.unfamiliar").to_string()]
        );
        assert!(unknown.iter().all(|s| !s.contains("l2-")), "{unknown:?}");
    }

    /// Three NICs sharing a name are ONE queue row carrying three candidates.
    #[test]
    fn three_nics_sharing_a_name_are_one_row_with_three_candidates() {
        let input = AmbiguityInput {
            pairs: vec![pair("i1", "i2"), pair("i1", "i3"), pair("i2", "i3")],
            sightings: Vec::new(),
        };
        let (rows, panes, _) = ambiguity_rows(
            &input,
            &[],
            chrono::DateTime::from_timestamp(1_000, 0).expect("in range"),
            false,
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(panes[0].1.candidates.len(), 3);
        assert!(rows[0].count.contains('3'), "{}", rows[0].count);
    }
}
