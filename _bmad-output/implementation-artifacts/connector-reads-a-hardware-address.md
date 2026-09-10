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
- **AC8** THE LIVE COUNT lives here: **840 → 859 tests** (572 bin + 188 core + 99 xtask — the sum
  re-added rather than recalled), instrument named — `cargo test --workspace --locked`, wall clock,
  warm, against a **VIRGIN** `mariadb:10.11.11` on port 13399: **8.4 s** live. ⚠️ It read *850* at
  the review; the nine the repair added are named in §7.
- **AC9** `CHANGELOG.md` and the administrator manual current in the same change.

## §2 — What carrying the race produced — 🔴 REWRITTEN: the first answer was wrong

🔴 **It reproduces. This section said it did not, and that was the story's headline.**

The original claim was: *"Eight concurrent `poll_ingest_resolve` passes over one MAC against a live
`mariadb:10.11.11`: one interface, one mint."* Two code-review layers refuted it independently, with
different instruments:

| instrument | result |
|---|---|
| 8 `tokio::spawn`s on one runtime (mine) | 1 interface — **a single trial** |
| 8 OS threads on a `std::sync::Barrier`, 20 rounds | **14 rounds of 20 minted 8 interfaces**; one minted 5 |
| 8 `poll_ingest_resolve` passes, 10 trials | 2 of 10 raced |
| 2 passes, 10 trials | 1 of 10 raced |
| controlled SQL interleaving | 2 interfaces, **every time** |

🔑 **The difference is the INSTRUMENT, not the tree.** Eight `tokio::spawn`s on one runtime can
serialise through the read-then-insert window; eight OS threads released together cannot. *A
negative result from an instrument that cannot open the window measures the instrument.*

⚠️ **And the deeper defect is that a ONE-SHOT was reported as a PROPERTY.** *"It does not
reproduce"* needed repetition before it needed more concurrency — at my own N=8 it races 2 trials in
10, so one trial had a 4-in-5 chance of saying what I wrote. AC6 asked for the outcome *"recorded
whatever it is"*, and what was recorded was a coin toss.

✅ **It is carried now, and deterministically**:
`repo::tests::the_mint_is_not_atomic_and_two_passes_can_duplicate_one_key` interleaves two
connections by hand — both read, then both write — and asserts two interfaces on one key. A test
that wins 14 times in 20 is a test that reds 6 times in 20; the mechanism is what needed pinning,
not the frequency. Closing the window (A mints before B reads) reds it, one test.

⚠️ **`neighbour.rs`'s module doc claimed the race was *"carried in the same change as this file"*
while no concurrency test existed anywhere in the diff** — found by the blind layer, from the diff
alone. Corrected in place.

