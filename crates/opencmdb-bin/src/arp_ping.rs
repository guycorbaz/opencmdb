//! A minimal ARP/ping connector (ping-only) — a real source of observations.
//!
//! It pings a declared set of hosts over an UNPRIVILEGED ICMP datagram socket (no `NET_RAW`
//! where `net.ipv4.ping_group_range` permits) and emits an [`Observation`] for each host that
//! answers, carrying an `IpV4`, an `Rtt`, a `Hostname` when the host answers to a name (since
//! 2026-09-09) and a `Mac` when the kernel's neighbour table knows one (since 2026-09-10), dated by
//! the poll's `now`.
//!
//! ⚠️ **It sends no ARP of its own.** This line read *"MAC facts (ARP) are the `NET_RAW` upgrade,
//! later"* until 2026-09-10, and it was wrong on both halves: the MAC arrives now, and it needs no
//! privilege — the ping populates the kernel's neighbour table as a side effect and
//! [`crate::neighbour`] reads it. What it needs instead is layer-2 presence, which that module's
//! doc measures.
//!
//! The name comes from a reverse lookup and only for hosts that ANSWERED, so an empty subnet
//! costs no DNS traffic at all; which resolver is asked is [`crate::reverse_dns`]'s subject, and
//! the answer decided the shape of the whole thing.
//!
//! ⚠️ It IS wired into the running app — `spawn_scan_loop` builds it from `OPENCMDB_SCAN_CIDR` and
//! drives it on a schedule. This line read *"wired in a later step"* until 2026-09-09, when the
//! blind review layer noticed the diff editing the lines above it and leaving it standing.
// 🔴 There is NO `#![allow(dead_code)]` here, and its absence is load-bearing.
// It sat at the top of this module until 2026-09-10, inherited from the days when this connector
// was not yet wired in. The code review measured its cost: with the blanket allow standing,
// severing the MAC read — `let mac = neighbours().get(&ip).copied();` becoming `let mac = None;` —
// left `neighbours()` and transitively `crate::neighbour::table()` completely unreferenced, and
// clippy STILL exited 0. So the product's new capability could be cut by a refactor with no test,
// no gate and no lint saying anything. Removing the attribute was measured to cost nothing: the
// module has no genuinely dead item. Do not put it back to silence one warning — narrow it to the
// item that needs it.

use std::collections::BTreeSet;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use futures_util::stream::{self, StreamExt};
use opencmdb_core::connector::{Connector, ConnectorError, ObservationSink, PollSummary};
use opencmdb_core::observation::{
    Capabilities, ConnectorId, Fact, FactKind, HostnameSource, MacAddr, ObsId, Observation, Scope,
    Timestamp,
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
    dns_server: Option<std::net::IpAddr>,
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
            dns_server: None,
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

    /// Ask this one server for the hostnames instead of the system resolver.
    ///
    /// `None` — the default — is the system configuration, which is what an operator who
    /// configures nothing expects. On a small network the names of the DHCP leases usually live at
    /// the box that issued them, and that box is frequently not the resolver the hosts are
    /// configured with: see [`crate::reverse_dns`] for the measurement that decided this shape.
    pub fn with_dns_server(mut self, server: Option<std::net::IpAddr>) -> Self {
        self.dns_server = server;
        self
    }
}

/// The NETWORK address and prefix a CIDR names, whatever address inside it was written.
///
/// # 🔑 One parser, so two readers cannot disagree about what a perimeter IS
///
/// `192.0.2.0/24`, `192.0.2.1/24` and `192.0.2.0/024` all name the same 254 hosts, and before
/// 2026-09-09 [`derived_connector_id`] hashed the raw string — so an operator who wrote one
/// spelling and later corrected it to another had **two sources over one network**, the retired
/// one holding a field in a conflict nothing could close. Measured at that day's code review, on
/// four spellings. The normaliser was already in this file; it was simply not consulted.
///
/// # Errors
///
/// The reason as a sentence: no `/`, an address or prefix that does not parse, a prefix above 32,
/// or a subnet below `/22` — which is refused so a fat-fingered mask cannot launch a huge scan.
pub fn parse_cidr(cidr: &str) -> Result<(Ipv4Addr, u32), String> {
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
    Ok((Ipv4Addr::from(u32::from(base) & mask), prefix))
}

