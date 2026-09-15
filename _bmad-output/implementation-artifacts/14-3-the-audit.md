# Story 14.3: The audit, and the address the product must not offer

Status: ready-for-dev

⚠️ **`ready-for-dev` is the workflow's status, not a statement that nothing is open.** Contexted and
VALIDATED 2026-09-15 by two fresh-context layers (fact-check; gap-hunt, which built and measured);
§2's decisions are Guy's and none is taken yet — **T0 takes them before any code**.

Epic 14 (IPAM). Baseline: `02d15f2` (master, after story 14.2b's second review), **924 tests**
(634 bin + 191 core + 99 xtask), ten gates — re-measured by the gap-hunt on a virgin store after one
warm run. 🔑 **This is the story the epic exists for.**

## Story

As the operator,
I want the product to tell me which addresses are in use but not in my plan,
so that I never hand a busy address to a second machine.

🔑 **The reason is the deliverable, not the highlight** (`epics.md:2343`): the *next free address*
panel proposes an address because no RECORD claims it. Under this story it must **exclude every
observed address** — *an exclusion, not a highlight, the only place the product PREVENTS the
duplicate rather than reporting it*. Every other criterion serves that one.

## §0 — What is settled, and must not be re-opened

**Guy's six ARBITRATIONS of 2026-09-10** (`CLAUDE.md` / `docs/project-context.md`, *"Next: IPAM"*):
a form rather than a file · ranges carry a policy · IPAM untouched by the documenting gesture · a
sighting holds an address until released · two distinct treatments · **the form warns and still
writes**. The last one is written into this story's own criterion (`epics.md:2433`): refusing was
refused, because the most frequent LEGITIMATE case is entering into the plan the machine already
there.

**The epic's six measured CONSTRAINTS** (`epics.md:2345-2357`, a different list with its own
numbering). The five that bind this story:

- **(2) The range's POLICY decides the verdict** (`:2349`). Measured on the reference LAN: **24 DHCP
  leases** of the 32 addresses reverse DNS resolves — an audit that highlights every unclaimed
  address highlights 24 permanently and stops being read. `dhcp-pool` is where an unclaimed address
  is NORMAL.
- **(3) A sighting protects an address FOREVER, until the operator releases it** (`:2351`). Refused:
  the last sweep only, and a named window — *the product cannot tell gone from silent* (one sweep
  sees 46 hosts where 49 exist). Accepted cost: **the grid fills and never empties by itself**; every
  held cell carries the date of its last sighting; RELEASE is story 14.4's fourth write route.
- **(4) IPAM and `declared_attribute` are TWO registers**, and the documenting gesture does not
  touch IPAM (`:2353`). ⚠️ The constraint says only that; *whether the audit or the offer READS
  `declared_attribute`* is not in it, and story 14.1 in fact foresaw that join (§1(d)) — decision 4.
- **(5) The vocabulary is CLOSED** (`:2355`). *The audit needs no state noun of its own*: *observed
  with no plan entry* is `undeclared`, *the plan says free while the network shows it occupied* is
  `gap` (`prd.md:1041-1045`). FR24's two-MAC case says « Conflit d'adresse », never a bare
  `conflict` (Guy, 2026-08-20, `deferred-work.md:4338`; the glossary's `conflict` means source
  against source, `prd.md:1016`).
- **(6) Three states cannot be told apart by colour** (`:2357`): every state carries a pattern and a
  word, never a hue alone (WCAG 1.4.1, NFR25).

**The binding PLAN table** (`prd.md:1025-1060`): `static` — one machine per address, chosen, an
`undeclared` address is an ANOMALY; `dhcp-pool` — the occupant changes by design, an `undeclared`
address is NORMAL; `reserved` — *set aside, not to be allocated*, nothing should answer yet, an
`undeclared` address is an anomaly and a surprise; `infrastructure` — gateway, equipment management,
network and broadcast addresses, *never offered as free*.

**Stories 14.2 and 14.2b, still in force**: cells carry a POLICY and a STATE on two axes; the EDGE
(`.0`, `.255`) is decided before any range and is never offerable; past `MAX_DRAWN_ADDRESSES` (1024)
no grid is built and the ranges are listed instead, the ceiling tested BEFORE anything is
materialised; a schema-legal bad row is skipped and named, never a 500 for the page; refusals are
keyed in both locales; source-scanning guards read through `src/source_scan.rs`, never a hand-rolled
stripper; `IpamWriteState` holds a port and never a pool.

## §1 — What this story inherits, each WITH the measurement that produced it

⚠️ **Line numbers are dated by `7373f9f`** (code identical to `02d15f2`), re-checked by the fact-check
layer — re-derive before citing them in code or in the record.

### (a) 🔴 The plan's own guard FORBIDS what this story must do — and its narrowing has limits

