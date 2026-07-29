# Story 5.4: `Decision` — the engine's return type, and its ruleset version

Status: review

<!-- Validation is MANDATORY here (Guy's decision, Epic 4 retrospective 2026-07-26): two
     fresh-context agents (fact-check + gap-hunt) BEFORE dev-story. The template banner saying
     "Validation is optional" does not apply to this project. -->

## Story

As the identity engine,
I want a return type carrying the verdict vector, the conclusion, its evidence and the ruleset
version,
so that the explanation is free (D13) and improving the engine is not a silent data migration (D14).

**This story writes TYPES and their tests. It writes no algebra.** There is no `decide()`, no
implementation of D13's six-row table, no rule, no join, no producer. Story **5.4b** — inserted at
this story's contexting, 2026-07-29, with Guy — owns the table and the anti-float gate; 5.5 owns the
join; 5.6 the blocker. The build order, quoted as epics.md:1317 groups it: *"the three debt stories
(5.1, 5.2, 5.2b) -> the engine's vocabulary (5.3, 5.4) -> the verdict algebra (5.4b) -> the pure join
(5.5) -> the blocker (5.6) -> wiring it to the corpus (5.7, 5.8) -> persistence (5.9, 5.10) -> the
invariants (5.11, 5.12, 5.13) -> the operator-visible surface (5.14)"* — **5.4b is its own step
there, not a tail of the vocabulary.**

**Why the split, recorded so it is not re-litigated:** story 5.3's own Dev Notes attribute
`fn decide(verdicts) -> Decision` to "5.4". At contexting the work was measured as two deliverables —
a vocabulary of five types, and a **total** function over a table whose input space has a class
D13 does not cover (see *The finding* below). Guy took the split on 2026-07-29, on the 5.2b idiom
(a letter suffix so 5.5–5.14 keep their numbers). Splitting the table out is what lets its own
totality be the subject of a story rather than a subsection of one.

**Nothing under `fixtures/` moves.** No artefact bytes, no `MANIFEST.toml`, no re-hash. This story
does not read the corpus at all. If any step appears to require re-authoring a committed artefact,
**STOP** — that is a finding, reported rather than absorbed. The procedure is in Dev Notes.

### The finding this story records but does not fix

Enumerating D13's table [architecture.md:967-974] over the PRESENCE of each verdict yields seven
input classes, and the table's six rows cover six of them. **The class `≥1 Opposes`, with no
`Decisive`, no `Supports` and no `Disqualifying`, is covered by no row**: it is not *"only `Neutral`
/ nothing"*, it is not *"`Supports` AND `Opposes`"*, and every other row requires a `Decisive` or a
`Supports`. Measured at contexting by enumeration, not by reading.

**Guy's call, 2026-07-29: that class conclues `Abstained { AbsenceOfProof }`** — nothing argues FOR
the merge, so there is no merge to refuse, and D13 deliberately reserves the refusal-that-names-a-rule
for `Disqualifying`. **The arbitration belongs to story 5.4b**, which writes the function that has to
be total. It is recorded here because this story is where it was found, and because a reader of
`Verdict`'s doc must not conclude that the five variants and the six rows are a complete
specification. This story's docs may **name** the gap; they must not pre-implement the answer.

## Acceptance Criteria

1. **AC1 — `Verdict` is D13's enumeration, spelled as the architecture spells it.**
   **Given** D13's *"each yields an enumerated verdict; verdicts combine by an **algebra, not a
   sum**"* and its literal `enum Verdict { Decisive, Supports, Neutral, Opposes, Disqualifying }`
   [architecture.md:960-965]
   **when** the type is defined
   **then** it carries **exactly those five variants, under those five names**, in
   `crates/opencmdb-core/src/identity/cascade.rs` — the file the architecture's source tree names for
   the verdict algebra [architecture.md:3369].

   Binding specifics, so they are not re-litigated at review:
   - **Name `Verdict`, not `RuleVerdict`, not `IdentityVerdict`.** The architecture writes `Verdict`;
     deviating from a locked document's own spelling needs a reason and there is none. It is **not**
     `error[E0252]` against `score::TrapVerdict` (different names — measured: `grep -rn "\bVerdict\b"
     crates xtask --include=*.rs` returns **two prose lines only**, `cascade.rs:6` and `score.rs:238`;
     no ITEM anywhere is named `Verdict`, and `lib.rs`'s flat block re-exports none, so the new name
     cannot collide. A separate grep for `TrapVerdict` finds the existing type). The two names are one
     letter apart in meaning and four in spelling, so **each type's doc names the other** (AC5's
     vocabulary table).
   - **Derives:** `Debug, Clone, Copy, PartialEq, Eq`. **No `PartialOrd`/`Ord`** — D20 is explicit
     that strength, if it ever returns, returns *"as an ORDINAL"* `Opposes(Weak) | Opposes(Strong)`
     under a four-condition ADR [architecture.md:1374-1394]; deriving `Ord` today would let a
     comparison of two verdicts compile, which is the "magnitude" D13 refuses. Record the reason in
     the doc — a reviewer will read the missing derive as an omission otherwise.
   - **No `Serialize`/`Deserialize`** on any type this story defines. Nothing persists a decision: the
     identity link table does not exist (story 5.9 creates it). The precedent is `ScoredRecord`'s own
     doc — *"deriving a wire format for a domain type with no consumer is a finding this project has
     already recorded once"* (`score.rs:310-311`) — and story 5.3 applied it to
     `IdentityAbstentionCause`.
   - **No `#[non_exhaustive]`**, for the reason 5.3 recorded and registered
     (`deferred-work.md:1005-1010`): `opencmdb-bin` is a different crate, so the attribute would force
     a `_` arm on every downstream `match` and destroy the `error[E0004]` that makes a new variant
     break its consumers. **`Verdict` is the type where that matters most** — 5.4b's table matches on
     it, and a sixth verdict must stop the build there.
   - **`pub fn all() -> [Self; 5]`** — the variant witness, in the exact idiom story 5.3 established
     and with the **corrected** doc its code review produced (**Task 2**).

2. **AC2 — The `(rule, verdict, evidence)` triple exists as a type, and evidence is `Vec<ObsId>`.**
   **Given** D13's *"the list of `(rule, verdict, evidence)` IS the explanation"* [architecture.md:977-978]
   and D19's *"a rule that fires without leaving its `rule_id` in the database is a rule we cannot
   debug in production"* [architecture.md:1309-1310] — ⚠️ **quote the architecture, not the register**:
   the wording *"a rule that fires must leave its `rule_id` and its evidence behind"* is
   `deferred-work.md:309-310`'s paraphrase of D19, and story 4.7a is its author
   **when** the element is defined
   **then** `RuleVerdict { rule: RuleId, verdict: Verdict, evidence: Vec<ObsId> }` carries all three,
   with a `///` on the struct **and on each field**.

   - **`evidence: Vec<ObsId>` is the MINIMUM that is not invented, and the doc says so.** `ObsId` is
     the corpus's stable name for an observation, chosen in stories 4.1/4.2 *because* a line number
     *"would silently shift under the truth"*. The architecture **mentions evidence on sixteen lines**
     (measured: `grep -c "evidence" architecture.md` → 16); **five of them concern the identity link's
     evidence** — `:978`, `:1015`, `:1032`, `:1309`, `:3378` — and **none of the five gives it a
     shape** (the last is a source-tree line for a `gap/evidence.rs` that does not exist on disk;
     `gap/` holds only `mod.rs`). A richer payload — the fact values, the candidate pair, a rendered sentence — has **no producer**
     until 5.5 and would be invented here. Registered with owner 5.5.
   - **Nothing enforces non-empty evidence, and that is registered, not fixed.** A `Neutral` verdict
     legitimately has nothing to show, so the rule is not "evidence is never empty" but "a verdict
     that ARGUES leaves evidence" — a validation with no producer to red it. The precedent is
     `ScoredRecord`: all fields `pub`, no constructor, the gap registered
     (`deferred-work.md:224-233`). Same shape here: `pub` fields, no constructor, a register entry
     with owner **5.5**.
   - **`RuleId` is NOT closed into an enum by this story**, although `trap.rs:33-35`'s doc says Epic 5
     closes it. Measured on the committed corpus: `grep -rhoP 'rule\s*=\s*"[^"]+"' fixtures/ | sort -u`
     returns **seven** distinct rule names — `l1-distinct-mac`, `l1-exact-mac`, `l2-different-hostname`,
     `l2-different-switch`, `l2-hostname-agrees`, `l2-uplink-agrees`, `l2-virtual-mac-prefix`. **Five of
     the seven are `l2-*`, which Epic 6 owns.** Closing the enum here would either enumerate five rules
     nobody has designed or make five sha256-locked trap files unparseable. The deferral is registered
     with owner **Epic 6** — and because that measurement **falsifies `trap.rs:33-35`'s own doc**
     (*"Epic 5 names them. It closes into an enum when it does"*), that doc sentence is corrected in
     this story under AC5's "a doc comment must be TRUE" charter. **Doc-only: `:38`'s declaration is
     byte-unchanged.**