🔑 **Not reachable through the shipped product today**, because `spawn_scan_loop` runs one pass at a
time in one process. That is the CALLER's property, not the mint's, and it is now written where the
next caller will read it.

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
ONE sweep: 46 sightings · 45 interfaces minted · 45 links, all placed
/triage's identity section: « Constats rattachés 45 »   (it read « aucun constat placé »)
obelix   .8 → 00:08:9b:ed:53:fd     .9 → 00:08:9b:ed:53:fe
```

🔴 **The `-1` had no cause in this section and now it has one: it is the scanner itself.** A host
keeps no neighbour entry for its own address, so the machine running opencmdb is the one machine it
can never place — reproduced by a review layer twice, with host networking (`192.168.1.190`, the
host) and from a macvlan container (`.120`, the container). ⚠️ Permanent, not incidental, and on a
NAS it is the operator's most important machine. Documented at `neighbour.rs` and in the manual.

⚠️ **And the two figures do not behave alike.** *Interfaces* counts THINGS and settles — a second
sweep re-uses the keys and mints nothing (a layer measured 52 → 54 over four sweeps, the two new
ones being machines that missed the first). *Sightings* counts TIMES WE LOOKED and climbs with every
sweep — 55 → 109 → 164 → 219 over four. Both were written here as if they were one kind of number;
the CHANGELOG carried the same conflation and both are corrected.

⚠️ **Two interfaces, not one device.** Grouping them is Epic 6's, and the two consecutive addresses
are a far stronger signal than the hostname — which is what makes them a corpus case rather than a
remark (#158).

## §5 — Mutation record (predictions written first)

⚠️ **Every row names the TREE it was measured on**, because P1 and P1b are the same mutation on two
different trees and the table stated one figure for both. *A mutation count is dated by the tree
that produced it* — story 6.7's own lesson, quoted by this project and not applied here until the
review.

| id | mutation | tree | predicted | measured |
|---|---|---|---|---|
| P1 | the incomplete-entry check is deleted | pre-P1b | red | 🔴 **red 0 tests, clippy only** |
| P1b | the same, after separating the two guards | shipped | red | red 1 ✅ |
| P1 (re-run) | the same, on the REPAIRED tree | repaired | red | red 1 + clippy ✅ |
| P2 | the all-zero address is emitted | shipped | red | red 1 ✅ |
| P3 | a longer address is truncated rather than refused | shipped | red | red 1 ✅ |
| P4 | the connector stops declaring `Mac` | shipped | red | red 3 ✅ |
| P5 | the emit site stops reading the table | shipped | green | 🔴 green — **and repairable** |
| P5 (re-run) | the same, once the blanket `allow(dead_code)` is gone | repaired | red | **red via clippy, 5 items named** ✅ |
| P6 | the U/L flag is asserted rather than derived | shipped | red | red 1 ✅ |
| P7 | the reverse-DNS half, re-run to confirm the MAC did not displace it | shipped | red | red 2 ✅ |

🔴 **P1 is the finding.** The incomplete-entry check and the all-zero check overlapped on the one
specimen the kernel really writes, so deleting the first left every test green and reddened only
clippy. *Two guards over one specimen measure one guard.* A stale-address-with-incomplete-flag case
separates them; P1b then reds.

🔴 **P5 was declared an unrepairable limit and it was repairable in one line.** This paragraph read
*"P5 is green and stated as a limit, not repaired: no test carries the LIVE read"*. The cause was
never measured: `arp_ping.rs` carried a blanket `#![allow(dead_code)]` — inherited from the days
before the connector was wired in — so severing the MAC read left `neighbours()` and transitively
`neighbour::table()` **completely unreferenced with clippy still green**. Removing the attribute was
measured to cost nothing (the module has no dead item) and P5 now reds, clippy naming five items.

🔑 *A limit accepted without measuring its cause is a limit invented.* And the residual the story
credited to clippy — *"left every test green and reddened only clippy"* — was measured in
`neighbour.rs`, which has no such allow; it did not transfer to the file where the wiring lives.

## §6 — Registered rather than fixed

⚠️ **This section claimed a registration it had not made.** It said *"#158 … now with their real
addresses"* and #158 had `createdAt == updatedAt`, zero comments and no hardware address in its
body: the addresses lived only in this file. *A section that says "registered" is not a
registration* — story 6b.9's finding, met again. All four rows below were verified against
`gh issue view` after being written.

- **#161** the mint race — 🔴 **title and body CORRECTED**: it reproduces (see §2), and the body's
  *"something between then and now serialises the read-then-insert"* had no referent. What stands is
  the refusal of the `UNIQUE` remedy by 5.9 AC5, re-verified by writing `0007` and running it.
- **#162** a container's MAC is derived from its IP — unchanged.
- **#158** `obelix` and `panoramix` as corpus cases — **now actually carrying** `00:08:9b:ed:53:fd`
  and `…fe`, with the observation that the two addresses are CONSECUTIVE and that this is a stronger
  grouping signal than the hostname, which no rule reads.
- **#164** NEW — a shared MAC merges two hosts into ONE interface with **zero abstentions**,
  measured; the grouping key now comes from the network rather than from a fixture, and the benign
  case (VRRP, a NIC team) is ordinary.

## §7 — The three-layer code review, and what it changed

**Three isolated layers, each in its own worktree and its own store. Three HIGHs were reached by two
layers independently**, which is the tell that they are in the product rather than in a reading.

