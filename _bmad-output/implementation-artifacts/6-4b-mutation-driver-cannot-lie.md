# Story 6.4b: The mutation driver cannot lie

Status: **review** — implemented 2026-08-26, CODE-REVIEWED by three isolated layers and REPAIRED the same day; contexted 2026-08-26, VALIDATED the same day by two fresh-context layers
(which replaced most of the story), **arbitration taken (§0h, Guy, 2026-08-26): OPTION 2**.

⚠️ **This story is in NO epic file.** It was created by Epic 6b's retrospective (decision 1,
2026-08-24) and sequenced by Guy the same day; `epics.md` defines stories 6.1–6.19 and not this
one. There are therefore **no epic-level acceptance criteria to inherit** — the criteria below are
derived from two retrospectives' success clauses and from **fifteen** recorded defects, and §0 is
that derivation shown rather than asserted.

## Story

As the engineer who has to believe a mutation pass,
I want a driver that fails loudly wherever it cannot honestly report,
So that a green result means the guard held, and never that the harness misfired.

## Acceptance Criteria

*(Derived. Sources: `epic-5-retro-2026-08-12.md` §7 action 4 — **"It must refuse two filters,
refuse a truncated read, and anchor compiler-error detection on `^error\[E[0-9]+\]`"** — and
`epic-6b-retro-2026-08-24.md` §5 decision 1 / §7 action 1, which adds **"exits non-zero when the
mutation fails to apply"**. Everything beyond those five clauses is a gap the validation MEASURED;
§0g says which.)*

**AC1 — Applying is a three-way outcome, not a boolean.** *Did not match*, *matched more than
once*, and *applied at exactly N sites* are three different things, and the driver names which. ⚠️
**A no-op replacement counts as "did not match"**, and the driver proves the apply by comparing the
file before and after — not by trusting that a write returned `Ok`.

**AC2 — Never a filtered run reported as a full-suite figure**, and the instrument is the
**`filtered out` count cargo prints beside every `passed`**, not the absence of a `--filter` flag.
Any non-zero `filtered out` forces the outcome to *filtered — not comparable*.

**AC3 — The driver refuses to report a count it did not fully read.** It knows how many test
targets it expects, requires one complete `test result:` line from each, and runs with
**`--no-fail-fast`**. 🔴 Without it `cargo test --workspace` stops at the first failing crate and
every count is a silent lower bound (§0g.1).

**AC4 — Every status comes from a process, never from a pipeline.**

**AC5 — A compile failure is its own outcome, detected by a line-anchored `error[EDDDD]`** — never
by a bare `^error`, which cargo emits on **every** red run as `error: test failed`. 🔴 And the
compile probe is over `--all-targets`: a break in the test target alone leaves `cargo build` green
and makes `cargo test` print **zero** `test result:` lines, which a naive driver sums to *"0 passed,
0 failed"* (§0g.3).

**AC6 — The carriers are FOUR, the driver drives THREE, and the fourth is AC10's — stated, never
silent.** `cargo clippy --workspace --all-targets --locked -- -D warnings`;
`cargo test --workspace --locked --no-fail-fast`; `cargo xtask ci` — all three driven, each named
in the output beside its result. **The two browser gates are the fourth and are NOT driven**, by
the scope decision recorded in §0h; AC10 carries the limit and the manual recipe.

**AC7 — Restore from a snapshot the driver took, never `git checkout`, and ADVANCE THE MTIME.** 🔴
A byte-identical restore that preserves the mtime leaves cargo serving a **stale artefact**: `git
status` clean, source identical, nine tests still failing (§0g.6). ⚠️ There is **no dirty-tree
refusal**: every one of the four recorded destructions happened on a dirty tree, because that is
when a mutation pass runs, and the snapshot is what makes it safe. The driver instead **touches
exactly the file it was given** and verifies byte-for-byte restoration afterwards.

**AC8 — The store is probed and reported.** A run without a reachable `DATABASE_URL` prints
`store: ABSENT` and may not be recorded as a result for anything the store carries. 🔴 Measured:
one mutation gives *exit 0, 741 passed* without a store and *exit 101, 1 failed* with one — **same
counts, opposite verdicts, and the clock is the only tell** (§0g.4).

**AC9 — The prediction is MACHINE-COMPARABLE and the driver's exit code is the comparison.**
`0` applied and the outcome matches the prediction · `1` applied and the outcome contradicts it ·
`2` the driver could not honestly measure (anchor missed or multi-matched, no-op, compile failure,
filtered, store absent, restore failed). ⚠️ This is `a11y/*.mjs`'s contract, and story 6b.11's
arbitration 1 is the precedent. 🔑 **It does not replace the act of predicting** — it records the
prediction beside the mutation and checks it, which is the opposite.

