# Story 6.4b: The mutation driver cannot lie

Status: **ready-for-dev** — contexted 2026-08-26. ⚠️ **ONE ARBITRATION IS OPEN (§0f)** and it
decides what is built.

⚠️ **This story is in NO epic file.** It was created by Epic 6b's retrospective (decision 1,
2026-08-24) and sequenced by Guy the same day; `epics.md` defines stories 6.1–6.19 and not this
one. There are therefore **no epic-level acceptance criteria to inherit** — the criteria below are
derived from the retrospective's success clause and from twelve recorded defects, and that
derivation is what §0 exists to show.

## Story

As the engineer who has to believe a mutation pass,
I want a driver that fails loudly wherever it cannot honestly report,
So that a green result means the guard held, and never that the harness misfired.

## Acceptance Criteria

*(Derived — see the note above. The retrospective's own success clause, `epic-6b-retro-2026-08-24.md`
§7 action 1: **"a driver that exits non-zero when the mutation fails to apply, refuses two filters,
and cannot report a truncated read"**, plus §5 decision 1's fourth requirement, **"anchor
compiler-error detection"**.)*

**AC1 — A mutation that does not APPLY is a hard failure, and it is distinguishable from a
mutation that applied and reddened nothing.** These are the two outcomes the recorded defects
collapse into one another, and *"0 red"* is what both look like from the outside.

**AC2 — The driver never passes two filters to `cargo test`, and never reports a FILTERED run as a
full-suite figure.** It states the command it ran, verbatim, beside every count it prints.

**AC3 — The driver cannot report a truncated read.** No count it prints may come from output it
read through a `head`, a `tail`, or any bounded window.

**AC4 — A status is read from the process, never from a pipeline.** `cmd | grep` yields grep's
status, which is how a commit went in over a red clippy and another over a red suite.

**AC5 — A COMPILE failure is its own outcome**, never a red count and never a green. A mutation
that stops the tree building has measured nothing about any guard.

**AC6 — The driver exercises BOTH cargo targets.** `cargo test` alone cannot see a `#[cfg(test)]`
that has slipped off its item — the cfg is active in the test build — and `cargo build` alone runs
no test.

**AC7 — A mutation is reverted from a SNAPSHOT the driver took, never with `git checkout --`**, and
the driver refuses to start on a tree whose relevant files are already dirty. This gesture has
destroyed uncommitted work four times in this project.

**AC8 — Every property above is measured on a PLANTED driver defect**, not argued. A driver whose
own failure modes are untested is the thing this story exists to stop shipping.

**AC9 — The live count lives in THIS file**, and every figure names the state it was taken against.

**AC10 — What the driver still cannot promise is stated**, in the story and at the code. This is a
tripwire against the ordinary mistake, never a barrier against a determined one (story 5.12's
narrowing, and it applies here as much as to any gate).

---

## §0 — What contexting established

### §0a. 🔴 THERE IS NO DRIVER TO FIX — the action item's own words are wrong

The retrospective's action 4 for Epic 5 read *"Fix the mutation driver once, in `xtask`"*, and its
Epic 6b successor repeats *fix*. **Measured:**

```
git ls-files | grep -i mutat      → (empty)
find . -name "*mutat*" …          → only .claude/worktrees/agent-a254902cb799dd878/mutate.sh
```

The single `mutate.sh` in the tree is inside an **abandoned agent worktree**, tracked by nothing.
🔑 **Every mutation pass this project has run was driven by a throw-away script written into a
scratchpad for that story and deleted with it** — which is precisely why the same defect recurs:
there is no artefact for a fix to land in, and each story re-derives the driver from memory.

**So this story BUILDS rather than repairs**, and the difference matters for its estimate and for
its acceptance criteria: there is no existing behaviour to preserve, and no caller to keep green.

### §0b. THE TWELVE RECORDED DEFECTS, and what each one demands

Every row is a defect this project measured and wrote down. The last three are story 6.4's, from
the week this story was created.

| # | story | the defect | what it demands |
|---|---|---|---|
| 1 | 5.13 | `cargo test --workspace A B` — two filters where cargo accepts one, so **nothing ran** and M6 reported 0 red | AC2 |
| 2 | 5.13b | `head -8` on the output: M7's *"unreachable in a full run"* was FALSE — 18 red | AC3 |
| 3 | 5.13b | M10 recorded as 1 red, measured at **37** — a FILTERED run reported as a full-suite figure | AC2 |
| 4 | 6b.1 | `cargo clippy … \| grep` takes grep's status; a commit went in over a RED clippy | AC4 |
| 5 | 6b.5 | the script mixed a scratchpad restore with `git checkout --`; `app.yml` went back to its last commit and **nine uncommitted keys were lost** | AC7 |
| 6 | 6b.6 | a `sed` replacing a string with itself — a silent no-op reported as 0 red | AC1 |
| 7 | 6b.6 | a `sed` with `\n` in the pattern, which GNU sed does not match across lines — likewise | AC1 |
| 8 | 6b.7 | `shutil.copy2` preserves mtime, so cargo saw a restored template **older** than the artefact: two full runs reported one test red over a clean `git status` | AC1 (the restore must be observable) |
| 9 | 6b.9 | a batch edit writing at the END lost three repairs to one failed anchor, **silently** | AC1 |
| 10 | 6b.11 | my own driver carried defect 1 again — caught only because a CONTROL printed nothing where it owed a result — and `M-D2` said four checks red where there were **eight** (`head -4`) | AC2, AC3 |
| 11 | 6.4 | M-R4's anchor missed after **rustfmt reflowed the code**; the script printed the traceback and CARRIED ON, and the green of an UNMUTATED tree was read as a result | AC1 |
| 12 | 6.4 | a `#[cfg(test)]` slipped off its item: `cargo test` stayed **green**, `cargo build` reddened | AC6 |

🔑 **Every one was caught the same way and only that way: a result contradicted a prediction
written in advance.** The retrospective says so in its own words, and it is why *write the
prediction first* is an action item with *"whoever writes a story"* as owner. ⚠️ **Where no
prediction existed, the defect would have been filed as a confirmation** — which is the strongest
argument for this story and the reason the driver may not simply print numbers.

### §0c. ⚠️ THE COMPILER-ERROR REQUIREMENT IS THE ONE WITHOUT A RECORDED DEFECT

Decision 1's fourth clause is *"anchor compiler-error detection"*, and unlike the other three it
names no incident. What it means, read against practice: several passes have recorded a mutation as
**compiler-carried** (story 6b.3's M12, `E0308`) and several others were CONSTRUCTED so the
compiler would refuse (story 6.1's M4, `E0277` on the `Handler` bound; story 6.4's `E0004` sites).
**Both are legitimate and they are different outcomes**, and a driver that reports *"N tests red"*
for a tree that never compiled has stated something false about every guard in the suite.

⚠️ It is the clause most likely to be under-built, because nothing hurts today. Stated here so the
dev agent does not quietly drop it.

### §0d. WHERE IT CAN LIVE, MEASURED

- `xtask` has **exactly one subcommand**, `ci`; `main.rs:67`'s `match` has an `Ok` arm, an unknown
  arm and a `None` arm. 🔴 **`CLAUDE.md`'s *"(Some other xtask subcommands are still stubs.)"* is
  FALSE** — `grep -rn stub xtask/src/*.rs` is empty. A twin defect, and this story is the one that
  meets it.
- 🔴 **`xtask/src/main.rs` is at 1939 code lines against the 2000 ceiling — 61 lines of headroom.**
  A new subcommand of any substance goes in **its own module**, as `copy_vocabulary.rs` and
  `observed_immutable.rs` already do. `CLAUDE.md`'s rule is *split, not grown*, and this file
  cannot absorb a driver.
- ⚠️ **And `page.rs` is at 1978 — twenty-two lines of headroom.** Not this story's file, but the
  measurement belongs somewhere a next story reads: story 6.4's split bought 182 lines and its
  code-review repair spent 160 of them.

### §0e. WHAT THE DRIVER MUST NOT BECOME

- **Not a gate.** `cargo xtask ci` is nine gates that run on every commit; a mutation pass is run
  deliberately, by a person, during a story. Wiring it into `ci` would make every commit mutate the
  tree.
- **Not a replacement for the prediction.** The retrospective's action 4 stands whatever this story
  builds: *write a prediction before every mutation, and treat a contradicted result as the
  finding*. ⚠️ A driver that made predictions unnecessary would remove the one instrument that
  caught all twelve defects above.
- **Not `cargo-mutants`.** This project's mutations are hand-authored, one per recorded claim, and
  the value is in the PREDICTION and the named carrier. An exhaustive mutation tool answers a
  different question. ⚠️ If the dev agent proposes one, that is a change of scope and belongs to
  Guy, not to the implementation.

### §0f. 🔴 THE ARBITRATION, WITH WHAT EACH OPTION COSTS

**What shape does the driver take?** The retrospective says *"in `xtask`"* and no more.

1. **An `xtask mutate` subcommand over a mutation SPEC file** — the story commits
   `mutations.toml` (id, file, anchor, replacement, prediction), and the driver applies, runs,
   restores and reports each row. ⚠️ Heaviest, and it makes the mutation set an artefact that
   outlives the story — which is either the point or scope creep, depending on Guy's reading.
2. **An `xtask mutate` subcommand over ONE mutation given on the command line** — apply, run,
   restore, report; the story's own notes stay the record. Lighter, and it fixes every recorded
   defect without inventing a corpus format.
3. **A library of helpers rather than a subcommand** — the throw-away scripts stay, and call into
   something that cannot lie. ⚠️ Cheapest, and it keeps the failure mode this story exists to
   remove: a script can still choose not to call the helper.

**Recommendation: (2).** It closes all twelve recorded defects, it needs no new file format, and it
leaves the per-story record where this project already keeps it. Option 1 can be built on top of it
later if a corpus turns out to be wanted; option 3 leaves the hole open by construction.

---

## Tasks / Subtasks

- [ ] **T0 — Take the arbitration (§0f)** before writing code. The three options produce different
      artefacts and the difference is not recoverable cheaply.
- [ ] **T1 — The module and the subcommand** (AC1). Its own file under `xtask/src/`, because
      `main.rs` has 61 lines of headroom — measured, not assumed.
- [ ] **T2 — Apply, and REFUSE when the anchor does not match** (AC1). ⚠️ The refusal must be
      distinguishable in the OUTPUT from a mutation that applied and reddened nothing: story 6b.6
      and story 6.4 both read *"0 red"* off a tree that was never mutated. Verify the applied text
      is present after writing, not merely that the write returned `Ok`.
- [ ] **T3 — Run, and read the status from the process** (AC2, AC4). One filter or none; the
      command echoed verbatim beside every count; no pipeline between the run and its status.
- [ ] **T4 — Read the whole output** (AC3). Parse the counts from the complete text; a bounded read
      is a defect, not an optimisation.
- [ ] **T5 — Both targets, and the compile outcome** (AC5, AC6). `cargo build` AND `cargo test`;
      a tree that does not compile is its own result, printed as such.
- [ ] **T6 — Snapshot and restore** (AC7). The driver takes its own copy before mutating and
      restores from it; it refuses a dirty tree; it never invokes `git checkout`. ⚠️ And the
      restore must be OBSERVABLE — story 6b.7's `copy2` preserved the mtime and cargo then
      believed a stale artefact.
- [ ] **T7 — Plant the driver's OWN defects and measure them** (AC8). One per row of §0b's table
      that the chosen shape can reach: an anchor that cannot match, a no-op replacement, two
      filters, a truncated read, a compile break, a `#[cfg(test)]` slip, a dirty tree. Each must
      make the driver FAIL, and the failure must name which.
- [ ] **T8 — State the residual** (AC10). What a determined author can still do around it.
- [ ] **T9 — The record** (AC9): the live count here; and correct `CLAUDE.md`'s false *"some other
      xtask subcommands are still stubs"* — ⚠️ **a story may not edit `epics.md`, but the twins are
      a different artefact and this one is measurably wrong.**

---

## Dev Notes

### What the previous story leaves you

Story 6.4 (`b6dfd69`, merged 2026-08-26) is the product's first live write gesture. Its own mutation
pass ran on the **unrepaired** driver by Guy's sequencing, and produced rows 11 and 12 of §0b — so
this story's subject was being demonstrated while it waited. Its repair pass ran ten mutations with
predictions written first, and all ten conformed; that is the shape to keep.

### The house rules that bite here

- **`cargo xtask ci` runs NINE gates** and this story adds none. A tenth gate is not what a driver
  is (§0e).
- **`xtask` is a workspace member and a dependency of nobody** (D47). It may not be depended on.
- **No source file over 2000 CODE lines** (D56b) — `main.rs` has 61 left.
- **Prove-to-red** (story 1.3): a guard is observed failing before it passes. ⚠️ For this story the
  guard IS the driver, so AC8 is that rule turned on the instrument itself.
- **The live count lives in the current story's file** (story 6.1's AC8).

### References

- `epic-6b-retro-2026-08-24.md` §5 decision 1 and §7 action 1 — the mandate, and the only place it
  exists.
- `sprint-status.yaml` — Guy's sequencing of 2026-08-24, and story 6.4's closing note, which
  records rows 11 and 12 by name.
- `xtask/src/main.rs:65` — the dispatch; `:107` — `workspace_root`.
- `xtask/src/copy_vocabulary.rs`, `xtask/src/observed_immutable.rs` — the module precedent.

---

## Dev Agent Record

*(to be filled by the dev agent)*

## Change Log

| Date | Change |
|---|---|
| 2026-08-26 | Story created and CONTEXTED. 🔴 **The action item's own verb is wrong: there is NO driver to fix** — `git ls-files` finds nothing, and the only `mutate.sh` in the tree sits in an abandoned agent worktree. Every pass this project ran was driven by a throw-away scratchpad script, which is exactly why the same defect recurs: there is no artefact for a fix to land in. **So this story BUILDS.** 🔑 Twelve recorded defects are tabled with the criterion each one demands, and all twelve were caught the same way — *a result contradicted a prediction written in advance*; where no prediction existed they would have been filed as confirmations. ⚠️ The compiler-error clause is the one requirement with NO recorded incident behind it, and therefore the one most likely to be quietly dropped. 🔴 Measured: `xtask/src/main.rs` is at **1939 of 2000** — 61 lines of headroom, so the driver goes in its own module — and `CLAUDE.md`'s *"some other xtask subcommands are still stubs"* is **FALSE**, there being exactly one subcommand and no stub. **ONE ARBITRATION OPEN (§0f)**: a spec-file corpus, a one-mutation command, or a library of helpers — recommended (2), because (3) leaves the hole open by construction. |
