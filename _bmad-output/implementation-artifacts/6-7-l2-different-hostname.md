# Story 6.7: `l2-different-hostname` — the first producer of `Opposes`

Status: **review** — implemented 2026-08-30 against a live `mariadb:10.11.11` (port 13369, virgin). Contexted 2026-08-30 against the committed corpus and the tree at
`db1e3f9`; **arbitration TAKEN by Guy the same day**; **VALIDATED the same day** by two
fresh-context layers, each in its own worktree — ten findings, three HIGH, all applied in place. ⚠️ **`create-story validate` (two fresh-context agents) is MANDATORY before `dev-story`**
and has NOT been run.

✅ **THE ARBITRATION IS TAKEN: OPTION (a), GUY, 2026-08-30** (§0c) — and it is HIS, where story
6.6's twin decision was mine by delegation. `cloned-mac-must-not-merge` **stays unanswerable at L2**;
the unanswerable bucket goes **11 → 4, not 11 → 3**; and **option (c) — the structural reading — is
REGISTERED BY NAME to story 6.11**, whose subject it already is. `epics.md` is not edited.

🔑 **The reason, in one sentence: the trap is not unanswerable because the engine is weak, it is
unanswerable because it interrogates the WRONG LAYER** — it asks an L2 rule to separate what the L1
key has already fused, and a rule comparing hostnames cannot undo a merge decided on the MAC.

⚠️ **The cost accepted, and stated rather than hidden: a committed trap is READ BY NOTHING until
story 6.11.** That is exactly the kind of silence this project's retrospectives find three epics
later, so it is covered the only way that works — **T6 asserts by NAME that
`cloned-mac-must-not-merge` is the one not answered**, so a second trap falling into this case reds a
test instead of vanishing.

## Story

As the operator,
I want two interfaces whose hostnames disagree to argue against being one device,
So that the cascade gains its first opposing voice.

## Acceptance Criteria

