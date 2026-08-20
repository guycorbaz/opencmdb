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
                    facts: emitted_facts(ip, millis),
                    raw: None,
                });
            }
        }

        Ok(PollSummary {
            capabilities: Capabilities {
                as_of: now,
                kinds: declared_kinds(),
            },
            scopes_covered: vec![self.scope],
        })
    }
}

/// The fact kinds a ping sweep can DECLARE — the descriptor half.
///
/// # 🔴 Story 5.14 named this, and the naming is the point
///
/// This connector carries TWO independent statements of what it can see: the kinds it declares
/// (here) and the facts it actually emits ([`emitted_facts`]). They sat forty lines apart as bare
/// literals with **no cross-check**, and story 5.14's validation measured the consequence: a pin on
/// one of them stays green when the other changes. Both are named now so both can be asserted, and
/// the tests state which one carries what.
///
/// ⚠️ **`Mac` is absent, and its absence is structural, not an omission.** A ping sweep sees an
/// address and a round-trip time; reading a MAC means the neighbour table, which is a privilege
/// question a connector story owns. The consequence is that `identity::l1::join`, which keys on
/// `(l2_domain, mac)`, can place NOTHING this connector produces — see the tests below.
pub(crate) fn declared_kinds() -> BTreeSet<FactKind> {
    BTreeSet::from([FactKind::IpV4, FactKind::Rtt])
}

/// What this connector is BUILT to observe, and what it is built NOT to observe.
///
/// # Returns
///
/// `(observes, cannot_see)` — [`declared_kinds`] and its complement over [`FactKind::ALL`], each in
/// the enum's own order.
///
/// # 🔴 What this is, and what it is NOT — story 6b.8's honesty hinges on the distinction
///
/// This is **what the source is BUILT to observe**, read from a compile-time constant. It is **not**
/// what it observed at the last scan. FR7 says in so many words that the descriptor *"is not a static
/// property of the source"* and that it *"travels with each batch"* — and the product really does
/// produce that dated descriptor, once per scan, inside [`PollSummary`]. ⚠️ **Nothing persists it**,
/// so a request-time screen cannot read it, and 6b.8's arbitration was to render the static half and
/// SAY that is what it is rather than dress a constant as a measurement.
///
/// 🔑 **So `/sources` delivers FR7's static half only, and the story says so.** Persisting the real
/// descriptor has a precondition that does not exist yet — a stable source identity, which is
/// Epic 11's — and an accumulation decision that is D32's and Epic 13's. Registered with both named.
///
/// ⚠️ The complement is derived from [`FactKind::ALL`], never from a second literal list: two
/// representations of one fact drift, and the guard that keeps `ALL` complete is a cross-crate row in
/// `screens.rs`'s `every_variant_of_a_navigated_enum_is_listed_in_all` — see [`FactKind::ALL`].
pub(crate) fn observes_and_cannot_see() -> (Vec<FactKind>, Vec<FactKind>) {
    let declared = declared_kinds();
    let observes: Vec<FactKind> = FactKind::ALL
        .iter()
        .copied()
        .filter(|kind| declared.contains(kind))
        .collect();
    let cannot_see: Vec<FactKind> = FactKind::ALL
        .iter()
        .copied()
        .filter(|kind| !declared.contains(kind))
        .collect();
    (observes, cannot_see)
}

/// The name this source carries on screen.
///
/// # 🔑 A TYPE name, and that is the true sentence today
///
/// Story 6b.4 registered that **the product has no connector registry** — `observation_record
/// .connector_id` is a bare `CHAR(36)` with no table behind it — and assigned the row to story 6b.8.
/// Guy's arbitration of 2026-08-20 (option (a) refined): the product has **one connector of one kind
/// and no notion of a source INSTANCE at all**, so naming the TYPE is honest and a table would invent
/// instance identity before anything can create instances.
///
/// ⚠️ **The limit, written rather than discovered**: this does not survive a second source configured
/// at runtime (FR1's UniFi controller, or several scopes for the scanner). The day that exists, a
/// stable source identity is the answer — **Epic 11, `Source UniFi complète`**, which covers FR1,
/// FR2, FR5, FR6 and FR7.
/// 🔴 **A KEY, and it shipped as an English LITERAL until someone looked at the screen.** The story's
/// own §0c wrote the name in French — *« Balayage ARP/ping »* — and the code carried
/// `"ARP/ping sweep"`, which rendered under a fully French interface with the whole suite green.
/// **Story 6b.6's `role_key` defect verbatim, in the story that quotes it**: a literal is not a key,
/// and `every_key_carries_both_locales` can only see keys.
///
/// ⚠️ **And it is a KEY rather than data by the same rule story 6b.7 arbitrated**: an owner's name is
/// a proper noun and stays untranslated; a source's TYPE is a **classification**, and a
/// classification is copy the operator reads in their own language.
pub(crate) const SOURCE_NAME_KEY: &str = "sources.name.arp_ping";

