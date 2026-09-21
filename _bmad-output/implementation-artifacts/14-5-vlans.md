# Story 14.5: VLANs — the segment a subnet belongs to

Status: in-progress

🔑 **In NO epic file.** `epics.md`'s Epic 14 body stops at story 14.4; the Epic List entry (`:474-476`)
names 14.5 and 14.6, added by the partial retrospective. So this story inherits **FR21's one sentence**
— *"The operator can manage subnets, VLANs, and DHCP ranges"* (`prd.md:906`) — and Epic 14's six
measured constraints, which it is downstream of and may not re-decide. Precedent for an epic-less
story: 6.4b, 14.4c.

✅ **Its precondition is met.** `vlan` / « VLAN » joined the PLAN axis of the binding vocabulary on
2026-09-21 (Guy's planning act, PR #192 `d192f11`, on PR #166's precedent) — epic constraint (5)
forbids a story to extend that table, and both twins carried a dated refusal: *"a term minted before its
screen exists is a term minted by accident"*. This story is that screen.

Baseline: `master` at `d192f11` — **1 018 tests** (717 bin + 191 core + 110 xtask), ten gates, axe 0
nodes over 10 routes + 5 states, kbd-probe 59 checks.

## Story

As the operator,
I want to say which VLAN a subnet belongs to,
So that my plan can hold the same address space twice — once per segment — and say which is which.

## ⚠️ Validated 2026-09-21 by two fresh-context layers, and rewritten from §0 to the Tasks

Fact-check: 3 HIGH, 9 MEDIUM, 8 LOW. Gap-hunt: 3 HIGH, 6 MEDIUM, 6 LOW, everything BUILT against its own
MariaDB 10.11.11 and its own worktree. 🔴 **Both layers reached the same headline independently, and the
gap-hunt MEASURED it with a control: the first draft's *"the audit does not change"* is FALSE the moment
the feature is used.** What the draft got wrong is recorded here rather than left beside the rewrite.

**The measurement (gap-hunt, with its control).** Subnet A (VLAN 10, `static` .1–.40, one sighting on
`.20`) alone, then the same A with a VLAN-20 twin on the same CIDR beside it — the twin's own
`dhcp-pool` over the same span, which is legal because the overlap rule is per `subnet_id`:

```
CONTROL (A alone):      audit=[(192.0.2.20, Some(Gap))]        next_offerable=Some(192.0.2.1)
AFTER (A + the twin):   audit=[(192.0.2.20, Some(Undeclared))] next_offerable=None
plan.subnets[0] == plan.subnets[1] ? true      audit(A) == audit(B) ? true
```

Declaring a subnet in ANOTHER segment turned A's `Gap` into `Undeclared` — *the gap is the product* —
emptied A's offer, and reported A's printer as *defined inside a dhcp-pool* because B has a pool there.
The cause is structural: `Subnet` is `{base, prefix_len}` with derived `Eq` and no id
(`ipam_repo.rs:99-104`), and `Plan`'s ranges and defined addresses are read PLAN-WIDE by decision 2.
The draft's argument — *nothing produces a VLAN, so the audit is safe* — is about the OBSERVED side and
never covered the DECLARED duplication its own AC2 introduces.

**Corrections of fact** (fact-check): `prd.md:1061` is now blank — the withheld sentence moved to
`:1073-1074` when Guy's act landed, and the single-VLAN evidence is `prd.md:1506`, not `:1498` (the
"F48" id could not be verified and is dropped); *"until its screen exists"* was a PARAPHRASE presented as
a quotation — the sentence is *"a term minted before its screen exists is a term minted by accident"*;
the §1 grep needed `-E` (run verbatim it returns nothing) and its perimeter omitted `fixtures/`, which
carries the word in a sha256-locked corpus (`fixtures.rs:2978`); *"FOUR tests went red"* is a DATED
figure — **2** on this tree, measured.

## §0 — What Guy settled, each with the option refused

1. **The word comes first, and it is HIS** — PR #192 `d192f11`, on PR #166's precedent; epic constraint
   (5) forbids a story to extend that table. Refused: the story proposing the row in its own T0, and
   shipping a VLAN with no word (FR21 unmet).
2. **The VLAN lives ON THE SUBNET**, and the uniqueness key goes from `(base, prefix_len)` to
   `(base, prefix_len, vlan)`. Refused: a `vlan` table subnets reference, and a free-text label.
