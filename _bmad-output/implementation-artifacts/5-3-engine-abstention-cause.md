# Story 5.3: The identity engine gets its own abstention cause

Status: review

<!-- Validation is MANDATORY here (Guy's decision, Epic 4 retrospective 2026-07-26): two
     fresh-context agents (fact-check + gap-hunt) BEFORE dev-story. The template banner saying
     "Validation is optional" does not apply to this project. -->

## Story

As the identity engine,
I want an abstention cause vocabulary of my own rather than the reconciliation engine's,
so that `Ambiguous` — which emerges from the verdict algebra and which none of the three
reconciliation causes names — can be produced and displayed honestly.

**This is the FIRST story of Epic 5's engine proper.** The three inherited-debt stories (5.1, 5.2,
5.2b) are all `done` and merged; the corpus is the oracle and it is now pinned. Build order
(epics.md:1317): **the engine's vocabulary (5.3, 5.4)** → the pure join (5.5) → the blocker (5.6) →
wiring it to the corpus (5.7, 5.8) → persistence (5.9, 5.10) → the invariants (5.11–5.13) → the
operator-visible surface (5.14).

**This story writes a TYPE and its tests. It writes no engine.** There is no cascade, no rule, no
`Decision`, no join, no producer — those are 5.4/5.5/5.6. What lands here is the vocabulary the
producer will speak, chosen now because story 4.6a recorded that the vocabulary in use is wrong and
named Epic 5 as the owner of the choice (`deferred-work.md:160-167`). Choosing it before the engine
exists is the same discipline as Epic 4's *"a metric written after the engine is bent to fit the
engine"*.

**Nothing under `fixtures/` moves.** No artefact bytes, no `MANIFEST.toml`, no re-hash. If any step
appears to require re-authoring a committed artefact, **STOP**: that is a finding, reported rather
than absorbed. The procedure is in Dev Notes.

## Acceptance Criteria

1. **AC1 — The identity engine has its own abstention-cause type, and `Ambiguous` is one of its
   variants.**
   **Given** `AbstentionCause { OutOfPerimeter, NoObservedValue, ConflictingObservations }`
   (`gap/mod.rs:24-34`), which `reconcile` matches exhaustively and which `score.rs:49-61` records as
   inadequate for the cascade
   **when** the engine abstains
   **then** it names a cause from its OWN enum, and `Ambiguous` — a `Decisive` verdict with at least
   one `Opposes`, the cloned-MAC case (D13, architecture.md:971) — is one of its variants.

   Binding specifics, so this is not re-litigated at review time:
   - **Name:** `IdentityAbstentionCause`. Not `AbstentionCause`, and the reason is a compile error,
     not a preference: `lib.rs:38` re-exports `gap::AbstentionCause` **flat** at the crate root, and
     `page.rs:15` imports it as `opencmdb_core::AbstentionCause`. A second type of the same name in
     that same `pub use` block is **`error[E0252]`**. Secondarily, `score.rs`'s test module holds
     both vocabularies in ONE scope — `must_abstain()` (`:607-611`) keeps `AbstentionCause` on the
     Expectation side while `abstained()` (`:628-632`) gains the new type on the Outcome side — so a
     shared spelling would force an alias at the one place the asymmetry must read clearly.
     *(`trap_gate.rs` is NOT that place: `Expectation` does not appear in it at all — its
     `must-abstain` side arrives as TOML text inside raw string literals, `:500` and `:639`.)*
   - **Location:** `crates/opencmdb-core/src/identity/cascade.rs`, with a new
     `crates/opencmdb-core/src/identity/mod.rs` declaring it. That is the file the architecture's
     source tree names for the verdict algebra — *"`cascade.rs` — the verdict algebra. No float
     decides. (D13)"* [architecture.md:3369], under `identity/` [architecture.md:3365]. Registered in
     `lib.rs` beside the existing modules and re-exported in the `pub use` block, following the
     `gap`/`score` idiom (`lib.rs:20-26`, `:38`, `:44-47`).
   - **Visibility:** plain `pub`. D54's `pub(in crate::identity)` discipline does not apply to this
     type — `score::Outcome`, a different subdomain, names it in its signature. Restricting it would
     not compile, and claiming the module boundary as an invariant guard here would be the "theatre"
     D54 itself refuses.
   - **Derives:** `Debug, Clone, Copy, PartialEq, Eq` — exactly what `Outcome`'s own derives require
     (`score.rs:62`) plus `Copy`, which a field-less enum gets for free and which `AbstentionCause`
     already has.
   - **NO `Serialize`/`Deserialize`, NO `PartialOrd`/`Ord`.** Nothing persists or orders one yet.
     `ScoredRecord`'s own doc states the precedent: *"No `Serialize`: nothing persists these yet, and
     deriving a wire format for a domain type with no consumer is a finding this project has already
     recorded once"* (`score.rs:304-305`). Story 5.9 adds serde if it persists one; story 5.14 adds
     `Ord` if it keys a `BTreeMap` to group by cause. Adding either here is scope creep with no
     producer.
   - **NO `#[non_exhaustive]`.** The attribute bites across crate boundaries, and `opencmdb-bin` is a
     different crate: every downstream `match` would be forced to carry a `_` arm — which is exactly
     the `error[E0004]` mechanism AC4/M4 and `all()` depend on. Recorded as a decision rather than
     left silent because the code review of 4.6a asked the same question of `ScoredRecord`
     (`deferred-work.md:205-209`), and a reviewer will ask it again of a type AC1b openly says may
     split later.
   - **NO `Display` impl.** Story 5.14 renders a cause through `page.rs`'s `cause_label` + `t!()`
     seam (`:112-119`), not through `Display`. Writing one "for 5.14" builds the wrong seam.
   - **`pub fn all() -> [Self; 2]`** — the variant witness. Signature and length are part of the
     contract: the array length is the SECOND guard (a third variant makes the literal
     `error[E0308]`), the exhaustive witness match inside is the first (`error[E0004]`). The idiom is
     spelled out in Task 3 — do not improvise it.

2. **AC1b — Exactly TWO variants, each traced to a row of D13's table; a third is a finding.**
   **Given** D13's verdict algebra [architecture.md:964-974], whose six rows yield three outcomes
   **when** the variants are chosen
   **then** the enum carries exactly:
   - **`Ambiguous`** — the three rows that produce `Ambiguous`: a `Decisive` with ≥1 `Opposes` (*the
     cloned-MAC case*, architecture.md:971), no `Decisive` with ≥1 `Supports` and no `Opposes` (weak
     evidence, `:972`), and `Supports` AND `Opposes` (conflict, `:973`).
   - **`AbsenceOfProof`** — architecture.md:974's row *"only `Neutral` / nothing → `NoMatch`
     (absence of proof)"*. This variant exists because of a register entry, not because it is tidy:
     *"`Outcome::Refused` requires a rule to name, so absence-of-proof has to map to `Abstained`. If
     Epic 5 maps `NoMatch → Refused` uniformly, every honest `must-abstain` trap fails"*
     (`deferred-work.md:217-224`) — the exact case D18 says must NOT be gated.

   **And** the OTHER two rows are accounted for on the page, so a reader does not find a row with no
   variant and guess: `architecture.md:969`'s *"any `Disqualifying` → `NoMatch`"* is an **active
   opposition** — a rule opposes, so it names that rule and is an `Outcome::Refused`, never an
   abstention (`deferred-work.md:217-224` is explicit that only the absence-of-proof half of `NoMatch`
   maps to `Abstained`, precisely *because* `Refused` requires a rule to name). `:970`'s *"a
   `Decisive`, no `Opposes` → `Match`"* is an `Outcome::Merged`. **Exactly one `NoMatch` row has no
   rule to name, and that row is `AbsenceOfProof`.** The enum's doc carries this sentence: six rows,
   two abstention variants, and the four-row remainder explained rather than omitted.

   **And** no variant is added for anything without a row behind it. In particular **`OutOfPerimeter`
   is NOT reproduced**: the cascade's table has no such row, and D16 names the failure mode of
   copying causes across — *"if `Ambiguous` means both 'real conflict' and 'unmodelled case', it
   means nothing, and the operator learns to ignore it"* [architecture.md:1112-1115]. A third variant
   arriving in this story is a **finding**, raised before it is written.

   **And** whether `Ambiguous` must later SPLIT along its three rows is **registered, not decided
   here**: 5.14 owns the operator-facing grouping and is the first place a split can be justified by
   a consumer. FR16b's three example causes [Source: prd.md:897] are a different axis, not a
   contradiction — *"no live source on the scope"* is a liveness question (D34), not an identity
   abstention at all; *"address grouping unresolved"* lands on `Ambiguous`; *"a signal absent from
   every capable source"* lands on `AbsenceOfProof`. Two independent confirmations that this
   vocabulary is the epic's own: epics.md's story 5.14 already writes *"Given an abstention whose
   cause is `Ambiguous`"*, and epics.md:1311 records the arbitration that sent the choice here.

   **And** a pre-empted objection, because a blind review layer will raise it: AC1 refuses serde for
   having *no consumer* while this AC ships `AbsenceOfProof` with *no producer*. These are not the
   same act. **A variant is the vocabulary this story is chartered to choose** — the register entry at
   `deferred-work.md:160-167` names the choice as Epic 5's, and choosing the vocabulary before the
   engine is D19's order, the same discipline that put the metric before the engine in Epic 4. **A
   derive is a mechanism with a downstream cost** (a wire format, a label, two locales) — deriving one
   before its consumer is the inversion of that order, not an instance of it.

3. **AC2 — `Expectation::MustAbstain` keeps `AbstentionCause`, and no corpus artefact is re-hashed.**
   **Given** the committed corpus, which writes `must-abstain = { cause = "NoObservedValue" }` in
   **three** sha256-locked trap files — `example.toml:44`, `hostname-absence.toml:47`,
   `shared-hardware-vm.toml:46` (counted on the committed tree at `12be2fc`)
   **when** the new type lands
   **then** `trap.rs:82`'s `MustAbstain { cause: AbstentionCause }` is **byte-unchanged**, the change
   is confined to `score.rs:69`'s `Outcome::Abstained`, and `git status` under `fixtures/` is empty
   at the end of the story.

   **And** the two existing guards that pin the committed spelling stay green and are **not
   modified**: `the_committed_abstention_vocabulary_is_pinned` (`trap.rs:584-591`, asserts
   `serde_json::to_string(&AbstentionCause::NoObservedValue) == "\"NoObservedValue\""`) and
   `an_unknown_field_is_refused` (`trap.rs:569-579`).

