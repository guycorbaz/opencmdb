//! The addressing plan — what the operator INTENDS an address or a stretch of them to be.
//!
//! # 🔴 Why a subdomain of its own, and why it holds no arithmetic
//!
//! `architecture.md:3208` prescribes it: *"one `thiserror` per subdomain IS one per decider (D47):
//! `identity::IdentityError`, `gap::GapError`, **`ipam::IpamError`**"*, and `:3366` lists `ipam/`
//! among the modules that *"arrive"* unbuilt. This is that arrival, and it arrives EMPTY of
//! behaviour on purpose: story 14.1 ships a domain, a schema and an adapter, and **no producer**.
//!
//! ⚠️ **This module is outside the `float-free` gate**, which walks
//! `crates/opencmdb-core/src/identity/` alone. That is correct today and is stated rather than
//! discovered: there is no arithmetic here at all, and the occupancy figure `/ipam` renders is
//! three COUNTS rather than a ratio (`example_data.rs`: *"why the occupancy is a LIST and never a
//! formula"*). **Story 14.3 is where a ratio could first appear**, and it is that story's to decide
//! whether the perimeter widens. D13's ban is scoped to identity DECISIONS; extending it by reflex
//! would apply a decision outside its stated domain.
//!
//! # What the plan is NOT
//!
//! 🔑 It is a SECOND declared register, beside `declared_attribute`, and Guy's arbitration (4) of
//! 2026-09-10 keeps them apart: **IPAM says what was MEANT to be there; `declared_attribute` says
//! what is DOCUMENTED.** A machine documented on an address the plan never planned is real
//! information, and the day the two are fused that information is gone. Nothing here refers to an
//! entity, an observation or an interface.

use std::fmt;

use serde::{Deserialize, Serialize};

/// What a stretch of address space is MEANT for — the PLAN axis of the binding vocabulary.
///
/// # 🔴 The policy is what decides a verdict, and without it an audit is unreadable
///
/// Ratified by Guy on 2026-09-11 (`prd.md`, `ux-design-specification.md`, PR #166) as a THIRD axis
/// beside the gesture and state axes. **No story may extend it**: one term, one translation, one
/// meaning.
///
/// The reason it exists is measured rather than preferred. The same unclaimed address is an anomaly
/// in a static zone and entirely ordinary in a pool — and on the reference LAN the router serves
/// **24 DHCP leases** of the 32 addresses reverse DNS resolves, so an audit that highlights every
/// unclaimed address highlights 24 of them permanently and stops being read.
///
/// ⚠️ **Nothing in this codebase reads a policy yet.** Story 14.3 is the audit; this enum is the
/// vocabulary it will read, posed once so that story does not invent it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IpPolicy {
    /// Addresses the operator assigns by hand — one machine per address, chosen.
    ///
    /// An observed address here that the plan does not name is an ANOMALY.
    Static,
    /// Addresses a DHCP server hands out.
    ///
    /// 🔑 The occupant changes **by design**, so an observed address here that the plan does not
    /// name is NORMAL. This is the variant that makes the audit legible rather than noisy.
    DhcpPool,
    /// Addresses set aside for a use that has not arrived.
    ///
    /// Nothing should answer here yet, so an observed address is an anomaly **and** a surprise.
    Reserved,
    /// Addresses belonging to the network itself — gateway, equipment management, and the network
    /// and broadcast addresses.
    ///
    /// Not host space, and **never offered as free**.
    ///
    /// ⚠️ `structural` was a displayed word for the network and broadcast addresses, derived from
    /// the CIDR by arithmetic. Guy retired it on 2026-09-11: two words with adjacent meanings on one
    /// screen is the synonym problem the binding table exists to prevent. `.0` and `.255` are simply
    /// infrastructure the product knows without being told. **Story 14.2 carries the removal from
    /// the code**, which still renders it.
    Infrastructure,
}

