# Story 4.4: `FixtureConnector` replays JSONL through the real trait

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the engine's test infrastructure,
I want a connector that replays a committed JSONL fixture and passes the connector contract test unchanged,
so that *"the fixture IS a connector"* is a fact the compiler checks rather than a slogan.

## Context

Stories 4.1–4.3 froze the corpus: a JSONL stream of `Observation`s, a TOML labelling format, and a lock that reds in both directions. Everything so far is **data**. This story is where the data becomes a **source**: `FixtureConnector` implements `opencmdb_core::connector::Connector` and replays a committed stream through the same trait `ArpPingConnector` implements, driven through the same `run_connector_contract` harness — no special-casing, no mock, no network.

The payoff is stated in D19 and it is not decoration: *"The punchline: `FixtureConnector` implements `Connector` by replaying JSONL. THE FIXTURE IS A CONNECTOR. Zero mocks. Zero network. If the trait does not allow this, the trait is wrong."* Everything downstream in this epic (the metrics harness 4.6, the trap runner 4.7, every trap family 4.9+) drives the engine through this type. If it lies about anything — order, identity, what a poll covered — every trap built on it inherits the lie.

**Explicitly OUT of scope** (do not build these here):
- **Scripting a poll's OUTCOME** — errors, partial-then-failed, mid-scan capability loss. That is story **4.5**, and it is the story that will need a per-poll record in the file (see AC8). This story replays a CLEAN poll only.
- **The metrics harness** (4.6), **the trap runner** (4.7), any **trap family** (4.9+), the wire-format traps (4.18, executable only in Epic 11).
- **Wiring the connector into `main.rs` / the scheduler.** It ships in the crate, unused, exactly as `arp_ping.rs` did when it landed.
- **Any new corpus artefact.** `minimal.jsonl` and `example-traps.jsonl` are enough. If you add one, the lock and the orphan gate both apply — see Dev Notes.

## Acceptance Criteria

1. **`FixtureConnector` implements `Connector` and lives in the shipped crate** — `crates/opencmdb-bin/src/`, beside `arp_ping.rs`, declared in `main.rs`. Not under `tests/`: there it would not face the same compilation gates as its siblings and *"zero mocks" would become a slogan* (D56). Its tests live in `bin` too, because that is the lowest crate that can see both the trait and `crate::fixtures` (D56b).

2. **A poll emits exactly the fixture's observations, in file order** — the values `read_jsonl` returns for that path, same sequence, nothing added, nothing reordered, nothing dropped. Replay order is part of what a trap asserts.

3. **A poll performs ZERO I/O.** The file is read once, at construction. `poll` takes no path, opens no file, reads no clock. This is a **structural** criterion, verified by reading the code, not by a test — say so in the completion record rather than claiming a test that does not exist. A poll that re-reads the disk is not a replay; it is a race against whatever else is writing.

4. **Polling the same instance twice emits the same stream twice.** A replay is idempotent: the second poll is not empty and not a continuation. The trap runner will poll more than once, and a later refactor to a draining iterator must break a test, not a trap. **`run_connector_contract` does NOT cover this** — it builds a fresh connector for each of its cases — so AC4 needs its own test.

5. **`FixtureConnector` passes `run_connector_contract` unchanged**, with no special-casing in the harness, driven over **two** inputs: (a) the committed `scenario/replay/minimal.jsonl`, and (b) an empty stream. Two drives are required because the harness runs the same two blocks whatever you give it — *empty* and *sparse* are properties of the `expected` the caller supplies, not cases the harness generates. The harness lives in `opencmdb-core` behind the `test-support` feature; enabling it must NOT change what `cargo build` ships.
   **The empty-stream drive passes a NON-EMPTY `scopes_covered`.** AC7 is vacuous over an empty stream, so any value would pass and the test would silently fix whichever the dev picked. *Covered and empty* — a scope claimed, nothing seen — is the case worth exercising, because `(connector, scope)` is D34 §3's blindness unit and a poll claiming to have covered nothing updates no liveness at all.

5c. **The `PollSummary` round-trips.** A test asserts that the returned `capabilities` and `scopes_covered` are exactly what the constructor was given. `run_connector_contract` ignores the summary entirely — it only `expect()`s `Ok` — so without this test the design decision AC8 turns on ships with no coverage at all.

5b. **The pre-cancelled poll case is real, not vacuous.** The contract's cancellation clause asserts `expected.starts_with(&sink.observations)`, which an empty sink satisfies and which a connector ignoring the token also satisfies. `poll` must check `cancel` BETWEEN emits (never mid-observation) and return `ConnectorError::Cancelled` (D34 §2 — what was already emitted is TRUE and survives).
   **`poll` contains no `.await`**, so no external task can interleave with it and cancel "midway": drive the mid-stream case from a **test-local `ObservationSink` that holds the `CancellationToken` and cancels it inside `emit`** after the first observation. `ObservationSink` is a plain sync trait, so this is a dozen lines. Assert exactly one observation in that sink and `Err(Cancelled)`. **Do NOT add a yield point to `poll`** to make an external cancel work — an await in a zero-I/O replay is a defect, not a fix. (`opencmdb-core` hit the same wall and wrote `CancelsMidway` for it.)

6. **The connector's identity and the stream's agree, or the load fails naming both.** `Connector::id()` returns the id the connector was constructed with; loading a stream containing an observation whose `connector_id` differs from it is an error naming the connector id, the foreign id, the `obs_id`, and the fixture. A connector emitting observations attributed to somebody else is a fabricated provenance, and the trap corpus is exactly where that must be impossible.
   **Consequence, decided deliberately: one stream = one connector.** A multi-source trap (two sources disagreeing about one device) is two `FixtureConnector`s over two files, never one file carrying two ids. Record that in the module doc — it is a constraint the trap families inherit.

7. **A clean poll may not claim to cover less than it observed.** Every distinct `scope` in the loaded stream must be present in `scopes_covered`, or the load fails naming the uncovered scope. The reverse stays legitimate and must stay so: *covered and empty* is a meaningful answer, not an error.
   **Bounded to a clean replay, on purpose.** Story 4.5 introduces partial-then-failed polls, where a scope can be observed and legitimately NOT covered — the poll died before finishing it. Record in `deferred-work.md` that 4.5 must be free to relax this check or move it from load time onto the clean-poll path, and that doing so is not a regression.

