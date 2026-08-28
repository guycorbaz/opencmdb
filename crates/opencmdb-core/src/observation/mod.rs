//! The observation vocabulary — what a source SAW, never what is "gone".
//!
//! Per D19, the fixture schema IS the `Observation` schema: every connector emits these
//! types, and the engine reads them. The single load-bearing constraint (NFR7/D35): an
//! `Observation` is **incapable** of expressing absence — there is no `Gone`/`Absent`/
//! `Missing`. Absence is DERIVED by the engine, and only when a source is live. "The
//! cheapest NFR7 test that exists: make the bug not compile."
//!
//! Time enters as data: `observed_at` comes from the source. `opencmdb-core`'s `chrono`
//! has its `clock` feature OFF, so `Utc::now()` is not even callable here (D19).

use std::collections::BTreeSet;
use std::fmt;
use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A moment, sourced from the outside (a connector's clock or a fixture), never minted here.
pub type Timestamp = chrono::DateTime<chrono::Utc>;

/// Opaque identity newtypes. UUIDv7 so they sort by creation time; minted in the composition
/// root, never derived from observed values.
macro_rules! uuid_newtype {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name(pub Uuid);

        impl $name {
            /// Wrap an existing UUID (identity is minted by the caller, D48).
            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }
            /// The underlying UUID.
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

uuid_newtype!(
    /// Stable id of one observation, so a truth-labelling can point at it (D19).
    ObsId
);
uuid_newtype!(
    /// Identifies the connector that produced an observation.
    ConnectorId
);
uuid_newtype!(
    /// The MAC's uniqueness space — the L2 domain in which a MAC is unique.
    L2DomainId
);
uuid_newtype!(
    /// WHO saw it — the observing vantage point.
    VantageId
);
uuid_newtype!(
    /// Identifies one interface — the thing the L1 join forms.
    ///
    /// At L1 an interface IS a scope-qualified key: [`crate::identity::l1::join`] groups
    /// observations by `(l2_domain, mac)` and each group is one interface. This newtype names the
    /// row that group is persisted as, so a link can point at it. It is **minted client-side**
    /// (D48) — no function here returns one, and it is never derived from an observed value.
    ///
    /// It lives beside [`ObsId`] rather than under `identity/` because `uuid_newtype!` is a bare
    /// `macro_rules!` reachable only from this module — and because "the folder is not the
    /// frontier, visibility is" (D54). [`L2DomainId`] and [`VantageId`] are not observations
    /// either.
    InterfaceId
);
uuid_newtype!(
    /// Identifies one identity link — one placement of an observation on an interface.
    ///
    /// A link is an ENTITY, not a foreign key (D14): it is versioned, it carries the rule and the
    /// evidence that justified it, and superseding it appends a row rather than overwriting one.
    /// That is why it needs an id of its own. **Minted client-side** (D48).
    LinkId
);
uuid_newtype!(
    /// Identifies one device — the thing L2 grouping forms out of interfaces.
    ///
    /// A device has **no business columns** and this type has no fields beyond its id, for the same
    /// reason (D21): *"everything a device is is either observed (via its interfaces) or declared. A
    /// device is an identifier and nothing else. If anyone proposes adding `hostname` to it, they
    /// have just restored the OBSERVED/DECLARED merge we forbade."* **Minted client-side** (D48).
    ///
    /// ⚠️ Nothing in this codebase produces one outside its own tests. Story 6.5 ships the schema;
    /// story 6.12 is the resolver that fills it.
    DeviceId
);

/// Which subtype an `entity` row is — the disjunction D21 makes structural.
///
/// It is a closed set on purpose: the supertype exists so that *"the disjunction is enforced by the
/// engine, not by convention"*, and a `kind` a future story adds here must also be added to the
/// `entity_kind_domain` CHECK in the schema.
///
/// 🔴 **That sentence used to end *"which is what makes the two representations testable against
/// each other"*, and it was FALSE when written.** The code review planted a third variant the CHECK
/// forbids and a CHECK token no variant names — **770 tests and ten gates green, both directions**.
/// The lesson this very story shipped for [`EntityState`] (*a count is not a set*) had been applied
/// to one column of a table and not to the one beside it in the same DDL. [`EntityKind::ALL`] and
/// the agreement guard in the adapter are what make the sentence true now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EntityKind {
    /// An interface — the thing the L1 join forms.
    ///
    /// ⚠️ **No `interface` row is an entity yet.** Guy's arbitration of 2026-08-28 (option (a))
    /// leaves the existing table outside the supertype; its adoption is story 6.12's, with the
    /// migration that lets a device be a placement subject and the resolver change that mints the
    /// parent row. The variant exists now so the domain is posed once rather than widened by an
    /// `ALTER` running at boot on a published product.
    Interface,
    /// A device — the thing L2 grouping forms.
    Device,
}

impl EntityKind {
    /// Every kind, in the order the schema's CHECK lists them.
    ///
    /// It exists for the same reason [`EntityState::ALL`] does — so a test can compare the two
    /// representations as SETS rather than trusting them to stay parallel.
    pub const ALL: [EntityKind; 2] = [EntityKind::Interface, EntityKind::Device];

    /// The token this kind is persisted as.
    ///
    /// ⚠️ The schema's CHECK holds the same set, and a test asserts it — but `ascii_bin` is a PAD
    /// SPACE collation, so a RAW write of `'device '` satisfies the CHECK and comes back here as an
    /// unfamiliar token. The adapter cannot produce that; a backfill can. `0006`'s header states it.
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityKind::Interface => "interface",
            EntityKind::Device => "device",
        }
    }
}

