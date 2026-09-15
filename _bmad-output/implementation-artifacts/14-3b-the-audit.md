# Story 14.3b: The audit, and the address the product must not offer

Status: in-progress

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
- [ ] **T1** (AC11, AC5, AC6) The pure audit core: from sightings (`Ipv4Addr` → per-MAC last seen),
  documented addresses and a plan, derive each address's finding and offerability — no store, no clock;
  tests first.
- [ ] **T2** (AC1, AC2, AC7) Verdicts per decisions 1, 2, 10, 13, including nesting.
- [ ] **T3** (AC3, AC13) The offer; the caveat; `next_free_none` rendered and reworded; `offerable`'s
  doc; the occupancy line; the too-large branch.
- [ ] **T4** (AC8, AC9) The reads (14.3a's reader, the documented read) inside the budget, after the
  ceiling; the guard narrowed with its three-part mutations; module docs corrected; the end-to-end
  timing at ≥ 1 M rows.
- [ ] **T5** (AC1, AC2, AC5, AC6, AC12) The screen: findings list, marker (if it reaches 3:1),
  outside-every-subnet list, keyed words, absolute dates, legend literals, stylesheet — §1(f)'s tests
  updated deliberately.
- [ ] **T6** (AC4) The warning fragment route with its perimeter test, budget and keyed copy; the range
  form's case.
- [ ] **T7** (AC10) The seed (and the summary for it), the `REQUIRE` flag, the contrast check, both
  gates' row-finding, the kbd-probe; both gates run.
- [ ] **T8** (AC14, AC15) Measure; docs; the live count.

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

### Debug Log References

### Completion Notes List

- Contexted 2026-09-15 from three research passes; validated the same day (fact-check + gap-hunt that
  built and measured); split and decided by Guy the same day.

### File List

### Change Log

- 2026-09-15 — **T0: Guy took all fourteen decisions and SPLIT the story** — 14.3a the sighting summary,
  14.3b this audit. File renamed from `14-3-the-audit.md`.
- 2026-09-15 — validated and rewritten whole: the offer serves `reserved`/`dhcp-pool` addresses today;
  reusing the observation reader fails NFR2 after ≈ 2 months; AC8 could not be carried by the guard;
  FR24's two MACs are a succession; a cell marker fails contrast where axe cannot see; §0 fused two
  lists of six.
- 2026-09-15 — contexted. The epic's AC1 and AC2 found to overlap, and the guard found to forbid the
  audit by design.