| # | finding | layers | state |
|---|---|---|---|
| 1 | the mint race REPRODUCES; §2's headline, `neighbour.rs`'s doc and #161's title were all false | edge + acceptance | **fixed** — deterministic test, docs and issue corrected |
| 2 | `/sources` offered to unlock, via a raw-socket privilege, the capability it had just gained — in both locales | edge + acceptance | **fixed** — copy rewritten, caveat added, derived guard |
| 3 | the LIVE read was carried by nothing: a typo in `/proc/net/arp` left 850 tests, ten gates and clippy green | edge + acceptance | **fixed** — `table_from` seam + a guard that reads the constant |
| 4 | the P1/P1b split's stated cause was FALSE — a STALE entry writes `0x2`, not `0x0` | blind | **fixed** — replaced by the measured `FAILED` specimen, which is worse |
| 5 | three production doc comments still asserted the structural zero this change removes | blind | **fixed** |
| 6 | broadcast and multicast addresses accepted as identity anchors | edge + blind | **fixed** — a property, not a value |
| 7 | `+`-signed octets parsed as a valid address | edge | **fixed** |
| 8 | the scanner's own machine can never be placed; the unexplained `-1` | edge + acceptance | **documented** — code, manual, CHANGELOG |
| 9 | the CHANGELOG's 45/45 are FIRST-SCAN figures; sightings accumulate | edge + acceptance | **fixed** |
| 10 | §6 claimed an update to #158 that was never made | acceptance | **fixed** — actually made |
| 11 | the mutation table dates no row; P1 is not reproducible on the shipped tree | blind + acceptance | **fixed** — every row names its tree |
| 12 | a shared MAC merges two hosts, zero abstentions | edge | **registered** — #164 |

### 🔴 What the repair found against ITSELF

- **The first `table_from` test did not reach `ARP_TABLE`.** It passed a path of its own, so the
  typo it was written for would still have been green. *A guard placed where the defect cannot
  occur reads as coverage and is none* — this epic's dominant class, committed by me inside the
  repair for it, and caught by asking what the mutation would do before running it. A second guard
  reads the constant; M-E1 now reds.
- **The new unlock guard reddened on the repair's OWN sentence**, which read *"no privilege would
  unlock"* — true, and indistinguishable to a substring match from an offer. *An unbounded needle
  cannot tell a denylist entry from a mention of one* (6b.10). The denial was MOVED to the caveat
  key rather than the needle blunted.
- **An oracle reddened for the WRONG REASON.** `sources.unlock` gained an apostrophe, askama escaped
  it, and a test comparing a raw translation against rendered HTML reported the sentence MISSING
  while it was on the screen. Every string containing `'`, `&`, `<`, `>` or `"` was latently
  affected. Fixed at the oracle — and the first fix used the NAMED entities a person reaches for,
  where askama writes NUMERIC ones (`&#39;`), so it failed again for the same wrong reason. The
  table is now copied from `askama-0.16.0/src/filters/escape.rs:129-134` rather than recalled.
- **One commit-blocking lint was invisible to the documented command.** `cargo clippy --workspace
  -- -D warnings` was green while `--all-targets` reported an error in a test module — the CI-only
  blind spot this project recorded at PR #115, live again.
- **The driver refused two runs and was right both times**: an anchor that matched twice (my own
  comment quoted the mutated line) and an orphan snapshot from an interrupted run. Both resolved by
  hand-diffing the snapshot, never `git checkout`.

### ✅ Refuted, with the check that refuted it

- **The per-host table re-read is not a cost.** Real `/proc/net/arp` (61 lines): **22.7 µs**.
  100 000 synthetic rows: 16 ms. `parse_cidr` refuses anything below `/22`, so the worst sweep the
  product permits is ≈152 ms of CPU once per interval.
- **The emitted MAC is what the kernel holds** — 54 hosts, `agree=54 disagree=0`, none stale.
- **The manual's bridge claim is exact**, reproduced: bridge = 1 ARP entry (the gateway); macvlan,
  same hosts = 6 real MACs. And `--cap-add NET_RAW --cap-add NET_ADMIN` on a bridge = **0**.
- **The macvlan/database trap is already in the manual** (#160's warning, the *database* bullet),
  with the exact failure a layer reproduced: `No route to host (os error 113)`.
- **Broadcast and multicast are LATENT, not live**: `subnet_hosts` skips network and broadcast for
  every prefix `<= 30`, and pinging either creates no `/proc/net/arp` row at all. Fixed anyway —
  it costs one comparison and the guard was a single value.
- **No NUD state is mis-classified**: `PERMANENT → 0x6`, `STALE/REACHABLE/DELAY/PROBE → 0x2`,
  `FAILED → 0x0`, proxy `→ 0xc` with an all-zero address.
