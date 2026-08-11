# Story 5.14: Wire the identity pass into the shipped binary — and measure what it cannot say

Status: ready-for-dev

<!-- ✅ VALIDATED 2026-08-11 by two fresh-context agents. **The gap-hunt BUILT the whole story** —
     the read, the wiring, the display, the bans, the AC7 assertion — reached 503 tests, then ran
     the REAL BINARY against a REAL MariaDB. What it found there is why this story is no longer the
     story it was.

     🔴 **SPLIT AT VALIDATION (Guy, arbitration 4): 5.14 WIRES AND MEASURES; 5.14b INSERTED,
     the DISPLAY.** Epic 5 → **20 stories**. The reason is measured, not structural taste:
     **wiring the pass replaces one structural zero with another**, and a display shipped on top of
     it would have been a number that looks like reach and measures uptime.

     🔴 **THE TWO MEASUREMENTS THAT SPLIT THE STORY.** (a) `arp_ping.rs:177` emits
     `vec![Fact::IpV4, Fact::Rtt]` and declares `Capabilities { kinds: {IpV4, Rtt} }` — **no MAC,
     ever** — and `join` keys on `(L2DomainId, MacAddr)`, so **every scanned observation abstains**
     and the *evaluated* half is a permanent zero. (b) Each scan mints fresh `obs_id`s, so each scan
     writes a NEW current abstention link and **nothing supersedes anything**: five runs over ONE
     host gave `current engine links = 1, 2, 3, 4, 5`. At a five-minute interval that one host reads
     ~105 000 after a year.

     ⚠️ **This story does NOT display anything.** It measures both zeros, pins them with tests that
     RED when they stop being true, and hands the denominator question to 5.14b — because deciding
     what an unplaceable observation counts AS is a grouping question, and grouping is Epic 6's. -->

## Story

As the operator,
I want the identity pass to actually run in the product I deploy,
So that what the engine knows — and what it cannot know — is a fact about my network rather than a
property of the test suite.

**And as the next developer, I want to be told what this wiring does NOT buy**, in tests rather than
in prose, so that the day a connector emits a MAC or a scan stops accumulating, something reds and
names what changed.

**What this story does NOT do:**

- it does **not** display anything. No template, no locale key, no view change — **5.14b owns the
  surface**, and a change under `templates/` in this diff is a FINDING;
- it does **not** decide what the abstention counter's population IS. §5 measures that a count over
  links measures uptime; choosing the denominator is a grouping decision and belongs with 5.14b and
  Epic 6;
- it does **not** implement an `l2-*` rule, and the trap gate stays **`passed() == false`** with 11
  unanswerable;
- it does **not** change BEHAVIOUR in `opencmdb-core` (the clause is scoped to behaviour on purpose
  — 5.13b measured that a bare *"does not touch X"* becomes a reason not to look at X);
- it does **not** touch the ARP/ping connector. §4a is a MEASUREMENT of what it emits, not a request
  to change it: giving it a MAC is a connector story with its own privilege questions (ARP needs the
  neighbour table, which is not free);
- it does **not** add a dependency, and it does **not** edit `epics.md` (§8 registers instead).

---

## 1. 🔴 Guy's arbitrations

| # | when | question | decision |
|---|---|---|---|
| 1 | contexting | the pass is not wired; the counter would read zero forever | **5.14 wires it.** A counter that cannot fall is decoration (D18). |
| 2 | contexting | `epics.md`'s `Ambiguous` AC is unreachable | **Re-owned to Epic 6 with the unreachability ASSERTED** → moved to **5.14b** with the display. |
| 3 | contexting | ~22 registered entries name 5.14 | **Answer the conditions one by one**, CLOSED / RE-OWNED with the measurement. Split between the two stories by §7. |
| 4 | **validation** | wiring replaces one structural zero with another, and the counter AGES | 🔴 **SPLIT. 5.14 wires and MEASURES; 5.14b (INSERTED) displays.** Epic 5 → 20 stories. Deciding the denominator under the pressure of a screen to ship would settle in a display story what Epic 6 exists to decide. |
| 5 | validation | the section inherits the declared side's visibility gate | **Hoist it out** — the reach counter is a property of the OBSERVED and owes the declared nothing. **5.14b's**, with the AC that forbade touching story 3.8's template amended and the reason written. |
| 6 | validation | AC1's guard is unsatisfiable: deleting the whole wiring leaves 282 tests green | **The seam IS the helper, decided and WRITTEN** — with the mutation that proves the last link is carried by nothing, rather than a guard that implies it had one. |

