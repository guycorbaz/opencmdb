//! The audit of the addressing plan against the network (story 14.3b) — and the address the
//! product must not offer.
//!
//! 🔑 **The reason is the deliverable, not the highlight** (`epics.md:2343`): the *next free address*
//! panel proposed an address because no RECORD claimed it. It now excludes every address the network
//! has been SEEN on, every address DOCUMENTED in `declared_attribute`, and every address outside a
//! `static` range — the only place the product PREVENTS a duplicate rather than reporting one.
//!
//! # 🔴 The one module of the plan allowed to read the network
//!
//! The plan's guard (`ipam_page::tests::the_plan_reads_the_network_only_through_the_audit`) forbids
//! every other `ipam_*` module from naming the sighting reader or the documented read, and forbids
//! all of them — this one included — from naming `identity_link` or writing SQL against a table
//! outside the plan. Guy's decision 6 (2026-09-15): the guard is NARROWED to this module, not retired.
//!
//! # The rules, Guy's decisions of 2026-09-15, each written where it is applied
//!
//! 1. **`gap` vs `undeclared` — by the OFFER.** An observed address the plan WOULD offer is a `gap`;
//!    every other observed address without an address row is `undeclared`; nothing covered ONLY by
//!    `dhcp-pool` ranges is a finding. ⚠️ *Only* is load-bearing and the first draft of this line
//!    left it out: an address a `static` range ALSO covers is judged by decision 2 like any other —
//!    the pool does not silence it, it makes it `undeclared`. The code has always done this; the
//!    sentence promised the opposite, which is the shape three reviews have caught in a doc.
//! 2. **The most protective range decides**, across overlapping ranges and nested subnets.
//! 3. **The offer draws from `static` ranges only.**
//! 4. **A documented address is never offered**, even never observed.
//! 5. The sightings are story 14.3a's summary, **merged across L2 domains here**.
//! 10. **« Conflit d'adresse »**: two hardware addresses on one IPv4 inside a `static` or `reserved`
//!     range — and a sighting without a MAC is not a second one.
//! 11. **Observed addresses outside every subnet** are listed plan-wide.
//! 13. **An address defined inside a `dhcp-pool`** carries a warning, not a conflict.
//!
//! 🔑 **Everything below the reads is PURE**: no clock, no store, no locale. An audit derives from
//! RECORDED sightings only (NFR8(a), FR19), so a blind scanner cannot add a finding — `no sighting,
//! no finding` is a test, and the derivation takes no instant to read.

use std::collections::{BTreeMap, BTreeSet};
use std::net::Ipv4Addr;

use opencmdb_core::ipam::IpPolicy;
use opencmdb_core::observation::{MacAddr, Timestamp};

use crate::ipam_repo::Subnet;
use crate::sighting_repo::Sighting;

/// Everything the network says about one address, merged across L2 domains.
///
/// ⚠️ **A sighting without a hardware address is kept apart** (`without_mac`), because story 14.3a's
/// summary carries it as its own row beside the MAC rows of the same address: counting it as a second
/// hardware address would turn every stale neighbour entry into a conflict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Seen {
    /// The address.
    pub(crate) addr: Ipv4Addr,
    /// Each hardware address seen on it, with the LAST instant it was seen (decision 9).
    pub(crate) macs: BTreeMap<MacAddr, Timestamp>,
    /// The last instant it was seen with no hardware address, if ever.
    pub(crate) without_mac: Option<Timestamp>,
}

impl Seen {
    /// The last instant this address was seen at all, whatever hardware address answered.
    ///
    /// 🔑 It is what ORDERS the two bounded lists (Guy's decision of 2026-09-15 at the code review):
    /// the three hardware addresses a cell's name carries, and the twenty addresses the outside list
    /// shows. A bound has to keep SOMETHING, and *most recently seen* is the half an operator acts
    /// on — an address last seen a year ago is the one they can most afford not to read today.
    ///
    /// `None` is unrepresentable in practice — a [`Seen`] exists because a row exists — and is
    /// returned rather than asserted, because an empty one would otherwise panic a render.
    pub(crate) fn last_seen(&self) -> Option<Timestamp> {
        self.macs.values().copied().chain(self.without_mac).max()
    }
}

/// Merge the summary's rows into one [`Seen`] per address, across L2 domains (decision 5).
///
/// The same MAC seen in two L2 domains is ONE hardware address, with the later of its two instants.
///
/// 🔴 **AND TWO DIFFERENT MACHINES ON ONE ADDRESS IN TWO L2 DOMAINS BECOME A CONFLICT HERE, which
/// decision 5 buys and nobody had written down.** `192.168.1.10` is the ordinary address of a
/// machine on every private network there is: two of them, in two separate broadcast domains, are
/// two hardware addresses on one IPv4 and are reported as « Conflit d'adresse » by
/// [`Plan::is_conflict`] — *reused private space read as a duplicate*. It is the price of decision 5,
/// which merges because the PLAN has no VLAN axis (FR21's VLAN half is Epic 14 scope and outside the
/// arbitrations), so an address is one address here whatever domain saw it.
///
/// ⚠️ **Not reachable on the shipped product and tested anyway**: the connector reports one
/// `l2_domain`, the nil UUID, so the second domain needs a second connector. The day FR21 lands, the
/// plan gains a VLAN and this merge is what must be revisited — registered rather than left to be
/// rediscovered by whoever meets the finding on a real network.
pub(crate) fn merge_sightings(sightings: &[Sighting]) -> BTreeMap<Ipv4Addr, Seen> {
    let mut seen: BTreeMap<Ipv4Addr, Seen> = BTreeMap::new();
    for sighting in sightings {
        let entry = seen.entry(sighting.addr).or_insert_with(|| Seen {
            addr: sighting.addr,
            macs: BTreeMap::new(),
            without_mac: None,
        });
        match sighting.mac {
            Some(mac) => {
                let last = entry.macs.entry(mac).or_insert(sighting.last_seen_at);
                *last = (*last).max(sighting.last_seen_at);
            }
            None => {
                entry.without_mac = Some(
                    entry
                        .without_mac
                        .map_or(sighting.last_seen_at, |at| at.max(sighting.last_seen_at)),
                );
            }
        }
    }
    seen
}

/// Forget what the network showed on each RELEASED address up to the instant of its release (story
/// 14.4b, Guy's decision 1 of 2026-09-19).
///
/// 🔑 **The release LAPSES by construction, never by a rule someone must remember**: a hardware
/// address seen AFTER the release keeps its sighting, so the address is held again — the network
/// answering again outranks the operator's word, because an address something answers on must never
/// be offered. A released address whose every sighting predates the release leaves the map entirely,
/// which is what makes it stop being a finding AND become offerable, in one act.
///
/// 🔑 **Per hardware address, not per address**: a NIC replaced after the release is a new MAC seen
/// later — it holds the address — while the old NIC's sighting stays forgotten, so a « Conflit
/// d'adresse » between the old and the new card cannot be manufactured by the release. This is the
/// case the refused *mark `address_sighting`* shape lost (`0009`'s header), answered here on the
/// merged view instead of on a summary row.
///
/// ⚠️ **An instant EQUAL to the release is forgotten**: the operator released what they saw, and what
/// they saw includes the sighting at that instant.
///
/// 🔴 **Applied ONCE, in [`read_the_network`], so every consumer agrees by construction**: the audit,
/// the offer, the grid's seen marker and the three warnings before a write all read `Network::seen`.
/// Applying it at each consumer would make agreement a discipline; here it is a property of the only
/// door.
pub(crate) fn forget_released(
    mut seen: BTreeMap<Ipv4Addr, Seen>,
    released: &BTreeMap<Ipv4Addr, Timestamp>,
) -> BTreeMap<Ipv4Addr, Seen> {
    for (addr, at) in released {
        let Some(entry) = seen.get_mut(addr) else {
            continue;
        };
        entry.macs.retain(|_, last| *last > *at);
        entry.without_mac = entry.without_mac.filter(|last| *last > *at);
        if entry.macs.is_empty() && entry.without_mac.is_none() {
            seen.remove(addr);
        }
    }
    seen
}