3. **AC3 — `Decision` carries the conclusion, the verdict vector and the ruleset version, and the
   version is mandatory by construction.**
   **Given** D14's *"`ruleset_version` is mandatory: without it, improving the engine is **a silent
   data migration — the worst kind**"* [architecture.md:1044-1045]
   **when** a decision exists
   **then** `Decision { conclusion: Conclusion, verdict_vector: Vec<RuleVerdict>, ruleset_version:
   RulesetVersion }` — a struct, so the version is carried **once** and cannot be forgotten on one
   variant of an enum.

   - **`RulesetVersion(pub u32)`** — a newtype, `Debug, Clone, Copy, PartialEq, Eq`. **No `Ord`**: the
     first consumer that ORDERS two versions is persistence (D20: *"existing links are not recomputed
     (they carry the version they were decided under)"*), which is story 5.9. Registered. This is the
     same "no derive without a consumer" argument 5.3 applied, applied consistently rather than
     bent because a version *feels* ordered.
   - **No `const CURRENT_RULESET_VERSION`, and no default value.** There is no ruleset: no rule
     exists. A constant `1` would be a value asserting that the rules it versions exist. The version
     arrives as a **parameter** at construction (5.4b's `decide` signature takes one; this story only
     requires that the field be non-optional and undefaulted). Registered with owner **5.5**, the
     first story with rules to version.
   - **No `#[derive(Default)]` on `Decision`, and no `Default` impl.** That is the mechanism behind
     "mandatory": there is no way to obtain a `Decision` without naming a version. AC6's M1 measures
     it.
   - **`verdict_vector: Vec<RuleVerdict>` — the field name is D18's word.** *"The harness records, for
     every case, the COMPLETE VERDICT VECTOR, not just the outcome… the anti-drift is not discipline,
     it is a data requirement"* [architecture.md:1397-1399].
   - **No value is refused, including `RulesetVersion(0)`.** D14's *"mandatory"* is about PRESENCE, not
     meaning. Meaning attaches the day a ruleset exists to be versioned; validating a number against
     nothing would be the same invention AC2 refuses for evidence. Registered with owner **5.5**.
   - ⚠️ **A `Decision` whose `Conclusion` names a rule ABSENT from its own `verdict_vector` is
     representable, and so is a `Match` with an empty vector — nothing refuses either.** Stated here
     because it is the objection a reviewer leads with, given that this story quotes *"the list of
     `(rule, verdict, evidence)` IS the explanation"* as its own justification: a `Decision { conclusion:
     Match { rule }, verdict_vector: vec![], .. }` compiles and means *"merged, with no explanation"*.
     It is **not fixed here, and the reason is a producer, not a preference**: the conclusion and the
     vector are first built together by 5.4b's `decide`, which is the only place a test could red.
     The shape is `ScoredRecord`'s precedent exactly — `pub` fields, no constructor, the gap
     registered (`deferred-work.md:224-233`). **Owner: story 5.4b**, and AC7 carries the entry.
   - **No type this story defines carries a float or a magnitude**, which is where epics.md's Story 5.4
     AC2 (*"no float crosses a decision boundary; any ranking value is an INTEGER in milli-units"*,
     epics.md:1435-1437) lands at the type level: `RulesetVersion(u32)` is an **identifier, not a
     weight**, and its doc says so; `Verdict` is enumerated and derives no `Ord`; no ranking value is
     invented because Epic 5's L1 is a deterministic lookup with nothing to rank. **The GATE that
     holds this mechanically is story 5.4b's** — measured at contexting: zero `f32`/`f64` in the whole
     workspace today, so the rule is currently true by accident.

4. **AC4 — `Conclusion` is D13's three-way decision, and it mirrors `Outcome`'s shape.**
   **Given** D13's three-way decision — *"`match` / `no-match` / **`ambiguous` -> abstain**"*
   [architecture.md:931-932] and `score::Outcome`'s `Merged { rule } | Refused { rule } | Abstained
   { cause }` (`score.rs:68-76`)
   **when** the type is defined
   **then** `Conclusion { Match { rule: RuleId }, NoMatch { rule: RuleId }, Abstained { cause:
   IdentityAbstentionCause } }` — **a decision names a rule, an abstention names a cause and no
   rule** — and `Decision::rule() -> Option<&RuleId>` returns `Some` on exactly the two decision
   variants, `None` on the abstention, **by construction rather than by a runtime guard**, exactly as
   `Outcome::rule()` does (`score.rs:86-91`).

   - **The mirror is held by a TEST, not by prose** (AC6): for each of the three conclusions and its
     counterpart outcome, `Decision::rule().is_some()` equals `Outcome::rule().is_some()`. That is the
     executable form of the epic's claim *"the same shape `Outcome` mirrors, so `run_trap`'s existing
     assertion needs no runtime guard"*.
   - **`Conclusion::NoMatch` names a rule, always** — which is precisely why D13's `NoMatch` row
     splits two ways onto this type. The `any Disqualifying` half has a rule to name and lands on
     `NoMatch`; the `only Neutral / nothing` half has none and lands on `Abstained { AbsenceOfProof }`
     (story 5.3's variant, traced to architecture.md:974). **This story creates the fork at the type
     level; 5.4b decides which side an input falls on.** The register entries that have been waiting
     for this since story 4.6a (`deferred-work.md:249-256`, `:328-332`) are annotated accordingly in
     AC7 — annotated, **not struck**.
   - **NO `From<Decision> for Outcome`, and no bridge in either direction.** `Outcome` is the trap
     harness's record of an answer; `Decision` is the engine's return. Mapping one onto the other is
     story **5.7**'s (the trap runner consuming a real engine), and it is a decision about the gate,
     not a convenience. The precedent is 5.3's refusal of a `From` between the two abstention
     vocabularies. Registered.
   - **NO `Decision::cause()` accessor.** `Outcome` has none either, and nothing consumes one until
     5.14 groups abstentions by cause. `rule()` exists because `run_trap` justifies it; `cause()` has
     no such justification today. Registered with owner 5.14. *(Adding it would be the same
     no-consumer act AC1 refuses for serde — pre-empted here because a reviewer will ask why one
     accessor exists and not the other.)*
   - **`rule()` lives on `Decision`, NOT on `Conclusion`, and there is no `Conclusion::rule()`.** The
     mirror is therefore structural rather than symmetric — `Outcome::rule()` sits on an enum, this
     one on the struct that wraps one — and that is deliberate: `run_trap`'s consumer holds a
     decision, not a conclusion, so an accessor on the inner enum would have no caller. Same
     no-consumer argument, applied consistently. Registered with owner **5.7**.

5. **AC5 — Four names now mean four different things, and one page says which.**
   **Given** `Verdict` (what ONE rule says), `Conclusion` (what the cascade concluded), `Outcome`
   (what the trap harness records as an answer, `score.rs:69`) and `TrapVerdict` (what the RUNNER
   says about a trap, `score.rs:205`) — all four in `opencmdb-core`, all four re-exported flat at the
   crate root
   **when** a reader meets them
   **then** `cascade.rs`'s module doc carries a table naming all four, one line each, saying whose
   judgement it is and which story owns it. `score.rs`'s `TrapVerdict` doc gains one sentence
   pointing at `Verdict` by full path, and vice versa.

   **And** **SIX** existing doc locations change and are re-read against the final tree — *"a doc
   comment must be TRUE"*, the rule this project has caught itself breaking three times. **FIVE of the
   six are FALSIFIED by this story and are rewritten** (`cascade.rs:1-11`, `identity/mod.rs:8-9`,
   `score.rs:46-47`, `score.rs:276-277`, `trap.rs:33-35`); **the sixth (`score.rs:197-203`) is not
   false** — it gains the reciprocal cross-reference this AC requires. The count is stated once, here,
   and Task 5 lists exactly six bullets against it — *"a count in a doc is a claim"* is this story's
   own inherited lesson #3, and the first place it applies is this sentence:
   - **`score.rs:46-47`** — *"**Not named `Decision`.** The architecture reserves that name for the
     engine's real return type **and never lists its fields**; taking it here would squat a type Epic
     5 has to define."* After this story `Decision` exists and its fields are listed. The weaker true
     sentence: `Outcome` is the harness's record, `Decision` (full path) is the engine's return, and
     nothing converts between them yet. **Doc-only edit; `Outcome`'s declaration does not move.**
   - **`score.rs:276-277`** — `VerdictVectorEntry`'s *"The vector's element is `(rule, verdict,
     evidence)` and **none of the three exists**: rules arrive in Epic 5."* After this story the
     element type exists (`RuleVerdict`) and has no PRODUCER. ⚠️ **The rest of that doc block stays
     true and must survive** (`:277-284`): `VerdictVectorEntry` remains **uninhabited** (the word is at
     `:277`) and `ScoredRecord::verdict_vector` remains **provably empty**. See Task 4 — replacing it
     is out of scope and would falsify three other places.
   - **`trap.rs:33-35`** — `RuleId`'s *"Epic 5 names them. It closes into an enum when it does."*
     AC2's measurement refutes it: five of the corpus's seven rule names are `l2-*`, which Epic 6
     owns. **Doc-only; `:38`'s declaration is byte-unchanged.**
   - **`identity/mod.rs:8-9`** and **`cascade.rs:1-11`** both say story 5.4 writes *"the algebra"*.
     After this story they must say what is true: this file holds the vocabulary **and the return
     type**; the algebra that combines verdicts is **5.4b**'s. One sentence naming the immediate next
     owner — *"a forward inventory is exactly the inventory-with-no-guard 5.2's review caught"*
     (5.3, Task 2).

6. **AC6 — Every claim about BEHAVIOUR or SHAPE is pinned by a test, and every new guard is proven to
   red.**
   ⚠️ **"Every claim" is scoped deliberately, because the unscoped version is unsatisfiable.** Six of
   this story's binding claims are carried by an **ABSENT derive or an absent item** — no serde on
   five types, no `Ord` on `Verdict`, no `Ord` on `RulesetVersion`, no `#[non_exhaustive]`, no
   `From<Decision> for Outcome`, no `Decision::cause()`. **None of them is testable in Rust without a
   compile-fail harness this project does not have**, and writing a vacuous `let _ = …` to look
   compliant would be worse than saying so. Each is instead carried by a doc sentence naming its owner
   **and** by an AC7 register entry, and the Completion Notes state that split explicitly rather than
   letting a blanket stand. This is the over-claim class 5.3's review recorded one level down
   (*"two of four tests are carried by the compiler, not by their assertions"*, `deferred-work.md:1045-1053`).

   Prove-to-red is the house rule (story 1.3). The four mutations of **Task 7** are **run**, and each
   records **every** file, line and failing test — or the exact compiler error code and site — that
   went red, then names which of those reds is the one the NEW guard contributes. A compile error
   counts as a red, but it must be **run and quoted, never predicted**.

   **A mutation with several reds is expected and is not a defect.** A record naming one red where
   four fired is the under-reporting this project's reviews keep catching, and it is the same defect
   as an over-claim (5.2b's lesson, inherited by 5.3).

   **And** the test-placement convention this file has been owed since 5.3's review
   (`deferred-work.md:1054-1059`, *"Owner: whoever next adds a test to `cascade.rs` — decide the
   convention once, in one place"*) is **decided and written down once**, in `cascade.rs`'s test
   module doc: *a test lives with the item whose CLAIM it pins; the items it merely READS are
   dependencies, imported and not owned.* The convention is stated with **the two cases it decides**,
   because a convention that does not resolve the case that opened the register entry closes nothing:
   - The **mirror test** (AC4) pins a claim about **`Decision`'s shape** — that it mirrors `Outcome`'s.
     `Outcome` is the dependency it reads. It belongs in `cascade.rs`.
   - **`an_abstention_names_no_rule_whatever_its_cause`** (5.3, `cascade.rs`) pins a claim about **the
     abstention VOCABULARY's relationship to a rule** — *"an abstention has no rule to name, for EVERY
     cause"*. The subject is `IdentityAbstentionCause`; `Outcome::rule()` is the mechanism it reads to
     express it. It therefore stays where it is, **by the convention rather than by fiat**. If the dev
     disagrees with that reading after writing it out, **say so and leave the register entry open** —
     an entry closed by assertion is worse than an entry still open.

7. **AC7 — The register is closed by appending, never rewriting.**
   **Given** `deferred-work.md`'s entries that name this story or its types
   **when** the story ends
   **then**:
   - `## Deferred from: code review of story-5.3`'s **AC7-drafting** entry (`:1073-1076`, *"Owner:
     whoever writes 5.4's AC7 — require the PR and stop"*) is marked **`✅ CLOSED by story 5.4`** in
     the file's own `✅ **CLOSED by story X.** ~~struck~~` idiom. **This story's own AC8 is that
     closure**: it requires the branch, the PR and green CI, and **stops there**.
   - The two `NoMatch` entries — `:249-256` and `:328-332` — keep their existing `↺ PARTLY closed`
     annotations and gain **one appended line each**: the type-level fork now exists
     (`Conclusion::NoMatch { rule }` vs `Abstained { cause }`); the MAPPING still has no producer.
     They are **not struck** — striking would claim a behaviour that exists nowhere, the over-claim
     this project's reviews keep catching.
     ⚠️ **The owner move is mechanical, and Task 8's "never rewrite a bullet" binds it.** The string
     `Owner stays **stories 5.4/5.5**` does **not** live in those two entries — it lives in the 5.3
     ANNOTATION bullets above them (`:240-248` and `:319-327`). So: **strike it in place**
     (`~~Owner stays **stories 5.4/5.5**~~`) and append `↺ **Owner UPDATED 2026-07-29 by story
     5.4: 5.4b/5.5** — 5.4 built the fork, 5.4b decides which side an input falls on.` Nothing else in
     either bullet is rewritten.
   - `## Deferred from: story-4.7a`'s **firing-rule/evidence contract** (`:309-318`, Owner: Epic 5) is
     annotated `↺ PARTLY closed by story 5.4`: the type that CARRIES `(rule, verdict, evidence)` now
     exists (`RuleVerdict`), and nothing produces one. **Owner moves to story 5.5** — *"a test must red
     if it does not"* needs a firing rule. Not struck.
     ⚠️ **Placement, measured — the file has BOTH idioms and only one thing is settled.** `:241-248`
     and `:319-327` sit **above** their target and say *"the entry BELOW"*; `:490` sits **below** its
     target and says *"the entry above"*. What 5.3's review actually settled is that an annotation
     **must NAME its target** (`:540-542` of that story's findings), not which side it sits on. Place
     this one immediately **above** `:309`, right after the `## Deferred from: story-4.7a` header,
     naming *"the entry BELOW (the firing-rule contract)"* — the idiom of its nearest neighbours.
     **Then re-read `:319-327`**: it is 5.3's own annotation and its text refers to the entries above
     and below it. If the insertion makes either reference stop resolving, amend that phrase **in the
     same edit** and say so in the Completion Notes. A commit that falsifies a sentence it did not
     touch is inherited lesson #1.
   - A new `## Deferred from: story-5.4` section is **appended at the END of the file**, after
     `## Deferred from: code review of story-5.3` (`:1017`) — the file is **chronological, not
     topical**. It carries **fifteen** items. **Thirteen name a STORY as owner; TWO are explicitly
     ownerless and say so** — and any prose about this section, here or in `sprint-status.yaml`,
     states that split rather than claiming fifteen named owners. *(5.3 shipped exactly this
     over-claim and its review wrote it up.)*
     **Owned by a story:** the evidence shape (5.5) · non-empty evidence unenforced (5.5) · a
     `Conclusion` naming a rule absent from its own `verdict_vector`, and `Match` with an empty vector
     (**5.4b**) · no serde on five types (5.9) · no `Ord` on `RulesetVersion` (5.9) · `RulesetVersion(0)`
     unrefused (5.5) · no `const` ruleset version (5.5) · `RuleId` not closed into an enum, with the
     seven measured corpus strings (Epic 6) · no `From<Decision> for Outcome` (5.7) · no
     `Decision::cause()` and no `Conclusion::rule()` (5.14 / 5.7) · `VerdictVectorEntry` vs
     `RuleVerdict`, two types for one triple (5.7) · the D13 table gap of *The finding* above (5.4b) ·
     **`Verdict::all()` inherits the measured lazy-repair residue of `IdentityAbstentionCause::all()`**
     (`:1025-1044`, owner **5.14**) — folded into that existing entry rather than duplicated.
     **Explicitly ownerless, each naming the CONDITION that would produce an owner:** no `Ord` on
     `Verdict` — `Owner: whoever writes D20's ADR` · the flat re-export idiom, now **five names
     heavier**, aggravating `:1067-1072` — `Owner: whoever revisits the crate's re-export policy`
     (that entry's own wording). The register's own idiom allows a condition in place of a name; what
     it does not allow is calling one a name.
   - **Two `Owner: Epic 5` entries are deliberately NOT touched**, and saying so turns silence into a
     scope statement: `:285-290` (lattice monotonicity — needs an engine producing verdicts across
     capability subsets, story 5.13) and `:336-344` (`RuleId` whitespace/case normalization in
     `run_trap` — bites when a PRODUCER emits a `RuleId`, story 5.5). A one-line note in the
     Completion Notes is enough; do not edit those entries.

8. **AC8 — The gate is green, the docs are current, and the flow stops at the PR.**
   Tasks 8–10 are acceptance criteria, not housekeeping, so an AC-by-AC auditor can see them:
   `cargo fmt --all` clean · `cargo clippy --workspace --locked --all-targets -- -D warnings` clean ·
   **`cargo clippy --workspace --locked -- -D warnings` clean** (the CI form, without
   `--all-targets`) · `cargo test --workspace --locked` green with the three per-crate counts
   **re-measured on the final tree** · `cargo xtask ci` green (`ℹ views-hash STALE`, exit 0, is
   correct and **must not be regenerated in a story**) · `git status` under `fixtures/` empty ·
   **the docs, with their scope stated rather than left conditional**: `sprint-status.yaml` and
   `docs/project-context.md` updated, `CLAUDE.md`'s Epic 5 sentence updated — **not conditional**:
   both `docs/project-context.md:45` and `CLAUDE.md:7` were corrected at this story's contexting when
   5.4b's insertion moved Epic 5 from 15 to 16 stories, so what the dev owes is the story's OWN
   effect on them, not that count · `epics.md` **verified only** — 5.4b is already present, the counts
   already read sixteen, and an edit there is a finding rather than a task ·
   **branch → PR → green CI. The story ends at status `review` and the PR open.** The
   merge is a separate act and it is what makes a story `done` in this project (5.1, 5.2, 5.2b, 5.3);
   the `code-review` workflow's own default of setting `done` at the end of the review is WRONG here.

## Tasks / Subtasks

- [x] **Task 1 — Read before writing** (AC1–AC5)
  - [x] `crates/opencmdb-core/src/identity/cascade.rs` **in full** (162 lines) — the module doc, the
        `IdentityAbstentionCause` doc block, `all()`'s **corrected** doc and its inline comment, and
        the two tests. The `all()` idiom you will copy for `Verdict` is there, and so is the honest
        statement of its limit; copy **both**, not just the code.
  - [x] `crates/opencmdb-core/src/identity/mod.rs` (21 lines) — every sentence in it is about to
        change tense.
  - [x] `crates/opencmdb-core/src/score.rs:38-92` (`Outcome`, its doc, `rule()`), `:197-250`
        (`TrapVerdict` and `run_trap` — the shape AC4 mirrors and the assertion it protects),
        `:252-286` (`SourceState` and `VerdictVectorEntry`, the uninhabited-placeholder precedent),
        `:288-338` (`ScoredRecord` — the `pub`-fields-no-constructor precedent AC2 cites).
  - [x] `crates/opencmdb-core/src/trap.rs:31-38` (`RuleId` and its "closes into an enum" doc at
        `:33-35`) and `:55-109` (`Expectation`, `column()`, `rule()`, to the close of the `impl`).
        Also `:413-415` — the private test helper `fn obs(n: u128) -> ObsId`, which is the crate's
        only idiom for minting an `ObsId` in a test (Task 6 needs it).
  - [x] `crates/opencmdb-core/src/observation/mod.rs:22-52` — the `uuid_newtype!` macro and `ObsId`.
        Note the derives at `:27`: `Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord,
        Serialize, Deserialize`. `Vec<ObsId>` is therefore `PartialEq` without any work.
  - [x] `architecture.md` **D13** (`:929-1011`) and **D14** (`:1013-1049`) — start from the Decision
        Index near the top of the file (F56), not from a grep. Read **D20** (`:1348-1399`) too: it is
        where the missing `Ord` on `Verdict` and the milli-units clause come from.
  - [x] `deferred-work.md:249-256`, `:309-318`, `:328-332`, `:1025-1044`, `:1054-1059`, `:1067-1076`.

- [x] **Task 2 — `Verdict`** (AC1)
  - [x] Five variants, D13's spelling, a `///` on the enum **and on each variant**. Each variant doc
        says what a rule MEANS by it — and `Disqualifying`'s cites its two committed instances from
        D13 itself: the IANA VRRP/HSRP prefixes and the U/L bit — *"**Both are** `Disqualifying` as
        grouping anchors, known at ingestion."* [architecture.md:1002]. That is the one variant whose
        meaning is already fixed by a decision rather than by a future rule.
  - [x] The enum doc states the **algebra clause** and attributes it: *"verdicts combine by an
        algebra, never a sum"* is D13's contract, **and story 5.4b implements it** — never phrased as
        a description of this file's behaviour. The doc may **name** the uncovered input class of
        *The finding*; it must not state what that class concludes. That answer is 5.4b's AC.
  - [x] `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` plus a one-line comment naming what is absent
        and why, with owners: serde (5.9), `Ord` (D20's ADR — *"if strength returns, it returns as an
        ORDINAL"*), `Display` (5.14), `#[non_exhaustive]` (never — it would destroy the `error[E0004]`
        5.4b's table depends on).
  - [x] **`pub fn all() -> [Self; 5]` — the exact idiom from `cascade.rs:104-119`, with the corrected
        doc.** Ship BOTH guards (the literal + the exhaustive witness match) and the `[Self; 5]`
        return type. ⚠️ **Do not copy the pre-review wording**: the two errors are **alternatives
        along one repair path, never simultaneous** (measured under 5.3's M1b/M1c), and repairing only
        the `error[E0004]` with a bare arm leaves `all()` returning the old list **while the suite
        stays green** (measured: 90 passed). The doc says the guarantee AND its limit, and the inline
        comment names the wrong repair. Copy `cascade.rs:84-119` as the model — the doc begins at
        `:84`, not at the `# What the witness below guarantees` heading — adapting the counts.

- [x] **Task 3 — `RuleVerdict`** (AC2)
  - [x] `pub struct RuleVerdict { pub rule: RuleId, pub verdict: Verdict, pub evidence: Vec<ObsId> }`,
        `#[derive(Debug, Clone, PartialEq, Eq)]` — **no `Copy`** (`RuleId` is a `String` newtype,
        `Vec` is not `Copy`).
  - [x] `///` on the struct and **on each of the three fields**. The `evidence` field's doc carries
        AC2's two sentences: what it is (the observations the rule read), and that a richer payload is
        deferred to 5.5 with no producer today.
  - [x] The struct doc names the relationship to `` [`crate::score::VerdictVectorEntry`] `` **by full
        path**: that placeholder is uninhabited and stays so; this is the engine-side element with no
        producer; **story 5.7 owns the unification**, when the harness first records a real run.
        ⚠️ **An import added only to shorten a doc link is an unused import** under the CI form of
        clippy — 5.3 measured this. Full paths, no `use`.
  - [x] `use crate::trap::RuleId;` and `use crate::observation::ObsId;` — both are real code uses, so
        both are live imports. **`trap.rs` is not edited.**

- [x] **Task 4 — `RulesetVersion`, `Conclusion`, `Decision`** (AC3, AC4)
  - [x] `pub struct RulesetVersion(pub u32);` — `Debug, Clone, Copy, PartialEq, Eq`, no `Ord`, no
        serde, no constant, no `Default`. Its doc quotes D14's *"a silent data migration — the worst
        kind"* and D20's *"any reintroduction increments `ruleset_version`; existing links are not
        recomputed"* [architecture.md:1392-1394], and says the value has no producer until 5.5.
  - [x] `pub enum Conclusion { Match { rule: RuleId }, NoMatch { rule: RuleId }, Abstained { cause:
        IdentityAbstentionCause } }` — `Debug, Clone, PartialEq, Eq`. Variant docs cite D13's
        three-way decision; `NoMatch`'s doc carries AC4's fork sentence and attributes the choice of
        side to **5.4b**.
  - [x] `pub struct Decision { pub conclusion: Conclusion, pub verdict_vector: Vec<RuleVerdict>, pub
        ruleset_version: RulesetVersion }` — `Debug, Clone, PartialEq, Eq`. **No `Default`, no
        constructor** (the `ScoredRecord` precedent: `pub` fields, and the validation gap registered
        rather than invented).
  - [x] `pub fn rule(&self) -> Option<&RuleId>` on `Decision`, matching exhaustively with **no `_`
        arm**, doc'd as the mirror of `` [`crate::score::Outcome::rule`] ``.
  - [x] ⚠️ **`score::VerdictVectorEntry` is NOT replaced, and `ScoredRecord` is NOT touched.**
        Replacing the uninhabited placeholder with `RuleVerdict` would falsify four things at once,
        with **no producer** to justify any of them: `VerdictVectorEntry`'s *"uninhabited"* doc
        (the word is at `score.rs:277`; the declaration at `:285-286`),
        `ScoredRecord::verdict_vector`'s *"Always empty… and provably so"* (`:335-337`),
        `comparable_fields`' *"no producer until an engine; empty on both sides"* (`:453`), and the
        register entry at `deferred-work.md:210-215`. **Owner: story 5.7.** If the work looks
        tempting, that is the story to put it in.
  - [x] `lib.rs`: extend the existing `pub use identity::cascade::…;` line (`:40`) to
        `pub use identity::cascade::{Conclusion, Decision, IdentityAbstentionCause, RuleVerdict,
        RulesetVersion, Verdict};` — alphabetical inside the braces, following the `gap` and `score`
        re-export idiom (`gap` at `:39`, `score` at `:46-49`). **This aggravates a registered
        asymmetry** (the flat
        re-export has no consumer, `deferred-work.md:1067-1072`); it is the crate's idiom and
        deviating for one module would be the inconsistency. Register the new cost (five names), do
        not deviate.

- [x] **Task 5 — The six doc locations** (AC5) — **all six are doc-only**; no declaration moves. Five
      are false after this story and are rewritten; the sixth gains a cross-reference.
  - [x] `cascade.rs`'s module doc — rewrite. It currently says *"Story 5.4 writes that algebra and the
        `Verdict` enum it combines; this file holds neither."* After this story it holds `Verdict` and
        not the algebra. Add AC5's four-name table (`Verdict` / `Conclusion` / `Outcome` /
        `TrapVerdict`), one line each, full paths for the two that live in `score`.
  - [x] `identity/mod.rs`'s module doc — rewrite `:8-9`. One sentence naming 5.4b as the algebra's
        owner; **not** a four-story inventory.
  - [x] `score.rs:46-47` — rewrite the *"Not named `Decision`"* paragraph. `Decision` exists now.
  - [x] `score.rs:276-277` — rewrite the one sentence about the element not existing. **Everything
        else in that doc block stays** (`:277-284`), including *"uninhabited"* and *"provably empty"*,
        which remain true.
  - [x] `trap.rs:33-35` — **one sentence, doc-only.** It predicts that Epic 5 closes `RuleId` into an
        enum; AC2's measurement refutes it (five of the corpus's seven rule names are `l2-*`, Epic 6's).
        The weaker true sentence: it closes when every rule the corpus names is designed, and five of
        them belong to Epic 6. **`:38`'s declaration is byte-unchanged**, and the two pins at
        `:569-579` / `:584-591` are not touched.
  - [x] `score.rs:197-203` (`TrapVerdict`'s doc) — one sentence pointing at
        `` [`crate::identity::cascade::Verdict`] `` and saying whose judgement each is. Reciprocal in
        `Verdict`'s own doc.
  - [x] ⚠️ **Re-read every rewritten sentence against the FINAL tree after the last edit.** 5.1 cited
        a `grep` its own diff broke; 5.2 replaced a false sentence with another one the same commit
        falsified. This is the inherited lesson, and this story touches **six** doc locations.

- [x] **Task 6 — The tests** (AC1–AC4, AC6) — inline, in `cascade.rs`'s trailing `#[cfg(test)] mod
      tests` (D56b, one per file). All of them live in `cascade.rs`: every claim below is a claim
      about a type this file defines.
  - [x] **Write the convention down first** (AC6), in the test module's own doc comment: *a test lives
        with the item whose CLAIM it pins, importing other modules as dependencies.* One paragraph,
        naming the mirror test as the case it decides and `an_abstention_names_no_rule_whatever_its_cause`
        as the case it retroactively covers. This closes `deferred-work.md:1054-1059`.
  - [x] **The five verdicts of D13.** `Verdict::all()` contains exactly the five, asserted as a SET.
        Do **not** assert the length — on `[Self; 5]` that is a tautology the return type guarantees;
        say so in a comment so a later reader does not "fix" the omission. ⚠️ And state, in the test's
        doc, what it does NOT catch: a sixth variant leaves the five in `all()` and this test green —
        what stops that is the witness's `error[E0004]`, whose doc says exactly how far it goes.
  - [x] **A decision names a rule; an abstention does not.** `Decision::rule()` is `Some` for `Match`
        and `NoMatch`, `None` for `Abstained` — over all of `IdentityAbstentionCause::all()`, not one
        hand-picked cause.
  - [x] **The mirror (AC4).** For the three conclusion/outcome counterparts —
        `(Match, Merged)`, `(NoMatch, Refused)`, `(Abstained, Abstained)` —
        `Decision::rule().is_some() == Outcome::rule().is_some()`. The assertion message names the
        pair. This is the executable form of *"the same shape `Outcome` mirrors, so `run_trap`'s
        existing assertion needs no runtime guard"*.
  - [x] **The verdict vector is carried verbatim.** A `Decision` built with three `RuleVerdict`s —
        distinct rules, three different `Verdict` variants, distinct non-empty `evidence` — returns
        them in order, each field intact. This is the test that reds when a field is dropped (M4).
        ⚠️ **Minting an `ObsId`: use the crate's own idiom, `ObsId::from_uuid(Uuid::from_u128(n))`
        with distinct `n`** — the shape `trap.rs:413-415` already uses. The test module therefore
        gains `use uuid::Uuid;`. **`Uuid::new_v4()` does NOT compile here**: `opencmdb-core` builds
        `uuid` with `features = ["v7", "serde"]`, so reaching for it yields `error[E0599]`. And
        `Uuid::nil()` three times would make the three evidence lists identical — the test would pass
        while proving nothing, which is the failure this project bans outright.
  - [x] Assertion messages name the variant AND the claim. *"expected Some"* is not actionable in a CI
        log with four verdict-ish types in scope.

- [x] **Task 7 — Prove to red** (AC6). Run each, quote the observed failure, restore, re-run green.
      **These are predictions to check against, not a licence to skip running**: if the observed set
      differs, the DIFFERENCE is the finding and it goes in the Completion Notes.
  - [x] **M1 — delete `ruleset_version` from `Decision`.** Predicted: `error[E0560]`/`E0063` at every
        construction site in the test module. Proves AC3's "mandatory by construction". Quote the
        first error and the site count.
  - [x] **M2 — make `Decision::rule()` return `None` for `NoMatch`.** A **behavioural** red, not a
        compile error: the rule test and the mirror test both fail. Name which is new. This is the one
        mutation that does not lean on the type system, and it is the one that proves the mirror is an
        assertion rather than a shape.
  - [x] **M3 — delete `Verdict::Disqualifying`.** Predicted: **`error[E0599]`** at `all()`'s literal,
        at its witness match arm, and at every test that spells the variant. ⚠️ **NOT `error[E0308]`** —
        this was measured at contexting by compiling the exact shape: deleting a variant aborts on the
        unknown name long before the array length is checked, so `E0308` never fires. `E0308` belongs
        to the OTHER repair path (shortening the literal while `[Self; 5]` stands), which is exactly
        what AC1 and Task 2 say: the two errors are **alternatives along one repair path, never
        simultaneous**. Quote the first error and the site count.
  - [x] **M4 — delete `evidence` from `RuleVerdict`.** Predicted: compile errors at the vector test's
        construction sites. Proves the triple is a triple and not a pair.
  - [x] ⚠️ **Two of these four reds are carried by the COMPILER, not by an assertion, and the record
        must say so.** M3's red is a spelling error (`E0599`) and M4's is a construction-site error —
        both would fire identically if the test body were `assert_eq!(1, 1)`. That is the class
        `deferred-work.md:1045-1053` records. **M1 and M2 are the two that are not**: M1 proves the
        field is unforgettable, and M2 is the only mutation that reds an ASSERTION rather than a name.
        The Completion Notes name which tests remain compiler-carried and **why no behavioural
        mutation exists for them** — a `pub` field has no code to break — and that register entry's
        owner stays **5.5**, the first story with a producer.
  - [x] ⚠️ **Restore after every mutation and verify with `git status` before the next one.** Local
        flakiness (issue #38) and a forgotten revert look identical.

- [x] **Task 8 — The register** (AC7) — append-and-strike, never rewrite a bullet.
  - [x] Close `:1073-1076` (AC7 drafting) with `✅ **CLOSED by story 5.4**`, naming what this story's
        AC8 says instead: branch → PR → green CI, stop.
  - [x] Append one line to each `NoMatch` entry (`:249-256`, `:328-332`). The owner move is made in
        the 5.3 ANNOTATION bullets that carry the string (`:240-248`, `:319-327`): strike
        `Owner stays **stories 5.4/5.5**` in place and append the updated owner. Nothing is rewritten.
  - [x] Annotate `:309-318` (firing-rule/evidence) `↺ PARTLY closed by story 5.4`, owner → **5.5**,
        placed **above** its target (the idiom of `:241` and `:319`) and **naming it explicitly**.
        Then re-read `:319-327` and confirm its own above/below references still resolve. Not struck.
  - [x] Open `## Deferred from: story-5.4` at the **END** of the file with AC7's **fifteen** items —
        thirteen owned by a story, two explicitly ownerless and saying so. The file is chronological.
  - [x] Note in the Completion Notes (not in the file) the two `Owner: Epic 5` entries left untouched
        on purpose — `:285-290`, `:336-344`.

- [x] **Task 9 — The full local gate, run WHOLE** (AC8; mirrors CI — Epic 3's retrospective recorded
      four CI-only failures from skipping exactly this)
  - [x] `cargo fmt --all` · `cargo clippy --workspace --locked --all-targets -- -D warnings` ·
        `cargo test --workspace --locked` · `cargo xtask ci`. **`--locked` everywhere.**
  - [x] ⚠️ **Also run clippy the way CI runs it — `cargo clippy --workspace --locked -- -D warnings`,
        WITHOUT `--all-targets`.** That compiles the lib without `cfg(test)`, and it is the ONLY
        invocation that catches an import kept alive solely by a test module or a doc link. Both must
        be green.
  - [x] Report the test count as three numbers (bin + core + xtask). **Baseline measured 2026-07-29 at
        `505379e`: 135 + 90 + 42 = 267, zero failures.** Only `core` should move.
  - [x] `git status` under `fixtures/` **empty**; `MANIFEST.toml` untouched.
  - [x] `cargo xtask ci` reporting `ℹ views-hash STALE` and exiting 0 is **expected and correct**
        (GitHub issue #50). **Do NOT regenerate `architecture-views.md`** — milestone task, never a
        story task.
  - [x] File-size headroom, so the gate's message is not a surprise: `cascade.rs` is **162 lines total
        with the first `#[cfg(test)]` at `:122` → 121 code lines**, ceiling 2000. `score.rs` is 1329
        total, first `#[cfg(test)]` at `:587` → **586 code lines**. Both far under.

- [x] **Task 10 — Docs current before push** (AC8; project rule)
  - [x] `sprint-status.yaml` — the `5-4-…` entry and the narrative comment block.
  - [x] `docs/project-context.md` — the test count and the Epic 5 line.
  - [x] `CLAUDE.md` — the Epic 5 sentence. **Not conditional**: this story changes what "the engine
        proper" has shipped, so the sentence naming 5.3 as the last engine story becomes stale on
        merge.
  - [x] `_bmad-output/planning-artifacts/epics.md` — **verify only, do not edit.** Story 5.4b is
        present, `:22` and `:1313` both read sixteen, and `:1317`'s build order names 5.4b as its own
        step — all corrected at this story's contexting (2026-07-29). **An edit here is a finding, not
        a task**; a missing insertion is likewise a finding, not something to improvise.
  - [x] No manual, README or gh-pages change is expected: this story ships nothing a user can see. If
        that turns out to be wrong, the doc moves in the same push.

- [x] **Task 11 — Branch → PR → green CI** (AC8). Never straight to `master` (`enforce_admins` is
      false; honouring it is on the author). Branch `story-5.4-decision-and-ruleset-version`. **The
      story ends at status `review` with the PR open and CI green.** The merge is a separate act.

## Dev Notes

### What was measured, before the story was written

All on the committed tree at `505379e` (2026-07-29), so the dev re-derives nothing and a surprise
reads as a **finding**:

- **`cargo test --workspace --locked` → 135 (bin) + 90 (core) + 42 (xtask) = 267, zero failures**
  (plus one ignored doc-test target reporting 0). That is the baseline this story moves, and only the
  `core` number should change.
- **Zero `f32` and zero `f64` in the entire Rust workspace.** `grep -rn "\bf32\b\|\bf64\b" crates
  xtask --include=*.rs | wc -l` → **0**. D13's *"if the output is a float, B has won in disguise"* is
  therefore currently true by accident rather than by a gate — **story 5.4b adds the gate**, and this
  story must not introduce the first float.
- **The corpus writes SEVEN distinct rule names**, five of them `l2-*` (AC2). This is the measurement
  that decides `RuleId` stays a `String` newtype here.
- **`Conclusion`, `RulesetVersion`, `RuleVerdict` and `Evidence` occur ZERO times** in `crates/` and
  `xtask/`. **`Decision` occurs TWICE, both in prose** — `score.rs:46` (the *"Not named `Decision`"*
  doc AC5 rewrites) and `fixtures.rs:1662` (*"The case Decision 2 is ARGUED from"*) — **never as an
  item**. `Verdict`, matched on a word boundary, returns two prose lines only (`cascade.rs:6`,
  `score.rs:238`). So no name this story takes is contested, and none is `error[E0252]` against
  `lib.rs`'s flat re-export block.
- **`ObsId` derives `Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize,
  Deserialize`** (`observation/mod.rs:27`, via `uuid_newtype!`). `Vec<ObsId>` is `PartialEq` for free;
  nothing needs to be added to make `RuleVerdict` comparable.
- **`identity/cascade.rs` is 162 lines, first `#[cfg(test)]` at `:122`** → 121 code lines. The five
  types, their docs and their tests fit with an order of magnitude to spare.
- **`Outcome::Abstained` has NINE construction sites, every one in a test module** — `cascade.rs:156`,
  `score.rs:644`/`:741`/`:762`/`:805`, `trap_gate.rs:450`/`:508`/`:652`/`:658`, all past their file's
  first `#[cfg(test)]`. ⚠️ **The figure SIX is 5.3's PRE-merge measurement**; three sites were added by
  5.3's own test loops and 5.3's completion record already corrected it
  (`sprint-status.yaml:452-453`). Re-introducing a corrected number is the same defect as inventing
  one, and this bullet exists because this story's first draft did exactly that. Production code
  constructs no outcome and no decision; this story adds no producer either.

### The one thing that would make this story fail review

**Writing `decide()`.** The table is in front of you at architecture.md:967-974, the types you just
wrote are exactly its input and output, and it looks like twenty minutes. It is story **5.4b**'s, and
writing it here would:

1. put a function whose input space has a **class D13 does not cover** into a story whose ACs never
   arbitrate it — the arbitration was taken separately, with Guy, precisely so it would be visible;
2. give the table's totality no story of its own, so *"the six rows are implemented"* would be
   reviewed as a subsection instead of as the subject;
3. and repeat, one story later, the exact mistake story 5.3's Dev Notes warned about when they
   refused to write `decide()` for the same reason.

**The deliverable is five types, their documentation, and the tests that pin their claims.** That is
the whole story.

### Why `Decision` is a struct and `Conclusion` an enum

The natural first draft is `enum Decision { Match { rule, evidence, ruleset_version }, … }`, and it
is wrong for one measurable reason: `ruleset_version` would be repeated on all three variants, so
"mandatory" would rest on three declarations agreeing rather than on one. D14's whole point is that
the version cannot be forgotten. A struct carries it once, and a variant added to `Conclusion` later
cannot forget it.

The same argument does **not** apply to `Outcome` (`score.rs:69`), which carries no common field —
which is why the two types are shaped differently while `Conclusion` mirrors `Outcome`'s three-way
algebra exactly. AC4's mirror test asserts the part that must agree (rule-or-cause) and is silent
about the part that must not (the envelope).

### The four names, and why none of them can be dropped

| type | whose judgement | how many | owner |
|---|---|---|---|
| `identity::cascade::Verdict` | ONE rule about ONE candidate pair | 5 | this story |
| `identity::cascade::Conclusion` | the CASCADE, over the verdict set | 3 | this story |
| `score::Outcome` | what the trap harness RECORDS as an answer | 3 | story 4.6a |
| `score::TrapVerdict` | the RUNNER, about one trap | 3 | story 4.7a |

`Outcome` and `Conclusion` look redundant and are not: `Outcome` is what a harness writes down about
*any* answer, including a hand-authored one in a test; `Conclusion` is what the engine concluded, and
it travels with its verdict vector and its ruleset version. The day they meet is story **5.7**, and
the mapping is that story's decision, not a `From` impl written here.

### The evidence question, stated plainly because a reviewer will ask

The architecture names the identity link's evidence on five lines and shapes it on none of them (AC2
carries the measurement and the full sixteen-line grep count). `Vec<ObsId>` is the smallest
thing that is **not invented**: it names the observations a rule read, using the identity the corpus
already froze. It is enough to make a firing rule debuggable — *"a rule that fires without leaving its
`rule_id` is undebuggable in production"* — and it is not enough to render an operator-facing
explanation. That richer shape has no producer until 5.5 and no consumer until 5.14; inventing it now
would be the inversion of the order Epic 4 was built to enforce.

**Do not add a `reason: String`.** An explanation assembled from `(rule, evidence)` is derived data;
a stored sentence is a second source of truth for it, and D47's *"an error there is domain data, not a
string"* is the same argument one layer up.

### What this touches, and what it must not break

- **`crates/opencmdb-core/src/identity/cascade.rs`** (UPDATE) — five new types, **four** new tests
  (Task 6 lists exactly four), the module doc rewritten. *Must be preserved:* `IdentityAbstentionCause`'s two variants and their docs,
  `all()`'s corrected doc and its inline comment, and both existing tests **unchanged**.
- **`crates/opencmdb-core/src/identity/mod.rs`** (UPDATE, doc only) — `:8-9`.
- **`crates/opencmdb-core/src/lib.rs`** (UPDATE) — one re-export line widened.
- **`crates/opencmdb-core/src/score.rs`** (UPDATE, **doc only** — three blocks: `:46-47` and
  `:276-277` rewritten, `:197-203` gains a cross-reference). *Must be preserved:* the exhaustive 3×3 with no `_` arm; `run_trap`'s positive gate
  (`!= Score::Pass`); `Outcome::rule()` returning `None` by construction; `comparable_fields`'
  `..`-free destructure; **`VerdictVectorEntry` uninhabited** and `ScoredRecord::verdict_vector`
  provably empty.
- **`crates/opencmdb-core/src/trap.rs`** (UPDATE, **doc only** — `:33-35`). `RuleId` stays a `String`
  newtype; only the sentence predicting that Epic 5 closes it into an enum is corrected, because AC2's
  own measurement refutes it. `:38`'s declaration and the two pins at `:569-579` / `:584-591` are
  byte-unchanged.
- **`crates/opencmdb-core/src/lib.rs:8-11`'s module doc** — re-read and **left standing**, recorded so
  the omission reads as a judgement rather than an oversight: five types with no producer and no
  algebra still *"assert nothing about identity yet"*. If the dev disagrees after writing the types,
  the correction belongs in this story, not in the next one.
- **`crates/opencmdb-core/src/gap/mod.rs`**, **`crates/opencmdb-bin/`** (all of it, including
  `page.rs`, `trap_gate.rs`, `fixtures.rs`, `locales/app.yml`), **`xtask/`** — **NOT touched.** No
  gate changes here; the anti-float gate is 5.4b's.
- **Under `fixtures/`: NOTHING.**
- **`deferred-work.md`** (UPDATE, append-only) · **`sprint-status.yaml`**, **`docs/project-context.md`**,
  **`epics.md`**, **`CLAUDE.md`** (UPDATE).

### What STOP means, procedurally

If a step appears to require re-authoring a committed artefact or re-hashing `MANIFEST.toml`: **stop,
do not edit the artefact.** Record what was attempted, what the tree says, and the exact command that
shows the conflict; report it as a finding in the Completion Notes, **open a GitHub issue on
`guycorbaz/opencmdb`** (CLAUDE.md: issues are the single source of truth for work items outside the
story flow — a finding that lives only in a story file is not tracked), and raise it with Guy.

### Inherited from stories 5.1, 5.2, 5.2b and 5.3 — read before writing a doc comment

1. **A check that its own commit falsifies is worse than no check.** 5.1 cited a `grep` its own diff
   broke; 5.2 replaced a false doc sentence with another the same commit falsified; 5.3 shipped a
   doc claiming a mechanism its own review measured as weaker. This story rewrites **five** doc
   blocks — re-read every one against the final tree after the last edit.
2. **An inventory in a doc comment has no guard behind it.** Say what THIS type is and what THIS test
   proves; let the register count what is open. One sentence naming 5.4b is a pointer; a list of four
   stories is an inventory.
3. **A count in a doc is a claim.** If this story writes "five types", "seven rule names", "three
   conclusions" or "121 code lines", it **re-counts them on the final tree**. The numbers in Dev Notes
   are the `505379e` baseline, not a post-condition.
4. **A red set is a count too.** Task 7's predictions are measurements taken before the change; the
   record must be what was OBSERVED. Reporting one red where four fired understates the guard exactly
   as an over-claim overstates it.
5. **Two of 5.3's four tests were carried by the compiler, not by their assertions**
   (`deferred-work.md:1045-1053`). ⚠️ **This story does NOT close that entry, and must not claim to.**
   **M2 is the only mutation here that reds an ASSERTION** rather than a name — M3's red is a spelling
   error and M4's a construction-site error, both of which would fire on a test body of
   `assert_eq!(1, 1)`. So the honest statement is *"one of the four mutations is behavioural"*, not
   *"the residue is answered"*. Where a test has no possible behavioural mutation — a `pub` field has
   no code to break — **say that in the record**; it is a real red and a weak test, and both facts
   belong there. The entry's owner stays **5.5**.
6. **Name the test behind every claim.** The temptation here is *"the identity engine now has a return
   type"*. What will hold is: *"`Decision` carries a conclusion, a verdict vector and a mandatory
   ruleset version; `Decision::rule()` mirrors `Outcome::rule()` and a test says so; nothing produces
   a `Decision` and no code combines a verdict."*

### House rules that bind this story

- **Prove-to-red is not optional** (story 1.3). Task 7 names four mutations.
- **Document every public item** — struct, enum, **field**, **variant**, function — in idiomatic
  rustdoc prose (never `@param`/`@return`). This story adds two structs with six fields between them,
  two enums with eight variants, one newtype and two functions: **every one carries a `///`**.
  **A doc comment must be TRUE**; prefer the weaker true sentence. `opencmdb-core` does not yet carry
  `#![deny(missing_docs)]` (`lib.rs:14-18` says why); that is not a licence to skip one.
- **DRY, with deliberate redundancy protected.** `Verdict::all()`'s literal + witness match is the
  protected kind: the exhaustive `match` is what makes a sixth variant a compile error. Do not
  collapse it.
- **File size:** ≤ 2000 CODE lines, tests excluded, counted to the first `#[cfg(test)]`.
- **Dependency frontier (D47):** `opencmdb-core` must not gain `anyhow`, `axum`, `sqlx` or `askama`.
  Nothing this story needs is outside `std` plus the crate's existing types.

### Testing standards

Tests live inline in a trailing `#[cfg(test)] mod tests` (D56b, one per file). Test names are
sentences that say what they prove — `a_decision_names_a_rule_and_an_abstention_does_not` reads at a
CI log; `test_rule` does not. Assertion messages name the variant AND the claim, which matters more
here than in 5.3: four verdict-ish types are in scope. The identity engine's unit tests are pure —
*"the engine is a pure function: a `FixtureConnector` and nothing else — no database"*
[architecture.md:3302], and *"the engine never touches the clock (D19)"* [architecture.md:3364].
Nothing in this story needs either, so neither claim is load-bearing here; they are quoted rather
than paraphrased because a paraphrase that widened them ("no I/O") would be a claim with no source.
⚠️ **Local flakiness is a known unexplained condition (GitHub issue #38)** — a failure that does not
reproduce is reported, never smoothed, and the "Synology Drive" explanation for it was **refuted by
measurement**. No test in this story reads a database, so `DATABASE_URL` is irrelevant to it.

### Project Structure Notes

`identity/` under `opencmdb-core/src/` is the architecture's own layout [architecture.md:3365-3373].
That tree also names `index.rs` (`:3367`), `blocking.rs` (`:3368`), `field_decision/` (`:3370-3372`)
and `migration.rs` (`:3373`) — **none of which this story creates**; `blocking.rs` is story 5.6's.
Everything this story writes goes into the **existing `cascade.rs`**, which the tree describes as
*"the verdict algebra. No float decides. (D13)"* (`:3369`) — so the five types land in the file the
architecture already names for them, and **no new module is created**. `mod.rs` (`:3366`) is
described as holding `IdentityError`; there is still no fallible operation, so it keeps the module
doc and its `pub mod cascade;` and says so. D54's point stands and is unchanged by this story: the
FOLDER is not the frontier — visibility is — and `identity/` earns its existence when
`pub(in crate::identity)` starts meaning something, which nothing here does.

### Git intelligence

Last five commits: `505379e` (issue-#50 bookkeeping, PR #51) · `d4151d1` (5.3 bookkeeping, PR #49) ·
`62f9c83` (**story 5.3**, PR #48) · `12be2fc` (5.2b bookkeeping, #47) · `fe6a19e` (**story 5.2b**, #46).

Story 5.3 is the immediate predecessor and the ONLY Epic 5 commit that touched `opencmdb-core`'s
engine surface — it created `identity/`, retyped `Outcome::Abstained` and edited `score.rs`,
`trap.rs` and `gap/mod.rs`. **So this story inherits a tree its predecessor just reshaped**, which is
why every `score.rs` and `cascade.rs` line number in this file was re-measured at `505379e` rather
than carried over from 5.3's own story file — and why one number carried over anyway and was caught
(the SIX/NINE construction sites). Every one of those commits went branch → PR → green CI → squash
merge; hold the same, and stop at the PR (AC8).

### Toolchain

No new dependency, no version to research. `uuid` is already a dependency of `opencmdb-core` with
`features = ["v7", "serde"]` — Task 6 depends on that fact. Rust 1.96+, edition 2024, `Cargo.lock`
committed, every build `--locked`. **Never invent a version** — pin from the real `Cargo.lock` if one
is ever needed.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 5.4] — epics.md:1423-1443. **Of its four
  acceptance criteria, this story implements three**: the `(rule, verdict, evidence)` triple, the
  no-float/milli-units clause at the TYPE level (its gate is 5.4b's), and `ruleset_version`. The
  fourth — *"combining verdicts is an algebra, never a sum"* — was **SPLIT to story 5.4b** at this
  story's contexting and epics.md:1443 carries the annotation. Build order at `:1317`.
- [Source: _bmad-output/planning-artifacts/architecture.md#D13] — `:929-1011`: the `Verdict` enum
  (`:964`), the six-row table (`:967-974`), *"explanation is free"* (`:977-978`), the refusal of
  `rule -> confidence: f64` (`:956-958`), the milli-units corollary (`:988-993`), `Disqualifying` as
  a structural fact (`:995-1002`).
- [Source: _bmad-output/planning-artifacts/architecture.md#D14] — `:1013-1049`: *"`ruleset_version` is
  mandatory… a silent data migration — the worst kind"* (`:1044-1045`).
- [Source: _bmad-output/planning-artifacts/architecture.md#D20] — `:1348-1399`: strength returns as an
  ORDINAL, never a weight (`:1374-1376`); the four-condition ADR (`:1378-1394`); the complete verdict
  vector as a **data requirement** (`:1396-1399`).
- [Source: _bmad-output/planning-artifacts/architecture.md#D19] — `:1309`: *"a rule that fires must
  leave its `rule_id` and its evidence behind"*.
- [Source: crates/opencmdb-core/src/score.rs] — `Outcome` (`:38-92`), `TrapVerdict` / `run_trap`
  (`:197-250`), the uninhabited placeholders (`:252-286`), `ScoredRecord` (`:288-338`).
- [Source: crates/opencmdb-core/src/identity/cascade.rs] — `IdentityAbstentionCause` and its doc
  (`:13-81`); the `all()` idiom this story copies, doc included (`:84-119`).
- [Source: _bmad-output/implementation-artifacts/deferred-work.md] — `:249-256`, `:309-318`,
  `:328-332`, `:1025-1044`, `:1054-1059`, `:1067-1076`.
- [Source: docs/project-context.md] — the grounding rules for this repository.

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context) — `claude-opus-5[1m]`.

### Debug Log References

No debug session was needed: the implementation compiled and the four new tests passed on the first
run. The four mutations of Task 7 were each run against a byte-copy of the file (`md5` verified
identical on restore, `cc91cf8dcd11880e0691f177c7f4e104` before and after every one), and
`git status fixtures/` was empty at every step.

### Completion Notes List

**In the weaker true sentence:** `identity/cascade.rs` now holds five types — `Verdict` (D13's five
variants and its spelling), `RuleVerdict { rule, verdict, evidence }`, `RulesetVersion(u32)`,
`Conclusion { Match | NoMatch | Abstained }` and `Decision { conclusion, verdict_vector,
ruleset_version }` — plus `Verdict::all()` and `Decision::rule()`. Four tests pin their claims.
**Nothing combines a verdict set, no rule produces a `Verdict`, and nothing produces a `Decision`.**
No byte moved under `fixtures/` (`git status fixtures/` and `MANIFEST.toml` both verified empty).

**Counts re-measured on the FINAL tree** (inherited lesson #3 — a count in a doc is a claim):

- **267 → 271 tests: 135 bin + 94 core + 42 xtask**, zero failures. Only `core` moved, by the four
  new tests, as predicted.
- `cascade.rs`: **660 lines total, first `#[cfg(test)]` at `:445` → 444 CODE lines**, ceiling 2000.
  The `file-size` gate reports 22 files under the ceiling, largest 884.
- `cargo doc --workspace --no-deps` → **3 warnings, the same three that were pre-existing** (`ing`,
  `comparable_fields`, `ScoredRecord`). None of the many new full-path intra-doc links is broken.
- ⚠️ **`Decision` no longer occurs twice.** The story's Dev Notes measured 2 prose occurrences at
  `505379e`; the workspace now has **26**, because the type exists. The baseline was a pre-condition,
  not a post-condition — recorded because this is precisely the number 5.3's review caught being
  carried past its own commit.

**Both forms of clippy are green** — `--all-targets` and the CI form without it, which is the only
one that catches an import kept alive solely by a test module or a `///` link. `cargo xtask ci`: all
five gates green, `ℹ views-hash STALE` exit 0 (expected, GitHub issue #50, **not regenerated**).

**FOUR mutations run, and one prediction was corrected BEFORE dev rather than after:**

- **M1** (delete `ruleset_version` from `Decision`) → **6 errors**: five `error[E0560]`
  (`cascade.rs:528`, `:541`, and three more construction sites) plus one `error[E0609]` at `:653:22`,
  the assertion that READS the field. This is AC3's "mandatory by construction" measured: there is no
  way to obtain a `Decision` without naming a version.
- **M2** (`Decision::rule()` returns `None` for `NoMatch`) → **2 failing tests, both assertion
  panics, not compile errors**: `a_decision_names_a_rule_and_an_abstention_does_not` (`:546`) and
  `the_conclusion_mirrors_the_outcomes_rule_shape` (`:603`). **Both are new, and this is the only one
  of the four that reds an assertion rather than a name.**
- **M3** (delete `Verdict::Disqualifying`) → **`error[E0599]` at three sites** (`all()`'s literal
  `:110`, its witness match `:124`, the test at `:502`) — the lib alone fails with two, the lib-test
  target adds the third. ⚠️ **`error[E0308]` NEVER fires**, exactly as the story predicts after the
  validation agent compiled the case: deleting a variant aborts on the unknown name long before the
  array length is checked. `E0308` belongs to the other repair path. *(The story's first draft
  predicted `E0308` here and was corrected at validation — the prediction was wrong on paper and the
  measurement agrees with the correction, not with the draft.)*
- **M4** (delete `evidence` from `RuleVerdict`) → **4 errors**: three `error[E0560]` at the vector
  test's construction sites (`:610`, `:615`, `:620`) and one `error[E0609]` at `:641:40`. **Plus an
  observed side effect the story did not predict and which is recorded rather than smoothed:** a
  `warning: unused import: crate::observation::ObsId` at `:30:5` — under the CI form of clippy that
  warning is a hard error, so the `evidence` field is what keeps that import live in the lib build.

⚠️ **Two of the four reds are carried by the COMPILER, not by an assertion, and the record says so**
rather than claiming the residue closed. M3's red is a spelling error and M4's a construction-site
error — both would fire identically on a test body of `assert_eq!(1, 1)`. M1 and M2 are the two that
are not. `deferred-work.md:1045-1053`'s owner **stays 5.5**; this story does not close it, and
`the_verdict_vector_carries_the_whole_triple_in_order` has **no possible behavioural mutation** —
a `pub` field has no code to break.

**Six doc locations changed, five of them because this story falsified them** — `cascade.rs`'s module
doc, `identity/mod.rs:8-9`, `score.rs:46-47` (*"Not named `Decision`"* — the name is now taken),
`score.rs`'s `VerdictVectorEntry` element sentence (the triple has a type; the *"uninhabited"* and
*"provably empty"* claims survive untouched, as AC5 required), and `trap.rs`'s `RuleId` doc, whose
*"Epic 5 names them"* prediction AC2's measurement refutes. The sixth, `TrapVerdict`'s doc, was not
false and gained the reciprocal cross-reference. **Every one was re-read against the final tree after
the last edit.**

**The register.** `deferred-work.md:1073-1076` (*"Owner: whoever writes 5.4's AC7 — require the PR and
stop"*) is **CLOSED by append-and-strike**, this story being the owner it names. Three entries are
annotated `↺` and **none is struck**: the two `NoMatch` bullets, whose owner moves to **5.4b/5.5**
(the owner string lives in 5.3's ANNOTATION bullets, so it was struck in place there and the update
appended — nothing was rewritten), and the firing-rule/evidence contract, whose owner moves to
**5.5**. ⚠️ The new annotation was placed **above** its target, the idiom of its two nearest
neighbours, and **`:319-327` was re-read afterwards**: 5.3's own *"the entry BELOW"* and *"the
firing-rule/evidence entry above it"* both still resolve — the section reads 5.4's annotation →
firing-rule entry → 5.3's annotation → `NoMatch` entry. No amendment was needed.

A new `## Deferred from: story-5.4` section is appended at the END (the file is chronological) with
**fifteen items — thirteen naming a story, two naming the condition that would produce an owner**
(`whoever writes D20's ADR`, `whoever revisits the crate's re-export policy`). The section's own
preamble states that split rather than claiming fifteen named owners.

**Two `Owner: Epic 5` entries were deliberately NOT touched**, and saying so turns silence into a
scope statement: `:285-290` (lattice monotonicity — needs an engine producing verdicts across
capability subsets, story 5.13) and `:336-344` (`RuleId` whitespace/case normalization in `run_trap`
— it bites when a PRODUCER emits a `RuleId`, story 5.5).

**Scope held.** No `decide()`, no table, no `xtask` gate (5.4b) · no rule, no join, no producer (5.5)
· no `From<Decision> for Outcome`, `VerdictVectorEntry` untouched and still uninhabited (5.7) · no
serde, no persistence (5.9) · no `Display`, no `cause()`, no grouping (5.14) · `RuleId` not closed
into an enum (Epic 6) · `page.rs`, `locales/app.yml`, `fixtures.rs`, `trap_gate.rs`, `gap/mod.rs`,
`xtask/` and everything under `fixtures/` untouched.

### File List

**Modified — code:**

- `crates/opencmdb-core/src/identity/cascade.rs` — five new types, `Verdict::all()`,
  `Decision::rule()`, four new tests, the test module's placement convention, module doc rewritten.
- `crates/opencmdb-core/src/identity/mod.rs` — module doc (`:8-9`), doc only.
- `crates/opencmdb-core/src/lib.rs` — the `identity::cascade` re-export widened to six names.
- `crates/opencmdb-core/src/score.rs` — three doc blocks (`Outcome`'s *"Not named `Decision`"*,
  `TrapVerdict`'s cross-reference, `VerdictVectorEntry`'s element sentence). **Doc only.**
- `crates/opencmdb-core/src/trap.rs` — `RuleId`'s doc. **Doc only**; the declaration is
  byte-unchanged.

**Modified — documents:**

- `_bmad-output/implementation-artifacts/deferred-work.md` — one closure, three `↺` annotations, one
  new section of fifteen items.
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/planning-artifacts/epics.md` — story 5.4b inserted, Epic 5 counts to sixteen, the
  float AC annotated as split *(done at contexting, before dev)*.
- `docs/project-context.md`, `CLAUDE.md`

**Added:**

- `_bmad-output/implementation-artifacts/5-4-decision-and-ruleset-version.md` (this file)

**Untouched, verified:** everything under `fixtures/` (including `MANIFEST.toml`), `Cargo.lock`,
`crates/opencmdb-bin/` in its entirety, `xtask/`, `crates/opencmdb-core/src/gap/mod.rs`.

## Change Log

| Date | Change |
|---|---|
| 2026-07-29 | Story contexted; story 5.4b inserted into `epics.md` with Guy; Epic 5 → 16 stories. |
| 2026-07-29 | Validated by two fresh-context agents (fact-check + gap-hunt); 11 HIGH + 16 MEDIUM/LOW applied, including a mutation prediction the gap-hunt agent compiled and refuted. |
| 2026-07-29 | Implemented: five types, four tests, six doc locations, four mutations run. 267 → 271 tests. Status → `review`. |
