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

use futures_util::stream::{self, StreamExt};
use opencmdb_core::connector::{Connector, ConnectorError, ObservationSink, PollSummary};
use opencmdb_core::observation::{
    Capabilities, ConnectorId, Fact, FactKind, ObsId, Observation, Scope, Timestamp,
};
use surge_ping::{Client, Config, ICMP, PingIdentifier, PingSequence};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// How many probes are in flight at once by default. The scan is I/O-bound — a probe sends 16
/// bytes and waits — so this is a politeness bound, not a throughput one: it caps how hard we
/// hit the gateway's ARP table, which matters on a /22 (1022 hosts). See `with_concurrency`.
pub const DEFAULT_CONCURRENCY: usize = 64;

/// How long a single probe waits for its reply, by default.
///
/// This is the knob that decides what a scan MISSES. Exactly one probe is sent per host and
/// there is no retry yet, so a device that answers more slowly than this — a sleeping wireless
/// client, a congested link — is recorded as absent rather than as unknown. It is also, together
/// with the concurrency, what a scan of a mostly-empty subnet costs: `targets / concurrency`
/// rounds of this timeout.
pub const DEFAULT_TIMEOUT_MS: u64 = 1_000;

/// Pings a fixed set of IPv4 targets, emitting one observation per host that replies.
pub struct ArpPingConnector {
    id: ConnectorId,
    scope: Scope,
    targets: Vec<Ipv4Addr>,
    timeout: Duration,
    concurrency: usize,
}

impl ArpPingConnector {
    /// A connector that will ping `targets` within `scope`, minted with `id`.
    pub fn new(id: ConnectorId, scope: Scope, targets: Vec<Ipv4Addr>) -> Self {
        Self {
            id,
            scope,
            targets,
            timeout: Duration::from_millis(DEFAULT_TIMEOUT_MS),
            concurrency: DEFAULT_CONCURRENCY,
        }
    }

    /// Build a connector for a declared IPv4 subnet in CIDR form (e.g. `192.0.2.0/24`).
    pub fn from_cidr(id: ConnectorId, scope: Scope, cidr: &str) -> Result<Self, String> {
        Ok(Self::new(id, scope, subnet_hosts(cidr)?))
    }

    /// Cap the number of probes in flight. Zero is meaningless and is clamped to one, which
    /// restores the fully sequential behaviour.
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency.max(1);
        self
    }

    /// How long a single probe waits for its reply. This is what a scan of a mostly-empty
    /// subnet actually costs, once probes overlap: `targets / concurrency` rounds of it.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Expand an IPv4 CIDR (`addr/prefix`) into its host addresses — excluding the network and
