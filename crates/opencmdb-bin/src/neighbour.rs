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
//! SHIELD and must carry this race with it"*.
//!
//! 🔴 **It reproduces, and the first measurement said it did not.** This line read *"it is carried
//! in the same change as this file"* while no concurrency test existed anywhere in the diff — the
//! blind review layer found that from the diff alone. Two other layers then reproduced the race
//! independently, and the difference from the story's own negative result is the HARNESS, not the
//! tree: eight `tokio::spawn`s on one runtime can serialise through the read-then-insert window,
//! eight OS threads released together by a `std::sync::Barrier` cannot. *A negative result from an
//! instrument that cannot open the window measures the instrument.* The test is in `repo.rs` now.
//!
//! 🔑 **And it is not reachable through the shipped product today**, because `spawn_scan_loop` runs
//! one pass at a time in one process. That is a property of the caller, not of the mint; it is
//! written here so that the day a second pass exists, this is what it reads.
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

/// 🔴 **What the operator's own machine can never be, and it is permanent.** The kernel keeps no
/// neighbour entry for a LOCAL address — a host does not ARP for itself — so the machine running
/// opencmdb is the one machine on the network whose hardware address this connector can never read.
/// It answers its own ping, it is recorded, it is named, and it lands on no interface.
///
/// That is the whole of the unexplained `-1` in this change's own headline figures (*"46 sightings,
/// 45 interfaces"*): the missing one is the scanner. Measured twice on 2026-09-10 — with host
/// networking the address without a MAC was `192.168.1.190`, the host's own; from a macvlan
/// container at `.120` it was `.120`. ⚠️ On a NAS deployment that is the operator's most important
/// machine, which is why this is a doc rather than a footnote.
const _SELF_IS_NEVER_A_NEIGHBOUR: () = ();

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
    table_from(ARP_TABLE)
}

/// The read itself, with the path as a PARAMETER so it has a test that needs no `/proc`.
///
/// # 🔴 Why this seam exists: a one-character typo was caught by nothing
///
/// [`table`] used to inline this, so the READ — as opposed to the parsing — was exercised by no
/// test at all, its path being a `const`. The code review measured the consequence: changing
/// `ARP_TABLE` to `"/proc/net/arp-absent"` left **850 tests, ten gates and clippy green** while the
/// product silently and permanently reverted to the structural zero this whole change exists to
/// remove. A typo in a path is the ordinary mistake, and nothing in the tree could see it.
///
/// ⚠️ It is a seam, not a configuration knob: nothing may make this path settable from outside.
/// The neighbour table's location is a kernel fact, and an operator who could point it elsewhere
/// could feed the identity engine arbitrary hardware addresses.
///
/// # Errors
///
/// The reason as a sentence when the file cannot be read.
fn table_from(path: &str) -> Result<BTreeMap<Ipv4Addr, MacAddr>, String> {
    let text =
        std::fs::read_to_string(path).map_err(|error| format!("could not read {path}: {error}"))?;
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
        // ⚠️ Refused for the same reason `reverse_dns::sanitise` refuses a name with no letter:
        // repeating a non-answer would put it on the operator's screen and, worse, into the
        // identity engine as a key that groups unrelated hosts onto one interface.
        if !is_an_identity(mac) {
            continue;
        }
        out.insert(address, mac);
    }
    out
}

