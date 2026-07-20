//! A minimal ARP/ping connector (ping-only) — a real source of observations.
//!
//! It pings a declared set of hosts over an UNPRIVILEGED ICMP datagram socket (no `NET_RAW`
//! where `net.ipv4.ping_group_range` permits) and emits an [`Observation`] for each host that
//! answers, carrying an `IpV4` and an `Rtt` fact, dated by the poll's `now`. It is ping-only:
//! MAC facts (ARP) are the `NET_RAW` upgrade, later.
//!
//! Wired into the running app in a later step; the contract + a gated network test prove it.
#![allow(dead_code)]

use std::collections::BTreeSet;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use opencmdb_core::connector::{Connector, ConnectorError, ObservationSink, PollSummary};
use opencmdb_core::observation::{
    Capabilities, ConnectorId, Fact, FactKind, ObsId, Observation, Scope, Timestamp,
};
use surge_ping::{Client, Config, ICMP, PingIdentifier, PingSequence};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Pings a fixed set of IPv4 targets, emitting one observation per host that replies.
pub struct ArpPingConnector {
    id: ConnectorId,
    scope: Scope,
    targets: Vec<Ipv4Addr>,
    timeout: Duration,
}

impl ArpPingConnector {
    /// A connector that will ping `targets` within `scope`, minted with `id`.
    pub fn new(id: ConnectorId, scope: Scope, targets: Vec<Ipv4Addr>) -> Self {
        Self {
            id,
            scope,
            targets,
            timeout: Duration::from_secs(1),
        }
    }
}

impl Connector for ArpPingConnector {
    fn id(&self) -> ConnectorId {
        self.id
    }

    async fn poll(
        &mut self,
        now: Timestamp,
        sink: &mut dyn ObservationSink,
        cancel: CancellationToken,
    ) -> Result<PollSummary, ConnectorError> {
        // Unprivileged ICMP: surge-ping defaults to a SOCK_DGRAM socket — no NET_RAW where the
        // kernel allows it (net.ipv4.ping_group_range).
        let config = Config::builder().kind(ICMP::V4).build();
        let client = Client::new(&config).map_err(|e| ConnectorError::Misconfigured {
            detail: format!("could not open an ICMP socket: {e}"),
        })?;

        for (i, ip) in self.targets.iter().enumerate() {
            // Cancellation point BETWEEN hosts — never mid-probe. Emitted observations survive.
            if cancel.is_cancelled() {
                return Err(ConnectorError::Cancelled);
            }
            let mut pinger = client
                .pinger(IpAddr::V4(*ip), PingIdentifier((i as u16).wrapping_add(1)))
                .await;
            pinger.timeout(self.timeout);
            if let Ok((_packet, rtt)) = pinger.ping(PingSequence(0), &[0u8; 16]).await {
                let millis = rtt.as_millis().min(u128::from(u32::MAX)) as u32;
                sink.emit(Observation {
                    obs_id: ObsId::from_uuid(Uuid::now_v7()),
                    connector_id: self.id,
                    observed_at: now,
                    scope: self.scope,
                    facts: vec![Fact::IpV4 { addr: *ip }, Fact::Rtt { millis }],
                    raw: None,
                });
            }
        }

        Ok(PollSummary {
            capabilities: Capabilities {
                as_of: now,
                kinds: BTreeSet::from([FactKind::IpV4, FactKind::Rtt]),
            },
            scopes_covered: vec![self.scope],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencmdb_core::connector::VecSink;
    use opencmdb_core::observation::{L2DomainId, VantageId};

    fn scope() -> Scope {
        Scope {
            l2_domain: L2DomainId::from_uuid(Uuid::nil()),
            vantage: VantageId::from_uuid(Uuid::nil()),
        }
    }

    fn now() -> Timestamp {
        chrono::DateTime::from_timestamp(0, 0).unwrap()
    }

    /// Universal contract invariant: a pre-cancelled poll emits nothing and returns cleanly.
    #[tokio::test]
    async fn cancelled_poll_emits_nothing() {
        let mut c = ArpPingConnector::new(
            ConnectorId::from_uuid(Uuid::nil()),
            scope(),
            vec![Ipv4Addr::LOCALHOST],
        );
        let mut sink = VecSink::default();
        let token = CancellationToken::new();
        token.cancel();
        let err = c.poll(now(), &mut sink, token).await.unwrap_err();
        assert_eq!(err, ConnectorError::Cancelled);
        assert!(sink.observations.is_empty());
    }

    /// Live network: scan loopback, which always answers. Gated on `OPENCMDB_NET_TESTS=1`
    /// (skipped in CI, where ICMP is not guaranteed). Runs unprivileged where
    /// `net.ipv4.ping_group_range` allows.
    #[tokio::test]
    async fn scans_loopback_when_enabled() {
        if std::env::var("OPENCMDB_NET_TESTS").as_deref() != Ok("1") {
            eprintln!("skipping network test: set OPENCMDB_NET_TESTS=1 to run");
            return;
        }
        let mut c = ArpPingConnector::new(
            ConnectorId::from_uuid(Uuid::nil()),
            scope(),
            vec![Ipv4Addr::LOCALHOST],
        );
        let mut sink = VecSink::default();
        c.poll(now(), &mut sink, CancellationToken::new())
            .await
            .expect("poll");
        assert_eq!(sink.observations.len(), 1, "loopback must answer");
        assert!(
            sink.observations[0]
                .facts
                .iter()
                .any(|f| matches!(f, Fact::IpV4 { addr } if *addr == Ipv4Addr::LOCALHOST)),
            "the observation carries an IpV4(127.0.0.1) fact"
        );
    }
}
