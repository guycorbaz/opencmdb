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
//! Guy's taxonomy): [`crate::l2_repo::load_current_ambiguous_pairs`] reads neither, and
//! `l2_pass`'s `the_screen_reads_engine_ambiguities_and_nothing_else` pins both. _(This read "a test in
//! `page.rs`" — the wrong file, found by story 6.14's blind review layer.)_

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
    /// `(interface_id, mac_canon, observation_id)` — EVERY latest placed observation of each interface
    /// (several when tied at one instant), and `None` for an interface with no current placement.
    pub(crate) sightings: Vec<(String, String, Option<String>)>,
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
                "l2-virtual-mac-prefix=disqualifying" => "triage.ambiguous.evidence.virtual_router",
                entry if entry.ends_with("=neutral") => continue,
                // ⚠️ Every rule THIS version ships is named above; this arm is for a token it does
                // not know — and its sentence says only that, where the first version claimed "a rule
                // this version does not describe" for rules it describes (story 6.14's blind review).
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
    // Per interface: its hardware address, and EVERY latest sighting (ties kept — see the reader).
    let mut sighting: BTreeMap<&str, (&str, Vec<&ObservedBatch>)> = BTreeMap::new();
    for (interface, mac, observation) in &input.sightings {
        let entry = sighting
            .entry(interface.as_str())
            .or_insert((mac.as_str(), Vec::new()));
        if let Some(batch) = observation.as_ref().and_then(|id| by_id.get(id)) {
            entry.1.push(batch);
        }
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
            let (mac, batches) = sighting
                .get(interface.as_str())
                .cloned()
                .unwrap_or(("", Vec::new()));
            let mut addresses: Vec<String> = Vec::new();
            let mut names: Vec<String> = Vec::new();
            for batch in &batches {
                for fact in &batch.facts {
                    match fact {
                        opencmdb_core::observation::Fact::IpV4 { addr } => {
                            let addr = addr.to_string();
                            if !addresses.contains(&addr) {
                                addresses.push(addr);
                            }
                        }
                        opencmdb_core::observation::Fact::Hostname { name, .. }
                            if !names.contains(name) =>
                        {
                            names.push(name.clone());
                        }
                        _ => {}
                    }
                }
                if short_name.is_empty() {
                    short_name = hostname_of(&batch.facts).unwrap_or_default();
                }
            }
            // 🔑 An address a candidate of TWO groups shows links to the FIRST group — groups come in
            // the order of their smallest member, so the choice is deterministic rather than whichever
            // was written last (story 6.14's review measured the last-write version as silent).
            for address in &addresses {
                address_to_group
                    .entry(address.clone())
                    .or_insert_with(|| group.id.clone());
            }
            all_addresses.extend(addresses.iter().cloned());
            let latest = batches.iter().map(|b| b.observed_at).max();
            if let Some(at) = latest {
                newest = Some(newest.map_or(at, |n| n.max(at)));
            }
            candidates.push(Candidate {
                mac: mac.to_string(),
                addresses: if addresses.is_empty() {
                    t!("triage.ambiguous.no_address").to_string()
                } else {
                    addresses.join(" · ")
                },
                names: names.join(" · "),
                freshness: latest.map_or_else(
                    || t!("meta.never_seen").to_string(),
                    |at| relative_time(now, at),
                ),
            });
        }
        let seen = newest.map_or_else(
            || t!("meta.never_seen").to_string(),
            |at| relative_time(now, at),
        );
        let kind = t!("state.ambiguous").to_string();
        // Never an empty token joined in: a candidate with no sighting has no address, and one whose
        // interface vanished has no MAC either — the review measured the first version rendering `" · "`.
        let entity = if all_addresses.is_empty() {
            candidates
                .iter()
                .map(|c| c.mac.clone())
                .filter(|mac| !mac.is_empty())
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

    /// 🔴 A member reached by a SECOND pair must not be torn from its first group. Folding the ids it
    /// was handed rather than their roots rewrites `i3`'s parent from `i1` to `i2`, and `i1` is left
    /// alone — two rows for one question. **Mutation N2 measured the out-of-order chain above GREEN
    /// under exactly that defect**: that chain never reaches one member twice. This is the input that
    /// separates the two.
    #[test]
    fn a_member_reached_twice_is_not_torn_from_its_first_group() {
        let groups = groups(&[pair("i1", "i3"), pair("i2", "i3")]);
        assert_eq!(groups.len(), 1, "{groups:?}");
        assert_eq!(groups[0].interfaces, vec!["i1", "i2", "i3"]);
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

    fn batch(id: u128, facts: Vec<opencmdb_core::observation::Fact>) -> ObservedBatch {
        ObservedBatch {
            id: opencmdb_core::observation::ObsId::from_uuid(uuid::Uuid::from_u128(id)),
            connector_id: "c".to_string(),
            observed_at: chrono::DateTime::from_timestamp(500, 0).expect("in range"),
            facts,
        }
    }

    fn ip(addr: &str) -> opencmdb_core::observation::Fact {
        opencmdb_core::observation::Fact::IpV4 {
            addr: addr.parse().expect("an address"),
        }
    }

    fn now() -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp(1_000, 0).expect("in range")
    }

    /// 🔴 An interface answering at TWO addresses in one sweep has two sightings tied at the sweep's
    /// instant, and the candidate shows BOTH — each linked to the question. The first version kept one
    /// by id; the review measured the other address's `Nouveau` row losing its link.
    #[test]
    fn an_interface_answering_at_two_addresses_shows_both() {
        let batches = vec![
            batch(1, vec![ip("192.0.2.8")]),
            batch(2, vec![ip("192.0.2.18")]),
            batch(3, vec![ip("192.0.2.9")]),
        ];
        let id = |n: u128| Some(batches[(n - 1) as usize].id.to_string());
        let input = AmbiguityInput {
            pairs: vec![pair("i1", "i2")],
            sightings: vec![
                ("i1".into(), "aa".into(), id(1)),
                ("i1".into(), "aa".into(), id(2)),
                ("i2".into(), "bb".into(), id(3)),
            ],
        };
        let (_, panes, links) = ambiguity_rows(&input, &batches, now(), false);
        assert_eq!(panes[0].1.candidates[0].addresses, "192.0.2.8 · 192.0.2.18");
        for address in ["192.0.2.8", "192.0.2.18", "192.0.2.9"] {
            assert_eq!(
                links.get(address).map(String::as_str),
                Some("ambigu:i1"),
                "{address}"
            );
        }
    }

    /// Each group shows ITS OWN pairs' verdicts. 🔴 Mutation `ma` (the filter made `true`) was green —
    /// latent while every vector is identical, wrong the day two differ.
    #[test]
    fn each_group_carries_only_its_own_verdicts() {
        let other = "l2-different-hostname=opposes;l2-hostname-agrees=supports";
        let groups = groups(&[pair("i1", "i2"), ("i7".into(), "i8".into(), other.into())]);
        assert_eq!(groups[0].verdicts, vec![AGREES.to_string()]);
        assert_eq!(groups[1].verdicts, vec![other.to_string()]);
    }

    /// A candidate whose interface has no current placement keeps its hardware address, and the row's
    /// label joins no empty token. 🔴 Measured on a server by the review: an EMPTY heading and `" · "`.
    #[test]
    fn a_candidate_with_no_placement_keeps_its_hardware_address() {
        let input = AmbiguityInput {
            pairs: vec![pair("i1", "i2")],
            sightings: vec![
                ("i1".into(), "aa".into(), None),
                ("i2".into(), "bb".into(), None),
            ],
        };
        let (rows, panes, _) = ambiguity_rows(&input, &[], now(), false);
        let macs: Vec<&str> = panes[0]
            .1
            .candidates
            .iter()
            .map(|c| c.mac.as_str())
            .collect();
        assert_eq!(macs, vec!["aa", "bb"]);
        assert_eq!(rows[0].entity, "aa · bb");
        let bare = AmbiguityInput {
            pairs: vec![pair("i1", "i2")],
            sightings: Vec::new(),
        };
        let (rows, _, _) = ambiguity_rows(&bare, &[], now(), false);
        assert!(
            !rows[0].entity.contains(" · "),
            "no empty token joined: {:?}",
            rows[0].entity
        );
    }

    /// An address two groups show links to the FIRST group, deterministically — never whichever was
    /// written last (the review found the silent overwrite).
    #[test]
    fn an_address_two_groups_show_links_to_the_first() {
        let batches = vec![
            batch(1, vec![ip("192.0.2.5")]),
            batch(2, vec![ip("192.0.2.5")]),
        ];
        let input = AmbiguityInput {
            pairs: vec![pair("i1", "i2"), pair("i7", "i8")],
            sightings: vec![
                ("i1".into(), "aa".into(), Some(batches[0].id.to_string())),
                ("i8".into(), "bb".into(), Some(batches[1].id.to_string())),
            ],
        };
        let (_, _, links) = ambiguity_rows(&input, &batches, now(), false);
        assert_eq!(
            links.get("192.0.2.5").map(String::as_str),
            Some("ambigu:i1")
        );
    }

    /// Every rule this version ships is described in words; the generic sentence is for a token it does
    /// not know. 🔴 It read "a rule this version does not describe" for rules it does (blind review).
    #[test]
    fn every_shipped_rule_is_described_by_name() {
        assert_eq!(
            evidence_sentences(&["l2-virtual-mac-prefix=disqualifying".to_string()]),
            vec![t!("triage.ambiguous.evidence.virtual_router").to_string()]
        );
        assert_eq!(
            evidence_sentences(&["l2-different-hostname=opposes".to_string()]),
            vec![t!("triage.ambiguous.evidence.different_names").to_string()]
        );
    }
}
