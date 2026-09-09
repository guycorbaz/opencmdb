//! Reverse DNS — the one lookup that turns a bare address into something an operator recognises.
//!
//! The triage queue's `Nouveau` rows are addresses. On a real network that is a list of
//! `192.168.1.171`, `192.168.1.173`, `192.168.1.177` — measured on the reference deployment after
//! ten days of unattended running: **69 questions, every one a bare address**. A PTR record turns
//! that into `wifi01-grange`, `swiss-domotique-plug-5751f8`, `sw03`. It is the cheapest fact this
//! product can learn about a host it can only ping.
//!
//! # 🔑 Which resolver is asked decides everything, and it was MEASURED before it was chosen
//!
//! On the reference LAN, over the 69 addresses the queue held on 2026-09-09:
//!
//! | asked | what it is | names obtained |
//! |---|---|---|
//! | the system resolver | `/etc/resolv.conf` | **2** |
//! | the DHCP server | the router that handed the leases out | **37** |
//!
//! A home or small-office network resolves its own hosts at the box that leases the addresses,
//! and that box is very often NOT the resolver the machines are configured with (an ad-blocking
//! forwarder usually sits in between). So the default is the system resolver — the unsurprising
//! behaviour, no configuration — and `OPENCMDB_DNS_SERVER` points the scan somewhere else when the
//! operator knows better. Guy's arbitration, 2026-09-09, on the measurement above.
//!
//! # Absence is not a failure
//!
//! A host with no PTR record is the ordinary case (32 of the 69 above). It yields no
//! [`Fact::Hostname`](opencmdb_core::observation::Fact::Hostname) and nothing else changes — NFR7:
//! the product never fabricates absence, and a name it could not learn is not a name that is gone.

use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use hickory_resolver::config::{NameServerConfig, ResolverConfig};
use hickory_resolver::net::runtime::TokioRuntimeProvider;
use hickory_resolver::proto::rr::RData;
use hickory_resolver::{Resolver, TokioResolver};

/// How long one reverse lookup may take before the scan gives up on the name.
///
/// The sweep must not be held hostage by a slow or dead resolver: it is what produces the
/// observations, and a name is a nicety on top of one. With one attempt and this timeout, a
/// resolver that answers nothing costs the sweep `targets / concurrency` rounds of two seconds in
/// the worst case, and nothing at all in the ordinary one (an absent PTR is an immediate NXDOMAIN,
/// not a timeout).
const LOOKUP_TIMEOUT: Duration = Duration::from_secs(2);

/// How many times one lookup is tried. One: a retry doubles the worst case to buy a name.
const LOOKUP_ATTEMPTS: usize = 1;

/// The longest name a PTR answer may carry, per RFC 1035 §2.3.4. A longer one is refused rather
/// than truncated — a truncated hostname is a WRONG hostname, and this product does not guess.
const MAX_NAME_LEN: usize = 253;

/// A reverse-DNS lookup, pointed either at the system resolver or at one named address.
pub(crate) struct ReverseDns {
    resolver: TokioResolver,
}

impl ReverseDns {
    /// Build a resolver. `server` is `None` for the system configuration (`/etc/resolv.conf`), or
    /// the address of the one server to ask — typically the DHCP server, which is the only host on
    /// a small network that knows the names of the leases it handed out.
    ///
    /// # Errors
    ///
    /// Returns the reason as a sentence when the system configuration cannot be read — a
    /// container image with no `/etc/resolv.conf`, for instance. The caller logs it and sweeps
    /// without names; a scan that cannot name its hosts is still a scan.
    pub(crate) fn new(server: Option<IpAddr>) -> Result<Self, String> {
        let mut builder = match server {
            Some(ip) => Resolver::builder_with_config(
                ResolverConfig::from_parts(None, vec![], vec![NameServerConfig::udp_and_tcp(ip)]),
                TokioRuntimeProvider::default(),
            ),
            None => Resolver::builder_tokio()
                .map_err(|error| format!("could not read the system resolver config: {error}"))?,
        };
        // Only these two options are overridden; everything else the system configuration supplies
        // is kept. `with_options` would replace the whole set, `ndots` and `search` included.
        let options = builder.options_mut();
        options.timeout = LOOKUP_TIMEOUT;
        options.attempts = LOOKUP_ATTEMPTS;
        let resolver = builder
            .build()
            .map_err(|error| format!("could not build the reverse resolver: {error}"))?;
        Ok(Self { resolver })
    }

