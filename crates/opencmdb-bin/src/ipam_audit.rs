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
//!    every other observed address without an address row is `undeclared`; nothing inside a
//!    `dhcp-pool` is a finding.
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

/// Merge the summary's rows into one [`Seen`] per address, across L2 domains (decision 5).
///
/// The same MAC seen in two L2 domains is ONE hardware address, with the later of its two instants.
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
    /// Observed, and the plan WOULD offer it: the plan says free, the network says occupied — the
    /// core object, seen on an address instead of on a field (`gap`, « écart »).
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
    /// Whether `declared_attribute` documents it — which also says whether triage has a
    /// `nouveau:` row for it: triage raises one only for an observed address no declared record
    /// claims.
    pub(crate) documented: bool,
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
            // Decision 1: nothing inside a `dhcp-pool` is a finding — the occupant changes by design.
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
    pub(crate) fn audit(
        &self,
        subnet: Subnet,
        seen: &BTreeMap<Ipv4Addr, Seen>,
        documented: &BTreeSet<Ipv4Addr>,
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
                    documented: documented.contains(&s.addr),
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
pub(crate) async fn read_the_network(
    pool: &sqlx::MySqlPool,
) -> Result<(BTreeMap<Ipv4Addr, Seen>, BTreeSet<Ipv4Addr>), opencmdb_core::repo::RepositoryError> {
    let sightings = crate::sighting_repo::load_sightings(pool)
        .await
        .map_err(crate::repo::classify)?;
    let documented = crate::repo::load_documented_ipv4s(pool)
        .await
        .map_err(crate::repo::classify)?;
    Ok((
        merge_sightings(&sightings),
        documented_addresses(&documented),
    ))
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
}
