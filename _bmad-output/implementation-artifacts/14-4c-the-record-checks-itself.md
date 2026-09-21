# Story 14.4c: The record checks itself

Status: done

🔑 **In NO epic file** — created by Epic 14's PARTIAL retrospective (`epic-14-retro-2026-09-19.md`,
actions **A1** and **A2**; Guy, 2026-09-19), sequenced BEFORE 14.5 so the two remaining Epic 14 stories'
reviews benefit from it. Precedent: story 6.4b, created by Epic 6b's retrospective, whose ACs were
likewise DERIVED from the retrospective rather than inherited from an epic.

Baseline: `master` at `365931d` — **1 007 tests** (717 bin + 191 core + 99 xtask), ten `cargo xtask ci`
gates, axe 0 nodes, kbd-probe 59 checks.

## Story

As the person who reviews and merges a story,
I want the story's own RECORD — its live count, what it says it registered, the files it says it
touched — checked by a command rather than by three review layers,
So that review rounds are spent on the code, where they find real defects, and not on arithmetic.

**Guy's words at the retrospective:** *"les trois revues ont coûté cher mais trouvé de vrais défauts."*
Decision 2: **keep the three layers on the CODE; check the RECORD mechanically.**

## ⚠️ Validated 2026-09-19 by two fresh-context layers, and rewritten from §0 to the Tasks

The **fact-check** layer found 3 HIGH, 4 MEDIUM, 6 LOW; the **gap-hunt** layer BUILT the checker twice
(a literal prototype of the first draft's ACs, and one with the closures below) in two scratch repos
and found 6 HIGH, 5 MEDIUM, 3 LOW. 🔴 **The first draft could be satisfied by records that lie in four
measured ways** — a needle matching an OLD register row, a `base:` set one commit up the branch, a prose
"registered" claim the checker never reads, and phantom test names replayed by cargo at column 0 — and
it carried two defects of the very class it exists to close: A2's call site carried by nothing (story
5.12's lesson), and a parse defeated by replayed output (`read_run`'s own documented class). The draft is
not kept beside the rewrite; what it got wrong is recorded here instead.

Corrections of fact: the *names* row is `deferred-work.md:5951`, raised by **14.4b** (`c605ac4`), owned by
*"the next story that touches `xtask/src/mutate.rs`"* — this story; 14.2b only noticed the symptom in
prose (`14-2b…md:1049`) and registered nothing. **6b.7 is NOT a precedent** for "registered with no row":
its own record refutes that finding (`6b-7…md:816-821`, ten rows present); 6b.9 is (`6b-9…md:773`). And
`screens.rs` reached no reviewer **until round D**, where it was read.

## §0 — Why, measured

- **Record defects in 7 of 7 Epic 14 stories.** Story 14.4's round D (26 distinct findings) found defects
  ONLY in the record.
- **Three recurring classes:**
  1. **The live count** — 14.4 AC7 read 990 where the tree shipped 995 (`14-4…md:808`); 14.4b's header
     carried another story's baseline (`14-4b…md:17`); ⚠️ 14.1 still carries TWO live counts, the right
     873 at `:200` and an unstruck 871 at `:575` — a class a block-only checker cannot see (§3).
  2. **"Registered" with no row** — 14.2 §5 (`14-2…md:645`), 14.2b round 1 (`14-2b…md:412`), 14.4 round
     D (`14-4…md:809`), and 6b.9 before them.
  3. **The File List** — 14.4 round D omitted `screens.rs` and `deferred-work.md`, and review slices are
     cut from the File List. 🔴 **Live in the latest merged story too**: 14.4b's diff (`c605ac4`) touches
     `docs/manuals/admin-manual/admin-manual.tex` and its File List omits it (gap-hunt, measured). T6
     corrects that record.
- **`cargo xtask mutate` prints counts, never names** (`deferred-work.md:5951`). In 14.4b five of nine
  first-pass mutations came back one red over prediction, found only by replaying one BY HAND.

## §1 — What exists, measured 2026-09-19 on `365931d`

