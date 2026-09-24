# Story 6.11: The virtual-MAC anchor — a structural fact that is NOT a rule

Status: **validated** 2026-09-24 by two fresh-context layers, the gap-hunt having BUILT both shapes of
§0.1, driven the trap end to end and run a nine-row mutation pass in which **five came back GREEN**.
**Guy's arbitration taken 2026-09-24** — the reading takes an `L2CandidatePair`, so *what the function
reads* carries the claim rather than its arity. Ready for dev.

**Epic 6**, on Guy's reorder of 2026-09-23: 6.9 (done) → **6.11** → 6.12, with 6.8 and 6.10 waiting for
a connector that emits an uplink or a switch port. **Base:** `8b36828`, clean tree.

---

## 0. What contexting measured, and the three things it will not settle alone

### §0.0 — ✅ UNLIKE STORY 6.9, THE ALGEBRA ADMITS A PASS HERE — and *"the trap is answerable"* is a different sentence that is FALSE

⚠️ **This section said *"`vrrp-virtual-mac-must-not-merge-master` is answerable and can pass"*, and the
fact-check layer refuted the present tense from a named assertion in the tree**: the trap is listed in
`l1_runner.rs`'s `expected_unanswered()` — *"the eleven traps L1 cannot answer"* — and in three more
committed lists. What the table below proves is that **`score`'s truth table admits a pass**, which is a
statement about the ALGEBRA and not about the trap being answered. *Distinguish the two, because AC1's
"driven end to end" reads as the second on first reading.*

Story 6.9 measured that a `Supports` rule can never satisfy a `must-merge` trap, because `decide`
reaches `Match` through one arm and that arm needs a `Decisive`. **The mirror question for this story
has the opposite answer, and it is measured rather than assumed:**

| | |
|---|---|
| `score.rs:279` | `(MustNotMerge, Refused) => Score::Pass` |
| `score.rs:280` | `(MustNotMerge, Abstained) => Score::Pass` — the tolerant cell `must-merge` lacks |
| `cascade.rs` | `Refused { rule }` ← `NoMatch { rule }` ← `(Some(rule), _, _, _)` — **a `Disqualifying`** |

So `vrrp-virtual-mac-must-not-merge-master` **is answerable and can pass**, by one path: something must
emit a `Disqualifying` carrying `RuleId("l2-virtual-mac-prefix")`. 🔑 *This is the first L2 trap of the
epic that can turn green on its own merits, and story 6.9's finding is what makes that worth stating
rather than assuming.*

### §0.1 — ~~THE FIRST QUESTION: what is *not a rule* must enter the verdict vector AS one~~ — the question SURVIVES and my answer to it was REFUTED by both layers, from two directions

**What survives, and it is measured**: `NoMatch { rule }` is reachable only from a `RuleVerdict` carrying
`Verdict::Disqualifying`. The fact-check layer swept every `Conclusion::NoMatch` construction in
`crates/` and `xtask/` — **`cascade.rs:524` is the only one outside a test module**. ⚠️ *One caveat it
added and I had not*: `Decision`'s fields are `pub` with no constructor, so *there is no second door
**through `decide`**, and the type permits one* — a residue the tree already registers.

~~The candidate distinction: a RULE compares the two sides; a READING is a property of ONE side's bytes,
so the function's signature carries the claim — a reading takes one side and cannot take two.~~

🔴 **REFUTED, and the two layers reached it by different roads.**

**The fact-check read the bytes.** `…0001` carries the virtual MAC in **both** pairs the family names —
the master pair (`0001`+`0002`) *and* the `must-merge` (`0001`+`0004`, one gateway re-seen after a
failover). So a one-sided predicate fires on the `must-merge` pair too, where `Disqualifying` takes
absolute priority: `NoMatch` → `Refused` → **`(MustMerge, Refused) => Fail`**. *My §0.1 recommended the
shape my own AC4 forbids, four paragraphs before warning against it.*

**The gap-hunt BUILT it, and it is worse than a sentence being wrong.** `L2Side` **carries no key** — its
own doc says so, citing D45 — so a one-sided reading must dig MACs out of the side's `facts`, and an
observation bearing several MACs stands on several interfaces. Measured, one observation carrying
`02:00:5e:00:53:8c` **and** `00:00:5e:00:01:0a`:

