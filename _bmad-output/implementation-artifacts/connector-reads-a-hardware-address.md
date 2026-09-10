# The connector reads a hardware address

status: review
branch: `story/connector-reads-a-mac` · baseline `c65bc08` (master)

⚠️ **In no epic file.** Chosen by Guy on 2026-09-10 over two cheaper alternatives, because it is the
one change everything else waits on. Precedent for a story outside the epics: 6.4b.

## §0 — Why this, and why now

`identity::l1::join` keys on `(l2_domain, mac)`. The shipped connector emitted no hardware address,
so **no interface was ever formed and the identity engine never ran**: forty-three stories of engine,
trap corpus and resolver, pinned by a test asserting `interfaces_minted == 0`. That is story 5.14's
*structural zero*, and it is the measured cause the 2026-08-30 outside reviews named — *epics 4–5 =
43 stories and nothing operator-visible*.

**Measured before building anything**: from an unprivileged macvlan container, after a ping,
`/proc/net/arp` carries `192.168.1.5 → 00:11:32:e9:2f:f8`. No capability, no dependency, no socket.

## Acceptance criteria

- **AC1** Every host that answers a ping and whose address the kernel's neighbour table carries
  yields `Fact::Mac`. A host whose address it does not carry yields none — never a zero one (NFR7).
- **AC2** `locally_administered` is DERIVED from the address, never asserted.
- **AC3** An INCOMPLETE neighbour entry is refused, and so is an all-zero address — independently.
- **AC4** `declared_kinds()` gains `Mac`; `/sources` derives *what it cannot see* from it, three
  kinds where four stood.
- **AC5** The two structural-zero pins are INVERTED rather than deleted, and the MAC-less path keeps
  a guard of its own.
- **AC6** The race `deferred-work.md` hands this story is CARRIED — measured, and its outcome
  recorded whatever it is.
- **AC7** No regression: ten gates, `clippy --all-targets`, `RUSTFLAGS="-D warnings"`, fmt,
  `cargo deny`, both browser gates.
- **AC8** THE LIVE COUNT lives here: **840 → 850 tests** (563 bin + 188 core + 99 xtask — the sum re-added rather than
  recalled), instrument named — `cargo test --workspace --locked`, wall clock, warm.
- **AC9** `CHANGELOG.md` and the administrator manual current in the same change.

## §2 — What carrying the race produced, and it is the story's best half

🔴 **It does not reproduce.** Eight concurrent `poll_ingest_resolve` passes over one MAC against a
live `mariadb:10.11.11`: **one interface, one mint**. Two passes give the same. The 2026-08-11
measurement stands for the tree it was taken on; no cause is claimed for the difference, and the
mechanism is still visibly non-atomic, so this is not an all-clear.

🔴 **And the remedy the register named is refused by an earlier decision.** The `UNIQUE` index was
written as `0007` and run: it reds `two_interfaces_may_share_one_l1_key`, whose doc is **story 5.9's
AC5** — *a cloned MAC is two real interfaces sharing one address, and a UNIQUE would turn the case
the engine must ABSTAIN on into a 500*. Migration withdrawn. → issue #161.

## §3 — What the network showed that no fixture could

🔴 **A container's hardware address is derived from its IP.** Over 64 neighbours: 19 locally
administered, **11 of the form `02:42:` + the four octets of the host's own IPv4**. For those the MAC
*is* the address, rewritten — an empty signal, not a false one. D13 calls such an address
disqualifying as a grouping anchor and story 5.5 deliberately did not implement that at L1; this is
the case that decision leaves open, now live at 17 % of one real network. → issue #162.

🔴 **It needs macvlan, ipvlan or host networking.** Behind a bridge the host answers every ARP and
the table holds the gateway alone. Such a deployment scans, records and names — and never groups —
**silently**. The manual carries it as a warning rather than a note.

## §4 — Measured on the reference network, end to end

```
46 sightings · 45 interfaces minted · 45 links, all placed
/triage's identity section: « Constats rattachés 45 »   (it read « aucun constat placé »)
obelix   .8 → 00:08:9b:ed:53:fd     .9 → 00:08:9b:ed:53:fe
```

⚠️ **Two interfaces, not one device.** Grouping them is Epic 6's, and the two consecutive addresses
are a far stronger signal than the hostname — which is what makes them a corpus case rather than a
remark (#158).

## §5 — Mutation record (predictions written first)

| id | mutation | predicted | measured |
|---|---|---|---|
| P1 | the incomplete-entry check is deleted | red | 🔴 **red 0 tests, clippy only** |
| P1b | the same, after separating the two guards | red | red 1 ✅ |
| P2 | the all-zero address is emitted | red | red 1 ✅ |
| P3 | a longer address is truncated rather than refused | red | red 1 ✅ |
| P4 | the connector stops declaring `Mac` | red | red 3 ✅ |
| P5 | the emit site stops reading the table | green | green ✅ |
| P6 | the U/L flag is asserted rather than derived | red | red 1 ✅ |
| P7 | the reverse-DNS half, re-run to confirm the MAC did not displace it | red | red 2 ✅ |

🔴 **P1 is the finding.** The incomplete-entry check and the all-zero check overlapped on the one
specimen the kernel really writes, so deleting the first left every test green and reddened only
clippy. *Two guards over one specimen measure one guard.* A stale-address-with-incomplete-flag case
separates them; P1b then reds.

⚠️ **P5 is green and stated as a limit**, not repaired: no test carries the LIVE read, exactly as
none carries the reverse lookup (#149). A tripwire against a change made through `emitted_facts`,
never a barrier.

## §6 — Registered rather than fixed

- **#161** the mint race: not reproducible, and its named remedy refused by 5.9 AC5.
- **#162** a container's MAC is derived from its IP.
- **#158** `obelix` and `panoramix` as corpus cases — now with their real addresses.