4. **AC3 — The asymmetry is documented on BOTH types, and the reason it is sound is the mechanism,
   not an assurance.**
   **Given** that the two sides of a trap now carry DIFFERENT cause types
   **when** a reader meets them
   **then** both `gap::AbstentionCause` and `identity::cascade::IdentityAbstentionCause` carry a doc
   paragraph naming the counterpart, saying which side of a trap it lives on, and stating why the
   asymmetry cannot make the gate asymmetric: **`score` is cause-blind by construction** — its 3×3
   matches `Outcome::Abstained { .. }` with `..` and never reads the payload (`score.rs:175-189`),
   and `run_trap` compares rules only where BOTH sides are `Some`, which an abstention never is
   (`score.rs:228-244`, `Outcome::rule()` at `:80-85`).

   **And every cross-module doc link is written as a FULL path** — `` [`crate::gap::AbstentionCause`] ``,
   `` [`crate::identity::cascade::IdentityAbstentionCause`] ``. **An import added only to shorten a doc
   link is an unused import**, measured: rustc emits `warning: unused import` when the sole use of a
   name is a `///` intra-doc link, and `-D warnings` turns that into a hard error. Three files are
   exposed to this (`score.rs`, `gap/mod.rs`, `cascade.rs`), because in each one the linked
   counterpart has no other non-test use.

   **And** THREE doc blocks become false the moment this story lands and are **rewritten, not left
   standing** — *"a doc comment must be TRUE"* is the rule this project has caught itself breaking
   three times:
   - `score.rs:49-61` — *"The abstention cause is known to be inadequate, and that is recorded rather
     than fixed… Epic 5 builds the cascade and should decide"*. Epic 5 has now decided.
   - `trap.rs:81` — `Expectation::MustAbstain`'s variant doc, *"the engine must refuse to decide, **for
     this cause**"*. After this story the engine's cause is a different type that is never compared
     against this one, so the doc asserts a coupling that does not exist. **Doc-only edit, explicitly
     exempt from AC2's "byte-unchanged"** which protects the declaration at `:82`. The weaker true
     sentence: this cause is the trap author's note about the SHAPE of the case, not the cause the
     engine answers with.
   - `score.rs:613-617` — *"The outcomes deliberately carry rules and **a cause** that DO NOT match the
     expectations above"*. Post-change the cause CANNOT match: the types differ. What was a deliberate
     choice becomes a type-level fact, and the sentence must say which.

   **And** the claim is carried by a TEST, not only by prose: scoring a `must-abstain` expectation
   (carrying `AbstentionCause::NoObservedValue`, the committed spelling) against an
   `Outcome::Abstained { cause: IdentityAbstentionCause::Ambiguous }` returns `Score::Pass` — the
   cross-vocabulary pair, which is what "nothing ever compares the two" means when it is executable.

5. **AC4 — `reconcile` is not widened with a variant it can never produce.**
   **Given** `AbstentionCause`'s three variants, produced at `gap/mod.rs:90`, `:108` and `:122`
   **when** this story ends
   **then** the enum's variant list is unchanged (the only edit to `gap/mod.rs` is the AC3 doc
   paragraph), and the mutation that proves widening is not free is recorded: adding `Ambiguous` to
   `AbstentionCause` produces **`error[E0004]`** in `page.rs:112-119`'s `cause_label`, whose match is
   exhaustive with no `_` arm — i.e. a variant `reconcile` can never produce would still demand a
   user-facing label and two locale strings (`crates/opencmdb-bin/locales/app.yml`, the three
   `cause.*` keys at `:48`, `:51`, `:54`, each with an `en:` and a `fr:`). **rustc reports the match
   scrutinee line, `page.rs:114:11`** — `:112` is the `fn`. Quote what rustc actually prints.

6. **AC5 — Every new guard is proven to red before it passes, and each mutation records the COMPLETE
   set of reds it produced.**
   Prove-to-red is the house rule (story 1.3). The five mutations of Task 6 are run and recorded, each
   naming **every** file, line and failing test — **or** the exact compiler error code and site — that
   went red, and then naming which of those reds is the one the NEW guard contributes. A compile error
   counts as a red — it is the same mechanism `score`'s own doc relies on (*"a new `Expectation` or
   `Outcome` variant must break THIS function and force a decision"*, `score.rs:169-171`) — but it must
   be **run and quoted**, never predicted.

   **A mutation with several reds is expected here and is not a defect.** `score.rs`'s 3×3 is the
   spine of the trap gate; flipping one cell reds five tests, not one. Task 6 lists the measured sets.
   The requirement is that the record be **exhaustive and observed** — never that the red be unique.
   A record naming one red where five fired is the under-reporting this project's reviews keep
   catching, and it is the same defect as an over-claim.

7. **AC6 — The register is closed by appending, never rewriting.**
   **Given** `deferred-work.md`'s three entries that name this decision
   **when** the story ends
   **then**:
   - `## Deferred from: story-4.6a`'s bullet at **`:160-167`** (*"`AbstentionCause` cannot express the
     identity cascade's `Ambiguous`, and Epic 5 must decide"*) is marked **`✅ CLOSED by story 5.3`**
     in the file's own `✅ **CLOSED by story X.** ~~struck~~` idiom, naming the branch taken (**a
     separate cause type**, not a widened enum) and the test that holds it.
   - The two `NoMatch` entries — `:217-224` (*"maps two ways onto `Outcome`, and only half of that is
     recorded"*) and `:287-291` (*"the `NoMatch → Refused` vs `Abstained` question is Epic 5's"*) —
     are **annotated as PARTLY closed, not struck**: this story creates the cause that
     absence-of-proof will carry; **the mapping itself has no producer until 5.4/5.5**, which remain
     the owners. Marking them closed would be the over-claim this project's reviews keep catching.
   - Anything this story leaves open (the `Ambiguous`-split question of AC1b; `Ord`/serde deferred to
     5.14/5.9; `Display` deferred to 5.14) opens a `## Deferred from: story-5.3` section with a named
     owner. The file is **chronological**: append it at the END, after the last section
     (`## Deferred from: code review of story-5.2b`, `:900`) — never interleaved by topic.
   - **Three other `Owner: Epic 5` entries are deliberately NOT touched**, and saying so is what turns
     silence into a scope statement: `:255-259` (lattice monotonicity), `:275-284` (the
     firing-rule/evidence contract), `:295-303` (`RuleId` whitespace/case normalization in
     `run_trap`). None concerns the abstention vocabulary, and each needs a producing engine
     (5.4–5.7). A one-line note in the Completion Notes is enough; do not edit those entries.

8. **AC7 — The gate is green, the docs are current, and the flow is honoured.**
   Tasks 8–10 are acceptance criteria, not housekeeping, so an AC-by-AC auditor can see them:
   `cargo fmt --all` clean · `cargo clippy --workspace --locked --all-targets -- -D warnings` clean ·
   `cargo test --workspace --locked` green with the three per-crate counts **re-measured on the final
   tree** · `cargo xtask ci` green (`ℹ views-hash STALE`, exit 0, is correct) · `git status` under
   `fixtures/` empty · `sprint-status.yaml` and `docs/project-context.md` updated in the same push ·
   branch → PR → green CI → squash merge, ending at status `review`, never `done`.

## Tasks / Subtasks

- [x] **Task 1 — Read before writing** (AC1, AC3)
  - [x] `crates/opencmdb-core/src/score.rs:32-86` (imports, `Outcome`, its doc block, `rule()`),
        `:175-189` (`score`), `:228-244` (`run_trap`), `:282-344` (`ScoredRecord`, for the no-serde
        precedent at `:304`).
  - [x] `score.rs`'s test module **preamble**: `:583` (`use super::*` — the ONLY way the tests reach
        `AbstentionCause` today), `:607-611` (`fn must_abstain()`, the seventh `Expectation`-side
        site, which stays on `AbstentionCause`), `:613-617` (the deliberate-mismatch doc),
        `:628-632` and `:717-719` (the two `Outcome::Abstained` constructors). Read these BEFORE
        touching the import at `:34` — that import is load-bearing for `:609`.
  - [x] `crates/opencmdb-core/src/gap/mod.rs:16-52` and the three `abstain(...)` call sites.
  - [x] `crates/opencmdb-core/src/trap.rs:55-102` (`Expectation`) and `:581-591` (the pinned spelling).
  - [x] `architecture.md:929` (D13) and `:960-1000` — the verdict enum and the six-row table. Start
        at the Decision Index near the top of the file (F56), not at a grep.
  - [x] `deferred-work.md:160-167`, `:217-224`, `:287-291`.

- [x] **Task 2 — Create the `identity/` module** (AC1)
  - [x] `crates/opencmdb-core/src/identity/mod.rs` — module doc saying what the subtree owns (FR9-20,
        composite identity) and what it holds **today** (`pub mod cascade;` and nothing else — there
        is no fallible operation yet, so the `IdentityError` the architecture's tree names at
        `architecture.md:3366` does not exist). Declare `pub mod cascade;` — **`pub`, not bare `mod`**:
        the re-export chain is what keeps `all()` publicly reachable and therefore out of `dead_code`.
  - [x] ⚠️ **The module doc names at most the immediate next owner, and no more.** A forward inventory
        (*"`Decision` → 5.4, the join → 5.5, the blocker → 5.6…"*) is exactly the "inventory in a doc
        comment with no guard behind it" that 5.2's review caught — the register counts what is open,
        the doc says what IS. One sentence attributing the algebra to 5.4 is a pointer, not an
        inventory; a list of four is an inventory.
  - [x] **No inner attributes** on either new file. `#![deny(missing_docs)]` stays off for
        `opencmdb-core` until the field/variant sweep lands (`lib.rs:14-18` says why); switching it on
        for one submodule is a per-module deviation nobody asked for.
  - [x] `crates/opencmdb-core/src/identity/cascade.rs` — module doc that says what **this file holds
        today**: the identity engine's abstention vocabulary. D13's contract (all rules evaluated,
        verdicts combine by an **algebra, never a sum**, no float crosses a decision boundary) is
        quoted **and attributed to story 5.4 in the same sentence** — never phrased as a description of
        this file's behaviour. *"A doc comment must be TRUE; prefer the weaker true sentence."* Three
        confident sentences followed by a hedge reads, to a reviewer and to `grep`, as three false
        sentences.
  - [x] `lib.rs`: add `pub mod identity;` to the module list (`:20-26`, alphabetical — after `gap`)
        and `pub use identity::cascade::IdentityAbstentionCause;` to the re-export block (`:36-47`,
        which is ordered **by module**: `clock`, `connector`, `gap`, `observation`, `repo`, `score` —
        so the new line goes between the `gap` re-export at `:38` and the `observation` one).

