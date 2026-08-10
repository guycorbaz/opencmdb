# Story 5.13: A faulted run cannot invent a fact — the monotone-honesty measurement

Status: review

<!-- ✅ VALIDATED 2026-08-10 by two fresh-context agents (fact-check + gap-hunt).
     **The gap-hunt BUILT the story** — both mutilations, the AC1/AC2 tests, and the whole layer-2
     apparatus against its own live `mariadb:10.11.11` on 13308 — then removed its worktree and
     container and left `master` untouched. **6 HIGH, 5 MEDIUM, 11 LOW applied; 2 arbitrations by
     Guy.** The story below is the corrected one.

     🔴 **THE HEADLINE: M2 — THE ONLY MUTATION NAMED FOR THE INCLUSION HALF, i.e. FOR THE NFR8(a)
     VIOLATION ITSELF — CAME BACK GREEN.** `poll` invents a `Hostname` on every emit → 0 new reds,
     because the mutation is UNCONDITIONAL: it lands on the clean run and the faulted run alike, the
     invented fact enters both sides of `f ⊆ c`, and the subset survives. **The general shape: any
     `poll` mutation that is a pure function of the record being emitted is INVISIBLE to a
     differential test.** The mutation must be FAULT-CONDITIONAL; §5's M2 is now the measured form
     that reds (2 tests, assertion-carried).

     🔴 **AC3's inverse direction was a measured NO-OP on both committed streams** — `clean = 8
     facts, faulted = 8 facts` on each — **and BOTH layers found it independently**, one by reading
     and one by running. **Guy's arbitration: it moves to 5.13b**, the story that commits streams
     capable of carrying it.

     🔴 **The oracle was the wrong STRUCTURE, and the reason is story 5.11b's own measured defect.**
     `Fact` derives neither `Ord` nor `Hash`, so `BTreeSet<(ObsId, Fact)>` does not compile. The
     obvious fix is two derives in `opencmdb-core` — but a SET would silently swallow `[x]` against
     `[x, x]`, which is verbatim the hole 5.11b measured on `contradicts`' `facts.len()` term.
     **Guy's arbitration: a MULTISET, inclusion by removal, on `PartialEq` alone.** No core change,
     and it is the right structure on the merits rather than a workaround.

     🔑 **WHAT HELD, also measured, and it is most of the story**: AC5 is NOT vacuous (2 abstention
     rows, `absence_of_proof`); AC4(a) IS reddenable (M8, assertion-carried); `interface_id` IS
     literally comparable (`minted=0 found=1`); a new `fault_injection.rs` passes **all seven
     gates** with clippy clean; and **M1 and M3 behave exactly as predicted** — M1 reds the
     strictness while the inclusion stays green, which is the two-halves reading the story asks for.

     ⚠️ **`DB_TEST_LOCK` (`main.rs:40`) is MANDATORY on both layer-2 tests.** Without it they collide
     on `ERROR 1062` — **intermittently**, which is indistinguishable from issue #38, the one thing
     this project cannot afford to add noise to. This is the first story to run the engine over the
     CORPUS's `obs_id`s, which two of its own tests share.
     ⚠️ **Commit before the mutation pass** — the driver's first act is `git checkout -- crates/`.
     🔑 `DATABASE_URL` is unset here and DB tests pass by `return`ing. §9 has the `docker run`; port
     **13308** (13306 is 5.11b's, 13307 is 5.12's). -->

## Story

As the operator whose network is scanned by sources that sometimes fail mid-sweep or go half-blind,
I want a faulted run to be able to REMOVE knowledge and never to ADD any,
So that degradation is honest rather than creative (NFR8's first falsifiable assertion, D35(a)).

**This story is a MEASUREMENT, and it ships almost no production code.** What it must not do is ship
a measurement that cannot fail — the defect six consecutive code reviews have caught, the one story
5.11b's review found *inside* 5.11b, and **the one this story's own validation found three times
inside this story**.

**What this story does NOT do:**

- it does **not** add, edit or remove a single byte under `fixtures/`. **No new stream, no new trap
  file, no `MANIFEST.toml` bump, no re-hash.** That is 5.13b's (§1, §11). **A `fixtures` gate
  reporting anything other than `25 fixture(s)` is a FINDING** — measured green with a new
  `fault_injection.rs` in the tree;
- it does **not** change the engine. `identity::{l1,blocking,cascade}` are untouched **in what
  ships** — a change there in the diff is a FINDING. A temporary mutation during T7 is not (§5);
- it does **not** touch `opencmdb-core` **at all**. §2's second arbitration is what makes that
  possible, and it is a design decision rather than an avoidance (§3b);
- it does **not** implement an `l2-*` rule. The trap corpus stays **24 discovered, 13 scored, 11
  unanswerable, `passed() == false`**. **If the gate turns green, that is a FINDING** — measured
  unaffected;
- it does **not** wire the resolver into `main.rs` (still 5.14's);
- it does **not** add a dependency;
- it does **not** claim lattice monotonicity (D36). §7 is why, and where that claim is registered.

---

## What this story inherits, measured rather than assumed

### 1. 🔴 Four findings from contexting — why `epics.md`'s 5.13 is not implementable as written

Each is established by code in the tree, and **all four were independently re-verified by the
fact-check layer**.

**1a. A trap cannot name the observation a fault REMOVED.** `read_traps` cross-checks every trap's
`obs_id`s against the replay stream it names and refuses the file with
`FixtureError::DanglingObservation` (`fixtures.rs:694`). The removal *is* the assertion.

**1b. The shape D35(a) needs is FORBIDDEN in a committed file and legal only in memory.**
`epics.md`'s story 4.5a carries two acceptance criteria that contradict each other, two lines apart
(`epics.md:1012` and `:1014`) — the first demanding observations AFTER the failure record *"or the
assertion cannot fail"*, the second forbidding anything to follow one.

Epic 4 resolved it by an asymmetry recorded in two places. `fixture_connector.rs:150-152`, verbatim:

> *"An in-memory stream is judged by no trap file, and a caller needs to build exactly that shape to
> prove a faulted replay emits a strict PREFIX of the clean one (D35(a)). Enforcing it here would
> forbid the test that proves the story's own criterion."*

`deferred-work.md:73-78` records the same decision in its own words — *"a caller **must be able to**
build …"* — and adds: *"If a future story gives in-memory streams a trap-like consumer, this needs
revisiting."* It names **a future story**, not this one. This is that story, and it revisits the
rule by **not** giving them such a consumer.

**1c. A `Trap` judges ONE stream and ONE pair; monotone honesty is a relation between TWO RUNS.**
`Expectation` has three columns (`trap.rs:69`) and `incomplete_families` (`trap.rs:412`) requires a
`must-merge` **and** a `must-not-merge` pole per family — a `must-abstain` sets neither. There is no
projection of *"the faulted run's facts are a subset"* onto those poles.

**1d. `deferred-work.md:324` assigns this story a FOURTH deliverable `epics.md` never mentions** —
lattice monotonicity, *"**Owner: Epic 5**, as its "monotone-honesty invariant trap family""*. It
needs `ScoredRecord`s, hence `capability_snapshot`s, and **11 of the 11 trap-named replay streams
carry no `capability` control record** (re-measured: the trap-named set and the control-record-free
set coincide exactly). §7 registers it rather than claiming it.

### 2. Guy's arbitrations — three at contexting, two at validation

| # | when | question | decision |
|---|---|---|---|
| 1 | contexting | one story or several? | **SPLIT.** 5.13 = the measurement. **5.13b INSERTED** = the committed family + the `MANIFEST` bump. Epic 5 → **19 stories**; 5.14 keeps its number (D56b's letter idiom). |
| 2 | contexting | what carries the faulted run? | **Two named mutilations, derived IN MEMORY from a committed clean stream.** Literally *"the same fixture"* as D35(a) demands, and the door story 4.5a left open (§1b). |
| 3 | contexting | what is `device_facts` at Epic 5? | **Both layers.** Layer 1 = the facts emitted into the sink. Layer 2 = the identity links `resolve_within` writes. |
| 4 | validation | AC3's inverse direction, measured a no-op | **Moves to 5.13b**, which commits the streams that could carry it (§3a). |
| 5 | validation | how is the fact set built, given `Fact: !Ord`? | **A MULTISET, inclusion by removal, on `PartialEq` alone.** No `opencmdb-core` change (§3b). |

### 3. 🔴 The failure mode: `⊆` is satisfied by `=` — and it bit three times inside this story

The property is **falsifiable in principle and unfalsifiable as usually written**: `facts(faulted) ⊆
facts(clean)` is satisfied by a connector that ignores every fault, by a mutilation that removes
nothing, and by a test whose faulted run is the clean run.

**Every AC therefore asserts a PAIR**, and the two halves are separately reddenable:

| half | statement | what it catches | failing means |
|---|---|---|---|
| (i) inclusion | `facts(faulted) ⊆ facts(clean)` | the run INVENTED | a **product** defect |
| (ii) strictness | `facts(faulted) ⊊ facts(clean)` | the fault did NOT BITE | a **test** defect |

⚠️ **A single combined assertion is refused**: it reds without saying which half failed, and the two
mean opposite things. Two assertions, two messages.

**The three times it bit, all measured:**

**3a. AC3's inverse direction.** Deriving the clean run from a committed control-record stream by
REMOVING the control record changes **nothing emitted**:

```
INVERSE partial-then-failed:  clean facts=8 faulted facts=8   clean obs=4 faulted obs=4
INVERSE capability-downgrade: clean facts=8 faulted facts=8   clean obs=4 faulted obs=4
```

Two structural causes. In `partial-then-failed.jsonl` the failure is **line 5 of 5** and must be
(finding 1b), so removing it changes the poll's `Result`, never the fact set. And **a `capability`
record does not filter facts at all** — `poll` only reassigns `in_force` (`fixture_connector.rs:324`)
and nothing downstream strips anything; the strip that makes M-B bite is the *mutilation's* work.
**Moved to 5.13b** (arbitration 4).

**3b. The oracle was a SET, and a set is the wrong structure.** `Fact` derives `Debug, Clone,
PartialEq, Eq, Serialize, Deserialize` (`observation/mod.rs:162`) — no `Ord`, no `Hash` — so
`BTreeSet<(ObsId, Fact)>` does not compile, and making it compile takes **two** derives in
`opencmdb-core` (`Fact`, and `HostnameSource` at `:150`, without which the first alone fails).
🔑 **But a set would silently swallow `[x]` against `[x, x]`** — verbatim the hole story 5.11b
measured, where `contradicts`' `facts.len()` term *"was droppable with the suite green, being the
only term that catches `[x]` against `[x, x]`"*. **A multiset is right on the merits**, inclusion by
successive removal needs `PartialEq` alone, and `n ≤ ~20` facts makes the quadratic cost nothing.

**3c. M2, the mutation for the inclusion half — see §5.**

### 4. The two mutilations, and why exactly these two

Both are **pure functions over `Vec<Record>`** — `Record` is `pub` (`fixtures.rs:88`) — and both
produce a stream `FixtureConnector::from_records` accepts.

**M-A — `cut_at(records, k)`: the poll fails after `k` records, and the tail is KEPT.**
Inserts `Record::Failure(ConnectorError::Unreachable { .. })` at index `k` and **leaves every
following record in place**. `poll` returns `Err` at the failure (`fixture_connector.rs:321`), so the
tail is never emitted.

🔑 **Keeping the tail is the whole point.** Truncating would make the two runs differ in their INPUT
and the claim would degenerate to *"a shorter input produces fewer facts"*, which is arithmetic. With
the tail kept, **the two runs are the same records** and the only difference is one control line —
`epics.md:1012`'s own clause, satisfied for the first time. _(Measured: M3 confirms this argument is
carried by exactly one guard, T1's record-count assertion — see §5.)_

**M-B — `blind_after(records, k, kinds, as_of)`: the source goes half-blind after `k` records.**
Inserts a `Record::Capability` restricted to `kinds` at index `k`, **and strips from every following
observation any fact whose kind is not in `kinds`**.

⚠️ **Three load-time refusals, not one**, all measured:

| refusal | site | what it forces |
|---|---|---|
| `UndeclaredFactKind` | `fixture_connector.rs:247-257` | the **strip is mandatory** — validation is positional, so a record denying `Mac` in front of an observation carrying one refuses the whole stream |
| `CapabilityPredatesObservation` | `:191` | `as_of ≥ max(observed_at)` over `records[..k]` — tracked as a **MAX**, not as the previous line |
| `CapabilityOutOfOrder` | `:201` | `as_of ≥` any preceding capability record's |

🔴 **Hence the fourth parameter.** The three-argument form has nowhere to put an instant, and **two
of the four obvious choices refuse to load** (measured: epoch → refused; the stream's own first
instant → refused; just after the k-th observation → loads; far future → loads). Either take `as_of`
explicitly or derive it and document the rule — do not discover it as a load failure.

The strip is also what makes the shape HONEST: a source that lost a capability stops reporting it.

**Why two.** They carry different halves and neither subsumes the other — **measured, and the
prediction held**:

| | removes | rows in `faulted \ clean` at layer 2 | committed witness |
|---|---|---|---|
| M-A | whole observations | **0** | `partial-then-failed.jsonl` |
| M-B | facts within an observation | **2 abstentions** | `capability-downgrade.jsonl` |

M-A alone would leave §6's second half untested.

### 5. 🔴 The mutation table — with the OBSERVED result beside each prediction

Every red below was classified by reading its panic message one at a time. **Zero compiler-carried,
zero `.expect()`-carried.**

| id | mutation | predicted | **OBSERVED** |
|---|---|---|---|
| **M1** | `poll` continues past `Record::Failure` | strictness reds, inclusion green | ✅ **as predicted** — 1 red, `AC1(ii) STRICTNESS: the fault did not bite`. The split the story asks to be read IS real |
| **M2-naive** | `poll` appends an invented `Fact::Hostname` to **every** emit | inclusion reds | 🔴 **GREEN — REFUTED, 0 new reds.** Unconditional, so it lands on both runs and enters both sides of `f ⊆ c` |
| **M2** | on a `Record::Failure`, **synthesise the observation the fault lost** | — | ✅ inclusion reds, **2 tests**, assertion-carried |
| **M3** | `cut_at` truncates instead of keeping the tail | green everywhere | ✅ **as predicted**, green except T1's record-count guard (`left: 2, right: 4`) — which is precisely the closure §4 prescribes |
| **M4** | `k = len` — the failure is last, nothing removed | strictness reds | ✅ degenerate on **all 11** streams (`clean=6 faulted=6`). ⚠️ An INPUT, not a code mutation — it needs its own test (§5b) |
| **M5** | `blind_after` skips the strip | fails to LOAD | ✅ `UndeclaredFactKind`. ⚠️ Carrier is an **`Err`**, not a panic — say so |
| **M6** | the placement partition counts abstentions as placements | §6 half (b) reds | 🔴 **reds half (a)**, and only on M-B. It cannot reach (b), which reads raw rows |
| **M7-naive** | delete half (b) | green | green **by construction** — deleting an assertion cannot red its own test (5.12's named family). Mis-designed |
| **M7** | M8 **with (a) deleted** | — | ✅ (b) reds: `AC4(b): a faulted-only row is a PLACEMENT` |
| **M8** | `resolve` mints instead of finding an interface | `interfaces_minted` reds | ✅ reds — but **AC4(a) fires first** (assertion order), on both mutilations |
| ~~M9~~ | compare with `==` rather than `⊆` | comparison reds | **dropped as redundant with §6(c)**: measured `pc = 3, pf = 1`, so `==` fails exactly where (c) passes |

🔑 **The transferable lesson, and it generalises past this story: a differential test is blind to any
mutation that is a pure function of the thing being differenced.** The mutation must be conditional
on the FAULT.

**5b. `M4` is an input, so the sweep cannot carry it.** See AC3's bound.

### 6. 🔴 Layer 2: what a "fact" is at the engine, and the row that is NOT one

Layer 2 uses story 5.10's apparatus: **run clean → `snapshot_links` → `purge_engine_links`
(interfaces are NOT purged) → run faulted → `snapshot_links` → compare.** `interface_id` is literally
comparable because the second pass FINDS its interfaces — measured, `minted=0 found=1`.

**The naive subset over full rows is WRONG.** On `randomized-mac.jsonl` (obs1 and obs2 share
`[2,0,94,0,83,32]`; obs3 carries `…,33`), under M-A at `k = 1` the faulted run emits one observation
and places it on the SAME interface as the clean run — but through `decide_singleton` rather than
`decide_pair`, so the justification differs.

⚠️ **How it differs is NOT what an earlier draft claimed, and the correction narrows the argument.**
`l1.rs:345-354`:

```rust
pub fn decide_singleton(observation: &Observation) -> Decision {
    decide(vec![RuleVerdict {
        rule: RuleId(L1_EXACT_MAC.to_string()),   // ← the SAME rule as decide_pair
        verdict: Verdict::Decisive,
        evidence: vec![observation.obs_id],       // ← and it DOES carry evidence
    }], CURRENT_RULESET_VERSION)
}
```

`l1.rs` even carries a heading — *"# The evidence is the observation itself"* — and two tests assert
both facts. So at L1 **today**, over every row that carries an interface, `rule_id` and `outcome` are
**CONSTANT**: every member of a `join` group shares the key, hence `l1-exact-mac` / `Decisive` /
`Match`. **`evidence` is the only justification column that can differ between a clean and a faulted
placement.** The design argument survives — it survives on one column, not three.

**So the comparison is over the PLACEMENT — `(observation_id, interface_id)` — and `rule_id`,
`evidence` and `outcome` are deliberately EXCLUDED.** Safe for the same reason story 5.10's `id`
exclusion was safe: **the thing the claim is about is compared.**

**And the second half:**

> 🔑 **An abstention is not an invented fact.** D35(a) forbids ADDING an *assertion*. An abstention
> asserts nothing — it is the recorded absence of one, and FR16 makes it a first-class outcome. Under
> M-B the faulted run writes rows the clean run does not have (measured: **2**, `interface_id = NULL`,
> `outcome = "abstained"`, `abstention_cause = "absence_of_proof"`). **Those rows are legal and they
> are the honest answer.**

The engine-layer claim is three statements:

- **(a)** `placements(faulted) ⊆ placements(clean)`;
- **(b)** every row in `faulted \ clean` is an **abstention**, never a placement;
- **(c)** `placements(faulted) ⊊ placements(clean)` — strictness.

⚠️ **(b) is largely IMPLIED by (a), and the story must say so rather than oversell it.** A
faulted-only row with `interface_id = Some(x)` puts `(obs, x)` in `pf \ pc`, which *is* (a) — proved
by M7, where (b) catches exactly what (a) catches. **(b)'s only independent clause is
`outcome == "abstained"`**: it is what refuses a faulted-only row that carries a NULL interface but
is not an abstention. That is a narrower claim than *"the one the whole story turns on"*, and it is
the true one.

⚠️ **`interfaces_minted == 0` cannot be violated by a fault, and that is a theorem rather than a
defect**: interfaces are found by `(l2_domain, mac_canon)`, the purge does not delete them, and the
faulted key set is a subset. Keep the assertion — it is one line and it reds under an engine
mutation — but do not present it as catching an input-level fault.

### 7. What this story does NOT claim, stated so it is never read as met

**Lattice monotonicity (D36) is NOT implemented**, and §6's exclusion of the justification columns is
exactly what leaves it open. D36's law [architecture.md:2075-2077] is a statement about the
JUSTIFICATION; this story compares placements. Implementing it needs a doubt ORDER on `Verdict` and
`ScoredRecord`s carrying `capability_snapshot`s, which **11 of 11 trap-named streams cannot supply**.

**Re-owned, not discharged**: the doubt order is Epic 6's (it needs `Supports`/`Opposes` to have a
producer); the capability-snapshot half is **5.13b's**. Append the re-owning to
`deferred-work.md:324`; **never rewrite a bullet**.

**NFR8 has four assertions and this story covers ONE.** (b) bounded blast radius, (c) convergence
after recovery and (d) exactly one actionable notification are untouched — (b) and (c) need the
scheduler, (d) the notification surface. *"NFR8 is verified"* must never be read as met.

### 8. The tree this story extends, measured on `6078246` (2026-08-10)

```
cargo test --workspace   248 bin + 159 core + 62 xtask = 469 passed, 1 ignored
cargo xtask ci           7 gates green; fixtures: 25 artefacts, 0 generated
                         file-size: 27 files under 2000 code lines (largest: 1787)
                         views-hash ℹ STALE — by design, do NOT regenerate (issue #50)
```

| what | where |
|---|---|
| `Record`, `read_records`, `read_jsonl` | `fixtures.rs:88`, `:490`, `:647` |
| `from_records`, `poll`, the three refusals | `fixture_connector.rs:153`, `:284`, `:191`/`:201`/`:247-257` |
| `VecSink` / `ObservationSink` | `connector/mod.rs:106` / `:99` |
| `Fact`, `HostnameSource` derives | `observation/mod.rs:162`, `:150` |
| `resolve`, `resolve_within`, `interfaces_minted` | `resolver.rs:207`, `:228`, `:132` |
| `snapshot_links`, `purge_engine_links`, `LinkSnapshot` | `repo.rs:1021`, `:942`, `:969` |
| **`DB_TEST_LOCK`** | **`main.rs:40`** — see §9 |
| the corpus streams | `fixtures/scenario/replay/*.jsonl` — **read only** |

### 9. 🔴 A green suite says NOTHING about layer 2 — and layer 2 must take the lock

`DATABASE_URL` is unset locally; DB-backed tests `return` and the counts are identical either way.

```
docker run -d --name opencmdb-5-13 -p 13308:3306 \
  -e MARIADB_ROOT_PASSWORD=… -e MARIADB_DATABASE=opencmdb mariadb:10.11.11
```

Port **13308** — 13306 is 5.11b's, 13307 is 5.12's, 3306 is held by an unrelated container.

🔴 **Both layer-2 tests MUST take `crate::DB_TEST_LOCK`.** Written without it they collide:

```
panicked: insert observation: MySqlDatabaseError { number: 1062,
  message: "Duplicate entry 'eeeeeeee-0000-4000-8000-000000000001' for key 'PRIMARY'" }
```

**This is the first story to run the engine over the CORPUS's `obs_id`s**, which two of its own tests
share — `resolver.rs`'s DB tests avoid the collision only because they use synthetic ids. `repo.rs`
takes the lock in ~18 places. Without it the failure is **intermittent and looks exactly like issue
#38**, whose cause is still open.

⚠️ **Commit before the mutation pass.** ⚠️ **A mutation must preserve the ARITY of a SQL statement's
bind parameters** or the MySQL protocol desynchronises and the suite HANGS — run every one under a
timeout.

### 10. Gates — measured green with a new `fault_injection.rs` in the tree

```
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask ci
```

- **`fixtures` must still report `25 fixture(s)`**;
- **`file-size`**: the workspace's largest file is **`xtask/src/main.rs` at 1787** code lines against
  the 2000 ceiling. **`fixtures.rs` is 728** — fourth largest, 1272 lines of headroom (its 4677 raw
  lines are 85 % test module). _(An earlier draft attached 1787 to `fixtures.rs` and made it the
  reason for a new file. The conclusion stands on house-shape grounds; the reason was false.)_
  With `fault_injection.rs` the gate reports **28 files**, largest unchanged;
- **`float-free`** walks `identity/` only; this story touches nothing there;
- **`views-hash`** reports `ℹ STALE` and exits 0 — **do not regenerate**;
- `#![deny(missing_docs)]` does not bite on `pub(crate)` — measured.

---

## Acceptance Criteria

**AC1 — the connector layer, inclusion AND strictness, over the cut.**
**Given** a committed replay stream carrying no control record, and the same records with a terminal
failure inserted at position `k` and **the tail kept**
**When** both are polled through `FixtureConnector` into a `VecSink`
**Then** the faulted run's `(obs_id, Fact)` **multiset** is included in the clean run's, **and**
strictly — two assertions, two messages, never one combined.
**And** the comparison is a multiset inclusion by successive removal, on `PartialEq` alone: a SET
would silently accept `[x]` against `[x, x]`, which is the hole story 5.11b measured on
`contradicts`. **`opencmdb-core` is not touched.**
_Reddened by: M2 (inclusion), M1 and M4 (strictness)._

**AC2 — the connector layer, over the blinding.**
**Given** the same stream with a `capability` record inserted at position `k` restricted to a kind
set, its `as_of` satisfying §4's two ordering rules, and every following observation stripped of the
facts that record denies
**When** both are polled
**Then** the same pair of assertions holds, **and** the faulted run emits the same NUMBER of
observations as the clean one — so the loss is measured in FACTS, not in observations, which is what
distinguishes AC2 from AC1.
_Reddened by: M2, M5 (`Err`-carried, not panic-carried)._

**AC3 — the sweep, bounded and non-degenerate.**
**Given** every committed stream that carries no control record — **11 streams, 39 observations**
**When** AC1 and AC2 are applied at every position `k` in **`0 ≤ k < len`**
**Then** each pair holds at each position, and **both** the number of positions and their
**non-degeneracy** are asserted.
⚠️ **`k = len` is excluded by name, and it is mutation M4**: measured, `0 ≤ k ≤ len` gives **50
positions of which 11 are degenerate** — exactly one per stream, all at `k = len`, all
`clean = faulted`. **M4 therefore needs its own test**; the sweep cannot carry it.
⚠️ **`kinds` is a second degeneracy axis the count does not guard.** Measured on one stream:
`kinds = everything the descriptor allows` → not strict; `= everything the stream carries` → not
strict; `= only IpV4` → strict; `= nothing` → strict. **The rule must be stated: `kinds` must deny at
least one kind the tail carries**, and the test must assert it rather than assume it.
_(The inverse direction — deriving the clean run from a committed control-record stream — **moved to
5.13b** by Guy's arbitration: measured a no-op on both streams, §3a.)_

**AC4 — the engine layer: a faulted pass places no observation the clean pass did not.**
**Given** a clean pass and a faulted pass over the same observations, with story 5.10's purge between
them, **both holding `DB_TEST_LOCK`**
**When** the two `snapshot_links` are compared on `(observation_id, interface_id)` over **placements
only**
**Then** the faulted run's placement set is included in the clean run's, and strictly, **and**
`interfaces_minted == 0` on the faulted pass.
**And** the exclusion of `rule_id`, `evidence` and `outcome` is stated in the test's own doc with
§6's reason — including that **`rule_id` and `outcome` are constant at L1 today**, so `evidence` is
the only one of the three that could have varied.
_Reddened by: M8 and M6, both assertion-carried; AC4(a) fires before `interfaces_minted`._

**AC5 — an abstention is not an invented fact.**
**Given** the blinding mutilation, which leaves an observation with no L1 key
**When** the two snapshots are compared
**Then** every row present in the faulted run and absent from the clean one is an **abstention**, and
the assertion names the `outcome` token it read.
**And** a test asserts at least one such row EXISTS — measured **2**, so it is not vacuous.
**And** the doc states honestly that (b) is largely implied by (a), its only independent clause being
`outcome == "abstained"` (§6).
_Reddened by: M7 (M8 with (a) deleted). The naive form — deleting (b) — is green by construction._

**AC6 — the measurement cannot be green by accident.**
**Given** each acceptance criterion above
**When** the mutation pass runs
**Then** §5's table is filled in with the OBSERVED result, the number of red tests, and **the carrier
of each red read from its own panic message**.
**And** a mutation whose result contradicts its prediction is written up as a FINDING with its
measurement. **The validation already produced three such refutations (M2-naive, M6, M7-naive) and
they are in the table** — a dev pass that reproduces them changes nothing; one that CONTRADICTS them
is the finding.

**AC7 — nothing under `fixtures/` moves, and the corpus gate stays red.**
`cargo xtask ci` reports **`25 fixture(s)`** and seven gates green, no `MANIFEST.toml` entry is added
or re-hashed, and the trap gate still reports **24 discovered, 13 scored, 11 unanswerable,
`passed() == false`** — a green trap gate is a FINDING (D18: *"a gate that cannot fall is
decoration"*).

**AC8 — the documents that describe this state are updated in the SAME commit.**
`CLAUDE.md`, `docs/project-context.md` and `sprint-status.yaml` carry the outcome, **the insertion of
5.13b** (Epic 5 → 19 stories), the live test count, and §1's four findings.
⚠️ **One live count for the story, in one place.**

**AC9 — what is NOT claimed is written down.**
Lattice monotonicity is **not** implemented and is re-owned (§7); NFR8 has four assertions and this
covers one; `deferred-work.md:324` is **appended to, never rewritten**.

---

## Tasks / Subtasks

- [x] **T1 — the two mutilations**, in a new `crates/opencmdb-bin/src/fault_injection.rs` (AC1, AC2)
  - [x] `cut_at(records, k)` — insert a terminal `Failure`, **keep the tail**
  - [x] `blind_after(records, k, kinds, as_of)` — insert a `Capability`, **strip the denied facts**;
        document §4's three refusals and the `as_of` rule
  - [x] a unit test per mutilation asserting its output's record COUNT — the guard M3 needs
- [x] **T2 — the multiset oracle** (AC1, AC2)
  - [x] inclusion by successive removal over `Vec<(ObsId, Fact)>`, `PartialEq` only
  - [x] a test asserting it distinguishes `[x]` from `[x, x]` — the 5.11b hole, closed by
        construction and pinned by a test rather than by a comment
  - [x] `raw` **excluded** (D19: no decision reads it), with a test asserting the bare comparison
        WOULD have differed
  - [x] inclusion and strictness as **two** assertions with distinct messages
- [x] **T3 — the sweep** (AC3)
  - [x] 11 streams × `0 ≤ k < len` × both mutilations; the count **and** the non-degeneracy asserted
  - [x] `kinds` chosen so it denies a kind the tail carries — asserted, not assumed
  - [x] M4 (`k = len`) gets its own test
- [x] **T4 — the engine layer** (AC4, AC5) — **needs a live MariaDB and `DB_TEST_LOCK` (§9)**
  - [x] `let _guard = crate::DB_TEST_LOCK.lock().await;` **first line of both tests**
  - [x] clean → snapshot → `purge_engine_links` → faulted → snapshot
  - [x] the placement partition and §6's (a)/(b)/(c); `interfaces_minted == 0`
  - [x] AC5's existence assertion
- [x] **T5 — prove-to-red** (AC6): run M1–M8; fill §5 with OBSERVED results; classify each red by
      reading its panic message; write up every divergence
- [x] **T6 — gates and documents** (AC7, AC8, AC9): the four commands; the twins; append the
      re-owning to `deferred-work.md`; register §1 and 3a with **5.13b**; **do not edit `epics.md`**

---

## Dev Notes

### Shapes to follow, not reinvent

- **The purge-and-replay apparatus is story 5.10's.** `purge_engine_links` is a `DELETE`; interfaces
  are not purged, which is what makes `interface_id` comparable.
- **`from_records` is the in-memory door** and the only one accepting a record after a terminal
  failure. Do not route through `load`.
- **`opencmdb-core` is not touched.** Arbitration 5 is what makes that true; if you find yourself
  reaching for a derive there, re-read §3b — the multiset is not a workaround.

### Where the code goes

A new `crates/opencmdb-bin/src/fault_injection.rs`, the house shape (`l1_runner.rs`, `resolver.rs`,
`permute.rs` were each created by the story that needed them), with the trailing `#[cfg(test)] mod
tests` (D56b) and `#![allow(dead_code)]`. **Measured: all seven gates green with it, clippy
`--all-targets -D warnings` clean.** Not in `fixtures.rs` — not for the file-size reason an earlier
draft gave (§10), but because a mutilation is not a reader.

### What a reviewer will challenge, and the answer that must already be measured

| challenge | the answer |
|---|---|
| *"`⊆` is satisfied by `=` — this cannot fail"* | §3, and M1/M4 in the filled table |
| *"M2 proves the inclusion half"* | **only the fault-CONDITIONAL form does** — §5, M2-naive is green |
| *"why a multiset and not a set?"* | §3b — 5.11b's measured `[x]` vs `[x, x]` hole |
| *"the faulted run ADDS a row, so the subset is false"* | §6(b), AC5, measured 2 rows |
| *"why exclude `rule_id`/`evidence`/`outcome`?"* | §6 — and two of the three are **constant** at L1 |
| *"is (b) load-bearing?"* | §6 — largely implied by (a); its one independent clause is named |
| *"is this lattice monotonicity?"* | §7 — **no**, re-owned rather than implied |
| *"why no committed fixture?"* | §1's four findings, §2's arbitration 1; 5.13b owns it |

### References

- [Source: `epics.md#Story 5.13`], [`epics.md:1012`/`:1014`] — 4.5a's two contradictory criteria
- [Source: `architecture.md:2011-2028`] — D35, NFR8's four assertions; (a) monotone honesty
- [Source: `architecture.md:2075-2077`] — D36's lattice monotonicity, which §7 does NOT claim
- [Source: `deferred-work.md`] — the 4.5a block (`:73-78`, the in-memory asymmetry), the 4.6b block
  (`:312-314`, two streams judged by no trap), the 4.6c block (`:324`, lattice monotonicity)
- [Source: `crates/opencmdb-bin/src/fixture_connector.rs:150-152`] — the asymmetry, naming D35(a)
- [Source: `crates/opencmdb-core/src/identity/l1.rs:345-354`] — `decide_singleton`, §6's correction
- [Source: `crates/opencmdb-core/src/trap.rs:412`] — `incomplete_families`, finding 1c

---

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context) — `claude-opus-5[1m]`.

### Debug Log References

**Layer 2 ran against a live database.** `mariadb:10.11.11`, container `opencmdb-5-13`, host port
**13308** (13306 is 5.11b's, 13307 is 5.12's, and 3306 is held by an unrelated container —
`kesh-mariadb-dev`, exactly the trap story 5.9's validation caught). `DATABASE_URL` =
`mysql://root:…@127.0.0.1:13308/opencmdb`.

**Every layer-2 mutation ran with the database UP.** The control was run both ways: without
`DATABASE_URL` the two engine tests print `skipping layer-2 test: DATABASE_URL unset` and pass by
returning, and the suite reports the same counts. The timing is the other witness — the `bin` suite
takes **0.05 s** without a database and **3.8 s** with one.

**Baseline re-measured on `6078246` before any code was written**: 469 (248 + 159 + 62), seven gates
green, 25 fixtures. **The tree was committed (`a63c44f`) before the mutation pass**, and
`git status` was verified empty after it.

### Completion Notes List

**485 tests (264 bin + 159 core + 62 xtask), +16.** Seven gates green, `fixtures` still **25**,
`file-size` now 28 files (largest unchanged at 1787, `xtask/src/main.rs`). The trap gate is still
**red** — `the_committed_corpus_is_red_with_eleven_unanswerable_traps` passes, so the corpus still
reports `passed() == false`. **No byte under `fixtures/` moved. `opencmdb-core` was not touched.**

#### The mutation table, with OBSERVED results (AC6)

| id | mutation | predicted | OBSERVED | carrier |
|---|---|---|---|---|
| **M1** | `poll` continues past `Record::Failure` | strictness reds, inclusion green | ✅ **as predicted** — 9 red, and `ac1` panics on `AC1(ii) STRICTNESS: the fault did not bite — clean=6 faulted=6`, i.e. **after** the inclusion assertion passed | assertion |
| **M2-naive** | `poll` invents a `Hostname` on **every** emit | GREEN (validation's finding) | 🔴 **REFUTED IN ITS PHRASING, CONFIRMED IN ITS SUBSTANCE** — 3 red, and **not one of them is a story-5.13 test**. All three are byte-equality tests in `fixture_connector`. The differential tests are blind to it, exactly as measured; *"0 new reds"* was the imprecise part | assertion |
| **M2** | on a `Failure`, synthesise the observation the fault lost | inclusion reds | ✅ 9 red — `AC1(i) INCLUSION: the faulted run INVENTED 3 fact(s)` | assertion |
| **M2b** | on a `Failure`, **re-emit the first observation UNCHANGED** | — (not prescribed) | ✅ 1 red — `INVENTED 2 fact(s)`, **both of them DUPLICATES and no new fact at all** | assertion |
| **M2b + set oracle** | the same, with the oracle turned into set semantics | — (the CONTROL) | 🔑 **GREEN.** The set lets the duplication through | — |
| **M3** | `cut_at` truncates instead of keeping the tail | green except T1's count guard | ✅ **as predicted** — exactly **1** red, `cut_at_keeps_the_tail_…`, `left: 2, right: 4` | assertion |
| **M4** | `k = len` | strictness reds | ✅ its own test; and `the_excluded_position_…` measures **11 degenerate of 50**, one per stream, all at `k = len` | assertion |
| **M5** | `blind_after` without the strip | fails to LOAD, `Err`-carried | ✅ its own test — `UndeclaredFactKind`. Carrier is an **`Err`**, not a panic | `expect_err` |
| **M6** | the partition counts abstentions as placements | reds half (a), on M-B only | ✅ **as the validation predicted** — 1 red, `ac5`, `AC4(a) INCLUSION … [(obs2, None), (obs3, None)]`; the cut test stays green | assertion |
| **M7** | M8 with (a) deleted | (b) reds | ✅ 1 red — `AC4(b): a faulted-only row is a PLACEMENT, not an abstention` | assertion |
| **M8** | `resolve` mints instead of finding | `interfaces_minted` reds | ✅ 19 red — but **`AC4(a)` fires FIRST**, as the validation warned; `interfaces_minted` is never reached | assertion |

**Eleven mutations, and every red is carried by a NAMED assertion** — classified by reading each
panic message one at a time. **Zero compiler-carried and zero `.expect()`-carried**; M5's carrier is
an `expect_err` on a `Result`, which is recorded as such rather than folded into "assertion".

#### 🔑 The finding the story did not prescribe: the multiset is load-bearing, and here is the number

M2 invented a `Hostname`, and the oracle reported **three** facts — the new one **plus two
duplicates**, because the mutation re-emitted an observation the cut had already emitted. That
prompted **M2b**, which re-emits the first observation *unchanged*: no new fact anywhere, and the
oracle still reds with `INVENTED 2 fact(s)`.

**And the control makes the pair mean something**: with the same mutation and the oracle switched to
set semantics — membership without removal — the test goes **GREEN**.

So the arbitration that made the oracle a multiset is no longer justified by story 5.11b's precedent
alone. **A run that invents nothing and merely repeats itself is a monotone-honesty violation, and a
set-based oracle cannot see it.**

#### ⚠️ A defect in the mutation DRIVER, caught by disbelieving its own result

M6 first reported **0 red**. The mutation was sound; the driver was not — `cargo test --workspace A B`
passes two filters where cargo accepts one, so nothing ran. Re-measured with the full suite, M6 reds
1 test on the expected assertion.

**It is the story's own subject applied to the story's own tooling**: a measurement that reports
success because it measured nothing. It is recorded rather than quietly fixed, because the reason it
was caught is that 0 red contradicted a prediction — had the prediction been *"green"*, the driver
defect would have been filed as a confirmation.

#### 🔑 A claim of §6, confirmed by a mutation's OUTPUT rather than by reading

M7's failure message prints the row it refused:

```
LinkSnapshot { observation_id: "eeeeeeee-…0001", interface_id: Some("019feb77-…"),
               outcome: "match", rule_id: Some("l1-exact-mac"),
               evidence: [ObsId(eeeeeeee-…0001)], … }
```

`rule_id` is **`l1-exact-mac`** and `evidence` is **the observation itself** — on a placement the
faulted pass settled through `decide_singleton`. That is the fact-check's correction to §6, arriving
a second time from a different direction: `decide_singleton` names the same rule as `decide_pair` and
does carry evidence, so `evidence` is the only excluded column that could have varied.

#### What is NOT claimed

Lattice monotonicity (D36) is **not** implemented — re-owned in `deferred-work.md` (Epic 6 for the
doubt order, 5.13b for the capability snapshot). **NFR8 has four assertions and this story covers
ONE**: bounded blast radius, convergence after recovery and exactly-one-notification are untouched.
AC3's inverse direction moved to 5.13b at validation, on a measurement.

### File List

- `crates/opencmdb-bin/src/fault_injection.rs` — NEW. The two mutilations, the multiset oracle, the
  bounded sweep, and the engine-layer comparison, with their tests.
- `crates/opencmdb-bin/src/main.rs` — MODIFIED. One line: `mod fault_injection;`.
- `_bmad-output/implementation-artifacts/5-13-monotone-honesty-measurement.md` — this file.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — status and the record.
- `_bmad-output/implementation-artifacts/deferred-work.md` — the re-owning, appended.
- `CLAUDE.md`, `docs/project-context.md` — the twins (AC8).

## Change Log

| date | what |
|---|---|
| 2026-08-10 | Created. **SPLIT at contexting with Guy**: 5.13 the measurement, 5.13b (INSERTED) the committed family. Three arbitrations, four findings against `epics.md`'s text, `epics.md` NOT edited. Baseline on `6078246`: 469 tests, seven gates, 25 fixtures. |
| 2026-08-10 | **VALIDATED** (fact-check + gap-hunt, the second having BUILT the story against a live MariaDB). 6 HIGH, 5 MEDIUM, 11 LOW applied; **2 further arbitrations by Guy** — AC3's inverse direction moves to 5.13b, and the oracle becomes a multiset on `PartialEq` alone. Three of the story's own mutations were **refuted by measurement** (M2-naive, M6, M7-naive) and are in §5 with their observed results. |
| 2026-08-10 | **IMPLEMENTED → `review`** (`done` is the MERGE's business). **469 → 485 tests** (264 + 159 + 62), seven gates green, 25 fixtures, trap gate still red, `opencmdb-core` untouched. **Eleven mutations, eleven reds, every one assertion-carried.** One unprescribed finding — a run that invents nothing and merely REPEATS itself violates monotone honesty, and a set-based oracle is measured GREEN on it — and one defect in the mutation driver, caught by disbelieving a 0-red result. |
