# Story 6.6: L2 candidate generation, and no rule

Status: **review** — implemented AND code-reviewed 2026-08-30 against a live `mariadb:10.11.11` (port 13366).
**783 tests green both ways** (515 bin + 169 core + 99 xtask), ten gates green, clippy over
`--all-targets`. Contexted 2026-08-30 against the committed corpus and the tree at
`8a19089`, then VALIDATED the same day by two fresh-context layers, each in its own worktree.
**§0's corrections are applied in place and §0i holds the record.**

✅ **VALIDATED 2026-08-30 by two fresh-context layers, each in its own worktree** — a fact-check
layer and a gap-hunt layer that BUILT the prototype against a live `mariadb:10.11.11`. **Sixteen
findings, six of them HIGH; every one is applied in place and §0i holds the record.** The gap-hunt
layer's eight mutations all conformed to a prediction written first.

✅ **BOTH ARBITRATIONS ARE TAKEN (2026-08-30) and the story is buildable.** They are **mine,
delegated by Guy in the same exchange**, and are recorded as mine so they can be reversed at the
right cost (story 6.5 §0d's precedent).

1. **§0f — the input stays `&[L1Key]`, and the TYPE REPLACES THE GUARD.** AC5's uplink half is
   WITHDRAWN, because the gap-hunt layer measured it unwritable (`E0425`) — and a guard that cannot
   red is this epic's dominant defect, committed inside the criterion written to avoid it.
2. **§0j — the L2 rules judge INTERFACE pairs, and the arithmetic is corrected rather than the
   conclusion: the unanswerable bucket goes 11 → 4, NOT 11 → 3.**

Each is recorded below with the option refused and the cost accepted.

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
corpus** (measured, §0e) — same `l2_domain`, and agreeing uplink `peer_mac`
**When** the story ships
**Then** the `l2_domain` narrowing carries a SYNTHETIC test that reds under it, written BEFORE the
production code, on story 5.6's precedent (`two_l2_domains_are_still_a_candidate_pair` exists for
exactly this reason and was written first).

🔴 **The uplink half of this criterion is WITHDRAWN, and the withdrawal is the deliverable.** Its
first draft demanded a synthetic guard for the uplink narrowing too. The validation MEASURED that no
such guard can exist under `&[L1Key]`: the argument carries no `Fact`, so the mutation that would red
it does not compile (`error[E0425]: cannot find function \`uplink_of\``). **A guard that cannot red
reads as coverage and is none** — this epic's dominant defect, committed inside the criterion written
to avoid it. What replaces it is not a weaker guard but a stronger carrier: **the TYPE**. See §0f.

⚠️ **And the risk does not vanish, it MOVES.** Filtering by uplink `peer_mac` **at the call site** was
measured leaving 783 tests, clippy and all ten gates GREEN. The call site is story 6.12's; it is
registered by name, not implied.

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
(`sprint-status.yaml`, cited by content: it was `:4826` on `8a19089` and this story's own commit
pushed it to `:4848` — *a line citation can be made stale by the commit that carries it*).
**This story produces no verdict at all.**

⚠️ The error was carried into a session memory before it was caught; the memory is corrected.

🔴 **The `must-merge` truth set is ELEVEN, not ten.** `CLAUDE.md` says *"10 `must-merge`"* — story
5.6's figure, true on 2026-08-01 and stale since story 5.13b added the blinded-source pair.
`fixtures.rs:4752-4753` asserts `11` today. **The L1 figure is not the L2 one** — see §0c.

### §0b. What exists, measured — the blocker at L1 and what it refuses

`crates/opencmdb-core/src/identity/blocking.rs`, **259** code lines by the `file-size` gate's own
rule — its `#[cfg(test)]` sits at line 260 and the gate counts the lines PRECEDING it — total 661. It ships:

- `CandidatePair` — private fields ordered by `new`, so `new(a, b) == new(b, a)` holds **by
  construction**; `new(a, a) -> None`, which closes the self-pair **in the type**.
- `candidates(&[Observation]) -> BTreeSet<CandidatePair>` — **TOTAL by decision**: every unordered
  pair of distinct `obs_id`s, no narrowing key.
- `BLOCKING_RECALL_FLOOR_PER_MILLE: u32 = 999` and `blocking_recall_per_mille`, integers on D13's
  own milli-units corollary.

Its module doc states the refusals this story inherits — **rendered here, not verbatim**: the
intra-doc links are reduced to their final segment and a leading `It` is dropped. It *"calls neither
`join` nor `decide_pair`"*, *"consumes no structural reading of a MAC (the U/L bit, the IANA
prefixes, the I/G bit) and reads no `Fact` at all"*, and *"writes nothing and reads nothing but its
argument"*. _(This paragraph claimed **verbatim** until the validation compared the strings. A
rendering presented as a quotation is a defect in this project, however faithful.)_

🔴 **THAT MODULE DOC CARRIES FOUR STALE FIGURES, and this story is what must fix them** — because
§Dev Notes orders the developer to *cite* it rather than paraphrase, and citing it today propagates
four falsehoods. Re-measured over the **eleven** required pairs (the doc says ten):

