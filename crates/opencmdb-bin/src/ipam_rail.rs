//! The rail's two lists — the view model behind the controls that CORRECT and REMOVE what the
//! operator declared (stories 14.4 and 14.5).
//!
//! 🔴 **SPLIT OUT OF `ipam_page.rs` AT STORY 14.5's CODE REVIEW, and the reason is a measured
//! ceiling rather than taste.** That file reached **1992** code lines of the 2000 the `file-size`
//! gate allows — eight of headroom — and `CLAUDE.md`'s rule is *split, not grown*, applied before
//! the growth rather than after it. Story 14.5's own §1 recorded 118 lines of headroom and its
//! Completion Notes recorded no new figure; the review read it off the gate.
//!
//! ⚠️ **The rail is what left, and it is not the biggest block that could have.** The three
//! `GET` checks that warn before a write (`address_check`, `range_check`, `delete_check` with their
//! `_data` and `render_*` halves) are ~550 lines and an equally sharp concept — *routes that warn
//! and refuse nothing*. They stayed because moving them after the review had run would ship an
//! unreviewed restructure to buy headroom a hundred lines already bought; the measurement is
//! registered so the next story choosing where to cut does not re-take it.
//!
//! 🔑 **Lists in the rail, not controls on the cells** (Guy's decision 3, 2026-09-16). A cell is
//! 14 px and already carries a state, a policy and an accessible name; two controls hung on it would
//! put the gesture where there is no room to say what it does.

use crate::ipam_page::{policy_key, tab_label};
use crate::ipam_repo;
use opencmdb_core::ipam::IpPolicy;
use std::net::IpAddr;

/// One declared range in the rail, with the id its controls name.
///
/// 🔑 **The current values travel with the row**, so the correction form is PRE-FILLED: an edit form
/// an operator must retype from scratch is a delete-and-redefine wearing another word, and it loses
/// the row's identity — which is exactly what `update_range` exists to keep.
#[derive(Debug, Clone)]
pub(crate) struct RailRangeRow {
    /// The record's id, which its controls name.
    pub(crate) id: String,
    /// The bounds as the operator reads them.
    pub(crate) bounds: String,
    /// The policy's own word, translated.
    pub(crate) policy: String,
    /// The policy's BINDING TOKEN — never translated, because the route compares it without
    /// trimming or folding.
    pub(crate) policy_token: &'static str,
    /// The first address, as the correction form pre-fills it.
    pub(crate) first: String,
    /// The last address, likewise.
    pub(crate) last: String,
    /// The operator's label.
    pub(crate) label: String,
}

/// One defined address in the rail, with the id its controls name.
#[derive(Debug, Clone)]
pub(crate) struct RailAddressRow {
    /// The record's id.
    pub(crate) id: String,
    /// The address, as the operator reads and corrects it.
    pub(crate) addr: String,
    /// The operator's label.
    pub(crate) label: String,
}

/// The rail's two lists — what the operator can correct on the subnet in force.
///
/// ⚠️ **Lists in the rail, not controls on the cells** (Guy's decision 3, 2026-09-16). A cell is
/// 14 px and already carries a state, a policy and an accessible name; two controls hung on it would
/// put the gesture where there is no room to say what it does.
#[derive(Debug, Clone)]
pub(crate) struct RailLists {
    /// The subnet in force, whose own removal the rail offers.
    pub(crate) subnet_id: String,
    /// What the operator calls it, as its correction form pre-fills it.
    pub(crate) subnet_label: String,
    /// How the selector names it — the CIDR, its VLAN and its label. 🔑 It is what the subnet's two
    /// controls say, because *"Correct — Subnet"* is byte-identical across every subnet in the plan
    /// and the keyboard gate cannot see that: it tests DISTINCTNESS and that name is distinct.
    pub(crate) subnet_name: String,
    /// Its VLAN as the form carries it — EMPTY for none, because *none* is what an empty field
    /// means on the way in (`parse_vlan`) and a `0` pre-filled here would be a number the operator
    /// never typed and cannot mean.
    pub(crate) subnet_vlan: String,
    /// Its declared ranges.
    pub(crate) ranges: Vec<RailRangeRow>,
    /// Its defined addresses.
    pub(crate) addresses: Vec<RailAddressRow>,
}

