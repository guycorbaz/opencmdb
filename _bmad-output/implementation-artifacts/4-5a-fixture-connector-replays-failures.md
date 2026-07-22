# Story 4.5a: `FixtureConnector` replays a terminal failure

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the engine's test infrastructure,
I want the fixture to replay a poll's OUTCOME — clean or partial-then-failed — from the file,
so that layer-A fault injection needs no state outside the JSONL.

## Context

epics.md defines one story 4.5 covering both halves of "the poll's outcome comes from the file". **It is split here into 4.5a and 4.5b**, because its own Decision 1 (below) establishes that the two record kinds are not variants of one idea, and because all the doctrinal risk sits in the second half. 4.5a lands the mechanism; 4.5b lands the meaning.

- **4.5a (this story)** — the stream can carry a record that is not an observation, and one of them ends the poll with a `ConnectorError`. `capabilities` stays constructor-supplied.
- **4.5b** — the capability record: a descriptor dated by the file, positional fact-kind containment, per-poll recomputation. It closes 4.4's AC8 for the capability half.

Story 4.4 made the fixture a connector and recorded, four times over, the one thing it could not do: **`capabilities` and `scopes_covered` are supplied at construction, because the frozen format is a stream of `Observation`s and nothing else.** This pair of stories is what epics.md, `deferred-work.md` and the `fixture_connector.rs` module doc all name as the owner of that gap.

It is **D35's layer A, verbatim**: *"Layer A — `FixtureConnector` replays `Result`s (401, timeout, partial). Tests the engine."* [architecture.md:2035]. Layer B (raw bytes under the real UniFi parser) is 4.18/4.19 and Epic 11, and D35 is blunt about why they must not be confused: *"Injecting `SchemaDrift` at layer A tests NOTHING… That is theatre, and it is the most insidious of the session because it LOOKS like fault injection"* [architecture.md:2032-2034].

**Explicitly OUT of scope:**

- **The capability record** — that is 4.5b, and this story must not anticipate its shape beyond reserving the marker key (AC1).
- **Moving `scopes_covered` into the file.** `deferred-work.md` says it "follows by extension" and is **not yet assigned**. Leave it constructor-supplied and leave the register entry standing.
- **Changing the `Connector` trait.** `PollSummary` on the error path is a real recorded contradiction (AC11) — this story sharpens the record and does not resolve it.
- **Adding, renaming or removing a `ConnectorError` variant.** The enum is closed (`connector/mod.rs:19-20`).
- **The metrics harness** (4.6), **the trap runner** (4.7), the register (4.8), the trap families (4.9+), layer B, the seeded generator.
- **Wiring `FixtureConnector` into `main.rs`.** It still ships unused.

## Decision 1 — a capability loss is not a failure, and that is why this story is only half

epics.md's 4.5 asks for both an error outcome and a *"mid-scan capability LOSS"*. The reflex is one enum with both. D33 already ruled otherwise:

> "`CapabilityLost` (an **event**, not a state — in steady state ping-only is an `Ok` with a reduced descriptor, **not an error**)" [architecture.md:1913-1914]

A source that lost NET_RAW is still **`Live`** — it is talking. Every `ConnectorError` except `Cancelled` blinds (`is_blinding`, connector/mod.rs:72-74), and blinding a live source is the false-"gone" NFR7 exists to make impossible. So the two records differ in what the poll RETURNS: one ends it with `Err`, the other leaves it `Ok` with a changed descriptor. This story builds only the first, and the second in 4.5b.

*(Note: `ConnectorError`'s closure was made by story 2.2, which deferred `RateLimited`, `CapabilityLost` and `ImplausibleResponse`. D33's own text still says "**Open:** the exact variant count" [architecture.md:1911]. Cite 2.2 for the closure, D33 for the reasoning.)*

## Decision 2 — how a non-observation line lives in a stream 4.1 froze and 4.3 locked

Today `read_jsonl` deserializes **every** non-empty line straight into `Observation`, which carries `#[serde(deny_unknown_fields)]` [observation/mod.rs:239]. 4.4 refused to pre-empt this and named 4.5 the owner.

**The architecture anticipates the line and rules on nothing else.** D34 §1: *"the fixture replays it for free — **one JSONL line reproduces a mid-scan NET_RAW loss**, zero mocks; with a separate getter the fixture would need state outside the JSONL"* [architecture.md:1928-1930]. So the existence of a non-observation line is sanctioned. Its **shape and discriminator** are unaddressed anywhere in `architecture.md`, and nothing forbids them — which is why this story fixes them itself rather than citing a ruling that does not exist.

**Two shapes are wrong, and one is not:**

- ❌ **Tag every line.** Rewrites both committed fixtures, bumps two sha256s, breaks 4.1's round-trip test, retires D19's sentence. Large blast radius bought for tidiness.
- ❌ **`#[serde(untagged)]`.** One line, compiles, and destroys the diagnostic 4.1 fought for: serde reports *"data did not match any variant of untagged enum"* with no line, no field, no cause, for **every** malformed line in the corpus.
- ✅ **A POSITIVE marker key, dispatched before parsing** (AC1). Not "absence of `obs_id`": an absence-based rule routes a line whose `obs_id` is misspelled into the control-record parser and reports an unknown-field error on a record the author never wrote — the exact opposite of what 4.1 fought for.

## Acceptance Criteria

### Must / must not — the short list

- **MUST** dispatch on a positive marker key, never on the absence of `obs_id`.
- **MUST NOT** let a stream script `Cancelled`.
- **MUST NOT** admit any record after a terminal failure record.
- **MUST** make the AC7 prefix assertion capable of failing — observations after the failure record.
- **MUST NOT** change the existing `from_observations(…, Vec<Observation>)` signature.
- **MUST** leave `minimal.jsonl`, `example-traps.jsonl` and their sha256 entries untouched.

---

1. **The stream carries records, dispatched on a positive marker, and the committed observation lines do not move.** A line is classified BEFORE it is parsed into a concrete type, by inspecting its top-level keys:
   - carries `record` → a control record; parse into the control type and report **its** serde error, with the line number;
   - carries `obs_id` (and no `record`) → an `Observation`; parse and report exactly the error it reports today, unchanged;
   - **neither** → a new `FixtureError` variant naming the 1-indexed line and stating that a line must carry `obs_id` or `record`. This variant is not optional: without it, `42`, `[]`, `"x"` and `{}` have no defined behaviour;
   - **both** → refused, naming the line. A line that is two things is a line whose meaning depends on the reader.

   `fixtures/scenario/replay/minimal.jsonl` and `example-traps.jsonl` keep their exact bytes and their existing `sha256` entries in `MANIFEST.toml`. **Reserve the marker value space**: this story defines one record kind; 4.5b adds `capability`. An unknown `record` value is refused naming the value and the line, never ignored.