    /// The name `ip` answers to, or `None` when it answers to none this product will repeat.
    ///
    /// Every failure mode collapses to `None` on purpose: no PTR record, a refusing server, a
    /// timeout and a name [`sanitise`] refuses are the same thing to the caller — *no name was
    /// learnt* — and none of them may stop a sweep that has already found the host.
    pub(crate) async fn name_of(&self, ip: Ipv4Addr) -> Option<String> {
        let lookup = self.resolver.reverse_lookup(ip).await.ok()?;
        // The FIRST PTR record, which is what every resolver library does. A host with two PTR
        // records is answering two names and this product picks neither as *the* name; it repeats
        // the first, deterministically, and the day that matters is the day a rule reads it.
        lookup
            .answers()
            .iter()
            .filter_map(|record| match &record.data {
                RData::PTR(ptr) => Some(ptr.0.to_string()),
                _ => None,
            })
            .find_map(|name| sanitise(&name))
    }
}

/// Turn a PTR answer into a hostname this product is willing to repeat, or refuse it.
///
/// # 🔴 A name that carries no letter and no digit is a refusal to answer, not an answer
///
/// Story 6.7 established the property for `l2-different-hostname` and it is the same property
/// here, one layer earlier: `"---"`, `"."` and a string of invisible characters are not names, and
/// admitting one would put it on the operator's screen, into a declared record the moment they
/// document the row, and — the day a MAC arrives — in front of a rule that opposes on hostnames.
/// **The check is a PROPERTY (at least one ASCII alphanumeric), never a list of bad shapes**: an
/// enumeration cannot claim the completeness of a property (story 5.12's sentence).
///
/// The trailing dot of a fully-qualified name is dropped, because it is DNS syntax rather than
/// part of the name — `wifi01-grange.home.arpa.` is displayed and declared as
/// `wifi01-grange.home.arpa`.
///
/// ⚠️ The name is kept WHOLE rather than shortened to its first label. Two hosts in two domains
/// can share a first label, the declared side compares strings, and a product that silently
/// documents `iPhone` for `iPhone.home.arpa` would be documenting something the network never
/// said.
pub(crate) fn sanitise(raw: &str) -> Option<String> {
    let name = raw.trim().trim_end_matches('.');
    if name.is_empty() || name.len() > MAX_NAME_LEN {
        return None;
    }
    if !name.chars().any(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    Some(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_the_trailing_dot_of_a_fully_qualified_name() {
        assert_eq!(
            sanitise("wifi01-grange.home.arpa."),
            Some("wifi01-grange.home.arpa".to_string()),
            "the trailing dot is DNS syntax, not part of the name"
        );
    }

    #[test]
    fn keeps_the_whole_name_never_the_first_label() {
        assert_eq!(
            sanitise("iPhone.home.arpa."),
            Some("iPhone.home.arpa".to_string()),
            "shortening to `iPhone` would declare something the network never said"
        );
    }

    #[test]
    fn refuses_a_name_carrying_no_letter_and_no_digit() {
        // Story 6.7's property, one layer earlier. Each of these RESOLVES as a DNS name and none
        // of them is a hostname.
        for junk in ["---", ".", "..", "-", "\u{200b}", "  \u{2062} "] {
            assert_eq!(
                sanitise(junk),
                None,
                "{junk:?} carries no ASCII alphanumeric and is not a name this product repeats"
            );
        }
    }

    #[test]
    fn refuses_an_empty_answer() {
        assert_eq!(sanitise(""), None);
        assert_eq!(sanitise("   "), None);
    }

    #[test]
    fn refuses_a_name_longer_than_dns_permits() {
        let too_long = format!("{}.home.arpa.", "a".repeat(MAX_NAME_LEN));
        assert_eq!(
            sanitise(&too_long),
            None,
            "refused whole rather than truncated — a truncated hostname is a wrong hostname"
        );
        let at_the_limit = "b".repeat(MAX_NAME_LEN);
        assert_eq!(
            sanitise(&at_the_limit),
            Some(at_the_limit.clone()),
            "the limit itself is admitted; the refusal starts one character later"
        );
    }

    #[test]
    fn a_single_letter_is_a_name() {
        assert_eq!(sanitise("a."), Some("a".to_string()));
    }

    /// The lookup itself needs a network, so what is asserted here is the part that does not: a
    /// resolver aimed at a named server is built without touching one.
    #[test]
    fn a_resolver_aimed_at_a_named_server_needs_no_system_configuration() {
        let dns = ReverseDns::new(Some("192.0.2.53".parse().unwrap()));
        assert!(
            dns.is_ok(),
            "an explicit server is a complete configuration on its own"
        );
    }
}
