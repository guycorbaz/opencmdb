# Story 4.8: Open the reality-debt register

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want a register of what the trap corpus does NOT cover, opened WITH the corpus itself,
so that the gate's honest limit is written down rather than discovered by a user.

## Acceptance Criteria

1. **Given** the trap corpus, **when** the register is opened, **then** it states the limit in
   **D18's own words** — the VERBATIM sentence, attributed to D18 and kept in its first person:
   *"a trap suite proves nothing about what I failed to imagine. At v0.1 the gate is weak and honest
   rather than strong and false."* (`architecture.md:1259-1260`). The register does not merely
   exist, it CARRIES that sentence unaltered, so the gate's weakness is admitted on the record and
   not softened. ⚠️ **Reproduce the architecture string, not this epic's paraphrase:** `epics.md:1081`
   and casual retellings say *"what was **not** imagined"* — that is a paraphrase; the AC is *"in
   D18's own words"*, so the string to commit is *"what **I failed to** imagine"*, quoted as D18's,
   not silently third-personed.
2. **Given** a real-world case the corpus cannot produce, **when** it is met, **then** the register
   defines HOW it is recorded — a named case, **its SOURCE**, and the D18 column(s) it will assert —
   and states plainly that the register is the QUEUE from which trap #51 and beyond are drawn. At
   v0.1 the register opens with ZERO case entries and its format only: no Tier 2 run has happened, so
   inventing a "discovered" case would be the dishonesty this story exists to prevent (the same
   "guard before the engine" discipline as 4.6/4.7 — the register is opened before it has content).
3. **And** the register NAMES Tier 2 (bulk observability, ~300 hosts / 36 subnets, published per
   release, **blocking nothing**) as the ONLY discovery mechanism for the unimagined — *Tier 2 is
   Tier 1's trap factory; the gate only grows by proof; the day the bulk drops a cluster Tier 1 did
   not foresee, that cluster becomes trap #51* (D18). It feeds the gate; it does not gate.
4. **And** the register is a PURE DOCUMENT — it lives at `fixtures/scenario/traps/README.md`, next
   to the corpus it describes, and carries no runtime behaviour, no test, no gate of its own. It is
   NOT locked: as a `README.md` it is exempt from `MANIFEST.toml`'s sha256 lock and from the orphan
   rule, so a maintainer appends a case to it WITHOUT a deliberate-bump commit — exactly what a queue
   needs (a locked queue is a contradiction).
5. **And** the committed corpus stays byte-for-byte unchanged: `example.toml` and `MANIFEST.toml`
   are NOT touched, no artefact is added to the manifest, and `cargo xtask ci`'s `fixtures` gate
   stays green (the README is orphan-exempt by name).
