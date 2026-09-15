# Story 14.3: The audit, and the address the product must not offer

Status: ready-for-dev

⚠️ **`ready-for-dev` is the workflow's status, not a statement that nothing is open.** §2 poses
decisions that are Guy's, and the mandatory validation (two fresh-context agents) has not run.

Epic 14 (IPAM). Baseline: `02d15f2` (master, after story 14.2b's second review), **924 tests**
(634 bin + 191 core + 99 xtask), ten gates. 🔑 **This is the story the epic exists for.**

## Story

As the operator,
I want the product to tell me which addresses are in use but not in my plan,
so that I never hand a busy address to a second machine.

🔑 **The reason is the deliverable, not the highlight** (`epics.md:2343`): the *next free address*
panel proposes an address because no RECORD claims it. Under this story it must **exclude every
observed address** — an exclusion, not a highlight, and *the only place the product PREVENTS the
duplicate rather than reporting it*. Every other criterion serves that one.

## §0 — What is settled, and must not be re-opened

**Guy's six arbitrations of 2026-09-10** are `epics.md`'s Epic 14 preamble (`:2345-2357`). The four
that bind this story:

- **(2) The range's POLICY decides the verdict.** Measured on the reference LAN: the router serves
  **24 DHCP leases** of the 32 addresses reverse DNS resolves — an audit that highlights every
  unclaimed address highlights 24 of them permanently and stops being read. `dhcp-pool` is where an
  unclaimed address is NORMAL.
- **(3) A sighting protects an address FOREVER, until the operator releases it.** Refused: the last
  sweep only, and a named window — *the product cannot tell gone from silent* (one sweep sees 46
  hosts where 49 exist). Accepted cost: **the grid fills and never empties by itself**; every held
  cell carries the date of its last sighting; RELEASE is story 14.4's fourth write route.
- **(4) IPAM and `declared_attribute` are TWO registers**, and the documenting gesture does not
  touch IPAM. The audit compares the PLAN with the NETWORK; it does not read `declared_attribute`
  to decide a verdict (but see §2 decision 4 for the offer).
- **(5) The vocabulary is CLOSED.** *The audit needs no state noun of its own*: *observed with no
  plan entry* is `undeclared`, *the plan says free while the network shows it occupied* is `gap`
  (`prd.md:1041-1045`). No story may extend the binding tables. FR24's two-MAC case says
  « Conflit d'adresse », never a bare `conflict` (the qualified form Guy chose on 2026-08-20,
  `deferred-work.md` — the glossary's `conflict` means source against source, `prd.md:1016`).
- **(6) Every state carries a pattern and a word, never a hue alone** (WCAG 1.4.1, NFR25).

**Arbitration 6 of the epic — *the form warns and still writes*** (`epics.md:2433`): refusing was
refused, because the most frequent LEGITIMATE case is entering into the plan the machine that is
already there.

**Stories 14.2 and 14.2b, still in force**: cells carry a POLICY and a STATE on two axes; the EDGE
(`.0`, `.255`) is decided before any range and is never offerable; past `MAX_DRAWN_ADDRESSES`
(1024) no grid is built and the ranges are listed instead, the ceiling tested BEFORE anything is
materialised; a schema-legal bad row is skipped and named, never a 500 for the page; refusals are
keyed in both locales; source-scanning guards read through `src/source_scan.rs`, never a
hand-rolled stripper.

## §1 — What this story inherits, each WITH the measurement that produced it

⚠️ **Line numbers were derived on `02d15f2` by a research pass and are dated by that tree** — re-derive
before citing them in code or in the record (*a line number is not a property*).

### (a) 🔴 The plan's own guard FORBIDS what this story must do

`the_plan_reads_no_observation` (`ipam_page.rs` tests, ~`:1173-1293`) exists because story 14.2's AC2
said the plan reads no observation — *the audit is 14.3's*. Its perimeter is derived (a file whose
name starts with `ipam`, or whose code names `ip_subnet`/`ip_range`/`ip_address`), asserted to be
exactly `["ipam_page.rs", "ipam_repo.rs", "ipam_write.rs"]`; it forbids the literals
`observation_record`, `identity_link`, `declared_attribute`, and any whole-word `repo` except
`crate::repo::classify` / `crate::repo::is_deadlock` and `opencmdb_core::repo`. **Any audit reach —
`crate::repo::load_observation_facts`, a `FROM observation_record` in `ipam_repo.rs`, or a new
`ipam_audit.rs` — reds it by design.** Its own doc names its limit: a read through another adapter is
invisible to it. 🔴 **Putting the join in an unguarded module to keep it green is exactly *a guard
placed where the defect cannot occur*.** Changing it is a decision (§2, decision 6), proven red.

