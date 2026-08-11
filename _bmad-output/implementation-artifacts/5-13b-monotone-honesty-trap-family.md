# Story 5.13b: The blinded-source family — the faulted shape becomes committed, reviewable bytes

Status: review

<!-- ✅ VALIDATED 2026-08-11 by two fresh-context agents (fact-check + gap-hunt). **The gap-hunt
     BUILT the story** in its own worktree — the three artefacts, both guards, the count sweep, the
     MANIFEST bump and thirteen mutations — reaching 492 tests / 28 fixtures / seven gates green, then
     left `master` untouched. **6 HIGH from each layer; 2 arbitrations by Guy.** The story below is
     the corrected one.

     🔴 **THE HEADLINE, and only ONE layer found it: `bdbdbdbd` — the `obs_id` prefix the first draft
     prescribed — is RESERVED, and the four UUIDs the draft wrote are byte-identical to committed
     ones** in `fixtures/scenario/wire/unifi-clients.expected.jsonl`, whose README says so in as many
     words. The draft's measurement was right and its CONCLUSION was false: it enumerated
     `scenario/replay/` and concluded "free" of the whole tree. ⚠️ **No gate would have caught it** —
     the cross-stream walk covers `scenario/replay/` only, `wire/` sits outside every corpus walk on
     purpose, and M10, the mutation written to prove that anchor live, is blind to it. The layer that
     BUILT the story did not see it, precisely because nothing reds. *A green build is not a
     uniqueness proof when the walk cannot see the file.*

     🔴 **FIVE findings were reached INDEPENDENTLY by both layers**, one by reading and one by
     running: §6's count table wrong AND incomplete; `fault_injection.rs` missing from it entirely;
     `fixture_connector.rs`'s context table covered by no task; M2's carrier mispredicted; and AC5's
     "green without the pin" refuted.

     🔴 **AC3's strictness half is structurally unreachable for a blinding, and the apparatus said so
     before the story was written** — `fault_injection.rs`'s module doc, lines 21-28. Guy's
     arbitration: **route it to the CUT**, which the clean twin already joins for free (§7a).

     ⚠️ **`DATABASE_URL` is unset, no database is needed, and both layers confirmed it.** Do not
     start a container. -->

## Story

As the operator whose sources go half-blind mid-sweep,
I want the faulted shape to live in the corpus as committed, reviewable bytes — and the identity
decision to be judged on it by a trap family with both of D18's poles,
So that degradation is part of the spec the engine is measured against, not a construction local to
one test file.

**This story's centre of gravity is under `fixtures/`, and that is what makes it different from
5.13.** 5.13 derived the faulted run in memory and touched no byte of the corpus; this one commits
three artefacts, bumps the lock, and moves counts in **five** source files plus a hand-maintained
data table.

**What this story does NOT do:**

- it does **not** claim lattice monotonicity (D36). It supplies a PRECONDITION and stops there — and
  §9 states exactly how narrow that precondition is, because a validation layer showed the first
  draft over-sold it;