/// broadcast addresses for prefixes `<= 30`. Rejects prefixes below `/22` so a fat-fingered
/// subnet cannot launch a huge scan (bounds it to ~1024 hosts).
pub fn subnet_hosts(cidr: &str) -> Result<Vec<Ipv4Addr>, String> {
    let (addr, prefix) = cidr.split_once('/').ok_or("expected `address/prefix`")?;
    let base: Ipv4Addr = addr
        .parse()
        .map_err(|_| format!("bad IPv4 address: {addr}"))?;
    let prefix: u32 = prefix
        .parse()
        .map_err(|_| format!("bad prefix: {prefix}"))?;
    if prefix > 32 {
        return Err(format!("prefix /{prefix} exceeds /32"));
    }
    if prefix < 22 {
        return Err(format!("subnet /{prefix} too large — use /22 or smaller"));
    }
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    let network = u32::from(base) & mask;
    let count = 1u32 << (32 - prefix);
    let mut hosts = Vec::new();
    for i in 0..count {
        // Skip the network (.0) and broadcast (last) addresses on real subnets (prefix <= 30).
        if prefix <= 30 && (i == 0 || i == count - 1) {
            continue;
        }
        hosts.push(Ipv4Addr::from(network + i));
    }
    Ok(hosts)
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
        // Cancellation point BEFORE any work — a pre-cancelled poll opens no socket and touches
        // no network (also why this returns Cancelled, not Misconfigured, on a cancelled start).
        if cancel.is_cancelled() {
            return Err(ConnectorError::Cancelled);
        }
        // Unprivileged ICMP: surge-ping defaults to a SOCK_DGRAM socket — no NET_RAW where the
        // kernel allows it (net.ipv4.ping_group_range).
        let config = Config::builder().kind(ICMP::V4).build();
        let client = Client::new(&config).map_err(|e| ConnectorError::Misconfigured {
            detail: format!("could not open an ICMP socket: {e}"),
        })?;

        // Probes overlap, up to `concurrency` of them in flight. This is NOT parallelism: a
        // single `Client` multiplexes every probe over ONE socket and demultiplexes replies by
        // `PingIdentifier`, so the whole scan runs on one thread. Sequentially, wall-clock was
        // `dead_hosts * timeout` — a /24 with 44 live hosts took 3m33s in the field, and the
        // cost grew with the SIZE OF THE SUBNET rather than with the number of devices found.
        //
        // `buffered` (not `buffer_unordered`) preserves target order, so the observations a
        // scan emits are deterministic — the connector contract tests depend on it.
        let timeout = self.timeout;
        let client = &client;
        let mut probes = stream::iter(self.targets.iter().copied().enumerate())
            .map(move |(i, ip)| async move {
                let mut pinger = client
                    .pinger(IpAddr::V4(ip), PingIdentifier((i as u16).wrapping_add(1)))
                    .await;
                pinger.timeout(timeout);
                match pinger.ping(PingSequence(0), &[0u8; 16]).await {
                    Ok((_packet, rtt)) => Some((ip, rtt)),
                    Err(_) => None,
                }
            })
            .buffered(self.concurrency);

        loop {
            // Cancellation is now checked against the probe stream itself, so a cancelled scan
            // stops promptly instead of running to the end of the current wave. Observations
            // already emitted survive, exactly as before.
            let next = tokio::select! {
                biased;
                () = cancel.cancelled() => return Err(ConnectorError::Cancelled),
                next = probes.next() => next,
            };
            let Some(answer) = next else { break };
            if let Some((ip, rtt)) = answer {
                let millis = rtt.as_millis().min(u128::from(u32::MAX)) as u32;
                sink.emit(Observation {
                    obs_id: ObsId::from_uuid(Uuid::now_v7()),
                    connector_id: self.id,
                    observed_at: now,
                    scope: self.scope,
                    facts: vec![Fact::IpV4 { addr: ip }, Fact::Rtt { millis }],
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

    #[test]
    fn subnet_hosts_expands_and_bounds() {
        // /30 → 2 usable hosts (.1, .2), skipping network/broadcast.
        assert_eq!(
            subnet_hosts("192.0.2.0/30").unwrap(),
            vec![Ipv4Addr::new(192, 0, 2, 1), Ipv4Addr::new(192, 0, 2, 2)]
        );
        // /24 → 254 usable hosts.
        assert_eq!(subnet_hosts("192.0.2.0/24").unwrap().len(), 254);
        // A too-large subnet and malformed input are rejected.
        assert!(subnet_hosts("10.0.0.0/8").is_err());
        assert!(subnet_hosts("not-a-cidr").is_err());
        assert!(subnet_hosts("192.0.2.0/33").is_err());
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

    #[test]
    fn concurrency_is_bounded_below_by_one() {
        let c = ArpPingConnector::new(ConnectorId::from_uuid(Uuid::nil()), scope(), vec![])
            .with_concurrency(0);
        // Zero probes in flight would never make progress; it means "sequential", not "stall".
        assert_eq!(c.concurrency, 1);
    }

    /// The point of issue #10: a scan of a mostly-dead subnet must cost roughly
    /// `targets / concurrency` timeouts, NOT `targets` timeouts. Uses TEST-NET-1 addresses,
    /// which are unroutable and therefore all time out — the worst case, and the one that made
    /// a real /24 take 3m33s. Gated with the other network tests because it still needs an
    /// ICMP socket.
    #[tokio::test]
    async fn dead_targets_overlap_instead_of_queueing() {
        if std::env::var("OPENCMDB_NET_TESTS").as_deref() != Ok("1") {
            eprintln!("skipping network test: set OPENCMDB_NET_TESTS=1 to run");
            return;
        }
        let targets: Vec<Ipv4Addr> = (1..=128).map(|i| Ipv4Addr::new(192, 0, 2, i)).collect();
        let mut c = ArpPingConnector::new(ConnectorId::from_uuid(Uuid::nil()), scope(), targets)
            .with_concurrency(64)
            .with_timeout(Duration::from_millis(200));
        let mut sink = VecSink::default();
        let started = std::time::Instant::now();
        c.poll(now(), &mut sink, CancellationToken::new())
            .await
            .expect("poll");
        let elapsed = started.elapsed();

        assert!(sink.observations.is_empty(), "TEST-NET-1 must not answer");
        // Sequentially this is 128 * 200ms = 25.6s. Overlapped it is 2 rounds ~= 400ms. The
        // bound is deliberately loose (10x headroom) so a slow machine cannot make it flaky,
        // while still failing outright if the probes ever go back to queueing.
        assert!(
            elapsed < Duration::from_secs(4),
            "128 dead targets at 64 in flight took {elapsed:?} — probes are not overlapping"
        );
    }

    /// Overlapping probes must not reorder the results: `buffered` preserves target order, and
    /// the connector contract wants a deterministic scan. Every 127.0.0.0/8 address answers on
    /// Linux, so this exercises the concurrent path with replies actually racing each other.
    #[tokio::test]
    async fn concurrent_probes_emit_in_target_order() {
        if std::env::var("OPENCMDB_NET_TESTS").as_deref() != Ok("1") {
            eprintln!("skipping network test: set OPENCMDB_NET_TESTS=1 to run");
            return;
        }
        let targets: Vec<Ipv4Addr> = (1..=8).map(|i| Ipv4Addr::new(127, 0, 0, i)).collect();
        let mut c = ArpPingConnector::new(
            ConnectorId::from_uuid(Uuid::nil()),
            scope(),
            targets.clone(),
        )
        .with_concurrency(8);
        let mut sink = VecSink::default();
        c.poll(now(), &mut sink, CancellationToken::new())
            .await
            .expect("poll");

        let seen: Vec<Ipv4Addr> = sink
            .observations
            .iter()
            .filter_map(|o| {
                o.facts.iter().find_map(|f| match f {
                    Fact::IpV4 { addr } => Some(*addr),
                    _ => None,
                })
            })
            .collect();
        assert_eq!(seen, targets, "observations must follow target order");
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
