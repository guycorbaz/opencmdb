# Story 4.3: The MANIFEST becomes a lockfile for data

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want `MANIFEST.toml` carrying sha256, seed and generator version per artefact, and a gate that also catches files nobody listed,
so that neither an edited fixture nor an unlisted one can enter the corpus silently.

## Context

The corpus now holds three artefacts and the lock covers **only what it lists**. Two review layers, independently, called the same thing out: *"the gate's real guarantee is 'listed files are unchanged', not 'the corpus is frozen' — adding a new trap file with no MANIFEST line is green."*

It is worse than it sounds, because the gate is also blind to *meaning*. A trap file added today is **neither hashed, nor parsed, nor validated, nor cross-checked** — the only caller of `read_traps` is a test with a hard-coded filename. So a trap file that fails `TrapFile::validate` *and* points at observations that do not exist would ship green, in a corpus whose entire job is to be the oracle.

This story closes both halves: bytes (the lock) and meaning (the corpus parses).

**Explicitly OUT of scope:**
- The **seeded generator** itself — ARCH-24 places it after the engine. This story only makes the manifest able to *record* a generator; nothing generates anything.
- `FixtureConnector` (4.4), the metrics harness (4.6), the trap runner (4.7), any trap family (4.9+).
- The `recapture` tool (Epic 11). This story documents the `capture/` ↔ `scenario/` rule; it does not implement a tool that has to obey it.

## Acceptance Criteria

1. **`MANIFEST.toml` replaces the provisional line format**, carrying per artefact: its corpus-relative `path`, its `sha256`, and an OPTIONAL `generator` (name, version, seed). The generator field must be absent-able — every artefact in the corpus today is hand-authored, and a format that cannot express that would be filled with lies.

2. **A listed file whose bytes changed is RED**, naming the file and both hashes. This behaviour exists and is proven-to-red; it must survive the format change unchanged.

3. **A file present under `fixtures/` but absent from the manifest is RED**, naming the orphan. This is the drift-in-the-ADD direction, deferred from the story-1.2 review and confirmed twice since. `README.md` files are the one exception, and the exception is explicit rather than a silent skip.

4. **A manifest entry pointing at a file that does not exist is RED**, naming it — the mirror of AC3, and the failure mode of a deleted fixture.

5. **Every trap file in the corpus is parsed, validated and cross-checked** — not one hard-coded example. Discovery must be by walking the corpus, so that a trap file added in story 4.9 is covered the day it lands, with no test to remember to update.

6. **A replay stream containing two observations with the same `obs_id` is refused.** The labelling format rests on `obs_id` being a stable anchor — *"never by line number"* — and two lines sharing one void that guarantee. Deferred from the 4.1 review to here.

7. **The walk is honest about what it cannot see**: it does not follow directory symlinks, and it fails loudly on an unreadable directory rather than silently searching less. A gate whose failure mode is "quietly saw less of the tree" is not a gate — the exact defect found in story 4.1's path-discipline test.

8. **`capture/` and `scenario/` are documented as different rot risks**, and the requirement that a future re-capture job be structurally unable to reach `scenario/` is recorded where the story that writes it will read it.

9. **All gates green:** `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo xtask ci`.

## Tasks / Subtasks

- [x] **Task 1 — the manifest format** (AC: 1)
  - [x] Define the TOML shape (array of tables — `[[artefact]]`), with `generator` optional.
  - [x] Add `toml` and `serde` to `xtask` (both already in the workspace tree; no new supply chain).
  - [x] `#[serde(deny_unknown_fields)]`, as stories 4.1 and 4.2 established: a lockfile that tolerates a misspelled key is not a lock.

- [x] **Task 2 — rewrite the gate over the new format** (AC: 2, 4)
  - [x] Keep the existing verify logic (recompute sha256, compare case-insensitively, name the drifted file) — it is stable and its tests exist. Only the parser changes.
  - [x] Keep failing CLOSED: a manifest that does not parse is RED, never "no entries, skipped".

