# Story 6.12: The resolver writes device groupings

Status: **developed 2026-09-24, code-reviewed by three isolated layers and REPAIRED 2026-09-25 — `review`,
not `done`, because that is the merge's business.** Contexted, validated and arbitrated the same day: §0 carries three findings
that falsify the premise this story was queued on; both validation layers tried to refute them and **all
three survive**. The validation added four HIGH findings against contexting's recommendation (§0.5).
**Guy took all seven decisions on 2026-09-24 (§0.6), and the criteria in §3 are written for them.**
⚠️ *The story now CONTAINS 6.13's writer; its scope is no longer the letter of `epics.md:1966-1982`.*

**Epic 6**, on Guy's reorder of 2026-09-23: 6.9 (done) → 6.11 (done) → **6.12** → 6.13 onward; 6.8 and
6.10 wait for a connector that emits an uplink or a switch port. **Base:** `master` at `6b3c04e`, clean
tree. **Live count at base:** bin=751 core=214 xtask=110 (`cargo test -p <crate> --locked -- --list`,
the command `cargo xtask record` runs; doc-tests excluded).

⚠️ *Contexting first wrote **xtask=111** here, with a warning that story 6.11's `## Record` was wrong.
**Both validation layers refuted it independently**: my `awk` attributed every `: test` line to the last
`Running` header, so `opencmdb_core`'s one doc-test — printed under `Doc-tests`, AFTER xtask's binary —
was counted as xtask's. 6.11's record was right. *A counting instrument that attributes by position
measures the order of the output, not the tests* — and I published it as a correction of someone else.*

---

## Story

As the operator,
I want the engine's grouping decisions persisted,
so that a device is a record rather than a computation repeated at each page load.

*(`epics.md:1966-1982`, verbatim. §0.3 measures that the "so that" clause describes a computation no page
performs.)*

## The epic's criteria, verbatim (`epics.md:1972-1982`)

1. **Given** a population of interfaces and the L2 cascade, **When** the pass runs, **Then** it writes
   `device` rows and their memberships, in D13's order, and **every placement is justified by a rule id
   and its evidence** — never *"merged, with no explanation"*.
2. **Given** a second identical pass, **When** it runs, **Then** it writes nothing — idempotence, story
   5.11's rule, and the same purge-and-replay invariant story 5.10 pinned at L1 must hold at L2.
3. **And** ⚠️ **two races registered at Epic 5 lose their shield here.** Two concurrent passes mint two
   interfaces for one MAC (`interface_l1_key` is a plain index and the mint is read-then-insert), and
   `current_subject IS NOT NULL` is not equivalent to `valid_to = OPEN_END`. Epic 5 recorded that *the
   connector story that gives it a MAC removes the shield*; **a device grouping that keys on interfaces
   reaches the same code.** This story carries them or names the story that does.

   *(This criterion was quoted with two clauses dropped while the heading said "verbatim"; the
   fact-check layer restored them. The second dropped clause is the epic already making §2's point
   about a read-then-insert device mint.)*

---

## 0. What contexting measured, and what it will not settle alone

### §0.1 — 🔴 THE HEADLINE: under the shipped algebra, NO L2 pair can ever conclude `Match` — so criterion 1 writes no grouping of two interfaces, on any network

`decide` (`cascade.rs:501-566`) reaches `Conclusion::Match` through exactly ONE arm,
`(None, Some(rule), _, false)`, and that arm needs a **`Decisive`**. The L2 rules that exist produce:

| rule | producer (production code only) | verdicts it can emit |
|---|---|---|
| `l2-different-hostname` (6.7) | `l2.rs:239` | `Opposes`, `Neutral` |
| `l2-hostname-agrees` (6.9) | `l2.rs:399` | `Supports`, `Neutral` |
| `l2-virtual-mac-prefix` (6.11) | `l2.rs:574` | `Disqualifying`, `Neutral` |

**No L2 production code emits `Decisive`** (measured: `rg 'Verdict::Decisive'` over `l2.rs` lines
1-599 returns nothing; its only production emitters are `l1.rs:279` and `l1.rs:361`). So over an
L2-only vector the reachable conclusions are `NoMatch { l2-virtual-mac-prefix }`,
`Abstained { AbsenceOfProof }` (nothing, or `Opposes` alone) and **`Abstained { Ambiguous }`**
(`Supports` alone, or `Supports` + `Opposes`). **`Match` is unreachable.**

