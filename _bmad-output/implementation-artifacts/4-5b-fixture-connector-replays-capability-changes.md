# Story 4.5b: `FixtureConnector` replays a capability change, dated by the file

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the engine's test infrastructure,
I want a mid-scan capability change to be one line of the fixture, with the descriptor dated by the file,
so that a downgrade trap needs no state outside the JSONL — and so a verdict can be replayed under the capability that produced it.

## Context

**Depends on 4.5a**, which built the record dispatcher, the reader that returns records, and the terminal-failure record. This story adds the second record kind and the invariant it forces.

This is the half that closes 4.4's AC8. Story 4.4 recorded it four times:

> "`capabilities` and `scopes_covered` are supplied at construction, not read from the file… D34 §1 argues the opposite — that the descriptor should travel with the batch, precisely because *'the fixture replays it for free — one JSONL line reproduces a mid-scan NET_RAW loss, zero mocks; with a separate getter the fixture would need state outside the JSONL'*. **Constructor parameters ARE that state outside the JSONL.** … **Owner: story 4.5** for the capability half" [deferred-work.md:26]

And the sharpest instance, which this story is the *"real fix"* for:

> "**`Capabilities.as_of` is unrelated to the file's `observed_at`.** It is caller-supplied while every observation is dated by the file, so a replay can date its capability descriptor in a moment its own stream contradicts — and D34 §1's whole content is that the descriptor is *a dated fact, not a constant*, with **4.5's downgrade traps being diffs over exactly that field**. … 4.5 puts the record in the file and can then **date the descriptor FROM the file**, which is the real fix rather than a rule bolted on." [deferred-work.md:27]

Both entries are closed by this story or they are not closed at all. AC9 makes that explicit.

**Explicitly OUT of scope:**

- **`scopes_covered`.** `deferred-work.md` says it follows by extension and is **not yet assigned**. It stays constructor-supplied; the register entry stands.
- **Changing the `Connector` trait.** `PollSummary` on the error path remains unresolved (AC9).
- **Adding a `ConnectorError` variant.** A capability change is not an error — that is Decision 1.
- **The metrics harness** (4.6), the trap runner (4.7), the trap families (4.9+), layer B, the seeded generator.

## Decision 1 — a capability change leaves the poll `Ok`

epics.md calls it a *"mid-scan capability LOSS"* and the reflex is an error variant. D33 ruled otherwise:

> "`CapabilityLost` (an **event**, not a state — in steady state ping-only is an `Ok` with a reduced descriptor, **not an error**)" [architecture.md:1913-1914]

A source that lost NET_RAW is **`Live`** — it is talking. Every `ConnectorError` except `Cancelled` blinds (`connector/mod.rs:72-74`), and blinding a live source is the false-"gone" that NFR7 exists to make structurally impossible. So the capability record changes the descriptor and the poll **continues**, returning `Ok` with the last descriptor in force.

The framing that makes it matter, one storey up:

> "**without the capability↔batch link, the engine sees an absent MAC and reads it as 'the MAC disappeared' — that is a FALSE MERGE, exactly what `capabilities()` existed to prevent.** Same doctrine one storey down: absence of proof ≠ proof of absence — **including when it is THE SENSOR that shrank, not the network that changed.**" [architecture.md:1955-1958]

## Decision 2 — what this story deliberately gives up, and what it gets

Story 4.4 established a containment invariant it argued at length and proved to red: *"the file may not exceed the declaration"* — an observation may not emit a fact kind the constructor's `Capabilities` deny (`fixture_connector.rs:134-143`, AC7b).

**This story supersedes it, and that must be stated rather than discovered.** Once the descriptor comes from the file, the file becomes the authority — which is D34 §1's actual content: *"The connector is no longer the authority — **the poll is.** That is a shift of authority, and it is the real content of the decision"* [architecture.md:1934-1935]. A file can therefore grant itself kinds the constructor never declared.

What replaces it is **stronger where it counts and weaker only where the authority moved**:

- the global check becomes **positional** (AC4): each observation is checked against the descriptor **in force at its own position**, not against one set for the whole file;
- so *capable and unseen* stays legal — the distinction the descriptor exists for;
- and *emitting what you just declared yourself blind to* becomes impossible, which the global check could not express at all.