/// The whole plan: every subnet, every range and every defined address, in any subnet.
///
/// 🔑 **Plan-wide and not per subnet, because decision 2 is plan-wide**: a nested subnet's `reserved`
/// range protects an address the outer subnet's `static` range would otherwise offer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Plan {
    /// Every subnet the plan holds.
    pub(crate) subnets: Vec<Subnet>,
    /// Every range, in any subnet, as `(first, last, policy)`.
    pub(crate) ranges: Vec<(Ipv4Addr, Ipv4Addr, IpPolicy)>,
    /// Every address an `ip_address` row names, in any subnet.
    pub(crate) defined: BTreeSet<Ipv4Addr>,
}

/// What an observed address is, as the audit words it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FindingKind {
    /// Observed, and the PLAN would offer it: the plan says free, the network says occupied — the
    /// core object, seen on an address instead of on a field (`gap`, « écart »).
    ///
    /// ⚠️ **The plan ALONE, which is not the same predicate as [`Plan::offerable`]** — and the first
    /// draft of this doc conflated them. A DOCUMENTED address the network has been seen on is a
    /// `gap` here and is never offered (decision 4): the two answer different questions, *does the
    /// plan say this address is free* and *may we hand it out*. Reading this as *"the product would
    /// offer it"* makes the documented case read as a contradiction when it is the ordinary one.
    Gap,
    /// Observed, no address row, and not an address the plan would offer — in a `reserved` or
    /// `infrastructure` range, on an edge, or in space no range covers (`undeclared`).
    Undeclared,
}

/// One observed address inside a subnet that the audit has something to say about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AddressFinding {
    /// What the network says about it.
    pub(crate) seen: Seen,
    /// Its verdict, or `None` for a defined address that is only a conflict.
    pub(crate) kind: Option<FindingKind>,
    /// Whether two hardware addresses were seen on it inside a `static` or `reserved` range.
    pub(crate) conflict: bool,
    /// Whether a declared record CLAIMS it, **compared the way triage compares it** — which is what
    /// says whether triage has a `nouveau:` row to link to.
    ///
    /// 🔴 **This was the PARSED set until the code review, and the two disagree on an ordinary
    /// value.** `page.rs:1156` filters the queue with `claimed.contains(&value)` over the declared
    /// `attr_value` STRINGS, verbatim; [`documented_addresses`] parses them, so a value stored as
    /// `" 192.0.2.12 "` documents the address for the OFFER and claims nothing for TRIAGE. Measured
    /// by the review's edge layer: the finding said *"documented, so triage asks nothing"* while
    /// triage was asking. 🔑 *Two questions, two comparisons, and the link must use the comparison of
    /// the screen it links to* — the offer keeps the protective parse, which is the direction where
    /// a false positive costs a duplicate address.
    pub(crate) claimed: bool,
}

impl Plan {
    /// The policies of every range covering `addr`, in any subnet.
    fn covering(&self, addr: Ipv4Addr) -> impl Iterator<Item = IpPolicy> + '_ {
        self.ranges
            .iter()
            .filter(move |(first, last, _)| addr >= *first && addr <= *last)
            .map(|(_, _, policy)| *policy)
    }

    /// Whether any subnet of the plan contains `addr`.
    pub(crate) fn contains(&self, addr: Ipv4Addr) -> bool {
        self.subnets.iter().any(|subnet| subnet.contains(addr))
    }

    /// Whether `addr` is the network or broadcast address of ANY subnet containing it.
    fn is_edge(&self, addr: Ipv4Addr) -> bool {
        self.subnets
            .iter()
            .any(|subnet| subnet.contains(addr) && subnet.is_edge(addr))
    }

    /// Whether the PLAN alone would offer `addr` (decisions 2 and 3): inside a subnet, not an edge,
    /// not defined, covered by at least one range, and every covering range `static`.
    fn plan_would_offer(&self, addr: Ipv4Addr) -> bool {
        let mut covered = false;
        for policy in self.covering(addr) {
            if policy != IpPolicy::Static {
                return false;
            }
            covered = true;
        }
        covered && self.contains(addr) && !self.is_edge(addr) && !self.defined.contains(&addr)
    }

    /// Whether `addr` may be offered as the next free address: the plan would offer it, the network
    /// has never been seen on it, and no declared record documents it (decisions 3 and 4).
    pub(crate) fn offerable(
        &self,
        addr: Ipv4Addr,
        seen: &BTreeMap<Ipv4Addr, Seen>,
        documented: &BTreeSet<Ipv4Addr>,
    ) -> bool {
        self.plan_would_offer(addr) && !seen.contains_key(&addr) && !documented.contains(&addr)
    }

    /// The verdict on an address the network has been seen on, inside the plan (decisions 1 and 2).
    fn verdict(&self, addr: Ipv4Addr) -> Option<FindingKind> {
        if self.defined.contains(&addr) {
            return None;
        }
        let policies: Vec<IpPolicy> = self.covering(addr).collect();
        if !policies.is_empty() && policies.iter().all(|p| *p == IpPolicy::DhcpPool) {
            // Decision 1: nothing covered ONLY by pools is a finding — the occupant changes by
            // design. An overlap with a `static` or `reserved` range falls through to decision 2.
            return None;
        }
        if self.plan_would_offer(addr) {
            Some(FindingKind::Gap)
        } else {
            Some(FindingKind::Undeclared)
        }
    }

    /// Whether two hardware addresses on one IPv4 are « Conflit d'adresse » (decision 10): only
    /// inside a `static` or `reserved` range — the most protective range deciding.
    fn is_conflict(&self, seen: &Seen) -> bool {
        seen.macs.len() >= 2
            && self
                .covering(seen.addr)
                .any(|p| matches!(p, IpPolicy::Static | IpPolicy::Reserved))
    }

    /// The audit of one subnet: every observed address inside it that is a finding or a conflict,
    /// in numeric order.
    /// `claimed` is the DECLARED VALUES AS STRINGS, because that is what triage compares (see
    /// [`AddressFinding::claimed`]); the offer's `documented` set is parsed and is a different
    /// question.
    pub(crate) fn audit(
        &self,
        subnet: Subnet,
        seen: &BTreeMap<Ipv4Addr, Seen>,
        claimed: &BTreeSet<String>,
    ) -> Vec<AddressFinding> {
        seen.values()
            .filter(|s| subnet.contains(s.addr))
            .filter_map(|s| {
                let kind = self.verdict(s.addr);
                let conflict = self.is_conflict(s);
                (kind.is_some() || conflict).then(|| AddressFinding {
                    seen: s.clone(),
                    kind,
                    conflict,
                    claimed: claimed.contains(&s.addr.to_string()),
                })
            })
            .collect()
    }

    /// Every observed address outside every subnet of the plan (decision 11), in numeric order.
    pub(crate) fn outside<'a>(&self, seen: &'a BTreeMap<Ipv4Addr, Seen>) -> Vec<&'a Seen> {
        seen.values().filter(|s| !self.contains(s.addr)).collect()
    }

    /// The addresses of `subnet` defined inside a `dhcp-pool` range (decision 13): a warning, never a
    /// conflict, and the write that made them still succeeds.
    pub(crate) fn defined_inside_a_pool(&self, subnet: Subnet) -> Vec<Ipv4Addr> {
        self.defined
            .iter()
            .copied()
            .filter(|addr| subnet.contains(*addr))
            .filter(|addr| self.covering(*addr).any(|p| p == IpPolicy::DhcpPool))
            .collect()
    }

    /// Whether a `dhcp-pool` range covers `addr`, in any subnet — the address check's warning that
    /// the pool may hand the address to another machine (decision 13).
    pub(crate) fn covered_by_a_pool(&self, addr: Ipv4Addr) -> bool {
        self.covering(addr).any(|p| p == IpPolicy::DhcpPool)
    }

    /// Whether an `ip_address` row names `addr`, in any subnet.
    pub(crate) fn defines(&self, addr: Ipv4Addr) -> bool {
        self.defined.contains(&addr)
    }

    /// Whether any `static` range reaches into `subnet` at all.
    ///
    /// 🔴 **IT IS THE DIFFERENCE BETWEEN TWO SENTENCES THE SCREEN USED TO CONFUSE**, found by the
    /// review's acceptance layer: *"every address a static range holds here is defined, documented,
    /// already seen, or an edge"* is VACUOUS over a subnet with no static range — it announces an
    /// exhausted offer where the truth is that nothing was ever on offer — and it is FALSE when a
    /// `reserved` range laid over the static one is what emptied it. Two states, two sentences.
    ///
    /// ⚠️ It answers *does a static range REACH this subnet*, not *does one offer anything*: an
    /// overlapping `reserved` range can empty the offer of a subnet that has one, which is exactly
    /// the case the first sentence is for.
    pub(crate) fn has_static_range(&self, subnet: Subnet) -> bool {
        self.ranges.iter().any(|(first, last, policy)| {
            *policy == IpPolicy::Static && subnet.overlaps(*first, *last)
        })
    }

    /// How many observed addresses would fall inside the closed interval `first..=last`.
    ///
    /// The range form's warning before the write (Guy, 2026-09-15): *n addresses seen would fall in
    /// this static range*. It refuses nothing — the write is untouched, like the address form's.
    pub(crate) fn seen_inside(
        seen: &BTreeMap<Ipv4Addr, Seen>,
        first: Ipv4Addr,
        last: Ipv4Addr,
    ) -> usize {
        seen.keys()
            .filter(|addr| **addr >= first && **addr <= last)
            .count()
    }

    /// The lowest address of `subnet` that may be offered, or `None`.
    pub(crate) fn next_offerable(
        &self,
        subnet: Subnet,
        seen: &BTreeMap<Ipv4Addr, Seen>,
        documented: &BTreeSet<Ipv4Addr>,
    ) -> Option<Ipv4Addr> {
        subnet
            .addresses()
            .find(|addr| self.offerable(*addr, seen, documented))
    }
}