/// Expand an IPv4 CIDR (`addr/prefix`) into its host addresses — excluding the network and
/// broadcast addresses for prefixes `<= 30`. Rejects prefixes below `/22` so a fat-fingered
/// subnet cannot launch a huge scan (bounds it to ~1024 hosts).
pub fn subnet_hosts(cidr: &str) -> Result<Vec<Ipv4Addr>, String> {
    let (network, prefix) = parse_cidr(cidr)?;
    let network = u32::from(network);
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
        // The reverse resolver, built once per sweep. Best-effort by design: a resolver that cannot
        // be built costs the sweep its hostnames and nothing else — the observations are what the
        // product is here for, and a name is a nicety on top of one. Rebuilt each sweep on purpose,
        // so an operator who fixes `/etc/resolv.conf` does not have to restart the product.
        let dns = match crate::reverse_dns::ReverseDns::new(self.dns_server) {
            Ok(dns) => Some(dns),
            Err(detail) => {
                tracing::warn!(%detail, "no reverse DNS this sweep — hosts stay unnamed");
                None
            }
        };
        let dns = dns.as_ref();

        let timeout = self.timeout;
        let client = &client;
        let mut probes = stream::iter(self.targets.iter().copied().enumerate())
            .map(move |(i, ip)| async move {
                let mut pinger = client
                    .pinger(IpAddr::V4(ip), PingIdentifier((i as u16).wrapping_add(1)))
                    .await;
                pinger.timeout(timeout);
                let rtt = match pinger.ping(PingSequence(0), &[0u8; 16]).await {
                    Ok((_packet, rtt)) => rtt,
                    Err(_) => return None,
                };
                // Only a host that ANSWERED is asked about: the sweep learns names for the hosts
                // it found, never for the 200-odd addresses of an empty subnet.
                let hostname = match dns {
                    Some(dns) => dns.name_of(ip).await,
                    None => None,
                };
                Some((ip, rtt, hostname))
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
            if let Some((ip, rtt, hostname)) = answer {
                let millis = rtt.as_millis().min(u128::from(u32::MAX)) as u32;
                // 🔑 **Read AFTER the reply, per host.** The kernel fills the neighbour table from
                // the reply itself, so by the time this line runs the entry exists — and reading it
                // here rather than once at the end of the sweep is what keeps the emission shape
                // and the cancellation contract intact: an observation already emitted survives a
                // cancelled scan, exactly as before. The table is a few kilobytes and this runs
                // once per ANSWERING host, once an interval.
                let mac = neighbours().get(&ip).copied();
                sink.emit(Observation {
                    obs_id: ObsId::from_uuid(Uuid::now_v7()),
                    connector_id: self.id,
                    observed_at: now,
                    scope: self.scope,
                    facts: emitted_facts(ip, millis, hostname, mac),
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
/// 🔴 **`Mac` is here, and until 2026-09-10 this paragraph said the opposite three lines above the
/// body that returns it.** It read *"`Mac` is absent, and its absence is structural"*, and drew the
/// consequence that `identity::l1::join` — which keys on `(l2_domain, mac)` — *"can place NOTHING
/// this connector produces"*. Both sentences were true for five weeks and false in the commit that
/// carried them; the blind review layer found them from the diff alone.
///
/// ⚠️ **And the reason the old sentence gave was wrong even while its conclusion held.** It called
/// the MAC *"a privilege question"*. It is not: `/proc/net/arp` needs no capability at all. What it
/// needs is **layer-2 presence** — `macvlan`, `ipvlan` or host networking — and behind an ordinary
/// bridge the table holds the gateway alone. See [`crate::neighbour`], which measured both.
///
/// 🔑 The declaration is UNCONDITIONAL: this connector declares `Mac` whatever network it sits on.
/// A descriptor says what a source is BUILT to observe, and reading nothing is what a network
/// answered, not what the source can ask (NFR7).
pub(crate) fn declared_kinds() -> BTreeSet<FactKind> {
    BTreeSet::from([
        FactKind::IpV4,
        FactKind::Rtt,
        FactKind::Hostname,
        FactKind::Mac,
    ])
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

/// The connector's identity, DERIVED from what it is and what it watches rather than minted fresh.
///
/// # 🔴 Why this is not a `Uuid::now_v7()`, since 2026-09-09
///
/// It was, and the consequence was measured on the reference deployment: **eight distinct
/// `connector_id`s in the store for ONE connector**, one per boot since July, seven of them dead.
/// `reconcile` compares the most recent sighting of each SOURCE, so a dead source's last words
/// would keep voting for ever — a host renamed after a restart would conflict with a boot that
/// ended weeks earlier, and no gesture could close it.
///
/// ⚠️ It also closes half of story 6b.8's registered defect: `page::source_label` renders the top
/// 32 bits of the id, which for a v7 UUID is a clock reading that **rolls every ≈65 seconds**, so
/// the operator-visible source name changed at every restart. It is now stable.
///
/// 🔑 The name is the connector's KIND and its perimeter, because those are what make two sources
/// different sources. Two instances watching two subnets are two sources; the same instance
/// restarted is one. ⚠️ **Changing the perimeter therefore mints a new identity**, and the
/// sightings of the old one stay in the store as a source that has stopped talking — registered,
/// and the real closure is Epic 11's source registry.
pub(crate) fn derived_connector_id(cidr: &str) -> ConnectorId {
    // ⚠️ The NETWORK, never the string the operator typed. Four spellings of one subnet minted
    // four sources until the 2026-09-09 review measured it; an unparseable value keeps its raw
    // form, because there is no network to name and the scan will refuse it anyway.
    let perimeter = parse_cidr(cidr).map_or_else(
        |_| cidr.to_string(),
        |(network, prefix)| format!("{network}/{prefix}"),
    );
    ConnectorId::from_uuid(Uuid::new_v5(
        &Uuid::NAMESPACE_OID,
        format!("opencmdb:arp_ping:{perimeter}").as_bytes(),
    ))
}

/// The neighbour table, or an empty one with the reason logged.
///
/// ⚠️ **Best-effort by design, exactly like the reverse lookup beside it.** A kernel without
/// `/proc`, a sandbox that hides it, a container behind a bridge where the table holds only the
/// gateway — all of them yield no hardware address, and none of them may cost the sweep the
/// observations it has already found. What the product must never do is invent one.
fn neighbours() -> std::collections::BTreeMap<std::net::Ipv4Addr, MacAddr> {
    match crate::neighbour::table() {
        Ok(table) => table,
        Err(detail) => {
            tracing::warn!(%detail, "no neighbour table this sweep — hosts stay without a MAC");
            std::collections::BTreeMap::new()
        }
    }
}

/// The facts one answered host yields — the emission half.
///
/// 🔴 **This is the half that USED to carry the structural zero**, because `join` reads FACTS and
/// never the descriptor: a pin on [`declared_kinds`] alone was measured GREEN while a `Fact::Mac`
/// was added here. That zero is gone — this vector carries a `Fact::Mac` now — and the sentence is
/// kept in the past tense rather than deleted, because the ASYMMETRY it names is permanent: the
/// descriptor and the emission are two independent statements, and only the second one the engine
/// reads.
///
/// # 🔑 Why the hostname is a PARAMETER and not looked up here
///
/// The test module below predicted the shape of the next fact this connector would gain: *"a MAC
/// comes from a neighbour lookup keyed on the address, which `emitted_facts(ip, millis)` cannot
/// reach"*, and warned that a fact added at the emit site inside `poll` leaves both pins green.
/// The hostname is exactly that shape — a PTR lookup keyed on the address, asynchronous, needing a
/// resolver. So it arrives here as an argument rather than being fetched at the emit site, and the
/// cross-check between what this vector carries and what [`declared_kinds`] declares keeps
/// covering it. **That closes the predicted bypass for this fact; it does not close it for the
/// next one**, which remains a connector story's to close (one construction site, or a CI that
/// sets `OPENCMDB_NET_TESTS`).
///
/// `hostname` is `None` when the host answers to no name — the ordinary case, and not a failure:
/// NFR7 forbids fabricating absence, so no fact is emitted rather than an empty one.
pub(crate) fn emitted_facts(
    addr: std::net::Ipv4Addr,
    millis: u32,
    hostname: Option<String>,
    mac: Option<MacAddr>,
) -> Vec<Fact> {
    let mut facts = vec![Fact::IpV4 { addr }, Fact::Rtt { millis }];
    if let Some(name) = hostname {
        facts.push(Fact::Hostname {
            name,
            source: HostnameSource::Dns,
        });
    }
    if let Some(addr) = mac {
        facts.push(Fact::Mac {
            // 🔑 **Derived from the address itself, never reported separately.** `MacAddr` owns the
            // U/L bit and `Fact::Mac`'s doc calls this field *"the source's CLAIM"* — a claim this
            // source is in no position to make independently, since it read the address and nothing
            // else. Deriving it keeps the two in step by construction; reporting a constant would
            // be a claim, and a wrong one for the 19 of 64 neighbours the reference network shows
            // with the bit set.
            locally_administered: addr.is_locally_administered(),
            addr,
        });
    }
    facts
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
    /// 🔴 **This pin read `!contains(&FactKind::Mac)` from story 5.14 until 2026-09-10.** It carried
    /// the DESCRIPTOR half of the structural zero: for five weeks the shipped connector declared no
    /// hardware address, so `identity::l1::join` — which keys on `(l2_domain, mac)` — could place
    /// nothing, and forty-three stories of engine, corpus and resolver ran on fixtures alone.
    ///
    /// 🔑 The sweep reads the kernel's neighbour table now. The pin is inverted rather than
    /// deleted, because *which* kinds are declared is still a decision someone takes and not a
    /// drift.
    #[test]
    fn the_ping_sweep_declares_the_four_kinds_it_can_read() {
        assert_eq!(
            declared_kinds(),
            BTreeSet::from([
                FactKind::IpV4,
                FactKind::Rtt,
                FactKind::Hostname,
                FactKind::Mac,
            ]),
            "exactly these four, so a fifth kind is a decision someone took rather than a drift"
        );
    }

    /// The hostname is DECLARED unconditionally, and that is a statement about the connector
    /// rather than about one sweep.
    ///
    /// 🔑 [`declared_kinds`] answers *what is this source BUILT to observe* — `/sources` renders it
    /// under that heading and story 6b.8's whole honesty rests on the distinction. A ping sweep is
    /// built to ask for a name; whether a given host answers to one is the sweep's business, not
    /// the descriptor's. Making the declaration conditional on `dns_server` would say *"this
    /// source cannot see hostnames"* to an operator who simply has no PTR records — the
    /// fabricated absence NFR7 forbids, on the descriptor side.
    ///
    /// ⚠️ **The blind review layer of 2026-09-09 found the first form measuring nothing about the
    /// resolver**: the assertion the test is named for sat OUTSIDE the loop, and `declared_kinds`
    /// takes no argument, so the conditional declaration this test argues against is
    /// unrepresentable in the function under test. What carries it now is the DESCRIPTOR the
    /// connector really publishes — `poll`'s `PollSummary`, built from the same connector whose
    /// resolver was varied.
    #[tokio::test]
    async fn the_hostname_is_declared_whichever_resolver_is_asked() {
        for server in [None, Some("192.0.2.53".parse().unwrap())] {
            let mut connector =
                ArpPingConnector::new(ConnectorId::from_uuid(Uuid::nil()), scope(), Vec::new())
                    .with_dns_server(server);
            assert_eq!(
                connector.dns_server, server,
                "the premise: the builder carries the operator's choice"
            );
            // An empty target list opens the socket and pings nothing, so this reaches the real
            // descriptor without touching the network.
            let mut sink = VecSink::default();
            let Ok(summary) = connector
                .poll(now(), &mut sink, CancellationToken::new())
                .await
            else {
                // No ICMP socket in this environment: the descriptor cannot be reached here, and
                // saying so is better than asserting the constant and calling it a measurement.
                return;
            };
            assert!(
                summary.capabilities.kinds.contains(&FactKind::Hostname),
                "the descriptor says what the source ASKS for, never what one network answered — \
                 and it says it with {server:?} configured"
            );
        }
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
    fn the_ping_sweep_emits_what_it_declares() {
        let facts = emitted_facts(
            "203.0.113.1".parse().expect("a documentation address"),
            7,
            Some("nas-01.home.arpa".to_string()),
            Some(MacAddr([0x00, 0x11, 0x32, 0xe9, 0x2f, 0xf8])),
        );
        assert_eq!(
            facts.iter().map(Fact::kind).collect::<BTreeSet<_>>(),
            declared_kinds(),
            "the emitted vector and the declaration agree — the cross-check whose ABSENCE let a \
             pin on one of them stay green while the other changed"
        );
        assert!(
            facts.iter().any(|f| matches!(f, Fact::Mac { .. })),
            "and the MAC is IN it: `identity::l1::join` keys on (l2_domain, mac) and reads FACTS, \
             so this vector is what decides whether anything the product scans can be placed on \
             an interface at all: {facts:?}"
        );
    }

    /// 🔴 **The `locally_administered` flag is DERIVED from the address, and the reference network
    /// is why that matters.** Measured over 64 neighbours on 2026-09-10: **19** carry the U/L bit,
    /// and **11** are Docker's `02:42:` prefix followed by the four octets of the host's own IPv4 —
    /// `02:42:c0:a8:01:0a` for `192.168.1.10`.
    ///
    /// ⚠️ *For those hosts the hardware address IS the IP address, rewritten*: it corroborates
    /// nothing, and grouping on it is grouping on the address. Not a false signal — an empty one.
    /// D13 calls a locally-administered address `Disqualifying` **as a grouping anchor** for
    /// exactly this reason, and story 5.5 deliberately did not implement that predicate at L1
    /// because two committed traps would have reddened. This flag is what a rule will read the day
    /// that decision is revisited, so it must be true rather than convenient.
    #[test]
    fn the_locally_administered_flag_is_read_from_the_address_and_not_asserted() {
        let docker = MacAddr([0x02, 0x42, 0xc0, 0xa8, 0x01, 0x0a]);
        let burned_in = MacAddr([0x00, 0x11, 0x32, 0xe9, 0x2f, 0xf8]);
        for (mac, expected) in [(docker, true), (burned_in, false)] {
            let facts = emitted_facts("203.0.113.1".parse().unwrap(), 7, None, Some(mac));
            let Some(Fact::Mac {
                locally_administered,
                ..
            }) = facts.iter().find(|f| matches!(f, Fact::Mac { .. }))
            else {
                panic!("the MAC is emitted");
            };
            assert_eq!(
                *locally_administered, expected,
                "{mac} — the connector read an address and nothing else, so the only honest \
                 claim it can make about the U/L bit is the one the address carries"
            );
        }
    }

    /// A host whose hardware address the sweep could not read emits no `Mac` fact — never a zero
    /// one. NFR7/D35: the product states what it observed and never fabricates absence, and an
    /// all-zero address is what an INCOMPLETE neighbour entry carries.
    #[test]
    fn a_host_with_no_readable_address_emits_no_mac() {
        let facts = emitted_facts("203.0.113.1".parse().unwrap(), 7, None, None);
        assert!(
            !facts.iter().any(|f| matches!(f, Fact::Mac { .. })),
            "no MAC, and no placeholder for one: {facts:?}"
        );
    }

    /// A named host carries the name, and the name is attributed to DNS.
    ///
    /// The `source` is not decoration: `HostnameSource` is what tells a later rule whether a name
    /// came from the DHCP lease, from mDNS or — here — from a PTR record, and a connector that
    /// mislabels its own provenance lies to every rule downstream of it.
    #[test]
    fn a_named_host_carries_its_name_and_says_where_it_came_from() {
        let facts = emitted_facts(
            "203.0.113.1".parse().expect("a documentation address"),
            7,
            Some("wifi01-grange.home.arpa".to_string()),
            None,
        );
        assert!(
            facts.iter().any(|f| matches!(
                f,
                Fact::Hostname { name, source }
                    if name == "wifi01-grange.home.arpa" && *source == HostnameSource::Dns
            )),
            "the PTR answer is emitted as a Hostname fact attributed to DNS, and it is what turns              a queue of bare addresses into a queue an operator recognises: {facts:?}"
        );
    }

    /// 🔴 **A host with no PTR record emits NO hostname fact — never an empty one.**
    ///
    /// 32 of the 69 addresses the reference deployment held on 2026-09-09 answer to no name, so
    /// this is the ordinary case rather than an edge. NFR7/D35: the product states what it
    /// observed and never fabricates absence, and `Fact::Hostname { name: "" }` would be an
    /// assertion that the host is called nothing — which is not what the network said.
    #[test]
    fn an_unnamed_host_emits_exactly_what_it_did_before() {
        let facts = emitted_facts(
            "203.0.113.1".parse().expect("a documentation address"),
            7,
            None,
            None,
        );
        assert_eq!(
            facts,
            vec![
                Fact::IpV4 {
                    addr: "203.0.113.1".parse().unwrap()
                },
                Fact::Rtt { millis: 7 }
            ],
            "an unnamed host is the pre-hostname vector unchanged — no empty name, no placeholder"
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

    /// 🔴 **A restart must not create a second source**, and before 2026-09-09 it did: the
    /// reference deployment's store carries EIGHT `connector_id`s for one connector, one per boot
    /// since July. `reconcile` compares the newest sighting of each source, so each dead boot
    /// would keep voting with the last thing it ever saw.
    /// ⚠️ **The stability half is pinned against a LITERAL, not against a second call.** The
    /// blind review layer of 2026-09-09 found the first form asserting purity where its name
    /// promises stability across runs: two calls in one process agree for any derivation,
    /// including a clock-derived one. This literal is what a restart — and a release — must
    /// reproduce.
    #[test]
    fn the_same_perimeter_yields_the_same_source_across_restarts() {
        assert_eq!(
            derived_connector_id("192.0.2.0/24").to_string(),
            "195df73c-b692-5608-9f18-f5a79b94939f",
            "pinned: a derivation that changed between runs would pass a self-comparison and \
             fail here, which is the whole difference between pure and stable"
        );
        assert_eq!(
            derived_connector_id("192.0.2.0/24"),
            derived_connector_id("192.0.2.0/24"),
            "the identity is derived from what the source IS, so a restart changes nothing"
        );
        // 🔴 Four spellings of ONE network are ONE source. Measured at the 2026-09-09 review:
        // hashing the raw string gave four ids, so correcting a typo in `OPENCMDB_SCAN_CIDR`
        // left two sources over one network and an unclosable conflict between them.
        for spelling in ["192.0.2.1/24", "192.0.2.77/24", "192.0.2.0/024"] {
            assert_eq!(
                derived_connector_id(spelling),
                derived_connector_id("192.0.2.0/24"),
                "{spelling:?} names the same 254 hosts, so it is the same source"
            );
        }
        assert_ne!(
            derived_connector_id("192.0.2.0/24"),
            derived_connector_id("198.51.100.0/24"),
            "and two perimeters are two sources — what they see cannot be compared as one"
        );
        // ⚠️ No `assert_ne!` against the nil UUID here: a v5 of a fixed non-empty name is nil only
        // by cryptographic accident, so it would be an assertion that cannot fail. The literal
        // above pins the value, which covers it and more.
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