impl RailLists {
    /// Build the rail's lists from what the adapter read.
    pub(crate) fn new(
        chosen: &ipam_repo::PlannedSubnet,
        ranges: &[(String, IpAddr, IpAddr, IpPolicy, String)],
        addresses: &[(String, IpAddr, String)],
    ) -> Self {
        Self {
            subnet_id: chosen.id.clone(),
            subnet_label: chosen.label.clone(),
            subnet_name: tab_label(chosen),
            subnet_vlan: if chosen.vlan == 0 {
                String::new()
            } else {
                chosen.vlan.to_string()
            },
            ranges: ranges
                .iter()
                .map(|(id, first, last, policy, label)| RailRangeRow {
                    id: id.clone(),
                    bounds: format!("{first} – {last}"),
                    policy: rust_i18n::t!(policy_key(*policy)).to_string(),
                    policy_token: policy.as_str(),
                    first: first.to_string(),
                    last: last.to_string(),
                    label: label.clone(),
                })
                .collect(),
            addresses: addresses
                .iter()
                .map(|(id, addr, label)| RailAddressRow {
                    id: id.clone(),
                    addr: addr.to_string(),
                    label: label.clone(),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipam_repo::{PlannedSubnet, Subnet};

    fn office() -> Subnet {
        Subnet::new("192.0.2.0".parse().expect("an address"), 24).expect("a /24")
    }

    fn planned(id: &str, label: &str, vlan: u16) -> PlannedSubnet {
        PlannedSubnet {
            id: id.to_string(),
            subnet: office(),
            label: label.to_string(),
            vlan,
        }
    }

    /// 🔴 **THE SENTINEL LEAVES THE STORE AND MUST NOT REACH THE FORM.** `0010` spells *no VLAN* as
    /// 0; the correction form must spell it as an EMPTY field, because an empty field is what
    /// `parse_vlan` reads back as *none* and a pre-filled `0` is the one value the route REFUSES by
    /// name. Story 14.5's mutation M8 measured that this translation was carried by no Rust test at
    /// all — only by a browser check, and only on its present half.
    #[test]
    fn no_vlan_leaves_the_store_as_a_sentinel_and_reaches_the_form_as_an_empty_field() {
        assert_eq!(
            RailLists::new(&planned("s1", "Office", 0), &[], &[]).subnet_vlan,
            "",
            "0 is *none*, and *none* is an empty field"
        );
        assert_eq!(
            RailLists::new(&planned("s1", "Office", 10), &[], &[]).subnet_vlan,
            "10",
            "a declared VLAN travels as the number the operator typed"
        );
    }

    /// The controls name the subnet the SELECTOR's way, because *"Correct — Subnet"* is
    /// byte-identical across every subnet in the plan and the keyboard gate cannot see that: it
    /// tests DISTINCTNESS, and that name is distinct.
    #[test]
    fn the_subnets_controls_name_it_as_the_selector_does() {
        let rail = RailLists::new(&planned("s1", "Office", 10), &[], &[]);
        assert_eq!(rail.subnet_name, tab_label(&planned("s1", "Office", 10)));
        assert!(
            rail.subnet_name.contains("192.0.2.0/24") && rail.subnet_name.contains("Office"),
            "the CIDR and the label are both in it: {}",
            rail.subnet_name
        );
        assert_eq!(
            rail.subnet_label, "Office",
            "the FORM pre-fills the label alone"
        );
    }

    /// ⚠️ **An empty rail is a `Some` carrying nothing, not a `None`** — the panel, both headings,
    /// both *nothing here yet* sentences and the subnet's own two controls all render over it.
    #[test]
    fn an_empty_rail_still_names_its_subnet() {
        let rail = RailLists::new(&planned("s1", "", 0), &[], &[]);
        assert_eq!(rail.subnet_id, "s1");
        assert!(rail.ranges.is_empty() && rail.addresses.is_empty());
        assert_eq!(
            rail.subnet_name, "192.0.2.0/24",
            "no VLAN and no label leaves the CIDR, which is what the selector shows"
        );
    }
}
