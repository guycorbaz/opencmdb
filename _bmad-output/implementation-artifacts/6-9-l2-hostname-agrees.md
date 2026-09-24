# Story 6.9: `l2-hostname-agrees`

Status: **developed and verified** 2026-09-24, awaiting its code review. Validated first by two
fresh-context layers (fact-check + gap-hunt, the latter having BUILT three readings of the rule and
run its own ten-row pass), then **both questions arbitrated by Guy on 2026-09-24** — §0.0 measured
rather than decided, §0.3 taken as set equality — then implemented, then proved to red over eight
mutations.

⚠️ *This line read "validated … ONE question for Guy, posed below and not settled here; then dev" until
the fourth instrument defect was written up, i.e. after the questions were answered, the code shipped
and the pass had run. **`cargo xtask record` cannot see it** — Epic 14's retrospective named `Status:`
as the first of the checker's three blind spots, and predicted in writing that the record defects
would migrate to exactly what it declines to read. They did, here, in the next story.*

**Epic 6**, reordered by Guy on 2026-09-23: **6.9 → 6.11 → 6.12**, with 6.8 and 6.10 waiting for a
connector that emits an uplink or a switch port. **Base:** `97e95dd` — the reorder's own
squash-merge on `master` (PR #205), which is what makes this story's promise *`epics.md` not edited*
true rather than merely written. ⚠️ *It read `e7e5143` until `cargo xtask record` refused it — the checker found a topology defect and
not a number, which is the part that held. 🔴 **The cause the note first gave was FALSE and the
geometry was misdescribed**: `e7e5143` was not on an "unpushed `plan/` branch" — I had pushed it as PR
#205's head myself, and CI ran green on it — and it is not "one commit earlier" than the branch point
but **not on `master` at all**, a squash-merge having replaced it with `97e95dd`. The acceptance-audit
layer refuted both from `gh pr view`, and §5 of this same document had the true version all along. *A
note written after the act that describes the state before it.*

⚠️ Both layers verified that `e7e5143` differs from its parent in **four documents and no code**, so
every measurement below transfers.

---

## 0. What contexting measured, what validation refuted, and the two questions Guy settled

### §0.0 — 🔴 THE HEADLINE IS EPIC-WIDE: A `Supports` RULE CANNOT ANSWER A `must-merge` TRAP

`epics.md:1922` says: *"it yields `Supports`, **and the trap expecting `l2-hostname-agrees` is
answered**"*. **Those two clauses cannot both be true**, and the gap-hunt measured it end to end over
the committed bytes:

```
### shared-hardware-vm-must-merge   expect = MustMerge { rule: "l2-hostname-agrees" }
  the rule           -> Supports, evidence [ …0001, …0002 ]
  L2 agrees alone    -> Abstained { Ambiguous }   outcome Abstained   run_trap = VerdictFail
  L2 agrees + differs-> Abstained { Ambiguous }   outcome Abstained   run_trap = VerdictFail
  L1 + L2 agrees     -> NoMatch { l1-distinct-mac } outcome Refused   run_trap = VerdictFail
```

with the isolated control that establishes it is the VERDICT VARIANT and nothing else:

```
Supports alone -> Abstained{Ambiguous}          score Fail
Decisive alone -> Match{l2-hostname-agrees}     score Pass
two Supports   -> Abstained{Ambiguous}          score Fail
```

**`decide` reaches `Conclusion::Match` through exactly one arm — `(None, Some(rule), _, false)` — so
a `Match` requires a `Decisive`.** A `Supports` alone lands on `(None, None, true, false)`, which
`architecture.md:1002` itself calls **weak evidence**, and `score.rs:276` makes `(MustMerge,
Abstained)` a **Fail** — the cell whose own test comment reads `// fail: cowardice`, D18's word.

🔴 **And it is not one trap.** Census over `fixtures/scenario/traps/`: **eleven `must-merge` traps,
eight naming `l1-exact-mac`** — which is `Decisive`, and is how they pass today — **and three naming
an `l2-*` rule**: `shared-hardware-vm-must-merge` (this story) plus `multi-nic-must-merge` and
`docker-veth-must-merge` (`l2-uplink-agrees`, **story 6.8**). `epics.md:1904` and `:1922` both
specify `Supports`; **no L2 story in the epic prescribes `Decisive` anywhere.** So under the algebra
as it stands, none of the three can ever score a pass.

🔑 **The epic did the arithmetic of the BUCKET and not of the FAILURES column.** Constraint (2) says
the five rules take the unanswerable bucket from 11 to 3 and that story 6.15 closes the rest. That is
about ROUTING. These three traps would leave the bucket and land in `failures`, so `passed()` stays
false and NFR4 cannot go green — for a reason the constraint does not name.

⚠️ **The word *"answered"* carries both readings** — *leaves the unanswerable bucket* (satisfiable)
or *scores a pass* (not) — and it is the same word Guy arbitrated on for story 6.7's
`cloned-mac-must-not-merge`. **This story does not settle it: a story may not re-scope an epic.**

✅ **GUY'S DECISION (2026-09-24), option (a): 6.9 MEASURES AND DOES NOT DECIDE.** The story asserts the
VERDICT — `Supports` on the trap's pair — and never that the trap is answered; the divergence from
`epics.md:1922` is registered **with story 6.8 named in it**, since its two traps carry the same
contradiction; and *what makes a merge at L2* goes to **Epic 6's retrospective**, which may edit
`epics.md` where a story may not. 🔑 **The reason it can wait is a measurement, not a preference:
the rule's verdict is `Supports` under every option on the table, so nothing in 6.9's code depends on
the answer — only the trap's SCORE does.**

⚠️ **Refused, and recorded with why**: making the rule `Decisive` (it would make a shared hostname as
strong as an exact MAC, so two printers carrying the factory default `doc-printer` would merge at L2 —
which `hostname-collision.toml` forbids in its own words, *"a signal that weak cannot outvote L1"*);
and settling the corpus bump now, i.e. turning the three `must-merge` traps into `must-abstain`
(defensible, and it is a PLANNING act that would be taken with no production caller and no screen,
which is exactly the state in which this project has decided things it later reversed).

🔑 **Why 6.7 looked clean and 6.9 does not**, worth writing because it is the whole asymmetry:
`(must-not-merge, Abstained)` is `score`'s **load-bearing PASS cell**, so `l2-different-hostname`'s
`Opposes` passed its traps *on an abstention*. `must-merge` has no tolerant cell.

### §0.1 — ~~THE CORPUS MAY NOT MEAN WHAT THE CRITERION MEANS~~ — conclusion CONFIRMED, stated reason REPLACED

~~`hostname-collision`'s header says the rule *"fires legitimately behind a SHARED uplink"*, which
reads as a precondition: hostname agreement AND topology corroboration. Under that reading 6.9
inherits 6.8's blocker — `Fact::Uplink` has no producer — and the reorder that put 6.9 first is
wrong.~~

**The conclusion is confirmed BY CONSTRUCTION**, the gap-hunt having built the rule and run it:

```
### hostname-collision-must-not-merge   expect = MustNotMerge { rule: "l1-distinct-mac" }
  the rule        -> Supports
  L1 + L2 agrees  -> NoMatch{l1-distinct-mac}  outcome Refused  run_trap = Pass
```

So the rule fires on **agreement alone**, loses, and the family passes. `epics.md`'s AC1 is right on
that half.

🔴 **But the REASON contexting gave was false, and the fact-check refuted it**: the two pairs are
identical on every axis this rule can read — distinct MACs in one `l2_domain`, one shared hostname —
so the `Disqualifying` short-circuit fires on **both**. It cannot be what separates the families.

🔑 **The two layers compose into the sentence neither carries alone: the short-circuit is what makes
`hostname-collision` PASS and what makes `shared-hardware-vm` FAIL.** The dissolution's own mechanism,
generalised one family over, is §0.0. *A decision explained by a false premise is one nobody can
re-derive*, and this one was headed **DISSOLVED**, which invites no re-derivation.

⚠️ **And the quotation was selective.** The clause contexting elided reads *"what separates that
family from this one is the topology corroboration, which this stream refuses to carry — no `Uplink`
fact is authored, deliberately"*, and **every factual assertion in it is measured true**: the
must-merge pair's MACs do differ, `shared-hardware-vm.jsonl` authors `Uplink` on all four
observations, `hostname-collision.jsonl` authors none. The header states a distinguisher; the story
quoted the fragment that supported its reading.

### §0.2 — ONE FUNCTION OR TWO — SETTLED by 6.7's shipped shape, not by preference

`verdict_for_hostname` returns `RuleId(L2_DIFFERENT_HOSTNAME)` on **both** its verdicts — `Opposes`
at `l2.rs:233`/`:234`, `Neutral` at `:239`/`:240`. A `Neutral` carrying its own rule's id is the
established convention, and one function whose id depends on its verdict would break it for one of
the two. **Two functions.** `decide` combines a verdict SET (`let has = |w| …iter().any(…)`), so this
needs no new algebra.

⚠️ One FACT now casts two votes, and `decide` cannot double-count — measured, two L2 rules in one
vector changed nothing. 🔑 **And D13's conflict row `(None, None, true, true)` is unreachable from
these two rules alone**, because `Supports` and `Opposes` are mutually exclusive (§0.3): worth one
sentence, because a reader will wonder.

### §0.3 — 🔑 THE ONE DECISION, with the argument that does NOT work struck

6.7 chose that `Opposes` requires the two name sets to share **nothing** — a partial overlap stays
`Neutral`, *"because a rule that opposes on a partial overlap claims to know which name is current,
and it does not"*. The mirror question is what `Supports` requires:

- **(a) set EQUALITY — share everything.** Partial overlap is `Neutral` under **both** rules.
- **(b) non-empty INTERSECTION — share anything.** Partial overlap yields `Supports`, i.e. the rule
  asserts one shared name settles it — **6.7's refused sentence read backwards**, and on D20's
  dangerous side.

~~🔑 (a) makes `Supports` and `Opposes` mutually exclusive by construction, which a property test can
assert.~~ 🔴 **REFUTED by an exhaustive sweep: 8×8 = 64 subset pairs of a three-name alphabet,
`eq_violations = 0` AND `int_violations = 0`** — and zero under the naive form and under `is_subset`
too. Exclusivity follows from *supports ⟹ shares a name ⟹ not disjoint* for **every** reading anyone
would write. It is not an argument for (a), and as written it invited a reviewer to believe this half
of the decision had been measured.

⚠️ **A second argument was inverted too**: §1's one-line lock is a **cost of (a)**, not a reason for
it — reading (b) is `!is_disjoint`, which is `false` for two empty sets, so **(b) needs no emptiness
check at all** (built and measured).

**What survives, and it is enough: the false-merge asymmetry.** A wrong `Supports` merges two
machines; *a false merge cannot be undone by looking harder* (Guy, 6.7).

✅ **GUY'S DECISION (2026-09-24): (a), set EQUALITY**, on that asymmetry and **not** on the two
arguments validation refuted — the exclusivity property (0 violations of 64 under both readings) and
the emptiness lock (a cost of (a), not a reason for it). The accepted cost is the D20 lock line that
(b) would not need, and **AC7 is what makes the decision reversible at the right price**: without a
partial-overlap population, the refused reading ships green.

🔴 **AND THE DECISION IS CARRIED BY NOTHING.** The gap-hunt built the AC set exactly as contexting
prescribed it and then substituted the refused reading:

| mutation | predicted | measured |
|---|---|---|
| **M2** — reading (b), `!is_disjoint` | red | **🔴 GREEN, 0 red** |
| **M7** — `is_subset`, a third plausible spelling | red | **🔴 GREEN, 0 red** |

**the whole suite green with the refused reading shipped** — the gap-hunt's own worktree reported
750/198/110, ⚠️ *a figure matching NO commit in this repository (the base is 1 046 and the shipped tree
1 060), cited as a flat measurement in three places until the acceptance-audit layer could not find its
tree. It is the VALIDATION layer's count and is named as such now* — story 5.14's *a validation
worktree's figures copied in as if they were this tree's*. *A decision a story takes out loud and no
guard can red is a decision the next story reverses silently.* The discriminating populations
are a **partial overlap** (`{doc-old, doc-x}` vs `{doc-x}`) and a **superset/subset** pair — neither
is in the committed corpus and neither was in contexting's AC list. **AC7 is what that earns.**

⚠️ **A forward dependency nobody had named**: an `L2Side` is *the observations that landed on one
interface*, and **what bounds that set is story 6.12's plumbing, not this module**. The corpus already
holds a side with two sightings an hour apart. Under (a), one rename silences the rule for that
interface **permanently** once a side spans more than one sweep (`{old, new}` never equals `{new}`);
under (b) it still supports. **§0.3 and 6.12's scoping are the same decision taken a story apart** —
registered, owner 6.12.

---

## 1. Measured facts

- **Exactly ONE trap names the rule**: `shared-hardware-vm-must-merge`, verified by a walk over all of
  `fixtures/scenario/traps/` (`found == 1`).
- ✅ **Its pair resolves to TWO DISTINCT interfaces** — `join` gives two `L1Key`s, the
  locally-administered MACs differing — so **6.7's collapse does not recur here**, and the new test
  asserts it rather than assuming it. ⚠️ *The collapsed set is `["cloned-mac-must-not-merge"]` only
  WITHIN the `l2-different-hostname` guard's population, which is where that assertion lives; unscoped
  it is false — **nine of the 25 pairs collapse**, the eight `l1-exact-mac` must-merge traps and
  `cloned-mac`, which independently reproduces story 6.6's own measurement.* Said in one line because 6.7 was bitten
  by the opposite.
- 🔴 **THE D20 MIRROR IS LOUD IN THE CODE AND INVISIBLE TO THE CORPUS.** `BTreeSet` compares **equal**
  when both sides are empty, so the naive `names_a == names_b` **SUPPORTS ON ABSENCE**. Measured over
  all 26 trap-named pairs: **8 flip to `Supports` on total absence**, six of them `must-not-merge`
  traps. ⚠️ **And not one of the eight is caught by the corpus** — ⚠️ *but the reason first given covers only
  SIX of them.* `(must-not-merge, Abstained)` being `score`'s **pass** cell accounts for the six
  `must-not-merge` traps; `hostname-absence-must-abstain` escapes through a different cell
  (`(MustAbstain, Abstained) => Pass`, and `score` ignores the cause) and `multi-nic-must-merge` is a
  **must-merge**, where an abstention is a FAIL. 🔑 **What actually makes all eight invisible is that
  there is no L2 runner at all** — which this story says elsewhere, so the sentence was true and its
  stated reason did not reach its own population. *The bug is carried by synthetic guards alone.* ⚠️ Under reading (b) there is no bug and no lock —
  §0.3.
- 🔑 **The stakes are not symmetric with 6.7's**: a false `Opposes` leaves two rows where one belongs
  and the operator can act; a false `Supports` merges two machines.
- 🔴 **6.9 IS THE FIRST PRODUCER OF `Verdict::Supports`** — no production producer exists today
  (`l1.rs:488` **panics** on it, inside `#[cfg(test)]`). Contexting never said so, and **three
  sentences go stale**, one of which is **already false**:
  - `cascade.rs:13-14` — *"`Verdict::Supports` and `Verdict::Opposes` still have no producer"* —
    **FALSE for `Opposes` since story 6.7**, live on `master`, in the file §0.1 quotes.
  - `scan_pass.rs:760` — same sentence, same staleness.
  - `epics.md:1894`/`:1904` — *"Story 6.8 … the first producer of `Supports`"*. The reorder moves it
    here; `epics.md` is not edited and the divergence is **registered**.
- **`hostnames_of` is already `pub`** (`l2.rs:152`) and carries the hard part: `trim`, the
  ASCII-alphanumeric property, `to_ascii_lowercase`, and the deliberate blindness to
  `HostnameSource`. 6.9 **reuses it** — a second extractor would let the two rules disagree about what
  a name is.
- **`Fact::Hostname` has ONE production emit site**, not three: `arp_ping.rs:450`, inside
  `emitted_facts`, behind one `if let Some(name)`. The other two occurrences are a test and a doc
  comment. `Fact::Uplink` has **zero**, which is why 6.8 and 6.10 wait.
- **The reach floor** is `prd.md:880`, verbatim and dated (reference device, 2026-07-16): hostname
  *"unusable on nearly half of known clients (absent on 71, empty string on 11, and never null)"*.
  🔴 **That is a PER-CLIENT figure describing a PER-PAIR rule**: this rule needs **both** sides to
  carry a name, so availability enters **squared** — on the reference LAN, 32 resolved of 46 answering
  ≈ 70 % per interface ≈ **49 % per pair**.
- **`l2.rs` is at 255 code lines** by the gate's own rule (`#[cfg(test)]` at `:256`), ceiling 2000 —
  no split.
- ⚠️ **`L2_HOSTNAME_AGREES` the CONSTANT exists nowhere; the STRING already appears at four sites**
  (`cascade.rs:902`, `l1_runner.rs:583`, `trap_gate.rs:1192`, `fixtures.rs:4148`), all hand-authored
  test literals keyed on the corpus TOML — **none would red on a constant typo** (measured). So
  *"two independent literals"* is really *one constant among five literals*, and a prediction written
  off *"the corpus already spells it"* mis-names the carriers.
- **There is NO L2 runner** — `l1_runner.rs` selects on `L1_PREFIX = "l1-"`, a prefix and never a
  list — so the trap gate stays **26 / 15 / 11**, RED by design. 6.9 answers its trap through a
  corpus-driven test in `fixtures.rs` mirroring 6.7's, never through the gate. ⚠️ **And §0.0 is a
  SECOND, independent reason the trap is not "answered" — the one that survives the day the runner
  exists.**

---

## 2. Acceptance criteria

**AC1 — the rule fires `Supports` on the trap's pair, driven end to end over the committed bytes**,
and the test carries a **terminal naming assertion** (`supported == ["shared-hardware-vm-must-merge"]`)
rather than a bare loop. 🔑 *Measured why*: under a corrupted constant the corpus walk iterates **zero
times**, so without that assertion AC4 becomes the sole carrier (M4 red 2 → control C2+M4 red 1).
🔴 **It does NOT claim the trap is answered** — §0.0 — and the divergence from `epics.md:1922` is
registered with Guy's arbitration attached.

**AC2 — absence yields `Neutral`, never `Supports`**, with the lock **seen red before it passes**.
⚠️ **Four guards, and the story says which THREE carry D20's lock**: both sides silent, invisible
characters only, and **pure punctuation** (the property's job, and contexting named invisibles alone).
🔴 **The asymmetric guard — one side names, the other is silent — CANNOT red on this mutation** and is
not claimed as a carrier: under (a) `{doc-a} ≠ {}` already. **Measured: M1 reds 3 of the 4.**

⚠️ *This read "three guards … which two … M1 reds 2 of the 3", and the `l2.rs` doc beside it said "its
two neighbours" — figures inherited from the validation layer's tree, where the punctuation guard did
not yet exist, while §4's table recorded `red 3` all along. **One count published three incompatible
ways in one story**, reconciled by the blind review layer reproducing the mutation on paper.*

🔴 **AND THE INVISIBLE GUARD COULD NOT FAIL ON THE PROPERTY IT NAMES.** It carried U+200B against
U+2062 — story 6.7's population, mirrored mechanically — and for EQUALITY that is the wrong shape: two
DIFFERENT invisible strings give unequal sets. Measured with both neighbours as controls, the
ASCII-alphanumeric property deleted from `hostnames_of`: **invisible GREEN, punctuation RED, absent
GREEN**. 🔑 *Disjointness wants two different non-names; equality wants the same non-name twice — the
mirror of a rule needs the mirror of its population, and mirroring the code while copying the data is
how a guard ends up unable to fail.* Both sides carry the same code point now, and **M9 is the
mutation the story owed**, since no row had touched `hostnames_of` at all.

**AC3 — the corpus's own temptation, and it is this story's best test.** On
`hostname-collision-must-not-merge` the rule fires `Supports`, loses, and the family still passes.
🔴 **The test asserts the RULE's verdict and must NOT build a mixed L1+L2 verdict vector** — `l2.rs`'s
own header forbids it (*"combining L1's verdict for the same pair with an L2 rule's … erases this
level"*) and `deferred-work.md` registers it with **owner 6.12**. ⚠️ Contexting prescribed exactly
that forbidden gesture. ~~And a mutual-exclusivity property.~~ **Struck: 0 violations of 64 under
both readings** (§0.3), and under every mutation built it is never the sole carrier.

**AC4 — the VERDICT names the rule, not merely the constant being spelled right.** 🔴 *Measured on the
VALIDATION layer's tree (750/198/110, which is NOT this one)*: with the `Supports` arm emitting the
other rule's id, the only red came from an assertion contexting did not prescribe, and removing it
shipped the wrong id green. ✅ **On the SHIPPED tree (747/203/110) M8 reds TWO** — the synthetic rule
assertion and the corpus walk's — so the criterion has two carriers here where the validation's tree
had one.

⚠️ *Both figures were first written side by side, in one tense, under one mutation id. That is this
project's recorded class* — a mutation named for one thing and applied to another, here **one id
reporting two trees**; the blind review layer noticed the counts could not both describe one tree.
Each figure now names the tree that produced it, on story 6.7's rule that *a mutation count is dated
by the tree that produced it*.

The double-literal pin stays, on 6.7's precedent, **and its reason is re-measured rather than
inherited**: 6.7's mechanism was `Opposes` → `AbsenceOfProof`, this one is `Supports` → `Ambiguous`,
two **different rows** of the table, and both carry no rule, so `run_trap`'s `(Some, Some)` never
fires.

**AC5 — the reach is STATED with its measurement**: `prd.md:880`'s dated figures, **squared** for a
per-pair rule (§1), and the sentence that this silence is data availability. ⚠️ *Two populations sit in
one sentence and the audit caught it: the ~70 % per interface is the **reference LAN's** 32-of-46, not
`prd.md`'s own denominator — under which availability is ~52 % per interface and **~27 % per pair**,
roughly half the figure quoted. Both readings support the criterion's point and they are not the same
number; the LAN's is named as the LAN's here and in `l2.rs`.* ⚠️ **Narrowed, because
it is false for name FORM**: an FQDN against a short label yields `Neutral` here **while 6.7's rule
confidently `Opposes`** — D20's bug, live on a real network the day a second hostname source exists.
Nothing is live today (`reverse_dns.rs:137` strips the trailing dot), **but that is a property of one
connector, not of `hostnames_of`**. The FQDN/short normalisation question is **registered, owner
6.12**.

**AC6 — a `Supports` carries its evidence**, both sides' `ObsId`s **sorted**, on 6.7's D19 criterion
that *a verdict which ARGUES leaves its observations behind* — and a merge is an argument. 🔴
*Measured on the validation layer's worktree* (750/198/110, which is no commit here): returning an
empty vector left the suite green. ✅ **On the shipped tree M3 reds 2.** ✅ The order-independence half 6.7 has for
`Opposes` **is carried here too and M6 proves it**: the driver **refused** the first attempt with
`ANCHOR MATCHED 2 TIMES`, the second site being 6.7's own inline sort, *which is what establishes the
two as one convention tied by nothing* — so `evidence_of` is shared and one mutation reds both. ⚠️ *This
read "is owed here too", a present-tense debt over a test that exists.*

**AC7 — the §0.3 decision gets a guard with the population that discriminates**: a **partial overlap**
and a **superset/subset** pair. 🔴 Without it, M2 and M7 ship the refused readings green.

**AC8 — no regression**: ten gates, clippy `--all-targets`, `RUSTFLAGS="-D warnings"`, fmt, and the
suite under both store conditions. ⚠️ **The two browser gates are NOT claimed**: this story renders
nothing.

⚠️ **The clock is still a tell, and what stopped working is narrower than the validation said —
corrected with the measurement rather than repeated.** The fact-check layer measured **5.82 s** with
no store against story 6.7's recorded **0.21 s** and concluded *the clock no longer discriminates*.
Re-measured here on the shipped tree: **5.05 s without a store** and **24.52 s against a VIRGIN
store**, a factor of ~5 — so the store/no-store comparison still works. 🔑 **What broke is comparing
today's no-store run against a HISTORICAL no-store figure**: 6.7's 0.21 s is no longer the baseline to
hold a run against.

🔴 **AND THE CAUSE FIRST GIVEN FOR THE 5 s WAS FALSE, refuted by the check.** It named the budget tests
(`page.rs:3222`, `diagnostic.rs:1386`/`:1464`) as *"worth ~5 s with no database at all"*. The
acceptance-audit layer measured: all 747 in **5.03 s**, and with **all three skipped**, 744 in
**5.05 s** — the three are worth **0.00 s**. It refuted two candidates of its own and named no
replacement. 🔑 *So this is recorded as a SYMPTOM with no cause*, which is this project's own rule — *a
cause needs a check, not a plausible story*, and the check is what refuted mine.

---

## 3. What this story must NOT do

- **Not give the operator anything.** No route, no screen, no write, no migration, no dependency, and
  **no production caller** — 6.12 is the first.
- 🔴 **Not compose an L1 verdict with an L2 verdict in one vector** — forbidden by `l2.rs`'s header,
  registered with owner 6.12. This is the larger of the two deferrals and contexting omitted it.
- **Not write a second hostname extractor.**
- **Not invent a trap** for a shape the source cannot produce (hostname is never null).
- **Not de-duplicate the evidence vector** — 6.7's, owner 6.12. This story adds a second voter.
- **Not edit `epics.md`.** §0.0 is Guy's.

---

## 4. Prove-to-red — eight mutations, predictions written to a file BEFORE the first run

Driven by `cargo xtask mutate --baseline` against a live **`mariadb:10.11.11`** on port **13419**. No
*"every red assertion-carried"* headline is claimed — the carriers are named per row.

🔴 **THE PROTOCOL IS THREE STEPS AND NOT ONE, and the acceptance-audit layer measured why this header
first said "dropped and recreated first".** Dropping ONCE and running the rows in sequence is not
enough: from row 2 on, story 6.6's registered non-determinism makes the driver refuse with *"the
baseline is not clean"* (it refused six of eight that way, and one row then reported **15** red where 2
are real). Dropping per row is not enough either: a **virgin** database reds **84** tests on the
unmutated tree — story 6.5's registered `migrate!` race. **What works, and what the reported rows were
measured under: drop, recreate, run ONE serial store-backed test to migrate, then `--baseline` per
row.** *The header is what the next story copies, so it owes the whole recipe rather than its first
step.*

| id | mutation | predicted | measured | carriers |
|---|---|---|---|---|
| **M1** | drop `both_sides_offer_a_name` (D20's lock) | red 3 | ✅ **red 3** | `two_absent…`, `two_invisible…`, `two_identical_punctuation…` — and `one_named_side_and_one_silent_side…` **GREEN as predicted**, so it is kept and NOT claimed as a lock carrier |
| **M2** | reading (b): `!names_a.is_disjoint(&names_b)` | red 3 | ✅ **red 3** | the three AC7 guards. 🔴 **This was GREEN across the whole suite before AC7 existed** — Guy's decision is carried now |
| **M3** | empty evidence on `Supports` | red 2 | ✅ **red 2** | `an_agreeing_verdict_carries…`, the corpus walk |
| **M4** | corrupt `L2_HOSTNAME_AGREES` | red 2 | ✅ **red 2** | the double-literal pin **and** the corpus walk — whose red comes from its TERMINAL naming assertion, the walk itself iterating zero times |
| **M5** | `Supports` → `Decisive` | red 3 | 🔴 **red 4 — CONTRADICTS** | the three predicted **plus `case_and_whitespace_do_not_stop_two_names_agreeing`** |
| **M6** | drop `evidence.sort()` in the shared `evidence_of` | red 2 | ✅ **red 2** | 🔑 6.9's order test **AND 6.7's** `the_evidence_does_not_depend_on_the_argument_order`, from ONE site |
| **M7** | `==` → `names_a.is_subset(&names_b)` | red 1 | ✅ **red 1** | `a_superset_of_names_does_not_agree_with_its_subset`, **on its second assertion** — the one added before the pass |
| **M8** | the `Supports` arm names `L2_DIFFERENT_HOSTNAME` | red 2 | ✅ **red 2** | both rule-naming assertions; AC4's real carrier proven |
| **M9** | drop the ASCII-alphanumeric property from `hostnames_of` | red 3 | 🔴 **red 4 — CONTRADICTS** | `two_invisible…` (**which is the repair working**), `two_identical_punctuation…`, **and 6.7's `an_empty_hostname_is_an_absence_not_a_value` and `a_name_carrying_no_alphanumeric_is_not_a_name`** |
| **M10** | `supports = false` — the rule never argues | red 6 | ✅ **red 6** | every test that reads the verdict, **AC3's new corpus test among them**, so the criterion the audit found uncarried is proven red before it passes |

✅ **M10's prediction was right on its first attempt because it was derived from WHAT READS THE MUTATED
VALUE** rather than from the criteria — M5's and M9's lesson, applied at its next use and confirmed.

🔴 **M5's divergence is about MY PREDICTION METHOD, not about the code, and I am not rewriting the
prediction after the fact.** I enumerated the carriers by listing the tests written **for the
criterion** rather than the tests that read the mutated **value** — and *any* test asserting
`Verdict::Supports` carries a `Supports → Decisive` mutation whatever criterion it was written for.
There are four. 🔑 The transferable form: **a carrier list derived from criteria is a guess; a carrier
list derived from the mutated value is a measurement.** The driver said exactly what story 6.4b built
it to say.

🔑 **M6 IS THE STORY'S TOOLING DELIVERABLE.** The first attempt at this row was **REFUSED** by the
driver with `ANCHOR MATCHED 2 TIMES`, the second site being `verdict_for_hostname`'s own inline copy —
so replacing both would have repaired the very guard the mutation was meant to red, *which is what
establishes the two as independent representations of one convention*. `evidence_of` is extracted and
shared, and M6 now reds **every rule that argues** from a single site.

🔴 **M9 IS THE ROW THE STORY OWED, and it diverged for the SAME METHOD DEFECT AS M5 — twice in one
story.** No original row touched `hostnames_of` at all: M1–M8 mutate the emptiness lock, the set
operator twice, the constant, the verdict variant, the sort and the rule id. **The property AC2 calls
*"the property's job"* had never been mutated, which is exactly why the invisible guard's hole survived
a pass the story called eight-for-eight.** I predicted 3 and measured 4, and the fourth and third
carriers are **story 6.7's** guards — `hostnames_of` is SHARED, so removing the property reds both
rules' tests. 🔑 *The carrier list must be derived from everything that reads the mutated code, across
the whole file, not from the tests this story wrote* — M5 said that about a verdict variant and M9 says
it about a shared helper.

⚠️ **TWO CONTROLS the criteria cite are in NO row, and the header's count does not include them**: AC1
cites *"control C2+M4 red 1"* and AC4 cites *"control C1+M8"*. Both were run on the validation layer's
worktree and both are logically sound; **neither is replayable from this table**, which is what a row is
for. This project counts controls as rows elsewhere (story 6.3: *"eleven mutation ids — nine mutations
and TWO CONTROLS green by design"*). Recorded as a gap in the table rather than silently dropped.

🔑 **M7 confirms a finding this pass produced BEFORE it ran.** Predicting the mutation showed that
AC7's population was **asymmetric**: `is_subset` is directional, and with the superset on the LEFT it
answers `false`, so the guard would have stayed GREEN under the mutation it names. Both orientations
are asserted now and M7 reds on the second. *A guard written for a directional operator and exercised
in one direction is a guard placed where half the defect cannot occur* — this epic's dominant class,
caught by prediction rather than by reading.

## 4b. What the blind review layer found, and what it refuted

🔴 **The blind layer scored for the TENTH CONSECUTIVE STORY, with the diff and nothing else.** Ten
findings, **every one in a SENTENCE except the first, which made a test unable to fail** — and it
reproduced seven of the eight mutation rows by reasoning and found them correct, which is what makes
the rest prose defects rather than measurement defects. Its own note is worth keeping: it wanted to open
`hostnames_of` and `resolver.rs` to confirm two premises and **did not**, reporting both as findings
because each premise was quoted from the diff.

Repaired: the invisible guard's population (AC2, and **M9**); the twins' premature `done` (§0 below);
`cascade.rs`'s *"none of them has a PRODUCTION caller"*, **false for the three L1 verdicts** and
refuted by the unchanged sentence five lines above it; AC2's three incompatible counts; the 26 → **25**
denominator in five places; the *"one constant among five literals"* arithmetic, wrong in count **and
in kind**; AC4's one id reporting two trees; `scan_pass.rs`'s strike that stopped one clause early,
leaving *"Epic 6 owns giving them one"* live in the commit that discharges it; and an assertion
**strictly implied** by the one above it.

✅ **REFUTED, with the check, so nobody re-chases them**: `evidence_of`'s *"unreachable through the
corpus today"* is TRUE — **zero** multi-MAC observations across every replay stream, swept; both cited
test names exist (`hostnames_differing_only_in_case_are_the_same_name`,
`the_evidence_does_not_depend_on_the_argument_order`); and the twin paragraph landed at
`docs/project-context.md:188`, directly after story 6.7's at `:186`, not inside Epic 5's.

⚠️ **And two defects of MY OWN surfaced while repairing**: I ran `cargo test -p opencmdb-core A B`,
which passes two filters and **executes nothing** — this project's most-recorded driver defect, caught
only because the output was a usage message rather than a test result. And 🔴 **I created a
three-process collision on one database**: both review layers were told to use container
`opencmdb-14-4b-db` and to DROP the `opencmdb` database, while I was measuring against it — so M9's
first baseline came back **RED at 132 tests**, every store-backed one. 🔑 *The driver REFUSED rather
than reporting it*, exit 2, *"the baseline is not clean"* — which is precisely what story 6.4b built
`--baseline` for, and what story 6.6's register row warns about: **without it, a store's red is
published as a mutation's.** I measure against `opencmdb_69` now, a database of my own, which removes
the collision instead of coordinating around it.

## 4c. What the acceptance-audit layer found, and the one thing it got wrong

✅ **It reproduced ALL EIGHT mutation rows — counts AND carrier lists — and confirmed M5's red 4 with the
fourth carrier being exactly the one the row names.** It also re-derived §0.3's two refuted arguments
independently (0 exclusivity violations under all four readings; equality and intersection diverge on
**30 of 64**), confirmed the 8-flip measurement *to the trap*, and verified the live count and **both
terms of its delta**. It found no unmet criterion on the product's behaviour, and no detached doc
comment — the fourth Epic-14 class did not recur.

🔴 **ITS HEADLINE IS THAT AC3 DELIVERED NOTHING**: `"hostname-collision"` occurs **zero** times in the
whole `crates/` diff, so the criterion this story calls *"its best test"* was carried by no committed
guard, and its second clause — *must NOT compose L1 and L2* — was satisfied **vacuously, a prohibition
met by the absence of the test it constrains**. The test is written now
(`the_rule_fires_on_the_collision_family_and_l1_is_what_refuses_it`) and **M10 proves it red**.

🔴 **And it measured that the §4 protocol header described a recipe that does not produce these
results** — one drop is not enough, a virgin store reds 84 on the unmutated tree, and the driver refused
six of eight rows when it tried the header literally. The header now carries the whole recipe, *because
the header is what the next story copies.*

⚠️ **Everything else it found is in prose, in `Status:`, or in the criteria — the three regions
`cargo xtask record` declines to read**, which Epic 14's retrospective named and predicted in writing.
Repaired: the Base note's false cause (refuted by §5 of this same document, and by `gh pr view` — the
commit WAS pushed, as PR #205's head); `epics.md:1894`/`:1904`, harvested from the pre-reorder tree and
wrong in four documents; `architecture.md:1002`, a **+30** drift I propagated into five new sites;
750/198/110, a figure matching **no commit here**, in three places; AC8's cause, **refuted by the check**
(the three budget tests are worth 0.00 s of the 5 s, so it is a symptom with no cause named); the
8-flip *reason*, which reached six of its eight; the collapsed-set claim, true only within one guard's
population where **nine of 25 pairs collapse**; AC5's two populations in one sentence; AC6's
present-tense debt over a test that exists; two controls the criteria cite and no row carries; and the
watcher row's claim about the register, made **without reading the register** — it holds one such row,
not two.

🔴 **AND ONE OF ITS FINDINGS WAS WRONG, WHICH I PROPAGATED BEFORE CHECKING.** It reported
`reverse_dns.rs:137` as `pub(crate) fn sanitise` with the trailing-dot strip at `:138`; measured, `:137`
**is** `let name = raw.trim().trim_end_matches('.');` and the original citation was right. I applied the
correction on trust and had to revert it. 🔑 *An audit layer's citation correction is itself a citation
and earns the same check — and a story that has spent its day on figures nobody re-derived is exactly
where a borrowed figure slips in.* Two of its three one-line corrections held (`l1.rs:488`,
`cascade.rs:13-14`); this one did not.

## 5. Instrument defects of my own, each caught by disbelieving a result

🔴 **I READ ANOTHER STORY'S MUTATION RESULTS AS MINE, and only a semantic tell caught it.** The
session scratchpad is shared across stories and I named my logs by POSITION (`m5.log`, `m6.log`…), so
story 14.5's logs of 2026-09-21 sat under exactly the names my own pass was about to write. Three of
them read as clean conforming results — one `CompileFailure` matching its prediction, one `red:1`
naming a test, one `Green` — and **nothing mechanical said they were three days old**: not the
filename, not the format, not an exit code. What said it was that `M5` named `declares_a_vlan` and
`M7` named *a subnet correction writes its label and its VLAN*, and this story has nothing to do with
VLANs. 🔑 **The remedy is a NAMESPACE and not vigilance** — the logs live in a per-story directory
now, cleared before the pass — because *a stale measurement under a fresh name is indistinguishable
from a measurement*, and this project's recorded class is exactly that: **a measurement taken on one
artefact and attributed to another**. Four fabricated rows were one `grep` away from the table.

⚠️ **And I committed the pipeline-status trap while recreating the database** — `DROP_EXIT=1` was the
`grep`'s status, not the command's. Caught because the output printed `recreated`, which contradicted
the 1. This project's own rule says to read `$?` from a file; I read it after a pipe, in the same
session whose notes name the defect.

🔴 **A FOURTH, and it is the sharpest of the four: a CI watcher that was blind BY CONSTRUCTION, and
it stayed silent over a run that had already SUCCEEDED.** The monitor armed on PR #205 polled
`gh pr checks 205 --json name,bucket` — and **`gh pr checks` has no `--json` flag at all** in the
installed `gh 2.46.0`, so every poll captured a usage error instead of data: `jq` failed, the
terminal-state break never fired, and the diffing step emitted nothing. It expired after fifteen
minutes with **zero events** while the run had finished green at **3m33s**. 🔑 *A watcher whose data
source never produces data cannot report success OR failure*, so its silence carried no information
whatever — and this project's register already holds two watcher rows of a weaker kind (one exiting 0
over zero checks, one killed and emitting nothing). **The rule this adds: a watcher must be proven to
emit at least once before its silence is allowed to mean anything** — I armed it without ever checking
that its command produces output in this environment. The machine-readable route here is
`gh pr view --json statusCheckRollup`, and the conclusion was read from `gh run view` in the end,
which is what confirmed CI green on `e7e5143` **itself** rather than inherited.

🔴 **A THIRD, and it is the one that would have cost the most: my waiter's needle was CASE-SENSITIVE
where the driver shouts.** The chain that was to run M6–M8 waited on `contradicts`; the driver prints
**`🔴 THE OUTCOME CONTRADICTS THE PREDICTION`**. So M5's divergence — the one real finding of the pass
— never woke the chain, and **three mutations silently never ran**. 🔑 *A waiter that cannot match the
failure message waits for ever, and a waiter that never fires is indistinguishable from a pass still
running* — this project's *silence is not a green*, one layer down: not a watcher that exits 0 over
nothing, but a watcher that **cannot see the one outcome worth waking for**. Caught by polling the
files directly instead of trusting the waiter, which is the same defence the register already records
for CI watchers. ⚠️ **And the shape is the story's own subject**: my needle was written for the happy
path and blind to the red, exactly as a guard written for a criterion is blind to the value the
mutation moves.

⚠️ **The driver REFUSED the store-free run, and the refusal is a result**: this story needs no store,
and `cargo xtask mutate` still answers `2` — *"`DATABASE_URL` is unset, so every store-backed test
passes by RETURNING and the totals are identical to a real run"*. It cannot tell *no store needed*
from *store tests silently returning*, so it declines rather than guess. The pass ran against a live
`mariadb:10.11` on port **13419** with the database dropped and recreated first, per story 6.6's
registered non-determinism row.

## 6. The figures, and what nothing owes

⚠️ **1 046 → 1 061 tests**, +15 — twelve synthetic in `l2.rs` and **three** corpus-driven in
`fixtures.rs`, the third being AC3's, which the acceptance audit found missing. ⚠️ *This read 1 060 and
+14 until that test was written, which is the delta moving under a repair rather than a figure written
in flight.*
`l2.rs` goes **255 → 404** code lines of the 2000-line ceiling, so no split is owed.

⚠️ **Neither manual owes a sentence.** This story ships no route, no screen and no production caller,
so nothing an operator reads changes; the administrator manual's grouping warning is about the **L1
key** and stays true exactly as written.

⚠️ **The block below carries `key: value` lines and nothing else** — `cargo xtask record` refused a
prose paragraph inside it, which is the checker doing its job, and the prose moved here rather than
the block being loosened.

## 7. Change Log

| when | what |
|---|---|
| 2026-09-23 | contexted; §0.1's worry posed, §0.2 settled by precedent, §0.3 posed |
| 2026-09-24 | validated by two fresh-context layers; §0.1 dissolved, §0.3's two arguments refuted, §0.0 discovered by the gap-hunt |
| 2026-09-24 | **Guy's two arbitrations**: §0.0 measured not decided · §0.3 taken as set equality |
| 2026-09-24 | implemented; eight mutations, seven conforming, M5 contradicting |
| 2026-09-24 | `cargo xtask record` refused twice — prose in the block, then a base that was not the branch point; PR #205 merged so the base became honest |
| 2026-09-24 | **blind review layer**: ten findings, nine prose and one guard unable to fail; repaired, M9 added and it contradicted too |
| 2026-09-24 | **acceptance-audit layer**: 8/8 rows reproduced, and **AC3 found delivering nothing** — its test written, M10 proves it red |
| 2026-09-24 | its citation corrections applied — and **one of them reverted**, `reverse_dns.rs:137` being right as first written |

⚠️ *This section did not exist for the story's first six commits, which is the SECOND of the three blind
spots Epic 14's retrospective named for `cargo xtask record` and the second story running to miss it.*

## Record

- live-count: bin=748 core=203 xtask=110
- base: 97e95dd07b47dde3524b8d669e8aa11f4fddc039
- registered: A `Supports` rule cannot pass a `must-merge` trap
- registered: The first producer of `Verdict::Supports` moves from story 6.8 to 6.9
- registered: An L2 side's SCOPE and the hostname-agreement reading are one decision taken a story apart
- registered: An FQDN and a short label are two spellings of one name
- registered: A watcher must be proven to emit at least once before its silence is allowed to mean anything
- registered: `architecture.md`'s line citations have drifted by a UNIFORM +30 across the identity engine
- registered: Story 6.15's own criterion promises what story 6.9 measured to be impossible
- file: _bmad-output/implementation-artifacts/6-9-l2-hostname-agrees.md
- file: _bmad-output/implementation-artifacts/deferred-work.md
- file: _bmad-output/implementation-artifacts/sprint-status.yaml
- file: crates/opencmdb-core/src/identity/l2.rs
- file: crates/opencmdb-core/src/identity/cascade.rs
- file: crates/opencmdb-bin/src/fixtures.rs
- file: crates/opencmdb-bin/src/scan_pass.rs
- file: CLAUDE.md
- file: docs/project-context.md