/// Read what the network and the declared register say, for the audit: the sightings merged per
/// address, and the documented addresses.
///
/// 🔴 **The only place in the plan's modules that reaches outside the plan's tables.** The guard
/// names the two readers called here and refuses them in every other `ipam_*` module.
///
/// # Errors
///
/// A backend failure, classified.
/// It answers THREE things, and the third exists because two of them are the same fact compared two
/// ways: the sightings, the documented ADDRESSES (parsed, for the offer) and the claimed VALUES
/// (verbatim, for the triage link) — see [`AddressFinding::claimed`].
///
/// 🔑 **Since story 14.4b the sightings arrive with the operator's releases already applied**
/// ([`forget_released`]). `address_release` is a PLAN table, so reading it here breaks none of the
/// guard's rules — and applying it here, the only door, is what makes every reader agree.
pub(crate) async fn read_the_network(
    pool: &sqlx::MySqlPool,
) -> Result<Network, opencmdb_core::repo::RepositoryError> {
    let sightings = crate::sighting_repo::load_sightings(pool)
        .await
        .map_err(crate::repo::classify)?;
    let values = crate::repo::load_documented_ipv4s(pool)
        .await
        .map_err(crate::repo::classify)?;
    let released = crate::ipam_repo::plan_releases(pool).await?;
    Ok(Network {
        seen: forget_released(merge_sightings(&sightings), &released),
        documented: documented_addresses(&values),
        claimed: values.into_iter().collect(),
    })
}

/// What the network and the declared register say, as the audit needs it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Network {
    /// What the network has shown, merged per address.
    pub(crate) seen: BTreeMap<Ipv4Addr, Seen>,
    /// The documented addresses, PARSED — the offer's question (decision 4).
    pub(crate) documented: BTreeSet<Ipv4Addr>,
    /// The declared `ipv4` values VERBATIM — triage's own comparison, and the only thing that says
    /// whether triage has a question to link to.
    pub(crate) claimed: BTreeSet<String>,
}