3. **A DECLARED axis** — nothing produces a VLAN: the connector stamps `Uuid::nil()` as `l2_domain`
   (`main.rs:819`), FR2's UniFi connector does not exist, and D61 measured `client.vlan` MISSING on 48
   of 48. Refused: writing the `l2_domain` → VLAN mapping now (code no test can reach), and deferring
   the axis to Epic 11.
4. 🔴 **The audit stays VLAN-BLIND, and the screen SAYS SO** (2026-09-21, on the measurement above).
   Refused: **(b)** scoping the ranges and the offer per `subnet_id` — the only reading under which
   *"declaring a VLAN changes nothing"* is true of what the operator SEES, and it undoes Guy's decision
   2 of 2026-09-10 (*the most protective range decides, plan-wide, including across nested subnets*);
   and **(c)** keeping the CIDR unique and making the VLAN a mention, which loses the canonical VLAN
   case — one address space in two segments — this story exists for. ⚠️ **The accepted cost is written
   rather than discovered**: an operator who declares two segments on one CIDR gets ONE audit for both,
   and the degradation the control measured is what the screen's sentence must warn about.
5. **A tenth route corrects the LABEL and the VLAN only** — never `base`, never `prefix_len`, carried by
   the request type and not by a sentence. Story 14.4's arbitration is the reason: an edit that moved a
   range off an address the DELETE refused to abandon was accepted, and a subnet edit that moved `base`
   would do worse with nothing to refuse it. Refused: no route at all — a mistyped VLAN would then be
   uncorrectable, since `DeleteSubnet` is refused while the subnet holds anything.
6. **Action A3 is TWO collisions, each closed at its own cause** (the gap-hunt measured both): the test's
   CIDR moves (never the fixture's, which would be a seed shaped by a test's namespace — the complaint
   `a11y/seed.sql:230-239` itself makes), and `the_store_returns_addresses_in_numeric_order`'s
   exact-equality assertion over the PLAN-WIDE `plan_addresses` is scoped to its own subnet, which no
   uniqueness key can do for it. Refused: giving the seed a VLAN to dissolve the first, and deferring.

✅ **Settled by measurement, not by preference — the sentinel.** `vlan SMALLINT UNSIGNED NOT NULL
DEFAULT 0`, **0 meaning none** (802.1Q reserves it for *priority-tagged, no VLAN*), with
`CHECK (vlan <= 4094)`. The NULLable form was BUILT and refused: with `NULL` for none, MariaDB accepted
the same CIDR **three times** (`SELECT COUNT(*) → 3`) — story 5.9's NULL-distinctness trap reproduced on
10.11.11.

## §1 — What the tree says today, measured 2026-09-21 on `d192f11`

- **No VLAN in the code**: `grep -rniE "vlan|802\.1q|dot1q" crates/ a11y/ docker/ fixtures/` returns
  `macvlan`/`ipvlan` (Docker prose), UniFi wire-format assertions that `vlan` is ABSENT, domain prose in
  `blocking.rs`/`l1.rs`, one positive mention inside the locked fixture corpus (`fixtures.rs:2978`), and
  comments saying the plan has no VLAN axis. **No column, type or key.**
- **`ip_subnet`** (`0007:91-109`): `UNIQUE KEY ip_subnet_cidr (base, prefix_len)` at `:101`, under
  *"A plan that holds one subnet twice is not a plan"* (`:100`) — a sentence this story rewrites.
- 🔴 **The DDL shape is new to this repository.** `0010` is the FIRST migration that ALTERs an existing
  table re-runnably. Measured: the naive form applied twice gives `ERROR 1826 … Duplicate CHECK
  constraint name`; the re-runnable one needs FOUR `IF NOT EXISTS` spellings — `ADD COLUMN`,
  `ADD UNIQUE KEY`, `DROP INDEX`, `ADD CONSTRAINT` (all supported on 10.11.11, the last spelled nowhere
  in this repository) — and the NEW key must be added BEFORE the old one is dropped, or a part-way
  failure leaves `ip_subnet` with no uniqueness key at all.
- ✅ **`ddl-collation` is green on a numeric column, and the gate was proved to READ the file**: planting
  a `VARCHAR(10)` into `0010` reds it naming that line; restored, green.