The 4.4 invariant is not dropped, it is relocated. **Say so in the module doc and in the register** (AC9), because a reviewer reading only the diff will see a proved-to-red guarantee apparently deleted.

## Acceptance Criteria

### Must / must not — the short list

- **MUST** carry `as_of` on the record itself, dated by the file.
- **MUST** check fact-kind containment against the descriptor in force **at each observation's own position**.
- **MUST NOT** write the in-force descriptor back onto `self` — recompute per poll.
- **MUST** report a positional violation from the LOAD path, not from `poll`.
- **MUST** state that 4.4's AC7b global containment is superseded, not dropped.

---

1. **A capability record is a dated descriptor, and its date comes from the file.** The record carries the full `Capabilities` — `as_of` and `kinds` — as a `record` line (4.5a's marker; value `capability`). `Capabilities` is already `Serialize + Deserialize + deny_unknown_fields` [observation/mod.rs:216-232], so no new domain type is needed. **`as_of` is authored in the file, never derived from a clock and never taken from the constructor.** This is what closes `deferred-work.md:27`.

2. **`as_of` is non-decreasing, and never predates a fact it postdates.** Two load-time rules, both cheap and both true:
   - a capability record's `as_of` must be **≥ the `observed_at` of every observation preceding it** in the stream — a descriptor cannot be dated before facts collected under it;
   - successive capability records have **non-decreasing** `as_of`.

   Each violation is refused at load, naming the record's `as_of`, the offending `observed_at`, and the origin. **This is the whole reason the record carries its own date** — without these rules the file could still date a descriptor in a moment its own stream contradicts, which is the defect `deferred-work.md:27` names.
   *(These two rules are the policy 4.4 deliberately refused to invent under implementation pressure. They are invented HERE, deliberately, in the story that owns the record — which is what the 4.3 review asked for.)*

3. **The poll returns `Ok` with the last descriptor in force, read from the file.** The replay continues past the record. `PollSummary.capabilities` is the descriptor in force when the stream ended. The constructor's `capabilities` parameter becomes the **initial** descriptor — the one in force before any record — and is retained, not removed (AC7).

4. **Fact-kind containment becomes POSITIONAL.** Each observation's `Fact::kind()`s must be contained in the descriptor **in force at that observation's position**, or the load fails naming the kind, the `obs_id`, the origin, **and the `as_of` of the descriptor that denies it** (which is how a record with no `obs_id` is named — see AC5). This is D34 §1's own doctrine one storey down:
   > "the doubt predicate must be evaluated **against the BATCH's descriptor**, not the current one… *Otherwise an observation ingested when NET_RAW existed would be re-evaluated later with a ping-only descriptor and become retroactively `SourceNotCapable`. **The past would change status.***" [architecture.md:1944-1947]

   **Both directions need a test, and the legal one is what a careless implementation breaks:** an observation emitting `Rtt` **before** the record that drops `Rtt` is legal and must stay legal; the same observation **after** it is refused. A test that only covers the refusal would pass an implementation that checks every observation against the final descriptor.

5. **A capability record is named by its `as_of`, never by a line number and never by an invented ordinal.** A control record has no `obs_id` — that is the disjointness 4.5a's dispatcher rests on — and story 4.2's rule is *"never by line number"* (a later edit shifts lines under the truth). `as_of` is the record's natural, stable, authored identity, and AC2 makes it unique enough to name one. **When the descriptor in force is still the constructor's initial one, the message must say that** rather than naming an `as_of` no record authored.

6. **An upgrade is legal, and the reason is recorded.** A record may ADD kinds — a source can regain NET_RAW, and FR5 treats capability as an axis that moves in both directions. The consequence is Decision 2: 4.4's global containment no longer holds and is superseded by AC4. **An empty `kinds` set is legal too** (a source capable of nothing, still `Live`, still `Ok`) — under FR19 that suppresses divergences on every field while leaving liveness untouched, which is a meaningful state and not an error. Both need a test, and both need one sentence in the module doc so the next reader does not read a bug.

7. **Replay stays idempotent, descriptor included.** Polling the same instance twice yields the same observations, the same outcome AND the same final descriptor. **This is the trap of the story**: an implementation that writes the reduced descriptor back into `self.capabilities` passes the first poll and starts the second already degraded. The in-force descriptor is computed during each replay and never persisted onto the connector; write the reason in the code next to it. 4.4's `polling_twice_replays_the_same_stream` (`fixture_connector.rs:349`) asserts only `first.observations == second.observations` and never touches the summary — extend it or add a sibling.

8. **A stream may carry a capability record AND a terminal failure, and the descriptor then dies with the `Err`.** This is legal and must be tested, because it is the concrete instance of the open contradiction: `poll` returns `PollSummary` only on `Ok`, so a poll that degraded and then failed reports **no descriptor at all** — while 4.6 requires a `capability_snapshot` on every scored record. Do not fix it here (AC9).

9. **The register is appended to, never rewritten** — `## Deferred from: story-4.5b (2026-07-22)`. Two entries CLOSE and two do not; say which is which against the existing bullets without editing them away:
    - ✅ **CLOSED — `capabilities` is no longer state outside the JSONL** (the capability half of `deferred-work.md:26`). The constructor now supplies only the *initial* descriptor.
    - ✅ **CLOSED — `Capabilities.as_of` is now dated by the file** (`deferred-work.md:27`), with the two ordering rules of AC2. This is the entry that named 4.5 as its real fix.
    - ⚠️ **STILL OPEN — `scopes_covered`** is constructor-supplied and assigned to nobody.
    - ⚠️ **STILL OPEN and now concrete — `PollSummary` on the error path** (`deferred-work.md:28`). AC8 makes a degraded-then-failed poll reachable, so the contradiction is now demonstrable rather than argued. **But the shape of 4.6's problem has changed**: the descriptor is now readable from the file, so 4.6 can obtain a `capability_snapshot` without a trait change. Record that — it is this story's actual contribution to 4.6.
    - **NEW — 4.4's AC7b global containment is superseded by AC4**, not dropped. Decision 2, in one bullet, so the next reviewer does not read a deleted guarantee.

10. **The documents are updated in the same change** (CLAUDE.md's docs-current-before-push rule): the `fixture_connector.rs` module doc's *"What the format cannot express"* section (:19-41) is now **wrong** for the capability half and must be rewritten, not annotated; `fixtures/scenario/README.md` gains the capability record; `fixtures.rs`'s module doc follows 4.5a's.

11. **Privacy and the lock.** Every authored value is synthetic: RFC 5737 addresses, locally-administered MACs, invented hostnames. The new fixture needs its `[[artefact]]` entry with sha256 or the orphan gate reds. 4.5a extended the corpus walks over `scenario/replay/` — the new file is inside them, and if it is not, that is a defect in 4.5a's walk, not an exemption.

12. **All gates green, locally, before done:** `cargo fmt --all` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo test --workspace` · `cargo xtask ci`.
    The `ℹ views-hash STALE` line is informational and the gate still exits 0. **Do not regenerate `architecture-views.md`.**

## Tasks / Subtasks

- [x] **Task 1 — the record and its ordering rules** (AC: 1, 2, 6)
  - [x] Add the `capability` arm to 4.5a's control-record type, carrying `Capabilities` with `deny_unknown_fields`.
  - [x] The two `as_of` rules of AC2, at load, each with its own `FixtureError` variant. Suggested — take them or name your own, but **say which in the record**, because the review will assert on the name and on what the message names: `CapabilityPredatesObservation { origin, as_of, observed_at, obs_id }` · `CapabilityOutOfOrder { origin, as_of, previous_as_of }`. These describe a stream that may never have been on disk, so they join the **`origin: String`** family (`fixtures.rs:65-69`), not `path: PathBuf`.
  - [x] Upgrade and empty-`kinds` explicitly permitted, with the reason in the module doc (AC6).

- [x] **Task 2 — positional containment** (AC: 4, 5)
  - [x] Walk the records in order, tracking the descriptor in force, starting from the constructor's initial value.
  - [x] **The check runs on the LOAD path, with the other invariants.** It cannot run during `poll`: the only error channel there is `Result<PollSummary, ConnectorError>`, whose taxonomy is closed and which this story may not extend — a replay-time check would have nowhere to report and could only panic or abuse `Misconfigured`.
  - [x] The violation message names kind + `obs_id` + origin + the denying descriptor's `as_of`, or "the initial descriptor" when no record is yet in force (AC5).

- [x] **Task 3 — the replay** (AC: 3, 7, 8)
  - [x] `poll` applies capability records as it walks; `PollSummary.capabilities` is the last in force.
  - [x] **Recompute per poll; never write back onto `self`** (AC7).
  - [x] The 4.5a cancellation and failure shapes are unchanged. Do not revisit the no-post-loop-check decision (4.4 review, 2026-07-22).

- [x] **Task 4 — the fixture** (AC: 1, 4, 11)
  - [x] Author ONE new committed stream under `fixtures/scenario/replay/`: observations emitting a kind, a capability record dropping that kind, then observations that no longer emit it. `Rtt` is the natural choice — the corpus already exercises it, and a scanner losing raw sockets is exactly the NET_RAW case D34 §1 names.
  - [x] **Do not touch `minimal.jsonl` or `example-traps.jsonl`.**
  - [x] Add its `[[artefact]]` entry with sha256 to `fixtures/MANIFEST.toml`.

- [x] **Task 5 — the tests** (AC: 2, 3, 4, 6, 7, 8)
  - [x] AC4's two directions: the legal *before* case and the refused *after* case. Put the offending observation **second, behind a valid one** — the 4.4 review found three tests a `take(1)` mutant survived.
  - [x] AC2's two ordering rules, each proven to red.
  - [x] AC6: an upgrade accepted, an empty `kinds` accepted.
  - [x] AC7's idempotency-including-descriptor test.
  - [x] AC8: degraded-then-failed returns `Err` and no summary.
  - [x] **Prove-to-red on every new guard**, each mutation recorded. **Do not write a comment asserting a coverage property you did not measure** — the 4.4 review found five.

- [x] **Task 6 — the record, the docs, the gates** (AC: 9, 10, 12)
  - [x] Append the five entries of AC9 to `deferred-work.md`. **Append; do not rewrite or drop an existing bullet** — the 4.3 review caught exactly that, and two of these entries mark existing bullets CLOSED, which is precisely where an accidental rewrite happens.
  - [x] Rewrite the `fixture_connector.rs` module doc section of AC10.
  - [x] Update `sprint-status.yaml` and put it in the File List.
  - [x] Run the four gates. **Name the command behind every claim in the completion record.**

## Dev Notes

### The one thing that must not be built

**Do NOT derive `capabilities` from the observations** — not as a fallback, not as a convenience, not for a stream with no record. A capability read off what was seen cannot express *"capable of hostnames, saw none"*, the single distinction the descriptor exists for:

> "the engine must never confuse *'no `Uplink` because there is none'* with *'no `Uplink` because this connector is blind to topology'*. Without it, a rule like 'no diverging uplink → same switch → merge' merges happily. **Absence of proof ≠ proof of absence, encoded in the type.**" [architecture.md:1297-1300]

A stream with no capability record uses the constructor's initial descriptor. That is the answer; there is no derivation anywhere.

### Why this story exists at all, in D36's words

> "**A verdict without its capability snapshot is UNFALSIFIABLE: you cannot tell a regression from a legitimate re-derivation.**" … "**The trap is lexical: we require REPRODUCIBILITY, not STABILITY.** Replay `(data, capability)` → the same verdict, always. The verdict is allowed to change over time. Requiring stability means pinning capability, i.e. reintroducing the false merge. **Anyone who 'fixes this flake' by pinning the capability has broken the product to make CI green.**" [architecture.md:2063-2073]

The `(data, capability)` pair is only replayable if the capability is IN the data. That is this story in one sentence.

### What already exists — use it, do not rewrite it

- **4.5a's dispatcher and record type** — the marker key, the reader returning records, the `path`-family error variants. Extend; do not fork a second dispatch path.
- **`Capabilities` / `can_emit`** [Source: crates/opencmdb-core/src/observation/mod.rs:216-232] — `{ as_of: Timestamp, kinds: BTreeSet<FactKind> }`, already serde with `deny_unknown_fields`. `FactKind` is `Ord`, so the usual ordered collections apply (unlike `Scope`, which is `Copy + Hash + Eq` but NOT `Ord` and has no `Display`).
- **`FixtureConnector::from_observations`** [Source: crates/opencmdb-bin/src/fixture_connector.rs:93-152] — 4.4's four load invariants on one path. AC4 replaces the fact-kind one; the other three (foreign `connector_id`, uncovered scope, repeated `obs_id`) are untouched and must stay that way.
- **`FixtureError`** [Source: crates/opencmdb-bin/src/fixtures.rs:70-236] — `Display` and `source()` are exhaustive matches; the compiler points at both.
- **`polling_twice_replays_the_same_stream`** [Source: crates/opencmdb-bin/src/fixture_connector.rs:349] — asserts observations only. AC7 is what it does not cover.
- **`the_fixtures_path_is_expressed_once`** [Source: crates/opencmdb-bin/src/fixtures.rs:855] — exactly 2 occurrences. Reach the corpus through `fixture_path`.

### Traps

1. **Persisting the degraded descriptor onto `self`.** AC7. Passes once, wrong twice — and the trap runner polls more than once.
2. **Checking containment against the FINAL descriptor for the whole file.** AC4. It retroactively invalidates observations that were legitimate when they were made — the exact D13 bug D34 §1 names. A test covering only the refusal will not catch it; the *before* case is the one that does.
3. **Reaching for an error variant for the capability loss.** Decision 1.
4. **Deriving capabilities from what was seen.** See above.
5. **Rewriting a `deferred-work.md` bullet while marking it CLOSED.** AC9. This is the highest-risk register edit in the epic so far.
6. **Claiming more than was measured.** Four consecutive completion records over-claimed; 4.4's asserted a MariaDB was reachable — it was not, and the count was wrong. `DATABASE_URL` is unset here and the four DB-backed tests `return` early and pass either way, so a green `cargo test --workspace` says **nothing** about the database.
7. **Skipping `--all-targets` or `xtask ci` locally.** Epic 3's retrospective recorded four CI-only failures from exactly that.

### Privacy, frontier, testing standards

Identical to 4.5a and equally binding: synthetic values only (RFC 5737, locally-administered MACs, invented hostnames — a real capture in this public repo is *"disqualifying. Not debatable."* [architecture.md:1318-1319]); `opencmdb-core` may not name `anyhow`/`axum`/`sqlx`/`askama` and `serde` is not on that list; no `anyhow::Result` in this module despite `bin` depending on it (D33, unenforced by any gate); tests inline in `#[cfg(test)] mod tests` (no `tests/` directory exists anywhere); `#[tokio::test]` for polls; assert the CAUSE and the NAME, not merely that an error occurred; prove-to-red is the house rule.

### Latest technical specifics

No new crate, no version bump. **Locked** (from the committed `Cargo.lock`, verified 2026-07-22): `tokio 1.53.0`, `tokio-util 0.7.18`, `serde 1.0.228`, `serde_json 1.0.150`, `uuid 1.24.0`, `chrono 0.4.45` with `default-features = false` so `Utc::now()` is not callable from the domain. **Never invent a version, and do not mistake a caret requirement for a pin.** Rust 1.96+, edition 2024, `resolver = "3"` (`Cargo.toml:6`).

### Project Structure Notes

- **Updated:** `crates/opencmdb-bin/src/fixtures.rs` (the `capability` arm, two ordering rules, new `origin`-family variants), `crates/opencmdb-bin/src/fixture_connector.rs` (positional containment, descriptor in force, module doc rewrite), `fixtures/MANIFEST.toml`, `fixtures/scenario/README.md`, `deferred-work.md`, `sprint-status.yaml`.
- **New:** one `.jsonl` under `fixtures/scenario/replay/`.
- **Unchanged, expected:** `minimal.jsonl`, `example-traps.jsonl` and their sha256 entries, `crates/opencmdb-core/` (this story adds nothing to the domain — `Capabilities` already serializes), `crates/opencmdb-bin/src/main.rs`.
- **`Cargo.lock` should not move.**

### References

- [Source: _bmad-output/planning-artifacts/epics.md:936-952 — Story 4.5, which 4.5a and 4.5b jointly implement; :948-950 is this story's criterion]
- [Source: _bmad-output/planning-artifacts/epics.md:954-974 — Story 4.6: `capability_snapshot` on every scored record, or the verdict is unfalsifiable (D36)]
- [Source: _bmad-output/planning-artifacts/architecture.md:1921-1958 — D34 §1 in full: the descriptor is a dated fact not a constant; the poll is the authority; the D13 latent bug (:1944-1947); the false-merge framing (:1955-1958)]
- [Source: _bmad-output/planning-artifacts/architecture.md:1927-1930 — "one JSONL line reproduces a mid-scan NET_RAW loss" — this story's mandate, verbatim]
- [Source: _bmad-output/planning-artifacts/architecture.md:1913-1914 — D33: `CapabilityLost` is an event, not a state; ping-only is an `Ok` with a reduced descriptor]
- [Source: _bmad-output/planning-artifacts/architecture.md:1297-1300 — D19: `capabilities()` is false-merge prevention, not decoration]
- [Source: _bmad-output/planning-artifacts/architecture.md:2057-2076 — D36: reproducibility not stability; a verdict without its snapshot is unfalsifiable; lattice monotonicity]
- [Source: _bmad-output/planning-artifacts/prd.md:848 — FR5: liveness and capability are two independent axes; downgrades are notifiable events]
- [Source: _bmad-output/planning-artifacts/prd.md:883 — FR19: a capability reduction suppresses divergences only on the fields that capability covered, not globally]
- [Source: crates/opencmdb-core/src/observation/mod.rs:216-232 — `Capabilities`, `can_emit`, already serde with `deny_unknown_fields`]
- [Source: crates/opencmdb-core/src/connector/mod.rs:72-75 — `is_blinding`: why a capability loss may not be an error]
- [Source: crates/opencmdb-bin/src/fixture_connector.rs:19-41 — the module doc AC10 rewrites; :93-152 — 4.4's four load invariants; :134-143 — the AC7b containment AC4 supersedes; :349 — the idempotency test AC7 extends]
- [Source: crates/opencmdb-bin/src/fixtures.rs:65-69 — `FixtureError`'s two families and when to use which]
- [Source: _bmad-output/implementation-artifacts/4-5a-fixture-connector-replays-failures.md — the dispatcher and record type this story extends]
- [Source: _bmad-output/implementation-artifacts/4-4-fixture-connector-replays-jsonl.md:54-61 — AC8, the recorded finding this story closes; :117, :127 — the review lessons carried forward]
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:26-29 — the four story-4.4 entries; two are closed here, two are not]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.8 — `claude-opus-4-8[1m]`

### Debug Log References

Every claim below is a command that was run, not an inference.

- **Five guards proven to red**, each mutation applied to a copy-backed tree and reverted after:
  | Mutation | Tests reddened |
  |---|---|
  | containment checks the stream's FINAL descriptor instead of the positional one | **8** — every 4.5b test, including `a_fact_emitted_before_the_descriptor_dropped_it_stays_legal`: under a final-descriptor implementation **the committed fixture does not even load** |
  | the in-force descriptor written back onto `self` | `polling_twice_yields_the_same_final_descriptor` (only) |
  | the "predates a preceding observation" rule removed | `a_capability_record_predating_a_preceding_observation_is_refused` |
  | the "non-decreasing `as_of`" rule removed | `capability_records_going_backwards_in_time_are_refused` |
  | `poll` ignores capability records | `a_capability_record_changes_the_descriptor_and_the_poll_continues`, `polling_twice_yields_the_same_final_descriptor` |

  _A first attempt at the containment mutation was discarded rather than reported: its probe helper
  had a nonsense fallback that reddened 26 tests and isolated nothing. The table above is the clean
  re-run._

- **Two of my own tests were wrong before the code was**, and both are recorded in the code:
  - the descriptor was named via chrono's `Display` (`2026-03-01 00:00:07 UTC`) while the file writes
    RFC 3339. **The code was changed, not the assertion** — naming a descriptor in a spelling the
    author cannot grep for is a defect, not a test detail.
  - the out-of-order test appended its second record at the end of the stream, where the *predates*
    rule fires first (measured: it got `CapabilityPredatesObservation`). The record is now inserted
    before the late observations, and the interaction is written into the test and the register.

- **The compiler enforced the privacy decision.** Adding `Record::Capability` broke four exhaustive
  `match`es, including the corpus privacy walk made exhaustive during the 4.5a review. Each was
  given a deliberate arm; the privacy arm states that a capability record carries a timestamp and a
  set of enum values and has no free text to scan.

- **Gates:** `cargo fmt --all` clean · `cargo clippy --workspace --all-targets -- -D warnings` clean
  · `cargo test --workspace` → **86 (bin) + 46 (core) + 38 (xtask), 0 failed** (bin gained 9).
- `cargo xtask ci` → all gates green, `✅ fixtures 5/5 (0 generated, 5 hand-authored)`;
  `architecture-views.md` NOT regenerated.
- **`Cargo.lock` is unchanged by this story** — `git diff --stat Cargo.lock` still shows the single
  `serde` line added by 4.5a, and nothing more.
- **The MariaDB-backed tests did NOT run.** `DATABASE_URL` is unset; the four DB-backed tests
  `return` early and pass either way. "86 passed" is zero evidence about the database.

### Completion Notes List

- **The descriptor now travels with the batch, dated by the file.** One JSONL line reproduces a
  mid-scan NET_RAW loss — D34 §1's own words, now literally what the corpus does. The constructor's
  `capabilities` became the INITIAL descriptor rather than the descriptor for the whole stream.
- **A capability change is `Ok`, never an error.** D33: *"`CapabilityLost` is an event, not a state
  — ping-only is an `Ok` with a reduced descriptor."* A source that lost NET_RAW is still `Live`;
  every `ConnectorError` but `Cancelled` blinds, and blinding a live source is the false-"gone"
  NFR7 exists to make impossible.
- **Containment became positional, and that supersedes 4.4's global rule rather than dropping it.**
  Each observation is checked against the descriptor in force at its own position. The legal
  direction is the one that matters: a fact emitted BEFORE a downgrade stays legal, because
  otherwise *"the past would change status"* (D34 §1). Measured: under a final-descriptor
  implementation the committed fixture fails to load.
- **An upgrade is legal and an empty `kinds` set is legal**, both tested and both explained in the
  module doc — a reader must not mistake either for a bug.
- **`as_of` is honest, by two rules invented here on purpose.** 4.4 deliberately refused to invent a
  validation policy under implementation pressure; this is the story that owns the record, so the
  rules were written here: no record may predate an observation before it, and successive records
  may not go backwards.
- **Variant names, as the story allowed:** `Record::Capability` · `ControlRecord::Capability` with
  `#[serde(flatten)]` so the line IS a `Capabilities` plus the marker ·
  `FixtureError::CapabilityPredatesObservation` · `FixtureError::CapabilityOutOfOrder`, both in the
  `origin` family. `UndeclaredFactKind` gained a `descriptor: String` field so the message can name
  WHICH descriptor denied the fact — a capability record has no `obs_id`, and story 4.2 forbids
  naming anything by line number.
- **What this story did NOT close, recorded in five register entries**: `scopes_covered` is still
  constructor-supplied and assigned to nobody; `PollSummary` still exists only on the `Ok` path, so a
  degraded-then-failed poll reports no descriptor — but 4.6 can now reconstruct the snapshot from
  the file without touching the trait, and that is this story's real contribution to 4.6.

### File List

- `crates/opencmdb-bin/src/fixtures.rs` (modified — `Record::Capability`, `ControlRecord::Capability`,
  2 `FixtureError` variants, `descriptor` field on `UndeclaredFactKind`, module doc)
- `crates/opencmdb-bin/src/fixture_connector.rs` (modified — positional containment, the two `as_of`
  rules, per-poll descriptor, `describe`, module doc; 9 tests)
- `fixtures/scenario/replay/capability-downgrade.jsonl` (new — 2 observations emitting `Rtt`, a
  record dropping it, 2 observations that do not)
- `fixtures/MANIFEST.toml` (modified — the new artefact and its sha256)
- `fixtures/scenario/README.md` (modified — the second control-record kind and its rules)
- `_bmad-output/implementation-artifacts/deferred-work.md` (modified — five story-4.5b entries
  appended; two mark existing bullets CLOSED without editing them)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modified)
- `Cargo.lock` — **unchanged by this story**, measured.

### Change Log

- 2026-07-22 — The capability descriptor moves into the file. A `capability` control record carries a
  dated `Capabilities`; the poll continues and returns `Ok` with the descriptor in force at the end.
  Fact-kind containment became positional, so a fact emitted before a downgrade stays legal — the
  past does not change status (D34 §1). Two load-time rules keep `as_of` honest. An upgrade and an
  empty descriptor are both legal, both tested. The descriptor is recomputed per poll and never
  written back onto the connector. Five guards proven to red; story 4.4's global containment
  superseded rather than dropped, and recorded as such.