## 2. What the validation established, so no one re-derives it

Measured by the layer that BUILT the story, in its own worktree, against `mariadb:10.11.11`:

- **`resolve` runs from the startup path and writes rows.** Real binary,
  `OPENCMDB_SCAN_CIDR=127.0.0.1/32`: `links_written=1 abstentions=1 interfaces_minted=0`;
- **`InstantRegressed` and `ContradictoryObservation` are UNREACHABLE from a scan slice** — fresh
  `Uuid::now_v7()` per observation, one `observed_at` per poll, five binary runs, no refusal. ⚠️ §5
  of the first draft called them *"the two refusals a real network can produce"*; **that was false**;
- **the transaction menu the first draft offered does not exist.** The ingest is already
  `for observation { repo.transact(…) }` — **one transaction per observation**. "The same unit as the
  observation writes" would have to be created. Two units is the only shape the current code offers;
- **the boundary IS observable**: a mutation reds `a_refused_pass_does_not_take_the_observations_down`.

## 3. The wiring

`resolver::resolve(conn, observations)` (`resolver.rs:207`) is idempotent (5.11), order-independent
(5.11b), and refuses a regressing instant and a contradictory `obs_id`. Do not widen its signature.

**Extract an `ingest_and_resolve` helper out of `spawn_startup_scan`** and test it. `main.rs:172-268`
is a `std::thread::spawn` with no join handle whose body is inseparable from `ArpPingConnector::poll`
(an ICMP socket), so a test cannot reach the thread.

🔴 **And SAY what that costs.** Arbitration 6: the helper is the seam, and **the last link — the call
site inside `spawn_startup_scan` — is carried by NOTHING**. The mutation is prescribed in §6 and its
GREEN is the finding, recorded rather than hidden. The house already has this idiom: *"a guard that
cannot have a mutation says so instead of implying it had one"* (`fixture_connector.rs`, story 5.1's
review).

**Two transaction units** — the observations' and the pass's. D34 §2: *"everything emitted before it
is still true"*, and FR11 makes an observation immutable and independently true. A refused pass must
not take the sweep's observations down with it, and that is AC2's assertion.

## 4. 🔴 The two structural zeros — this story's real deliverable

### 4a. The only connector `main.rs` reaches emits NO MAC, ever

`arp_ping.rs:177` emits `vec![Fact::IpV4, Fact::Rtt]` and declares
`Capabilities { kinds: {IpV4, Rtt} }`. `identity::l1::join` keys on `(L2DomainId, MacAddr)`. So
**every observation the shipped product produces falls to the abstention path**, and
`interfaces_minted` is 0 not by accident but by construction.

⚠️ **This is not a defect in the connector.** ARP/ping without `NET_RAW` is a ping sweep; reading the
neighbour table for a MAC is a privilege question a connector story owns. What is a defect is
*claiming a reach counter measures reach* while the only producer cannot produce the signal identity
needs.

### 4b. The population ACCUMULATES: the count measures uptime, not reach

Each scan mints fresh `obs_id`s, so each scan writes a NEW current abstention link and nothing
supersedes anything. Measured: five runs of the binary over ONE host →
`current engine links = 1, 2, 3, 4, 5`.

⚠️ **The `current_subject IS NOT NULL` filter does NOT stop this** — the first draft believed it did.
The population is OBSERVATIONS, and observations accumulate. A count over links therefore violates
the UX ban it is meant to serve (*"six months of inaction and it still reads 113"*) on the first day.

🔑 **Why this story does not fix it.** An abstained observation has no interface — that is what
abstention MEANS — so collapsing sightings of one unplaceable thing requires deciding what makes two
sightings the same thing without an identity. **That is grouping, and grouping is Epic 6.** Choosing
a denominator here would settle in a wiring story what the next epic exists to decide.

## 5. What must be PINNED, not merely written

Both zeros get a test that **reds when the fact stops being true**, so the successor is named by a
falling test rather than by a paragraph:

- **the connector's kind set** — assert `ArpPingConnector`'s declared capabilities contain no
  `FactKind::Mac`, with a message naming what changes when it does;