- **The live count, by three unambiguous invocations** — measured 717 / 191 / 99, exit 0 each, **0.58 s
  warm** together (39.5 s cold, it compiles), no database:
  `cargo test -p opencmdb-bin --bins --locked -- --list` · `cargo test -p opencmdb-core --lib --locked --
  --list` · `cargo test -p xtask --bins --locked -- --list`. 🔴 **Why not `--workspace`**: its target
  names (`Running …`) go to STDERR and its counts to STDOUT, and `mutate.rs::run` (`:379-391`) appends
  stderr AFTER stdout, so only position could pair them; and two targets are both `src/main.rs`. The
  three invocations also drop the doc-test target by construction. ⚠️ `--list` counts `#[ignore]` tests
  and `test result:` does not — zero ignored today (listed = passed); the live count is `--list`'s figure.
- **`xtask/src/mutate.rs`** (923 code lines of 2000): `measure` (`:638-755`) runs for the baseline
  (`:527`) AND the mutated tree (`:593`); `test_results` `:263`, `read_run` `:286`. 🔴 **A failing test's
  captured output is replayed at column 0**, so `test X ... FAILED` can be spoofed — measured: a naive
  grep yielded **4 names, 2 of them phantoms**. The honest source is the indented list after the LAST
  `failures:` line before each target's `test result:` line, whose length must equal that line's
  `failed` field. 🔴 **No test asserts on the driver's stdout** — the only end-to-end
  (`the_driver_drives_a_real_cargo_run_end_to_end`, `:1342`) checks the exit code and the restore — so
  anything only PRINTED is carried by nothing.
- **`xtask/src/main.rs`** dispatches at `:88-113` (`mutate` arm `:100-106`); the usage strings at `:109`
  and `:114` must gain `record`. `the_module_doc_lists_exactly_the_gates_run_ci_reports` (`:1757`) pins the
  GATE list — a subcommand is not a gate and must not be written as one.
- **`git diff --name-only`** shows a rename as its NEW path only (`--no-renames` shows both), and a file
  changed then reverted inside the branch disappears (the diff is NET). It reads COMMITS; `--list`
  compiles the WORKING TREE — the three checks read different trees unless the tree is clean.