| `blocking.rs` says | measured today |
|---|---|
| the corpus has **ten** `must-merge` pairs | **eleven** since story 5.13b |
| a MAC-blocked universe scores **700‰** | **727‰** (8/11) |
| a hostname-blocked one **400‰** | **363‰** strict (4/11) — and **818‰** under the loose reading |
| with 10 required pairs one miss gives **900‰** | **909‰** at eleven |

Only *"an `l2_domain`-blocked one scores 1000"* survives. ⚠️ The hostname row matters beyond
arithmetic: under the natural reading it is **818‰**, so the doc's *"the other two keys the corpus
would catch on its own"* loses most of its force.

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
| agreeing hostname, **strict** (`is_some() &&` equal) | **333‰** | reds |
| agreeing hostname, **loose** (`Option == Option`) | **666‰** | reds |

🔴 **The hostname row needed TWO lines and the first draft gave one, at the tighter figure.** The
validation reproduced 333‰ only under a convention the draft never stated: that an interface with NO
hostname agrees with nobody, *not even with another that has none*. The obvious filter a developer
writes — `hostname(a) == hostname(b)` — scores **666‰**, and they would have recorded a divergence
that is not one. Same family as the four recorded occurrences of *a row named for a CLASS reporting
one convention's count*.

🔑 **And the reason for the gap is itself the finding**: under the loose reading
`multi-nic-must-merge` survives the hostname narrowing **because neither of its sides carries a
hostname at all** (verified: `multi-nic.jsonl` holds `Mac` + `IpV4` + `Uplink`, no `Hostname`).
*An agreement between two absences is an empty agreement, and it is exactly what makes a narrowing
look safer than it is.*

🔑 **The two blind rows are the story's centre.** *Block on the uplink* is the most tempting L2
narrowing there is — it is literally the signal `l2-uplink-agrees` scores on, one story later — and
**it passes the entire committed corpus**. A blocker narrowed on it would be the echo AC1 forbids,
and no committed trap would say so. This is story 5.6's `l2_domain` finding, one level up and with a
sharper temptation.

**Therefore AC5, in the form the validation left it**: the `l2_domain` narrowing gets a synthetic
test written FIRST — two interfaces that must remain a candidate pair *although* they sit in
different L2 domains. **The uplink half is withdrawn**: under `&[L1Key]` no such test can red, and
the type is what carries it (§0f).

⚠️ 🔴 **AC3 IS SATISFIED ONLY BY IMPLAUSIBLE MUTATIONS, and that is written rather than ticked.** Of
the four narrowings, exactly **one** is expressible inside the recommended function — `l2_domain` —
and it is one of the two the corpus is blind to, so it reds the synthetic test alone while the corpus
stays green. The gap-hunt layer measured the only narrowing that reds the corpus assertion: **blocking
L2 on MAC equality (5 red)** — which is the inverse of what L2 does. **So no plausible AND expressible
narrowing can red the L2 recall assertion.** Say so; do not tick AC3.

✅ **What IS load-bearing, measured: AC4.** Silently shrinking the truth set to two pairs reds two
tests. Without it the recall would have stayed at 1000‰ over an amputated set. **AC4 is this story's
best criterion**, and it was added by contexting rather than inherited.

### §0f. ✅ ARBITRATION TAKEN (mine, delegated by Guy 2026-08-30) — where the code lives, and what it takes

**Decided: extend `blocking.rs`; do NOT create `l2.rs` in this story. Input type `&[L1Key]`.**

`blocking.rs` is the module whose whole subject is candidate generation, and its doc already promises
the refusals AC1 restates. Putting the L2 blocker beside the L1 one keeps *one home for blocking* and
leaves `l2.rs` to be created by story 6.7 for the **rules** — which is what makes *"the blocker
consults no rule"* structurally visible rather than merely asserted: the rules are not in the file.
Size is not a constraint (259 code lines against a 2000 ceiling). `BLOCKING_RECALL_FLOOR_PER_MILLE`
is shared, which is right: D13 gives one floor.

**Refused: a new `l2.rs` holding the blocker now.** It reads as the natural mirror of `l1.rs`, but
`l1.rs` holds RULES and the join, and one home for blocking is worth keeping.

🔴 **THE FIRST DRAFT'S REASON FOR THAT REFUSAL WAS REFUTED, and the refutation is kept rather than
overwritten.** It argued that a file mixing this blocker with story 6.7's first rule *"lets a later
edit make the blocker read a verdict without any reviewer noticing"*. The gap-hunt layer measured the
opposite: adding `decide(Vec::new(), CURRENT_RULESET_VERSION)` **inside `blocking.rs`** — literally
the act AC1 forbids — leaves **783 tests, clippy and all ten gates GREEN**. The edit is exactly as
invisible here as it would be in `l2.rs`. **The file choice buys documentation, not constraint**, and
the decision stands on that weaker and true ground. *This project requires an option to be refused ON
A MEASUREMENT; the first draft refused it on a reason a measurement contradicts.*