/// Where an entity sits in its lifecycle (D21, `architecture.md:1502`).
///
/// 🔴 **Six values, where `epics.md:1826` names two.** The epic's `active`/`dormant` are the subset
/// FR38b needs; this is the domain the architecture enumerates. Shipping the subset would buy an
/// `ALTER` running at boot on a published product the day a lifecycle story needs `Superseded` —
/// the hazard `0003_resolver_guards.sql`'s header documents — so the domain is posed once. The
/// divergence is registered rather than taken in silence.
///
/// ⚠️ **This is a domain and no behaviour.** Nothing in this codebase sets any value but
/// [`EntityState::Active`]. `Dormant` in particular is valid only for an interface whose address is
/// locally administered, a scope that needs a `mac_kind` column no table carries; the invariant is
/// story 6.18's, which owns FR38b's transition, because a scoping rule belongs with the transition
/// it scopes rather than with the column it constrains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EntityState {
    /// Live, counted, and eligible for candidate generation. The default for a new entity.
    Active,
    /// Unobserved past the dormancy window. FR38b says such an entity **will be** excluded from
    /// divergence metrics and from automatic candidate generation while staying queryable, and
    /// **will return** to `Active` if the address is observed again — the same entity, not a new
    /// one.
    ///
    /// ⚠️ **Written in the future tense because none of it exists.** Nothing in this codebase sets
    /// this value, sweeps for it or reads it; story 6.18 carries FR38b's transition. *A doc comment
    /// must be true, so prefer the weaker true sentence* — the first draft asserted the mechanism
    /// in the present tense and the code review caught it.
    Dormant,
    /// Replaced by another entity through an identity migration; kept because a bad grouping is
    /// UNLINKED, never erased (D14).
    Superseded,
    /// Held out of automatic processing pending a human decision.
    Quarantined,
    /// Mid-migration: an identity move has begun and is not yet complete.
    PendingMigration,
    /// A reserved row that stands for "no entity" rather than for a thing on the network — D21's
    /// sentinel idiom, whose interface half `identity_link.current_subject` already uses.
    Sentinel,
}

impl EntityState {
    /// The token this state is persisted as. The schema's CHECK holds the same set.
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityState::Active => "active",
            EntityState::Dormant => "dormant",
            EntityState::Superseded => "superseded",
            EntityState::Quarantined => "quarantined",
            EntityState::PendingMigration => "pending_migration",
            EntityState::Sentinel => "sentinel",
        }
    }

    /// Every state, in the order the schema's CHECK lists them.
    ///
    /// It exists so a test can assert the two representations agree — the `CLAUDE.md` idiom of a
    /// *deliberate* redundancy pinned by an equality test, as `score.rs`'s `Column::as_str` and
    /// `Expectation::column` already are.
    pub const ALL: [EntityState; 6] = [
        EntityState::Active,
        EntityState::Dormant,
        EntityState::Superseded,
        EntityState::Quarantined,
        EntityState::PendingMigration,
        EntityState::Sentinel,
    ];
}