- **`join` over a MAC-less slice yields no interface** — assert on the pass's own `Resolution`
  (`interfaces_minted == 0`, abstentions == the slice length) rather than through a new read. **The
  read is 5.14b's**;
- **the accumulation** — run the pass twice over two slices carrying DIFFERENT `obs_id`s for the same
  address and assert the current-link count is 2, with a message saying that this is the counter's
  denominator problem and naming 5.14b / Epic 6.

⚠️ **The third one asserts a DEFECT, deliberately.** It is not "the desired behaviour"; it is the
measurement that stops 5.14b inheriting the problem silently. Its message must say so, or a later
reader will take it for a specification.

## 6. Prove-to-red

Predictions written first; a result contradicting one is the finding. Carriers read from each panic
message, and the command that carried each red named (`cargo test` does not run the `fixtures` gate).

| id | mutation | predicted |
|---|---|---|
| **M1** | delete the `resolve` call inside `spawn_startup_scan` | 🔴 **GREEN — and that green is arbitration 6's whole point.** Record it; do not "fix" it by asserting through the helper and calling it covered |
| **M2** | delete the `resolve` call inside `ingest_and_resolve` | RED — the helper's own tests |
| **M3** | run the pass in the SAME transaction as the observation writes | RED — `a_refused_pass_does_not_take_the_observations_down` |
| **M4** | give the ARP/ping connector a `Mac` fact | RED — §5's first pin, naming what changed |
| **M5** | make `join` key on something a MAC-less observation carries | RED — §5's second pin (`interfaces_minted` stops being 0) |
| **M6** | make the second pass reuse the first slice's `obs_id`s | RED — §5's third pin stops seeing 2, which is the accumulation measured |
| **M7** | drop `decided_by = 'ENGINE'` from any query this story adds | ⚠️ predicted **GREEN** — the population frontier is undecided. If green, that is a finding to register with 5.14b, not to paper over |

⚠️ **`M7`'s prediction is the one most likely to be wrong**, and it is included for that.

## 7. The register, split between the two stories

Of the **twenty-two** `deferred-work.md` entries naming 5.14 (counted, not estimated), this story
takes the ones the WIRING answers; **5.14b takes the rest** and its §7 lists them.

| entry | disposition here |
|---|---|
| **`:2407` the resolver is NOT wired into `main.rs`** | 🔴 **CLOSED by this story.** ⚠️ Its stated reason for deferral was *"wiring it would make every deployment write links with no page to display them"* — **and after arbitration 4 that is exactly what ships.** The entry must be closed with that tension NAMED, not quietly ticked: the deployment now writes links no page shows, for one story |
| `:2700` `observed_at` stability across passes is a CALLER'S DISCIPLINE | **must be handled** — the scan re-scans, so this is the ordinary path. §4b's accumulation is its consequence |
| `:2772` the `ContradictoryObservation` refusal's REACHABILITY rests on one test | **answered by MEASUREMENT** — unreachable from a scan slice (§2). Record the measurement; the entry survives for the helper's callers |
| `:2391` `count_identity_links` has no production caller | **RE-OWNED to 5.14b** — this story adds no read |
| every other entry | **RE-OWNED to 5.14b**, listed there |

---

## Acceptance Criteria

**AC1 — the pass runs in the shipped binary.**
**Given** the startup scan has ingested its observations
**When** the binary starts with a scan configured
**Then** `resolve` runs over that slice through a tested `ingest_and_resolve` helper, and a failure
is logged at `error!` naming the refusal.
🔴 **And the story STATES that the last link — the call inside `spawn_startup_scan` — is carried by
nothing**, with M1's GREEN as the evidence. Arbitration 6. ⚠️ Do not claim a guard on the startup
path; there is none, and saying so is the deliverable.

**AC2 — two transaction units, and the boundary is asserted.**
A refused pass must not take the sweep's observations down with it.
_Reddened by: M3._

**AC3 — the connector's MAC-lessness is PINNED.**
A test asserts `ArpPingConnector` declares no `FactKind::Mac`, with a message naming what changes the
day it does.
_Reddened by: M4._