7b. **A clean poll may not emit a fact kind it claims it cannot emit.** Every `Fact::kind()` in the loaded stream must be present in `capabilities.kinds`, or the load fails naming the kind, the `obs_id` and the origin. The reverse stays legitimate and must stay so — *capable and unseen* is the whole reason `capabilities` exists (D34 §1).
   **This is containment, not derivation**, and the distinction is the point: it never invents a capability from what was observed (which AC8 forbids), it only refuses a replay that contradicts itself. Without it a `FixtureConnector` can emit a `Hostname` fact while declaring itself blind to hostnames — self-contradictory in exactly the dimension the false-merge argument protects. `Fact::kind()` and `Capabilities::can_emit()` both already exist.

7c. **An in-memory stream is held to the same `obs_id` uniqueness as a file.** `read_jsonl` refuses a repeated `obs_id`, but `from_observations` takes a `Vec` that never passed through it — so every in-memory construction (this story's tests, 4.6's harness, 4.7's trap runner) would bypass the anchor the whole labelling format rests on. Check it on the shared path. The existing `DuplicateObservationId` variant is keyed on line numbers a `Vec` does not have, so this needs its own variant keyed on `origin` + `obs_id`.

8. **The recorded finding (D19's clause, in its mirror direction).** `Capabilities` and `scopes_covered` are supplied at construction, NOT read from the file, because the frozen format cannot express them — while D34 §1 argues they should travel with the batch (*"the fixture replays it for free — one JSONL line reproduces a mid-scan NET_RAW loss, zero mocks; with a separate getter the fixture would need state outside the JSONL"*). That IS the "state outside the JSONL" D34 refused. **Record it** — in the module doc AND in `deferred-work.md`:
   - `capabilities` → owned by **story 4.5**, whose AC is literally the mid-scan capability loss.
   - `scopes_covered` → 4.5 **by extension** (it is the story that puts a per-poll record in the file); epics.md assigns it to nobody, so say "not yet assigned" rather than inventing an owner.
   - The shape is *"one JSONL line"* (epics.md's words). Do not call it an envelope or design it here.
   **Do NOT work around it by deriving `capabilities` from the observations.** A capability derived from what was seen cannot express *"capable of hostnames, saw none"* — the single distinction `capabilities` exists for, and the one that stops a rule like *"no diverging uplink → same switch → merge"* from merging happily (D34 §1 / D19 false-merge prevention).
   **`Capabilities.as_of` is the sharpest instance, and it must be recorded too.** It is caller-supplied while every observation is dated by the file, so a replay can date its capability descriptor in a moment its own stream contradicts — and D34 §1's entire content is that the descriptor is *a dated fact, not a constant*, with 4.5's downgrade traps being diffs over exactly that field. The cheap guard available today is `capabilities.as_of >= max(observed_at)` on a clean replay. **Record the question; do not impose the guard here.** Inventing a validation policy under implementation pressure is what the 4.3 review called out; 4.5 puts the record in the file and can then date the descriptor from the file, which is the real fix.
   **And the epic's criterion stands in its own direction too:** if while implementing you find the TRAIT cannot express what the fixture needs, the trait is wrong, not the fixture — record it and raise it, do not bend the fixture.
   **One candidate is already visible, so flag it rather than let 4.5/4.6 rediscover it:** `PollSummary` is returned only on `Ok`, so a cancelled or partial poll carries no `capabilities` — while story 4.6 requires a `capability_snapshot` on *every* scored record (*"a verdict without its capability snapshot is unfalsifiable"*, D36). Do not change the trait here. Record it.

9. **The corpus path is still expressed once.** `the_fixtures_path_is_expressed_once` must still pass at exactly 2 occurrences. Reach the corpus through `crate::fixtures::fixture_path`; never write the `/../../fixtures` literal again.

10. **All gates green, locally, before done:** `cargo fmt --all` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo test --workspace` · `cargo xtask ci`.
    Note: `cargo xtask ci` already prints `ℹ views-hash STALE — regenerate at next milestone` and still exits green — that line is informational. **Do not regenerate `architecture-views.md` in this story.**

## Tasks / Subtasks

- [x] **Task 1 — the module** (AC: 1, 3, 6, 8, 9)
  - [x] Create `crates/opencmdb-bin/src/fixture_connector.rs`; add `mod fixture_connector;` to `main.rs` (alphabetical, beside `mod fixtures;`).
  - [x] `#![allow(dead_code)]` with the same rationale comment `arp_ping.rs` carries — it is wired into the running app later.
  - [x] **`#[derive(Debug)]` on `FixtureConnector`.** Task 3's load-invariant tests call `Result::expect_err`, which requires `T: Debug` on the `Ok` type — and the `Ok` type here is `Self`. `ArpPingConnector` derives nothing, so copying the shipped sibling gets this wrong and costs a compile cycle.
  - [x] Module doc: D19's punchline, the one-stream-one-connector consequence (AC6), and the AC8 findings.
  - [x] Construction reads the file ONCE via `crate::fixtures::{fixture_path, read_jsonl}`. Suggested surface:
    - `load(id, capabilities, scopes_covered, relative_path) -> Result<Self, FixtureError>` for the corpus;
    - `from_observations(id, capabilities, scopes_covered, origin: &str, Vec<Observation>) -> Result<Self, FixtureError>` for the in-memory cases, with `load` delegating to it so **all four** load invariants (AC6, AC7, AC7b, AC7c) are checked on ONE path.
    - **`origin` is not decoration, and it is a deliberate break with the enum's habit.** Every existing `FixtureError` variant except `OutsideCorpus` carries a `path: PathBuf` that its `Display` names first. The new variants may describe a stream that has no path at all, so they carry **`origin: String`, not `path: PathBuf`** — a fabricated `PathBuf::from("<in-memory>")` would be a lie in the type to preserve a habit. Say so in the enum's doc comment. `load` passes the corpus-relative path; an in-memory caller passes a label.
    - Names and signatures are yours; the ACs are what binds. If you deviate, say so in the record.

- [x] **Task 2 — `impl Connector`** (AC: 2, 3, 5b)
  - [x] `id()` returns the constructed id (`ConnectorId` is `Copy`). `poll` ignores `now`: observations are dated by the FILE, and the engine never touches the clock (D19).
  - [x] Emit each observation into the sink in order, checking `cancel.is_cancelled()` BEFORE each emit and returning `ConnectorError::Cancelled`. Copy the cancellation-point shape from `ScriptedConnector::poll` — it is already the reviewed one.
  - [x] Return `PollSummary { capabilities: self.capabilities.clone(), scopes_covered: self.scopes_covered.clone() }`. `Capabilities` is `Clone` but not `Copy`; `Scope` is `Copy`.

- [x] **Task 3 — the four load-time invariants** (AC: 6, 7, 7b, 7c)
  - [x] Add the new `FixtureError` variants in `fixtures.rs` beside the existing ones. `FixtureError` is named nowhere outside `fixtures.rs` (verified by grep), so only its `Display` and `source()` matches need updating — both are exhaustive, so the compiler will point at them. **Widen the enum's own doc comment**: it says *"Why a fixture could not be read"*, and these variants are about what a fixture may not CLAIM, from a stream that may have no file at all.
  - [x] Suggested shapes — take them or name your own, but say which in the record, because a review will assert on the variant name and on what the message names:
    `ForeignConnectorId { origin, expected, found, obs_id }` · `UncoveredScope { origin, l2_domain, vantage, obs_id }` · `UndeclaredFactKind { origin, kind, obs_id }` · `RepeatedObservationId { origin, obs_id }`.
  - [x] Foreign `connector_id` (AC6) → name the expected id, the found id, the `obs_id` and the origin. Name the **`obs_id`, not a line number**: the labelling format's rule is *"never by line number"* (4.2), and `read_jsonl` has already dropped blank lines, so any index you compute is not the editor's line.
  - [x] Uncovered `scope` (AC7) → name `scope.l2_domain` and `scope.vantage`. **`Scope` derives `Copy`, `Hash`, `Eq` but NOT `Ord`, and has no `Display`** — the module's usual `BTreeSet` will not compile here; use a `HashSet<Scope>` or a linear `contains`, and interpolate the two id fields, which do have `Display`.
  - [x] Undeclared fact kind (AC7b) → containment only: `observations.flat_map(Fact::kind) ⊆ capabilities.kinds`. `FactKind` is `Ord` (it lives in a `BTreeSet` on `Capabilities`), so the usual collections apply here.
  - [x] Repeated `obs_id` on the in-memory path (AC7c) → keyed on `origin` + `obs_id`, no line numbers. For a stream that came through `load`, `read_jsonl` has already refused it, so this check only ever fires for `from_observations` — that is the hole it exists to close, not a duplicate.
  - [x] All four proven to red.

- [x] **Task 4 — the contract and the behavioural tests** (AC: 4, 5, 5b, 5c)
  - [x] Add to `crates/opencmdb-bin/Cargo.toml`: `[dev-dependencies] opencmdb-core = { workspace = true, features = ["test-support"] }`. The feature exists and is documented for exactly this use.
  - [x] **Verify the shipped build is unchanged** rather than believing it: `cargo tree -p opencmdb-bin -e normal --locked` must show `opencmdb-core` with no features, `cargo build --locked` must still compile, and `cargo xtask ci` must stay green. Record what you ran.
  - [x] Drive `run_connector_contract` over the committed `minimal.jsonl`, with `expected` obtained from `read_jsonl` on the same path — connector and expectation must both come from the file, never from a Rust literal.
  - [x] The harness is `F: Fn() -> C` with `C: Connector` and nothing more, so the factory must **unwrap `load`'s `Result` itself** (`.expect("the committed fixture must load")`). It is `Fn`, not `FnOnce`: either clone anything a closure captures (`testing/mod.rs:298` shows that shape), or write the factory as a plain helper `fn`, which captures nothing. Both are fine.
  - [x] Drive it a second time over an empty stream, via `from_observations` with `vec![]` — and a NON-EMPTY `scopes_covered` (AC5). Do NOT add an empty `.jsonl` to the corpus for this.
  - [x] **The idempotency test (AC4):** one connector, polled twice into two sinks, two equal non-empty streams.
  - [x] **The summary round-trip test (AC5c):** assert the returned `PollSummary.capabilities` and `.scopes_covered` are exactly what the constructor was given. Nothing else in the test set looks at the summary.
  - [x] **The mid-stream cancellation test (AC5b):** the cancelling sink described in AC5b — one observation in the sink, `Cancelled` returned.

- [x] **Task 5 — record the findings, and run the gates** (AC: 7, 8, 10)
  - [x] Append to `_bmad-output/implementation-artifacts/deferred-work.md`, under `## Deferred from: story-4.4 (2026-07-21)` — a heading that omits "code review of" because these findings are raised BY the story, not by its review. Four entries: the capabilities/`scopes_covered` limit (AC8) · `Capabilities.as_of` unrelated to the file's `observed_at` (AC8) · `PollSummary` absent on the error path vs 4.6's required `capability_snapshot` (AC8) · AC7's clean-replay bound, which 4.5 may relax. **Append; do not rewrite or drop existing bullets** — the 4.3 review caught exactly that.
  - [x] Run the four gates. If you added or edited any file under `fixtures/`, add its `[[artefact]]` entry with sha256 to `fixtures/MANIFEST.toml`, or the orphan gate reds.

### Review Findings

_Code review 2026-07-22 — three parallel layers (Blind Hunter, Edge Case Hunter, Acceptance Auditor). All 14 ACs SATISFIED per the auditor's line-by-line map, and the AC5b mutation claim reproduced exactly. One real behavioural defect, found independently by two layers and then confirmed by measurement; the rest are false claims in comments and in the completion record, and test coverage thinner than the criteria it claims to prove._

- [x] [Review][Decision] **Should a cancellation arriving during the LAST emit report `Cancelled`?** — decided 2026-07-22 (Guy), option (a): leave it `Ok`, with the asymmetry written into `poll` as a deliberate choice. With the before-loop check added (patch 1), a pre-cancelled poll is `Cancelled` regardless of stream length. What remains: a token cancelled while the final observation is emitted returns `Ok`, because the loop then simply ends. Option (a) leave it — every observation was emitted, the poll did its whole job, and reporting `Cancelled` would discard a complete result; D34 says a cancelled poll KEEPS what it emitted, not that a completed one must claim cancellation. Option (b) add a post-loop check — the reviewer's argument is that one shutdown event then yields `Ok` or `Cancelled` purely by stream length. Recommendation: (a), stated in the code so the asymmetry is deliberate.

- [x] [Review][Patch] **A pre-cancelled poll over an EMPTY stream returns `Ok`, claiming coverage it never performed.** The `is_cancelled()` check lives only INSIDE `for observation in &self.observations`, so a zero-observation stream never reaches it. Measured: `Ok(PollSummary { scopes_covered: [Scope { … }] })` for a poll cancelled before it began. Under NFR7/D34 `Cancelled` must leave liveness UNCHANGED, and an `Ok` summary asserting `scopes_covered` actively refreshes it. `arp_ping.rs:127` checks the token before any work for exactly this reason; `run_connector_contract`'s cancellation block uses `if let Err(e)`, so it accepts `Ok` and cannot catch it — the empty-stream drive passes vacuously on that clause. Found independently by Blind Hunter and Edge Case Hunter [crates/opencmdb-bin/src/fixture_connector.rs:170]
- [x] [Review][Patch] **A mutant validating only `observations.first()` survives three of the four invariant tests.** `a_foreign_connector_id_is_refused`, `a_scope_the_poll_does_not_cover_is_refused` and `a_fact_kind_the_capabilities_deny_is_refused` each pass a one-element `Vec`, so nothing proves the loop validates past index 0 [crates/opencmdb-bin/src/fixture_connector.rs:682]
- [x] [Review][Patch] **`the_hand_authored_values_are_synthetic` cannot fail for what it names.** Its doc claims *"every value authored in this module is synthetic"*; it checks two helper functions and ignores the UUIDs, the scopes and the ids. `let _ = HostnameSource::Dhcp;` asserts nothing at all while its comment claims it makes a future author meet the rule — adding a real hostname to this module would pass it untouched. A privacy test that cannot fail is worse than none, because the next reviewer reads the name and stops [crates/opencmdb-bin/src/fixture_connector.rs:868]
- [x] [Review][Patch] **The `corpus_*` helpers' doc says the opposite of what they do** — *"read back from the FILE … nothing here restates its bytes"*, above three hard-coded UUID literals that restate exactly the file's `connector_id` and `scope`. Found by two layers [crates/opencmdb-bin/src/fixture_connector.rs:374]
- [x] [Review][Patch] **The cancellation test's mutation claim is false**: *"reds only this test"* — removing the check also reds `a_pre_cancelled_poll_emits_nothing`. A comment asserting a checkable coverage property, and it is wrong [crates/opencmdb-bin/src/fixture_connector.rs:625]
- [x] [Review][Patch] **`a_poll_emits_the_file_in_file_order` proves less than its comment claims.** Both sides call the same `read_jsonl` on the same path, so the assertion shows `poll` neither drops nor reorders — not that it "replays THE FILE and not a restatement of it" [crates/opencmdb-bin/src/fixture_connector.rs:468]
- [x] [Review][Patch] **`from_observations`' doc invites a caller bug.** It says the file path and the in-memory path *"cannot diverge in what they admit"* — true — but the SAME violation reports two different variants (`DuplicateObservationId` from a file, `RepeatedObservationId` in memory). A caller matching on one silently misses half the cases, and the reassurance is what invites them to [crates/opencmdb-bin/src/fixture_connector.rs:243]
- [x] [Review][Patch] **Two `pub` items with no non-test caller.** `observations()` is read by nothing; `split_by_connector` is called only by the test that exists to exercise it — a test asserting the existence of its own subject. Both are additions the story never asked for. Removing them also removes the `HashMap` iteration-nondeterminism finding and the empty-input finding raised against `split_by_connector` [crates/opencmdb-bin/src/fixture_connector.rs:307]
- [x] [Review][Patch] **`#[derive(Clone)]` is undeclared and unused** — the story asked for `Debug` (for `expect_err`) and the doc comment explains only `Debug` [crates/opencmdb-bin/src/fixture_connector.rs:55]
- [x] [Review][Patch] **Two test assertions are thinner than the ACs they prove.** AC6 requires the message to name the `obs_id`; the test asserts three of four. AC7b requires kind + `obs_id` + origin; the test asserts only `"IpV4"`. The `Display` impls are correct — it is the proof that is one notch weak [crates/opencmdb-bin/src/fixture_connector.rs:539]
- [x] [Review][Patch] **The `Cargo.toml` comment misdescribes its own change.** `opencmdb-core` is ALREADY a normal dependency, so this is not "a DEV edge" — it is a feature added to an edge that exists. And D47's frontier governs what `core` may depend on, not `bin → core`, which is always legal. The resolver-3 claim is right for `cargo build` but does NOT hold for `cargo clippy --all-targets` / `cargo test`, which do build core with `test-support` — worth stating, given this project's history of CI-only failures from that exact class [crates/opencmdb-bin/Cargo.toml:9]
- [x] [Review][Patch] **The completion record claims a MariaDB was reachable. It was not, and the count is wrong.** *"the 9 MariaDB-backed tests ran, a DB was reachable"* — the DB tests `return` early when `DATABASE_URL` is unset and pass identically either way, so "60 passed" is zero evidence; the auditor reproduced 60/60 with the variable unset and captured the `skipping …` output. And there are 4 such tests, not 9. This is precisely the over-claim pattern three previous reviews caught [this file]
- [x] [Review][Patch] **The completion record's "all four went RED and nothing else moved" is imprecise** — neutralising the `connector_id` guard reds two tests (`a_foreign_connector_id_is_refused` and `load_applies_the_same_invariants_as_from_observations`), both on the same invariant. The other three mutations reddened exactly one each [this file]

- [x] [Review][Defer] **`fixture_path`'s containment is lexical, so a symlink inside `fixtures/` escapes the corpus** [crates/opencmdb-bin/src/fixtures.rs:42] — deferred, pre-existing (story 4.1). `read_to_string` follows a symlink that no `Component` check can see, and the corpus's own symlink guard only walks `scenario/traps/`, never `scenario/replay/`. This story makes it reachable from the threat the containment comment already names.
- [x] [Review][Defer] **`poll` has no `.await`, so `ConnectorError::Timeout` is unreachable for this connector and a blocking sink monopolises the worker thread** [crates/opencmdb-bin/src/fixture_connector.rs:170] — deferred, deliberate: AC5b explicitly forbids adding a yield point ("an await in a zero-I/O replay is a defect, not a fix"). The consequence was not stated, and should be.

_Dismissed as noise (6): the per-item check ORDER giving a partial diagnosis (already assigned to 4.7 in the register) · `seen.insert` running before the other checks (the function returns immediately; unreachable consequence) · the blanket `#![allow(dead_code)]` (matches `arp_ping.rs`'s house style, and the module is legitimately unused until wired) · `{kind:?}` in `UndeclaredFactKind` (`FactKind` has no `Display`; `Debug` is the only option) · `HashMap` nondeterminism and the empty-input case in `split_by_connector` (both vanish with the deletion above)._

## Dev Notes

### The one design question this story turns on

The trait says a poll returns `PollSummary { capabilities, scopes_covered }`. The frozen fixture format (4.1) is a stream of `Observation`s and nothing else — **it cannot express either field.** Three ways out, and two are wrong:

- ❌ **Derive `capabilities` from the observations** (union of `Fact::kind()`). Wrong, and quietly so: it collapses *"blind to hostnames"* into *"capable of hostnames, saw none"*, which is the ONE distinction `capabilities` exists to carry. D34 §1: *"the engine must never confuse 'no `Uplink` because there is none' with 'no `Uplink` because this connector is blind to topology'. Without it, a rule like 'no diverging uplink → same switch → merge' merges happily."* Deriving it hands the identity engine a false merge and calls it a fixture.
- ❌ **Invent a header line in the JSONL now.** That changes the format 4.1 froze and 4.3 locked, in a story whose ACs say nothing about the format, and it collides head-on with 4.5's scope.
- ✅ **Take both at construction, and RECORD that the format cannot express them** (AC8). **There is precedent, not just an argument:** story 2.4's `ScriptedConnector` already takes `Capabilities` and `scopes_covered` as constructor parameters and returns them in its `PollSummary`. `FixtureConnector` is its file-fed twin; it inherits the convention rather than inventing one.

`scopes_covered` has the same shape and the same answer, plus one honest invariant available today (AC7) — bounded to the clean replay, because 4.5's partial polls will legitimately observe a scope they did not finish covering.

**Constructor-supplied does not mean unchecked, and that is the whole trick.** AC7, AC7b and AC7c are all *containment* checks running in the same direction: the file may not exceed what the constructor declared. Declared ⊋ observed stays legal everywhere — *covered and empty*, *capable and unseen* — which is precisely the space derivation would have collapsed. So the story refuses to invent the values and still refuses to let the file contradict them.

### Where it goes, and the deviation to state out loud

Architecture says `crates/opencmdb-bin/src/connectors/fixture.rs`. **That directory does not exist**: the tree's only connector is the flat `crates/opencmdb-bin/src/arp_ping.rs`. Creating `connectors/` for one file, or moving `arp_ping.rs` into it, is a refactor this story did not ask for.

**Put it at `crates/opencmdb-bin/src/fixture_connector.rs`**, flat, beside its sibling. D56's binding requirement is *the shipped crate, not `tests/`* — satisfied. The subdirectory is a diagram detail of the same kind already corrected once (`da23f9f`). State the deviation in the Dev Agent Record; do not take it quietly.

### What already exists — use it, do not rewrite it

- **`crate::fixtures::fixture_path(relative)` and `read_jsonl(path)`** [Source: crates/opencmdb-bin/src/fixtures.rs:42, :164]. They already refuse absolute paths, `..` and `./`, name a malformed line by its 1-indexed number, refuse a repeated `obs_id`, and preserve order. Call them; do not open a file yourself.
- **`FixtureError`** [Source: crates/opencmdb-bin/src/fixtures.rs:61-158] — extend it; `Display` and `source()` are exhaustive matches.
- **`ScriptedConnector`** [Source: crates/opencmdb-core/src/testing/mod.rs:72-99] is the reviewed reference for a `Connector` emitting a `Vec` with cancellation between emits, and the precedent for constructor-supplied capabilities. **It is NOT a base class and NOT to be reused** — it lives behind `test-support` and never ships [Source: crates/opencmdb-core/src/testing/mod.rs:5-7].
- **`run_connector_contract(make, expected)`** [Source: crates/opencmdb-core/src/testing/mod.rs:102-161]. Read the BODY, not only the doc comment: it runs exactly **two** blocks — a clean poll asserting `sink.observations == expected`, and a pre-cancelled poll asserting `Cancelled` plus `expected.starts_with(&sink.observations)`. *"Empty stream"* and *"missing fields"* are not cases it generates; they are inputs the CALLER supplies through `expected`. That is why AC5 demands two drives.
  Partial-then-error is deliberately outside it (it cannot be expressed through an "emit `expected` then complete" factory) and is 4.5's. The **timeout** case is neither this story's nor 4.5's — the per-scope time budget is the scheduler's (D34).
- **`ArpPingConnector`** [Source: crates/opencmdb-bin/src/arp_ping.rs:1-77] — the house style for a connector in `bin`: `pub struct` holding `id` + config, `new(id, …)`, `with_*` builders, `#![allow(dead_code)]` at the top with a reason (line 9).
- **`scratch_dir(tag)` and `expected()`** [Source: crates/opencmdb-bin/src/fixtures.rs:475, :274] are both **private to `fixtures.rs`'s own `mod tests`** and callable from nowhere else. So: hand-author the observations your load-invariant tests need (a small `obs(id, scope, …)` helper minting `obs_id`s with `Uuid::from_u128` is enough), and drive AC6/AC7/AC7b/AC7c through `from_observations` in memory rather than writing a temp `.jsonl`. Do not widen `fixtures::tests` to borrow either helper.

### Corpus facts you can rely on (measured, 2026-07-21)

- `fixtures/scenario/replay/minimal.jsonl` — 3 observations, ONE `connector_id` (`33333333-…`), ONE scope; facts spanning `Mac`/`IpV4`/`Hostname`/`OuiVendor`/`Rtt`, the third carrying a `raw` payload.
- `fixtures/scenario/replay/example-traps.jsonl` — 3 observations, same single `connector_id`, same single scope.
- So AC6 and AC7 hold over the corpus as it stands: **neither invariant should require touching a committed fixture.** If one does, look at the invariant first — but check the corpus too rather than assuming.

### Traps this story is most likely to fall into

1. **A vacuous cancellation test.** The contract's cancellation clause is satisfied by an empty sink AND by a connector that never reads the token. Only the mid-stream case proves the cancellation point exists — and only the cancelling sink of AC5b can produce it, because `poll` never awaits.
2. **Claiming more than was measured.** Three consecutive reviews caught completion records asserting guarantees the code did not carry (4.3's claimed the trap discovery "walks"; it used `read_dir`). Name the test or the command behind every claim, or write the weaker true sentence. AC3 in particular is structural — say that, do not imply a test.
3. **Asserting against a Rust literal instead of the file.** `expected()` in `fixtures.rs` exists to be a SECOND, independent statement of the bytes. The contract test here must take `expected` from `read_jsonl`, so it proves the connector replays *the file*.
4. **Reading the clock.** `poll` receives `now` and must ignore it. `observed_at` comes from the file; that determinism is what makes the corpus an oracle rather than a snapshot (D19).
5. **Skipping `--all-targets` or `xtask ci` locally.** Epic 3's retrospective recorded four CI-only failures from exactly that.

### Privacy — it applies to test code too

AC6 and AC7 cannot be reddened from the committed corpus, so you will author observations in-test, carrying MACs, IPv4 addresses and hostnames. **The rule is not limited to committed fixtures.** Every value you write uses RFC 5737 documentation addresses (`192.0.2.0/24`, `198.51.100.0/24`, `203.0.113.0/24`), locally-administered MACs, and invented hostnames. This repository is public; D19 calls a real capture in it *"disqualifying. Not debatable."*

### Dependency frontier (D47) and error discipline (D33)

`opencmdb-core` may not name `anyhow`, `axum`, `sqlx` or `askama`; `cargo xtask ci`'s frontier gate greps for it. This story adds nothing to `core` — the connector is file I/O, which is `bin`'s privilege (D55), and that is exactly why it belongs there. The only manifest change is a `[dev-dependencies]` line in `opencmdb-bin`, and the gate runs `cargo tree … -e normal`, which excludes dev edges.

**But D33 still binds inside `bin`, and the gate will not enforce it for you**: `bin` legitimately depends on `anyhow`, so nothing mechanical stops `anyhow::Result` appearing in this module. It must not. `poll` returns `Result<PollSummary, ConnectorError>` and adds **no variant** to that closed taxonomy; construction returns `FixtureError`. This one is discipline, not a gate.

### Testing standards

- Tests live inline in `#[cfg(test)] mod tests` beside the code — the workspace's only style — and in `bin`, the lowest crate that sees everything they need (D56b).
- `#[tokio::test]` for anything awaiting a poll; `opencmdb-bin` has `tokio` with `features = ["full"]`.
- **Prove-to-red is the house rule** (story 1.3 exists solely for it): AC6 and AC7 each need a test observed failing before it passed.
- Assert the CAUSE and the NAME: match the error variant, and assert the message names the offending id/scope. Two reviews found tests asserting only that an error occurred, and one asserting a substring common to every message.

### Previous story intelligence (4.1 → 4.3, all three reviewed)

- **`#[serde(deny_unknown_fields)]` everywhere** on anything parsed. Nothing new is parsed here; if you add a type that is, it joins the rule.
- **The lock is bidirectional now.** Any file created under `fixtures/` needs an `[[artefact]]` entry with its sha256 or `cargo xtask ci` reds on the orphan. `README.md` and `MANIFEST.toml` are the only exemptions.
- **Walks that quietly see less** were the recurring defect of 4.1 and 4.3 (a swallowed `read_dir` error, a non-recursive scan, an invisible symlink). This story walks nothing — keep it that way.
- **The deferred-work register must not lose an item.** An edit in 4.3 silently dropped half a bullet while rewriting it; the review called a register that loses an item worse than no register.
- **Do not forget the File List**: `sprint-status.yaml` and `Cargo.lock` (if it moves) belong in it.
- Still open and NOT this story's business: non-UTF-8 payloads passing the sha gate (deliberately re-deferred), `Fact::Mac.locally_administered` denormalization (revisit at 4.9+), collecting all validation errors instead of the first (4.7), duplicate `obs_id` ACROSS two streams.

### Git intelligence

The last three commits are one per Epic-4 story, each landing the story AND its review in a single commit (`47c9c59 Story 4.3: the corpus lock closes in both directions, and its review`). Expect the same shape. The working tree was clean at the start of this story, on `master`.

### Latest technical specifics

Nothing here needs a new crate or a version bump. **Locked** versions (from the committed `Cargo.lock`, which is what actually builds): `tokio 1.53.0` (features `full` in `bin`), `tokio-util 0.7.18` (`CancellationToken` — the one runtime primitive allowed in `core`), `serde_json 1.0.150`, `uuid 1.24.0`, `chrono 0.4.45` with `default-features = false` so the `clock` feature stays OFF workspace-wide and `Utc::now()` is not callable from the domain. The manifests carry looser caret requirements (`tokio-util 0.7.16`, `serde_json 1.0.145`) — **never invent a version, and do not mistake a requirement for a pin.** Rust 1.96+, edition 2024.

One resolver note for Task 4: `resolver = "3"` is set at the workspace root, so dev-dependency features are not unified into non-test builds. That is the mechanism AC5 relies on — measure it, do not assume it.

### Project Structure Notes

- **New:** `crates/opencmdb-bin/src/fixture_connector.rs` · **Updated:** `crates/opencmdb-bin/src/main.rs` (one `mod` line), `crates/opencmdb-bin/src/fixtures.rs` (the new `FixtureError` variants + a widened enum doc), `crates/opencmdb-bin/Cargo.toml` (one dev-dependency), `_bmad-output/implementation-artifacts/deferred-work.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml`.
- **`Cargo.lock` does NOT move** — measured: the dev-dependency resolves to the same workspace crate already in the graph. If it moves, something else changed; find out what before committing it.
- **Variance from architecture:** flat `src/fixture_connector.rs` rather than `src/connectors/fixture.rs` — see above, and state it in the record.
- **`fixtures/` is untouched by this story.** That is the expected outcome, not an omission.

### References

- [Source: _bmad-output/planning-artifacts/epics.md:914-934 — Story 4.4's four criteria, which these ACs expand]
- [Source: _bmad-output/planning-artifacts/epics.md:936-952 — Story 4.5, the owner of the outcome record and of AC8's limit]
- [Source: _bmad-output/planning-artifacts/epics.md:671-673 — Story 2.4's precedent: a scripted connector takes its capabilities and scopes as parameters]
- [Source: _bmad-output/planning-artifacts/epics.md:966-970 — Story 4.6: every scored record carries a `capability_snapshot`, or the verdict is unfalsifiable (D36)]
- [Source: _bmad-output/planning-artifacts/architecture.md:1290-1310 — D19: what the fixture imposes on the trait; `capabilities()` is false-merge prevention; the punchline]
- [Source: _bmad-output/planning-artifacts/architecture.md:1318-1319 — D19: a real capture in a public repo is disqualifying, not debatable]
- [Source: _bmad-output/planning-artifacts/architecture.md:1919-1935 — D34 §1: capabilities travel with the batch, and why a getter would need state outside the JSONL]
- [Source: _bmad-output/planning-artifacts/architecture.md:1960-1979 — D34 §2: already-emitted observations are true; the connector chooses its cancellation points, between probes and never mid-probe]
- [Source: _bmad-output/planning-artifacts/architecture.md:3285, :3432 — `FixtureConnector` in the shipped crate, not under `tests/`]
- [Source: _bmad-output/planning-artifacts/architecture.md:3289-3302 — D56b: a test lives in the lowest crate that can see everything it needs]
- [Source: crates/opencmdb-core/src/connector/mod.rs:86-140 — `ObservationSink`, `VecSink`, `PollSummary`, `Connector`, and why `poll` is object-unsafe]
- [Source: crates/opencmdb-core/src/connector/mod.rs:330-354 — `CancelsMidway`: the precedent that a mid-stream cancellation needs a bespoke driver. Its OWN shape — the connector cancels itself from inside `poll` — is NOT what AC5b prescribes; the cancelling sink is.]
- [Source: crates/opencmdb-core/src/testing/mod.rs:72-161 — `ScriptedConnector` and `run_connector_contract`]
- [Source: crates/opencmdb-core/src/observation/mod.rs:209-226 — `Scope` (Copy, Hash, no Ord, no Display) and `Capabilities` (Clone, not Copy)]
- [Source: crates/opencmdb-bin/src/fixtures.rs:23-247 — `FIXTURES_DIR`, `fixture_path`, `FixtureError`, `read_jsonl`, `read_traps`]
- [Source: crates/opencmdb-bin/src/arp_ping.rs:1-77 — the house style for a connector in `bin`]
- [Source: _bmad-output/implementation-artifacts/4-3-manifest-lockfile-and-orphans.md — the corpus lock, and the review lessons carried forward]
- [Source: _bmad-output/implementation-artifacts/deferred-work.md — the register to append to, never to rewrite]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.8 — `claude-opus-4-8`

### Debug Log References

Every claim below is a command that was run, not an inference.

- **The four load invariants, proven to red one at a time.** Each guard was neutralised to `if false {`, the matching test run, and the file restored. All four went RED. Precisely: three mutations reddened exactly one test each; neutralising the `connector_id` guard reddened TWO — `a_foreign_connector_id_is_refused` and `load_applies_the_same_invariants_as_from_observations`, both on that same invariant. _(An earlier version of this line said "nothing else moved"; the review measured otherwise.)_
- **After the review: a `for observation in observations.iter().take(1)` mutant reddens all four invariant tests.** Before the review it reddened only one — three of the tests passed a single-element `Vec`, so nothing proved the loop validated past index 0. The offending observation now sits second, behind a valid one.
- **AC5b's non-vacuity, demonstrated rather than argued.** Replacing `poll`'s `if cancel.is_cancelled() { … }` with `let _ = &cancel;` left **both contract drives GREEN** — `the_contract_passes_over_the_committed_fixture` and `the_contract_passes_over_an_empty_stream` — and reddened only the two bespoke cancellation tests (16 passed, 2 failed). That is the story's whole argument about the harness's blind spot, measured.
- **The shipped build is unchanged by the dev-dependency.** `cargo tree -p opencmdb-bin -e normal --locked --depth 1` → `opencmdb-core … FEATURES=[]`; the same command *with* dev edges → `FEATURES=[test-support]`. Decisive probe: a temporary non-`cfg(test)` reference to `opencmdb_core::testing::FixedClock` made `cargo build --locked` fail with `error[E0433]: cannot find 'testing' in 'opencmdb_core'` … `note: found an item that was configured out`, while `cargo test -p opencmdb-bin --no-run --locked` finished. Probe reverted.
- **`Cargo.lock` did not move** — `git diff --stat Cargo.lock` empty after every step. The story's "if it moves" hedge resolves to "it does not".
- **Gates:** `cargo fmt --all` clean · `cargo clippy --workspace --all-targets -- -D warnings` clean · `cargo test --workspace` → **60 (bin) + 43 (core) + 38 (xtask), 0 failed** (bin gained 18). The single "ignored" is the pre-existing ```ignore doc example on `run_connector_contract`, untouched.
- **The MariaDB-backed tests did NOT run, and there are four of them, not nine.** `DATABASE_URL` is unset in this environment; `crates/opencmdb-bin/src/{main.rs,repo.rs}` each hold two tests that `return` early when it is absent, printing `skipping …`, and they PASS either way. So "60 passed" is zero evidence about the database, and the DB-backed paths of this workspace were **not exercised**. _(An earlier version of this record claimed "the 9 MariaDB-backed tests ran, a DB was reachable" — false on both counts, caught by the review's Acceptance Auditor, which reproduced 60/60 with the variable unset and captured the skip output. This is the third consecutive story whose completion record over-claimed; the pattern is the claim that sounds like evidence but is not.)_
- `cargo xtask ci` → `✅ frontier · ✅ ddl-collation · ✅ vocabulary · ✅ fixtures 3/3 (0 generated, 3 hand-authored) · ℹ views-hash STALE` → **all gates green**. `fixtures/` was not touched, so the orphan gate never engaged and `MANIFEST.toml` is unchanged.

### Completion Notes List

- **The fixture is a connector, and the compiler now says so.** `FixtureConnector` implements the same `Connector` trait as `ArpPingConnector` and passes `run_connector_contract` with no special-casing, driven twice — over the committed `minimal.jsonl` and over an empty stream. `expected` comes from `read_jsonl` on the same path, never from a Rust literal, so the test proves the connector replays *the file*.
- **AC3 is structural, and is reported as such.** `poll` takes no path, opens no file and ignores its `now` parameter; the file is read once in `load`. There is no test asserting "zero I/O" and this record does not claim one. What is tested is the consequence: `polling_twice_replays_the_same_stream` shows a second poll returns the same non-empty stream, which a draining or re-reading implementation would break.
- **The cancellation test is the one that could have been theatre, and it is not.** `poll` contains no `.await`, so its future runs to completion the first time it is polled and no external task can interleave. The mid-stream case is driven by a test-local `ObservationSink` that cancels the token inside `emit`. Measured above: the contract harness stays green against a connector that never reads the token; only this test catches it.
- **Constructor-supplied does not mean unchecked.** `capabilities` and `scopes_covered` come from the caller (AC8), but construction refuses a stream that contradicts them. All three checks run in ONE direction — the file may not exceed the declaration — so *covered and empty* and *capable and unseen* stay legal, and two tests pin that they do. That is containment, never derivation: nothing here reads a capability off what was observed.
- **`from_observations` closed a hole it would otherwise have opened.** Routing `load` through it means the in-memory path needed `read_jsonl`'s `obs_id`-uniqueness guarantee, which a `Vec` never gets. Without that check, every construction not going through a file — this module's tests, and the harnesses of 4.6 and 4.7 — would have bypassed the anchor the labelling format rests on. New variant, keyed on `origin` + `obs_id`, no line numbers.
- **The new error variants carry `origin: String`, not `path: PathBuf`** — a deliberate break with the enum's habit, stated in its doc comment. A stream handed to `from_observations` may never have been on disk; a fabricated `PathBuf::from("<in-memory>")` would be a lie in the type told only to preserve a convention.
- **One stream is one connector, with a route out rather than only a refusal.** `split_by_connector` groups a multi-source stream by `connector_id` so a caller holding one can replay each part through its own connector. A test walks that route end to end.
- **Four findings recorded in `deferred-work.md`, none worked around**: the capabilities/`scopes_covered` limit and its owner (4.5) · `Capabilities.as_of` unrelated to the file's `observed_at` — the question is recorded and the guard deliberately NOT imposed, because inventing a validation policy under implementation pressure is what the 4.3 review sanctioned · `PollSummary` existing only on the `Ok` path while 4.6 needs a `capability_snapshot` on every scored record — the epic's own "the trait cannot express what the fixture needs" clause, flagged not fixed · AC7's bound to the clean replay, which 4.5 is free to relax. Existing bullets were appended to, not rewritten.
- **Deviation from the architecture, stated rather than taken quietly:** the module is the flat `crates/opencmdb-bin/src/fixture_connector.rs`, not `src/connectors/fixture.rs`. That directory does not exist and creating it for one file — or moving `arp_ping.rs` into it — is a refactor this story did not ask for. D56's binding requirement is *the shipped crate, not `tests/`*, which is satisfied.
- Variant names and signatures were the dev's choice, as the story allowed: `ForeignConnectorId` · `UncoveredScope` · `UndeclaredFactKind` · `RepeatedObservationId`, and `load(id, capabilities, scopes_covered, relative_path)` / `from_observations(…, origin, observations)`. `split_by_connector` and the `observations()` accessor are additions the story did not name.

### File List

- `crates/opencmdb-bin/src/fixture_connector.rs` (new — `FixtureConnector`, `split_by_connector`, 18 tests)
- `crates/opencmdb-bin/src/fixtures.rs` (modified — 4 `FixtureError` variants, their `Display` and `source()` arms, widened enum doc)
- `crates/opencmdb-bin/src/main.rs` (modified — one `mod` line)
- `crates/opencmdb-bin/Cargo.toml` (modified — the `test-support` dev-dependency)
- `_bmad-output/implementation-artifacts/deferred-work.md` (modified — four story-4.4 entries appended)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modified)
- `Cargo.lock` — **unchanged**, measured.
- `fixtures/` — **untouched**, as the story required.

### Change Log

- 2026-07-21 — The fixture becomes a connector. `FixtureConnector` replays a committed JSONL stream through the real `Connector` trait and passes `run_connector_contract` unchanged, driven over the corpus and over an empty stream. A poll reads nothing, replays in file order, and is idempotent. Construction refuses a stream that contradicts what it was told: a foreign `connector_id`, a scope the poll does not cover, a fact kind the capabilities deny, a repeated `obs_id` in memory — all four proven to red, all containment and never derivation. The format's inability to carry `capabilities`/`scopes_covered` is recorded with its owner rather than papered over, together with three neighbouring findings the implementation surfaced.