/// A 48-bit hardware address held as its exact 6 bytes.
///
/// Bytes, not a `String`: device identity is compared byte-exact. A textual MAC would
/// reintroduce case/locale ambiguity — the very thing D64's binary collation forbids one
/// layer down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MacAddr(pub [u8; 6]);

impl MacAddr {
    /// True when the U/L bit (bit 1 of the first octet) is set — a locally-administered
    /// address (e.g. MAC randomisation). This is the ground truth a connector's reported
    /// `locally_administered` flag can be cross-checked against.
    pub fn is_locally_administered(&self) -> bool {
        self.0[0] & 0b0000_0010 != 0
    }
}

impl fmt::Display for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b = self.0;
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            b[0], b[1], b[2], b[3], b[4], b[5]
        )
    }
}

/// A MAC could not be parsed from text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacParseError;

impl fmt::Display for MacParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expected six colon-separated hex octets (aa:bb:cc:dd:ee:ff)")
    }
}

impl std::error::Error for MacParseError {}

impl std::str::FromStr for MacAddr {
    type Err = MacParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0u8; 6];
        let mut parts = s.split(':');
        for slot in &mut bytes {
            let part = parts.next().ok_or(MacParseError)?;
            if part.len() != 2 {
                return Err(MacParseError);
            }
            *slot = u8::from_str_radix(part, 16).map_err(|_| MacParseError)?;
        }
        if parts.next().is_some() {
            return Err(MacParseError); // more than six octets
        }
        Ok(MacAddr(bytes))
    }
}

/// Where a hostname was learned — the same name from DHCP and from mDNS are not equally
/// trustworthy, and the engine may weight them differently later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HostnameSource {
    Dhcp,
    Dns,
    Mdns,
    Netbios,
    Other,
}

/// One thing a source observed about a device. A positive statement only — there is
/// deliberately no variant meaning "absent"/"gone" (NFR7/D35).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// A fixture is an ORACLE: an unknown or misspelled field must fail loudly rather than be
// ignored. Without this, a line carrying `"gone":true` — or `locally_administred` beside the
// correct spelling — parses silently and the corpus means something other than it says.
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub enum Fact {
    /// A hardware address, plus the source's claim about whether it is locally administered.
    Mac {
        addr: MacAddr,
        locally_administered: bool,
    },
    /// An IPv4 address seen for the device. (IPv6 is a future `#[non_exhaustive]` addition.)
    IpV4 { addr: Ipv4Addr },
    /// A hostname and where it came from.
    Hostname {
        name: String,
        source: HostnameSource,
    },
    /// A DHCP lease: the leased address and, when known, its expiry.
    DhcpLease {
        ip: Ipv4Addr,
        expires_at: Option<Timestamp>,
    },
    /// A topology edge: the peer's MAC and the port it was seen on.
    Uplink {
        peer_mac: MacAddr,
        peer_port: String,
    },
    /// The OUI-derived vendor of the MAC.
    OuiVendor { vendor: String },
    /// A measured round-trip time, in milliseconds.
    Rtt { millis: u32 },
}

/// The kind of a [`Fact`] without its payload — the alphabet a source's [`Capabilities`]
/// enumerate. Kept in lockstep with `Fact` (see [`Fact::kind`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FactKind {
    Mac,
    IpV4,
    Hostname,
    DhcpLease,
    Uplink,
    OuiVendor,
    Rtt,
}

impl FactKind {
    /// Every kind a source could declare, in the enum's own order.
    ///
    /// # Why this exists, and why it is a `const` rather than a `match`
    ///
    /// 🔴 **Story 6b.8 needs the COMPLEMENT** — *what a source cannot see* is
    /// `ALL \ Capabilities::kinds`, and the shipped ARP/ping connector declares two of these seven,
    /// so the operator-facing answer is five kinds derived at runtime with no database and no
    /// invention. That is the one section of `/sources` its acceptance criterion requires to be
    /// REAL.
    ///
    /// ⚠️ **`FactKind` is `#[non_exhaustive]`, so a downstream crate cannot match it exhaustively**:
    /// any `match` in `opencmdb-bin` needs a `_` arm, and once that arm exists it is permanently
    /// silent — an eighth kind would fall into the wildcard and the screen would UNDER-report what
    /// a source cannot see, which is the lie in the safe-looking direction. The story's validation
    /// measured exactly that: an eighth variant left out of this list passed the suite, `clippy -D
    /// warnings` and all eight gates.
    ///
    /// 🔑 **The carrier is therefore a TEST and it is named here so the link is not lost**:
    /// `crate::screens::tests::every_variant_of_a_navigated_enum_is_listed_in_all` in
    /// `opencmdb-bin` reads this file with `include_str!` and reds when a variant is declared and
    /// absent from this list. It is one row in that guard's table; without it, nothing at all
    /// catches the omission.
    pub const ALL: [FactKind; 7] = [
        FactKind::Mac,
        FactKind::IpV4,
        FactKind::Hostname,
        FactKind::DhcpLease,
        FactKind::Uplink,
        FactKind::OuiVendor,
        FactKind::Rtt,
    ];
}