2. **A malformed line is still named by its 1-indexed line number, with a message that says what is wrong.** This is 4.1's AC5 and it is what the dispatcher exists to protect. Four tests, each asserting the line number AND that the message names the offending field or value:
   - an observation line with a misspelled field → **today's exact error**, unchanged;
   - a control record with a misspelled field → an equally precise error, on the control record;
   - a line that is neither → the new variant;
   - a line that is not a JSON object at all.

   **The existing `a_malformed_line_names_its_line_number` (`fixtures.rs:489`) does NOT cover this** — it uses `{ not json`, syntactically invalid, so it stays green whatever the dispatcher does. Do not cite it as coverage.
   `#[serde(deny_unknown_fields)]` applies to every new type: *"a line that parses while meaning something other than it says is worse than a line that does not parse at all"* (observation/mod.rs:368-370).

3. **A failure record ends the poll with that `ConnectorError` variant, and everything before it is still emitted.** `poll` emits the preceding observations, then returns `Err(e)` with the exact variant and payload the file names. The sink is **not** rolled back — those observations are true (D34 §2, architecture.md:1977-1979). No `ConnectorError` variant is added, renamed or removed.

4. **A stream may not script `Cancelled`, and the refusal is at load.** Cancellation comes from the token, not from the corpus; `Cancelled` is the sole non-blinding variant, so a file able to mint it could claim liveness was left unchanged when nothing cancelled anything. Refuse it at load, naming the line, with a new `FixtureError` variant.

5. **Nothing may follow a terminal failure record, and the refusal is at load.** A trailing observation would be unreachable — and `read_traps` (`fixtures.rs:286-325`) cross-checks a trap's `obs_id`s against **what is in the file**, not what is reachable. A trap judging an unreachable observation would pass the cross-check and never fire: *"a trap that can never fire would sit in the corpus looking like coverage, and the gate counts traps"* (`fixtures/scenario/README.md`). That hole is the one 4.1 and 4.2 exist to close; do not reopen it. **A failure record is therefore the last record of its stream, or the load fails naming the line.**

6. **`ConnectorError` acquires a serde representation, and whichever route is taken, the closed-taxonomy guardrail is real.** It has none today (`Debug, Clone, PartialEq, Eq, Error`, connector/mod.rs:28). Two legal routes:
   - **derive** `Serialize, Deserialize` (+ `deny_unknown_fields`) on `ConnectorError` in `opencmdb-core`. `serde 1.0.228` is a **normal** dependency of `core`, so this does not cross D47's frontier. `Cancelled` becomes deserializable, so **AC4's refusal is a hand-written runtime check** — say so where you write it.
   - **mirror** the enum in `opencmdb-bin` with an exhaustive conversion. `Cancelled` can then be made **structurally** unscriptable by simply having no mirror arm for it, which is stronger than a runtime check.

   **Whichever you take, AC6 is only satisfied if some `match` scrutinises `ConnectorError` itself.** A mirror's exhaustive `match` is exhaustive over the *mirror*: adding a variant to `ConnectorError` leaves it compiling untouched, and the guardrail — *"adding a variant must break every downstream `match`"* — is silently absent. `one_of_each()` (`connector/mod.rs:188-210`) is the existing helper that exists for exactly this; a round-trip (or a mirror-conversion) test iterating it is what makes the guardrail real. **State the route, and state where the `Cancelled` refusal lives.**