- [x] **Task 3 — orphan detection** (AC: 3, 7)
  - [x] Walk `fixtures/` with `walkdir` (already an `xtask` dependency), NOT following symlinks, and propagate errors rather than swallowing them.
  - [x] Red on any file present but unlisted, naming it. Skip `README.md` by an explicit, commented rule.
  - [x] Proven-to-red test for the orphan case, matching the existing gate tests' style.

- [x] **Task 4 — the corpus must PARSE, not merely hash** (AC: 5)
  - [x] Replace the hard-coded `EXAMPLE_TRAPS` test with one that DISCOVERS every `scenario/traps/*.toml` and runs the real `read_traps` over each — parse, validate, and cross-check its observations.
  - [x] Fail if the corpus contains zero trap files: a discovery test that finds nothing must not pass silently (the same vacuity trap the fixtures gate had until story 4.1).

- [x] **Task 5 — `obs_id` is a real anchor** (AC: 6)
  - [x] `read_jsonl` refuses a stream containing a repeated `obs_id`, naming the id and both line numbers.
  - [x] Test it.

- [x] **Task 6 — documentation and gates** (AC: 8, 9)
  - [x] Verify `fixtures/README.md` no longer overclaims: story 4.1's review made it say the lock covers only *listed* files, with a pointer to this story. That caveat must now be REMOVED, because it is being closed here.
  - [x] Run the full gate set.

### Review Findings

_Code review 2026-07-21 — three parallel layers, all three probing empirically. 6 of 9 ACs satisfied, AC3 partial, **AC5 NOT satisfied**. Scope: one overstep. D47 and D56 clean. The deleted-line inventory found one guarantee dropped and one weakened._

- [x] [Review][Decision] **Dot-files are skipped by the corpus walk** — decided 2026-07-21 (Guy), option b, with an explicit warning about BMad. Verified: `_bmad/`, `_bmad-output/` and `.claude/` live at the REPOSITORY root and are unreachable from a walk rooted at `fixtures/`; the code says so, so re-rooting the walk forces the question again. Original finding: `.DS_Store`, `.gitkeep` and a live vim `.swp` inside `fixtures/` each RED the gate today. Fail-closed is defensible, but it means a local `cargo xtask ci` fails on files git does not track. Options: keep fail-closed; skip dot-files; or consider only git-tracked files. This is a daily-ergonomics decision, not a correctness one.
- [x] [Review][Decision] **The `generator`-without-`seed` rule is KEPT and named** — decided 2026-07-21 (Guy), option a. It is now recorded in the code as a policy adopted deliberately by this story rather than inherited by accident from the story that adds the generator. Original finding: The story said it would only make the manifest *able to record* a generator, and this invents a validation policy — under lint pressure, which is not a specification. It is well-reasoned (a provenance claim nobody can re-run is theatre) and harmless today, but the generator story will inherit a rule it did not choose.