🔑 **What IS structural, and the first draft did not say it:** under `&[L1Key]` neither `join` nor
`decide_pair` is reachable at all — both demand `Observation`s, verified at compile time. So AC1's
refusal is **half carried by the type and half carried by a sentence**, and the half carried by a
sentence — `decide` — is the one AC1 names first. ⚠️ **State it as a TRIPWIRE** (story 5.12's
precedent), never as a guarantee.

**Input type — recommended `&[L1Key]`** (`pub type L1Key = (L2DomainId, MacAddr)`, `l1.rs:89`),
mirroring `candidates(&[Observation])`. At L1 an interface **is** an `L1Key`: `join` returns
`BTreeMap<L1Key, BTreeSet<ObsId>>` and `resolver.rs`'s doc says *"`join` NAMES the interface"*. A
caller passes `join(&observations).keys()`.

⚠️ **Not `InterfaceId`, and the first draft gave the wrong reason.** It said D47 forbids it —
**false**: `InterfaceId` is declared in `opencmdb-core` itself (`observation/mod.rs:77`), so a domain
function may perfectly well take one. The real cost is the sentence that followed: **the corpus has
no `InterfaceId` to supply**, there being no store, and AC2's whole measurement lives there. The
conclusion holds; its stated motive did not.

🔑 **AC5's uplink guard is UNWRITABLE here, and that is this decision's accepted cost, stated rather
than discovered.** `L1Key` carries no uplink, no hostname, no `Fact`. So the narrowing AC1 most fears
cannot be expressed inside the function — **the type carries what the guard claimed to carry**, which
is story 5.6's own idiom (`new(a, a) -> None` closed the self-pair IN THE TYPE) and story 6b.3's
arbitration 3. ⚠️ **And it moves the risk to the call site, where it is expressible and measured
INVISIBLE** (783 green). Registered by name to story **6.12**, the first caller.

⚠️ **Duplicates are a COROLLARY OF THE TYPE, not a separate rule** — the first draft asked for an
explicit decision and a test of its own. Measured: under a pair type refusing `new(k, k)` there is no
code to write, and the two tests cannot red separately (one mutation reds both). Write it as a
corollary and do not claim a second carrier. Same family as story 6.5's M6 and 6.4b's P3.

### §0g. What this story does NOT do, stated so nobody discovers it

- ⚠️ **No production caller for the L2 blocker.** Nothing hands it a population; story **6.12** (the
  resolver writing device groupings) is the first that will.
  🔴 _(The first draft added: "the L1 blocker lived five stories in exactly this state and the
  register says so". **Both halves were wrong and the validation measured it.** The register line it
  quotes is verbatim but is marked **✅ CLOSED** further down: `resolver.rs:211` calls
  `identity::blocking::candidates` in PRODUCTION code since story 5.9b, and `scan_pass.rs` reaches it
  since 5.14. The state ran 5.6→5.9 — four stories, five only if you count the one that ended it.
  **Citing the oldest occurrence of a register line makes a closed debt read as open.**)_
- ⚠️ **No verdict, no rule, no `Decision`.** `Verdict::Supports` and `Verdict::Opposes` still have no
  producer after this story — 6.7 and 6.8 are where that changes.
- ⚠️ **The trap gate stays RED at 26/15/11.** This story routes nothing and answers no trap.
  🔴 **But the bucket goes 11 → 4, NOT 11 → 3 — see §0j.** `epics.md`'s Epic 6 constraint (2) says
  three; one trap is unreachable at L2 by construction. Do not report an improvement the gate does
  not show, and do not repeat the epic's arithmetic without §0j's correction.
- ⚠️ **D17's `dormant` exclusion is NOT implemented here** — and the reason had to be corrected.
  🔴 The first draft said *"there is no lifecycle state"*, quoting a register line dated 2026-08-01.
  **False since story 6.5**, the story immediately before this one, which the Dev Notes below cite by
  name: `EntityState::Dormant` is a public variant of `opencmdb-core` (`observation/mod.rs:190`) and a
  token of `0006`'s `CHECK`. What survives is the half that matters: **nothing SETS `Dormant`**,
  `entity` holds no interface row, and **`mac_kind` — the field `dormant`'s scope depends on —
  exists in no table and no `.rs`**, so there is still no field a blocker could read. ⚠️ The owner is
  **story 6.18**, re-assigned by story 6.5; *"the lifecycle epic"* is the stale clause. Do not invent
  the state in order to filter on it (D45).
- ⚠️ **`opencmdb-core` gains behaviour**, so the usual *"byte-identical"* claim does not apply.
  Narrow the promise to *no behaviour change elsewhere in the crate*, on story 5.13b's finding that
  *a promise of non-modification protects behaviour and shelters false sentences.*

### §0h. Gates and the house rules that bite here

- **`float-free`** walks `crates/opencmdb-core/src/identity/` — **four files today, five if a new one
  lands**. A float literal or a float type anywhere in the new code reds it. The recall is `u32` in
  per-mille; the per-mille arithmetic is integer division, and the test's own expected values are
  integers.
