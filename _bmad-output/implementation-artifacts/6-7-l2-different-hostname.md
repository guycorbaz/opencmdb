# Story 6.7: `l2-different-hostname` — the first producer of `Opposes`

Status: **ready-for-dev** — contexted 2026-08-30 against the committed corpus and the tree at
`db1e3f9`; **arbitration TAKEN by Guy the same day**. ⚠️ **`create-story validate` (two fresh-context agents) is MANDATORY before `dev-story`**
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
**Then** it yields `Neutral`, never `Opposes`. ⚠️ **D20 names this as the common bug**
[`architecture.md:1409-1412`]: *"the rule that wrongly `Opposes` should return `Neutral` — it does
not KNOW, it BELIEVES it knows. **This is the real lock: nine parasitic abstentions out of ten are
that.** Weighting is almost always the wrong fix for a wrong verdict — a rule that claims to know
what it does not know IS the bug; the weight merely masks the lie by attenuating it."* The
`hostname-absence` family exists to catch it.

**AC3 — the spelling**

**And** the rule id is spelled exactly as the corpus spells it, or the trap reds as `rule_mismatch`.

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

- [ ] **T1 — Validation.** Two fresh-context agents, own worktree each. §0c is SETTLED (Guy, option
      (a)); the validation inherits it rather than re-opening it.
- [ ] **T2 — The rule's input type** (AC1): an interface with its observations. Decide it explicitly
      and document why an `L2CandidatePair` alone cannot serve.
- [ ] **T3 — The absence guard FIRST** (AC2): write the `Neutral`-on-absent and `Neutral`-on-empty
      tests against a deliberately wrong rule that opposes on absence, observe the assertion-carried
      red, then correct. This is D20's lock and it is the story's centre.
- [ ] **T4 — The rule** (AC1, AC3): `l2-different-hostname`, spelled exactly as the corpus spells it,
      yielding `Opposes` on disjoint non-empty name sets and `Neutral` otherwise.
- [ ] **T5 — The multi-hostname decision** (AC4), tested, with its limit stated in the doc.
- [ ] **T6 — The corpus half**: the two answerable traps answered end to end; **assert by NAME that
      `cloned-mac-must-not-merge` is the one that is not**, so the residue cannot grow in silence.
- [ ] **T7 — The mutation pass**, predictions written first, virgin store, `--baseline`. At minimum:
      oppose on absence (must red T3), oppose on empty, oppose on partial overlap, misspell the rule
      id (predict `rule_mismatch`), and one prediction of GREEN named as a limit.
- [ ] **T8 — Gates**: `cargo xtask ci`, fmt, clippy `--all-targets`, `cargo test --workspace
      --locked --no-fail-fast` both with and without a store; record the clock.
- [ ] **T9 — `l1.rs`'s citation drift**, inherited by name (§0f).
- [ ] **T10 — The record**: this file, `sprint-status.yaml`, and the twins **byte-for-byte identical**
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
- `crates/opencmdb-bin/src/fixtures.rs` or `l1_runner.rs` — the corpus half (D47: core reads no
  files).
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

_(to be filled at implementation)_

## Change Log

| Date | Change |
|---|---|
| 2026-08-30 | Story created and contexted against `db1e3f9`. 🔴 **AC1's "three traps" is TWO** — `cloned-mac-must-not-merge` has no L2 pair under story 6.6's arbitration (§0b). 🔑 **And the measurement opened a third option nobody had posed**: the ONE interface in the whole corpus carrying two different hostnames is exactly the one `cloned-mac` collapses onto, so *the signal is not lost, the SHAPE is* — a cloned MAC presents as one interface contradicting itself, which is story 6.11's structural-fact shape arriving early (§0c). Arbitration left OPEN and referred to Guy. Also measured: 17 hostname-bearing interfaces, exactly one multi-named (§0d); the absence/emptiness equivalence is stated by the corpus itself and **no trap in that family names this rule**, so AC2 needs a test that reds when the rule wrongly opposes (§0e). |
| 2026-08-30 | ✅ **ARBITRATION TAKEN — OPTION (a), GUY.** `cloned-mac-must-not-merge` stays unanswerable at L2; the bucket goes **11 → 4**; **(c), the structural reading, is registered by name to story 6.11**, whose subject it already is. 🔑 *The trap is not unanswerable because the engine is weak — it interrogates the wrong layer.* (b) was refused knowing it would never be cheaper. ⚠️ The accepted cost is written: a committed trap is read by nothing until 6.11, and **T6 asserts by NAME which one**, so a second such trap reds a test instead of vanishing. |