/// Whether a hardware address can stand for a THING, as opposed to a non-answer or a destination.
///
/// # 🔴 A property, never a list of values
///
/// This began as `mac.0 == [0; 6]` — one value — and the code review measured what one value
/// misses: `ff:ff:ff:ff:ff:ff` and `01:00:5e:11:22:33` both parse, the kernel really will hold both
/// as complete entries, and two sightings carrying the broadcast address were measured MERGING into
/// one interface. Enumerating the values someone thought of closes only those; the property closes
/// the class (story 5.12's sentence, applied again).
///
/// Two clauses, and each says a different thing:
///
/// - **all zeroes** — the kernel saying *I asked and nobody has answered*. Not an address.
/// - **the I/G bit set** (bit 0 of the first octet) — multicast, and broadcast with it. These are
///   DESTINATIONS, not interfaces: no host owns one, so several hosts answering on one would be
///   grouped into a single device. A false merge, and a false merge cannot be undone by looking
///   harder (D20).
///
/// ⚠️ **The locally-administered bit is DELIBERATELY not here.** It is bit 1 of the same octet, it
/// is set on every Docker container's address, and D13 calls it disqualifying *as a grouping
/// anchor* — which is an L2 judgement about how much a MAC is worth, not an L1 judgement about
/// whether it is an address at all. Story 5.5 refused to implement it at L1 on purpose, and two
/// committed traps assert `l1-exact-mac` still fires on a locally-administered MAC and on a VRRP
/// one. Refusing it here would redden both. → issue #162.
///
/// 🔑 The reachability today is narrow and is stated rather than implied: `subnet_hosts` skips the
/// network and broadcast addresses for every prefix `<= 30`, `parse_cidr` refuses anything below
/// `/22`, and pinging a broadcast or multicast address creates no `/proc/net/arp` row at all — all
/// three measured. So this closes a LATENT class, not a live defect. It costs one comparison.
fn is_an_identity(mac: MacAddr) -> bool {
    mac.0 != [0; 6] && mac.0[0] & 0x01 == 0
}