- [x] **Task 3 — Define `IdentityAbstentionCause`** (AC1, AC1b, AC3)
  - [x] The two variants of AC1b, with a `///` on the enum **and on each variant** (CLAUDE.md: every
        `pub` struct, enum, **field**, **variant** and function carries one). Each variant doc cites
        the architecture row it comes from — the row, not a paraphrase.
  - [x] The enum doc also carries AC1b's six-row accounting: two variants, and why the other four rows
        produce no variant. That paragraph is the guard against a third variant arriving later
        "because a row looked uncovered".
  - [x] The asymmetry paragraph (AC3): names `` [`crate::gap::AbstentionCause`] `` **by full path**,
        says which side of a trap each lives on, and names `score`'s cause-blindness as the mechanism.
        **No `use` for the doc link** — it would be an unused import under `-D warnings`.
  - [x] `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` — and a one-line comment saying why serde,
        `Ord`, `Display` and `#[non_exhaustive]` are absent, with their owners (5.9, 5.14, 5.14, and
        "never" respectively).
  - [x] **`pub fn all() -> [Self; 2]` — the exact idiom, because there is no way to build a `[T; N]`
        without listing its elements.** A literal alone is not enough; a match alone cannot return the
        array. Ship BOTH guards:
        ```rust
        pub fn all() -> [Self; 2] {
            let all = [Self::Ambiguous, Self::AbsenceOfProof];
            // The witness: a new variant makes this match error[E0004]; a variant added to the
            // literal without widening the return type is error[E0308]. Neither can drift silently.
            for c in all {
                match c {
                    Self::Ambiguous => {}
                    Self::AbsenceOfProof => {}
                }
            }
            all
        }
        ```
        The **return type** is the second mechanism — pin the length at `2`, never `[Self; N]` behind a
        const. (The precedent for two spellings pinned against each other is
        `Column::as_str`/`Expectation::column` — CLAUDE.md's protected "deliberate redundancy".)

- [x] **Task 4 — Retype `Outcome::Abstained`** (AC2, AC3)
  - [x] `score.rs:69` → `Abstained { cause: IdentityAbstentionCause }`.
  - [x] ⚠️ **The two import edits are NOT symmetric, and the naïve version of either reds CI.** Read
        this bullet before touching a `use` line.
        - **`score.rs:34`** is **replaced** by `use crate::identity::cascade::IdentityAbstentionCause;`.
          The test module then gains **its own** `use crate::gap::AbstentionCause;` inside
          `mod tests` (beside `:584`). Why it cannot simply stay at `:34`: after the retype,
          production `score.rs` names `AbstentionCause` only in doc comments (`:51`, `:96`), and
          **CI runs `cargo clippy --workspace --locked` with NO `--all-targets`** — the lib is compiled
          without `cfg(test)`, so an import kept alive only by the test module is `unused_imports` →
          hard error under `-D warnings`. Why it cannot simply be deleted: the test module reaches
          `AbstentionCause` **only** through `use super::*` (`:583`), and still needs it at `:609`.
        - **`trap_gate.rs:389`** is **replaced** by
          `use opencmdb_core::identity::cascade::IdentityAbstentionCause;` — **not doubled.** After the
          four sites are retyped, `gap::AbstentionCause` has **zero** remaining uses anywhere in that
          file: its `must-abstain` side arrives as TOML text in raw string literals (`:500`, `:639`),
          never as the Rust type, and `Expectation` does not appear in the file at all. Keeping the old
          import is an unused import.
  - [x] **Rewrite the doc block at `:49-61`.** It currently says the vocabulary is inadequate, that
        it is used anyway, and that Epic 5 *should decide*. Replace with what is now true: the
        outcome speaks the engine's vocabulary, the expectation speaks the corpus's, nothing compares
        them, and the two-sentence reason why that is sound. Do not leave a "recorded rather than
        fixed" sentence behind a change that fixed it. Any link to the counterpart type is a **full
        path**, never an import.
  - [x] **Rewrite `score.rs:613-617`** (the deliberate-mismatch doc on the test fixtures) and
        **`trap.rs:81`** (`MustAbstain`'s variant doc) — both listed in AC3, both doc-only, both false
        the moment the field type changes.
  - [x] Fix the six construction sites — **all in test modules, none in production code — and the
        variant is named PER SITE, not left to taste**:
        | site | today | after |
        |---|---|---|
        | `score.rs:628-632` (`fn abstained()`) | `AbstentionCause::ConflictingObservations` | `IdentityAbstentionCause::Ambiguous` |
        | `score.rs:717-719` (`other_cause`) | `AbstentionCause::ConflictingObservations` | `IdentityAbstentionCause::AbsenceOfProof` |
        | `trap_gate.rs:451`, `:509`, `:653`, `:659` | `AbstentionCause::NoObservedValue` | `IdentityAbstentionCause::Ambiguous` |
  - [x] ⚠️ **`abstained()` and `other_cause` MUST carry different variants.** `other_cause` exists for
        one reason: to give `scoring_ignores_the_rule_because_the_trap_runner_owns_it` (`:702-721`) a
        cause that DIFFERS from the one under test. Retype both to the same variant and the two helpers
        become value-identical, the assertion at `:720` duplicates `:693`, and the sub-claim evaporates
        **while the test stays green** — a check that proves nothing, which is the failure this project
        bans outright. This is also the only place `AbsenceOfProof` is exercised as a value, which is
        why it is the one assigned here.
  - [x] That test's comment currently justifies itself with *"same for a mismatched abstention cause"*
        — a mismatch between two values of ONE type. After the change the mismatch is **cross-type**.
        **Update the comment to say what it now proves**; the assertion stays, its justification
        changes, and leaving the old sentence would be a false doc.
  - [x] **The seventh site is the trap: `score.rs:607-611` (`fn must_abstain()`) STAYS on
        `AbstentionCause::NoObservedValue`.** It is the `Expectation` side, it feeds nine cell
        assertions, and retyping it would violate AC2. It is also the reason `:34` cannot simply be
        swapped.
  - [x] `Expectation::MustAbstain`'s declaration (`trap.rs:82`), `gap::AbstentionCause`'s variants,
        `page.rs::cause_label`, the locale file and everything under `fixtures/` are **NOT touched**.
        Three doc-only exceptions, all listed in AC3: the counterpart paragraph in `gap/mod.rs`,
        `trap.rs:81`'s variant doc, and `score.rs:613-617`.

- [x] **Task 5 — The tests** (AC1b, AC3, AC5) — inline, in trailing `#[cfg(test)] mod tests` modules
      (D56b: one per file). **The tests are SPLIT across two files, and which file gets which is part
      of the spec.** The rule this codebase already follows: *a test module tests the items of its own
      file, importing other modules only as dependencies* (`gap/mod.rs:130` + `:132`, `score.rs:582` +
      `:584`, `trap.rs:400`). Since `score` depends on `cascade` and not the reverse, a test of
      `score()`'s truth table belongs in `score.rs` — putting it in `cascade.rs` would invert the
      only convention the crate has, and would land Task 6's M2/M3 reds in a different file from the
      mutation.

  **In `identity/cascade.rs`'s test module — the type's own claims:**
  - [x] **The variant set cannot go stale.** `all()` returns exactly `{Ambiguous, AbsenceOfProof}` as a
        SET — so a third variant added without a row behind it (AC1b) fails a test rather than passing
        silently. Do **not** assert the length: on `[Self; 2]` that is a tautology the type already
        guarantees. Assert the set; say so in a comment, so a later reader does not "fix" the omission.
  - [x] **An abstention has no rule, by construction.** For each element of `all()`,
        `Outcome::Abstained { cause }.rule()` is `None`.

  **In `score.rs`'s test module — the scoring claims, beside the 3×3 they concern:**
  - [x] **`score` is cause-blind across every variant.** For **each** element of `all()`:
        `(MustAbstain, Abstained{cause})` → `Pass`, `(MustNotMerge, Abstained{cause})` → `Pass` (the
        load-bearing cell, `score.rs:158-167`), `(MustMerge, Abstained{cause})` → `Fail` (cowardice).
        Loop over `all()`, do not hand-write two blocks — this is behaviour with one source of truth,
        not the deliberate-redundancy case.
  - [x] **The cross-vocabulary pair (AC3).** `MustAbstain { cause: AbstentionCause::NoObservedValue }`
        — the spelling the three committed trap files carry — against
        `Abstained { cause: IdentityAbstentionCause::Ambiguous }` → `Score::Pass`. The test name says
        what it proves: the two vocabularies are never compared.
  - [x] **`run_trap` returns `TrapVerdict::Pass`, never `WrongRule`**, for each variant against a
        `must-abstain` expectation.

  - [x] Assertion messages name the variant AND the cell. With two vocabularies in play,
        *"expected Pass"* is not actionable at a CI log.

- [x] **Task 6 — Prove to red** (AC5). Run each, quote the observed failure, restore, re-run green.
      **The expected red SETS are listed below, measured on the committed tree.** They are a
      prediction to check against, not a licence to skip running: if the observed set differs from the
      predicted one, the DIFFERENCE is the finding, and it goes in the Completion Notes.
  - [x] **M1** — delete the `Ambiguous` variant → compile errors, not test failures: `all()`'s literal
        (`error[E0308]`, wrong array length) and its witness match, the variant-set test, the
        cross-vocabulary test, and every retyped call site naming `Ambiguous` (`score.rs`'s
        `abstained()`, `trap_gate.rs`'s four). Quote the first error and the site count.
  - [x] **M2** — `score.rs:187`: `(MustAbstain, Abstained)` → `Score::Fail`. Predicted reds:
        `must_abstain_fails_on_any_decision` (`:693`),
        `an_engine_that_abstains_on_everything_is_demolished_by_the_middle_column` (`:666`),
        `an_abstention_on_a_passing_cell_is_never_a_wrong_rule` (`:995`), `trap_gate.rs`'s
        `a_trap_with_no_answer_is_discovered_but_not_scored` and its mixed-corpus test, plus the two
        NEW guards (cause-blindness loop, cross-vocabulary pair). **Name which of those is new** —
        that is the one this story earns.
  - [x] **M3** — `score.rs:183`: `(MustNotMerge, Abstained)` → `Score::Fail`, D18's load-bearing cell.
        Predicted reds: `must_not_merge_fails_only_on_a_merge` (`:650`),
        `an_engine_that_abstains_on_everything_is_demolished_by_the_middle_column` (`:664`),
        `a_column_that_never_ran_is_distinguishable_from_one_that_passed` (`:787`),
        `an_abstention_on_a_passing_cell_is_never_a_wrong_rule` (`:989`, `:993`), plus the new
        cause-blindness loop — **five or more, not two.** The cell is defended in depth; record all of
        it.
  - [x] **M4** — add `Ambiguous` to `gap::AbstentionCause` → **exactly one** `error[E0004]`, at
        **`page.rs:114:11`** (the `match cause` scrutinee inside `cause_label`; `:112` is the `fn`).
        Nothing else breaks — not serde, not `trap.rs:584-591`'s spelling pin, not `fixtures.rs`. This
        is AC4's evidence: widening the reconciliation enum is not free, and a variant `reconcile` can
        never produce would still demand a label and two locale strings. Quote the error verbatim.
  - [x] **M5** — revert `Outcome::Abstained`'s field type to `gap::AbstentionCause` → the new tests
        fail to compile. Proves the type change is what the tests are about, not incidental.
  - [x] ⚠️ **Restore after every mutation and verify with `git status` before the next one.** Local
        flakiness (issue #38) and a forgotten revert look identical.

- [x] **Task 7 — The register** (AC6) — append-and-strike, never rewrite a bullet.
  - [x] Close `:160-167` with the branch taken and the test that holds it, in the file's own
        `- ✅ **CLOSED by story 5.3.** ~~struck~~` shape (two recent examples at `:399` and `:428`).
  - [x] Annotate `:217-224` and `:287-291` as PARTLY closed, owner 5.4/5.5. Do not strike them.
  - [x] Open `## Deferred from: story-5.3` for what stays open, each with an owner — **appended at the
        end of the file**, after `## Deferred from: code review of story-5.2b` (`:900`). The file is
        chronological, not topical.
  - [x] Note in the Completion Notes (not in the file) the three `Owner: Epic 5` entries left
        untouched on purpose — `:255-259`, `:275-284`, `:295-303` (AC6).

- [x] **Task 8 — The full local gate, run WHOLE** (AC7; mirrors CI — Epic 3's retrospective recorded four
      CI-only failures from skipping exactly this)
  - [x] `cargo fmt --all` · `cargo clippy --workspace --locked --all-targets -- -D warnings` ·
        `cargo test --workspace --locked` · `cargo xtask ci`. **`--locked` everywhere** — it is what CI
        runs and `Cargo.lock` is committed.
  - [x] ⚠️ **Also run clippy the way CI runs it — `cargo clippy --workspace --locked -- -D warnings`,
        WITHOUT `--all-targets`.** That compiles the lib without `cfg(test)`, and it is the ONLY
        invocation that catches an import kept alive solely by a test module or a doc link. The
        `--all-targets` run will not. Both must be green.
  - [x] `git status` under `fixtures/` **empty**; `MANIFEST.toml` untouched.
  - [x] Report the test count as three numbers (bin + core + xtask). **Baseline measured 2026-07-28
        at `12be2fc`: 135 + 86 + 42 = 263, all green.**
  - [x] `cargo xtask ci` reporting `ℹ views-hash STALE` and exiting 0 is **expected and correct**.
        **Do NOT regenerate `architecture-views.md`** — that is a milestone task, never a story task.

- [x] **Task 9 — Docs current before push** (AC7; project rule)
  - [x] `sprint-status.yaml` — `5-3-engine-abstention-cause` and the narrative comment block.
  - [x] `docs/project-context.md` — the test count and the Epic 5 line.
  - [x] No manual, README or gh-pages change is expected: this story ships nothing a user can see.
        If that turns out to be wrong, the doc moves in the same push.

- [x] **Task 10 — Branch → PR → green CI → squash merge** (AC7). Never straight to `master`
      (`enforce_admins` is false; honouring it is on the author). `review`, not `done`, at the end of
      dev-story — **`done` is the MERGE's business** in this project's flow, confirmed by 5.1, 5.2 and
      5.2b. The `code-review` workflow's own default (setting `done` at the end of the review) is
      WRONG here.

### Review Findings (AI, 2026-07-29)

_Three parallel layers — Blind Hunter (diff only), Edge Case Hunter (diff + repo, no spec),
Acceptance Auditor (diff + spec + repo). **All three converged on the enum doc's row count**, and
TWO layers independently attacked `all()`'s witness — the Edge Case Hunter by RUNNING it. The
Acceptance Auditor reproduced SIX of the seven mutations and reports that **not one observed red set
differed from the record**, including both places where the spec's own predictions were wrong._

⚠️ **Orchestration defect, mine, recorded because it nearly cost a measurement:** two layers with
write access ran against ONE working tree. The Edge Case Hunter observed `score.rs:189` carrying
another agent's M3 mid-pass and correctly left it alone. Nothing was corrupted — the tree was
verified clean and green afterwards (135 + 90 + 42, no stray variant, `cascade.rs` md5 stable) — but
the next multi-layer review runs each writing layer in its own git worktree.

- [x] [Review][Decision] **RESOLVED 2026-07-29 — Guy's call: keep the mechanism, tell the truth
      about it, register the residue (owner 5.14).** Two candidate closures were BUILT and MEASURED
      before deciding: an `ordinal()`/`slot()` witness is green with the variant missing (91 passed
      — it closes nothing, and it was the reviewer's own first recommendation, withdrawn on
      measurement), and a single-source `macro_rules!` works but violates AC1's literal `[Self; 2]`
      return type and makes adding a variant frictionless, which is what AC1b calls a finding.
      `strum` refused as disproportionate (D47). Shipped: the true weaker doc, plus a comment at
      the witness naming the wrong repair. Full record in `deferred-work.md`.
      ~~**`all()`'s witness has a silent lazy-repair path, and it was MEASURED**~~ —
      adding a third variant and repairing ONLY the `error[E0004]` (adding a bare match arm) leaves
      `all()` returning 2 of 3 while `cargo test -p opencmdb-core` reports **90 passed**. No
      `E0308` fires, because repairing the match never forces the literal. So the guard AC1b rests
      on has a green exit. The doc correction is a patch below either way; the DECISION is whether
      to also strengthen the guard.

- [x] [Review][Patch] The enum doc says *"the **four** D13 rows that produce none"* and *"the other
      **four** rows"*, then lists exactly TWO — four rows produce a variant, two produce none
      [crates/opencmdb-core/src/identity/cascade.rs:15-19]
- [x] [Review][Patch] *"Two of its three decisions are abstentions"* is contradicted two lines later
      by *"`NoMatch` … is an **active opposition** … never an abstention"*
      [crates/opencmdb-core/src/identity/cascade.rs:17-27]
- [x] [Review][Patch] `all()`'s doc claims a new variant *"breaks this function **twice**"*; M1b and
      M1c measured the two errors as MUTUALLY EXCLUSIVE, never simultaneous — and the inline comment
      three lines below states it differently again [crates/opencmdb-core/src/identity/cascade.rs:83-90]
- [x] [Review][Patch] The variant-set test's doc claims a third variant *"makes `all()` no longer
      contain these two"* — false; `all()` still contains both
      [crates/opencmdb-core/src/identity/cascade.rs:106-110]
- [x] [Review][Patch] `gap/mod.rs`'s closing paragraph drifts from *"`Ambiguous`"* to *"a variant
      **here**"*, making it false of the three variants `reconcile` does produce
      [crates/opencmdb-core/src/gap/mod.rs:42-44]
- [x] [Review][Patch] The cross-vocabulary test pins the corpus side to one variant, so it covers 2
      of 6 pairs while its name states the general claim [crates/opencmdb-core/src/score.rs:787-798]
- [x] [Review][Patch] `other_cause`'s justification comment defends a uniqueness this same commit
      removed — the new cause-blindness loop already asserts that pair
      [crates/opencmdb-core/src/score.rs:733-740]
- [x] [Review][Patch] `Expectation::MustAbstain` went from SEVEN Rust construction sites to EIGHT
      (`score.rs:788` is new) and the "counts re-measured on the FINAL tree" block omits it — the one
      number the story's own inherited lesson #3 names literally
      [_bmad-output/implementation-artifacts/5-3-engine-abstention-cause.md, Completion Notes]
- [x] [Review][Patch] AC1's `Given` says `AbstentionCause` is one *"which `reconcile` matches
      exhaustively"*; the story's own M4 refutes it — the only value-level match is `page.rs`'s
      `cause_label`. Record as a correction to the story [story file, AC1]
- [x] [Review][Patch] The environment note says *"nine untracked entries"* and lists eleven; `git
      status --porcelain | wc -l` → 11 [story file, Completion Notes]
- [x] [Review][Patch] M5's record says *"three of the five are the new guards"* without naming which
      three, where M1–M4 all name theirs [story file, Completion Notes]
- [x] [Review][Patch] *"named in **seven** files"* is true of source files only — four more Markdown
      files name the type in the same commit [story file, Completion Notes]
- [x] [Review][Patch] The mutation line:col citations point at the MUTATED tree, not the committed
      one; a reader opening `cascade.rs:88` finds the wrong line [story file, Completion Notes]
- [x] [Review][Patch] The 4.6a closure announces *"One sentence below is now FALSE"* and then leaves
      that sentence un-struck, which is the failure mode the note itself names
      [_bmad-output/implementation-artifacts/deferred-work.md:160-184]
- [x] [Review][Patch] Both `↺ PARTLY closed` annotations sit ABOVE their target bullet and name no
      target; the file's own idiom writes *"the entry above"*
      [_bmad-output/implementation-artifacts/deferred-work.md:234-239, :310-315]
- [x] [Review][Patch] `sprint-status.yaml` claims the new register section opens *"FIVE items, each
      with a named owner"*; two carry `Owner: nobody` and `Owner: whoever needs the comparison`
      [_bmad-output/implementation-artifacts/sprint-status.yaml]
- [x] [Review][Patch] `docs/project-context.md`: the inserted 5.3 clause orphans *"That pin"*, which
      now reads as pointing at 5.3's tests [docs/project-context.md:68-70]
- [x] [Review][Patch] `CLAUDE.md` still says *"story 5.1 is `done`"* with 5.2 and 5.2b since merged —
      inherited drift from those two stories, but docs-current-before-push is a project rule and this
      push is the occasion [CLAUDE.md:7]

- [x] [Review][Defer] Two of the four new tests are carried by the type system, not by their
      assertions — their only recorded reds are `E0599`/`E0308` that fire because the test SPELLS the
      variant [crates/opencmdb-core/src/identity/cascade.rs:113, :129] — deferred, partly addressed
      by the decision above
- [x] [Review][Defer] `an_abstention_names_no_rule_whatever_its_cause` lives in `cascade.rs` but
      asserts a property of `score::Outcome::rule()`, inverting the very convention Task 5 invoked to
      justify splitting the tests [crates/opencmdb-core/src/identity/cascade.rs:129] — deferred, the
      spec bound the placement
- [x] [Review][Defer] A trap file may write `must-abstain = { cause = "OutOfPerimeter" }` and no
      validation refuses it, though the cascade has no such row
      [crates/opencmdb-core/src/trap.rs:89] — deferred, pre-existing corpus-format question
- [x] [Review][Defer] `lib.rs`'s new flat re-export has no consumer — `trap_gate.rs` imports the long
      path — while the story refuses serde and `Ord` on a no-consumer argument
      [crates/opencmdb-core/src/lib.rs:40] — deferred, pre-existing crate idiom
- [x] [Review][Defer] AC7 as drafted requires a squash merge inside a workflow that must end at
      `review`; every Epic 5 story will hit it [story file, AC7] — deferred, spec-drafting wart
- [x] [Review][Defer] CI runs no `cargo doc`, so broken intra-doc links are gated by nothing; this
      diff is clean by luck, not by mechanism [.github/workflows/ci.yml] — deferred, pre-existing

## Dev Notes

### What was measured, before the story was written

All on the committed tree at `12be2fc` (2026-07-28), so the dev re-derives nothing and a surprise
reads as a **finding**:

- **`cargo test --workspace` → 135 (bin) + 86 (core) + 42 (xtask) = 263, zero failures.** That is the
  baseline this story moves.
- **`Outcome::Abstained` has SIX construction sites, and every one is in a test module**:
  `score.rs:628-631`, `score.rs:717-719`, `trap_gate.rs:450`, `:508`, `:652`, `:658`. Production code
  constructs an abstention **nowhere** — which is the whole reason retyping it is cheap today and
  will not be after 5.5.
- **`Expectation::MustAbstain` has three committed occurrences in the corpus** (`example.toml:44`,
  `hostname-absence.toml:47`, `shared-hardware-vm.toml:46`, all three sha256-locked at
  `fixtures/MANIFEST.toml:36`, `:75`, `:169`) and **SEVEN Rust construction sites**:
  `trap.rs:416`/`:561`/`:630`, `fixtures.rs:3536`/`:3982`/`:4245`, and — the one that is easy to miss
  because it sits in the file being edited — **`score.rs:608`** (`fn must_abstain()`, feeding nine cell
  assertions). `trap_gate.rs` constructs none of them: its `must-abstain` side is TOML text (`:500`,
  `:639`). **All seven keep `AbstentionCause` and none of them changes.** If a diff touches one, it is
  a mistake, not a consequence — and `score.rs:608` is why the import at `:34` cannot simply be
  swapped.
- **`page.rs:112-119`'s `cause_label` match is exhaustive with no `_` arm** — the only value-level
  match on `AbstentionCause` in the workspace — and `crates/opencmdb-bin/locales/app.yml` carries
  exactly three `cause.*` keys (`:48`, `:51`, `:54`), each with `en:` and `fr:`. M4 was **run** on the
  committed tree: one `error[E0004]`, at `page.rs:114:11`, and nothing else broke. That is what makes
  M4 a real red rather than a story.
- **`trap_gate.rs` will have ZERO uses of `gap::AbstentionCause` after the retype.** All five
  occurrences today (`:389` import, `:451`, `:509`, `:653`, `:659`) are on the `Outcome` side; the two
  `Column::MustAbstain` hits at `:537`/`:666` are `score::Column`, imported separately at `:390`. The
  import is replaced, not doubled.
- **`crates/opencmdb-core/src/identity/` does not exist.** `src/` holds `clock.rs`, `connector/`,
  `gap/`, `lib.rs`, `observation/`, `repo/`, `score.rs`, `testing/`, `trap.rs`. This story creates
  the subtree the architecture's source tree has been describing since planning.
- **File-size gate headroom is ample.** `score.rs` is 1231 lines total with the first `#[cfg(test)]`
  at `:581` → **580 code lines**, ceiling 2000. `gap/mod.rs`: 128 code lines. The new `cascade.rs`
  will be far under. The gate counts CODE lines only, to the first `#[cfg(test)]`.

### The one thing that would make this story fail review

**Building the cascade.** The temptation is real: the type is small, the algebra is written out in
architecture.md:969-974, and it looks like ten minutes of work. It is story **5.4**'s (the `Verdict`
enum, `Decision`, `ruleset_version`) and **5.5**'s (the join). Writing a `fn decide(verdicts) ->
Decision` here would:

1. produce a `Decision` type 5.4 has to define, repeating the exact mistake `score.rs:46-47` was
   careful NOT to make (*"the architecture reserves that name… taking it here would squat a type
   Epic 5 has to define"*);
2. give this story an untestable surface — there is no rule, so no verdict set is producible from
   anything but a hand-built literal;
3. and bend the vocabulary to the first algebra written, which is the inversion of Epic 4's rule.

**The deliverable is a type, its documentation, and the tests that pin its two claims.** That is the
whole story.

### Why the cause type is a SEPARATE enum and not a widened `AbstentionCause`

Story 4.6a named the two branches and left the choice to Epic 5 (`deferred-work.md:166-167`). The
branch taken is the second, for reasons that should survive in the code's doc, not only here:

- **The two enums answer different questions.** `AbstentionCause` says why *reconciling a declared
  field against observations* did not conclude — a perimeter question, a presence question, a
  disagreement between two observed values. `IdentityAbstentionCause` says why *the identity cascade*
  did not conclude — a question about a verdict set. `OutOfPerimeter` is meaningless to the cascade,
  and `Ambiguous` is meaningless to `reconcile`.
- **Widening costs more than it looks.** `AbstentionCause` is `Serialize`/`Deserialize` and the
  committed corpus depends on its PascalCase spelling (`trap.rs:584-591` exists to say so). A variant
  added to it is a variant the corpus format can express, that `cause_label` must label, that two
  locales must translate — for something `reconcile` can never produce. AC4's M4 measures exactly
  that.
- **The asymmetry is safe because scoring is cause-blind, and that is structural, not a promise.**
  `score`'s 3×3 matches `Outcome::Abstained { .. }`; the payload is unreachable from it. `run_trap`
  only compares where both `rule()`s are `Some`, and an abstention's is `None` **by type**. Two
  different types on the two sides of a trap therefore cannot make the gate asymmetric — there is no
  comparison to go asymmetric. AC3's cross-vocabulary test is that sentence, executable.

### The vocabulary the corpus writes vs. the vocabulary the engine will answer

Worth stating plainly, because it looks wrong at first glance and a reviewer will ask:

The three committed `must-abstain` traps say `cause = "NoObservedValue"`. When the engine answers
one, it will say `AbsenceOfProof` (or `Ambiguous`). **Those never meet.** The corpus's cause is the
trap author's note about the SHAPE of the case; the engine's cause is what the cascade concluded.
D18's truth table is about the verdict alone, and `score` implements exactly that. If a future story
ever wants to compare them, it needs a mapping and a decision — and that is a new story, not a silent
`impl From`. **Do not write a `From`/`Into` bridge between the two types in this story.**

### What this touches, and what it must not break

- **`crates/opencmdb-core/src/identity/mod.rs`** (NEW) · **`identity/cascade.rs`** (NEW).
- **`crates/opencmdb-core/src/lib.rs`** (UPDATE) — one `pub mod`, one `pub use`. Note `lib.rs:8-10`'s
  module doc already promises *"the identity engine, the verdict algebra… live under here"* and
  `:11` already says the crate *"asserts nothing about identity yet"*; this story is the first
  instalment and the doc needs no correction, but read it before adding another sentence that would
  then be an over-claim. `:38`'s flat `pub use gap::{AbstentionCause, …}` is what makes the naming
  question an `error[E0252]` rather than a preference (AC1).
- **`crates/opencmdb-core/src/score.rs`** (UPDATE) — the field type at `:69`, the import at `:34`
  (replaced, with a test-module import added), the doc blocks at `:49-61` and `:613-617`, two test
  constructors, and the new tests of Task 5.
  *Must be preserved:* the exhaustive 3×3 with no `_` arm; `run_trap`'s positive gate
  (`!= Score::Pass`, not `== Score::Fail`, `:229`); `Outcome::rule()` returning `None` for an
  abstention **by construction**; `comparable_fields`' `..`-free destructure (`:448-460`);
  `must_abstain()` (`:607-611`) on `AbstentionCause`; and the *intent* of the deliberate mismatch
  between test expectations and outcomes — its doc at `:613-617` is rewritten because the mismatch
  becomes a type-level fact, but the fixtures themselves must still NOT match.
- **`crates/opencmdb-core/src/trap.rs`** (UPDATE, **doc only**) — `:81`'s variant doc. `:82`'s
  declaration is byte-unchanged (AC2), and the two pins at `:569-579` / `:584-591` are not modified.
- **`crates/opencmdb-core/src/gap/mod.rs`** (UPDATE, **doc only**) — the AC3 counterpart paragraph.
  The variant list, `reconcile` and its three `abstain` calls do not move.
- **`crates/opencmdb-bin/src/trap_gate.rs`** (UPDATE, tests only) — four constructors and the import.
- **`crates/opencmdb-bin/src/page.rs`, `locales/app.yml`** — **NOT touched.** Displaying the new cause
  is story **5.14** (*"abstention is displayed, counted and grouped by cause — and never as a
  reproach"*). Adding a `cause_label` arm or a locale key here builds a UI for a value nothing
  produces.
- **`crates/opencmdb-bin/src/fixtures.rs`** — **NOT touched.** Its three `AbstentionCause` uses
  (`:3536`, `:3982`, `:4245`) are all inside `Expectation::MustAbstain`.
  **`fixture_connector.rs`** — **NOT touched**, and it names `AbstentionCause` zero times, so it
  cannot be affected either way.
- **`xtask/`** — **NOT touched.** No gate changes. (For reassurance: the retired-vocabulary gate's
  denylist is `pending_accept`, `reverting`, `accept_as_declared`, `accept-as-declared` plus the
  `ignore`/`exclude` co-presence pair, `xtask/src/main.rs:366-382` — nothing this story writes is
  near it.)
- **Under `fixtures/`: NOTHING.**
- **`deferred-work.md`** (UPDATE, append-only) · **`sprint-status.yaml`**, **`docs/project-context.md`**
  (UPDATE).

### What STOP means, procedurally

If a step appears to require re-authoring a committed artefact or re-hashing `MANIFEST.toml`: **stop,
do not edit the artefact.** Record what was attempted, what the tree says, and the exact command that
shows the conflict; report it as a finding in the Completion Notes, **open a GitHub issue on
`guycorbaz/opencmdb`** (CLAUDE.md: issues are the single source of truth for work items outside the
story flow — a finding that lives only in a story file is not tracked), and raise it with Guy. The corpus
is the oracle for the whole epic; a story that quietly re-authors it to fit its own code is the
"bending the metric to the engine" failure Epic 4 was built to prevent.

### Inherited from stories 5.1, 5.2 and 5.2b — read before writing a doc comment

1. **A check that its own commit falsifies is worse than no check.** 5.1 cited a `grep` its own diff
   broke; 5.2 replaced a false doc sentence with another one the same commit falsified. This story
   rewrites a doc block (AC3) — **re-read it against the final tree after the last edit.**
2. **An inventory in a doc comment has no guard behind it.** Say what THIS type is and what THIS test
   proves; let the register count what is open.
3. **A count in a doc is a claim.** 5.2b's review found *"all ELEVEN committed traps"* against a
   corpus of 24, in four documents. If this story writes "six call sites", "seven `MustAbstain`
   sites" or "three committed `must-abstain` traps", it **re-counts them on the final tree** — the
   numbers in Dev Notes are the `12be2fc` baseline, not a post-condition.
4. **A red set is a count too.** Task 6's predicted reds are measurements taken before the change;
   the record must be what was OBSERVED. Reporting one red where five fired understates the guard
   exactly as an over-claim overstates it, and both are the same defect: a claim that does not match
   its verification.
5. **Name the test behind every claim.** The temptation here is *"the identity engine now has its own
   abstention vocabulary"*. What will hold is: *"`Outcome::Abstained` carries
   `IdentityAbstentionCause`; two variants are named by tests; nothing compares the two vocabularies
   and a test says so; no engine produces one yet."*

### House rules that bind this story

- **Prove-to-red is not optional** (story 1.3): a guard is observed failing before it passes, and the
  mutation is recorded. Task 6 names five.
- **Document every public item** — struct, enum, **field**, **variant**, function — in idiomatic
  rustdoc prose (never `@param`/`@return`). **A doc comment must be TRUE**; prefer the weaker true
  sentence. `opencmdb-core` does not yet carry `#![deny(missing_docs)]` (`lib.rs:14-18` says why);
  that is not a licence to skip one.
- **DRY, with deliberate redundancy protected.** The cause-blindness test loops over `all()` — one
  source of truth for behaviour. Do NOT collapse `all()` itself into a literal array: the exhaustive
  `match` is the mechanism that makes a new variant a compile error.
- **File size:** ≤ 2000 CODE lines, tests excluded, counted to the first `#[cfg(test)]`.
- **Dependency frontier (D47):** `opencmdb-core` must not gain `anyhow`, `axum`, `sqlx` or `askama`.
  Nothing here needs one; `cargo xtask ci` checks it.
- **`DATABASE_URL` is usually unset locally** and the MariaDB-backed tests `return` early — a green
  suite says nothing about the database. Irrelevant here; do not cite it as evidence of anything.
- **Known local flakiness (issue #38):** unexplained non-determinism, 8 failures across 5 runs on
  stories 4.15/4.17. The "Synology Drive" explanation is **REFUTED by measurement — do not re-adopt
  it.** If something reds unexpectedly, re-run and check `git status` before diagnosing.

### Testing standards

Tests live inline in a trailing `#[cfg(test)] mod tests` (D56b, one per file). Test names are
sentences that say what they prove — `the_two_abstention_vocabularies_are_never_compared` reads at a
CI log; `test_cause` does not. Assertion messages name the variant and the cell. The identity engine's
unit tests are pure — *"the engine is a pure function: a `FixtureConnector` and nothing else — no
database"* [architecture.md:3302], and *"the engine never touches the clock (D19)"*
[architecture.md:3364]. Nothing in this story needs either, so neither claim is load-bearing here;
they are quoted rather than paraphrased because a paraphrase that widens them ("no I/O") would be a
claim with no source.

### Project Structure Notes

`identity/` under `opencmdb-core/src/` is the architecture's own layout [architecture.md:3365-3373],
which also names `index.rs` (`:3367`), `blocking.rs` (`:3368`), `field_decision/` (`:3370-3372`) and
`migration.rs` (`:3373`) — **none of which this story creates**. `mod.rs` (`:3366`) is described as
holding `IdentityError`; there is no fallible
operation yet, so it holds the module doc and the `pub mod cascade;` declaration and says so.
D54's point is worth carrying into the module doc: the FOLDER is not the frontier — visibility is —
so `identity/` earns its existence when `pub(in crate::identity)` starts meaning something (5.9+),
and claiming today that the tidying implements AC-8b would be theatre by D54's own words.

### References

- Story source: [Source: _bmad-output/planning-artifacts/epics.md#Story 5.3] (epics.md:1401-1421);
  the epic preamble and build order, epics.md:1309-1317; the arbitration recorded at decomposition
  (*"the identity engine gets its OWN abstention cause, with `Expectation` and the sha256-locked
  corpus untouched"*), epics.md frontmatter `resumePoint` and epics.md:1311.
- D13 — the matching engine, the verdict algebra and its six-row table, the refusal of
  `rule -> confidence: f64`, structural facts never scored:
  [Source: _bmad-output/planning-artifacts/architecture.md#D13] (`:929`, table at `:964-974`).
  Start any further question at the Decision Index near the top of that file (F56).
- D16 — why abstention must not become a semantic dustbin: [architecture.md:1112-1115].
- D18 — the gate, its three columns, and *"an engine that abstains on everything… gets demolished by
  the middle column"*; D19 — the oracle is the author, and the metric comes before the engine.
- FR16 / FR16b — abstention as a first-class, displayed, counted, grouped-by-cause outcome:
  [Source: _bmad-output/planning-artifacts/prd.md] `:896-897`.
- Register entries this story acts on: [Source:
  _bmad-output/implementation-artifacts/deferred-work.md] `:160-167` (closed), `:217-224` and
  `:287-291` (annotated, owner 5.4/5.5). The closure idiom is the file's own
  `✅ **CLOSED by story X.** ~~struck~~` shape — see `:399` and `:428` for two recent examples.
- The three immediately previous stories, for the prove-to-red record shape and the review lessons:
  `5-1-corpus-pins-obs-id-binding.md`, `5-2-privacy-floor-reaches-the-bytes.md`,
  `5-2b-four-families-byte-pins.md`.

### Git intelligence

Last five commits: `12be2fc` (5.2b bookkeeping, PR #47) · `fe6a19e` (**story 5.2b**, PR #46) ·
`95d4d69` (5.2 bookkeeping, #45) · `4d2044b` (**story 5.2**, #44) · `e846836` (5.1 bookkeeping, #42).

All three code stories touched `crates/opencmdb-bin/src/fixtures.rs` and the corpus; **none of them
touched `score.rs`, `gap/mod.rs` or `trap.rs`.** This story is the first Epic 5 work in
`opencmdb-core`'s engine surface, so the "what did the previous story leave half-done here" risk is
low — the inheritance is procedural (prove-to-red discipline, the doc-truth lesson, the
register idiom), not structural. Every one of those commits went branch → PR → green CI → squash
merge; hold the same. Do not quote a running "N/N green" tally — say "green CI on every PR", which is
reconstructible.

### Toolchain

No new dependency, no version to research. Rust 1.96+, edition 2024, `Cargo.lock` committed, every
build `--locked`. **Never invent a version** — pin from the real `Cargo.lock` if one is ever needed.

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context) — `claude-opus-5[1m]`, via `bmad-dev-story`.

### Debug Log References

Every command below was RUN on the tree it names; nothing here is predicted.

**Baseline, re-measured at the start on `12be2fc`:** `cargo test --workspace --locked` →
135 (bin) + 86 (core) + 42 (xtask) = **263**, zero failures. Matches the story's Dev Notes exactly.

**Final tree:** 135 + **90** + 42 = **267**. The four new tests are all in `opencmdb-core`: two in
`identity/cascade.rs`, two in `score.rs`. `bin` and `xtask` counts are unchanged, which is what a
story that adds no bin-side guard should look like.

**The gate, run whole and in this order:**

| command | result |
|---|---|
| `cargo fmt --all` | clean |
| `cargo clippy --workspace --locked --all-targets -- -D warnings` | clean |
| `cargo clippy --workspace --locked -- -D warnings` (**the CI form**, no `--all-targets`) | clean |
| `cargo test --workspace --locked` | 135 + 90 + 42, zero failures |
| `cargo xtask ci` | all gates green; `ℹ views-hash STALE`, exit 0 — expected, NOT regenerated |
| `git status --short fixtures/` | **empty** |
| `git status --short fixtures/MANIFEST.toml` | **empty** |

`cargo xtask ci` reported `file-size 22 file(s) under 2000 code lines (largest: 884)`. Measured
directly **after the code review's patches**: `score.rs` 586 code lines (580 before the story),
`identity/cascade.rs` **121** (101 before the patches — the doc grew), `gap/mod.rs` **151** (128
before the story) — the ceiling is not in view.

**`cargo doc --workspace --no-deps --locked`** was also run, because clippy does not check
intra-doc links and this story writes many. It emits **three** warnings, and all three are
PRE-EXISTING: `score.rs:429`'s `break[ing]` (an unescaped `[`, unrelated), `is_unchanged` linking to
the private `comparable_fields`, and one in `bin`. **None of the new full-path links warns** — that
is the measurement behind AC3's "write them as full paths", not an assumption. Re-run after the code
review's patches: still exactly those three, none in new or edited docs. ⚠️ `grep -c "^warning:"`
returns **5**, not 3 — two of the lines are rustdoc's own `generated N warnings` summaries. Counting
those as findings is a trap this note exists to disarm.

### Completion Notes List

**What was built, in the weaker true sentence.** `Outcome::Abstained` carries
`IdentityAbstentionCause` (`identity/cascade.rs`), whose two variants are `Ambiguous` and
`AbsenceOfProof`, each traced to a row of D13's table. Four tests pin its claims. Nothing compares
the two abstention vocabularies and a test says so. **No engine produces one yet** — there is no
cascade, no rule, no `Decision`, no join. `Expectation::MustAbstain` still carries
`gap::AbstentionCause`, `reconcile`'s enum is byte-unchanged, and no corpus artefact moved.

**Counts re-measured on the FINAL tree** (the story's Dev Notes numbers are the `12be2fc` baseline,
not a post-condition — a count in a doc is a claim):

- **Three** committed `must-abstain` expectations write `cause = "NoObservedValue"`:
  `example.toml:44`, `hostname-absence.toml:47`, `shared-hardware-vm.toml:46`. Unchanged.
- **`Outcome::Abstained` now has NINE construction sites, not six** — this story added three
  (two in `score.rs`, one in `cascade.rs`, all inside the new loops). **All nine are still
  in test modules; production code constructs an abstention nowhere.** M5 measured that
  independently: reverting the field type broke only the `lib test` target, never the lib.
- **`Expectation::MustAbstain` went from SEVEN Rust construction sites to EIGHT.** The eighth is
  this story's own `the_two_abstention_vocabularies_are_never_compared`, in `score.rs`. All eight
  keep `AbstentionCause`, so Dev Notes' *"all seven keep `AbstentionCause` and none of them
  changes"* still holds of the seven it counted — but the NUMBER is stale, and it is the one the
  story's own inherited lesson #3 names literally. **Caught by the code review, not here**, which is
  the honest record: the same re-count was done for the two numbers above and skipped for this one.
- `IdentityAbstentionCause` is named in **seven SOURCE files**: `lib.rs`, `identity/mod.rs`,
  `identity/cascade.rs`, `score.rs`, `gap/mod.rs`, `trap.rs` (doc only) and `trap_gate.rs`. Four
  Markdown files in this same commit name it too — "seven" is a claim about `crates/`, and saying so
  is the difference between a count and a true count.

**ONE CORRECTION to the story, and it is inherited rather than introduced.** AC1's `Given` describes
`AbstentionCause` as one *"which `reconcile` matches exhaustively"*. **It does not.** `reconcile`
only CONSTRUCTS a cause (three `abstain(...)` calls in `gap/mod.rs`); the sole value-level `match` on
the enum in the whole workspace is `page.rs`'s `cause_label`, which is exactly what M4 measures — one
`error[E0004]`, there and nowhere else. So this story contains its own refutation, and the false
clause traces back to the story-4.6a register bullet, where it is now struck. The AC's *conclusion*
is unaffected: widening the enum still costs a label and two locale strings.

**The mutations — SEVEN run, not five.** Each was run, its complete observed red set recorded,
restored, and `git status` checked before the next. Two of the seven are additions the story did not
ask for, and the reason is given below.

⚠️ **Every `file:line:col` below is a coordinate on the MUTATED tree, not on the committed one** —
that is what rustc printed, and a red record that "cleans up" its coordinates is no longer the
observation. A reader opening the committed `cascade.rs` at those lines finds neighbouring code.

- **M1 — delete the `Ambiguous` variant.** Six `error[E0599]` ("no variant … named `Ambiguous`")
  across **four** distinct sites: `cascade.rs:88:26` (the `all()` literal) and `cascade.rs:93:23`
  (the witness match), each reported twice (lib **and** lib test), plus `cascade.rs:115:52` (the new
  variant-set test) and `score.rs:645:45` (`abstained()`), lib test only.
  ⚠️ **Two differences from the story's prediction, and both are findings.** (a) The predicted
  `error[E0308]` on the literal did **not** occur — deleting a variant makes the literal `E0599`
  before any length check; `E0308` is what ADDING one to the literal produces, which M1 cannot show
  (see M1c). (b) `trap_gate.rs`'s four retyped sites did **not** red: `opencmdb-core` fails to
  compile, so `opencmdb-bin` is never compiled at all. The new cross-vocabulary test also did not
  red, correctly — it reaches the variants through `all()` and never spells `Ambiguous`.
- **M1b — add a third variant (`Unmodelled`) to the enum.** Exactly one `error[E0004]`
  ("non-exhaustive patterns: `IdentityAbstentionCause::Unmodelled` not covered") at
  `cascade.rs:95:19`, the witness match, reported for lib and lib test. This is AC1b's *"a third
  variant is a finding"* made mechanical. **Added because M1 does not prove it**: M1 proves the
  witness catches a REMOVED variant, and the interesting direction is the added one.
- **M1c — the same third variant, also added to the `all()` literal and the match.**
  `error[E0308]` at `cascade.rs:101:9`, rustc printing *"expected `[IdentityAbstentionCause; 2]`
  because of return type"*. **Added because `all()`'s doc comment asserts two mechanisms and M1/M1b
  measure only the first.** A doc comment must be TRUE; the second sentence would otherwise have
  been a prediction shipped as a statement — the exact defect the three previous stories' reviews
  kept catching.
- **M2 — `score.rs`: `(MustAbstain, Abstained)` → `Score::Fail`.** **EIGHT reds**, run with
  `--no-fail-fast` (without it cargo stops at the first crate and reports two): in `opencmdb-bin`,
  `a_trap_with_no_answer_is_discovered_but_not_scored` and
  `a_correct_answer_stays_out_of_failures_while_a_wrong_one_enters`; in `opencmdb-core`,
  `must_abstain_fails_on_any_decision`,
  `an_engine_that_abstains_on_everything_is_demolished_by_the_middle_column`,
  `an_abstention_on_a_passing_cell_is_never_a_wrong_rule`,
  `scoring_ignores_the_rule_because_the_trap_runner_owns_it`, and **the two this story earns** —
  `scoring_is_blind_to_the_abstention_cause_whatever_it_is` and
  `the_two_abstention_vocabularies_are_never_compared`.
  Difference from the prediction: the story listed seven and named the bin test only descriptively
  ("its mixed-corpus test" = `a_correct_answer_stays_out_of_failures_while_a_wrong_one_enters`); it
  did not foresee `scoring_ignores_the_rule_because_the_trap_runner_owns_it`, whose last assertion
  is exactly this cell. **Eight observed, seven predicted.**
- **M3 — `score.rs`: `(MustNotMerge, Abstained)` → `Score::Fail`** (D18's load-bearing cell).
  **FIVE reds, all in `opencmdb-core`, and the observed set is exactly the predicted one**:
  `must_not_merge_fails_only_on_a_merge`,
  `an_engine_that_abstains_on_everything_is_demolished_by_the_middle_column`,
  `a_column_that_never_ran_is_distinguishable_from_one_that_passed`,
  `an_abstention_on_a_passing_cell_is_never_a_wrong_rule`, plus the new
  `scoring_is_blind_to_the_abstention_cause_whatever_it_is`. The cross-vocabulary test does NOT red
  here and should not — it only exercises the `must-abstain` column.
- **M4 — add `Ambiguous` to `gap::AbstentionCause`** (AC4's evidence). **Exactly one**
  `error[E0004]`, verbatim: *"non-exhaustive patterns: `AbstentionCause::Ambiguous` not covered"* at
  **`crates/opencmdb-bin/src/page.rs:114:11`**, pointing at `match cause`. **Nothing else in the
  workspace breaks, and that was measured rather than assumed**: under the mutation
  `cargo test -p opencmdb-core` still reports **90 passed** (serde round-trips and
  `the_committed_abstention_vocabulary_is_pinned` included) and `cargo test -p xtask` **42 passed**.
  So widening the reconciliation enum costs a user-facing label plus two locale strings, for a
  variant `reconcile` can never produce — which is the argument for a separate type, now measured.
- **M5 — revert `Outcome::Abstained`'s field to `gap::AbstentionCause`.** Five `error[E0308]`, all
  in the `lib test` target: `cascade.rs:132:38`, `score.rs:645:20`, `:740:20`, `:760:50`,
  `:792:52` — **three of the five are the new guards, and they are**
  `an_abstention_names_no_rule_whatever_its_cause` (`cascade.rs:132`),
  `scoring_is_blind_to_the_abstention_cause_whatever_it_is` (`score.rs:760`) and
  `the_two_abstention_vocabularies_are_never_compared` (`score.rs:792`); the other two are the
  pre-existing helpers `abstained()` (`:645`) and `other_cause` (`:740`). Plus `warning: unused import:
  crate::identity::cascade::IdentityAbstentionCause` at `score.rs:34:5`. **The lib itself still
  compiles**, which measures the story's premise that production constructs no abstention.

**One thing the story asked for that I did NOT do as written, with the reason.** AC1b's variant docs
were to cite architecture rows "the row, not a paraphrase" — done — but `Ambiguous`'s doc also
had to record the possible three-way split. I put the *decision* in the register (a named owner,
story 5.14) and only a POINTER in the doc, per Task 2's rule that a doc must not carry an inventory
the register already counts.

**Three `Owner: Epic 5` register entries were deliberately left untouched** (AC6): lattice
monotonicity (`deferred-work.md`, `## Deferred from: story-4.6c`), the firing-rule/evidence contract
and the `RuleId` whitespace/case normalization in `run_trap` (both under the 4.7a sections). None
concerns the abstention vocabulary and each needs a producing engine (5.4–5.7). Saying so is a scope
statement; silence would not be.

**Environment observation, recorded as a symptom and NOT as a cause.** `git status` at the repo root
lists **eleven** untracked entries (`git status --porcelain | wc -l` → 11; this note said "nine"
while listing eleven until the code review counted them) — `.bashrc`, `.bash_profile`, `.profile`, `.zshrc`, `.zprofile`,
`.gitconfig`, `.gitmodules`, `.idea`, `.mcp.json`, `.ripgreprc`, `.vscode` — which `ls -l` shows are
**character devices** (`crw-rw-rw- root 1, 3` = `/dev/null`), all dated 2026-07-28 09:30 and owned by
`nobody:nogroup`. They are agent-sandbox mounts, not repository content; nothing in this story
created or committed them, and `git add` was used per path, never `-A`. **No claim is made that they
relate to issue #38** — that would be a cause without a check, which this project bans. They are
recorded because they make a bare `git status` misread as a dirty tree.

**Nothing under `fixtures/` moved**, so the STOP procedure was never entered and no issue was opened.

**Task 10, honestly:** branch `story-5.3-engine-abstention-cause` → **PR #48** → **CI green in
1m00s** (run 30389174367). The squash merge is NOT done and is not dev-story's to do: this project's
flow puts `code-review` between here and the merge, and `done` is the MERGE's business — established
by 5.1 and confirmed by 5.2 and 5.2b. Status is `review`.

### File List

- `crates/opencmdb-core/src/identity/mod.rs` — **NEW**. Module doc + `pub mod cascade;`.
- `crates/opencmdb-core/src/identity/cascade.rs` — **NEW**. `IdentityAbstentionCause`, `all()`, and
  two tests.
- `crates/opencmdb-core/src/lib.rs` — `pub mod identity;` and the flat re-export.
- `crates/opencmdb-core/src/score.rs` — the field type at `Outcome::Abstained`, the import, the
  rewritten `Outcome` doc block, the rewritten test-fixture doc, two retyped test constructors, the
  test-module import of `gap::AbstentionCause`, and two new tests.
- `crates/opencmdb-core/src/gap/mod.rs` — **doc only**: the counterpart paragraph on
  `AbstentionCause`.
- `crates/opencmdb-core/src/trap.rs` — **doc only**: `Expectation::MustAbstain`'s variant doc. The
  declaration is byte-unchanged.
- `crates/opencmdb-bin/src/trap_gate.rs` — **tests only**: the import and four constructors.
- `_bmad-output/implementation-artifacts/deferred-work.md` — one entry closed, two annotated partly
  closed, one new section appended at the end.
- `_bmad-output/implementation-artifacts/sprint-status.yaml`, `docs/project-context.md`,
  and this story file.

## Change Log

- 2026-07-28 — Story created (`bmad-create-story`). Baseline measured on `12be2fc`: 263 tests
  (135 + 86 + 42), zero failures; six `Outcome::Abstained` call sites, all in test modules; three
  committed `must-abstain` traps; `identity/` does not exist. Status `ready-for-dev`.
- 2026-07-28 — **Validated** (`create-story validate`, two fresh-context agents: fact-check +
  gap-hunt, per the Epic 4 retrospective). 28 findings applied. The load-bearing ones, all measured
  rather than argued:
  - **Both import instructions were inverted.** `trap_gate.rs:389` must be *replaced* (zero remaining
    uses after the retype, so keeping it is `unused_imports` under `-D warnings`); `score.rs:34` must
    be replaced *and* re-added inside `mod tests`, because a **seventh** `MustAbstain` site
    (`score.rs:608`) still needs it and CI's clippy compiles the lib without `cfg(test)`.
  - **AC1's naming evidence did not exist** — `Expectation` never appears in `trap_gate.rs`. Replaced
    with two checkable claims: `lib.rs:38`'s flat re-export makes a homonym `error[E0252]`, and
    `score.rs`'s test module is the real co-presence.
  - **`all()` as specified was not implementable**; the literal + witness-match idiom is now written
    out, with the length pinned at `2` as a second guard.
  - **AC5's "SOLE red" was unsatisfiable** and self-contradictory; M2/M3 red **five or more** tests
    each. AC5 now requires the complete observed set.
  - **Per-site variants are now bound**: retyping `abstained()` and `other_cause` to the same variant
    would have silently voided an existing sub-claim while staying green.
  - Doc links must be full paths (an import kept alive by a `///` link is an unused import —
    measured), two further doc blocks (`trap.rs:81`, `score.rs:613-617`) become false and are added to
    the rewrite list, `#[non_exhaustive]` is decided (no, it would destroy the `E0004` mechanism),
    AC7 makes the gate/docs/flow auditable, and seven line-range citations were corrected (M4 reds at
    `page.rs:114:11`, not `:112`).
  Verified sound and unchanged: the 263-test baseline, the six `Outcome::Abstained` sites, the three
  sha256-locked traps, `score`'s structural cause-blindness, no hidden `Outcome` consumer, no
  `dead_code` risk on `all()`, and the scope (all four epics.md clauses delivered, nothing squatted
  from 5.4/5.5/5.9/5.14). Status stays `ready-for-dev`. **Next: `dev-story`.**
- 2026-07-28 — **Implemented** (`bmad-dev-story`, branch `story-5.3-engine-abstention-cause`).
  `IdentityAbstentionCause` lands in the new `identity/cascade.rs` with two variants and `all()`;
  `Outcome::Abstained` carries it; `Expectation::MustAbstain` and `gap::AbstentionCause` are
  unchanged in substance (two doc-only edits). 263 → **267** tests (135 bin + **90** core + 42
  xtask); all four gates green, both clippy forms green, `fixtures/` untouched.
  **SEVEN mutations run**, not the five specified: `all()`'s doc asserts two mechanisms and the
  five could only measure one, so M1b (a third variant → `E0004`) and M1c (widening the literal →
  `E0308`) were added rather than shipping half the sentence as a prediction.
  Two prediction/observation differences recorded rather than smoothed: **M1 produces `E0599`, not
  the predicted `E0308`, and never reaches `trap_gate.rs`** (core fails first, so bin is never
  compiled); **M2 reds EIGHT tests, not seven** — the story missed
  `scoring_ignores_the_rule_because_the_trap_runner_owns_it`, whose last assertion is that very
  cell. M3's five-red set matched the prediction exactly. M4 reproduced its predicted verbatim
  error at `page.rs:114:11` and — measured, not assumed — left core's 90 and xtask's 42 green.
  One count in the story's Dev Notes is now stale by this story's own commit and is corrected in
  the Completion Notes: `Outcome::Abstained` has **nine** construction sites, not six. All nine are
  still in test modules. Status → `review`.
- 2026-07-29 — **Code review held** (`bmad-code-review`, three parallel layers). 1 decision + 18
  patches applied + 6 defers + 3 dismissed. **No behavioural defect was found in any layer**; what
  fell was documentation truth, in a story built to guard it. The Acceptance Auditor reproduced SIX
  of the seven mutations and **not one observed red set differed from the record**.
  The decision, and the only finding that touched a mechanism: **`all()`'s witness stops the build
  on a new variant but does not force the variant into the list** — measured by RUNNING the lazy
  repair (bare match arm → 90 tests green, `all()` returning 2 of 3). Guy's call: keep the
  mechanism, state its limit truthfully, register the residue with owner 5.14. Two closures were
  built and measured first and both rejected — an `ordinal()` witness is green with the variant
  missing (91 passed, so it closes nothing; it was the reviewer's own first recommendation,
  withdrawn on measurement), and a single-source `macro_rules!` works but violates AC1's literal
  `[Self; 2]` and makes adding a variant frictionless, which AC1b calls a finding.
  All three layers converged on a false row count in the enum's guard paragraph (*"the four D13
  rows that produce none"* — it is two), inherited from an AC1b that contradicts itself four lines
  apart. Also patched: `all()`'s doc claiming a new variant breaks it *"twice"* when the two errors
  are alternatives; the variant-set test's doc asserting a guard it does not have; a re-count the
  story pre-named and skipped (`Expectation::MustAbstain` 7 → 8); AC1's `Given` clause, refuted by
  the story's own M4 and traced back to the 4.6a register bullet, now struck there with a second
  false clause beside it; the cross-vocabulary test widened from 2 of 6 pairs to all 6 and
  **re-proven red**; and eight smaller count/antecedent/placement defects across four documents.
  Test count UNCHANGED at 267 — the widened test is still one test — so the number says nothing
  about the patches; what says something is the re-run red set. `CLAUDE.md`'s Epic 5 line, stale
  since 5.2 and 5.2b merged, was corrected here rather than deferred. Status stays `review`:
  **`done` is the MERGE's business**, and this workflow's own default of setting `done` at the end
  of a review is wrong for this project.
