# Story 6.14c: A machine the operator called one is shown as one

Status: **ready-for-dev** — contexted, arbitrated (§0.8), validated by two layers (§0.9, one of them building a
prototype on its own database), and re-arbitrated on the validation's findings (§0.10, 2026-09-26). Not
`ready-for-dev`: its criteria are written UNDER the recommendations and move with the arbitration.

**Epic 6.** Inserted by Guy's act of 2026-09-25 at story 6.14b's arbitration (A1). **Base:** `master` at
`bc4ee23` (6.14b done). **Live count at base:** 825 + 219 + 110, measured at 6.14b's merge; re-measure
before the Record.

## Story

As the operator,
I want the machine I said was one to appear once in the inventory,
So that the product counts boxes, not network cards.

## The epic's text (`epics.md`, story 6.14c), an insertion note with no criteria

> 6.14b records the operator's answer and mints nothing. **This story's producer is 6.14b's OPERATOR
> `match`** — the first real producer the device mint has had, which 6.12 found missing. Its scope: the
> device and its N:N membership (D14, *"`interface.device_id` is NOT a unique FK"*); the inventory showing
> a declared entity through the device its interface belongs to, **without ever updating
> `declared_attribute.entity_id`** (D15) — a path address → interface already exists
> (`ambiguity_view::ambiguity_rows`); and the double documentation 6.14b registers. Its first obligation is
> a planning act on what the inventory row of a grouped machine SHOWS. ⚠️ **It inherits a question, not an
> answer**: a chain A–B, B–C whose A–C carries an ENGINE `no_match` joins A and C by union-find.

It also inherits two register rows by name (`deferred-work.md`): *after the same machine, both addresses
offer Add again and can be recorded twice*, and *an answer can still wait on — and deadlock with — the
pair that is its NEIGHBOUR in the unique key* (owned here because a device writer would be a third writer
on that table).

## 0. What contexting measured, and what it will not settle alone

### §0.1 — 🔴 The inventory lists RECORDS, and nothing ties a record to an interface today

- `/devices`' real section is `inventory_view::build_inventory`: one row per **declared entity id**
  (`declared_attribute.entity_id`), keyed on its `ipv4` (`inventory_view.rs:196`). `obelix` is two rows
  because it was documented twice, once per address.