impl IpPolicy {
    /// Every policy, in the order the schema's CHECK lists them.
    ///
    /// It exists so a test can compare the two representations as **SETS** rather than trusting
    /// them to stay parallel. 🔴 Story 6.5's **M8** is why the word SETS is load-bearing: its
    /// predecessor compared COUNTS, so pointing two variants at one token left it green — six
    /// variants, six quoted tokens, one variant unreachable and one schema value nothing could read
    /// back. ***A count is not a set.***
    ///
    /// ⚠️ **And a set comparison closes ONE direction only.** A variant missing from this constant
    /// with a token the CHECK refuses leaves the whole suite, the ten gates and clippy GREEN, while
    /// anything writing it takes `ERROR 4025` in production — measured by story 14.1's validation,
    /// and measured green TODAY on [`crate::observation::EntityState`]. The other direction is
    /// carried by `screens.rs::every_variant_of_a_navigated_enum_is_listed_in_all`, where this enum
    /// is a row.
    pub const ALL: [IpPolicy; 4] = [
        IpPolicy::Static,
        IpPolicy::DhcpPool,
        IpPolicy::Reserved,
        IpPolicy::Infrastructure,
    ];

    /// The token this policy is persisted as. The schema's CHECK holds the same set.
    ///
    /// ⚠️ `ascii_bin` is a PAD SPACE collation, so a RAW write of `'static '` would satisfy the
    /// CHECK and come back here as an unfamiliar token. The adapter cannot produce that and
    /// `0007`'s own `CHECK` refuses it — see that migration's header, which states the measurement.
    pub fn as_str(&self) -> &'static str {
        match self {
            IpPolicy::Static => "static",
            IpPolicy::DhcpPool => "dhcp-pool",
            IpPolicy::Reserved => "reserved",
            IpPolicy::Infrastructure => "infrastructure",
        }
    }
}

impl fmt::Display for IpPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What the addressing plan refuses, as domain data rather than as a string.
///
/// # 🔑 Why these are NAMED variants and not a `String`
///
/// D47: *"an error there is domain data, not a string"*. Story 14.1's validation built the adapter
/// and had to write `.map_err(|e| RepositoryError::Backend(e.to_string()))` for every one of these,
/// which is precisely what D47 forbids — so the refusals live here and the adapter maps them into
/// `RepositoryError::Constraint` with a name the caller can match on.
///
/// ⚠️ Every one of these is refused by the **ADAPTER** and not by the DDL, and the reason is the
/// same in each case: the rule compares two TABLES, and a `CHECK` that references another table is
/// `ERROR 1901` on MariaDB 10.11 — measured, not supposed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpamError {
    /// A range's bounds fall outside the subnet it belongs to.
    RangeOutsideSubnet,
    /// A range overlaps another range already defined in the same subnet.
    ///
    /// ⚠️ Two overlapping ranges cannot be refused by a `CHECK` either — the rule compares a row
    /// against its SIBLINGS, and MariaDB refuses a subquery in a `CHECK` for the same reason.
    RangeOverlapsAnother,
    /// An individually defined address falls outside the subnet it belongs to.
    AddressOutsideSubnet,
    /// A subnet's base is not the network address of its own CIDR.
    ///
    /// `192.0.2.5/24` is refused: containment is arithmetic on the base, and a base that is not the
    /// network address makes every answer wrong in a way nothing downstream can detect.
    BaseIsNotTheNetworkAddress,
    /// A prefix length that cannot belong to its address family.
    ///
    /// 🔴 The validation measured why this is a refusal and not a lint: a `/64` accepted on an IPv4
    /// base makes the natural containment arithmetic compute `32 - 64` and **panic on
    /// subtract-with-overflow**. *A schema that admits an impossible pair hands the arithmetic an
    /// impossible input.*
    PrefixLengthNotInFamily,
    /// A range whose last address precedes its first.
    ///
    /// 🔴 **Found by the review, and the defect was the REASON rather than the acceptance.** An
    /// inverted range inside a populated subnet was refused as `RangeOverlapsAnother` — an empty
    /// interval overlaps nothing, and the DDL's `ip_range_bounds_ordered` caught it only when it
    /// happened not to intersect a sibling. Story 14.2 renders these refusals to the operator, so a
    /// refusal that names the wrong rule is a sentence the operator cannot act on.
    RangeBoundsInverted,
    /// A value that is not this store's canonical form for an address.
    ///
    /// One address has ONE spelling — the zero-padded dotted quad — and it is imposed by the DDL as
    /// well as here, because *a canonical form that is not imposed is not a canonical form*.
    MalformedAddress,
}