- it does **not** implement an `l2-*` rule. The trap gate must stay **`passed() == false`** with
  **11 unanswerable**; a green trap gate is a FINDING (D18: *"a gate that cannot fall is
  decoration"*);
- it does **not** change the identity engine. `identity::{l1,blocking,cascade}` are untouched — a
  change there in the diff is a FINDING;
- it does **not** touch `opencmdb-core`;
- it does **not** weaken `no_obs_id_is_shared_across_replay_streams` (`fixtures.rs:1858`). Guy's
  arbitration 4 chose the design that keeps that anchor intact — §5;
- it does **not** add a dependency, and it does **not** wire the resolver into `main.rs` (5.14's);
- it does **not** regenerate `architecture-views.md` (issue #50 — never inside a story);
- it does **not** edit `epics.md`. §10 registers the corrections instead.

---

## 1. What this story inherits, and from where

| inherited | from | state on arrival |
|---|---|---|
| the committed family + the `MANIFEST` bump | `epics.md`'s story 5.13, clauses 2 and 3 | never implementable as a *monotone-honesty* trap — §3b |
| AC3's **inverse direction** | 5.13's arbitration 4 (validation, 2026-08-10) | measured a no-op on both committed streams; §3a shows it is a no-op on **every** committed stream |
| the **capability-snapshot** half of D36 | 5.13 §7, *"re-owned, not discharged"* | precondition unmet: **11 of 11** trap-named streams carry no `capability` record (re-verified independently) |
| *"the correction is registered with **5.13b as owner**"* | 5.13's contexting record | discharged into Epic 5's retrospective (§10) |

## 2. 🔴 Guy's arbitrations

| # | when | question | decision |
|---|---|---|---|
| 1 | contexting | what does 5.13b commit? | **The pair of streams AND the trap family** — clean twin + blinded twin + two traps (both poles) + the `MANIFEST` bump + the count sweep. `epics.md`'s clause in full. |
| 2 | contexting | the inverse direction, measured impossible by construction? | **Requalified**: not *"derive the clean run by removing the control record"* but *"the committed blinded twin **is** the committed clean twin, blinded"* — carried by a guard, never by two files trusted to stay in step. |
| 3 | contexting | D36's capability-snapshot half? | **Stays REGISTERED, not discharged** (§9). |
| 4 | contexting | how does the twin pair survive the corpus-wide `obs_id` anchor? | **Distinct `obs_id`s on the two committed twins; the STRICT claim stays on the in-memory derivation.** §5. This *reverses the literal form of arbitration 2*, on a fact found after it was taken; both are recorded rather than one quietly overwritten. |
| 5 | validation | AC3's strictness half, structurally unreachable for a blinding? | **Routed to the CUT**, which the clean twin already joins for free in 5.13's inherited sweep. AC3 keeps 5.13's pair — each half on the mutilation that can carry it (§7a). |
| 6 | validation | the count sweep is ~4× wider than the draft claimed — split the story? | **NO, one story.** The sweep is one deliverable's mechanical consequence, not a second deliverable; and the halves are separable only on paper, since adding the artefacts reds a dozen assertions at once. §6 is rewritten as a METHOD instead of a list. |

## 3. 🔴 Three inherited clauses, each refuted by code

Every claim names the file and line that establishes it. **Both validation layers re-established
these independently and both confirmed them** — but re-verify rather than quoting: line numbers are
the first thing an unrelated commit invalidates.

### 3a. The inverse direction is a no-op for **every** committed file

5.13 measured `clean = 8 facts, faulted = 8 facts` on `partial-then-failed.jsonl` and on
`capability-downgrade.jsonl` and moved the clause here, expecting 5.13b to commit *"streams capable
of carrying it"*. **No such stream can exist**, for two independent reasons, both verified twice:

- **a `failure` record must be LAST in a committed file.** `read_records` calls
  `reject_if_after_terminal` on all three record kinds (`fixtures.rs:568`, `:573`, `:585`; the
  refusal at `:610`, its message at `:419`). There is no suppressed tail to restore;
- **a `capability` record's strip is BAKED INTO the committed observations.** `from_records`
  validates each observation against the descriptor in force **at its own position**
  (`fixture_connector.rs:250`, `UndeclaredFactKind`), so a committed stream whose tail still carried
  the denied kind would refuse to LOAD; and the capability arm (`:207-211`) only reassigns
  `in_force`. Removing the record restores no fact.

⚠️ **The generalisation the first draft drew from this is FALSE, and the counterexample is in the
code it cites.** The draft wrote *"removing a control record can only ever WIDEN what is
permitted"* and labelled it *"the transferable half"*. `fixture_connector.rs:208-209` says the
opposite in its own comment — *"An **UPGRADE is legal**, and so is an empty `kinds` set … Nothing
here compares the new descriptor to the old one"* — so a stream whose initial descriptor denies a
kind, carrying a widening `capability` record followed by an observation using that kind, **LOSES**
its permission when the record is removed. **Removing a control record can NARROW.** The two
specific claims above stand; only the sentence that generalised them was wrong, and it was the one
the draft was proudest of.

### 3b. A monotone-honesty TRAP still cannot exist — 5.13's finding 1c, re-verified

`Expectation` (`trap.rs:69`) has exactly three variants, all about **one merge decision on one
stream**; `incomplete_families` reads only `MustMerge`/`MustNotMerge` (`trap.rs:421-425`), so a
`must-abstain` counts for **neither** pole. There is no projection of *"the faulted run's facts are
a subset"* — a relation between two RUNS — onto any of them.

**So `epics.md`'s *"positive AND negative form"* cannot be honoured for the subset claim.** What the
family does instead is judge the identity decision **on the blinded bytes**.

⚠️ **State that weakly, because the draft over-stated it.** The draft said the traps *"assert that
losing `Rtt` changes no L1 verdict"*. They do not: **both traps judge stream B only**, nothing in
the corpus judges the same pair on stream A, so no trap makes a before/after comparison. And
`keys_of` (`l1.rs:128-136`) reads `Fact::Mac` alone, so `Rtt` is irrelevant to L1 **by
construction**, not by measurement. What the two traps really assert is that `l1-exact-mac` and
`l1-distinct-mac` fire correctly on a stream that carries a capability record — full stop. That is
worth having and it is smaller than the draft claimed.

### 3c. `capability-downgrade.jsonl` cannot carry a complete family, so a new stream is forced

Read from the committed bytes: obs 1 carries `02:00:5e:00:55:01`, obs 4 carries `02:00:5e:00:55:04`
— **distinct**; obs 2 and obs 3 carry **no `Mac` fact at all**. No `must-merge` pole is
constructible there. A new stream is forced, not preferred.

## 4. What is committed — the three artefacts

🔴 **READ THIS BEFORE THE TABLES.** The first draft prescribed the `obs_id` prefix `bdbdbdbd` on a
measurement scoped to `scenario/replay/`, and **`bdbdbdbd` is RESERVED by
`fixtures/scenario/wire/`**, whose `README.md:51-52` says so verbatim: *"The `bdbdbdbd` obs_id prefix
is RESERVED by this directory — the cross-stream uniqueness walk covers `scenario/replay/` only and
cannot see these files."* The four UUIDs the draft wrote are **byte-identical** to
`unifi-clients.expected.jsonl` lines 1-4, which `fixtures.rs:4386` pins.

⚠️ **No gate would have caught it.** `no_obs_id_is_shared_across_replay_streams`
(`fixtures.rs:1858`) walks `scenario/replay/` only, `fixtures.rs:1063-1067` states that the wire
artefact *"sits outside every corpus walk on purpose"*, and **M10 — the mutation this story writes
to prove that anchor live — is blind to it too.** The layer that BUILT the story reached 492 green
tests without noticing.

🔑 **The transferable rule: verify a prefix against the whole `fixtures/` tree and against
`crates/`, never against the walk.** The check is
`rg -n '<prefix>' fixtures/ crates/` and it must return nothing.

### Stream A — `fixtures/scenario/replay/blinded-source.jsonl` (the CLEAN twin, no control record)

`obs_id` prefix **`dbdbdbdb`** — measured free by the rule above, `rg` over `fixtures/` **and**
`crates/` returning nothing. Re-run that check before writing; do not trust this line.

| # | `obs_id` | `observed_at` | facts |
|---|---|---|---|
| 1 | `dbdbdbdb-0000-4000-8000-000000000001` | `2026-04-01T00:00:00Z` | Mac `02:00:5e:00:56:01`, IpV4 `203.0.113.40`, Rtt 5 |
| 2 | `dbdbdbdb-…0002` | `…00:00:05Z` | Mac `02:00:5e:00:56:02`, IpV4 `203.0.113.41`, Rtt 7 |
| 3 | `dbdbdbdb-…0003` | `…00:00:10Z` | Mac `02:00:5e:00:56:01`, IpV4 `203.0.113.42`, Rtt 9 |
| 4 | `dbdbdbdb-…0004` | `…00:00:15Z` | Mac `02:00:5e:00:56:02`, IpV4 `203.0.113.43`, Rtt 11 |

- **MAC `02:00:5e:00:56:xx` is free** — verified over `fixtures/` and `crates/`, no hits. ⚠️ The
  draft's enumeration of what the corpus *does* use was itself incomplete (it missed
  `02:00:5e:00:60:xx`, carried by `Uplink.peer_mac` in five streams); the conclusion survives because
  it rests on the `rg`, not on the enumeration. **That is the same defect as the prefix, caught
  before it bit** — an enumeration cannot establish absence;
- **`203.0.113.0/24` is unused corpus-wide** and is admitted by `assert_documentation_ip`
  (`fixtures.rs:1445-1448`, reached from `assert_facts_are_synthetic` at `:1402`);
- one `connector_id` and one `scope` for both twins. **Sharing a `connector_id` is precedented** —
  `33333333-…` appears in 11 of 13 committed streams, and only `obs_id` is checked across streams.

### Stream B — `fixtures/scenario/replay/blinded-source-blinded.jsonl` (the BLINDED twin)

`obs_id` prefix **`bebebebe`** — verified free over `fixtures/` and `crates/` by both layers.
Identical to stream A **in every field but `obs_id`**, plus the capability record at index 2 and the
strip it forces:

```
bebebebe-…0001   Mac 02:00:5e:00:56:01, IpV4 203.0.113.40, Rtt 5
bebebebe-…0002   Mac 02:00:5e:00:56:02, IpV4 203.0.113.41, Rtt 7
{"record":"capability","as_of":"2026-04-01T00:00:07Z","kinds":["Mac","IpV4"]}
bebebebe-…0003   Mac 02:00:5e:00:56:01, IpV4 203.0.113.42          ← Rtt stripped
bebebebe-…0004   Mac 02:00:5e:00:56:02, IpV4 203.0.113.43          ← Rtt stripped
```

- **`kinds = {Mac, IpV4}` denies `Rtt`, which the tail carries** — the non-degeneracy
  `blind_after`'s own doc warns about (`fault_injection.rs:136-144`), asserted via
  `denied_kinds_present_after` (`:177`), never assumed. The JSON spelling `["Mac","IpV4"]` is what a
  `BTreeSet<FactKind>` emits — verified against `capability-downgrade.jsonl` line 3;
- **`as_of = 00:00:07Z` is legal** (`≥ max(observed_at) = 00:00:05Z` over the prefix, no preceding
  capability record). ⚠️ **`earliest_legal_as_of(A, 2)` returns `00:00:05Z`, NOT `00:00:07Z`** — so a
  twin guard that rebuilds the derivation with `earliest_legal_as_of` instead of the committed
  literal reds spuriously. Use the literal; `earliest_legal_as_of` gives the BOUND, not the value;
- measured: **12 facts clean, 10 blinded, 4 observations each side.**

### Artefact C — `fixtures/scenario/traps/blinded-source.toml`, judging **stream B**

| id | observations | expect |
|---|---|---|
| `blinded-source-must-merge` | `bebebebe-…0001`, `bebebebe-…0003` | `must-merge = { rule = "l1-exact-mac" }` |
| `blinded-source-must-not-merge` | `bebebebe-…0001`, `bebebebe-…0004` | `must-not-merge = { rule = "l1-distinct-mac" }` |

Both carry `family = "blinded-source"`, so `incomplete_families` stays empty. Both pairs straddle the
capability record. **Both traps were traced by hand and measured to PASS** — `l1-exact-mac`/`Match`
and `l1-distinct-mac`/`NoMatch`.

⚠️ **The `.jsonl` files can carry NO header comment.** `read_records` parses every non-empty line as
JSON, and `every_replay_stream_re_serializes_to_its_committed_bytes` requires every non-blank line to
re-serialise to a record. `rg -n '^\s*#' fixtures/scenario/replay/` returns nothing. **The prose that
explains the pair therefore lives in the trap file's header and in the twin guard's doc** — this
matters, because §5's DRY justification depends on a label existing somewhere.

## 5. 🔴 The twin relation, and why the strict claim does NOT move onto the committed pair

**The constraint that decided this.** `no_obs_id_is_shared_across_replay_streams`
(`fixtures.rs:1858`) refuses any `obs_id` present in two `scenario/replay/` streams — and
`blind_after` **preserves `obs_id`s** (`fault_injection.rs:163`). Two literal twins red it.

Giving them distinct ids breaks the comparison from the other side: 5.13's oracle compares
`Claim::Fact(ObsId, Fact)`, so **every** claim would differ. Normalising by POSITION was refused: it
would blind the oracle to a run substituting a different observation into slot *n*, verbatim the M2b
class 5.13 measured.

| what | over what | asserts |
|---|---|---|
| **the STRICT claim** (inclusion + strictness) | committed stream A vs `blind_after(A, 2, {Mac,IpV4}, as_of)` **in memory** | `obs_id`s equal **by construction**, no normalisation, oracle undiminished |
| **the TWIN guard** | committed stream B vs that same derivation | stream B **is** the derivation, **modulo `obs_id` only** |

### 🔴 5a. "Total and injective" is NOT enough, and that was measured with a control

The first draft required the mapping be *"asserted total and injective — never a positional zip"*.
The gap-hunt built **both** forms and swapped stream B's observations 3 and 4 (both *after* the
capability record, so the stream still loads):

| guard form | result |
|---|---|
| rekey, then `assert_eq!` on the **whole `Vec<Record>`** | **1 red**, the twin guard, assertion-carried |
| **keyed lookup** by mapped `obs_id` — literally the draft's letter, no position anywhere | **271 passed; 0 failed** |

**A guard satisfying the draft's letter is blind to the mutation the draft wrote to catch it.** The
missing property is not injectivity — it is a claim about the SEQUENCE. So:

> **The guard is: state the `obs_id` mapping as a table, assert it total and injective, apply it as
> a REWRITE of the derivation, then `assert_eq!` the whole `Vec<Record>` against the committed
> stream B.** A per-observation lookup, however well keyed, does not assert the order.

🔑 **Why commit stream B at all.** It is the trap-named stream (§9) and the corpus is a SPEC (D56).
The redundancy is **deliberate**, which the house DRY rule permits only when *"a test pins it and a
comment labels it"* — the twin guard is that test, and the label lives in the trap file's header and
the guard's own doc, **not** in a `.jsonl` comment that cannot exist (§4).

## 6. 🔴 The count sweep — a METHOD, because two inventories of it were wrong

The first draft claimed *"eight literal oracles across three files"* and shipped a table that
contradicted its own header. Measured, it is **~31 literals across five files plus a
hand-maintained data table**. Both validation layers found this independently, and the list below is
**still not asserted to be complete**.

⚠️ **The list is a starting point; the RUN is the authority.** Three reasons the draft's method
failed and this one must not:

1. **the draft's own `rg -n '\b24\b|\b23\b|\b13\b'` misses the sharpest sites**, whose literals are
   **11, 39 and 50**;
2. **the failures surface in TWO WAVES** — a failing assertion hides the later ones in the same
   test — so one green-to-red run does not find them all. Run to exhaustion;
3. **`cargo test --workspace` never runs the `fixtures` sha256 gate.** Every fixture edit also needs
   `cargo xtask ci`.

### 6a. The file the draft missed entirely — and it is the file this story writes into

**The clean twin is control-free, so it silently joins story 5.13's own AC3 sweep**, which discovers
its streams through `walk_replay_streams` (`fault_injection.rs:463-479`). The story must say this and
assert it, not inherit it in silence.

| site | today | after |
|---|---|---|
| `fault_injection.rs:1015` `streams.len()` — *"a new one belongs in this sweep"* | 11 | **12** |
| `:1079` `cut_positions` | 39 | **43** |
| `:1084` `blind_positions` | 39 | **43** |
| `:1112` `total_unbounded` | 50 | **55** |
| `:1116` `degenerate` | 11 | **12** |

### 6b. The hand-maintained data table no task covered

`fixture_connector.rs:1509` `committed_stream_contexts()`, checked **in both directions** with
`checked == table.len()` by `every_committed_replay_stream_is_admissible_to_the_connector` (`:1570`).
Its panic names the requirement: *"committed under scenario/replay/ but absent from this test's
context table — a new stream must state its connector_id, scope and capabilities there"*. **Two
entries are needed.** This is also where M1's and M2's real carriers live.

### 6c. The rest, measured

| site | today | after |
|---|---|---|
| `cargo xtask ci` → `fixtures` | 25 | **28** |
| `fixtures.rs:4658` `corpus.traps` | 24 | **26** |
| `fixtures.rs:4636` `corpus.pairs.len()` | 23 | **25** |
| `fixtures.rs:4573` `corpus.required.len()` | 10 | **11** |
| `fixtures.rs:4620` `checked` | 10 | **11** |
| `fixtures.rs:4594` recall | 1000‰ | **unchanged** |
| `l1_runner.rs:390` `expected_answered()` | 13 ids | **15 ids** |
| `l1_runner.rs:468` `answers.len()` | 24 | **26** |
| `l1_runner.rs:484` answered count | 13 | **15** |
| `l1_runner.rs:496` `all.len()` | 24 | **26** |
| `l1_runner.rs:508` `unanswered.len()` | 11 | **unchanged** |
| `l1_runner.rs:890` `checked` — traps naming a rule | 21 | **23** |
| `l1_runner.rs:895` `distinct` — distinct rule ids | 7 | **unchanged** (both traps reuse the two L1 rules) |
| `l1_runner.rs:1062` `files.len()` | 10 | **11** |
| `l1_runner.rs:1071` `ids.len()` | 24 | **26** |
| `trap_gate.rs:646-651` the per-file enumeration comment | **ten** files | **eleven** |
| `trap_gate.rs` `discovered` — `:652`, `:671`, `:689`, `:718`, `:991`, `:1060`, `:1198` | 24 | **26** (**seven** sites, not six) |
| `trap_gate.rs` `scored` — `:723`, `:992`, `:1199`, `:1322` | 13 | **15** |
| `trap_gate.rs:1067` `unaccounted()` | 24 | **26** |
| `trap_gate.rs:1217` / `:1218` / `:1219` `scored_in` | 7 / 6 / 0 | **8 / 7 / 0** |
| `trap_gate.rs:873` the prose *"13 + 11 == 24"* | — | **15 + 11 == 26** |
| two test NAMES carrying counts: `l1_runner.rs:1060` `…twenty_four_distinct_ids_across_ten_files`, `trap_gate.rs:1193` `the_report_line_says_thirteen_scored` | — | renamed |
| `Report::passed()` | **false** | **false** — a green gate is a FINDING |
| `failures()` | 0 | **0** |

### 6d. Dated prose the growth falsifies, which no assertion guards

Comment-only, and therefore invisible to the suite — which is exactly why it rots. At least:
`fixtures.rs:1194` (*"all 13 replay streams"*), `:1025`, `:2292`, `:4509`;
`fixture_connector.rs:351-353`, `:380`, `:1508`, `:1557` (*"eleven of the thirteen"*), `:1627`
(*"14 entries over 13 files"*); `resolver.rs:708`, `:3330`; `l1_runner.rs:379`, `:408`, `:461`;
`trap_gate.rs:165`, `:638`, `:703`. **Sweep for `thirteen`/`13 streams`/`ten … files` rather than
trusting this list.**

## 7. Prove-to-red — the mutation set

Predictions are written **before** running. 🔴 **A result contradicting its prediction is the
finding.** Classify each red by **reading its own panic message**, one at a time.

### 7a. 🔴 AC3's strictness half — Guy's arbitration 5

The draft predicted *"M3 → strictness reds, inclusion stays green"*. **Refuted, twice over:**

- M3 (widen `kinds` so nothing is denied) reds the **non-degeneracy** assertion first; strictness is
  never reached — story 5.13's own assertion-order finding, arriving again;
- the mutation that targets strictness directly (make `blind_after`'s `retain` a no-op) dies at the
  **load**: an unstripped stream cannot load;
- and **`fault_injection.rs`'s module doc said so before this story was written** (lines 21-28):
  *"under M-B strictness is guaranteed before the connector is invoked, and what M-B really measures
  is the INCLUSION half."*

**Guy's arbitration: the strictness half is routed to the CUT, and it is already there for free.**
The clean twin joins 5.13's AC3 sweep (§6a), where `cut_at` keeps the tail and 5.13 measured M1
redding strictness with inclusion green. AC3 therefore keeps 5.13's pair — **each half on the
mutilation that can carry it** — and the story asserts the routing rather than inheriting it in
silence.

### 7b. The table

| id | mutation | predicted |
|---|---|---|
| **M1** | delete the `capability` line from committed stream B | twin guard reds; the stream still **loads** (§3a made executable). Also moves the control-free count 12 → 13 |
| **M2** | put `Rtt` back on stream B's obs 3 | fails to load at the CONNECTOR. ⚠️ **panic-carried, not `Err`-carried** — the draft got this wrong and both layers caught it: `read_records` accepts the line, and the carrier is `every_committed_replay_stream_is_admissible_to_the_connector`'s `unwrap_or_else` panic (§6b), plus the twin guard by assertion |
| **M3** | widen `kinds` so nothing is denied | **non-degeneracy** reds first, not strictness (§7a). Name the ordering |
| **M3b** | make `blind_after`'s `retain` a no-op | dies at `run`'s load panic. The mutation that proves strictness is unreachable for a blinding |
| **M4** | change one IpV4 in committed stream B | twin guard reds — the normalisation is on `obs_id` and nothing else |
| **M5** | swap stream B's observations **3 and 4** (both AFTER the capability record) | the whole-vector guard reds; **the keyed guard stays GREEN** (§5a). ⚠️ Reversing ALL observations instead is CONFOUNDED — it makes the stream inadmissible, so it reds via the load and proves nothing about the mapping |
| **M6** | swap the two traps' `observations` vectors | the pin reds. ⚠️ **And so does the gate, independently** — the draft claimed the suite would be green without the pin, quoting story 5.2b; **that measurement predates story 5.7's producer** and is stale. The pin is a second oracle, not the sole carrier — say the true, weaker thing |
| **M7** | delete `blinded-source-must-not-merge` | `incomplete_families` reds at `trap_gate.rs:1183`. ⚠️ Drowned in count reds — isolate it |
| **M8** | rename the expected rule to `l2-uplink-agrees` | the trap moves to the **unanswerable** bucket, 11 → 12. ⚠️ The draft flagged this as its likeliest wrong prediction; **both layers measured it CORRECT**, so the flag is removed rather than left as unearned caution |
| **M9a** | corrupt one new `sha256` | `cargo xtask ci` reds, EDITED direction |
| **M9b** | omit one new artefact from the MANIFEST | `cargo xtask ci` reds, ORPHAN direction |
| **M10** | give stream B stream A's `obs_id`s | the anchor reds, naming both files. ⚠️ **This mutation does NOT prove the anchor covers the corpus** — it covers `scenario/replay/` only, which is how the draft's `bdbdbdbd` collision would have shipped (§4) |

⚠️ **Every fixture-editing mutation (M1, M2, M4, M5, M6, M7, M10) also reds the `fixtures` sha256
gate**, which `cargo test --workspace` does not run. Say which command carries each red.

## 8. What the validation established, so a dev pass does not re-derive it

Measured by the layer that BUILT the story, in its own worktree, `master` untouched:

- baseline exact: **489** (268 + 159 + 62), seven gates, 25 fixtures — **no drift**;
- after building: **492** tests, **28 fixtures**, seven gates green, `fmt` and
  `clippy --all-targets -D warnings` clean;
- **AC1, AC4, AC6, AC7 measured MET**: 26 discovered, 15 scored, 0 failures, 0 wrong-rule, 11
  unanswerable, `passed() == false`, `scored_in` 8/7/0, recall 1000‰. Both new traps pass;
- **no database is needed** and none was started, by either layer.

⚠️ **T1's *"generate it, do not hand-write it twice"* is only PARTLY executable.** `ControlRecord`
(`fixtures.rs:131`) is private, so a generator cannot render the capability line in the committed
spelling — `serde_json::json!` emits keys alphabetically and
`every_replay_stream_re_serializes_to_its_committed_bytes` reds on the ordering. **The observations
are generated; the one capability line is hand-written.** Say so rather than discovering it.

## 9. D36 — the precondition is supplied, and it is NARROWER than the draft claimed

`deferred-work.md:324` records lattice monotonicity with **Owner: Epic 5**. 5.13 §7 re-owned the
halves: the doubt order to Epic 6, the capability snapshot here.

**What this story supplies:** `blinded-source-blinded.jsonl` is the **first trap-named stream whose
committed bytes carry a `capability` control record** — the count was 11 of 11 without one,
re-verified independently.

⚠️ **And here is the qualification the draft lacked: the trap path is PROVABLY BLIND to that
record.** `read_traps` and `l1_runner::answer_trap` both go through `read_jsonl`, which drops control
records with an exhaustive `Failure | Capability => None` (`fixtures.rs:647-657`). So AC4's *"the
family judges the stream that carries the capability record"* must **never** be read as *"the record
participates in the judgement"*. What is supplied is a property of the committed BYTES, reachable by
a future `ScoredRecord` producer — defensible, and smaller than *"the precondition is met"*.

**Not supplied, per Guy's arbitration 3:** no `ScoredRecord` is produced; the other 24 traps still
name streams with no capability record; the doubt ORDER on `Verdict` does not exist and needs
`Supports`/`Opposes` to have a producer (Epic 6). ***"D36 is now testable"* must never be read as
met.** Append to `deferred-work.md`; **never rewrite a bullet**.

## 10. What is registered rather than fixed

- **`epics.md`'s story 5.13 is not implementable as written** — 5.13's four findings, plus §3a's
  strengthening and its own correction (§3a's generalisation was false). 5.13's contexting named
  **5.13b as owner**; this story discharges that by carrying it into **Epic 5's retrospective**, and
  `epics.md` is **not edited**;
- **the corpus-wide `obs_id` anchor does not cover `fixtures/scenario/wire/`**, and the reservation
  that compensates for it lives in a README rather than in a check. This story hit it and repaired
  by hand (§4). **Register it** — a gate would be the honest closure;
- **the clean twin is judged by no trap**, making three such streams. **Append** to the
  `deferred-work.md` entry, never rewrite it;
- **NFR8 has four assertions**; 5.13 covered one and this story covers none of the remaining three.

---

## Acceptance Criteria

**AC1 — the clean twin is committed and is ordinary.**
**Given** `fixtures/scenario/replay/blinded-source.jsonl` as authored in §4
**When** the corpus-wide walks run
**Then** it parses, carries four observations and no control record, passes
`assert_facts_are_synthetic`, and shares no `obs_id` with any other committed stream.
🔴 **And the `obs_id` prefix, the MAC range and the subnet are each verified free by `rg` over
`fixtures/` AND `crates/` — never against a walk, and never by enumerating what the corpus uses.**
The first draft failed on exactly that and no gate would have caught it (§4).

**AC2 — the blinded twin is committed, loads, and is the clean twin blinded.**
**Given** `blinded-source-blinded.jsonl` as authored in §4
**When** it is compared with `blind_after(read_records(A), 2, {Mac, IpV4}, as_of)`
**Then** the `obs_id` mapping is stated as a table, asserted **total and injective**, applied as a
**rewrite**, and the two whole `Vec<Record>` are compared with a single `assert_eq!`.
🔴 **A per-observation keyed lookup is REFUSED by name**: it satisfies "total and injective" and was
**measured green** under M5 (§5a). The missing property is the SEQUENCE.
_Reddened by: M1, M4, M5._

**AC3 — monotone honesty, each half on the mutilation that can carry it.**
**Given** committed stream A
**When** it is blinded in memory and both runs are polled into a `VecSink`
**Then** the blinded run's claim multiset is **included** in the clean run's, the two runs emit the
**same number of observations** (4 = 4), and the non-degeneracy of `kinds` is **asserted** via
`denied_kinds_present_after` rather than assumed.
**And** the **strictness** half is asserted on the **CUT**, not on the blinding — stream A joins
5.13's AC3 sweep (§6a) and that routing is **asserted, not inherited in silence**.
🔴 **The reason is written in the test's own doc**: under a blinding, strictness holds before the
connector is invoked (`fault_injection.rs`, module doc, lines 21-28), so asserting it there would be
an assertion carried by nothing — the defect this project has caught four times.
_Reddened by: M3 (non-degeneracy), M3b (the load), and 5.13's M1 on the cut._

**AC4 — the trap family, both poles, on the blinded stream.**
**Given** `fixtures/scenario/traps/blinded-source.toml` with the two traps of §4
**When** the committed corpus is walked and scored through `l1_runner`
**Then** both traps are **answered and PASS**, `incomplete_families` is empty, and the gate reports
**26 discovered, 15 scored, 0 failures, 0 wrong-rule, 11 unanswerable, `passed() == false`**.
⚠️ **A green trap gate is a FINDING.**
⚠️ **The doc states that `read_jsonl` DROPS control records** (§9), so *"the family judges the stream
that carries the capability record"* is never read as *"the record participates in the judgement"*.
_Reddened by: M7, M8, M6._

**AC5 — the new traps' bytes are pinned, on story 5.2b's doctrine.**
**Given** the two new traps
**When** the pin test runs
**Then** each trap's `observations` vector, its `Expectation` and its `family` are asserted against
the committed bytes.
🔴 **And the justification is the TRUE one, measured**: swapping the vectors reds the pin **and the
gate independently**, because since story 5.7 the corpus is scored by the real engine and the swap
demands a merge the engine refuses. Story 5.2b's *"green without the pin"* **predates that producer
and is stale**. The pin is a second oracle, not the sole carrier — write the weaker true sentence.
_Reddened by: M6._

**AC6 — the lock is bumped in both directions.**
`MANIFEST.toml` gains three entries with their real `sha256` and a comment each; `cargo xtask ci`
reports **`28 fixture(s)`**, seven gates green. Both directions proven: a corrupted hash reds, an
omitted entry reds as an orphan.
_Reddened by: M9a, M9b._

**AC7 — the count sweep is run to exhaustion, not to a list.**
Every literal §6 names is re-derived with **the measurement that produced it**, and the sweep is
continued **until a full `cargo test --workspace` plus `cargo xtask ci` are green** — because §6's
list is explicitly not asserted complete, the failures surface in two waves, and the `fixtures` gate
is not run by `cargo test`.
**And** the counts that must NOT move are asserted as such: `unanswered.len() == 11`,
`scored_in(MustAbstain) == 0`, `distinct == 7`, recall `1000‰`, `failures() == 0`,
**`passed() == false`**.
**And** §6d's dated prose is swept.

**AC8 — the mutation pass runs and its divergences are findings.**
§7's twelve mutations run; the table is filled with the OBSERVED result, the red count, and **the
carrier of each red read from its own panic message**, naming **which command** carried it (the
`fixtures` gate is not `cargo test`).
⚠️ **Do not write a headline count of reds that the table below it refutes** — that defect has
recurred in three consecutive stories.

**AC9 — the documents that describe this state are updated in the SAME commit.**
`CLAUDE.md`, `docs/project-context.md` and `sprint-status.yaml` carry the outcome, the new corpus
counts, the live test count and §3/§4's findings. ⚠️ **One live count for the story, in one place.**

**AC10 — what is NOT claimed is written down.**
D36 stays **registered, not discharged**, with §9's qualification; NFR8's other three assertions are
untouched; `deferred-work.md` is **appended to**; `epics.md` is **not edited**; §10's three
registrations are carried to Epic 5's retrospective.

---

## Tasks / Subtasks

- [x] **T0 — verify before writing** (AC1): `rg -n 'dbdbdbdb|bebebebe' fixtures/ crates/` and the MAC
      range, over the WHOLE tree. A hit is a FINDING, not a rename
- [x] **T1 — author the three artefacts** (AC1, AC2, AC4)
  - [x] `blinded-source.jsonl`, exactly §4's table
  - [x] `blinded-source-blinded.jsonl` — observations GENERATED from A via `blind_after`, the one
        capability line **hand-written** (§8: `ControlRecord` is private and key order matters)
  - [x] `blinded-source.toml` — two traps, both poles, one family, a checkable `reason` on each, and
        the header prose the `.jsonl` cannot carry (§4)
- [x] **T2 — the twin guard** (AC2), in `fault_injection.rs`'s test module: mapping table, total and
      injective, applied as a rewrite, **one `assert_eq!` over the whole `Vec<Record>`**
- [x] **T3 — monotone honesty** (AC3): inclusion + observation count + non-degeneracy on the
      blinding; the strictness half asserted on the CUT via 5.13's sweep
- [x] **T4 — the family in the gate** (AC4, AC5): the byte pin; the gate's new numbers; the ones that
      must not move
- [x] **T5 — the context table** (AC7): two entries in `fixture_connector.rs:1509`
      `committed_stream_contexts()` — checked in both directions, covered by no task in the draft
- [x] **T6 — the lock and the sweep** (AC6, AC7): three `MANIFEST.toml` entries; sweep to exhaustion
      over both commands, in two waves; §6d's dated prose
- [x] **T7 — prove-to-red** (AC8): M1–M10 plus M3b; predictions written first; every red classified
      by its panic message and by the command that carried it
- [x] **T8 — gates and documents** (AC9, AC10): the four commands; the twins; append to
      `deferred-work.md`; register §10's three items with Epic 5's retrospective; **do not edit
      `epics.md`**

---

## Dev Notes

### Shapes to follow, not reinvent

- **The mutilations are story 5.13's**, all `pub(crate)` in `crates/opencmdb-bin/src/fault_injection.rs`:
  `cut_at`, `blind_after`, `earliest_legal_as_of`, `denied_kinds_present_after`,
  `kinds_denying_something_after`, `StreamContext`, `stream_context`, `run`, `RunOutcome`, `Claim`,
  `multiset_included`, `unaccounted` (all twelve confirmed by validation). Reuse them.
- **The oracle is a MULTISET, on `PartialEq` alone** — measured, not preferred: a set-based oracle is
  GREEN on a run that invents nothing and merely repeats itself. `opencmdb-core` is not touched.
- **`Record` derives `PartialEq, Eq`** (`fixtures.rs:87`), so the twin comparison is one `assert_eq!`.
- **The byte-pin shape is story 5.2b's**, already applied to all 24 committed traps.

### Where the code goes

No new source file. Both guards in `fault_injection.rs`'s trailing `#[cfg(test)] mod tests` (D56b);
the byte pins in `fixtures.rs`'s; the context-table entries in `fixture_connector.rs`'s; the count
updates across the five files of §6.

### Traps in the tooling, each one measured

- ⚠️ **`cargo test --workspace A B` passes two filters where cargo accepts one, so NOTHING runs** and
  it reports 0 red for a sound mutation. One filter, or the full suite;
- ⚠️ **Commit before the mutation pass** — 5.13's driver ran `git checkout -- crates/` and ate the
  uncommitted work mid-pass, caught only because four greens contradicted a prediction;
- ⚠️ **`cargo test` does not run the `fixtures` sha256 gate.** Every fixture edit needs `cargo xtask ci`;
- ⚠️ **No database.** `DATABASE_URL` is unset, DB tests pass by returning, and both validation layers
  confirmed this story needs none. If a test does reach the database it **must** take
  `crate::DB_TEST_LOCK` (`main.rs:41`) — without it the collision is intermittent and
  indistinguishable from open issue #38.

### Gates

```
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask ci
```

`fixtures` must report **28**; `views-hash` reports `ℹ STALE` and exits 0 — **do not regenerate**
(issue #50); `float-free` walks `identity/` only; `declared-authorship` adds nothing here.

### The tree this story extends

`master` at `55a2cbc`: **489 tests** (268 bin + 159 core + 62 xtask, 1 ignored), seven gates green,
25 fixtures, trap gate RED. **Re-measured by both validation layers, no drift** — but re-measure
again rather than quoting.

### What a reviewer will challenge, and the answer that must already be measured

| challenge | the answer |
|---|---|
| *"the prefix is free"* | §4 — the draft's was **not**, and no gate would have caught it. Check the whole tree |
| *"the twin guard is keyed, so order is covered"* | §5a — **measured green** under M5. Whole-vector equality or nothing |
| *"AC3 asserts the pair like 5.13 says"* | §7a — it does, with the strictness half on the CUT, because a blinding's strictness holds before the connector runs |
| *"M6 shows the pin is load-bearing"* | AC5 — the gate reds independently; 5.2b's measurement predates story 5.7 |
| *"the family judges a capability record"* | §9 — `read_jsonl` DROPS control records. The bytes carry it; the judgement does not read it |
| *"§6 lists the counts"* | §6 — the list is explicitly incomplete; the run is the authority |
| *"is D36 met?"* | §9 — **no**, and narrower than the draft said |

### References

- [Source: `epics.md#Story 5.13`] — the two clauses inherited; **not edited**
- [Source: `5-13-monotone-honesty-measurement.md`] — the parent: its §1 findings, §5 mutation table,
  §7 re-owning
- [Source: `architecture.md:2011-2028`] — D35, NFR8's four assertions; [`:2075-2077`] — D36
- [Source: `deferred-work.md:73-78`, `:312-314`, `:324`] — the in-memory asymmetry, the streams
  judged by no trap, lattice monotonicity (*"Owner: Epic 5"*)
- [Source: `fixtures/scenario/wire/README.md:48-52`] — the `bdbdbdbd` reservation, and the statement
  that the cross-stream walk cannot see those files
- [Source: `crates/opencmdb-bin/src/fault_injection.rs`, module doc lines 21-28] — why a blinding's
  strictness is guaranteed before the connector is invoked
- [Source: `crates/opencmdb-bin/src/fixture_connector.rs:1509`/`:1570`] — the context table
- [Source: `crates/opencmdb-bin/src/fixtures.rs:647-657`] — `read_jsonl` drops control records
- [Source: `crates/opencmdb-core/src/trap.rs:69`/`:421-425`] — the three columns, the two poles

---

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context) — `claude-opus-5[1m]`.

### Debug Log References

**Baseline RE-MEASURED on `55a2cbc` before a byte was written**: 489 (268 bin + 159 core + 62
xtask, 1 ignored), seven gates green, 25 fixtures. **No drift** — the story's figure was exact.

**No database.** `DATABASE_URL` unset throughout; this story reaches neither `resolver` nor `repo`.
No container was started, and the prediction that none was needed held.

**The tree was committed (`cea4772`, branch `story-5-13b`) BEFORE the mutation pass**, and
`git status` was verified empty after each revert.

⚠️ Every fixture-editing mutation also reds the `fixtures` sha256 gate, which `cargo test` never
runs — so each row below names the command that carried it.

### Completion Notes List

**489 → 493 tests (272 bin + 159 core + 62 xtask), +4.** Seven gates green, `fixtures` now **28**,
`file-size` 28 files (largest unchanged at 1787). The trap gate is still **RED**: **26 discovered,
15 scored, 0 failures, 0 wrong-rule, 11 unanswerable, `passed() == false`**. `opencmdb-core` was
not touched, no dependency was added, `epics.md` was not edited.

#### The count sweep: ONE wave, not two, and the reason is the validation

§6 warned that the failures surface in two waves. Measured: **twenty tests red in a single wave and
one round of repairs took the suite green.** That is not a refutation of the warning — it is what
the validation bought. The inventory was pre-computed by two layers, so every site was repaired
before any of them could hide another. Had the sweep been driven from the draft's own
`rg '\b24\b|\b23\b|\b13\b'`, the five `fault_injection.rs` sites (literals **11, 39, 50**) would
have formed the second wave exactly as predicted.

**The `fixtures.rs` line numbers §6c cites are the `assert_eq!` ARGUMENT; the panics name the macro
line** — `4573`/`4572`, `4636`/`4635`, `4620`/`4619`. Both point at one assertion; the gap-hunt and
the fact-check each reported one of the two and neither was wrong. Recorded so a reader does not
file it as a defect.

#### 🔑 What the implementation found that neither the story nor its validation had

**The tight initial descriptor was carried by nothing, and now it is — M2b.** The twins are declared
with `blinded_source_caps()` = `{Mac, IpV4, Rtt}` rather than `corpus_caps()`'s seven kinds, on the
argument that a wide descriptor would make the faulted twin's narrowing vacuous. **That argument was
a sentence with no measurement behind it**, exactly the shape six reviews have caught. So: plant a
`Hostname` on the faulted twin's FIRST observation, before the capability record. It reds, and the
refusal names *"the **initial** descriptor"*. **And the CONTROL is what makes it mean something** —
the same planted fact with the twins declared under `corpus_caps()` leaves
`every_committed_replay_stream_is_admissible_to_the_connector` **GREEN**. The descriptor's tightness
is now measured, not asserted.

#### ⚠️ Two mutations whose carrier was NOT what the row predicted

**M7's completeness assertion is unreachable in a full run.** The story predicted
`trap_gate.rs:1183` would red. In the whole suite it never gets there: deleting a pole drops the
trap count, and `discovered == 26` fires first — story 5.13's assertion-order finding, arriving a
third time. Re-run against a SINGLE test, the completeness carrier is
`a_mixed_family_splits_between_the_engine_and_the_bucket`, printing
`IncompleteFamily { family: FamilyId("blinded-source"), has_merge: true, has_not_merge: false }`.
**The row is kept with both measurements**, because *"M7 reds the completeness check"* is true only
under isolation and would otherwise be a claim outrunning its measurement.

**M10's carrier is an explicit `panic!`, not an assertion.** Recorded rather than folded in — the
*"every red is assertion-carried"* headline has now been refuted by review in three separate
stories, and this table will not restate it.

#### The mutation table, with OBSERVED results (AC8)

| id | mutation | predicted | OBSERVED | red | carrier |
|---|---|---|---|---|---|
| **M1** | delete the `capability` line from committed twin B | twin guard reds; the stream still LOADS | ✅ **as predicted** — and the connector test stays GREEN, which is §3a made executable. Bonus red: twin B becomes control-free and enters the sweep, which `the_clean_twin_carries_the_strictness_half_through_the_cut` refuses by name | 4 | assertion ×4 |
| **M2** | put `Rtt` back on twin B's obs 3 | no load; **panic-carried, not `Err`** | ✅ the draft's `Err` was wrong and both layers caught it: `unwrap_or_else` panics naming `Rtt` and the descriptor dated `00:00:07` — the RECORD's, not the initial one | 3 | 2 assertion, 1 panic |
| **M2b** | a `Hostname` on twin B's obs 1, BEFORE the record | — (not prescribed) | ✅ reds, naming *"the **initial** descriptor"* | 3 | 2 assertion, 1 panic |
| **M2b CONTROL** | the same, twins declared under `corpus_caps()` | — (the control) | 🔑 the connector test goes **GREEN** — the tight descriptor is what catches it | — | — |
| **M3** | widen `kinds` so nothing is denied | non-degeneracy reds FIRST, not strictness | ✅ **exactly** — `the blinding must deny something the tail CARRIES` | 2 | assertion |
| **M3b** | `blind_after`'s `retain` a no-op | dies at the LOAD | ✅ `the mutilated stream must still load: … emits a Mac fact, which the descriptor … says the source cannot emit` | 8 | 1 load panic, rest assertion |
| **M4** | change one IpV4 in committed twin B | twin guard reds | ✅ **exactly 1 red**, and it is the twin guard | 1 | assertion |
| **M5** | swap twin B's obs **3 and 4** (both AFTER the record, so it still loads) | whole-vector guard reds | ✅ | 2 | assertion |
| **M5 CONTROL** | the same, guard replaced by a **keyed lookup** | the guard goes green | 🔑 **the twin guard leaves the failure list.** ⚠️ The SUITE does not go green — the byte pin still reds, for its own reason. The claim is about the GUARD | 1 | — |
| **M6** | swap the two traps' `observations` vectors | pin reds AND the gate reds independently | ✅ 4 red, **three of them `trap_gate` tests that never consult the pin** — AC5's true justification, measured | 4 | assertion |
| **M7** | delete `blinded-source-must-not-merge` | `trap_gate.rs:1183` reds | ⚠️ **NOT in a full run** — the count fires first. Isolated, the carrier is `a_mixed_family_splits_between_the_engine_and_the_bucket` | many / 1 isolated | assertion |
| **M8** | rename the expected rule to `l2-uplink-agrees` | routed to the bucket, 11 → 12 | ✅ `26 discovered, **14 scored**, 0 failures, **12 unanswerable**`, the trap explained by its author's rule. The draft's *"likeliest wrong prediction"* flag was unearned and both layers said so | 1+ | assertion |
| **M9a** | corrupt a `sha256` | EDITED direction reds | ✅ `sha256 mismatch (manifest 000000000000… ≠ file 45139d7d388e…)` | gate | `cargo xtask ci` |
| **M9b** | omit an artefact from the MANIFEST | ORPHAN direction reds | ✅ measured **on the real tree** before the entries existed: three `present but absent from MANIFEST.toml (orphan)` findings | gate | `cargo xtask ci` |
| **M10** | give twin B twin A's `obs_id`s | the anchor reds | ✅ names both files and quotes its reason. ⚠️ **`panic!`-carried**, not assertion | 1 | panic |

**Fourteen rows: twelve mutations, two CONTROLS that are GREEN by design.** Carriers were read from
each panic message one at a time; they are **mixed**, and the table says which is which rather than
collapsing them to one label.

#### What is NOT claimed

D36 stays **registered, not discharged**, and §9's qualification is the point: `read_jsonl` drops
control records, so the trap path is provably blind to the capability record this story commits.
What is supplied is a property of the committed BYTES. NFR8's other three assertions are untouched.
The corpus-wide `obs_id` anchor still does not cover `fixtures/scenario/wire/` — registered (§10),
not fixed here.

### File List

- `fixtures/scenario/replay/blinded-source.jsonl` — NEW. The clean twin.
- `fixtures/scenario/replay/blinded-source-blinded.jsonl` — NEW. The faulted twin.
- `fixtures/scenario/traps/blinded-source.toml` — NEW. The family, both poles.
- `fixtures/MANIFEST.toml` — MODIFIED. Three entries, 25 → 28.
- `crates/opencmdb-bin/src/fault_injection.rs` — MODIFIED. The twin guard, AC3's inclusion half,
  the strictness ROUTING, and the sweep's five counts.
- `crates/opencmdb-bin/src/fixtures.rs` — MODIFIED. The byte pin, four counts, dated prose.
- `crates/opencmdb-bin/src/fixture_connector.rs` — MODIFIED. `blinded_source_caps()` and the two
  context-table entries; dated prose.
- `crates/opencmdb-bin/src/l1_runner.rs` — MODIFIED. Seven counts, `expected_answered()`, two test
  names, dated prose.
- `crates/opencmdb-bin/src/trap_gate.rs` — MODIFIED. Eleven counts, the enumeration comment, one
  test name, dated prose.
- `crates/opencmdb-bin/src/resolver.rs` — MODIFIED. Dated prose only.
- `_bmad-output/implementation-artifacts/5-13b-monotone-honesty-trap-family.md` — this file.
- `_bmad-output/implementation-artifacts/sprint-status.yaml`, `deferred-work.md`, `CLAUDE.md`,
  `docs/project-context.md` — the record and the twins (AC9, AC10).

---

## Change Log

| date | what |
|---|---|
| 2026-08-11 | Created. **Four contexting arbitrations by Guy**, the fourth reversing the literal form of the second on a fact of the code; both recorded. Three inherited clauses refuted by code (§3). |
| 2026-08-11 | **IMPLEMENTED → `review`** (`done` is the MERGE's business). **489 → 493 tests** (272 + 159 + 62), seven gates green, **28 fixtures**, **26 traps across ten families**, trap gate still RED, `opencmdb-core` untouched. The count sweep came in **ONE wave, not two** — which is what the validation bought, the inventory having been pre-computed. 🔑 One finding the story did not prescribe: the twins' tight descriptor was justified by a sentence and carried by nothing, closed by **M2b** and its **CONTROL** (the same planted fact under `corpus_caps()` is measured GREEN). ⚠️ Two carriers were not what their row predicted — M7's completeness assertion is unreachable in a full run and had to be isolated, and M10 is `panic!`-carried. **Fourteen rows: twelve mutations, two CONTROLS green by design, carriers MIXED and named row by row.** |
| 2026-08-11 | **VALIDATED** (fact-check + gap-hunt, the second having BUILT the story to 492 tests / 28 fixtures in its own worktree). **6 HIGH per layer, 5 findings reached INDEPENDENTLY by both; 2 further arbitrations by Guy** — AC3's strictness routed to the CUT, and the story kept whole rather than split. 🔴 The headline came from ONE layer: the draft's `obs_id` prefix `bdbdbdbd` is RESERVED and its four UUIDs were byte-identical to committed ones, **invisible to every gate including the story's own M10** — the layer that BUILT the story reached 492 green without seeing it. 🔴 Four of the draft's own claims refuted by measurement: *"total and injective"* is satisfied by a guard measured GREEN on the mutation it exists to catch; AC3's strictness is unreachable for a blinding and the apparatus said so first; AC5's *"green without the pin"* is stale since story 5.7; and §3a's *"transferable"* generalisation has a counterexample in the code it cites — removing a control record can NARROW. §6 rewritten as a METHOD after two inventories of it were wrong. |