/// Parse `declared_attribute.ipv4` values into addresses, skipping what does not parse — a declared
/// value is operator text, and one that is not an address documents no address.
pub(crate) fn documented_addresses(values: &[String]) -> BTreeSet<Ipv4Addr> {
    values
        .iter()
        .filter_map(|value| value.trim().parse::<Ipv4Addr>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencmdb_core::observation::L2DomainId;

    fn v4(text: &str) -> Ipv4Addr {
        text.parse().expect("a v4 address")
    }

    fn at(text: &str) -> Timestamp {
        chrono::DateTime::parse_from_rfc3339(text)
            .expect("an instant")
            .with_timezone(&chrono::Utc)
    }

    fn mac(last: u8) -> MacAddr {
        MacAddr([0x02, 0, 0, 0, 0, last])
    }

    fn sighting(addr: &str, domain: u128, mac_octet: Option<u8>, last: &str) -> Sighting {
        Sighting {
            addr: v4(addr),
            l2_domain: L2DomainId::from_uuid(uuid::Uuid::from_u128(domain)),
            mac: mac_octet.map(mac),
            first_seen_at: at("2026-01-01T00:00:00Z"),
            last_seen_at: at(last),
        }
    }

    fn subnet(base: &str, prefix: u8) -> Subnet {
        Subnet::new(v4(base), prefix).expect("a subnet")
    }

    fn office() -> Subnet {
        subnet("192.0.2.0", 24)
    }

    /// Office: `static` .1–.40, `dhcp-pool` .80–.126, `reserved` .130–.150, `infrastructure`
    /// .200–.210, defined .9; .41–.79 covered by nothing.
    fn plan() -> Plan {
        Plan {
            subnets: vec![office()],
            ranges: vec![
                (v4("192.0.2.1"), v4("192.0.2.40"), IpPolicy::Static),
                (v4("192.0.2.80"), v4("192.0.2.126"), IpPolicy::DhcpPool),
                (v4("192.0.2.130"), v4("192.0.2.150"), IpPolicy::Reserved),
                (
                    v4("192.0.2.200"),
                    v4("192.0.2.210"),
                    IpPolicy::Infrastructure,
                ),
            ],
            defined: BTreeSet::from([v4("192.0.2.9")]),
        }
    }

    fn seen_at(addrs: &[&str]) -> BTreeMap<Ipv4Addr, Seen> {
        let sightings: Vec<Sighting> = addrs
            .iter()
            .map(|addr| sighting(addr, 1, Some(1), "2026-09-01T10:00:00Z"))
            .collect();
        merge_sightings(&sightings)
    }

    fn kinds(findings: &[AddressFinding]) -> Vec<(Ipv4Addr, Option<FindingKind>, bool)> {
        findings
            .iter()
            .map(|f| (f.seen.addr, f.kind, f.conflict))
            .collect()
    }

    /// AC11 — the audit derives from recorded sightings only: no sighting, no finding, and no offer
    /// withheld on the network's account.
    #[test]
    fn no_sighting_means_no_finding() {
        let plan = plan();
        let none = BTreeMap::new();
        assert!(plan.audit(office(), &none, &BTreeSet::new()).is_empty());
        assert!(plan.outside(&none).is_empty());
        assert_eq!(
            plan.next_offerable(office(), &none, &BTreeSet::new()),
            Some(v4("192.0.2.1"))
        );
    }

    /// Decision 5 — one address, merged across L2 domains: the same MAC in two domains is one
    /// hardware address with its later instant, and a sighting without a MAC is kept apart.
    #[test]
    fn sightings_merge_across_domains_and_keep_the_no_mac_row_apart() {
        let merged = merge_sightings(&[
            sighting("192.0.2.20", 1, Some(1), "2026-09-01T10:00:00Z"),
            sighting("192.0.2.20", 2, Some(1), "2026-09-03T10:00:00Z"),
            sighting("192.0.2.20", 1, None, "2026-09-02T10:00:00Z"),
        ]);
        let seen = &merged[&v4("192.0.2.20")];
        assert_eq!(seen.macs.len(), 1, "one MAC in two domains is one MAC");
        assert_eq!(seen.macs[&mac(1)], at("2026-09-03T10:00:00Z"));
        assert_eq!(seen.without_mac, Some(at("2026-09-02T10:00:00Z")));
    }

    /// AC1/AC2 — decision 1 across every kind of space the plan has.
    #[test]
    fn the_verdict_is_decided_by_the_offer() {
        let plan = plan();
        let seen = seen_at(&[
            "192.0.2.20",  // static, free → the plan would offer it → gap
            "192.0.2.9",   // defined → a held cell, no finding
            "192.0.2.50",  // covered by nothing → undeclared
            "192.0.2.99",  // dhcp-pool → never a finding
            "192.0.2.140", // reserved → undeclared
            "192.0.2.205", // infrastructure range → undeclared
            "192.0.2.255", // the broadcast edge → undeclared
        ]);
        assert_eq!(
            kinds(&plan.audit(office(), &seen, &BTreeSet::new())),
            vec![
                (v4("192.0.2.20"), Some(FindingKind::Gap), false),
                (v4("192.0.2.50"), Some(FindingKind::Undeclared), false),
                (v4("192.0.2.140"), Some(FindingKind::Undeclared), false),
                (v4("192.0.2.205"), Some(FindingKind::Undeclared), false),
                (v4("192.0.2.255"), Some(FindingKind::Undeclared), false),
            ]
        );
    }

    /// Decision 2 — the most protective range decides, across OVERLAPPING ranges and NESTED subnets.
    #[test]
    fn the_most_protective_range_decides_across_overlaps_and_nested_subnets() {
        let nested = subnet("192.0.2.0", 26);
        let plan = Plan {
            subnets: vec![office(), nested],
            ranges: vec![
                (v4("192.0.2.1"), v4("192.0.2.254"), IpPolicy::Static),
                // A nested subnet's reserved range over part of the outer static one.
                (v4("192.0.2.10"), v4("192.0.2.19"), IpPolicy::Reserved),
                // An overlapping dhcp-pool over part of the static one.
                (v4("192.0.2.100"), v4("192.0.2.109"), IpPolicy::DhcpPool),
            ],
            defined: BTreeSet::new(),
        };
        let seen = seen_at(&["192.0.2.12", "192.0.2.105", "192.0.2.200"]);
        assert_eq!(
            kinds(&plan.audit(office(), &seen, &BTreeSet::new())),
            vec![
                (v4("192.0.2.12"), Some(FindingKind::Undeclared), false),
                (v4("192.0.2.105"), Some(FindingKind::Undeclared), false),
                (v4("192.0.2.200"), Some(FindingKind::Gap), false),
            ],
            "a reserved or dhcp-pool range laid over a static one takes the address out of the offer, \
             so a sighting there is undeclared, not a gap"
        );
        // And the edge of the NESTED subnet is an edge too: .63 is the /26's broadcast address.
        assert!(!plan.offerable(v4("192.0.2.63"), &BTreeMap::new(), &BTreeSet::new()));
        assert!(plan.offerable(v4("192.0.2.64"), &BTreeMap::new(), &BTreeSet::new()));
    }

    /// Decision 11 — observed addresses outside every subnet are listed plan-wide.
    #[test]
    fn an_address_outside_every_subnet_is_listed_plan_wide() {
        let plan = plan();
        let seen = seen_at(&["10.9.9.9", "192.0.2.20"]);
        let outside: Vec<Ipv4Addr> = plan.outside(&seen).iter().map(|s| s.addr).collect();
        assert_eq!(outside, vec![v4("10.9.9.9")]);
        assert!(
            plan.audit(office(), &seen, &BTreeSet::new())
                .iter()
                .all(|f| f.seen.addr != v4("10.9.9.9")),
            "and it is not a finding of a subnet it is not in"
        );
    }

    /// AC5 — « Conflit d'adresse » only inside a `static` or `reserved` range.
    #[test]
    fn two_mac_addresses_conflict_only_inside_static_or_reserved() {
        let plan = plan();
        let two = |addr: &str| {
            vec![
                sighting(addr, 1, Some(1), "2026-09-01T10:00:00Z"),
                sighting(addr, 1, Some(2), "2026-09-02T10:00:00Z"),
            ]
        };
        let mut sightings = Vec::new();
        for addr in [
            "192.0.2.20",
            "192.0.2.140",
            "192.0.2.99",
            "192.0.2.50",
            "192.0.2.9",
        ] {
            sightings.extend(two(addr));
        }
        // The same MAC twice (two domains) and a MAC beside a no-MAC row are NOT two MACs.
        sightings.push(sighting("192.0.2.21", 1, Some(3), "2026-09-01T10:00:00Z"));
        sightings.push(sighting("192.0.2.21", 2, Some(3), "2026-09-02T10:00:00Z"));
        sightings.push(sighting("192.0.2.22", 1, Some(4), "2026-09-01T10:00:00Z"));
        sightings.push(sighting("192.0.2.22", 1, None, "2026-09-02T10:00:00Z"));
        let seen = merge_sightings(&sightings);
        let conflicts: Vec<Ipv4Addr> = plan
            .audit(office(), &seen, &BTreeSet::new())
            .iter()
            .filter(|f| f.conflict)
            .map(|f| f.seen.addr)
            .collect();
        assert_eq!(
            conflicts,
            vec![v4("192.0.2.9"), v4("192.0.2.20"), v4("192.0.2.140")],
            "static (defined or not) and reserved conflict; a dhcp-pool, uncovered space, the same \
             MAC twice and a MAC beside a no-MAC sighting do not"
        );
        let defined = plan
            .audit(office(), &seen, &BTreeSet::new())
            .into_iter()
            .find(|f| f.seen.addr == v4("192.0.2.9"))
            .expect("a defined address with two MACs is reported");
        assert_eq!(
            defined.kind, None,
            "a held cell is no gap and no undeclared"
        );
    }

    /// AC3 — THE OFFER EXCLUDES EVERY OBSERVED ADDRESS, every documented one, and every address
    /// outside a `static` range.
    #[test]
    fn the_offer_excludes_seen_documented_and_everything_outside_static() {
        let plan = plan();
        let none = BTreeMap::new();
        let nobody = BTreeSet::new();
        assert_eq!(
            plan.next_offerable(office(), &none, &nobody),
            Some(v4("192.0.2.1"))
        );
        assert_eq!(
            plan.next_offerable(office(), &seen_at(&["192.0.2.1"]), &nobody),
            Some(v4("192.0.2.2")),
            "a seen address is never offered"
        );
        assert_eq!(
            plan.next_offerable(office(), &none, &BTreeSet::from([v4("192.0.2.1")])),
            Some(v4("192.0.2.2")),
            "a documented address is never offered, even never observed"
        );
        for (addr, why) in [
            ("192.0.2.9", "a defined address"),
            ("192.0.2.0", "the network edge"),
            ("192.0.2.50", "space no range covers"),
            ("192.0.2.99", "a dhcp-pool"),
            ("192.0.2.140", "a reserved range"),
            ("192.0.2.205", "an infrastructure range"),
        ] {
            assert!(
                !plan.offerable(v4(addr), &none, &nobody),
                "{why} ({addr}) must not be offered"
            );
        }
        // A subnet with no static range proposes nothing.
        let workshop = subnet("198.51.100.128", 25);
        let reserved_only = Plan {
            subnets: vec![workshop],
            ranges: vec![(
                v4("198.51.100.129"),
                v4("198.51.100.200"),
                IpPolicy::Reserved,
            )],
            defined: BTreeSet::new(),
        };
        assert_eq!(
            reserved_only.next_offerable(workshop, &none, &nobody),
            None,
            "the offer served 198.51.100.129, a reserved address, until this story"
        );
    }

    /// Decision 13 — an address defined inside a `dhcp-pool` is a warning.
    #[test]
    fn an_address_defined_inside_a_pool_is_named() {
        let mut plan = plan();
        plan.defined.insert(v4("192.0.2.90"));
        assert_eq!(plan.defined_inside_a_pool(office()), vec![v4("192.0.2.90")]);
    }

    /// Documented values are parsed in Rust (D10), and a value that is not an address documents none.
    #[test]
    fn documented_values_are_addresses_or_nothing() {
        let values = vec![
            "192.0.2.11".to_string(),
            " 192.0.2.12 ".to_string(),
            "nas-01".to_string(),
            "192.0.2.999".to_string(),
        ];
        assert_eq!(
            documented_addresses(&values),
            BTreeSet::from([v4("192.0.2.11"), v4("192.0.2.12")])
        );
    }

    /// 🔴 **AC3's PROOF RE-ADMITTED EVERY SEEN ADDRESS, AND THE MUTATION THAT RE-ADMITS ONLY THE
    /// MAC-LESS ONES CAME BACK GREEN** — the acceptance layer's MX1b, measured over 677/191/99
    /// tests, clippy and ten gates. Every sighting in every offer test carried a hardware address,
    /// so *seen* and *seen with a MAC* were the same population and nothing could tell them apart.
    ///
    /// 🔑 **It is the ordinary shape on the shipped product**, which is what makes the hole matter:
    /// the ARP/ping connector reads a MAC only when the neighbour table has one, and story 14.3a's
    /// summary keeps the MAC-less sighting as a row of its own. *An address that answered a ping
    /// and gave no hardware address is in use, and the plan must not hand it out.*
    #[test]
    fn an_address_seen_with_no_hardware_address_is_still_never_offered() {
        let plan = plan();
        let nobody = BTreeSet::new();
        let seen = merge_sightings(&[sighting("192.0.2.1", 1, None, "2026-09-01T10:00:00Z")]);
        assert!(
            seen[&v4("192.0.2.1")].macs.is_empty(),
            "the premise: this sighting carries NO hardware address, which is what MX1b re-admitted"
        );
        assert!(
            !plan.offerable(v4("192.0.2.1"), &seen, &nobody),
            "an address seen with no hardware address is in use and is never offered"
        );
        assert_eq!(
            plan.next_offerable(office(), &seen, &nobody),
            Some(v4("192.0.2.2")),
            "and the offer moves past it"
        );
        assert_eq!(
            kinds(&plan.audit(office(), &seen, &BTreeSet::new())),
            vec![(v4("192.0.2.1"), Some(FindingKind::Gap), false)],
            "it is a finding like any other — the plan says free, the network says occupied"
        );
    }

    /// 🔴 **THE TRIAGE LINK USES TRIAGE'S OWN COMPARISON AND THE OFFER USES ITS OWN, and the two
    /// disagree on a value an operator can type today.** `page.rs:1156` filters the queue with
    /// `claimed.contains(&value)` over the declared `attr_value` STRINGS; the offer parses them. A
    /// value stored as `" 192.0.2.12 "` therefore documents the address for the offer and claims
    /// nothing for triage — measured by the review's edge layer, where the finding said *"documented,
    /// so triage asks nothing"* over a triage that was asking.
    ///
    /// 🔑 The direction of each is deliberate: the OFFER keeps the protective parse, because a false
    /// positive there costs a duplicate address; the LINK follows the screen it links to.
    #[test]
    fn the_triage_link_follows_triages_comparison_and_the_offer_keeps_the_parse() {
        let plan = plan();
        let stored = " 192.0.2.12 ".to_string();
        let documented = documented_addresses(std::slice::from_ref(&stored));
        let claimed: BTreeSet<String> = BTreeSet::from([stored]);
        assert!(
            documented.contains(&v4("192.0.2.12")),
            "the parse sees the address through the whitespace"
        );
        assert!(
            !plan.offerable(v4("192.0.2.12"), &BTreeMap::new(), &documented),
            "so the offer never proposes it"
        );
        let seen = seen_at(&["192.0.2.12"]);
        let findings = plan.audit(office(), &seen, &claimed);
        assert_eq!(findings.len(), 1);
        assert!(
            !findings[0].claimed,
            "and the finding must say triage HAS a question, because triage compares the stored \
             string and that string is not `192.0.2.12`"
        );
    }

    /// 🔴 **A `/31` AND A `/32` HAVE NO NETWORK OR BROADCAST ADDRESS (RFC 3021), and the offer took
    /// both addresses of a point-to-point link out of service.** Measured by the review's edge
    /// layer: `Subnet::is_edge` compared against both bounds, which on a `/31` is every address it
    /// has — so the two shapes an operator declares for exactly the addresses they mean to assign
    /// were the two shapes the plan refused to offer at all.
    #[test]
    fn a_point_to_point_link_offers_both_its_addresses() {
        let link = subnet("192.0.2.4", 31);
        let host = subnet("192.0.2.8", 32);
        let plan = Plan {
            subnets: vec![link, host],
            ranges: vec![
                (v4("192.0.2.4"), v4("192.0.2.5"), IpPolicy::Static),
                (v4("192.0.2.8"), v4("192.0.2.8"), IpPolicy::Static),
            ],
            defined: BTreeSet::new(),
        };
        let (none, nobody) = (BTreeMap::new(), BTreeSet::new());
        assert_eq!(
            link.addresses().collect::<Vec<_>>(),
            vec![v4("192.0.2.4"), v4("192.0.2.5")],
            "the premise: a /31 holds exactly two addresses"
        );
        assert_eq!(
            plan.next_offerable(link, &none, &nobody),
            Some(v4("192.0.2.4")),
            "RFC 3021: both addresses of a /31 are usable, the first one included"
        );
        assert!(
            plan.offerable(v4("192.0.2.5"), &none, &nobody),
            "and so is the second"
        );
        assert_eq!(
            plan.next_offerable(host, &none, &nobody),
            Some(v4("192.0.2.8")),
            "a /32 is a single host, not a network address with nothing behind it"
        );
    }

    /// The conflict rule on the two spaces the first test could not reach: an `infrastructure`
    /// range, and an EDGE that a range covers.
    ///
    /// 🔴 **Both cases sat in UNCOVERED space in the original test, so they passed for the wrong
    /// reason** (blind layer): uncovered space excludes a conflict on its own, whatever the policy
    /// rule does, so the `infrastructure` exclusion was carried by nothing.
    ///
    /// 🔴 **AND THE COVERED EDGE IS A CONFLICT — this test asserted it was not, and the PRODUCT was
    /// right.** The expectation was invented while writing the test and the assertion reddened on
    /// the first run: decision 10 says *two hardware addresses on one IPv4 inside a `static` or
    /// `reserved` range*, and it grants no exemption to an edge the operator laid a range over.
    /// Two machines answering on a subnet's broadcast address is exactly the sort of thing an
    /// operator wants named. 🔑 *The edge rule belongs to the OFFER — an edge is never assignable —
    /// and reading it into the conflict rule would have been a second meaning for one arbitration.*
    /// The expectation is corrected, never the product (story 6b.7's rule, met the other way round).
    #[test]
    fn two_macs_under_infrastructure_but_not_on_a_covered_edge_are_not_a_conflict() {
        let plan = Plan {
            subnets: vec![office()],
            ranges: vec![
                (v4("192.0.2.0"), v4("192.0.2.10"), IpPolicy::Infrastructure),
                (v4("192.0.2.11"), v4("192.0.2.255"), IpPolicy::Static),
            ],
            defined: BTreeSet::new(),
        };
        let two = |addr: &str| {
            vec![
                sighting(addr, 1, Some(1), "2026-09-01T10:00:00Z"),
                sighting(addr, 1, Some(2), "2026-09-02T10:00:00Z"),
            ]
        };
        let mut sightings = two("192.0.2.5");
        sightings.extend(two("192.0.2.255"));
        sightings.extend(two("192.0.2.20"));
        let seen = merge_sightings(&sightings);
        let conflicts: Vec<Ipv4Addr> = plan
            .audit(office(), &seen, &BTreeSet::new())
            .iter()
            .filter(|f| f.conflict)
            .map(|f| f.seen.addr)
            .collect();
        assert_eq!(
            conflicts,
            vec![v4("192.0.2.20"), v4("192.0.2.255")],
            "an `infrastructure` range is NOT a conflict even with two hardware addresses — that is \
             the exclusion this test exists for, and it was carried by nothing while both cases sat \
             in uncovered space. The broadcast edge covered by a `static` range IS one: the edge \
             rule takes an address out of the OFFER, and decision 10 grants it no exemption from a \
             conflict the operator would want to know about"
        );
        assert!(
            !plan.offerable(v4("192.0.2.255"), &BTreeMap::new(), &BTreeSet::new()),
            "the control that keeps the two rules apart: the same address is still never OFFERED"
        );
    }

    /// 🔴 **DECISION 5 MERGES ACROSS L2 DOMAINS, WHICH TURNS REUSED PRIVATE SPACE INTO A CONFLICT**
    /// — unstated and untested until the blind layer named it. Two machines answering at the same
    /// ordinary private address in two separate broadcast domains are two hardware addresses on one
    /// IPv4 here, and the screen calls it « Conflit d'adresse ».
    ///
    /// ⚠️ It is the PRICE of decision 5 and not a defect to fix in this story: the plan has no VLAN
    /// axis (FR21's VLAN half is outside Epic 14's arbitrations), so an address is one address
    /// whatever domain saw it. Registered, and pinned here so the day FR21 lands this test is what
    /// says the behaviour changed.
    #[test]
    fn one_address_in_two_l2_domains_is_read_as_a_conflict() {
        let plan = plan();
        let seen = merge_sightings(&[
            sighting("192.0.2.20", 1, Some(1), "2026-09-01T10:00:00Z"),
            sighting("192.0.2.20", 2, Some(2), "2026-09-02T10:00:00Z"),
        ]);
        let findings = plan.audit(office(), &seen, &BTreeSet::new());
        assert_eq!(
            kinds(&findings),
            vec![(v4("192.0.2.20"), Some(FindingKind::Gap), true)],
            "two DIFFERENT hardware addresses in two domains read as a conflict — the merge is what \
             makes it one address, and the plan has no VLAN to tell the two apart"
        );
    }

    // ── Story 14.4b: the release ────────────────────────────────────────────────────────────────

    fn released(pairs: &[(&str, &str)]) -> BTreeMap<Ipv4Addr, Timestamp> {
        pairs
            .iter()
            .map(|(addr, when)| (v4(addr), at(when)))
            .collect()
    }

    /// Decision 1 — a release forgets every sighting up to its instant, and an instant EQUAL to it.
    /// 🔑 The CONTROL is the same map with no release, without which the removal proves nothing.
    #[test]
    fn a_release_forgets_every_sighting_up_to_its_instant() {
        let seen = merge_sightings(&[
            sighting("192.0.2.20", 1, Some(1), "2026-09-01T10:00:00Z"),
            sighting("192.0.2.20", 1, None, "2026-09-01T10:30:00Z"),
            sighting("192.0.2.21", 1, Some(2), "2026-09-01T11:00:00Z"),
        ]);
        let control = forget_released(seen.clone(), &BTreeMap::new());
        assert_eq!(control, seen, "no release forgets nothing");

        let after = forget_released(
            seen,
            &released(&[
                ("192.0.2.20", "2026-09-01T11:00:00Z"),
                ("192.0.2.21", "2026-09-01T11:00:00Z"),
            ]),
        );
        assert!(
            after.is_empty(),
            "every sighting, with or without a MAC, at or before the release is forgotten: {after:?}"
        );
    }

    /// Decision 1 — the network answering AFTER the release holds the address again, and a NIC
    /// replaced after the release does not manufacture a conflict with the one it replaced.
    #[test]
    fn a_sighting_after_the_release_holds_the_address_again_on_its_own_mac() {
        let seen = merge_sightings(&[
            sighting("192.0.2.20", 1, Some(1), "2026-09-01T10:00:00Z"),
            sighting("192.0.2.20", 1, Some(7), "2026-09-01T11:06:00Z"),
        ]);
        let plan = plan();
        assert!(
            plan.audit(office(), &seen, &BTreeSet::new())[0].conflict,
            "the premise: without the release, two MACs inside `static` are a conflict"
        );

        let after = forget_released(seen, &released(&[("192.0.2.20", "2026-09-01T11:00:00Z")]));
        let held = after
            .get(&v4("192.0.2.20"))
            .expect("the later MAC holds it");
        assert_eq!(
            held.macs.keys().copied().collect::<Vec<_>>(),
            vec![mac(7)],
            "only the hardware address seen AFTER the release is kept"
        );
        let findings = plan.audit(office(), &after, &BTreeSet::new());
        assert_eq!(
            kinds(&findings),
            vec![(v4("192.0.2.20"), Some(FindingKind::Gap), false)],
            "held again, as an ordinary finding — and no conflict with the card it replaced"
        );
        assert!(!plan.offerable(v4("192.0.2.20"), &after, &BTreeSet::new()));
    }

    /// AC4 — a released address stops being a finding AND becomes offerable, in one act; and a
    /// released address the plan would never offer stops being a finding without being offered.
    #[test]
    fn a_released_address_stops_being_a_finding_and_the_offer_agrees() {
        let plan = plan();
        let seen = seen_at(&["192.0.2.1", "192.0.2.140"]);
        assert_eq!(
            kinds(&plan.audit(office(), &seen, &BTreeSet::new())),
            vec![
                (v4("192.0.2.1"), Some(FindingKind::Gap), false),
                (v4("192.0.2.140"), Some(FindingKind::Undeclared), false),
            ],
            "the premise: both are findings before the release"
        );
        assert_eq!(
            plan.next_offerable(office(), &seen, &BTreeSet::new()),
            Some(v4("192.0.2.2")),
            "the premise: the seen .1 is not offered"
        );

        let after = forget_released(
            seen,
            &released(&[
                ("192.0.2.1", "2026-09-02T00:00:00Z"),
                ("192.0.2.140", "2026-09-02T00:00:00Z"),
            ]),
        );
        assert!(
            plan.audit(office(), &after, &BTreeSet::new()).is_empty(),
            "a released address is no longer a finding"
        );
        assert_eq!(
            plan.next_offerable(office(), &after, &BTreeSet::new()),
            Some(v4("192.0.2.1")),
            "and the `static` one is offered again — the audit and the offer agree"
        );
        assert!(
            !plan.offerable(v4("192.0.2.140"), &after, &BTreeSet::new()),
            "a `reserved` address is never offered, released or not"
        );
    }

    // ── Story 14.4b, against the store ──────────────────────────────────────────────────────────

    fn observed(
        addr: Ipv4Addr,
        mac_octet: u8,
        when: &str,
    ) -> opencmdb_core::observation::Observation {
        use opencmdb_core::observation::{ConnectorId, Fact, ObsId, Observation, Scope, VantageId};
        Observation {
            obs_id: ObsId::from_uuid(uuid::Uuid::now_v7()),
            connector_id: ConnectorId::from_uuid(uuid::Uuid::nil()),
            observed_at: at(when),
            scope: Scope {
                l2_domain: L2DomainId::from_uuid(uuid::Uuid::nil()),
                vantage: VantageId::from_uuid(uuid::Uuid::nil()),
            },
            facts: vec![
                Fact::IpV4 { addr },
                Fact::Mac {
                    addr: mac(mac_octet),
                    locally_administered: true,
                },
            ],
            raw: None,
        }
    }

    /// Clear everything these tests write for one address and one subnet, so a re-run against a
    /// store that kept the last run's rows starts clean.
    async fn forget_release_fixture(pool: &sqlx::MySqlPool, subnet_id: &str, addr: Ipv4Addr) {
        let canonical = crate::ipam_repo::canonical(addr);
        for statement in [
            "DELETE FROM address_release WHERE addr = ?",
            "DELETE FROM address_sighting WHERE addr = ?",
        ] {
            sqlx::query(statement)
                .bind(&canonical)
                .execute(pool)
                .await
                .expect("clean");
        }
        sqlx::query(
            "DELETE FROM observation_record WHERE JSON_SEARCH(facts, 'one', ?) IS NOT NULL",
        )
        .bind(addr.to_string())
        .execute(pool)
        .await
        .expect("clean the observations");
        crate::ipam_repo::tests::forget_subnet(pool, subnet_id).await;
    }

    /// Run a store-backed test body, then clean up WHATEVER it did — and only then re-raise its panic.
    ///
    /// 🔴 **Found by this story's own mutation pass, and the count it corrected is the reason it
    /// exists.** Five of nine mutations came back with ONE red more than predicted, every time the
    /// same way: a test of this module panicked before its trailing cleanup, its DEFINED address
    /// survived, and `ipam_repo`'s `the_store_returns_addresses_in_numeric_order` — which reads the
    /// plan WIDE — reddened on it. *A carrier count inflated by collateral is not a measurement of
    /// carriers* (story 14.1's own finding, one module over). The cleanup now runs on both paths.
    async fn cleaned_up(
        pool: &sqlx::MySqlPool,
        subnet_id: &str,
        addrs: &[Ipv4Addr],
        body: impl std::future::Future<Output = ()>,
    ) {
        let outcome =
            futures_util::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(body)).await;
        for addr in addrs {
            forget_release_fixture(pool, subnet_id, *addr).await;
        }
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }

    async fn release_at(
        pool: &sqlx::MySqlPool,
        addr: Ipv4Addr,
        when: &str,
    ) -> Result<Option<String>, opencmdb_core::repo::RepositoryError> {
        let mut conn = pool.acquire().await.expect("a connection");
        crate::ipam_repo::release_address(&mut conn, addr, at(when)).await
    }

    /// AC2 and AC3 — the release is RECORDED, and nothing the ingest path does to the summary undoes
    /// it; only a sighting LATER than the release holds the address again (decision 1).
    ///
    /// 🔴 **AC2's letter reads "ingests an observation afterwards and the audit is unchanged", and
    /// under decision 1 that holds for an observation DATED before the release** — a late arrival,
    /// a replay — which is exactly what the refused *delete only* shape could not survive: the upsert
    /// re-creates the row it deleted. A sighting dated AFTER the release is the network answering,
    /// and holding the address again is decision 1 itself, asserted in the last step.
    #[tokio::test]
    async fn a_release_is_recorded_and_only_a_later_sighting_undoes_it() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = crate::ipam_repo::tests::ipam_fixture().await else {
            return;
        };
        let subnet_id = "t-rel-a";
        let addr = v4("100.64.141.20");
        forget_release_fixture(&pool, subnet_id, addr).await;

        cleaned_up(&pool, subnet_id, &[addr], async {
            crate::ipam_repo::insert_subnet(
                &pool,
                subnet_id,
                subnet("100.64.141.0", 24),
                "release",
            )
            .await
            .expect("subnet");
            {
                let mut conn = pool.acquire().await.expect("a connection");
                crate::ipam_repo::insert_range(
                    &mut conn,
                    "t-rel-a-r",
                    subnet_id,
                    v4("100.64.141.10"),
                    v4("100.64.141.30"),
                    IpPolicy::Static,
                    "static",
                )
                .await
                .expect("range");
            }
            let plan = Plan {
                subnets: vec![subnet("100.64.141.0", 24)],
                ranges: vec![(v4("100.64.141.10"), v4("100.64.141.30"), IpPolicy::Static)],
                defined: BTreeSet::new(),
            };
            let here = subnet("100.64.141.0", 24);

            crate::sighting_repo::insert_with_sightings(
                &pool,
                &observed(addr, 1, "2026-09-01T10:00:00Z"),
            )
            .await
            .expect("ingest");
            let before = read_the_network(&pool).await.expect("read");
            assert_eq!(
                kinds(&plan.audit(here, &before.seen, &before.claimed)),
                vec![(addr, Some(FindingKind::Gap), false)],
                "the premise: the sighting holds the address"
            );

            assert_eq!(
                release_at(&pool, addr, "2026-09-01T11:00:00Z")
                    .await
                    .expect("released"),
                Some(subnet_id.to_owned()),
                "the operator goes back to the subnet that contains the address"
            );
            // AC3 — written and READ BACK.
            let recorded = crate::ipam_repo::plan_releases(&pool)
                .await
                .expect("read back");
            assert_eq!(recorded.get(&addr), Some(&at("2026-09-01T11:00:00Z")));

            let after = read_the_network(&pool).await.expect("read");
            assert!(
                plan.audit(here, &after.seen, &after.claimed).is_empty(),
                "released: no longer a finding"
            );
            assert_eq!(
                plan.next_offerable(here, &after.seen, &after.documented),
                Some(v4("100.64.141.10"))
            );
            assert!(
                plan.offerable(addr, &after.seen, &after.documented),
                "and offerable again"
            );

            // AC2 — an observation dated BEFORE the release: the upsert widens the summary row, and the
            // release stands. The summary itself kept its history — the release deleted nothing.
            crate::sighting_repo::insert_with_sightings(
                &pool,
                &observed(addr, 1, "2026-09-01T10:30:00Z"),
            )
            .await
            .expect("ingest a late arrival");
            let late = read_the_network(&pool).await.expect("read");
            assert!(
                plan.audit(here, &late.seen, &late.claimed).is_empty(),
                "an observation older than the release does not undo it"
            );
            let (first_seen, rows): (String, i64) = sqlx::query_as(
                "SELECT DATE_FORMAT(MIN(first_seen_at), '%Y-%m-%dT%H:%i:%sZ'), COUNT(*) \
                 FROM address_sighting WHERE addr = ?",
            )
            .bind(crate::ipam_repo::canonical(addr))
            .fetch_one(&pool)
            .await
            .expect("the summary");
            assert_eq!(
                (first_seen.as_str(), rows),
                ("2026-09-01T10:00:00Z", 1),
                "the summary keeps its first sighting: a release is a row, never a deletion"
            );

            // Decision 1 — the network answering after the release holds the address again, here on a
            // NEW hardware address, which is the case the refused *mark the summary* shape lost.
            crate::sighting_repo::insert_with_sightings(
                &pool,
                &observed(addr, 9, "2026-09-01T11:06:00Z"),
            )
            .await
            .expect("ingest a later sighting");
            let answered = read_the_network(&pool).await.expect("read");
            assert_eq!(
                kinds(&plan.audit(here, &answered.seen, &answered.claimed)),
                vec![(addr, Some(FindingKind::Gap), false)],
                "held again, and no conflict with the MAC the release forgot"
            );
            assert!(!plan.offerable(addr, &answered.seen, &answered.documented));
        })
        .await;
    }

    /// Decision 4 — a DEFINED address cannot be released, and nothing is written; and a second
    /// release WIDENS the instant, never narrows it.
    #[tokio::test]
    async fn a_defined_address_is_refused_and_a_second_release_only_widens() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = crate::ipam_repo::tests::ipam_fixture().await else {
            return;
        };
        let subnet_id = "t-rel-b";
        let defined = v4("100.64.142.9");
        let stray = v4("100.64.142.30");
        forget_release_fixture(&pool, subnet_id, defined).await;
        forget_release_fixture(&pool, subnet_id, stray).await;

        cleaned_up(&pool, subnet_id, &[defined, stray], async {
            crate::ipam_repo::insert_subnet(
                &pool,
                subnet_id,
                subnet("100.64.142.0", 24),
                "release",
            )
            .await
            .expect("subnet");
            {
                let mut conn = pool.acquire().await.expect("a connection");
                crate::ipam_repo::insert_address(&mut conn, "t-rel-b-a", subnet_id, defined, "nas")
                    .await
                    .expect("address");
            }

            let refused = release_at(&pool, defined, "2026-09-01T11:00:00Z").await;
            assert!(
                matches!(
                    refused,
                    Err(opencmdb_core::repo::RepositoryError::Ipam(
                        opencmdb_core::ipam::IpamError::ReleaseOfADefinedAddress
                    ))
                ),
                "a defined address is refused by NAME: {refused:?}"
            );
            assert!(
                !crate::ipam_repo::plan_releases(&pool)
                    .await
                    .expect("read")
                    .contains_key(&defined),
                "and the refusal wrote nothing"
            );

            release_at(&pool, stray, "2026-09-01T11:00:00Z")
                .await
                .expect("released");
            release_at(&pool, stray, "2026-09-01T09:00:00Z")
                .await
                .expect("released again, earlier");
            assert_eq!(
                crate::ipam_repo::plan_releases(&pool)
                    .await
                    .expect("read")
                    .get(&stray),
                Some(&at("2026-09-01T11:00:00Z")),
                "an earlier second release does not narrow what the first forgot"
            );
            release_at(&pool, stray, "2026-09-01T12:00:00Z")
                .await
                .expect("released later");
            assert_eq!(
                crate::ipam_repo::plan_releases(&pool)
                    .await
                    .expect("read")
                    .get(&stray),
                Some(&at("2026-09-01T12:00:00Z")),
                "a later one widens it"
            );
        })
        .await;
    }

    /// The production route, end to end: the clock read at the edge, the transaction, the redirect
    /// to the innermost subnet, the audit agreeing — and decision 4's refusal as a KEYED 409.
    #[tokio::test]
    async fn the_release_route_writes_through_the_store_and_the_audit_agrees() {
        use tower::ServiceExt;
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = crate::ipam_repo::tests::ipam_fixture().await else {
            return;
        };
        let subnet_id = "t-rel-c";
        let held = v4("100.64.143.20");
        let defined = v4("100.64.143.9");
        forget_release_fixture(&pool, subnet_id, held).await;
        forget_release_fixture(&pool, subnet_id, defined).await;

        cleaned_up(&pool, subnet_id, &[held, defined], async {
            crate::ipam_repo::insert_subnet(
                &pool,
                subnet_id,
                subnet("100.64.143.0", 24),
                "release",
            )
            .await
            .expect("subnet");
            {
                let mut conn = pool.acquire().await.expect("a connection");
                crate::ipam_repo::insert_address(&mut conn, "t-rel-c-a", subnet_id, defined, "nas")
                    .await
                    .expect("address");
            }
            crate::sighting_repo::insert_with_sightings(
                &pool,
                &observed(held, 1, "2026-09-01T10:00:00Z"),
            )
            .await
            .expect("ingest");

            let post = |addr: Ipv4Addr| {
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/ipam/release")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .header("host", "opencmdb.example")
                    .header("origin", "https://opencmdb.example")
                    .body(axum::body::Body::from(format!("addr={addr}")))
                    .expect("a request")
            };
            let response = crate::ipam_write::router(pool.clone())
                .oneshot(post(held))
                .await
                .expect("an answer");
            assert_eq!(response.status(), axum::http::StatusCode::OK);
            assert_eq!(
                response
                    .headers()
                    .get("hx-redirect")
                    .and_then(|v| v.to_str().ok()),
                Some(format!("/ipam?subnet={subnet_id}").as_str()),
                "back to the subnet that contains the address"
            );
            let recorded = crate::ipam_repo::plan_releases(&pool).await.expect("read");
            assert!(
                recorded
                    .get(&held)
                    .is_some_and(|at| *at > self::at("2026-09-01T10:00:00Z")),
                "the release is dated by the clock, after the sighting it forgets: {recorded:?}"
            );
            let network = read_the_network(&pool).await.expect("read");
            assert!(
                !network.seen.contains_key(&held),
                "the audit forgot the sighting"
            );

            let refused = crate::ipam_write::router(pool.clone())
                .oneshot(post(defined))
                .await
                .expect("an answer");
            assert_eq!(refused.status(), axum::http::StatusCode::CONFLICT);
            let body = axum::body::to_bytes(refused.into_body(), usize::MAX)
                .await
                .expect("a body");
            assert_eq!(
                String::from_utf8_lossy(&body),
                rust_i18n::t!("ipam.refusal.release_of_defined").to_string(),
                "decision 4's refusal is the keyed sentence that names the gesture which does apply"
            );
            assert!(
                !crate::ipam_repo::plan_releases(&pool)
                    .await
                    .expect("read")
                    .contains_key(&defined),
                "and wrote nothing"
            );
        })
        .await;
    }
}