🔑 **Consequence for criterion 1:** *"it writes `device` rows and their memberships … every placement
justified by a rule id and its evidence"* can be satisfied by a multi-interface device **only in a test
that hand-builds a `Decisive` L2 verdict** — which no rule produces. *A criterion satisfiable only with a
fabricated verdict is a guard placed where the defect cannot occur* (Epic 5's dominant class, named at its
retrospective). ✅ *Both layers confirmed it; the gap-hunt's prototype tallied **1035 pairs over a
46-interface sweep with no `Match` among them**.* ⚠️ The claim is about an **L2-ONLY** vector: a MIXED
vector can reach `Match` — a multi-homed observation standing on both sides makes L1 answer `Decisive`
(`l2.rs:34-42`'s own counterexample) — which is exactly the composition §5 forbids, and is unreachable
through the shipped connector (one MAC per observation). Story 6.9 already measured the same algebra from the trap side (`Supports` alone → Fail
on `must-merge`) and registered *"what makes a merge at L2"* with **Epic 6's retrospective**; this story
is where that open question stops being a trap-gate question and becomes a **product** one.

### §0.2 — 🔴 `obelix` does not stop being two rows here — it becomes an `Ambiguous`, and the writer REFUSES it

Issue #158 records `192.168.1.8` and `.9` both resolving to `obelix.home.arpa`. With PR #163's MAC read,
two distinct NICs give two interfaces; one sweep's side for each carries `{obelix.home.arpa}`. Over the
three L2 rules: `l2-hostname-agrees` → **`Supports`** (equal sets), `l2-different-hostname` → `Neutral`
(not disjoint), `l2-virtual-mac-prefix` → `Neutral` (not IANA). **`decide` → `Abstained { Ambiguous }`.**

✅ **Two NICs, confirmed by Guy on 2026-09-24** — the conditional (*a bond would give one interface and no
pair*) is discharged by the owner of the machine. ⚠️ *Stated as Guy's knowledge of his hardware, not as a
reading of the NAS store*: the store's `interface` rows for `.8` and `.9` are the product-side
confirmation, and the first sweep after this story ships is where it is seen.

🔴 **And the ONLY writer in the tree refuses that decision.** `resolver::guard_decision`
(`resolver.rs:712-727`) returns `Constraint("ambiguity_without_candidates")` for any `Ambiguous` written
with an empty candidate slice, and its only call site passes `&[]` (`resolver.rs:528`). `scan_pass.rs:178`
runs `resolve` inside ONE `transact`. So **an L2 write routed through the same guard, inside the same
transaction, turns `obelix` into a rolled-back sweep** — ✅ *measured by the gap-hunt end to end
(`resolve` then `guard_decision(&d, &[])` in one `transact` → `Err(Constraint("ambiguity_without_candidates"))`,
then `identity_link = 0, interface = 0`; control without the guard: 2 links), and already recorded in the
tree by the doc of 5.14b's tripwire at `scan_pass.rs:746-748`* — — every L1 link of that sweep lost with it, at every
sweep, on the reference network. *The pivot's live specimen is exactly the input that would break the
pass.* Filling `link_candidate` is story **6.13**'s criterion, and 6.13's own AC says story 5.14b's tripwire
reds when it lands.

🔑 **The reorder note at `epics.md:1883-1888` and `sprint-status.yaml`'s comment on this key both promise
*"it is where `obelix` … stops being two rows"*. Under the present algebra and the present story order,
that sentence is false twice** (no `Match`; and the `Ambiguous` it does produce is unwritable). Neither
file is edited here — a story may not edit `epics.md` — and §0.4 poses the correction.

### §0.3 — 🔴 No screen reads an interface or a device, so even a written grouping changes no pixel

Measured with `rg` over `page.rs`, `identity_view.rs`, `dashboard_view.rs`, `inventory_view.rs` and
`opencmdb-core/src/gap/`:

- the **triage queue** and the **gap** are keyed on an IDENTITY FIELD VALUE — `gap::reconcile(identity:
  (&str, &str), …)` (`gap/mod.rs:120`), the perimeter key being `ipv4` — so `obelix`'s two rows are two
  ADDRESSES, and they stay two rows whatever `device` holds;
- the **inventory** lists DECLARED entities (`inventory_view.rs:19-22`: *"the declared side, and only
  that"*), whose `entity_id` sits **outside** the supertype (`deferred-work.md:5127-5133`);
- the **only** query reading `identity_link` is `repo.rs:1425 count_engine_reach`, which counts
  `COUNT(DISTINCT observation_id)` with **no level filter** — and it feeds **four** screens:
  `page.rs:1364` (`reconcile_view`), `page.rs:1412` (triage), `page.rs:1637` (dashboard) and
  `diagnostic.rs:619`, which derives placed / not placed from `outcome == "match"`.

So the story's *"so that a device is a record rather than a computation repeated at each page load"*
describes a computation **no page performs**. Making `obelix` one row is a change to the gap's subject
(architecture: *"the gap is computed at the `device` level"*, `architecture.md:1486`) — a change the
product has never made and that **no Epic 6 story names** (6.19 is a per-device history read, not the
gap). ⚠️ **What the operator gains from 6.12 under ANY option below: nothing visible** — the SIXTH
consecutive engine-spine story in that shape (6.5, 6.6, 6.7, 6.9, 6.11, registered by 6.11's audit layer
with Epic 6's retrospective as owner). This story should say so in its §6 and carry the count.

### §0.4 — The four decisions that are Guy's, each with a recommendation

**(A) What 6.12 is, given §0.1–§0.3.** Three shapes, none of which delivers the promise as written:

- **(A1) Plumbing to the letter.** Build the L2 pass, the membership table and the device mint; persist
  `Match` and `NoMatch`; **do not persist L2 abstentions** (6.13 owns `Ambiguous` + candidates). On every
  real input the pass then writes **no device row at all** (no `Match` exists), and criterion 1 is carried
  by a hand-built `Decisive` only. Cheap, honest if said, and the sixth story giving nothing.
- **(A2) 6.12 absorbs 6.13's writer.** Persist every L2 decision — `Match`, `NoMatch`, **and `Ambiguous`
  with its candidates** — so `obelix` becomes ONE persisted question with two candidates and their
  evidence. 6.14 then displays it and carries the *lift the doubt* gesture (`epics.md:2002-2016`). 🔑 This
  is where the operator value actually sits: the architecture's own wording is *"we do not guess, we
  expose"* (`architecture.md:1481`), and a shared hostname is weak evidence BY DESIGN (issue #159).
  ⚠️ It merges two stories and re-scopes the epic — a planning act. 🔴 *Contexting also wrote here that
  it "reds 5.14b's tripwire by design (6.13's AC)". **Both layers refuted that, by two independent
  reasons**: the tripwire's slice forms ONE interface and therefore **0 L2 pairs** (measured, `0`), and it
  reads `count_engine_reach`, which reads `identity_link` only — so under (B) no L2 `Ambiguous` can ever
  reach it. **6.13's own AC ("REDS here, by design") is unmeetable through that test under ANY option**,
  and is registered (§0.5).*
- **(A3) Settle "what makes a merge at L2" first.** A D13 amendment or a `Decisive` L2 rule, taken as a
  planning act before any story; registered by 6.9 with Epic 6's retrospective. Until then no grouping of
  two interfaces exists anywhere.

**Recommendation: (A2), preceded by nothing** — it is the only shape in which the reference network's live
specimen reaches a persisted state the product can later show, and it does not require deciding (A3)'s
false-merge question now. ⚠️ Recorded as a recommendation, not a decision: it changes two stories' scope.

**(B) Where an L2 decision is persisted.** `0006`'s header and `deferred-work.md:5140` record the plan as
*"the migration widening `identity_link` to admit a device as a placement subject"*. Measured against the
schema, widening is costly (✅ confirmed by the fact-check): `identity_link.observation_id` is `NOT NULL` with a foreign key to
`observation_record` (`0003`), its uniqueness key is `(observation_id, current_subject)`, and
`count_engine_reach` counts rows by observation with no level column — an L2 row either cannot be written
or silently enters the reach section's counts. **Recommendation: a SIBLING SCD2 table** with
`identity_link`'s shape at L2 — subject interface, placement device, `outcome`, `rule_id`,
`abstention_cause`, `evidence`, `ruleset_version`, `decided_by`, `valid_from`, `valid_to`,
`current_subject` — *D12's "one engine, instantiated twice" is a statement about the ENGINE's shape, not a
requirement to share one table.* ⚠️ This DIVERGES from the recorded plan's wording, so it is Guy's.

**(C) Does `interface` join the `entity` supertype here?** 6.5's arbitration (a) assigned the adoption to
6.12 *"together with"* the widening and the resolver mint, as ONE gesture (65 tests red when split). Under
(B)'s sibling table the membership can reference `interface(id)` and `device(id)` directly, and the
adoption becomes independent of this story — but it is a migration that BACKFILLS existing rows at boot on
a published product (`v0.4.0` on the NAS, ~45 interfaces). **Recommendation: adopt only if (B) keeps
`identity_link`; otherwise re-own it by name** (to the first story whose query needs an interface as an
`entity`, e.g. 6.18's `dormant`, which sets `entity.state` on an interface).

**(D) The population an L2 pass judges.** An `L2Side` is *"the observations that landed on one
interface"* and its bound is this story's (`deferred-work.md:6316`). The pass sees ONE sweep's `landed`
slice (`scan_pass.rs:178`), so `join(landed)` already yields each key's side for that sweep.
**Recommendation: judge the interfaces THIS pass touched, with sides drawn from THIS pass's slice, and
never vacate a membership for an interface absent from the slice** — story 5.14 measured that the naive
widening *"erases a host that missed a single scan"*. ⚠️ That choice silences nothing across sweeps for
`l2-hostname-agrees`'s set equality (one sweep, one name), which answers the register row; it also means
two interfaces seen in DIFFERENT sweeps are never paired, which is a reach limit to state.

### §0.5 — 🔴 What the validation added: four HIGH findings against contexting's recommendation, and three more decisions

The gap-hunt BUILT (A2)+(B) — a prototype migration `0012`, an L2 pass, a sibling candidate table — ran it
against a virgin store and ran two mutations with predictions written first. **(B) as contexting drew it
does not work.**

**(E) 🔴 THE GRAIN. The §0.4(B) table cannot hold two L2 decisions about one interface, and the first
collision rolls the whole sweep back, L1 included.** L2 decisions are about PAIRS; the drawn table keeps
one current row per *(subject interface, current_subject)*, and an abstention or an L2 `NoMatch` has **no
device to name** (at L1 a `NoMatch` names the interface it excluded; at L2 there is nothing), so every such
row lands in the nil slot. Measured, each pair written for both its interfaces: `obelix` alone → `Ok(2)`;
**three NICs named obelix → `Err(Constraint("unique"))` and `identity_link = 0` afterwards**; obelix +
asterix with `AbsenceOfProof` persisted → same; obelix + one VRRP MAC → same. *§0.2's rollback, returning
through the uniqueness key instead of the guard* — and any LAN with a VRRP MAC produces n−1 `NoMatch` rows
on one interface. **Options:** (E1) a PAIR-keyed table — `(interface_low, interface_high)` plus the current
sentinel — which matches what an L2 decision IS; (E2) a per-interface aggregation rule, the L2 twin of
L1's smallest-witness convention, which needs its own arbitration. **Recommendation: (E1).** AC3 and AC5
cannot be written until this is taken.

**(F) 🔴 CROSS-SWEEP IDEMPOTENCE. A real second sweep of an UNCHANGED network writes again.**
`arp_ping.rs:254` mints `Uuid::now_v7()` per observation and evidence is a list of `ObsId`s, so evidence
changes at every sweep. Measured (`gh7_two_real_sweeps_of_an_unchanged_network`): same two obelix NICs,
fresh ids → *"first wrote 1, second wrote 1"*. Under SCD2 every current `Ambiguous` is closed and
re-appended every five minutes — **~105 000 history rows per ambiguous pair per year** (reasoned from 288
sweeps a day, not measured over a year) — the same growth story 5.14 and the `sightings_without_a_key`
counter already paid for once. Hostname flapping (PTR answering one sweep and not the next) adds churn.
Criterion 2's *"a second identical pass"* is satisfiable only with identical ids, which the shipped
connector never produces. **Options:** (F1) the decision-bearing columns exclude per-sweep observation ids
and the evidence is keyed on INTERFACES (the pair's keys), which are stable across sweeps; (F2) keep
`ObsId` evidence and accept the churn, stated. **Recommendation: (F1)** — and AC5 gains the property that
matters: *an unchanged network writes nothing*.

**(G) 🔴 WHICH OUTCOMES ARE PERSISTED — scale and the transaction unit.** On 46 interfaces the pass judges
**1035 pairs per sweep: 989 `AbsenceOfProof`, 45 `NoMatch`, 1 `Ambiguous`** (measured). The architecture
caps a transaction at ~100 decisions or 1000 rows (`architecture.md:1498`) and `scan_pass.rs` runs the whole
pass in ONE `transact`. Persisting `AbsenceOfProof` pairs breaks the cap and writes ~1000 rows about
nothing; not persisting them means a pair that DECAYS from `Ambiguous` to `AbsenceOfProof` needs an explicit
close rule, which (D)'s *"never vacate"* does not cover. **Recommendation: persist `Ambiguous` (with
candidates) and `NoMatch`; do not persist `AbsenceOfProof`; close a current row when a pass that CARRIED
both interfaces reaches a different conclusion** — (D) amended to *"never vacate a pair unless both its
interfaces were in the slice"*. ⚠️ 45 `NoMatch` rows per sweep for the VRRP family is itself a volume to
weigh; persisting only `Ambiguous` is the smaller alternative.

**Four more findings the ACs must carry:**

- 🔴 **Candidates cannot use `link_candidate`** (fact-check): its `link_id` references `identity_link(id)`
  with `ON DELETE CASCADE` (`0002:106-118`). (B) needs a sibling candidate table, covered by an L2 purge
  and by the replay snapshot — under (E1) the pair IS the candidate set, and a separate table may be
  unnecessary: *to decide with (E)*.
- 🔴 **Two draft guards could not fail** (gap-hunt, `cargo xtask mutate --baseline`, virgin store,
  predictions first, both conforming): **M1** removed the seam's containment check → **red 1**, only the
  CONTROL — AC2's test withheld an ordinary pair, which concludes `AbsenceOfProof`, which (A2) never
  persists, so it stayed GREEN (clippy reddened only incidentally, on an unused variable a subtler mutation
  avoids); **M2**, a writer that persists NOTHING → **red 1**, only the control — *"a second pass writes
  nothing"* is green under a writer that never writes. AC2 and AC5 are rewritten below.
- **Evidence.** `write_link`'s first-verdict evidence persists `obelix` with **EMPTY** evidence: its vector
  is `[Neutral (0 ids), Supports (2 ids), Neutral (0 ids)]` in rule order. And **all 45 VRRP `NoMatch`
  carry empty evidence on every verdict** — `l2-virtual-mac-prefix` reads no observation, by 6.11's
  design. AC3 takes the UNION and states the rule whose evidence is the pair's keys.
- **Twenty `DELETE FROM interface` sites in nine files** (tests' cleanups in `resolver`, `scan_pass`,
  `repo`, `main`, `page`, `diagnostic`, `sighting_repo`, `fault_injection`, `ipam_page`) fail `ERROR 1451`
  once a new child table of `interface` holds a row; `purge_engine_links` purges `identity_link` only. A
  task, and a DRY opportunity (one shared cleanup helper) the dev must take or refuse in writing.

✅ **Confirmed by the prototype**: the new coupling `(valid_to = OPEN_END) = (current_subject IS NOT NULL)`
refuses the UNKNOWN row by name (`4025 … failed`) where `identity_link` accepts it (`Ok(1)`, then missed by
`count_engine_reach`) — ⚠️ provided the VALUE check does not compare a NULL device (`COALESCE(device_id,
nil)` was clean; a bare `current_subject = device_id` reopens UNKNOWN). (A2) needs no device row for an
`Ambiguous` pair. (B) leaves `count_engine_reach` untouched structurally. `0012` alone reds no existing test.

⚠️ **Still unmeasured**: whether `obelix` is two NICs or a bond (no NAS store was available); whether a VRRP
MAC exists on the reference LAN; the #161 race (not reproduced by the gap-hunt, whose instrument was not
built to open that window).

### §0.6 — ✅ GUY'S ARBITRATION, 2026-09-24 — all seven decisions, each on the recommendation

| | decision | refused, and why |
|---|---|---|
| **A** | **(A2)**: 6.12 persists L2 decisions **including `Ambiguous` with its candidates** — 6.13's writer moves here | (A1) writes nothing on any real input, a sixth invisible story; (A3) forces the false-merge question now without needing its answer |
| **B** | **A SIBLING table**, not a widened `identity_link` | widening breaks its `NOT NULL` observation FK, its uniqueness key, and the reach count feeding four screens |
| **C** | **`interface` is NOT adopted into `entity` here — re-owned by name to story 6.18** (`dormant`, the first story that writes an interface's `entity.state`) | an adoption with a boot-time backfill on a published product, for nothing this story reads. ⚠️ This DIVERGES from 6.5's arbitration (a) recording the three gestures as one: under (B) the widening no longer exists, so the bundle's premise is gone |
| **D** | **Judge the interfaces THIS sweep carried; close a pair's current row only when BOTH its interfaces were in the slice** | vacating absent interfaces erases a host that missed one sweep (5.14) |
| **E** | **A PAIR-keyed table** — `(interface_low, interface_high)` plus the current sentinel; the pair IS its candidate set | a per-interface row collides on the nil slot and rolls the sweep back (measured); an L2 smallest-witness rule would need an arbitration of its own |
| **F** | **Evidence keyed on INTERFACES**, so decision-bearing columns hold nothing minted per sweep: **an unchanged network writes nothing** | `ObsId` evidence re-writes every current row every five minutes (~105 000 history rows per pair per year, reasoned). Cost accepted: which OBSERVATIONS a rule read is recoverable through the interfaces, not stored |
| **G** | **Persist `Ambiguous` and `NoMatch`; never `AbsenceOfProof`** | 989 "nothing to say" pairs per 46-interface sweep would break the transaction cap and say nothing; under (F) the VRRP `NoMatch` rows are written once and then never again |

🔑 **And two decisions outside this story, taken in the same breath:**

- **Story 6.14 follows 6.12 directly** — it displays the ambiguity and carries *lift the doubt*, and is where
  `obelix` becomes VISIBLE. Without it, (A2) still changes no pixel.
- **`epics.md` is corrected by Guy's planning act**, not by this story: the reorder note's *"obelix stops
  being two rows"* (`:1883-1888`), story 6.13's writer moving here, and 6.13's criterion that 5.14b's
  tripwire *"REDS here, by design"*, which is unmeetable under every option (§0.5).

⚠️ **Consequences the dev agent inherits from the arbitration:**

- **No `device` row is written by this story on any real input** — `Match` is unreachable (§0.1), and
  neither `NoMatch` nor `Ambiguous` needs a device under (E). `insert_device` keeps NO production caller.
  Criterion 1's *"writes `device` rows"* is therefore **not met, by decision** — say so in §6 and in the
  register, owner: whichever story first produces an L2 `Decisive` (the open question 6.9 registered with
  Epic 6's retrospective). Do not satisfy it with a hand-built `Decisive` in a test and call it met.
- A later `Match` will need a device mint with **no read-then-insert window** (a derived id, e.g. UUIDv5 of
  the pair — reasoned, not measured); that is the same owner's, and it is where criterion 3's *"a device
  grouping that keys on interfaces reaches the same code"* actually bites.
- The interface-mint race (#161) is **not** carried here: this story mints no interface it did not already
  mint. Re-owned by name to the FR6 scheduler story, with reachability stated (`spawn_scan_loop` runs one
  pass at a time; two instances on one store are needed) and the two registrations (`:2472`, `:3068`)
  reconciled into one.

---

## 1. Measured facts (at `6b3c04e`)

- **Schema.** `interface` (`0002`) is outside `entity` (`0006` header). `entity(id, kind, state)` with
  `PRIMARY KEY (id)` + `UNIQUE (id, kind)`; `entity_kind_domain` already admits `'interface'`.
  `device(id, kind='device')` with `device_entity_fk (id, kind) → entity`. ⚠️ D16's `virtual_device`
  (`kind='virtual'`) is **refused** by `device_kind_constant CHECK (kind = 'device')` — not this story's.
- **The adapter.** `insert_entity` (`repo.rs:1692`), `insert_device` (`repo.rs:1737`, atomic since 6.5's
  review), `load_entity` (`repo.rs:1768`). **No production caller of any of them.**
- **`repo.rs` is at 1789 code lines** (the lines before the last top-level `#[cfg(test)]` at `:1790`; the `file-size` gate is
  blind to it, registered by 6.5). 211 lines of headroom: **any new adapter goes in a new module**
  (`device_repo.rs`, on `sighting_repo.rs` / `ipam_repo.rs`'s precedent), not in `repo.rs`.
- **`resolver.rs`** — 751 code lines. ⚠️ **Its module doc is FALSE since story 5.14**: *"# Not wired into
  `main.rs` — by decision"* plus `#![allow(dead_code)]` (`resolver.rs:97-103`), while `scan_pass.rs:178`
  calls `resolve` in production. This story touches the file and owes the correction, and must narrow the
  blanket `allow` rather than inherit it (PR #163's P5: a blanket `allow` hid a severed MAC read).
- **The L2 organs**, all in `opencmdb-core`: `l1::join` → `BTreeMap<L1Key, BTreeSet<ObsId>>`;
  `blocking::l2_candidates(&[L1Key]) -> BTreeSet<L2CandidatePair>` (TOTAL, no production caller);
  `L2Side::new(Vec<&Observation>)`; the three rules above. **No `l2::decide_pair` exists** — L1 has one
  (`l1.rs:317`); L2 has none, so today a caller assembles the vector by hand.
- **`write_link` keeps the FIRST verdict's evidence only** (`resolver.rs:529-533`), with a doc saying
  *"Epic 6's cascade ends it … registered with Epic 6"*. **At L2 the vector has three verdicts** — this
  story is that day. It pairs with `deferred-work.md:5420` (evidence not de-duplicated): **one decision,
  taken once**, in the composing function or in the writer, not in both.
- **Test infrastructure.** DB-backed tests take `DB_TEST_LOCK` (6.5's virgin-store race). The suite is
  non-deterministic against a REUSED database (6.6's row): **drop and recreate before every count**. Live
  MariaDB for this project: container on port **13419**; **port 3306 is another project's**. Never
  `DATABASE_URL=""` — use `env -u DATABASE_URL`.

## 2. The register rows this story owns by name (`deferred-work.md`)

Each must be **answered, re-owned by name, or closed** — never left pointing at a finished story.

| line | row (first bold title) | contexting's reading |
|---|---|---|
| 5140 | `interface` is NOT a subtype of `entity` | §0.4(C) |
| 5236 | `UPDATE entity SET kind` succeeds on a childless row | answer only if this story writes `entity`; else re-own |
| 5274 | The uplink narrowing is UNWRITABLE inside the L2 blocker … | **must carry**: the production caller's universe is `l2_candidates` over EVERY key of the pass, asserted through a seam (`resolve_within`'s precedent) |
| 5389 | Nothing says that `decide` must receive verdicts from ONE LEVEL | **must carry, in a TYPE**: an `l2::decide_pair(pair, &a, &b) -> Decision` composing exactly the L2 rules, so no caller builds a vector; residue (a caller can still call `cascade::decide`) stated as a tripwire |
| 5404 | RE-OWNED — the twinning of `l2_corpus()` and `corpus_pairs()` | pose again or answer; do not leave it naming 6.12 after merge |
| 5420 | `verdict_for_hostname`'s evidence is NOT de-duplicated | §1, with `write_link`'s first-verdict evidence |
| 5483 | `EntityState` and `EntityKind` are not rows in `every_variant_of_a_navigated_enum_is_listed_in_all` | one row per enum; cheap — take it if the story touches either enum |
| 6316 | An L2 side's SCOPE and the hostname-agreement reading are one decision | §0.4(D); owes a sentence |
| 6324 | An FQDN and a short label are two spellings of one name | not live (one name source, dot stripped at `reverse_dns.rs:137`, `sanitise`): re-own to the second hostname source |
| 6392 | An invisible character INSIDE a real name still produces a false `Opposes` | not live through the shipped connector (`reverse_dns::sanitise`); re-own likewise |
| 6421 | No L2 rule can ever be NAMED in the presence of any L1 `Disqualifying` | **must carry**: the `l2::decide_pair` type above IS the guarantee |
| 6435 | EPIC 6's ENGINE SPINE HAS STOPPED COUNTING ITS RUN | owner is the retrospective; this story's §6 states the count (six) |

**Rows owned by "Epic 6" whose trigger IS this story** (found by the fact-check; contexting's table listed
only rows naming "6.12"):

| line | row | reading |
|---|---|---|
| 2511 | `write_link` keeps the FIRST verdict's evidence only | due now — §0.5's evidence finding, AC3 |
| 2517 | Nothing fills `guard_decision`'s `candidates_for_link` | due under (A2) |
| 2630 | the comparison is blind to `link_candidate` | due under (A2); AC5's snapshot must compare candidates |
| 2472 | Two concurrent passes can mint two interfaces for one L1 key | a SECOND registration of #161 — reconcile with `:3068` |
| 2236 | No `entity` supertype / `device` table | stale since story 6.5 — close it |

**To register (raised by the validation):** 5.14b's tripwire `the_production_pass_produces_no_ambiguous_abstention`
runs over a slice with **no L2 pair** and reads `identity_link` only, so it can never red on an L2
ambiguity and **6.13's AC cannot be met through it** — owner 6.13 (or 6.14): a new slice with two
interfaces and a shared name, read from wherever L2 decisions live.

**Plus the two Epic 5 races the epic names**:

- **The interface mint race** (`deferred-work.md:3068`, issue **#161**, OPEN). The memory of PR #163
  records that it **reproduces** with OS threads on a `Barrier` (8 interfaces for one MAC, 14 rounds of
  20); issue #161's BODY still says *"it does not reproduce"*, and a COMMENT on the issue carries the
  correction — the body, not the issue, is stale. A `UNIQUE` on `interface_l1_key` is
  **refused by D21** (a cloned MAC is two interfaces) and was withdrawn once. Reachability: `spawn_scan_loop`
  runs one pass at a time, so it needs two instances on one store. ⚠️ **A device mint written as
  read-then-insert reproduces the same shape one level up**; if the device id is derived rather than
  looked up, the race does not arise there. **Recommendation: name #161's owner as the FR6 scheduler
  story with the reachability stated, and design the device mint so it has no read-then-insert window.**
- **`current_subject IS NOT NULL` ≠ `valid_to = OPEN_END`** (`deferred-work.md:3091`, owner unassigned).
  A NULL makes `identity_link_current_subject` evaluate to UNKNOWN, which a `CHECK` accepts. Tightening it
  reds exactly one test whose doc is wrong. **If (B) creates a sibling table, it must be born without the
  defect** — write the coupling with no NULL comparison, e.g. `(valid_to = OPEN_END) = (current_subject IS
  NOT NULL)` plus the value check — **and must carry a raw-SQL probe proving the UNKNOWN row is refused**
  (story 5.9's M3 lesson: a guard the adapter cannot violate is reachable only around it).

---

## 3. Acceptance criteria — for Guy's arbitration of 2026-09-24 (§0.6)

**AC1 — one L2 decision per candidate pair, from L2 verdicts ONLY, in D13's order.** The pass calls
`l2_candidates` ONCE over every key `join` produced, then `l2::decide_pair` per proposed pair. The
guarantee that no L1 verdict enters the vector is carried by `l2::decide_pair`'s signature (it takes a
pair and two sides and composes the three L2 rules itself); the residue — `cascade::decide` stays callable
with any vector — is stated as a tripwire on story 5.12's precedent, never as a barrier.

**AC2 — the universe is the blocker's, and falsifiably so.** A `resolve_within`-style seam takes the L2
universe from the caller; a test hands it a narrowed universe **withholding a pair that would otherwise be
PERSISTED** (the `obelix` pair, not an `AbsenceOfProof` one) and asserts it gets no row, with a control
asserting the same pair IS persisted under the full universe. 🔴 *The validation's M1 measured the naive
version green: the withheld pair concluded `AbsenceOfProof`, which is never persisted.*

**AC3 — every persisted decision names its rule and its evidence.** 🔴 *Its evidence half is SUPERSEDED by
decision F* (Guy, 2026-09-25, at the code review, which found it unimplemented and unannounced — the blind and
acceptance layers independently): the table stores the pair and the verdict vector, no evidence column, and
story 6.14 reads the names from the two interfaces' current observations at display time. What follows is
the criterion as contexting wrote it, kept so the divergence can be read against it. `NoMatch` rows carry a non-empty
`rule_id`; the evidence is the de-duplicated **UNION** over the vector's verdicts, taken ONCE (in
`l2::decide_pair` or in the writer, not both — register rows 2511 and 5420). **`l2-virtual-mac-prefix`
reads no observation**, so its evidence is stated to be the pair's keys (or empty, by design) rather than
asserted non-empty; an `Ambiguous` row carries its candidates (A2). **Never** a struct-literal `Decision`.
A test asserts `obelix`'s persisted evidence is NON-empty — the first-verdict rule gives empty.

**AC4 — `obelix`'s shape, driven through the pass, persists as ONE `Ambiguous` with two candidates** —
two interfaces, two observations each carrying the same hostname, distinct non-IANA MACs — **and the
sweep's L1 links commit with it.** 🔴 The second half is the one that matters: a variant of the pass that
routes the L2 write through today's `guard_decision(&[])` must RED this test, by rolling back L1.

**AC5 — idempotence ACROSS SWEEPS, and purge-and-replay at L2.** A premise assertion that the first pass
wrote **at least one** L2 row (🔴 *M2: a writer persisting nothing left the naive test green*); then a
second pass over the **same network with FRESH observation ids** — what the shipped connector produces —
writes nothing (F1). Purging engine L2 rows (a purge of their own; `purge_engine_links` covers
`identity_link` only) and replaying reproduces every decision-bearing column, **candidates included**
(register row 2630). Three NICs sharing one name persist without a uniqueness collision (E's measurement).

**AC5b — a pair is closed only when both its interfaces were in the slice (D).** A test where one
interface of a persisted `Ambiguous` pair is absent from the next sweep leaves the row current; a control
where both are present and the pair now concludes `AbsenceOfProof` closes it, with no successor (G).

**AC6 — the new table is born without the UNKNOWN defect**, proven by raw-SQL inserts the schema refuses
by constraint name: `(valid_to = OPEN_END, current_subject = NULL)` for the COUPLING, and a row whose VALUE
check would compare a NULL — so the value check is written with `COALESCE`, never a bare equality against
a nullable column.

**AC7 — the two Epic 5 races are carried or re-owned by name, with reachability measured, not supposed**
(§2). A race claimed "not reproducible" needs repetition before it needs more concurrency (PR #163's
lesson) — and a negative result from an instrument that cannot open the window measures the instrument.

**AC8 — no screen changes, and the story says so in its §6, with the run counted.** `count_engine_reach`
and every render of its four consumers — `/triage`, `/dashboard`, the reconciliation card and
**`/diagnostic`** — stay byte-identical in output over the existing fixtures (under (B) this is structural;
under a widened `identity_link` it is the story's heaviest criterion).

**AC9 — the documents this story falsifies are corrected in place**: `resolver.rs`'s *"Not wired"* doc and
its blanket `allow`; `l2.rs`/`identity/mod.rs`'s *"Story 6.12 is the first such caller"* sentences once
true; `write_link`'s evidence doc.

## 4. Tasks (for the dev agent, after arbitration)

- [x] T0 — read §0, and §0.6 above all: the arbitration is recorded, and it narrows the epic's letter.
- [x] T1 — `l2::decide_pair` in `opencmdb-core/src/identity/l2.rs` (composes the three rules; returns
      `Decision` via `cascade::decide`; evidence decision per §1). `float-free` then walks the same 5 files.
      ⚠️ Story numbers in assertion MESSAGES red `float-free` (`6.12` is digit-dot-digit) — write them in
      doc comments.
- [x] T2 — migration `0012_*` (re-runnable: `IF NOT EXISTS`, `0010`'s conditional idiom for any `ALTER`;
      recovery recipe in the header on `0006`'s model); binary collations (D64); `CHECK`s named.
- [x] T3 — `device_repo.rs` *(shipped as `l2_repo.rs`: it holds no device, and a name promising one
      would be the first false sentence of the module)*: insert / load-current / close for the L2 table, `DB_TEST_LOCK` in every DB
      test, raw-SQL probes for each CHECK.
- [x] T4 — the L2 pass in the resolver (or a sibling `l2_pass.rs` if `resolver.rs` grows past comfort),
      inside the same `transact`, after L1, reusing `join`'s groups as sides.
- [x] T5 — AC4's `obelix` test, AC5's replay, AC6's probe, AC2's seam.
- [x] T5b — the twenty `DELETE FROM interface` test cleanups: give the new child table its own purge and
      clean it first at every site (or extract ONE shared cleanup helper — decide and say which).
- [x] T6 — prove-to-red: predictions written to a file BEFORE the first run; carriers **derived by
      grepping the mutated token**, never recalled; `cargo xtask mutate --baseline` on a VIRGIN database of
      the story's own.
- [x] T7 — register: answer / re-own / close every §2 row; add what the story raises; `cargo xtask record`.
- [x] T8 — docs-current-before-push: `CLAUDE.md`, `docs/project-context.md`, the story's Change Log and
      §6. **No manual sentence is owed unless a screen changes.**

### Review Findings (code review 2026-09-25 — three isolated layers, one database each)

Blind layer (diff only): 16 findings. Edge layer (built and mutated): 7 findings + 6 refuted suspicions. Acceptance
layer (re-ran suite, gates, record, every mutation log — all conform): 9 findings. **25 distinct after merging; 0
dismissed.** Reached by two layers independently: AC3's evidence (blind+audit), the OPERATOR row (blind+edge), the
quadratic cost (blind+audit+edge — all three), AC4's unmeasured red (blind+audit), AC7 (blind+audit), the
stale sprint-status lines (blind+audit).

- [x] [Review][Decision→Patch] ✅ **Guy, 2026-09-25: AC3 is declared SUPERSEDED by decision F** — no evidence column; 6.14 reads the names from the two interfaces' CURRENT observations at display time (cost stated: today's names, not those at decision time); the divergence is written in §3 and §8. AC3's evidence half is not implemented, and the record never says so — the table stores the verdict vector and no evidence; no test asserts `obelix`'s evidence non-empty; decision F ("evidence keyed on INTERFACES") was re-read as "the pair is the evidence's subject" without saying so; and 6.14 must show "candidates and their evidence" from this table, which holds none (blind 1, audit 2).
- [x] [Review][Decision→Patch] ✅ **Guy, 2026-09-25: the pass SKIPS a pair holding a current OPERATOR row** (D14: a human's row is an input the engine neither adopts nor supersedes), counted in `L2Resolution`, pinned by a test that the row survives, the sweep commits and L1 links are written. A current OPERATOR row in a pair's slot rolls back EVERY sweep, L1 included — measured (edge P1: `Err(Constraint("unique"))`, links 2→2), and the `decided_by = 'ENGINE'` filter causing it is carried by no test (mutation green). Story 5.10's mutual-exclusion finding, reproduced in the sibling table, one story before 6.14 writes operator rows (blind 3, edge 2).
- [x] [Review][Patch] The L2 pass is O(n²) round trips inside the sweep's transaction — at 300 interfaces +2.8 s per sweep (edge P3) and 10–50× the reference-scale test (audit: 139–382 ms → 3.7–9.8 s); load the sweep's current rows in ONE query, and record the measured pass time [l2_pass.rs:107]
- [x] [Review][Patch] The pair-ordering swap is carried by no test — `if true` left 777+219+110 green; add the cross-sweep case (higher key minted first) [l2_pass.rs:124]
- [x] [Review][Patch] The supersede branch (persisted → different persisted) is exercised by nothing — drive it with a stored row carrying other verdicts [l2_pass.rs:144]
- [x] [Review][Patch] AC4's "must RED" half was never measured — run the guard-routed variant as a mutation [l2_pass.rs:157]
- [x] [Review][Patch] AC2's seam fixture has one pair, so withholding it is an EMPTY universe — withhold one pair of two [l2_pass.rs tests]
- [x] [Review][Patch] "The uplink narrowing is CARRIED" overstates — the fixtures carry no uplink, and the count is taken before the loop; add an uplink-bearing population and narrow the claim [deferred-work.md, blocking.rs:280]
- [x] [Review][Patch] Two guard branches (key absent from `groups`, key placed on no interface) are driven by no test [l2_pass.rs:108-121]
- [x] [Review][Patch] `InstantRegressed` fires on the changed branch only; `judge`'s doc promises it for any earlier re-judgement [l2_pass.rs:70]
- [x] [Review][Patch] Refusing a `Match` rolls the whole sweep back, the shape the module rejects for `guard_decision` — say that the loudness is meant to land in CI, where the first `Decisive` rule's own tests reach it, and why that differs [l2_repo.rs:61, l2_pass.rs:15]
- [x] [Review][Patch] `l2.rs` doc comments still name 6.12 as owner of rows it re-owned (`:160`, `:238`, `:285`, `:386`)
- [x] [Review][Patch] Vacuous "the refusal is never shown without the two interfaces" — nothing shows it at all [l2.rs:557]
- [x] [Review][Patch] sprint-status keeps "NOT developable yet" and "where `obelix` stops being two rows" under 6.12's key [sprint-status.yaml]
- [x] [Review][Patch] "An unchanged network writes nothing" holds for a deterministic connector only — a flapping PTR closes and reopens; qualify it in the story and both twins
- [x] [Review][Patch] M6's needle half overstated — in the rows the tests insert the value CHECK is TRUE, so the unbounded needle could not have missed; "a real defect" → "latent" [story §8, twins]
- [x] [Review][Patch] M4's 21 pre-existing reds have TWO carriers (the adapter refusal and `0012`'s CHECK) and need a store [story §8]
- [x] [Review][Patch] AC8 is carried by one reach-count assertion plus a structural argument, not by comparing renders — say so [story §8]
- [x] [Review][Patch] AC6's letter says COALESCE; `0012` uses `IS NULL OR`, equivalent and unannounced [story §8]
- [x] [Review][Patch] Mutation ids skip M8 with no word [story §8]
- [x] [Review][Patch] AC7 reachability "stated" — cite the code that establishes one pass at a time, and name an owner for `identity_link`'s UNKNOWN row as criterion 3 requires [deferred-work.md]
- [x] [Review][Patch] An L2 side is the whole `join` group, including observations L1 declined to place under a narrowed universe — say so [l2_pass.rs:114]
- [x] [Review][Defer] One L2 transaction can write thousands of rows for a same-name cluster (100 × `espressif` → 4950 rows, against the ~1000-row cap) [l2_pass.rs] — deferred, the cap's "never split a decision" must first be read at pair level
- [x] [Review][Defer] `verdicts VARCHAR(512)` has no length guard; a longer vector rolls the sweep back under strict mode [0012] — deferred, ~100 chars today
- [x] [Review][Defer] One observation carrying two MACs is the strongest co-location signal and L2 treats it as weakly as two sightings [l2.rs] — deferred to "what makes a merge at L2"

## 5. What this story must NOT do

- **Not edit `epics.md`.** §0.2's false promise and §0.4(A) are Guy's.
- **Not compose L1 and L2 verdicts in one vector** — fatal totally, not per case (`deferred-work.md:6421`).
- **Not narrow the L2 blocker at its call site** (`deferred-work.md:5274`); the corpus is blind to it.
- **Not write an `Ambiguous` through `guard_decision(&[])` in the pass's transaction** (§0.2).
- **Not add a `UNIQUE` on `interface_l1_key`** — D21, and it was built and withdrawn once.
- **Not vacate a membership for an interface the slice did not carry** (5.14's measurement).
- **Not claim the trap gate moves or NFR4/J4 close** — 6.9's algebra finding stands; the gate stays RED
  26/15/11.
- **Not use `git checkout -- <file>` to revert a mutation on a dirty tree** — five destructions recorded.
- **Not put a new adapter in `repo.rs`** (1789/2000).

## 5b. Previous-story intelligence (6.9, 6.11)

- **Derive a mutation's carriers by grepping the mutated token.** Four divergences in two stories came
  from enumerating by memory; the three mechanically derived rows all conformed.
- **`cargo fmt` reflows lines**, so an anchor quoting a wrapped line misses — anchor on a single physical
  line just read.
- **A register row is identified by its FIRST bold title**; a re-ownership goes BESIDE the title.
- **Check whether the register already predicted a finding** before publishing it as a discovery (6.11's
  `float-free` red was registered by 5.4b).
- **Each review layer and each measuring process gets its own database**; the waiter's needle must match
  the driver's SHOUTED failure text; a watcher must be proven to emit before its silence means anything
  (`gh pr view --json statusCheckRollup`).
- **`Status:`, the Change Log and the criteria are the regions `cargo xtask record` does not read** —
  keep them current by hand; three stories in a row missed one.

## 5c. Git intelligence

Last five commits: `6b3c04e` (6.11 status act), `60d07f5` (6.11), `8b36828` (6.9 status act), `5b85d93`
(6.9), `97e95dd` (Epic 6 unfrozen and reordered). All engine-spine work since the reorder touched
`opencmdb-core/src/identity/l2.rs`, `opencmdb-bin/src/fixtures.rs` and the register; **none touched
`resolver.rs`, `scan_pass.rs` or a migration** — this story is the first since `0011` (story 14.6) to add
DDL, and the first since 5.14 to change what the shipped binary does per sweep.

## 5d. Latest technical information

No new dependency is needed or wanted. Stack pinned by `Cargo.lock` (sqlx `=0.9.0`, MariaDB-only). MariaDB
10.11 facts this story leans on, all measured earlier in this repository rather than looked up: DDL is not
transactional (`0006`, `0010`); a generated column coalescing to a literal cannot be indexed (ERROR 1901,
5.9); NULLs are distinct in a `UNIQUE` (5.9); `ascii_bin` is PAD SPACE (6.5); a `CHECK` evaluating to
UNKNOWN passes (5.14).

## 6. What the operator gains

**Nothing visible** — verified rather than asserted: no route, no screen, no template, and no view reads
`l2_pair_decision` (`an_l2_decision_changes_nothing_the_reach_section_counts` pins the one query four
screens share). **The run of engine-spine stories giving the operator nothing is SIX** — 6.5, 6.6, 6.7,
6.9, 6.11, 6.12 — and the register row that counts it says so.

🔑 **What is different about this one is that it WRITES on a real network.** From the first sweep after
it ships, `obelix`'s two NICs persist as ONE `Ambiguous` pair, and any VRRP address on the LAN as
`NoMatch` pairs naming `l2-virtual-mac-prefix`. The store holds the question; story 6.14, which follows
directly by Guy's decision, is where the operator sees it and lifts it. ⚠️ **And it writes no device, on
any input** — criterion 1's *"writes `device` rows"* is not met, by decision, and registered.

## 7. Change Log

| when | what |
|---|---|
| 2026-09-24 | contexted on `6b3c04e`; three findings falsifying the queued premise (§0.1–§0.3); four decisions posed to Guy (§0.4) |
| 2026-09-24 | validated by two fresh-context layers, each with its own database — **§0.1–§0.3 survive both**; the fact-check refuted my xtask count (110, not 111), restored criterion 3's dropped clauses, added five register rows and a fourth screen; the gap-hunt BUILT (A2)+(B) and found four HIGHs against it (§0.5): the grain, cross-sweep churn, two guards that cannot fail, and the tripwire promise; three decisions added (E–G) |
| 2026-09-24 | **Guy's arbitration: all seven decisions on the recommendation** (§0.6); `obelix` confirmed two NICs; 6.14 to follow directly; `epics.md` to be corrected by a planning act |
| 2026-09-24 | the planning act merged (PR #210, `ddbc5f0`); this branch rebased onto it |
| 2026-09-24 | implemented: `l2::decide_pair`, migration `0012`, `l2_repo.rs`, `l2_pass.rs`, the pass wired after L1 in `resolve_within`; `resolver.rs`'s false *"Not wired"* doc and blanket `allow(dead_code)` removed |
| 2026-09-24 | mutation pass: nine rows, eight conforming; **M6 contradicted** (2 red for 1) and exposed an unbounded constraint-name needle, bounded |
| 2026-09-24 | register: nineteen owned rows answered, re-owned or closed in place; four new rows |
| 2026-09-25 | **code review, three isolated layers, one database each**: 25 distinct findings, 0 dismissed; the quadratic cost reached by ALL THREE layers; two decisions by Guy (AC3 superseded by F; an operator's pair left alone) |
| 2026-09-25 | repair: 22 patches — one batch read per sweep (3.7–9.8 s → 314–359 ms), operator pairs skipped, five new tests, docs narrowed; M11, M12b, M13 conforming, M12 refused by the driver on a two-site anchor; three rows deferred, two decisions registered. **782 + 219 + 110** with `RUSTFLAGS="-D warnings"` on a VIRGIN store (bin 25.55 s) and without one (bin 5.05 s); fmt, clippy `--all-targets`, ten gates |

## 8. Dev Agent Record

### What was built, and the decisions the dev took (each reversible, each mine)

- **`l2::decide_pair(pair, a, b)`** (`opencmdb-core`) — the three L2 rules composed in one fixed order,
  `CURRENT_RULESET_VERSION` reused (the L2 rules change no L1 decision, so no bump). Five unit tests.
- **Migration `0012_l2_pair_decision.sql`** — keyed on `(interface_low, interface_high, is_current)`, where
  `is_current` is `1`/NULL so closed versions drop out of the key (the `current_subject` idiom); outcomes
  `no_match`/`abstained` only, cause `ambiguous` only; `verdicts` holds `rule=verdict;…` and no ids.
  🔑 **The coupling CHECK compares two never-NULL expressions** — `(valid_to = OPEN_END) = (is_current IS
  NOT NULL)` — so it cannot evaluate to UNKNOWN. ⚠️ **Mine: the interface FKs are `ON DELETE CASCADE`**,
  so the twenty `DELETE FROM interface` test cleanups (T5b) needed no edit; production never deletes an
  interface, and no row here means anything once one is gone. `link_candidate` chose RESTRICT; the
  difference is stated in the migration.
- **`l2_repo.rs`** — not `device_repo.rs` (it holds no device). The writer refuses `Match`
  (`l2_match_not_persisted`) and `AbsenceOfProof` by name before the schema does; `close_l2_decision`
  refuses the sentinel and an unknown or already-closed row.
- **`l2_pass.rs`** — `judge` / `judge_within` (the seam, AC2). Per pair: no row + persisted → insert;
  same decision → unchanged; different and persisted → close + append; now `AbsenceOfProof` → close with
  no successor; an earlier instant than the current version → `InstantRegressed`. ⚠️ **Mine: the instant a
  decision is reached is the LATEST `observed_at` of the two sides' observations** — derived, never the
  clock.
- **`resolve_within`** records the interface each key landed on and calls `judge` after the L1 tail, in the
  same transaction; `Resolution` gains an `l2` field.
- **AC4's guard variant** (routing the L2 write through `guard_decision(&[])`) was **not re-run as a
  mutation here**: `guard_decision` is private to `resolver.rs`, and the validation's gap-hunt measured that
  exact composition rolling back every L1 link. The test that would red is
  `obelix_persists_one_ambiguous_pair_and_the_sweeps_l1_links_commit_with_it` (it asserts both the commit
  and the L1 links) — **stated as reasoned, not measured on this tree.**

### Mutation pass — predictions written first (`scratchpad/mut-6-12/predictions.txt`), carriers derived by grep

`cargo xtask mutate --baseline` on a store of the story's own (`story_6_12`, port 13419); M6 on a VIRGIN
store with no baseline, since a migration edit cannot share a store with its unmutated baseline.

| id | mutation | predicted | measured | carriers |
|---|---|---|---|---|
| M1 | `decide_pair` drops `l2-hostname-agrees` (the only `Supports`) | red:12 | ✅ red 12 | 2 core (`decide_pair` tests) + 10 `l2_pass` tests |
| M2 | `judge`'s universe narrowed to its first pair (6.6's call-site narrowing) | red:5 | ✅ red 5 | three_nics, vrrp, absence, production_universe, purge |
| M3 | `judge_within` ignores the universe it was handed | red:1 | ✅ red 1 | `only_the_pairs_the_blocker_proposed_are_judged` |
| M4 | `is_persisted`: `AbsenceOfProof` → persisted | red | ✅ red **26** | 15 resolver, 3 fault_injection, 3 l2_pass, 2 l2_repo, 1 page, 1 scan_pass, 1 sighting_repo — every multi-interface sweep in the suite |
| M5 | `carries()` stops comparing `verdicts` | red:1 | ✅ red 1 | `each_decision_bearing_column_is_compared` |
| M6 | `0012`'s coupling in `identity_link`'s UNKNOWN-prone form | red:1 | 🔴 **red 2** | + `a_marker_other_than_one_is_refused` — see below |
| M7 | the writer's ordering refusal dropped | red:1 | ✅ red 1 | `the_writer_refuses_what_the_table_would_refuse` |
| M9 | a decayed pair treated as unchanged (never vacated) | red:2 | ✅ red 2 | decays, past |
| M10 | the instant-regression guard neutered | red:1 | ✅ red 1 | `a_pair_rejudged_in_the_past_is_refused` |
| M11 | *(review)* AC4's variant — every `Ambiguous` write refused as `guard_decision(&[])` would, in the sweep's transaction | red | ✅ red **13** | exactly the thirteen `l2_pass` tests whose sweep writes an `Ambiguous` — **no pre-existing test produces one**; `obelix_persists_…` among them, ⚠️ `.expect("the sweep must commit")`-carried rather than assertion-carried |
| M12 | *(review)* the pair-ordering swap neutered | red:1 | ⛔ **refused** | `ANCHOR MATCHED 2 TIMES` — the seam test repeats the line; replacing both would have changed the test's own oracle. A result, not a failure |
| M12b | *(review)* the same, re-anchored on the `else` branch | red:1 | ✅ red 1 | `a_pair_whose_higher_key_was_minted_first_is_persisted` — the edge layer had measured this mutation GREEN before the test existed |
| M13 | *(review)* operator rows filtered out of the batch read again | red:1 | ✅ red 1 | `a_pair_an_operator_decided_is_left_to_the_operator` |

⚠️ **There is no M8.** The contexting pass numbered its candidates, and M8 (a side-swap in the pass) was dropped
before predictions were written, when `swapping_the_sides_changes_no_conclusion` showed the swap cannot change
a conclusion. The gap was left unexplained until the acceptance layer asked.

🔴 **M6 is the finding, and it is two findings.** *(a)* My prediction listed the test ABOUT the coupling,
not every test inserting a row the coupling judges: with `is_current = 2` on a current row the mutated
coupling refuses FIRST, so the marker test met a different constraint name. **The fifth named instance of
enumerating carriers by what a test is about rather than by what reads the mutated token** — derived by
grep for Rust tokens and by hand for a CHECK, and the hand is where it failed. *(b)* Reading why exposed a
LATENT defect: `refused_by` matched the constraint name as an **unbounded substring**, and
`l2_pair_decision_current` is a substring of `l2_pair_decision_current_value`. ⚠️ *First written here as "a
real defect … the coupling tests could not tell the two refusals apart", and the acceptance layer refuted
the second half*: in the two rows the coupling test inserts, the value CHECK is TRUE, so no existing test
could have been fooled. Bounded anyway, because the next row someone adds would be. The bounding's own red
was not measured by a mutation.

⚠️ **M4's reds have TWO carriers** — the adapter's refusal and `0012`'s CHECK — and appear only against a
store: without `DATABASE_URL` those tests return early. So M4 measures the pair, never either alone (blind
review layer). ✅ **Its 26 is still a measurement worth keeping**: the suite carries 21 PRE-EXISTING tests (26 minus this story's five)
whose sweeps the L2 pass now judges, and every one of them fails loudly if `AbsenceOfProof` is ever persisted — the
decision G is carried far beyond this story's own tests.

### Verification (story branch, 2026-09-24)

- At development: **777 + 219 + 110**; after the review's repair: **782 + 219 + 110** (five new `l2_pass`
  tests: the cross-sweep pair, the supersede branch, the operator's pair, the uplink population, the two
  guard branches — the seam test rewritten in place). Re-measured on a VIRGIN store and without one, the
  clock the tell; the figures are in the Change Log row of the repair.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` green; `cargo fmt --all` clean;
  `cargo xtask ci` **ten gates green** (`file-size` largest 1954; `float-free` still 5 files; `ddl-collation`
  accepts `0012`); `cargo doc`: no new warning (the same twenty-five as `master`).
- **Browser gates not run**: no template, asset or route changed. ⚠️ **AC8's *"byte-identical renders of
  four screens"* is carried by a PROXY** — one `count_engine_reach` assertion on one fixture plus the
  structural fact that no view reads `l2_pair_decision` — and not by comparing renders. Stated at the code
  review rather than claimed.
- ⚠️ **AC6's letter says `COALESCE`; `0012` writes the value check as `is_current IS NULL OR is_current = 1`**,
  which is equivalent (`TRUE OR NULL` is `TRUE`) and was unannounced until the acceptance layer asked.
- 🔴 **The pass time, measured after the review's repair** (`one_full_pass_at_the_reference_scale`, 300
  interfaces, 44 850 pairs, virgin store, two runs): **cold 314–359 ms, idempotent rerun 281–287 ms**. The
  first version read each pair's row separately and the review measured it at **3.7–9.8 s** (edge: +2.8 s
  per sweep at 300 interfaces); `master` measures 382 ms cold and 139 ms rerun, so L2 now costs ~150 ms of
  CPU per rerun and no per-pair round trip. *The record carried no pass time at all until the review; story
  5.9b's had one.*

### File List

- `crates/opencmdb-core/src/identity/l2.rs` — `decide_pair` + five tests; module doc corrected
- `crates/opencmdb-core/src/identity/mod.rs`, `cascade.rs`, `blocking.rs` — sentences 6.12 falsified
- `crates/opencmdb-bin/migrations/0012_l2_pair_decision.sql` — new
- `crates/opencmdb-bin/src/l2_repo.rs` — new, with fourteen tests
- `crates/opencmdb-bin/src/l2_pass.rs` — new, with seventeen tests (twelve at development, five added by the review)
- `crates/opencmdb-bin/src/resolver.rs` — the L2 call, `Resolution::l2`, docs corrected, blanket `allow` removed
- `crates/opencmdb-bin/src/repo.rs` — `DecidedBy::token` made `pub(crate)`
- `crates/opencmdb-bin/src/main.rs` — two `mod` lines
- `_bmad-output/implementation-artifacts/deferred-work.md` — nineteen rows answered/re-owned/closed, four new
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/6-12-resolver-writes-device-groupings.md`
- `CLAUDE.md`, `docs/project-context.md`

## Record

- live-count: bin=782 core=219 xtask=110
- base: ddbc5f0d75d2eea87b34b57aefc156c4e978f93d
- registered: Story 6.12's first criterion
- registered: Story 5.14b's tripwire `the_production_pass_produces_no_ambiguous_abstention` can never red
- registered: Two interfaces seen only in DIFFERENT sweeps are never paired
- registered: A flapping reverse-DNS answer churns L2 history
- registered: One L2 transaction can write thousands of rows
- registered: `l2_pair_decision.verdicts` is `VARCHAR(512)` with no length guard
- registered: One observation carrying two MACs is the strongest co-location evidence
- registered: Story 6.14 must show *"candidates and their evidence"*
- registered: A pair holding a current OPERATOR row is left to the operator
- file: CLAUDE.md
- file: docs/project-context.md
- file: _bmad-output/implementation-artifacts/6-12-resolver-writes-device-groupings.md
- file: _bmad-output/implementation-artifacts/deferred-work.md
- file: _bmad-output/implementation-artifacts/sprint-status.yaml
- file: crates/opencmdb-bin/migrations/0012_l2_pair_decision.sql
- file: crates/opencmdb-bin/src/l2_pass.rs
- file: crates/opencmdb-bin/src/l2_repo.rs
- file: crates/opencmdb-bin/src/main.rs
- file: crates/opencmdb-bin/src/repo.rs
- file: crates/opencmdb-bin/src/resolver.rs
- file: crates/opencmdb-core/src/identity/blocking.rs
- file: crates/opencmdb-core/src/identity/cascade.rs
- file: crates/opencmdb-core/src/identity/l2.rs
- file: crates/opencmdb-core/src/identity/mod.rs

## References

- `_bmad-output/planning-artifacts/epics.md:1966-1982` (story), `:1860-1891` (the reorder note)
- `_bmad-output/planning-artifacts/architecture.md` D12 `:909`, D13 `:959`, D14 `:1068`, D16, D17, `:1486`, `:1524`
- `crates/opencmdb-core/src/identity/cascade.rs:501-566`; `l2.rs:1-84, 239, 399, 574`; `blocking.rs:285`
- `crates/opencmdb-bin/src/resolver.rs:97-103, 246-397, 522-603, 712-727`; `scan_pass.rs:178`; `repo.rs:1425, 1692-1790`
- `crates/opencmdb-bin/migrations/0002`, `0003`, `0004`, `0006`
- `_bmad-output/implementation-artifacts/deferred-work.md` (rows in §2); issues #158, #159, #161, #162
- Previous story: `6-11-virtual-mac-anchor-is-not-a-rule.md`