/// The facts one answered host yields — the emission half.
///
/// 🔴 **This is the half that carries the structural zero**, because `join` reads FACTS and never
/// the descriptor. A pin on [`declared_kinds`] alone was measured GREEN while a `Fact::Mac` was
/// added here.
pub(crate) fn emitted_facts(addr: std::net::Ipv4Addr, millis: u32) -> Vec<Fact> {
    vec![Fact::IpV4 { addr }, Fact::Rtt { millis }]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// # 🔴 WHAT THESE TWO PINS DO NOT COVER — read this before trusting them
    ///
    /// Story 5.14's code review measured it: adding a `Fact::Mac` **at the emit site inside
    /// `poll`**, rather than inside [`emitted_facts`], leaves **all 502 tests green** — both pins
    /// and `scan_pass`'s structural-zero pin included — while the real binary then mints an
    /// interface and places a link where the committed tree abstains.
    ///
    /// ⚠️ And that bypass is not an adversary's trick, it is **the shape the upgrade must take**:
    /// a MAC comes from a neighbour lookup keyed on the address, which `emitted_facts(ip, millis)`
    /// cannot reach.
    ///
    /// **So the narrowed, true promise, on story 5.12's precedent: these pins are a TRIPWIRE
    /// against a change made through these two functions, never a barrier against a MAC arriving
    /// by another route.** Read them as *"the named fact set and the named descriptor still agree
    /// and still exclude the MAC"*, and never as *"nothing the shipped product emits can carry
    /// one"*. The barrier would need a test over what `poll` really emits, which needs the ICMP
    /// socket — and every such test is gated on `OPENCMDB_NET_TESTS`, which CI never sets, so a
    /// mutation against it comes back green because the test was SKIPPED.
    ///
    /// The honest closure is a connector story that routes every fact through one construction
    /// site, or a CI that sets the variable. Registered; not implied by these.
    /// **Story 5.14 AC3, first half** — the connector DECLARES no MAC.
    ///
    /// Runs everywhere, including CI: it reads a named function and opens no socket.
    ///
    /// ⚠️ **And it is NOT the pin that carries the structural zero.** See its sibling below. An
    /// earlier draft of story 5.14 pinned only this half and prescribed a mutation on the emitted
    /// facts; both validation layers measured that combination GREEN, because the two literals are
    /// independent. Whichever of the two you change, change the other's pin's expectation too — or
    /// find out that you did not.
    #[test]
    fn the_ping_sweep_declares_no_mac() {
        assert!(
            !declared_kinds().contains(&FactKind::Mac),
            "a ping sweep declares an address and a round-trip time. The day it declares a MAC,              read `emitted_facts`' pin too: the identity engine keys on what is EMITTED"
        );
        assert_eq!(
            declared_kinds(),
            BTreeSet::from([FactKind::IpV4, FactKind::Rtt]),
            "exactly these two, so a third kind is a decision someone took rather than a drift"
        );
    }

    /// **Story 5.14 AC3, second half — the pin that carries the structural zero.**
    ///
    /// `identity::l1::join` keys on `(l2_domain, mac)` and reads FACTS, not descriptors. So it is
    /// this vector, not the declaration, that makes every observation the shipped product produces
    /// fall to the abstention path — `interfaces_minted == 0`, by construction rather than by
    /// accident (see `scan_pass`'s own pin).
    ///
    /// It runs in CI because `emitted_facts` is a named function; every test that reaches a LIVE
    /// emit is gated on `OPENCMDB_NET_TESTS`, which CI never sets, so a pin written against the
    /// live path would be skipped — and a mutation against a skipped test comes back green.
    #[test]
    fn the_ping_sweep_emits_no_mac() {
        let facts = emitted_facts("203.0.113.1".parse().expect("a documentation address"), 7);
        assert!(
            !facts.iter().any(|f| matches!(f, Fact::Mac { .. })),
            "the identity engine keys on (l2_domain, mac) and reads FACTS: while this vector              carries no MAC, NOTHING the shipped product scans can ever be placed on an interface.              That is story 5.14's structural zero, and this assertion is what carries it"
        );
        assert_eq!(
            facts.iter().map(Fact::kind).collect::<BTreeSet<_>>(),
            declared_kinds(),
            "and the two halves agree today — the cross-check whose ABSENCE let a pin on one of              them stay green while the other changed"
        );
    }
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