**AC4 — the structural zero is PINNED on the pass's own outcome.**
Over a MAC-less slice, `interfaces_minted == 0` and every observation abstains — asserted on
`Resolution`, **not** through a new read (that is 5.14b's).
_Reddened by: M5._

**AC5 — the accumulation is PINNED, and named as a DEFECT.**
Two passes over slices carrying different `obs_id`s for one address leave TWO current links, and the
assertion's message says this is the counter's denominator problem and names 5.14b / Epic 6 — never
reading as a specification.
_Reddened by: M6._

**AC6 — nothing is displayed.**
No file under `templates/`, no locale key, no view builder change. A diff touching them is a FINDING.

**AC7 — the gates and the corpus are untouched.**
`cargo xtask ci`: **28 fixtures**, seven gates green; trap gate still **26 discovered, 15 scored, 11
unanswerable, `passed() == false`**.

**AC8 — the register.**
§7's five dispositions are recorded with their measurements, appended to `deferred-work.md`, never
rewriting a bullet. **`:2407` is closed with its tension named.**

**AC9 — documents in the same commit**, including **the insertion of 5.14b** (Epic 5 → 20 stories).
⚠️ One live count, in one place.

---

## Tasks / Subtasks

- [ ] **T1 — extract and test `ingest_and_resolve`** (AC1, AC2); two transaction units
- [ ] **T2 — the three pins** (AC3, AC4, AC5), the third with its "this is a defect" message
- [ ] **T3 — prove-to-red** (AC1, AC2): M1–M7, predictions first, M1's GREEN recorded as the finding
- [ ] **T4 — the register** (AC8): §7's five entries with their measurements
- [ ] **T5 — gates and documents** (AC7, AC9): the four commands; the twins; 5.14b inserted

---

## Dev Notes

### Traps, each measured on this project

- ⚠️ **DB-heavy.** Your OWN `mariadb:10.11.11` on your OWN port — layers sharing a schema fabricate a
  symptom indistinguishable from open issue #38. 13306/13307/13308/13311/13312 are taken;
- ⚠️ **DB tests take `crate::DB_TEST_LOCK`** (`main.rs:41`);
- ⚠️ **`cargo test --workspace A B` passes two filters where cargo accepts one** — nothing runs and it
  reports 0 red for a sound mutation;
- ⚠️ **Commit before the mutation pass, and revert the MUTATION, never the FILE**;
- ⚠️ **Do not read a measurement through a truncation** — 5.13b recorded a divergence that never
  happened because `head -8` hid its own evidence;
- ⚠️ **sqlx 0.9 refuses a `format!`ed SQL string** (`SqlSafeStr` is `&'static str` only); test cleanup
  needs literals or `AssertSqlSafe`.

### The tree this story extends, to be RE-MEASURED

`master` at `6ceb284`: **494 tests** (273 bin + 159 core + 62 xtask), seven gates green, 28 fixtures,
26 traps across ten families, trap gate RED.

### References

- [Source: `epics.md#Story 5.14`] — **not edited**; its display clauses are 5.14b's
- [Source: `crates/opencmdb-bin/src/arp_ping.rs:177`] — the MAC-less fact set
- [Source: `crates/opencmdb-bin/src/main.rs:130-135`, `:172-268`, `:244-257`, `:41`]
- [Source: `crates/opencmdb-bin/src/resolver.rs:207`] — `resolve`
- [Source: `crates/opencmdb-core/src/identity/l1.rs`] — `join`'s key
- [Source: `deferred-work.md:2407`, `:2700`, `:2772`, `:2391`]

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

---

## Change Log

| date | what |
|---|---|
| 2026-08-11 | Created as the display story; **three arbitrations at contexting**. |
| 2026-08-11 | **VALIDATED, and SPLIT at validation (arbitration 4).** The gap-hunt built the whole story and ran the real binary against a real database: **wiring replaces one structural zero with another** — the only connector `main.rs` reaches emits no MAC, ever, so the *evaluated* half is permanently 0 — **and the counter AGES**, five runs over one host giving five current links, which falsifies the UX ban it exists to serve. 🔴 **5.14 becomes the WIRING and the MEASUREMENT; 5.14b is INSERTED for the display.** Epic 5 → 20 stories. Two further arbitrations: the section is hoisted out of the declared side's visibility gate (5.14b's), and the guard seam IS the helper, **written with the mutation that proves the last link is carried by nothing** rather than implied. |