Text that states the old rule and becomes false: the module docs of `ipam_page.rs` (~`:6-8`),
`ipam_write.rs` (~`:22-25`) and `opencmdb-core/src/ipam/mod.rs` (~`:20-24`, *"Nothing here refers to
an entity, an observation or an interface"*); and `mod.rs` ~`:43-44` (*"Nothing in this codebase
reads a policy yet"*), already stale since `offerable` reads one.

### (b) 🔴 No function answers the question, and the one reader there is reads EVERYTHING

Nothing returns the set of observed IPv4 addresses with their hardware addresses and last sighting.

- `repo::load_observation_facts(executor) -> Result<Vec<ObservedBatch>, sqlx::Error>` (~`repo.rs:628`)
  is `SELECT id, connector_id, …observed_at…, facts FROM observation_record ORDER BY observed_at, id`
  — **no `WHERE`, no `LIMIT`**, registered as *"every page render loads every observation ever
  recorded"* (`deferred-work.md` ~`:5384-5401`, issue #150), owner *"the first story after the freeze
  is lifted"*. ⚠️ **Constraint 3 needs every past sighting**, so this story meets that row head-on
  (§2, decision 5).
- The closest existing shape is `inventory_view.rs` ~`:174-190`: a *freshest sighting per address*
  map, `BTreeMap<String, &ObservedBatch>` keyed by `addr.to_string()`. The triage `Nouveau` loop
  (`page.rs` ~`:1141-1245`) does the same fold with the latest instant winning.
- Errors come back as `sqlx::Error`, mapped by `page::server_error`.

### (c) The IPv4 ↔ MAC pairing exists ONLY inside one observation

`Fact::IpV4 { addr }` and `Fact::Mac { addr, locally_administered }` (`opencmdb-core/src/observation/mod.rs`
~`:293-320`) sit side by side in one observation's `facts` (`LONGTEXT` JSON). The ARP/ping connector
emits one observation per host: `IpV4` + `Rtt` + optional `Hostname` + optional `Mac`
(`arp_ping.rs::emitted_facts`, ~`:441-468`). `interface` has a MAC and `last_seen_at` but **no IP
column**. So *"two hardware addresses seen on one IPv4"* (FR24) is a fold over observations in Rust:
group by `Ipv4Addr`, collect the distinct `Mac`s, keep the latest `observed_at`.

### (d) Two formats, and no text conversion is needed

The plan stores the padded canonical form (`192.000.002.009`, `ipam_repo::canonical` /
`from_canonical`); observations store plain dotted text inside JSON. **Both decode to
`std::net::Ipv4Addr`** — the plan through `from_canonical`, observations through serde — so the
comparison is on `Ipv4Addr`, in Rust (`architecture.md:239`: *"All value comparison/normalization
happens in Rust, never in SQL"*). A SQL join is not possible anyway: `facts` is JSON. Story 14.1
named this seam as 14.3's (`14-1-plan-schema-and-no-producer.md` ~`:98-100`); any SQL containment
(`BETWEEN first_addr AND last_addr`) pays F57 (`architecture.md:4847`).

### (e) 🔴 A warning in the write's answer is thrown away

`ipam_write.rs::answer` (~`:601-633`) answers a success with **201 and `HX-Redirect:
/ipam?subnet=…`**; htmx follows the redirect and never shows the body. So *"the form warns"* cannot be
a sentence in the 201 body (read from the code, not yet measured in a browser — §2, decision 7).
`IpamWriteState` holds a port and never a pool (story 14.2b's AC2): anything the write needs to know
about the network goes through the port.

### (f) The cell model, and the tests that pin it

`CellState` (`ipam_page.rs` ~`:57`): `Defined(Option<IpPolicy>)`, `Free(IpPolicy)`,
`Infrastructure(Option<IpPolicy>)`, `NotCovered`. `PlanView::derive(subnet, &bounds, &defined)`
(~`:185-215`) resolves defined → edge → covering range (the lowest `first_addr` wins among
overlaps — *a rendering decision, not a repair*) → not-covered. `offerable()` (~`:141`) is
`Free(policy)` with `policy != Infrastructure`, and its doc already says **14.3 must additionally
exclude every OBSERVED address**. `next_offerable()` (~`:218`) takes the first offerable cell;
`counts()` a four-tuple. `strings()` builds the `next_free_caveat` sentence, today *"From the plan
alone; the network has not been consulted"* (`app.yml` ~`:840`) — **false after this story**.

Tests that pin today's model and must be updated deliberately, not loosened: the offerable count
`254` and `next_offerable == 192.0.2.1`; the four-state lists in
`every_state_carries_its_own_key_and_its_own_modifier` and
`the_stylesheet_defines_every_modifier_the_grid_can_emit` (hand-written — a new state is invisible to
them until added); `the_blank_cell_is_the_one_modifier_no_legend_entry_carries` (every new modifier
must be a literal in the legend); `the_four_states_carry_four_distinct_fills`; the rendered
`aria-label` count `258`; every three-argument `PlanView::derive` call; `main.rs`'s selector test.

### (g) The visual channel is EXHAUSTED on a 14 px cell

Story 14.2 measured it (`app.css` ~`:937-940` and its comment): three border styles render at 1 px,
`static` and `infrastructure` are identical pixels, and *the fill is already spoken for by the four
STATES*. Guy's stated limit (2026-09-12): *"the grid separates the STATES, the policy is carried by
each cell's accessible name and by the legend"*. This story adds `undeclared`, `gap`, a held cell's
sighting and « Conflit d'adresse » **to a channel with no room left** (§2, decision 8). The axe gate
compares `free` against `not-covered` in the computed style — *axe itself reports 0 violations under
the mutation that breaks it*, so a browser check is the sole carrier of "these look different".

### (h) The triage row the form must link to

The triage `Nouveau` row id is `nouveau:<ip>`, its link `/triage?sel=nouveau:<ip>` (`page.rs`
~`:1197`, `row_href`/`url_escape` ~`:1264`, `:1333`). ⚠️ That row exists only for an observed address
the DECLARED side does not claim — a different register from the plan (arbitration 4). An address
undeclared in the PLAN but documented in `declared_attribute` has **no** `Nouveau` row to link to.

### (i) The seed cannot reach a single audit state today

`a11y/seed.sql` (~`:54-82`, `:146-160`): four observations at `NOW(6)`, **no `Mac` facts** —
`192.0.2.11`/`.12`/`.13` (free `static` cells) and `192.0.2.99` (inside the `dhcp-pool`, so NOT
highlighted). Office `192.0.2.0/25`: `static` .1–.40, `dhcp-pool` .80–.126, defined `.9`; .41–.79 not
covered. Workshop `198.51.100.128/25`: `reserved` .129–.200. Missing: a sighting outside every range,
one in `reserved`, one on a defined address, a two-MAC sighting on one IPv4. **`AXE_REQUIRE_PLAN=1`
refuses a run that draws no cell; nothing refuses a run that draws no AUDIT state.**

### (j) File sizes (code lines before the first line-start `#[cfg(test)]`)

`ipam_page.rs` **729**, `ipam_repo.rs` **752**, `ipam_write.rs` **854**, `page.rs` **1954 — 46 lines
left, it must not grow**. `repo.rs`'s gate count is misleading (it stops at a test helper at `:184`);
new observation readers do not belong there anyway.

### (k) What holds by STRUCTURE, and is owed a check rather than a decision

NFR8(a) — *a fault may only REMOVE knowledge, never ADD an assertion* (`prd.md:1319-1324`) — and
FR19's blind-source suppression (`prd.md:902`): the audit derives from RECORDED sightings only, so a
blind or failing scanner cannot add one. That is true by construction and it is **owed a test**, not
assumed: an empty observation set yields no finding, and a subnet's audit is a function of the
recorded rows alone (no clock read in the derivation — story 6b.4's clock guard precedent).

## §2 — Decisions this story must take, and which are Guy's

Each is posed with the options and a recommendation. **None is settled here.** A measurement at
validation that refutes a recommendation returns it to Guy.

1. **`undeclared` or `gap` for a free cell with a sighting?** AC1 and AC2 as written in `epics.md`
   OVERLAP: an observed address with no entry in a `static` range is *"observed with no plan entry"*
   (`undeclared`) AND *"the plan declares free, the network shows occupied"* (`gap`); and under the
   current schema the plan has no other way to "declare free" than a range. The two corrections the
   criteria name differ — *add an entry* vs. *correct a wrong record*. Options:
   - **(a) by coverage** — outside every range: `undeclared` (no plan entry at all → add one); inside
     a `static` or `reserved` range with no address row: `gap` (the plan claims that stretch and says
     it is free → correct the plan or the machine). ⚠️ Contradicts AC1's letter for `static`/`reserved`.
   - **(b) by policy** — `static` and outside every range: `undeclared`; `reserved`: `gap` (the plan
     said nothing should answer here). ⚠️ Contradicts AC1's letter for `reserved`.
   - **(c) `undeclared` everywhere AC1 says**, and `gap` reserved for a case the schema cannot yet
     express, stated and registered — AC2 then ships unreachable.
   - *Recommendation: (a)* — it gives each state a distinct correction, and the letter of AC1 is the
     cheaper thing to register than an unreachable AC2.
2. **Which policy decides when ranges overlap, or subnets nest?** (register row, owner 14.3). Options:
   **(a) the most protective** — any non-`dhcp-pool` range covering the address makes it a finding
   (the epic's purpose is preventing duplicates); (b) the most specific (smallest range / longest
   prefix); (c) report the contradiction itself. *Recommendation: (a) for the verdict, keeping 14.2's
   lowest-`first_addr` rule for what the cell LOOKS like, both stated.*
3. **The plan-against-plan warning** (register row, owner 14.3; story 14.2b decision 5): an address
   defined inside a `dhcp-pool` range is accepted silently today. And PRD FR24 counts *"a
   static-declared IP inside a DHCP range"* as an IP conflict, while 14.2b called a static reservation
   inside a pool *"ordinary and correct"*. Options: (a) a WARNING on `/ipam` for that address, not a
   conflict, the FR24 half registered as a divergence; (b) an FR24 « Conflit d'adresse »; (c) nothing,
   registered. *Recommendation: (a).*
4. **What does the offer exclude?** Certainly every observed address (the epic's criterion), every
   defined address and every edge whatever the plan declares (register row, owner 14.3). Open: an
   address DOCUMENTED in `declared_attribute.ipv4` but never observed. Arbitration 4 keeps the
   registers apart for VERDICTS; the offer's purpose is preventing a duplicate. Options: (a) exclude it
   too — a read of `declared_attribute` from the offer, stated as the one crossing; (b) plan and
   network only, registered. *Recommendation: (a).*
5. **How the sightings are read.** Constraint 3 needs every past sighting; the only reader loads every
   row, and that is registered as a cost. Options: (a) reuse `load_observation_facts` and fold in
   Rust, inside the page budget, with the cost MEASURED at a stated scale and the register row
   updated rather than silently worsened; (b) a new reader returning only observations carrying an
   `IpV4` fact (still JSON, still a full scan); (c) wait for the retention decision. *Recommendation:
   (a), with the measurement.*
6. **The guard `the_plan_reads_no_observation`.** Options: **(a) narrow it** — a new
   `ipam_audit.rs` (or a named function) is the ONE module allowed to reach the observation reader
   (and `declared_attribute` if decision 4 is (a)), the guard renamed to say so, and `identity_link`
   still forbidden; (b) retire it. *Recommendation: (a)* — the property worth keeping is *the plan's
   drawing and writing code does not read the network*, and the audit is exactly one place.
7. **Where the form's warning appears**, given (e). Options: (a) **after the write**, `/ipam` rendered
   with the warning selected by the URL the redirect carries (story 6.4's arbitration 2′ precedent:
   the confirmation rides in the URL, *one URL per state*); (b) **before the write**, an `hx-get` on
   the address field to a new read route. *Recommendation: (a)* — one more read route is a fourth
   surface the budget, the auth perimeter and both browser gates must cover.
8. **How the audit is SHOWN on a channel with no room.** Options: (a) **a findings LIST under the grid**
   — address, state word, last sighting as an absolute date, hardware addresses, link — as the primary
   carrier, and ONE non-colour cell marker ("this address was seen") with the state in the accessible
   name; (b) new cell treatments per state, measured in a browser. *Recommendation: (a)*, measured.
9. **The date of a last sighting** against the UX bans (*no age brandished as reproach*, *render at
   t+1 day and t+6 months → identical output*, `ux-design-specification.md` ~`:1514-1522`):
   *recommendation: an ABSOLUTE date, never a relative age* — confirmed or refused.
10. **A subnet too large to draw** has no grid and no offer today. *Recommendation: it stays without
    an offer, said on screen*, rather than computing one over up to 2^32 addresses.

## Acceptance Criteria

⚠️ **The shapes below depend on §2.** Where a decision changes an AC, T0 rewrites the AC in the same
act and records the option refused.

**AC1 — `undeclared` is highlighted where the policy makes it an anomaly, and never in a
`dhcp-pool`.** An observed address with no plan entry (per decision 1) is shown as `undeclared`, with
its word in both locales from the EXISTING key (`state.undeclared`, constraint 5); inside a
`dhcp-pool` it is not a finding. A test covers each policy and the uncovered case, and the verdict
rule for overlapping ranges (decision 2) is tested, not implied.

**AC2 — `gap` is distinct from `undeclared`**, by its word (the existing `triage.kind.ecart` key) and
by its treatment, because the corrections differ. A test asserts the two are never rendered alike —
in the HTML (distinct modifier/word) AND in a browser (distinct computed style or distinct marker),
since axe cannot see identical treatments.

**AC3 — THE OFFER EXCLUDES EVERY OBSERVED ADDRESS.** `next_offerable` never returns an address with a
recorded sighting, a defined address, or an edge, whatever the plan declares over them (register row
2); per decision 4, nor a documented one. The `next_free_caveat` sentence stops saying the network was
not consulted and says what WAS consulted. A subnet whose every host is excluded renders
`ipam.next_free_none` — ⚠️ registered as UNREACHABLE until now (`deferred-work.md`), and reachable
here, with a render test. 🔑 **This AC is proven red by the mutation that re-admits one observed
address, and that mutation's carrier is named.**

**AC4 — the form warns, names what it knows, links, and STILL WRITES.** Entering a highlighted address
writes the row (201) and the operator then reads, per decision 7, a warning naming the last sighting
(absolute date) and the hardware address(es) known, with a link to `/triage?sel=nouveau:<ip>` WHEN
that row exists (§1(h)) and no link otherwise — said, not broken. Keyed in both locales; never a Rust
`format!` with a raw value in English. A browser check: the warning is visible after the write, and
focus lands where story 6.4's contract says.

**AC5 — FR24: two hardware addresses on one IPv4 outside a `dhcp-pool` is « Conflit d'adresse »,
with BOTH addresses.** Inside a `dhcp-pool` it is not a conflict. Two sightings of the SAME MAC are
not a conflict; a sighting with no MAC contributes no second address. Tested on each case.

**AC6 — a held cell carries the date of its last sighting**, as an absolute date (per decision 9),
in its accessible name and in the findings list. A test renders the same data at two clock instants
and asserts identical output (the UX ban's own test), and the derivation reads no clock.

**AC7 — the plan-against-plan warning** (decision 3): an address defined inside a `dhcp-pool` is
shown with its warning on `/ipam`, keyed, and the write still succeeds. Register row 3 is closed or
re-owned in the same act.

**AC8 — the guard is reshaped, not bypassed** (decision 6): the observation reader is reachable from
exactly the place decided, the guard's name and doc say so, and mutations prove it STILL reds on
`identity_link`, on a `declared_attribute` read outside the decided place, and on an observation read
from `ipam_page.rs`'s drawing code or `ipam_write.rs`. Every module doc that states the old rule is
corrected (§1(a)).

**AC9 — the budget covers the audit, and the ceiling still comes first.** The sighting read is inside
`store_within`; the size ceiling is tested BEFORE any cell or finding is materialised; the audit fold
is measured at a stated scale (e.g. 10 000 and 100 000 observation rows) against NFR2's p95 1.5 s,
the figures recorded with their command, and the register row on the unbounded read updated with
them (decision 5).

**AC10 — the browser gates REACH the audit.** `a11y/seed.sql` gains sightings for each state (outside
every range, `reserved`, a defined address, two MACs on one IPv4, inside the `dhcp-pool`) with `Mac`
facts; the axe gate REFUSES (exit 2) a seeded run that draws no audit state (a `REQUIRE` flag on
story 6b.11's precedent, set in `ci.yml`); every new state has a pattern and a word (constraint 6);
the kbd-probe covers the findings list and the warning's link.

**AC11 — structure holds and is tested** (§1(k)): an empty observation set yields no finding; the
audit is a function of the recorded rows, with no clock read.

**AC12 — THE LIVE COUNT lives here**, with the command and both store conditions named (a virgin
store after one warm run; `env -u DATABASE_URL`, never `DATABASE_URL=`). Baseline **924** (634 + 191 +
99) at `02d15f2`.

**AC13 — no regression**: ten gates, `clippy --workspace --all-targets -D warnings`, `RUSTFLAGS="-D
warnings"`, fmt, `cargo deny`, **both browser gates claimed and run**; no file over 2000 code lines
(`page.rs` does not grow); docs current before push — the user manual's IPAM chapter (its `planned`
block names the audit and the offer as not implemented), both twins, and the register.

## §3 — What the validation measured that this story must carry

_(Empty until the mandatory validation runs.)_

## Tasks / Subtasks

- [ ] **T0** Take §2's decisions with Guy; rewrite the affected ACs in the same act, each with the
  option refused.
- [ ] **T1** (AC11, AC5, AC6) The audit's pure core: from recorded sightings (`Ipv4Addr` → distinct
  MACs, latest `observed_at`) and a subnet's plan, derive each address's finding — no store, no
  clock. Where it lives is decided by decision 6 (a pure module the guard can name); tests first.
- [ ] **T2** (AC1, AC2, AC7) The verdicts: `undeclared` / `gap` per decisions 1–3, the overlap rule,
  the plan-against-plan warning.
- [ ] **T3** (AC3) The offer: `offerable` + sightings (+ documented per decision 4); the caveat
  sentence; `next_free_none` reachable and rendered.
- [ ] **T4** (AC8, AC9) The read and the guard: the sighting read inside the budget, after the
  ceiling; the guard reshaped with its mutations; module docs corrected.
- [ ] **T5** (AC1, AC2, AC5, AC6) The screen: the findings list and the cell marker per decision 8,
  keyed words, absolute dates, legend literals, stylesheet rules — and the tests of §1(f) updated
  deliberately.
- [ ] **T6** (AC4) The warning after the write per decision 7, and its link.
- [ ] **T7** (AC10) The seed, the `REQUIRE` flag in both gate and `ci.yml`, the kbd-probe block; both
  gates run in a browser.
- [ ] **T8** (AC9) The scale measurement, recorded, and the register row updated.
- [ ] **T9** (AC12, AC13) Measure: both store conditions, `RUSTFLAGS`, gates, deny, both browser
  gates; docs; the live count.

## Dev Notes

### Traps this project has paid for, and which apply here

- 🔴 **A guard placed where the defect cannot occur reads as coverage and is none** — the audit join in
  an unguarded module to keep `the_plan_reads_no_observation` green is exactly that.
- 🔴 **A budget that bounds the wrong half of a handler is none** (14.2's denial of service): the fold
  is synchronous; the ceiling must still hold before materialising.
- 🔴 **A test that pins the ugly thing demands it** (`offerable == 256`, 14.2): update §1(f)'s tests to
  the new truth, never loosen them to pass.
- 🔴 **A guard that greps a file greps its prose** — use `src/source_scan.rs` (string-aware, both
  comment forms, raw strings, char literals); it exists because two hand-rolled strippers were
  defeated by ordinary code.
- 🔴 **A hand-written state list cannot see a new state** — every list of four in §1(f) must grow, and
  a distinctness assertion must cover the new ones in a browser.
- ⚠️ **`app.yml` is invisible to incremental builds**: a mutation editing only the locale measures
  nothing.
- ⚠️ **`ascii_bin` is PAD SPACE**; the carrier is `LENGTH(x) = LENGTH(TRIM(x))`.
- ⚠️ **Store-backed tests** take `DB_TEST_LOCK` and own their CIDR; a virgin store races on
  `migrate!` — one warm run first; drop and recreate before any count or mutation pass.
- ⚠️ **`cargo xtask mutate` reports counts, never test names** — name each carrier by reading the red
  test's own panic message; say assertion / `.expect()` / `panic!` / compiler.
- ⚠️ **`cargo fmt` can re-wrap an anchor**; the driver refuses an anchor that matches twice, a comment
  quoting the line included.
- ⚠️ **"Registered" means a row exists in `deferred-work.md`** — 14.2b was caught twice on this.
- ⚠️ **A cause needs a check** — 14.2b's first repair presented a held test connection's behaviour as
  a production defect; its second review withdrew it.

### What the store must answer, and what it must not

The audit reads the PLAN (`list_subnets`, `ranges_in`, `addresses_in`) and the RECORDED sightings; it
never reads `identity_link` (grouping is Epic 6's), and reads `declared_attribute` only if decision 4
is (a), from the one place decision 6 names. No new migration is expected: the audit is derived, not
stored.

### Project Structure Notes

- The pure audit belongs where the guard can name it (decision 6) — a new `crates/opencmdb-bin/src/`
  module or `opencmdb-core/src/ipam/` for the verdict algebra (D47: its errors are `ipam::IpamError`,
  no sqlx in core). ⚠️ `float-free` does not walk `ipam/`; *14.3 is where a ratio could first appear*
  (14.1) — occupancy stays counts.
- `page.rs` must not grow (46 lines left). `ipam_page.rs` (729), `ipam_repo.rs` (752) and
  `ipam_write.rs` (854) have room; split before the ceiling, not after.
- No new dependency.

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 14 preamble `:2337-2357`, Story 14.3 `:2411-2441`.
- `_bmad-output/planning-artifacts/prd.md` — FR21–FR24 `:906-910`, FR19 `:902`, NFR2 `:1235`, NFR8(a)
  `:1319-1324`, NFR25 `:1467`, STATE axis `:1004-1023`, PLAN axis `:1025-1060`.
- `_bmad-output/planning-artifacts/ux-design-specification.md` — occupancy grid `:529-531`,
  `:1227-1231`, accessibility `:1632-1636`, hard bans `:1509-1522`.
- `_bmad-output/planning-artifacts/architecture.md` — comparisons in Rust `:239`, D47 `:2639`,
  `ipam/` `:3208`, `:3366`, F57 `:4847`.
- `_bmad-output/implementation-artifacts/deferred-work.md` — rows owned by 14.3 (nested subnets, edge
  addresses, an address inside a range), the unbounded observation read, `next_free_none`
  unreachable, `role="grid"` divergence, « Conflit d'adresse ».
- Stories `14-1-plan-schema-and-no-producer.md`, `14-2-the-plan-on-screen.md`,
  `14-2b-the-plan-in-the-operators-hands.md`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Contexted 2026-09-15 from three research passes (code, planning documents, prior stories). Ten
  decisions posed in §2; none settled.

### File List

### Change Log

- 2026-09-15 — contexted. The epic's ACs 1 and 2 were found to overlap on a free cell with a
  sighting, and the guard `the_plan_reads_no_observation` to forbid the audit by design; both posed
  as decisions rather than settled.