`the_plan_reads_no_observation` (`ipam_page.rs:1174`, doc from ~`:1140`) exists because story 14.2's
AC2 said the plan reads no observation. Its perimeter is derived (a file whose name starts with
`ipam`, or whose non-test code names `ip_subnet`/`ip_range`/`ip_address`; each with exactly ONE
line-start `#[cfg(test)]`), asserted equal to `["ipam_page.rs", "ipam_repo.rs", "ipam_write.rs"]`; it
forbids the literals `observation_record`, `identity_link`, `declared_attribute`, and any whole-word
`repo` except `crate::repo::classify` / `crate::repo::is_deadlock` and `opencmdb_core::repo`. **Any
audit reach reds it by design** — measured (P4): `crate::repo::load_observation_facts` named in
`ipam_page.rs` reds. 🔴 **Putting the join in an unguarded module to keep it green is *a guard placed
where the defect cannot occur*.**

The gap-hunt PROTOTYPED decision 6(a) (a new `ipam_audit.rs` allowed to name the reader) and planted:

| plant | result |
|---|---|
| an `identity_link` literal inside `ipam_audit.rs` | **red** |
| the reader named from `ipam_page.rs` | **red** |
| `ipam_write.rs` calling `crate::ipam_audit::…` | 🔴 **GREEN** — what keeps a write from reading the network is the pool-free `IpamWriteState` TYPE, not this guard |
| the whole join + an `identity_link` query in a new `src/audit.rs` (no `ipam` prefix, no plan table) | 🔴 **GREEN** — the guard's stated limit, now measured |

Also: a narrower reader writing `FROM observation_record` in any `ipam_*` file reds on the literal,
so a new reader lives in `repo.rs` and is allowed BY NAME; the guard's failure message says *"the
audit is story 14.3's"*, stale after this story.

Text that states the old rule and becomes false: the module doc of `ipam_page.rs` (~`:6-8`) and of
`opencmdb-core/src/ipam/mod.rs` (`:22-24`, *"Nothing here refers to an entity, an observation or an
interface"*; `:43`, *"Nothing in this codebase reads a policy yet"* — already stale). `ipam_write.rs`
`:22-25` is about provenance and says the plan and `declared_attribute` *"never touch"*: it becomes
false only if decision 4 is (a).

### (b) 🔴 No function answers the question, and the one that returns every observation's facts reads EVERYTHING — measured

Nothing returns observed IPv4 addresses with their hardware addresses and last sighting.
`repo::load_observation_facts(executor) -> Result<Vec<ObservedBatch>, sqlx::Error>` (`repo.rs:628-634`)
is the only reader returning every observation's facts (others exist: `load_observation_by_id`
`:376`, `count_observations` `:466`, `last_observed_at` `:503`) — `SELECT id, connector_id,
…observed_at…, facts FROM observation_record ORDER BY observed_at, id`, **no `WHERE`, no `LIMIT`**.
Registered as *"every page render loads every observation ever recorded"* (`deferred-work.md:5383-5401`,
owner *"the first story after the freeze is lifted"*); the same defect is open as issue #150 (cited in
`inventory_view.rs:166`, not in the register). `/triage` already pays it (`page.rs` ~`:1363`, `:1411`,
`:1699`).

**The gap-hunt measured it** (`cargo test --release -p opencmdb-bin`, rows shaped like the ARP/ping
connector's — `IpV4` + `Rtt` + `Hostname` + `Mac`, 46 hosts, a sweep every 5 min — on a 32-core box,
much faster than the NAS):

| rows | ≈ days of sweeping | reader as shipped | narrower (`observed_at, facts`, no `ORDER BY`) | Rust fold | peak RSS |
|---|---|---|---|---|---|
| 10 000 | 0.75 | 12–17 ms | 6–7 ms | 0 ms | 15 MB |
| 100 000 | 7.5 | 129–135 ms (debug 537–565) | 61–64 ms | 1–3 ms | 58 MB |
| 1 000 000 | 75 | **3 004–3 325 ms** | **1 438–1 541 ms** | 18–49 ms | **492 MB** |

46 × 288 ≈ 13 250 rows a day — **a year is ≈ 4.8 M rows**. The fold costs nothing; the READ is the
whole cost. Reusing the reader fails NFR2's p95 1.5 s (`prd.md:1235`) after about two months on that
machine; the narrower one is at the limit at 1 M. And constraint 3 needs one fact per distinct
(address, MAC) — at 1 M rows there are **46** distinct addresses. With `PAGE_STORE_BUDGET` at 5 s, the
screen would start answering the budget's error at a few million rows (a release-build `timeout`
did preempt the SQL phase at 1 500 / 2 000 / 2 600 ms; a 50 ms budget did NOT fire on a debug build
at 100 k — cause not established).

The closest existing shape is `inventory_view.rs:174-190`: a *freshest sighting per address*
`BTreeMap<String, &ObservedBatch>`; the triage `Nouveau` loop (`page.rs:1151-1246`) folds the same way.
On `/ipam` the error path is NOT `page::server_error`: `plan_data` returns
`opencmdb_core::repo::RepositoryError` (`ipam_page.rs:317-320`), failures render
`render_error_body()` inside `store_within` (`:294-306`), and a `sqlx::Error` crosses through
`crate::repo::classify` (`repo.rs:1607`), already allowed by the guard.