*(Source: `epics.md:1848-1864`, verbatim. Everything beyond it is §0's, and §0 says which.)*

**AC1 — the first `Opposes`**

**Given** a candidate pair whose hostnames are both present and differ
**When** the rule is evaluated
**Then** it yields `Verdict::Opposes` — **the first producer of that variant in this codebase** — and
the three committed traps expecting `l2-different-hostname` are answered by it.

🔴 **AC1's last clause is UNSATISFIABLE, measured: only TWO of the three can be answered** (§0b),
and Guy's arbitration (a) accepts that rather than reshaping the level to fit it. **Read the clause
as *the two committed traps that HAVE an L2 pair*,** with the third named in T6 as the excluded one.
`epics.md` is not edited; the divergence is registered.

**AC2 — the common bug, named by D20**

**Given** a pair where either hostname is absent or empty
**When** the rule is evaluated
**Then** it yields `Neutral`, never `Opposes`. ⚠️ **D20 names this as the common bug**: *"the rule
that wrongly `Opposes` should return `Neutral` — it does not KNOW, it BELIEVES it knows; nine
parasitic abstentions out of ten are that."* The `hostname-absence` family exists to catch it.

🔴 _(The criterion above is now `epics.md`'s own words. **An earlier draft silently replaced D20's
quotation INSIDE this AC block with the longer form from `architecture.md:1408-1412`** — a truer
quotation of D20, but not what the epic says, under a header promising the ACs are verbatim, and
without the divergence note AC1 gets three lines below. **Two standards in one file**, and *a
quotation that is not verbatim* is what this project ranks highest. Found by the validation's
fact-check layer.)_

⚠️ **D20's fuller statement is worth reading and is NOT part of the criterion**
[`architecture.md:1408-1412`]: *"Weighting is almost always the wrong fix for a wrong verdict — a
rule that claims to know what it does not know IS the bug; the weight merely masks the lie by
attenuating it."*

**AC3 — the spelling**

**And** the rule id is spelled exactly as the corpus spells it, or the trap reds as `rule_mismatch`.

🔴 **AC3's stated MECHANISM is structurally unreachable, measured end to end (§0i), and the criterion
ships with a different carrier.** A misspelled rule id leaves the trap **PASSING**, not
`WrongRule` — so the mutation T7 first prescribed for it was **measured GREEN**. Read AC3 as *the
rule id is pinned against the corpus spelling*, carried by a **double-literal test** on L1's
`CORPUS_EXACT_MAC` idiom, **never through `run_trap`**.

**AC4 (this story's, §0d) — the multi-hostname interface is DECIDED, not defaulted**

**Given** an interface whose observations carry two DIFFERENT hostnames
**When** the rule is evaluated on a pair containing it
**Then** the behaviour is chosen explicitly, tested, and its **limit stated**: the committed corpus
exercises this on exactly one interface, and that one is excluded from L2 by §0b — so the case is
**reachable in production and unexercised by the corpus**, which is a sentence to write rather than a
gap to hide.

**AC5 (this story's) — the live count**

The workspace test count and the gate count are recorded IN THIS FILE, each figure naming the state
it was measured on.

## §0 — What contexting MEASURED

Run against `db1e3f9`, working tree clean.

### §0a. What exists, and what this story is the first to do

`Verdict::Opposes` is declared in `cascade.rs` and **has no producer**: measured at story 6.6's
review, every construction of `Supports`/`Opposes` in the tree sits under `#[cfg(test)]`; the
production uses in `decide` are `has(...)` reads. This story makes one real.

`blocking::l2_candidates(&[L1Key]) -> BTreeSet<L2CandidatePair>` ships since 6.6 — TOTAL, no
narrowing key, **no production caller** (story 6.12 is the first). `decide` is TOTAL over D13's table
plus the row it leaves uncovered. `CURRENT_RULESET_VERSION` is `RulesetVersion(1)`.

⚠️ **`L1Key = (L2DomainId, MacAddr)` carries NO `Fact`.** That is deliberate — it is what makes the
uplink narrowing inexpressible inside the blocker (story 6.6 §0f) — and it means **a rule cannot take
an `L2CandidatePair` alone**: hostnames live on `Observation`s. The rule needs an interface WITH its
observations, and choosing that type is this story's first structural act.

### §0b. 🔴 AC1's "three traps" is TWO, and the arithmetic was already registered

Measured by walking every trap file and resolving each named observation through `join`:

| trap | family | an L2 pair? |
|---|---|---|
| `shared-hardware-vm-must-not-merge` | shared-hardware-vm | **yes** — `doc-vm-alpha` vs `doc-vm-beta` |
| `vrrp-virtual-mac-must-not-merge-bearers` | vrrp-virtual-mac | **yes** — `doc-rtr-alpha` vs `doc-rtr-bravo` |
| `cloned-mac-must-not-merge` | cloned-mac | 🔴 **NO — collapsed onto ONE interface** |

`cloned-mac`'s two observations carry the **same** `MacAddr` in one `l2_domain`, so `join`'s
`(l2_domain, mac)` key makes them one interface and no pair exists to judge. This is story 6.6's §0j,
already registered with the bucket correction **11 → 4, not 11 → 3**; what is new here is that it
**bites a criterion**, not just a count.

### §0c. 🔴 THE ARBITRATION — and the measurement produced a THIRD option

🔑 **THE FINDING THAT REFRAMES IT: the ONE interface in the entire committed corpus carrying two
different hostnames is exactly the interface `cloned-mac` collapses onto.** Measured over all
seventeen hostname-bearing interfaces across every replay stream — one has two names
(`doc-host-echo`, `doc-host-foxtrot`, over three observations), and it is that one.

***The signal is not lost. The SHAPE is.*** A cloned MAC does not present as two interfaces
disagreeing; it presents as **one interface contradicting itself**.

**(a) Keep story 6.6's arbitration. `cloned-mac-must-not-merge` stays unanswerable at L2.**
The bucket is 11 → 4, story 6.15 inherits one trap more than its criterion states, and AC1's third
trap is answered by nothing. Cost: a committed trap that names an `l2-*` rule is never asked about,
and the corpus keeps a row no engine reads.

**(b) Reverse it: the L2 rules judge OBSERVATION pairs.** All three traps become answerable and the
bucket really is 11 → 3. Cost, measured at story 6.6: the blocker that story ships is then **not**
what feeds the rules, and its recall floor measures a population no rule consumes. It also re-opens
the type for 6.8–6.11.

**(c) 🔑 THE THIRD ROAD, which the measurement above opens and neither prior story posed: an
interface whose own observations disagree on hostname is a STRUCTURAL FACT, not a pair rule.**
That is precisely the shape `epics.md` already gives story **6.11** for the virtual-MAC anchor —
*"a STRUCTURAL FACT READ AT INGESTION, not a rule that scores. Guy's arbitration 2026-08-12: **there
is no rule**"* — and D21's own words for this case, quoted in the trap file, are *"a cloned MAC = two
real interfaces, same MAC"*, which says the L1 KEY is what is wrong, not the rule. Under (c),
`cloned-mac-must-not-merge` is answered by a structural reading rather than by
`l2-different-hostname`, ⚠️ **which means the corpus's expected rule for that trap would be wrong and
a corpus bump would be needed** — an act Epic 4's retrospective warns about by name.

### ✅ TAKEN (Guy, 2026-08-30): **OPTION (a)**, with **(c) registered to story 6.11**

**Refused: (b).** It repairs the symptom by breaking the distinction the whole level rests on — L2
reasons about DEVICES; making it reason about observations to recover one trap pays the structure for
a case. ⚠️ And it was refused **knowing it would never be cheaper**: every L2 rule that ships after
this one raises the price of reversing.

**Refused HERE, accepted THERE: (c).** It is right on the substance — D21 says the KEY is what is
wrong — but it is story 6.11's subject, it would arrive three stories early, and it would require a
**corpus bump** in passing, which Epic 4's retrospective names as a thing not to do lightly.

⚠️ **The accepted cost, written rather than implied**: a committed trap is read by nothing until
6.11. T6's naming assertion is what stops that silence from growing.

### §0i. 🔴 AC3 CANNOT BE CARRIED BY THE TRAP RUNNER, and this story is where a dormant hole first bites

The validation's gap-hunt layer built the rule and drove the two answerable traps end to end. The
chain is:

```
decide(vec![the L2 verdict alone])  ->  Conclusion::Abstained { AbsenceOfProof }
outcome_of(...)                     ->  Outcome::Abstained { .. }      // .rule() == None
run_trap(MustNotMerge { l2-different-hostname }, that outcome)  ->  TrapVerdict::Pass
```

🔑 **That is D13's ratified arbitration, not a defect**: `>= 1 Opposes` **alone**, with no
`Disqualifying`, abstains on `AbsenceOfProof` — Guy's own decision of 2026-07-29, the one that
became **GitHub issue #54** and the sixth `xtask` gate's story (5.4b). An `Abstained` carries **no
rule**, so `run_trap`'s `(Some, Some)` rule comparison never fires.

🔴 **Measured, not reasoned: corrupting the rule id to `l2-totally-wrong-id` and replaying the whole
pipeline leaves the trap PASSING.** So T7's prescribed *"misspell the id, predict `rule_mismatch`"*
is **false as written** — it was measured GREEN.

⚠️ **This is the first time anything in this codebase produces `Opposes`**, so a hole that has been
dormant in the algebra since story 5.4b — for want of a producer — bites here. It is not a corner
case; it is the level's opening bill.

**The carrier that works, and it is L1's own idiom**: a test holding the rule id as a literal on both
sides — the constant and the corpus's spelling — independent of `decide`. The gap-hunt layer built it
and proved by mutation that it is the **only** test that reds on a typo, with `cargo xtask ci` and
clippy green throughout.

### §0j. 🔴 A TEMPTING COMBINATION THAT MAKES THIS RULE INVISIBLE — and T2 must warn against it

For **any** valid L2 pair the two `L1Key`s differ by construction (equal keys are one interface —
§0b). So evaluating `l1::verdict_for_pair` on the same two observations always yields
**`Disqualifying` via `l1-distinct-mac`**, and `decide` gives `Disqualifying` absolute priority.
Measured on the real `doc-vm-alpha` / `doc-vm-beta` data:

```
decide(vec![the L1 verdict, the L2 verdict])  ->  Conclusion::NoMatch { rule: "l1-distinct-mac" }
```

🔑 **So the L2 verdict can never name the conclusion**: absent from it (§0i) or overwritten by L1.
***The L2 `decide` must receive ONLY L2 verdicts.*** `l1.rs`'s doc says the two organs do not consult
each other, but **nothing has ever said it about `decide`'s ARGUMENT**, and the combination is the
obvious thing a developer reaches for. **Write it before story 6.12 wires the pass.**

### §0d. The multi-hostname interface, and why AC4 exists

`join` groups N observations onto one interface, so a side of the pair holds a SET of hostnames, not
one. Measured: **17 hostname-bearing interfaces, exactly one with more than a single distinct name**
— the excluded one. So for every pair this story can actually answer, each side has at most one name,
and **the multi-name case is unexercised by the corpus while being reachable in production** (DHCP
churn, a renamed host).

⚠️ *A behaviour the corpus cannot exercise must be chosen and stated, never defaulted.* The two
honest readings: **any disagreement opposes** (a pair whose sides share no name at all), or
**agreement is set intersection** (`Neutral` while any name is shared). Prescribed: **`Opposes` only
when both sides are non-empty and their name sets are DISJOINT** — it is the reading that cannot
oppose on a partial overlap, and D20's lock is about not claiming to know.

✅ **Measured by the validation and worth keeping**: a multi-homed observation sharing its own
hostname across both sides of a pair does **not** manufacture a false `Opposes` — the shared name
makes the sets non-disjoint. A pleasant property, and one to document rather than rediscover.

⚠️ **A counting nuance the first draft did not state**: the **17** above counts an interface as
hostname-bearing whenever a `Hostname` fact is present, **empty string included**. Under §0e's own
semantics — an empty name IS an absence — the figure is **14**. The conclusion is identical in both
readings (one multi-named interface, and it is `cloned-mac`'s), but *the number is only reproducible
under the literal reading*, and saying which was meant is the difference between a figure and a
recollection. Found by the validation's fact-check layer.

### §0d2. 🔴 HOSTNAME CASE IS A DECISION NOBODY HAD TAKEN, and the default is D20's forbidden shape

✅ **Taken (mine, delegated — reversible at the right cost): compare hostnames CASE-INSENSITIVELY,
by ASCII lowercasing.**

The validation built the rule with the obvious case-SENSITIVE comparison and measured the
consequence: **`NAS-01` versus `nas-01` OPPOSES** — two sources reporting one machine with different
capitalisation, argued into being two devices. That is exactly *the rule that BELIEVES it knows*,
D20's named bug, produced by doing nothing in particular.

**D10 puts the choice here rather than in SQL**: hostname matching is an identity anchor, and
comparison *"never descends into SQL"* — so the collation is not the answer and the rule owns it.

⚠️ **ASCII lowercasing, not `to_lowercase()`**: full Unicode case folding has traps of its own
(Turkish dotless ı, sigma), and a DNS label is ASCII. **The limit is stated rather than implied** —
this is right for hostnames and would be wrong for arbitrary text. ⚠️ **No committed trap exercises
case in either direction**, so this behaviour ships with a synthetic test and a written limit, on
exactly the footing AC4 gives the multi-hostname case.

⚠️ **`HostnameSource` is deliberately IGNORED** — the rule does not care whether a name came from
DHCP, DNS, mDNS or NetBIOS. Consistent with D20 (invent no weighting) and with the type's own doc;
said here because a silence about a field is not a decision about it.

### §0e. Absence and emptiness are ONE case, and the corpus says so in its own header

`Fact::Hostname { name: String, source: HostnameSource }` — `name` is a `String`, **not an
`Option<String>`**, so a null hostname is unrepresentable in the format. The `hostname-absence`
family's header states the equivalence this story must implement: *"MISSING and EMPTY are both the
absence of a signal: an empty string is not a matchable value (`"" == ""` is not hostname agreement),
a byte-present empty name counts as NO observed value, and a name that stops resolving opposes
nothing."*

⚠️ **No trap in that family names `l2-different-hostname`** — its three expect `l1-distinct-mac` and
a `MustAbstain`. So the family constrains this rule by **not letting it fire**, which is a negative
requirement and needs a test that reds when the rule wrongly opposes, not a trap that turns green.

⚠️ Trimming is a DECISION, not an obvious step: is `"  "` empty? Prescribed **yes** — trim before
testing emptiness, on `page.rs`'s measured precedent that `"\u{200B}".trim().is_empty()` is `false`
in Rust, so a whitespace test is not a presence test.

### §0f. Where the code goes, and the debt this story inherits by name

`crates/opencmdb-core/src/identity/l2.rs` — **NEW**, and this is where story 6.6's §0f said it
would be: the blocker stays in `blocking.rs` and the RULES live here, which is what makes *"the
blocker consults no rule"* visible in the structure. ⚠️ Story 6.6 measured that this is
**documentation, not constraint** — calling `decide` inside the blocker leaves everything green — so
do not restate it as a guarantee.

⚠️ **Registered to THIS story by name** (`deferred-work.md`, story 6.6's review): `l1.rs` cites
`architecture.md` ~25 lines off in three places (`:984-986`, `:984-985` twice). This story is in
`identity/` anyway. **Re-derive by `grep` on the quoted sentence, never by adding 25.**

⚠️ Also registered here: the near-textual twinning of `l2_corpus()` and `corpus_pairs()` in
`fixtures.rs`, with the question posed rather than answered — the two walk different populations and
one containment assertion is a corollary where the other is not, **a difference a shared helper would
hide**.

### §0g. What this story does NOT do

- **No production caller**, still. Story 6.12 is the first.
- **The trap gate does not fall green**: it moves the unanswerable bucket only when 6.7–6.11 have all
  shipped, and then to **4** rather than 0 under §0c(a). 6.15 closes it.
- **No `Supports`** — that is story 6.8, `l2-uplink-agrees`.
- **`opencmdb-core` gains behaviour**, so narrow the promise to *no behaviour change elsewhere*.

### §0h. Gates and house rules that bite here

- **`float-free`** walks `identity/` — **4 files today, 5 once `l2.rs` lands**. No float type, no
  float literal.
- **`file-size`**: a new file, ample.
- ⚠️ **Run the mutation pass on a VIRGIN database and pass `--baseline`** — story 6.6's measured
  rule: this suite is non-deterministic against a reused store, so a red count without a baseline is
  a guess.
- ⚠️ **A claim of SOLE carriership is worth exactly the mutation that checked it** (6.6, three
  refuted doc comments).
- **Prove-to-red**, and arrange for the red to be **assertion-carried**: ship the guard beside a
  deliberately wrong rule, observe the red on its own message, then correct. *A guard first seen red
  by the compiler has not been seen red.*
- `cargo clippy --workspace --all-targets -- -D warnings`; never read a status through a pipe.

## Tasks / Subtasks

- [x] **T1 — Validation.** Two fresh-context agents, own worktree each. §0c is SETTLED (Guy, option
      (a)); the validation inherits it rather than re-opening it.
- [x] **T2 — The rule's input type** (AC1): an interface with its observations. Document why an
      `L2CandidatePair` alone cannot serve, **and that the `ObsId → &Observation` resolution is the
      CALLER's** (§0f).
      🔴 **And write the warning §0j earns: the L2 `decide` receives ONLY L2 verdicts.** Combining
      the L1 verdict for the same pair is the obvious gesture and it makes this rule INVISIBLE —
      measured `NoMatch { rule: "l1-distinct-mac" }`, because any valid L2 pair has distinct L1 keys
      by construction and `Disqualifying` wins outright.
- [x] **T3 — The absence guard FIRST** (AC2): write the `Neutral`-on-absent and `Neutral`-on-empty
      tests against a deliberately wrong rule that opposes on absence, observe the assertion-carried
      red, then correct. This is D20's lock and it is the story's centre.
- [x] **T4 — The rule** (AC1, AC3): `l2-different-hostname`, spelled exactly as the corpus spells it,
      yielding `Opposes` on disjoint non-empty name sets and `Neutral` otherwise, **comparing
      case-insensitively by ASCII lowercasing** (§0d2). AC3's carrier is a **double-literal test**,
      never `run_trap` (§0i).
- [x] **T5 — The two decisions the corpus cannot exercise** (AC4 and §0d2): the multi-hostname set
      semantics AND the case-insensitivity, each tested synthetically with its limit written in the
      doc. Also state that `HostnameSource` is ignored.
- [x] **T6 — The corpus half**: the two answerable traps answered end to end; **assert by NAME that
      `cloned-mac-must-not-merge` is the one that is not**, so the residue cannot grow in silence.
- [x] **T7 — The mutation pass**, predictions written first, virgin store, `--baseline`. At minimum:
      oppose on absence (must red T3), oppose on empty, oppose on partial overlap, **compare
      case-sensitively** (must red §0d2's guard), and **misspell the rule id — predicted RED on the
      double-literal test and GREEN through `run_trap`, and BOTH halves must be run**, because the
      second is §0i's whole finding.
      🔴 **Do NOT restate the first draft's row** *"misspell the id → `rule_mismatch`"*: it was
      **measured GREEN** and is false as written.
- [x] **T8 — Gates**: `cargo xtask ci`, fmt, clippy `--all-targets`, `cargo test --workspace
      --locked --no-fail-fast` both with and without a store; record the clock.
- [x] **T9 — `l1.rs`'s citation drift**, inherited by name (§0f).
- [x] **T10 — The record**: this file, `sprint-status.yaml`, and the twins **byte-for-byte identical**
      — verified by comparison, not by intention.

## Dev Notes

### The one sentence to keep in view

D20: *a rule that claims to know what it does not know IS the bug.* This story's failure mode is not
a missing `Opposes`; it is an `Opposes` that should have been `Neutral`. Every test that matters here
is a test that the rule **stays quiet**.

### Project Structure Notes

- `crates/opencmdb-core/src/identity/l2.rs` — **NEW**.
- `crates/opencmdb-core/src/identity/mod.rs` — UPDATE (declare the module; its doc was repaired at
  6.6's review and must stay true).
- `crates/opencmdb-bin/src/fixtures.rs` — the corpus half (D47: core reads no files).
  🔴 **NOT `l1_runner.rs`, measured**: its `L1_PREFIX = "l1-"`, `expects_an_l1_rule` and an
  `answer_pair` hard-wired to `decide_pair` offer **no extension point for `l2-*`**. The corpus half
  is an isolated test reusing `l2_corpus()`'s own `join`-inversion, exactly as story 6.6 did — and
  the story must say so, because a developer will otherwise try to move `l1_answers` and find out the
  hard way. The unanswerable bucket does not move here; **6.15 closes it.**
- ⚠️ **The `ObsId → &Observation` resolution belongs to the CALLER, not to `l2.rs`.** Starting from
  an `L2CandidatePair` and `join`'s `BTreeMap<L1Key, BTreeSet<ObsId>>`, someone must map ids back to
  observations against the original slice. `l2_corpus()` already does it internally. That someone is
  story **6.12**; `l2.rs` takes the observations it is handed.
- No migration, no route, no screen, no dependency, no fixture change.

### References

- `epics.md:1848-1864` — the three criteria, verbatim above.
- `architecture.md:1373` — D20, and `:1409-1412` for the common bug.
- `fixtures/scenario/traps/hostname-absence.toml` header — the absence/emptiness equivalence, in the
  corpus's own words.
- `fixtures/scenario/traps/cloned-mac.toml` — D21's *"a cloned MAC = two real interfaces, same MAC"*.
- Story 6.6's `blocking.rs` module doc and §0f/§0j — the type decision this story inherits.
- `deferred-work.md`, story 6.6's review rows — the `l1.rs` drift and the twinning question.

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context), 2026-08-30.

### The live count (AC5)

**783 → 796 tests** (517 bin + 180 core + 99 xtask), `cargo test --workspace --locked
--no-fail-fast`, wall clock, warm: **0.21 s with no store** and **5.76 s against a live
`mariadb:10.11.11` on port 13369, created virgin for this pass** — the clock is the tell. **Ten
gates green**; `float-free` now walks **5** files under `identity/` where it walked 4. Clippy over
`--all-targets`. 28 fixtures, **trap gate still RED at 26/15/11**, no migration, no route, no write,
no dependency, no production caller.

### What was built

`crates/opencmdb-core/src/identity/l2.rs` — **NEW**: `L2_DIFFERENT_HOSTNAME`, `L2Side<'a>` (the
observations that landed on one interface), `hostnames_of` (trim, drop empty, ASCII-fold) and
`verdict_for_hostname` — **the first producer of `Verdict::Opposes` in this codebase**. The corpus
half is two tests in `fixtures.rs`. `l1.rs`'s three drifted citations are repaired (T9).

### 🔴 T3's red was ARRANGED, and D20's bug appeared by doing nothing in particular

The rule shipped first WITHOUT its emptiness check — `BTreeSet::is_disjoint` says **true** of two
empty sets, so *absence opposed*. **Four tests reddened on their own assertions**
(`left: Opposes, right: Neutral`), not on a compiler error. The lock is one line, and it is now the
only thing between this rule and D20's named bug:

```rust
let both_sides_offer_a_name = !names_a.is_empty() && !names_b.is_empty();
```

🔑 *The bug D20 calls the real lock is not something you write — it is what you get for not writing
one line.* That is why the guard was seen red before it passed.

### The mutation pass — six mutations, predictions written first, **virgin store and `--baseline`**

Story 6.6's rule, applied: this suite is non-deterministic against a reused database, so every run
below measured a clean baseline first.

| id | mutation | predicted | measured |
|---|---|---|---|
| M1 | remove D20's lock (oppose on absence) | red | **red 4** (+ clippy) |
| M2 | compare case-SENSITIVELY | red:1 | red 1 |
| M3 | oppose on a partial overlap (`!=` for disjoint) | red:1 | red 1 |
| M4 | drop the evidence on `Opposes` (D19) | red | red 2 |
| M5 | misspell the rule id | red:2 | red 2 |
| M6 | return `Neutral` always | red | red 3 |

**Six for six.** ⚠️ **M5's SECOND half is not executable on this tree, and that is §0i's finding
rather than an omission**: the trap-path measurement — a misspelled id leaving the trap PASSING —
needs a path from this rule to `run_trap`, and `l1_runner` has no extension point for `l2-*`
(measured at validation). What ships instead is stronger than a re-run: **the reason is now an
executable test.**

### 🔑 §0i's finding is PINNED rather than quoted

`an_opposes_only_verdict_abstains_and_names_no_rule` drives the real algebra and asserts both halves:
`decide(vec![an Opposes])` gives `Abstained { AbsenceOfProof }`, and `decision.rule()` is **`None`**.
*That is why a misspelled id is invisible to the gate*, and why AC3 ships on a double literal. The
sentence is no longer inherited from a validation report — it fails if D13's arbitration ever
changes.

### AC1 ships MET on two traps of three, and the third is NAMED

`the_answerable_hostname_traps_are_opposed_and_the_third_is_named` asserts the two answerable ids by
name AND asserts that `cloned-mac-must-not-merge` is the collapsed one. **A second trap falling into
this case reds a test instead of vanishing** — which is the whole reason for naming rather than
counting, and the cost Guy's arbitration (a) explicitly accepted.

### The two decisions the corpus cannot exercise, both guarded and both limited in writing

**Case** (§0d2, mine): ASCII-folded, so `NAS-01` and `nas-01` are one name — M2 reds when it is not.
**Multi-name sets** (AC4): `Opposes` only on DISJOINT non-empty sets, so a partial overlap stays
quiet — M3 reds when it does not. Neither has a committed trap; both have a synthetic guard and a
doc paragraph saying so.

### T9 — `l1.rs`'s citations, re-derived rather than shifted

`:984-986` → `:1009-1011` and two `:984-985` → `:1009-1010`, each found by `grep` on the quoted
sentence. The file carries a note saying what it used to say and that the drift was **inherited
rather than measured** — the same defect `blocking.rs` carried and story 6.6 repaired.

### File List

- `crates/opencmdb-core/src/identity/l2.rs` — NEW
- `crates/opencmdb-core/src/identity/mod.rs` — MODIFIED (module declaration)
- `crates/opencmdb-core/src/identity/l1.rs` — MODIFIED (T9, citations only)
- `crates/opencmdb-bin/src/fixtures.rs` — MODIFIED (the corpus half)
- `_bmad-output/implementation-artifacts/6-7-l2-different-hostname.md` — MODIFIED
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — MODIFIED

## Change Log

| Date | Change |
|---|---|
| 2026-08-30 | Story created and contexted against `db1e3f9`. 🔴 **AC1's "three traps" is TWO** — `cloned-mac-must-not-merge` has no L2 pair under story 6.6's arbitration (§0b). 🔑 **And the measurement opened a third option nobody had posed**: the ONE interface in the whole corpus carrying two different hostnames is exactly the one `cloned-mac` collapses onto, so *the signal is not lost, the SHAPE is* — a cloned MAC presents as one interface contradicting itself, which is story 6.11's structural-fact shape arriving early (§0c). Arbitration left OPEN and referred to Guy. Also measured: 17 hostname-bearing interfaces, exactly one multi-named (§0d); the absence/emptiness equivalence is stated by the corpus itself and **no trap in that family names this rule**, so AC2 needs a test that reds when the rule wrongly opposes (§0e). |
| 2026-08-30 | ✅ **ARBITRATION TAKEN — OPTION (a), GUY.** `cloned-mac-must-not-merge` stays unanswerable at L2; the bucket goes **11 → 4**; **(c), the structural reading, is registered by name to story 6.11**, whose subject it already is. 🔑 *The trap is not unanswerable because the engine is weak — it interrogates the wrong layer.* (b) was refused knowing it would never be cheaper. ⚠️ The accepted cost is written: a committed trap is read by nothing until 6.11, and **T6 asserts by NAME which one**, so a second such trap reds a test instead of vanishing. |
| 2026-08-30 | **VALIDATED** by two fresh-context layers, each in its own worktree; the gap-hunt layer BUILT the rule against a live store and ran four mutations with `--baseline`. **Ten findings, three HIGH, all applied.** 🔴 **AC3's stated mechanism is structurally unreachable**: `>= 1 Opposes` alone abstains on `AbsenceOfProof` (D13, issue #54), an abstention carries no rule, so a misspelled id leaves the trap **PASSING** — measured. The first draft's mutation for it was **GREEN**. AC3 now ships with a double-literal carrier, and **this story is where a hole dormant since 5.4b first bites, for want of a producer until now**. 🔴 **Combining the L1 and L2 verdicts for one pair makes this rule INVISIBLE** — any valid L2 pair has distinct L1 keys, so L1 always says `Disqualifying` and `decide` gives it absolute priority: measured `NoMatch { rule: "l1-distinct-mac" }`. T2 now carries the warning, before 6.12 wires the pass. 🔴 **Hostname CASE was a decision nobody had taken and the default is D20's forbidden shape**: `NAS-01` vs `nas-01` opposes. Taken (mine, delegated): ASCII case-insensitive, with the limit written. Also: AC2 was **not verbatim** — a longer D20 quotation had silently replaced the epic's inside the AC block, under a header promising verbatim and without AC1's divergence note; the corpus half **cannot** go through `l1_runner`, measured; and the **17** interface count is the literal reading, **14** under §0e's own semantics. |
| 2026-08-30 | **IMPLEMENTED.** `l2.rs` ships `l2-different-hostname`, **the first producer of `Verdict::Opposes`**. **783 → 796 tests**, ten gates, `float-free` now over 5 files. 🔴 **T3's red was arranged and D20's bug appeared by doing nothing**: without the emptiness check, `is_disjoint` says true of two empty sets and absence OPPOSES — four assertion-carried reds, closed by one line. **Six mutations, six conforming, every one on a virgin store with `--baseline`.** ⚠️ M5's trap-path half is **not executable here** (no path from an `l2-*` rule to `run_trap`), so §0i's finding is **pinned as an executable test instead**: an `Opposes`-only verdict abstains and `decision.rule()` is `None` — the reason a misspelled id is invisible, now failing if D13's arbitration changes. AC1 ships MET on two traps of three with the third **named**, so a second such trap reds a test. T9 repaired `l1.rs`'s three drifted citations by grep on the sentences. |