- [x] [Review][Patch] **A symlinked FILE is neither hashed nor flagged** — `file_type().is_file()` is false for a symlink, so it is dropped before the manifest comparison. Not following it is correct; saying nothing about it is the "quietly saw less of the tree" shape AC7 forbids [xtask/src/main.rs:512]
- [x] [Review][Patch] **A symlinked DIRECTORY hides an entire unlisted subtree** — strictly larger than the previous hole: arbitrarily many files, gate green [xtask/src/main.rs:510]
- [x] [Review][Patch] **`.replace('\\', "/")` aliases a real backslash filename onto a listed path** — on Linux `\` is a legal filename byte, so `a\b.jsonl` collides with a listed `a/b.jsonl` and the smuggled file passes as the legitimate one. Gate the replacement behind `cfg(windows)` [xtask/src/main.rs:519]
- [x] [Review][Patch] **AC5 NOT SATISFIED: trap discovery does not walk.** `read_dir` is one level, and trap FAMILIES (4.9+) are exactly what will add a subdirectory. Reproduced: a listed, correctly-hashed trap at `traps/family/broken.toml` pointing at a nonexistent stream — gate green, discovery test green [crates/opencmdb-bin/src/fixtures.rs:515]
- [x] [Review][Patch] The trap-discovery extension filter is case-sensitive, so `broken.TOML` is hashed and never read [crates/opencmdb-bin/src/fixtures.rs:522]
- [x] [Review][Patch] **The README exemption is a raw suffix match** — `NOTREADME.md`, `evil-README.md` all escape the orphan rule. Compare the file-name component, not the string suffix [xtask/src/main.rs:535]
- [x] [Review][Patch] **Nothing tests the gate against a real corpus.** Every gate test calls `manifest_findings` or `orphan_findings` directly; `corpus_files` has zero tests, so its symlink handling, its separator normalisation and its error propagation live only in a doc comment, and a swapped argument between `listed_paths()` and `corpus_files()` would ship green [xtask/src/main.rs:434]
- [x] [Review][Patch] **Duplicate `path` entries inflate the count** — two entries for one file report `4 fixture(s)` for a corpus of 3. A lockfile with a repeated key is malformed; nothing checks `artefact.len() == listed_paths().len()` [xtask/src/main.rs:498]
- [x] [Review][Patch] A corpus containing only READMEs plus an empty artefact list is GREEN — the vacuity the story itself refuses on the test side (`checked > 0`) [xtask/src/main.rs:471]
- [x] [Review][Patch] A deleted `fixtures/` directory reports green ("no fixtures — skipped") and a test pins it. Measured: six tests fail, so CI still reds — but the gate states a guarantee it does not provide [xtask/src/main.rs:411]
- [x] [Review][Patch] A listed FIFO hangs the gate forever — `std::fs::read` blocks and `cargo xtask ci` never returns [xtask/src/main.rs:432]
- [x] [Review][Patch] Non-UTF-8 filenames collapse to identical `U+FFFD` strings: no manifest entry can ever match, so the gate is permanently RED with an undiagnosable message [xtask/src/main.rs:519]
- [x] [Review][Patch] On a case-insensitive filesystem one file gets two contradictory verdicts: `manifest_findings` resolves it through the OS and passes, `orphan_findings` does an exact string lookup and reports it orphaned [xtask/src/main.rs:536]
- [x] [Review][Patch] A newline in a filename injects fabricated lines into the finding report, one of which reads like a separate benign gate line [xtask/src/main.rs:537]
- [x] [Review][Patch] A manifest path naming a directory is diagnosed "missing" — the `Err(_)` arm discards `IsADirectory` [xtask/src/main.rs:456]
- [x] [Review][Patch] Listing `MANIFEST.toml` inside itself is unfixable (recording the hash changes the hash); the orphan filter exempts it but the hash check does not [xtask/src/main.rs:432]
- [x] [Review][Patch] The containment check splits only on `/`, so `..\secret` is not recognised as escaping on Unix and falls through to a generic "missing" [xtask/src/main.rs:604]
- [x] [Review][Patch] `sha256` is an unvalidated `String`: a truncated or non-hex lock is diagnosed identically to a tampered fixture, though the repairs are opposite [xtask/src/main.rs:474]
- [x] [Review][Patch] `fixtures_gate_mismatch_prefix_is_char_safe` lost its assertion that the finding NAMES the offender — a guarantee removed against an explicit Dev Notes instruction [xtask/src/main.rs:891]
- [x] [Review][Patch] The gate's "a manifest that does not parse is RED" arm is untested; the replacement test asserts serde's behaviour, not the gate's [xtask/src/main.rs:922]
- [x] [Review][Patch] `read_jsonl`'s duplicate check is per-file but its message asserts a corpus-wide guarantee; two streams may share an `obs_id` [crates/opencmdb-bin/src/fixtures.rs:186]
- [x] [Review][Patch] **The deferred-work register lost an open defect**: the rewritten bullet enumerates what remains open and omits the duplicate-manifest-path half, which this story did not fix [deferred-work.md]
- [x] [Review][Patch] The deferred item claiming this story would "validate encoding there" was neither delivered nor re-deferred; the spaced-filename item is now obsolete (TOML expresses it) and was not closed [deferred-work.md]
- [x] [Review][Patch] `xtask`'s module doc still describes the gate this story replaced — "fixtures (D56, scaffold)… `fixtures/MANIFEST`… PROVISIONAL". Every clause is now false, in the file that changed [xtask/src/main.rs:15]
- [x] [Review][Patch] **Two Dev Agent Record claims are false**: that the discovery test "walks" `scenario/traps/` (it uses `read_dir`), and that a test "pins that the list is exactly those two" exemptions (it pins that those two are exempt, not that nothing else is) [this file]
- [x] [Review][Patch] `xtask` test temp directories leak on failure and reuse the pid, so a stale `fixtures/` can flip the no-fixtures test; `scratch_dir` exists in the other crate for exactly this [xtask/src/main.rs:957]
- [x] [Review][Patch] AC3 says `README.md` is "the ONE exception"; `MANIFEST.toml` is a second. Defensible and commented, but it is an unauthorised deviation and was not raised as one [xtask/src/main.rs:535]

- [x] [Review][Defer] Two paths differing only by Unicode normalisation produce visually identical findings [xtask/src/main.rs:537] — deferred: fail-closed already, and the fix needs a normalisation policy the corpus does not yet have
- [x] [Review][Defer] `architecture.md`'s D56 text still names `fixtures/scenario/replay/MANIFEST.toml` [architecture.md:3259] — deferred to the architecture-views regeneration, which is its own task
- [x] [Review][Defer] Collecting all validation errors instead of stopping at the first [crates/opencmdb-core/src/trap.rs] — deferred to 4.7, unchanged

_Dismissed as noise (1): "an empty manifest is green" as a general claim — measured false. With a real corpus, emptying the manifest reds loudly through orphan detection (3 findings). Only the corpus-of-READMEs-only case survives, and it is listed above as its own narrower patch._

## Dev Notes

### The manifest lives at the corpus root — decided 2026-07-21 (Guy)

D56 names `fixtures/scenario/replay/MANIFEST.toml`, written when replay streams were the only artefact imagined. The corpus now also has `scenario/traps/`, and will have `capture/`. A manifest scoped to one subdirectory cannot express orphan detection over the corpus — "a file present under `fixtures/` but absent from the manifest" is not answerable by a file that only knows about `replay/`.

**Decided: one `fixtures/MANIFEST.toml` at the corpus root**, replacing today's `fixtures/MANIFEST`. It matches where the current file already sits, it makes AC3 well-defined, and it keeps one lock for one corpus. This is a deviation from D56's literal path — a small one, and of the same kind already corrected in the architecture diagram (`da23f9f`), and it was raised rather than taken quietly.

### The dependency question this story must not get wrong

`xtask` today depends on `anyhow`, `sha2`, `walkdir` — nothing else, and **nothing from the workspace**. The gate must validate trap files (AC5), and the real reader (`read_traps`) lives in `opencmdb-bin`, which is a **binary crate with no lib target**: `xtask` cannot import from it.

Three ways out, and the third is the one to take:

1. `xtask` depends on `opencmdb-core` for the types and re-implements the reading. **Rejected**: two readers of one format drift, and the drift is invisible until a trap file passes one and fails the other — the exact asymmetry story 4.1's review found between the reader and the lock.
2. Give `opencmdb-bin` a `lib.rs`. **Rejected here**: a real change to the crate's shape, for one gate, in a story about a manifest.
3. **Split by concern: `xtask` owns BYTES, a test in `opencmdb-bin` owns MEANING.** The gate hashes and detects orphans (no domain knowledge needed); a discovery test in `bin` parses and cross-checks using the real `read_traps`. Both run in CI (`cargo test --workspace` and `cargo xtask ci` are both CI steps), so coverage is equivalent, there is exactly one reader, and no new dependency edge appears.

### What already exists — do not rewrite it

- `gate_fixture_manifest` [Source: xtask/src/main.rs:403] and its helpers `parse_manifest` / `manifest_findings`. The **verify logic is stable and tested**: sha mismatch, byte match, case-insensitive comparison, a missing file, malformed lines, the no-fixtures case [Source: xtask/src/main.rs:760-845]. Only the PARSER changes. Do not redesign the finding format or the reporting.
- `manifest_findings` takes its reader as `&dyn Fn(&str) -> std::io::Result<Vec<u8>>` — that injection is what makes the gate testable without touching the filesystem. Keep it, and give the orphan walk the same treatment.
- `read_traps` and `read_jsonl` [Source: crates/opencmdb-bin/src/fixtures.rs]. `read_traps` already validates and cross-checks; Task 4 is about DISCOVERY, not about re-implementing what it does.
- `scratch_dir(tag)` for tests that write files — pid-suffixed and cleaned up. A shared constant temp path was a review finding in 4.1; do not reintroduce one.

### The trap this story is most likely to fall into

**Writing a walk that quietly sees less than the whole corpus.** Story 4.1 shipped one — `read_dir(&dir).into_iter().flatten().flatten()` discarded both the directory error and every per-entry error, so an unreadable subtree silently shrank the search space and the test went green. The review caught it; the fix was `entry.file_type()` (which does not follow symlinks) plus loud failures. **This story's walk is a GATE, so the same defect there is strictly worse.** `walkdir` is already a dependency and handles both concerns — use `follow_links(false)` (its default) and do not swallow its `Result`s.

### Testing standards

- Gate tests live beside the gate in `xtask/src/main.rs`, in the established style: build the input in memory, call the pure helper, assert the findings. Follow it.
- **Prove-to-red is the house rule for gates** (story 1.3 exists solely for it): a gate whose red has never been observed is decoration. AC3 and AC4 each need one.
- Full local gate before done: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo xtask ci`. Epic 3's retrospective recorded four CI-only failures from skipping `--all-targets` or `xtask ci` locally.

