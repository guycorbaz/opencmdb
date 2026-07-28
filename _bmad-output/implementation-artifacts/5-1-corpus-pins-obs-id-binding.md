# Story 5.1: The corpus pins the obs_id-to-line binding, and every stream goes through the connector

Status: review

<!-- Validation is MANDATORY here (Guy's decision, Epic 4 retrospective 2026-07-26): two
     fresh-context agents (fact-check + gap-hunt) before dev-story. The template banner saying
     validation is optional does not apply to this project. -->

> **Validation pass held 2026-07-26** — two fresh-context agents (fact-check + gap-hunt). Three
> HIGH and six MEDIUM findings, all applied above. The three HIGH: the file-size gate claim was
> backwards (the hoist RAISES the count, and the count is 698 not 699); a FOURTH byte-fidelity
> register entry (story-4.13's) was about to be silently orphaned; and *"the one place
> `UndeclaredFactKind` bites"* was false twice over — the dev had been told to put it in a doc
> comment. Both agents independently re-measured the corpus byte-shape and the `Serialize` derive;
> both came back clean, so AC3 carries a measurement rather than a hope.

## Story

As the owner of the corpus,
I want each byte-pin test to assert the `obs_id` of the line it reads, and every committed replay
stream to be loaded through `FixtureConnector::load` and round-tripped through serde,
so that a re-authored stream cannot invert what its traps judge — or drift out of the canonical
byte-shape — while every assertion stays green.

**This is inherited debt, placed at the HEAD of Epic 5 on Guy's decision** (epics.md:1312): the
corpus byte-fidelity theme had accumulated three unowned entries in `deferred-work.md`. It comes
first because **Epic 5 bumps the corpus**, and hardening after the bump means replaying every entry
against artefacts that have moved. **No corpus bytes change in this story** — it is test-side work
only, plus one `derive` (see AC3). If a task appears to require re-authoring a committed artefact or
re-hashing `MANIFEST.toml`, STOP: that is a finding, not a task (see Dev Notes → "The corpus does
not move here").

## Acceptance Criteria

1. **AC1 — the `obs_id` ↔ line binding is pinned on the two byte-pins that read purely by index.**
   **Given** `the_dhcp_churn_stream_moves_the_address_only_through_observed_at`
   (`crates/opencmdb-bin/src/fixtures.rs:1789`) and
   `the_vrrp_stream_shares_one_virtual_mac_and_moves_its_uplink` (`fixtures.rs:1866`), which read
   observations by INDEX while `dhcp-churn.toml` / `vrrp-virtual-mac.toml` judge by `obs_id`
   **when** the pins are strengthened **then** each pinned observation asserts its `obs_id`:
   - `dhcp-churn.jsonl` — 3 lines, `adadadad-0000-4000-8000-000000000001` … `…0003`;
   - `vrrp-virtual-mac.jsonl` — 4 lines, `aeaeaeae-0000-4000-8000-000000000001` … `…0004`.

   **And** the assertion is expressed ONCE. Six call sites of the same loop is accidental
   duplication, which the DRY rule forbids: introduce a single test helper in `fixtures.rs`'s test
   module —

   ```rust
   /// Pin the `obs_id` ↔ LINE binding of a committed stream: line `n` (0-indexed) carries
   /// `{prefix}-0000-4000-8000-{n+1:012}`. The traps judge by `obs_id` while every byte-pin
   /// reads by index; without this, a deliberate re-authoring that swaps two lines' ids (with a
   /// re-hashed manifest) would invert what a family's traps judge while every byte-level
   /// assertion stayed green (registered under story 4.15's review).
   fn assert_obs_ids(observations: &[Observation], prefix: &str) { … }
   ```

   — and fold the four existing copies into it: `fixtures.rs:2093-2103` (hostname-collision, 4.15,
   prefix `afafafaf`), `:2245-2252` (docker-veth, 4.16, `babababa`), `:2397-2404`
   (hostname-absence, 4.17, `bcbcbcbc`), `:2700-2711` (the wire spec, 4.18, `bdbdbdbd` — its loop
   is FUSED with its placeholder-context pins; the `obs_id` half is `:2707-2711`: call the helper
   and drop only that assertion from the loop). If you leave a call site inline, the story record
   must say which and why.

   **And the DRY verdict is argued, not asserted.** The house rule protects deliberate redundancy
   — but all four existing loops already COMPUTE their ids with `format!("…{suffix:012}")` rather
   than restating them, so what is being removed is mechanical duplication, not a second oracle
   (unlike `expected()`, which restates VALUES and must survive). The helper therefore encodes two
   corpus conventions in one place: the fixed `-0000-4000-8000-` middle segment, and sequential
   numbering from 1 — both true of all 13 committed streams as of this story. Say that in its doc,
   and say that a future stream numbered otherwise gets its own assertion rather than being
   re-authored to satisfy a helper.

2. **AC2 — every committed replay stream is admissible to the connector, not merely parseable.**
   **Given** that the 4.4 admissibility layer (foreign `connector_id`, uncovered scope, undeclared
   fact kind, repeated `obs_id`, scripted cancellation, capability ordering) is exercised against
   `minimal.jsonl`, `partial-then-failed.jsonl` and `capability-downgrade.jsonl` only — every
   FAMILY stream since 4.9 is gated for parseability by the fixtures walks and by nobody else
   (registered under story 4.12's review) **when** the corpus walk runs **then** every `.jsonl`
   under `fixtures/scenario/replay/` is loaded through `FixtureConnector::load` and must return
   `Ok`, so corpus-level parseability and connector-level admissibility stop being two different
   claims.

   **And** the walk is driven by a HAND-AUTHORED per-stream table of `(relative path,
   ConnectorId, scopes_covered, initial Capabilities)` — a second, independent statement of each
   stream's context, in the `expected()` idiom — **never derived from the observations**. Derivation
   would make `ForeignConnectorId` and `UncoveredScope` vacuous by construction and would be the
   move `fixture_connector.rs`'s module doc refuses ("a capability read off what was seen cannot
   express *capable of hostnames, saw none*").

   **And** the table is checked in BOTH directions, mirroring the corpus lock's own rule: a stream
   the walk finds with no table entry is RED (the orphan direction), and a table entry naming a
   file that does not exist is RED. A new committed stream must therefore state its context, or
   the suite reds.

   **And** the test's doc comment states plainly what the walk does NOT prove. The walk shows every
   committed stream is ADMISSIBLE; it never observes `UndeclaredFactKind` FIRING. Where the check is
   non-vacuous: `partial-then-failed.jsonl` (`partial_caps()` declares exactly the four kinds the
   stream emits) and `capability-downgrade.jsonl` on BOTH sides of its capability record
   (`downgrade_initial_caps()` declares four; the record narrows to three). Where it is vacuous: the
   eleven `corpus_*` streams, because `corpus_caps()` declares all seven `FactKind`s — deliberately
   wider than what is emitted, which is the whole reason the descriptor exists. Write that; do not
   write "the walk proves fact-kind coverage corpus-wide", and do not write "the one place it
   bites".

3. **AC3 — the round-trip byte-shape witness covers every committed stream, not `minimal.jsonl`
   alone.** **Given** `re_serializing_reproduces_the_committed_bytes` (`fixtures.rs:794`), which
   round-trips `minimal.jsonl` only, so no other stream — and no CONTROL record at all — has its
   exact serialized byte-shape (field order, `MacAddr` array encoding, `Uplink` field names, the
   internally-tagged `record` marker) pinned by a parse→re-serialize→compare test (registered under
   story 4.10's review) **when** the corpus walk runs **then** every `.jsonl` under
   `fixtures/scenario/replay/` is read with `read_records`, re-serialized record by record, and
   compared to the committed bytes LINE BY LINE, the failure naming the file and its 1-indexed line
   number.

   **And** `re_serializing_reproduces_the_committed_bytes` SURVIVES unchanged, with a comment
   saying why the two are not duplicates: it starts from `expected()` — an independently authored
   Rust literal — so it pins the VALUES as well as the shape; the corpus-wide witness starts from
   the file, so it pins the shape only. This is deliberate redundancy of the kind the DRY rule
   protects, and a future DRY pass must not collapse it.

   **And** the control-record shape is covered, which today it is not: `ControlRecord`
   (`fixtures.rs:122-135`) is `Deserialize`-only, so it gains `Serialize` and the round-trip renders
   a `Record::Failure` / `Record::Capability` back to its committed line. **Measured on the pinned
   serde/chrono:** the derive compiles under `tag` + `rename_all` + `deny_unknown_fields` + a
   `#[serde(flatten)] Capabilities`, emits the tag FIRST, and reproduces both committed control
   lines byte-exactly. The `#[cfg(test)]` mirror-struct fallback should therefore NOT be needed; if
   you end up taking it, say why.

   **And** note the mapping the story would otherwise leave implicit: `Record` (`fixtures.rs:88-104`)
   holds `ConnectorError` / `Capabilities` directly, **not** `ControlRecord`. Rendering means
   `Record::Failure(e)` → `ControlRecord::Failure { error: e.clone() }` and `Record::Capability(c)`
   → `ControlRecord::Capability { capabilities: c.clone() }`. Both inner types are `Clone`.

4. **AC4 — every strengthened guard is proven to red before it passes, and the mutation is
   recorded** (house rule, story 1.3). One recorded mutation per AC, minimum:
   - **AC1:** swap two `obs_id` values between two lines of `dhcp-churn.jsonl`, run
     `cargo test -p opencmdb-bin the_dhcp_churn_stream`, observe the named red, `git checkout` the
     file. Record the exact message. (`cargo test` does not consult `MANIFEST.toml`, so no re-hash
     is needed for the observation — and none must be committed.)
   - **AC2:** point one table entry's `ConnectorId` at a different UUID → expect `ForeignConnectorId`
     naming the stream and the offending `obs_id`; then DELETE one table entry → expect the orphan
     direction to red naming the unlisted stream. Both mutations are test-side; the corpus is not
     touched.
   - **AC3:** insert one space after a colon on line 1 of **`dhcp-churn.jsonl`** → expect the
     round-trip red naming that file and line 1; `git checkout` the file. **Not `minimal.jsonl`:**
     an edit there also reds `the_committed_fixture_reads_back_exactly` (`fixtures.rs:784`) and
     `re_serializing_reproduces_the_committed_bytes` (`:794`), so the new guard would be observed
     for the wrong reason — the same care AC1's mutation takes. Then do the same on the CONTROL
     line of `capability-downgrade.jsonl` (line 3), so the control-record half is proven too — a
     round-trip that silently skipped control records would pass the first mutation.

5. **AC5 — the local gate is green and the corpus is byte-identical.** `cargo fmt --all`,
   `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and
   `cargo xtask ci` all pass, and `git status` shows no change under `fixtures/`. The
   `views-hash STALE` line from `xtask ci` is expected and exits 0 — **do not regenerate
   `architecture-views.md` in this story** (it is a milestone task, project-context.md:127-130).

6. **AC6 — the register is updated by APPENDING, never by rewriting.** The three
   `deferred-work.md` entries this story closes are marked closed in place, in the file's
   established idiom (`✅ **CLOSED by story 5.1.** ~~old text struck~~ …`), keeping the original
   text readable:
   - "Deferred from: code review of story-4.10" — the round-trip byte-shape entry (AC3);
   - "Deferred from: code review of story-4.12" — the `FixtureConnector::load` entry (AC2);
   - "Deferred from: code review of story-4.15" — the `obs_id` ↔ line binding entry (AC1).

   **And** the FOURTH byte-fidelity entry is annotated in place as **still open**, not silently
   left behind: `deferred-work.md:365-378` (story-4.13's review — *"the dhcp-churn byte-pin test
   pins MAC/hostname values relationally, never by value"*), whose owner line names this very
   theme. **This story does not close it:** AC3's round-trip starts from the FILE, so it pins the
   byte-SHAPE and never the authored values; AC1 pins `obs_id`s only. Append a one-line
   `↺ **STILL OPEN after story 5.1** — …` note saying exactly that; do NOT strike it. *A register
   that loses an item is worse than no register* (the file's own line 16).

   **And** a new `## Deferred from: story-5.1` section records the finding this story surfaces but
   does NOT close (see Dev Notes → "Found while scoping, deliberately out of scope"): four
   committed family streams — `randomized-mac.jsonl`, `multi-nic.jsonl`,
   `shared-hardware-vm.jsonl`, `cloned-mac.jsonl` — have **no byte-pin test at all**, so their
   `obs_id` ↔ line binding and their authored values are asserted by nothing narrower than the
   corpus walks and the sha256 lock. AC1 cannot cover them: there is no test to strengthen.

   **And** the entry names its OWNER, because the work is already scheduled: **story 5.2b**,
   inserted on 2026-07-26 in Epic 5's debt block, immediately after 5.2 and ahead of the L1 join
   (epics.md, "Story 5.2b"). The entry says so — a register item with a named owner and a slot is
   not a deferral, and writing it as one would misstate the plan.

   **And** this stays in `deferred-work.md` and does NOT become a GitHub issue. The register is the
   established home for review-surfaced corpus debt (every entry since 4.1); an issue is reserved
   for scope that MOVES between epics — the 4.19b precedent (#34). Nothing moves here: the work
   stays in Epic 5, three stories later.

## Tasks / Subtasks

- [x] **Task 1 — the `obs_id` pin helper (AC1)**
  - [x] Add `assert_obs_ids(observations: &[Observation], prefix: &str)` to `fixtures.rs`'s test
        module, with the doc comment stating WHY (traps judge by `obs_id`, byte-pins read by index).
  - [x] Call it from `the_dhcp_churn_stream_moves_the_address_only_through_observed_at` (`adadadad`)
        and `the_vrrp_stream_shares_one_virtual_mac_and_moves_its_uplink` (`aeaeaeae`).
  - [x] Fold the four existing copies (`:2093-2103`, `:2245-2252`, `:2397-2404`, `:2700-2711`) into
        the helper, and record the DRY argument from AC1 in its doc. **All four folded, none left
        inline.** The wire-spec site (`:2700-2711`) kept its fused placeholder-context pins and lost
        only its `obs_id` half, exactly as AC1 prescribes.
  - [x] Prove-to-red per AC4, record the message.

- [x] **Task 2 — share the replay walk across the two test modules (AC2, AC3)**
  - [x] Hoist `walk_replay_streams` (`fixtures.rs:1389`) out of `mod tests` to module scope as
        `#[cfg(test)] pub(crate) fn`, placed immediately BEFORE `#[cfg(test)] mod tests`. Its doc
        comment is kept and extended with the new callers.
  - [x] Confirm the `file-size` gate stays green — and do NOT claim a reduction. **Measured: 698 →
        720 code lines, a RISE of 22** (the hoisted doc comment plus its blank separator, counted
        after `cargo fmt`), far under the 2000 ceiling. The gate reports
        `✅ file-size 20 file(s) under 2000 code lines (largest: 884)` — `fixtures.rs` is not the
        largest file in the tree.
  - [x] Do not touch `trap_gate.rs`'s `discover_trap_files`. **Not touched, not read for editing.**
        Its sentence stays true: `walk_replay_streams` is still `#[cfg(test)]` and `trap_gate` gained
        no caller.

- [x] **Task 3 — the connector admissibility walk (AC2)**
  - [x] Build the per-stream table from the THREE contexts that already exist there. Reused via a
        `corpus(relative_path)` closure for the eleven; `partial_*` and `downgrade_*` spelled out.
        **No fourth copy of any UUID was authored.**
  - [x] 13 entries, exactly.
  - [x] **Convert the walked path correctly** — `path.strip_prefix(fixtures_dir())`, the `fixtures/`
        prefix never re-written. `the_fixtures_path_is_expressed_once` stays green (still exactly two
        occurrences).
  - [x] Walk `scenario/replay/`, `load` each stream with its table entry, `expect` `Ok`.
  - [x] Assert both directions: no stream without an entry, no entry without a file. **Three guards
        in total** — plus `checked == table.len()`, which also catches a duplicated entry.
  - [x] Doc comment states the honest limit (AC2's last clause) AND the `as_of` caveat.
  - [x] **Update two stale doc comments** — `corpus_id()` and `corpus_caps()` now say they are the
        declared context of ELEVEN streams, and `corpus_caps()` says its wideness is exactly why the
        fact-kind check is vacuous for those eleven.
  - [x] Prove-to-red per AC4 (both mutations) — **plus a third**, for the entry-without-a-file
        direction, which AC4 did not name but which is its own guard.

- [x] **Task 4 — the corpus-wide round-trip witness (AC3)**
  - [x] Add `Serialize` to `ControlRecord`'s derive. **The mirror-struct fallback was NOT needed** —
        the derive compiles under `tag` + `rename_all` + `deny_unknown_fields` + the flattened
        `Capabilities`, emits the tag first, and reproduces both committed control lines byte-exactly,
        confirming the story's measurement.
  - [x] Add the render path and the walking test in `fixtures.rs`'s test module, beside
        `re_serializing_reproduces_the_committed_bytes`. The `match` in `render_record` is exhaustive
        with no `_` arm.
  - [x] **Get the line number right** — zipped against
        `text.lines().enumerate().filter(|(_, l)| !l.trim().is_empty())`, both exhaustion directions
        asserted with their own message.
  - [x] Compare line by line; the panic message names file + 1-indexed line + both strings.
  - [x] Leave `re_serializing_reproduces_the_committed_bytes` in place and add the
        "not-a-duplicate" comment (AC3).
  - [x] Prove-to-red per AC4 (observation line AND control line).

- [x] **Task 5 — the register (AC6)**
  - [x] Mark the three entries closed in place, appending, never rewriting. The idiom used is the
        file's own (`✅ **CLOSED by story 5.1.** … ~~original headline struck~~ …`), with the body
        prose of each entry left intact and readable.
  - [x] Annotate the FOURTH entry (story-4.13's review) `↺ **STILL OPEN after story 5.1**`, not
        struck.
  - [x] Add the `## Deferred from: story-5.1` section with the four-unpinned-families finding, its
        owner (story 5.2b) and the reason it is not a GitHub issue.

- [x] **Task 6 — the gate and the branch (AC5)**
  - [x] Full local gate: all four commands run, all green (see Completion Notes).
  - [x] `git status` clean under `fixtures/`; `MANIFEST.toml` unchanged.
  - [x] Update `sprint-status.yaml` with a comment saying what was delivered AND what moved.
        **Set to `review`, not `done`** — `code-review` has not run, and this project's flow is
        `dev-story → code-review → merge`. Writing `done` here would claim a review that has not
        happened; `done` is the merge's business. Flagged for Guy in the completion message.
  - [ ] Branch → PR → green CI → squash merge. **NOT DONE — deliberately deferred, and this box
        stays unchecked.** The work is committed on `story/5-1-corpus-byte-pins` and has NOT been
        pushed; no PR is open. `code-review` runs first (project-context.md's flow, and it is
        recommended on a fresh context / different LLM), then push + PR + green CI + squash merge.

### Review Findings

_Code review held 2026-07-27 — three parallel layers (Blind Hunter, diff only · Edge Case Hunter,
diff + read access · Acceptance Auditor, diff + spec + context docs). Every factual claim below was
re-verified against the tree by the reviewer, not taken from a layer's word. AC verdict: **AC1 MET ·
AC2 MET · AC3 MET · AC4 PARTIAL · AC5 MET (gate reproduced independently: clippy clean, 249 tests,
all `xtask ci` gates green, first `#[cfg(test)]` at line 721 → the 698→720 measurement is correct) ·
AC6 PARTIAL**._

> **Resolution, 2026-07-28** — all **12 `[Review][Patch]` items applied**; the 5 `[Review][Defer]`
> items are registered in `deferred-work.md` under `## Deferred from: code review of story-5.1`.
> **AC4 and AC6 now stand MET**: AC4's citations were re-observed on the final tree and six further
> mutations were taken (12 total, one assertion named as unfalsifiable rather than credited with a
> red), and AC6's register entry no longer rests on a check its own commit falsified. Two of Guy's
> three arbitrations knowingly revise the story's own text — the `assert_obs_ids` shape and the
> `done`-vs-`review` instruction — and both revisions are annotated in the Dev Notes rather than
> rewritten over. See Completion Notes → "The review-fix pass" for what changed and for the one
> thing deliberately NOT recorded (the scratch directories are not a cause for issue #38).

- [x] [Review][Patch] **`assert_obs_ids` pins `obs_id` ↔ observation INDEX, not ↔ file LINE, and asserts no count** — **RESOLVED 2026-07-27 (Guy): strengthen the helper.** It gains an `expected_len: usize` parameter and asserts the count itself, so it is non-vacuous by construction and story 5.2b inherits a shape that cannot pass on a truncated stream; the doc is reworded to *"the n-th OBSERVATION (0-indexed)"* and names `capability-downgrade.jsonl` as the counter-example where observation index ≠ file line. This knowingly revises the Dev Notes' *"a shape four more families can call without change"* — the shape changes, and 5.2b's four call sites pass their length instead of restating it beside the call. Costs six call-site edits and one prove-to-red. Detail — its doc (`fixtures.rs:1893`) says *"line `n` (0-indexed) carries `{prefix}-…-{n+1:012}`"* and *"Both hold for all 13 committed streams"*. **Verified false for `capability-downgrade.jsonl`**: its capability record is on file line 3, so `obs_id …0003` sits on file **line 4**. The slice always comes from `read_jsonl`, which drops control records, so file lines are invisible to the helper by construction. Separately, the body is one bare `for` loop with no length assertion — `assert_obs_ids(&[], "afafafaf")` is green, whereas the four folded loops hard-indexed fixed positions and would panic on a truncated stream. All six call sites happen to assert `observations.len()` first (`:1930/2011/2226/2372/2519/2820`), so AC1's counts hold today — but the guard lives at the call site, not in the helper, and story 5.2b adds four call sites to streams with no such sibling assert. **This is the vacuity story 5.1 exists to close, re-introduced in the helper that closes it.** Choice: change the helper's shape (take an expected count, or take line numbers) versus keep the shape 5.2b was promised and weaken the doc + push the count onto the call sites by convention.
- [x] [Review][Patch] **The corpus-wide round-trip pins per-line CONTENT, never the bytes between or after lines** — **RESOLVED 2026-07-27 (Guy): add the two guards, with a prove-to-red each.** `assert!(text.ends_with('\n'))` and a `\r` refusal go into the witness, so its name stops overstating what it pins and the story's own threat model — a deliberate re-authoring WITH a refreshed manifest, the one case where the sha256 lock is not the backstop — is actually covered. Mutations: truncate the final newline of `dhcp-churn.jsonl`, then convert it to CRLF; `git checkout` after each, no `MANIFEST.toml` re-hash. Detail — `fixtures.rs:926-929` compares `text.lines()` output, and `str::lines()` strips a trailing `\r`; there is no `ends_with('\n')` check and no CR rejection. So a stream re-authored with CRLF, without its final newline, or with a blank line inserted round-trips green — for 12 of 13 streams, since only `minimal.jsonl` has the older whole-file `assert_eq!(rendered, on_disk)` (`:874`). **Measured: all 13 committed streams are LF-only and newline-terminated today, so nothing is currently wrong** — but the test is named `…re_serializes_to_its_committed_bytes` and its doc states the shape-vs-values limit while omitting this one. The threat model the story cites is a deliberate re-authoring *with a refreshed manifest*, which is exactly the case where the sha256 lock stops being the backstop. Choice: add the two guards (costs a prove-to-red) versus state the limit in the doc and register it for 5.2b.
- [x] [Review][Patch] **`fixtures/scenario/replay/.claude/.cc-writes` exists in the working tree and the walk descends into it** — **RESOLVED 2026-07-27 (Guy): exempt dot-entries in the walk.** `walk_replay_streams` skips any entry whose name starts with `.`, file or directory, which closes the class rather than the instance — whatever the next tool writes under `fixtures/`. The cost is named rather than hidden: a `.hidden.jsonl` would no longer be seen by the walk, which is acceptable because the corpus never hides an artefact (`MANIFEST.toml` lists every one by its visible name, and the xtask lock walks the same tree from the manifest side). The existing empty directories are left in place — the walk no longer cares. Detail — verified present, created 2026-07-26 20:29, currently **empty**, which is the only reason the suite is green. `walk_replay_streams` (`fixtures.rs:739-753`) pushes every subdirectory and then asserts `is_jsonl` with only an exact-name `README.md` exemption, so the first file any tool writes under there reds four tests in `fixtures.rs` plus the new connector walk, with the message *"only .jsonl replay streams and README.md belong under scenario/replay/"* pointing at a tooling artefact. The diff widened the blast radius: the new `assert_eq!(checked, table.len())` (`fixture_connector.rs:1618`) makes such a failure read as *"the context table is wrong"*. ⚠️ **This is NOT a cause for issue #38 and must not be recorded as one** — the directories postdate stories 4.15/4.17 by a day and are empty; *a cause needs a check, not a plausible story*. Choice: exempt dot-entries in the walk (slightly weakens the "only .jsonl belongs here" gate) versus `.gitignore` + remove the directories (leaves the walk fragile to the next tool).
- [x] [Review][Patch] **The cited verification is falsified by this very commit** — `deferred-work.md:458`, this story `:336` and `epics.md:1371` all state the four families are *"named by NO test in the tree — verified by `grep -rn "<name>.jsonl" --include=*.rs crates xtask`, which returns nothing for all four"*. **I re-ran that exact grep: four hits**, `fixture_connector.rs:1519-1522`. The entry's *conclusion* (no byte-pin/value test) still holds; its named *check* no longer does. Worst placement is `epics.md`, where it is the **Given** clause of story 5.2b — the next story's premise. Weaker true sentence: *"named by no VALUE test; their only mention is the context table added in 5.1."* [`deferred-work.md:458`, `5-1-…md:336`, `epics.md:1371`]
- [x] [Review][Patch] **Both quoted panic locations in the Debug Log point at code that is not in the tree** — mutation 1 is recorded as `fixtures.rs:1803:13`, but the `assert_eq!` bearing *"line {n} carries its authored obs_id"* is at **`:1913:13`** (`:1803` is an unrelated trap literal, `id = "points-at-nothing"`); mutation 5 is recorded as `:941:17`, but the round-trip `assert_eq!` opens at **`:939`** (`:941` is its `line,` argument). The messages match the shipped code and both guards demonstrably exist and pass — the citations were taken at intermediate states and not refreshed, so nobody can reproduce the recorded observation from the committed tree. [`5-1-…md:485`, `:523`]
- [x] [Review][Patch] **The record contradicts itself on how many mutations were taken** — *"five"* at `5-1-…md:479` and at `sprint-status.yaml:168` (**the live source of truth**) versus *"six"* at `:590`, `:641` and in the commit message. Six are enumerated. And `:590`'s parenthetical is wrong in the other direction: AC4 *required* the control-line mutation on `capability-downgrade.jsonl:3`, so only **one** was surplus (the entry-without-a-file direction), not two. [`sprint-status.yaml:168`, `5-1-…md:479`, `:590`]
- [x] [Review][Patch] **A new doc comment asserts a failure mode the code structurally cannot have** — `fixtures.rs:124`: *"Without it the witness would silently skip every control record."* Without `Serialize`, `render_record`'s `match` (`:895-908`, exhaustive, no `_` arm, deliberately) would not **compile** — a build error, not a silent skip. The stated counterfactual describes a design the code refuses. Weaker true sentence: *"so control records are covered rather than excluded from the witness."* [`crates/opencmdb-bin/src/fixtures.rs:124`]
- [x] [Review][Patch] **Three guards ship in the connector walk with no recorded red, and one can never have one** — `fixture_connector.rs:1607` `assert!(walked.insert(…), "walked twice")` is **unfalsifiable**: a stack walk that panics on symlinks cannot yield one path twice. (The `insert` itself is load-bearing — `walked` is what the orphan-entry loop reads — only the assertion is dead.) `:1610` `checked > 0` and `:1618` `checked == table.len()` also ship unproven, while Task 3 credits the latter with *"which also catches a duplicated entry"* (true by construction — 14 entries vs 13 files — but never observed) and `deferred-work.md:368` says *"All three guards proven to red"* over a walk that ships more than three assertions. Either record a mutation or say plainly which assertions are defence-in-depth that cannot be proven red. [`crates/opencmdb-bin/src/fixture_connector.rs:1607`]
- [x] [Review][Patch] **`"no replay stream found under scenario/replay/"` is now verbatim in five places across two files** — `fixtures.rs:961, 998, 1517, 1539` and `fixture_connector.rs:1610`. So the panic never says WHICH walk found nothing, against this file's own testing standard (*a message must name the offending thing, or it is not actionable*). Four are pre-existing; **the diff added the fifth**, and the hoist created the obvious single home: assert non-emptiness inside `walk_replay_streams`, which already returns the count, and the invariant becomes unskippable for the callers 5.2b adds. [`crates/opencmdb-bin/src/fixture_connector.rs:1610`]
- [x] [Review][Patch] **Three doc comments overstate or will decay** — (a) the round-trip's name and doc say *"every committed stream"* while `scenario/wire/unifi-clients.expected.jsonl` is a committed `.jsonl` deliberately outside the walk: say *"every stream under `scenario/replay/`"*; (b) the hoisted walker's doc (`fixtures.rs:712-715`) enumerates **five callers** — an inventory nothing checks and that 5.2b falsifies, while the load-bearing sentence (*"two claims … must walk the same tree"*) needs no inventory to be true; (c) the helper's doc scopes its conventions to *"all 13 committed streams"* while its sixth call site is the wire artefact, a 14th. [`crates/opencmdb-bin/src/fixtures.rs:712`, `:900`, `:1908`]
- [x] [Review][Patch] **`epics.md` story 5.2b contradicts its own arity** — the randomized-mac AC opens *"**Given** `randomized-mac.jsonl` — 3 presences"*, names N1/N2/N3, then constrains *"**both lines** carry exactly 2 facts"*. N3's fact count is left unspecified in the one spec whose entire purpose is that a family cannot state a premise its bytes contradict. Also the title (*"The four unpinned families"*) undercounts: the story's fifth Given extends `dhcp-churn`'s existing pin — the body's *"I want"* does say so, but the title is what the tracker carries. [`_bmad-output/planning-artifacts/epics.md`, Story 5.2b]
- [x] [Review][Patch] **The story mandates in one section what it refuses in another** — Dev Notes: *"It is `ready-for-dev` today and **must reach `done`** in this story's File List"*, versus Completion Notes deviation 1 setting `review`. The deviation is correctly declared and is the right call; annotate the Dev Notes line so the document has a single answer to *"what was required"*. [`5-1-…md:372`]
- [x] [Review][Defer] **`walk_replay_streams` never symlink-checks its own root** [`crates/opencmdb-bin/src/fixtures.rs:723`] — deferred, pre-existing.
- [x] [Review][Defer] **No `is_file()` check: a FIFO named `x.jsonl` makes the suite HANG rather than fail** [`crates/opencmdb-bin/src/fixtures.rs:733`] — deferred, pre-existing.
- [x] [Review][Defer] **The walk yields unsorted `read_dir` order while its sibling `trap_gate.rs` sorts for determinism** [`crates/opencmdb-bin/src/fixtures.rs:726`] — deferred, pre-existing.
- [x] [Review][Defer] **`scenario/wire/unifi-clients.expected.jsonl` has no round-trip byte-shape pin at all** [`fixtures/scenario/wire/unifi-clients.expected.jsonl`] — deferred, deliberately outside AC3's scope.
- [x] [Review][Defer] **Four nits: unconditional `Serialize`, a doubled path in a panic, `{:012}` decimal in a hex field, an inline `serde::` path** [`crates/opencmdb-bin/src/fixtures.rs:126`] — deferred, cosmetic.

_Dismissed as noise (2): **AC2's banned sentence "is present"** — it appears only at `fixture_connector.rs:1559` as an explicit denial (`This is not "the walk proves fact-kind coverage corpus-wide".`); letter, not intent, and the denial is clearer than a paraphrase would be. **`strip_prefix` string comparison is platform-separator dependent** — Linux/Docker-only project (D64, MariaDB on DSM), no Windows CI, not a live path._

## Dev Notes

### The corpus does not move here

This story adds tests and one `derive`. It re-authors nothing. `fixtures/` is a SPEC locked in both
directions by `MANIFEST.toml` (edited AND orphan), and a bump reads in review as *"I am changing the
spec"*. **The measurement says no bump is needed:** every one of the 13 committed streams is already
compact JSON (`,`/`:` separators, no spaces) with top-level keys in `Observation`'s declaration
order (`obs_id, connector_id, observed_at, scope, facts, raw`), nested orders matching
(`scope: {l2_domain, vantage}`, `Mac: {addr, locally_administered}`, `Uplink: {peer_mac, peer_port}`,
`Hostname: {name, source}`), and the one capability line's `kinds` in `BTreeSet`/declaration order
(`["Mac","IpV4","Hostname"]`). *How this was checked:* each line was re-emitted with a compact JSON
dump preserving the file's own key order and compared to the raw line, and the key orders were
enumerated across all 13 files — a whitespace/order check outside Rust, **not** a run of the actual
serde serializer. A second, independent pass re-canonicalized all 13 streams (51 observation lines +
2 control lines) plus the wire file against the real Rust declaration orders and confirmed **zero
mismatches**; it also established that no committed stream carries a blank line, every file is
newline-terminated, and every `observed_at`/`as_of` is second-precision `Z`-suffixed (so chrono's
`AutoSi` round-trips exactly). The Rust round-trip remains the real check and it is Task 4's job to
observe it. If it reds on a committed line, that is a genuine finding: report it and stop rather
than re-authoring the corpus to fit the test.

### The corpus as measured (drives Task 3's table)

Every stream carries exactly ONE `connector_id` and ONE `Scope`. Three contexts cover all 13:

| Context (existing helpers) | `connector_id` | `l2_domain` / `vantage` | Streams |
|---|---|---|---|
| `corpus_*` | `3333…` | `1111…` / `2222…` | `minimal`, `example-traps`, `randomized-mac`, `multi-nic`, `shared-hardware-vm`, `cloned-mac`, `dhcp-churn`, `vrrp-virtual-mac`, `hostname-collision`, `docker-veth`, `hostname-absence` (11) |
| `partial_*` | `4444…` | `5555…` / `6666…` | `partial-then-failed` |
| `downgrade_*` | `7777…` | `8888…` / `9999…` | `capability-downgrade` |

`corpus_caps()` declares all seven `FactKind`s, so it admits every family stream's kinds
(`Mac`, `IpV4`, `Hostname`, `Uplink`, `OuiVendor`, `Rtt` are what the corpus actually emits;
`DhcpLease` is declared and never emitted — deliberately, per its doc). `partial_caps()` declares
`Mac/IpV4/Rtt/Hostname`, which is exactly what `partial-then-failed.jsonl` emits.
`downgrade_initial_caps()` declares `Mac/IpV4/Rtt/Hostname` and the file's own capability record
narrows to `Mac/IpV4/Hostname` mid-stream; the two observations after it emit `{IpV4, Hostname}` and
`{Mac}` — inside the narrowed set.

**Where the fact-kind check is non-vacuous:** `partial-then-failed.jsonl` (tight descriptor — a
fifth kind added to that stream would red) and `capability-downgrade.jsonl` on both sides of its
record. **Where it is vacuous:** the eleven `corpus_*` streams, since `corpus_caps()` declares all
seven kinds. And "non-vacuous" is not "fires": the walk asserts every stream is admissible, so
`UndeclaredFactKind` is never observed. Keep those three sentences distinct — collapsing them is
how a false doc comment gets written.

Control records exist in exactly two streams: a terminal `failure` (last line of
`partial-then-failed.jsonl`) and one `capability` (line 3 of `capability-downgrade.jsonl`). Task 4's
round-trip must render both, or the control half is untested — hence AC4's second AC3 mutation.

### Scope boundary: `scenario/replay/` only

`fixtures/scenario/wire/unifi-clients.expected.jsonl` is a `.jsonl` of `Observation`s but sits
OUTSIDE `scenario/replay/`, is `CONSUMER PENDING` until Epic 11's parser (issue #34), and already
has its own test which is also its privacy coverage
(`the_wire_spec_encodes_the_measured_field_behaviours`, `fixtures.rs:2584`, and see its doc: *"this
directory sits outside every corpus walk"*). **It is out of scope for AC2 and AC3.** Do not widen
the walk to reach it; do not move it.

### Found while scoping — out of scope HERE, and owned by story 5.2b

`randomized-mac.jsonl` (4.9), `multi-nic.jsonl` (4.10), `shared-hardware-vm.jsonl` (4.11) and
`cloned-mac.jsonl` (4.12) are named by **no VALUE test in the tree**: their only mention is the
per-stream context table this story adds (`fixture_connector.rs`, `committed_stream_contexts()`),
which states each stream's declared context and asserts nothing about its contents.
_(Annotated 2026-07-27 by this story's own code review. This paragraph read "named by **no test in
the tree** — verified by `grep -rn "<name>.jsonl" --include=*.rs crates xtask`, which returns
nothing for all four". **The commit that implemented this story falsified its own check**: that grep
now returns four hits, all in the new table. The conclusion held; the named check did not, and a
cause needs a check that still holds.)_ They have
no byte-pin test, so AC1 has nothing to strengthen there, and their `obs_id` ↔ line binding is
weaker than dhcp-churn's was: `read_traps`'s cross-check only asserts that a trap's `obs_id`s EXIST
in the stream, so swapping two ids inside one of those four files inverts what its traps judge and
nothing reds. AC2 and AC3 give them admissibility and byte-shape coverage — **not** value pins.
Register it (AC6); do not fix it here.

**It is already scheduled: story 5.2b**, inserted on 2026-07-26 in Epic 5's debt block (after 5.2,
ahead of the L1 join at 5.5). Two consequences for THIS story: the `assert_obs_ids` helper AC1
introduces is what 5.2b builds on, so give it a shape that four more families can call without
change; and do not pre-empt 5.2b by adding value pins here — its ACs are written and its
prove-to-red budget is its own.
_(⚠️ **The first consequence was overruled on 2026-07-27**, by Guy, on this story's code review. The
helper as first shipped asserted no length, so an empty or truncated slice passed it — the vacuity
this story exists to close, re-introduced in the helper that closes it, and worse for 5.2b's four
families because they have no sibling length assertion to inherit. The shape therefore CHANGED: it
takes `expected_len` and asserts the count itself. 5.2b's call sites pass their length; that is one
argument, not a redesign.)_

### What this touches, and what it must not break

- **`crates/opencmdb-bin/src/fixtures.rs`** (UPDATE) — 2795 lines, of which ~698 are CODE (the
  `#[cfg(test)] mod tests` opens at line 699).
  *Today:* the corpus reader (`read_records:483`, `read_jsonl:640`, `read_traps:658`), the error
  taxonomy, and the corpus's inline test module holding `expected():724`, the privacy walk
  (`assert_text_is_synthetic:848`), the corpus validity walks, `walk_replay_streams:1389`, and every
  byte-pin. *This story changes:* one derive
  on the private `ControlRecord:122`; the hoist of `walk_replay_streams` to `#[cfg(test)]
  pub(crate)`; a new `assert_obs_ids` helper; a new round-trip walk; two byte-pins gain their pins;
  four gain a call to the helper. *Must be preserved:* `expected()` as the second independent oracle;
  `re_serializing_reproduces_the_committed_bytes` as the only value-pinning round-trip; the privacy
  walk's exhaustive `match` with no `_` arm (a new `Record` or `Fact` variant must still break it);
  `walk_replay_streams`'s symlink refusal, `README.md` exemption at any depth, and non-swallowed
  read errors (*"walks that quietly see less"* was the recurring defect of 4.1/4.3).
- **`crates/opencmdb-bin/src/fixture_connector.rs`** (UPDATE) — 1475 lines, of which ~337 are CODE
  (its test module opens at line 338).
  *Today:* `load:106` (reads the file, then `from_records`) and `from_records:153` (every
  admissibility invariant, positional capability containment). *This story changes:* the test module
  only — a per-stream table and one walking test. *Must be preserved:* the three existing context
  helper groups and every test that uses them; `load` and `from_records` are NOT modified.
- **`_bmad-output/implementation-artifacts/deferred-work.md`** (UPDATE) — append-only discipline.
- **`_bmad-output/implementation-artifacts/sprint-status.yaml`** (UPDATE) — the live source of truth
  (CLAUDE.md). It is `ready-for-dev` today and must reach `done` in this story's File List, with a
  comment saying what moved.
  _(⚠️ **Superseded 2026-07-27, on this story's code review.** `done` was the wrong target and the
  implementation was right to refuse it (Completion Notes, deviation 1): this project's flow is
  `dev-story → code-review → merge`, so `review` is what a dev-story may claim and `done` is the
  merge's business. The document had two answers to "what was required"; this is the one that
  stands. The instruction is left readable rather than rewritten.)_
- **Nothing under `fixtures/`.** No bytes, no `MANIFEST.toml`.

### House rules that bind this story

- **Prove-to-red is not optional** (story 1.3): a guard is observed failing before it passes, and
  the mutation is recorded. AC4 names one per AC. *"Where possible" excuses the genuinely untestable
  — it does not excuse a new guard shipping without a test that reds when it is removed.*
- **Name the test behind every claim.** Four consecutive completion records over-claimed and every
  review caught it. In this story the temptation is exact and named: writing *"connector-level
  admissibility now holds corpus-wide"* when what holds is *"every committed stream loads with its
  declared context; fact-kind coverage is proven only where a capability record narrows it."* Write
  the weaker true sentence.
- **A doc comment must be TRUE.** Three reviews caught docs asserting behaviour the code did not
  have. The new test docs make claims about what the walk proves — hold them to the code.
- **DRY, with deliberate redundancy protected.** Six copies of the `obs_id` loop is accidental
  duplication → extract (AC1). `expected()` vs the committed bytes, and the minimal round-trip vs
  the corpus-wide one, are deliberate second oracles → keep both, and label them (AC3).
- **Document every public item.** Nothing new becomes `pub` here; `walk_replay_streams` becomes
  `pub(crate)` under `#[cfg(test)]` and keeps its doc.
- **File-size gate:** ≤2000 CODE lines, tests excluded, counted to the first `#[cfg(test)]`.
  `fixtures.rs` counts **698** today; the hoist raises it by the hoisted doc comment's length. Far
  under the ceiling either way — and it is a rise, not a reduction.
- **Dependency frontier (D47):** all work here is in `opencmdb-bin`. `opencmdb-core` is untouched;
  do not reach for `anyhow`/`axum`/`sqlx`/`askama` anywhere near it.
- **`DATABASE_URL` is usually unset locally** and the MariaDB-backed tests `return` early either
  way — a green suite says nothing about the database. Irrelevant to this story's guards, but do not
  cite a green suite as evidence of anything it did not run.
- **Known local flakiness (issue #38):** the suite has shown unexplained non-determinism (8 failures
  across 5 runs, identical sha256, clean `git status`, then 15+ green — stories 4.15/4.17). CI on a
  clean checkout has never reproduced it and **the "Synology Drive" explanation is REFUTED by
  measurement — do not re-adopt it**. If a corpus test reds unexpectedly, re-run and check
  `git status` before diagnosing; if it persists, that is data for #38, not a cause you may write
  down. *A cause needs a check, not a plausible story.*

### Testing standards

Tests live inline in the trailing `#[cfg(test)] mod tests` (D56b, one per file). Test names are
sentences (`the_corpus_carries_no_real_network_data`). Assertion messages name the offending FILE —
with a corpus walk, *"a stream is inadmissible"* is not actionable unless it says which. Prefer
`#[should_panic(expected = "…")]` with a substring when pinning a panic, so a pass-for-the-wrong-panic
is impossible (4.14's idiom, reused by 4.17).

### Project Structure Notes

Paths follow the established layout with no variance: corpus at the workspace root in `fixtures/`
(D56 — a file under `tests/` reads as the property of the test); the fixture reader and
`FixtureConnector` in `crates/opencmdb-bin/src` beside `arp_ping.rs`, deliberately in the shipped
crate so *"zero mocks"* is a gate rather than a slogan; `FIXTURES_DIR` expressed exactly once
(`fixtures.rs:48`) — take the path from `fixtures_dir()`/`fixture_path()`, never re-write the string
(there is a test for this: `the_fixtures_path_is_expressed_once`, `fixtures.rs:1734`, and Task 3's
path conversion is where a dev is most likely to trip it).

One judgment call to be aware of: the connector walk (AC2) lives in `fixture_connector.rs` and the
round-trip walk (AC3) in `fixtures.rs`, each at its own layer, sharing ONE walker via the Task 2
hoist. The alternative — both tests in `fixtures.rs` — avoids the hoist but makes the lower layer's
tests depend on the higher one. The hoist is the recommendation; if you take the alternative, say
why in the completion record.

### References

- Story source: [Source: _bmad-output/planning-artifacts/epics.md#Story 5.1] (epics.md:1316-1334);
  Epic 5 framing and build order, epics.md:1306-1314; the inherited-debt-at-the-head decision,
  epics.md:1312.
- Deferred entries: [Source: _bmad-output/implementation-artifacts/deferred-work.md] — CLOSED by
  this story: story-4.10 review (line 336), story-4.12 review (line 353), story-4.15 review
  (line 398). **Annotated still-open:** story-4.13 review (lines 365-378). The closure idiom and the
  *"a register that loses an item"* lesson are at lines 14, 16, 17, 126, 131.
- Corpus doctrine (fixture-as-spec, fixture-as-connector, obs_id as the anchor, no clock):
  [Source: _bmad-output/planning-artifacts/architecture.md#D19] and the module docs of
  `fixtures.rs:1-30` and `fixture_connector.rs:1-68`.
- Descriptor travels with the batch, positional containment, *"the past would change status"*:
  [Source: architecture.md#D34 §1] via `fixture_connector.rs:153-260` (the capability-ordering
  checks specifically at `:187-206`, the fact-kind check at `:236-247`).
- Corpus lock, both directions: `fixtures/MANIFEST.toml:1-13`; gate in `xtask/src/main.rs`.
- File-size rule and its counting method: `xtask/src/main.rs:72-81` [Source: CLAUDE.md#Engineering
  conventions].
- Story flow, PR discipline, local gate, `architecture-views.md` staleness:
  [Source: docs/project-context.md] §"Working conventions" and lines 127-130.
- The two previous stories' shape (byte-pin idiom, prove-to-red records):
  `_bmad-output/implementation-artifacts/4-17-trap-hostname-absent-or-empty.md`,
  `…/4-18-wire-format-traps.md`.

### Git intelligence

Last five commits: `8c81386` (Epic 5 decomposition, PR #40) · `b568815` (Epic 4 retrospective, #39)
· `b4ceca3` (Epic 4 report, #37) · `199ba79` (Epic 4 closure, #36) · `7b93c20` (Story 4.18, #35).
`fixtures.rs` was last touched by 4.18, 4.17, 4.16, 4.15, 4.14 — each adding one byte-pin test and,
from 4.15 onward, its `obs_id` pins. **This story is the back-fill of the two families that predate
that rule.** Every one of those commits went through a branch → PR → green CI → squash merge; hold
the same.

### Toolchain

No new dependency, no version to research. Rust 1.96+, edition 2024; `serde`/`serde_json` already in
`opencmdb-bin`'s tree and already used by both files. Pin nothing new — and never invent a version;
`Cargo.lock` is committed and every build is `--locked`.

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context) — `claude-opus-5[1m]`, via `bmad-dev-story`, 2026-07-27.

### Debug Log References

**TWELVE prove-to-red observations: six for AC4, six more for the review-fix pass.** Every mutation
was reverted; `git status` under `fixtures/` is empty and `MANIFEST.toml` was never re-hashed.

> **Every line number and message below was RE-OBSERVED on the final tree** (after `cargo fmt`, on
> 2026-07-28), not carried over from the run that first took the mutation. The review found that two
> citations pointed at code that was not in the tree — taken at intermediate states and never
> refreshed — so the whole log was re-run rather than patched. Mutations 2, 3 and 4 quote a message
> and no line, which is what the panic prints; their messages are unchanged in the code.

1. **AC1 — swap two `obs_id`s between lines 1 and 2 of `dhcp-churn.jsonl`.**
   `cargo test -p opencmdb-bin the_dhcp_churn_stream` →
   ```
   panicked at crates/opencmdb-bin/src/fixtures.rs:1993:13:
   assertion `left == right` failed: observation 0 carries its authored obs_id
     left: "adadadad-0000-4000-8000-000000000002"
    right: "adadadad-0000-4000-8000-000000000001"
   ```
   Restored with `git checkout`; the test re-ran green. _(Recorded as `:1803:13` and "line 0" until
   2026-07-28: the line was wrong from the start — `:1803` is an unrelated trap literal — and the
   message then changed with the helper's rewording. Both re-measured.)_

2. **AC2 (a) — point the `dhcp-churn` table entry's `ConnectorId` at
   `30303030-3030-4030-8030-303030303030`.** Test-side only; the corpus was not touched. →
   ```
   scenario/replay/dhcp-churn.jsonl: inadmissible to the connector with its declared context:
   scenario/replay/dhcp-churn.jsonl: observation adadadad-0000-4000-8000-000000000001 is
   attributed to connector 33333333-3333-4333-8333-333333333333, but this replay is connector
   30303030-3030-4030-8030-303030303030 — one stream is one connector, and emitting another's
   observations would fabricate provenance
   ```
   `ForeignConnectorId`, naming the stream AND the offending `obs_id`, as AC4 requires.

3. **AC2 (b) — delete the `cloned-mac.jsonl` table entry** (the orphan direction: a walked stream
   with no entry) →
   ```
   scenario/replay/cloned-mac.jsonl: committed under scenario/replay/ but absent from this test's
   context table — a new stream must state its connector_id, scope and capabilities there
   ```

4. **AC2 (c), not named by AC4 but its own guard — add a table entry
   `scenario/replay/no-such-stream.jsonl`** (an entry naming no file) →
   ```
   scenario/replay/no-such-stream.jsonl: named by the context table but not found under
   scenario/replay/
   ```
   Recorded because the house rule is that a new guard does not ship without a test that reds when
   it is removed, and AC2's "both directions" is two guards, not one.

5. **AC3 (a) — insert one space after the first colon on line 1 of `dhcp-churn.jsonl`** (NOT
   `minimal.jsonl`, per AC4: an edit there also reds two older tests, so the new guard would be
   observed for the wrong reason) →
   ```
   panicked at crates/opencmdb-bin/src/fixtures.rs:994:17:
   assertion `left == right` failed: …/fixtures/scenario/replay/dhcp-churn.jsonl:1:
   re-serializing does not reproduce the committed bytes
     left: "{\"obs_id\":\"adadadad-…\",…}"
    right: "{\"obs_id\": \"adadadad-…\",…}"
   ```
   The file and its 1-indexed line are named, and both strings are printed. _(Recorded as `:941:17`
   until 2026-07-28 — the assertion's ARGUMENT line, not the line the panic prints.)_

6. **AC3 (b) — the same edit on the CONTROL line (line 3) of `capability-downgrade.jsonl`** →
   ```
   panicked at crates/opencmdb-bin/src/fixtures.rs:994:17:
   …/fixtures/scenario/replay/capability-downgrade.jsonl:3: re-serializing does not reproduce the
   committed bytes
     left: "{\"record\":\"capability\",\"as_of\":\"2026-03-01T00:00:07Z\",\"kinds\":[…]}"
    right: "{\"record\": \"capability\",…}"
   ```
   This is the observation that the control half is genuinely covered: a round-trip silently
   skipping control records would have passed mutation 5 and failed here.

**The review-fix pass (2026-07-28) — six more.** Every guard the code review added, or found
shipping unproven, is observed here. They are numbered on from AC4's six because the code comments
cite them by number.

7. **The helper's `expected_len` count guard** — hand
   `the_dhcp_churn_stream_moves_the_address_only_through_observed_at` a two-observation slice of the
   three-observation stream (`&observations[..2]`). Test-side only. →
   ```
   panicked at crates/opencmdb-bin/src/fixtures.rs:1987:9:
   assertion `left == right` failed: the stream must carry exactly 3 observations
     left: 2
    right: 3
   ```
   This is the guard the review found MISSING: before it, `assert_obs_ids(&[], "adadadad")` was
   green, so an empty or truncated slice passed the very check the story exists to make
   non-vacuous.

8. **The newline-termination guard** — `truncate -s -1 fixtures/scenario/replay/dhcp-churn.jsonl` →
   ```
   panicked at crates/opencmdb-bin/src/fixtures.rs:962:13:
   …/fixtures/scenario/replay/dhcp-churn.jsonl: a committed stream must be newline-terminated
   ```

9. **The CRLF guard** — convert every `\n` in `dhcp-churn.jsonl` to `\r\n` →
   ```
   panicked at crates/opencmdb-bin/src/fixtures.rs:967:13:
   …/fixtures/scenario/replay/dhcp-churn.jsonl: a committed stream must use LF endings, never CR or CRLF
   ```
   8 and 9 both round-tripped **GREEN** before this pass: the comparison runs on `str::lines()`,
   which strips a trailing `\r` along with the `\n`. `git checkout` after each, no `MANIFEST.toml`
   re-hash — and the threat model is a deliberate re-authoring, which refreshes the manifest by
   definition, so the sha256 lock was never the backstop here.

10. **`checked == table.len()`, which shipped unproven** — duplicate the `dhcp-churn` entry in the
    context table (14 entries over 13 files: every file matched, every entry walked, so neither
    orphan direction sees it) →
    ```
    panicked at crates/opencmdb-bin/src/fixture_connector.rs:1632:9:
    assertion `left == right` failed: the context table must have exactly one entry per committed stream
      left: 13
     right: 14
    ```

11. **`checked > 0`, hoisted into `walk_replay_streams` and previously five verbatim copies** —
    suppress the `checked += 1;` increment inside the walker →
    ```
    panicked at crates/opencmdb-bin/src/fixtures.rs:788:5:
    no replay stream found under scenario/replay/ — every caller of this walk would otherwise pass
    by proving nothing
    ```
    The message now names the walk. The five copies it replaced all printed the same sentence, so a
    reader could not tell WHICH walk had found nothing.

12. **The dot-entry skip — observed on BOTH sides**, because a skip has no assertion of its own and
    its "red" is the counterfactual. Write `probe.txt` under
    `fixtures/scenario/replay/.claude/.cc-writes/` (git-ignored, so `git status` stays clean), then:
    - **with** the skip → `test result: ok. 1 passed`;
    - **without** it (`continue;` commented out) →
      ```
      panicked at crates/opencmdb-bin/src/fixtures.rs:779:13:
      …/fixtures/scenario/replay/.claude/.cc-writes/probe.txt: only .jsonl replay streams and
      README.md belong under scenario/replay/
      ```
    So the review's finding is confirmed by measurement rather than by argument: without the skip,
    the first file any tool writes under that scratch directory makes the suite accuse the CORPUS of
    a defect it does not have. The probe was deleted afterwards.

**One assertion is defence-in-depth and CANNOT be proven red, and says so in its own comment:**
`walked.insert(relative.clone())` (`fixture_connector.rs`). A stack walk that panics on symlinks
cannot yield one path twice. The `insert` is load-bearing — the orphan-entry loop reads `walked` —
but its assertion is dead, and the house rule is that a guard which cannot have a mutation states
that instead of implying it had one.

**Gate output (AC5), re-run in full after the review-fix pass on 2026-07-28.**
`cargo fmt --all` clean · `cargo clippy --workspace --all-targets -- -D warnings` clean ·
`cargo test --workspace` → **121 (bin) + 86 (core) + 42 (xtask) = 249 passed, 0 failed**
(247 before this story; +2 are the two new walks — the review-fix pass added guards INSIDE existing
tests, so the count is unchanged by it) · `cargo xtask ci` →
```
✅ frontier       domain graph clean; xtask depended on by nobody
✅ ddl-collation  every text column carries an explicit binary collation
✅ vocabulary     co-presence green across docs; code clean
✅ fixtures       25 fixture(s) match their recorded sha256 (0 generated, 25 hand-authored)
✅ file-size      20 file(s) under 2000 code lines (largest: 884)
ℹ  views-hash     STALE — regenerate at next milestone
✅ all gates green
```

⚠️ **The state the review handed back did not COMPILE**, and that is recorded rather than quietly
fixed. The two byte-level guards (mutations 8 and 9) had been inserted into
`every_replay_stream_re_serializes_to_its_committed_bytes` in place of its
`let records = read_records(path)…` binding, so the crate failed with two `E0425 cannot find value
records` before this pass restored it. Nothing had been run against that state — which is why the
gate is re-run here in full rather than cited from the implementation pass.
The `views-hash STALE` line is expected and exits 0. **`architecture-views.md` was NOT regenerated**
— it is a milestone task, not a story task.

⚠️ **What the green suite does not say.** `DATABASE_URL` was unset, so the MariaDB-backed tests
returned early. Nothing in this story touches the database, but the suite is not evidence about it.

### Completion Notes List

**What was implemented, in the weaker true sentence.**

- **AC1.** `assert_obs_ids(observations, prefix, expected_len)` lives in `fixtures.rs`'s test module.
  _(The signature gained `expected_len` on 2026-07-28, on Guy's call at the code review: as first
  shipped the helper asserted no count, so an empty or truncated slice passed it — the vacuity this
  story exists to close, re-introduced in the helper that closes it. Its doc also said "line `n`"
  when the slice comes from `read_jsonl`, which DROPS control records: in `capability-downgrade.jsonl`
  the capability record sits on file line 3, so `obs_id …0003` is on file LINE 4. It now says
  OBSERVATION order and names that counter-example.)_ It is called
  from six sites: the two byte-pins that read purely by index and now assert their ids
  (`dhcp-churn` → `adadadad`, `vrrp-virtual-mac` → `aeaeaeae`), and the four that already had the
  loop inline (`hostname-collision` `afafafaf`, `docker-veth` `babababa`, `hostname-absence`
  `bcbcbcbc`, the wire spec `bdbdbdbd`). **No call site was left inline.** The DRY argument is in the
  helper's doc: all four existing loops already COMPUTED their ids with `format!`, so what was
  removed is mechanical duplication and not a second oracle — and the helper is explicit that it
  encodes two corpus CONVENTIONS (the fixed `-0000-4000-8000-` middle segment, sequential numbering
  from 1, both true of all 13 streams today) and that a future stream numbered otherwise gets its own
  assertion rather than being re-authored to fit.

- **AC2.** `every_committed_replay_stream_is_admissible_to_the_connector` in
  `fixture_connector.rs`. All 13 committed streams load through `FixtureConnector::load` with a
  hand-authored `StreamContext` table — never derived from the observations. Both orphan directions
  are checked and both were proven to red. **The honest limit is written into the test's own doc and
  is repeated here:** the walk shows every committed stream is ADMISSIBLE; it never observes
  `UndeclaredFactKind` firing. The fact-kind check is non-vacuous only on `partial-then-failed.jsonl`
  and on both sides of `capability-downgrade.jsonl`'s record; it is vacuous on the eleven `corpus_*`
  streams because `corpus_caps()` declares all seven kinds. This is **not** "connector-level
  admissibility now holds corpus-wide with fact-kind coverage" — the temptation the story named
  explicitly.

- **AC3.** `every_replay_stream_re_serializes_to_its_committed_bytes` in `fixtures.rs`, plus
  `render_record`. `ControlRecord` gained `Serialize`; **the `#[cfg(test)]` mirror-struct fallback was
  not needed**, confirming the story's measurement. What it pins is the byte-SHAPE, starting from the
  file — never the authored values. `re_serializing_reproduces_the_committed_bytes` survives
  unchanged and carries the "not a duplicate, do not collapse" comment.

- **AC4.** **Twelve** mutations observed and recorded above. Six for AC4 itself: it asked for four,
  and exactly **ONE** was surplus — the entry-without-a-file direction. _(This said "two extra …
  the entry-without-a-file direction, and the control-line half of AC3", and elsewhere said "five".
  Both wrong, and in opposite directions: AC4's third bullet REQUIRES the control-line mutation on
  `capability-downgrade.jsonl:3`, so it was never surplus; and six were enumerated all along.
  Corrected 2026-07-28.)_ Six more come from the review-fix pass — the `expected_len` count guard,
  the newline and CRLF guards, `checked == table.len()`, `checked > 0`, and the two-sided
  observation of the dot-entry skip. One assertion is named as unfalsifiable defence-in-depth
  instead of being credited with a red it cannot have.

- **AC5.** Green, and `git status` under `fixtures/` is empty. Re-run in full on 2026-07-28, after
  the review-fix pass restored a tree that did not compile.

- **AC6.** Three register entries closed in place by appending; the fourth annotated STILL OPEN and
  not struck; a new `## Deferred from: story-5.1` section names the four unpinned families and their
  owner, story 5.2b. _(That new section stated a check the same commit falsified: it cited
  `grep -rn "<name>.jsonl" --include=*.rs crates xtask` "returns nothing for all four" while the
  context table AC2 adds names all four. The conclusion held — no test asserts their values — so the
  CHECK was restated, not the conclusion re-asserted, in all three places that carried it:
  `deferred-work.md`, this story's Dev Notes, and `epics.md`'s Given clause for story 5.2b, where it
  was the next story's premise. Corrected 2026-07-28.)_

**The review-fix pass (2026-07-28) — what changed, and one thing that is not a fix.**

All **12 `[Review][Patch]` items are resolved**; the 5 `[Review][Defer]` items stay deferred and are
recorded in `deferred-work.md` under `## Deferred from: code review of story-5.1`. What the pass
actually changed, beyond the annotations already noted per-AC above:

- **It restored a tree that did not compile.** See the AC5 note. Nothing had been measured against
  that state, so the whole gate and the whole prove-to-red log were re-run rather than cited.
- **`every_committed_stream_re_serializes_to_its_committed_bytes` was RENAMED** to
  `every_replay_stream_re_serializes_to_its_committed_bytes`. Its name claimed every committed
  stream while `scenario/wire/unifi-clients.expected.jsonl` is a committed `.jsonl` deliberately
  outside the walk. Fixing the doc alone would have left the name — which is what a reader greps —
  making the wider claim.
- **The hoisted walker's doc lost its caller inventory.** It enumerated five callers; nothing checks
  such a list, and story 5.2b would have falsified it silently. The load-bearing sentence — two
  claims at two layers must walk the same tree — needs no inventory to be true.
- **Two deviations from the story's own text are declared, not absorbed:** the `assert_obs_ids`
  shape changed (Dev Notes had promised 5.2b a shape "four more families can call without change"),
  and the Dev Notes line requiring `sprint-status.yaml` to reach `done` is annotated as superseded.
  Both annotations sit in the Dev Notes rather than rewriting it.
- **`_bmad-output/planning-artifacts/epics.md` was edited** — outside a story's usual reach, and
  deliberate: two of the corrections land in story 5.2b's spec, one of them in its `Given` clause.
  A next story whose premise is false is the defect this project keeps catching.

**What is NOT recorded, on purpose.** The `.claude/.cc-writes` directories under `fixtures/` are
**not** a cause for issue #38. They were created 2026-07-26, a day after the stories that flaked,
they are empty, and they are git-ignored. Mutation 12 measures what they WOULD have done to the walk;
it measures nothing about #38. *A cause needs a check, not a plausible story.*

**Two deviations from the task list, both stated rather than silent.**

1. **`sprint-status.yaml` says `review`, not `done`** (Task 6 asked for `done`). `code-review` has not
   run, and this project's own flow is `dev-story → code-review → merge`. `done` before review would
   be a claim the work has not earned. Guy's call whether to overrule.
2. **The story was NOT pushed and no PR was opened**; the last Task 6 checkbox is left unchecked. The
   commit sits on `story/5-1-corpus-byte-pins`. Pushing before `code-review` inverts the flow, and a
   push is outward-facing on a public repo.

**One measurement worth carrying forward.** `fixtures.rs` went from **698 to 720** code lines — a
RISE of 22, caused by the hoisted doc comment sitting above its `#[cfg(test)]` attribute. Predicted by
the story; recorded here so nobody records it as a reduction. The review-fix pass took it to **728**
(first `#[cfg(test)]` now at line 729), the extra 8 being the dot-entry skip and the walker's own
non-emptiness assertion, both of which live in the hoisted function. Ceiling is 2000 and the largest
file in the tree is 884.

**Nothing found that contradicts the corpus.** Task 4's round-trip is the real Rust check the story's
Dev Notes said had not yet been run against the committed bytes. It passes on all 13 streams and both
control lines with no re-authoring — so the Dev Notes' out-of-Rust canonicalization measurement is
confirmed by the serializer itself.

### File List

- `crates/opencmdb-bin/src/fixtures.rs` (MODIFIED) — `Serialize` on `ControlRecord`;
  `walk_replay_streams` hoisted to `#[cfg(test)] pub(crate)` module scope with an extended doc;
  new test helpers `assert_obs_ids` and `render_record`; new test
  `every_replay_stream_re_serializes_to_its_committed_bytes`; two byte-pins gained their `obs_id`
  pins; four inline loops folded into the helper;
  `re_serializing_reproduces_the_committed_bytes` gained its "not a duplicate" doc.
  **Review-fix pass:** the missing `let records = read_records(path)` restored (the crate did not
  compile); `assert_obs_ids` gained `expected_len` and its doc corrected from "line" to
  "observation"; the round-trip gained a newline-termination and a CRLF refusal and was RENAMED to
  `every_replay_stream_re_serializes_to_its_committed_bytes`; the walker gained a dot-entry skip and
  its own non-emptiness assertion (five verbatim copies removed from the call sites) and lost its
  caller inventory; `ControlRecord`'s doc corrected — a missing `Serialize` would not compile, it
  would not "silently skip".
- `crates/opencmdb-bin/src/fixture_connector.rs` (MODIFIED) — test module only: `StreamContext`,
  `committed_stream_contexts()`, new test
  `every_committed_replay_stream_is_admissible_to_the_connector`; `corpus_id()` and `corpus_caps()`
  doc comments corrected. `load` and `from_records` are UNCHANGED.
  **Review-fix pass:** the duplicate-entry and unfalsifiable-assertion comments now state which
  guard was observed red and which one cannot be.
- `_bmad-output/implementation-artifacts/deferred-work.md` (MODIFIED) — three entries closed in
  place, one annotated still-open, one new `## Deferred from: story-5.1` section, one new
  `## Deferred from: code review of story-5.1` section (5 items). Append-only discipline held.
  **Review-fix pass:** the story-4.10 entry records the newline/CRLF strengthening; the story-4.12
  entry's "All three guards proven to red" corrected to the measured tally; the story-5.1 entry's
  falsified grep check restated.
- `_bmad-output/planning-artifacts/epics.md` (MODIFIED, review-fix pass only) — story 5.2b: the
  falsified grep in its `Given` clause restated, the randomized-mac AC's fact-count constraint
  corrected from "both lines" to all three (measured: 3 observations, 2 facts each — `Mac` +
  `IpV4`), and the title widened to name dhcp-churn's value pins, which its fifth `Given` already
  required.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (MODIFIED) — `5.1` → `review`, with the
  delivered/moved comment. **Review-fix pass:** the comment corrected (mutation count) and extended
  with what the review changed.
- `_bmad-output/implementation-artifacts/5-1-corpus-pins-obs-id-binding.md` (MODIFIED) — this record.
- `docs/project-context.md` and `CLAUDE.md` (MODIFIED, at push time) — the docs-current-before-push
  rule: both still said Epic 5 was backlog and "next", and `project-context.md` carried master's
  test counts. Epic 5 is in-progress and this branch runs 249. Minimal factual alignment, no other
  claim touched.
- **Nothing under `fixtures/`.** No bytes, no `MANIFEST.toml`. Mutations 8, 9 and 12 touched
  `fixtures/` transiently and every one was reverted; `git status` under `fixtures/` is empty.

## Change Log

| Date | Change |
|---|---|
| 2026-07-28 | Code review findings addressed — **12 of 12 `[Review][Patch]` items resolved**, 5 `[Review][Defer]` items registered. Restored a tree that did not compile (`records` binding lost when the byte-level guards were inserted). `assert_obs_ids` gained `expected_len`; the round-trip gained newline-termination and CRLF guards and was renamed to `every_replay_stream_re_serializes_to_its_committed_bytes`; the walker gained a dot-entry skip and its own non-emptiness assertion. **Six more mutations proven to red (12 total), and the whole prove-to-red log re-observed on the final tree** — two citations had pointed at code that was not in it. The falsified `grep` check corrected in all three documents that carried it, including `epics.md`'s `Given` clause for story 5.2b. Gate re-run in full: clippy clean, 249 tests, all `xtask ci` gates green, corpus bytes unchanged. |
| 2026-07-27 | Story implemented. `obs_id` ↔ line binding pinned on `dhcp-churn` and `vrrp-virtual-mac` via one shared helper (six call sites); all 13 committed streams walked through `FixtureConnector::load` with a hand-authored context table checked both directions; all 13 round-tripped to their committed bytes line by line, control records included. Six mutations proven to red. 247 → 249 tests, all gates green, corpus bytes unchanged. Status → `review`. |