```
reading over the REAL interface's SIDE      -> Disqualifying   <-- the WRONG interface
reading over that interface's own KEY MAC   -> Neutral
```

🔑 **The reading disqualifies a perfectly ordinary interface because some observation standing on it also
bore a virtual MAC — and that IS the ordinary VRRP geometry**, a router port bearing its own address and
the VIP. ⚠️ **And the corpus cannot separate the two shapes**: over all **31** interfaces in every
trap-named stream, **0 disagreements**. A story picking either would see no difference and a review layer
reading tests could not tell them apart.

### §0.1b — ✅ THE QUESTION AS IT REALLY STOOD, and Guy's answer to it

To be correct the reading needs the **KEY**, not a side — at which point `L2Side` is useless to it. And it
must be **two-sided**, because the distinction it exists to draw is *do these two interfaces share the
virtual MAC* (one gateway re-seen → merge) or *does one of them bear it and the other not* (the VIP
against its master → refuse).

🔑 **That type already exists and story 6.6 minted it**: `L2CandidatePair` holds two `L1Key`s privately,
`L1Key` **is** `(L2DomainId, MacAddr)`, `low()` and `high()` are public, and `new(a, a)` returns `None` —
the self-pair closed in the type. It is what `l2_candidates` produces.

So the shape that makes *"there is no rule"* structural is **not arity — it is WHAT THE FUNCTION READS**:

| | takes | reads | needs |
|---|---|---|---|
| a **RULE** | two `L2Side`s | the **FACTS** the sides carry | the observations |
| a **READING** | one `L2CandidatePair` | the **KEY** and nothing else | no observation at all |

⚠️ **A function that cannot see a `Fact` cannot score**, and that is checkable by the compiler rather
than promised in prose — this project's own precedent (story 6.6: *the TYPE replaces the guard*). The
alternatives, with their costs:

- **(a) `verdict_for_virtual_mac(&L2CandidatePair) -> RuleVerdict`** — ✅ **GUY'S DECISION, 2026-09-24.**
  Reads the key, cannot reach a fact, two-sided, and reuses story 6.6's type. 🔑 *The distinction that
  makes "there is no rule" structural is therefore not arity but WHAT THE FUNCTION READS, and the compiler
  checks it: a function that cannot see a `Fact` cannot score.* ⚠️ **The two costs are accepted in
  writing**: `l2.rs` holds two signature conventions from now on — and the story must SAY which is which
  rather than leaving a reader to infer it — and **the evidence question sharpens rather than resolves**,
  because a pair of keys carries no `ObsId` at all, so AC6 must decide what a `Disqualifying` leaves
  behind when the function that produced it never saw an observation.
- **(b) one `MacAddr`** — REFUSED. Correct about the key and **one-sided**, so it cannot draw the
  distinction AC4 needs and the caller must draw it instead — *which moves the decision out of the audited
  function*, and that is where this project has repeatedly lost its guards.
- **(c) two `L2Side`s, the existing convention** — REFUSED, and **measured wrong** rather than merely
  unproven, with the corpus unable to say so (0 disagreements over 31 interfaces).

### §0.2 — 🔴 THE SECOND QUESTION — and the table MISSED the only PRODUCTION site, which already owns this question

🔴 **`crates/opencmdb-bin/src/neighbour.rs:178`, `fn is_an_identity(mac) -> bool`, in the SHIPPED
connector's ingestion path**, documented as *"whether a hardware address can stand for a THING"*. Its doc
already argues D13, the U/L bit and story 5.5's refusal, and routes the open question to **GitHub issue
#162**. Its test at `:402` asserts a VRRP MAC **is** an identity.

⚠️ **My table counted three representations and none of them was this one — and both of the three I
counted are TEST code** (`fixtures.rs:1372` sits inside the `#[cfg(test)]` opening at `:921`;
`l1.rs:662` inside the one at `:389`). So the real picture:

| where | kind | why |
|---|---|---|
| `neighbour.rs:178` `is_an_identity` | **PRODUCTION**, the connector's ingestion filter | may this MAC stand for a thing at all |
| `fixtures.rs:1372` `is_synthetic_mac` | test oracle | the corpus PRIVACY allowlist |
| `l1.rs:662` | test oracle | story 5.5's pin that the fixture is really in the range |
| **6.11** | production | may this MAC anchor a GROUPING |

🔴 **Adding 6.11's judgement without naming `is_an_identity` gives two PRODUCTION predicates, in two
crates, each with a doc citing D13, neither aware of the other** — worse drift than the three I counted.
And **M8 measures the cost of the misreading AC1's own words invite**: adding the IANA refusal to
`is_an_identity` — the *literal* ingestion site — reds **exactly one test, named for a different
property** (`a_locally_administered_address_is_still_an_identity`), so a dev meets a single red pointing
at the wrong concern and may well "update" it. No trap, no corpus test, no gate reds, because **the corpus
never passes through the connector's parser**.

🔑 **And `neighbour.rs:167-172` already contains the sentence AC2 asks this story to write**, one crate
away: *"the locally-administered bit is DELIBERATELY not here … D13 calls it disqualifying **as a grouping
anchor** — which is an L2 judgement about how much a MAC is worth, not an L1 judgement about whether it is
an address at all."* **AC7 must cite it rather than re-derive it.**

**The DRY fork survives and its reason is STRONGER than I gave**: keep the representations separate and
**pin them equal**, because `CLAUDE.md`'s DRY rule names exactly this shape as protected — *"a test that
restates the corpus bytes as a second independent oracle"* — and the corpus's allowlist is **CLOSED by
decision** while the engine's set is whatever IANA assigns. ⚠️ *My stated reason — "a privacy widening
silently widens the engine" — describes a test helper, so it is weaker than the reason available.*

🔴 **AND MY OWN PROPOSED REMEDY WALKS THROUGH.** The gap-hunt built option (b) literally — a loop
asserting the two predicates agree over `0..=255` — and then widened the engine's predicate from **five
octets to four**, admitting the IANA VRRP **IPv6** block `00:00:5e:00:02:xx`. **755 tests, clippy and all
ten gates GREEN, the pin included**, because the pin enumerates the *last* octet and never crosses the
boundary. 🔑 *An enumeration cannot claim the completeness of a property* — **the pin must vary the fourth
and fifth octets, not the sixth.**

⚠️ **And the five-octet width is load-bearing against the product's own shipped dataset**:
`example_data.rs` carries **thirty-odd production MACs of the form `00:00:5E:00:53:xx`** — RFC 7042
documentation addresses sharing the IANA OUI `00:00:5e` and differing only at the fifth octet. **A
predicate written on three octets would disqualify the entire example inventory.** That belongs in AC5 as
the reason the width is not a detail.

### §0.3 — ⚠️ THE THIRD QUESTION: the trap's own stated temptation is NOT LIVE today

`vrrp-virtual-mac.toml`'s header argues the rule's necessity like this:

> *"No existing id can oppose that pair: `multi-nic`'s own must-merge REWARDS grouping distinct-MAC
> interfaces whose uplink agrees (`l2-uplink-agrees`), and V1 shares `doc-rtr-alpha`'s exact uplink —
> **an engine following the committed vocabulary to the letter would fold the VIP into its master.**
> Only the prefix reading stops it."*

🔴 **Measured: `l2-uplink-agrees` is story 6.8's, and 6.8 is WAITING** — `Fact::Uplink` has zero
occurrences in `arp_ping.rs` and Guy's reorder of 2026-09-23 parked 6.8 for that reason. So **nothing
in the engine today would fold the VIP into its master**, and the temptation the trap exists to defeat
is currently unreachable.

⚠️ That does **not** make the story premature — the trap expects `l2-virtual-mac-prefix` and cannot pass
without it, whatever the temptation's state. But it changes what the story may CLAIM: *the reading
stops a fold that no shipped rule can currently perform*, which is a guard placed **before** the defect
rather than where it cannot occur. 🔑 *The distinction matters because this epic's dominant class is a
guard where the defect cannot occur, and this is the honest cousin of it: a guard that is early rather
than misplaced.* The story must say which it is, and 6.8's arrival is what converts it.