### (c) The IPv4 ↔ MAC pairing exists only inside one observation, and "two MACs" is always a SUCCESSION

`Fact::IpV4 { addr }` and `Fact::Mac { addr, locally_administered }` (`opencmdb-core/src/observation/mod.rs`
~`:293-320`, externally tagged JSON) sit side by side in one observation's `facts`. The ARP/ping
connector emits one observation per host (`arp_ping.rs::emitted_facts`, `:442`): `IpV4` + `Rtt` +
optional `Hostname` + optional `Mac`. `interface` has a MAC and `last_seen_at` but **no IP column**.
The gap-hunt's fold test: two different MACs → 2; the same MAC twice → 1; a MAC plus a sighting with
no MAC → 1; a locally-administered MAC counts by its bytes.

🔴 **The neighbour table is a `BTreeMap<Ipv4Addr, MacAddr>`** (`neighbour.rs:83`) — one MAC per address
per sweep. So *"two hardware addresses seen on one IPv4"* can never be told apart from a SUCCESSION:
a NIC swap, a regenerated VM MAC, a phone's randomised MAC (19 of 64 neighbours on the reference LAN
are locally administered, `arp_ping.rs`). With constraint 3, **any succession outside a `dhcp-pool`
becomes a PERMANENT « Conflit d'adresse »** until 14.4 releases it — including every lease in space no
range covers, which is the 24-lease problem coming back through FR24 (decision 10). And a fold that
keeps one date per ADDRESS cannot say which MAC was seen last: keep a last-seen date per MAC.

### (d) Formats, and the seam 14.1 actually named