- 🔴 **`cargo xtask mutate` CANNOT drive a DDL mutation** — `deferred-work.md:5164`, owner *"whichever
  story next writes a migration"*, which is this one. Two stories have routed around it.
- **`list_subnets`** orders `BY base, prefix_len` (`ipam_repo.rs:1151`) — **no longer a total order**
  once one CIDR may appear twice, and two visible things ride on the tie: which segment `/ipam` shows by
  default (`subnets.first()`), and `release_address`'s innermost-subnet redirect (measured: it answered
  *whichever row came first*, i.e. the other segment's plan).
- **The selector survives duplicate CIDRs** (it keys on the row id) — but two subnets with the same CIDR
  and empty labels render **two byte-identical links to different destinations**, measured, and no gate
  in this project can see it.
- **The port takes `(Subnet, label)`** (`ipam_write.rs:1683`), and `FakePort` logs `(cidr, label)`: a
  VLAN passed as a THIRD parameter and dropped by the handler leaves every port assertion green.
- **Action A3, measured in three arms** on a virgin store, seeded, `cargo test -p opencmdb-bin
  --no-fail-fast`: `master` → 715/2; `master` + `0010` at the 0 sentinel → **715/2, identical**; with the
  seed given a VLAN → 716/1. The second failure is `the_store_returns_addresses_in_numeric_order`, a
  plan-wide read compared for exact equality, which **no key can dissolve**.
- **`ipam_page.rs` is at 1882 code lines of 2000** (118 of headroom) and AC4 + AC7 both land there; story
  6.4 hit 2033 in this area and the answer was a split. 🔴 **And `ipam_write.rs` is a second, unregistered
  instance of `repo.rs`'s blindness**: 1640 production lines where the `file-size` gate reads **302**,
  because a `#[cfg(test)]` sits on a helper inside an `impl` at line 302.
- **The outside list's release** is a control, not a rule: the route accepts one and redirects to `/ipam`
  (measured); the list lives in ONE shared partial (`_ipam_audit.html:89-113`), and its rows carry
  `kind: None`, so the existing `{% if row.gap || row.undeclared %}` renders nothing for them.
- **The manual already claims VLANs**: `user-manual.tex:154` ships FR21's sentence with **no
  `\planned` block**, and its *Defining the addressing plan* list carries CIDR and label only.

## Acceptance Criteria

**AC1 — migration `0010` gives `ip_subnet` its VLAN axis**: the sentinel above, the key widened to
`(base, prefix_len, vlan)` — the new key ADDED BEFORE the old is dropped — the four `IF NOT EXISTS`
spellings, the `success = 0` recovery recipe, and **proved re-runnable by applying `0010` twice to one
store**. `ddl-collation` green (a numeric column carries no collation, and saying so is the criterion).

**AC2 — the plan holds ONE CIDR TWICE**, once per VLAN, and refuses it twice in one VLAN with a 409
**whose sentence names the VLAN**: the refusal must not read as a refusal of the CIDR, which is the
capability this story ships. `ipam.refusal.malformed_subnet` names the optional VLAN among the form's
fields.

**AC3 — the subnet form carries the VLAN**, optional, refused by a keyed 422 when it is not a number in
range; §0.5's correction route edits the label and the VLAN under the same rules, with **its own
refusal sentence** for a correction that would collide; and the VLAN reaches the store **through the
port** — the fake port records it and a test asserts it, or it rides inside the type the port takes.

**AC4 — the VLAN is on the screen, and the two sentences it owes are rendered.** The selector tab reads
`CIDR · VLAN n · label`, and **a VLAN of 0 renders nothing** (0 is *none*; rendering it states something
false about the plan). The property, not the appearance, is the criterion: **no two selector tabs carry
the same accessible name.** One keyed sentence under the selector, whenever any subnet declares a VLAN,
says both halves of §0.4: *the plan has a VLAN; the ranges, the findings and the offer do not.* The seed
declares a VLAN so the sentence is on a page the browser gates walk, with an **`AXE_REQUIRE_VLAN=1`**
that makes a run without one *the gate could not run* rather than a pass.