impl IpamError {
    /// Every refusal, so a test can assert a PROPERTY over them rather than over a list it wrote.
    ///
    /// 🔴 **The review measured why this exists.** `every_refusal_carries_a_sentence` hand-wrote its
    /// six-element array inside its own body, so a seventh variant whose `Display` returned a
    /// VERBATIM duplicate of another's left **871 tests, ten gates and clippy green** — the exact
    /// thing that test's own message calls *"two refusals read the same, so the operator cannot tell
    /// them apart"*.
    ///
    /// 🔑 And the mechanism was already in this commit, one type over: [`IpPolicy::ALL`] plus a row
    /// in `screens.rs`'s enum-completeness guard. *The hole was recognised for one type and left
    /// open for its neighbour, in the same file.* Both are rows now.
    pub const ALL: [IpamError; 7] = [
        IpamError::RangeOutsideSubnet,
        IpamError::RangeOverlapsAnother,
        IpamError::RangeBoundsInverted,
        IpamError::AddressOutsideSubnet,
        IpamError::BaseIsNotTheNetworkAddress,
        IpamError::PrefixLengthNotInFamily,
        IpamError::MalformedAddress,
    ];
}

impl fmt::Display for IpamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            IpamError::RangeOutsideSubnet => "the range falls outside its subnet",
            IpamError::RangeOverlapsAnother => "the range overlaps another in the same subnet",
            IpamError::AddressOutsideSubnet => "the address falls outside its subnet",
            IpamError::BaseIsNotTheNetworkAddress => {
                "the subnet's base is not the network address of its own prefix"
            }
            IpamError::PrefixLengthNotInFamily => {
                "the prefix length cannot belong to this address family"
            }
            IpamError::RangeBoundsInverted => "the range ends before it begins",
            IpamError::MalformedAddress => "the address is not in this store's canonical form",
        })
    }
}

impl std::error::Error for IpamError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every policy renders the token the schema's `CHECK` holds, and no two share one.
    ///
    /// 🔑 The cross-crate half — comparing this set against the LIVE `CHECK` clause — lives in
    /// `opencmdb-bin`, because D47 forbids this crate to reach a database. What is asserted HERE is
    /// the property that needs no store: the mapping is total and injective.
    #[test]
    fn every_policy_has_its_own_token() {
        let tokens: std::collections::BTreeSet<&str> =
            IpPolicy::ALL.iter().map(IpPolicy::as_str).collect();
        assert_eq!(
            tokens.len(),
            IpPolicy::ALL.len(),
            "two policies persist as the same token — one of them can never be read back, and a \
             COUNT of four would have hidden it (story 6.5's M8: a count is not a set)"
        );
        assert_eq!(IpPolicy::Static.as_str(), "static");
        assert_eq!(IpPolicy::DhcpPool.as_str(), "dhcp-pool");
    }

    /// `Display` renders the persisted token rather than the Rust name — so a log line and a stored
    /// row say the same word.
    #[test]
    fn display_is_the_persisted_token() {
        for policy in IpPolicy::ALL {
            assert_eq!(policy.to_string(), policy.as_str());
        }
        assert_eq!(IpPolicy::DhcpPool.to_string(), "dhcp-pool");
    }

    /// Every refusal says what it refused, in words the operator could read.
    ///
    /// ⚠️ The sentences are deliberately about the PLAN and not about SQL: D47's *"an error there is
    /// domain data, not a string"* is what this enum exists for, and a message naming a constraint
    /// or an error number would put the driver's vocabulary back where the domain's belongs.
    #[test]
    fn every_refusal_carries_a_sentence() {
        // 🔴 Reads `ALL` rather than a list of its own. The review added a seventh variant with a
        // DUPLICATE sentence and this test — which then enumerated six by hand — stayed green.
        let sentences: std::collections::BTreeSet<String> =
            IpamError::ALL.iter().map(ToString::to_string).collect();
        assert_eq!(
            sentences.len(),
            IpamError::ALL.len(),
            "two refusals read the same, so the operator cannot tell them apart"
        );
        for sentence in &sentences {
            assert!(!sentence.is_empty());
            assert!(
                !sentence.contains("ERROR") && !sentence.contains("sqlx"),
                "{sentence:?} carries the driver's vocabulary where the domain's belongs"
            );
        }
    }
}