- A declared entity id is **not** an `entity` row: `declared_attribute` has **zero foreign keys** (story
  14.1's measurement, re-read), and `entity`/`device` (`0006`) have **no producer**. There is no table,
  column or function linking a record to an interface.
- **Two paths exist, both computable at read time without writing anything:**
  1. **By hardware address.** Since PR #163 the documenting gesture writes a declared `mac`
     (`gap::project`, `gap/mod.rs:110`), rendered by `MacAddr`'s `Display` — lowercase, colon-separated —
     which is **exactly** `interface.mac_canon`'s spelling (`repo.rs:769`: *"There is no second"*). A
     record documented since 2026-09-10 names its interface directly.
  2. **By address.** A record's `ipv4` → the observations carrying it → their current `match` placement
     in `identity_link` → the interface. This is what `ambiguity_rows` already does for the triage link
     (6.14b §0.9(K)). It is the only path for a record documented before PR #163 — possibly `obelix`'s.

### §0.2 — 🔴 A `device` table would be a CACHE of the operator's answers, and D14 names that

The grouping 6.14c must show is **already recorded**: 6.14b writes one current OPERATOR `match` row per
question-pair, which the engine never supersedes (6.12's skip) and which survives an ENGINE purge (D14:
*"those are INPUTS, not derivations"*). Union-find over those rows IS the set of machines the operator
called one.

A `device` row plus a membership table minted from them would be a **second representation of the same
fact**, derived from the first — and D14 argues exactly this against `link_current`: *"a derived
materialised table IS a cache: what is its invalidation key?"* Concretely, a device table would have to be
kept in step with every future change of an answer (registered: *changing an answer*, owner Epic 6's
retrospective), every purge, and every sweep that vacates an interface — the drift class this project has
paid for repeatedly (*two representations with nothing tying them*).

⚠️ **What the epic text assumed, and why it may not hold**: *"the first real producer the device mint has
had"* is true — but a producer of the grouping FACT is not a reason to materialise it twice. The day a
SECOND producer exists (an L2 `Decisive` rule, whose `match` would be the engine's), there are two inputs
to one grouping, and a device table earns its keep as the place they meet. Until then it has one input.

⚠️ **What is NOT in question**: D12's *"the `device` level is non-negotiable"* is about what the operator
SEES and acts on — *a box* — and a grouped inventory row is that box. The question is only whether its
identity is stored or computed.

### §0.3 — Measured facts the dev agent inherits

- `interface.mac_canon` has **no** uniqueness across `l2_domain`: one MAC in two domains is two interfaces
  (`0002`'s `interface_l1_key`). A declared `mac` carries no domain, so path 1 can name two interfaces —
  in which case both belong to the record.
- `load_current_operator_answers` (6.14b) already reads every current OPERATOR row with both interfaces'
  MACs; `ambiguity_view::groups` is the union-find. Neither needs a change to be REUSED.
- `/devices` is `Nature::Mixed`; its real half is budgeted (`PAGE_STORE_BUDGET`) and reads declared
  attributes, provenance and every observation (issue #150: unbounded). Path 2 needs `identity_link`
  placements for the records' addresses — a bounded read by address, not a new scan.
- The inventory's header count is **records** (`inventory.n_entities`, *« fiches »*). Grouped rows are
  **machines**. Two counts of one table in two units is story 5.14b's defect.
- The triage queue's *Pending* count (registered, owner Epic 6's retrospective) is NOT this story's.

### §0.4 — What this story must not do

- Update `declared_attribute.entity_id` (D15) — the rows stay; only how they are GROUPED on screen changes.
- Touch the triage queue, the answer route, or the L2 pass.
- Decide how an answer is changed (registered to the retrospective).

### §0.5 — The decisions that are Guy's, each with a recommendation

**(A) Where the grouping lives.**
(A1) **Computed at read time** from the current OPERATOR `match` rows (union-find), no new table, no writer.
(A2) **Minted**: `entity` + `device` rows and an SCD2 N:N `device_membership` table (`0014`), written in
the answer's transaction, as the epic text reads.
**Recommendation: (A1)**, for §0.2's reason — the operator's rows ARE the record, and a device table with
one input is a cache of it. Its costs, stated: `entity`/`device` stay without a producer; story 6.19
(*history per device*) will need a stable device id, and a computed group's id (its smallest interface)
changes when a member is added — **registered to 6.19**, which is where two producers or a stable id are
first needed. ⚠️ Under (A1) the inherited *key-neighbour deadlock* row loses its premise (no third writer)
and returns to its owner as a residual of 6.14b's writer alone.

**(B) How a record reaches an interface.**
(B1) The declared `mac` first (→ `interface.mac_canon`), else the declared `ipv4` through its current
placement. (B2) The address only.
**Recommendation: (B1).** The MAC is L1 identity (D12) and survives a DHCP address change; the address is
the fallback for records documented before PR #163. A record reaching no interface stands alone.

**(C) What a grouped row SHOWS — the epic's first obligation.**
(C1) **One row per machine**: its names (distinct, joined), its addresses (joined), *N records* beside the
field count, the freshest provenance, documentation and sighting — the row says it groups several records.
(C2) One row per machine with its records as sub-rows beneath it.
(C3) The records keep their rows, visually bracketed as *one machine*.
**Recommendation: (C1)**: it is what *"counts boxes, not network cards"* says; (C2) doubles the table's
height for the information one cell carries; (C3) keeps the over-count the story exists to remove.

**(D) The header count.**
(D1) *« 3 machines · 4 fiches »* — both, named. (D2) Machines only. (D3) Records only (unchanged).
**Recommendation: (D1)**: the table's rows are machines, and the records are what the operator wrote —
hiding either leaves one number unexplained; naming both avoids 5.14b's two-unit trap by construction.

**(E) The chain across an ENGINE `no_match`** (inherited, unreachable with today's rules).
(E1) Group by the operator's `match` rows as they stand — the operator's answer is the input, the engine's
`no_match` on a pair never asked is not an answer. (E2) Refuse to join across it (split the group).
(E3) Join, and the row says the engine judged two of its interfaces distinct.
**Recommendation: (E1)**, with a test pinning it and the register row kept for the rule that makes it
reachable (story 6.8's non-transitive rules): (E2) cannot be expressed by union-find without deciding
which side a member goes to, which is the partition question registered at 6.14b; (E3) writes a sentence
for a state no current rule produces.

**(F) Two records reaching the SAME interface** (the double documentation 6.14b registered).
(F1) One row — same interface, same machine. (F2) Two rows.
**Recommendation: (F1)**: it closes the inherited row *by display*, and D15 is untouched — both records
stay, grouped.

### §0.8 — ✅ GUY'S ARBITRATION, 2026-09-26: all six on the recommendation

**A1 B1 C1 D1 E1 F1.** The grouping is COMPUTED at read time from the operator's current `match` rows —
no device table, no migration, no writer (the device table is deferred to 6.19, where a stable id is first
needed). A record reaches its interface by its declared MAC, else its address's current placement. One
inventory row per machine, carrying *N records*; the header names machines AND records. The chain across
an ENGINE `no_match` groups by the operator's answers and is pinned by a test. Two records on one interface
are one row, which closes the double documentation by display.

⚠️ **This diverges from `epics.md`'s insertion note** (*"the device and its N:N membership"*); the planning
PR that carries this story annotates the note, as 6.14b's did.

### §0.9 — 🔴 What the validation found (fact-check + a BUILT prototype on its own database)

**The prototype works for `obelix`**: two records answered *the same machine* are ONE row, with or without
declared MACs; *distinct* gives two rows; an operator match whose interface has no record adds no row; ten
gates stay green (no write, no provenance read). What moved:

**(A) 🔴 §0.2 presented an INTERPRETATION as what D14 says, and Guy arbitrated A1 on it.** The architecture
wants a STORED device: 6.12's user story reads *"so that a device is a record rather than a computation
repeated at each page load"* (`epics.md:1991-1993`), and its re-scope gave the device write to *"whichever
story first produces an L2 `Decisive`"* (`:2005`); D15 case A (`architecture.md:1111-1113`) assumes a stored,
SCD2-closable membership (*"close the grouping, open a new one"*); D14 (`:1080-1083`) makes the LINK a stored
row precisely so that *"no derived state exists"*; `software.device_id` and AC-M-04 (`:4748-4758`) need a
device id that survives a split. §0.2's reading — the operator's `match` rows are the input, a membership
table would copy them — is defensible, but it is a reading. ⚠️ And *"registered to 6.19"* was unsupported:
6.19 is *"derived from the observations"* and never asks for a stable id; the first written need is FR26's
`software.device_id` (Epic 15). → **re-posed to Guy (§0.10 A).**

**(B) 🔴 The ADDRESS fallback merges two different machines with no answer — measured.** A laptop seen at
`.20` and documented without a MAC (a record from before PR #163), then a phone holding `.20` by DHCP and
documented: ONE row, *"laptop, phone"*. Current `match` links are never closed across sweeps, so an address
resolves to whoever holds it now — and the reference LAN hands out 24 leases. The product refuses a merge
without proof and the inventory would draw one from a reused address. **A third path exists that §0.1 missed**:
`declared_attribute.origin_obs_id` names the exact observation an adopted record came from, and its current
`match` link names the interface — exact, DHCP-proof, no heuristic. ⚠️ It is a PROVENANCE read, gated by the
`authorship` gate's `SANCTIONED_READS` (3 entries, FR13). → **re-posed (§0.10 B).**

**(C) 🔴 AC1 and AC3 contradicted each other** — two records sharing one interface are one row (F1) while
*"no answer → nothing is grouped"*; and a record whose declared MAC reaches two interfaces (two `l2_domain`s,
or the registered interface-mint race, one domain) would BRIDGE them with no answer. → **re-posed (§0.10 C).**

**(D) The word *machines* is in no binding glossary row**; the screen's noun is *Devices* / *Appareils*
(`app.yml:202`). A displayed *machines* beside *Appareils* is the noun-synonym problem story 6b.6 registered.
→ **re-posed (§0.10 D).**

**(E) Nothing in the suite would red on a wrong grouping — measured**: the prototype with a hard-coded
English header, and a mutation feeding DISTINCT answers into the union-find, left 833 + 219 + 110 and ten
gates green. The criteria must be carried by a store-backed ROUTE test on `GET /devices` with a DISTINCT
control, never by the pure builder alone (story 6b.4's `triage_html` shape).

**(F) Cost is where the budget is not**: the naive address path is records × observations × placements —
**50.7 s** at 300 MAC-less records and 18 000 observations, in the synchronous build `PAGE_STORE_BUDGET` does
not wrap (story 14.2's denial-of-service shape). The grouping read itself is linear (+32 ms at 18 000 links).
A scale criterion, timed on the route, is owed.

**(G) Smaller corrections.**
- *"seen"* is keyed on the record's address, so a MAC-grouped row showed the PHONE's sighting at `.20` for the
  laptop; freshness must come from the member interfaces.
- Copy that becomes false: `inventory.title` (*"Your records"*), `inventory.lede` (*"A row here is a record"*),
  `inventory.no_authoring`; the header needs plural keys (the prototype printed *"1 machines"*).
- C1 left undecided: which field count, which record supplies origin/documented, conflicting names, sort.
- `load_current_operator_answers` returns `no_match` rows too: the caller MUST filter `outcome = 'match'`.
- *"This is what `ambiguity_rows` already does"* was imprecise: it maps interface → address for OPEN
  questions only, and an answered group's interfaces fall out of its reader.
- `interface_l1_key` is a plain index, not unique even within one domain (`architecture.md:1525`).
- F's attribution was wrong: the inherited *"Add offered again"* row is `obelix`'s case (two interfaces),
  handled by A/C1, not F; and grouping on screen does not stop the second write — *"over-count removed from
  the inventory, the double write still possible"*, not CLOSED.
- The key-neighbour row's owner IS this story; *"returns to its owner"* named nobody.
- `/triage` still reconciles each record separately, so `obelix` keeps two gap rows under one inventory row:
  a unit difference between screens, to register.
- Line: the `ipv4` skip is at `inventory_view.rs:214-219`, not `:196`.

### §0.10 — ✅ GUY'S SECOND ARBITRATION, 2026-09-26: all four on the recommendation

**(A) Compute now, store later.** A1 stands, taken this time AGAINST the counter-arguments of §0.9(A): the
grouping is computed at read time from the operator's current `match` rows; the divergence from 6.12's
user story, D15 case A and `software.device_id` is REGISTERED, owner **the first consumer needing a stable
device id — Epic 15 (FR26)**; the day a device is stored, it is built from the `match` rows, which stay the
source.
**(B) MAC, else the record's ORIGIN observation.** The declared `mac` → `interface.mac_canon`; without one,
`declared_attribute.origin_obs_id` → that observation's current `match` link → its interface. The address
path is DROPPED (§0.9(B)). The provenance read is SANCTIONED by name in the `authorship` gate — read-only,
for display only, on story 6b.4's precedent for `origin`.
**(C) One NIC = one row; never a bridge.** Records reaching one interface are one row (L1 identity). Only the
operator's `match` rows join two interfaces; a record reaching several interfaces never joins them — it
attaches to the FIRST (by interface id), and the case is registered.
**(D) *Appareils*, the screen's own noun.** The header reads *« N appareils · M fiches »*; the section title
becomes *« Vos appareils »* / *"Your devices"*. No new glossary term.

Row content (proposed with D, not contested): distinct names joined in record order; addresses joined;
*last seen* from the member INTERFACES (never the record's address — §0.9(G)); origin and *documented* from
the most recently written record; the field-count cell reads *« N fiches »* when a row holds several.
Sorted freshest first, as today.

## 1. Acceptance criteria — under Guy's arbitrations (§0.8, §0.10) and the validation (§0.9)

1. **One row per device.** **Given** records whose interfaces are joined by current OPERATOR `match` rows,
   or that reach one interface, **then** `GET /devices` shows them as ONE row: their names and addresses
   joined, *« N fiches »*, the freshest interface sighting, the most recent record's origin and date. A record
   reaching no interface stands alone. **The test is store-backed and goes through the ROUTE**: `obelix`
   documented twice through `POST /document-all`'s adapter and answered *the same machine* through
   `l2_answer::record_operator_answer` is one row — with a *distinct* CONTROL giving two rows.
2. **A record reaches its interface exactly.** The declared `mac` matches `interface.mac_canon` (compared in
   Rust, D10: `attr_value` is `utf8mb4_bin`, `mac_canon` `ascii_bin`); without a `mac`, `origin_obs_id`'s current
   `match` link. Tests: each path; a record whose MAC and origin point at different interfaces (the MAC wins);
   and **the DHCP case** — a MAC-less record and another machine later holding its address stay TWO rows
   (§0.9(B), measured merging before).
3. **Only answers join interfaces.** **Given** no answer, or a *distinct* answer, **then** no two INTERFACES are
   grouped; records sharing one interface still are. A record whose MAC reaches two interfaces does not join
   them (a test, on the fixture connector's two domains). The union-find is fed `outcome = 'match'` rows ONLY —
   a test feeds a *distinct* answer and asserts two rows (the validation's mutation was green without it).
4. **The header names both units**: *« N appareils · M fiches »* in both locales, with plural keys; a route test
   asserts both numbers on a grouped store.
5. **No write.** No migration, no table, no `UPDATE`; `declared_attribute` untouched (`entity-id-immutable`
   green). The ONE new sanction is the `authorship` gate's read of `origin_obs_id`, named by path and
   function, display-only, and the gate reds without it (prove-to-red).
6. **Cost.** The build uses indexed maps, never records × observations; a timed test at a few hundred records
   and tens of thousands of observations stays well inside `PAGE_STORE_BUDGET`, measured on the route (the
   naive shape measured **50.7 s**).
7. **The chain (E1)** — grouped by the operator's answers, pinned by a test on hand-built rows, stated
   unreachable with today's rules.
8. **The copy says what the screen now is**: `inventory.title`, `inventory.lede` and `inventory.no_authoring`
   rewritten (rows are devices, grouping several records); the user manual's `/devices` section and the
   CHANGELOG's *Unreleased* section say what the inventory groups, by what, and that it groups only what the
   operator answered.
9. **Registers**: the stored device deferred (owner Epic 15, FR26), with the divergence from 6.12 and D15; the
   inherited *"Add offered again"* row NARROWED — the over-count is gone from the inventory, the second write is
   still possible; the key-neighbour row re-owned to **the story that first produces an L2 `Decisive`** (no
   third writer here); a record whose MAC reaches several interfaces; `/triage` still reconciling each record
   separately (two gap rows under one inventory row).

## 1b. Tasks

- [ ] T1 `inventory_view.rs`: record → interface (MAC, else origin observation), union-find over records'
      interfaces and `match` rows, one row per device; indexed maps (AC1–AC3, AC6).
- [ ] T2 The readers: interfaces by MAC, the origin observations' current placements, the `match` rows
      (filtered); `/devices` loads them within its budget (AC1, AC6).
- [ ] T3 The `authorship` sanction for the `origin_obs_id` read, proved red without it (AC5).
- [ ] T4 Copy and keys, both locales; the header's two plurals (AC4, AC8).
- [ ] T5 Route-level tests with a store, the DHCP case, the *distinct* control, the timed scale test (AC1–AC7).
- [ ] T6 Mutation pass: predictions first, carriers derived from the mutated VALUE's readers (the last two
      stories each missed one), `--baseline`, one database per run.
- [ ] T7 Docs, register rows, twins, the Record (AC8, AC9).

## 2. What this story must NOT do

- Store a device, add a table or a migration (A1).
- Update `declared_attribute.entity_id` (D15), or write anything.
- Group two interfaces on anything but an operator's `match` (C).
- Touch the triage queue, the answer route or the L2 pass.

## Change Log

| Date | Change |
|---|---|
| 2026-09-26 | **Guy's arbitration**: all six on the recommendation (§0.8). |
| 2026-09-26 | Validated by two layers (§0.9): A1 had been posed on an interpretation, the address path MEASURED merging two machines by DHCP, AC1/AC3 contradicted; **Guy's second arbitration** (§0.10), all four on the recommendation; criteria rewritten; `ready-for-dev`. |
| 2026-09-26 | Contexted. Six decisions posed with recommendations (§0.5); the central one (A) argues that a device table would be a cache of the operator's answers. Awaiting Guy's arbitration, then the mandatory validation. |

## References

`epics.md` story 6.14c · `architecture.md` D12 (`:909`), D14 (`:1068–1104`), D15 (`:1106`) ·
`0006_entity_device_and_state.sql` · `0013_operator_answers.sql` · `inventory_view.rs:159–290` ·
`page.rs` `devices` · `l2_repo::load_current_operator_answers` · `ambiguity_view::groups` ·
`gap/mod.rs:110` · `repo.rs:769` · `deferred-work.md` (6.14b's section, and *"A chain's `match` rows join
across an ENGINE `no_match`"*).
