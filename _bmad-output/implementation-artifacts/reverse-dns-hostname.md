# Reverse DNS: the queue says a name, and a rename is a gap

status: review
branch: `story/reverse-dns-hostname` · PR #143 · commits `b4434b0`, `4b8ac64`
baseline_commit: `7b0b55c` (master)

⚠️ **This story is in no epic file.** It is point 1 of the plan the 2026-08-30 project review left
(`session-2026-08-30`), sequenced by Guy on 2026-09-09. Precedent for a story outside the epics:
6.4b. It carries ~150 lines by the cap Guy approved in shape on 2026-08-30.

## §0 — Why, and what was measured before anything was built

Ten days of unattended running on the NAS produced **69 questions in the triage queue and every one
was a bare IP address**. The connector emits `IpV4` and `Rtt` and nothing else, so the queue could
name nothing.

**The measurement that shaped the design**, taken over those 69 addresses on 2026-09-09:

| asked | what it is | names obtained |
|---|---|---|
| the system resolver (`/etc/resolv.conf`, an ad-blocking forwarder) | the default | **2** |
| `192.168.1.1`, the router that issued the DHCP leases | | **37** |

🔑 A small network resolves its own hosts at the box that leases the addresses, and that box is
usually not the resolver the hosts are pointed at.

## Arbitrations

1. **Guy, 2026-09-09** — the resolver is the system one by default and `OPENCMDB_DNS_SERVER` points
   it elsewhere; `hickory-resolver` over a hand-written PTR query. Refused: the system resolver
   alone (2/69, measured), and the hand-rolled UDP query (~120 lines incl. compression pointers).
2. **Guy, 2026-09-09** — the queue shows the FIRST LABEL of the name; the fact and the declared
   record keep the whole name. Taken at a browser: `swiss-domotique-plug-5751f8.home.arpa` (37
   chars) wrapped the row onto two lines while shorter names did not. Refused: the whole name.
3. **Guy, 2026-09-09** — `reconcile` compares only the most recent sighting **of each source**.
   Refused: shipping the conflict as measured, and dropping the display half.
   🔴 **Re-taken at the code review the same day, on a defect this one caused** (see §5): the unit
   is now the last value each source gave **per FIELD**. Refused: keeping the rule and registering
   the cost; keeping the rule and rewording the sentence (measured to require the same history
   lookup, for half the benefit).
4. **Mine, delegated** — the connector's identity is DERIVED, not minted per boot. Required for
   arbitration 3 to deliver anything: see §2. 🔴 **Widened at the review**: derived from the
   NETWORK, not from the string typed — four spellings of one subnet minted four sources.
5. **Guy, 2026-09-09, at the code review** — `/sources` names the resolver in force. Refused:
   correcting the code's over-broad sentence and registering the display.

## Acceptance criteria

- **AC1** Every host that answers a ping is looked up in reverse DNS and, when it answers to a
  name, the observation carries `Fact::Hostname { source: Dns }`. A host with no PTR record emits
  **no hostname fact** — never an empty one (NFR7/D35).