impl Fact {
    /// The [`FactKind`] discriminant of this fact.
    pub fn kind(&self) -> FactKind {
        match self {
            Fact::Mac { .. } => FactKind::Mac,
            Fact::IpV4 { .. } => FactKind::IpV4,
            Fact::Hostname { .. } => FactKind::Hostname,
            Fact::DhcpLease { .. } => FactKind::DhcpLease,
            Fact::Uplink { .. } => FactKind::Uplink,
            Fact::OuiVendor { .. } => FactKind::OuiVendor,
            Fact::Rtt { .. } => FactKind::Rtt,
        }
    }
}

/// An observation's scope (D19): the MAC's uniqueness space and who saw it.
///
/// NOTE: this is the *observation* scope of D19 — NOT D34 §3's liveness-blindness scope
/// (`(connector, scope)`, the smallest set that can go blind), which is a separate type
/// built later with source liveness. They share a word, not a meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub l2_domain: L2DomainId,
    pub vantage: VantageId,
}

/// Which [`FactKind`]s a source CAN emit, as of a moment. A DATED FACT, not a constant
/// (D34 §1): it travels with a batch so the engine can tell "no `Uplink` because there is
/// none" from "no `Uplink` because this source is blind to topology" (false-merge
/// prevention, D19), and so a capability downgrade is a diff `caps(N-1) -> caps(N)` (FR5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    pub as_of: Timestamp,
    pub kinds: BTreeSet<FactKind>,
}

impl Capabilities {
    /// Whether the source could emit facts of this kind as of `as_of`.
    pub fn can_emit(&self, kind: FactKind) -> bool {
        self.kinds.contains(&kind)
    }
}

