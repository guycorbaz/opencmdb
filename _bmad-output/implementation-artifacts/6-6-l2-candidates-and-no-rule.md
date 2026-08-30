# Story 6.6: L2 candidate generation, and no rule

Status: **ready-for-dev** — contexted 2026-08-30 against the committed corpus and the tree at
`8a19089`. ⚠️ **`create-story validate` (two fresh-context agents, fact-check + gap-hunt) is
MANDATORY before `dev-story`** (Guy's decision, Epic 4 retrospective) and has NOT been run.

⚠️ **ONE ARBITRATION IS OPEN and is named in §0f.** It is where the L2 blocker's code lives and what
its input type is. A recommendation is given with the option refused and the measurement behind it;
it must be settled at the validation pass, not discovered during implementation.

## Story

As the next developer,
I want the set of interface pairs that could be one device, computed by something that consults no rule,
So that a blocker cannot become the echo of the rule it feeds.

## Acceptance Criteria

*(Source: `epics.md:1830-1846`, quoted verbatim. Everything beyond it is §0's, and §0 says which.)*

**AC1 — the shape, and the refusal that defines it**

**Given** a population of interfaces
**When** L2 candidates are generated
**Then** the result is a set of unordered pairs of distinct interfaces, and the generator **calls no
`l2-*` rule and no `decide`** — story 5.6's rule, and its reason: *a blocker that consults a rule is
that rule's echo.*

**AC2 — the recall floor, in integers**

**Given** the committed trap corpus
**When** the L2 recall is measured
**Then** it is asserted against D13's floor in **milli-units** (`u32`), never a float — the
`float-free` gate walks `identity/` and must stay green.

**AC3 — the measurement must be able to fail**

**And** the measurement is a real one: story 5.6 found that blocking on the MAC scores 700‰ and on
the hostname 400‰, so **the recall assertion must be able to fail.**

**AC4 (this story's, from §0d) — the L2 denominator is ASSERTED, not inherited**

**Given** the L2 truth set
**When** the recall test runs
**Then** the test asserts its own denominator by value before computing anything, because at
**three** required pairs a denominator that shrinks in silence is a gate that quietly stops testing —
`blocking_recall_above_999` already does exactly this at L1 (`fixtures.rs:4748`) and its comment says
why.

**AC5 (this story's, from §0e) — the corpus's BLIND narrowings are named and covered synthetically**

**Given** that two of the four candidate narrowing keys score **1000‰ over the whole committed
corpus** (measured, §0e)
**When** the story ships
**Then** each blind narrowing carries a SYNTHETIC test that reds under it, written BEFORE the
production code, on story 5.6's precedent (`two_l2_domains_are_still_a_candidate_pair` exists for
exactly this reason and was written first).

**AC6 (this story's) — the live count**

The workspace test count and the gate count are recorded IN THIS FILE, each figure naming the state
it was measured on. The project carries no live count anywhere else (`CLAUDE.md`'s rule).

## §0 — What contexting MEASURED

Everything in this section was run against the tree at `8a19089`, working tree clean. Figures that
were read rather than run say so.

### §0a. The two premises the sprint file got wrong, and one of them was mine

🔴 **Story 6.6 is NOT "the first producer of `Verdict::Opposes`".** That is story **6.7**
(`l2-different-hostname`). `sprint-status.yaml` writes each story's note on the line **above** its
key, and the note was read as belonging to the key above it. Settled by reading three consecutive
entries: the note above `6-8-l2-uplink-agrees` says *"the FIRST producer of `Verdict::Supports`"*,
and `epics.md:1866` confirms 6.8 is `l2-uplink-agrees`, so the notes precede their keys. Story 6.6's
own note is `# 5.6's rule: a blocker that consults a rule is that rule's echo; recall in milli-units`
(`sprint-status.yaml:4826`). **This story produces no verdict at all.**

⚠️ The error was carried into a session memory before it was caught; the memory is corrected.

🔴 **The `must-merge` truth set is ELEVEN, not ten.** `CLAUDE.md` says *"10 `must-merge`"* — story
5.6's figure, true on 2026-08-01 and stale since story 5.13b added the blinded-source pair.
`fixtures.rs:4752-4753` asserts `11` today. **The L1 figure is not the L2 one** — see §0c.

### §0b. What exists, measured — the blocker at L1 and what it refuses

`crates/opencmdb-core/src/identity/blocking.rs`, 260 code lines before its `#[cfg(test)]` (total
661). It ships:

- `CandidatePair` — private fields ordered by `new`, so `new(a, b) == new(b, a)` holds **by
  construction**; `new(a, a) -> None`, which closes the self-pair **in the type**.
- `candidates(&[Observation]) -> BTreeSet<CandidatePair>` — **TOTAL by decision**: every unordered
  pair of distinct `obs_id`s, no narrowing key.
- `BLOCKING_RECALL_FLOOR_PER_MILLE: u32 = 999` and `blocking_recall_per_mille`, integers on D13's
  own milli-units corollary.

Its module doc states the refusals this story inherits **verbatim and must not weaken**: it *"calls
neither `join` nor `decide_pair`"*, *"consumes no structural reading of a MAC (the U/L bit, the IANA
prefixes, the I/G bit) and reads no `Fact` at all"*, and *"writes nothing and reads nothing but its
argument"*.

⚠️ **A blocker's recall test is green by CONSTRUCTION when the universe is total.** At L1 AC3's
ancestor was discharged by MUTATION, never by a corpus that can fail on the shipped code — M1
(blocking on an L1 key) scores 700‰ and M2 (blocking on `l2_domain`) leaves the whole corpus GREEN.
**Expect the same shape here and plan for it (§0e), rather than meeting it during implementation.**

### §0c. 🔴 THE L2 TRUTH SET IS **THREE** PAIRS, and each was verified to be a real interface pair

Measured by walking every committed trap file, keeping the `must-merge` traps whose expected rule
starts with `l2-`, and resolving each named observation to its L1 key:

| trap id | family | expected rule | two DISTINCT L1 keys? |
|---|---|---|---|
| `multi-nic-must-merge` | multi-nic | `l2-uplink-agrees` | **yes** |
| `shared-hardware-vm-must-merge` | shared-hardware-vm | `l2-hostname-agrees` | **yes** |
| `docker-veth-must-merge` | docker-veth | `l2-uplink-agrees` | **yes** |

🔑 **The premise had to be checked and is not obvious**: had the two observations of a pair landed on
the SAME L1 key, they would be one interface and there would be no L2 pair to require at all. All
three are confirmed distinct-key pairs — distinct MACs inside one `l2_domain`.

⚠️ **The second `l2-uplink-agrees` trap is in `docker-veth`, not a second one in `multi-nic`.** A
count taken from the rule tally alone (2 × `l2-uplink-agrees`) would have named the wrong family;
the families were resolved by walking the files.

⚠️ **`>= 999‰` at a denominator of THREE is zero-tolerance**: one miss scores **666‰** and the floor
reds. That is the binary form NFR4 demands. `deferred-work.md:1874` already carries the boundary
arithmetic (`>= 1000` required pairs is where the constant becomes a real tolerance) and this story
does not approach it — do not "fix" the constant.

### §0d. The L1 truth set and the L2 truth set are DIFFERENT SETS, and the difference is the story

`fixtures.rs`'s `corpus_pairs()` collects **every** `must-merge` trap into `required`, rule-agnostic
— eleven pairs. Eight of them expect `l1-exact-mac`: two observations of **one** interface. At L2
those eight are not pairs at all; they collapse to a single interface.

🔴 **So an L2 recall computed over `required` would be measuring the wrong population**, and it would
look plausible: eleven is a bigger, more reassuring denominator than three. The L2 truth set is the
three rows of §0c, and AC4 exists so the number is asserted rather than inherited.

### §0e. 🔴 THE MEASUREMENT AC3 ASKS FOR — four narrowing keys, run over the committed corpus

Each row narrows the universe by a key and re-measures L2 recall over the three required pairs.
Run 2026-08-30; the script rebuilds `join`'s keying (`(l2_domain, mac)`, one key per MAC) and the
per-stream universe of unordered distinct-interface pairs.

| narrowing key | L2 recall | verdict |
|---|---|---|
| **TOTAL** (no key — the shipped shape) | **1000‰** | the floor holds |
| same `l2_domain` | **1000‰** | 🔴 **the corpus is BLIND** |
| agreeing uplink `peer_mac` | **1000‰** | 🔴 **the corpus is BLIND** |
| agreeing uplink `peer_mac` + `peer_port` | **666‰** | reds |
| agreeing hostname | **333‰** | reds |

🔑 **The two blind rows are the story's centre.** *Block on the uplink* is the most tempting L2
narrowing there is — it is literally the signal `l2-uplink-agrees` scores on, one story later — and
**it passes the entire committed corpus**. A blocker narrowed on it would be the echo AC1 forbids,
and no committed trap would say so. This is story 5.6's `l2_domain` finding, one level up and with a
sharper temptation.

**Therefore AC5**: each blind narrowing gets a synthetic test written FIRST — two interfaces that
must remain a candidate pair *although* they sit in different L2 domains, and two that must remain a
pair *although* their uplinks disagree (a device dual-homed into two switches, which is precisely the
multi-NIC shape D12 makes the product's promise about).

⚠️ **The two non-blind rows are the mutations AC3 wants** and their predicted scores are written
above, measured, not guessed. Record the observed score beside the prediction; a divergence is a
finding.

### §0f. ⚠️ THE OPEN ARBITRATION — where the code lives, and what it takes

**Recommended: extend `blocking.rs`; do NOT create `l2.rs` in this story.**

`blocking.rs` is the module whose whole subject is candidate generation, and its doc already promises
the refusals AC1 restates. Putting the L2 blocker beside the L1 one keeps *one home for blocking* and
leaves `l2.rs` to be created by story 6.7 for the **rules** — which is what makes *"the blocker
consults no rule"* structurally visible rather than merely asserted: the rules are not in the file.
Size is not a constraint (260 code lines against a 2000 ceiling). `BLOCKING_RECALL_FLOOR_PER_MILLE`
is shared, which is right: D13 gives one floor.

**Refused: a new `l2.rs` holding the blocker now.** It reads as the natural mirror of `l1.rs`, but
`l1.rs` holds RULES and the join; a file mixing story 6.6's blocker with story 6.7's first rule is
the shape that lets a later edit make the blocker read a verdict without any reviewer noticing a
boundary was crossed.

**Input type — recommended `&[L1Key]`** (`pub type L1Key = (L2DomainId, MacAddr)`, `l1.rs:89`),
mirroring `candidates(&[Observation])`. At L1 an interface **is** an `L1Key`: `join` returns
`BTreeMap<L1Key, BTreeSet<ObsId>>` and `resolver.rs`'s doc says *"`join` NAMES the interface"*. A
caller passes `join(&observations).keys()`. ⚠️ **Not `InterfaceId`**: that is a database id
(`observation/mod.rs`), and D47 forbids `opencmdb-core` to touch the store, so a core function keyed
on it could not be measured against the corpus at all — which is where AC2's whole measurement lives.

⚠️ **A slice admits duplicates where `join`'s keys cannot.** Decide it explicitly and test it:
duplicates collapse (the output is a set), on the precedent of story 5.6's repeated-`obs_id` rule,
which is *"one rule, named and tested, not a narrowing"*.

### §0g. What this story does NOT do, stated so nobody discovers it

- ⚠️ **No production caller.** Nothing hands the L2 blocker a population; story **6.12** (the resolver
  writing device groupings) is the first that will. The L1 blocker lived five stories in exactly this
  state and the register says so (`deferred-work.md`: *"nothing calls the blocker and the engine in
  sequence"*).
- ⚠️ **No verdict, no rule, no `Decision`.** `Verdict::Supports` and `Verdict::Opposes` still have no
  producer after this story — 6.7 and 6.8 are where that changes.
- ⚠️ **The trap gate stays RED at 26/15/11.** This story routes nothing and answers no trap. The
  bucket moves with 6.7–6.11 (11 → 3) and closes at **6.15**, which is what `epics.md` has always
  said. Do not report an improvement the gate does not show.
- ⚠️ **D17's `dormant` exclusion is NOT implemented here.** `deferred-work.md` records it measured:
  there is no lifecycle state and no field a blocker could read; the owner is the lifecycle epic. Do
  not invent the state in order to filter on it (D45).
- ⚠️ **`opencmdb-core` gains behaviour**, so the usual *"byte-identical"* claim does not apply.
  Narrow the promise to *no behaviour change elsewhere in the crate*, on story 5.13b's finding that
  *a promise of non-modification protects behaviour and shelters false sentences.*

### §0h. Gates and the house rules that bite here

- **`float-free`** walks `crates/opencmdb-core/src/identity/` — **four files today, five if a new one
  lands**. A float literal or a float type anywhere in the new code reds it. The recall is `u32` in
  per-mille; the per-mille arithmetic is integer division, and the test's own expected values are
  integers.
- **`file-size`** (2000 code lines before the first `#[cfg(test)]`): `blocking.rs` is at 260 — ample.
- **Rustdoc on every `pub` item**, including fields and variants, and **a doc comment must be TRUE**:
  prefer the weaker true sentence. `opencmdb-bin` and `xtask` carry `#![deny(missing_docs)]`;
  `opencmdb-core` does not yet, so the compiler will NOT name a missing doc here.
- **Prove-to-red**: every guard is observed failing before it passes, and the mutation is recorded.
- **`cargo xtask mutate`** exists since story 6.4b — use it rather than a throw-away script, and
  read its three-way exit contract (`0` matches the prediction · `1` contradicts it · `2` could not
  honestly measure). ⚠️ It does **not** drive the two browser gates; irrelevant here (no screen).
- ⚠️ **`cargo test --workspace` stops at the first failing crate** — pass `--no-fail-fast` or every
  recorded red is a lower bound (story 6.4b's finding).
- ⚠️ **CI compiles the tests under `-D warnings`; local `cargo clippy --workspace` does not.**
  Run `cargo clippy --workspace --all-targets -- -D warnings` before pushing.
- ⚠️ **Never read a status through a pipe** (`cargo test | grep` takes the pipeline's status). Two
  commits went in over a red suite that way.

## Tasks / Subtasks

- [ ] **T1 — Validation first.** Run `create-story validate` with two fresh-context agents
      (fact-check + gap-hunt), each in its own worktree. Settle §0f's arbitration there. (MANDATORY)
- [ ] **T2 — Write the synthetic AC5 guards BEFORE the production code** (AC5): two interfaces in
      different L2 domains remain a pair; two interfaces with disagreeing uplinks remain a pair.
      Observe them fail to compile / fail, then pass.
- [ ] **T3 — The pair type**: unordered by construction, `new(a, a) -> None`, private fields, full
      rustdoc. Test that `new(a, b) == new(b, a)` and that the self-pair is refused. (AC1)
- [ ] **T4 — The generator**: TOTAL over the supplied population, duplicates collapse, calls no rule
      and no `decide`. Test the duplicate rule explicitly. (AC1)
- [ ] **T5 — The L2 recall**, integer per-mille, reusing `BLOCKING_RECALL_FLOOR_PER_MILLE`. (AC2)
- [ ] **T6 — The corpus assertion in `opencmdb-bin`** (D47 forbids core to read files): build the L2
      truth set from the `must-merge` traps whose expected rule starts with `l2-`, **assert the
      denominator is 3 by value** (AC4), then assert recall ≥ the floor. Keep the per-trap
      containment assertion a SEPARATE test, on `blocking_recall_above_999`'s stated reason: with
      both in one function a missing pair panics before any recall exists.
- [ ] **T7 — The mutation pass**, predictions written BEFORE any plant, each row naming its carrier:
      narrow on hostname (predict **333‰**, reds), narrow on uplink `peer_mac`+`peer_port` (predict
      **666‰**, reds), narrow on `l2_domain` (predict **1000‰ — GREEN over the corpus**, red only on
      the AC5 synthetic), narrow on uplink `peer_mac` (predict **1000‰ — GREEN over the corpus**, red
      only on the AC5 synthetic), delete the denominator assertion, delete the generator's body.
      Record observed against predicted; **a divergence is a finding, not a correction.**
- [ ] **T8 — Gates**: `cargo xtask ci` (ten gates), `cargo fmt --all`,
      `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked
      --no-fail-fast`. Record the count and the wall clock, and say whether a store was present.
- [ ] **T9 — Update the record**: this file (AC6), `sprint-status.yaml`, `docs/project-context.md`
      and `CLAUDE.md`'s status paragraph — the twins, in the same push (docs-current-before-push).
      Register anything raised in `deferred-work.md` with a named owner; ⚠️ *a section that says
      "registered" is not a registration* (story 6b.9).

## Dev Notes

### What the previous stories leave you

- **Story 5.6** is this story's direct ancestor and its module doc is the specification of the
  refusals. Read `blocking.rs`'s doc in full before writing a line; do not paraphrase it into the new
  code, cite it.
- **Story 6.5** shipped `entity` and `device` with **no producer**, and `interface` is **not** a
  subtype — its adoption is story 6.12's, *together with* the `identity_link` widening and the
  resolver change, measured as one gesture. Nothing in this story touches those tables.
- **Story 6.4b** shipped `cargo xtask mutate`. Its whole product is refusals; use it.
- 🔴 **Story 6.5's two hard-earned rules**, if any test here touches a store (none should): every
  DB-touching test takes `crate::DB_TEST_LOCK` first, and a DDL mutation needs a virgin schema.

### The defect class this epic keeps producing

***A guard placed where the defect cannot occur reads as coverage and is none.*** Counted in at least
nine of Epic 5's twenty stories and four times in one story of Epic 6b. **Reading a guard cannot find
it — the guard is correct about what it tests.** Only running the mutation does.

Its live specimen here: a recall test over a TOTAL universe cannot fail on the shipped code. AC5 and
T7 exist so the coverage is real rather than apparent, and §0e says which two narrowings the corpus
cannot see at all.

Second class: ***the mutation driver lies*** — five epics running. `cargo xtask mutate` is the answer
this project built for it; if a result contradicts a prediction, suspect the driver before the
finding.

### Project Structure Notes

- `crates/opencmdb-core/src/identity/blocking.rs` — **UPDATE** (recommended home, §0f).
- `crates/opencmdb-bin/src/fixtures.rs` — **UPDATE**, the corpus-side assertions (D47: the domain
  crate may not read files). Everything new goes in the trailing `#[cfg(test)]` module; nothing above
  it changes and no new `pub` item appears in that crate — the shape story 5.6 established.
- **No migration, no route, no screen, no new dependency, no fixture change.** The corpus is
  **28 artefacts / 26 traps across ten families** and this story does not touch it.

### References

- `epics.md:1830-1846` — the three criteria, verbatim above.
- `epics.md:1722-1738` — Epic 6's four measured constraints, notably **(1)** the corpus already NAMES
  the L2 rules and **(2)** the five rules take the bucket 11 → 3, not 0.
- `architecture.md:1004-1011` — D13 on why a blocker exists: *"nobody tests blockers"*, *"without
  blocking, abstention has no denominator"*.
- `architecture.md:988-993` — the milli-units corollary AC2 rests on.
- `architecture.md:891-894` — the L1/L2 split and the trap matrix: *"multi-NIC false-split = L1
  correct, L2 failed to group"*.
- `architecture.md:1246-1253` — D18, and why this is **not** the recall gate it refuses.
- `blocking.rs` module doc — the refusals, and the three measured L1 narrowing scores.
- `fixtures.rs:4658-4790` — `CorpusPairs`, `corpus_pairs()` and `blocking_recall_above_999`: the
  pattern T6 mirrors, including why the containment assertion is a separate test.
- `deferred-work.md` — the quadratic-universe row (owner: *Epic 6, whichever first hands the blocker
  a set of observations*), the `>= 999` boundary arithmetic, and D17's `dormant` exclusion.

### Project context reference

`docs/project-context.md` and `CLAUDE.md`. ⚠️ Both carry DATED figures beside living ones; where they
disagree with a measurement taken on the tree, **the measurement wins and the document is corrected**
— that is the project's own rule, and §0a is this story's first application of it.

## Dev Agent Record

### Agent Model Used

_(to be filled at implementation)_

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Change |
|---|---|
| 2026-08-30 | Story created and contexted. §0 measured against `8a19089`: the L2 truth set is **three** pairs (§0c), two of four narrowing keys are **invisible to the committed corpus** (§0e), and two premises inherited from `sprint-status.yaml` were **refuted** (§0a). One arbitration left open (§0f). |