6. **And** the one place that would red on the new file is fixed at the same time: the `#[cfg(test)]`
   traps walk `every_trap_file_in_the_corpus_is_valid` (`crates/opencmdb-bin/src/fixtures.rs`)
   asserts every file under `scenario/traps/` is `.toml` and lacks the `README.md` exemption that the
   PRODUCTION walk (`discover_trap_files`, `trap_gate.rs:334`) and the replay walk (`fixtures.rs:1373`)
   already carry — and that `fixtures/README.md` already PROMISES (*"`README.md` is exempt from both,
   at any depth"*). The exemption is added there, mirroring those two, so the code matches its own
   documented contract. **Prove-to-red:** with the register committed, removing that one line makes
   `every_trap_file_in_the_corpus_is_valid` panic (*"only .toml trap files belong under
   scenario/traps/"*).

## Tasks / Subtasks

- [x] **Task 1 — write the register** `fixtures/scenario/traps/README.md` (AC: 1, 2, 3, 4) — pure prose, zero code
  - [x] Open with a one-line orientation: this directory holds the **truth labelling** — which
        observations a case judges, which of D18's three columns is correct (`must-merge` /
        `must-not-merge` / `must-abstain`), and the author's mandatory one-sentence reason. Link
        `example.toml` (the format), `../README.md` (the scenario split), and `../../README.md` (the
        corpus lock). Match the existing corpus READMEs' voice: declarative, no hedging, each claim
        checkable. Read all three before writing (`fixtures/README.md`, `fixtures/scenario/README.md`,
        `fixtures/scenario/traps/example.toml`'s header).
  - [x] **The honest limit (AC1).** A section that states, attributed to D18: *a trap suite proves
        nothing about what was not imagined. At v0.1 the gate is weak and honest rather than strong
        and false.* Explain WHY it is on the record: coverage is not completeness — ~50 traps in three
        columns prove the engine against what we thought to write down, and nothing about the case we
        did not. Quote D18's exact framing; cite `architecture.md:1259-1261`.
  - [x] **The register itself (AC2).** A section defining the queue: what it is for (real-world
        identity/observation cases the corpus cannot yet produce), and the ENTRY FORMAT. Because AC2/AC3
        call this **a QUEUE that entries LEAVE** (each becomes trap #51, #52, …), the format must let an
        entry visibly DRAIN, not merely accumulate — so give it these columns: the **case** (the
        pattern, synthetic terms only); **its SOURCE** (where the real case was seen — a Tier 2 bulk
        run, a user report, a captured pattern); the **D18 column(s)** it will assert (`must-merge` /
        `must-not-merge` / `must-abstain`); and a **status / becomes** column recording, once drawn, the
        trap id or family it graduated into (so the queue is drainable and the "trap #51" framing is
        literal). State plainly: **this register is the queue from which trap #51 and beyond are drawn**
        (`architecture.md:1256-1257`). Provide the empty table/template with these columns and a
        one-line worked EXAMPLE OF THE FORMAT clearly marked as illustrative-not-real (mirroring how
        `example.toml` is "an EXAMPLE of the trap format, not a trap family"). **Do NOT invent a real
        entry** — the register opens empty of cases by design (AC2); an illustrative format row is not
        a case.
  - [x] **Tier 2 is the discovery mechanism (AC3).** A section naming **Tier 2** — bulk observability,
        ~300 hosts / 36 subnets, published per release with confidence intervals, trended, **blocking
        nothing** — as the ONLY discovery mechanism for the unimagined: *Tier 2 is Tier 1's trap
        factory. The gate only grows by proof. The day the bulk drops a cluster Tier 1 did not foresee,
        that cluster becomes trap #51.* Cite `architecture.md:1246-1261`. Make the direction explicit:
        Tier 2 FEEDS this register (and thence the gate); it never gates a release itself.
  - [x] **Distinguish it from `deferred-work.md`** in one sentence, so the two registers are never
        confused: `deferred-work.md` records known ENGINEERING gaps deferred from story reviews (code
        that exists but is incomplete); the reality-debt register records real-world CASES the corpus
        never imagined (a gap in the SPEC, discovered by Tier 2, that becomes a future trap). Link it.
  - [x] Close with the corpus's standing rule, restated because this file invites appends: **synthetic
        values only** — RFC 5737 documentation addresses, locally-administered MACs, invented
        hostnames; a real capture is disqualifying, not a preference (D19). A recorded case describes
        the PATTERN, never a real MAC/hostname/IP (see [[no-private-data-in-artifacts]]).
  - [x] **Add ONE down-pointer** in `fixtures/scenario/README.md`'s existing traps paragraph (~lines
        52-56, the *"`traps/` holds the truth labelling…"* sentence): a single clause noting that
        `traps/README.md` is the reality-debt register — what the corpus does NOT cover, and the queue
        for trap #51+. Without it, a reader at the scenario level never discovers the register one level
        down (docs-current-before-push). Keep it to a pointer, NOT a restatement. `scenario/README.md`
        is itself a `README.md` → editing it touches no locked bytes (orphan-exempt, not in MANIFEST).

- [x] **Task 2 — let the traps walk tolerate the README** (AC: 6) — one line + prove-to-red, `opencmdb-bin`
  - [x] In `crates/opencmdb-bin/src/fixtures.rs`, in the `#[cfg(test)]` test
        `every_trap_file_in_the_corpus_is_valid` (the recursive walk of `scenario/traps/`), add the
        `README.md` skip right after the `is_dir` continue and BEFORE the `is_toml` computation (~line
        1449). Add it **with a one-line rationale comment, not the bare `if`** — both sibling walks
        document their skip (the replay walk's comment at `fixtures.rs:1370-1372`, the production walk's
        fn doc at `trap_gate.rs:295-296`), and this repo's standing lesson is *a checkable claim gets
        checked*: a silent magic skip invites a Blind Hunter "why is README skipped here?". Mirror the
        replay walk's comment + line:
        ```rust
        // `README.md` is exempt at any depth, exactly as the corpus lock's orphan rule exempts it,
        // so documenting the traps directory does not turn this walk red.
        if path.file_name().and_then(|n| n.to_str()) == Some("README.md") {
            continue;
        }
        ```
        The `if` is BYTE-IDENTICAL to the replay walk (`fixtures.rs:1373`) and the production
        `discover_trap_files` (`trap_gate.rs:334`). This is deliberate mirrored redundancy (three
        independent walks agreeing about what the corpus may contain), NOT accidental duplication — do
        NOT collapse the three walks (the DRY rule keeps deliberate redundancy; they are independent
        oracles).
  - [x] **Prove-to-red and record the mutation.** **Do Task 1 first** — the register file must be on
        disk under `scenario/traps/` for this check to mean anything: without a committed `README.md`
        there, the exemption line is dead code and deleting it reds nothing (a vacuous "green either
        way"). With the register committed, the exemption is load-bearing: delete the one line, run
        `cargo test -p opencmdb-bin every_trap_file_in_the_corpus_is_valid`, observe the panic
        *"only .toml trap files belong under scenario/traps/"*, restore the line. Record that mutation
        in the Debug Log. (No new scratch test is needed: the committed corpus's own integrity test IS
        the guard that reds — the same mechanism `the_committed_corpus_is_discovered_and_scored_by_nothing`
        uses for `discover_trap_files`. A scratch test would re-implement the inline walk for no added
        assurance.)
  - [x] **Do NOT touch `discover_trap_files`** (`trap_gate.rs:301`): it already exempts `README.md`
        (line 334) and already names it in its error message (line 345). The production gate is
        already correct for a `traps/README.md`; only the `fixtures.rs` test walk lags.

- [x] **Task 3 — (optional) a doc line worth a careful second look** (AC: 6) — truthful-doc, `opencmdb-bin`
  - [x] `discover_trap_files`'s doc comment (`trap_gate.rs:298-300`) says the `#[cfg(test)]` walks in
        `fixtures.rs` are test HELPERS and names `walk_replay_streams` as *"scans replay streams, not
        traps"*. Read strictly, that parenthetical is arguably STILL TRUE: it speaks of the helper
        *functions*, and `walk_replay_streams` is the only walk-helper — `every_trap_file_in_the_corpus_is_valid`
        is an inline `#[test]`, not a helper. So only its IMPLICATION ("no test walks traps") has gone
        stale. **This task is genuinely optional.** If you touch it, the ONLY safe edit is one that stays
        TRUE — e.g. note that an inline `#[test]` (not a helper) now walks traps and, after Task 2, also
        exempts README. **Do NOT** replace a true statement about `walk_replay_streams` with a new claim
        that could itself be false — a fresh documented-but-false is worse than a slightly-stale-by-
        implication true one. When unsure, leave it and note it in the Completion Notes as pre-existing.

- [x] **Task 4 — gates and the corpus lock** (AC: 4, 5)
  - [x] `cargo fmt --all` · `cargo clippy --workspace --all-targets -- -D warnings` ·
        `cargo test --workspace` · `cargo run -p xtask -- ci` — all green.
  - [x] **`fixtures/MANIFEST.toml` and `fixtures/scenario/traps/example.toml` are byte-for-byte
        unchanged.** Do NOT add the README to the manifest (READMEs are exempt — `fixtures/README.md`,
        `MANIFEST.toml:12-13`). Confirm `cargo xtask ci`'s `fixtures` gate reports the same 5 artefacts
        matching sha256 and no orphan finding for the new README. Expected `git status --short`: the new
        `traps/README.md`, `fixtures.rs`, the down-pointer in `scenario/README.md`, the story file and
        `sprint-status.yaml` (and `trap_gate.rs` only if Task 3 was taken) — and NOTHING under
        `MANIFEST.toml` / `example.toml` / the replay streams.
  - [x] `Cargo.lock` unchanged (no dependency touched — one prose file, one test line, one doc line).
  - [x] `file-size` gate: the README is Markdown (not counted — the gate measures `.rs` before the
        first `#[cfg(test)]`); `fixtures.rs` and `trap_gate.rs` are far under 2000 code lines and do
        not grow materially. `architecture-views.md` is NOT regenerated inside this story (its
        `ℹ views-hash STALE` is by design — regenerate at the Epic-4 milestone, not here).

## Dev Notes

### The shape of this story in one paragraph

This is a **documentation story with a one-line correctness fix**. The deliverable is
`fixtures/scenario/traps/README.md` — the "reality-debt register": a prose file, next to the trap
corpus, that (1) admits D18's honest limit on the record, (2) defines the queue-and-entry-format for
real-world cases the corpus cannot yet produce, opened empty of cases by design, and (3) names Tier 2
bulk observability as the sole discovery mechanism that feeds — never gates — that queue. The register
carries NO code, NO test, NO gate: it is a pure document (Guy's decision, 2026-07-24). The only code is
a single mirrored line: the `#[cfg(test)]` traps walk `every_trap_file_in_the_corpus_is_valid` lacks
the `README.md` exemption that both the production gate walk and the replay walk already have — and
that `fixtures/README.md` already claims exists — so committing the README would red that test until
the exemption is added there too. Adding it makes the code match its documented contract, and the
committed README makes that line prove-to-red on removal.

### Where the register lives, and why it is NOT locked (AC4) — decided with Guy 2026-07-24

- **Location:** `fixtures/scenario/traps/README.md` — WITH the corpus (*"opened with the corpus
  itself"*), where the trap authors of stories 4.9+ already look. Alternatives were considered and
  rejected: a dedicated `REALITY-DEBT.md` would be an orphan (a bigger gate change), and a sibling of
  `deferred-work.md` in `implementation-artifacts/` is less literally "with the corpus".
- **Not locked, and that is the point.** A `README.md` is exempt from BOTH `MANIFEST.toml` directions
  (sha256 edit-lock AND orphan rule) — stated at `fixtures/README.md` and enforced in the xtask orphan
  gate (`xtask/src/main.rs:735`, `name == Some("README.md")`) and the manifest header
  (`MANIFEST.toml:12-13`). So a maintainer can APPEND a discovered case to the register without a
  deliberate-bump commit. A queue you cannot append to without re-locking the corpus is not a queue.
- Consequence: **do NOT add the README to `MANIFEST.toml`.** Adding it would (a) lock it, defeating the
  append-freely purpose, and (b) is unnecessary — the orphan rule already exempts it, so its absence
  from the manifest is not an orphan finding.

### The one code change, exactly (AC6) — three walks, one lagging

Three independent walks decide what `scenario/traps/` (or `scenario/replay/`) may contain. Two already
exempt `README.md`; one does not:

| Walk | File · line | Exempts `README.md`? | Role |
|---|---|---|---|
| `discover_trap_files` | `trap_gate.rs:334` | **YES** (and names it in the error, :345) | PRODUCTION — the release-gate corpus walk (`score_corpus`) |
| replay walk (test) | `fixtures.rs:1373` | **YES** | `#[cfg(test)]` — walks `scenario/replay/` for `.jsonl` |
| `every_trap_file_in_the_corpus_is_valid` (test) | `fixtures.rs:~1453` | **NO** ← the gap | `#[cfg(test)]` — walks `scenario/traps/` for `.toml` |

The fix is to add to the third walk the SAME exemption the other two carry (byte-identical to
`fixtures.rs:1373`), placed right before its `is_toml` assert. This is not new behaviour invented for
this story — it is the exemption `fixtures/README.md` ALREADY documents as universal (*"`README.md` is
exempt from both, at any depth, exactly as it is exempt from the orphan rule"*). Today that sentence is
a **documented-but-false** claim for the traps walk; Task 2 makes it true. (This codebase has a
standing lesson: *a comment asserting a checkable property gets checked* — story 2.2 shipped exactly
such a false comment and it survived until a later story added a case and watched the build. This is
one of those, caught during story preparation rather than in production.)

**Deliberate redundancy — do NOT collapse the three walks.** They are independent oracles by design
(the DRY rule for this repo keeps redundancy a test pins on purpose). `discover_trap_files` is the
harness's own walk; the two `#[cfg(test)]` walks re-check the committed corpus independently. Adding a
fourth abstraction to "share" the exemption would erase the redundancy the corpus-integrity tests rely
on. Mirror the line; do not refactor.

### What the register must literally say (AC1/AC2/AC3) — the D18 anchors

These are the exact phrasings to carry, attributed to D18 (`architecture.md:1208-1265`):

- **AC1, the honest limit** — *"a trap suite proves nothing about what I failed to imagine. At v0.1
  the gate is weak and honest rather than strong and false."* [architecture.md:1259-1261]
- **AC2, the queue** — *"Tier 2 is Tier 1's trap factory. The gate only grows by proof. The day the
  bulk drops a cluster Tier 1 did not foresee, that cluster becomes trap #51."* [architecture.md:1256-1257]
- **AC3, Tier 2** — *"OBSERVABILITY = Tier 2 (bulk, 300 hosts / 36 subnets), published per release with
  confidence intervals, trended — blocking nothing … Tier 2 is the only discovery mechanism for the
  unimagined. That is why it lives."* [architecture.md:1246-1261]
- The three columns the register orients around: `must-not-merge` (a merge fails), `must-merge` (an
  abstention fails — the anti-cowardice column), `must-abstain` (a decision fails). [architecture.md:1228-1244]

Write the register in this vocabulary. `must-merge` / `must-not-merge` / `must-abstain` are the
CANONICAL D18 column names (used throughout `example.toml`, `fixtures.rs`, `trap.rs`) — they are
correct and safe, NOT retired triage verbs. (For reference: the terms the xtask retired-vocabulary
gate actually watches are `pending_accept`, `reverting`, `accept-as-declared`, and `ignore`
(`xtask/src/main.rs`, the `PAIRS`/`CODE_RETIRED` consts) — English triage-`merge` is a retired UI
verb by project convention (D65) but is NOT in that gate's list. None of these belong in this file,
and none is what a trap column is called; and in any case no gate scans this file, so this is style,
not a constraint.)

### No gate scans this file (verified) — so the register is unconstrained prose

- **Retired-vocabulary gate (D65).** Volet B scans only the 7 docs in the `DOCS` const
  (`xtask/src/main.rs:354-362` — six planning artefacts + `docs/project-context.md`); `fixtures/…/README.md`
  is NOT among them. Volet A scans `crates/` only. So the register is outside both volets; no
  co-presence check applies. (Advice above to use canonical vocabulary is style/correctness, not gate
  avoidance.)
- **Fixtures gate (xtask).** Orphan rule exempts `README.md` by name; sha256 lock lists only the 5
  corpus artefacts. The register is invisible to the lock, by design.
- **Corpus walks (bin tests + prod).** After Task 2, all three exempt `README.md`. Before Task 2, only
  `every_trap_file_in_the_corpus_is_valid` bites — the one thing Task 2 fixes.

### What already exists — read it, match it, do not rewrite it

- **`fixtures/README.md`** — the corpus lock, the `scenario/` vs `capture/` split, the "these are a
  SPEC not test data" framing, the README-exempt-at-any-depth promise (which Task 2 makes true for the
  traps walk), and the **synthetic-values-only** rule. The register's voice must match this file.
- **`fixtures/scenario/README.md`** — the scenario split, the replay marker-key format, and the
  traps-labelling paragraph (*"`traps/` holds the truth labelling: which observations a case judges,
  which of D18's three columns is correct … a trap that can never fire would sit in the corpus looking
  like coverage, and the gate counts traps"*). The new `traps/README.md` sits one level DOWN and
  should not restate scenario/README wholesale — orient, then focus on the register.
- **`fixtures/scenario/traps/example.toml`** header — the model for "an EXAMPLE of the format, not a
  real X" framing. Use the same device for the register's illustrative format row.
- **`deferred-work.md`** — the OTHER register. Distinguish, link, do not merge. It is engineering debt
  (deferred from reviews); the reality-debt register is spec debt (unimagined real-world cases).
- **D18** (`architecture.md:1208-1265`) — the single source for every quoted phrase. The Decision
  Index at the top of `architecture.md` lists D18 at line 1208; scan the index before opening any
  further architecture question (project convention — F56).

### Project Structure Notes

- **NEW:** `fixtures/scenario/traps/README.md` (the register — pure prose, orphan-exempt, NOT in
  `MANIFEST.toml`).
- **Updated:** `crates/opencmdb-bin/src/fixtures.rs` — the `README.md` exemption (+ rationale comment)
  in the `#[cfg(test)]` `every_trap_file_in_the_corpus_is_valid` walk. `fixtures/scenario/README.md` —
  one down-pointer clause to the register (Task 1; a README, so no locked bytes). Optionally
  `crates/opencmdb-bin/src/trap_gate.rs` — the doc line Task 3 flags (only if it can be made *more*
  true).
- **Unchanged, expected:** `fixtures/MANIFEST.toml`, `fixtures/scenario/traps/example.toml` (both
  byte-for-byte — AC5), all replay streams, `discover_trap_files` (already correct — AC6), `Cargo.lock`,
  every other `.rs`. **`fixtures/README.md` unchanged** — its *"`README.md` is exempt from both, at any
  depth"* claim (`:52-54`) simply becomes TRUE once Task 2 lands; do NOT reword it. No domain type, no
  engine, no runtime path is touched — the identity engine is Epic 5; this story only opens a register
  and unblocks a README next to the corpus.
- **Out of scope, deliberately:** `docs/project-context.md` and `CLAUDE.md` are NOT edited here.
  Announcing a second maintainer-facing register is milestone/push-level under docs-current-before-push
  (this story does not push and is not a milestone); fold it in at the Epic-4 milestone with the
  `architecture-views.md` regeneration, not per-story.

### Traps (mistakes this story must not make)

1. **Inventing a "discovered" case to fill the register.** AC2: it opens EMPTY of cases, with its
   format only. No Tier 2 has run; a fabricated entry is the dishonesty the register exists to prevent.
   An illustrative FORMAT row (clearly marked not-real, as `example.toml` does) is fine; a real case is
   not, until Tier 2 finds one.
2. **Adding the README to `MANIFEST.toml`.** It would lock the queue and it is unnecessary (orphan-exempt).
3. **Writing a test or gate for the register.** Guy chose a pure document (2026-07-24). The only code
   is Task 2's exemption, which is a fix to a walk, not a guard on the register.
4. **Collapsing the three corpus walks to "share" the exemption.** Deliberate redundancy — mirror the
   one line, do not refactor (DRY rule keeps pinned redundancy).
5. **Touching `discover_trap_files`.** It already exempts `README.md`; changing it is churn and risks
   the production gate.
6. **Editing `example.toml` or any locked artefact.** Any byte change reds the sha256 lock. The story
   adds a file and one test line; it changes no committed corpus bytes.
7. **Recording a real MAC/hostname/IP as an example.** Synthetic-only, always ([[no-private-data-in-artifacts]]).
8. **Claiming more than measured.** Name the command behind every count; record Task 2's mutation to
   red (the standing lesson — completion records have over-claimed before; write the weaker true
   sentence). [[claims-must-match-verification]]

### Latest technical specifics

No new crate, no version bump, no domain code. Rust 1.96+, edition 2024. One Markdown file, one
`#[cfg(test)]` line in `opencmdb-bin`, optionally one doc line. **Never invent a version — pin from the
committed `Cargo.lock`, which does not move here.**

### References

- [Source: _bmad-output/planning-artifacts/epics.md:1071-1087 — Story 4.8 "Open the reality-debt
  register": the three ACs (honest limit in D18's words; a real case recorded with its source, the
  queue for trap #51+; Tier 2 the only discovery mechanism, blocking nothing)]
- [Source: _bmad-output/planning-artifacts/epics.md:393 — Epic 4 charter: "Open the 'reality-debt'
  register for traps the real connectors will later add"]
- [Source: _bmad-output/planning-artifacts/architecture.md:1208-1265 — D18, the release gate: Tier 1
  binary / three columns, and the Tier 2 "trap factory" + "weak and honest rather than strong and
  false" + "trap #51" language every AC quotes]
- [Source: fixtures/README.md — the corpus lock, README-exempt-at-any-depth (the contract Task 2 makes
  true for the traps walk), the scenario/capture split, synthetic-values-only]
- [Source: fixtures/scenario/README.md — the scenario split and the traps-labelling paragraph; the
  register sits one level down and orients from here]
- [Source: fixtures/scenario/traps/example.toml:1-12 — "an EXAMPLE of the format, not a trap family";
  the illustrative-not-real device the register's format row reuses]
- [Source: crates/opencmdb-bin/src/fixtures.rs:1343-1382 — the replay walk's README exemption (:1373),
  the byte-identical model for Task 2]
- [Source: crates/opencmdb-bin/src/fixtures.rs:1424-1467 — `every_trap_file_in_the_corpus_is_valid`,
  the traps walk missing the exemption (~:1453), the one place Task 2 changes]
- [Source: crates/opencmdb-bin/src/trap_gate.rs:291-356 — `discover_trap_files`, the PRODUCTION walk
  that ALREADY exempts README.md (:334-335, :345); and its doc line :298-300 that Task 3 may correct]
- [Source: xtask/src/main.rs:709-745 — `orphan_findings`, the fixtures orphan gate that exempts
  `README.md` by name (:735); MANIFEST.toml:12-13 records the same exemption]
- [Source: xtask/src/main.rs:354-362 — `DOCS`, the retired-vocabulary volet-B scope; the register is
  NOT in it, so no vocabulary gate touches the file]
- [Source: _bmad-output/implementation-artifacts/deferred-work.md — the OTHER register (engineering
  debt from reviews); the reality-debt register distinguishes itself from it]
- [Source: _bmad-output/implementation-artifacts/4-7b-trap-corpus-positive-and-negative.md — the sibling
  story: the corpus completeness guard; 4.8 opens the register that names what completeness cannot
  reach]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.8 — `claude-opus-4-8[1m]`

### Debug Log References

Every claim below is a command that was run, not an inference.

- **Prove-to-red, observed (AC6).** After committing `fixtures/scenario/traps/README.md` and BEFORE
  adding the exemption, `cargo test -p opencmdb-bin every_trap_file_in_the_corpus_is_valid` **FAILED**
  with the exact predicted panic: `crates/opencmdb-bin/src/fixtures.rs:1453 … only .toml trap files
  belong under scenario/traps/` on the new `README.md`. After adding the one-line exemption (+ its
  rationale comment), the same test **passed**. Mutation recorded: the committed register reds that
  walk unless the exemption is present — the walk itself is the guard, no scratch test added.
  | Mutation | Test reddened |
  |---|---|
  | register committed, exemption line ABSENT | `every_trap_file_in_the_corpus_is_valid` — panic `only .toml trap files belong under scenario/traps/` at `fixtures.rs:1453` |
- **Gates:** `cargo fmt --all` clean · `cargo clippy --workspace --all-targets --locked -- -D warnings`
  clean · `cargo test --workspace --locked` → **107 (bin) + 86 (core) + 42 (xtask), 0 failed** (counts
  UNCHANGED — no test function added; the existing corpus-integrity test became the guard, as the story
  specified).
- `cargo run -p xtask -- ci` → all gates green: `fixtures` **5 sha256 match** (the new README is
  orphan-exempt by name — not counted, not listed), `file-size` largest **884** (far under 2000; the
  README is Markdown, not counted), `views-hash STALE` exits 0 by design (NOT regenerated this story).
- **Locked corpus + `Cargo.lock` byte-unchanged**, measured: `git status --short` for
  `fixtures/MANIFEST.toml`, `fixtures/scenario/traps/example.toml`, `fixtures/scenario/replay/` and
  `Cargo.lock` is **empty**. No dependency touched.
- **The MariaDB-backed tests did NOT run** (`DATABASE_URL` unset) — the green suite says nothing about
  the database, and this story touches no DB path.

### Completion Notes List

- **AC1/AC2/AC3 — the register is a pure document at `fixtures/scenario/traps/README.md`.** It carries
  D18's honest limit VERBATIM and attributed (*"a trap suite proves nothing about what I failed to
  imagine. At v0.1 the gate is weak and honest rather than strong and false."* — not the epic's
  "was not imagined" paraphrase); defines the queue and a **drainable** entry format (Case / Source /
  D18 column(s) / Status → becomes trap #NN), opened EMPTY of cases with one clearly-illustrative
  format row (AC2 — no Tier 2 run has happened, inventing a case would be the dishonesty it prevents);
  and names **Tier 2** (bulk observability, ~300 hosts / 36 subnets, blocking nothing) as the sole
  discovery mechanism that FEEDS the register which feeds the gate (AC3). It is distinguished from
  `deferred-work.md` (spec debt vs engineering debt) and closes with the synthetic-data-only rule.
- **AC4/AC5 — pure document, corpus untouched.** No code, no test, no gate belongs to the register. As
  a `README.md` it is exempt from `MANIFEST.toml`'s sha256 lock and the orphan rule, so the queue is
  appendable without a bump. `MANIFEST.toml`, `example.toml`, the replay streams and `Cargo.lock` are
  byte-for-byte unchanged (measured); `xtask ci`'s `fixtures` gate stays green.
- **AC6 — one mirrored line fixes a documented-but-false contract.** The `#[cfg(test)]` traps walk
  `every_trap_file_in_the_corpus_is_valid` (fixtures.rs) now exempts `README.md`, matching the
  production walk `discover_trap_files` (trap_gate.rs:334), the replay walk (fixtures.rs:1373), and
  `fixtures/README.md`'s already-stated *"exempt … at any depth"* promise. The exemption carries the
  same rationale comment its siblings do (no silent magic skip). `discover_trap_files`'s logic was NOT
  touched (already correct).
- **Task 3 taken (truthful-doc).** `discover_trap_files`'s doc line said the `#[cfg(test)]` walks are
  "test helpers (`walk_replay_streams` scans replay streams, not traps)" — over-general, since
  `every_trap_file_in_the_corpus_is_valid` is an inline test that walks traps. Rewrote it to state that
  accurately (keeping the true statement about `walk_replay_streams`) and to note it now also exempts
  `README.md`. A MORE-true edit, not a new claim.
- **E3 — the parent README points down.** `fixtures/scenario/README.md`'s traps paragraph gained one
  clause naming `traps/README.md` as the reality-debt register, so a reader at the scenario level
  discovers it (docs-current-before-push). A pointer, not a restatement.
- **Deliberately out of scope:** `docs/project-context.md` / `CLAUDE.md` (milestone/push-level), the
  `architecture-views.md` regeneration (Epic-4 milestone), and any real register case (Tier 2 has not
  run).

### File List

- `fixtures/scenario/traps/README.md` (**new** — the reality-debt register; pure prose, orphan-exempt,
  not in `MANIFEST.toml`)
- `crates/opencmdb-bin/src/fixtures.rs` (modified — `README.md` exemption + rationale comment in the
  `#[cfg(test)]` `every_trap_file_in_the_corpus_is_valid` traps walk)
- `crates/opencmdb-bin/src/trap_gate.rs` (modified — Task 3: corrected the `discover_trap_files` doc
  line to accurately describe the inline traps-walking test and its README exemption)
- `fixtures/scenario/README.md` (modified — one down-pointer clause to the register; a README, no
  locked bytes)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modified — 4-8 status)
- `fixtures/MANIFEST.toml`, `fixtures/scenario/traps/example.toml`, `fixtures/scenario/replay/*`,
  `Cargo.lock` — **unchanged**, measured
- `crates/opencmdb-bin/src/trap_gate.rs::discover_trap_files` logic — **unchanged** (already exempts
  README; only its doc comment was corrected)

### Change Log

- 2026-07-24 — Opened the reality-debt register at `fixtures/scenario/traps/README.md`: D18's honest
  limit on the record (verbatim), a drainable entry format (case + source + D18 column + becomes-trap),
  opened empty of cases by design, and Tier 2 named as the sole discovery mechanism that feeds the gate.
  One code change: the `#[cfg(test)]` traps walk now exempts `README.md` (mirroring the production and
  replay walks and honouring `fixtures/README.md`'s already-stated contract) — proven to red (the
  committed register panics the walk without it). Task 3: corrected a `discover_trap_files` doc line to
  the truth. The parent `scenario/README.md` points down to the register. Locked corpus and `Cargo.lock`
  byte-unchanged; 107 bin + 86 core + 42 xtask, 0 failed; `xtask ci` all green.

## Review Findings

_Code review 2026-07-24 (Blind Hunter + Edge Case Hunter + Acceptance Auditor). Acceptance Auditor:
PASS on AC1–AC6 + all 8 traps — the verbatim D18 quote flagged in prep shipped correctly ("what I
failed to imagine", not the epic paraphrase), locked corpus byte-unchanged, prove-to-red recorded, no
over-claim. Edge Case Hunter: no unhandled edge cases (all four corpus walks agree on the exact-case
`README.md` exemption, symlink/dir/case ordering sound, all five relative links resolve). No
Critical/High. Findings below._

- [x] [Review][Patch APPLIED] The D18 Tier-2 blockquote adds a `~` absent from the quoted source —
      [fixtures/scenario/traps/README.md] — the `> OBSERVABILITY = Tier 2 (bulk, ~300 hosts / 36
      subnets)…` line was rendered as an attributed D18 quote, but `architecture.md:1246` reads
      `300 hosts` (no tilde). Dropped the `~` inside the blockquote so it is character-exact to D18;
      the register carries no other `~300` in prose. Non-locked README — no gate reads it, corpus
      lock unaffected.
- [x] [Review][Defer] The reality-debt register is outside the privacy walk's reach —
      [fixtures/scenario/traps/README.md] — deferred, pre-existing. `assert_facts_are_synthetic` /
      `the_corpus_carries_no_real_network_data` scan `scenario/replay/` streams only; no automated
      check scans any README. Not introduced by this diff (no README was ever scanned), but the
      register is by design the corpus file most likely to tempt a pasted real MAC/IP, since it
      accumulates cases sourced from real Tier-2 bulk runs. Mitigation today is prose discipline (D19 +
      the register's own "Never real network data" section). Recorded in `deferred-work.md`.

- Dismissed (3, all from the diff-only Blind Hunter and resolved by the repo-access layers): the
  exact-case `README.md` match is NOT an accidental asymmetry — all four corpus walks (traps test,
  replay test, production `discover_trap_files`, xtask orphan gate) use the identical exact-case
  matcher, and `scenario/readme.md`-must-be-an-orphan is a pinned test (`xtask/src/main.rs`), so the
  convention is deliberate and 4.8 mirrors it correctly; the "identical exemptions" doc claim is TRUE
  (verified byte-identical across all four); the walked-but-unvalidated README is no smuggle vector
  (`discover_trap_files` also skips it, so it is never parsed as a trap) and is symmetric with the
  three other corpus READMEs.
