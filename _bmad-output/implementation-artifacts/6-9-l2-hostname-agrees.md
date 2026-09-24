# Story 6.9: `l2-hostname-agrees`

Status: **validated** 2026-09-23 by two fresh-context layers (fact-check + gap-hunt, the latter having
BUILT three readings of the rule and run a ten-row mutation pass). **ONE question for Guy, posed
below and not settled here**; then dev.

**Epic 6**, reordered by Guy on 2026-09-23: **6.9 → 6.11 → 6.12**, with 6.8 and 6.10 waiting for a
connector that emits an uplink or a switch port. **Base:** `e7e5143`, clean tree.

⚠️ Both layers verified that `e7e5143` differs from its parent in **four documents and no code**, so
every measurement below transfers.

---

## 0. What contexting measured, what validation refuted, and the one question that is Guy's

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
`architecture.md:972` itself calls **weak evidence**, and `score.rs:276` makes `(MustMerge,
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

**750 + 198 + 110 tests green with the refused reading shipped.** *A decision a story takes out loud
and no guard can red is a decision the next story reverses silently.* The discriminating populations
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
  locally-administered MACs differing — so **6.7's collapse does not recur here**. The corpus's
  collapsed set stays exactly `["cloned-mac-must-not-merge"]`. Said in one line because 6.7 was bitten
  by the opposite.
- 🔴 **THE D20 MIRROR IS LOUD IN THE CODE AND INVISIBLE TO THE CORPUS.** `BTreeSet` compares **equal**
  when both sides are empty, so the naive `names_a == names_b` **SUPPORTS ON ABSENCE**. Measured over
  all 26 trap-named pairs: **8 flip to `Supports` on total absence**, six of them `must-not-merge`
  traps. ⚠️ **And not one of the eight is caught by the corpus**, because a false `Supports` alone
  still gives `Abstained { Ambiguous }` and `(must-not-merge, Abstained)` is `score`'s **pass** cell.
  *The bug is carried by synthetic guards alone.* ⚠️ Under reading (b) there is no bug and no lock —
  §0.3.
- 🔑 **The stakes are not symmetric with 6.7's**: a false `Opposes` leaves two rows where one belongs
  and the operator can act; a false `Supports` merges two machines.
- 🔴 **6.9 IS THE FIRST PRODUCER OF `Verdict::Supports`** — no production producer exists today
  (`l1.rs:487` **panics** on it, inside `#[cfg(test)]`). Contexting never said so, and **three
  sentences go stale**, one of which is **already false**:
  - `cascade.rs:12-14` — *"`Verdict::Supports` and `Verdict::Opposes` still have no producer"* —
    **FALSE for `Opposes` since story 6.7**, live on `master`, in the file §0.1 quotes.
  - `scan_pass.rs:760` — same sentence, same staleness.
  - `epics.md:1866`/`:1877` — *"Story 6.8 … the first producer of `Supports`"*. The reorder moves it
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
⚠️ **Three guards, and the story says which two carry D20's lock**: both sides silent, and invisible
characters only, **plus pure punctuation** (the property's job, and contexting named invisibles
alone). 🔴 **The asymmetric guard — one side names, the other is silent — CANNOT red on this
mutation** and is not claimed as a carrier: under (a) `{doc-a} ≠ {}` already, measured (M1 reds 2 of
the 3). It is kept because it catches others.

**AC3 — the corpus's own temptation, and it is this story's best test.** On
`hostname-collision-must-not-merge` the rule fires `Supports`, loses, and the family still passes.
🔴 **The test asserts the RULE's verdict and must NOT build a mixed L1+L2 verdict vector** — `l2.rs`'s
own header forbids it (*"combining L1's verdict for the same pair with an L2 rule's … erases this
level"*) and `deferred-work.md` registers it with **owner 6.12**. ⚠️ Contexting prescribed exactly
that forbidden gesture. ~~And a mutual-exclusivity property.~~ **Struck: 0 violations of 64 under
both readings** (§0.3), and under every mutation built it is never the sole carrier.

**AC4 — the VERDICT names the rule, not merely the constant being spelled right.** 🔴 *Measured*:
with the `Supports` arm emitting the other rule's id, the only red came from an assertion contexting
did not prescribe; remove it (control C1+M8) and **a rule emitting the wrong id ships 750/198/110
green**. The double-literal pin stays, on 6.7's precedent, **and its reason is re-measured rather than
inherited**: 6.7's mechanism was `Opposes` → `AbsenceOfProof`, this one is `Supports` → `Ambiguous`,
two **different rows** of the table, and both carry no rule, so `run_trap`'s `(Some, Some)` never
fires.

**AC5 — the reach is STATED with its measurement**: `prd.md:880`'s dated figures, **squared** for a
per-pair rule (§1), and the sentence that this silence is data availability. ⚠️ **Narrowed, because
it is false for name FORM**: an FQDN against a short label yields `Neutral` here **while 6.7's rule
confidently `Opposes`** — D20's bug, live on a real network the day a second hostname source exists.
Nothing is live today (`reverse_dns.rs:137` strips the trailing dot), **but that is a property of one
connector, not of `hostnames_of`**. The FQDN/short normalisation question is **registered, owner
6.12**.

**AC6 — a `Supports` carries its evidence**, both sides' `ObsId`s **sorted**, on 6.7's D19 criterion
that *a verdict which ARGUES leaves its observations behind* — and a merge is an argument. 🔴
*Measured*: returning an empty vector left **750/198/110 green** (M3). ⚠️ The order-independence test
6.7 has for `Opposes` is owed here too — the driver **refused** M6 with `ANCHOR MATCHED 2 TIMES`, the
second site being 6.7's own sort, *which is what establishes the two as one convention tied by
nothing*.

**AC7 — the §0.3 decision gets a guard with the population that discriminates**: a **partial overlap**
and a **superset/subset** pair. 🔴 Without it, M2 and M7 ship the refused readings green.

**AC8 — no regression**: ten gates, clippy `--all-targets`, `RUSTFLAGS="-D warnings"`, fmt, and the
suite under both store conditions. ⚠️ **The two browser gates are NOT claimed**: this story renders
nothing. ⚠️ **And the clock is no longer the tell** — measured with no store, the suite now takes
**5.82 s** (budget tests: `page.rs:3222`, `diagnostic.rs:1386`/`:1464`), against 0.21 s at 6.7. A
store run must be established some other way.

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

## Record

- live-count: bin=745 core=191 xtask=110
- base: e7e5143ae3f4c0192944bd9723c5788cf3734590
- registered: a `Supports` rule cannot pass a `must-merge` trap
- registered: the first producer of `Verdict::Supports` moves from story 6.8 to 6.9
- registered: an L2 side's scope and the agreement reading are one decision
- registered: an FQDN and a short label are two spellings of one name
- file: _bmad-output/implementation-artifacts/6-9-l2-hostname-agrees.md