⚠️ **Widened by one, measured**: `l2-different-switch` is unimplemented too, and the trap header's
argument that the `must-merge` is the family's teeth rests on it *"overcoming a committed OPPOSING L2
signal — the committed shape of `l2-different-switch"*. **So TWO of the rules the header reasons with
do not exist.**

### §0.4 — 🔴 THE FINDING NEITHER LAYER STATES AS ONE SENTENCE: AC4's third pole has NO CARRIER at the level this story writes

The fact-check measured that a one-sided reading breaks the `must-merge`. The gap-hunt measured that the
pole **holds at L1** — `…0001`/`…0004` share an interface, `decide_pair` gives `Match { l1-exact-mac }` —
and that **M7**, the literal reading of *"read at ingestion"* implemented by dropping IANA MACs out of
`l1::keys_of`, reds **ten** tests including story 5.5's pin and four `trap_gate` tests. **The L1-level
misreading is well guarded.**

🔴 **But the L2-level one — the one this story actually writes — is invisible to the corpus.** The gap-hunt
set out to confirm that the careless *suspicious MAC → refuse* reading breaks the `must-merge` pole
through the gate, and **refuted its own suspicion with something worse**: the gate routes
`vrrp-virtual-mac-must-merge` to the **L1** runner by its `l1-` rule prefix, so an L2-level careless
reading never reaches it. Measured directly instead — composing the careless reading with L1's `Decisive`
for that same pair gives `NoMatch { l2-virtual-mac-prefix }` / `Refused`: **the failover re-sighting
refused, with the trap gate GREEN.**

🔑 *So AC4 names the right danger and the corpus cannot see the form of it this story can commit.* The pole
needs a synthetic guard at L2, and saying so is the criterion.

### §0.5 — 🔴 A PARAGRAPH FROM 6.7 AND 6.9 THAT MUST NOT BE COPIED, because its mechanism does not transfer

Both earlier L2 stories carry a long, load-bearing paragraph: *the trap gate cannot catch a misspelled
rule id, because an abstention names no rule, so a double-literal pin is the carrier.* 🔴 **That reasoning
is FALSE here.** `run_trap` compares rules when **both** sides name one; 6.7's `Opposes` and 6.9's
`Supports` both abstain, and an abstention carries no rule — but **a `Disqualifying` yields
`Refused { rule }`, which DOES carry one.** So 6.11 is the **first L2 trap in this epic whose expected rule
the gate's comparison can actually check.**

⚠️ **The pin is still the only carrier TODAY, for a completely different reason: there is no runner to
ask.** 🔑 *A paragraph inherited with its old mechanism is this project's recurring defect — the story must
state the new mechanism or the next reader inherits a refuted one.*

---

## 1. Measured facts

- **6.11's trap is ONE**: `vrrp-virtual-mac-must-not-merge-master` (observations `…0001` / `…0002`,
  `expect = { must-not-merge = { rule = "l2-virtual-mac-prefix" } }`). The family's other two are
  already owned: `-bearers` expects `l2-different-hostname` (story **6.7**'s, and 6.7's corpus test
  already asserts its `Opposes`) and `must-merge` expects `l1-exact-mac` (L1's, **answered today**).
- 🔴 **6.11 would be the SECOND producer of `Verdict::Disqualifying` and the FIRST at L2.** The only
  producer today is `l1-distinct-mac` (`l1.rs:281`).
- 🔴 **Story 5.5's refusal is pinned by TWO tests and 6.11 must not red them**:
  `an_exact_match_fires_on_a_locally_administered_mac` and
  `an_exact_match_fires_on_a_redundancy_protocol_mac` (`l1.rs:658`), the second asserting the fixture
  really sits in the IANA range and that *"L1 is deterministic on the key, so the exact-MAC rule fires
  for a virtual MAC exactly as for any other"*. **The virtual-device recognition is layered ON TOP of
  L1, never inside it** — the trap file says so itself, and its `must-merge` deliberately does not cite
  the prefix reading.
- **No production predicate exists**: no IANA or VRRP byte test anywhere under `crates/*/src` outside
  test modules and the corpus privacy walk. `Fact::Mac` carries `addr: MacAddr` and
  `locally_administered: bool` — **the U/L bit is already a structural reading** that reaches the
  engine, so the shape has a precedent in the data model.
- **D16 REJECTED both easy outs**, in the trap header's own words: folding the VIP into its master
  (*"choosing a winner between two legitimate owners is merging"*) and abstaining (*"a SEMANTIC
  DUSTBIN … this is not an ambiguity, it is a topology fact"*). ⚠️ **That second rejection is why the
  family has NO `must-abstain` pole** — *the absent third column is itself a spec assertion, not an
  omission* — and it forbids the cheapest implementation, which is to abstain and call it honest.
- ⚠️ **The corpus's IANA allowlist is CLOSED by decision** and HSRP is out until a fixture commits its
  octets *"with its own prove-to-red"*. A story that widens the engine's set beyond VRRP without a
  fixture would be minting reach the corpus cannot measure.
- **`float-free` walks 5 files** under `identity/`; a new module there makes **6**, which is a figure to
  measure rather than predict.
- ⚠️ **`vrrp-virtual-mac-must-merge` is the family's teeth against `cloned-mac`**: that family taught
  *same MAC, different hostnames → refuse*, and an engine generalising it to *suspicious MAC → refuse*
  **is demolished here**. So a predicate that refuses grouping on the prefix must not also refuse the
  L1 re-sighting of one virtual gateway across a failover — the two families hold each other in check,
  and this story is where a careless reading breaks both.

---

## 2. Acceptance criteria — rewritten on the validation's measurements

🔴 **The gap-hunt built the test set these criteria literally demanded and no more, then ran nine
mutations: FIVE came back GREEN.** Each AC below names what that measured, so the criteria carry the holes
rather than the dev rediscovering them.

**AC1 — the STRONG oracle, named explicitly: `decide` over an L2-ONLY vector answers
`Conclusion::NoMatch { rule: l2-virtual-mac-prefix }`** for the master pair, driven over the committed
bytes. 🔴 *Not "the trap is answered" and not "it can pass"* — **M5** (`Disqualifying` → `Opposes`) reds
**exactly one** test, the one asserting the conclusion, while the weak *"the trap scores a pass"* oracle
stayed **GREEN** (a lone `Opposes` abstains on `AbsenceOfProof`, and that cell passes too). ⚠️ And a
one-sided reading applied to the **wrong side** also passes, because the master trap is **asymmetric in
argument order**: had the TOML listed `…0002` first, such a reading would have "passed" while doing
nothing.

✅ **The L2-only vector is measured sufficient — 6.11 does NOT need story 6.12's plumbing first**, which
contexting had feared. The gap-hunt built it: `Neutral` · `Neutral` · the reading's `Disqualifying` →
`NoMatch { l2-virtual-mac-prefix }` → `Refused` → `(MustNotMerge, Refused) => Pass`.

🔴 **And the forbidden gesture is fatal TOTALLY, not per case.** `decide` names the lexicographically
smallest `Disqualifying`, and **every `l1-*` id sorts before every `l2-*` id**, so **no L2 rule can ever be
named in the presence of any L1 `Disqualifying`, for any trap, ever.** `l2.rs`'s header states this as a
per-case argument; it is a property of the naming convention meeting `min()`, and it is registered.

**AC2 — the decision NAMES `l2-virtual-mac-prefix`, and the story says WHY a rule identifier names
something that is not a rule** — 🔑 **by citing `neighbour.rs:167-172`, which already contains that
sentence**, rather than re-deriving it (§0.2).

**AC3 — story 5.5's refusals stay GREEN, and the count is THREE sites not two**: the two `l1.rs` pins,
**plus `neighbour.rs:402`'s `a_locally_administered_address_is_still_an_identity`, which asserts a VRRP MAC
IS an identity** at the very ingestion boundary AC1's wording names. ✅ M7 measures the L1-level
misreading at **ten** reds; **M8 measures the ingestion-level one at ONE**, named for a different property.

**AC4 — the three poles hold, AND the third gets a SYNTHETIC guard at L2** (§0.4). 🔴 *The corpus cannot
see the form of the defect this story can commit*: composing the careless reading with L1's `Decisive`
refuses the failover re-sighting **with the trap gate green**, because the gate routes that trap to the L1
runner by its `l1-` prefix.

**AC5 — the byte width is FIVE octets and the pin varies the FOURTH and FIFTH, not the sixth.** 🔴 **M1**
widened five to four — admitting the IANA VRRP **IPv6** block — and left 755 tests, clippy, ten gates
**and the equality pin itself GREEN**. ⚠️ And the width is load-bearing against the product's own
`example_data.rs`, whose thirty-odd `00:00:5E:00:53:xx` documentation MACs share the IANA OUI: **a
three-octet predicate disqualifies the entire example inventory.**

**AC6 — the reading carries its EVIDENCE, and what it carries is DECIDED rather than inherited.** 🔴 **M3**
dropped it entirely → **GREEN**; no AC mentioned it and D19 does. ⚠️ *And the content is a real decision*:
a one-sided reading naturally carries only the bearing side's ids (measured `[…0001, …0004]`, never
`…0002`), so **an operator reading the refusal cannot see which pair was refused** — while a pair-shaped
one carries all three through `evidence_of`. Under §0.1b(a) the reading holds two KEYS and no `ObsId` at
all, which sharpens the question rather than answering it.

**AC7 — the reach and its limits are STATED *and probed*.** 🔴 **M2** added HSRP `00:00:0c:07:ac` — the
widening the trap header forbids in writing — and **the whole suite, clippy and ten gates stayed GREEN**. A
negative probe costs one line. ⚠️ **The allowlist is closed against HSRP *and HSRPv2*** — contexting
dropped the second from a quotation it presented as verbatim, and silenced the plural `stay` into `stays`.
⚠️ And D13's OTHER half is **unimplementable against this corpus, measured**: 40 of the 42 committed
`Fact::Mac` facts are locally administered, because the privacy rule forces it — so implementing the U/L
half as `Disqualifying` would disqualify **40 of 42 committed anchors**. Its owner is **issue #162**, open,
measured on the reference LAN at **19 U/L of 64 neighbours, 11 of them Docker MACs derived from the host's
own IPv4** — *not a false signal, an empty one*. The story must name #162 and say it does not close it.

**AC8 — the quantifier over a side's observations is DECIDED.** 🔴 **M4** changed *any observation bears a
virtual MAC* to *all do* → **GREEN**, because the master trap's side A holds two observations and **both**
bear it, so the corpus exercises only where the quantifiers coincide. The mixed side is where they
diverge. ⚠️ Also uncovered and measured: an **empty** side → `Neutral` (safe), and **both sides virtual** →
`Disqualifying` (right, for two different VRIDs, and named by no criterion).

**AC9 — the reading trusts the BYTES, not the connector's flag, and a test says so.** 🔴 **M6** made it
consult `locally_administered` → **GREEN**, because the fixture's flag happens to be `false`. ⚠️ *A
connector lying about that flag would silently disable the disqualification and nothing would notice* —
and `MacAddr::is_locally_administered()` is documented as *"the ground truth a connector's reported flag
can be cross-checked against"* while **nothing cross-checks**.

**AC10 — no regression**: ten gates, clippy `--all-targets`, `RUSTFLAGS="-D warnings"`, fmt, both store
conditions. ⚠️ **`float-free` measures 5 files if the reading extends `l2.rs` and 6 if a sibling opens** —
a function of §0.1b's decision, to be **measured and not predicted**. `file-size` is a non-issue
(`l2.rs` at 425 code lines against 2000). ⚠️ **The two browser gates are NOT claimed**: this story renders
nothing, and saying so is the criterion.

**AC11 — the trap gate does NOT move, and the story prices that.** 🔴 The trap is hard-coded as unanswerable
in **four** committed lists (`l1_runner.rs`'s `expected_unanswered()` and its declined-level map,
`trap_gate.rs:855` and `:1176`), plus `the_report_line_says_fifteen_scored`. Implementing the reading moves
**nothing**: the gate stays **26/15/11**. *A story whose criterion reads "driven end to end" owes that
sentence.*

## 3. What this story must NOT do

- **Not implement the reading inside L1.** Story 5.5 refused it there with two pinning tests and the
  trap file's own argument; reopening it would red the corpus's `must-merge`.
- **Not abstain.** D16 rejected it by name — *"a semantic dustbin … this is not an ambiguity, it is a
  topology fact"* — and `score.rs:280` makes an abstention a PASS on this column, so abstaining would
  make the trap green while betraying the decision. 🔑 *The tolerant cell is exactly what makes the
  cowardly implementation look like a success here.*
- **Not widen the allowlist beyond VRRP** without a committed fixture and its own prove-to-red.
- **Not bump the corpus.** `epics.md`'s AC2 says so in those words.
- **Not edit `epics.md`.** §0.1's tension is posed; if it resolves against the epic's letter, that is
  Guy's.
- **Not promise the trap gate closes.** Story 6.9 measured that three L2 `must-merge` traps cannot pass
  under the present algebra, so no L2 story may claim NFR4 or J4.
- 🔴 **Not copy 6.7's and 6.9's "the gate cannot catch a misspelled id" paragraph** — §0.5: its mechanism
  does not transfer, because a `Disqualifying` carries a rule where an abstention does not.
- ⚠️ **Not close story 5.5's third neighbour either.** `l1.rs`'s
  `a_group_address_merges_today_and_that_is_unresolved` — the broadcast MAC merges at L1 — says in its own
  doc that *"if a later story decides to refuse group addresses at L1, THIS test is the one that must red
  and be rewritten"*. 6.11 is the nearest story to *a MAC that must not anchor a grouping* and **is not that
  story**; one line saying so stops the next reader assuming otherwise. (Measured: multicast
  `33:33:ff:00:60:0a` and HSRP both read `Neutral`.)

---

## 4. What the two validation layers found, and the two refusals that are results

✅ **The measurements of the EXISTING tree came through**: every line citation the fact-check checked lands
right, the live count is exact, `float-free` really walks 5, story 5.5's two pins exist at the cited lines
and pass, the family holds exactly three traps expecting what contexting said, and 6.7's corpus test really
does name `-bearers`. **The defects were all in contexting's INFERENCES about what 6.11 can deliver** — and
the two sharpest were a section contradicting another section of the same document, and a shape that is
wrong rather than merely unproven.

🔴 **FIVE of nine mutations came back GREEN** against the test set the criteria literally demanded. That is
the headline: *the criteria as drafted left five ordinary mutations invisible*, and AC5 through AC9 exist
because of them.

⚠️ **TWO DRIVER REFUSALS, and both are results.** `cargo xtask mutate` refused a run with `DATABASE_URL`
absent — *"every store-backed test passes by RETURNING and the totals are identical to a real run"* — and
then refused a run whose **baseline clippy was RED** in the gap-hunt's own scratch build, *catching a defect
in the measuring instrument before it could be attributed to a mutation*. 🔑 That is the second review in a
row where `--baseline` earned its place, and this time it protected the layer from itself.

✅ **Refuted suspicions, recorded with the check so nobody re-chases them**: the pair does **not** collapse
(3 interfaces, distinct keys, and the committed `corpus.pairs.len() == 7` already counts this trap); D13's
U/L half is **not** unowned (issue #162, with the distinction already drawn at `neighbour.rs:167`);
`is_synthetic_mac` is **not** production code; and AC1 does **not** need the forbidden L1+L2 gesture.

⚠️ **Each layer ran with a DATABASE OF ITS OWN** — `factcheck_6_11` and `gaphunt_6_11` — which is the row
story 6.9's review registered after one shared database produced four bogus baselines and a published false
cause. **It held: no layer reported a dirty baseline it could not explain.**

---

## Record

- live-count: bin=748 core=204 xtask=110
- base: 8b3682844ca5f90caea894289ff66be3d3bfa595
- registered:
- file: _bmad-output/implementation-artifacts/6-11-virtual-mac-anchor-is-not-a-rule.md