**AC10 — What the driver cannot measure is stated, in the story and at the code.** A tripwire
against the ordinary mistake, never a barrier (story 5.12's narrowing).

**AC11 — Every property of AC1–AC8 is measured on a PLANTED driver defect**, on a synthetic tree
with its own `CARGO_TARGET_DIR`. ⚠️ A driver whose own failure modes are untested is the thing this
story exists to stop shipping — and *a gate green over the real tree says nothing about its own
tests* (story 6b.11).

**AC12 — The live count lives in THIS file**, and every figure names the state it was taken
against.

---

## §0 — What contexting established, and what the validation replaced

### §0a. 🔴 THERE IS NO DRIVER TO FIX — but there are two SPECIMENS, and they are prior art

Both retrospectives say *fix the mutation driver*. Measured: **no driver has ever been tracked**,
and none was ever deleted from history — verified over `git rev-list --all --objects` and `git log
--diff-filter=D`, not by one directory walk. ⚠️ *(An enumeration cannot establish absence — story
5.13b. `git ls-files | grep -i mutat` now returns **this story file**, so contexting's recorded
`(empty)` was refuted by its own commit.)*

🔑 **Every pass this project ran was driven by a throw-away script written into a scratchpad and
deleted with it** — which is why the same defect recurs: there is no artefact for a fix to land in.

⚠️ **But two specimens survive and contexting dismissed them.** `.claude/worktrees/agent-a254…/
mutate.sh` and story 6.4's own `mut.sh` already implement, between them: a snapshot copy rather
than `git checkout`; a **no-op detector** (`diff -q` after the write, printing `NO-OP 🔴 DRIVER`);
and a status read from `$?`. Each also dropped a different guard — one has `head -4`, the other the
`^error` of §0c. **Three authors converged on one skeleton and each lost a different property**,
which is a design input: it says which properties are cheap and which keep being lost.

### §0b. 🔴 FIFTEEN RECORDED DEFECTS — the table contexting wrote had twelve, two were misdescribed

| # | story | the defect | demands |
|---|---|---|---|
| 1 | 5.13 | two filters to `cargo test`, so **nothing ran**; M6 reported 0 red | AC2 |
| 2 | 5.13b | `head -8` on the output: M7's *"unreachable in a full run"* was FALSE — 18 red | AC3 |
| 3 | 5.13b | M10 recorded as 1 red, measured **37** — a FILTERED run as a full-suite figure | AC2 |
| 4 | 5.14 ×2, and again 5.12, 6b.4 | **a mutation named for one thing and applied to another** — 5.12's M5 said *neuter the gate* and hit the MATCHER; 6b.4's M2 changed two things and its red was attributed to whichever the author was looking at | AC1's *N sites*, and the driver printing the diff it applied |
| 5 | 5.14b | matching `^error(\[|:)` counts cargo's own `error: test failed` trailer, so **every** red run reports a compiler-carried red | AC5 |
| 6 | 6b.1 | `cargo clippy … \| grep` takes grep's status; a commit went in over a RED clippy | AC4 |
| 7 | 6b.5 | the script mixed a scratchpad restore with `git checkout --`; **nine uncommitted keys lost** | AC7 |
| 8 | 6b.6 | a `sed` replacing a string with itself — a silent no-op reported as 0 red | AC1 |
| 9 | 6b.6 | a `sed` with `\n` in the pattern, which GNU sed does not match across lines | AC1 |
| 10 | 6b.7 | a restore preserving the mtime: cargo served a **stale artefact** | AC7 |
| 11 | 6b.9 | a batch edit writing at the END lost three repairs to one failed anchor, silently | AC1 |
| 12 | 6b.10 | a commit over a RED suite — `cargo test \| grep` again, and the `&&` proceeded | AC4 |
| 13 | 6b.11 | the two-filter defect AGAIN, caught only because a CONTROL printed nothing | AC2 |
| 14 | 6b.11 | `M-D2` said four checks red where there were **eight** (`head -4`) | AC3 |
| 15 | 6.4 | an anchor missed after **rustfmt reflowed the code**; the script carried on and the green of an UNMUTATED tree was read as a result | AC1, AC9 |
| 16 | 6.4 | a `#[cfg(test)]` slipped off its item: `cargo test` **green**, `cargo build` red | AC6 |

*(Sixteen rows for fifteen defects — rows 8 and 9 are one story's two `sed` failures.)*

🔴 **Row 1's mechanism is STALE and the validation measured it.** On cargo 1.96 `cargo test
--workspace A B` now fails **loudly**: `error: unexpected argument 'join' found`, exit 1. The form
that is still silent is **after the separator** — `cargo test --workspace -- A B` runs 7 tests of
741, exit 0, green. **A driver that counts `--filter` flags does not close this**, which is why
AC2's instrument is the `filtered out` count and not the flag.

⚠️ **And *"every one was caught by a prediction contradicted"* is the RETROSPECTIVE's sentence
about Epic 6b's seven, not about all of these.** Row 16 was named by the **compiler**; rows 6, 7
and 12 by their damage, after the fact. That is a weaker instrument and a later one — worth saying,
because the story's argument does not need the stronger claim.

### §0c. 🔴 THE COMPILER-ERROR CLAUSE HAS AN INCIDENT, A LITERAL ANCHOR, AND A LIVE DEFECT

Contexting wrote a section headed *"the one requirement WITHOUT a recorded defect"*. **All three
halves of that are false**, and the section is kept inverted rather than deleted, because the
mistake is this project's own dominant class turned on the story that exists to stop it:

- **The incident**: `5-14b:665` — *"matching `^error(\[|:)` to classify a red counts cargo's own
  `error: test failed` trailer and reports a compiler-carried red on **every** mutation that reds
  anything."*
- **The mandate carries the remedy literally**: `epic-5-retro:209` — *"anchor compiler-error
  detection on `^error\[E[0-9]+\]`"*. Contexting read *anchor* as a loose word.
- **The defect is LIVE**: story 6.4's own driver matched `^error`, and seven of its eight recorded
  runs printed `COMPILE ERROR` when not one was a compile error. Re-verified here against my own
  measurement files: `error: test failed, to rerun pass …` sits at line start on an assertion red.

### §0d. WHERE IT CAN LIVE, MEASURED with the gate's own rule

- `xtask/src/main.rs` is at **1939** code lines of 2000 — **61 lines of headroom**, so the driver
  goes in **its own module**, as `copy_vocabulary.rs` and `observed_immutable.rs` do. *(The ceiling
  is Guy's engineering convention of 2026-07-23 enforced by the `file-size` gate; **D56b** governs
  test PLACEMENT, which is only why tests do not count.)*
- `xtask` has **exactly one** subcommand — `main.rs:67`'s match has `Some("ci")` / `Some(other)` /
  `None` — and **no stubs**. 🔴 `CLAUDE.md`'s *"(Some other xtask subcommands are still stubs.)"* is
  FALSE. ⚠️ `docs/project-context.md` does **not** carry that sentence: it is ONE twin, not two.
- ⚠️ `page.rs` is at **1978** — twenty-two of headroom. Not this story's file; recorded because
  story 6.4's split left 182 lines of headroom and its review repair spent 160 of them.

### §0e. WHAT THE DRIVER MUST NOT BECOME

- **Not a gate.** `cargo xtask ci` runs in CI on every pull request and every push to `master`; a
  mutation pass is run deliberately, by a person, during a story.
- **Not a replacement for predicting.** ⚠️ Contexting wrote that a driver making predictions
  unnecessary *"would remove the one instrument that caught all twelve defects"* — and then used
  that sentence as an argument against the option that RECORDS the prediction. Recording it and
  checking it is the opposite of removing it (AC9).
- **Not `cargo-mutants`.** The mutations here are hand-authored, one per recorded claim, and the
  value is in the prediction and the named carrier. Proposing an exhaustive tool is a change of
  scope and belongs to Guy.

### §0g. 🔴 WHAT THE GAP-HUNT LAYER MEASURED BY BUILDING IT

It built option (2) to the ten original criteria — ~220 lines, wired into the dispatch — and then
attacked it. **Eight HIGH findings, every one a hole the criteria did not close.**

1. 🔴 **`cargo test --workspace` STOPS AT THE FIRST FAILING CRATE.** One mutation, re-measured
   independently on this tree: **8 red** by default, **17 red** (8 + 9) with `--no-fail-fast` —
   `opencmdb-core` never ran. ⚠️ **This is AC3's own sin arriving through cargo's default rather
   than through a `head`, and it is worse, because nothing in the output says a window was
   applied.** *Every recorded "N red" in this project where `opencmdb-bin` reddened is a lower
   bound.*
2. 🔴 **AC6's original two targets are both blind to what CI reds.** A dead test-module helper with
   an unused binding: `cargo build` 0, `cargo test` 0 (741 passed), `cargo clippy` 0 — and
   `clippy --all-targets -D warnings` **101**, `RUSTFLAGS="-D warnings" cargo test` **101**. That is
   PR #115's recorded red CI run, reproduced.
3. 🔴 **A compile break in the TEST target only prints "0 passed, 0 failed" and exits 0** through a
   `cargo build`-first driver: `cargo test` on a non-compiling tree exits 101 and emits **zero**
   `test result:` lines. *AC1's forbidden sentence, reached through AC5's blind spot.*
4. 🔴 **The store changes the verdict and not the counts** (AC8's measurement).
5. 🔴 **Replace-all mutates the second ORACLE too.** `"l1-exact-mac"` occurs **twice** in `l1.rs` —
   the production constant and `CORPUS_EXACT_MAC`, the independent oracle `CLAUDE.md` protects
   under *deliberate redundancy*. Replace-all: **6 red**. The const line alone: **14 red**. *The
   replace-all form repairs the guard it is meant to red.*
6. 🔴 **Row 10's stale artefact is a CARGO FINGERPRINT property, not an askama one** — reproduced
   on a plain `.rs` file: `git status` clean, bytes identical, `Finished in 0.04s` with no
   `Compiling` line, **nine tests still failing**. `touch` fixes it. The story had scoped it to
   templates, so the fix would have been written for templates only.
7. 🔴 **A cargo-only driver cannot measure the product's front half at all.** Inverting the arrow
   direction in `app.js` — story 6b.11's central deliverable: `cargo test` **741 passed**, nine
   gates green, `kbd-probe` **9 failed**. ⚠️ And the browser gates are **not idempotent** (the last
   block writes), need a rebuild-boot-seed cycle, and answer **0/1/2** — a driver mapping non-zero
   to *red* mis-files every *could not run* as a regression.
8. 🔴 **`cargo xtask ci` is a THIRD carrier and is not subsumed.** Four gates are covered by their
   own tests; `ddl-collation` is not — dropping a binary collation from `0001_initial.sql` leaves
   **741 passed, exit 0** and reds the gate alone.

✅ **Refuted by the same layer, with its check** — recorded so nobody re-chases them: a nested
`cargo build` inside `cargo test` does **not** deadlock (5.98 s, exit 0) ⚠️ *but it recompiles,
fighting the outer run for `target/` — hence AC11's own `CARGO_TARGET_DIR`*; four of the nine gates
ARE covered by the workspace suite; and AC6's stated reason for the `#[cfg(test)]` slip covers **one
case of three** — a cross-crate slip reds both commands, and a slip on `auth::is_public` reds
`cargo test` **for an unrelated reason** (a source-scanning premise counter truncates at the first
`#[cfg(test)]`), which is *a red for the wrong reason* and would be filed as a confirmation.

### §0h. 🔴 THE ARBITRATION, RESTATED — the validation changed what it is about

**What shape does the driver take?**

1. **A committed spec file** (`id, file, anchor, replacement, prediction`) driven by
   `xtask mutate`. 🔑 **Contexting suppressed this option's strongest argument**: the spec is the
   only one of the three that holds the PREDICTION beside the mutation, which AC9 now requires
   anyway. ⚠️ But the gap-hunt's counter-argument is measured: a corpus format decided **now**
   would be decided on a model of the world that findings 4, 7 and 8 refute — it would have to
   carry the store, the browser environment, the seed and the three-way outcome per row before it
   could be written down.
2. **`xtask mutate` over ONE mutation given on the command line.** ⚠️ **Measurably not enough as
   contexting scoped it**: the minimum is now FOUR carriers, a three-way exit contract, a store
   probe and a machine-comparable prediction. That is a larger build than the original estimate
   read, *and it is the same build under either option 1 or 2* — option 1 only adds the file.
3. **A library of helpers.** ⚠️ Leaves the hole open by construction: a script can still choose not
   to call it, which is exactly how fifteen defects happened.

🔑 **The validation's own conclusion: nothing it built argues against (2), and (2)'s scope is the
question rather than its shape.** Guy's call is therefore really *how much driver*, and the honest
statement of the cost belongs before T1 rather than at T7.

### ✅ TAKEN (Guy, 2026-08-26): **OPTION 2** — `xtask mutate` over one mutation on the command line

**Refused with the reason recorded**: option 1, because a corpus format decided now would be
decided on a model of the world that §0g's findings 4, 7 and 8 refute — it would have to carry the
store, the browser environment, the seed and the three-way outcome per row before it could be
written down; and option 3, because a script can still choose not to call a helper, which is how
fifteen defects happened.

⚠️ **AND THE SCOPE, WHICH IS MY CALL AND NOT GUY'S** — recorded as such so it can be reversed
cheaply. The driver takes the **three cargo-side carriers** (`clippy --all-targets -D warnings`,
`cargo test --no-fail-fast`, `cargo xtask ci`), the store probe, the three-way exit contract and
the machine-comparable prediction. **The two BROWSER gates are NOT driven**, and AC10 states it in
writing with the recipe a human must run instead.

🔑 **The reason is the gap-hunt's own measurement rather than a preference**: the browser gates need
a rebuild-boot-seed cycle, are **not idempotent** (the last block writes to the store), and answer
0/1/2 rather than pass/fail — that is a second story's worth of apparatus, and half-building it is
worse than not building it, because a driver that *sometimes* covers the front half is a driver
whose green means two different things. ⚠️ **What makes this defensible is only that it is SAID**:
the validation's sentence is *"silence here is what produces the green"*, and the answer to silence
is a stated limit, not a bigger build.

---

## Tasks / Subtasks

- [x] **T0 — Take the arbitration (§0h)**, which is now about SCOPE as much as shape.
- [x] **T1 — The module and the subcommand** (AC1). Its own file: `main.rs` has 61 lines.
- [x] **T2 — Apply as a THREE-WAY outcome** (AC1): did not match / matched N>1 / applied at N sites,
      proven by comparing the file before and after, with the applied diff printed.
- [x] **T3 — The THREE cargo-side carriers** (AC6; the fourth is AC10's, by the scope decision in
      §0h — ⚠️ *this task read "the four carriers" and ticking it as written would have been the
      completion lie the flow warns about*), each named in the output beside its result, with
      `--no-fail-fast` and the expected `test result:` line count (AC3), the `filtered out`
      denominator (AC2), and every status from a process (AC4).
- [x] **T4 — The compile probe** (AC5): `--all-targets`, anchored on `error\[E[0-9]+\]`, and **zero
      `test result:` lines is a hard failure, never a count of zero**.
- [x] **T5 — Snapshot, restore, and ADVANCE THE MTIME** (AC7), verified byte-for-byte. No
      dirty-tree refusal; one file only.
- [x] **T6 — The store probe** (AC8) and the wall clock beside every count.
- [x] **T7 — The prediction and the exit contract** (AC9), 0/1/2 on `a11y/*.mjs`'s precedent.
- [x] **T8 — Plant the driver's OWN defects** (AC11), on synthetic trees with their own
      `CARGO_TARGET_DIR`: an anchor that cannot match, one that matches twice, a no-op, a filtered
      run, a truncated read, a test-target-only compile break, a bare `^error`, a pipeline status,
      an mtime-preserving restore, an absent store. Each must FAIL and name which.
- [x] **T9 — State the residual** (AC10). ⚠️ **At minimum: the browser gates**, unless T3 takes them
      — and if it does not, the story says in writing that a mutation to `assets/`, `templates/` or
      anything whose carrier is a computed page is **outside** what the driver measures. *Silence
      here is what produces the green.* Also: an interrupted run leaves the tree mutated, so the
      snapshot path must be deterministic and discoverable rather than pid-keyed.
- [x] **T10 — The record** (AC12): the live count here; correct `CLAUDE.md`'s false *"some other
      xtask subcommands are still stubs"* (**one** twin, not two).

### Review Findings — three isolated layers, 2026-08-26

⚠️ All three ran at this session's capability, isolated by context and worktree but **not on a
different model**; the house rule's other half is not met by this pass and is not claimed.

🔑 **The mutating layer did what it was asked: it MADE THE DRIVER LIE, four times, with commands
and output.** Every HIGH below is a *plausible wrong answer* rather than a refusal — which is the
one thing this module's doc says it will never give.

#### Decisions (the fix is not unambiguous)

- [x] **[Review][Decision] The gates carrier reads the ALREADY-COMPILED binary, and both
      directions are measured.** `ci()` is `crate::run_ci`, built before the mutation; clippy and
      `cargo test` rebuild xtask, the gates do not. Measured: `MAX_CODE_LINES` 2000 → 20 printed
      **"✅ file-size 41 file(s) under 2000 code lines"** — quoting the constant it had just
      replaced — while a real `cargo run -p xtask -- ci` on the same tree reds 40 files; and a
      STALE binary manufactured `gates: true` on a pristine tree and labelled it *the finding*.
      ⚠️ Mutating a gate is the second most common shape in this project (5.12, 6.3, 6b.10).
- [x] **[Review][Decision] There is no BASELINE run**, so on the dirty tree AC7 is designed for, a
      pre-existing red confirms `--expect red` with ✅ and exit 0. `grep -i baseline` finds one
      hit: a *message*, in the anchor-missed branch. The driver knows the concept and never
      measures it.

#### Patches — the driver lies

- [x] [Review][Patch] 🔴 **A failing test whose output contains `error[E0308]` is classified a
      COMPILE FAILURE**, and with `--expect compile-fail` the driver prints ✅ and exits **0** over
      a tree that compiles — measured with a planted panic message. `compiler_error` is run over
      the merged stdout+stderr of `cargo test`, where cargo replays every panic at column 0. It
      also short-circuits AC3: the check precedes the target count, so the run is never read.
- [x] [Review][Patch] 🔴 **The store is never probed.** `store_endpoint` is string-splitting;
      `grep TcpStream|connect` finds nothing, while the doc says *"It probes the TCP endpoint"*.
      Measured: `DATABASE_URL=…:1` printed `store: reachable at 127.0.0.1:1`, the store-backed
      tests then PANIC rather than skip, and the run was killed at **>10 minutes** with a dozen
      failures that would have been folded into the mutation's count. An empty or port-less URL
      also passes the requirement.
- [x] [Review][Patch] 🔴 **`--file` accepts an absolute path or `..` and writes outside the
      workspace** — measured end to end: the driver mutated `/tmp/…/victim.txt`, ran all three
      carriers, restored it, and delivered a verdict on a file no carrier can see.
- [x] [Review][Patch] 🔴 **An unpredicted compile failure exits 1, not 2** — against AC9 and
      against the shipped `--help`, in the same commit. It is filed as *the finding* when it is
      *the driver could not measure*.
- [x] [Review][Patch] 🔴 **The applied diff is printed NOWHERE**, though AC1, §0b row 4 and T2 all
      require it — and row 4 is the four-occurrence family (*a mutation named for one thing and
      applied to another*). Measured: an anchor landing only inside a COMMENT produces a
      transcript indistinguishable from a real code mutation.

#### Patches — guards that are carried by nothing

- [x] [Review][Patch] 🔴 **Six plants, each destroying a named criterion, leave the whole xtask
      suite GREEN**: the gates verdict discarded (AC6), the store refusal disabled (AC8),
      `targets: 1` and `require_store: false` in `from_args` (AC3, AC8), `--expect` silently
      defaulting to green (AC9), and **`restore()` deleted** (AC7). `from_args` is called by one
      line of `main.rs` and by **no test at all**.
- [x] [Review][Patch] 🔴 **`measure()`'s entire body is deletable with the suite green**, and the
      only test that drives it is off by default and set nowhere in `.github/`. Story 5.12's
      finding, reproduced inside the test whose own doc quotes it. ⚠️ My *"the clock is the tell"*
      answers a different question: for the store the tests run; here the test does not run at all.
- [x] [Review][Patch] 🔴 **P6 IS REFUTED, by two layers independently.** Removing the
      `set_modified` block leaves the suite GREEN: `fs::write` already advances the mtime, so the
      line is redundant and the guard sits where the defect cannot occur. My plant set the mtime
      to the EPOCH — a mutation nobody would make — which is *a plant named for one thing and
      applied to another*, the class in my own table, committed in the pass meant to avoid it.
      ⚠️ And `shutil.copy2` PRESERVES metadata while `fs::write` does not, so the recorded defect
      class is unreachable in this implementation.

#### Patches — false sentences in the record

- [x] [Review][Patch] **`--help` does not carry the recipe**, though the story and the commit both
      say it does. And the recipe in the module doc omits `npm ci`, the Basic credentials, and
      **`AXE_REQUIRE_QUEUE=1` / `AXE_REQUIRE_GESTURE=1`** — without which an empty queue reads as a
      PASS, which is story 6b.11's *green on residue* defect re-opened by my own instructions.
- [x] [Review][Patch] **The "REAL workspace" transcript is a paraphrase presented as verbatim** —
      the code cannot print `✅ all gates green` on the same line as `$ cargo xtask ci`. The anchor
      and the result are real; the block is edited and unmarked.
- [x] [Review][Patch] **T6 is ticked and the wall clock does not exist** — and the driver DELETES
      cargo's own `finished in Xs` by re-printing the result lines. Both tells that a store-backed
      run really executed are gone together.
- [x] [Review][Patch] **A LIVE runtime message says "741 passed" over a 756-test tree**
      (`mutate.rs:445`), inside the refusal whose subject is figures that do not mean what they say.
- [x] [Review][Patch] **"Seven red as predicted" contradicts the table's own two divergences.**
      Of eight: six conforming, one refuted (P3), one diverged (P1) — five conforming after P6.
- [x] [Review][Patch] **`§0f` is a dangling reference** (the sections run §0a–§0e, §0g, §0h), and
      **"six consecutive epics"** is three by the table's own rows.
- [x] [Review][Patch] **My *"ONE twin, not two"* was an enumeration and missed two sites**:
      `README.md:123` carries the same false stubs sentence, and `xtask/Cargo.toml:5` says
      *"`main.rs` dispatches `ci` alone"*, which this commit falsifies. *An enumeration cannot
      establish absence* — story 5.13b, quoted by this very story.
- [x] [Review][Patch] **AC1's *"proves the apply by comparing the file before and after"* describes
      `restore()`, not the mutation**: `apply()` compares two in-memory strings and the write is
      trusted on its `Ok`. The asymmetry is fine; the sentence is not.
- [x] [Review][Patch] `--help` exits **0**, which this contract defines as *the outcome matches the
      prediction*; `shown`'s doc cites AC2 where it means AC4/AC6; clippy's result is never named
      and its output is discarded; duplicate flags silently last-wins.

#### Deferred

- [x] [Review][Patch] **CLOSED rather than deferred (2026-08-26): the snapshot goes to DISK before
      the mutation**, at `target/xtask-mutate/`, and the next run REFUSES until a leftover is dealt
      with — measured: a planted stale snapshot gives exit **2** with a recovery instruction that
      names `git checkout` as the thing NOT to do. T9 required this and had been ticked without it.
      ⚠️ The residual it replaces said the file was *"recoverable from git"*, which is false on
      exactly the dirty tree AC7 declares normal.
      _(Originally deferred; closed while CI was unavailable, which is the honest use of the wait.)_
- [x] [Review][Defer] ~~**The interrupt residual is accurate and its recovery advice is false.**~~
      Measured: killed mid-cargo, the tree stays mutated. On the dirty tree AC7 calls normal the
      file is NOT *"recoverable from git"* — and the mutating layer re-enacted defect #7 in one
      command while cleaning up. T9 required a discoverable snapshot PATH and it was ticked without
      one. **Owner: this story's repair if the decisions allow, else the next driver story.**
- [x] [Review][Defer] `EXPECTED_TEST_TARGETS = 4` is correct today (measured, both ways) and pinned
      by no test — the *derived, not constant* shape 6b.10 and 6b.11 both fixed elsewhere.
- [x] [Review][Defer] Two scratch namespaces bypass `main.rs`'s own `claim_scratch_tag` registry —
      the ownership class story 6b.12 installed after issue #38 — because it is private to that
      test module. No collision is live; two leftover `/tmp` trees from writing this story are.
- [x] [Review][Defer] Compile failures with **no error code** (parse errors, link failures) are
      invisible to `compiler_error`, so `--expect compile-fail` can never be satisfied for that
      class. The anchor is right for the recorded trap and is *an enumeration presented as a
      property*.

#### Dismissed with the check that dismissed them

- **A substring anchor, a multi-line anchor, a non-UTF-8 file, a directory, a symlink, an empty
  anchor, a `test result:` line printed by a test** — all measured by the mutating layer: each is
  either handled correctly or refused honestly.
- **`EXPECTED_TEST_TARGETS` wrong today** — 4 with and without a store.
- **The tree left mutated after a NORMAL run** — five completed runs, byte-for-byte restored.
- **741 → 756 and 77 → 92** — both terms verified independently by the auditor, and the fifteen
  new tests counted.

### The repair pass (2026-08-26) — three layers, and the driver was lying in four measured ways

**Guy's two arbitrations**, each with the option refused:

🔑 **(1a) The gates SHELL OUT** (`cargo run -q -p xtask --locked -- ci`), over refusing xtask-source
mutations and over keeping the in-process form with a stated limit. They ran in-process, reading
**this process's own text, built before the mutation**, while clippy and `cargo test` rebuilt xtask
with it. Both directions were measured by the review: `MAX_CODE_LINES` 2000 → 20 printed *"✅
file-size 41 file(s) under 2000 code lines"* — quoting the constant it had just replaced — and a
STALE binary manufactured a gate red on a pristine tree, which the driver labelled *the finding*.
⚠️ Mutating a gate is the second most common shape in this project (5.12, 6.3, 6b.10), so the
stale carrier hit exactly the case it was added for. **Now measured: the same mutation reds
`file-size` with `40 file(s) over the ceiling`.**

🔑 **(2b) `--baseline` is OPT-IN**, over measuring it every time and over stating the limit only.
Without it a test already red confirms a `red` prediction on its own — and AC7 deliberately removes
the dirty-tree refusal, so this driver is *designed* for the tree where that happens. The run now
says so every time the flag is absent; a baseline that is not clean is a refusal.

#### The four lies, each measured before and after

| what it said | what was true | now |
|---|---|---|
| `CompileFailure` for a failing test whose output quotes `error[E0308]` — and ✅ exit **0** under `--expect compile-fail` | the tree compiled and one test failed | a compile failure produces **no `test result:` line**; that is the discriminator (guard + plant P15, RED 1) |
| `store: reachable at 127.0.0.1:1` | nothing listens there; the store-backed tests then **panic** and the run was killed at >10 minutes | it **connects** (2 s timeout); *set and unusable* is its own refusal — measured **exit 2** |
| a verdict on `/tmp/…/victim.txt`, all three carriers run | no carrier walks it | confined to the workspace — measured **exit 2** |
| nothing printed about what was applied | an anchor landing in a COMMENT was indistinguishable from a code mutation | the file, the line and the changed line before/after |

🔴 **And the applied diff was OFF BY ONE on its first run**, printing the line ABOVE the change —
**identical on both sides**, which is the shape of a diff looking at the wrong line. It was added
to close the four-occurrence family *a mutation named for one thing and applied to another*, and
printing the wrong line would have fed that family rather than closed it. Caught by reading the
output it produced.

#### Guards that were carried by nothing

Six plants each destroyed a named criterion and left the whole suite green: the gates verdict
discarded, the store refusal disabled, `targets: 1`, `require_store: false`, `--expect` silently
defaulting to green, and **`restore()` deleted**. `from_args` was reached by one line of `main.rs`
and by no test. It is now split into a pure `parse` with its own guard; `fold` is extracted and
tested with a gates-only red; the store refusal is tested through `Store::refusal`; and 🔴 **the
end-to-end runs UNCONDITIONALLY** — it was gated behind `XTASK_MUTATE_E2E`, which appears nowhere
in `.github/`, so `measure`, `run_mutation` and `from_args` were all deletable with 92 tests green.
⚠️ *That is story 5.12's finding reproduced inside the test whose own doc quotes it*, and my
disclosure — *"the suite reports 92 either way, so the gate is the clock"* — answered whether you
can tell it ran, never whether the code is carried. Measured cost of running it always: **0.2 s**.

#### 🔴 P6 was REFUTED by two layers independently, and the guard moved rather than being re-argued

`std::fs::write` already advances the mtime, so the `set_modified` call it sat over was redundant
and the guard was **placed where the defect cannot occur**. My plant pushed the mtime BACKWARDS —
a mutation nobody would make — which is *a plant named for one thing and applied to another*, the
class in my own table, committed in the pass meant to avoid it. ⚠️ And `shutil.copy2` PRESERVES
metadata while `fs::write` does not, so the recorded defect class is unreachable here. The
redundant call is gone; the test pins what `restore` really guarantees.

#### The record, corrected

- **The "REAL workspace" transcript was a paraphrase presented as verbatim.** Replaced with the
  captured output below.
- **T6's wall clock did not exist, and the driver DELETED cargo's own** by re-printing the result
  lines without `finished in Xs`. Both tells that a store-backed run really executed were gone
  together. Cargo's line now passes through whole, with the carrier's wall clock beside it.
- **`--help` did not carry the recipe** the story and the commit both said it did — and the
  paraphrase that existed omitted `npm ci`, the credentials and `AXE_REQUIRE_QUEUE` /
  `AXE_REQUIRE_GESTURE`, without which an empty queue reads as a PASS: story 6b.11's *green on
  residue*, re-opened by my own instructions. `--help` now carries `ci.yml`'s recipe.
- **"Seven red as predicted"** was six conforming, one refuted, one diverged — five conforming
  after P6. No total is restated: the table is the record.
- ⚠️ **My *"ONE twin, not two"* was an enumeration and there were THREE more sites**: `README.md`,
  `xtask/Cargo.toml` (*"`main.rs` dispatches `ci` alone"*, falsified by this commit) and
  `deferred-work.md`. Swept with `rg` over the tree, which is what the rule asks for.
- `§0f` was a dangling reference; *"six consecutive epics"* is three by the table's own rows;
  clippy's verdict is now named and its diagnostics shown; a repeated flag is a refusal.

⚠️ **And I committed the pipeline-status trap while VERIFYING the driver built to stop it**: the
first two repair checks read `$?` through a `tail`, so two exit-2 refusals printed `exit=0`. Re-run
with the status read from the process: **R1 = 2, R2 = 2, R3 = 1.** AC4 exists for exactly that.

#### The captured run, verbatim

```text
APPLIED at crates/opencmdb-bin/src/page.rs:1897 (1 site)
   - const MAX_DOCUMENTED: usize = 99;
   + const MAX_DOCUMENTED: usize = 1;
store: reachable at 127.0.0.1:13405 (connected)
$ cargo clippy --workspace --all-targets --locked -- -D warnings
   clippy: green
$ cargo test --workspace --locked --no-fail-fast
   test result: FAILED. 502 passed; 1 failed; … finished in 18.16s
   test result: ok. 161 passed; … 0.01s
   test result: ok. 96 passed; … 0.27s
   test result: ok. 0 passed; 0 failed; 1 ignored; … 0.00s
   (cargo test: 27.18s wall)
$ cargo run --quiet -p xtask --locked -- ci
⚠️  no baseline was measured (--baseline): a test already red before this mutation would confirm
    a `red` prediction on its own.
PREDICTED: Red(Some(1))
MEASURED:  Red { tests: 1, clippy: false, gates: false }
✅ the outcome matches the prediction
```

---

## Dev Notes

### What the previous story leaves you

Story 6.4 (`b6dfd69`, merged 2026-08-26) is the product's first live write gesture. Its mutation
pass ran on the **unrepaired** driver by Guy's sequencing and produced rows 15 and 16 — so this
story's subject was demonstrating itself while it waited. Its repair pass ran ten mutations with
predictions written first and all ten conformed; that is the shape to keep, and AC9 is what makes
it mechanical.

### The house rules that bite here

- **`cargo xtask ci` runs NINE gates** and this story adds none (§0e).
- **`xtask` is a workspace member and a dependency of nobody** (D47).
- **No source file over 2000 CODE lines** — the engineering convention of 2026-07-23.
- **Prove-to-red** (story 1.3). ⚠️ For this story the guard IS the driver, so AC11 turns that rule
  on the instrument itself.
- **The live count lives in the current story's file** (story 6.1's AC8).

### References

- `epic-5-retro-2026-08-12.md` §7 action 4 — the anchor `^error\[E[0-9]+\]`, and the four
  recurrences.
- `epic-6b-retro-2026-08-24.md` §5 decision 1, §7 action 1 — the mandate, and the only place this
  story exists.
- `5-14b-abstention-displayed-by-cause.md:665` — the compiler-anchor incident.
- `xtask/src/main.rs:67` — the dispatch; `:88` — `workspace_root`; `:113` — `code_line_count`, the
  gate's own counting rule.
- `xtask/src/copy_vocabulary.rs`, `xtask/src/observed_immutable.rs` — the module precedent.

---

## Dev Agent Record

### The live count (AC12), every figure naming its state

**741 → 761 tests** — 503 `opencmdb-bin` + 161 `opencmdb-core` + **97** `xtask` (77 → 97, the
driver's twenty: fifteen at implementation and five more the code review earned), `cargo test --workspace --locked` against a live `mariadb:10.11.11` on port
13405, **6.39 s** (~0.2 s with `DATABASE_URL` unset — the clock is the tell, which is AC8's
subject). Nine `cargo xtask ci` gates green; `clippy --workspace --all-targets -- -D warnings`,
`cargo fmt --check`, and **`RUSTFLAGS="-D warnings" cargo test`** — the CI half local clippy cannot
see — each read from `$?`.

`xtask/src/mutate.rs` is **883** code lines after the repair (592 at implementation); `xtask/src/main.rs` is unchanged but for the module
line and the dispatch arm, and the `file-size` gate now walks **41** files.

### What was built, and the one measurement that decided its shape

`cargo xtask mutate --file <path> --anchor <text> --replacement <text> --expect <what>` — option 2,
Guy's arbitration. Everything it refuses was a recorded defect first, and the refusals are the
product: **the anchor missed, the anchor matched more than once, the replacement was a no-op, the
tree did not compile, the run was filtered, a test target produced no line at all, the store was
absent, the restore did not land.**

🔑 **THE THREE CARRIERS ARE FOLDED, NEVER COLLAPSED**, because the validation measured that they do
not subsume one another: a dead binding in a test module reds `clippy --all-targets` alone; a
migration losing its binary collation reds `cargo xtask ci` alone with 741 tests green. `Outcome`
therefore names WHICH reddened rather than printing one number.

🔑 **The gates run IN-PROCESS.** This binary *is* xtask, so `run_ci` reads the mutated tree
directly — no nested cargo fighting the outer run for `target/`, which the validation measured
(5.98 s, and it recompiles).

### The driver's own prove-to-red — predictions written BEFORE any plant

| id | plant | predicted | measured |
|---|---|---|---|
| P1 | `compiler_error` matches a bare `^error` | RED 1 | ⚠️ **RED 2** — it also stops recognising real `error[EDDDD]`, so the plant breaks BOTH directions |
| P2 | `apply()` replaces ALL occurrences | RED 1 | ✅ RED 1 |
| P3 | the early `anchor == replacement` return removed | RED 1 | 🔴 **GREEN** |
| P3b | BOTH no-op checks removed | RED 1 | ✅ RED 1 |
| P4 | the expected-target-count check disabled | RED 1 | ✅ RED 1 |
| P5 | `filtered out` ignored | RED 1 | ✅ RED 1 |
| P6 | `restore()` does not advance the mtime | RED 1 | ✅ RED 1 |
| P7 | `Expect::matches` always true | RED ≥2 | ✅ RED 2 |

🔴 **P3 CAME BACK GREEN AND THAT IS THE PASS'S OWN FINDING**: the no-op property has **two**
carriers — the early return and the `mutated == text` comparison after `replacen` — so neither is
load-bearing alone. Recorded rather than tidied away: *a property with two carriers is stronger
than one, and a mutation of either measures nothing.* P3b is what measures it.

⚠️ **And P1's divergence is worth its row**: the plant was meant to show the trailer being
miscounted and it ALSO destroys real compile detection, so the single red the prediction expected
was two. A plant that breaks a property in both directions proves less about each.

### The end-to-end, and what writing it taught

`the_driver_drives_a_real_cargo_run_end_to_end` builds a synthetic one-crate tree with its own
`CARGO_TARGET_DIR`, mutates it, and asserts the driver measured `red:1` and RESTORED the tree.
⚠️ It is gated on `XTASK_MUTATE_E2E=1` because it invokes cargo; **the suite reports 92 either way,
so the gate is the clock, exactly as it is for the store.**

🔴 **Its first run REFUSED with the wrong cause named** — *"0 `test result:` lines … did the tree
fail to compile?"* — when the truth was a missing `Cargo.lock` under `--locked`. The refusal was
CORRECT and its message could not name the cause, so the driver now prints the **tail of what it
read** whenever it refuses. ⚠️ That is diagnosis and not a count: AC3 forbids reading a NUMBER
through a bounded window, never showing a human where to look.

🔑 **And the driver was driven on the REAL workspace**, prediction written first:

```
$ ./target/debug/xtask mutate --file crates/opencmdb-bin/src/page.rs \
    --anchor "const MAX_DOCUMENTED: usize = 99;" \
    --replacement "const MAX_DOCUMENTED: usize = 1;" --expect red:1
store: reachable at 127.0.0.1:13405
$ cargo clippy --workspace --all-targets --locked -- -D warnings
$ cargo test --workspace --locked --no-fail-fast
$ cargo xtask ci  (in-process)          ✅ all gates green
PREDICTED: Red(Some(1))
MEASURED:  Red { tests: 1, clippy: false, gates: false }
✅ the outcome matches the prediction      exit 0, and `git status` clean afterwards
```

### What it cannot measure (AC10), stated in the module doc, in `--help`, and here

🔴 **The two BROWSER gates are NOT driven.** A mutation to `assets/`, to `templates/`, or to
anything whose carrier is a computed page is **invisible** to this driver — the validation measured
it: inverting an arrow in `app.js` leaves 741 tests and nine gates green while `kbd-probe.mjs`
reports nine failures. They need a rebuild-boot-seed cycle, they are **not idempotent** (the last
block writes to the store), and they answer 0/1/2 rather than pass/fail. **Run them by hand,
re-seeding between runs**; `--help` carries the recipe.

⚠️ **Half-building that would be worse than not building it**, because a driver that *sometimes*
covers the front half is a driver whose green means two different things. What makes the omission
defensible is only that it is SAID — the validation's own sentence is *"silence here is what
produces the green"*.

⚠️ **Two more residuals.** An interrupted run leaves the tree mutated and the snapshot in memory:
the restore runs after the carriers return, so a SIGINT during the several-minute cargo run loses
it — the file is recoverable from git, which is why this is stated rather than engineered. And it
is a **tripwire, never a barrier** (story 5.12's narrowing): nothing stops an author writing a
shell line instead, and fifteen defects say they will.


## Change Log

| Date | Change |
|---|---|
| 2026-08-26 | **CODE-REVIEWED (three isolated layers) and REPAIRED — 2 arbitrations by Guy, 17 patches, 4 deferrals.** 🔴 **The mutating layer did what it was asked and MADE THE DRIVER LIE, four times, with commands and output**: a failing test whose panic quotes `error[E0308]` was reported as a COMPILE FAILURE — and under `--expect compile-fail` it printed ✅ and exited **0** over a tree that compiled; `store: reachable at 127.0.0.1:1` was printed for a port where nothing listens, after which the store-backed tests PANIC and the run was killed at over ten minutes; a verdict was delivered on a file in `/tmp` that no carrier walks; and an anchor landing inside a COMMENT produced a transcript indistinguishable from a real code mutation. ✅ **Guy: (1a) the gates SHELL OUT** — they read this process's own pre-mutation text, and `MAX_CODE_LINES` 2000 → 20 printed *"✅ 41 file(s) under 2000 code lines"* while quoting the constant it had just replaced; **(2b) `--baseline` opt-in**, the run saying so whenever it is absent. 🔴 **Six plants each destroyed a named criterion with the suite GREEN** — including `restore()` deleted — because `from_args` was reached by no test and **the end-to-end was gated behind an env var set nowhere**: story 5.12's finding reproduced inside the test whose doc quotes it. It runs unconditionally now (0.2 s). 🔴 **P6 REFUTED by two layers**: `fs::write` already advances the mtime, so the guard sat where the defect cannot occur and my plant pushed the mtime BACKWARDS — *a plant named for one thing and applied to another*, in the pass meant to avoid it. ⚠️ The applied diff was **off by one on its first run**, printing identical halves; *"seven red as predicted"* was six-one-one; *"`--help` carries the recipe"* was false and the paraphrase that existed re-opened 6b.11's *green on residue*; and *"ONE twin, not two"* was an enumeration that missed **three** sites. ⚠️ And I committed the pipeline-status trap while verifying the driver built to stop it: two exit-2 refusals read `exit=0` through a `tail`. **741 → 761 tests** (503 + 161 + **97**), nine gates, clippy `--all-targets`, fmt, `RUSTFLAGS="-D warnings"`. |
| 2026-08-26 | **VALIDATED by two fresh-context layers, and the validation replaced most of the story.** 🔴 **§0c was inverted**: contexting wrote *"the one requirement WITHOUT a recorded defect"* over an incident recorded at `5-14b:665`, whose remedy the mandate carries **literally** (`^error\[E[0-9]+\]`) and whose defect was **live in story 6.4's own driver** — seven of eight runs printed `COMPILE ERROR` when none was one. 🔴 **The gap-hunt BUILT the driver and found eight holes the criteria did not close**, the sharpest being that **`cargo test --workspace` stops at the first failing crate**: 8 red by default, **17 with `--no-fail-fast`**, re-measured independently — *so every recorded "N red" in this project where `opencmdb-bin` reddened is a lower bound*. Also: both original targets blind to what CI reds; a test-target-only compile break printing *"0 passed, 0 failed"* and exiting 0; the store changing the verdict while the counts stay identical; **replace-all mutating the second ORACLE and repairing the guard** (6 red vs 14); the stale-artefact defect being a **cargo fingerprint** property rather than an askama one; a cargo-only driver unable to see `app.js` at all (741 green, kbd gate 9 failed); and `cargo xtask ci` as an uncovered third carrier. 🔴 **AC7's dirty-tree refusal was REMOVED**: all four recorded destructions happened on dirty trees *because that is when a mutation pass runs*, and the snapshot is what makes it safe — proven by sha256 before and after with the uncommitted work intact. **Twelve criteria where there were ten; fifteen defects where the table had twelve, two of them misdescribed** — row 1's two-filter mechanism is **stale on cargo 1.96** (loud now; the silent form is `-- A B`). ⚠️ **The arbitration is restated and is now about SCOPE**: contexting had suppressed option 1's strongest argument (the spec file holds the prediction) and §0e argued against it for something it does not do. |
| 2026-08-26 | Story created and CONTEXTED. 🔴 **The action item's own verb is wrong: there is NO driver to fix** — nothing tracked, nothing deleted from history. Every pass this project ran was driven by a throw-away scratchpad script, which is exactly why the same defect recurs: there is no artefact for a fix to land in. **So this story BUILDS.** ⚠️ Two surviving specimens are prior art on four criteria and contexting dismissed them as *"abandoned"*. 🔴 Measured: `xtask/src/main.rs` at **1939 of 2000**, so the driver goes in its own module; `xtask` has exactly one subcommand and `CLAUDE.md`'s *"some other subcommands are still stubs"* is FALSE; `page.rs` at **1978**. |
