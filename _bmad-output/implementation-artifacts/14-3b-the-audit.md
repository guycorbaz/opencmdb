# Story 14.3b: The audit, and the address the product must not offer

Status: review

⚠️ **`ready-for-dev` is the workflow's status, not a statement that nothing is open.** Contexted and
VALIDATED 2026-09-15 by two fresh-context layers (fact-check; gap-hunt, which built and measured);
**all fourteen decisions TAKEN by Guy the same day (T0)**. 🔴 **BLOCKED ON STORY 14.3a** — the
sighting summary this story reads does not exist until 14.3a merges.

Epic 14 (IPAM). 🔴 **SPLIT at its validation, 2026-09-15 (Guy)**: story 14.3 became **14.3a** — the
bounded sighting summary (schema, ingest maintenance, backfill, no screen) — and **14.3b**, this
story, the audit on top of it. `epics.md` describes FOUR stories and there are now SIX (14.2b was
inserted at 14.2's implementation); a story may not edit it, and the divergence is registered with
Epic 14's retrospective. Precedent: 5.9/5.9b, 5.14/5.14b, 6b.4/6b.4b, 14.2/14.2b.

Baseline: **924 tests** (634 bin + 191 core + 99 xtask) at `02d15f2` — ⚠️ **re-measured after 14.3a
merges**, which adds its own tests. 🔑 **This is the story the epic exists for.**

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
writes** (`epics.md:2433`: refusing was refused — the most frequent LEGITIMATE case is entering into
the plan the machine already there).

**The epic's six measured CONSTRAINTS** (`epics.md:2345-2357`, a different list, its own numbering).
The five that bind this story:

- **(2) The range's POLICY decides the verdict** (`:2349`) — 24 DHCP leases of 32 resolved addresses
  on the reference LAN; `dhcp-pool` is where an unclaimed address is NORMAL.
- **(3) A sighting protects an address FOREVER, until the operator releases it** (`:2351`) — the
  product cannot tell gone from silent; the grid never empties by itself; every held cell carries its
  last sighting; RELEASE is 14.4's.
- **(4) IPAM and `declared_attribute` are TWO registers**; the documenting gesture does not touch
  IPAM (`:2353`). What the OFFER reads is decision 4 below, not this constraint.
- **(5) The vocabulary is CLOSED** (`:2355`): `undeclared` and `gap` are the audit's words
  (`prd.md:1041-1045`); FR24's case says « Conflit d'adresse », never a bare `conflict`
  (`deferred-work.md:4338`; the glossary's `conflict` is source against source, `prd.md:1016`).
- **(6) Three states cannot be told apart by colour** (`:2357`): a pattern and a word, never a hue.

**The binding PLAN table** (`prd.md:1025-1060`): `static` — one machine per address, chosen;
`dhcp-pool` — the occupant changes by design; `reserved` — *set aside, not to be allocated*;
`infrastructure` — gateway, equipment management, network and broadcast, *never offered as free*.

**Stories 14.2 and 14.2b, still in force**: POLICY and STATE on two axes; the EDGE (`.0`, `.255`)
decided before any range and never offerable; past `MAX_DRAWN_ADDRESSES` (1024) no grid, the ceiling
tested BEFORE materialising; a bad row skipped and named, never a 500; refusals keyed in both
locales; source-scanning guards through `src/source_scan.rs`; `IpamWriteState` holds a port, never a
pool.

### Guy's fourteen decisions of 2026-09-15 (T0), each with the option refused

1. **`gap` vs `undeclared` — BY THE OFFER.** `gap` = an observed address the plan WOULD OFFER (a free
   cell that decision 3 makes offerable); `undeclared` = every other observed address with no address
   row (in a `reserved` range, in an `infrastructure` range, in space no range covers). **Never a
   finding inside a `dhcp-pool`.** Refused: by coverage, by policy, AC1's letter (each left the
   `infrastructure` range unanswered, and AC2 read literally made every DHCP lease a `gap`). ⚠️ Breaks
   the epic's AC1 letter for `static` — registered.
2. **Overlapping ranges and nested subnets — THE MOST PROTECTIVE decides** the verdict and the offer:
   any non-`dhcp-pool` range covering the address, in ANY subnet of the plan, makes it a finding and
   takes it out of the offer. What the cell LOOKS like keeps 14.2's lowest-`first_addr` rule; both
   rules are written. Refused: the most specific; reporting the contradiction.
3. **The offer draws from `static` ranges ONLY.** Refused: `static` + `dhcp-pool`; keeping 14.2's
   behaviour. A subnet with no `static` range proposes nothing, and says so.
4. **The offer excludes an address DOCUMENTED in `declared_attribute.ipv4`**, even never observed —
   one named read, from the one module decision 6 allows; `ipam_write.rs:22-25`'s *"never touch"* doc
   corrected. Refused: plan and network only.
5. **The sightings are read from STORY 14.3a's bounded summary** (split). Refused: reusing
   `load_observation_facts` (fails NFR2 after ≈ 2 months), a narrower reader (at the limit at ≈ 75
   days), waiting for retention. ⚠️ **The reader's shape is 14.3a's (validated 2026-09-15)**: one row per
   (address, `l2_domain`, MAC) with `Option<MacAddr>` — `None` for the `'-'` sentinel — and first/last
   seen. **Merging across `l2_domain`s is THIS story's**, and so is the sentinel's treatment: an address
   seen both with and without a MAC carries a `None` row beside its MAC row, and the two-MAC conflict
   (decision 9) must NOT count `None` as a second hardware address.
6. **The guard is NARROWED**: one named module may reach the sighting reader and the documented read;
   `identity_link` stays forbidden; renamed; its two measured greens written as limits. Refused:
   retiring it.
7. **The form warns BEFORE the write**: the address field asks a new read route (`hx-get`) and warns;
   the write stays possible. Refused: after the write through the redirect URL (survives reload, no
   focus contract), no event (a state rather than a warning).
8. **The audit is shown as a findings LIST under the grid, plus ONE "seen" marker on the cell kept only
   if it reaches 3:1 on every fill**, checked in the browser gate (axe cannot). Refused: the list only.
9. **Last sightings are ABSOLUTE dates, one per MAC** — never a relative age.
10. **« Conflit d'adresse » only inside a `static` or `reserved` range** (most protective), with copy
    that names that a succession and a duplicate look alike; not in a `dhcp-pool`, not in uncovered
    space. PRD FR24's second half (*"a static-declared IP inside a DHCP range"*, `prd.md:909`) is
    decision 13's warning — a registered divergence.
11. **A short plan-wide list of observed addresses outside every subnet of the plan** on `/ipam`.
12. **The occupancy line counts as free only what the offer would propose; NO "seen" count.**
13. **An address defined inside a `dhcp-pool`** carries a keyed WARNING on `/ipam`, not a conflict, and
    still writes.
14. **A subnet too large to draw** has no offer (said on screen) and DOES show its findings list.

## §1 — What this story inherits, each WITH the measurement that produced it

⚠️ **Line numbers are dated by `7373f9f`** (code identical to `02d15f2`) and will move when 14.3a
merges — re-derive before citing.

### (a) 🔴 The plan's own guard FORBIDS the audit — and the narrowing has measured limits

`the_plan_reads_no_observation` (`ipam_page.rs:1174`, doc from ~`:1140`): perimeter derived (a file
whose name starts with `ipam`, or whose non-test code names a plan table; each with exactly ONE
line-start `#[cfg(test)]`), asserted equal to `["ipam_page.rs", "ipam_repo.rs", "ipam_write.rs"]`;
forbids the literals `observation_record`, `identity_link`, `declared_attribute`, and any whole-word
`repo` except `crate::repo::classify` / `crate::repo::is_deadlock` / `opencmdb_core::repo`. The
gap-hunt PROTOTYPED decision 6 (a new `ipam_audit.rs` allowed to name the reader):

| plant | result |
|---|---|
| an `identity_link` literal inside `ipam_audit.rs` | **red** |
| the reader named from `ipam_page.rs` | **red** |
| `ipam_write.rs` calling `crate::ipam_audit::…` | 🔴 **GREEN** — the write's guarantee is the pool-free `IpamWriteState` TYPE |
| the whole join + an `identity_link` query in a new `src/audit.rs` | 🔴 **GREEN** — the guard's stated limit, measured |

A literal `FROM observation_record` / `declared_attribute` in any `ipam_*` file reds, so the readers
live in `repo.rs` (or 14.3a's module) and are allowed BY NAME. The guard's message *"the audit is story
14.3's"* goes stale. Docs stating the old rule: `ipam_page.rs` ~`:6-8`; `opencmdb-core/src/ipam/mod.rs`
`:22-24`, `:43`; `ipam_write.rs` `:22-25` (decision 4).

### (b) The sighting read, measured — why 14.3a exists

Rows shaped like the ARP/ping connector's, 46 hosts, a sweep every 5 min, 32-core box (the NAS is
slower), `cargo test --release`:

| rows | ≈ days | `load_observation_facts` | narrower reader | Rust fold | peak RSS |
|---|---|---|---|---|---|
| 10 000 | 0.75 | 12–17 ms | 6–7 ms | 0 ms | 15 MB |
| 100 000 | 7.5 | 129–135 ms | 61–64 ms | 1–3 ms | 58 MB |
| 1 000 000 | 75 | **3 004–3 325 ms** | **1 438–1 541 ms** | 18–49 ms | **492 MB** |

A year is ≈ 4.8 M rows; at 1 M rows there are **46** distinct addresses. **This story reads 14.3a's
summary instead**, and re-measures `/ipam` end to end at ≥ 1 M observation rows (AC9). On `/ipam` the
error path is `RepositoryError` → `render_error_body()` inside `store_within` (`ipam_page.rs:294-320`);
a `sqlx::Error` crosses through `crate::repo::classify`.

### (c) "Two MACs" is always a SUCCESSION

The neighbour table is a `BTreeMap<Ipv4Addr, MacAddr>` (`neighbour.rs:83`) — one MAC per address per
sweep. A NIC swap, a regenerated VM MAC, a randomised phone MAC (19 of 64 reference neighbours are
locally administered) look exactly like a duplicate, and under constraint 3 stay forever. Hence
decision 10's scope and copy, and one last-seen date PER MAC (decision 9).

### (d) Formats

The plan stores the padded canonical form; observations and `declared_attribute.ipv4` store plain
dotted text. All compare as `std::net::Ipv4Addr`, in Rust (`architecture.md:239`, D10). Story 14.1's
seam (`14-1-plan-schema-and-no-producer.md:97-100`) is the join with `declared_attribute.ipv4` —
decision 4.

### (e) 🔴 The warning before the write needs a route no guard covers today

`ipam_write.rs::answer` (`:601-630`) answers 201 + `hx-redirect`; htmx 2.0.4 follows it before any swap;
the 201 body is never shown and no swap is focused (`kbd-probe.mjs` ~`:729-737`). Decision 7's `hx-get`
fragment is a NEW GET route: the write-route perimeter guard walks POST paths, the screen guard walks
`Screen::ALL` — **the fragment is in neither**, so it owes a perimeter test (401 without a credential,
exists with one), a budget, and both browser gates. The range form also "enters highlighted addresses"
(a `static` range laid over observed ones): decision 7 covers it.

### (f) The cell model, the OFFER as it is, and the tests that pin it

`CellState` (`ipam_page.rs:57`), `PlanView::derive` (`:185-215`, defined → edge → covering range, lowest
`first_addr` → not-covered), `ranges_in` read before the ceiling (`:343-346`). 🔴 **`offerable()`
(`:141-143`) offers `reserved` and `dhcp-pool` cells** — measured `198.51.100.129` on the seed's
Workshop subnet (`reserved` only). `counts()` counts observed addresses as free (seed: `1 defined · 86
free · 2 infrastructure · 39 not covered`). `next_free_caveat` (`app.yml:840-842`) says the network was
not consulted; `ipam.next_free_none` (`app.yml:838`) is reachable since 14.2 (`ipam_page.rs:607`) with
NO render test, its text false after this story, and its register row (`deferred-work.md:4403-4409`,
6b.7's dataset) stale.

Tests pinning today's model, to update deliberately: offerable `254`, `next_offerable == 192.0.2.1`;
the hand-written state lists (`every_state_carries_its_own_key_and_its_own_modifier`,
`the_stylesheet_defines_every_modifier_the_grid_can_emit`, `the_blank_cell_is_the_one_modifier_no_legend_entry_carries`
`:1016-1020`); `the_four_states_carry_four_distinct_fills` (asserts a background is DECLARED, not
distinct); `aria-label` count `258` (`:1420`); every three-argument `PlanView::derive`; `main.rs`'s
selector test.

### (g) The marker: possible, and failing contrast where axe cannot see

Chrome 151, 1280 px, cell box **15.34 px**: a 4 px `::after` dot, a 2 px inset `box-shadow`, a
diagonal stroke all distinguish every pair, axe 0 violations — 🔴 but `rgb(29,31,32)` on the defined
fill `rgb(65,97,128)` is **2.56:1** (< 3:1, WCAG 1.4.11) on the held cell, and axe does not measure
pseudo-elements or box-shadows. Decision 8: kept only at ≥ 3:1 on every fill, checked in the gate.

### (h) The triage link depends on the DECLARED register

`/triage?sel=nouveau:<ip>` (`page.rs:1203`, `row_href` `:1320`, `url_escape` `:1333`) exists only for an
observed address `declared_attribute` does not claim. Seed: the only `nouveau:` row is `192.0.2.99`
(pool, not highlighted); `.11/.12/.13` are `ecart:` rows. One sighting per audit case grows the queue
**5 → 10 rows**, which both gates' row-finding must survive.

### (i) The seed reaches no audit state

`a11y/seed.sql` (`:54-82`, `:146-160`): no `Mac` facts; `.11/.12/.13` free `static`, `.99` pool; Office
`static` .1–.40, `dhcp-pool` .80–.126, defined `.9`, .41–.79 uncovered; Workshop `reserved` .129–.200.
Nothing refuses a seeded run that draws no audit state. An address in no plan subnet (e.g. `10.9.9.9`)
has no `/ipam` screen today (decision 11). ⚠️ After 14.3a, the seed must also populate the SUMMARY, or
`/ipam` sees no sighting at all. ~~or the backfill must run over it~~ — **struck at 14.3a's validation**:
the seeds run AFTER boot (`ci.yml:111` starts the binary, `:146` seeds), so a backfill that runs once at
boot never sees seed rows; story 14.3a's AC8 makes both seeds write their own summary from one `@t`.

### (j) File sizes (code lines before the first line-start `#[cfg(test)]`)

`ipam_page.rs` **729**, `ipam_repo.rs` **752**, `ipam_write.rs` **854**, `page.rs` **1954 — it must not
grow**. Re-measure after 14.3a.

### (k) What holds by STRUCTURE, owed a check

NFR8(a) (`prd.md:1324`) and FR19 (`prd.md:902`): the audit derives from RECORDED sightings only, so a
blind scanner cannot add one — owed a test (an empty sighting set yields no finding; no clock read).

## Acceptance Criteria

**AC1 — `undeclared`, per decision 1**: an observed address with no address row, NOT offerable, in a
`reserved` range, an `infrastructure` range or uncovered space → `undeclared` (the existing
`state.undeclared` key); inside a `dhcp-pool` → no finding; on a defined address → no finding (a held
cell). The most-protective rule (decision 2) is tested across overlapping ranges AND nested subnets.
The epic's AC1 letter for `static` is registered as a divergence.

**AC2 — `gap`, per decision 1**: an observed address the plan would OFFER → `gap` (the existing
`triage.kind.ecart` key), never inside a `dhcp-pool`; distinct from `undeclared` by word and treatment,
asserted in the HTML and in the browser.

**AC3 — THE OFFER EXCLUDES EVERY OBSERVED ADDRESS.** An address is offerable only if it is not an edge,
not defined, not seen (14.3a's summary), not documented (decision 4), and every range covering it in
any subnet is `static` (decisions 2–3). Tests: the seed's Workshop subnet (`reserved` only → no
offer), a `dhcp-pool`-only subnet (no offer), a `static` range over a seen address, a documented
address, an edge under a `static` range, a nested subnet. `offerable`'s doc corrected; the caveat says
what WAS consulted; `ipam.next_free_none` RENDERED by a test with text naming the new exclusions in
both locales; its stale register row closed. 🔑 **Proven red by the mutation that re-admits ONE seen
address, its carrier named.**

**AC4 — the form warns BEFORE the write, names what it knows, links, and STILL WRITES** (decision 7):
the address field asks a read fragment; for a seen address it names each MAC with its absolute last
sighting, links to `/triage?sel=nouveau:<ip>` WHEN that row exists and says so when it does not (both
branches tested); for a documented or plan-defined address it names that too; the POST still succeeds.
The range form warns when its `static` range covers seen addresses, or the story says why not. The
fragment route answers 401 without a credential and exists with one — **a perimeter test that names
it** (it is neither a POST nor a `Screen`) — is budgeted, keyed in both locales, and walked by both
browser gates, which define and assert its focus contract.

**AC5 — « Conflit d'adresse »** (decision 10): two MACs on one IPv4 inside a `static` or `reserved`
range → the conflict, BOTH addresses with each one's last sighting, copy naming the
succession-or-duplicate ambiguity; in a `dhcp-pool`, in uncovered space, the same MAC twice, a sighting
with no MAC → none. Each case tested.

**AC6 — a held cell carries its last sightings** (decision 9): absolute, per MAC, in its accessible
name and in the findings list; the same data rendered at two clock instants gives identical output;
the derivation reads no clock.

**AC7 — the plan-against-plan warning** (decision 13): an address defined inside a `dhcp-pool` is shown
with a keyed warning, the write still succeeds; register row `deferred-work.md:5526` closed.

**AC8 — the guard is narrowed, in three parts, each proven red** (decision 6): (i) an `identity_link`
literal — and a `declared_attribute` or sighting read outside the allowed module — reds; (ii) the
sighting reader named from any OTHER plan file reds; (iii) `ipam_write.rs` cannot read the network
because `IpamWriteState` holds no pool — proven by a compile-fail mutation. The two measured greens are
written as limits; the failure message and every stale module doc corrected.

**AC9 — bounded and measured**: the sighting read (14.3a's reader) and the documented read inside
`store_within`, the size ceiling tested before anything is materialised; `/ipam` timed end to end at
**≥ 1 M observation rows** (≈ 75 days at the reference rate) and at the summary's size, time AND peak
memory recorded with their command, against NFR2's p95 1.5 s.

**AC10 — the browser gates REACH the audit**: the seed gains sightings with `Mac` facts for each case
(uncovered, `reserved`, `infrastructure` range, defined, two MACs in `static`, inside the `dhcp-pool`,
outside every subnet, a documented one with no triage link and an undocumented one with a link), and
the summary is populated for them; the axe gate REFUSES (exit 2) a seeded run with no audit state (a
`REQUIRE` flag in `ci.yml`); a computed-contrast check covers the marker on every fill (if kept); both
gates' row-finding survives the larger queue; the kbd-probe reaches the warning fragment and the
findings list.

**AC11 — structure** (§1(k)): no sighting → no finding; no clock read.

**AC12 — outside every subnet** (decision 11): a plan-wide list names observed addresses in no subnet
of the plan, with their last sightings; tested.

**AC13 — the occupancy line and large subnets** (decisions 12, 14): `free` counts only offerable
addresses; no count of seen addresses; a subnet too large to draw says it has no offer and shows its
findings list; tested.

**AC14 — THE LIVE COUNT lives here**, both store conditions named (a virgin store after one warm run;
`env -u DATABASE_URL`, never `DATABASE_URL=`), baseline re-measured after 14.3a.

**AC15 — no regression**: ten gates, `clippy --workspace --all-targets -D warnings`, `RUSTFLAGS="-D
warnings"`, fmt, `cargo deny`, **both browser gates claimed and run**; no file over 2000 code lines
(`page.rs` does not grow); docs current — the user manual's IPAM chapter (`user-manual.tex:180-184`),
both twins, the register.

## §3 — What the validation measured that this story carries

Two fresh-context layers on `7373f9f`, 2026-09-15 — fact-check (3 HIGH, 7 MED, 6 LOW) and gap-hunt
(3 HIGH, 6 MED, 2 LOW) — then Guy's fourteen decisions. The findings that changed the story's shape:

- 🔴 **The offer serves `reserved` and `dhcp-pool` addresses** (measured `198.51.100.129`) → decision 3.
- 🔴 **Reusing the observation reader fails NFR2 after ≈ 2 months** (1 M rows) → decision 5, the SPLIT.
- 🔴 **AC8 as first written could not be carried by the guard** → three parts, limits written.
- 🔴 **§0 fused the epic's constraints with Guy's arbitrations**; constraint (4) was given a rule it does
  not contain → rewritten.
- ⚠️ FR24's two MACs are a succession (`neighbour.rs:83`) → decision 10; no option answered an
  `infrastructure` range and AC2's letter made every lease a `gap` → decision 1(d); a marker fails 3:1
  and axe cannot see it → decision 8; a URL-carried warning survives reload → decision 7(b), whose GET
  route no perimeter guard covers → AC4.
- ⚠️ Facts corrected: 14.1's seam is with `declared_attribute`; `/ipam`'s error path; `next_free_none`
  reachable since 14.2; issue #150 not in the register row; "one URL per state" is constraint 4; F57 is
  not the containment rule; citations re-derived; the cell is 15.34 px.

## Tasks / Subtasks

- [x] **T0** Take §2's decisions with Guy. ✅ 2026-09-15 — all fourteen, the recommendations in every
  case; the story split (14.3a / 14.3b).
- [x] **T1** (AC11, AC5, AC6) The pure audit core: from sightings (`Ipv4Addr` → per-MAC last seen),
  documented addresses and a plan, derive each address's finding and offerability — no store, no clock;
  tests first.
- [x] **T2** (AC1, AC2, AC7) Verdicts per decisions 1, 2, 10, 13, including nesting.
- [x] **T3** (AC3, AC13) The offer; the caveat; `next_free_none` rendered and reworded; `offerable`'s
  doc; the occupancy line; the too-large branch.
- [x] **T4** (AC8, AC9) The reads (14.3a's reader, the documented read) inside the budget, after the
  ceiling; the guard narrowed with its three-part mutations; module docs corrected; the end-to-end
  timing at ≥ 1 M rows.
- [x] **T5** (AC1, AC2, AC5, AC6, AC12) The screen: findings list, marker (if it reaches 3:1),
  outside-every-subnet list, keyed words, absolute dates, legend literals, stylesheet — §1(f)'s tests
  updated deliberately. *(No marker: it cannot reach 3:1 on every fill — see the record.)*
- [x] **T6** (AC4) The warning fragment route with its perimeter test, budget and keyed copy; the range
  form's case. *(The range form's case is said and registered, not built — see the record.)*
- [x] **T7** (AC10) The seed (and the summary for it), the `REQUIRE` flag, the contrast check, both
  gates' row-finding, the kbd-probe; both gates run.
- [x] **T8** (AC14, AC15) Measure; docs; the live count.

### Review Findings

Three isolated layers, 2026-09-15, on `9f13513`: Blind Hunter (code diff only), Edge Case Hunter (own
worktree, store and server; measured MAC churn, locks, planted reads), Acceptance Auditor (full diff, spec,
register; re-ran MA1, MG2, both gates). 39 raw findings (20 blind, 8 edge, 11 auditor) → **33 after dedup:
4 decisions, 25 patches, 1 deferral, 3 dismissed** — every one applied.

⚠️ **This header read *"34 after dedup: … 26 patches"* until the repair pass RECOUNTED the list below and
found 25.** The number was written beside the list rather than derived from it, which is this project's
own four-time defect (*a tally written in flight, inside the document explaining why not to write counts in
flight*). It is corrected here rather than defended, and the list is the record.

- [x] [Review][Decision] **The page grows without bound under MAC churn and outside sightings** (edge,
  blind; auditor on "short") — measured 2.26 MB, 5 007 findings, a 13 688-byte cell name, axe 75 s, on 47
  pool addresses × 200 MACs + 5 000 outside addresses. Caps for the cell name, the outside list (decision
  11 says *short*) and the findings list are a design choice.
- [x] [Review][Decision] **A nested subnet's page offers an address its grid draws as not covered, and
  counts it twice** (blind) — the offer and the verdict read plan-wide ranges, the grid only the subnet's
  own; a sighting there reads `gap` over a blank cell.
- [x] [Review][Decision] **The range form's warning was deferred by the implementer, not by Guy**
  (auditor) — §1(e) said decision 7 covers it; AC4 permits *"or says why not"*.
- [x] [Review][Decision] **The marker was dropped on a false argument** (blind, auditor) — a near-black
  marker reaches 3:1 on the defined fill (≈ 3.25) and on the ground (≈ 18.8); the "≤ 0.30" bound is 0.263;
  the argument named two fills of four. Decision 8's condition is reachable.
- [x] [Review][Patch] **One unreadable range or address row anywhere takes down every `/ipam` page and every
  check** (blind) — `plan_ranges`/`plan_addresses` must skip and name, as `list_subnets` does [ipam_repo.rs]
- [x] [Review][Patch] **AC3's proof re-admits EVERY seen address; re-admitting only MAC-less sightings left
  677/191/99, clippy and the gates green** (auditor MX1b) — test the MAC-less shape and record the mutation
  [ipam_audit.rs tests]
- [x] [Review][Patch] **The triage link disagrees with triage on a declared value with whitespace**
  (edge measured, auditor) — the link must use triage's raw comparison; the offer keeps the protective parse;
  and a gate follows the link to a real row [ipam_audit.rs, ipam_page.rs, kbd-probe.mjs]
- [x] [Review][Patch] **The empty-offer sentence is vacuous on a subnet with no static range and false when
  an overlapping range alone empties it** (auditor, blind) — two sentences, both rendered [app.yml,
  ipam_page.rs]
- [x] [Review][Patch] **The address check does not say "no triage question" when there is none** (auditor)
  [_ipam_address_check.html]
- [x] [Review][Patch] **The outside list is missing on the empty and unknown-subnet pages** (auditor)
  [ipam_page.rs]
- [x] [Review][Patch] **A failed or timed-out check leaves the previous address's warning under a new value;
  a render error would put a whole error page in the live region** (edge measured, blind) — answer a keyed
  "could not check" fragment [ipam_page.rs]
- [x] [Review][Patch] **Both gates report a product regression as "could not run" once the page and the form
  exist** (blind) [axe-gate.mjs, kbd-probe.mjs]
- [x] [Review][Patch] **The "Tab from the top" walk starts from the address field** (blind) — walk a fresh
  page [kbd-probe.mjs]
- [x] [Review][Patch] **The browser compares only border-style of two words; « Conflit d'adresse » is never
  compared** (blind, auditor) [axe-gate.mjs]
- [x] [Review][Patch] **The seed walks no pool warning and no reserved or defined conflict** (auditor)
  [a11y/seed.sql]
- [x] [Review][Patch] **The debounce and the live region are each held by one thing** (edge) [ipam_page.rs,
  kbd-probe.mjs]
- [x] [Review][Patch] **A /31 or /32 subnet takes its own addresses out of the offer** (edge measured) —
  RFC 3021: no edges at 31 and 32 [ipam_repo.rs]
- [x] [Review][Patch] **The module doc says nothing in a dhcp-pool is a finding; an overlap with static is**
  (blind) [ipam_audit.rs]
- [x] [Review][Patch] **Merging across L2 domains turns reused private space into conflicts — unstated and
  untested** (blind) [ipam_audit.rs, register]
- [x] [Review][Patch] **Tests passing for the wrong reason**: the edge cases sit in uncovered space; the
  infrastructure exclusion from conflicts is untested (blind) [ipam_audit.rs tests]
- [x] [Review][Patch] **The guard misses a read made by calling another module's reader** (edge planted
  `scan_pass::counted_current_engine_links`, green) — a third limit, stated [ipam_page.rs]
- [x] [Review][Patch] **The user manual says "safe to assign"** — the audit cannot see a sleeping machine or
  anything outside the scan perimeter (auditor) [user-manual.tex]
- [x] [Review][Patch] **Stale module docs AC8 named** — `ipam_write.rs:22-25`, `opencmdb-core` `ipam/mod.rs`
  `:22-24`, `:43` (auditor)
- [x] [Review][Patch] **The record overstates**: `next_free_none` rendered in English only while the register
  closes it; AC6's clock claim rests on two back-to-back renders; AC9 measured `plan_data`, worst of 20, not
  HTTP p95; the register's closure sits above its stale paragraph (auditor) [this file, register]
- [x] [Review][Patch] **The axe gate counts `/ipam` violations twice and skips silently without the form**
  (blind) [axe-gate.mjs]
- [x] [Review][Patch] **Comments contradicted by their own file** — the seed's "each with a hardware address",
  `FindingKind::Gap`'s "the plan would offer it" for a documented address (blind) [seed.sql, ipam_audit.rs]
- [x] [Review][Patch] **The probe address is duplicated in the keyboard gate** (blind) [kbd-probe.mjs]
- [x] [Review][Patch] **The warning is silent for reserved, infrastructure, edge and outside addresses, and
  announces intermediate addresses while typing** (blind) — registered as a stated limit [register]
- [x] [Review][Patch] **The measurement's generator holds distinct MACs at 46**, so "their number no longer
  reaches the screen" is true of rows, not of pairs (blind, edge) [ipam_page.rs doc, record]
- [x] [Review][Defer] **A check waiting on a metadata-locked summary keeps its pool connection; fourteen
  exhausted the pool and `/triage` failed** (edge measured) — reads carry no lock-wait cap, a class every
  screen shares; narrow trigger (DDL, `LOCK TABLES`) [ipam_page.rs] — deferred, cross-cutting
- Dismissed (3): leading-zero input silently unchecked (edge refuted: the write route refuses the same
  spelling, so the warning never goes silent on a spelling that writes); a repeated `addr` parameter answers
  axum's plain 400 (unreachable through htmx); the opt-in measurement clears its store (documented, opt-in,
  like 14.3a's).

## Dev Notes

### Traps this project has paid for, and which apply here

- 🔴 **A guard placed where the defect cannot occur reads as coverage and is none**; the narrowed guard's
  limit is measured (a non-`ipam` file is invisible to it).
- 🔴 **A budget that bounds the wrong half of a handler is none**; a read that grows with time is
  measured in DAYS of sweeping, not in rows chosen to pass.
- 🔴 **A test that pins the ugly thing demands it** (`offerable == 256`; the reserved offer, unpinned).
- 🔴 **axe cannot see a pseudo-element or a box-shadow** — a marker needs a computed-contrast check.
- 🔴 **A new GET route is in no perimeter guard by default** — story 6b.2's defect.
- 🔴 **A guard that greps a file greps its prose** — `src/source_scan.rs`.
- 🔴 **A hand-written state list cannot see a new state.**
- ⚠️ `app.yml` is invisible to incremental builds; `ascii_bin` is PAD SPACE; store-backed tests take
  `DB_TEST_LOCK` and own their CIDR, warm once, drop/recreate before counting; `cargo xtask mutate`
  reports counts, never names; `cargo fmt` re-wraps anchors; "registered" means a row exists; a cause
  needs a check.

### What the store must answer, and what it must not

The audit reads the PLAN (`list_subnets`, `ranges_in`, `addresses_in`), the SIGHTINGS from 14.3a's
summary, and the DOCUMENTED IPv4 values (decision 4), the last two from the one module decision 6 names;
it never reads `identity_link`. No migration in this story — 14.3a carries it.

### Project Structure Notes

- The pure audit where the guard can name it; D47 errors in `ipam::IpamError`, no sqlx in core;
  `float-free` walks `identity/` only — occupancy stays counts.
- `page.rs` must not grow; the IPAM files have room — split before the ceiling.
- No new dependency.

### References

- `_bmad-output/planning-artifacts/epics.md` — constraints `:2345-2357`, Story 14.3 `:2411-2441`.
- `_bmad-output/planning-artifacts/prd.md` — FR19 `:902`, FR21–FR24 `:906-909`, NFR2 `:1235`, NFR8(a)
  `:1324`, NFR25 `:1467`, STATE axis `:1004-1023`, PLAN axis `:1025-1060`.
- `_bmad-output/planning-artifacts/ux-design-specification.md` — grid `:529-531`, `:1227-1231`,
  accessibility `:1632-1636`, bans `:1509-1522`.
- `_bmad-output/planning-artifacts/architecture.md` — `:239`, D47 `:2639`, `ipam/` `:3208`, `:3366`.
- `_bmad-output/implementation-artifacts/deferred-work.md` — nested subnets `:5510`, edges `:5519`, an
  address inside a range `:5526`, the unbounded read `:5383-5401`, `next_free_none` `:4403-4409`,
  « Conflit d'adresse » `:4338`, `role="grid"` `:5420`.
- Story 14.3a (the sighting summary); stories 14.1, 14.2, 14.2b.

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context), `claude-opus-5[1m]`.

### Debug Log References

- Store: `mariadb:10.11` on port **13450** (never 3306): `opencmdb_143b` for the suite and the mutations,
  `opencmdb_143b_gate` for the browser gates, each dropped and recreated before every count and mutation.
- Browser gates run as `ci.yml` runs them: a debug build serving on :8080 with
  `OPENCMDB_DOCUMENT_ENABLED=1`, `a11y/empty-plan.sql` then the empty-plan axe pass, `a11y/seed.sql` then
  axe with `AXE_REQUIRE_QUEUE/GESTURE/PLAN/AUDIT=1`, re-seed, then `kbd-probe.mjs`. Logs in the session
  scratchpad (`g-*.log`, `c-*.log`, `bmut-*.log`).

### Completion Notes List

- Contexted 2026-09-15 from three research passes; validated the same day (fact-check + gap-hunt that
  built and measured); split and decided by Guy the same day. Unblocked by story 14.3a's merge
  (PR #179, `8d8a45f`) and developed the same day.

**What shipped, by AC.**

- **AC1 / AC2 — MET.** `ipam_audit.rs` (new) derives, from story 14.3a's summary merged across L2
  domains, the documented addresses and the WHOLE plan: `gap` = observed and the plan would offer it;
  `undeclared` = observed, no address row, not offerable (a `reserved` or `infrastructure` range, an edge,
  uncovered space); nothing inside a `dhcp-pool`; a defined address is a held cell. The most protective
  range decides across overlapping ranges AND nested subnets (a nested /26's edge is an edge too).
  On screen the two words are the binding keys (`triage.kind.ecart`, `state.undeclared`) with two
  treatments — a filled badge and a dashed outline — and the axe gate compares them WITHOUT colour in the
  browser (`solid vs dashed`). ⚠️ The epic's AC1 letter for `static` is the registered divergence.
  🔴 **Since the code review the GRID is derived from the same plan-wide ranges** (Guy's decision): it read
  the selected subnet's own, so a NESTED subnet's page drew as *not covered* an address a parent's
  `reserved` range protects, counted it as not covered, and named it a `gap` in the list underneath — one
  address, one page, three answers. The overlap now resolves to the MOST PROTECTIVE covering range, which
  is what the offer and the verdict already did, so the three agree by construction rather than by a
  comment asking them to. ⚠️ And the cell's accessible name is **bounded to three hardware addresses**
  (the review measured a 13 688-byte name under MAC churn); the list keeps every one of them.
- **AC3 — MET.** `ipam_audit::Plan::offerable`: inside a subnet, not an edge, not defined, every covering
  range `static` (at least one), never seen, never documented. `CellState::offerable` and
  `PlanView::next_offerable` are GONE — a cell cannot answer a plan-wide question — and §1(f)'s tests were
  moved to the plan's answer deliberately. The seed's reserved-only Workshop offers nothing (it offered
  `198.51.100.129`). The caveat and `ipam.next_free_none` are reworded; the latter is RENDERED by a test
  (English, the default locale). 🔑 **Proven red by MA1**, which re-admits every seen address: 2 tests
  (`the_offer_excludes_seen_documented_and_everything_outside_static` and
  `the_occupancy_and_the_empty_offer_follow_the_audit`), plus clippy on the unused parameter.
  🔴 **And that proof re-admitted EVERY seen address, so a mutation re-admitting only the MAC-LESS ones
  came back GREEN** (acceptance layer, MX1b): every sighting in every offer test carried a hardware
  address, so *seen* and *seen with a MAC* were one population. It is the ordinary shape on the shipped
  product — the connector reads a MAC only when the neighbour table has one — and
  `an_address_seen_with_no_hardware_address_is_still_never_offered` is what now separates them.
  🔴 **A `/31` and a `/32` had their own addresses taken out of the offer** (edge layer): `is_edge`
  compared against both bounds, which on a `/31` is every address it has, so a point-to-point link
  offered neither of its two and a `/32` host route offered nothing — RFC 3021 says both are usable, and
  the prefix length now settles it. ⚠️ The empty offer says **which of two states** it is: a subnet no
  `static` range reaches was never offering anything, where an exhausted one names the reasons.
- **AC4 — MET for the address form; the range form's case is SAID, not built.** `GET /ipam/address-check`
  answers, as the operator types (`hx-get`, `input changed delay:400ms`), what the network and the
  registers know: seen (each MAC with its absolute last sighting, and the triage link when triage asks),
  documented, already defined, inside a `dhcp-pool` — and *"You can still define it"*; the POST is
  untouched. A polite live region described by the field; focus stays on the field. Named in a perimeter
  test (401 without a credential, 200 with one), and probed by name in the page-budget guard. The
  keyboard gate types `192.0.2.20` and reads the warning, the unmoved focus and the link; the axe gate
  runs axe over the page with the warning shown. **MW** (route unmounted) reds 2 tests and clippy.
  🔴 **The range form's warning is BUILT at the code review and this line said it was registered** — the
  deferral was the implementer's and not Guy's, which the acceptance layer named: §1(e) claimed decision 7
  covered the range form, and AC4's *"or says why not"* was spent on a reason nobody had taken.
  `GET /ipam/range-check` now answers *n addresses already seen on the network would fall inside this
  static range*, on the address check's own contract — it warns, it refuses nothing, the POST is
  untouched — with its own live region, its own perimeter half, its own budget probe and two keyboard-gate
  checks. **AC4 is MET on both forms.**
- **AC5 — MET.** « Conflit d'adresse » only for two MACs inside a `static` or `reserved` range; not in a
  pool, not in uncovered space, not the same MAC in two L2 domains, not a MAC beside a no-MAC sighting; a
  defined address can conflict. Both MACs shown with their dates, and a sentence that a replaced card and a
  duplicate look the same. **MA5** (a no-MAC sighting counted as a MAC) reds 1.
- **AC6 — MET, and its clock claim is narrowed to what the test measures.** A held cell's accessible name
  and each finding carry every hardware address with an ABSOLUTE date (`YYYY-MM-DD HH:MM UTC`).
  ⚠️ *"The render is pure"* rests on **two back-to-back renders being identical**, which is a weak
  instrument and is now said to be one: it catches a clock read at render time only if the clock ticks
  between the two calls. What really carries the property is that the derivation is handed no clock —
  `ipam_audit` takes no `Timestamp` argument and calls nothing that reads one — and story 6b.4's M6
  measured the mirror trap (`chrono::Utc::now()` is `E0599` in the domain crate while `SystemTime::now()`
  is not, so *a feature flag guards a spelling, never the act of reading a clock*). **MA6** (an hour
  instead of a date) reds 2. ⚠️ A cell's name is **bounded to the three most recent** hardware addresses
  since the code review; the findings list keeps every one of them.
- **AC7 — MET.** An address defined inside a `dhcp-pool` is listed under *Defined inside a DHCP pool* and
  warned about before the write; register row closed.
- **AC8 — MET, three parts, each proven.** `the_plan_reads_the_network_only_through_the_audit` (renamed):
  the perimeter is now four files; no plan module names `observation_record`, `identity_link`,
  `declared_attribute` or `address_sighting`; only `ipam_audit.rs` names `sighting_repo` or
  `crate::repo::load_documented_ipv4s`. **MG1** (`identity_link` in `ipam_audit.rs`) reds 1; **MG2** (the
  sighting reader named from `ipam_page.rs`) reds 1; **MG3** (`declared_attribute` in `ipam_page.rs`) reds
  1; **MG4** (`State<MySqlPool>` added to a write handler) is a compile refusal, `E0277` on the `Handler`
  bound. The two measured greens are written as limits in the guard's doc; `ipam_page.rs`'s module doc is
  corrected.
- **AC9 — MET.** The reads — the plan whole (`ipam_repo::plan_ranges`, `plan_addresses`), the summary and
  the documented values — run inside `store_within`, before the ceiling decides whether to draw; none is
  grid-sized. The end-to-end measurement is recorded in the Change Log with its command.
  ⚠️ **What it measures, said rather than implied** (acceptance and blind layers): `plan_data` — the read
  and the render — **worst of twenty**, and NOT an HTTP p95: no server is bound in that test, so the shell,
  the response and the socket are outside the figure. And the generator holds the DISTINCT hardware
  address count at **46** (`(seq - 1) MOD 46`), so *"their number no longer reaches the screen"* is true of
  observation ROWS and says nothing about a network whose distinct pairs grow — which is the axis story
  14.3a's register row names and the one the summary does not bound.
- **AC10 — MET.** `a11y/seed.sql` gains one sighting per case (a two-MAC gap, a documented gap, uncovered,
  an `infrastructure` range added for it, a defined address, the reserved Workshop, outside every subnet)
  with their summary rows from `@t`; `AXE_REQUIRE_AUDIT=1` in `ci.yml` refuses a run without findings or
  without both words to compare; row-finding survives the longer queue; the kbd-probe reaches the warning
  and the findings list (37 → **45** checks after the code review). 🔑 The browser comparison was proven
  red by hand: `.ipam-word-undeclared` set to `solid` makes the axe gate exit 1 (Change Log).
  🔴 **THE MARKER EXISTS, and this paragraph said the condition could not be met.** The arithmetic was
  wrong in the safe direction: it required ONE colour to clear both bounds and named two fills of four.
  ⚠️ A `::after` dot is **invisible to axe**, so the gate computes the ratio in the browser against all
  four fills and fails under 3:1: *a treatment no gate measures is a treatment nobody maintains*.
  ✅ **MEASURED BY THE GATE ITSELF, not by my hand**: `ipam-cell-defined` **3.25:1**, and the other three
  **18.77:1** each (patterns over the page ground, which is what the eye sees between their strokes), over
  8 seen cells. 🔑 *My own estimate for the ground was 18.6 — close, and still an estimate; the figure
  that belongs in a record is the one an instrument produced.* `--color-text` (#1d1f20) reaches **2.56:1**
  on the defined fill and would NOT have met the condition, which is what makes the marker black rather
  than *near enough to the text colour*.
  ⚠️ The seed also gained the three cases it walked past — an address defined inside the pool (decision
  13's warning), a conflict under `reserved`, and a conflict on the DEFINED address — so two of the
  audit's rules stopped being on no page either gate opens.
- **AC11 — MET.** No sighting → no finding and no offer withheld (a test); no clock anywhere in the core.
- **AC12 — MET, bounded, and on three pages rather than one.** Observed addresses outside every subnet are
  listed plan-wide on `/ipam`, with dates — the **twenty most recently seen**, and the list SAYS how many
  it is not showing (the review measured 5 000 of them on one page, 2.26 MB, axe 75 s). 🔴 **The two pages
  with no subnet in force showed none of it**: an empty plan and an identifier no subnet carries dropped
  the list entirely, and on an empty plan every observed address is outside the plan — *the only true
  thing that screen has to say on a fresh install*.
- **AC13 — MET.** The occupancy line counts as free only what the offer proposes (*free to assign*); no
  count of seen addresses; a subnet too large to draw says it offers nothing and shows its findings.
  ⚠️ Since the code review the count is taken over the PLAN-WIDE cells, so an address protected by a range
  declared in another subnet is no longer counted as *not covered* on one page and as protected on the
  next — the blind layer's *counts it twice*.
- **AC14 — THE LIVE COUNT.** Baseline **950** (660 + 191 + 99) at `8d8a45f` (story 14.3a merged). At
  implementation: **967** (677 + 191 + 99) — 9 tests in `ipam_audit.rs`, 7 in `ipam_page.rs` (one of them
  the opt-in `/ipam` measurement, which returns at once unless `OPENCMDB_MEASURE_IPAM` is set) and 1 in
  `main.rs`. **After the code review's repair: 980** (690 + 191 + 99), **+13** — 5 in `ipam_audit.rs` (the
  MAC-less offer, the triage link against triage's own comparison, a `/31` and a `/32`, the
  `infrastructure` conflict exclusion on covered space, the L2-domain merge read as a conflict), 7 in
  `ipam_page.rs` (the nested subnet's page agreeing with itself, the two bounded lists, the two
  empty-offer sentences, the range check, the *could not check* fragment, the two subnet-less pages, the
  marker) and 1
  in `ipam_repo.rs` (which spellings this build calls unreadable). ⚠️ **No test was deleted**: the
  order-by-the-store test was RE-AIMED at `plan_addresses` when `addresses_in` lost its last production
  caller and was removed. Both store conditions in the final verification (Change Log).
- **AC15 — see the Change Log's verification line.** The user manual's IPAM chapter gains *What the network
  shows against the plan* and *The next address the plan can offer*; both twins (the browser gates and the
  status); the register (two rows closed, five raised).

**Mutations** — `cargo xtask mutate`, each on a virgin store, counts and never names; the carrier column is
the test written for that defect.

| id | mutation | predicted | measured | carrier |
|---|---|---|---|---|
| MA1 | the offer re-admits seen addresses | red | 🔴 2 + clippy | the offer test and the occupancy/empty-offer test |
| MA2 | the offer re-admits documented addresses | red | 🔴 2 + clippy | the same two |
| MA3 | the offer from any non-infrastructure range | red | 🔴 5 | the verdict, nesting, offer, occupancy tests |
| MA4 | a `reserved`-only coverage treated like a pool | red | 🔴 3 | the verdict and findings tests |
| MA5 | a no-MAC sighting counted as a second MAC | red | 🔴 1 | `two_mac_addresses_conflict_only_inside_static_or_reserved` |
| MA6 | an hour instead of an absolute date | red | 🔴 2 | the absolute-date tests |
| MG1 | `identity_link` in `ipam_audit.rs` | red | 🔴 1 | the plan guard, part 1 |
| MG2 | the sighting reader named from `ipam_page.rs` | red | 🔴 1 | the plan guard, part 2 |
| MG3 | `declared_attribute` in `ipam_page.rs` | red | 🔴 1 | the plan guard, part 1 |
| MG4 | a pool injected into a write handler | compile-fail | 🔴 `E0277` | the `IpamWriteState` type, part 3 |
| MW | the address check unmounted | red | 🔴 2 + clippy | the perimeter test and the budget guard |

**The code review's repair pass — ten mutations, each on a DROPPED-AND-RECREATED store with `--baseline`,
predictions written before the run.** Nine conformed; **one contradicted its prediction, and it is worth
the other nine.**

| id | mutation | predicted | measured | carrier |
|---|---|---|---|---|
| MC1 | the cell name's three-MAC cap raised to 99 | red | 🔴 conforms | the bounded-lists test |
| MC2 | the outside list's twenty-address cap raised to 500 | red | 🔴 conforms | the bounded-lists test |
| MD2 | the covering range resolved by `.next()` instead of the most protective | red | 🔴 conforms | the nested-subnet agreement test |
| MK1 | the seen marker never emitted (`seen: false`) | red | 🔴 conforms | the marker test |
| MR1 | the range check mounted at another path | red | 🔴 conforms | the perimeter test and the budget guard |
| MU1 | the *could not check* fragment replaced by an empty body | red | 🔴 conforms | `main.rs`'s budget guard, which reads the sentence |
| MP4 | both empty-offer states given the same sentence | red | 🔴 conforms | the two-sentences test |
| MP6 | `subnet_in_force` forced true on every page | red | 🟢 **GREEN — contradicted**, then 🔴 1 after the repair | the subnet-less pages test, rewritten |
| MX1b | the offer re-admits a sighting with no hardware address | red | 🔴 conforms | `an_address_seen_with_no_hardware_address_is_still_never_offered` |
| ME1 | the `/31`–`/32` edge rule reverted | red | 🔴 conforms | `a_point_to_point_link_offers_both_its_addresses` |

🔴 **MP6 IS THE PASS'S OWN FINDING, AND IT WAS MY ORACLE THAT WAS WORTHLESS, NOT THE PRODUCT.** The test
asserted `!body.contains(&t!("ipam.findings.none"))`; that key's value is *"Nothing the network has shown
contradicts this subnet's plan."*, **Askama escapes the apostrophe to `&#x27;`**, and a resolved string
never matches an escaped render — so the negative assertion passed over a page that carried the sentence.
Measured rather than reasoned: the plant was confirmed present at `ipam_page.rs:848` and the crate
recompiled before the run, then the rendered body was PRINTED — the heading appeared twice and the
sentence once. 🔑 ***A `contains` oracle over a translated sentence can only fail in the direction
escaping does not touch, and the negative direction is the one it cannot measure.*** The needle is now the
`id` the template emits as a literal, and MP6 reds 1.

🔴 **AND THE REPAIR EXPOSED A DEFECT NO ASSERTION AND NEITHER BROWSER GATE HAD SEEN**: with no subnet in
force the page emitted an **empty ARIA landmark whose `aria-labelledby` named an `id` the page does not
carry**. Both gates had been green over it. The section is now rendered only when it has content, and the
attribute only where its target exists.

### File List

- `crates/opencmdb-bin/src/ipam_audit.rs` — new
- `crates/opencmdb-bin/templates/_ipam_audit.html`, `_ipam_address_check.html` — new
- `crates/opencmdb-bin/src/ipam_page.rs` — the audit on screen, the address check, the guard, tests
- `crates/opencmdb-bin/src/ipam_repo.rs` — `plan_ranges`, `plan_addresses`; at the code review they SKIP
  and NAME an unreadable row, `Subnet::is_edge` learns RFC 3021 (`/31`, `/32`), `Subnet::overlaps` arrives,
  and **`addresses_in` is REMOVED** — the plan-wide grid left it with no production caller
- `crates/opencmdb-bin/src/ipam_write.rs` — module doc: the two registers are COMPARED since this story,
  where it said they *never touch*
- `crates/opencmdb-core/src/ipam/mod.rs` — module doc, and `IpPolicy`'s *"nothing reads a policy yet"*,
  false since story 14.2 and thoroughly false here
- `crates/opencmdb-bin/src/repo.rs` — `load_documented_ipv4s`
- `crates/opencmdb-bin/src/sighting_repo.rs` — the reader's dead-code allowance removed; the seed test's count
- `crates/opencmdb-bin/src/main.rs` — `mod ipam_audit`; the address check's perimeter test; the budget guard
- `crates/opencmdb-bin/templates/_ipam.html`, `_ipam_forms.html`; `assets/app.css`; `locales/app.yml`
- `a11y/seed.sql`, `a11y/axe-gate.mjs`, `a11y/kbd-probe.mjs`; `.github/workflows/ci.yml`
- `docs/manuals/user-manual/user-manual.tex`; `CLAUDE.md`, `docs/project-context.md`
- `_bmad-output/implementation-artifacts/deferred-work.md`, `sprint-status.yaml`, `14-3a-the-sightings-summary.md`
  (status `done`), this file

### Change Log

- 2026-09-15 — **implemented** (T1–T8): the audit core, the offer, the findings list, the address check,
  the narrowed guard, the seed and both browser gates; 950 → 967 tests; eleven mutations as predicted.
- 2026-09-15 — **AC9 measured**, release build, 32 cores:
  `OPENCMDB_MEASURE_IPAM=1000000 DATABASE_URL=… cargo test --release -p opencmdb-bin
  ipam_page::tests::measure_the_plan_screen_over_a_long_history -- --exact --nocapture` — 1 000 000
  observation rows generated and backfilled in 3.9 s; **`/ipam`'s read-and-render, worst of 20: 7.1 ms**,
  56 679 bytes; whole-process peak resident 9 640 → 10 604 kB (a lower bound of the render's own
  footprint). Against NFR2's 1.5 s, where story 14.3's validation had timed the observation reader at
  3.0–3.3 s over the same rows. ⚠️ At the reference network's 46 distinct addresses; not measured at a
  plan of thousands of ranges (registered).
- 2026-09-15 — **the browser comparison proven red by hand**: `.ipam-word-undeclared` given `border-style:
  solid` makes `node a11y/axe-gate.mjs` (with `AXE_REQUIRE_QUEUE/GESTURE/PLAN/AUDIT=1`) exit **1**, naming
  *"a gap and an undeclared finding carry the same border style (solid)"*; the stylesheet restored from a
  copy, `git diff` empty afterwards.
- 2026-09-15 — **verified on the final tree** (`20190b4` + the record), every status read from a log file:
  `cargo fmt --check` 0 · `cargo clippy --workspace --all-targets -- -D warnings` 0 · `cargo xtask ci` 0
  (ten gates; `file-size` 57 files, largest 1954) · `cargo deny check` 0 · `make` in `docs/manuals` 0 ·
  `RUSTFLAGS="-D warnings" cargo test --workspace --locked` on a virgin store after one warm run **677 + 191
  + 99 = 967**, 22.5 s, and with `env -u DATABASE_URL` **967** · browser gates as `ci.yml` runs them: empty
  plan 0, axe 0 (10 routes + 4 states, 7 findings, the warning measured), kbd 41/41. Status → `review`.

- 2026-09-16 — **the code review's 4 decisions and 25 patches APPLIED, and the tree re-verified whole**,
  every status read from a log file: `cargo fmt --check` 0 · `cargo clippy --workspace --all-targets --
  -D warnings` 0 · `cargo xtask ci` 0 (ten gates) · `cargo deny --manifest-path Cargo.toml check` 0
  (⚠️ the first run of this pass put `--manifest-path` after `check` and exited 2 — **my command, not the
  tree**) · `make -C docs/manuals` 0 · `RUSTFLAGS="-D warnings" cargo test --workspace --locked` on a
  DROPPED-AND-RECREATED store after one warm run **689 + 191 + 99 = 979**, 22.4 s, and with
  `env -u DATABASE_URL` **979**, 5.0 s — the clock is the tell.
  ✅ **Re-verified WHOLE after the MP6 repair, on the final tree**: fmt 0 · clippy `--all-targets` 0 ·
  ten gates 0 · `cargo deny` 0 · manuals 0 · **690 + 191 + 99 = 980** on a dropped-and-recreated store
  after one warm run (22.2 s) and **980** with `env -u DATABASE_URL` (5.0 s) · **MP6 re-measured: RED, 1
  test** · both browser gates re-run because the template had changed under them — **axe (empty plan) 0,
  axe (seeded) 0 over 10 routes + 5 states, kbd 45/45**, with the gate printing the marker's own contrast
  (`ipam-cell-defined` **3.25:1**, the other three **18.77:1**) and the three audit words differing
  without colour (`solid/1px/400` · `dashed/1px/400` · `solid/2px/600`).
  🔴 **Two of MY OWN test expectations were wrong and were corrected in the product's direction, never the
  product's**: a covered broadcast edge with two hardware addresses **IS** « Conflit d'adresse » (decision
  10 grants an edge no exemption — the edge rule belongs to the OFFER), and the seed's premise floor read
  eleven where the seed now sights **thirteen** pairs. ⚠️ **And a method fault of mine is recorded rather
  than hidden**: a template was edited WHILE a measurement was compiling, so that run measured a tree that
  no longer existed; the whole verification was re-run on the final tree with no source touched during it.
- 2026-09-15 — **T0: Guy took all fourteen decisions and SPLIT the story** — 14.3a the sighting summary,
  14.3b this audit. File renamed from `14-3-the-audit.md`.
- 2026-09-15 — validated and rewritten whole: the offer serves `reserved`/`dhcp-pool` addresses today;
  reusing the observation reader fails NFR2 after ≈ 2 months; AC8 could not be carried by the guard;
  FR24's two MACs are a succession; a cell marker fails contrast where axe cannot see; §0 fused two
  lists of six.
- 2026-09-15 — contexted. The epic's AC1 and AC2 found to overlap, and the guard found to forbid the
  audit by design.