- **`deferred-work.md`**: an EDIT to an existing row shows as a `-- …`/`+- …` pair (measured: `031e2d7`
  4 added / 3 removed, three being old rows re-marked ✅; `c605ac4` 9 added, one struck in place), and a
  registration can be an indented continuation (`+  ✅ **ANSWERED …`). **Counting `+- ` lines counts
  edits as registrations.** Register rows wrap at ~100 columns: a literal `contains` of a real row's
  phrase is FALSE, whitespace-normalised TRUE (story 5.14b's split-needle trap).
- **After the squash merge the check is meaningless** — on `master` the merge-base with `master` is
  `HEAD` and the diff is empty. It runs on the story branch's last commit before the merge.

## §2 — Decisions T0 must take with Guy (recommendations first, revised by the validation)

1. **The convention.** *(a, recommended)* ONE `## Record` block (a line equal to `## Record`, outside
   code fences, exactly once — the template's `## Dev Agent Record` must not match), one entry per
   physical line, every line keyed, any other line → exit 2:
   `live-count: bin=717 core=191 xtask=99` · `base: <sha>` · `registered: <phrase>` (one per row ADDED)
   · `file: <path>` (one per touched file). **The live count lives ONLY in the block**; prose cites it.
   *(b)* Heuristics over prose — refused-by-recommendation (wrong in both directions).
2. **Where it runs.** *(a, recommended)* `cargo xtask record <story-file>`, run on the story branch's
   LAST commit before the merge (by the developer at the close, and again after every review repair),
   NOT a `cargo xtask ci` gate (historical story files carry no block; CI's `checkout@v5` fetches depth 1
   and has no base). *(b)* Add now a presence tripwire in `githooks/pre-push`: a story file changed on
   the branch and created after 14.4c must contain `## Record` — cheap, no build. Recommended as
   **registered, not built**, so the story stays one deliverable.
3. **The base.** *(a, recommended)* The checker COMPUTES `git merge-base HEAD master` and reds if `base:`
   differs; a DIRTY tree (tracked changes or non-ignored untracked files) → exit 2; the diff is
   `--no-renames` and NET. Stated limit: a branch that merges `master` into itself moves the merge-base —
   rebase, do not merge.
4. **A2.** *(a, recommended)* Names read from each target's `failures:` list, their count checked against
   that target's `failed` field (a mismatch is a refusal to NAME, printed as such, never a guess), and
   CARRIED IN THE RETURNED VALUE so a test can assert them; the existing end-to-end asserts the planted
   test is named. Exit contract unchanged. *(b)* Also diff the baseline's red set against the mutated
   run's — registered.

## §3 — What the checker does NOT close, stated so nobody reads it as closed

- **A prose claim.** "We registered three rows" with no `registered:` line and no row exits **0** —
  measured. The checker refuses to read prose, rightly (*a guard that greps a file greps its prose*), so
  class 2 is closed only for claims written in the block. What it adds: it ALWAYS PRINTS the rows the
  branch added, so a reviewer compares a list instead of believing a sentence.
- **Registrations outside `deferred-work.md`** — GitHub issues, retrospective tables, sprint-status
  notes.
- **A second live count elsewhere in the file** (14.1's case).
- **A story file without a block** — until §2.2(b)'s tripwire exists.

## Acceptance Criteria

**AC1 — `cargo xtask record <story-file>`** follows the driver's contract: **0** the record matches the
tree, **1** it does not (each mismatch named), **2** it could not honestly run — no `## Record` block or
more than one, a line in the block that parses as nothing, a dirty tree, `base:` ≠ the computed merge-base,
an unreadable file, a failed build.

**AC2 — the live count** is compared target by target with the three `-p … -- --list` invocations; a
mismatch names the target and both numbers. No database.

**AC3 — registrations.** Each `registered:` phrase, whitespace-normalised, is ABSENT from
`deferred-work.md` at the base and present EXACTLY ONCE at `HEAD`; and the number of `registered:` lines
EQUALS the number of rows the branch ADDED (new bullets, not edited ones). The checker prints every added
row it found.

**AC4 — the File List** (`file:` lines) equals `git diff --no-renames --name-only <base>...HEAD`, in BOTH
directions.

**AC5 — A2**: a red run names its red tests from the `failures:` lists, for the baseline and the mutated
run; the names are in the returned value; a unit test over a captured output with a failing test whose
OWN output mimics a `test … FAILED` line yields no phantom; the end-to-end test asserts the planted test is
named; every existing `xtask mutate` test passes.

**AC6 — prove-to-red, end to end through the subcommand**, predictions first, six plants: a wrong count;
a `registered:` phrase absent from the register; a File List missing a touched file; `base:` = `HEAD~1`;
a `registered:` phrase matching an OLD row; and, for A2, a replayed phantom. The subcommand's core takes
the target table and the target directory as PARAMETERS (passed to child commands with `Command::env`,
never the process environment — the mutate end-to-end already mutates `CARGO_TARGET_DIR` globally).

**AC7 — the convention is where the next story meets it**: this story's own `## Record` block, checked
(exit 0); the practice (§2.2) in `CLAUDE.md` and `docs/project-context.md`; `deferred-work.md:5951`
closed; 14.4b's File List corrected (`admin-manual.tex`).

**AC8 — THE LIVE COUNT lives in this story's `## Record` block and is CHECKED by this story's
subcommand.**

**AC9 — no regression**: ten gates, `clippy --all-targets -D warnings`, `RUSTFLAGS="-D warnings"`, both
store conditions, `cargo deny`; the browser gates are NOT touched (no product change), said as such.

## Tasks / Subtasks

- [x] **T0** Take §2's four decisions with Guy — ✅ **all four (a), 2026-09-19**: one `## Record` block;
      a subcommand with the `pre-push` tripwire REGISTERED not built; the base computed and checked;
      names from the `failures:` lists, carried in the returned value.
- [x] **T1** (AC5) A2 in `mutate.rs`: names from the `failures:` lists, count-checked, carried in the
      returned value; the phantom unit test; the end-to-end assertion. Predict, then mutate the CALL SITE.
- [x] **T2** (AC1) `xtask/src/record.rs` (a NEW module): the block parser (one block, keyed lines,
      refusals), the dirty-tree and merge-base refusals; dispatch and usage in `main.rs`.
- [x] **T3** (AC2) The three `-- --list` invocations, with the target table a parameter.
- [x] **T4** (AC3, AC4) Registrations against the base and HEAD registers; the net, no-renames File List.
- [x] **T5** (AC6) Six plants driven end to end through a seam taking the target table and directory.
- [x] **T6** (AC7, AC8) This story's `## Record` block, checked; the twins; row 5951 closed; 14.4b's
      File List corrected.
- [x] **T7** (AC9) Both store conditions; ten gates; clippy; `cargo deny`.

### Review Findings — three isolated layers, 2026-09-19/20

39 raw findings → 25 distinct: **1 decision, 18 patches, 2 deferrals, 4 dismissed with their check.**
(Recounted against the list below before it was written: the first draft of this line said 24 and 17.)
Two layers MEASURED the same thing independently: **the checker still accepts registers that lie.**

- [x] [Review][Decision] **A register row is identified by its first `**bold title**`** (Guy, 2026-09-20)
  — a title survives closing, striking and appending, where the row's TEXT does not. A row is NEW when
  its title is absent at the base; every `registered:` phrase must fall in a new row; every new row must
  be claimed exactly once; a title that DISAPPEARS is an error, because this register strikes rows and
  never deletes them. Refused: identity by normalised text with deletions forbidden (closing a row
  changes its text, so every closure would read as a deletion); and printing without judging.
- [x] [Review][Patch] the net row count let a DELETED row mask an unclaimed added one (blind + edge, both measured exit 0)
- [x] [Review][Patch] a `registered:` phrase could claim an EDITED OLD row while the new row went unclaimed (blind + edge + auditor, measured)
- [x] [Review][Patch] **panic (exit 101) printing a row whose byte 140 falls inside a multibyte character** (blind + edge, measured `é` at 139..141) — the exit contract broken by the listing itself
- [x] [Review][Patch] duplicate `live-count:` / `base:` lines: the last one wins silently, so the block can carry two counts (blind + auditor + edge, measured exit 0)
- [x] [Review][Patch] a story file OUTSIDE the repository (absolute path) validated a lying record in it (edge, measured exit 0)
- [x] [Review][Patch] `git show` failures on the register were swallowed (`unwrap_or_default`), so a moved register checked nothing (blind)
- [x] [Review][Patch] `git diff --name-only` quotes non-ASCII paths: an honest `file: docs/café.md` could never match (blind + edge, measured)
- [x] [Review][Patch] `base:` accepted a ref (`master`, `HEAD`), which records no commit (blind + edge)
- [x] [Review][Patch] two guards of `registration_mismatches` were carried by no test — mutating either left 109/109 green (edge, measured); the one-letter probe reddened through another rule
- [x] [Review][Patch] `git status --porcelain` honours `status.showUntrackedFiles=no` (blind)
- [x] [Review][Patch] an inherited `GIT_DIR`/`GIT_WORK_TREE` would retarget every git call (blind)
- [x] [Review][Patch] the end-to-end plants asserted the exit CODE only, never WHICH rule fired — X4's own lesson (auditor)
- [x] [Review][Patch] the messages said *"the File List"* for the block's `file:` lines, which is not the prose list (blind)
- [x] [Review][Patch] `TARGETS` and `from_args` were reached by no test (auditor + blind) — 6.4b's own shape
- [x] [Review][Patch] a weakened assertion: the `17` test ignored `names` entirely (blind)
- [x] [Review][Patch] AC9's *"see the final measurement below"* pointed at nothing (auditor)
- [x] [Review][Patch] 🔴 **the story file itself had lost §2-§3, its ACs, its tasks and its Dev Notes** — an edit anchored on the STRING `## Dev Agent Record` matched the phrase quoted inside §2.1, the very trap §2.1 names; `cargo xtask record` exited **0** over it, which is §3's stated limit made concrete (auditor)
- [x] [Review][Patch] the block's syntax was documented only in `record.rs` (auditor)
- [x] [Review][Defer] `red_names` does not qualify a name by its target, so `tests::x` in two crates is ambiguous (blind + edge)
- [x] [Review][Defer] the mutate end-to-end still sets `CARGO_TARGET_DIR` process-wide (`unsafe set_var`), which no `Command::env` can undo (blind) — pre-existing, story 6.4b's
- Dismissed with their check: a phantom on REAL cargo output (edge planted four spoofs over a two-crate
  workspace — `Named` carried none, and a well-formed phantom `test result:` makes `read_run` refuse
  before naming); the snapshot test's path is the one the code builds (edge read `replace('/', "%")`);
  CRLF and a `~~~` fence are refusals, never a lie that passes (edge, measured); `--list` counting
  `#[ignore]` is already stated in §1.

## Dev Notes

### Traps this project has paid for, and which apply here

- 🔴 **A gate whose helpers are tested and whose BODY is not is carried by nothing** (5.12; 6.4b's
  `from_args` reached by no test) — AC6 drives the subcommand end to end.
- 🔴 **A guard that greps a file greps its prose** (14.2, 14.2b, 14.4): the checker reads ONLY the
  `## Record` block, never the story's narrative — or the paragraph explaining a defect satisfies it.
- 🔴 **Read an exit status from a FILE, never through a pipe** (6b.1, 6b.10, 6.4b).
- ⚠️ `cargo test -- A B` runs two filters; `--list` must be passed after `--` and ALONE, or the counts
  are of a filtered set (6.4b's refusal).
- ⚠️ A test that sets a process-global env var (`CARGO_TARGET_DIR`) races every other such test — pass
  it to the child with `Command::env`.
- ⚠️ `xtask/src/main.rs` carries the module-doc gate list pinned by
  `the_module_doc_lists_exactly_the_gates_run_ci_reports` — a new SUBCOMMAND is not a gate and must not
  be added to that list (6.5's test would red for the right reason).
- ⚠️ `deferred-work.md` rows are multi-line bullets that wrap at ~100 columns: normalise whitespace on
  both sides, and count a row as ADDED only if it is new — an edited row is a `-`/`+` pair (§1).

### References

- `epic-14-retro-2026-09-19.md` §3, §5 (decision 2), §6 (A1, A2).
- `xtask/src/mutate.rs` `:209-337` (`apply`, `compiler_error`, `test_results`, `read_run`), `:638-762`
  (`measure`, to `:755`), `:887` (`USAGE`), `:1342` (the end-to-end); `xtask/src/main.rs` `:88-113`, `:1757`.
- Story 6.4b (`6-4b-mutation-driver-cannot-lie.md`) — the exit contract and its refusals.
- Memory: *the File List is load-bearing*; *a watch is not a result*.

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context), 2026-09-19.

### Debug Log References

**The mutation pass — predictions written to a file BEFORE any run** (`cargo xtask mutate --baseline`,
the store dropped and recreated before each):

| id | mutation | predicted | measured |
|---|---|---|---|
| X1 | `red_names` never refuses a short list | red:1 | **red 2** ❌ — the second carrier is `a_test_that_prints_a_diagnostic_is_not_a_compile_failure`, whose synthetic output has no `failures:` list and expects `Unnamed`: a legitimate carrier the prediction missed |
| X2 | `read_run` carries no names | red:2 | red 2 ✅ |
| X3 | a `base:` that is not the branch point is accepted | red:1 | red 1 ✅ |
| X4 | a phrase already in the base register is accepted | red:2 | **red 1** ❌ — the END-TO-END plant wrote `an old row …` where the row reads `An old row …`, so it reddened through the *"in no row"* rule and not the one it names; repaired (case kept) |
| X4b | the same, with the plant repaired | red:2 | red 2 ✅ |
| X5 | fewer `registered:` lines than added rows is accepted | red:1 | red 1 ✅ |
| X6 | the File List compared in one direction only | red:1 | red 1 ✅ |
| X7 | a dirty tree is not refused | red:1 | red 1 ✅ |
| X8 | the snapshot written BEFORE the baseline again | red:1 | red 1 ✅ |

🔑 **Both contradictions were diagnosed from the driver's own `red:` lines, with no replay by hand** —
the deliverable A2 working on its own story's first pass.

🔴 **The pass found a defect in the DRIVER before it found anything else.** Its first run was refused at
the baseline (no `DATABASE_URL`: the driver requires a store, by 6.4b's decision, and says so) — and that
refusal LEFT THE SNAPSHOT ON DISK, because the snapshot was written before the baseline. Every following
run then refused with *"a snapshot from an earlier run is still here … may still be MUTATED"* over files
never touched (compared byte for byte before deleting them). Seven refusals in a row. Fixed — the snapshot
is written immediately before the file is mutated, which is the only window it recovers — and pinned by
`a_refusal_before_the_mutation_leaves_no_snapshot_behind`, which X8 reds.

⚠️ **And the second run was refused for a REAL reason**: the baseline was red on `clippy --all-targets`
(a needless `mut` in the record end-to-end), invisible to `cargo test`. The driver's refusal is what caught
it; it left no snapshot this time.

### Completion Notes List

- **What shipped.** `cargo xtask record <story-file>` (`xtask/src/record.rs`): the `## Record` block, the
  live count by three `-p … -- --list` invocations, registrations against the base and HEAD registers
  (absent at the base, one row at HEAD, net count equal), the File List against
  `git diff --no-renames --name-only <merge-base>...HEAD` both ways, refusals for a dirty tree and a `base:`
  that is not the branch point; `0` / `1` / `2`. `xtask mutate`: `Outcome::Red { names }`, read from
  cargo's `failures:` lists and count-checked; `measure` prints `red: <test>` from that value; the
  stale-snapshot defect fixed.
- **AC6's plants**, end to end through `check` over a scratch repository and crate: a wrong count, a
  registration with no row, a File List missing a touched file, a phrase naming an OLD row (each `1`), a
  `base:` one commit up the branch, no block, a dirty tree (each `2`); the honest record `0`. A2's phantom
  lives in `mutate.rs`'s unit test.
- **AC7**: this story's `## Record` block below, checked; the practice in `CLAUDE.md` and
  `docs/project-context.md`; `deferred-work.md:5951` closed; 14.4b's File List corrected; two rows
  registered (the `pre-push` tripwire, the baseline-vs-mutated red-set comparison).
- **AC8 — THE LIVE COUNT is the block's `live-count:`** — checked by `cargo xtask record` on this branch.
- **AC9 — the measurement**, `cargo test --workspace --locked`, wall clock, one warm run first:
  **1 018 tests** (717 bin + 191 core + 110 xtask) — **24.4 s** against a DROPPED-AND-RECREATED
  `mariadb:10.11.11` (port 13419) and **5.8 s** with `DATABASE_URL` unset; ten `cargo xtask ci` gates,
  `cargo fmt --check`, `clippy --workspace --all-targets --locked -D warnings`,
  `RUSTFLAGS="-D warnings" cargo test`, `cargo deny check` — all green. ⚠️ The browser gates are NOT
  touched by this story and were not run: it ships no product change, and saying so is AC9's own letter.
  ⚠️ The figures above are the REVIEW's; at implementation the story measured 1 017 (xtask=109).
- ⚠️ **What the operator gains: nothing** — tooling; no route, no screen, no migration. The browser gates
  are not touched and were not run, by AC9's own letter.

## Record

- live-count: bin=717 core=191 xtask=110
- base: 365931d07fa99062b7f4241adf0cc8c7953d956c
- registered: A story file without a `## Record` block is checked by nothing
- registered: names the red tests of each run but does not COMPARE
- registered: does not qualify a red test by its TARGET
- registered: still sets `CARGO_TARGET_DIR` process-wide
- file: CLAUDE.md
- file: _bmad-output/implementation-artifacts/14-4b-releasing-an-address.md
- file: _bmad-output/implementation-artifacts/14-4c-the-record-checks-itself.md
- file: _bmad-output/implementation-artifacts/deferred-work.md
- file: _bmad-output/implementation-artifacts/sprint-status.yaml
- file: docs/project-context.md
- file: xtask/src/main.rs
- file: xtask/src/mutate.rs
- file: xtask/src/record.rs

### File List

The `## Record` block's `file:` lines are this story's File List (checked by `cargo xtask record`).

### Change Log

- 2026-09-19 — Contexted, validated by two fresh-context layers (rewritten from §0), T0 taken with Guy
  (all four (a)), implemented, mutation-passed (9 ids + X4b); status → `review`.
- 2026-09-20 — Code review (three isolated layers): 1 decision by Guy (a register row is its bold
  title), 18 patches applied, 2 deferrals, 4 dismissed with their check. Stays `review` until the merge.
- 2026-09-20 — Merged: PR #190 squash-merged as `4c195f0`, CI green on the head `c12ce37` itself. Status → `done`.