- **`file-size`** (2000 code lines before the first `#[cfg(test)]`): `blocking.rs` is at **259** —
  ample. _(Three sections said 260. `code_line_count` returns the number of lines PRECEDING the first
  `#[cfg(test)]`, which sits at line 260; total 661 is right. The same off-by-one the story-6.4b
  review caught — "883 where the gate's own rule gives 919" — in the one direction the gate's rule
  determines.)_
- ⚠️ 🔴 **`float-free` CANNOT see the per-mille arithmetic break, and AC2 must not lean on it.**
  Measured: reordering `hits * PER_MILLE / len` into `hits / len * PER_MILLE` reds **one** test and
  leaves all ten gates green — the gate guards the TYPE, never the computation. **The only real
  carrier is a synthetic truncation test**, which T5 now prescribes.
- ⚠️ **`blocking_recall_per_mille` is MONOMORPHIC in `CandidatePair`**, so AC2's *"reusing
  `BLOCKING_RECALL_FLOOR_PER_MILLE`"* cannot mean reusing the function. Two roads: duplicate twelve
  lines, or make it generic over `<T: Ord>`. **Take the generic** — the constant and `PER_MILLE` stay
  shared, and this repository's DRY rule admits deliberate redundancy only where a test pins it and a
  comment labels it, which is not the case here. The validation proved the duplication byte-for-byte:
  `cargo xtask mutate` refused an anchor that **matched twice**.
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

### §0i. What the VALIDATION measured, 2026-08-30 (two fresh-context layers, own worktree each)

**Sixteen findings, six HIGH. All applied above, in place.** The gap-hunt layer built the prototype
(783 tests: 514 bin + 170 core + 99 xtask) and ran **eight mutations against a live
`mariadb:10.11.11`, every one conforming to a prediction written first**.

🔑 **The two layers barely overlap, and the one that BUILT found what the one that READ could not.**
Four of the six HIGH are the gap-hunt layer's, and three of those are invisible to any reading: a
guard that cannot compile, a forbidden call that changes nothing, and a trap that collapses.

🔑 **Where they DID converge, they arrived by different roads**: the fact-check layer found three
`architecture.md` citations pointing ~25 lines off and traced them to `blocking.rs`'s module doc;
the gap-hunt layer independently found that same doc carrying four stale FIGURES. **The file this
story extends is a source of false sentences, and the story orders the developer to cite it.**

**Confirmed rather than believed** — and worth not re-checking: the three-pair table cell by cell,
re-derived independently; the five recall figures, re-measured under **four** different attribute
models to rule out a method artefact; the eight `l1-exact-mac` pairs collapsing at L2; `blocking.rs`
at 661 lines; `float-free` walking four files; ten gates green; 772 tests on `master`.

**Refuted suspicions, kept with their check**: the L2 blocker cannot call `join` or `decide_pair`
under `&[L1Key]` (compile-checked — only `decide` remains reachable); the corpus gate does not move;
and **no trap-named observation carries more than one MAC** (measured over all 26). ⚠️ That last one
is a property of the CORPUS, not of the type: `keys_of` admits N keys, so **T6 must decide what to do
at zero keys and at two or more** — no criterion said so, and it is the bucket the prototype had to
invent that produced §0j.

### §0j. ✅ ARBITRATION TAKEN (mine, delegated by Guy 2026-08-30) — the L2 rules judge INTERFACE pairs, and the bucket goes 11 → **4**

🔴 **The gap-hunt layer found a trap that names an `l2-*` rule and has NO L2 pair at all.**
`cloned-mac-must-not-merge` expects `l2-different-hostname`, and its two observations carry the
**same** `MacAddr([2,0,94,0,83,112])` in the same `l2_domain`. `join` keys on `(l2_domain, mac)`, so
they collapse onto **one** interface. An interface-keyed universe can never propose that pair, and
the rule can never be asked about it.

🔑 **The trap file says so itself, citing D21**: *"A cloned MAC = two real interfaces, same MAC"* —
which is exactly what `join`'s key makes unrepresentable.

**Decided: the L2 rules judge INTERFACE pairs.** It is what `epics.md` means by L2 (*device
grouping*), it is what this story's blocker feeds, and it keeps one subject for the whole level.

**The cost, accepted and WRITTEN rather than discovered:**

- `cloned-mac-must-not-merge` stays **unanswerable at L2**. The unanswerable bucket therefore goes
  **11 → 4** with stories 6.7–6.11, **not 11 → 3**. `epics.md`'s Epic 6 constraint (2) says three and
  is **not edited** — a story may not; the divergence is **registered**, owner Epic 6's retrospective,
  with story **6.15** told by name that it inherits one trap more than its criterion states.
- ⚠️ This arbitration does not only choose a file: **it fixes the argument type of every L2 rule from
  6.7 to 6.11.** Whoever contexts 6.7 inherits it and should not re-open it silently.

**Refused: observation pairs.** It would make `cloned-mac-must-not-merge` reachable and the bucket
really 3 — but then the blocker this story ships is **not** what feeds the rules, and story 6.6 loses
its object. Refused on that measured cost, not on preference.

