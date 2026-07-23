# Story 4.7b: Every trap exists in positive and negative form

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author of the trap corpus,
I want a corpus missing one polarity of a trap family reported as INCOMPLETE,
so that the gate cannot pass on a family that was only ever tested one way.

## Acceptance Criteria

1. **Given** the trap suite, **when** a trap family exists in only one decision form — a `must-merge`
   trap with no `must-not-merge` sibling, or the reverse — **then** the corpus is reported as
   INCOMPLETE rather than passing, and the report NAMES the family and WHICH pole is missing (a
   one-sided family is a gate that was never shown it can fail the other way).
2. **And** a family present in BOTH decision forms (≥1 `must-merge` AND ≥1 `must-not-merge`) is
   complete and does not appear in the report — the check tightens the gate, it does not reject a
   corpus that is genuinely two-sided.
3. **And** the family is an EXPLICIT `family` field on the trap, not derived from the trap id or file
   name (Guy's decision 2026-07-23: explicit types over string conventions, the same reason
   observations are keyed by `obs_id` and `Expectation` is an enum, not optional fields). The two
   poles are DERIVED from the existing [`Expectation`] column (`must-merge` / `must-not-merge`); no
   separate `polarity` field is introduced.
4. **And** a trap that declares NO `family` is exempt — it is a format/example trap, not a family, and
   `fixtures/scenario/traps/example.toml` (which says so in its own header) must stay complete-by-
   exemption and BYTE-UNCHANGED, so the corpus sha256 lock (`MANIFEST.toml`) stays green. Adding the
   optional field to the `Trap` type must not require editing any committed `.toml`.
5. **And** the completeness verdict blocks `passed()` exactly as a truth-table failure and a wrong-rule
   mismatch do (story 4.6b / 4.7a), while the incomplete-family count stays a THIRD readable number,
   separate from `failures()` and `rule_mismatches()`. Grouping is cross-FILE and case-folded, the
   same normalization `TrapFile::validate` already uses for `DuplicateId` (`"Randomized-MAC"` and
   `"randomized-mac"` are one family), so a family split across two files is still seen as one.