The plan stores the padded canonical form (`192.000.002.009`, `ipam_repo::canonical` / `from_canonical`);
observations store plain dotted text inside JSON. **Both decode to `std::net::Ipv4Addr`**, so the
comparison is on `Ipv4Addr`, in Rust (`architecture.md:239`: *"All value comparison/normalization
happens in Rust, never in SQL"*; D10). A SQL join on `facts` is not possible anyway.

⚠️ Story 14.1's own words (`14-1-plan-schema-and-no-producer.md:97-100`): *"the audit joins plan rows
to **`declared_attribute.ipv4`**, which stores the **unpadded** dotted quad"* — 14.1 foresaw a join with
the DECLARED register, not with observations. That bears on decision 4.

### (e) 🔴 A warning in the write's answer is thrown away, and there is no focus contract on that path

`ipam_write.rs::answer` (`:601-630`) answers a success with **201 and `hx-redirect: /ipam?subnet=…`**.
htmx 2.0.4 handles `HX-Redirect` with `location.href = …; return` before any swap; story 14.2b's
kbd-probe already measured `status=201 answer="" activeElement=""` (`kbd-probe.mjs` ~`:729-737`). **The
201 body is never shown, and there is no swap to focus**: story 6.4's focus contract lives on the
REFUSAL path only. `IpamWriteState` holds a port and never a pool: anything the write needs to know
about the network goes through the port. The phrase *"one URL per state"* is epic constraint 4
(`_triage.html:3`, `ipam_write.rs:600`); `kbd-probe.mjs`'s comment *"no confirmation rides in the URL"*
would become false under a URL-carried warning.

### (f) The cell model, the OFFER as it is, and the tests that pin it

`CellState` (`ipam_page.rs:57`): `Defined(Option<IpPolicy>)`, `Free(IpPolicy)`,
`Infrastructure(Option<IpPolicy>)`, `NotCovered`. `PlanView::derive` (`:185-215`) resolves defined → edge
→ covering range (lowest `first_addr` wins, `ranges_in` sorting `ORDER BY first_addr`, `ipam_repo.rs:698`
— *a rendering decision, not a repair*) → not-covered. `ranges_in` is read BEFORE the ceiling
(`:343-346`); `addresses_in` and `derive` after.

🔴 **`offerable()` (`:141-143`) is `Free(policy) if policy != Infrastructure` — it OFFERS `reserved` and
`dhcp-pool` cells today.** Measured on the seed's Workshop subnet (`reserved` .129–.200 only):
`aria-label="198.51.100.129 · free, reserved"` and **next free = `198.51.100.129`** — an address the
binding table calls *not to be allocated*, and a pool address handed out for a static assignment is
the classic way to make a duplicate. No test pins or refutes it (decision 3). Its doc's *"14.3 must
additionally exclude every OBSERVED address"* is therefore incomplete.

`next_offerable()` (`:218`) takes the first offerable cell; `counts()` returns
`(defined, free, infrastructure, not_covered)` — on the seed `1 defined · 86 free · 2 infrastructure ·
39 not covered`, where "free" includes the OBSERVED .11/.12/.13 (decision 12). `strings()` builds
`next_free_caveat` (`app.yml:840-842`), *"From the plan alone; the network has not been consulted"* —
false after this story. `ipam.next_free_none` (`app.yml:838`, *"None — every address in this subnet is
defined, infrastructure, or outside every range"*) is **reachable since 14.2** (any subnet with no range
renders it, `ipam_page.rs:607`) but **no test renders it** — ⚠️ the register row calling it unreachable
(`deferred-work.md:4403-4409`) dates from story 6b.7's deleted example dataset, owner "Epic 14"; and its
TEXT becomes false once observed (and perhaps reserved) addresses are excluded.

Tests that pin today's model and must be updated deliberately, never loosened: the offerable count
`254` and `next_offerable == 192.0.2.1`; `every_state_carries_its_own_key_and_its_own_modifier` and
`the_stylesheet_defines_every_modifier_the_grid_can_emit` (hand-written four-state lists);
`the_blank_cell_is_the_one_modifier_no_legend_entry_carries` (a hand-written list of three literals,
`:1016-1020`, and every new modifier must be a legend literal); `the_four_states_carry_four_distinct_fills`
(⚠️ it asserts each rule DECLARES a background, not that fills are distinct — the name overstates it);
the rendered `aria-label` count `258` (`:1420`); every three-argument `PlanView::derive` call; `main.rs`'s
selector test.

### (g) The visual channel is exhausted — a marker is possible and fails CONTRAST where it matters

Story 14.2 measured three border styles rendering at 1 px, `static` and `infrastructure` identical
pixels, the fill *already spoken for by the four STATES* (`app.css:927-940`); Guy's stated limit
(2026-09-12): the grid separates the STATES, the policy is carried by the accessible name and the
legend. The gap-hunt measured in Chrome 151 at 1280 px (the cell box is **15.34 px**): a 4 px `::after`
dot, a 2 px inset `box-shadow` and a diagonal stroke each make every screenshot pair differ, and axe
reports **0 violations** for all three — 🔴 but the marker `rgb(29,31,32)` on the defined fill
`rgb(65,97,128)` is **2.56:1**, under WCAG 1.4.11's 3:1, on exactly the held (defined + seen) cell AC6
is about, and **axe does not measure pseudo-elements or box-shadows**. One marker carries one bit:
`gap` vs `undeclared` would then be told apart by the underlying fill, not by the audit.

### (h) The triage link depends on the DECLARED register, not on the plan

The triage `Nouveau` row id is `nouveau:<ip>` (`page.rs:1203`), linked `/triage?sel=nouveau:<ip>`
(`row_href` `:1320`, `url_escape` `:1333`). It exists only for an observed address `declared_attribute`
does not claim. Measured on the seed: the only `nouveau:` row is `192.0.2.99` — inside the pool, not
highlighted; the highlighted `.11/.12/.13` are `ecart:` rows. With one sighting per audit case added,
`nouveau:` rows appear for each undocumented one and **the queue grows from 5 to 10 rows**, which both
browser gates' row-finding must survive. A `Mac` added to a declared sighting changes no triage row.

### (i) The seed reaches no audit state, and observations outside every SUBNET have no screen

`a11y/seed.sql` (`:54-82`, `:146-160`): four observations at `NOW(6)`, **no `Mac` facts** —
`192.0.2.11`/`.12`/`.13` (free `static` cells) and `192.0.2.99` (inside the `dhcp-pool`). Office
`192.0.2.0/25`: `static` .1–.40, `dhcp-pool` .80–.126, defined `.9`; .41–.79 not covered. Workshop
`198.51.100.128/25`: `reserved` .129–.200. **Nothing refuses a seeded run that draws no AUDIT state.**
An observed address in NO plan subnet (e.g. `10.9.9.9`) has a triage row and **no `/ipam` screen at
all** — though *"in use but not in my plan"* is this story's own phrase (decision 11). Nested subnets
are accepted (register, Guy 2026-09-15), so one address can sit on two screens under two policies.

### (j) File sizes (code lines before the first line-start `#[cfg(test)]`)

`ipam_page.rs` **729**, `ipam_repo.rs` **752**, `ipam_write.rs` **854**, `page.rs` **1954 — 46 lines
left, it must not grow**. `repo.rs`'s gate count stops at a test struct at `:184` and is misleading.

### (k) What holds by STRUCTURE, and is owed a check rather than a decision

NFR8(a) — *a fault may only REMOVE knowledge, never ADD an assertion* (`prd.md:1324`) — and FR19's
blind-source suppression (`prd.md:902`): the audit derives from RECORDED sightings only, so a blind or
failing scanner cannot add one. True by construction, **owed a test**: an empty sighting set yields no
finding, and the derivation reads no clock (story 6b.4's clock-guard precedent).

## §2 — Decisions this story must take, and which are Guy's

Each is posed with its options and a recommendation, **refined by the validation's measurements**.
None is settled here. The first eight shape the story; 9–14 are defaults the developer applies unless
Guy refuses them.

1. **`undeclared` or `gap` for an observed address with no address row?** The epic's AC1 and AC2
   OVERLAP (a free `static` cell with a sighting fits both); read literally AC2 would make every DHCP
   lease a `gap`, against constraint 2; and **no option in the first draft answered an `infrastructure`
   range** — where the gateway at `.1` is the commonest sighting. Options: (a) by coverage;
   (b) by policy; (c) AC1's letter, AC2 unreachable; **(d) by the OFFER** — `gap` = an observed address
   the plan WOULD OFFER (decision 3's predicate: a free `static` cell); `undeclared` = every other
   observed address with no row (`reserved`, `infrastructure` range, not covered). *Recommendation:
   (d)* — it agrees with the binding table (`reserved` is *not to be allocated*, so the plan does not
   call it free), answers `infrastructure`, and gives AC2 and AC3 ONE shared, testable predicate; it
   breaks AC1's letter for `static` only, which is registered. In every option **a `dhcp-pool` address
   is never a finding**, written.
2. **Which policy decides when ranges overlap, or subnets nest?** (register row "nested and
   overlapping subnets", `deferred-work.md:5510`). Options: **(a) the most protective** across every
   range covering the address, in any subnet (any non-`dhcp-pool` range → a finding); (b) the most
   specific; (c) report the contradiction. *Recommendation: (a) for the verdict and the offer*, keeping
   14.2's lowest-`first_addr` rule for what the cell LOOKS like, both stated.
3. **Which policies may the offer draw from?** 🔴 Measured: it offers `reserved` and `dhcp-pool` today.
   Options: **(a) `static` only**, by the table's words; (b) `static` and `dhcp-pool`; (c) keep 14.2's
   behaviour, registered. *Recommendation: (a).* Edges and defined addresses are never offered whatever
   is declared over them (register row "network and broadcast addresses", `deferred-work.md:5519`).
4. **Does the offer exclude an address DOCUMENTED in `declared_attribute.ipv4` but never observed?**
   Story 14.1 foresaw that join (§1(d)); arbitration 4 keeps the registers apart for the DOCUMENTING
   gesture. Options: (a) exclude it — one named read of `declared_attribute`, from the one place
   decision 6 allows, and `ipam_write.rs:22-25`'s doc corrected; (b) plan and network only, registered.
   *Recommendation: (a)* — the offer's purpose is preventing a duplicate.
5. **How the sightings are read** — measured in §1(b). Options: (a) reuse `load_observation_facts` and
   fold — **fails NFR2 after ≈ 2 months**; (b) a narrower reader (`observed_at, facts`, no `ORDER BY`) —
   at the limit at 1 M rows (≈ 75 days); (c) wait for the retention decision; **(d) a bounded SIGHTING
   SUMMARY** — one row per distinct (`l2_domain`?, IPv4, MAC) with its last-seen instant, maintained at
   ingest and backfilled once from `observation_record`; bounded by distinct pairs (46 on the
   reference LAN), not by time. ⚠️ (d) is a MIGRATION and an ingest change — this story's first draft
   said none was expected. *Recommendation: (d)*, the only option that holds constraint 3 ("forever")
   within NFR2 — and it can later serve `/triage`'s same read (issue #150). ⚠️ **Guy may prefer to SPLIT**:
   14.3a the summary (schema + ingest + backfill, no screen), 14.3b the audit on top.
6. **The guard.** Options: **(a) narrow it** — one named module (e.g. `ipam_audit.rs`, with one
   line-start `#[cfg(test)]`) may name the sighting reader (and `declared_attribute` if 4a), the guard
   renamed, `identity_link` still forbidden; the two measured greens of §1(a) written as its limits;
   (b) retire it. *Recommendation: (a)*, with AC8 in three parts.
7. **Where the form's warning appears**, given §1(e). Options: (a) **after the write, carried by the
   redirect URL** — traps measured: it survives reload, bookmark and back (a reload re-shows a warning
   about a write made a week ago, and the store cannot tell that event from the legitimate end state);
   no focus contract exists there; the gates cannot use a literal state (the URL carries a subnet UUID);
   (b) **before the write**, an `hx-get` on the address field to a read fragment — ⚠️ a NEW GET route is
   in no perimeter guard today (the write-route guard walks POST paths, the screen guard walks
   `Screen::ALL`), so it owes one; (c) **no event**: after the write, `/ipam`'s findings list simply shows
   the defined-and-seen address with its last sighting and MACs — a state, not a warning. *Recommendation:
   (b)* — the only option that warns before the operator commits, which is the arbitration's point; its
   perimeter cost written into AC4. And **the range form** also "enters highlighted addresses" (a
   `static` range laid over observed addresses): the same warning covers it, or the story says why not.
8. **How the audit is SHOWN.** Options: **(a) a findings LIST under the grid** — address, state word,
   policy, last sighting as an absolute date PER MAC, hardware addresses, link when one exists — as the
   carrier of the state; plus ONE cell marker ("this address was seen") whose contrast is checked in the
   browser gate against every fill (axe cannot); (b) the list only. *Recommendation: (a)*, the marker
   dropped if it cannot reach 3:1 on every fill.
9. **The date of a last sighting**: ABSOLUTE, never a relative age (UX bans, `ux-design-specification.md:1514-1522`),
   one per MAC.
10. **FR24 is a succession, measured (§1(c)).** Default: « Conflit d'adresse » only inside a `static`
    or `reserved` range (not in a `dhcp-pool`, not in space no range covers — where it would return the
    24-lease problem), with copy that NAMES the succession-or-duplicate ambiguity; PRD FR24's second half
    (*"a static-declared IP inside a DHCP range"*, `prd.md:909`) is decision 13's warning, registered as
    a divergence.
11. **Observed addresses in no plan subnet**: default — a short plan-wide list on `/ipam` ("seen,
    outside every subnet of the plan"), since the story's own phrase is *in use but not in my plan*.
12. **The occupancy line**: default — `free` counts only what the offer would propose; **no "seen"
    count** (a number that only grows is the UX ban); the findings list is a list, not a number.
13. **The plan-against-plan warning** (register row "an address defined inside an existing range",
    `deferred-work.md:5526`; 14.2b decision 5): default — an address defined inside a `dhcp-pool` is shown
    with a keyed warning on `/ipam`, not a conflict, and still writes.
14. **A subnet too large to draw**: default — no offer (said on screen); its findings list IS shown,
    being bounded by sightings, not by the subnet's size.

## Acceptance Criteria

⚠️ **The shapes below assume the recommendations.** T0 rewrites any AC a decision changes, in the same
act, with the option refused.

**AC1 — `undeclared` is highlighted where the policy makes it an anomaly, never in a `dhcp-pool`**,
per decision 1: each case — not covered, `reserved`, `infrastructure` range, `static` (per the option),
`dhcp-pool` (no finding), a defined address (no finding) — has a test, in both locales from the EXISTING
key `state.undeclared` (constraint 5). The overlap rule (decision 2) is tested, including a nested subnet.

**AC2 — `gap` is distinct from `undeclared`** by word (the existing `triage.kind.ecart`) and by
treatment, and **never inside a `dhcp-pool`**. A test asserts the two are never rendered alike, in the
HTML and in the browser.

**AC3 — THE OFFER EXCLUDES EVERY OBSERVED ADDRESS**, and draws only from the policies decision 3 allows
— never a `reserved` or `dhcp-pool` address under the recommendation, never a defined address, never an
edge whatever the plan declares, and (decision 4) never a documented one. Tests on the seed's Workshop
subnet (`reserved` only → no offer) and on a `dhcp-pool`-only subnet. `offerable`'s doc is corrected.
The `next_free_caveat` sentence says what WAS consulted; `ipam.next_free_none` is RENDERED by a test and
its text names the new exclusions in both locales; the register row calling it unreachable is closed as
stale. 🔑 **Proven red by the mutation that re-admits ONE observed address, its carrier named.**

**AC4 — the form warns, names what it knows, links, and STILL WRITES**, per decision 7: for an address
with a recorded sighting it names the last sighting (absolute date) and the hardware address(es), links
to `/triage?sel=nouveau:<ip>` WHEN that row exists and says so when it does not (§1(h) — both branches
tested), and the write still succeeds (201). Keyed in both locales. Under (b): the fragment route is
authenticated AND covered by a perimeter guard (it is neither a POST nor a `Screen`), budgeted, and
walked by both browser gates; the range form's case is covered or stated. A browser check defines and
asserts the focus contract of the path actually taken.

**AC5 — FR24: « Conflit d'adresse », with BOTH hardware addresses and each one's last sighting**, per
decision 10: tested for two MACs in `static`, in `reserved`, in a `dhcp-pool` (none), in uncovered space
(per the default, none), the same MAC twice (none), a sighting with no MAC (no second address). The copy
names that a succession and a duplicate look alike.

**AC6 — a held cell carries the date of its last sighting**, absolute, per MAC, in its accessible name
and in the findings list. A test renders the same data at two clock instants and asserts identical
output (the UX ban's own test); the derivation reads no clock.

**AC7 — the plan-against-plan warning** (decision 13): tested; the write still succeeds; the register row
closed or re-owned.

**AC8 — the guard is reshaped, not bypassed** (decision 6), in three parts, each proven red:
(i) an `identity_link` literal — and a `declared_attribute` one outside the allowed place — inside the
allowed module reds; (ii) the sighting reader named from any OTHER plan file reds; (iii) that
`ipam_write.rs` cannot read the network is carried by the pool-free `IpamWriteState` TYPE — said, and
proven by a compile-fail mutation. The guard's two measured greens (§1(a)) are written as its limits;
its failure message and every module doc stating the old rule are corrected.

**AC9 — the read is bounded and measured at a real scale**, per decision 5: inside `store_within`, the
size ceiling tested before anything is materialised; timed at the reference rate expressed in DAYS of
sweeping, up to **at least 1 M observation rows** (≈ 75 days), time AND peak memory recorded with their
command, against NFR2's p95 1.5 s; under (d), the summary's size shown bounded by distinct pairs, its
ingest maintenance tested (a new sighting of a known pair updates, a new pair inserts), its backfill
idempotent; the register row on the unbounded read updated with the figures.

**AC10 — the browser gates REACH the audit.** `a11y/seed.sql` gains sightings with `Mac` facts for each
case (not covered, `reserved`, an `infrastructure` range, a defined address, two MACs on one IPv4, inside
the `dhcp-pool`, one in no subnet), including a highlighted address that IS documented (no link) and one
that is not (link); the axe gate REFUSES (exit 2) a seeded run that draws no audit state (a `REQUIRE` flag
set in `ci.yml`); a computed-contrast check covers any cell marker on every fill (§1(g)); both gates'
row-finding survives the larger queue; the kbd-probe reaches the warning and the findings list (⚠️ its
`198.18.0.0/24` has no sighting and never reaches the warning).

**AC11 — structure holds and is tested** (§1(k)): an empty sighting set yields no finding; no clock read.

**AC12 — THE LIVE COUNT lives here**, with the command and both store conditions named (a virgin store
after one warm run; `env -u DATABASE_URL`, never `DATABASE_URL=`). Baseline **924** (634 + 191 + 99) at
`02d15f2`.

**AC13 — no regression**: ten gates, `clippy --workspace --all-targets -D warnings`, `RUSTFLAGS="-D
warnings"`, fmt, `cargo deny`, **both browser gates claimed and run**; no file over 2000 code lines
(`page.rs` does not grow); docs current before push — the user manual's IPAM chapter (its `planned` block,
`user-manual.tex:180-184`, names the audit and the offer), both twins, and the register.

## §3 — What the validation measured that this story carries

Two fresh-context layers on `7373f9f`, 2026-09-15. **Fact-check** (read-only): 3 HIGH, 7 MED, 6 LOW.
**Gap-hunt** (built and measured, own worktree and store): 3 HIGH, 6 MED, 2 LOW. Applied in place above;
the ones that changed the story's shape:

- 🔴 **The offer serves `reserved` and `dhcp-pool` addresses** (both layers; measured `198.51.100.129`) —
  the first draft's AC3 would have kept doing it in the story whose purpose is not handing out addresses
  wrongly. → decision 3, AC3.
- 🔴 **The recommended read fails NFR2 after ≈ 2 months** (gap-hunt, measured to 1 M rows) — the first
  draft's scale (10 k / 100 k) was ≈ one week of production. → decision 5 gains option (d), AC9's scale.
- 🔴 **AC8 as written could not be carried by the guard** for `ipam_write.rs`, and two plants stay green
  (gap-hunt prototype). → AC8 in three parts, limits written.
- 🔴 **§0 fused the epic's six constraints with Guy's six arbitrations**, numbering both "(6)"; and gave
  constraint (4) a rule it does not contain (fact-check). → §0 rewritten.
- ⚠️ **FR24's "two MACs" is always a succession** (`neighbour.rs:83`), permanent under constraint 3 →
  decision 10, AC5. **No option answered an `infrastructure` range**, and AC2 read literally makes every
  lease a `gap` → decision 1(d). **A cell marker fails 3:1 contrast and axe cannot see it** → decision 8,
  AC10. **7(a)'s URL survives reload; no focus contract exists on the redirect path** → decision 7.
- ⚠️ Facts corrected: 14.1's seam is with `declared_attribute`, not observations; `/ipam`'s error path is
  `RepositoryError` → `render_error_body`, not `page::server_error`; `next_free_none` is reachable since
  14.2 (untested, and its text becomes false); issue #150 is not in the register row; "one URL per state"
  is constraint 4, not 6.4's arbitration; F57 is not the containment rule (D10 / `architecture.md:239`
  is); citations re-derived (`page.rs:1203`, `:1320`; `arp_ping.rs:442`; `ipam_page.rs:1174`;
  `prd.md:906-909`); the cell is 15.34 px, not 14.

## Tasks / Subtasks

- [ ] **T0** Take §2's decisions with Guy (1–8), confirm or refuse the defaults (9–14), decide whether
  to split (decision 5); rewrite affected ACs in the same act, each with the option refused.
- [ ] **T1** (AC11, AC5, AC6) The pure audit core: from sightings (`Ipv4Addr` → per-MAC last seen) and a
  plan, derive each address's finding — no store, no clock; tests first.
- [ ] **T2** (AC1, AC2, AC7) Verdicts per decisions 1, 2, 10, 13, including nesting.
- [ ] **T3** (AC3) The offer per decisions 3–4; the caveat; `next_free_none` rendered and reworded;
  `offerable`'s doc; the occupancy line (decision 12).
- [ ] **T4** (AC9) The read per decision 5 (under (d): migration, ingest maintenance, backfill), inside the
  budget, after the ceiling; the scale measurement to ≥ 1 M rows; the register row.
- [ ] **T5** (AC8) The guard reshaped with its three-part mutations; module docs corrected.
- [ ] **T6** (AC1, AC2, AC5, AC6, decisions 8, 11, 14) The screen: findings list, marker (if kept),
  outside-every-subnet list, keyed words, absolute dates, legend literals, stylesheet — §1(f)'s tests
  updated deliberately.
- [ ] **T7** (AC4) The warning per decision 7, with its route's perimeter guard if (b).
- [ ] **T8** (AC10) The seed, the `REQUIRE` flag, the contrast check, both gates' row-finding, the
  kbd-probe; both gates run in a browser.
- [ ] **T9** (AC12, AC13) Measure; docs; the live count.

## Dev Notes

### Traps this project has paid for, and which apply here

- 🔴 **A guard placed where the defect cannot occur reads as coverage and is none** — the join in an
  unguarded module; and the measured limit: a new non-`ipam` file is invisible to the guard.
- 🔴 **A budget that bounds the wrong half of a handler is none** (14.2's denial of service); and a
  budget cannot save a read that grows with time — measure in days of sweeping, not in rows chosen to pass.
- 🔴 **A test that pins the ugly thing demands it** (`offerable == 256`, 14.2; the reserved offer is the
  same shape, unpinned) — update §1(f)'s tests to the new truth, never loosen them.
- 🔴 **axe cannot see a pseudo-element or a box-shadow** — a marker's contrast needs its own computed
  check.
- 🔴 **A guard that greps a file greps its prose** — `src/source_scan.rs`.
- 🔴 **A hand-written state list cannot see a new state** — every list in §1(f) grows.
- ⚠️ **`app.yml` is invisible to incremental builds**; **`ascii_bin` is PAD SPACE**; **store-backed tests**
  take `DB_TEST_LOCK` and own their CIDR, a virgin store races on `migrate!` — warm once, drop/recreate
  before counting; **`cargo xtask mutate` reports counts, never names** — read each red test's panic
  message; **`cargo fmt` can re-wrap an anchor**; **"registered" means a row exists**; **a cause needs a
  check** (14.2b's first repair presented a held test connection's behaviour as a production defect).

### What the store must answer, and what it must not

The audit reads the PLAN (`list_subnets`, `ranges_in`, `addresses_in`) and the SIGHTINGS (per decision 5);
it never reads `identity_link` (grouping is Epic 6's); it reads `declared_attribute` only under decision
4(a), from the one place decision 6 names. Under decision 5(d) a migration is expected (the next number,
DDL on `ascii_bin`, never an `ENUM`, story 6.5's lessons), and ingest (`scan_pass.rs`) maintains it.

### Project Structure Notes

- The pure audit where the guard can name it (decision 6): a new `crates/opencmdb-bin/src/` module, or
  `opencmdb-core/src/ipam/` for the verdict algebra (D47: errors are `ipam::IpamError`, no sqlx in core).
  ⚠️ `float-free` walks `identity/` only; *14.3 is where a ratio could first appear* (14.1) — occupancy
  stays counts.
- `page.rs` must not grow (46 lines left). `ipam_page.rs` (729), `ipam_repo.rs` (752), `ipam_write.rs`
  (854) have room; split before the ceiling.
- No new dependency.

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 14 constraints `:2345-2357`, Story 14.3 `:2411-2441`.
- `_bmad-output/planning-artifacts/prd.md` — FR19 `:902`, FR21–FR24 `:906-909`, NFR2 `:1235`, NFR8(a)
  `:1324`, NFR25 `:1467`, STATE axis `:1004-1023`, PLAN axis `:1025-1060`.
- `_bmad-output/planning-artifacts/ux-design-specification.md` — occupancy grid `:529-531`, `:1227-1231`,
  accessibility `:1632-1636`, hard bans `:1509-1522`.
- `_bmad-output/planning-artifacts/architecture.md` — comparisons in Rust `:239`, D47 `:2639`, `ipam/`
  `:3208`, `:3366`.
- `_bmad-output/implementation-artifacts/deferred-work.md` — nested subnets `:5510`, edge addresses
  `:5519`, an address inside a range `:5526`, the unbounded read `:5383-5401`, `next_free_none`
  `:4403-4409`, « Conflit d'adresse » `:4338`, `role="grid"` `:5420`.
- Stories `14-1-plan-schema-and-no-producer.md`, `14-2-the-plan-on-screen.md`,
  `14-2b-the-plan-in-the-operators-hands.md`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Contexted 2026-09-15 from three research passes; validated the same day by a fact-check layer and a
  gap-hunt layer that built and measured. Fourteen decisions posed in §2 (eight Guy's, six defaults);
  none settled.

### File List

### Change Log

- 2026-09-15 — **validated and rewritten whole** rather than appended to (*a correction that adds the
  true text without removing the false leaves the record carrying both*). The validation refuted the
  first draft's recommended read (fails NFR2 after ≈ 2 months), found the offer serving `reserved` and
  `dhcp-pool` addresses, an AC8 the guard could not carry, FR24's two MACs always a succession, a marker
  failing contrast where axe cannot see, and §0 fusing two lists of six. See §3.
- 2026-09-15 — contexted. The epic's AC1 and AC2 were found to overlap, and the guard
  `the_plan_reads_no_observation` to forbid the audit by design; both posed as decisions.
