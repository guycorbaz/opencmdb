//! The kernel's neighbour table — where a ping sweep learns a hardware address.
//!
//! # 🔴 Why this file removes a shield rather than adding a feature
//!
//! `identity::l1::join` keys on `(l2_domain, mac)`, so an observation carrying no MAC lands on no
//! interface, and **no interface means the identity engine never runs**. That is story 5.14's
//! *structural zero*: forty-three stories of engine, corpus and resolver, pinned by a test
//! asserting `interfaces_minted == 0` because the shipped connector could not produce the one fact
//! the engine keys on.
//!
//! ⚠️ The shield was load-bearing. `deferred-work.md` records that **two concurrent passes mint two
//! interfaces for one MAC, both reporting success** — `interface_l1_key` is a plain index and the
//! mint is read-then-insert — and that *"the connector story that gives it a MAC REMOVES THAT
//! SHIELD and must carry this race with it"*. It is carried in the same change as this file.
//!
//! # What it reads, and what that costs
//!
//! `/proc/net/arp`, the IPv4 neighbour table, after the sweep has pinged a host: the kernel fills
//! it from the reply. No capability is needed beyond opening a file — measured from an unprivileged
//! container.
//!
//! ⚠️ **It only works where the container has real layer-2 presence** — `macvlan`, `ipvlan` or host
//! networking. Behind an ordinary bridge the table holds the gateway and nothing else, because the
//! host answers every ARP on the container's behalf. Measured on 2026-09-10: from a macvlan
//! container, `192.168.1.5 → 00:11:32:e9:2f:f8`; the same ping from a bridge container yields no
//! entry for it at all.
//!
//! 🔑 That is an ABSENCE, not a lesser capability. The connector declares `Mac` whatever network it
//! sits on, for the reason [`crate::arp_ping::declared_kinds`] gives about the hostname: a
//! descriptor says what a source is BUILT to observe, and reading nothing is what a network
//! answered, not what the source can ask. NFR7 forbids fabricating the difference.

use std::collections::BTreeMap;
use std::net::Ipv4Addr;

use opencmdb_core::observation::MacAddr;

/// Where the IPv4 neighbour table lives on Linux.
///
/// ⚠️ `/proc/net/arp` is the legacy interface — netlink is its successor and carries IPv6 too. It
/// is chosen here because it needs no crate, no socket and no capability, and because this
/// connector is IPv4-only anyway (`Fact::IpV4`, `subnet_hosts`). The day an IPv6 sweep exists, this
/// is the file that has to change, and its doc is where that will be read.
const ARP_TABLE: &str = "/proc/net/arp";

/// The flag bit the kernel sets on a COMPLETE entry (`ATF_COM`).
///
/// An incomplete entry carries `0x0` and an all-zero hardware address — the kernel saying *I asked
/// and nobody answered yet*. Repeating that as a fact would be inventing one.
const ATF_COM: u32 = 0x2;

/// Read the neighbour table: the hardware address the kernel currently associates with each IPv4.
///
/// # Errors
///
/// The reason as a sentence when the table cannot be read at all — a kernel without `/proc`, a
/// sandbox that hides it. The caller logs it and sweeps without hardware addresses; a scan that
/// cannot read a MAC is still a scan.
pub(crate) fn table() -> Result<BTreeMap<Ipv4Addr, MacAddr>, String> {
    let text = std::fs::read_to_string(ARP_TABLE)
        .map_err(|error| format!("could not read {ARP_TABLE}: {error}"))?;
    Ok(parse(&text))
}

/// Parse the table's text. Separated from the read so the parsing has tests that need no `/proc`.
///
/// Unparseable lines are SKIPPED rather than failing the table: this file's format is stable but
/// not a contract, and one malformed row must not cost the sweep every other host's address.
pub(crate) fn parse(text: &str) -> BTreeMap<Ipv4Addr, MacAddr> {
    let mut out = BTreeMap::new();
    // The first line is the column header, and it parses as nothing anyway.
    for line in text.lines().skip(1) {
        let mut fields = line.split_whitespace();
        let (Some(address), Some(_hw_type), Some(flags), Some(hardware)) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let Ok(address) = address.parse::<Ipv4Addr>() else {
            continue;
        };
        // The flags are hexadecimal with an `0x` prefix.
        let complete = flags
            .strip_prefix("0x")
            .and_then(|hex| u32::from_str_radix(hex, 16).ok())
            .is_some_and(|flags| flags & ATF_COM != 0);
        if !complete {
            continue;
        }
        let Some(mac) = parse_mac(hardware) else {
            continue;
        };
        // ⚠️ The all-zero address is what an incomplete entry carries, and it is also a legal MAC
        // nobody uses. Refused for the same reason `reverse_dns::sanitise` refuses a name with no
        // letter: repeating it would put a non-answer on the operator's screen and, worse, into
        // the identity engine as a key that groups every unanswered host onto one interface.
        if mac.0 == [0; 6] {
            continue;
        }
        out.insert(address, mac);
    }
    out
}