**Refused: deferring to story 6.7.** §0f fixes the type implicitly whatever we say, so deferring
would leave a decision taken without being posed — the exact defect this project has caught in its
own stories five times.

## Tasks / Subtasks

- [x] **T1 — Validation.** Run 2026-08-30 by two fresh-context layers, each in its own worktree.
      Sixteen findings, six HIGH, all applied; §0i holds the record. Both arbitrations taken (§0f,
      §0j).
- [x] **T2 — Write the synthetic AC5 guard BEFORE the production code** (AC5): two interfaces in
      different L2 domains remain a candidate pair. Observe it fail, then pass.
      🔴 **One guard, not two** — the uplink twin is withdrawn as unwritable (§0f); do not restore it.
- [x] **T3 — The pair type**: unordered by construction, `new(a, a) -> None`, private fields, full
      rustdoc. Test that `new(a, b) == new(b, a)` and that the self-pair is refused. (AC1)
- [x] **T4 — The generator**: TOTAL over the supplied population, calls no rule and no `decide`.
      ⚠️ Duplicates collapse **as a corollary of the pair type**, not as a rule of its own — one
      mutation reds both tests, so do not claim two carriers (§0f). (AC1)
- [x] **T5 — The L2 recall**, integer per-mille. Make `blocking_recall_per_mille` **generic over
      `<T: Ord>`** rather than duplicating it, so the floor constant stays shared (§0h). **Add a
      synthetic truncation test**: `float-free` guards the type and cannot see the arithmetic
      reordered — measured. (AC2)
- [x] **T6 — The corpus assertions in `opencmdb-bin`** (D47 forbids core to read files), **and there
      are TWO of different natures — the L1 pattern has both and the first draft prescribed only
      one**:
      **(a) RECALL** — build the L2 truth set from the `must-merge` traps whose expected rule starts
      with `l2-`, **assert the denominator is 3 by value** (AC4), then assert recall ≥ the floor.
      **(b) COVERAGE** — every trap pair, *all poles*, is in the universe, mirroring
      `every_trap_pair_is_in_the_universe`. 🔑 **This is the assertion that found §0j**, and its L1
      twin says why: a pair outside the universe can never be answered by anything.
      Keep them SEPARATE tests, on `blocking_recall_above_999`'s stated reason: with both in one
      function a missing pair panics before any recall exists.
      ⚠️ **Decide and test what happens at ZERO L1 keys and at TWO or more.** No criterion said so;
      the prototype needed three buckets (`interfaceless`, `multi_homed`, `collapsed`) to avoid losing
      a trap in silence, and `collapsed` is what produced §0j. Today no trap-named observation carries
      more than one MAC — a property of the corpus, not of the type.