6. **(Pre-family corpus — no family fires in Epic 4's committed corpus.)** The committed corpus today
   is only `example.toml`, which declares no family, so the incomplete-family report is empty and the
   gate stays green. This story adds the GUARD; the families it guards arrive from story 4.9 onward
   (`randomized-mac`, `multi-nic`, `shared-hardware-vm`, …), each committed in both forms. 4.7b is the
   check written BEFORE those families, the same "metric before the engine" discipline as 4.6/4.7a.

## Tasks / Subtasks

- [x] **Task 1 — the `family` field and its id type** (AC: 3, 4) — pure `opencmdb-core`
  - [x] Add `pub struct FamilyId(pub String)` in `trap.rs`, the newtype sibling of [`TrapId`] /
        [`RuleId`] — `#[serde(transparent)]`, same derives (`Clone, PartialEq, Eq, PartialOrd, Ord,
        Hash, Serialize, Deserialize`). Doc it: the stable name that groups a trap family across the
        corpus.
  - [x] Add `pub family: Option<FamilyId>` to `Trap`, `#[serde(default)]` so an ABSENT key is `None`
        (exempt), not a serde error — the same idiom `reason` already uses (trap.rs:102), which is why
        it works under `deny_unknown_fields`. `Trap` carries `#[serde(deny_unknown_fields)]`; adding a
        known optional field keeps every existing `.toml` valid and byte-unchanged (AC4). Doc: a family
        groups the positive and negative forms of one identity trap; `None` marks a format/example
        trap that is not part of any family and is exempt from the completeness check.
  - [x] `Trap` is not `#[non_exhaustive]` and all fields are public. The ONLY struct-literal
        construction in the tree is the `trap()` test helper (trap.rs:298); add `family: None` there or
        it will not compile. Every other `Trap` is TOML-parsed and needs no change (the compiler names
        this instantly, but state it so it is not a surprise).
  - [x] Validate in `Trap::validate`: a `family` PRESENT but blank/whitespace is `TrapError::
        FamilyEmpty { trap }` — the mirror of the existing `IdMissing` / `ReplayMissing` guards. A
        `None` family is legal (exempt) and validates fine. Add the `FamilyEmpty` variant + its
        `Display` arm.
  - [x] Test: a trap with `family = Some("randomized-mac")` validates; `family = Some("  ")` reds with
        `FamilyEmpty`; `family = None` validates. **Prove-to-red** the blank case (a mutation dropping
        the `FamilyEmpty` guard lets `"  "` through).

- [x] **Task 2 — the completeness check** (AC: 1, 2, 3, 5) — pure `opencmdb-core`
  - [x] Add `pub struct IncompleteFamily { pub family: FamilyId, pub has_merge: bool, pub
        has_not_merge: bool }` in `trap.rs`, `#[derive(Debug, Clone, PartialEq, Eq)]` — the same
        derives as `RuleMismatch` (trap_gate.rs:67), REQUIRED because `Report` derives `PartialEq, Eq`
        and will hold a `Vec<IncompleteFamily>`; omit them and `Report`'s derive fails to compile.
        Invariant: constructed ONLY when `!(has_merge && has_not_merge)` — a complete family is never
        reported. `family` carries the ORIGINAL author casing (first-seen), not the folded key, so the
        report names the family as written (AC1); the fold is a grouping key only. Doc each field; doc
        that it names which pole is present so a red gate is debuggable without opening the corpus (the
        sibling of 4.7a's `RuleMismatch`).
  - [x] Add `pub fn incomplete_families<'a>(traps: impl IntoIterator<Item = &'a Trap>) ->
        Vec<IncompleteFamily>`. Group the traps that DECLARE a family by the case-folded family id
        (`trim().to_lowercase()`, the exact normalization `TrapFile::validate` uses for `DuplicateId`
        at trap.rs:276), tracking per family whether a `must-merge` and a `must-not-merge` trap were
        seen AND the first-seen original `FamilyId` for the message. A `must-abstain` trap in a family
        contributes NEITHER pole (DR1: it is D18's orthogonal third column, not a decision form). Return one
        `IncompleteFamily` per one-sided family, sorted by the folded key for determinism. Traps with
        `family == None` are skipped entirely (AC4). The function assumes validated input
        (`Trap::validate` has run, so no blank-after-trim family reaches it); it need not re-guard the
        `Some("")` case — state this so the omission reads as a decision.
  - [x] Poles are derived from [`Expectation`]: `MustMerge` → the merge pole, `MustNotMerge` → the
        not-merge pole, `MustAbstain` → neither. Do NOT add a `polarity` field (AC3).
  - [x] Tests (prove-to-red), offending case where the house rule wants it:
        - a family with a `must-merge` AND a `must-not-merge` trap → `incomplete_families` is empty
          (AC2, the complete case);
        - a family with only `must-merge` → one `IncompleteFamily { has_merge: true, has_not_merge:
          false }` (AC1, missing pole named);
        - a family with only `must-not-merge` → one `IncompleteFamily` the other way, so BOTH one-sided
          directions are pinned, not just one;
        - a family of `{must-merge, must-abstain}` → still INCOMPLETE, missing `must-not-merge` (the
          load-bearing rule that an abstention counts for NEITHER pole). **Prove-to-red**: a mutation
          letting `MustAbstain` satisfy the not-merge pole would turn this green, so this test reds it;
        - two traps of the SAME family across the check input in both poles → complete (cross-grouping);
        - a mix of `family = Some(...)` and `family = None` traps → the `None` ones never appear in the
          result (AC4, exemption);
        - case-folding: `Some("Randomized-MAC")` + `Some("randomized-mac")`, one merge one not-merge →
          complete (one family, AC5). **Prove-to-red**: collapsing the normalization to a raw-string
          group reds this (two families, both one-sided).

- [x] **Task 3 — surface incomplete families in the harness** (AC: 1, 5) — `opencmdb-bin`
  - [x] In `trap_gate.rs`, grow `Report` with `incomplete_families: Vec<IncompleteFamily>`. During the
        existing single corpus walk in `score_corpus`, each file's `TrapFile` is a LOCAL that drops at
        the end of its loop iteration, so a `&Trap` into it cannot survive to the end of the walk
        (E0597). Accumulate OWNED clones into a `let mut all_traps: Vec<Trap> = Vec::new();`
        (`all_traps.push(trap.clone())` inside the existing `for trap in &traps.trap` loop), then call
        `opencmdb_core::trap::incomplete_families(&all_traps)` once at the end and store the result.
        The family check is answer-INDEPENDENT — it is about corpus SHAPE, not scoring — so it runs
        whether or not any trap has an answer, and the vacuity/duplicate-id/unknown-answer guards stay
        untouched.
  - [x] `Report::incomplete_families() -> &[IncompleteFamily]`. `passed()` becomes `discovered > 0 &&
        failures() == 0 && rule_mismatches().is_empty() && incomplete_families().is_empty()` (AC5).
  - [x] `Display`: per DR2, append `", J incomplete-famil{y|ies}"` to the FIRST line when non-empty,
        AFTER 4.7a's `", K wrong-rule failure(s)"` suffix (fixed order: truth-table → wrong-rule →
        incomplete-family), so the first line alone can never read as a pass while the gate is red. Then
        append one line per incomplete family naming the family and the missing pole, e.g. ``incomplete
        family `randomized-mac`: has must-merge, missing must-not-merge`` (and the "has neither pole"
        form for an abstain-only family, DR1). **Additive only** — the 4.6b first line's asserted
        substrings (`the_report_says_plainly_that_nothing_was_scored`) and 4.7a's wrong-rule lines stay
        intact; the counts stay readable as separate numbers (AC5).
  - [x] Tests (prove-to-red): a scratch corpus with a family in only one pole → `passed()` false, one
        entry in `incomplete_families()`, `failures()` and `rule_mismatches()` both still 0 (it is a
        DISTINCT third condition, not double-counted); the same family with both poles → green.
        Offending (one-sided) corpus present; a mutation dropping `incomplete_families` from `passed()`
        reds the `!passed()` assertion.
  - [x] Test the RENDERED `Display` (AC1 requires the report to NAME the family and the missing pole —
        the prescribed `passed()`/count asserts above do NOT pin the string, so a wrong name or pole
        would pass them). Assert `report.to_string()` contains the exact line, e.g. ``incomplete family
        `randomized-mac`: has must-merge, missing must-not-merge``, the way 4.7a pinned its wrong-rule
        line (trap_gate.rs:780,784). **Prove-to-red**: mangling the family name or the pole reds it.

- [x] **Task 4 — do NOT touch the committed corpus** (AC: 4)
  - [x] `fixtures/scenario/traps/example.toml` stays byte-for-byte unchanged — its three traps declare
        no `family` and are exempt. Do NOT add a `family` key to them, do NOT add a new committed trap
        file. Confirm `cargo xtask ci`'s `fixtures` gate stays green (sha256 unchanged, no orphan).
  - [x] If a note is warranted, it goes in `deferred-work.md` (the families arrive in 4.9+), NOT in the
        committed corpus.

- [x] **Task 5 — gates** (all AC)
  - [x] `cargo fmt --all` · `cargo clippy --workspace --all-targets -- -D warnings` ·
        `cargo test --workspace` · `cargo run -p xtask -- ci` all green.
  - [x] `Cargo.lock` unchanged (no new dependency — pure domain + existing bin deps).
  - [x] `trap.rs` and `trap_gate.rs` stay well under the 2000-code-line `file-size` gate (largest file
        was 884 at 4.7a).
  - [x] Make `FamilyId`, `IncompleteFamily`, `incomplete_families` reachable as
        `opencmdb_core::trap::…`, the SAME path `trap_gate.rs:58` already uses for `RuleId`/`TrapId`.
        **Do NOT add a crate-root re-export.** 4.7a's `TrapVerdict`/`run_trap` are re-exported at the
        root (`lib.rs:44-47`) only because they live in `score`, which is already re-exported; the
        `trap` module is exposed as `pub mod trap;` (lib.rs:26) and NONE of its items (`TrapId`, `Trap`,
        `Expectation`) are re-exported at the root. Adding a `pub use trap::{FamilyId, …}` line would be
        an asymmetric partial export (`FamilyId` at root but not its sibling `TrapId`). Just make the
        items `pub`; the harness reaches them by module path.

## Dev Notes

### What already exists — use it, do not rewrite it

- **`Trap`** [Source: crates/opencmdb-core/src/trap.rs:90] — `{ id, replay, observations, reason,
  expect }`, `#[serde(deny_unknown_fields)]` (the `pub struct` is at :90; :87 is its doc comment). Task
  1 adds ONE optional field beside these; nothing else on the type changes.
- **`TrapId` / `RuleId`** [Source: crates/opencmdb-core/src/trap.rs:38-43] — `#[serde(transparent)]`
  newtypes over `String`. `FamilyId` (Task 1) is their exact sibling; build it the same way.
- **`Expectation`** [Source: crates/opencmdb-core/src/trap.rs:55] — the enum whose `must-merge` /
  `must-not-merge` / `must-abstain` columns ARE the poles. `Expectation::column()` (trap.rs:70) already
  names them. Derive the pole from the variant; do not add a parallel `polarity`.
- **`Trap::validate` / `TrapFile::validate`** [Source: crates/opencmdb-core/src/trap.rs:219, 269] — the
  admissibility guards. `FamilyEmpty` is a new `Trap::validate` guard, the mirror of `ReplayMissing`.
  The case-folding for grouping (Task 2) is the SAME `trim().to_lowercase()` `TrapFile::validate`
  already uses for `DuplicateId` (trap.rs:276) — reuse the idiom, do not invent a second normalization.
- **`TrapError`** [Source: crates/opencmdb-core/src/trap.rs:117] — the closed error taxonomy (D47:
  domain data, not strings). Add `FamilyEmpty { trap: TrapId }` and its `Display` arm.
- **`score_corpus` / `Report`** [Source: crates/opencmdb-bin/src/trap_gate.rs] — the harness that walks
  every trap file once. It already collects `seen` (every trap id + its file); Task 3 additionally
  retains each `Trap` so the corpus-level `incomplete_families` check can run on the full set. This is
  the SAME shape 4.7a used to add `rule_mismatches`: a new `Vec` on `Report`, a new accessor, a term in
  `passed()`, appended `Display` lines.
- **The 4.7a pattern is the template — with ONE layer difference.** 4.7b is 4.7a's structural twin one
  level up: 4.7a asserts `(verdict, rule)` per trap; 4.7b asserts `(family has both poles)` per family.
  `run_trap` ↔ `incomplete_families` are BOTH pure core functions (a true mirror). But the record type
  is NOT a mirror of layer: 4.7a's `RuleMismatch` lives in **bin** (`trap_gate.rs:68`), whereas
  `IncompleteFamily` lives in **core** (`trap.rs`, with its function) because the family check is pure
  domain over `&Trap`. So `IncompleteFamily` is defined in core and merely CARRIED by `Report` in bin —
  do not put it in `trap_gate.rs`. The Report-side plumbing (new `Vec` field, accessor, `passed()`
  term, appended `Display` lines) is the accurate part of the analogy. Read
  `4-7a-trap-runner-asserts-the-rule.md` before starting.

### The design in one paragraph

A trap may declare a `family`. The completeness check groups the family-declaring traps by their
case-folded family id and asks, per family, "did I see both decision poles?" — at least one
`must-merge` AND at least one `must-not-merge`, the two forms every identity family must be tested in
(D18: a family shown only one way is a gate never shown it can fail the other). The poles are the
existing `Expectation` columns; a `must-abstain` trap is neither pole. A one-sided family is an
`IncompleteFamily` naming which pole is present, carried in a third `Report` bucket beside
`failures` (4.6b) and `rule_mismatches` (4.7a), blocking `passed()`. Traps without a family are
exempt, so the format/example corpus stays green and the real families (4.9+) are the only thing the
guard bites.

### Traps

1. **Deriving family or polarity from the trap id / file name.** AC3 forbids it — Guy chose an explicit
   field. A naming convention (`randomized-mac/positive`) is the fragile path the codebase avoids
   everywhere else (obs by `obs_id`, not line; `Expectation` an enum, not optional fields).
2. **Editing `example.toml` to give its traps a family.** AC4: it is NOT a family (its own header says
   so), it is the exemption case, and any byte change reds the `fixtures` sha256 lock. The exemption is
   tested by a `family = None` trap NOT appearing in `incomplete_families`, in a scratch corpus — not by
   touching the committed file.
3. **Grouping case-sensitively.** `TrapFile::validate` already folds case for `DuplicateId`; a family
   split as `Randomized-MAC` / `randomized-mac` across two files must be ONE family, or the check both
   misses a real one-sided family and invents false ones. Reuse `trim().to_lowercase()` (AC5). Prove
   the folding to red.
4. **Counting `must-abstain` as a pole.** A family's two forms are the two DECISIONS (merge /
   not-merge). An abstention is neither; a family of `{must-merge, must-abstain}` is still missing its
   `must-not-merge` pole and is INCOMPLETE. Do not let an abstention satisfy either pole. (Whether an
   abstention may even belong to a family is an Open Design Question — default: it may, but it counts
   for neither pole.)
5. **Double-counting.** An incomplete family is NOT also a truth-table failure or a wrong-rule mismatch
   — it is a corpus-shape defect, orthogonal to any answer. `failures()` and `rule_mismatches()` stay 0
   while `incomplete_families()` is 1 in the one-sided-family test. Three separate numbers (AC5).
6. **Making the check answer-dependent.** Completeness is about the corpus, not about whether an engine
   answered. It must run over the discovered traps regardless of the `answers` map — including when
   `answers` is empty (the pre-engine state). Do not fold it into the per-answer scoring loop.
7. **Building families now.** There are no families in Epic 4's committed corpus (only `example.toml`).
   4.7b is the GUARD; 4.9–4.11 commit the families. Inventing a family here to "exercise" the check is
   the wrong move — exercise it with SCRATCH corpora in tests, exactly as 4.6b/4.7a exercise their
   guards, and leave the committed corpus family-less (AC6).
8. **Claiming more than measured.** Name the command behind every count; prove every new guard to red
   and record the mutation (the standing lesson — four completion records over-claimed before).

### Resolved Design Decisions (locked with Guy 2026-07-23 — implement as stated)

- **A "family" is a named class of identity scenario** (randomized-mac, multi-nic, shared-hardware-vm,
  …; one per stories 4.9–4.17). "Complete" means tested in BOTH decision directions — the D18 theorem:
  an always-merge engine passes a merge-only family, an always-refuse engine passes a not-merge-only
  family, so a one-sided family is a gate never challenged to fail the other way.
- **DR1 (was DQ1) — a `must-abstain` trap MAY carry a family but counts for NEITHER pole.** A family is
  complete iff it has ≥1 `must-merge` AND ≥1 `must-not-merge`; abstention is D18's orthogonal third
  column, not one of the two decision forms, so it is grouped with its family but never satisfies a
  pole. `must-abstain` is NOT forbidden on a family (a 3-column family — merge + not-merge + an ambiguous
  edge of the same scenario — is expressible and useful). Consequence, accepted: a family carrying ONLY
  `must-abstain` traps reports incomplete with `has_merge=false, has_not_merge=false`; its `Display` is
  ``incomplete family `X`: has neither pole (needs must-merge and must-not-merge)``.
- **DR2 (was DQ2) — the first `Display` line DOES carry an incomplete-family count**, mirroring 4.7a's
  `", K wrong-rule failure(s)"` suffix: append `", J incomplete-famil{y|ies}"` to the first line when
  non-empty, so the line alone can never read as a pass while the gate is red. **Order is fixed and
  deterministic**: truth-table count, then wrong-rule suffix (4.7a), then incomplete-family suffix —
  all on the one first line — or the rendered string flakes its substring tests. Pin the exact first
  line in a test the way 4.7a did.

### Project Structure Notes

- **Updated:** `crates/opencmdb-core/src/trap.rs` (`FamilyId`, `Trap.family`, `TrapError::FamilyEmpty`
  variant + its `Display` arm, `IncompleteFamily`, `incomplete_families`),
  `crates/opencmdb-core/src/lib.rs` (exports), `crates/opencmdb-bin/src/trap_gate.rs` (retain traps in
  `score_corpus`, `Report.incomplete_families`, accessor, `passed()`, `Display`).
- **Unchanged, expected:** `score()` / `Tally` / `run_trap` (4.6a/4.7a semantics frozen),
  `fixtures/` (byte-for-byte — AC4), `Cargo.lock`. `example.toml` MUST NOT change.
- The completeness logic is PURE domain (D47): it lives in `opencmdb-core` and takes `&Trap`s; the bin
  harness only reads files and surfaces the result — the same frontier split as 4.7a's `run_trap`.

### Latest technical specifics

No new crate, no version bump. Rust 1.96+, edition 2024. Pure domain code in core (serde derive already
present on `Trap`), plus existing bin deps in the harness. **Never invent a version — pin from the
committed `Cargo.lock`.**

### References

- [Source: _bmad-output/planning-artifacts/epics.md:1058 — Story 4.7b: "Every trap exists in positive
  and negative form"; the AC — a one-sided family is reported INCOMPLETE, not passing]
- [Source: _bmad-output/planning-artifacts/epics.md:1023 — the 4.7 → 4.7a/4.7b split note: 4.7b is a
  check on the CORPUS (every family in positive AND negative form), independent of 4.7a's scoring change]
- [Source: _bmad-output/planning-artifacts/epics.md:1089-1128 — stories 4.9 (:1089), 4.10 (:1107),
  4.11 (:1121), the families
  this guard protects, each "committed in positive and negative form"; 4.9's positive form is a
  must-not-merge and its negative form a must-merge — the two poles the check derives]
- [Source: _bmad-output/planning-artifacts/architecture.md:1208 — D18: the release gate is one number,
  and a family shown only one way is a gate never shown it can fail the other way]
- [Source: crates/opencmdb-core/src/trap.rs:38-43 — `TrapId`/`RuleId`, the newtype pattern `FamilyId`
  follows]
- [Source: crates/opencmdb-core/src/trap.rs:55-85 — `Expectation`, the enum the two poles derive from]
- [Source: crates/opencmdb-core/src/trap.rs:219-283 — `Trap::validate` / `TrapFile::validate`; the
  `FamilyEmpty` guard's siblings and the `trim().to_lowercase()` case-folding to reuse (trap.rs:276)]
- [Source: crates/opencmdb-bin/src/trap_gate.rs — `score_corpus` / `Report`, the harness Task 3 extends,
  and the 4.7a `RuleMismatch` shape this mirrors]
- [Source: fixtures/scenario/traps/example.toml:1-3 — "An EXAMPLE of the trap format, not a trap
  family. The families arrive from story 4.9 onward" — the exemption AC4 rests on]
- [Source: _bmad-output/implementation-artifacts/4-7a-trap-runner-asserts-the-rule.md — the structural
  twin: a pure core check surfaced into `Report`, blocking `passed()` as a distinct third count]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.8 — `claude-opus-4-8[1m]`

### Debug Log References

Every claim below is a command that was run, not an inference.

- **Five guards proven to red**, each mutation reverted from a backup:
  | Mutation | Test reddened |
  |---|---|
  | `Trap::validate` drops the `FamilyEmpty` guard | `a_present_but_blank_family_is_refused_while_absent_or_named_is_ok` — `Ok` vs `Err(FamilyEmpty)` on `Some("  ")` |
  | `incomplete_families` lets `MustAbstain` set the not-merge pole | `an_abstention_in_a_family_satisfies_no_pole` — `{must-merge, must-abstain}` reads complete instead of incomplete |
  | `incomplete_families` groups by raw string (no `trim().to_lowercase()`) | `family_grouping_is_case_folded` — `Randomized-MAC`/`randomized-mac` become two one-sided families |
  | `Report::passed()` drops the `incomplete_families.is_empty()` term | `a_one_sided_family_reddens_the_gate_on_its_own` — the `!passed()` assertion (AC5) |
  | `Display` drops the first-line `", J incomplete-famil{y|ies}"` suffix | `a_one_sided_family_reddens_the_gate_on_its_own` — the `"0 truth-table failure(s), 1 incomplete-family"` adjacency (DR2) |

- **Gates:** `cargo fmt --all` clean · `cargo clippy --workspace --all-targets -- -D warnings` clean ·
  `cargo test --workspace` → **106 (bin, +2) + 84 (core, +7) + 42 (xtask), 0 failed**.
- `cargo run -p xtask -- ci` → all gates green; `fixtures` gate green (5 sha256 match, `example.toml`
  byte-unchanged — AC4/AC6), `file-size` largest 884 (trap.rs and trap_gate.rs far under 2000).
  `architecture-views.md` NOT regenerated (`ℹ views-hash STALE`, by design).
- **`Cargo.lock` did not move** and **`fixtures/` is unmodified** (`git status --short` empty for both).
  No dependency added; pure domain code in core + existing bin deps. The MariaDB-backed tests did NOT
  run (`DATABASE_URL` unset) — the counts say nothing about the database.

### Completion Notes List

- **A trap declares an explicit `Option<FamilyId>`; the two poles derive from `Expectation` (AC3).**
  `FamilyId` is the newtype sibling of `TrapId`/`RuleId`; `Trap.family` is `#[serde(default)]` so an
  absent key is `None` (exempt) under `deny_unknown_fields` — no committed `.toml` changed. No
  `polarity` field: the merge/not-merge poles are read off the existing enum.
- **`incomplete_families` is a pure `opencmdb-core` function (D47); the harness only surfaces it.** It
  groups family-declaring traps by the case-folded name (the same `trim().to_lowercase()` fold
  `TrapFile::validate` uses for `DuplicateId`, so cross-file/cross-casing grouping is one family),
  derives each pole from `Expectation`, and returns one `IncompleteFamily` per family missing either
  pole, sorted by the folded key. A `must-abstain` counts for NEITHER pole (DR1), so an abstain-only
  family reports "has neither pole".
- **The gate blocks on a one-sided family, counted separately (AC5).** `score_corpus` retains each
  discovered `Trap` (owned clones — a per-file `TrapFile` local cannot outlive its loop) and calls the
  check once at the end, answer-independently. `Report.incomplete_families` is a THIRD bucket beside
  `failures()` and `rule_mismatches()`; `passed()` requires it empty; `Display` appends a first-line
  `", J incomplete-famil{y|ies}"` count AFTER 4.7a's wrong-rule suffix (fixed, deterministic order),
  then one line per family naming the missing pole.
- **The committed corpus is untouched (AC4/AC6).** `example.toml`'s three traps declare no family →
  exempt → `incomplete_families` empty → the gate stays green, as `the_committed_corpus_…` still
  asserts. The families this guard protects arrive from story 4.9 onward. No `deferred-work.md` entry
  was warranted — nothing new is deferred by 4.7b.
- **`lib.rs` was NOT modified** (correcting the story's Project Structure Notes, drafted before DR): the
  `trap` module is `pub mod trap;` and its items are `pub`, so the harness reaches `FamilyId` /
  `IncompleteFamily` / `incomplete_families` by module path (`opencmdb_core::trap::…`), exactly as it
  already reaches `TrapId`/`RuleId`. Adding a crate-root re-export would be an asymmetric partial export.

### File List

- `crates/opencmdb-core/src/trap.rs` (modified — `FamilyId`, `Trap.family`, `TrapError::FamilyEmpty` and
  `TrapError::FamilyMalformed` + `Display` arms + `validate` guards, `IncompleteFamily`,
  `incomplete_families`, `trap()` helper updated; 9 tests — 7 dev + 2 review)
- `crates/opencmdb-bin/src/trap_gate.rs` (modified — import, `Report.incomplete_families` + accessor,
  `score_corpus` retains traps and runs the check, `passed()`, `Display`; 3 tests — 2 dev + 1 review)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modified — 4-7b status)
- `crates/opencmdb-core/src/lib.rs` — **unchanged** (no crate-root re-export; see Completion Notes)
- `Cargo.lock`, `fixtures/` — **unchanged**, measured

### Change Log

- 2026-07-23 — The trap corpus is checked positive-and-negative. A trap may declare an explicit
  `family`; `incomplete_families` (pure core) groups the family-declaring traps case-folded and reports
  any family missing a decision pole (`must-merge` XOR `must-not-merge`), a `must-abstain` counting for
  neither (DR1). The harness carries these in a third `Report.incomplete_families` bucket that blocks
  `passed()` without touching the truth-table or wrong-rule counts; `Display` names each on the first
  line and its own line (DR2). Family-less traps are exempt, so the committed `example.toml` stays green
  and byte-unchanged. Five guards proven to red.
- 2026-07-23 (review) — Applied 3 review patches: `TrapError::FamilyMalformed` rejects a family name
  with a control char or surrounding whitespace (closing a `Display` newline-injection that defeated
  DR2's first-line grep-safety; proven to red), plus two test-coverage additions — the first-seen
  original-casing path on a one-sided family, and the abstain-only ``has neither pole`` `Display` line.
  Post-review: `cargo test` → 107 (bin) + 86 (core) + 42 (xtask), 0 failed; clippy/fmt/`xtask ci` green;
  `fixtures/` and `Cargo.lock` still unchanged. One finding deferred (pre-existing cross-file trap-id
  fold asymmetry, in `deferred-work.md`).

## Review Findings

_Code review 2026-07-23 (Blind Hunter + Edge Case Hunter + Acceptance Auditor). Acceptance Auditor:
clean pass on AC1–AC6 + DR1/DR2, all 8 traps avoided, all 5 prove-to-red reproduced independently, gate
counts verified. No Critical/High. Findings below._

- [x] [Review][Patch APPLIED] Family name accepted control characters and surrounding whitespace,
      breaking `Display` integrity — [crates/opencmdb-core/src/trap.rs] — added `TrapError::FamilyMalformed`
      + its `Display` arm, and a `Trap::validate` guard rejecting a family name that is not equal to its
      trimmed form OR contains a control character (an internal space stays legal). This closes the
      `"multi\nnic"` newline-injection into the one-line-per-family report and DR2's first-line
      grep-safety. **Proven to red** (`a_family_name_with_whitespace_or_a_control_char_is_refused` reds
      when the guard is dropped).
- [x] [Review][Patch APPLIED] `family_grouping_is_case_folded` did not pin the original-casing claim —
      [crates/opencmdb-core/src/trap.rs] — removed the unbacked clause from that test's docstring and
      added `a_one_sided_family_keeps_the_first_seen_original_casing`: a one-sided family across two
      casings is reported ONCE with the first-seen casing `FamilyId("Randomized-MAC")`, exercising the
      reported-name path the old test never emitted.
- [x] [Review][Patch APPLIED] The abstain-only `Display` line was not pinned by any test —
      [crates/opencmdb-bin/src/trap_gate.rs] — added `an_abstain_only_family_renders_the_neither_pole_line`
      (bin, where `Display` lives): a scratch abstain-only family renders and the test asserts the exact
      ``has neither pole (needs must-merge and must-not-merge)`` string (the struct-level core test does
      not exercise the rendering).
- [x] [Review][Defer] Cross-file trap-id guard is exact, not case/trim-folded — asymmetric with the
      within-file fold [crates/opencmdb-bin/src/trap_gate.rs:207] — deferred, PRE-EXISTING (the `seen`
      map predates 4.7b; this story did not touch it). `TrapFile::validate` folds ids `trim().to_lowercase()`
      for `DuplicateId`, but the cross-file `seen: BTreeMap<TrapId, PathBuf>` matches exactly, so
      `id = "randomized-mac"` in one file and `"Randomized-MAC"` in another are both discovered with no
      error — the same message-confusability harm the within-file fold exists to prevent. Recorded in
      `deferred-work.md`.
- Dismissed (2): reported-name casing is first-seen-wins and thus iteration-order dependent — but the
  walk is deterministic (`discover_trap_files` sorts), so it is stable in practice (cosmetic); and
  `incomplete_families` folding a blank family into a phantom `""` group only bites on UNVALIDATED input,
  which its documented "Assumes validated input" contract excludes (`Trap::validate` fires `FamilyEmpty`
  first inside the harness).