7. **A faulted replay may only REMOVE knowledge, never ADD an assertion — and the assertion must be able to fail.** D35(a)/NFR8(a): the observations emitted by the faulted stream are a **strict prefix** of those emitted by the clean replay of the same stream with the failure record removed. Assert `clean.starts_with(&faulted) && faulted.len() < clean.len()`.
   **The fixture must place observations AFTER the failure record for this to mean anything.** With the record last, `faulted == clean`, the prefix holds trivially, and a `poll` that ignored the failure record entirely would pass — the defect class the 4.4 review found four times. Record the mutation you ran to prove it can fail.
   *(Note the interaction with AC5: nothing may FOLLOW a terminal failure in a committed stream. So the clean twin is the one that is committed and hand-authored, and the faulted variant is built from it in-memory by inserting the record — which is also why AC5's refusal needs its own load-time test.)*

8. **A clean stream behaves exactly as it did, and the contract still passes.** `run_connector_contract` passes unchanged over `minimal.jsonl` and over an empty stream, no special-casing. The outcome cases are **NOT** added to the shared contract — `testing/mod.rs:118-120` states why, and its clean block calls `.expect("must return Ok")` (`:129`), so it can never be driven over a faulted fixture at all.

9. **A pre-cancelled poll returns `Cancelled` even on a stream scripting a failure.** The pre-loop check (`fixture_connector.rs:175`) wins, and the reason is directional: `Cancelled` leaves liveness unchanged while every scripted variant blinds, so the token winning is the safe answer under NFR7. Cancellation is the runtime's business; the file's outcome is the source's. State it in the code and test it.

10. **The existing constructor signature does not change.** `from_observations(id, capabilities, scopes_covered, origin, Vec<Observation>)` stays as it is — roughly a dozen of this module's 18 tests and both `run_connector_contract` drives go through it (`fixture_connector.rs:426`). Add a **sibling** taking records, and route `load` through the sibling. 4.4's property that all load invariants run on ONE path must survive: the `Vec<Observation>` entry point delegates to the records path with an empty record list.

11. **The register is appended to, never rewritten** — `## Deferred from: story-4.5a (2026-07-22)`:
    - **`PollSummary` on the error path, unchanged and now concrete.** A failed poll returns no summary, so 4.6's required `capability_snapshot` has no home on that path. This story makes the case real rather than hypothetical; it does not fix it. The trait is Epic 2's.
    - **`ConnectorError::Timeout` is now reachable for `FixtureConnector`** — a scripted `Timeout` is how a fixture *presents* one (D35 names timeout as a layer-A case). **This does not close the 4.4-review finding**, which is different and stands: `poll` has no `.await`, so the scheduler's per-scope budget still cannot interrupt a replay.
    - **Whether 4.4's AC7 (`UncoveredScope`) needed relaxing.** `deferred-work.md` pre-authorised it. A failed poll returns `Err` and therefore claims no coverage at all, so it very likely did not. **If it did not, say so and leave the check intact** — do not spend the permission because it was granted.
    - **`capabilities` and `scopes_covered` are still constructor-supplied.** Note that 4.5b takes the capability half and `scopes_covered` remains assigned to nobody.

12. **The corpus walks must reach `scenario/replay/`, or the new fixture is locked bytes nobody validates.** Today `the_corpus_carries_no_real_network_data` (`fixtures.rs:440`) reads **only `minimal.jsonl`**, and the recursive validation walk (`fixtures.rs:592`) covers **only `scenario/traps/`**. Nothing walks `scenario/replay/`. Extend the privacy check (and a "it parses" check) over every `.jsonl` under `scenario/replay/`, mirroring the traps walk — otherwise `fixtures/README.md`'s claim that *"Bytes are not the whole story… every `scenario/traps/*.toml` is therefore discovered and parsed by the test suite"* becomes a half-truth the moment this story lands. Update that README paragraph to say what is actually walked.

13. **The documents describing the format are updated in the same change** (CLAUDE.md's docs-current-before-push rule): `fixtures/scenario/README.md:13-15` (*"one serialized `Observation` per line… The fixture schema IS the observation schema"* — now true of the observation lines, not of the stream), the `fixtures.rs` module doc (:8-11), the `fixture_connector.rs` module doc (:19-41, whose "what the format cannot express" section is now partly obsolete), and `fixtures/README.md` per AC12.

14. **Privacy and the lock.** Every authored value — fixture or test — is synthetic: RFC 5737 addresses, locally-administered MACs, invented hostnames. Any new file under `fixtures/` needs its `[[artefact]]` entry with sha256 or the orphan gate reds.

15. **All gates green, locally, before done:** `cargo fmt --all` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo test --workspace` · `cargo xtask ci`.
    `cargo xtask ci` prints `ℹ views-hash STALE — regenerate at next milestone` and still exits 0 — informational. **Do not regenerate `architecture-views.md`.**

## Tasks / Subtasks

- [x] **Task 1 — the dispatcher and the record type** (AC: 1, 2, 4, 5, 6)
  - [x] Add the control-record type with `#[serde(deny_unknown_fields)]`, internally tagged on `record`, in `opencmdb-bin` beside the reader (D47-safe; `core` needs to name it nowhere).
  - [x] Give `ConnectorError` a serde representation (AC6). Note its serialized shape is **heterogeneous**: externally-tagged serde renders `Timeout` and `Cancelled` as bare JSON strings but `{"Unauthorized":{"detail":"…"}}` for struct variants. Author the fixture accordingly and put an example of each in the module doc.
  - [x] The dispatcher: classify by top-level key before parsing, with the four outcomes of AC1.
  - [x] Suggested `FixtureError` variants — take them or name your own, but **say which in the record**, because the review will assert on the name and on what the message names: `UnrecognisedLine { path, lineno }` · `AmbiguousLine { path, lineno }` · `ControlRecordLine { path, lineno, source: serde_json::Error }` · `CancellationScripted { path, lineno }` · `RecordAfterTerminalFailure { path, lineno }`. The first five are all "read from a file", so they join the **`path: PathBuf`** family, not the `origin: String` one (`fixtures.rs:65-69`).
  - [x] Keep an observations-only accessor: `read_traps` (`fixtures.rs:305`) and `the_committed_fixture_reads_back_exactly` (`fixtures.rs:412`) both go through `read_jsonl` and must keep working. *(`re_serializing_reproduces_the_committed_bytes` at `:422` never calls `read_jsonl` and is not a regression surface.)*

- [x] **Task 2 — `FixtureConnector` replays the failure** (AC: 3, 9, 10)
  - [x] Add the records-taking constructor; route `load` through it; keep `from_observations` byte-identical in signature (AC10).
  - [x] `poll` walks records in file order: emit, then stop at the failure and return its error.
  - [x] The 4.4 cancellation shape is unchanged: check before the loop and between emits, never mid-observation, **no check after the loop** — decided 2026-07-22, do not silently revisit. AC9 is the pre-loop check winning over a scripted failure; write the reason next to it.

- [x] **Task 3 — the fixtures** (AC: 1, 7, 14)
  - [x] Author ONE new committed stream under `fixtures/scenario/replay/`: the **clean twin**, with at least four observations. **Do not touch `minimal.jsonl` or `example-traps.jsonl`.**
  - [x] Build the faulted variant in-memory by inserting the failure record after observation 2 — so AC7's prefix is strict and AC5 is not violated in a committed file.
  - [x] Add the new file's `[[artefact]]` entry with its sha256 to `fixtures/MANIFEST.toml`, or the orphan gate reds.

- [x] **Task 4 — the tests** (AC: 2, 3, 4, 5, 7, 8, 9, 12)
  - [x] Reader: the four dispatcher tests of AC2; `Cancelled` scripted → refused (AC4); a record after a terminal failure → refused (AC5); an unknown `record` value → refused.
  - [x] Connector: the failure replay (variant + payload + preceding observations survive); AC7's strict-prefix assertion; AC9's cancellation precedence.
  - [x] The guardrail test of AC6 iterating `one_of_each()`.
  - [x] The corpus walks of AC12.
  - [x] **Prove-to-red on every new guard** (story 1.3 exists for it), and record each mutation. Put the offending item **second, behind a valid one** — the 4.4 review found three tests a `take(1)` mutant survived. **Do not write a comment asserting a coverage property you did not measure** — the same review found five.
  - [x] Re-run both `run_connector_contract` drives (AC8); they should need no change.

- [x] **Task 5 — the record, the docs, the gates** (AC: 11, 12, 13, 15)
  - [x] Append the four entries of AC11 to `deferred-work.md`. **Append; do not rewrite or drop an existing bullet** — the 4.3 review caught exactly that.
  - [x] Update the four documents of AC13.
  - [x] Update `sprint-status.yaml` and put it in the File List.
  - [x] Run the four gates. **Name the command behind every claim in the completion record.**

### Review Findings

_Code review 2026-07-22 — three parallel layers (Blind Hunter, Edge Case Hunter, Acceptance Auditor). 14 of 15 ACs SATISFIED per the auditor's line-by-line map; **AC6 VIOLATED and proven so** by adding a variant to `ConnectorError` in a probe workspace (`cargo build` compiled, `cargo test` 45/45 passed). Eight of the nine measured claims in the Dev Agent Record were independently reproduced; the ninth is refuted below._

- [x] [Review][Decision→Patch] **DECIDED 2026-07-22 (Guy): option (a) — build the real guardrail**, including fixing story 2.2's originating comment at `connector/mod.rs:199-200`, because a comment that lies about a checkable property is the defect three consecutive reviews caught here. **AC6's closed-taxonomy guardrail does not exist, and this story shipped a NEW comment asserting it does.** `one_of_each()` (`connector/mod.rs:201`) is a `vec![...]` of constructor expressions, not a `match`; the only `match` on `ConnectorError` (`:301`) ends `_ => unreachable!()`. So a new variant can reach the fixture format with nobody deciding how it is written — the exact outcome AC6 was written to prevent. The false premise is PRE-EXISTING (story 2.2's comment at `:199-200` says "this array stops compiling"), but 4.5a was asked to make it real and instead restated it more confidently at `:241-244` and in the Completion Notes. Options: **(a)** make it real — add an exhaustive `fn variant_name(&ConnectorError) -> &'static str` with no `_` arm, used by the round-trip test, so the scrutinee is the enum itself; **(b)** weaker — `assert_eq!(one_of_each().len(), 7)`, which catches a variant added but not listed, though only at runtime; **(c)** correct both false comments and record the gap without building the guardrail. Recommendation: **(a)**, and fix story 2.2's comment in the same change since it is the origin of the claim.

- [x] [Review][Patch] **The privacy walk is structurally blind to the half of the corpus this story added.** `the_corpus_carries_no_real_network_data` calls `read_jsonl`, which this story changed to DROP control records — so a failure's hand-authored `detail` (free text, the obvious place a real hostname or IP lands) is inspected by no rule. Found independently by all three layers. Fix: walk `read_records` and assert the synthetic rule over control-record payloads too [crates/opencmdb-bin/src/fixtures.rs:687]
- [x] [Review][Patch] **`fixtures/README.md` now claims "every value in it is checked against the synthetic-data rule". That is false in two directions** — control records are dropped before the check, and `Observation.raw` is never inspected. Broadening a privacy claim without broadening the check is the worst direction to get this wrong in a public repo [fixtures/README.md:44]
- [x] [Review][Patch] **`an_in_memory_stream_may_not_script_cancellation` uses single-element vectors, twice** — the offending record is first, last and alone, so a `from_records` inspecting only `.first()` passes both halves. Every file-path test in this story correctly puts the offender second; this one violates the story's own Task 4 instruction verbatim [crates/opencmdb-bin/src/fixture_connector.rs:519]
- [x] [Review][Patch] **`from_records`' doc says the two paths "cannot diverge in what they ADMIT", four lines above documenting exactly such a divergence.** The file path refuses a record after a terminal failure; the in-memory path admits it deliberately. Stale prose left standing above its own counterexample [crates/opencmdb-bin/src/fixture_connector.rs:262]
- [x] [Review][Patch] **The register's `UncoveredScope` entry says "measured instead of assumed" and then gives a deduction — and the deduction is a non sequitur.** No measurement is named, and the stated reason ("a failed poll returns `Err`, so it claims no coverage") is irrelevant: `UncoveredScope` is a LOAD-time invariant evaluated over every observation before any poll happens. The real reason is that `from_records` still validates every observation's scope regardless of failure records. A false rationale in the register is worse than none, because the next story reasons from it [_bmad-output/implementation-artifacts/deferred-work.md:40]
- [x] [Review][Patch] **epics.md's split note claims the criteria are "unchanged, only distributed". The same diff adds five new ones** and narrows 4.5a's story sentence from "clean, failed, or partial-then-failed" to "clean or partial-then-failed" [_bmad-output/planning-artifacts/epics.md:938]
- [x] [Review][Patch] **The File List omits two changed artefacts** — `epics.md` (modified, +30/-6, acceptance criteria rewritten) and the new `4-5b-…md`. Rewriting a planning artifact's ACs is also outside the story's declared Project Structure Notes [this file]
- [x] [Review][Patch] **The strict-prefix test identifies the failure record as "not an observation", which story 4.5b will silently break.** `committed.iter().find(|r| r.as_observation().is_none())` will pick a `Record::Capability` once 4.5b lands — whose story file ships in this same diff. Match `Record::Failure(_)` explicitly, as `read_jsonl` already does [crates/opencmdb-bin/src/fixture_connector.rs:460]
- [x] [Review][Patch] **The terminal-failure guard runs before parsing and before the `Cancelled` check, masking the more serious violation.** A file whose line 3 tries to mint `Cancelled` after a failure on line 2 is told only that line 3 is unreachable; the author needs two edit cycles to learn what they actually did wrong. A malformed line after a failure likewise reports "could never be reached" about a line that is not a record at all [crates/opencmdb-bin/src/fixtures.rs:407]
- [x] [Review][Patch] **Two gates now disagree about what the corpus may contain.** `xtask ci`'s orphan rule exempts `README.md` at any depth (`xtask/src/main.rs:651`); the new walk panics on any non-`.jsonl` entry. Documenting the new control-record format at `fixtures/scenario/replay/README.md` is legal under CI and reds two tests [crates/opencmdb-bin/src/fixtures.rs:1062]
- [x] [Review][Patch] **`UnrecognisedLine.found` names nothing for the case it exists for.** Its doc says it "says what was there instead"; for an object with neither key it is the constant `"an object with neither key"`, naming none of the keys the author wrote. The test asserts only that constant plus two literals of the message template [crates/opencmdb-bin/src/fixtures.rs:443]
- [x] [Review][Patch] **The design's motivating example — a misspelled `obs_id` — has no test.** The module doc justifies the positive marker with that exact case; the test written for it misspells `facts`, which leaves `obs_id` intact and never exercises the dispatch decision the rationale is about [crates/opencmdb-bin/src/fixtures.rs:861]
- [x] [Review][Patch] **A poll whose FIRST record is a failure — emitting nothing — is never tested.** It is the only shape where the emit loop returns before `sink.emit` is ever called; epics.md's "clean, **failed**, or partial-then-failed" names it explicitly [crates/opencmdb-bin/src/fixture_connector.rs:423]
- [x] [Review][Patch] **`a_scripted_failure_ends_the_poll_and_keeps_what_was_emitted` compares the connector's output against the same reader that produced its input**, so it cannot detect a reader that mis-parses or reorders; and `detail.contains("mid-sweep")` would pass a hard-coded `Unreachable`, while the comment above it claims "the payload comes from the FILE" [crates/opencmdb-bin/src/fixture_connector.rs:436]
- [x] [Review][Patch] **`read_jsonl`'s doc and `fixture_connector.rs`'s module doc both assert more than the code holds.** The former justifies dropping control records with an invariant that is file-path-scoped only; the latter says the module "relies on" two reader rules it relies on neither of (`poll` returns at the first failure regardless of what follows, and `from_records` re-checks `Cancelled` itself) [crates/opencmdb-bin/src/fixtures.rs:508]
- [x] [Review][Patch] **`assert_documentation_ip`/`assert_synthetic_mac` take `&std::path::Display<'_>`** — a formatting adapter leaked into three signatures where `&Path` would do [crates/opencmdb-bin/src/fixtures.rs:983]

- [x] [Review][Defer] **`Observation.raw` is inspected by no privacy rule, and `OuiVendor.vendor` / `Uplink.peer_port` are silently exempted** [crates/opencmdb-bin/src/fixtures.rs:701] — deferred, pre-existing (story 4.1). `raw` is documented as the source's original payload — the single most likely place a real capture reaches the repo — and `minimal.jsonl` already ships a non-null one. Deferred rather than patched because checking an opaque JSON blob needs a stated policy, and inventing one under implementation pressure is what the 4.3 review sanctioned.
- [x] [Review][Defer] **Every unit `ConnectorError` variant has two accepted on-disk spellings** (`"Timeout"` and `{"Timeout":null}`), and only the serializer's output is pinned [crates/opencmdb-core/src/connector/mod.rs:255] — deferred. In a corpus whose bytes ARE the spec, two files can express one outcome two ways. Refusing the second needs a custom `Deserialize`; decide with 4.5b, which adds the second record kind.
- [x] [Review][Defer] **The new committed stream is judged by no trap, and its outcome cannot be expressed as one** [fixtures/scenario/replay/partial-then-failed.jsonl] — deferred. A trap must name ≥1 `obs_id` (`trap.rs:254`) and a failure record has none, so the 4.2 truth format cannot say "this poll ends `Unreachable`". Belongs with 4.7's trap runner.
- [x] [Review][Defer] **A UTF-8 BOM is reported as a JSON syntax error at line 1 column 1** [crates/opencmdb-bin/src/fixtures.rs:394] — deferred, pre-existing (story 4.1, already in the register as the non-UTF-8 entry). Newly relevant because hand-editing the control-record line is the likeliest way a BOM enters.
- [x] [Review][Defer] **`Serialize` was derived on a domain type with no production consumer** [crates/opencmdb-core/src/connector/mod.rs:40] — deferred, deliberate. Fixtures only ever READ a `ConnectorError`; `Serialize` exists for the round-trip test, and it makes the enum's JSON rendering a compatibility surface of the pure domain crate. Revisit if the guardrail decision above removes the need.

_Dismissed as noise (4): `json_kind`'s unreachable `Object` arm (defensive completeness in a diagnostic helper) · `CancellationInStream` being unreachable from `load` (the duplication is deliberate and already recorded in the register) · `ControlRecordLine` conflating an unknown record kind with a field typo (both are "this control record does not parse"; no caller distinguishes them) · scratch directories left behind when a test assertion fails (test hygiene, not a defect in the change)._

## Dev Notes

### What already exists — use it, do not rewrite it

- **`ScriptedOutcome`** [Source: crates/opencmdb-core/src/testing/mod.rs:25-33] — `Complete | Fail(ConnectorError)`, doc: *"Any observations scripted before it are still emitted first (they are true; D34 §2)"*. **The in-memory precedent for AC3.** Read it; do not reuse it (it lives behind `test-support` and never ships).
- **`ScriptedConnector::poll`** [Source: crates/opencmdb-core/src/testing/mod.rs:77-98] — the reviewed emit-then-outcome shape.
- **`run_connector_contract`** [Source: crates/opencmdb-core/src/testing/mod.rs:121-162] — read the BODY. Two blocks. The cancellation clause is `if let Err(e)`, so it **accepts `Ok`** and cannot catch a connector ignoring the token; that blind spot is why 4.4 wrote bespoke cancellation tests.
- **`FixtureConnector`** [Source: crates/opencmdb-bin/src/fixture_connector.rs:55-196] — `load`/`from_observations` sharing one invariant path; `poll`'s three-position cancellation decision (:175, :182, :187-190).
- **`read_jsonl`** [Source: crates/opencmdb-bin/src/fixtures.rs:242-278] — skips only `line.is_empty()`, 1-indexed over raw lines, refuses a repeated `obs_id` naming both lines, preserves order.
- **`FixtureError`** [Source: crates/opencmdb-bin/src/fixtures.rs:70-236] — `Display` and `source()` are exhaustive matches; the compiler points at both. Two families: `path: PathBuf` for reading, `origin: String` for 4.4's replay-admissibility (documented :65-69).
- **`ConnectorError`** [Source: crates/opencmdb-core/src/connector/mod.rs:28-75] — seven variants; `is_blinding()` = everything but `Cancelled`; `one_of_each()` at :188-210 is the "a new variant stops this compiling" helper.
- **`the_fixtures_path_is_expressed_once`** [Source: crates/opencmdb-bin/src/fixtures.rs:855] asserts exactly 2 occurrences of the corpus path. Reach the corpus through `fixture_path`; a new module-level path constant reds it.

### Corpus facts (measured, 2026-07-22)

`minimal.jsonl` — 3 observations, one `connector_id` `33333333-…`, one scope (`11111111-…`/`22222222-…`), `observed_at` `2026-01-01T00:00:{00,05,10}Z`, facts spanning `Mac`/`IpV4`/`Hostname`/`OuiVendor`/`Rtt`, third line the only non-null `raw`. `example-traps.jsonl` — 3 observations, same id, same scope. `MANIFEST.toml` — exactly three `[[artefact]]` entries; `MANIFEST.toml` itself and `README.md` files are the only orphan-rule exemptions.

### Traps

1. **An absence-based discriminator.** Decision 2. It compiles, it is short, and it inverts the diagnostic on the most common authoring mistake.
2. **A prefix assertion that cannot fail.** AC7. The variant check is the easy half; the prefix is the story's content.
3. **A mirror enum that satisfies AC6's letter and ships no guardrail.** AC6.
4. **Claiming more than was measured.** Four consecutive completion records over-claimed; 4.4's asserted a MariaDB was reachable — it was not, and the test count was wrong. `DATABASE_URL` is unset here and the four DB-backed tests `return` early and pass either way, so a green `cargo test --workspace` says **nothing** about the database. Name the test or the command behind every claim, or write the weaker true sentence.
5. **Skipping `--all-targets` or `xtask ci` locally.** Epic 3's retrospective recorded four CI-only failures from exactly that.

### Privacy — it applies to test code too

Every authored value uses RFC 5737 documentation addresses (`192.0.2.0/24`, `198.51.100.0/24`, `203.0.113.0/24`), locally-administered MACs, and invented hostnames. This repository is public; D19 calls a real capture in it *"disqualifying. Not debatable."* [architecture.md:1318-1319].

### Dependency frontier (D47) and error discipline (D33)

`opencmdb-core` may not name `anyhow`, `axum`, `sqlx` or `askama`; `cargo xtask ci` greps for it. **`serde` is not on that list** and is a normal dependency of `core` (`serde 1.0.228`), so AC6's derive route is frontier-safe. `serde_json` in `core` is a **dev**-dependency — fine for a round-trip test, not for anything shipped. File I/O stays in `bin` (D55).

D33 binds inside `bin` and no gate enforces it: `bin` legitimately depends on `anyhow`, so nothing mechanical stops `anyhow::Result` appearing here. It must not. `poll` returns `Result<PollSummary, ConnectorError>`; construction returns `FixtureError`.

### Testing standards

- Tests live inline in `#[cfg(test)] mod tests` beside the code — the workspace's only style, no `tests/` directory exists anywhere — in the lowest crate that sees what they need (D56b).
- `#[tokio::test]` for anything awaiting a poll; `opencmdb-bin` has `tokio` with `features = ["full"]`.
- The `test-support` dev-dependency edge already exists in `opencmdb-bin/Cargo.toml`. Resolver 3 keeps it out of `cargo build` but **not** out of `cargo clippy --all-targets` or `cargo test` — the distinction 4.4's review had to correct, and the class this project's CI-only failures came from.
- Assert the CAUSE and the NAME: match the variant, and assert the message names the offending line/field/value. Two reviews found tests asserting only that an error occurred.

### Previous story intelligence (4.1 → 4.4, all four reviewed)

- **`#[serde(deny_unknown_fields)]` on everything parsed.** This story adds parsed types; no exception.
- **The lock is bidirectional.** Any new file under `fixtures/` needs its `[[artefact]]` entry or `xtask ci` reds on the orphan.
- **Walks that quietly see less** were the recurring defect of 4.1 and 4.3 (a swallowed `read_dir` error, a non-recursive scan, an invisible symlink). AC12 adds a walk — write it so it cannot see less than it claims.
- **Single-element test vectors hide loop bugs.** Offending item second, always.
- **Comments that assert a checkable property get checked.** If you write *"reds only this test"*, run the mutation and count.
- Still open and NOT this story's business: non-UTF-8 payloads passing the sha gate, `Fact::Mac.locally_administered` denormalization (4.9+), collecting all validation errors instead of the first (4.7), duplicate `obs_id` ACROSS two streams, `fixture_path`'s lexical containment vs a symlink (pre-existing, 4.1).

### Git intelligence

Epic 4's stories land one per commit, story AND review together (`e00fd04 Story 4.4: the fixture becomes a connector, and its review`). The last five commits are four such stories plus `da23f9f`, an architecture-only freshness pass. Working tree clean on `master` at story start.

### Latest technical specifics

No new crate, no version bump. **Locked** (from the committed `Cargo.lock`, verified 2026-07-22): `tokio 1.53.0` (`full` in `bin`), `tokio-util 0.7.18`, `serde 1.0.228`, `serde_json 1.0.150`, `uuid 1.24.0`, `chrono 0.4.45` with `default-features = false` so the `clock` feature stays OFF and `Utc::now()` is not callable from the domain. Manifests carry looser caret requirements — **never invent a version, and do not mistake a requirement for a pin.** Rust 1.96+, edition 2024, `resolver = "3"` at the workspace root (`Cargo.toml:6`).

### Project Structure Notes

- **Updated:** `crates/opencmdb-bin/src/fixtures.rs` (record type, dispatcher, new `FixtureError` variants, corpus walks, module doc), `crates/opencmdb-bin/src/fixture_connector.rs` (records constructor, failure replay, module doc), `crates/opencmdb-core/src/connector/mod.rs` (only on AC6's derive route), `fixtures/MANIFEST.toml`, `fixtures/README.md`, `fixtures/scenario/README.md`, `deferred-work.md`, `sprint-status.yaml`.
- **New:** one `.jsonl` under `fixtures/scenario/replay/`.
- **Unchanged, expected:** `minimal.jsonl`, `example-traps.jsonl` and their sha256 entries, `crates/opencmdb-bin/src/main.rs`.
- **`Cargo.lock` should not move.** If it does, find out what changed before committing it.

### References

- [Source: _bmad-output/planning-artifacts/epics.md:936-952 — Story 4.5, which 4.5a and 4.5b jointly implement]
- [Source: _bmad-output/planning-artifacts/epics.md:954-974 — Story 4.6: every scored record carries a `capability_snapshot` (D36)]
- [Source: _bmad-output/planning-artifacts/architecture.md:1911-1914 — D33: `CapabilityLost` is an EVENT, not a state; and the variant count is left "Open" there, closed by story 2.2]
- [Source: _bmad-output/planning-artifacts/architecture.md:1859-1889 — D33: the admission rule, the liveness table, `is_blinding`'s safe default]
- [Source: _bmad-output/planning-artifacts/architecture.md:1927-1930 — D34 §1: a capability IS an observation; ONE JSONL LINE reproduces a mid-scan NET_RAW loss — the passage that sanctions a non-observation line]
- [Source: _bmad-output/planning-artifacts/architecture.md:1960-1983 — D34 §2: already-emitted observations are true (:1977-1979); cancellation points between probes, never mid-probe (:1978)]
- [Source: _bmad-output/planning-artifacts/architecture.md:1998-2000 — D34 §3b: the per-scope time budget is the SCHEDULER's, the per-host probe timeout is the connector's]
- [Source: _bmad-output/planning-artifacts/architecture.md:2032-2035 — D35: "injecting SchemaDrift at layer A tests NOTHING"; Layer A replays `Result`s (401, timeout, partial)]
- [Source: _bmad-output/planning-artifacts/architecture.md:2093-2094 — "the faulted run cannot invent a single fact. Everything else is observability." (under "Category 3 — theatre named", not under D35)]
- [Source: _bmad-output/planning-artifacts/architecture.md:1267-1347 — D19: the punchline (:1302-1303), the fixture schema IS the Observation schema (:1273), the synthetic-data rule (:1313-1325)]
- [Source: _bmad-output/planning-artifacts/prd.md:1207-1220 — NFR7: an observation is structurally incapable of expressing "gone"]
- [Source: _bmad-output/planning-artifacts/prd.md:1230-1240 — NFR8's four falsifiable assertions; (a) is this story's AC7]
- [Source: crates/opencmdb-core/src/testing/mod.rs:25-98 — `ScriptedOutcome` and `ScriptedConnector`: the precedent to read, not to reuse]
- [Source: crates/opencmdb-core/src/testing/mod.rs:118-162 — `run_connector_contract`: its stated exclusion of partial-then-error and timeout, and its `Ok`-accepting cancellation clause]
- [Source: crates/opencmdb-core/src/connector/mod.rs:19-75, :188-210 — the closed taxonomy, `is_blinding`, and `one_of_each()`]
- [Source: crates/opencmdb-bin/src/fixtures.rs:65-278 — `FixtureError`'s two families, `read_jsonl`'s guarantees]
- [Source: crates/opencmdb-bin/src/fixtures.rs:286-325 — `read_traps`' cross-check: why AC5 exists]
- [Source: crates/opencmdb-bin/src/fixtures.rs:412, :422, :440, :489, :592, :855 — the existing tests this story must not weaken, and the two walks AC12 extends]
- [Source: crates/opencmdb-bin/src/fixture_connector.rs:19-196, :426 — the module doc AC13 updates, the poll shape AC9 preserves, the contract drive AC10 protects]
- [Source: _bmad-output/implementation-artifacts/4-4-fixture-connector-replays-jsonl.md:142 — 4.4 naming 4.5 the owner of this format change; :117, :127, :254 — the review lessons carried forward]
- [Source: _bmad-output/implementation-artifacts/deferred-work.md — the four story-4.4 entries and the two 4.4-review entries; append, never rewrite]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.8 — `claude-opus-4-8[1m]`

### Debug Log References

Every claim below is a command that was run, not an inference.

- **Seven guards, proven to red one at a time.** Each guard was neutralised, the full workspace
  suite run, and the file restored from a backup taken before the first mutation (`grep` for
  residual mutation markers afterwards returned none). Each mutation reddened **exactly one** test,
  except the first, which reddened two on the same behaviour:
  | Mutation | Tests reddened |
  |---|---|
  | `poll` ignores `Record::Failure` (`continue` instead of `return Err`) | `a_scripted_failure_ends_the_poll_and_keeps_what_was_emitted`, `a_faulted_replay_emits_a_strict_prefix_of_the_clean_one` |
  | a line with NEITHER marker falls through to `Observation` | `a_line_carrying_neither_marker_is_named` |
  | reader stops refusing a scripted `Cancelled` | `a_stream_may_not_script_cancellation` |
  | reader stops refusing a record after a terminal failure | `nothing_may_follow_a_terminal_failure` |
  | a line with BOTH markers is parsed as an observation | `a_line_carrying_both_markers_is_refused` |
  | `from_records` stops refusing an in-memory `Cancelled` | `an_in_memory_stream_may_not_script_cancellation` |
  | `ConnectorError` loses `deny_unknown_fields` | `an_unknown_field_on_a_variant_is_refused` |

- **AC7's vacuity, demonstrated rather than argued — and the first attempt at the demonstration was
  wrong.** The claim under test: with the failure record LAST (the only shape a committed stream may
  have), the prefix assertion alone cannot fail. First probe — fault last, strictness assertion
  neutralised, `poll` ignoring the record — went RED, so it proved nothing about the prefix: the
  test's separate `assert!(faulted_error.is_some())` was catching the mutant. Re-run with that
  assertion ALSO neutralised, leaving only `clean.starts_with(&faulted)`: **PASSED.** So a `poll`
  that ignores the failure record entirely does satisfy the prefix assertion when the fault is last.
  That is why the test injects the fault mid-stream, and why `from_records` deliberately does not
  enforce the "nothing may follow" rule.

- **The corpus walk was measured, not assumed.** Replacing `assert!(checked > 0)` with
  `assert_eq!(checked, 999)` reported `left: 3` — the walk visits all three committed replay
  streams. Before this story `the_corpus_carries_no_real_network_data` read **`minimal.jsonl`
  alone**, so `example-traps.jsonl` was locked by sha256 and inspected by no privacy rule; it is now
  inspected, and that is a pre-existing hole this story closed as a side effect.

- **`Cargo.lock` DID move, contrary to the story's expectation, and by exactly one line.**
  `git diff Cargo.lock` → `+ "serde",` inside `opencmdb-bin`'s dependency list. `cargo update
  --offline -p opencmdb-bin` reported `Locking 0 packages`: **no crate added to the graph, no
  version changed** — only the direct edge recorded. The story's "should not move" was written
  before the manifest need was known; this is the measured correction, not a silent deviation.

- **The shipped build is unchanged.** `cargo build --locked` compiles;
  `cargo tree -p opencmdb-bin -e normal --locked --depth 1` shows `opencmdb-core` with no features,
  and the same command with dev edges shows the second (`test-support`) edge.

- **Gates:** `cargo fmt --all` clean · `cargo clippy --workspace --all-targets -- -D warnings` clean
  · `cargo test --workspace` → **74 (bin) + 45 (core) + 38 (xtask), 0 failed** (bin gained 14, core
  gained 2). The single "ignored" is the pre-existing ```ignore doc example on
  `run_connector_contract`, untouched.
- `cargo xtask ci` → `✅ frontier · ✅ ddl-collation · ✅ vocabulary · ✅ fixtures 4/4 (0 generated,
  4 hand-authored) · ℹ views-hash STALE` → **all gates green.** The fixture count rose 3 → 4 with
  the new stream; `architecture-views.md` was NOT regenerated, as the story required.

- **The MariaDB-backed tests did NOT run.** `DATABASE_URL` is unset in this environment and the four
  DB-backed tests in `main.rs`/`repo.rs` `return` early, printing `skipping …`, passing either way.
  So "74 passed" is **zero evidence** about the database, and no DB-backed path was exercised.

### Completion Notes List

- **The poll's outcome now comes from the file, and the diagnostic survived it.** A stream may carry
  a `failure` control record; `poll` emits everything before it and returns that `ConnectorError`
  with the payload the file names. The sink is not rolled back — those observations are true
  (D34 §2). All 18 pre-existing `fixture_connector` tests and all 18 pre-existing `fixtures` tests
  passed **unchanged**, which is the evidence for AC10: no call site moved.

- **The discriminator is positive, and that was the story's sharpest constraint.** A line is
  classified by top-level key before any typed parse: `record` → control, `obs_id` → observation,
  neither → `UnrecognisedLine` naming what was there instead, both → `AmbiguousLine`. An
  absence-based rule would have routed a misspelled `obs_id` into the control parser and blamed a
  record the author never wrote. Measured consequence: an observation line with a typo still reports
  `FixtureError::Line` naming `factz` at line 2, exactly as before.

- **Syntactically invalid lines are still `FixtureError::Line`.** The classification step parses to
  `serde_json::Value` first, so `{ not json` fails as it always did and
  `a_malformed_line_names_its_line_number` keeps its meaning — but, as the story warned, that test
  could not have caught a dispatcher regression, so four new tests cover the four dispatch outcomes.

- **`ConnectorError` took the derive route, not the mirror.** `Serialize + Deserialize +
  deny_unknown_fields` in `opencmdb-core`; `serde` is a normal dependency of `core`, so D47's
  frontier is untouched. The guardrail AC6 demands is `every_variant_round_trips_through_json`,
  **and this claim was WRONG as first shipped.** `one_of_each()` is a `vec![…]`, not a `match`; the
  code review proved a new variant compiled straight past it (build OK, 45/45 green). The guardrail
  now exists as `variant_name`, an exhaustive `match` with no `_` arm that the round-trip test
  calls — re-proved by adding a variant, which fails `cargo build --tests` with
  `E0004: non-exhaustive patterns … not covered`. Story 2.2's originating comment, which made the
  same false claim, was corrected in the same change.
  Because the derive makes `"Cancelled"` deserializable, the refusal is a **runtime check on two
  paths**, and that duplication is recorded in `deferred-work.md` rather than hidden.

- **Two rules protect the trap corpus, and one of them is deliberately asymmetric.** A file may not
  script `Cancelled`, and nothing may follow a terminal failure — an unreachable observation would
  still satisfy `read_traps`' cross-check and yield a trap that can never fire. The second rule is
  enforced on the FILE path only: an in-memory stream is judged by no trap file, and forbidding the
  shape there would forbid the very test that proves AC7. Stated in `from_records`' doc comment and
  in the register, not left to be discovered in the diff.

- **A pre-existing privacy hole closed as a side effect.** The corpus privacy check now WALKS
  `scenario/replay/` recursively (refusing symlinks and foreign extensions) instead of naming one
  file, and a second walk parses every stream. Measured: 3 streams visited, where 1 was inspected
  before. `fixtures/README.md`'s claim about what the suite reads was updated to match.

- **One manifest change beyond the story's "no new dependency".** `serde` with `derive` became a
  direct dependency of `opencmdb-bin`, because that crate now derives on its own types. It was
  already in the tree via `opencmdb-core`, `serde_json`, `toml`, `config` and `sqlx`, and
  `cargo update --offline` confirmed **0 packages locked** — the same reasoning the manifest already
  records for `futures-util`. Flagged here rather than buried: it is a manifest edit the story did
  not authorise in so many words.

- **Naming choices, as the story allowed:** `Record` / `Record::Observation` / `Record::Failure`;
  `read_records` alongside a narrowed `read_jsonl`; `FixtureConnector::from_records` beside an
  unchanged `from_observations`; and the variants `UnrecognisedLine` · `AmbiguousLine` ·
  `ControlRecordLine` · `CancellationScripted` · `RecordAfterTerminalFailure` (path family) and
  `CancellationInStream` (origin family). `read_jsonl` now DROPS control records — documented as
  deliberate, and sound only because nothing may follow a terminal failure, so every observation in
  an admissible stream is reachable.

### File List

- `crates/opencmdb-core/src/connector/mod.rs` (modified — serde derive + `deny_unknown_fields` on
  `ConnectorError`, 2 tests)
- `crates/opencmdb-bin/src/fixtures.rs` (modified — `Record`, `ControlRecord`, `read_records`, a
  narrowed `read_jsonl`, 6 `FixtureError` variants, the replay-stream walk, module doc; 10 tests)
- `crates/opencmdb-bin/src/fixture_connector.rs` (modified — `from_records`, records-based `poll`,
  module doc; 4 tests)
- `crates/opencmdb-bin/Cargo.toml` (modified — `serde` as a direct dependency)
- `fixtures/scenario/replay/partial-then-failed.jsonl` (new — 4 observations then a failure record)
- `fixtures/MANIFEST.toml` (modified — the new artefact and its sha256)
- `fixtures/README.md` (modified — what the suite actually walks and reads)
- `fixtures/scenario/README.md` (modified — the two line shapes and the two corpus rules)
- `_bmad-output/implementation-artifacts/deferred-work.md` (modified — six story-4.5a entries
  appended)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modified)
- `_bmad-output/planning-artifacts/epics.md` (modified — story 4.5 split in place into 4.5a/4.5b,
  criteria redistributed and sharpened). **Not declared in the story's Project Structure Notes**;
  added here after the code review pointed it out. Editing a planning artifact was a consequence of
  the split, not of the implementation.
- `_bmad-output/implementation-artifacts/4-5b-fixture-connector-replays-capability-changes.md`
  (new — the sibling story, written at preparation time and referenced by the sprint-status edit)
- `Cargo.lock` — **moved by one line**, measured; see the Debug Log.

### Change Log

- 2026-07-22 — The poll's outcome comes from the file. A replay stream may carry a `failure` control
  record that ends the poll with a `ConnectorError` named in the file, with everything emitted
  before it surviving (D34 §2). Lines are classified by a positive marker key before parsing, so
  every malformed line still fails with its own diagnostic and its own 1-indexed number. A stream
  may not script `Cancelled`, and nothing may follow a terminal failure — the second rule on the
  file path only, deliberately. `ConnectorError` gained a serde representation in `opencmdb-core`
  with a round-trip guardrail over `one_of_each()`. The corpus privacy check now walks
  `scenario/replay/` instead of naming one file, closing a pre-existing hole. Seven guards proven to
  red; AC7's prefix assertion demonstrated to be vacuous when the fault is last, which is why the
  test injects it mid-stream.
- 2026-07-22 — Code review (three parallel layers), 17 findings applied. The AC6 guardrail did not
  exist and was built: `variant_name`, an exhaustive `match` over `ConnectorError`, re-proved by
  adding a variant and watching `cargo build --tests` fail with `E0004`. The corpus privacy walk
  now reads RECORDS, so a failure's free text is scanned for real addresses — it previously went
  through `read_jsonl`, which drops control records, leaving the half this story added inspected by
  nothing. Terminal-failure detection moved AFTER parsing so an inadmissible line is diagnosed for
  what it is. `UnrecognisedLine` now names the keys it found, which gave the design's motivating
  case — a misspelled `obs_id` — its first test. Three tests added (misspelled `obs_id`,
  failure-as-first-record, inadmissible-line-after-failure) and the in-memory cancellation test's
  one-element vectors fixed. Six false or over-strong claims corrected across `fixtures/README.md`,
  `epics.md`, `deferred-work.md`, two module docs and this record.
