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
pub struct Scope {
    pub l2_domain: L2DomainId,
    pub vantage: VantageId,
}

/// Which [`FactKind`]s a source CAN emit, as of a moment. A DATED FACT, not a constant
/// (D34 §1): it travels with a batch so the engine can tell "no `Uplink` because there is
/// none" from "no `Uplink` because this source is blind to topology" (false-merge
/// prevention, D19), and so a capability downgrade is a diff `caps(N-1) -> caps(N)` (FR5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