- [x] **T7 — The mutation pass**, predictions written BEFORE any plant, each row naming its carrier.
      ⚠️ **Only the narrowings EXPRESSIBLE under `&[L1Key]` may be listed as mutations** — the
      hostname and uplink rows of §0e are corpus measurements, not plantable mutations here, and the
      first draft listed them as if they were:
      · narrow on `l2_domain` → predict **red 1, the synthetic alone; corpus GREEN**
      · narrow on MAC equality → predict **red ~5, the corpus assertion falls** (implausible, and
        that is AC3's real answer)
      · reorder the per-mille arithmetic → predict **red 1, ten gates GREEN**
      · shrink the truth set in silence → predict **red 2** (AC4's carrier)
      · call `decide` inside the generator → predict **GREEN**, and record it as the measured limit
        of AC1 rather than as a pass
      · delete the generator's body → predict red
      Record observed against predicted; **a divergence is a finding, not a correction.**
- [x] **T8 — Gates**: `cargo xtask ci` (ten gates), `cargo fmt --all`,
      `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked
      --no-fail-fast`. Record the count and the wall clock, and say whether a store was present.
- [x] **T10 — Repair `blocking.rs`'s module doc** (§0b): four stale figures, in the file this story
      extends and which the Dev Notes order the developer to cite. Not optional — a false doc is a
      defect, and citing it propagates four.
- [x] **T9 — Update the record**: this file (AC6), `sprint-status.yaml`, `docs/project-context.md`,
      `CLAUDE.md`'s status paragraph — and **`identity/mod.rs`**, whose *"the two organs do not
      consult each other"* deserves a re-read now that `blocking` imports `L1Key` in production code
      (the sentence stays literally true; importing a type alias is not calling).
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
  the L2 rules and **(2)** the five rules take the bucket 11 → 3 — ⚠️ **which §0j corrects to 11 → 4.**
- `architecture.md:1029-1037` — D13 on why a blocker exists: *"nobody tests blockers"* (`:1031`),
  *"without blocking, abstention has no denominator"* (`:1036`).
- `architecture.md:1013-1018` — the milli-units corollary AC2 rests on (`"INTEGER in milli-units"`
  at `:1016`).
- `architecture.md:891-894` — the L1/L2 split and the trap matrix: *"multi-NIC false-split = L1
  correct, L2 failed to group"*.
- `architecture.md:1272-1279` — D18, and why this is **not** the recall gate it refuses.
  🔴 _(The first draft cited `:1004-1011`, `:988-993` and `:1246-1253` for the three above — all
  **~25 lines off**, and all three are, to the digit, the numbers `blocking.rs`'s module doc carries.
  **They were INHERITED, not measured**, inside a §0 that promises everything in it was run against
  the tree. `l1.rs` carries the same drift. Only `:891-894` was right, and it is the only one absent
  from `blocking.rs`. Corrected by `grep` on the quoted sentences.)_
- `blocking.rs` module doc — the refusals. ⚠️ **Its three L1 narrowing scores and its "ten
  `must-merge`" are STALE**; see §0b's table and T10.
- `fixtures.rs:4658-4790` — `CorpusPairs`, `corpus_pairs()`, `blocking_recall_above_999` **and
  `every_trap_pair_is_in_the_universe`**: the pattern T6 mirrors in BOTH its natures.
- `sprint-status.yaml` — story 6.6's own note, *"5.6's rule: a blocker that consults a rule is that
  rule's echo"*. ⚠️ Cited by CONTENT, not by line: it was `:4826` on `8a19089` and the story's own
  commit pushed it to `:4848`. *A line citation can be made stale by the commit that carries it.*
- `deferred-work.md` — the `>= 999` boundary arithmetic (`:1874`), D17's `dormant` exclusion, and the
  quadratic-universe row. ⚠️ **Cite the LIVING clause of a register row, not its oldest occurrence**:
  the quadratic row's owner reads *"the first story that hands the resolver a slice it did not choose
  (a real scan)"* at `:2399` — discharged by story 5.14 — and *"nothing calls the blocker and the
  engine in sequence"* is marked ✅ CLOSED at `:2329`.

### Project context reference

`docs/project-context.md` and `CLAUDE.md`. ⚠️ Both carry DATED figures beside living ones; where they
disagree with a measurement taken on the tree, **the measurement wins and the document is corrected**
— that is the project's own rule, and §0a is this story's first application of it.

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context), 2026-08-30.

### The live count (AC6), every figure naming its state

**772 → 783 tests** (515 bin + 169 core + 99 xtask), measured with
`cargo test --workspace --locked --no-fail-fast`, wall clock, warm:
**0.22 s with no store** and **7.04 s against a live `mariadb:10.11.11` on port 13366** — *the clock
is the tell that the store-backed tests genuinely ran.* **Ten `cargo xtask ci` gates green**;
`float-free` walks **4** files under `identity/` (no new file landed — the L2 blocker extends
`blocking.rs`), `file-size` reports 43 files, largest 1978. 28 fixtures, **trap gate still RED at
26/15/11**, no migration, no dependency, no route, no screen, no write.

### What was built

`crates/opencmdb-core/src/identity/blocking.rs` gains `L2CandidatePair` (private fields ordered by
`new`, `new(a, a) -> None`) and `l2_candidates(&[L1Key]) -> BTreeSet<L2CandidatePair>` — TOTAL, no
narrowing key — and `blocking_recall_per_mille` becomes **generic over `<T: Ord>`** so the floor
constant is shared rather than the function duplicated. `crates/opencmdb-bin/src/fixtures.rs` gains
the corpus half: `l2_corpus()`, the recall assertion with its denominator asserted at **3**, the
per-stream containment assertion, and the COVERAGE assertion.

### T2's red was ASSERTION-carried, and that was arranged rather than hoped for

The guard was written together with a **deliberately narrowed** generator, so the first run reddened
on `left: 0, right: 1` — its own message — rather than on a compiler error. The narrowing was then
removed and the test went green. *A guard first seen red by the compiler has not been seen red.*

### The mutation pass — six mutations, predictions written BEFORE any plant

Driven by `cargo xtask mutate` against a live store, never a throw-away script.

| id | mutation | predicted | measured | verdict |
|---|---|---|---|---|
| M1 | narrow the L2 universe on `l2_domain` | red:1, corpus green | **red 2**, corpus green | 🔴 divergence — see below |
| M2 | narrow on MAC equality | red, the corpus falls | red 5 | ✅ |
| M3 | divide before scaling the per-mille | red:1, ten gates green | **red 3**, gates green | 🔴 divergence — see below |
| M4 | call `decide` inside the generator | **green** | **green** | ✅ AC1's measured limit |
| M5 | shrink the WALK's `l2-` filter to `l2-uplink` | red | red 3 | ✅ AC4 is load-bearing |
| M5b | shrink `required` only, walk intact (the review's shape) | red:2 | red 2 | ✅ the control that sizes M5 |
| M7 | route every 2-observation trap into `wrong_arity` | red | red 3 | ✅ the review's new bucket is carried |
| M6 | empty the generator's body | red | red 5, **clippy also red** | ✅ |

🔴 **BOTH DIVERGENCES REFUTED A SENTENCE OF MINE, and the sentences were corrected rather than the
measurements.** M1 was predicted to red the AC5 guard **alone**; it reds two, because
`l2_the_universe_is_total_over_distinct_interfaces` happens to pin a cross-domain pair as well. M3
was predicted to red the new truncation test alone; it reds three, the two L1 truncation tests
exercising the same arithmetic. In both cases the doc said *"the only carrier"* / *"the only thing
standing between that narrowing and green"*. **A claim of sole carriership is worth exactly the
mutation that checked it** — three doc comments now say what was measured, and say what they used to
say.

⚠️ **M4 is the story's most important GREEN.** Calling `decide` inside the blocker — literally the
act AC1 forbids — leaves 783 tests, clippy and all ten gates green. It is recorded as **AC1's
measured limit**, not as a pass: the `join`/`decide_pair` half of the refusal is carried by the TYPE
(neither is reachable without `Observation`s), the `decide` half is a **TRIPWIRE**.

### Code review — three isolated layers on a DIFFERENT model, 2026-08-30

Three layers, each on Sonnet where the implementation was Opus, each isolated: a **blind** layer
given the code diff and nothing else, an **edge-case** layer with its own worktree and store, and an
**acceptance auditor**. **Seven distinct findings; the numbers, citations and mutations they
re-measured all reproduced.**

🔴 **THE HIGH IS THE STORY'S OWN LESSON, COMMITTED IN THE FILE ITS OWN TASK NAMED.**
`identity/mod.rs:21` still asserted *"the blocker still has no production caller at all"* — true when
written, **false since story 5.9b** (`resolver.rs:211` calls `candidates` in production), and the
register marks the matching row ✅ CLOSED. §0g of this very story corrects **exactly that confusion
about exactly that sentence**, two files away; T9 named `identity/mod.rs` and re-read the sentence
*beside* it, which is true, while the false one sat two lines above. 🔑 ***A promise to re-read is
not a re-read of what you did not look at.*** Corrected in place, with what it used to say.

🔴 **THE EPIC'S DOMINANT CLASS, IN MY OWN CODE, FOUND BY THE BLIND LAYER FROM THE DIFF ALONE.**
`every_l2_trap_pair_is_in_the_universe`'s containment loop **cannot red on its own**: `pair` is built
from `groups`, the universe from the keys of that same `groups`, so a total `l2_candidates` contains
it by construction. 🔑 **Its L1 twin is NOT a corollary** — there the pair comes from the `obs_id`s a
trap NAMES, which need not appear in the stream at all. *I inherited the shape without inheriting the
property.* What actually carries the test is the BUCKET assertions, which is also what found §0j. The
loop is kept as a labelled second oracle and its doc now says which half carries what.

🔴 **A BUCKET THAT WOULD HAVE ACCUSED THE WRONG CAUSE** (blind and edge layers, independently): the
`let [a, b] = … else` arm filed **any** wrong-arity trap under `interfaceless`, whose message reads
*"names an observation without an L1 key"*. Empty today — all eight `l2-*` traps name a pair — so a
naming hole and not a coverage hole, and closed before the corpus grows into it: `wrong_arity` is now
its own bucket with its own message.

🔑 **M5 DID NOT REPRODUCE FOR EITHER SIGHTED LAYER, AND THE RESOLUTION IS A MEASUREMENT WITH A
CONTROL.** Both measured **2** where the table said 3. Re-run on a **virgin database with
`--baseline`**: my injection gives **3** and theirs gives **2**, both confirmed. Neither figure was
wrong — mine shrinks the WALK, so `pairs.len() == 7` falls too; theirs shrinks `required` alone. ⚠️
**The row's NAME described their mutation while its number described mine**, which is this project's
four-time class *a mutation named for one thing and applied to another*. The table now carries both,
M5b as the control that sizes M5.

⚠️ **AND THE SIX ORIGINAL MUTATIONS WERE RUN WITHOUT `--baseline`, the driver warning every time.**
They held on re-measurement, but that is luck being confirmed rather than rigour: on a reused
database this suite is non-deterministic (registered), so a red count taken without a baseline is a
guess. *The tool built to stop exactly this told me, once per run, and I read past it.*

✅ **Reproduced by the layers at the figure**: 727 / 363 / 818 / 909 re-derived independently by
both sighted layers; the five `architecture.md` citations verified by grep on the quoted sentences,
not on the numbers; `E0308` for `join` and `E0425` for `uplink_of` reproduced, so AC1's *half carried
by the type* and AC5's withdrawal are both established rather than argued; M1, M2, M3, M4 and M6
conform; the four register rows exist with a live owner each; **both terms of `772 → 783`** checked
against `master` rather than by re-adding the parts.

✅ **Refuted with their checks**: §0b's `259 / line 260 / total 661` are the **baseline** figures and
§0b says so — the edge layer verified them against `8a19089` rather than the working tree; and
`cargo doc` shows no new broken intra-doc link.

### 🔑 The coverage assertion confirmed §0j on its own

`every_l2_trap_pair_is_in_the_universe` names `cloned-mac-must-not-merge` in its `collapsed` bucket
and passed on the first run — an independent confirmation of the arbitration, reached by the
assertion rather than by the argument. **The first draft of T6 prescribed only the recall half**; the
coverage half was added at the validation, and it is the half that carries this.

### AC3 is ANSWERED, not ticked

Of the four narrowings §0e measures, exactly one is expressible inside `l2_candidates` (`l2_domain`)
and the corpus is blind to it; the only narrowing that reds the corpus assertion is MAC equality,
which is the inverse of what L2 does. **So no plausible AND expressible narrowing can red the L2
recall assertion**, and the criterion is met by the synthetic guard plus the recorded measurement,
never by the corpus.

### T10 — `blocking.rs`'s module doc repaired, figures re-measured myself

Four stale figures (ten `must-merge`, 700‰, 400‰, 900‰) → **eleven, 727‰, 363‰ strict / 818‰ loose,
909‰**, re-derived over the committed corpus rather than taken from the validation report. **Five
`architecture.md` citations corrected in this file** (`:988-993` → `:1013-1018`, `:1004-1007` →
`:1029-1032`, `:1009-1011` → `:1034-1036`, `:1009` → `:1034`, `:1246-1253` → `:1272-1279`), each
re-derived by `grep` on the quoted sentence. ⚠️ **`l1.rs` carries the same drift and is NOT touched
here** — registered, owner story 6.7.

### File List

- `crates/opencmdb-core/src/identity/blocking.rs` — MODIFIED
- `crates/opencmdb-bin/src/fixtures.rs` — MODIFIED
- `_bmad-output/implementation-artifacts/6-6-l2-candidates-and-no-rule.md` — MODIFIED
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — MODIFIED

## Change Log

| Date | Change |
|---|---|
| 2026-08-30 | Story created and contexted. §0 measured against `8a19089`: the L2 truth set is **three** pairs (§0c), two of four narrowing keys are **invisible to the committed corpus** (§0e), and two premises inherited from `sprint-status.yaml` were **refuted** (§0a). One arbitration left open (§0f). |
| 2026-08-30 | **VALIDATED** by two fresh-context layers (fact-check + gap-hunt, own worktree each; the gap-hunt layer BUILT the prototype and ran eight mutations against a live store). **Sixteen findings, six HIGH, all applied in place.** Both arbitrations TAKEN and recorded as mine, delegated: **§0f** — the input stays `&[L1Key]` and the TYPE replaces the guard, so **AC5's uplink half is WITHDRAWN as unwritable** (`E0425`); **§0j** — the L2 rules judge INTERFACE pairs, so the unanswerable bucket goes **11 → 4, not 11 → 3**. Corrected: three `architecture.md` citations ~25 lines off and **inherited rather than measured**; *"no lifecycle state"*, false since story 6.5; a register line quoted at its oldest occurrence while it is marked ✅ CLOSED; 333‰ needed a second row at **666‰** under the natural reading; 260 → **259** code lines; *"verbatim"* → *rendered*; the `InterfaceId` refusal's stated motive, which did not hold. Added: T6's missing **COVERAGE** half — the assertion that found §0j — a truncation test `float-free` cannot replace, T10 for `blocking.rs`'s four stale figures, and four register rows with named owners. |
| 2026-08-30 | **IMPLEMENTED.** `L2CandidatePair` + `l2_candidates` (TOTAL, `&[L1Key]`) in `blocking.rs`; `blocking_recall_per_mille` made **generic** rather than duplicated; the corpus half in `fixtures.rs` — recall with its denominator asserted at **3**, per-stream containment, and the **COVERAGE** assertion, which **confirmed §0j on its own first run** by naming `cloned-mac-must-not-merge`. **772 → 783 tests**, ten gates, clippy `--all-targets`. Six mutations, predictions written first: 🔴 **two diverged and both refuted a sentence of mine** — M1 reds 2 not 1, M3 reds 3 not 1, so three doc comments claiming *"the only carrier"* were corrected. ⚠️ **M4 is green by measurement**: calling `decide` inside the blocker changes nothing, recorded as AC1's limit and a TRIPWIRE. T10 repaired four stale figures and five `architecture.md` citations in `blocking.rs`, each re-derived by grep. |
| 2026-08-30 | **CODE-REVIEWED** by three isolated layers on a different model, and REPAIRED. Seven findings. 🔴 The HIGH is this story's own lesson committed in the file its own T9 named: `identity/mod.rs` still said *"the blocker still has no production caller at all"*, false since 5.9b, two lines from the sentence T9 did re-read. 🔴 The blind layer found, **from the diff alone**, that the coverage loop cannot red on its own — a corollary of totality where its L1 twin is not — and two layers independently found the `interfaceless` bucket filing wrong-arity traps under a message accusing the wrong cause. 🔑 M5 reproduced for neither sighted layer; re-measured on a **virgin database with `--baseline`**, my injection gives 3 and theirs gives 2 — **the row's NAME described their mutation and its number described mine**. M5b added as the control. ⚠️ And the six original mutations were run **without `--baseline`**, the driver warning every time. Three register rows written, including the non-determinism of this suite against a reused database, recorded as a SYMPTOM with no cause. |