### Previous story intelligence (4.1 and 4.2, both reviewed)

- **`#[serde(deny_unknown_fields)]` everywhere** — on `Observation`, `Fact`, `Scope`, `Capabilities` (4.1) and on `Trap`, `TrapFile`, `Expectation` (4.2). The manifest types join them.
- **One path expression only.** `FIXTURES_DIR` lives in `fixtures.rs` and a test counts *occurrences* across `crates/*/src` and `xtask/src`. `xtask` reaches the corpus through its own `root.join("fixtures")`, which is a different mechanism and does not match the needle — check the test still passes and do not add a second copy of the literal.
- **`fixture_path` refuses absolute paths, `..` and `./`.** A manifest `path` should be held to the same rule: a lockfile whose keys can escape the corpus is not a lock.
- **Messages must name the offender.** Both reviews found tests asserting only that an error occurred, and one asserting a substring that was a constant of every message. Assert the cause, and assert the name.
- Two of the six items deferred from 4.1 and 4.2 are closed by this story (orphan detection, duplicate `obs_id`); the third (collecting all validation errors instead of the first) stays deferred to 4.7.

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#D56 — the manifest as a lockfile for data; `capture/` ↔ `scenario/`; the re-capture tool's signature]
- [Source: _bmad-output/planning-artifacts/epics.md#Story 4.3 — the four acceptance criteria this story expands]
- [Source: _bmad-output/implementation-artifacts/deferred-work.md — the orphan and duplicate-`obs_id` entries this story closes]
- [Source: xtask/src/main.rs:397-520 — the gate, its parser and its injected reader]
- [Source: xtask/src/main.rs:760-845 — the existing gate tests, whose style to follow]
- [Source: crates/opencmdb-bin/src/fixtures.rs — `read_jsonl`, `read_traps`, `fixture_path`, `scratch_dir`]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.8 (1M context) — `claude-opus-4-8[1m]`

### Debug Log References

- Orphan detection proven end to end, not only by unit test: dropping an unlisted
  `sneaked-in.jsonl` into `fixtures/scenario/replay/` turned `cargo xtask ci` red naming it, and
  removing it returned the gate to green.
- The gate now reports `3 fixture(s) match their recorded sha256 (0 generated, 3 hand-authored)`.
- Workspace: 42 + 43 + 28 tests (xtask gained 5).

### Completion Notes List

- **The gate fails CLOSED on a corpus with no lock.** The scaffold answered "skipped" when
  `MANIFEST.toml` was absent, which was defensible only while `fixtures/` was empty. A corpus
  present and unlocked is now RED — it is the state the gate exists to forbid.
- **`xtask` owns bytes, a test in `bin` owns meaning.** The gate hashes and detects orphans with
  no domain knowledge; a discovery test recursively walks `scenario/traps/` and runs the REAL `read_traps`
  over every file it finds. That keeps exactly one reader of the format — a second one in `xtask`
  would drift, and the drift would be invisible until a file passed one and failed the other,
  which is precisely the reader/lock asymmetry story 4.1's review found.
- **The discovery test refuses to pass on an empty corpus.** A test that finds nothing and reports
  success is the vacuity the fixtures gate carried from Epic 1 until 4.1 put a file on disk.
- **`generator` earned its keep rather than being silenced.** Clippy flagged the field as never
  read; the honest fix was not `#[allow]` but a rule: a generator record without a `seed` cannot
  reproduce the artefact, so it is a provenance claim nobody can check — now a finding. The gate
  also reports the generated/hand-authored split, so the day a generated artefact enters the
  corpus, it says so.
- **Orphan exemptions are exactly two, and explicit**: `MANIFEST.toml` (a lock cannot list itself)
  and `README.md` files. A test pins that the list is exactly those two.
- **The walk does not follow symlinks, propagates every error, and REPORTS what it will not follow.** Story 4.1 shipped a walk that
  discarded both, so an unreadable subtree silently shrank the search space into a false green.
  The same defect in a GATE would be strictly worse.
- **`read_jsonl` now refuses a repeated `obs_id`**, naming both line numbers. The labelling format
  points at observations "never by line number"; two lines sharing an id void that guarantee.

### File List

- `xtask/src/main.rs` (modified — TOML manifest, orphan detection, corpus walk, 5 new tests)
- `xtask/Cargo.toml` (modified — `serde`, `toml`)
- `crates/opencmdb-bin/src/fixtures.rs` (modified — duplicate `obs_id`, trap discovery test)
- `fixtures/MANIFEST.toml` (new — replaces `fixtures/MANIFEST`)
- `fixtures/MANIFEST` (deleted)
- `fixtures/README.md` (modified — both directions of the lock, and the parse guarantee)
- `Cargo.lock` (modified)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modified)