/// `aa:bb:cc:dd:ee:ff` as the kernel writes it.
fn parse_mac(text: &str) -> Option<MacAddr> {
    let mut octets = [0_u8; 6];
    let mut parts = text.split(':');
    for octet in &mut octets {
        let part = parts.next()?;
        // ⚠️ `u8::from_str_radix` ACCEPTS A SIGN, so `+0:+11:+32:…` parsed as a valid address until
        // the code review tried it. That is not leniency, it is a wrong reading: the kernel writes
        // two lowercase hex digits and nothing else, and a row that does not look like that is a
        // row we do not understand. Refused rather than guessed.
        if !part.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        *octet = u8::from_str_radix(part, 16).ok()?;
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
    /// 🔴 **The reason first written here was FALSE, and the true one is worse.** It said this is
    /// *"what the kernel writes while it re-resolves a neighbour it already knew"* — measured on a
    /// live kernel, a `STALE` entry carries flags **`0x2`**, not `0x0`, so the specimen this test
    /// exists for was a shape nobody had seen. A cause needs a check, not a plausible story, and
    /// that sentence was the whole justification for splitting the two guards.
    ///
    /// The real specimen, measured with two dummy devices on 2026-09-10: a **`FAILED`** entry
    /// prints flags `0x0` beside **the device's OWN hardware address** —
    /// `198.51.100.16 0x1 0x0 06:6e:94:67:ae:0f * ep0`, where `06:6e:94:67:ae:0f` is `ep0`'s MAC.
    ///
    /// 🔑 So without the flags check every DEAD host on the network would be filed under **the
    /// scanner's own hardware address**, and the identity engine would group all of them onto one
    /// interface — with the scanner. The guard is load-bearing on kernel-produced input, not only
    /// on a hand-written row, and it is more load-bearing than the sentence it replaces claimed.
    #[test]
    fn an_incomplete_entry_is_refused_even_when_it_carries_an_address() {
        // The shape a live kernel really produced for a FAILED neighbour: the probing device's
        // own address, with the incomplete flag beside it.
        let text = "IP address       HW type     Flags       HW address            Mask     Device
198.51.100.16    0x1         0x0         06:6e:94:67:ae:0f     *        ep0";
        assert!(
            parse(text).is_empty(),
            "the kernel says nobody answered, and the address shown is the PROBER's own"
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

    /// 🔴 **The READ, which no test reached until the code review.** `table_from`'s path was a
    /// `const` inlined into `table`, so changing it to `"/proc/net/arp-absent"` left 850 tests, ten
    /// gates and clippy GREEN while the product silently lost the one fact this whole change exists
    /// to produce. Both directions are asserted, because a read that always fails and a read that
    /// always succeeds are each satisfied by half of this.
    #[test]
    fn the_read_itself_is_carried() {
        let dir = std::env::temp_dir().join(format!("opencmdb-neighbour-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch directory");
        let path = dir.join("arp");
        std::fs::write(&path, LIVE).expect("write the fixture");

        let table = table_from(path.to_str().expect("utf-8 path")).expect("the file is readable");
        assert_eq!(
            table.get(&"192.168.1.5".parse().unwrap()),
            Some(&MacAddr([0x00, 0x11, 0x32, 0xe9, 0x2f, 0xf8])),
            "the read reaches the parser and the parser's result reaches the caller"
        );

        let missing = table_from(dir.join("no-such-table").to_str().expect("utf-8 path"));
        assert!(
            missing.is_err(),
            "a path that does not exist is an Err the caller can log, never an empty table \
             indistinguishable from a network that answered nothing"
        );
        assert!(
            missing.unwrap_err().contains("no-such-table"),
            "and the reason names the path, so a typo in it is readable in the log"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    /// 🔴 **And the SEAM's test does not reach the constant** — it passes a path of its own, so a
    /// typo in `ARP_TABLE` leaves it green. That is the defect class this project has met in nine
    /// stories: *a guard placed where the defect cannot occur reads as coverage and is none*, and I
    /// wrote one here before measuring it. This is the half that reads the constant.
    ///
    /// Two clauses, and the second is the one that bites: the literal is PINNED, and — because a
    /// pinned literal only catches a typo somebody re-reads — `table()` itself must reach a real
    /// file wherever the kernel provides one. The test names the canonical path independently, so a
    /// `const` pointing anywhere else fails the second assertion even if the first were relaxed.
    #[test]
    fn the_production_path_is_the_kernels_own() {
        const KERNEL_PATH: &str = "/proc/net/arp";
        assert_eq!(
            ARP_TABLE, KERNEL_PATH,
            "the neighbour table's location is a kernel fact, not a preference"
        );
        if std::path::Path::new(KERNEL_PATH).exists() {
            assert!(
                table().is_ok(),
                "the kernel provides {KERNEL_PATH} and table() must actually reach it — this is \
                 what reds when ARP_TABLE gains a typo"
            );
        }
    }

    /// 🔴 **A destination is not an interface.** The broadcast and multicast addresses parse, the
    /// kernel really holds them as complete entries, and two sightings carrying one were measured
    /// merging into a single interface — several unrelated hosts made one device. The all-zero
    /// check alone did not see either.
    #[test]
    fn an_address_no_host_owns_is_refused() {
        for hardware in [
            "ff:ff:ff:ff:ff:ff",
            "01:00:5e:11:22:33",
            "33:33:00:00:00:01",
        ] {
            let text = format!(
                "IP address       HW type     Flags       HW address            Mask     Device\n\
                 192.168.1.7      0x1         0x2         {hardware}     *        eth0"
            );
            assert!(
                parse(&text).is_empty(),
                "{hardware} is a destination, not an identity"
            );
        }
    }

    /// The two clauses of [`is_an_identity`] are independent, and the locally-administered bit is
    /// deliberately NOT one of them — story 5.5 refused that at L1 and two committed traps assert
    /// `l1-exact-mac` still fires on such an address.
    #[test]
    fn a_locally_administered_address_is_still_an_identity() {
        assert!(
            is_an_identity(MacAddr([0x02, 0x42, 0xc0, 0xa8, 0x01, 0x0a])),
            "a container's address corroborates nothing (#162) and is still an address"
        );
        assert!(
            is_an_identity(MacAddr([0x00, 0x00, 0x5e, 0x00, 0x01, 0x2a])),
            "and so is a VRRP one"
        );
        assert!(!is_an_identity(MacAddr([0; 6])), "the non-answer is not");
        assert!(
            !is_an_identity(MacAddr([0xff; 6])),
            "and neither is the broadcast destination"
        );
    }

    /// ⚠️ `u8::from_str_radix` accepts a sign, so `+0:+11:…` parsed as a valid address. The kernel
    /// writes two lowercase hex digits per octet; anything else is a row we do not understand.
    #[test]
    fn a_signed_octet_is_not_a_hex_octet() {
        assert_eq!(parse_mac("+0:+11:+32:+e9:+2f:+f8"), None);
        assert_eq!(parse_mac("00:11:32:e9:2f:-8"), None);
        assert_eq!(
            parse_mac("00:11:32:e9:2f:f8"),
            Some(MacAddr([0x00, 0x11, 0x32, 0xe9, 0x2f, 0xf8])),
            "and the shape the kernel really writes still parses"
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