- **AC2** The lookup runs only for hosts that ANSWERED, so an empty subnet costs no DNS traffic.
- **AC3** `OPENCMDB_DNS_SERVER` selects the resolver; unset or blank = the system one. A value that
  is not a bare IP address is refused **at boot, by name** (`scan_interval`'s precedent). A value
  carrying a port is refused rather than stripped.
- **AC4** `declared_kinds()` declares `Hostname` unconditionally, and `/sources` derives *what it
  cannot see* from that declaration — four kinds where five stood.
- **AC5** The triage queue shows the name beside the address, as its first label; the fact and the
  documented record keep the whole name.
- **AC6** `reconcile` compares, per source and per field, the last value that source gave. One
  source superseding itself yields a **gap**; two sources that disagree **abstain** (FR16); a
  sweep that learned nothing about a field **retracts nothing**. Out-of-perimeter reach unchanged.
- **AC6b** `/sources` names the resolver in force, in both directions (configured address, or the
  words for the system one), asserted on the rendered page.
- **AC7** The connector's id is stable across restarts and differs per perimeter.
- **AC8** No regression: ten `cargo xtask ci` gates, `clippy --all-targets`,
  `RUSTFLAGS="-D warnings"`, fmt, both browser gates.
- **AC9** THE LIVE COUNT lives here: **798 → 827 tests** (540 bin + 188 core + 99 xtask) after the
  code review. Both terms measured — `master` re-run in its own worktree, not recalled, by me and
  again by the acceptance layer.
  ⚠️ **The instrument is named, because the first form did not name it**: `cargo test --workspace
  --locked`, **wall clock, warm** — **0.79 s** without a store and **16.0 s** against a live
  `mariadb:10.11.11`. The acceptance layer found the first form's *0.21 s* to be the `opencmdb-bin`
  crate's own `finished in` line: *one number named a command and its neighbour named a crate.*
- **AC10** Documents current in the same change: `CHANGELOG.md`, `docker/.env.example`, the admin
  manual (rebuilt), `README.md`.

## §2 — What the implementation found that the plan did not

🔴 **A renamed host produced `Conflit` + `Absence`, permanently.** `load_observation_facts` is an
unbounded `SELECT`, so `reconcile` compared the declared value against every sighting ever
recorded. `hostname` is the first field on which one source can disagree with itself — `ipv4` is
the perimeter key, so two in-perimeter sightings agree on it by construction. Measured on the
served page before the fix; the `Absence` pane read *"Aucune source n'a rapporté de valeur"* about
a host seen one minute earlier that was reporting two names.

🔴 **Arbitration 3 alone did not fix it.** `connector_id` was `Uuid::now_v7()` per boot: the NAS
store carries **eight** ids for one connector, seven dead. A dead source's last words keep voting.

🔴 **Two store-backed tests of story 6.3 asserted the OLD behaviour**, modelling "two disagreeing
sightings" with ONE source, and their own comment called it a fact of nature. Rewritten to assert
both sides. ⚠️ They were **invisible locally** (`DATABASE_URL` unset ⇒ early `return`); the
mutation driver, which requires a store, surfaced them.

⚠️ **`page.rs` reached the 2000-line ceiling** and was SPLIT into `sources_view.rs` on
`identity_view.rs`'s precedent, not shortened.

⚠️ **`cargo deny` is a CI step and NOT one of the ten gates**, so the prescribed local commands are
blind to it. `chacha20 0.10.1` was yanked upstream after 2026-08-30 — **measured red on `master`
too**, in its own worktree — and is fixed in commit `4b8ac64`, kept separate.

## §3 — Mutation record (`cargo xtask mutate`, predictions written first)

| id | mutation | predicted | measured |
|---|---|---|---|
| M1 | `sanitise` drops the ASCII-alphanumeric property | red | red 1 ✅ |
| M2 | `sanitise` keeps the trailing dot | red | red 3 ✅ |
| M3 | `emitted_facts` drops the hostname | red | red 2 ✅ |
| M4 | `declared_kinds` drops `Hostname` | red | red 4 ✅ |
| M5 | `poll` never calls the resolver | green | 🔴 **RED via clippy, 0 tests** |
| M5b | the call runs, its answer is discarded | green | green ✅ |
| M6 | `reconcile` compares every sighting again | red | red 4 ✅ |
| M7 | `reconcile` keeps the OLDEST per source | red | red 4 ✅ |
| M8 | the comparison loses the real source (nil again) | red | red 3 ✅ |
| M9 | `short_name` returns the whole name | red | red 2 ✅ |
| M10 | the source id is minted fresh again | red | red 1 ✅ |
| M11 | the row keeps a name the latest sighting dropped | red | red 1 ✅ |
| M12 | the template stops printing the name | red | red 1 ✅ |
| M13 | the hostname gap prints the name twice | red | red 1 ✅ |
| M14 | a bad `OPENCMDB_DNS_SERVER` is silently ignored | red | red 1 ✅ |
| M15 | out-of-perimeter reach is not counted | red | red 5 ✅ |
| M16 | `sanitise` accepts an address again | red | red 1 ✅ |
| M17 | `sanitise` accepts a bidi override again | red | red 1 ✅ |
| M18 | `short_name` shows an alphanumeric-free first label | red | red 1 ✅ |
| M19 | the perimeter is hashed raw again | red | red 1 ✅ |
| M20 | the queue overwrite goes back to positional | red | red 1 ✅ |
| M21 | `/sources` stops naming the resolver | red | red 1 ✅ |
| M22 | `reconcile` lets an older value overwrite a newer one | red | red 1 ✅ |

⚠️ M16–M22 are the REPAIR's own pass; M6–M8 describe the rule as it stood before the review
re-arbitrated it, and are kept as the record of what was measured then rather than rewritten.
⚠️ **M19's first form did not compile** and the driver refused to measure it rather than reporting a
red — the exit-2 contract doing its job — and was re-run in a form that compiles.

🔴 **M5 is the finding.** Removing the lookup makes the whole `reverse_dns` module dead code, so
clippy carries it and **no test does**. M5b isolates it: green. **No test carries the live
lookup** — a TRIPWIRE against a change made through `emitted_facts`, never a barrier, which is the
limit stories 5.14 and 6.4 already wrote for this connector. Carriers are MIXED and named per row;
*"every red assertion-carried"* is **not** claimed.

## §5 — The three-layer code review, 2026-09-09

Three isolated layers, each in its own worktree, two with their own live `mariadb:10.11.11`.
⚠️ **All three ran at this session's capability, NOT on a different model** — that half is not
claimed. **35 raw findings → 27 distinct; 2 arbitrations, 19 patches, 6 registered, and a set of
suspicions refuted with the check that refuted them.**

🔴 **THE HEADLINE WAS REACHED BY ALL THREE, BY THREE ROADS.** The first form of arbitration 3 made
the newest SIGHTING authoritative for absence as well as for value, so one lost PTR answer WITHDREW
a field the previous sweep had supplied — and the page then said *"no source reported a value"*
about a host seen a minute earlier, **the exact sentence this story exists to abolish**. The blind
layer deduced it from the diff alone; the acceptance layer measured it with a `master` binary on
the identical store as a control; the edge layer reproduced the whole pane on the served page. Guy
re-arbitrated to the last value **per (source, field)**, and `Reconciliation::observed_from` exists
so the page can date a value by the sighting that carries it rather than by the row.

🔴 **The edge layer found six nobody else did, all measured.** `short_name` broke the property
`sanitise` admits a name on, so `-.example.com` rendered a bare `-` as a machine's name. A PTR
answer that IS an address was accepted, displayed as `192`, and **written into a declared record by
the gesture**. `U+202E` survived into the served page and the record — askama escapes markup, which
the layer verified, and escaping does not touch a bidi override. Four spellings of one CIDR minted
four sources. And `OPENCMDB_DNS_SERVER` reached **no screen at all**, leaving true the very
sentence the code gives as its reason for refusing a typo at boot.

🔴 **A weak guard hid a live defect.** The blind layer found
`a_name_the_latest_sighting_does_not_carry_is_not_kept` unable to tell *latest wins* from *last row
wins* — the later sighting was also the later element. Rebuilt in both orders, it reddened: the
`Nouveau` overwrite was **positional**, correct only because `load_observation_facts` sorts
ascending. A pure builder whose answer depended on an ordering stated nowhere in its signature.

⚠️ **Three of my own numbers were refuted.** *0.21 s* was the crate's `finished in` line and not a
wall clock (**0.726 s**); §4 said *registered* while the branch touched no artefact at all (story
6b.9's finding verbatim — now GitHub issues #144–#149); and the literal I first pinned for
`derived_connector_id` was **copied from a review report instead of measured** — the pinned test
caught it on its first run, which is what pinning a literal is for.

✅ **Refuted with the check, so nobody re-chases them**: no XSS (askama escapes, no `|safe`);
`source_uuid` cannot collide two sources into one; the 253-byte boundary is right; the
out-of-perimeter count is unchanged; the per-source narrowing IS guarded (red 4); and the blind
layer's cascade suspicion — measured in Chrome on the served page, `.entity-name` renders **Barlow
600** against the address in **ui-monospace 400**, so the declaration wins.

## §4 — Registered rather than fixed

Every row is a GitHub issue, because a section that says *registered* is not a registration — the
acceptance layer measured that this branch touched no register file at all.

- **#144** a source that has stopped talking keeps voting for ever (owner: Epic 11; liveness: D32 /
  Epic 13). ⚠️ Existing rows keep their per-boot ids; what makes them harmless is that they carry
  no hostname, which is a property of today's data and not a mechanism.
- **#145** `cargo deny` is a CI step and none of the ten gates.
- **#146** the queue row can still wrap: `short_name` shortens by structure, never by length.
- **#147** `OPENCMDB_DNS_SERVER` refuses the one link-local spelling that works.
- **#148** `/diagnostic` renders an unsubstituted `%{badge}` (pre-existing, found while reviewing).
- **#149** the lookup half is carried by no test — a TRIPWIRE, never a barrier.
- `load_observation_facts` is still unbounded (PR #142's row). This narrows what the unbounded read
  can DO; it does not bound it.