### Change Log

- 2026-07-21 — The corpus is frozen in both directions: a listed artefact whose bytes changed is
  red, and a file nobody listed is red. Trap files are discovered and parsed rather than hashed
  and trusted. `obs_id` became a real anchor. Closes two of the six findings deferred from the
  4.1 and 4.2 reviews.
- 2026-07-21 — Code review (three parallel layers, all probing empirically): 27 patches applied,
  2 decisions taken, 3 deferred, 1 dismissed on measurement. **AC5 was NOT satisfied**: trap
  discovery used `read_dir`, one level, where the criterion said "walking the corpus" — and trap
  families are exactly what will add a subdirectory. A reviewer planted a listed, correctly-hashed
  trap at `traps/family/broken.toml` pointing at a nonexistent stream: gate green, test green.
  Both directions now catch it.
  Also closed: symlinked files AND directories were invisible to the walk (a symlinked directory
  hid an arbitrarily large subtree); `.replace('\\', "/")` aliased a real `a\b.jsonl` onto a
  listed `a/b.jsonl` so a smuggled file passed as a legitimate one; the README exemption was a
  string suffix, so `evil-README.md` escaped; duplicate manifest paths inflated the success count;
  a listed FIFO hung the gate forever; and a deleted `fixtures/` reported green. Test coverage was
  the meta-defect — every gate test exercised a helper, none the gate, so a wiring mistake between
  the two directions would have shipped green; there are now six gate-level tests against a real
  corpus on disk.
  Two false claims in this record were corrected rather than quietly fixed, and the deferred-work
  register was repaired: an earlier edit had enumerated what remained open and dropped the
  duplicate-manifest-path half.