**AC5 — the audit is VLAN-BLIND, and a test MEASURES what that costs.** A store test seeds one CIDR in
two VLANs and asserts, **with the control of the same subnet alone**, that the two tabs carry the same
findings, the same grid and the same offer — and that the twin CHANGES the first subnet's answers, which
is the measurement above made permanent. `one_address_in_two_l2_domains_is_read_as_a_conflict` stays
green with its reason rewritten.

**AC6 — action A3 is discharged, and it is TWO collisions**, each closed at its own cause (§0.6):
`cargo test --workspace` is green when run AFTER both browser gates against one store — measured, not
asserted — and a comment names both collisions and which mechanism closed each. ⚠️ It records that the
widened key dissolves NEITHER on its own (715/2 with and without `0010`).

**AC7 — the outside list offers a release**, keyboard-reachable and under axe; its rows carry no
`kind`, so the condition that renders the existing control does not reach them.

**AC8 — every refusal is a KEYED body in both locales**, and the vocabulary gates stay green. ⚠️ The
second half of the first draft — *"`l2_domain` and `network_id` must not appear in the interface copy"* —
is **withdrawn as unimplementable**: `l2_domain` ALREADY renders on the example identity screen
(`app.yml:678`, eight sites), and a denylist entry would red on those and on the row Guy just wrote.
What ships instead is the SENTENCE of AC4, which says which word means what.

**AC9 — THE LIVE COUNT lives in this story's `## Record` block**, checked by `cargo xtask record`.

**AC10 — no regression**: ten gates, `clippy --all-targets`, `RUSTFLAGS="-D warnings"`, both store
conditions, `cargo deny`, **both browser gates** (this story changes the screen), both manuals — and the
user manual's IPAM chapter gains the VLAN, which no other criterion owns.

## Tasks / Subtasks