/// What a source saw, in one batch item. Dated by the source; the engine never touches the
/// clock. `raw` is opaque provenance (the source's original payload as text) that NO
/// decision ever reads (D19) — kept as a `String` so `opencmdb-core` need not depend on a
/// JSON type for a field nothing inspects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub obs_id: ObsId,
    pub connector_id: ConnectorId,
    pub observed_at: Timestamp,
    pub scope: Scope,
    pub facts: Vec<Fact>,
    pub raw: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn ts(s: &str) -> Timestamp {
        chrono::DateTime::parse_from_rfc3339(s)
            .unwrap()
            .with_timezone(&chrono::Utc)
    }

    #[test]
    fn mac_roundtrips_through_parse_and_display() {
        let m = MacAddr::from_str("0a:1b:2c:3d:4e:5f").unwrap();
        assert_eq!(m.0, [0x0a, 0x1b, 0x2c, 0x3d, 0x4e, 0x5f]);
        assert_eq!(m.to_string(), "0a:1b:2c:3d:4e:5f");
        // Display is lowercase regardless of input case.
        assert_eq!(
            MacAddr::from_str("AA:BB:CC:DD:EE:FF").unwrap().to_string(),
            "aa:bb:cc:dd:ee:ff"
        );
    }

    #[test]
    fn mac_rejects_malformed_text() {
        for bad in [
            "",
            "aa:bb:cc:dd:ee",
            "aa:bb:cc:dd:ee:ff:00",
            "aa:bb:cc:dd:ee:gg",
            "aabbccddeeff",
            "a:b:c:d:e:f",
        ] {
            assert!(MacAddr::from_str(bad).is_err(), "should reject {bad:?}");
        }
    }

    #[test]
    fn locally_administered_reads_the_ul_bit() {
        // U/L bit set (0x02) -> locally administered.
        assert!(
            MacAddr::from_str("02:00:00:00:00:00")
                .unwrap()
                .is_locally_administered()
        );
        assert!(
            MacAddr::from_str("0a:00:00:00:00:00")
                .unwrap()
                .is_locally_administered()
        );
        // Globally unique (bit clear).
        assert!(
            !MacAddr::from_str("00:11:22:33:44:55")
                .unwrap()
                .is_locally_administered()
        );
        assert!(
            !MacAddr::from_str("08:00:27:00:00:00")
                .unwrap()
                .is_locally_administered()
        );
    }

    #[test]
    fn fact_kind_maps_every_variant() {
        let mac = MacAddr([0; 6]);
        let cases = [
            (
                Fact::Mac {
                    addr: mac,
                    locally_administered: false,
                },
                FactKind::Mac,
            ),
            (
                Fact::IpV4 {
                    addr: Ipv4Addr::new(192, 0, 2, 1),
                },
                FactKind::IpV4,
            ),
            (
                Fact::Hostname {
                    name: "h".into(),
                    source: HostnameSource::Dhcp,
                },
                FactKind::Hostname,
            ),
            (
                Fact::DhcpLease {
                    ip: Ipv4Addr::new(192, 0, 2, 2),
                    expires_at: None,
                },
                FactKind::DhcpLease,
            ),
            (
                Fact::Uplink {
                    peer_mac: mac,
                    peer_port: "1".into(),
                },
                FactKind::Uplink,
            ),
            (Fact::OuiVendor { vendor: "v".into() }, FactKind::OuiVendor),
            (Fact::Rtt { millis: 3 }, FactKind::Rtt),
        ];
        for (fact, kind) in cases {
            assert_eq!(fact.kind(), kind);
        }
    }

    #[test]
    fn capabilities_answers_can_emit() {
        let caps = Capabilities {
            as_of: ts("2026-07-20T12:00:00Z"),
            kinds: [FactKind::Mac, FactKind::IpV4].into_iter().collect(),
        };
        assert!(caps.can_emit(FactKind::Mac));
        assert!(!caps.can_emit(FactKind::Uplink)); // blind to topology -> not "no uplink"
    }

    /// An unknown or misspelled field must FAIL, not be ignored. The fixture corpus is an
    /// oracle: a line that parses while meaning something other than it says is worse than a
    /// line that does not parse at all.
    #[test]
    fn an_unknown_field_is_refused_at_every_level() {
        let obs = r#"{"obs_id":"00000000-0000-0000-0000-000000000000","connector_id":"00000000-0000-0000-0000-000000000000","observed_at":"1970-01-01T00:00:00Z","scope":{"l2_domain":"00000000-0000-0000-0000-000000000000","vantage":"00000000-0000-0000-0000-000000000000"},"facts":[],"raw":null,"gone":true}"#;
        assert!(
            serde_json::from_str::<Observation>(obs).is_err(),
            "an unknown field on Observation must be refused"
        );

        let scope = r#"{"l2_domain":"00000000-0000-0000-0000-000000000000","vantage":"00000000-0000-0000-0000-000000000000","extra":1}"#;
        assert!(
            serde_json::from_str::<Scope>(scope).is_err(),
            "an unknown field on Scope must be refused"
        );

        // The misspelling that motivated this: the correct field is missing, but a reader that
        // tolerated the typo would report a confusing "missing field" instead of naming it.
        let fact = r#"{"Mac":{"addr":[2,0,0,0,0,1],"locally_administered":true,"locally_administred":true}}"#;
        assert!(
            serde_json::from_str::<Fact>(fact).is_err(),
            "an unknown field on a Fact variant must be refused"
        );
    }

    #[test]
    fn observation_serde_roundtrips() {
        let obs = Observation {
            obs_id: ObsId::from_uuid(Uuid::nil()),
            connector_id: ConnectorId::from_uuid(Uuid::nil()),
            observed_at: ts("2026-07-20T12:00:00Z"),
            scope: Scope {
                l2_domain: L2DomainId::from_uuid(Uuid::nil()),
                vantage: VantageId::from_uuid(Uuid::nil()),
            },
            facts: vec![
                Fact::Mac {
                    addr: MacAddr([0x0a, 1, 2, 3, 4, 5]),
                    locally_administered: true,
                },
                Fact::Rtt { millis: 7 },
            ],
            raw: Some("{\"src\":\"opaque\"}".into()),
        };
        let json = serde_json::to_string(&obs).unwrap();
        let back: Observation = serde_json::from_str(&json).unwrap();
        assert_eq!(obs, back);
    }
}