/// `aa:bb:cc:dd:ee:ff` as the kernel writes it.
fn parse_mac(text: &str) -> Option<MacAddr> {
    let mut octets = [0_u8; 6];
    let mut parts = text.split(':');
    for octet in &mut octets {
        *octet = u8::from_str_radix(parts.next()?, 16).ok()?;
    }
    // Exactly six, so a longer address is refused rather than truncated to something plausible.
    if parts.next().is_some() {
        return None;
    }
    Some(MacAddr(octets))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The real shape, copied from a live macvlan container on 2026-09-10.
    const LIVE: &str =
        "IP address       HW type     Flags       HW address            Mask     Device
192.168.1.5      0x1         0x2         00:11:32:e9:2f:f8     *        eth0
192.168.1.40     0x1         0x2         a4:06:e9:5b:30:6d     *        eth0";

    #[test]
    fn reads_the_table_the_kernel_really_writes() {
        let table = parse(LIVE);
        assert_eq!(table.len(), 2);
        assert_eq!(
            table.get(&"192.168.1.5".parse().unwrap()),
            Some(&MacAddr([0x00, 0x11, 0x32, 0xe9, 0x2f, 0xf8])),
            "the header line is skipped and the columns are read in the kernel's order"
        );
    }

    /// 🔴 **An INCOMPLETE entry is not an address.** The kernel writes `0x0` and an all-zero
    /// hardware address while it is still asking — *I have not been told yet*. Emitting that would
    /// hand the identity engine a key on which every unanswered host groups together, which is a
    /// false MERGE, and a false merge cannot be undone by looking harder (D20).
    #[test]
    fn an_incomplete_entry_is_not_an_address() {
        let text = "IP address       HW type     Flags       HW address            Mask     Device
192.168.1.7      0x1         0x0         00:00:00:00:00:00     *        eth0
192.168.1.8      0x1         0x2         00:08:9b:ed:53:fe     *        eth0";
        let table = parse(text);
        assert_eq!(
            table.get(&"192.168.1.7".parse().unwrap()),
            None,
            "the flags say the kernel is still asking"
        );
        assert_eq!(table.len(), 1, "and the complete entry beside it is kept");
    }

    /// 🔴 **The FLAGS check is carried by this test and by nothing else**, and the mutation pass is
    /// what showed it. `an_incomplete_entry_is_not_an_address` uses the shape the kernel really
    /// writes — flags `0x0` beside an all-zero address — so the zero check below catches it too:
    /// deleting the flags check left every test green and reddened only clippy, on the constant
    /// going unused. *Two guards over one specimen measure one guard.*
    ///
    /// This entry carries a STALE address with the incomplete flag, which is what the kernel writes
    /// while it re-resolves a neighbour it already knew. The address is real and the kernel is
    /// saying it no longer stands behind it.
    #[test]
    fn an_incomplete_entry_is_refused_even_when_it_carries_an_address() {
        let text = "IP address       HW type     Flags       HW address            Mask     Device
192.168.1.7      0x1         0x0         00:08:9b:ed:53:fe     *        eth0";
        assert!(
            parse(text).is_empty(),
            "the kernel says it is still asking, whatever address the row still shows"
        );
    }

    /// The all-zero address is refused even when the flags claim the entry is complete — the two
    /// checks are independent, because one is the kernel's opinion and the other is the value.
    #[test]
    fn an_all_zero_address_is_refused_whatever_the_flags_say() {
        let text = "IP address       HW type     Flags       HW address            Mask     Device
192.168.1.7      0x1         0x2         00:00:00:00:00:00     *        eth0";
        assert!(parse(text).is_empty());
    }

    /// A malformed row costs its own host and nothing else.
    #[test]
    fn one_bad_row_does_not_cost_the_table() {
        let text = "IP address       HW type     Flags       HW address            Mask     Device
not-an-address   0x1         0x2         00:11:32:e9:2f:f8     *        eth0
192.168.1.8      0x1         0x2         zz:zz:zz:zz:zz:zz     *        eth0
192.168.1.9      0x1         0x2         00:08:9b:ed:53:fe     *        eth0
192.168.1.10     0x1
192.168.1.11     0x1         0x2         00:08:9b:ed:53                 eth0";
        let table = parse(text);
        assert_eq!(
            table.keys().copied().collect::<Vec<_>>(),
            vec!["192.168.1.9".parse::<Ipv4Addr>().unwrap()],
            "a bad address, a bad MAC, a short row and a truncated address are each skipped alone"
        );
    }

    /// A six-octet address is not a prefix of a longer one: refused whole rather than truncated,
    /// on `reverse_dns::sanitise`'s precedent — a truncated key is a WRONG key.
    #[test]
    fn a_longer_address_is_refused_rather_than_truncated() {
        assert_eq!(parse_mac("00:11:32:e9:2f:f8:aa"), None);
        assert_eq!(
            parse_mac("00:11:32:e9:2f:f8"),
            Some(MacAddr([0x00, 0x11, 0x32, 0xe9, 0x2f, 0xf8]))
        );
    }

    /// An empty table is a table, not a failure: a sweep that pinged nothing has no neighbours.
    #[test]
    fn an_empty_table_is_not_an_error() {
        assert!(
            parse(
                "IP address       HW type     Flags       HW address            Mask     Device\n"
            )
            .is_empty()
        );
        assert!(parse("").is_empty());
    }
}