- [x] **T0** Decisions taken with Guy (§0.4, §0.5, §0.6 on 2026-09-21; the sentinel settled by the
      gap-hunt's measurement).
- [ ] **T1** (AC1, AC2) Migration `0010` applied twice; the adapter, the widened key, `ORDER BY … , vlan`;
      the two store tests and the 409 that names the VLAN.
- [ ] **T2** (AC3) The form field, the port, the correction route and its refusals.
- [ ] **T3** (AC4) The tab, the two sentences, `AXE_REQUIRE_VLAN`, the seed's VLAN; the distinct-name
      property.
- [ ] **T4** (AC5) The audit tests with their control; the rewritten reasons.
- [ ] **T5** (AC6) Both collisions, measured after the gates on one store.
- [ ] **T6** (AC7) The outside list's release control; both browser gates.
- [ ] **T7** (AC8, AC9, AC10) Measure; prove-to-red — ⚠️ **`cargo xtask mutate` cannot drive the DDL
      half** (`deferred-work.md:5164`, owner this story): drive it with a purpose-built script and
      **record the pass back into that register row**. `cargo xtask record`; the manuals; both twins.

## Dev Notes

### Traps this project has paid for, and which apply here

- 🔴 **MariaDB holds NULLs DISTINCT in a UNIQUE key** — §2.1's whole reason (story 5.9's measured trap).
- 🔴 **A true sentence about a changed world** — eight sites, and the first draft listed four:
  `ipam_audit.rs:76-91` and `:898-923`, `deferred-work.md:5699`, `0007:100`, **`ipam_repo.rs:1144`'s
  `list_subnets` doc** (*"no caller sorts and no caller can forget to"*), **`a11y/seed.sql:230-239`**
  (A3's whole narrative), **`ipam_write.rs:788-792`** (*"two subnets may legally carry ranges with
  identical bounds"* — after this story that is the MECHANISM of §0.4's cost, not a curiosity), and
  **`ipam.refusal.already_defined_subnet`** (AC2). Rewrite them in the same commit.
- 🔴 **A guard where the defect cannot occur** — §0's decision 3 refuses the mapping for that reason.
- ⚠️ **Exact counts that WILL red** (equalities, moved only after reading what they print):
  `every_ipam_key_this_file_names_resolves_in_both_locales` (**77**), `ipam_write.rs`'s key guard
  (**42**), `every_variant_is_in_the_route_list` (**9** → 10 with §0.5's route), and `main.rs:1816`'s
  *"ten write routes today"* premise (→ **11**). Conditional: the seed's **15** sightings (only if T5
  reshapes its addresses). 🔴 **NOT counts that will red, where the first draft said they were**:
  `MIN_CHECKS` (59) and `MIN_ROWS` are FLOORS — adding checks leaves them green, so they are raised by
  discipline; `EXPECTED_ROUTES` (10) counts NAVIGATION screens and this story adds none.
- ⚠️ `cargo xtask mutate` REQUIRES a store and refuses at the baseline without `DATABASE_URL`; a
  refused baseline no longer leaves a snapshot (14.4c). It cannot drive DDL at all (T7).
- ⚠️ **One address, two segments**: `ip_address` is `UNIQUE (subnet_id, addr)`, so `192.0.2.5` can be
  defined in both VLANs with two labels, while `Plan.defined` is a `BTreeSet<Ipv4Addr>` that merges them
  — the rail shows two records the grid draws as one cell. Benign (it only makes the audit more
  protective) and named so nobody meets it mid-T1.
- ⚠️ **`ipam_page.rs` has 118 lines of headroom and `ipam_write.rs`'s size is invisible to the gate**
  (§1): if AC4 or AC7 crosses the ceiling, the answer is the split `CLAUDE.md` prescribes, and the
  second blindness is REGISTERED rather than rediscovered.
- ⚠️ **This story cannot be verified on the reference LAN** (`prd.md:1506`; D61): the developer's network
  is single-VLAN. Every VLAN behaviour is exercised by fixtures, and saying so is honest rather than a
  gap to hide.

### References

- `prd.md:906` (FR21), `:1042` (the `vlan` row), `:1061` (what the row replaced), `:588`, `:1498`.
- `epics.md:474-476`, `:2339-2359` (the six constraints).
- `architecture.md:4319-4343` (**D61**, the open risk), `:1040-1041` (L1's key), `:1196-1201`.
- `0007_addressing_plan.sql:91-109`; `0008:60-79`; `0009:53-58`.
- `ipam_audit.rs:76-114`, `:899-922`; `ipam_repo.rs:99-104`, `:237`, `:1144`, `:2218-2239`;
  `ipam_write.rs:218-228`; `a11y/seed.sql:230-253`.
- `deferred-work.md:5699-5705` (the merge's price, owner this story), `:5921-5924` (the outside list).
- `epic-14-retro-2026-09-19.md` §5 decision 1, §6 A3, §7.
- Precedents: 14.1 (a schema whose criterion is *no producer*), 14.2b (the write machinery, constraint 1).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

## Record

- live-count: bin=727 core=191 xtask=110
- base: d192f11e7f6fd356be2207ce5aaed321ac94f107
- registered: Two tests carry an UNQUALIFIED `DELETE FROM ip_subnet`
- registered: is a second, now-registered instance of `repo.rs`'s `file-size` blindness
- registered: press depends on focus left by an EARLIER check
- file: .github/workflows/ci.yml
- file: _bmad-output/implementation-artifacts/14-5-vlans.md
- file: _bmad-output/implementation-artifacts/deferred-work.md
- file: _bmad-output/implementation-artifacts/sprint-status.yaml
- file: a11y/axe-gate.mjs
- file: a11y/kbd-probe.mjs
- file: a11y/seed.sql
- file: crates/opencmdb-bin/assets/app.css
- file: crates/opencmdb-bin/locales/app.yml
- file: crates/opencmdb-bin/migrations/0010_subnet_vlan.sql
- file: crates/opencmdb-bin/src/ipam_audit.rs
- file: crates/opencmdb-bin/src/ipam_page.rs
- file: crates/opencmdb-bin/src/ipam_repo.rs
- file: crates/opencmdb-bin/src/ipam_write.rs
- file: crates/opencmdb-bin/src/main.rs
- file: crates/opencmdb-bin/templates/_ipam.html
- file: crates/opencmdb-bin/templates/_ipam_audit.html
- file: crates/opencmdb-bin/templates/_ipam_forms.html
- file: crates/opencmdb-bin/templates/_ipam_rail_lists.html
- file: docs/manuals/user-manual/user-manual.tex

### File List

The `## Record` block's `file:` lines are this story's File List (checked by `cargo xtask record`).

### Change Log

- 2026-09-21 — Contexted, on Guy's three decisions of 2026-09-21 (the word minted first, the VLAN on the
  subnet, a declared axis whose limit is said).
