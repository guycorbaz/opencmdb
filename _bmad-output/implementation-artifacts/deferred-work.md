# Deferred Work

## Deferred from: code review of story-1.1 (2026-07-19)

- **Frontier gate — forbidden dep invisible under optional / non-default-feature / cfg-target / build-dependency edges.** `gate_dependency_frontier` runs `cargo tree -p <pkg> -e normal --locked`, which resolves only the default feature set, the host target, and normal edges. A banned crate (`anyhow`/`axum`/`sqlx`/`askama`) declared in `opencmdb-core` as `optional = true`, behind a non-default feature, under `[target.'cfg(...)'.dependencies]`, or as a `[build-dependencies]` entry, is absent from that graph — so the gate stays GREEN while the manifest genuinely names the forbidden crate. This is a latent false-negative with zero impact today (core declares no features, no cfg-target deps, no `build.rs`), but it is the reflex gate's (D53) assumed boundary. Fully closing it needs a feature-matrix approach (`--all-features` alone risks false positives via workspace feature unification). **Tracked as GitHub issue [#2](https://github.com/guycorbaz/opencmdb/issues/2)** — revisit before the gate is relied upon for a core crate that grows optional/feature-gated deps.

## Deferred from: code review of story-1.2 (2026-07-19)

- **Fixtures gate — untracked fixture on disk but absent from `fixtures/MANIFEST` is never checked.** `gate_fixture_manifest` iterates only the MANIFEST entries; nothing walks `fixtures/` to find files present on disk yet unlisted. So a new/modified fixture that isn't added to the MANIFEST passes CI unnoticed — drift in the *add* direction, the mirror of the sha-mismatch case the gate does catch. This is inherent to the scaffold and is **Epic 4 scope**: D56's real `fixtures/scenario/replay/MANIFEST.toml` enumerates every artefact and the `recapture` tool owns the corpus, so orphan-fixture detection lands naturally there. No separate GitHub issue — it is core Epic 4 work (epics.md Epic 4 "Infra fixtures & corpus de pièges"), not a gap in an otherwise-complete feature.

## Deferred from: code review of story-4.1 (2026-07-21)

- **Non-UTF-8 bytes pass the sha256 gate but fail the reader.** `gate_fixture_manifest` hashes raw bytes (`xtask/src/main.rs:504`) while `read_jsonl` uses `read_to_string`, so `cargo xtask ci` can report a fixture as locked and unchanged while the reader cannot open it — and the resulting `FixtureError::Io` carries no byte offset. A UTF-8 BOM is worse: it survives `trim()`, so the failure is reported as a JSON authoring error on line 1 rather than a byte-level defect. **Still open after story 4.3.** The gate was reworked into `MANIFEST.toml`, but NO encoding validation was added: a non-UTF-8 *path* is now a finding, a non-UTF-8 *payload* still passes the sha check and fails the reader. Re-deferred, deliberately.
- ✅ **CLOSED by story 4.3.** ~~Orphan fixtures remain undetected.~~ A file present under `fixtures/` but absent from the MANIFEST is never checked; the gate's real guarantee is "listed files are unchanged", not "the corpus is frozen". Confirmed independently by two review layers. **Already story 4.3** (and originally deferred from the story-1.2 review).
- **`Fact::Mac.locally_administered` is denormalized.** The flag is derivable from bit 1 of octet 0, yet stored alongside the address, so a committed fixture line can assert a value that contradicts its own MAC and nothing compares the two. Pre-existing Epic 2 domain design, not introduced by 4.1 — but it matters more now that fixtures are an oracle. Revisit when the trap families land (4.9+).
- ⚠️ **PARTLY CLOSED by story 4.3.** Duplicate `obs_id`s within a stream are now refused, and duplicate MANIFEST paths are now a finding. **Still open:** collecting all validation errors instead of stopping at the first (deferred to 4.7). ✅ **Duplicate `obs_id`s ACROSS two streams is now CLOSED** (2026-07-23): `no_obs_id_is_shared_across_replay_streams` walks the corpus and refuses an id in two files, naming both — proven to red against a reintroduced collision. (A real one existed until 2026-07-22 in `partial-then-failed.jsonl`.) _(An earlier edit of this bullet enumerated what remained open and silently dropped the duplicate-path half — caught by the 4.3 review. A register that loses an item is worse than no register.)_ ~~The reader stops at the first bad line, and duplicates are invisible.~~ Five broken lines need five edit-run cycles; nothing rejects two lines sharing an `obs_id`, and the MANIFEST parser accepts the same path listed twice. Corpus-integrity work — belongs with 4.2/4.3.
- ✅ **CLOSED by story 4.3** — TOML quotes the value, so a space in a path is expressible. ~~A fixture filename containing a space can never be locked.~~ `parse_manifest` splits on whitespace and requires exactly two tokens (`xtask/src/main.rs:471`), so such a file is `Malformed` → gate RED, while the reader loads it happily. The reader accepts a namespace the lock cannot express. Pre-existing.
- **No size cap on a fixture or on a single line.** `read_to_string` loads the whole file and `serde_json` parses an arbitrarily long line, so a multi-GB artefact is an OOM rather than a diagnosable error. Not reachable at corpus scale; revisit if `capture/` ever holds real payloads.

## Deferred from: story-4.4 (2026-07-21)

_Raised BY the story, not by its review — hence no "code review of" in the heading. All four are
consequences of one fact: the fixture format frozen by 4.1 is a stream of `Observation`s and
nothing else, so a poll's ENVELOPE has nowhere to live in the file._

- **`capabilities` and `scopes_covered` are supplied at construction, not read from the file.** D34 §1 argues the opposite — that the descriptor should travel with the batch, precisely because *"the fixture replays it for free — one JSONL line reproduces a mid-scan NET_RAW loss, zero mocks; with a separate getter the fixture would need state outside the JSONL"*. Constructor parameters ARE that state outside the JSONL. Deriving them from the observations was refused as strictly worse: a capability read off what was seen cannot express *"capable of hostnames, saw none"*, the one distinction the descriptor exists for. **Owner: story 4.5** for the capability half (its AC is literally the mid-scan capability loss, and epics.md's words for the shape are *"one JSONL line"* — not an envelope, which is nobody's design yet). `scopes_covered` follows by extension; epics.md assigns it to no story, so it is **not yet assigned**. Mitigated meanwhile: construction refuses a stream that CONTRADICTS the declared values (uncovered scope, undeclared fact kind) — containment only, never derivation.
- **`Capabilities.as_of` is unrelated to the file's `observed_at`.** It is caller-supplied while every observation is dated by the file, so a replay can date its capability descriptor in a moment its own stream contradicts — and D34 §1's whole content is that the descriptor is *a dated fact, not a constant*, with 4.5's downgrade traps being diffs over exactly that field. The cheap guard available today is `capabilities.as_of >= max(observed_at)` on a clean replay. **Deliberately NOT imposed by 4.4**: inventing a validation policy under implementation pressure is what the 4.3 review sanctioned, and 4.5 puts the record in the file and can then date the descriptor FROM the file, which is the real fix rather than a rule bolted on.
- **`PollSummary` exists only on the `Ok` path, and story 4.6 needs it on every path.** `Connector::poll` returns `Result<PollSummary, ConnectorError>`, so a cancelled poll (4.4) or a partial-then-failed poll (4.5) carries no `capabilities` at all — while 4.6 requires every scored record to carry a `capability_snapshot`, *"a verdict without its capability snapshot is unfalsifiable"* (D36). This is the epic's own clause in its stated direction: **the trait cannot express what the fixture needs.** Not changed here, deliberately — the trait is Epic 2's and a poll's error path is 4.5's. Flagged so 4.5/4.6 meet it as a known question instead of rediscovering it.
- **AC7 (a clean poll may not claim to cover less than it observed) is bound to the CLEAN replay.** Story 4.5 introduces partial-then-failed polls, where a scope can be legitimately observed and NOT covered — the poll died before finishing it. **4.5 is free to relax this check, or move it off the load path onto the clean-poll path, and that is not a regression.** Recorded here so the loosening reads as the design it is rather than as a guarantee quietly dropped.

## Deferred from: code review of story-4.4 (2026-07-22)

- **`fixture_path`'s corpus containment is LEXICAL, so a symlink escapes it.** `fixture_path` (`crates/opencmdb-bin/src/fixtures.rs:42`) inspects only `Component::ParentDir`/`CurDir`/`is_absolute`; a `Normal` component naming a symlink passes, and `read_jsonl`'s `read_to_string` follows it. A link committed at `fixtures/scenario/replay/evil.jsonl → /etc/…` is read from outside the corpus while `load_refuses_a_path_leaving_the_corpus` stays green. The corpus's own symlink guard (the trap-discovery walk) covers `scenario/traps/` only, never `scenario/replay/`. **Pre-existing** (story 4.1), but story 4.4 makes it reachable from exactly the threat the containment comment names — *"any future connector taking a fixture name from configuration would read arbitrary files"*. The fix is a `canonicalize()` + `starts_with(fixtures_dir().canonicalize()?)` check; it needs a decision about whether the corpus may ever legitimately contain a link, which is really a `capture/` question.
- **`FixtureConnector::poll` has no `.await`, so `ConnectorError::Timeout` is unreachable for it and a slow sink blocks the runtime worker.** The whole emit loop runs inside one `Future::poll`, so `tokio::time::timeout` around a replay is a silent no-op and D34 §2's "a timed-out poll keeps what it emitted" case degenerates to all-or-nothing for this connector. **Deliberate**: story 4.4's AC5b forbids adding a yield point ("an await in a zero-I/O replay is a defect, not a fix"), and `opencmdb-core`'s `YieldingConnector` exists precisely to cover the timeout case against a connector that does yield. Recorded because the CONSEQUENCE was never stated: whoever builds the scheduler's per-scope time budget (D34) must know that a fixture replay cannot be interrupted by it.

## Deferred from: story-4.5a (2026-07-22)

_Raised BY the story, not by its review. Story 4.5a put the poll's FAILURE in the file; the
capability half is story 4.5b's, so the entries below say what moved and what did not._

- **`PollSummary` still exists only on the `Ok` path, and story 4.6 still needs it on every path.**
  Unchanged in substance from the story-4.4 entry above — but no longer hypothetical: a stream can
  now script a failure, so a poll that emits four observations and then fails is a real,
  committed artefact (`fixtures/scenario/replay/partial-then-failed.jsonl`), and it carries no
  `capabilities` at all. 4.6 requires a `capability_snapshot` on every scored record — *"a verdict
  without its capability snapshot is unfalsifiable"* (D36). **Not fixed here, deliberately**: the
  trait is Epic 2's, and 4.5b changes the shape of the problem by putting the descriptor in the
  file, from where 4.6 can read it without touching the trait.
- **`ConnectorError::Timeout` is now reachable for `FixtureConnector`, and the 4.4-review finding
  above still stands.** A scripted `Timeout` is how a fixture PRESENTS one, which is what D35's
  layer-A list ("401, timeout, partial") asks for. The open item is a different thing and is
  unchanged: `poll` has no `.await`, so the scheduler's per-scope budget still cannot interrupt a
  replay, and `tokio::time::timeout` around one is still a silent no-op.
- **4.4's AC7 (`UncoveredScope`) did NOT need relaxing, and was left intact.** `deferred-work.md`
  pre-authorised loosening it for partial polls. The reason it was unnecessary: `UncoveredScope` is
  a **load-time** invariant, evaluated over every observation in the stream before any poll happens,
  and `from_records` still validates every observation's scope whether or not the stream carries a
  failure record. Whether the later poll returns `Err` has no bearing on it. The permission was
  granted and deliberately not spent. _(An earlier version of this bullet said "measured instead of
  assumed" and then gave a deduction — about the poll returning no `PollSummary` — which is true but
  irrelevant to a load-time check. Corrected by the story-4.5a code review; no measurement was ever
  run, and none was needed.)_
- **`capabilities` and `scopes_covered` are still constructor-supplied.** 4.5a moved the poll's
  outcome into the file and nothing else. The capability half is **story 4.5b**; `scopes_covered`
  remains assigned to nobody, exactly as the story-4.4 entry above records.
- **A stream may not script `Cancelled`, and the refusal is duplicated on two paths.**
  `read_records` refuses it naming the line (`CancellationScripted`); `from_records` refuses it
  naming the origin (`CancellationInStream`). This mirrors the existing
  `DuplicateObservationId`/`RepeatedObservationId` pair and carries the same cost: **a caller that
  wants to handle "this stream mints a cancellation" must match BOTH variants.** Collapsing the two
  families into one error type is a wider refactor than this story, and is not proposed lightly —
  the `path`-vs-`origin` split is load-bearing (story 4.4).
- **"Nothing may follow a terminal failure" is enforced on the FILE path only, and that asymmetry
  is deliberate.** Its whole rationale is `read_traps`' cross-check against a committed file. An
  in-memory stream is judged by no trap file, and a caller must be able to build exactly that shape
  to prove a faulted replay emits a strict PREFIX of the clean one (D35(a)) — enforcing it in
  `from_records` would forbid the test that proves the story's own criterion. If a future story
  gives in-memory streams a trap-like consumer, this needs revisiting.

## Deferred from: code review of story-4.5a (2026-07-22)

- **`Observation.raw` is inspected by no privacy rule, and two fact fields are silently exempted.**
  `assert_facts_are_synthetic` (`crates/opencmdb-bin/src/fixtures.rs:701`) takes `&[Fact]`, so `raw`
  — documented as *"the source's original payload as text"*, and the single most likely place a real
  capture reaches a public repo — is never seen; `minimal.jsonl` already ships a non-null one. In the
  same match, `Fact::OuiVendor { .. }` drops `vendor` unchecked and `Fact::Uplink { peer_mac, .. }`
  checks the MAC but never `peer_port` (a real interface name such as `Gi1/0/24`). **Pre-existing
  since story 4.1**, but story 4.5a rewrote that walk on the argument that *"a privacy rule that
  cannot see the file it governs is not a rule"*, which makes the omission load-bearing rather than
  incidental. **Deferred rather than patched on purpose**: asserting anything about an opaque JSON
  blob requires a stated policy, and inventing a validation policy under implementation pressure is
  exactly what the 4.3 review sanctioned.
- **Every unit `ConnectorError` variant has TWO accepted on-disk spellings, and only one is pinned.**
  `serde_json` accepts both `"Timeout"` and `{"Timeout":null}` for an externally tagged unit variant
  (verified by probe). `every_variant_round_trips_through_json`
  (`crates/opencmdb-core/src/connector/mod.rs:255`) pins only the SERIALIZER's output, so the second
  spelling is admissible in a committed file with nothing saying so. In a corpus whose stated
  premise is that the bytes ARE the spec, two files can express one scripted outcome two ways and
  the sha256 lock freezes whichever the author typed. Refusing the second needs a hand-written
  `Deserialize`; decide it with **story 4.5b**, which adds the second record kind and would double
  the surface.
- **The committed `partial-then-failed.jsonl` is judged by no trap, and its outcome cannot be
  expressed as one.** `Trap::validate` requires at least one `obs_id`
  (`crates/opencmdb-core/src/trap.rs:254`) and a failure record has none, so the truth format frozen
  by story 4.2 has no way to say *"this poll ends `Unreachable`"*. The stream is hashed by the lock
  and parsed by the corpus walk, but nothing asserts what it MEANS. This is story 4.5a's own
  argument in reverse — *"a trap that can never fire would sit in the corpus looking like coverage,
  and the gate counts traps"*. Belongs with **4.7**, the trap runner.
- **A UTF-8 BOM is reported as a JSON syntax error at line 1 column 1.** `read_to_string`
  (`crates/opencmdb-bin/src/fixtures.rs:394`) keeps U+FEFF, so a valid first line fails with
  `expected value at line 1 column 1` and the author is sent looking for a syntax error that does not
  exist. **Pre-existing** and adjacent to the non-UTF-8 entry already in this register; re-recorded
  because hand-editing the new control-record line is now the likeliest way a BOM enters the corpus.
- **`Serialize` was derived on `ConnectorError` with no production consumer.** Fixtures only ever
  READ one; the only serializing caller is the round-trip test serializing in order to deserialize
  its own output. It adds a permanent wire-format obligation to the public API of the pure domain
  crate for a test-shaped need, and the two pinned-shape assertions make the enum's JSON rendering a
  compatibility surface the corpus lock also depends on. Revisit if the AC6 guardrail decision
  removes the need for the round-trip.

## Deferred from: story-4.5b (2026-07-22)

_Story 4.5b put the capability descriptor in the file. Two entries above are CLOSED by it, two are
not, and one guarantee changed shape. Stated against the existing bullets without editing them._

- ✅ **CLOSED — `capabilities` is no longer state outside the JSONL** (the capability half of the
  story-4.4 entry above). A `capability` control record carries the full `Capabilities`, and the
  constructor now supplies only the INITIAL descriptor — the one in force before any record. D34 §1's
  *"the fixture replays it for free — one JSONL line reproduces a mid-scan NET_RAW loss, zero mocks"*
  is now literally what the corpus does (`fixtures/scenario/replay/capability-downgrade.jsonl`).
- ✅ **CLOSED — `Capabilities.as_of` is now dated by the file.** The record carries its own `as_of`,
  and two load-time rules keep it honest: it may not predate any observation before it in the stream,
  and successive records may not go backwards. This is the entry that named story 4.5 as its "real
  fix rather than a rule bolted on"; the rules were invented here, in the story that owns the record,
  which is what the 4.3 review asked for. **Note the two rules interact**: a record appended after a
  late observation trips the *predates* rule before the *out-of-order* rule can fire — measured while
  writing the test, and recorded in the test itself.
- ⚠️ **STILL OPEN — `scopes_covered` is constructor-supplied and assigned to nobody.** 4.5b moved the
  capability half only. epics.md gives it to no story; it remains "not yet assigned" exactly as the
  story-4.4 entry says.
- ⚠️ **STILL OPEN, now demonstrable, and its SHAPE has changed — `PollSummary` on the error path.**
  A poll that degrades and then fails is now a real, tested case
  (`a_degraded_then_failed_poll_reports_no_descriptor_at_all`): it returns `Err`, so it carries no
  `PollSummary` and therefore no descriptor, while story 4.6 requires a `capability_snapshot` on
  every scored record — *"a verdict without its capability snapshot is unfalsifiable"* (D36).
  **What changed for 4.6: the descriptor is now readable from the FILE.** 4.6 can reconstruct the
  snapshot by walking the records itself, without changing `Connector::poll`. That is this story's
  actual contribution to 4.6, and it is why the trait was not touched.
- **NEW — story 4.4's global containment (its AC7b) is SUPERSEDED by positional containment, not
  dropped.** 4.4 proved to red that the file may not exceed the constructor's declaration. Once the
  descriptor comes from the file, the file is the authority (D34 §1: *"the connector is no longer the
  authority — the poll is"*), so a stream may now declare kinds the constructor never did. What
  replaces it is stronger where it counts: each observation is checked against the descriptor in
  force AT ITS OWN POSITION, so emitting a fact kind you just declared yourself blind to is
  impossible — which the global check could not express at all. Recorded because a reviewer reading
  only the diff sees a proved-to-red guarantee apparently deleted.

## Deferred from: story-4.6a (2026-07-22)

- ✅ **CLOSED by story 5.3**, which is the owner this bullet names. The branch taken is the SECOND
  of the two it offers: **a separate cause type**, `identity::cascade::IdentityAbstentionCause`
  (`Ambiguous` | `AbsenceOfProof`, each traced to a row of D13's table), carried by
  `Outcome::Abstained`. `AbstentionCause`'s variant list is byte-unchanged and
  `Expectation::MustAbstain` still carries it, so no committed artefact was re-hashed.
  ~~`AbstentionCause` cannot express the identity cascade's `Ambiguous`, and Epic 5 must decide.~~
  ⚠️ **TWO sentences below are FALSE. They are struck in place AND corrected here** — striking alone
  loses the reason, and correcting alone leaves a reader who lands on the sentence in isolation
  reading a false one. (This note said "One sentence" and struck nothing until the code review of
  2026-07-29 measured the second and caught the omission.)
  **(a)** A different type on the outcome side *"would make comparison asymmetric against a
  locked format"*. There is no comparison to go asymmetric: `score`'s 3×3 matches
  `Outcome::Abstained { .. }` and cannot reach the payload, and `run_trap` compares rules only where
  both sides are `Some`, which an abstention never is. The tests that hold it are
  `the_two_abstention_vocabularies_are_never_compared` and
  `scoring_is_blind_to_the_abstention_cause_whatever_it_is` (`score.rs`), both proven red by
  flipping the `(must-abstain, Abstained)` cell. The *cost* half of the sentence was real and was
  measured the other way round: adding `Ambiguous` to `AbstentionCause` produces exactly one
  `error[E0004]`, at `page.rs:114:11`, and nothing else in the workspace breaks — so widening buys a
  user-facing label and two locale strings for a variant `reconcile` can never produce.
  **(b)** *"`reconcile` matches on it exhaustively"*. It does not: `reconcile` only ever
  CONSTRUCTS a cause (`gap/mod.rs`, three `abstain(...)` calls). The sole value-level `match` on
  `AbstentionCause` in the workspace is `page.rs`'s `cause_label`, which is what M4 measured —
  widening the enum yields one `error[E0004]`, there and nowhere else. The claim was inherited
  verbatim into story 5.3's AC1 and is corrected in that story's Completion Notes too.
  It is the RECONCILIATION vocabulary (`OutOfPerimeter | NoObservedValue | ConflictingObservations`,
  story 3.6). The cascade's abstention arises from the verdict algebra — the cloned-MAC case — and
  none of the three names it. 4.6a uses it on BOTH sides anyway, because story 4.2 froze the truth
  format on it and the committed corpus already writes `cause = "NoObservedValue"`; ~~a different type
  on the outcome side would make comparison asymmetric against a locked format~~ (a). **Not widened
  here**: ~~`reconcile` matches on it exhaustively~~ (b) and there is no producer yet. Epic 5 builds
  the cascade and chooses — widen the enum, or give `Outcome::Abstained` its own cause type.
- **`fixture_seq` is NOT implemented, and D36's five-field list is therefore not fully satisfied.**
  It occurs exactly once in `architecture.md` (inside D36's list), zero times in the PRD, zero in
  code — no type, no shape, no prose. The obvious reading, an ordinal into the stream, contradicts a
  locked decision: stories 4.1/4.2 chose `obs_id` *because* a line number *"would silently shift
  under the truth"*. `ScoredRecord` instead carries `trap: TrapId` + `replay: String`, the names the
  corpus already froze. Recorded as a deliberate substitution so a reviewer comparing against D36
  sees a decision rather than an omission.
- **`(TrapId, replay)` is not a globally unique key.** `TrapError::DuplicateId` is enforced per FILE
  — *"two traps in the same file share an id"* — so at ~50 traps across many files (4.9+), two files
  could both define `mac-randomized-01` against the same stream. The record's key is **provisional**.
  A cross-file `TrapId` guard belongs with the corpus-hygiene work, beside the cross-stream `obs_id`
  guard that is still outstanding.
- **`source_state` is `Option<SourceState>` where `SourceState` is UNINHABITED, until Epic 13.**
  The field is provably `None` — witnessed by `size_of::<Option<SourceState>>() == 0`, not by an
  `is_none()` assertion, which would pass for any inhabited type. ⚠️ **What survives Epic 13 is the
  field's name and its `Option`-ness, not this type**: D32's `SourceState` is a STRUCT
  (`{ liveness, capabilities }`), so Epic 13 will REPLACE the placeholder, not add variants to it.
  (An earlier draft of the story claimed the opposite; corrected before implementation.)
- **The complete verdict vector has no producer, and the field is provably empty.**
  `architecture.md`'s *"the harness records… the COMPLETE VERDICT VECTOR… the anti-drift is not
  discipline, it is a data requirement"* is a requirement on the harness that D36's five-field list
  omits. Its element is `(rule, verdict, evidence)` and none exists before Epic 5, so
  `VerdictVectorEntry` is uninhabited by the same standard as `SourceState` rather than the field
  being empty by comment.

## Deferred from: code review of story-4.6a (2026-07-22)

- **`Tally::record` takes no `TrapId`, so one trap can be scored twice and inflate the gate.**
  Probed: two identical `record` calls give `scored = 2, failures = 2`. Reachable, not theoretical —
  `TrapError::DuplicateId` is enforced PER FILE, so `mac-randomized-01` defined in two corpus files
  is legal today. **Owner: story 4.6b**, which owns the join between records and the tally and is the
  first real producer.
- **`ScoredRecord`'s `reason`, `replay` and `trap` are unvalidated `String`s that bypass
  `Trap::validate`'s contract.** The corpus refuses an empty, multi-line or >300-character reason
  (`REASON_MIN_CHARS = 20`, `trap.rs`); the record accepts all of them — every field is `pub`, there
  is no constructor. In practice the harness will build records from an already-validated `Trap`, so
  the value arrives validated; nothing enforces it. A constructor (and `#[non_exhaustive]`, below)
  belongs with **4.6b**.
- **`ScoredRecord` is not `#[non_exhaustive]` although it is designed to change shape.**
  `fixture_seq` may return, `SourceState` is replaced by Epic 13, `VerdictVectorEntry` gains a real
  element type. Every field is `pub` with no constructor, so each struct literal is a
  breaking-change site. Pairs with the constructor above.
- **The `size_of` uninhabitedness witness rests on a layout OPTIMISATION, not a language guarantee.**
  The Reference specifies `Option<T>`'s layout only for the null-pointer cases; an `Option` of an
  uninhabited type collapsing to zero bytes is the compiler's choice. Verified on rustc 1.97.1, and
  verified benign in both directions: replacing `SourceState` with D32's struct still compiles
  everywhere and fails **usefully** (`left: 48, right: 0`), and deriving serde later does not break
  on an uninhabited field. Recorded so a future rustc layout change is diagnosed as what it is
  rather than as a semantic regression.
- ↺ **PARTLY closed by story 5.3 — this annotation belongs to the entry BELOW** (*"The cascade's
  `NoMatch` maps two ways onto `Outcome`"*), **not to the `size_of` entry above it**, **and is
  deliberately NOT struck.** 5.3 created the cause that
  absence-of-proof will carry (`IdentityAbstentionCause::AbsenceOfProof`, traced to
  architecture.md:974's row) and wrote its variant doc around this very entry. **The MAPPING still
  has no producer**: nothing decides what the cascade returns, because there is no cascade. ~~Owner
  stays **stories 5.4/5.5** — the `Decision` type and the L1 join.~~ Striking this would claim a
  behaviour that exists nowhere.
  ↺ **Owner UPDATED 2026-07-29 by story 5.4: stories 5.4b/5.5.** The `Decision` type now exists
  (`identity/cascade.rs`) and its `Conclusion` carries both halves of `NoMatch` as distinct variants
  — a refusal that names a rule, and an abstention that cannot. **The mapping still has no
  producer.** Story 5.4b writes the function that chooses. Still not struck.
  ↺ **CLOSED IN PART by story 5.4b, 2026-07-29 — the CHOOSING now exists.**
  `identity::cascade::decide` implements D13's table and picks the side: a `Disqualifying` present
  lands on `NoMatch { rule }` naming the smallest qualifying rule; its absence, with nothing arguing
  FOR the pair, lands on `Abstained { AbsenceOfProof }`. All 32 verdict subsets are exercised against
  an oracle written from D13 itself. ⚠️ **What is still open is the mapping onto `Outcome`** — nothing
  converts a `Decision` into what the trap harness records, and no rule produces a `Verdict`. **Owner
  of the remaining half: story 5.7** (the trap runner consuming a real engine). Not struck.
  ✅ **CLOSED by story 5.7, 2026-08-01.** `score::outcome_of(&Decision) -> Outcome` exists: an
  exhaustive match with no `_` arm over `Conclusion`'s three variants
  (`Match { rule } -> Merged { rule }`, `NoMatch { rule } -> Refused { rule }`,
  `Abstained { cause } -> Abstained { cause }`, the same `IdentityAbstentionCause` on both sides).
  Five tests pin it, including `outcome_of(&d).rule() == d.rule()` on every row — the mirror
  `run_trap` depends on. `verdict_vector` and `ruleset_version` are DROPPED, stated at the function
  and asserted by a test; that residue is the `VerdictVectorEntry` entry below, not this one.
  **Struck.**
- **The cascade's `NoMatch` maps two ways onto `Outcome`, and only half of that is recorded.**
  `architecture.md:967-974` makes `NoMatch` cover BOTH an active opposition (`any Disqualifying`) and
  a mere absence of proof (`only Neutral / nothing`). `Outcome::Refused` requires a rule to name, so
  absence-of-proof has to map to `Abstained`. **If Epic 5 maps `NoMatch → Refused` uniformly, every
  honest `must-abstain` trap fails** — the exact case D18 says must NOT be gated (*"an engine that
  abstains because there is NOT ENOUGH SIGNAL is being honest… We do not gate that"*). The
  story-4.6a entry above records the `Ambiguous`-has-no-cause half; this is the other half.
  **Owner: Epic 5**, with 4.7 as the first place it can bite.
  ↺ **Story 5.4 built the FORK at the type level and nothing more:** `Conclusion::NoMatch { rule }`
  and `Conclusion::Abstained { cause }` are now two distinct variants, so the two halves have
  somewhere to land. **The MAPPING still has no producer** — nothing decides which side an input
  falls on. Owner of that decision: **story 5.4b**. Not struck.
  ↺ **CLOSED IN PART by story 5.4b, 2026-07-29:** `decide` decides. `any Disqualifying` →
  `NoMatch { rule }`; `only Neutral / nothing` AND the class D13's table does not cover →
  `Abstained { AbsenceOfProof }`. **The `Outcome` mapping remains unbuilt — owner story 5.7.** Not
  struck.
  ✅ **CLOSED by story 5.7, 2026-08-01.** `score::outcome_of` maps BOTH halves and keeps them apart:
  `NoMatch { rule } -> Refused { rule }` and `Abstained { cause } -> Abstained { cause }`. The
  feared uniform `NoMatch -> Refused` is unrepresentable — the fork is in the type, and `outcome_of`
  matches it exhaustively. Measured on the corpus rather than argued: the six committed
  `must-not-merge` traps L1 answers all reach `Refused` and pass, and the three `must-abstain` traps
  are not answered at all (`Report::scored_in(MustAbstain) == 0`), so no honest abstention was
  failed. **Struck.**

## Deferred from: story-4.6b (2026-07-22)

- **The gate's number is NOT published by `cargo xtask ci`.** The harness lives in `opencmdb-bin`
  beside `read_traps`, not at the architecture's `xtask/src/gen_metrics.rs`, because `xtask` cannot
  reach `bin`'s corpus reader without depending on `opencmdb-bin` — which would drag sqlx, axum and
  askama into the dev-tool runner (D56 makes `xtask` a dependency of nobody, and the reverse has
  never been sanctioned). Two candidate resolutions, neither chosen here: **(a)** let `xtask` depend
  on `opencmdb-bin` for the corpus reader only, or **(b)** move the corpus reader (`read_traps` /
  `read_jsonl` / the walk) into a place `xtask` may depend on. Until then the release gate is
  runnable and tested, but not wired into CI.
- **The corpus reader is dev-only by construction, so the harness cannot ship in the binary.**
  `fixtures.rs` carries `#![allow(dead_code)]` and bakes `FIXTURES_DIR` from `CARGO_MANIFEST_DIR` at
  compile time; the path exists on no deployed machine. `trap_gate` inherits that: it is
  `#![allow(dead_code)]`, exercised by tests, and reached by no runtime path. Making the gate a real
  CI check (above) and making it shippable are the same unblocking work.
- **`read_traps` resolves a trap's `replay` against the BAKED corpus root, not against the root the
  harness was given.** So a scratch trap corpus may only reference replay streams that exist in the
  committed corpus. It is enough for 4.6b's red-able demonstration (a scratch trap varies its
  expectation, not its stream), and it means a future fully-independent scratch corpus — traps AND
  streams under one scratch root — needs `read_traps` to take a root too. Not needed yet.
- **Two committed replay streams are judged by no trap** (`partial-then-failed.jsonl`,
  `capability-downgrade.jsonl`). The trap-gate walk scans `scenario/traps/`, not `scenario/replay/`,
  so it never meets them; they are discovered by no trap and scored by nothing. Expected, owned by
  **story 4.7** (the trap runner), recorded so nobody "fixes" it here.

## Deferred from: story-4.6c (2026-07-23)

- **Lattice monotonicity is NOT implemented.** *"Losing a capability can only move a verdict TOWARD
  doubt, never toward certainty. `C' ⊆ C ⟹ verdict(C') at least as doubtful`"* [architecture.md:2075-2077]
  is the law that makes run comparison exhaustively testable (2^n capability subsets × the fixture
  bank), and it needs an engine to produce verdicts across subsets. 4.6c refuses a differing-snapshot
  comparison but does not yet check that a smaller capability only moves the verdict toward doubt.
  **Owner: Epic 5**, as its *"monotone-honesty invariant trap family"*.
- **`source_state` is EXCLUDED from the comparison key, deliberately.** `compare_records`
  destructures it with `source_state: _` and never reads it, because it is uninhabited until Epic 13
  (4.6a) — comparing it is vacuous today. **When Epic 13 fills it, this exclusion must be revisited:**
  two verdicts under the same capability snapshot but different liveness (`Live` vs `Blind`) may or
  may not be comparable, and that is a D34/D36 question Epic 13 owns. The exhaustive destructure (no
  `..`) guarantees a compile error forces that decision the day the field gains a type.
- **The comparison key is PAIRWISE, not run-level.** A run is a set of records; 4.5b made the
  capability descriptor positional, so two records in one run legitimately carry different snapshots
  and "the run's snapshot" is not well-defined. `compare_runs` therefore matches by `TrapId` and a
  run may be *partly* comparable — some pairs compared, others refused. Run-level comparability was
  rejected for this reason; recorded so the choice reads as a decision, not an accident. The
  comparison is a PURE function in `opencmdb-core` (AC3): no persistence, no I/O — two in-memory runs
  in, one `RunComparison` out. If a future story needs to compare runs from different processes, it
  serializes a run then (never under `fixtures/`, the locked oracle) and that is where the format
  decision lives.

## Deferred from: story-4.7a (2026-07-23)

- ↺ **PARTLY closed by story 5.4 — this annotation belongs to the entry BELOW** (*"The firing-rule
  contract (AC6) is RECORDED, not built"*), **and is deliberately NOT struck.** The TYPE that carries
  the `(rule, verdict, evidence)` triple now exists: `identity::cascade::RuleVerdict`, with
  `evidence: Vec<ObsId>` — the smallest shape that is not invented, since the architecture mentions
  the identity link's evidence on **five** lines and shapes it on none of them (`architecture.md:978`,
  `:1015`, `:1032`, `:1309`, `:3378`). **Nothing produces one**: no rule speaks, so no
  verdict vector is ever built, and *"a test must red if it does not"* still has nothing to red.
  `score::VerdictVectorEntry` therefore stays uninhabited and `ScoredRecord::verdict_vector` stays
  provably empty — story 5.7 owns that unification. _(⚠️ RE-OWNED by story 5.7 itself, 2026-08-01:
  it did NOT unify the two, and the obstacle is measured. See the `## Deferred from: story-5.7`
  section for the new owner and the measurement.)_ ⚠️ Nor does anything enforce that a verdict which
  ARGUES leaves non-empty evidence: `RuleVerdict`'s fields are `pub` with no constructor
  (`ScoredRecord`'s precedent). **Owner moves from Epic 5 to story 5.5**, the first story with a
  firing rule.

- **The firing-rule contract (AC6) is RECORDED, not built.** D19/D46b: *"a rule that fires must
  leave its `rule_id` and its evidence behind — a rule that fires without leaving its `rule_id` is
  undebuggable in production."* There is no rule and no producer in Epic 4, so 4.7a's `run_trap`
  ASSERTS `(verdict, rule)` on already-produced answers but cannot enforce that a firing rule records
  its evidence — that is the Epic-5 engine's obligation. It is pinned today only by the uninhabited
  `VerdictVectorEntry` placeholder (4.6a), whose element is the `(rule, verdict, evidence)` triple
  D18's harness requires [architecture.md:1397]. **Owner: Epic 5** — when the identity cascade
  produces verdicts, each rule must emit its `rule_id` and evidence into the verdict vector, and a
  test must red if it does not. Inventing a producer to "satisfy" AC6 now would be the *"metric
  written after the engine"* mistake in reverse.
- ↺ **PARTLY closed by story 5.3 — this annotation belongs to the entry BELOW** (*"The
  `NoMatch → Refused` vs `Abstained` question is Epic 5's"*), **not to the firing-rule/evidence
  entry above it, which story 5.3 deliberately did not touch** (AC6). **Not struck.**
  The VOCABULARY half is decided: an
  absence of proof has a name to abstain with (`IdentityAbstentionCause::AbsenceOfProof`) that is
  not a refusal, so the failure mode this entry's sibling warns about — mapping `NoMatch → Refused`
  uniformly and failing every honest `must-abstain` trap — now has a type-level alternative. The
  QUESTION itself is untouched: no engine decides which half of `NoMatch` it is in. ~~Owner stays
  **stories 5.4/5.5**~~
  ↺ **Owner UPDATED 2026-07-29 by story 5.4: stories 5.4b/5.5.** 5.4 built the FORK at the type
  level — `Conclusion::NoMatch { rule }` for the `any Disqualifying` half, `Conclusion::Abstained
  { cause: AbsenceOfProof }` for the half with no rule to name — and `Conclusion::NoMatch`'s own doc
  carries the argument. **Which side an input falls on is still decided by nothing**: story 5.4b
  writes the combining function. Still not struck.
  ↺ **CLOSED IN PART by story 5.4b, 2026-07-29:** the combining function exists
  (`identity::cascade::decide`) and chooses the half by the presence of a `Disqualifying`. **The
  `NoMatch → Refused` vs `Abstained` mapping onto `Outcome` is still nobody's code — owner story
  5.7.** Not struck.
  ✅ **CLOSED by story 5.7, 2026-08-01** — `score::outcome_of`. Same closure as the two bullets in
  `## Deferred from: story-4.6a`; recorded here too because this entry states the requirement in the
  form that would have bitten (*"if Epic 5 maps `NoMatch → Refused` uniformly, every honest
  `must-abstain` trap fails"*), and it is that form the corpus measurement answers. **Struck.**
- **The `NoMatch → Refused` vs `Abstained` question is Epic 5's, not scored here.** `run_trap` scores
  answers; it does not decide what an engine that finds no merging rule should return. Whether "no
  rule matched" is a `Refused` (a decision, names an opposing rule) or an `Abstained` (no decision,
  names a cause) is an engine-design question the identity cascade owns. Recorded so 4.7a's silence
  on it reads as scope, not oversight.
  ↺ **Story 5.4 built the FORK at the type level and nothing more:** the cascade can now say
  `NoMatch { rule }` or `Abstained { cause }` and the two are different types of answer. **Which one
  a given input gets is still undecided and unproduced** — owner **story 5.4b**. Not struck.
  ↺ **CLOSED IN PART by story 5.4b, 2026-07-29:** which one a given input gets is now decided, by
  `decide`, over every one of D13's input classes. **Producing one still needs a rule (story 5.5) and
  mapping one onto `Outcome` still needs story 5.7.** Not struck.
  ✅ **CLOSED by story 5.7, 2026-08-01.** Both remaining halves are now in place: story 5.5 supplied
  the rule and story 5.7 the mapping (`score::outcome_of`), and `opencmdb_bin`'s `l1_runner` runs
  the whole chain over the committed corpus. `run_trap`'s silence on the question was scope, and it
  stays scope: `score.rs` still does not decide what an engine should return, it now merely records
  what one did. **Struck.**

## Deferred from: code review of story-4.7a (2026-07-23)

- **`(verdict, rule)` comparison is whitespace/case-sensitive, no normalization** — Owner: Epic 5.
  `run_trap` compares `expected.rule() != actual.rule()` on the raw `RuleId` strings. The `Outcome`
  side's `RuleId` is never validated; the `Expectation` side is only emptiness-checked, NOT trimmed or
  lowercased the way `TrapId` is (`trap.rs`). So `rule = "l1-exact-mac "` (trailing space, passes
  validation) versus a clean engine-emitted `l1-exact-mac`, or a casing difference, would be a
  false-positive `WrongRule` — a red gate on a correct answer. Harmless pre-engine (no real rule
  producer exists in Epic 4; rules come from hand-authored fixtures), but when Epic 5 supplies a
  producer the rule identity must be normalized on both sides — or the trap corpus authoring rules
  must be locked to a canonical form — before this comparison can be trusted.

## Deferred from: code review of story-4.7b (2026-07-23)

- **Cross-file trap-id uniqueness is exact, not case/trim-folded — asymmetric with the within-file
  guard.** PRE-EXISTING (the harness's `seen: BTreeMap<TrapId, PathBuf>` predates 4.7b; that story did
  not touch it). `TrapFile::validate` folds ids `trim().to_lowercase()` for `DuplicateId` (trap.rs),
  precisely because two ids "indistinguishable in a failure message" are a defect. But `score_corpus`'s
  cross-file `seen` map matches `TrapId` EXACTLY, so `id = "randomized-mac"` in `a.toml` and
  `"Randomized-MAC"` (or `"randomized-mac "`) in `b.toml` are both discovered with NO error. Exact
  duplicates across files ARE caught (no double-scoring); the gap is the near-duplicate: message
  confusability plus a near-twin left silently discovered-but-unscored if a future answer map keys only
  one casing. Owner: whoever hardens the cross-file corpus guards — fold the cross-file `seen` key with
  the SAME `trim().to_lowercase()` the within-file guard uses, and add a test with two files whose ids
  differ only by case/whitespace.

## Deferred from: code review of story-4.8 (2026-07-24)

- **The reality-debt register is outside the privacy walk's reach.** The synthetic-data guard
  (`assert_facts_are_synthetic` / `the_corpus_carries_no_real_network_data`, `crates/opencmdb-bin/src/fixtures.rs`)
  scans `scenario/replay/` observation streams only; no automated check scans any `README.md`, including
  the new register at `fixtures/scenario/traps/README.md`. **PRE-EXISTING** (no README was ever
  scanned; the register did not create the gap) but now more pointed: the register is BY DESIGN the
  corpus file most likely to tempt a pasted real MAC/hostname/IP, because every entry is sourced from a
  real Tier-2 bulk run. Today the only guard is prose discipline — D19 plus the register's own "Never
  real network data" section, which requires a recorded case to be a PATTERN (*"two randomized MACs,
  one physical interface"*), never a capture. This is the same class as the already-recorded
  *"a hostname written in prose — a machine cannot recognise the second"* residual (story-4.1 review):
  a machine cannot reliably tell a synthetic pattern from a real one in free text. Owner: whoever
  hardens corpus privacy — a lint that flags a non-RFC-5737 IPv4 or a non-locally-administered MAC
  literal appearing in any committed corpus `README.md` would catch the obvious paste; the harder
  hostname-in-prose case stays a review-discipline matter.

## Deferred from: code review of story-4.10 (2026-07-24)

- ✅ **CLOSED by story 5.1.** `every_replay_stream_re_serializes_to_its_committed_bytes`
  (`crates/opencmdb-bin/src/fixtures.rs`) walks every `.jsonl` under `scenario/replay/`, re-serializes
  record by record and compares to the committed bytes line by line, naming the file and its 1-indexed
  line on failure. CONTROL records are covered too — `ControlRecord` gained `Serialize` for exactly
  that, so the internally-tagged `record` marker is pinned as well. Proven to red twice: a space
  inserted after a colon on line 1 of `dhcp-churn.jsonl`, and the same on the capability line
  (line 3) of `capability-downgrade.jsonl`. What it pins is the SHAPE, starting from the file —
  never the authored values; see the story-4.13 entry below, which stays open for that reason.
  _(Strengthened 2026-07-27, on this story's own code review: the line-by-line comparison ran on
  `str::lines()`, which strips a trailing `\r` as well as the `\n`, so a stream re-authored with
  CRLF endings or with its final newline dropped round-tripped GREEN — for 12 of the 13 streams,
  since only `minimal.jsonl` carried whole-file byte equality. The witness now refuses both, each
  proven red on `dhcp-churn.jsonl`. The sha256 lock is not the backstop here: the threat model this
  entry exists for is a DELIBERATE re-authoring, which refreshes `MANIFEST.toml` by definition.)_
  ~~A new committed replay stream's serde byte-shape is not pinned by a round-trip test.~~ The
  byte-exactness guard `re_serializing_reproduces_the_committed_bytes`
  (`crates/opencmdb-bin/src/fixtures.rs`) round-trips only `minimal.jsonl`, so no other committed
  stream — including `randomized-mac.jsonl`, `example-traps.jsonl`, and now `multi-nic.jsonl` — has its
  exact serialized byte-shape (field order, `MacAddr` array encoding, `Uplink` field names) pinned by a
  parse→re-serialize→compare test. **PRE-EXISTING** (true of every stream since the corpus began; the
  multi-NIC family did not create the gap) but newly pointed because `multi-nic.jsonl` is the first
  stream to carry the `Uplink` fact, whose byte-shape has no round-trip witness. Today the streams are
  still gated for *parseability* by `every_replay_stream_in_the_corpus_is_valid` (a wrong field name or
  a malformed `MacAddr` array would red it) and their bytes are frozen by `MANIFEST.toml`'s sha256 — so
  a silent drift cannot land — but "these bytes are exactly what the type re-emits" is asserted for one
  stream only. Owner: whoever hardens corpus byte-fidelity — extend the round-trip witness to walk every
  committed stream (or at least one carrying each fact kind), so the assertion "the committed bytes are
  the canonical serialization" holds corpus-wide, not just for `minimal.jsonl`.

## Deferred from: code review of story-4.12 (2026-07-24)

- ✅ **CLOSED by story 5.1.** `every_committed_replay_stream_is_admissible_to_the_connector`
  (`crates/opencmdb-bin/src/fixture_connector.rs`) loads all 13 committed streams through
  `FixtureConnector::load` with a HAND-AUTHORED per-stream context table (path, `ConnectorId`,
  `scopes_covered`, initial `Capabilities`) — never derived from the observations, which would make
  `ForeignConnectorId` and `UncoveredScope` vacuous by construction. Checked in both directions: a
  walked stream with no entry reds, and an entry naming no file reds.
  _(This read "All three guards proven to red" until the story's own code review on 2026-07-27,
  which counted the assertions the walk actually ships and found more than three. The true tally,
  after the review-fix pass: **four proven red** — admissibility itself (a foreign `ConnectorId`),
  the stream-without-an-entry direction, the entry-without-a-file direction, and
  `checked == table.len()` against a duplicated entry — plus `checked > 0`, which moved into
  `walk_replay_streams` and was proven red there, and ONE that is defence-in-depth and says so in
  its own comment: `walked.insert(...)` is unfalsifiable, because a stack walk that panics on
  symlinks cannot yield the same path twice.)_
  **What it does NOT prove** is stated in the test's own doc: the walk shows every stream is
  ADMISSIBLE and never observes `UndeclaredFactKind` firing; the fact-kind check is non-vacuous only
  on `partial-then-failed.jsonl` and on both sides of `capability-downgrade.jsonl`'s record, and is
  vacuous on the eleven `corpus_*` streams because `corpus_caps()` declares all seven kinds.
  ~~Family replay streams are never exercised through `FixtureConnector::load`'s admissibility
  checks.~~ The 4.4 admissibility layer (foreign `connector_id`, uncovered scope, undeclared fact
  kind, repeated `obs_id`) is only ever run against `minimal.jsonl`; every family stream since 4.9
  (`randomized-mac`, `multi-nic`, `shared-hardware-vm`, `cloned-mac`) is gated for parseability and
  corpus validity by the fixtures walks, but no test loads them through the connector. Pre-existing,
  not caused by 4.12 (whose stream would pass those checks — verified during its review). Related to,
  but distinct from, the story-4.10 round-trip byte-shape defer above. Owner: whoever hardens corpus
  byte-fidelity — the natural fix walks every committed stream through `FixtureConnector::load` in
  one test.

## Deferred from: code review of story-4.13 (2026-07-24)

- ✅ **CLOSED by story 5.2b.** ~~The dhcp-churn byte-pin test pins MAC/hostname values relationally,
  never by value.~~ `the_dhcp_churn_stream_moves_the_address_only_through_observed_at` was EXTENDED
  — not duplicated — to pin `02:00:5e:00:53:78` = `MacAddr([2, 0, 94, 0, 83, 120])`,
  `doc-host-golf` and `doc-host-hotel` by VALUE, alongside every relational assertion it already
  carried. Proven to red TWO-SIDED, which is this entry's whole justification: with
  `doc-host-hotel` renamed to `doc-host-india` in the committed stream, the PRE-STORY tree reported
  **130 + 86 + 42, zero failures** — the relational `assert_ne!(hostname(2), hostname(0))` stayed
  green because golf ≠ india — while the extended test reds naming N3 and the value it expected.
  Both halves were run (`git stash push` on `fixtures.rs`, then `pop`); the artefact was restored
  and `git status fixtures/` verified empty.
  ⚠️ **This bullet's own next sentence is FALSE, and is corrected here rather than struck
  silently — the register is append-and-strike, so a wrong sentence survives unless its correction
  travels with the closure.** It says the three constants are cited by *"both `reason` strings"*.
  Counted on the committed file: the MAC appears in ONE reason (`dhcp-churn.toml:39`),
  `doc-host-hotel` in ONE (`:28`), and `doc-host-golf` in BOTH. The UNION of the two reasons cites
  all three; NEITHER reason does. The conclusion the entry rests on is unchanged — all three are
  cited by prose and were asserted by no test. The same false distribution is in `epics.md:1391`
  and was not repeated by the story that closed this.
  The
  constants both `reason` strings cite (`02:00:5e:00:53:78`, `doc-host-golf`, `doc-host-hotel`)
  are asserted by no test: `the_dhcp_churn_stream_moves_the_address_only_through_observed_at`
  asserts equality/inequality BETWEEN observations (per AC3's own wording), not the authored
  values themselves, so a re-authored stream with different (still-synthetic) MACs/hostnames and
  a refreshed sha256 would strand the reason text while every test stays green. Pre-existing
  class, not caused by 4.13 — exact-value pinning (`expected()`) and the round-trip witness cover
  only `minimal.jsonl` (see the story-4.10 defer above); the review's patch items cover the values
  the family premise depends on (the three `IpV4`s) and the fact-count, and leave whole-value
  pinning to the same owner: whoever hardens corpus byte-fidelity, corpus-wide rather than
  per-family.
- ✅ **CLOSED by story 5.2b**, which is the owner this bullet names. ~~↺ STILL OPEN after story
  5.1~~ — the authored VALUES the two `reason` strings cite are now pinned, and so are those of
  every other family whose prose cites constants: `randomized-mac`, `multi-nic`,
  `shared-hardware-vm`, `cloned-mac` and `dhcp-churn`, plus the `example-traps` stream, assert
  their authored MACs, addresses, hostnames, uplinks and instants by value.
  ⚠️ **"Corpus-wide" would be too strong and is not claimed** — corrected on this story's code
  review, which counted it. There are **13** streams under `scenario/replay/`; eight had no value
  pin, and **two still have none**: `partial-then-failed.jsonl` and `capability-downgrade.jsonl`
  (their facts — `[2,0,94,0,84,1]`, `doc-host-c`, `[2,0,94,0,85,1]`, `198.51.100.10`, `doc-host-d`
  — appear in no `.rs` file; `fixture_connector.rs` READS those streams rather than restating them,
  so it is not a second oracle for them). They carry control records rather than a trap family,
  which is why no `reason` prose strands on them — but the sentence "all committed streams" would
  have been false. Registered under `## Deferred from: story-5.2b`. The re-authoring this bullet describes — different-but-still-synthetic values with a
  refreshed sha256 — now reds. **Nine mutations recorded across eight artefacts** (seven in the
  story, two more on its code review), each the SOLE red; six of the nine are aimed at a stream no
  other value test reads, and three at a trap `.toml` rather than a stream. *(That count was
  "six" here and "seven" in the story until the code review; AC6 enumerates seven and its preamble
  says "five, at minimum", so no single figure matched. The counted one is recorded rather than a
  third guess.)* *(Both bullets in this section are closed together on purpose: closing only the first
  would leave the section ending on a bullet asserting the item is open and naming an unfilled
  owner.)*
  ↺ ~~**STILL OPEN after story 5.1**~~ — the entry above is NOT closed by it, deliberately. 5.1's
  corpus-wide round-trip witness starts from the FILE, so it pins the byte-SHAPE and never the
  authored values; its `obs_id` helper pins ids only. A re-authored `dhcp-churn.jsonl` with
  different-but-still-synthetic MACs/hostnames and a refreshed sha256 would still strand the two
  `reason` strings while every test stays green. Owner unchanged: whoever pins the authored VALUES
  corpus-wide.

## Deferred from: code review of story-4.14 (2026-07-25)

- ✅ **CLOSED by story 5.2.** `the_committed_trap_text_carries_no_real_network_data` reads every
  trap file's raw bytes with `read_to_string` and scans them through `assert_text_is_synthetic`
  BEFORE `read_traps`, so header comments — which TOML parsing discards — are covered. Proven to
  red by putting `00:11:22:33:44:55` into `example.toml`'s header COMMENT: the panic names the
  file. It also asserts its own COVERAGE (4 distinct MACs, 3 distinct IPs — the measured values),
  because ten files carrying zero addresses would be as vacuous as zero files and the walk's
  `checked > 0` counts only files.
  ⚠️ **This bullet's own premise was FALSE when it was closed, and is corrected rather than struck
  silently:** it says the scanner's *"only corpus call site is the `Record::Failure` walk"*. Story
  4.18 falsified that on 2026-07-25 by adding a second one (`fixtures.rs`, the wire body
  `scenario/wire/unifi-clients.json`, whose directory sits outside every corpus walk) — one day
  BEFORE this entry was written. Closing an item on a check its own tree falsifies is the failure
  story 5.1's review named; the false sentence in the 4.14 wiring test's doc that said the same
  thing was removed in this story rather than re-enumerated.
  ~~Trap-file text is scanned by no privacy rule.~~ `assert_text_is_synthetic`'s only corpus call
  site is the `Record::Failure` walk over replay streams (fixtures.rs); the headers and `reason`
  strings of `fixtures/scenario/traps/*.toml` reach it never — `example.toml`, `randomized-mac.toml`
  and `dhcp-churn.toml` have carried raw (synthetic) MAC strings in reasons since they landed, and
  4.14's header commits the first full universally-administered MAC string. Pre-existing since 4.2,
  stated honestly by 4.14's own scanner-wiring test doc; the 4.14 story holds its no-octets-in-reasons
  rule by review, not by gate. Owner: whoever hardens corpus privacy — the natural fix walks trap
  files' raw text through the same scanner (comments included, before TOML parsing discards them).
- ✅ **CLOSED by story 5.2** — all three named evasions, each with its own guard and its own
  recorded red. The tokenizer became **boundary-anchored longest-match**: `-` is normalised to `:`
  inside the scanner (`MacAddr::from_str` stays colon-only — widening a domain parser for a test's
  convenience is a D47 frontier violation), the text splits into maximal `[0-9a-fA-F.:]` runs, and
  inside a run the LONGEST prefix that parses is taken and the scan resumes after it. That sees a
  trailing `.` or `:`, an INTERIOR separator (`198.18.0.1:8080` — the row an edge-trim cannot
  reach) and the dash form. `is_synthetic_mac` now refuses MULTICAST (`addr.0[0] & 1`) whatever the
  U/L bit says, which makes `33:33:…` and `01:00:5e:…` consistent, and the refusal MESSAGE was
  split so it does not assert the false sentence *"neither locally administered nor …"* about an
  address that IS locally administered. All six rows were observed on both sides: rows (a)–(e) were
  GREEN before the fix (the `should_panic` tests failed with *"test did not panic as expected"*) and
  row (f) panicked with the OLD wording, so only its stated reason changed.
  **Two things were MEASURED here and are worth more than the closure:** enumerating every
  substring instead — the obvious fix — reds the committed corpus, because `Ipv4Addr::from_str`
  rejects only leading zeros and `92.0.2.90` parses out of `192.0.2.90` (run: the wire test reds
  naming it). And of the shape's two conjuncts, it is the RESUME that earns that, not the start
  anchor: removing the anchor alone leaves the whole suite green. The anchor is kept, but its
  contribution is NOT claimed — its one observable consequence is the residual limit
  (`ab198.18.0.1` stays invisible), which now has a guard,
  `the_text_scanner_is_blind_to_an_address_glued_to_hex`.
  **What did NOT close, and is re-registered under `## Deferred from: story-5.2` below:** a hostname
  in prose, `Fact::OuiVendor { vendor }`, `Fact::Uplink { peer_port }` and every `README.md`. The
  scanner is still a floor; three of its four remaining holes now have an owner.
  ~~The free-text scanner is a floor with named evasions.~~ Its own doc says "a floor, not a proof";
  the review named the specific gaps: a MAC/IP immediately followed by kept punctuation (`.`/`:`)
  tokenizes unparseable and evades; dash-form MACs (`00-00-5e-…`) are never seen (`MacAddr::from_str`
  is colon-only); and the U/L-bit rule reads any `x2/x3/x6/x7/xa/xb/xe/xf`-first-octet MAC as
  synthetic, which admits IPv6-multicast-shaped strings (`33:33:ff:…`) that can embed real interface
  identifier bytes while refusing IPv4-multicast (`01:00:5e:…`) — an inconsistency, not a leak, in
  committed synthetic data. Pre-existing, not aggravated by 4.14. Owner: same as above, one scanner
  hardening pass.

## Deferred from: code review of story-4.15 (2026-07-25)

- ✅ **CLOSED by story 5.1.** Both named byte-pins now assert their `obs_id`s
  (`dhcp-churn` → `adadadad-…001`…`003`, `vrrp-virtual-mac` → `aeaeaeae-…001`…`004`), through a
  single test helper `assert_obs_ids(observations, prefix)` into which the four existing copies of
  the loop were folded — six call sites of one mechanical loop was accidental duplication, and all
  four already COMPUTED their ids rather than restating them, so no second oracle was lost. Proven to
  red by swapping two `obs_id`s between lines of `dhcp-churn.jsonl`. **This covers only streams that
  HAVE a byte-pin test:** `randomized-mac`, `multi-nic`, `shared-hardware-vm` and `cloned-mac` have
  none — see the story-5.1 section at the end of this file, owned by story 5.2b.
  ~~The older byte-pin tests do not pin the obs_id ↔ line binding.~~ The trap files judge by
  `obs_id`; the byte-pin tests read by index. 4.15's own test now pins its three obs_ids (the
  review's patch), but the sibling byte-pins (`the_dhcp_churn_stream_moves_the_address_only_
  through_observed_at`, `the_vrrp_stream_shares_one_virtual_mac_and_moves_its_uplink`) still
  read purely by index: a deliberate re-authoring that swaps two lines' obs_ids (with a re-hashed
  manifest) would invert what those families' traps judge while every byte-level assertion stayed
  green. Pre-existing pattern since 4.13, not aggravated by 4.15. Owner: whoever hardens corpus
  byte-fidelity — three `assert_eq!` per older test, same shape as 4.15's.

## Deferred from: code review of story-4.16 (2026-07-25)

- ✅ **CLOSED by story 5.2** — with the weaker true sentence, not the stronger one. `raw` now goes
  through `assert_text_is_synthetic` in the `Record::Observation` arm, which story 5.2 extracted
  into `assert_record_is_synthetic` (the `match` stays exhaustive with no `_` arm — a new `Record`
  variant must still break it). **The call site is VACUOUS on today's corpus and says so in its own
  doc:** across all 13 replay streams and the wire artefact exactly ONE observation carries a
  non-null `raw` — `minimal.jsonl:3`, `{"provenance":"never read by a decision"}` — and it holds no
  address. So this is *"`raw` is scanned by the same rule"*, never *"`raw` is now covered
  corpus-wide"*. Because it is vacuous it does not defend itself, so it ships with a PERMANENT
  guard rather than only a mutation record: `an_observations_raw_payload_is_scanned` drives a
  hand-built record whose `raw` names `198.18.0.1` through the very function the walk drives.
  Deleting the call site reds exactly that one test and nothing else — which IS the vacuity,
  measured. The mutation is record-side BECAUSE the corpus has no `raw` to break, and a scratch
  tree cannot substitute: `walk_replay_streams` hardcodes its root and must keep it (story 5.1's
  callers depend on it).
  ~~`Observation.raw` is scanned by no privacy rule.~~ The corpus walk
  `the_corpus_carries_no_real_network_data` inspects `facts` (and `Failure` detail text), never
  the free-text `raw` payload — and `minimal.jsonl`'s third observation already carries prose in
  `raw` that nothing inspects. `raw` is documented as "never read by a decision", but the privacy
  rule's charter is the COMMITTED BYTES, not what decisions read. Pre-existing since 4.1, not
  aggravated by 4.16 (its four lines carry `raw: null`). Owner: whoever hardens corpus privacy —
  route `raw` through `assert_text_is_synthetic` in the same walk arm.

## Deferred from: story-5.1 (2026-07-27)

_Raised BY the story while scoping it, not by its review — hence no "code review of" in the heading.
It is the one finding 5.1 surfaced and deliberately did not fix._

- ✅ **CLOSED by story 5.2b**, the owner this bullet names. ~~Four committed family streams are
  named by no VALUE test.~~ Each of the four now has its own byte-pin test in
  `crates/opencmdb-bin/src/fixtures.rs`, in the `dhcp-churn` idiom — length, exact per-line fact
  count, `assert_obs_ids` with its prefix and length, then the authored values, then the instants
  vector: `the_randomized_mac_stream_rests_on_one_octet`,
  `the_multi_nic_stream_pins_both_halves_of_its_uplink`,
  `the_shared_hardware_vm_stream_shares_one_uplink_and_falls_silent_on_the_abstain_pole`,
  `the_cloned_mac_stream_wears_one_mac_on_every_line`. Five explicit tests, not one table-driven
  loop — a table restates the corpus in one place and stops being independent of it.
  **And the `obs_id`↔line binding this bullet asks for turned out to be only HALF the hole.** Every
  pin above lives on the `.jsonl`; a trap file declares which pair of `obs_id`s it judges and under
  which column and rule, and nothing asserted that. `read_traps` cross-checks only that a trap's
  `obs_id`s EXIST (a `BTreeSet` membership test) and `trap_gate`'s completeness check only asks
  that both poles of a family are present — which an EXCHANGE of the two poles' vectors preserves.
  Measured during this story's validation pass and reproduced during its implementation: exchanging
  the two `observations` vectors in `fixtures/scenario/traps/cloned-mac.toml` — no stream byte
  touched — makes the corpus DEMAND the false merge (`doc-host-echo` + `doc-host-foxtrot` under
  `must-merge`/`l1-exact-mac`, the two genuine `doc-host-echo` presences under `must-not-merge`),
  and the whole workspace suite stayed green. The new `assert_trap_binds` helper pins, per trap id,
  the exact `observations` vector *in order*, the whole `Expectation`, and the declared `family`.
  **All 24 committed traps across all ten trap files are bound.** That exchange now reds.
  _(Scope corrected on this story's CODE REVIEW, and the correction is the substantive part.
  The story shipped the helper against only the five families it pinned — **14 of the 24 traps** —
  while four documents claimed "all eleven committed traps", a figure that contradicted its own
  enumeration (9 + 2 + 3 = 14) and the corpus total (24). Two review layers independently MEASURED
  the residue: exchanging the two `observations` vectors in `hostname-collision.toml` left the
  suite at 135 + 86 + 42, zero failures, while the corpus DEMANDED `must-merge`/`l1-exact-mac` on
  two DIFFERENT MACs — two physically distinct boxes that merely share a hostname, D10's
  catastrophic direction. Guy's call was to close the CLASS rather than register the instance:
  `docker-veth`, `hostname-absence`, `hostname-collision` and `vrrp-virtual-mac` are now bound
  too, folded into their existing byte-pins. The `hostname-collision` exchange reds, proven.)_
  _(`family` was added to the helper by the same review, also measured: deleting BOTH `family`
  lines from `cloned-mac.toml` left the suite green while silently exempting the family from
  `incomplete_families` — after which it could be reduced to one pole with `trap_gate` still
  green. Deleting only ONE line already reddened the gate; deleting both did not, until now.)_
  Four committed family streams ~~are named by no VALUE test~~. `randomized-mac.jsonl` (4.9),
  `multi-nic.jsonl` (4.10), `shared-hardware-vm.jsonl` (4.11) and `cloned-mac.jsonl` (4.12) have no
  byte-pin test; their only mention anywhere in the tree is the context table story 5.1 added
  (`fixture_connector.rs`, `committed_stream_contexts()`), which states each stream's declared
  context and asserts nothing about its contents.
  _(This bullet's headline read "**named by NO test at all**" and cited
  `grep -rn "<name>.jsonl" --include=*.rs crates xtask`, "which returns nothing for all four", until
  the story's own code review on 2026-07-27. **The named check was falsified by the very commit that
  wrote it**: that grep now returns four hits, all in the table above. The CONCLUSION is unchanged —
  no test asserts these four streams' values or their `obs_id` order — but a cause needs a check that
  still holds, so the check is restated rather than the conclusion re-asserted.)_
  So their `obs_id` ↔ line binding and their authored values are asserted by
  nothing narrower than the corpus walks and the sha256 lock, and their binding is WEAKER than
  `dhcp-churn`'s was before this story: `read_traps`'s cross-check only asserts that a trap's
  `obs_id`s EXIST in the stream, so swapping two ids inside one of those four files inverts what its
  traps judge and nothing reds. **Story 5.1 could not close it:** AC1 strengthens byte-pins that
  exist, and here there is no test to strengthen. What 5.1 does give them is admissibility
  (`every_committed_replay_stream_is_admissible_to_the_connector`) and byte-SHAPE
  (`every_replay_stream_re_serializes_to_its_committed_bytes`) — **not** value pins.
  **Owner: story 5.2b**, inserted on 2026-07-26 in Epic 5's debt block, immediately after 5.2 and
  ahead of the L1 join at 5.5 — the corpus is the oracle that join is judged against. A register item
  with a named owner and a slot is not a deferral, and writing it as one would misstate the plan.
  The `assert_obs_ids` helper 5.1 introduces is what 5.2b builds on: each of the four families calls
  it with its prefix and its observation count.
  _(This read "it takes four more families **without change**" until story 5.1's code review: the
  review found the helper asserted no length, so an empty or truncated slice passed it silently —
  the exact vacuity 5.1 exists to close, re-introduced in the helper that closes it. Guy's call was
  to change the shape rather than keep the promise: it now takes `expected_len` and asserts the
  count itself, so 5.2b's four call sites pass their length instead of restating it beside the
  call.)_
  _This stays here and does NOT become a GitHub issue: the register is the established home for
  review-surfaced corpus debt (every entry since 4.1), and an issue is reserved for scope that MOVES
  between epics (the 4.19b precedent, #34). Nothing moves — the work stays in Epic 5, three stories
  later._

## Deferred from: code review of story-5.1 (2026-07-27)

_Five items the review raised and did not fix. The first four are pre-existing behaviour of
`walk_replay_streams`, which story 5.1 did not change — but the hoist made that one function the
shared definition of "every committed stream" across TWO test modules, so each item's blast radius
grew even though its code did not. Recorded here because a walk that quietly sees less, or sees a
different set per run, was the recurring defect of 4.1/4.3._

- **`walk_replay_streams` never symlink-checks its own root.** `fixtures.rs:723` takes
  `fixture_path("scenario/replay")` and pushes it straight onto the stack; the `is_symlink` panic at
  `:733` only ever inspects entries found INSIDE an already-opened directory, and `read_dir` follows
  a symlinked root silently. So the doc's promise at `:720` — *"refusing symlinks"* — does not hold
  for `fixtures/`, `fixtures/scenario/` or `fixtures/scenario/replay/` themselves: the whole corpus
  could be a link to bytes outside the repository while every walk, and now the corpus-wide
  admissibility and round-trip claims, reported success. Owner: whoever next touches the walker.

- **No `is_file()` check, so a non-regular entry makes the suite HANG instead of fail.**
  `fixtures.rs:733-758` tests only `is_symlink()` and `is_dir()`, then calls `visit(&path)`, and
  every caller immediately does `std::fs::read_to_string` (`:915`, `:982`, `:1508`, `:1528`). A FIFO
  or device node named `x.jsonl` under `scenario/replay/` therefore blocks forever — the one failure
  mode with no diagnostic at all. The sibling walk in the same file already guards this
  (`fixtures.rs:1872`, `file_type.is_file() && path.extension()…`), so the fix is one condition and
  the precedent is local.

- **The walk yields unsorted `read_dir` order while its documented sibling sorts.**
  `fixtures.rs:726-761` pushes and pops a stack with no ordering, versus `trap_gate.rs:355`
  `found.sort();` — *"Sorted so a discovery run is deterministic regardless of readdir order."* With
  a corpus carrying two defects, WHICH stream's panic surfaces first varies run to run and machine to
  machine, so the connector walk and the round-trip witness can report a different failure on each
  run of the same broken corpus. ⚠️ Relevant to open issue **#38** (unexplained local
  non-determinism) as a thing to RULE OUT, **not** as a cause: nothing has been measured here, and
  *a cause needs a check, not a plausible story*. Owner: whoever next touches the walker; one
  `sort()` closes it and makes any future #38 observation reproducible.

- **`scenario/wire/unifi-clients.expected.jsonl` has no round-trip byte-shape pin at all.** It is a
  committed `.jsonl` of `Observation`s, but it sits outside `scenario/replay/`, so story 5.1's
  corpus-wide witness cannot reach it (deliberately — AC3 scoped the walk to `scenario/replay/` and
  the story's Dev Notes say so explicitly). Its own test `:2818` asserts facts, `obs_id`s and
  privacy, but never re-serializes. It is therefore the ONE committed stream whose per-line
  serialized shape and field order are pinned by nothing but the sha256 lock — a hand edit to its
  key order is invisible to every other gate. Owner: Epic 11's wire parser (`CONSUMER PENDING`,
  issue #34), which is when the file gains a real consumer and the pin stops being speculative.

- **Four cosmetic nits, grouped so none is lost.** (1) `ControlRecord` gained an **unconditional**
  `Serialize` (`fixtures.rs:126`) for a need its own doc calls `#[cfg(test)]`-only;
  `#[cfg_attr(test, derive(serde::Serialize))]` expresses the actual requirement — low stakes, the
  type is private. (2) The same derive uses an inline `serde::Serialize` path beside an imported
  `Deserialize`. (3) `fixture_connector.rs:1605` prefixes the panic with `{relative}` when
  `FixtureError`'s `Display` already carries the path, so the recorded red names the file twice —
  defensible as belt-and-braces, but the diff itself contains the evidence it is redundant.
  (4) `format!("{prefix}-0000-4000-8000-{:012}", n + 1)` renders a DECIMAL sequence into a
  hexadecimal UUID field; invisible until a stream exceeds nine lines (the longest today is six), and
  the helper's doc lists two conventions without naming this third one.

## Deferred from: story-5.2 (2026-07-28)

_Raised BY the story, not by its review. Three surfaces the story explicitly did not close, plus
the disposal of the register items its hoist inherited._

### The register items the trap-walk hoist inherited — DISPOSED OF, not carried

The three `walk_replay_streams` items above (root symlink, `is_file()`, `sort()`) were recorded
against the REPLAY walk with the rationale that a hoist grows an item's blast radius even when its
code does not change. Story 5.2 hoisted the TRAP walk the same way, and had all three. All three
are **CLOSED in `walk_trap_files`** rather than inherited:

- the root is `symlink_metadata`-checked before the stack is seeded, so the doc's "refuses
  symlinks" now covers `scenario/traps/` itself;
- `file_type.is_file()` is asserted by name. **Measured, because the register calls this "the one
  failure mode with no diagnostic at all":** a FIFO named `fifo.toml` under `scenario/traps/` makes
  the guarded walk fail with *"only regular files belong under scenario/traps/"* naming the path,
  and makes the unguarded walk HANG — a 60-second run was killed by SIGTERM with no output. Not a
  plausible story; a check that was run.
  ⚠️ **Corrected by story 5.2's own code review, because the sentence above compared a FILTERED run
  to a full one:** the guard closes the class in `walk_trap_files` and NOWHERE ELSE. The production
  walk `discover_trap_files` has no `is_file()` refusal, six `trap_gate` tests drive it against the
  committed root, and `read_traps` reads with `read_to_string` — so **the SUITE still hangs**. Run
  on the finished tree, twice: `timeout 90 cargo test -p opencmdb-bin` returns **143 (SIGTERM) with
  no output** WITH the guard in place; only `cargo test -p opencmdb-bin
  every_trap_file_in_the_corpus_is_valid` surfaces the named failure. The remedy was scoped out
  deliberately rather than taken in review (Guy's call, 2026-07-28): story 5.2's ACs cover the test
  walk, and widening it to production code would need its own prove-to-red. **Owner: whoever next
  touches `discover_trap_files`** — the fix is the foreign-extension arm's `FixtureError` idiom
  three lines below the gap;
- `found.sort()` before `visit`, matching `discover_trap_files`, so with two broken files WHICH one
  panics is the same on every run. ⚠️ Its own red is **not observable** — nondeterminism has no
  failing test — so no red is claimed for it. And as the 5.1 entry says: for issue **#38** this is a
  thing to RULE OUT, never to record as a cause.

**The replay-side twins stay OPEN**, deliberately and with the divergence named: `walk_replay_streams`
still has no root symlink check, no `is_file()` and no `sort()`, so the two sibling walks now differ
on three points. Fixing them is one line each and the precedent is now local — owner: whoever next
touches the replay walker. This story did not take them because its own AC scoped it to the trap
tree, and widening a story to a second tree is how a diff stops being reviewable.

### The dot-entry class is now closed on BOTH trees

Story 5.1 closed it for `scenario/replay/` and only there; 5.2 closed it for `scenario/traps/`, in
BOTH of that tree's walks (`walk_trap_files` and the production `discover_trap_files`). Measured
before and after: one `probe.txt` under `fixtures/scenario/traps/.claude/.cc-writes/` red
`every_trap_file_in_the_corpus_is_valid` **and six `trap_gate` tests** — including
`an_answer_for_an_unknown_trap_is_refused`, which expects an error and got `Io` instead of
`AnswerForUnknownTrap` because discovery fails before `score_corpus` validates anything — and is
green with the probe still in place afterwards. `cargo xtask ci` was GREEN throughout; its own
corpus walk has skipped dot-entries since 2026-07-21. ⚠️ **NOT a cause for issue #38** — the
directories were created 2026-07-26, they are empty, and #38's failures predate them.

### What the privacy floor still does not see — three surfaces, one owner each

Story 5.2's title is a direction, not a completion claim. The scanner covers address-shaped tokens
in trap text, control-record free text, the wire body and `Observation.raw`. It does not cover:

- **`Fact::OuiVendor { vendor }`** — free author-typed text that `assert_facts_are_synthetic`
  discards by construction (its `Fact::OuiVendor { .. } | Fact::Rtt { .. } => {}` arm). A real
  vendor string is not an address, so the address scanner would not catch it even if routed; the
  fix is a rule of its own. Owner: whoever next hardens corpus privacy — the natural moment is
  Epic 11, when the UniFi parser starts producing `oui` values from a real payload shape.
- **`Fact::Uplink { peer_port }`** — same shape, same arm (`Fact::Uplink { peer_mac, .. }` binds
  only the MAC). Same owner.
- **Every `README.md`, exempt at any depth in all three corpus walks.**
  `fixtures/scenario/traps/README.md` is 6 KB of prose — the largest un-scanned text in the corpus.
  The exemption is deliberate and load-bearing (the corpus lock's orphan rule exempts the same name,
  and two gates disagreeing about what the corpus may contain would make documenting a directory red
  the suite), so this is not a defect to fix by deleting the exemption. Owner: whoever decides
  whether a README is scanned by a rule of its own — note that a README legitimately quotes
  addresses when explaining the corpus, so the rule cannot be the current one.

A fourth hole is named but has no owner because it is not mechanically closable: **a hostname in
prose** cannot be recognised as private text. The scanner's own doc has said so since 4.2 and still
does.

## Deferred from: code review of story-5.2 (2026-07-28)

_Four items the review surfaced and did not fix. All four are pre-existing shapes the story
inherited or relocated rather than created — which is why they are registered rather than patched.
The review's substantive findings against the story itself were patched in place and are listed in
the story file's `### Review Findings`._

- **The `Record::Failure` scan call site is as vacuous as `raw`, and unlike `raw` it got no
  permanent guard.** Story 5.2's own argument, written into `assert_record_is_synthetic`'s
  neighbourhood, is that a vacuous call site *"does not defend itself, so it ships with a PERMANENT
  guard rather than only a mutation record"* — and that is why
  `an_observations_raw_payload_is_scanned` exists. The arm three lines below has the same property
  and no such guard. **Measured by the review:** the committed corpus holds exactly ONE failure text
  — `"the documentation subnet stopped answering mid-sweep"` — and it carries no address, so
  emptying the `Record::Failure` arm reds nothing. The fix is the one-line twin of the `raw` guard
  (a hand-built `Record::Failure` whose detail names `198.18.0.1`, driven through
  `assert_record_is_synthetic` under `should_panic`). Not taken here because the wiring predates
  story 5.2 — 4.x put it there and 5.2 only relocated it into the extracted helper. Owner: whoever
  next touches the replay-side privacy walk, alongside the three `walk_replay_streams` items above.
- **The dot-entry skip is evaluated AFTER the symlink refusal, in all three corpus walks.** A
  tooling scratch entry materialised as a SYMLINK rather than a real directory (`.cache` →
  elsewhere, which is how a worktree or isolation harness would most plausibly create it) is refused
  by `entry.file_type().is_symlink()` before the skip's `continue` is ever reached, so the walk
  accuses the corpus of a defect it does not have — the very outcome the skip was added to prevent.
  The class is therefore closed for real dot-directories only. Not taken here because the ordering
  copies story 5.1's replay walk verbatim: fixing it in one walk would open a fourth divergence
  between siblings that this story worked to make agree. Owner: whoever next touches the walkers —
  it is one reordering in three places, and it belongs with the replay-side twins.
- **A non-UTF-8 filename bypasses the dot-entry skip.** All three walks test
  `entry.file_name().to_str().is_some_and(|n| n.starts_with('.'))`, which yields `false` — not
  `true` — when the name is not valid UTF-8 (legal on Linux). So `.cache-\xFF` is NOT skipped, falls
  through to the foreign-extension refusal, and reds the suite naming the corpus. The intent
  ("tooling scratch is not corpus") calls for the byte-level test
  `entry.file_name().as_encoded_bytes().starts_with(b".")`. Exotic, but it fails on exactly the
  class of name a tool is most likely to emit byte-wise, and it is one decision across three walks
  rather than three. Same owner as the item above.
- **The scanner's residual floor is longer than the story named — five more shapes pass clean.**
  Story 5.2's `## Deferred from: story-5.2` section above names three surfaces plus the
  hostname-in-prose hole, and the scanner's doc named one tokenizer limit (the glued prefix). The
  review measured five more against the shipped tokenizer, and they are added to the doc rather
  than left to be re-discovered: **IPv6 literals** are attempted by no branch at all (pure hex and
  colons, so they are collected as runs and discarded) — the sharpest one, because
  `Observation.raw` is the surface this story just wired and a real capture's global-unicast IPv6
  is more identifying than the IPv4 the rule guards; **zero-padded IPv4** (`010.001.002.003`), the
  mirror of the leading-zero rejection that makes the substring route unusable; **the Cisco dotted
  MAC** (`0011.2233.4455`) and **the bare form** (`001122334455`), the same address row (d) closed,
  in the notations an IOS/Aruba/HP CLI actually prints; **the glue limit is any HEXDIGIT**, so
  `1198.18.0.1` is as invisible as `ab198.18.0.1` while the doc and the guard's name framed it as a
  hex-LETTER case; and **the resume can swallow a real address adjacent to an accepted one** —
  longest-match never backtracks, so `0a:00:11:22:33:44:55` matches the synthetic
  `0a:00:11:22:33:44` and skips the vendor MAC three bytes in, and `192.0.2.110.0.0.1` matches the
  documentation `192.0.2.110` and skips `10.0.0.1`. That last one is a limit of the mechanism story
  5.2 introduced, not an inherited one, which is why it is named beside the anchor's. None is
  closed: closing IPv6 is a rule of its own, and the notation gaps would each need a prove-to-red
  and a corpus fixture. Owner: whoever next hardens corpus privacy — IPv6 first, since it is the
  only one of the five that hides a whole address family rather than a spelling of one.
- **`Fact::Hostname { name }` is prefix-checked, never text-scanned.** The privacy arm asserts
  `name.is_empty() || name.starts_with("doc-")`, so `doc-192.168.1.1` satisfies the hostname rule
  and its address never reaches `assert_text_is_synthetic`. This is the same shape as the
  `Fact::OuiVendor { vendor }` and `Fact::Uplink { peer_port }` surfaces story 5.2 registered above
  — author-typed text a structured rule waves through — and it belongs beside them rather than in a
  section of its own. Unlike those two, the fix here is cheap and does not need a new rule: route
  `name` through the address scanner in addition to the prefix check. The rule predates story 5.2
  (4.17 last shaped it), which is why this is registered and not patched. Owner: whoever next
  hardens corpus privacy.

## Deferred from: story-5.2b (2026-07-28)

_Raised BY the story while implementing it, not by its review. One item deferred; one decided and
closed in the story rather than deferred, recorded here because the story was required to say which
branch it took._

- **A trap's `reason` prose is still not mechanically tied to the values it cites.** This story
  pins the authored VALUES of all six committed streams that had none, and pins each trap's
  `observations` vector and `Expectation`. What it does NOT do is connect the two: a `reason` may
  still cite a constant the stream does not hold, and only a human reader would notice. The
  concrete residue is measurable — `example.toml:36` says *"their MACs differ in the final octet"*
  and `dhcp-churn.toml:39` names `02:00:5e:00:53:78`; both are true today because this story
  asserted the bytes beside them, but neither reason is checked AGAINST the bytes. That is what
  this family of tests can reach and this story deliberately did not attempt: it needs a
  value-extraction pass over free-form English, which is a rule of its own rather than another
  pin. The file that names the risk best is `example.toml`'s own header — *"the first version of
  this file claimed two observations shared a MAC when the committed bytes said otherwise, and a
  reader caught it precisely because the claim was written down."* A reader caught it once.
  Owner: whoever next hardens corpus byte-fidelity — and it should be weighed against simply
  accepting that `reason` is prose, since a scanner over English is the kind of check that fails
  in the direction of false confidence.
- ✅ **DECIDED and closed in the story, not deferred: `example-traps.jsonl` is now pinned.** The
  story's validation pass surfaced it as a SIXTH committed stream named by no value test —
  previously unregistered, and carrying exactly the shape the story-4.13 entry above is about
  (`example.toml:26`'s reason cites `02:00:5e:00:53:10`, asserted by nothing). The story offered
  two branches, pin it or register it with a named owner; the pin was taken, because it is three
  lines of corpus and closing the theme while leaving a known instance of it open would have made
  the closure claim above narrower than it reads. The test is
  `the_example_trap_stream_carries_the_values_its_reasons_cite`, proven to red by collapsing E3's
  final octet onto E1's. It also binds all three of `example.toml`'s traps, including the one that
  judges `minimal.jsonl` — a trap names the stream it judges, and nothing assumes there is one.
- **Two committed streams still have no value pin: `partial-then-failed.jsonl` and
  `capability-downgrade.jsonl`.** Counted on this story's code review: 13 streams under
  `scenario/replay/`, eight had none, six gained one here. These two hold real authored facts
  (`Mac [2,0,94,0,84,1]`, `doc-host-c`, `192.0.2.30/.31/.32`; `Mac [2,0,94,0,85,1]`,
  `198.51.100.10`, `doc-host-d`) that appear in no `.rs` file. `fixture_connector.rs`'s
  `partial_observations()` READS the stream rather than restating it, so it is not a second
  oracle — it pins `connector_id`, `scope`, capabilities and `obs_id`s, never a `Fact` value.
  They are the two streams built around CONTROL records (`failure`, `capability`) rather than a
  trap family, so no `reason` prose strands on them and the pressure is lower than it was for the
  six closed here — which is why this is registered rather than fixed. Owner: whoever next hardens
  corpus byte-fidelity, alongside the `reason`-prose item above.
- **`raw` is pinned by nothing on the six newly-pinned streams.** `expected()` restates
  `raw: None` for every line of `minimal.jsonl` — which is what "a second oracle in the spirit of
  `expected()`" leads a reader to expect — but the new pins read `facts`, `obs_id` and
  `observed_at` only. Story 5.2's `raw` scan reddens on an address- or MAC-shaped payload, so a
  privacy leak is caught; an arbitrary non-address string appearing in `raw` is not. Pre-existing
  class, not caused by this story. Owner: whoever next hardens corpus byte-fidelity.

**Still open after this story, and deliberately so — the narrow true claim.** Every register entry
that was owned by *"whoever hardens corpus byte-fidelity"* **when this story opened** is now
closed. The complete list is four entries, not the three an earlier draft of this paragraph gave:
the `code review of story-4.10`, `code review of story-4.12` and `code review of story-4.15` defers
(all three closed by 5.1) and both `code review of story-4.13` bullets (closed here). *(The
enumeration omitted 4.12 and 4.15 until this story's own code review — the owner phrase wraps
across a line break there, so a single-line `grep` misses it. The universal claim was true either
way; the list after the colon was not, and an incomplete inventory is the failure Dev Notes lesson
3 names.)* Verified by re-reading the register AFTER the last edit, not before — story 5.1's review
found a citation its own diff had falsified, and 5.2's replaced a false sentence with another its
own commit falsified.

That is NOT the same sentence as `epics.md:1397`'s *"the corpus byte-fidelity theme carries no open
item"*, which was true when written on 2026-07-26 and false by the next day. **Two things in this
area read open after this story, and both must keep reading open:**

- `scenario/wire/unifi-clients.expected.jsonl` has **no round-trip byte-shape pin at all** — see
  the code-review-of-story-5.1 section above. It sits outside `scenario/replay/`, so 5.1's
  corpus-wide witness cannot reach it by design. Owner: **Epic 11's wire parser** (`CONSUMER
  PENDING`, issue #34). Not this story's, and untouched by it.
- the `reason`-prose item this story opened at the top of this very section, whose owner reads
  *"whoever next hardens corpus byte-fidelity"*. It is NEW — it did not exist when the claim above
  was scoped — and naming it here is what keeps that claim from being falsified by its own commit.

## Deferred from: code review of story-5.2b (2026-07-28)

_Three parallel layers (Blind Hunter, Edge Case Hunter, Acceptance Auditor). Two HIGH findings and
three MED were FIXED in the story rather than deferred — the trap-binding scope (14 → 24 traps,
Guy's call to close the class), the `family` pin, the false order-rationale doc, the "eleven traps"
figure, and the dhcp-churn D2/D3 MAC pins; those are recorded at their register entries above. Five
items were deferred, all pre-existing or inherent. One finding was dismissed: the claim that
collapsing dhcp-churn D3's MAC onto D1's would red nothing — the pre-existing
`assert_ne!(mac(2), mac(0))` catches exactly that._

- **The `fact()` closure's panic names the observation but never the fact KIND.** `panic!(
  "observation {n} must carry the fact")` is the only signal for a whole class of corpus edits —
  substituting one fact kind for another keeps the per-line count intact and dies here — but it
  does not say whether the missing fact was the `Mac`, `IpV4`, `Hostname` or `Uplink`, so a
  maintainer must read the test to find out which of four closures fired. It is load-bearing rather
  than incidental: `shared-hardware-vm`'s assertion ORDER is chosen specifically so this panic does
  NOT fire before the absence pin. Deferred as pre-existing: the idiom is copied verbatim from the
  4.13 and 4.14 pins and is now shared by ten tests, so fixing it in one story would split it.
  Owner: whoever next touches the byte-pin idiom — thread a `&'static str` label through `fact()`.
- **`N1/N2/N3` labels two different streams.** The pre-existing `dhcp-churn` pin calls its lines
  N1/N2/N3 (the story's own Dev Notes call them D1/D2/D3), and the new `randomized-mac` pin uses
  the same three labels for a different stream. With ten streams pinned, a CI log reading
  `N3 answers to doc-host-hotel` beside `N3 wears 02:00:5e:00:53:21` is the ambiguity the Testing
  standards section exists to prevent. Deferred: the collision predates this story, whose new
  assertions followed the convention already inside that test rather than inventing a second one
  within one function. Owner: whoever next touches the dhcp-churn pin — relabel it D1/D2/D3.
- **Each test's trap-binding block is terminal.** The `read_traps` block is last in every test, so
  a combined stream+`.toml` re-authoring panics on the byte and the binding pin is never evaluated
  — the recorded red names the value, not the inversion. Within one test the second
  `assert_trap_binds` is likewise unreachable once the first reds, which is exactly what this
  story's mutation 6 shows. Deferred: inherent to Task 5b's explicit "fold it in" choice, and the
  alternative (a separate binding test) re-opens the gap where "the family" means two sets. Owner:
  nobody yet — recorded so the next reader does not mistake a single red for a single defect.
- **`the_dhcp_churn_stream_moves_the_address_only_through_observed_at` no longer describes all it
  does** — it now also pins three authored values and the family's two trap bindings. Deferred:
  the register cites that test by name in two closed bullets, so renaming it costs more than it
  returns. Owner: whoever next renames a cited test, together with the citations.
- **The trap `reason` prose is still not mechanically tied to the values it cites** — see the
  `## Deferred from: story-5.2b` section above, where this was already registered by the story
  itself. Repeated here only so this section is not read as the complete review residue.

## Deferred from: story-5.3 (2026-07-28)

_The story wrote a TYPE and its tests, and no engine. Everything below is open because it needs a
producer, a consumer, or a decision that no code yet forces — not because it was skipped._

- **Whether `IdentityAbstentionCause::Ambiguous` must SPLIT into its three D13 rows.** The variant
  covers three conditions over the verdict set — *"a `Decisive`, >=1 `Opposes`"* (the cloned-MAC
  case, architecture.md:971), *"no `Decisive`, >=1 `Supports`, no `Opposes`"* (weak evidence,
  `:972`) and *"`Supports` AND `Opposes`"* (conflict, `:973`) — and an operator reading "ambiguous"
  cannot tell which fired. Not split here because a split with no consumer is symmetry, not
  information, and D16 warns about the opposite failure only (*"if `Ambiguous` means both 'real
  conflict' and 'unmodelled case', it means nothing"*). **Owner: story 5.14**, which owns the
  operator-facing grouping and is the first place a consumer can justify one; 5.5's evidence vector
  is where the distinguishing data would come from.
- **`IdentityAbstentionCause` derives no `Serialize`/`Deserialize`.** Nothing persists a cause: the
  identity link table does not exist. Deriving a wire format for a domain type with no consumer is a
  finding this project has already recorded once (`ScoredRecord`, 4.6a). **Owner: story 5.9**, which
  persists the interface and the identity link, if it persists a cause at all.
- **It derives no `PartialOrd`/`Ord` and has no `Display`.** Nothing orders or keys one — the
  precedent for `Ord` is `Reconciliation::abstentions: BTreeMap<AbstentionCause, usize>`, and no such
  map exists on the identity side. Rendering goes through `page.rs`'s `cause_label` + `t!()` seam
  (Story 3.8), never through `Display`; writing one now would build the wrong seam. **Owner: story
  5.14** for both, together with the two locale keys per variant it would need.
- **`error[E0004]` on a new variant is the mechanism, so `#[non_exhaustive]` is refused — and that
  refusal has a cost worth naming.** `opencmdb-bin` is a different crate, so the attribute would
  force a `_` arm on every downstream match and a new variant would stop breaking its consumers.
  The price is that this enum is a semver hazard for any out-of-workspace consumer. There is none,
  and the workspace is the product. **Owner: nobody** — recorded so a reviewer reading `#[derive]`
  and no attribute sees a decision rather than an omission.
- **No `From`/`Into` bridge between `gap::AbstentionCause` and `IdentityAbstentionCause`, and none
  should appear silently.** The corpus's cause is the trap author's note about the SHAPE of the
  case; the engine's is what the cascade concluded. Mapping one onto the other is a decision about
  the truth format, not a convenience. **Owner: whoever needs the comparison** — as a story, with a
  `must-abstain` corpus consequence spelled out first.

## Deferred from: code review of story-5.3 (2026-07-29)

_Three parallel layers (Blind Hunter, Edge Case Hunter, Acceptance Auditor). All three converged on
a false row count in the enum's guard paragraph, and TWO independently attacked `all()`'s witness —
the Edge Case Hunter by RUNNING the lazy-repair path. The Acceptance Auditor reproduced SIX of the
seven mutations and found not one observed red set differing from the record. Those items were
PATCHED in the story, not deferred; what follows is the residue._

- **`IdentityAbstentionCause::all()`'s witness stops the build on a new variant but does not force
  the variant into the list — MEASURED, and deliberately left open.** Adding a third variant and
  repairing only the `error[E0004]` (a bare `Self::NewThing => {}` arm) leaves `all()` returning the
  old two-element list while `cargo test -p opencmdb-core` reports **90 passed**. No `error[E0308]`
  fires, because repairing the match never touches the literal. **Guy's call, 2026-07-29: keep the
  mechanism, tell the truth about it, register the residue.** Two closures were built and measured
  before deciding, and both were REJECTED:
  · an `ordinal()`/`slot()` witness — **measured GREEN with the variant missing (91 passed)**, so it
    closes nothing: `slot()` is never called on a variant that is not already in `all()`. (This was
    the reviewer's first recommendation and it was withdrawn after measurement, not after argument.)
  · a `macro_rules!` emitting the enum and the list from ONE variant list — **measured to work**
    (`all().len() = 3`, contains the new variant) — but it violates story 5.3's AC1, which binds the
    return type to a literal `[Self; 2]` and forbids `[Self; N]` behind a const; and it makes adding
    a variant *frictionless*, which is the opposite of what AC1b wants (a third variant is a
    FINDING). A `strum` dependency in `opencmdb-core` was refused as disproportionate for a
    two-variant enum (D47 makes that crate's dependency list a decision).
  What shipped instead: `all()`'s doc states the guarantee and its limit, and the witness carries a
  comment naming the wrong repair, so the developer who arrives on the `error[E0004]` reads it.
  **Owner: story 5.14** — the first story with a real reason to add or split a variant, and the
  first place a mechanism would earn its keep.
  ↺ **RE-MEASURED on a second enum by story 5.4, same idiom, same limit.** `Verdict::all()` copies
  this mechanism at five variants; deleting `Verdict::Disqualifying` gives `error[E0599]` at three
  sites (the `all()` literal, the witness match arm, and the one test that spells the variant) and
  **never `error[E0308]`** — the two errors are alternatives along one repair path, never
  simultaneous, exactly as recorded here for two variants. The residue is therefore inherited, not
  duplicated: **this entry owns it for both enums.** Owner unchanged (5.14).
- **Two of story 5.3's four new tests are carried by the type system, not by their assertions.**
  `the_vocabulary_is_exactly_ambiguous_and_absence_of_proof` red under M1 only because it SPELLS
  `Ambiguous` (an `E0599`), and `an_abstention_names_no_rule_whatever_its_cause` red under M5 only
  because the field type changed (`E0308`) — an error that would fire identically if its body were
  `assert_eq!(1, 1)`. `Outcome::Abstained` carries no rule field, so no mutation short of fabricating
  a `RuleId` can make the second assertion fail. AC5 blesses a compile error as a red, so this is not
  a violation; it is recorded because "four tests pin its claims" is carried, for two of the four, by
  the compiler. Owner: story 5.5 — the first story with a producer, where an abstention becomes
  reachable from something other than a literal.
- ✅ **CLOSED by story 5.4**, which is the owner this bullet names — it is the story that next added
  a test to `cascade.rs`, and its AC6 required the convention be *"decided and written down once, in
  one place"*. It is now the test module's own doc comment in `cascade.rs`: **a test lives with the
  item whose CLAIM it pins; the items it merely READS are dependencies, imported and not owned.**
  The convention RESOLVES both cases rather than merely stating a rule: 5.4's mirror test pins a
  claim about `Decision`'s shape (`Outcome` is the dependency it reads) and belongs in `cascade.rs`;
  `an_abstention_names_no_rule_whatever_its_cause` pins a claim about the abstention VOCABULARY's
  relationship to a rule (`Outcome::rule()` is the mechanism it reads to express it) and therefore
  **stays where it is by the convention rather than by fiat**.
  ~~**`an_abstention_names_no_rule_whatever_its_cause` inverts the test-placement convention its own
  story invoked.** Task 5 established that *"a test module tests the items of its own file, importing
  other modules only as dependencies"* and used it to keep the truth-table tests in `score.rs`; this
  test lives in `cascade.rs` and asserts a property of `crate::score::Outcome::rule()`. The SPEC bound
  the placement, so the implementation is conformant and moving it now would deviate from an approved
  AC. Owner: whoever next adds a test to `cascade.rs` — decide the convention once, in one place.~~
- **A trap file may declare `must-abstain = { cause = "OutOfPerimeter" }` and nothing refuses it.**
  `TrapFile::validate` has no arm for the cause's SEMANTICS, so a sha256-locked corpus artefact can
  commit an abstention cause the identity cascade has no row for. Harmless today — scoring is
  cause-blind and no committed trap does it (all three write `NoObservedValue`) — but the field is an
  oracle no code reads. PRE-EXISTING: the hole predates 5.3, which only made the two vocabularies
  distinct enough to notice it. Owner: whoever adds semantic validation to the truth format; it is a
  corpus-format decision, not a code fix.
- **`lib.rs`'s flat `pub use identity::cascade::IdentityAbstentionCause;` has no consumer.** The one
  downstream importer, `trap_gate.rs`, uses the long path. Story 5.3 refuses `Serialize` and `Ord` on
  an explicit "no consumer" argument and then ships a crate-root re-export nothing consumes — the
  same argument, not applied. PRE-EXISTING idiom: `lib.rs` re-exports every module's public surface
  flat, and deviating for one type would be the inconsistency. Recorded so the asymmetry reads as
  inherited rather than chosen. Owner: whoever revisits the crate's re-export policy.
- ✅ **CLOSED by story 5.4.** ⚠️ The bullet asks for *"whoever writes 5.4's **AC7**"*; the wording
  that answers it is 5.4's **AC8**, because 5.4's AC7 turned out to be the register itself. Named
  rather than silently substituted. That AC8 requires *"branch → PR → green CI. The story ends at
  status `review` and the PR open"* and **says nothing at all about the merge** — being silent is
  what resolves the contradiction, not a clause asserting the merge is separate. The wording is
  available to 5.4b–5.14 to inherit.
  ~~**AC7's drafting requires a squash merge inside a workflow that must end at `review`.** It reads
  *"branch → PR → green CI → squash merge, ending at status `review`, never `done`"* — the two halves
  cannot both hold, because the merge is what makes a story `done` in this project. Every Epic 5
  story inherits the wording. Owner: whoever writes 5.4's AC7 — require the PR and stop.~~
- **CI runs no `cargo doc`, so broken intra-doc links are gated by nothing.** `cargo clippy` does not
  check them, and story 5.3 added many full-path `[`crate::...`]` links precisely to avoid unused
  imports. The tree is clean (verified twice, three pre-existing warnings, none in new code) — but by
  measurement on a developer's machine, not by a mechanism. Owner: whoever next touches
  `.github/workflows/ci.yml`; a `cargo doc --workspace --no-deps` with
  `-D rustdoc::broken_intra_doc_links` would need the three pre-existing warnings fixed first.

## Deferred from: story-5.4 (2026-07-29)

_The story wrote five TYPES and their tests, and no algebra. Everything below is open because it
needs a producer, a consumer, or a decision no code yet forces — not because it was skipped.
**Eighteen items: fifteen name a story as owner, ONE names an EPIC (`Epic 6`), two name the
CONDITION that would produce an owner.** The register's idiom allows a condition in place of a name;
what it does not allow is calling one a name — nor calling an epic a story. ⚠️ _(This preamble read
**"fifteen items: thirteen name a story, two name the condition"** until story 5.4's code review
counted them bullet by bullet: twelve named a story, one named `Epic 6`, two named a condition. The
thirteenth "story" was an epic, mis-labelled in the very sentence that forbids exactly that. The
review then added three items — hence eighteen. **Both numbers here were re-counted mechanically
after the last edit, not carried from the sentence they replace.**)_

⚠️ **Two marks, two meanings, stated here because this file now uses both.** A `~~struck~~` bullet
wrapped by `✅ **CLOSED by story X**` is a CLOSED entry. A `~~struck~~` clause **inside** a live
bullet retires only that clause — typically an owner string that moved — and such a bullet often
says *"Not struck"* two lines later, meaning the ENTRY is not struck. Both appear above._

- **`RuleVerdict::evidence` is `Vec<ObsId>`, the smallest shape that is not invented.** The
  architecture requires a firing rule to leave evidence — *"a rule that fires without leaving its
  `rule_id` in the database is a rule we cannot debug in production"* [architecture.md:1309-1310] —
  and **shapes it on none of the five lines that mention the identity link's evidence** (`:978`,
  `:1015`, `:1032`, `:1309`, `:3378`; the last names a `gap/evidence.rs` that does not exist on
  disk). A richer payload — the fact values, the candidate pair, a rendered sentence — is a design
  with no producer. **Owner: story 5.5**, the first story with a rule that fires.
- **Nothing enforces that a verdict which ARGUES leaves non-empty evidence.** `RuleVerdict`'s fields
  are `pub` with no constructor (`ScoredRecord`'s precedent, `:224-233`). A `Neutral` verdict
  legitimately has nothing to show, so the rule is not "evidence is never empty" — it needs a
  producer to state and to red. **Owner: story 5.5.**
- **A `Decision` whose `Conclusion` names a rule ABSENT from its own `verdict_vector` is
  representable, and so is a `Conclusion::Match` with an EMPTY vector.** That is *"merged, with no
  explanation"* — exactly what D13's *"the list of `(rule, verdict, evidence)` IS the explanation"*
  exists to prevent. Not fixed here because the conclusion and the vector are first built together by
  the combining function, which is the only place a test could red. **Owner: story 5.4b.**
  ✅ **CLOSED by story 5.4b, 2026-07-29 — by CONSTRUCTION, and the mechanism is named.**
  `decide(Vec<RuleVerdict>, RulesetVersion) -> Decision` selects the named rule FROM the vector it
  then returns, so a conclusion naming a rule absent from its own vector is unreachable through it;
  and `decide(vec![], _)` falls into the absence-of-proof arm, so a `Match` with an empty vector
  cannot be produced at all. `a_named_rule_is_always_present_in_the_vector_it_travels_with` walks all
  32 verdict subsets and asserts both. ⚠️ **A struct literal built elsewhere is still unconstrained**
  — the fields are `pub` and there is no constructor. **That residue moves to story 5.9**, the first
  story that reconstructs a `Decision` from anywhere other than `decide`.
  *(Raised by the gap-hunt validation agent, 2026-07-29; no AC, doc or register entry had covered it.)*
- **None of the five new types derives `Serialize`/`Deserialize`.** Nothing persists a decision: the
  identity link table does not exist. Deriving a wire format for a domain type with no consumer is a
  finding this project has already recorded once (`ScoredRecord`, 4.6a). **Owner: story 5.9**, which
  persists the interface and the identity link, if it persists a decision at all.
- **`RulesetVersion` derives no `PartialOrd`/`Ord`.** The first consumer that ORDERS two versions is
  persistence — D20's *"existing links are not recomputed (they carry the version they were decided
  under)"* is a claim about which version a row carries, not a comparison anything performs.
  **Owner: story 5.9.** Recorded because "a version feels ordered" is exactly the argument that would
  bend the no-derive-without-a-consumer rule 5.3 set.
- **`RulesetVersion(0)` is constructible and means nothing; no value is refused.** D14's "mandatory"
  is about PRESENCE. Meaning attaches the day a ruleset exists to be versioned; validating a number
  against nothing would be the same invention this story refuses for evidence. **Owner: story 5.5.**
- **There is no `CURRENT_RULESET_VERSION` constant and no `Default` on `Decision`.** There is no
  ruleset: no rule exists, so a constant would assert that the rules it versions are there. The
  absent `Default` is deliberate and load-bearing — it is what makes the version unforgettable
  (measured: removing the field gives five `error[E0560]` plus one `error[E0609]`). **Owner: story
  5.5**, the first story with rules to version.
- **`RuleId` is NOT closed into an enum, although `trap.rs`'s doc predicted Epic 5 would close it.**
  Measured on the committed corpus: `grep -rhoP 'rule\s*=\s*"[^"]+"' fixtures/ | sort -u` returns
  **seven** names — `l1-distinct-mac`, `l1-exact-mac`, `l2-different-hostname`, `l2-different-switch`,
  `l2-hostname-agrees`, `l2-uplink-agrees`, `l2-virtual-mac-prefix` — and **five are `l2-*`**.
  Closing it would enumerate five rules nobody has designed or make five sha256-locked trap files
  unparseable. The doc sentence was corrected in this story rather than left standing.
  **Owner: Epic 6**, which designs the `l2-*` half.
- **No `From<Decision> for Outcome` in either direction.** `Outcome` is the harness's record of an
  answer; `Decision` is the engine's return. Mapping one onto the other is a decision about the
  release gate, not a convenience — the same refusal, for the same reason, that kept the two
  abstention vocabularies unbridged in 5.3. **Owner: story 5.7**, the trap runner consuming a real
  engine.
  ✅ **CLOSED by story 5.7, 2026-08-01 — and the refusal STANDS.** The mapping exists as
  `score::outcome_of`, a named free function; **no `From` impl was added in either direction**, for
  exactly the reason recorded here: a `From` makes the conversion free at every call site (`.into()`),
  which is the invisibility the refusal was about. A named function has to be typed out, so a reader
  of a call site sees that a gate decision was taken. `cascade.rs`'s sentence stating the refusal was
  kept and updated to say the mapping now exists. **Struck.**
- **No `Decision::cause()` and no `Conclusion::rule()`.** `Outcome` has no `cause()` either, and
  nothing groups abstentions by cause until 5.14. `rule()` exists on `Decision` because a consumer
  holds a decision; an accessor on the inner enum would have no caller. **Owner: story 5.14** for
  `cause()`, **story 5.7** for `Conclusion::rule()` if a consumer ever holds a bare conclusion.
  ✅ **The `Conclusion::rule()` half is CLOSED by story 5.7, 2026-08-01, by ANSWERING the condition
  rather than by building it: no consumer holds a bare conclusion.** `outcome_of` takes a
  `&Decision` and matches `&decision.conclusion` in place; `l1_runner` holds a `Decision` and hands
  it straight on. Nothing in the tree destructures a `Conclusion` away from its envelope, so the
  accessor would still have no caller. `Decision::cause()` is untouched and stays with **5.14**.
- **`score::VerdictVectorEntry` and `identity::cascade::RuleVerdict` are two types for one triple.**
  The first is the harness-side placeholder, deliberately **uninhabited** so
  `ScoredRecord::verdict_vector` is provably empty; the second is the engine-side element, with no
  producer. Replacing the placeholder now would falsify four places at once (`score.rs`'s
  "uninhabited" doc, `ScoredRecord::verdict_vector`'s "always empty… provably so",
  `comparable_fields`' "empty on both sides", and `:210-215` of this file) with nothing to justify
  it. **Owner: story 5.7**, when the harness first records a run a real engine produced.
  ⚠️ **RE-OWNED by story 5.7, 2026-08-01, WITHOUT being done — the condition was not met.** The
  harness now scores a run a real engine produced, but it records the run as `Outcome`s, not as
  `ScoredRecord`s, and an `Outcome` has no vector to fill. The obstacle is `ScoredRecord`'s
  `capability_snapshot`. See `## Deferred from: story-5.7` for the measurement and the new owner.
  **Not struck.**
- **`Verdict::all()` inherits the measured lazy-repair residue of `IdentityAbstentionCause::all()`.**
  Same idiom, same limit: the witness stops the build on a new variant (`error[E0004]`) but does not
  force it into the list. **This is a CROSS-REFERENCE, not a second entry**: the measurement itself
  was appended to the `IdentityAbstentionCause::all()` entry in `## Deferred from: code review of
  story-5.3`, which now owns the residue for both enums. **Owner: story 5.14**, there.
  ⚠️ *(This bullet said "Folded into the existing entry at `:1025-1044` rather than duplicated" while
  no such fold had been made — caught by two review layers. The fold has since been performed and
  this sentence now describes it.)*
- **D13's six-row table does not cover every input, and story 5.4 only NAMES the gap.** Enumerated
  over the PRESENCE of each verdict, the table leaves exactly one class unanswered: at least one
  `Opposes`, with no `Decisive`, no `Supports` and no `Disqualifying`. It is not *"only `Neutral` /
  nothing"* and not *"`Supports` AND `Opposes`"*. **Guy's arbitration, 2026-07-29: it concludes
  `Abstained { AbsenceOfProof }`** — nothing argues FOR the merge, so there is no merge to refuse, and
  D13 reserves the refusal-that-names-a-rule for `Disqualifying`. Independently re-derived and
  confirmed by both validation agents. **Owner: story 5.4b**, which writes the function that must be
  total; the correction to D13 itself belongs to a milestone edit of `architecture.md`, never to a
  story.
  ✅ **CLOSED by story 5.4b, 2026-07-29 — implemented, and the compiler now guards the totality.**
  `decide`'s arms are a `match` on the presence tuple `(disqualifying, decisive, supports, opposes)`,
  so a missing class is `error[E0004]` rather than a silent fallthrough. **Measured**: deleting the
  arbitration arm gives *"non-exhaustive patterns: `(false, false, false, true)` not covered"* — the
  compiler NAMES the class. An `if`-chain was measured to swallow the same deletion with all 16
  classes keeping their answer, which is why the construct is binding rather than stylistic.
  ⚠️ **The correction to D13's own table is NOT closed and is not a story's to make.** It is now
  **GitHub issue #54**, alongside #50 — the register is no longer its only record.
- **`Verdict` derives no `PartialOrd`/`Ord`, and that is D20's business, not a story's.** D20:
  *"if strength returns, it returns as an ORDINAL, not a weight: `Opposes(Weak) | Opposes(Strong)`"*
  [architecture.md:1374-1376], under four conjoint conditions demonstrated **before any code**
  [architecture.md:1378-1394]. An orderable `Verdict` would let magnitude compile today, which is the
  move that ADR gates. **Owner: whoever writes D20's ADR** — there is no story, and inventing one
  would be the reintroduction the ADR exists to refuse.
- **`lib.rs`'s flat re-export block grew by five names with no consumer**, aggravating the
  *"`lib.rs` re-exports every module's public surface flat"* entry in `## Deferred from: code review
  of story-5.3` by a measured amount: `Conclusion`, `Decision`, `RuleVerdict`, `RulesetVersion` and
  `Verdict` are all re-exported at the crate root and nothing imports them from there. Following the
  crate's idiom was chosen over deviating for one module, and the cost is recorded rather than
  hidden. **Owner: whoever revisits the crate's re-export policy** (that entry's own wording).
  ↺ **GREW to SIX names by story 5.4b, recorded 2026-07-30 by its code review, not by the story.**
  `decide` joined the same flat block and nothing imports it from the crate root either
  (`grep -rn "decide(" crates/ xtask/` finds no caller outside `cascade.rs`). 5.4b's File List
  claimed *"the cost is recorded on the existing re-export entry"* while this bullet still said five
  and enumerated five — the annotation the story's own binding Dev Note asked for was never written.
  Owner unchanged.

_The three items below were added by story 5.4's CODE REVIEW (2026-07-29), not by the story. They
are in this section rather than in a review section of their own because they are properties of what
the story shipped._

- **A `verdict_vector` may carry the SAME `RuleId` twice, and an `Abstained { Ambiguous }` may carry
  an EMPTY vector. Nothing refuses either.** D13 is *"all rules are evaluated … **each** yields an
  enumerated verdict"* [architecture.md:960] — one verdict per rule — yet
  `vec![RuleVerdict { rule: rule("l1-exact-mac"), verdict: Supports, .. }, RuleVerdict { rule:
  rule("l1-exact-mac"), verdict: Opposes, .. }]` compiles and fabricates D13's *"`Supports` AND
  `Opposes`"* row [architecture.md:973] out of a single rule contradicting itself. Symmetrically,
  *"ambiguous"* with nothing arguing either way is a conclusion none of D13's `Ambiguous` rows can
  produce. Same shape and same reason as the sibling entry above (a conclusion naming a rule absent
  from its own vector): `pub` fields, no constructor, and the only place a test could red is where
  the vector and the conclusion are first built together. **Owner: story 5.4b.**
  ↺ **PARTLY closed by story 5.4b, 2026-07-29 — TOTALITY yes, REFUSAL no.** `decide` answers for a
  vector naming the same rule twice, deterministically, and a test pins it with the register's own
  example: `("a", Decisive)` + `("a", Opposes)` → `Abstained { Ambiguous }`, one rule fabricating
  D13's conflict row on its own. **Refusing it is not closed and cannot be here**: a refusal needs a
  PRODUCER that emits one verdict per rule, and no rule exists. **Owner of that half moves to story
  5.5.** Likewise an `Abstained { Ambiguous }` with an empty vector stays representable via a struct
  literal (story 5.9's residue, above). Not struck.
  *(Found by the Edge Case Hunter at story 5.4's code review; no AC, doc or register entry covered
  it, whereas both neighbouring representable states were already owned.)*
- **Story 5.4 introduced the workspace's FIRST `f32`/`f64` token, and it sits in the subtree story
  5.4b's gate is specified to grep.** Measured: `grep -rn "\bf32\b\|\bf64\b" crates xtask
  --include=*.rs` returns **1** on the story-5.4 branch and **0** on `master`. The single hit is
  `identity/cascade.rs:42`, a **quotation** of D13's own refusal (*"REFUSED: `rule -> confidence:
  f64`"*). **No TYPE carries a float**, so story 5.4's AC3 holds and nothing is wrong in the code —
  but `epics.md`'s story-5.4b criterion says *"a gate reds on any `f32` or `f64` under
  `crates/opencmdb-core/src/identity/`, in the idiom of the existing DDL-collation and
  retired-vocabulary greps"*, and those greps are LINE greps. As specified, the gate reds on day one
  on a citation of the decision it enforces. **Owner: story 5.4b**, at its contexting — whether the
  gate strips `///`/`//!` lines is a design question for the story that writes it, deliberately not
  answered here. **Guy's call, 2026-07-29: record it, do not pre-write the AC.** This line is the
  gate's committed test case either way.
  ✅ **CLOSED by story 5.4b, 2026-07-29.** The `float-free` gate strips from the first `//` to end of
  line before matching, so the citation (now `cascade.rs:52`) is tolerated and is the gate's committed
  negative test case, asserted by name. ⚠️ **The stripping turned out to do far more work than
  tolerating one citation, and the number is worth keeping**: removing it makes the gate report **47**
  offenders on the committed tree, because a story reference in prose — `5.4b`, `4.6a`, `4.7a` — is
  literally a digit-dot-digit, and the gate also matches BARE float literals (`let confidence = 0.85;`
  has no `f32`/`f64` token at all and is the likeliest shape a weight takes). The gate's own doc
  carries its limits, including two false-POSITIVE directions: a float in a block comment, and a
  decimal inside a string literal in code. Neither occurs under `identity/` today.
  ↺ **THREE corrections by the 2026-07-30 code review, appended rather than rewritten above.**
  (1) The line citation in the sentence above says `cascade.rs:52`; it is **`:53`**, and AC8's own
  rule — *"cite entries by TITLE, not by line number… a line citation written here will rot the same
  way"* — is what the sentence broke, one bullet after stating it. The story's Completion Notes say
  `:53`, so the two records disagreed. (2) **The 47 is wrong: the tree it described gave 45**
  (44 in `cascade.rs` + 1 in `mod.rs`, re-measured two independent ways) — 47 was true at the WIP
  commit `1ced9e2` and was never re-measured after the doc pass shortened two prose lines. ⚠️ **And
  45 did not survive this review either**: replacing the matcher with a tokeniser stopped story
  references like `5.4b` from counting, so the same command now returns 42. Three values for one
  sentence inside one story. No figure is quoted in code any more — a test asserts the gap instead,
  because a count in a comment rots and an assertion does not. (3) The limits list was INCOMPLETE in the direction that matters:
  `1e-3` and `1.` are both `f64` and were both GREEN. The matcher is now a numeric-literal tokeniser,
  which also stopped reddening `"192.168.0.1"`, `t.0.1` and `a_f64_never_decides()`.
- **Three documents name story 5.4b owner of the conclusion↔`verdict_vector` coherence invariant,
  and 5.4b's acceptance criteria do not mention it.** The owner is assigned in
  `identity/cascade.rs`'s `Decision` doc, in the sibling entry of this section, and in
  `sprint-status.yaml`; `epics.md`'s `### Story 5.4b` block carries six criteria (a total `decide`,
  the uncovered input class, order-independent rule choice, the float gate, the milli-units
  deferral, totality by exhaustive classes) and **none** covers *"a `Conclusion`'s rule must appear
  in its own `verdict_vector`"* or *"a `Match` may not carry an empty vector"*. Related and
  unanswered in the same block: what `decide` RETURNS (a `Conclusion`? a `Decision`? a `Decision`
  minus its version?) and who supplies `ruleset_version`. **Owner: story 5.4b**, at its contexting.
  **Guy's call, 2026-07-29: hand it over, do not pre-write the AC** — writing 5.4b's criteria from
  inside 5.4 would be the same act 5.4 refused when it declined to write `decide()`.
  ✅ **CLOSED by story 5.4b, 2026-07-29**, which is where the answers were written: `decide` takes
  `(Vec<RuleVerdict>, RulesetVersion)` and returns a `Decision` (its AC1), the version arrives as a
  parameter and is passed through unvalidated, and the coherence invariant is its AC5. The hand-off
  worked as intended — the gap was answered at the contexting that owned it, and `epics.md` was not
  edited by story 5.4.
  *(Found by the Blind Hunter, which had no access to `epics.md`'s history.)*

## Deferred from: code review of story-5.4 (2026-07-29)

_Three parallel layers (Blind Hunter, Edge Case Hunter, Acceptance Auditor), none failed. The scope
held and all eight gates re-ran green; the residue the review found was almost entirely COUNTS AND
CITATIONS in this story's own prose, and it was PATCHED in the story rather than deferred. **One item
is deferred, because it has nothing to fix today.**_

- **The four new tests in `cascade.rs` normalise the exact state story 5.4b is chartered to refuse.**
  Six `Decision` literals across `a_decision_names_a_rule_and_an_abstention_does_not` and
  `the_conclusion_mirrors_the_outcomes_rule_shape` build `Conclusion::Match { rule }` or
  `NoMatch { rule }` with `verdict_vector: Vec::new()` — precisely the *"merged, with no
  explanation"* shape that `Decision`'s own doc and the entry above (a `Conclusion` naming a rule
  absent from its own vector, and `Match` with an empty vector) call the thing D13 exists to prevent.
  **Nothing is wrong today**: there is no constructor, nothing enforces the invariant, and the tests
  pin claims about `rule()` that are indifferent to the vector. But the day 5.4b enforces the
  invariant it is already named owner of, **every one of those six literals has to be rewritten**,
  and a test suite that has to be rewritten by the story that adds a guard is the kind of surprise
  this register exists to remove. Recorded so 5.4b budgets it rather than discovers it.
  **Owner: story 5.4b.** *(Raised by the Blind Hunter, which had no access to 5.4b's charter.)*
  ✅ **CLOSED by story 5.4b, 2026-07-29 — and its own prediction was REFUTED by measurement, which is
  why this says so instead of quietly striking it.** The entry predicted the literals *"must be
  rewritten the day 5.4b enforces the invariant"*. They were not touched: `decide` constrains what IT
  returns, and constrains nothing about a hand-built struct literal, so all four sites compiled
  unchanged and their tests still pass. ⚠️ **The count was also wrong**: the entry says "six"; there
  are **FOUR** literal sites carrying an empty `verdict_vector`, two of them inside loops, giving
  seven constructions at runtime. Neither number is six. Corrected here rather than in place, since a
  bullet is never rewritten.

## Deferred from: story-5.4b (2026-07-29)

_The story wrote the verdict algebra and one gate. It wrote no rule and no producer: nothing emits a
`Verdict`, so nothing calls `decide` outside its own tests. Everything below is open because it needs
a producer, a consumer, or a decision no code yet forces._

**NINE items: SIX name a story, TWO name the CONDITION that would produce an owner, and ONE names a
MILESTONE (GitHub issue #54).** One of the six additionally names **`Epic 6`** as a second owner
alongside story 5.5 — the `l2-*` half of the tiebreak — so "an epic is named" is true while "an item
is owned by an epic" is not; the two are different claims and only the first holds here.

Counted bullet by bullet after the last edit. The split is spelled out to this precision because
story 5.4 shipped *"thirteen name a story"* when one of the thirteen named an epic, in the very
sentence forbidding that — and because a first draft of THIS preamble said "six name a story, one
names an epic, two name the condition", which adds to nine only by counting the epic as its own item
and forgetting the milestone entirely. The arithmetic was wrong before it was measured.

- **`decide` carries no `#[must_use]`.** Discarding its result is always a bug, and the attribute
  would say so — but the workspace carries **exactly one** `#[must_use]` in total
  (`opencmdb-bin/src/main.rs`), so adding one here is the deviation, not the convention. Measured, not
  assumed. **Owner: whoever revisits the workspace's `must_use` policy** — a condition, not a story:
  the decision is about all of `crates/`, not about this function.
- **`Abstained { Ambiguous }` does not record WHICH of D13's three rows produced it.** Three rows
  collapse onto one variant — `a Decisive with >=1 Opposes` (the cloned-MAC case, `:971`), weak
  evidence (`:972`) and `Supports AND Opposes` (conflict, `:973`) — and `decide` throws the
  distinction away. Splitting it would invent a vocabulary D13 does not have. **Owner: story 5.14**,
  the first story that groups abstentions for an operator and so the first with a consumer that could
  justify the split.
- **The lexicographic tiebreak is a placeholder, and it has no semantic content.** When several rules
  are `Disqualifying` (or several `Decisive`), the one named is the smallest `RuleId`. That is
  order-independent and invents nothing, which is why it was chosen — but `l1-distinct-mac` is not
  "more disqualifying" than `l1-exact-mac`, and the day rules have a DESIGNED priority it replaces
  this. **Owner: story 5.5** for the L1 rules, **Epic 6** for the `l2-*` half — this is the item that
  names an epic.
- **The float gate's limits are documented, not closed.** Two false-POSITIVE directions: a float
  inside a block comment `/* … */`, and a decimal inside a string literal in code (`"0.1.1"`). One
  false negative: a `//` inside a string literal truncates the line early. Measured: none of the three
  occurs under `crates/opencmdb-core/src/identity/` today. **Owner: whoever meets one of them on a
  real tree** — a condition, and deliberately so: inventing a Rust-aware scanner for a case that does
  not exist would be the over-engineering the reflex-gate idiom (D53) refuses.
- **The gate scopes to `identity/` and no wider.** `opencmdb-bin` may legitimately want a float for a
  UI ranking one day — D13 permits *"floats may RANK, never DECIDE"* [architecture.md:988-990] — so
  widening this gate to the workspace is a different decision with a different blast radius.
  **Owner: story 5.14**, the first story with a ranking surface.
- **No milli-unit type, constant or field exists.** D13's corollary — *"`confidence` is an INTEGER in
  milli-units (0..1000), never `REAL`/`DOUBLE`"* [architecture.md:991-993] — binds the day a ranking
  value appears. Epic 5's L1 is a deterministic lookup with nothing to rank, so a `0..1000` integer
  here would be a value asserting that a ranking exists. **Owner: story 5.14**, the Resolve panel.
- **An incoherent `Decision` is still buildable by struct literal.** `decide` makes "a conclusion
  naming a rule absent from its own vector" and "a `Match` with an empty vector" unreachable *through
  the function*; the fields are `pub` with no constructor, so a literal built anywhere else is
  unconstrained. **Owner: story 5.9**, the first story that reconstructs a `Decision` from somewhere
  other than `decide` (persistence).
- **Refusing a `verdict_vector` that names the same rule twice needs a producer.** `decide` is TOTAL
  over it — measured and tested — but total is not the same as validated: "one verdict per rule" is a
  claim about what a PRODUCER emits, and no rule exists to emit anything. **Owner: story 5.5.**
- **D13's own table in `architecture.md` is still short one row.** The arbitration is implemented and
  documented at `decide`, but the decision body has not been corrected, and correcting a locked
  planning document is a MILESTONE act, never a story task (`epics.md:1461`). **Owner: a milestone
  edit, carried by GitHub issue #54** — which also records that `architecture-views.md` must be
  regenerated in the same pass (issue #50). Not a condition and not a story: an issue.

## Deferred from: code review of story-5.4b (2026-07-30)

Five items, from a three-layer review (Blind Hunter · Edge Case Hunter · Acceptance Auditor). All
five are **repo-wide or producer-blocked**, which is why they are deferrals and not patches: the
review's 23 patch items are in the story file. Two of them arrived as HIGH findings against 5.4b and
were **moved here by re-measurement** — recorded that way on purpose, because a deferral that hides
why it was downgraded is worse than no deferral.

- **Four `cargo xtask ci` gates swallow every `walkdir` error.** `filter_map(Result::ok)` at
  `xtask/src/main.rs:105` (`gate_file_size`), `:395` (`gate_float_free`), `:439`
  (`gate_ddl_collation`) and `:557` (`gate_vocabulary`): an unreadable subdirectory, a metadata
  failure or a loop drops its files from the walk, the `checked` count shrinks with them, and the
  gate reports a pass over a tree it did not fully see. ⚠️ **The Edge layer raised this as HIGH
  against story 5.4b**, on the true observation that the same file refuses this failure mode by name
  at `corpus_entries` (`:753-806`: *"a walk whose failure mode is 'quietly saw less of the tree' is
  not a gate"*). Re-measurement moved it: **three of the four sites predate 5.4b**, and AC6 told the
  story to write the gate "in the idiom of its two neighbours" — which it did exactly. So this is not
  a defect the story introduced, and fixing only the newest gate would be the least useful version of
  the fix. **Owner: a condition** — the next story that touches `xtask`'s gate plumbing, or a chore
  PR; whoever takes it should convert all four together and mirror `corpus_entries`' `with_context`.
- **The gates do not follow symlinks and do not report them.** No `.follow_links(…)` on any gate's
  `WalkDir`, so a symlinked subdirectory under a guarded tree is yielded as one entry and never
  descended; a module compiled in via `#[path = "…"]` or an `include!("…")` is likewise outside the
  walk, and the extension filter is `.rs`-only. Asymmetric with `corpus_entries`, which refuses to
  skip a symlink in silence (`CorpusEntry::Symlink`, `:744-747`). Same repo-wide shape as the entry
  above and best taken in the same pass. **Owner: a condition** — the same pass that fixes the walk
  errors.
- **Nothing refuses a blank `RuleId` on the `RuleVerdict` side.** `RuleId(pub String)` derives `Ord`
  (`trap.rs:39-41`), so `decide`'s tiebreak is byte order and `RuleId("")` sorts before everything:
  a verdict vector carrying one yields `Conclusion::NoMatch { rule: RuleId("") }`, on which
  `Decision::rule()` still answers `Some` — so "every decision names a rule" degenerates into naming
  nothing, while `Trap::validate` already refuses a blank rule on the expectation side
  (`trap.rs:302`). Not reachable today because nothing produces a `RuleVerdict` outside tests.
  **Owner: story 5.5**, which is where the first producer appears and therefore the first place a
  validation can be stated about what a producer emits.
- **A named rule with EMPTY `evidence` yields a `Match` that explains nothing.** `decide` never
  inspects `evidence`, so `RuleVerdict { verdict: Decisive, evidence: vec![] }` produces a `Match`
  defeating D13's *"the list IS the explanation"* one level below the empty-**vector** case story
  5.4b genuinely closed by construction. The story records the invariant as needing a firing rule to
  state, and that is right — this entry only pins that the two emptinesses are different and that
  `Decision`'s doc currently closes them in one sentence (the doc half is a patch item in the story).
  **Owner: story 5.5.**
- **The `xtask` module doc's list of gates is not itself gated, and has already drifted once.**
  Evidence rather than prediction: story 5.4b had to ADD the `file-size` entry to that list, which
  means the story that shipped `file-size` left it out and CI never noticed. Nothing checks that the
  list of gates in the module doc matches the gates `run_ci` actually runs, so the sixth gate's entry
  will rot the same way. **Owner: a condition** — the same `xtask` pass as the walk items; the cheap
  version is a test asserting the doc names every gate `run_ci` calls.

## Deferred from: story-5.5 (2026-07-31)

_Story 5.5 shipped the L1 join, the two rules the committed corpus names, and the first caller of
`decide` outside its own tests. The eight requirements the register owed this story are dispositioned
below **by requirement, not by section** — the same requirement was registered at up to four sites,
because the register is append-only and chronological. Nothing above is struck: the annotations are
appended here._

**Enumerated with `grep -n '5\.5' deferred-work.md` — NINETEEN lines.** `grep -n 'story 5\.5'` gives
eleven and misses two owner strings that wrap across a newline. That is the undercount that made
story 5.4b claim eight register entries where ten existed, so the wider grep is the enumeration.

### Dispositioned by this story

⚠️ **FOUR closed, FOUR open.** An earlier version of this section reported five closed; story 5.5's
code review downgraded R7 to partial, because the entry is titled after two tests it does not touch.
The correction is made in place below rather than appended, since this section is 5.5's own and
nothing on `master` is rewritten by it.

- ✅ **R1 — a firing rule leaves its `rule_id` AND its evidence, with a test that reds. CLOSED for
  the L1 producer, by a mechanism.** Registered at four sites: *"The firing-rule contract (AC6) is
  RECORDED, not built"* (§story-4.7a) and its owner-moving annotation; *"`RuleVerdict::evidence` is
  `Vec<ObsId>`, the smallest shape that is not invented"* and *"Nothing enforces that a verdict which
  ARGUES leaves non-empty evidence"* (§story-5.4); *"A named rule with EMPTY `evidence` yields a
  `Match` that explains nothing"* (§code review of story-5.4b).
  `identity::l1::verdict_for_pair` carries both sides' `ObsId`s on every verdict that ARGUES and
  none on a `Neutral` — so the invariant is **not** "evidence is never empty", exactly as the
  register insisted. `an_arguing_verdict_never_ships_empty_evidence` reds when it stops: **measured,
  mutation M3, three assertion-carried reds.** ⚠️ **Scope of the closure:** this holds for what the
  L1 producer emits. `RuleVerdict`'s fields are `pub` with no constructor, so a struct literal built
  elsewhere is unconstrained — that residue is story 5.9's and is NOT closed here.
- ✅ **R4 — the `RulesetVersion` constant and what `0` means. CLOSED.** Registered at
  *"`RulesetVersion(0)` is constructible and means nothing; no value is refused"* and *"There is no
  `CURRENT_RULESET_VERSION` constant and no `Default` on `Decision`"* (both §story-5.4).
  `identity::l1::CURRENT_RULESET_VERSION = RulesetVersion(1)` — it lives beside the rules it
  versions, not in `cascade.rs`, so the value asserting "these rules are there" sits where they are.
  **No `Default` was added.** No value is refused, and the story states the weaker true thing
  instead: the emitted version is not the meaningless zero (`the_ruleset_version_is_not_the_
  meaningless_zero`).
  ⚠️ **The `error[E0560]`/`error[E0609]` figure in the older entry is a story-5.4-era measurement and
  is stale.** On this tree, deleting the field gives **six `E0560` plus two `E0609`** under
  `cargo check -p opencmdb-core --tests`; the entry said five plus one, which was true before
  `decide` and its tests existed. Recorded rather than edited, per append-only.
- ✅ **R6 — the producer emits a canonical `RuleId`. CLOSED for the producer half.** Registered at
  *"`(verdict, rule)` comparison is whitespace/case-sensitive, no normalization"* (§code review of
  story-4.7a), deferred to this story by name at story 5.4. `L1_EXACT_MAC` and `L1_DISTINCT_MAC` are
  spelled exactly as `fixtures/scenario/traps/*.toml` spells them, and
  `the_producers_rule_ids_are_canonical` asserts each equals its own `trim()` and its own
  `to_lowercase()`.
  🔑 **The guard that makes this real is that the tests restate the two ids as INDEPENDENT string
  literals** (`CORPUS_EXACT_MAC`, `CORPUS_DISTINCT_MAC`), labelled as protected deliberate
  redundancy. This was measured, not assumed: story 5.5's gap-hunt validation agent built the
  self-referential version — every expectation derived from the constant it checks — and mutating the
  constants to `"L1-Exact-MAC "` and `"l1_distinct_mac"` left its **entire suite green**. On the
  shipped tests the same mutation (M6) reds **ten** tests. ⚠️ The `run_trap` comparison half stays
  with **story 5.7**; `run_trap` is in `score.rs`, whose code this story does not touch.
  ✅ **The `run_trap` comparison half is CLOSED by story 5.7, 2026-08-01** — not by normalizing the
  comparison, but by proving normalization unnecessary on the committed bytes.
  `l1_runner`'s `the_producers_rule_ids_are_the_corpus_spelling` walks every committed trap file
  through `trap_gate::discover_trap_files(root)` and asserts that **all seven** ids the corpus
  writes — `l2-*` included — equal their own `trim()` and their own `to_lowercase()`, so an
  unnormalized `RuleId` comparison cannot produce a false wrong-rule failure there. ⚠️ **What that
  does NOT close**: `run_trap` still compares raw strings, so a FUTURE non-canonical id on either
  side would still be a *"red gate on a correct answer"*. The test is what makes that a red rather
  than a surprise.
- ⚠️ **R7 — story 5.3's two compiler-carried tests. The OWNER CLAUSE is discharged; the entry's
  TITLE is not. Reported as partial, after the code review measured the overstatement.**
  Registered at *"Two of story 5.3's four new tests are carried by the type system, not by their
  assertions"* (§code review of story-5.3), whose owner clause reads *"the first story with a
  producer, where an abstention becomes reachable from something other than a literal"*. That is now
  true: `identity::l1`'s `a_produced_abstention_names_no_rule` reaches
  `Abstained { AbsenceOfProof }` through `decide_pair` from two real observations, and asserts
  `Decision::rule() == None`. ⚠️ **It does not retro-fit 5.3's two tests**, which keep their narrower
  subjects and their honest limits; it adds the behavioural reach the entry asked for.
  🔴 **And that is why this is NOT closed.** The entry is TITLED after those two tests, and by its
  own title they are still carried by the compiler. AC8 offered two ways out — *"either make one
  behavioural now, or write the weaker true sentence explaining why it still cannot be"* — and this
  story took neither: it added a third, new test instead. **The weaker true sentence, owed and now
  written:** `an_abstention_names_no_rule_whatever_its_cause` reds under its mutation only because
  the field TYPE changes (`E0308`), and `Outcome::Abstained` carries no rule field, so **no mutation
  short of fabricating a `RuleId` field can make its assertion fail** — a producer does not change
  that, because the test's subject is the VOCABULARY, not a produced value. Making it behavioural
  needs a type that can carry a rule on an abstention, which the design refuses on purpose. **Owner:
  nobody — it cannot be made behavioural, and this sentence is the disposition.** The sibling,
  `the_vocabulary_is_exactly_ambiguous_and_absence_of_proof`, stays compiler-carried by design too:
  its `E0599` IS the guard.
  _(This entry was reported ✅ CLOSED until story 5.5's code review measured the gap between the
  owner clause and the title. Corrected in the same story, not carried forward.)_
- ✅ **R8 — the `NoMatch` PRODUCER half. CLOSED.** Registered at *"The `NoMatch → Refused` vs
  `Abstained` question is Epic 5's, not scored here"* (§story-4.7a), whose last annotation reads
  *"Producing one still needs a rule (story 5.5) and mapping one onto `Outcome` still needs story
  5.7."* `l1-distinct-mac` emits `Disqualifying`, so a key mismatch reaches
  `Conclusion::NoMatch { rule }` and NAMES its rule. **The derivation was confirmed by running its
  counter-hypothesis, not by argument** (mutation M2): with `Opposes` instead, the conclusion is
  `Abstained { AbsenceOfProof }` and `Decision::rule()` is `None` — which names no rule and would
  make story 5.7's comparison unsatisfiable. ⚠️ **The `Decision → Outcome` mapping half is
  untouched** and stays with story 5.7.
  ✅ **The mapping half is CLOSED by story 5.7, 2026-08-01** — `score::outcome_of`. And the
  counter-hypothesis this entry ran is now measured end to end: the six committed `must-not-merge`
  traps L1 answers reach `Outcome::Refused { l1-distinct-mac }` and PASS both the truth table and
  the rule, so `Decision::rule()` being `Some` is what makes the comparison satisfiable in practice
  and not only in the type.

### Open, with what this story measured about them

- ⚠️ **R3 — "one verdict per rule" is stated but TRIVIALLY true here, so the half with content stays
  open.** Registered at the duplicate-rule annotation (§code review of story-5.4, *"Owner of that
  half moves to story 5.5"*) and at *"Refusing a `verdict_vector` that names the same rule twice
  needs a producer"* (§story-5.4b). The L1 producer's body emits **exactly one** verdict for one
  pair, so it cannot duplicate a rule — and therefore cannot exercise the refusal either.
  `the_producer_emits_exactly_one_verdict_for_one_pair` asserts the shape, but no mutation available
  at L1 makes it red for the right reason. **Owner of the remaining half: Epic 6**, the first
  producer that emits several verdicts into one vector. Recorded as NOT closed rather than reported
  closed, because a trivially-true assertion is not a guard.
- ⚠️ **R2 — a blank `RuleId` is ASSERTED, not refused, and the difference is deliberate.** Registered
  at *"Nothing refuses a blank `RuleId` on the `RuleVerdict` side"* (§code review of story-5.4b).
  A runtime refusal has **no reachable branch**: the entry point returns a `Decision` and not a
  `Result`, and the ids come from constants, so a `panic!` would sit on a dead arm and adding a
  `Result` would contradict the story's own AC. What ships is the testable form —
  `every_emitted_rule_id_is_non_blank` asserts the emitted ids are non-empty after `trim()`,
  mirroring `Trap::validate`'s `rule.0.trim().is_empty()` on the expectation side. **Measured,
  mutation M5: seven assertion-carried reds.** The TYPE-level refusal, for a `RuleVerdict` built by
  struct literal anywhere, is not closed and belongs with story 5.9's constructor residue.
- ⚠️ **R5 — the lexicographic tiebreak keeps its placeholder, because L1 supplies no TIE.**
  Registered at *"The lexicographic tiebreak is a placeholder, and it has no semantic content"*
  (§story-5.4b), owner *"story 5.5 for the L1 rules, Epic 6 for the `l2-*` half"*. The L1 producer
  emits one verdict per pair, so its two rules never appear in one vector and the tiebreak is **never
  consulted on an L1 decision**. Designing a priority between them would order two things that
  cannot meet. The placeholder and its three measured costs stand; **the first vector that can hold
  two verdicts is Epic 6's**, and that is where a designed priority gets an input.

### New, raised by this story

- **`Verdict::Supports` and `Verdict::Opposes` have no producer.** L1 emits three of the five
  variants — `Decisive`, `Disqualifying`, `Neutral`. **Two, not three**, remain producerless, and
  `cascade.rs`'s corrected docs now say so by name rather than claiming "rules produce verdicts"
  flatly. **Owner: Epic 6**, whose `l2-*` rules are where an argument that neither settles nor
  forbids first appears.
- **The `identity::l1` producer is reachable from nothing but its own tests.** It is the mirror of
  the situation `decide` was in before this story: a producer exists, and no caller in the shipped
  binary reaches it. `score_corpus`'s `answers` parameter is still fed an empty map, so every trap is
  still "scored by nothing". **Owner: story 5.7**, which crosses that seam — and the crate frontier
  (D47) is why this story could not.
  ✅ **CLOSED by story 5.7, 2026-08-01.** `opencmdb_bin`'s `l1_runner` calls
  `identity::l1::decide_pair` on the pair each committed trap names, maps the result through
  `score::outcome_of`, and fills `score_corpus`'s `answers` map. The committed corpus reports
  `discovered=24, scored=13, failures=0, rule_mismatches=[], passed=true` — `scored` had read **0**
  since story 4.6b. **Struck.**
- ⚠️ **`xtask/src/main.rs`'s dotted-quad assertion carried a PREDICTION about this story, and the
  prediction did not come true.** Its message read *"story 5.5 writes IP literals under the guarded
  subtree"*. **Measured on the shipped tree: zero `Ipv4Addr` and zero dotted quads under
  `identity/`** — the L1 key is a MAC and an opaque domain id, so no IP literal was needed. The
  message was corrected to the rationale that is true regardless (a dotted quad has three dots and is
  not a numeric literal). Recorded because a green-case rationale that names a future story is a
  claim with an expiry date, and this one expired. **No owner — closed by the correction.**
- ⚠️ **Flagged FORWARD for story 5.6, unsolved here:** `epics.md:1507` gives the blocker the
  assertion `blocking_recall >= 0.999`. That is a bare float literal, and **the `float-free` gate
  reds on it** — confirmed by measurement during this story's validation, not by reading the gate.
  If the blocker lives under `identity/`, story 5.6 trips the gate on its first assertion. **Owner:
  story 5.6**, at contexting, so it is a decision and not a surprise.

## Deferred from: code review of story-5.5 (2026-07-31)

_Three layers (Blind Hunter · Edge Case Hunter · Acceptance Auditor). Six HIGH after deduplication;
all six were PATCHED in the story rather than deferred, except the one below that needs a decision.
Two findings arrived independently from two layers — the evidence asymmetry and the self-pair — and
that agreement is why they were fixed rather than argued. The items here are the residue: what the
patches did not close, and what the review revealed about claims rather than code._

- 🔴 **L1 treats a GROUP address (multicast or broadcast) as an ordinary interface identity, and
  that is an unexamined class, not a position.** The I/G bit — bit 0 of the first octet — marks an
  address that names no interface; `opencmdb-bin`'s corpus privacy scan states exactly that
  (`fixtures.rs`, `is_multicast_mac`: *"Set means the address is a group (multicast or broadcast)
  address, **which names no interface**"*), written in this same epic by story 5.2. `identity::l1`
  does not consume the reading: three observations reporting `ff:ff:ff:ff:ff:ff` join into ONE group
  and any pair of them concludes `Match`. **The failure mode is a FALSE MERGE**, which is what NFR4
  exists to prevent.
  Measured at the review: the committed corpus **cannot** catch it — 14 replay streams, 39 distinct
  MACs, **zero** with the I/G bit set and zero all-zero — and the privacy scan now refuses to let one
  be committed. Measured also: refusing group addresses at L1 would red **no** committed trap, because
  the two traps that forbid a structural L1 refusal (`randomized-mac`, `vrrp-virtual-mac`) involve no
  I/G-set address. So the two questions are separable.
  **Guy's decision, 2026-07-31: DOCUMENT and REGISTER, do not filter.** Consuming a structural
  reading at L1 is precisely what story 5.5's own module doc says L1 does not do, and reversing that
  is a decision with a story behind it, not a patch inside a review. The current behaviour is pinned
  by `a_group_address_merges_today_and_that_is_unresolved` so it cannot change in silence, and the
  module doc names the class as a recorded gap rather than a reasoned position.
  **Owner: a decision, at Epic 6's contexting** — the first epic with an `l2-*` vocabulary in which a
  group-address refusal could be a named rule rather than a silent filter. Whoever takes it should
  note that a trap would have to be authored for it, since the corpus cannot express the case today.
- ✅ **`keys_of`'s `_ => None` catch-all — CLOSED in the review, and it was measured first.** The
  match is now exhaustive over `Fact`, in the idiom `fixtures.rs`'s `assert_facts_are_synthetic`
  established (*"Exhaustive on purpose — no `_` arm … a new variant carrying an address must break
  THIS test and force a decision"*). `Fact` is `#[non_exhaustive]` but that is inert inside its own
  crate, so the compiler can force the decision and now does. **Why it mattered: with the catch-all,
  adding `Fact::Uplink { peer_mac }` as an identity key left the ENTIRE workspace green** — a switch
  merging with everything plugged into it. Now pinned twice: by the exhaustive match (compile error
  on a new variant) and by `a_peers_mac_is_not_this_devices_identity` (mutation M8, 1 assertion red).
- ✅ **Evidence was not symmetric — CLOSED by normalising.** `decide_pair(a, b) != decide_pair(b, a)`:
  the conclusion agreed, the `verdict_vector` did not, because evidence carried argument order
  (`[1,2]` vs `[2,1]`) and `Decision` derives `PartialEq`. A candidate pair is unordered, so any
  downstream dedup or "have we decided this pair already" check would have seen one logical pair as
  two. Evidence is now sorted, by the same reasoning that made [`join`]'s value a `BTreeSet`: the
  property holds by construction. Mutation M7 reds `a_pair_decides_the_same_whichever_side_is_the_
  left_argument`. **Found independently by two review layers.**
- ⚠️ **`verdict_for_pair(a, a)` — the self-pair is answered but undocumented, and no test covers it.**
  Measured: `Decisive` with `evidence = [x, x]` (now `[x, x]` still, since sorting two equal ids
  changes nothing), i.e. the doc's *"both observations' `ObsId`s"* degenerates to one id listed twice.
  The answer is defensible — an observation is trivially its own interface — but `decide_pair`'s doc
  tells a future candidate generator that the pair *"arrives as an argument"* without telling it that
  excluding `i == j` is the generator's responsibility. **Owner: story 5.6**, which writes that
  generator and is the first place the precondition has a holder. Not patched here because inventing
  a refusal for a caller that does not exist is the shape this project refuses.
- ⚠️ **The rule id `l1-distinct-mac` fires on pairs whose MACs are IDENTICAL.** The condition is
  distinct KEY, not distinct MAC: two observations carrying the same address in different
  `l2_domain`s share no key, so the rule names MAC-distinctness for a case where the MACs are equal.
  The id is the corpus's and story 5.5 does not own it, so the overload is documented at
  `verdict_for_pair` rather than renamed. Invisible on the committed corpus, where every stream
  carries one `l2_domain` (D61). **Owner: story 5.7**, which compares this id against the corpus
  bytes and is where the collision would first show; if a trap ever separates the two cases, either
  the trap or the producer has to move.
  ↺ **The COMPARISON exists since story 5.7, 2026-08-01; the OVERLOAD does not. Not struck.** The
  byte comparison pins the SPELLING and says nothing about the condition, and it cannot: no
  committed trap separates *"different MAC"* from *"same MAC, different `l2_domain`"* — every
  committed stream carries one `l2_domain` (D61), so the two are indistinguishable on the corpus.
  **Owner: the story that commits a cross-domain trap**, which is the first place the collision can
  show. Recorded so the closure of the comparison is not read as the closure of this.
- ⚠️ **The doc claims the two rule ids match `fixtures/*.toml` byte for byte, and nothing in this
  crate checks it.** The test-side redundancy (`CORPUS_EXACT_MAC`/`CORPUS_DISTINCT_MAC`) catches a
  rename of ONE constant — verified, mutation M6 reds ten tests — but cannot catch **both** literals
  being wrong relative to the TOML, which is what the doc asserts. Story 5.5 may not read `fixtures/`
  (its own AC8), so the check cannot live here. **Owner: story 5.7**, which reads the corpus and is
  the natural home for the comparison. Recorded because the module doc states the stronger property
  than the redundancy delivers. *(Verified by hand at the review: the corpus spells 7 × `l1-exact-mac`
  and 6 × `l1-distinct-mac`, matching the constants.)*
  ✅ **CLOSED by story 5.7, 2026-08-01.** `l1_runner`'s
  `the_producers_rule_ids_are_the_corpus_spelling` reads the TOML and compares it against the two
  constants: every id beginning `l1-` is one of them, **both** occur (so the assertion cannot pass by
  finding none), and the walk asserts it found at least one rule id at all. It is the THIRD
  independent statement of the two ids — beside the constants and `l1.rs`'s test-side literals — and
  it is the one that catches BOTH literals being wrong relative to the corpus, which neither of the
  other two can. `l1.rs`'s doc was rewritten to say a test holds the claim rather than to assert it.
  **Struck.**
- ⚠️ **The `&str` rule-id constants make every call site perform the `RuleId` wrap.** `RuleId` is not
  const-constructible, which the doc explains — but it stops there and does not weigh
  `fn l1_exact_mac() -> RuleId`, a `LazyLock` static, or a `Cow<'static, str>` payload. As shipped,
  the public surface hands out a raw `&str` and asks each caller to wrap it correctly, which is a
  weaker version of the inconsistency the constants exist to prevent; it also allocates once per
  verdict, on a function a blocker will call O(pairs) times. **Owner: a condition** — whoever first
  measures the blocker's allocation profile, or the story that closes `RuleId` into an enum (Epic 6),
  whichever comes first.
- ⚠️ **`L1Key` is a bare tuple alias, so the "`vantage` is not in the key" warning is unenforceable.**
  `pub type L1Key = (L2DomainId, MacAddr)` creates no distinct type, carries no invariant, hosts no
  impl, and freezes arity at every construction and lookup site. A struct would make the doc's
  warning a type-level fact instead of prose. **Owner: story 5.9**, the first story to persist a key
  and therefore the first with a reason to give it a name that survives a schema.
- 🔴 **Three of the review's findings were about CLAIMS, not code — and all three were mine.**
  Recorded because this is the fourth consecutive story in which the completion record over-claimed,
  and the pattern is now the most reliable defect in this project's process:
  (1) `cascade.rs`'s corrected `evidence` doc asserted the L1 producer *"fills this with both sides'
  `ObsId`s"* **unqualified** — false for a `Neutral`, and the same story's `l1.rs` says so correctly
  three files away; (2) the File List claimed the gap between 12 corrected claims and 17 diff hunks
  was `cargo fmt` reflow — **`rustfmt --check` on master's `cascade.rs` exits 0**, so `cargo fmt`
  contributed nothing and all 17 hunks are content; (3) the Debug Log tabulated six observed mutation
  counts and commented only on the one that matched its prediction, leaving four divergences unstated.
  All three are corrected in the story. **No owner — the lesson is the entry.**

## Deferred from: story-5.6 (2026-08-01)

_The blocker and the recall assertion. Two register entries named this story as owner and both are
disposed of below; **four** new ones are opened. Nothing that belongs to another story is closed
here. (This sentence read "three" until this story's code review counted the bullets under
`### New, raised by this story` — the fourth entry was legitimate and carried an owner; only the
count was never re-measured after the last edit, which is inherited lesson 2 in this very story.)_

### Dispositioned by this story

- ✅ **R1 — the float, CLOSED by expressing the floor as an INTEGER in per-mille.** Registered at
  *"Flagged FORWARD for story 5.6, unsolved here"* (§story-5.5, *New, raised by this story*), which
  measured that `epics.md`'s `blocking_recall >= 0.999` reds the `float-free` gate. What ships is
  `BLOCKING_RECALL_FLOOR_PER_MILLE: u32 = 999` and a `blocking_recall_per_mille` that compares
  integers. **The gate was not weakened, skipped, `#[allow]`ed or narrowed** — it walks **4** `.rs`
  files under `identity/` now where it walked 3, and `cargo xtask ci` reports six green gates. Two
  grounds, neither invented here: D13's own milli-units corollary [architecture.md:988-993], and the
  architecture's ratified test name `blocking_recall_above_999` [:2954], which already carries no
  float and is used verbatim. ⚠️ The float has not been *avoided*, it has been *typed*: the value
  D13 writes is unchanged.
- ✅ **R2 — the self-pair, CLOSED in the TYPE rather than in a comment.** Registered at
  *"`verdict_for_pair(a, a)` — the self-pair is answered but undocumented, and no test covers it"*
  (§code review of story-5.5), owner *"story 5.6, which writes that generator and is the first place
  the precondition has a holder"*. `CandidatePair::new(a, a)` returns `None`, so a pair that comes
  out of the blocker can never be a self-pair, and `decide_pair`'s doc now names the holder instead
  of leaving the precondition ownerless. `verdict_for_pair`'s own behaviour on `(a, a)` is
  **unchanged** — it answers `Decisive` when the observation carries a MAC and `Neutral` when it
  carries none (so `decide_pair` returns `Abstained { AbsenceOfProof }` there, not a merge), which
  is defensible and is not this story's to revisit. _(This sentence said "it still answers
  `Decisive`" full stop until this story's code review read the `Neutral` guard at `l1.rs:252-258`
  — the unconditional form was false for every MAC-less observation, and the same overclaim had
  been written into two doc comments.)_ Pinned by `the_self_pair_is_refused`; mutation M3 (admitting the self-pair) reds it and
  `a_repeated_obs_id_yields_no_pair`, **2 assertion-carried reds**.

### Read, and deliberately NOT closed — they belong to others

- **`L1Key` is a bare tuple alias** — owner **story 5.9**. This story neither renames nor wraps it.
  `CandidatePair`'s private ordered fields are the same *argument* applied to a different type, and
  the doc says so, but the entry is about `L1Key` and stays open.
- **The `&str` rule-id constants allocate** *"on a function a blocker will call O(pairs) times"* —
  owner **a condition**. The condition has NOT been met: `candidates` calls no rule at all, so
  nothing yet calls `verdict_for_pair` per pair and no allocation profile has been measured. The
  entry stays open with its owner unchanged.
- **The group-address gap** (Epic 6) and **`RuleId` → enum** (Epic 6) — untouched. The blocker reads
  no `Fact` and consumes no structural reading of a MAC.

### New, raised by this story

- ⚠️ **D17's `dormant` exclusion is NOT implemented, because no lifecycle state exists.**
  [architecture.md:1205-1206] says *"the blocker excludes `dormant` from automatic candidate
  generation"*. **Measured: `grep -rn 'dormant\|Dormant' crates/ xtask/ --include=*.rs` returns
  nothing** — there is no lifecycle state, no `presence` level (D17 refuses one at
  [:1171-1173]), and no field a blocker could read. Writing the exclusion today would mean inventing
  the state it filters on, which is writing from belief (D45). **Owner: the lifecycle epic
  (FR40-42)**, which is the first place `dormant` becomes a value rather than a word. Whoever takes
  it inherits a live consequence: adding the exclusion makes the universe non-total, so
  `blocking_recall_above_999` becomes the assertion that says whether the narrowing was safe — which
  is exactly the order D13 wanted.
- ⚠️ **The universe is quadratic in the slice the CALLER supplies, and nothing yet bounds that
  slice.** D13's *"90k pairs is noise on a NAS i5"* [architecture.md:1009] is one poll of 300 hosts.
  Every caller today hands `candidates` one replay stream, so the figure holds — but the day a caller
  hands it a retention window instead of a poll, `n` is no longer host count and a narrowing key
  becomes required. **The recall assertion is what would make that narrowing safe rather than
  silent**, and that is the whole reason it is written before any caller exists. **Owner: 5.9 or 5.7,
  whichever first hands the blocker something other than one poll.**
  ↺ **NOT story 5.7, measured 2026-08-01: it hands the blocker nothing at all.** `l1_runner` never
  calls `candidates` — a trap NAMES its pair, so there is nothing to generate. The owner clause is
  therefore unchanged in substance and 5.7 drops out of it: **owner: 5.9 or Epic 6, whichever first
  hands the blocker a set of observations.** Not struck.
  Recorded rather than built for:
  a bound with no measured caller would be a guess at which key to narrow on, and §5 of this story is
  the measurement of how wrong that guess can be while staying green.
- ⚠️ **`>= 999` per-mille is zero-tolerance BELOW 1000 required pairs and a real tolerance from
  1000 onwards.** The boundary is `>= 1000`, not `> 1000`: at exactly 1000 required pairs one miss
  scores 999 and `999 >= 999` passes. _(This entry said "above it" until this story's code review did
  the arithmetic.)_ At the committed denominator of 10, one miss scores 900 and the floor reds — binary, which is
  the form NFR4 demands. Past 1000 required pairs the same constant would admit a genuine miss, and
  NFR4's *"any fraction is theatre"* [prd.md:1182] would bite. The module doc states this rather than
  letting the per-mille dress imply a tolerance the corpus cannot support. **Owner: the story that
  first grows the truth set past that size** (Tier 2, Epic 11+). It is not a defect today and must
  not be "fixed" pre-emptively — a threshold tuned for a denominator that does not exist is the
  decoration D18 refuses.
- ⚠️ **Nothing calls the blocker and the engine in sequence, and the blocker has no production
  caller at all.** `candidates` is reached from its own tests and from `fixtures.rs`'s test module.
  It is the same shape `decide` was in before story 5.5 and `l1` is in today: the seam is
  `score_corpus`'s `answers` map, still fed empty. **Owner: story 5.7.** Recorded because the module
  doc claims the two organs do not consult each other, and the *reason* that claim is currently
  unfalsifiable-in-production is that neither is in production.
  ↺ **ANSWERED, NOT CLOSED, by story 5.7, 2026-08-01 — and the narrow sentence is the answer.**
  `score_corpus`'s `answers` map is no longer fed empty: `l1_runner` fills it with 13 real engine
  answers. **`l1` is therefore in production and the blocker is still not**, and 5.7 declined to
  make it so on purpose: a trap hands the runner a PAIR (`Trap::observations`), so the runner has
  nothing to generate, and a runner that generated its own pairs would ignore the corpus's own
  statement of what is under judgement. `candidates` is still reached only from its own tests and
  from `fixtures.rs`'s test module. **Owner re-stated: the first caller that holds a set of
  observations and no trap — story 5.9 or Epic 6.** Not struck.

## Deferred from: code review of story-5.6-blocker-and-recall-assertion (2026-08-01)

_Two latent defects in `fixtures.rs`'s new corpus assertion. Both are green today for a reason the
corpus supplies rather than the code enforces, so both are recorded rather than fixed: the fix is
cheap, but neither has a failing case that exists._

- ⚠️ **`assert_eq!(checked, 10)` counts required-pair OCCURRENCES, not `must-merge` traps**
  [`crates/opencmdb-bin/src/fixtures.rs:4601-4614`]. The loop filters on
  `corpus.required.contains(pair)` over `corpus.pairs`, which is a `Vec`, while `required` is a
  `BTreeSet`. A second trap — of ANY expectation, in any stream — naming an id pair already in
  `required` would increment `checked` to 11 while `required.len()` stayed 10, and the test would red
  with the message *"every `must-merge` trap must have been checked"*, which is not the cause.
  **Measured not live:** 24 traps, 23 two-observation traps, **23 distinct pairs** — no collision
  today. **Owner: the story that commits a trap re-using an existing id pair**, or 5.7 if it touches
  this walk. Recorded rather than fixed because the correct assertion depends on what a second
  occurrence would MEAN, and no such trap exists to answer that.
  ↺ **Story 5.7 did NOT touch that walk, 2026-08-01.** `l1_runner` discovers trap files through
  `trap_gate::discover_trap_files`, not through `fixtures::walk_trap_files`, and the assertion in
  question is untouched. The `or 5.7` half of the owner clause lapses; the first half stands. Not
  struck.
- ⚠️ **The residue assertion compares an order-dependent `Vec`**
  [`crates/opencmdb-bin/src/fixtures.rs:4653`]. `corpus.without_a_pair` is pushed in
  `walk_trap_files` order and asserted equal to `vec!["example-must-abstain".to_string()]`. Green
  today because there is exactly one element, so no order can be wrong. The day a second
  one-observation trap is committed, the assertion's outcome depends on directory iteration order
  rather than on content — the exact failure mode `candidates` and `join` both avoid by returning a
  `BTreeSet`, and which `corpus.required` and `corpus.universes` already avoid in this same file.
  **Owner: the story that commits a second trap with fewer than two observations.** A `BTreeSet`
  removes the dependency in one line; it is not applied now because doing so would change a passing
  assertion with no failing case behind it, which is the change this project asks to be justified by
  a red.

## Deferred from: story-5.7-trap-runner-stops-scoring-nothing (2026-08-01)

_The story that made the committed corpus a gate that RUNS: `score_corpus` scores 13 of 24 traps
where it scored 0 for nine stories. What follows is what it deliberately did not do, each with the
measurement that says why, and one correction it owes to the story that comes next._

- 🔴 **`epics.md:1545` gives story 5.8 the premise that 8 committed traps are unanswerable at L1.
  Measured: there are ELEVEN, in three distinct classes, and only the first is the one that premise
  names.** The residue is asserted by name — not by count — in
  `l1_runner`'s `the_eleven_unanswered_traps_are_named_one_by_one`.

  | class | n | why L1 cannot answer it | what happens if it is answered anyway |
  |---|---|---|---|
  | expected rule is `l2-*` | 8 | the level is not implemented | 4 `VerdictFail` + 4 `WrongRule` |
  | `must-abstain`, names a pair | 2 | the expectation names **no rule**, so there is no level to route on | 2 `VerdictFail` |
  | `must-abstain`, names ONE observation | 1 | there is no pair at all | it would **PASS**, for the wrong reason |

  The three `must-abstain` traps are `hostname-absence-must-abstain`,
  `shared-hardware-vm-must-abstain` and `example-must-abstain`. They are invisible to an `l2-*`
  selector because `Expectation::MustAbstain` carries a CAUSE and no rule, so `Expectation::rule()`
  returns `None` for all three [`crates/opencmdb-core/src/trap.rs`]. The eleven ids in full:
  `cloned-mac-must-not-merge`, `docker-veth-must-merge`, `example-must-abstain`,
  `hostname-absence-must-abstain`, `multi-nic-must-merge`, `multi-nic-must-not-merge`,
  `shared-hardware-vm-must-abstain`, `shared-hardware-vm-must-merge`,
  `shared-hardware-vm-must-not-merge`, `vrrp-virtual-mac-must-not-merge-bearers`,
  `vrrp-virtual-mac-must-not-merge-master`.
  ⚠️ **`epics.md` is NOT edited by this story** — it was verify-only, and an edit there would have
  been a finding. The correction is registered here instead and flagged FORWARD, the same way story
  5.5 flagged the float forward to 5.6: *so it is a decision and not a surprise*. **Owner: story
  5.8**, whose bucket has to hold eleven, not eight.

  ✅ **CLOSED by story 5.8**, commit `2871ebe` on branch `story-5.8-unanswerable-bucket`.
  `epics.md:1545` now reads eleven in three classes; the 8 / 2 / 1 split is asserted by
  `l1_runner`'s `the_residue_decomposes_into_eight_two_and_one`. _(Appended, not rewritten — the
  bullet above is left as it was written so the register keeps its history.)_

- ⚠️ **The `VerdictVectorEntry` / `RuleVerdict` unification is RE-OWNED, and the obstacle is
  measured rather than a matter of appetite.** Five sites named story 5.7 for it; two stated the
  condition — *"when the trap runner first records **a run** a real engine produced"* — and three
  named the story with no condition at all. That condition is **not met**, and the reason is that a
  "run" in `score.rs`'s vocabulary is a `Vec<ScoredRecord>`, and a `ScoredRecord` carries
  `capability_snapshot: Capabilities` — D36's whole point, *"a verdict without its capability
  snapshot is UNFALSIFIABLE"*. Measured at contexting on `6cc137b`:
  - **11 replay streams are referenced by a trap, and not one carries a `capability` control
    record.** Only `capability-downgrade.jsonl` and `partial-then-failed.jsonl` carry control
    records at all, and no trap names either;
  - `read_jsonl`, the reader the runner uses, **discards control records by construction**;
  - a production `Capabilities` DOES exist (`arp_ping.rs` builds one for its `PollSummary`, and
    `capability-downgrade.jsonl` carries a committed one) — **but none of them is on the trap-run
    path**, which is the narrower claim the conclusion actually needs.

  ⇒ producing a `ScoredRecord` here would mean **inventing a capability snapshot for all 24 traps**,
  which is D36's unfalsifiability in reverse and D45's *"a gate on a false truth"*. **Owner: the
  story that gives a trap run a real capability snapshot** — the `FixtureConnector` read path, which
  replays control records, not `read_jsonl`. All five sites were narrowed to the sentence that is
  true after this story; **none was left saying "story 5.7 owns it"**, which is the promise-re-made
  defect five consecutive code reviews have caught.

- ⚠️ **The blocker STILL has no production caller, and this story declined to be it.** Story 5.6
  registered *"nothing calls the blocker and the engine in sequence… Owner: story 5.7"*. The narrow
  true answer: `l1_runner` calls `identity::l1::decide_pair` and never
  `identity::blocking::candidates`, because a trap NAMES the pair it puts under judgement
  (`Trap::observations`) — the runner has nothing to generate, and a runner that generated its own
  pairs would ignore the corpus's own statement of what is being judged. **Owner: the first caller
  that holds a set of observations and no trap — story 5.9 or Epic 6.** Annotated on story 5.6's own
  entry as well; recorded here because "the trap runner will call it" was the expectation, and it
  turned out to be the wrong shape rather than merely deferred.

- ⚠️ **The `must-abstain` column is now measured by NOTHING, and the gate says so rather than
  hiding it.** `Tally::scored_in(Column::MustAbstain) == 0` on the committed corpus after this
  story: all three `must-abstain` traps are in the residue above. That zero is exactly the vacuity
  `scored_in` was built to make visible — *"the column held"* vs *"the column was empty"* — and it
  is asserted, with the reason in the assertion's own message. **Owner: story 5.14 and Epic 6**,
  which are where an abstention first has a producer the corpus can judge.

- ⚠️ **`answer_trap` resolves a trap's `replay` against the BAKED corpus root, never against the
  root handed to `l1_answers`.** So `l1_answers(scratch)` reads trap FILES from the scratch root and
  STREAMS from `fixtures/`, and a scratch trap corpus may only reference committed replay streams.
  This is the same limit `read_traps` carries — recorded at `score_corpus` since story 4.6b — and it
  is stated on the runner too rather than inherited silently. It is **load-bearing rather than
  incidental**: it is what lets the two scratch tests vary an expectation while judging real
  committed observations. **Owner: the story that needs a scratch corpus with its own streams.** Not
  a defect today.

- ⚠️ **`run_trap` still compares raw `RuleId` strings with no normalization; what closed is the
  claim about the CORPUS, not the comparison.** `the_producers_rule_ids_are_the_corpus_spelling`
  asserts that all seven ids the committed corpus writes equal their own `trim()` and
  `to_lowercase()`, so the unnormalized comparison is trustworthy **on the committed bytes**. A
  future non-canonical id on either side would still be a *"red gate on a correct answer"* — the
  test is what makes that a red rather than a surprise. **Owner: the story that admits a
  non-canonical rule id**, if one ever is.

---

## Deferred from: code review of story-5.7 (2026-08-02)

Three-layer review (Blind Hunter · Edge Case Hunter · Acceptance Auditor) of commit `b555712`.
20 unique findings: 1 decision, 10 patches, 2 deferred (below), 7 dismissed. Both entries here are
**measured**, not suspected, and neither is a defect on the committed corpus today.

- ⚠️ **`l1_answers` has no cross-file `TrapId` uniqueness check, so a duplicate id silently
  overwrites.** `answers.insert(trap.id.clone(), …)` [`crates/opencmdb-bin/src/l1_runner.rs:223`]
  walks every discovered trap file and blindly inserts; `TrapFile::validate` enforces uniqueness
  only WITHIN a file. Composed with the harness nothing ships wrong — `score_corpus` raises
  `FixtureError::DuplicateTrapId` [`trap_gate.rs:259-265`] before scoring anything — so this is not
  reachable through the release gate. But `l1_answers` is `pub` and its doc calls the result
  *"exactly the `answers` map `score_corpus` takes"*; a caller that reads `answers.len()` **alone**
  gets a silently short count with no diagnostic, and the residue arithmetic story 5.8 is about to
  write is precisely that shape of caller. The committed corpus has no duplicate id (measured: 24
  distinct ids across ten files), so nothing reds today. **Owner: story 5.8**, as the first consumer
  of this map that counts rather than scores.

  ✅ **CLOSED by story 5.8**, commit `2871ebe`, and **strengthened by its code review** (commit on
  the same branch): `l1_answers` raises `FixtureError::DuplicateTrapId` on a cross-file duplicate,
  compared **folded** (`trim().to_lowercase()`) as `TrapFile::validate` folds within a file — the
  raw-keyed first version let `"Shared-Id"` and `"shared-id"` through, inflating `discovered`. The
  test calls `l1_answers` DIRECTLY, because `score_corpus` refuses the same corpus for its own
  reasons and a harness-level test stays green with the runner's guard deleted (measured, M7).
  _(Appended, not rewritten.)_

- ⚠️ **`outcome_of`'s abstaining row has no end-to-end path through the runner.** All 13 traps the
  runner answers carry a MAC on both sides, so `verdict_for_pair`'s `Neutral` branch
  [`identity/l1.rs:257-263`] is never taken through `l1_answers` or `answer_trap`, and
  `Conclusion::Abstained -> Outcome::Abstained` is proved only by `score.rs`'s own unit tests and by
  a test that calls `decide(vec![], _)` directly. **Nothing in the runner's tests would notice if
  `answer_pair` mishandled a MAC-less observation** — the mapping's third row is exercised beside
  the runner, never through it. This is the same vacuity `Tally::scored_in(MustAbstain) == 0`
  reports, seen from the mapping's side rather than the tally's, and the two entries close together.
  **Owner: story 5.14 / Epic 6**, when an abstention first has a producer the corpus can judge.

---

## Deferred from: story-5.8-unimplemented-level-counts-as-not-passing (2026-08-02)

_The story that made the residue BLOCK: the committed gate now reports `24 discovered, 13 scored,
11 unanswerable` and **does not pass**, and it will not until Epic 6 implements `l2-*`. What follows
is what this story CLOSED, what it deliberately did not do, and one question it measured rather than
decided._

### Closed by this story

- ✅ **CLOSED — the 8→11 correction** (registered by story 5.7 with story 5.8 as owner,
  `deferred-work.md:1937-1960`), by commit **`2871ebe`**. `epics.md:1545` now reads **eleven, in
  three classes (8 / 2 / 1)**,
  with a dated parenthetical naming what it replaced and why. The split is **asserted** by
  `l1_runner`'s `the_residue_decomposes_into_eight_two_and_one` and by `trap_gate`'s
  `the_committed_corpus_is_red_with_eleven_unanswerable_traps` — the second also pins the COLUMN
  each trap was declined in, which the first does not. `epics.md` was edited by this story and by no
  other: it is the one lifting of the verify-only rule, taken because leaving a false premise in the
  epic file is the defect six consecutive reviews have caught.

- ✅ **CLOSED — `l1_answers` has no cross-file `TrapId` uniqueness check** (registered by 5.7's code
  review, `deferred-work.md:2026-2036`, owner story 5.8 as *"the first consumer of this map that
  counts rather than scores"*), by commit **`2871ebe`** and folded correctly by its code review.
  It now raises `FixtureError::DuplicateTrapId { trap, first, second }`
  — the existing variant, no new one. The guard became load-bearing rather than tidy the moment the
  map went TOTAL: its LENGTH is read by the residue arithmetic, so a duplicate shortens a
  denominator with no diagnostic. ⚠️ **The test calls `l1_answers` DIRECTLY, and that is the
  finding, not a detail**: `score_corpus` refuses the same corpus for its own reasons
  [`trap_gate.rs`], so a test written through the harness stays GREEN with the runner's guard
  deleted — it would be measuring `score_corpus`, which already worked. The test asserts the
  harness's behaviour too, as the stated reason for the direct call.

### New, raised by this story

- ⚠️ **Should a non-empty but PARTIAL answers map block? — measured, NOT decided here.** Story 5.8's
  bucket is filled only by an explicit `Answer::Unanswerable`; a trap simply ABSENT from the map is
  neither scored nor bucketed, and does not block. That is deliberate and it is what keeps **4.6b's
  AC1** literally true — *"it reports truth-table failures per D18 column and is GREEN vacuously —
  it must not require an engine to exist"* [`epics.md:1055`] — pinned by
  `an_absent_answer_is_not_a_decline_and_does_not_block`, which asserts `passed() == true` and
  `unaccounted() == 24` over the committed corpus with an empty map.

  The hole this leaves is real and is stated rather than hidden: **a future producer that answers
  some traps and declares nothing about the rest is green, and silently so.** `l1_answers` cannot be
  that producer — it is TOTAL by construction, asserted at 24 entries — so nothing ships wrong
  today. `Report::unaccounted()` exists to make the state readable and is deliberately **not
  rendered and not blocking**: blocking on it would overturn an epic-level acceptance criterion
  inside a story that was not given that decision. **Owner: the story that adds a SECOND producer**,
  or Epic 6, whichever first holds a map that is not total by construction.

- ⚠️ **`Report::unaccounted()` has no production consumer.** It is reached from one test. That is
  recorded rather than resolved: the accessor is what makes the question above measurable, and
  deleting it would leave the vacuous state named by nothing. Mutation M8 (returning 0
  unconditionally) reds that one test, so it is not untested — it is unconsumed.

### Read, and deliberately NOT closed — they belong to others

- **No `l2-*` rule is designed or implemented.** This story makes the absence *countable*, never
  smaller. **Owner: Epic 6**, which is also what empties the bucket and, by construction, deletes
  the `NFR4 NOT MET` line from the report — that sentence is rendered only while the bucket is
  non-empty, so it cannot go stale.
- **No `ScoredRecord`, no `VerdictVectorEntry`.** Re-owned by story 5.7 to *"the story that gives a
  trap run a real capability snapshot"*, with the obstacle measured there. Unchanged.
- **The blocker still has no production caller.** `identity::blocking::candidates` is reached from
  its own tests and from `fixtures.rs`'s test module only. Unchanged by this story and for the same
  reason story 5.7 recorded: a trap NAMES the pair it puts under judgement, so a runner has nothing
  to generate. **Owner: story 5.9 or Epic 6** — the first caller that holds observations and no trap.
- **The `must-abstain` column is still measured by nothing** (`Tally::scored_in(MustAbstain) == 0`).
  All three of its traps are in the bucket. **Owner: story 5.14 / Epic 6**, unchanged.

---

## Deferred from: code review of story-5.8 (2026-08-03)

Three-layer review (Blind Hunter · Edge Case Hunter · Acceptance Auditor) of `master...0e805e6`.
24 unique findings: 2 decisions, 19 patches, 1 deferred (below), 2 dismissed with the measurement
that killed each. The one entry here is **measured, not suspected**.

- ⚠️ **Totality is relative to a ROOT, and nothing ties `l1_answers`' root to `score_corpus`'s.**
  `score_corpus(root_a, &l1_answers(root_b))` leaves every trap in `root_a \ root_b` **unaccounted**:
  not scored, not bucketed, not blocking — `passed() == true` over a corpus nothing answered, with
  `Report::unaccounted()` naming the state and no production caller reading it. The harness refuses
  EXTRA keys (`AnswerForUnknownTrap`) and has never refused MISSING ones.

  This story's own register entry argues the partial-map hole is theoretical today because
  *"`l1_answers` cannot be that producer — it is TOTAL by construction, asserted at 24 entries"*.
  That totality is **root-relative**: it holds for the root `l1_answers` was handed. What keeps the
  hole closed in practice is that `committed_report()` passes the same root twice — **a convention
  inside a test helper, not a property of either function**. So the hole is reachable with today's
  two functions, not only with a future second producer, and that narrows the register entry above
  rather than contradicting it.

  Deferred rather than patched: closing it means either threading one root through both calls or
  making a partial map block, and the second is exactly the epic-level decision this story declined
  to take alone. **Owner: the same story that answers *"should a non-empty but PARTIAL answers map
  block?"*** — they are one decision, and splitting them would produce two half-answers.

## Deferred from: story-5.9-persist-interface-and-identity-link (2026-08-03)

Eight entries named story 5.9 as owner. **Two are CLOSED, three are ANSWERED-not-closed, three are
RE-OWNED to story 5.9b.** An answered entry says *"the condition was measured and not met"*; a
closed one says *"the thing was done"* — reporting the first as the second is the over-claim this
project's reviews have caught repeatedly, so the distinction is kept explicit here.

- ✅ **CLOSED — `IdentityAbstentionCause` derives no `Serialize`/`Deserialize`** *(owner clause:
  "if it persists a cause at all")*. This story persists a cause, **and closes the entry by
  REFUSING the derive**. `repo::cause_token` is an exhaustive `match` in the ADAPTER with no `_`
  arm. Two reasons: a derived variant name is a wire format nobody chose, and renaming a variant
  would silently rewrite stored bytes — the *"silent data migration, the worst kind"* D14 names
  about `ruleset_version`; and `IdentityAbstentionCause` is deliberately not `#[non_exhaustive]`
  precisely so a new variant produces `error[E0004]` downstream. A `#[derive]` bypasses that
  mechanism; the `match` uses it. Both tokens are pinned by `every_persisted_token_is_pinned`.

- ✅ **CLOSED — none of the five new types derives `Serialize`/`Deserialize`** *(owner clause: "if
  it persists a decision at all")*. Same refusal, same reason. This story persists a decision's
  *components* as COLUMNS (`outcome`, `rule_id`, `abstention_cause`, `ruleset_version`), never a
  `Decision` as a blob. Nothing serialises `Verdict`, `RuleVerdict`, `Conclusion` or `Decision`.

- ↺ **ANSWERED, NOT CLOSED — `RulesetVersion` derives no `PartialOrd`/`Ord`.** The entry predicted
  that *"the first consumer that ORDERS two versions is persistence"*. **Measured here: it does
  not.** `insert_identity_link` binds the version and `load_current_links_for_observation` reads it
  back; nothing compares two. *"The link decided under the current ruleset"* is an EQUALITY, not an
  order. No `Ord` was added. **The owner clause stands, unmet** — the first story that ORDERS two
  versions still owns it.

- ↺ **ANSWERED, NOT CLOSED — an incoherent `Decision` is still buildable by struct literal**
  *(owner clause: "the first story that reconstructs a `Decision` from somewhere other than
  `decide`")*. The condition is **not met**: this story's read side returns `PersistedLink` — rows —
  and the `verdict_vector` is deliberately not stored, so a `Decision` **cannot** be rebuilt from a
  row. No constructor was written. ⚠️ Note the tests DO build `Decision` by struct literal
  (`a_match`, `an_abstention` in `repo.rs`), which is test code and does not meet the clause, but
  it is recorded here rather than left for a reviewer to find.

- ↺ **ANSWERED, NOT CLOSED — nothing enforces that a `RuleVerdict` built by struct literal leaves
  non-empty evidence.** Same reason as above: no `RuleVerdict` is constructed by this story's
  production code.

- ↺ **RE-OWNED to story 5.9b — `L1Key` is a bare tuple alias** *(owner clause: "the first story to
  persist a key")*. This story persists the key's two COMPONENTS as columns (`interface.l2_domain`,
  `interface.mac_canon`) and **never holds an `L1Key` value** — it does not call `join`. 5.9b does.

- ↺ **RE-OWNED to story 5.9b — the blocker STILL has no production caller.** Unchanged in
  substance: `identity::blocking::candidates` has no production caller after this story either, and
  `identity::l1::join` still has no cross-crate caller at all. Neither is called from `repo.rs`.

- ↺ **RE-OWNED to story 5.9b — the universe is quadratic in the slice the CALLER supplies, and
  nothing yet bounds that slice.** Distinct from the entry above: that one is *"has no caller"*,
  this one is *"the caller's slice is unbounded"*. This story hands the blocker nothing, so the
  condition is untouched. 5.9b is the first story that hands it a set of observations and therefore
  the first that can measure `n`. _(This entry was missed by the story's own §5, which counted
  seven; the validation's fact-check found it.)_

### New, raised by this story

- ⚠️ **No `state` column on `interface` and no `entity` supertype / `device` table** — three
  distinct entries, split here because decision 4 names three and an earlier draft merged the
  first two into one bullet (caught by the code review).

- ⚠️ **No `entity` supertype table and no `device` table.** D21's supertype
  [architecture.md:1450-1454] exists to make the interface/device disjunction structural. With
  `device` absent the disjunction has ONE arm, and a supertype over one subtype enforces nothing —
  it is the speculation the *"create tables only when the story needs them"* rule refuses.
  **Owner: Epic 6, with `device`.** Deferred, not dropped.

- ⚠️ **No `state` column on `interface`.** D21's extended `entity.state`
  (`active|dormant|…`, [architecture.md:1477-1479]) and F17's lifecycle are read by nothing before
  the lifecycle epic. **Owner: the lifecycle epic (FR40-42).**

- ⚠️ **The `verdict_vector` is NOT stored, so a persisted link cannot be turned back into a
  `Decision`.** D14's list of what a link carries [architecture.md:1015-1016] does not include it,
  and `epics.md`'s AC2 restates that list without it. D18's *"the harness records the COMPLETE
  VERDICT VECTOR"* is a requirement on the **trap harness** (`ScoredRecord`, `score.rs`), a
  different sentence about a different object. Storing it would mean deriving a wire format for
  four domain types to serve no reader. **Owner: whichever of Epic 6 or story 5.14 first needs the
  vector** — that is a schema addition with a named consumer, not a refactor.

- 🔴 **`architecture.md` D14 and `epics.md` disagree about a `confidence` column, and the
  discrepancy is recorded rather than patched.** D14's sentence says a link carries *"the rule
  applied, the evidence, **the confidence**, when, by whom"* [architecture.md:1015]; `epics.md`'s
  AC2 for story 5.9 omits `confidence` and lists `ruleset_version` in its place. **The omission is
  the later and the correct one**, on D13's authority: *"REFUSED: `rule -> confidence: f64` … if
  the output is a float, B has won in disguise"* [architecture.md:956-958]. Its milli-unit
  corollary binds *"the day a ranking value appears"*, and L1 is a deterministic join with nothing
  to rank — so a `confidence` column here would be a value asserting that a ranking exists. No
  column was created. **Owner: story 5.14**, the first story with a ranking surface, which already
  owns the milli-unit entry. `architecture.md` was NOT edited (issue #54's precedent: a correction
  to a decision body is a milestone act, never a story's).

- 🔴 **MariaDB 10.11 cannot INDEX a generated column whose expression coalesces to a string
  literal, and the story's decision 9 prescribed exactly that.** Measured at implementation:
  `CHAR(36) … AS (COALESCE(interface_id, '000…')) STORED` is refused outright with **error 1901**;
  the same expression as `VIRTUAL` creates fine but **error 1901 returns the moment a `UNIQUE KEY`
  names the column** — the literal's charset is session-dependent, so the expression is not
  indexable. The sentinel therefore ships as a **written** `link_subject` column plus a CHECK
  (`identity_link_subject_matches`) that makes it unable to drift, which is the same guarantee by a
  different mechanism. Recorded because the next reader will meet decision 9's text before the
  DDL. **No owner: this is closed by the implementation**, and it is here as the measurement.

- ⚠️ **`load_link_valid_to` renders a `DATETIME(6)` with `CAST(… AS CHAR)` in SQL**, because
  `sqlx` is built here without its `chrono` feature and has no Rust type to decode one into. That
  is transport, not comparison — D10 forbids SQL to descend into a domain value and an instant's
  wire encoding is not one — but it IS a second rendering site, and the write path renders in Rust.
  **Owner: the first story that needs to read an instant back as a value** (rather than to compare
  it against a sentinel); enabling `sqlx`'s `chrono` feature would collapse the two.

## Deferred from: code review of story-5.9 (2026-08-03)

Three-layer review (Blind Hunter · Edge Case Hunter · Acceptance Auditor) of `master...8659cf4`.
Two layers ran against their own live `mariadb:10.11.11`; the Auditor re-executed the whole
mutation pass. **Thirteen entries deferred, each measured rather than suspected.**

- ⚠️ **No `find_interface_by_key`, no `touch_interface_last_seen`.** `0002`'s header states the
  design — *"the re-run finds an interface by its key"* — and no lookup exists. With the L1 key
  deliberately non-unique, a second scan cannot tell "cloned MAC" from "we forgot to look it up",
  and `last_seen_at` is write-once. **Owner: story 5.9b**, the resolver; a lookup with no caller is
  the speculation decision 4 refuses.
- ⚠️ **`identity_link.observation_id` carries no foreign key** while `interface_id`,
  `link_candidate.link_id` and `link_candidate.interface_id` all do. Measured: a link whose
  observation does not exist inserts `Ok(())`. All nine of this story's tests depend on it — they
  mint an `ObsId` and never insert an observation — so adding the FK today reds 8 tests.
  **Owner: story 5.9b**, which writes links from observations that exist.
- ⚠️ **`sql_mode` is not pinned on connect**, so silent truncation of `rule_id` (VARCHAR(64)),
  `mac_canon` (17) and `abstention_cause` (32) depends on the operator's server configuration.
  Measured under `sql_mode=''`: an 80-char rule id stores as 64 and a long MAC as 17. A truncated
  `rule_id` is exactly the silent corruption story 5.10's bit-for-bit replay would report as a
  mismatch with no cause. Pre-existing (`main.rs` connects bare). **Owner: the first story that
  hardens the connection.**
- ⚠️ **A length violation classifies as `Backend("… 1406 …")`**, not a `Constraint`. Pre-existing
  in `classify`. **Owner: with the entry above.**
- ⚠️ **An `Ambiguous` abstention with ZERO candidates is storable**, so *"the ambiguity is DATA, not
  a hole"* is a convention rather than an invariant — FR16 would render an empty candidate list.
  Nothing writes link+candidates as one unit; this story's tests assemble it by hand inside
  `transact`. **Owner: story 5.9b.**
- ⚠️ **`link_candidate` rows attach happily to a MATCH link**, and a candidate may name the link's
  own interface. Measured `Ok(())`. A renderer that shows a disambiguation UI whenever the list is
  non-empty would show it on a decisively-matched link. **Owner: story 5.14**, the FR16 surface.
- ⚠️ **One corrupt `evidence` blob blinds the whole observation.**
  `load_current_links_for_observation` collects with `?`, so a single undecodable row errors ALL
  current links. `CHECK (json_valid(evidence))` exists in MariaDB 10.11. **Owner: story 5.14.**
- ⚠️ **`datetime_literal` truncates below the microsecond in silence** (`…123_456_789 ns` →
  `.123456`), and story 5.10 compares in-memory values to stored ones. Nothing asserts it.
  **Owner: story 5.10**, where it would first bite.
- ⚠️ **`mac_canon`'s canonical form is asserted by a comment only.** One writer using uppercase
  creates a second `interface` row for the same physical NIC, invisibly, because the index is
  deliberately non-unique. `CHECK (mac_canon = LOWER(mac_canon))` is the candidate fix.
  **Owner: story 5.9b**, the first writer.
- ⚠️ **`rule_id = ''` satisfies the rule-XOR-cause CHECK.** *"A decision names the rule that
  settled it"* is met by an empty name; the CHECK only tests `IS NOT NULL`. **Owner: story 5.9b.**
- ⚠️ **A `Timestamp` past year 9999** renders `+11476-08-15 …` and returns an unclassified
  `Backend("… 1292 …")` for an argument the adapter could reject. **Owner: unassigned, low.**
- ⚠️ **`PersistedLink.id`/`.interface_id` and `load_link_candidates`' first element are bare
  `String`** while the write side is fully typed (`LinkId`, `InterfaceId`, `ObsId`). The first
  reader that needs the typed form should drive it rather than this story guessing.
  **Owner: story 5.14.**
- ⚠️ **`count_identity_links` is `pub` with no caller and no test.** Consistent with the module's
  existing `#![allow(dead_code)]` skeleton, recorded so it is not mistaken for coverage.
  **Owner: story 5.9b.**

## Deferred from: story-5.9b-engine-resolves-and-writes-links (2026-08-04)

**Eleven entries named this story as owner — nine by NAME and two by CONDITION**, the second pair
found by the validation's fact-check rather than by a grep. Their disposition below: **six CLOSED,
four ANSWERED-not-closed, one measured and left OPEN by decision.** An answered entry says *"the
condition was measured and not met"* — or, twice here, *"the condition was met and the story refused
it"*; a closed one says *"the thing was done"*. Reporting the first as the second is the over-claim
seven consecutive code reviews have caught.

- ✅ **CLOSED — the blocker has a production caller.** `crates/opencmdb-bin/src/resolver.rs` calls
  `identity::blocking::candidates` once over the whole slice, before any verdict is asked for, and
  `identity::l1::join` for the grouping. Both had waited since story 5.6; `join` had **no
  cross-crate caller at all**.

- ✅ **CLOSED — `find_interface_by_l1_key` and `widen_interface_seen_window` exist.** `0002`'s header
  had stated the design — *"the re-run finds an interface by its key"* — with no lookup behind it.
  The lookup is what makes an interface id stable across runs, which story 5.10's bit-for-bit replay
  depends on, and what makes read-your-own-writes real rather than a convention:
  `the_second_observation_sees_the_first_ones_interface_in_one_transaction` measures it.

- ✅ **CLOSED — `identity_link.observation_id` carries its foreign key** (`0003_resolver_guards.sql`).
  🔑 **The register said it reds 8 tests; measured, it reds TWELVE** in `repo.rs`, all
  `Constraint("foreign_key")` panic-carried. **And two more then appeared** —
  `repo::tests::ingest_observation_round_trip` and `main::tests::index_renders_the_real_gap` — which
  `DELETE FROM observation_record` without deleting links first and fail **ERROR 1451**.
  🔑 **Those two were INVISIBLE until the twelve were fixed**, because a failing test rolls back and
  leaves no link behind for the cleanup to trip over. Fourteen tests across two files, in two waves;
  the second wave is order-dependent and a filtered run hides it.

- ✅ **CLOSED — an `Ambiguous` abstention with no candidates is refused**, by `resolver::guard_decision`,
  returning `Constraint("ambiguity_without_candidates")`. ⚠️ **Unreachable through the resolver**: L1
  emits no `Supports` and no `Opposes`, so `decide` cannot conclude `Ambiguous` at all. The guard is
  therefore tested by calling it DIRECTLY — a test written through the pass would stay green with the
  guard deleted, which is story 5.8's measured lesson.

- ✅ **CLOSED — `mac_canon` must equal its own `LOWER()`** (`0003`). `MacAddr`'s `Display` cannot emit
  anything else, so this is a second line of defence; it is measured by a RAW insert going around the
  adapter, in the idiom story 5.9's M3 forced.

- ✅ **CLOSED — `rule_id = ''` is refused twice**: by `resolver::guard_decision`
  (`Constraint("rule_id_empty")`) and by `identity_link_rule_id_not_empty` in `0003`. Both are
  measured, and separately: dropping either leaves the other carrying the test, which is exactly how
  story 5.9's M3 first came back green.

- ↺ **ANSWERED, NOT CLOSED — `L1Key` is still a bare tuple alias, and the newtype is REFUSED.**
  *(Owner clause: "the first story to persist a key".)* The condition is now MET — this story holds
  `L1Key` values and persists both components. The newtype was still not introduced, and the reason
  is a measurement rather than a preference: the key is destructured at its ONE use site
  (`for ((l2_domain, mac_canon), group) in &groups`) and never travels as a value —
  `find_interface_by_l1_key` takes the two components as separate parameters, and the schema stores
  them as two columns. A newtype would wrap something that is never passed around. **The owner clause
  is spent; the residue is now a refusal with its reason.**

- ↺ **ANSWERED, NOT CLOSED — an incoherent `Decision` is still buildable by struct literal, and this
  story is the one that ALMOST met the clause.** *(Owner clause: "the first story that reconstructs a
  `Decision` from somewhere other than `decide`".)* Persisting a placement needs a `Decision`;
  an observation alone on its L1 key has no pair to produce one; so a struct literal with an empty
  `verdict_vector` was one line away — the *"merged, with no explanation"* shape `Decision`'s own doc
  warns about and D13's *"the list IS the explanation"* forbids. **`identity::l1::decide_singleton`
  exists instead**: it builds the one-element `Decisive` verdict and returns `decide`'s value, so
  nothing bypasses the algebra. **The clause therefore stands, unmet.** The same disposition applies
  to its twin (nothing enforces non-empty evidence on a `RuleVerdict` built by struct literal).
  🔑 The residue is stated in THREE places — here, its twin above, and the doc comment on
  `cascade.rs`'s `Decision`, which named story 5.9 as owner. All three were updated together; two of
  three is the doc-twin defect four of story 5.9's review patches were.

- ↺ **ANSWERED, NOT CLOSED — `count_identity_links` still has no PRODUCTION caller.** 🔑 **The entry's
  premise was half stale when it was written**: it says *"no caller and no test"*, and it has had a
  test since story 5.9's own code review (`repo.rs`, the `ON DELETE CASCADE` test). This story gives
  it two more test callers and still no production one, so it is neither closed nor deleted — a
  counting query with no reader is exactly what the module's `#![allow(dead_code)]` skeleton is for.
  **Owner: story 5.14**, the first story with a surface that counts anything for a human.

- ⚠️ **OPEN, with its number — the universe is quadratic and nothing bounds the caller's slice.**
  This is the first story that could measure `n`, and it does:
  `the_universe_is_quadratic_and_its_size_is_asserted` asserts `candidates(&obs).len() == n(n-1)/2`
  over distinct ids at several sizes and **44 850 at the reference scale of 300 hosts**.
  🔑 **D13's prose says "90k pairs" for the same scale — the figure counts pairs the other way, and
  the measured number is half of it.** No refusal threshold was installed: a bound with no measured
  need is the speculation the *"create only what the story needs"* rule refuses. **Owner: the first
  story that hands the resolver a slice it did not choose** (a real scan, i.e. the wiring decision 3
  defers).

### New, raised by this story

- ⚠️ **The resolver is NOT wired into `main.rs`'s startup scan.** By decision: the named consumers are
  stories 5.10 and 5.11, and wiring it would make every deployment write links with no page to
  display them and no purge to remove them. **Owner: story 5.14**, the FR16 surface.

- ⚠️ **The pass is NOT idempotent over the same observations.** Measured: running `resolve` twice over
  one slice inside one `transact` is `Err(Constraint("unique"))` and a **full rollback** — 0
  interfaces, 0 links. `insert_identity_link` appends and `identity_link_one_current` refuses the
  second current row. Superseding an unchanged decision instead of appending is *"no new version for
  an unchanged decision"*, which `0002`'s own header already names. **Owner: story 5.11.**

- ⚠️ **A link's evidence names the PAIR that justified it, not the whole group.** `decide_pair`'s
  evidence is the sorted pair (D19), so a link on a five-observation interface names two ids. That is
  the engine's own evidence rather than the resolver's construction, which is why it was chosen;
  a display may want more. **Owner: story 5.14.**

- ⚠️ **`absence_of_proof` is a cause of CONVENIENCE for an observation whose every pair the blocker
  excluded.** The engine has two causes and neither means *"the blocker declined to propose"*.
  Nothing can reach that branch today — `candidates` is TOTAL — so choosing a semantics now would be
  inventing one no caller can produce. **Owner: the first story that NARROWS the blocker**; F17's
  `dormant` exclusion [architecture.md:1205] is the named candidate.

- ⚠️ **The seen-window is widened with `LEAST`/`GREATEST` in SQL.** Not the comparison D10 forbids —
  no domain value is under judgement, and MariaDB is the only engine (D64) — but it IS arithmetic in
  SQL, chosen because `sqlx` is built here without its `chrono` feature and a `DATETIME(6)` has no
  Rust type to decode into. Enabling that feature would let the widening be computed in Rust and
  would collapse `load_link_valid_to`'s `CAST(… AS CHAR)` with it. **Owner: the first story that needs
  to read an instant back as a VALUE** — the entry story 5.9 opened, whose condition this story does
  **not** meet: it compares rendered strings against `datetime_literal`, which is transport.

- 🔴 **`epics.md`'s AC1 for story 5.9b is FALSIFIED by the code it describes, and `epics.md` was not
  edited.** It says *"each observation carrying a MAC lands on exactly ONE `interface`"*; `join` loops
  `for key in keys_of(observation)` [`l1.rs:174-178`] and `keys_of` returns one entry per `Fact::Mac`,
  so a two-MAC observation lands on TWO. Guy widened the criterion at contexting — *one interface per
  L1 key* — and `one_observation_with_two_macs_lands_on_two_interfaces` asserts it. Story 5.8's
  precedent for editing `epics.md` does not apply: 5.7 had handed 5.8 that correction with a named
  owner, and here the contexting found its own. **Owner: Epic 5's retrospective.**

- 🔴 **`epics.md:1616` is departed from too**: it requires an abstention link to carry *"its
  `link_candidate` rows"*, and this pass writes **zero** for an `absence_of_proof` — correctly, since
  nothing was a candidate. The story's guard refuses the incoherent case (`Ambiguous` with none) and
  permits this one. A second unregistered divergence from the epic, recorded beside the first.
  **Owner: Epic 5's retrospective.**

- ⚠️ **`every_ddl_guard_refuses_what_it_names` could have stopped measuring its own guard, silently.**
  After `0003`, MariaDB reports `identity_link_observation_fk` (ERROR 1452) before
  `identity_link_interface_fk`, and both classify as `Constraint("foreign_key")` — so a case built on
  an observation that does not exist would be satisfied by the WRONG constraint while staying green.
  It is fixed here (every link test now names a real observation), and it is recorded because
  **nothing would have routed a reader to it**: the test never reds. **No owner — closed by the
  implementation**, and here as the measurement.

## Deferred from: code review of story-5.9b (2026-08-04)

Three-layer review (Blind Hunter · Edge Case Hunter · Acceptance Auditor) of `master...18844ea`.
Two layers ran against their own live `mariadb:10.11.11`; the Auditor re-executed all fourteen
mutations independently. **Five entries deferred, each measured rather than suspected.**

- ⚠️ **Two concurrent passes can mint two interfaces for one L1 key.** `find_interface_by_l1_key` is
  a plain `SELECT` under InnoDB's default REPEATABLE READ, and `interface_l1_key` is deliberately
  NOT unique (D21), so two transactions resolving the same MAC both read "not found" and both
  insert. Afterwards the `ORDER BY id LIMIT 1` silently picks the lower UUID. The resolver's doc
  says a second row on one key is *"unreachable through the resolver"* — true per pass, false
  across passes. Unreachable in practice because D21 puts identity resolution inside a **single**
  writer actor, but that precondition is stated nowhere in `resolve`'s signature or doc.
  **Owner: the first story that gives the resolver more than one writer** — the wiring decision 3
  defers.
- ⚠️ **`widen_interface_seen_window` ignores `rows_affected()`**, so widening a non-existent
  interface returns `Ok(())`. This is the silent-success shape story 5.9's code review closed in
  `close_identity_link`, reappearing in the neighbouring function. Only ever called with an id the
  same transaction just found or minted. **Owner: the first story that widens a window it did not
  itself look up.**
- ⚠️ **An `observed_at` at or past `OPEN_END` fails the whole batch** with
  `Constraint("check")` from `identity_link_interval`, naming no column. One garbage instant from a
  connector takes down every link in the pass. `close_identity_link` already guards the sentinel
  explicitly; the write path does not. **Owner: the first story that ingests instants it did not
  synthesise.**
- ⚠️ **Sub-microsecond `observed_at` is truncated** by `datetime_literal`'s `%.6f`, so two distinct
  instants store as one — measured, `…20.000000001` and `…20.000000999` both become
  `…20.000000`. `the_stored_instants_are_the_derived_ones` therefore asserts a property that holds
  only at microsecond granularity. **Already registered by story 5.9 with story 5.10 as owner**;
  recorded here because this story is the first whose test depends on it.
- ⚠️ **The eleven register entries this story disposed of still stand unmarked at their original
  lines** (`:2199`–`:2318`), with their dispositions 120 lines below. This is the house pattern —
  story 5.9's entries are still greppable too — but §5 of this story counted its own scope BY that
  grep, so the next story inherits a count that includes eleven closed items. **Owner: whichever
  story next counts its scope by grepping this file** — the fix is either an annotation in place or
  a different way of counting.

### New, raised by the code review and carried forward

- ⚠️ **`write_link` keeps the FIRST verdict's evidence only.** Every `Decision` L1 produces carries
  exactly one verdict, so nothing is lost today — but that is an L1 accident, not a property, and
  `cascade.rs` says Epic 6's cascade ends it. On that day the evidence of verdicts 2..n vanishes with
  nothing red. Not pre-solved: unioning evidence across a vector is a decision about what a link
  MEANS, and no producer exists to decide it against. **Owner: Epic 6**, with the first multi-verdict
  `Decision`.
- ⚠️ **Nothing fills `guard_decision`'s `candidates_for_link`.** The only call site passes `&[]`, and
  the pass writes no `link_candidate` row, because L1 has no ambiguity to hold candidates for. So the
  day a producer of `Ambiguous` arrives, the guard would refuse a LEGITIMATE ambiguity rather than
  let it be written with its candidates — the inverse of FR16. The signature already takes the slice
  so it need not change under whoever fills it. **Owner: Epic 6**, the first producer of `Ambiguous`.
- ⚠️ **`resolve` takes a bare `&mut MySqlConnection`, so D21's "never split across two transactions"
  is a PRECONDITION, not a structure.** Measured: called on a pooled connection under autocommit, a
  pass that then failed left **2 interfaces and 2 links committed**. Taking a unit-of-work type
  instead would make it structural, and would move every call site story 5.9 wrote. **Owner: the
  first story that gives the resolver a second caller** — the wiring decision 3 defers.
- 🔴 **`uuid::Uuid::now_v7()` reads the clock, and its output is a PRIMARY KEY.** A v7 UUID embeds a
  48-bit wall-clock millisecond, and this pass mints one per interface and one per link — decoded
  from two runs over identical input, 57 ms apart. This is the house idiom (`ObsId`, `ConnectorId`
  are v7 too), so the code is not the problem. **The consequence is story 5.10's**: its *"reproduced
  identically, bit for bit"* can only ever mean *modulo the ids*, and that has to be written into
  5.10 before it is written against. **Owner: story 5.10.**
- ⚠️ **A group whose every member abstains still mints an interface nothing points at**, and
  interfaces are never purged, so the orphan is permanent. Consistent with decision 2 (`join` NAMES
  the interface, and it names it whether or not a verdict follows), but unreachable through `resolve`
  today and unasserted. **Owner: the first story that narrows the blocker** — the same one that owns
  decision 11's cause-of-convenience.

## Deferred from: story-5-10-purge-test (2026-08-04)

Three entries named this story as owner — **two by NAME** (the `uuid` v7 consequence, and the
`datetime_literal` truncation, registered TWICE) and the `sqlx chrono` one **by a CONDITION this
story does not meet**. One CLOSED, two ANSWERED-AND-RE-OWNED, one left untouched.

- ✅ **CLOSED — `uuid::Uuid::now_v7()` stamps a wall-clock millisecond into every primary key, so
  *"bit for bit"* can only mean MODULO THE IDS.** It is no longer a note to be rediscovered: it is
  Guy's arbitration in §1, it is the shape of `LinkSnapshot` (**no `id` field at all**, so the
  exclusion is structural), it is the name of the test
  (`every_decision_bearing_column_survives_a_purge_and_replay`), and **mutation M6 proves it load-bearing**
  — put the id back and the comparison reds at once.

- ⚗️ **SPLIT — `datetime_literal` truncates below the microsecond.** Guy's arbitration at the code
  review: the entry contains **two debts**, and assigning it whole is what has made it circle for
  three stories.
  - ✅ **CLOSED here — the half that is a property of a pure function.** The entry's own reproach is
    *"Nothing asserts it"*, and nothing did: changing the format string reddened no test that named
    the truncation. `repo::tests::datetime_literal_truncates_below_the_microsecond` now pins it —
    two instants **788 ns apart**, distinct in Rust, rendering identically. **Mutations M10
    (`%.9f`) and M11 (`%.3f`) both red it, assertion-carried.** No future story was needed for this
    half, and it cost two lines.
  - ↺ **RE-OWNED — the half that is a risk.** A caller comparing an instant it HOLDS against one it
    STORED can be wrong, and nothing does that yet. **Owner: story 5.11**, and the reason is
    substantive rather than convenient: 5.11 supersedes, so it is the first story that holds **two
    instants for one placement** and must decide whether they denote the same thing. Story 5.14 only
    renders them, and a human reading a microsecond-truncated timestamp loses nothing.
  🔴 **And the second bullet's own evidence is refuted**: it cites
  `the_stored_instants_are_the_derived_ones` as *"asserting a property that holds only at microsecond
  granularity"* — but that test **cannot catch the truncation at all**, because it builds its
  expected value by passing the instant through the truncating function itself. Both sides truncate
  identically. **It is the same bilateral-oracle shape this story's code review found in
  `snapshot_links`**, and it is written into the new test's doc so 5.11 does not inherit the belief
  that the point is covered.
  ⚠️ **An earlier disposition re-owned this to *"the first story that compares an instant it holds in
  memory against a stored one"* — a CONDITION, which is precisely the form AC7 rejects as *"a debt
  nobody holds"*.** Caught by the Acceptance Auditor, in the story's own criterion.

- ⚪ **UNTOUCHED — the `sqlx` `chrono` feature.** Its owner is *"the first story that needs to read an
  instant back as a VALUE (rather than to compare it against a sentinel)"* — a CONDITION, and this
  story is never named. Measured: `snapshot_links` compares two renderings and **never produces a
  `Timestamp`**, so the condition is not met. Not this story's to close.

### New, raised by this story

- 🔴 **The two natures are MUTUALLY EXCLUSIVE on one placement.** `identity_link_one_current` is
  `(observation_id, current_subject)` and the purge removes only `decided_by='ENGINE'`, so an
  operator can **never** confirm or correct a placement the engine already holds — the write is
  refused `Err(Constraint("unique"))` — and an operator row occupying a slot the replay needs makes
  the **entire replay roll back**. D14's *"two natures in one table"* is true of the TABLE and false
  of one `(observation, subject)`. This story measures what it can — an operator row on a subject the
  engine does not place survives the purge with its own `id` — and registers the model question:
  **can an operator ever confirm, correct or override an engine placement?** **Owner: story 5.14**,
  the FR16 surface, which is where a human first touches a link.

- 🔴 **`epics.md:1634` asks for something a purge-and-replay can never measure**, and this is now
  measured rather than argued: mutations M1 and M1-noop are the same mutation up to what they key on,
  and only the one keyed on state the purge does NOT restore can red. *"The test reds if any engine
  decision is made to depend on state the purge removes"* points the wrong way — the purge restores
  the run-1 start state, so such a dependency is invisible by construction. `epics.md` is NOT edited.
  **Owner: Epic 5's retrospective**, beside §2's `TRUNCATE ... WHERE` correction and story 5.9b's two.

- ⚠️ **`epics.md:1627` and D14 both write the purge as `TRUNCATE ... WHERE decided_by='ENGINE'`, and
  MariaDB's `TRUNCATE` takes no `WHERE`** — measured at the parser, `ERROR 1064`. The shipped
  statement is a `DELETE`, and `purge_engine_links`' doc says so. **Owner: Epic 5's retrospective**
  for `epics.md`; `architecture.md` is a milestone act (issue #54).

- ⚠️ **A query body generic over `sqlx::Executor` cannot issue two statements.** The executor is
  consumed by value, so `purge_engine_links` would need `&mut MySqlConnection` — measured while
  writing mutation M2, which could not be applied without changing the signature AND the one call
  site that passes `&pool`. The shipped one-statement form hides the constraint. **Owner: the first
  story that needs a multi-statement query body.**

- ⚠️ **A summary assertion placed before a specific one pre-empts the red an AC names.** In this
  story the interface-id-set assertion sits BEFORE the replay deliberately, and M2's red lands on it
  as AC6 requires. The validation measured the opposite ordering doing the opposite. Not a defect
  here; recorded as a shape to watch, because two mutations of story 5.9b landed on a summary line
  rather than on the assertion their AC named.

## Deferred from: code review of story-5-10 (2026-08-05)

Three-layer review of `master...a82fa3a`; **all three layers ran their own live
`mariadb:10.11.11`** and the Auditor re-executed the mutation pass. Three entries deferred.

- ⚠️ **The purge deletes superseded rows the snapshot never compared.** `purge_engine_links` has no
  `current_subject` filter; `snapshot_links` restricts to current rows. So "bit for bit" covers
  what is current, while the purge erases history as well — measured: a pass, then
  `close_identity_link`, gives `rows_in_table = 1, snapshot_len = 0, purge_count = 1`. Inert today
  because nothing supersedes, and `a_superseded_engine_link_is_not_restored_by_the_replay` now names
  the asymmetry rather than leaving it silent. **Owner: story 5.11**, the first story that
  supersedes — it must decide whether the replay owes history anything.
- ⚠️ **The comparison is blind to `link_candidate` and to `interface`'s own columns.**
  `snapshot_links` covers `identity_link` only; `interface_ids` compares the `id` column and not
  `l2_domain`, `mac_canon` or the seen-window. More materially, the purge CASCADES candidates away
  and nothing asserts the replay puts them back — latent, because L1 has no `Ambiguous` producer and
  the resolver writes no candidate. **Owner: Epic 6**, the first producer of `Ambiguous`; a
  `snapshot_candidates` compared alongside would turn the blindness into a red the day it stops
  being empty.
- ⚠️ **The purge and the replay run in TWO transactions, and the composed shape runs nowhere.**
  `purge_engine_links` commits, and only then does the replay open its own transaction. If the
  replay fails — a unique collision with an operator row is exactly that case, and
  `an_operator_cannot_take_a_slot_the_engine_holds` measures it — the purge is already committed and
  the engine's links are gone with nothing to restore them. Both in one unit of work is what a
  production caller needs. **Owner: the first story that gives the purge a production caller.**

## Deferred from: story-5-11-idempotent-supersede (2026-08-05)

The first story that SUPERSEDES. Three entries were registered at its name and all three are
disposed of here; four are opened.

### Disposed of

- ✅ **CLOSED — the pass is not idempotent.** Registered by story 5.9b with 5.11 as owner:
  *"running `resolve` twice over one slice is `Err(Constraint("unique"))` and a full rollback"*.
  `write_link` now reads the current ENGINE version of the slot before writing and takes one of
  three branches. `a_second_identical_pass_writes_nothing_at_all` measures it on the `id`s — the
  same rows, not re-minted ones — and at the reference scale a rerun reports
  `written 0, superseded 0, unchanged 300`. Mutations M3a and M3b red it.

- ✅ **DISCHARGED — the `datetime_literal` open half. A production caller now exists, and it is this
  story's own.**
  ⚠️ **This entry was first written as CLOSED-because-unreachable, and the story's own code review
  falsified that within the hour.** The disposal said *"no PRODUCTION caller compares an instant it
  HOLDS against one it STORED, story 5.11 included"*, which was true of the code as it then stood.
  Then the review's arbitration added the instant-regression guard, and
  **`resolver.rs`'s `write_link` now does exactly that**:
  `if datetime_literal(observation.observed_at) < current.valid_from`. The register was right to
  keep the entry alive for three stories and the closure was premature; recorded that way rather
  than quietly rewritten.
  🔑 **The residue is real, named, and pinned by a test that already exists.** The comparison is on
  RENDERINGS — `sqlx` is built without its `chrono` feature, so a `DATETIME(6)` has no Rust type to
  decode into — and `datetime_literal`'s fixed-width `%Y-%m-%d %H:%M:%S%.6f` makes lexicographic
  order agree with chronological order. But the rendering **truncates below the microsecond**, so
  two instants less than 1 µs apart compare EQUAL and the guard does not fire.
  `repo::tests::datetime_literal_truncates_below_the_microsecond` (story 5.10's half of the split)
  is what pins the truncation, and it is now load-bearing for a production guard rather than for
  nothing.
  ⚠️ The test sites that compare a read-back `CAST(… AS CHAR)` against `datetime_literal(held)` are
  deliberately NOT counted here: three successive documents have carried a wrong figure for them.
  The reproducible form is what a reader needs —
  `rg -n 'datetime_literal\(' crates/opencmdb-bin/src/` and read the assertions — and
  `deferred-work.md:2551-2556` already records why those sites cannot catch a truncation change:
  both sides pass through the truncating function.

- ✅ **CLOSED — the purge deletes superseded rows the snapshot never compared.** Guy's arbitration
  at contexting: **the replay owes history nothing.** A link is *"a cache of attention, not of
  truth"*, so the purge is an assumed reset of the engine's beliefs AND of their history, and
  `architecture.md:1016-1017`'s *"a bad link is UNLINKED, never erased"* governs an operator's
  correction of a live belief rather than the engine's scratch history. `purge_engine_links`' doc
  now says so, and `a_purge_after_a_supersede_loses_history_and_still_replays` measures BOTH
  numbers — 3 rows before, 2 after, snapshots equal — because equal snapshots alone hide the loss.
  M7 reds it, and reds story 5.10's `a_superseded_engine_link_is_not_restored_by_the_replay` too.

### New, raised by this story

- ⚗️ **A version could be dated by the instant that CAUSED it rather than by the observation's own.**
  Today every version of one placement opens at the same `valid_from`, so a superseded engine
  version is zero-length and an engine link's history is ordered by INSERTION rather than by time.
  Dating a version by the maximum `observed_at` over its new evidence would give real intervals in
  the ordinary case and degenerate only when the newcomer is older. Not taken here: it changes
  `valid_from` for every paired link, reds `the_stored_instants_are_the_derived_ones`, and changes
  the content of story 5.10's snapshots. **It is also what would make the `datetime_literal` risk
  above real.** **Owner: story 5.14**, the first story that renders a link's history to a human and
  therefore the first that needs it to read as a chronology.

- 🔴 **The stability of an observation's `observed_at` across passes is a CALLER'S DISCIPLINE and
  nothing enforces it.** `valid_from` comes from the IN-MEMORY `Observation`; `0003`'s foreign key
  checks only that `observation_id` EXISTS; and **nothing in the workspace reads
  `observation_record.observed_at` back as a value** — the only `SELECT` naming that column uses it
  as an `ORDER BY` (`repo.rs:205`). Hand the pass one `obs_id` with a later instant and a
  non-zero-length supersede is produced; hand it an earlier one and it is now refused by name
  (`RepositoryError::InstantRegressed`, installed at this story's code review).
  🔴 **A second consequence, measured while writing that guard's test:** an unchanged slot keeps the
  `valid_from` it was FIRST written with, while a purge-and-replay from an empty store writes the
  instant it is handed now — so re-supplying one `obs_id` with a later instant makes story 5.10's
  replay comparison red on `valid_from` alone. `the_replay_invariant_survives_an_observation_that_lost_a_key`
  holds both slices at one instant deliberately, and its doc says why.
  **Owner: story 5.14** — the wiring story this file names as 5.14 three times elsewhere, so it is
  named here too rather than described as *"the first story that ingests observations it did not
  construct"*, which is the condition form this register refuses.

- ✅ **CLOSED by story 5.11b, 2026-08-06 — both halves, by different means.** The
  last-duplicate-wins half is closed by a REFUSAL rather than by a rule for picking a winner:
  `resolve_within` returns `RepositoryError::ContradictoryObservation` when one `obs_id` arrives
  twice carrying different decision-bearing content. `raw` is excluded from that comparison (D19 —
  no decision reads it) and so is the ORDER of `facts`; both exclusions are measured against the
  bare `a != b` they replace, so the explicit comparison is load-bearing rather than decorative.
  Mutation M4 (drop the refusal) reds exactly one test. The tail-abstention half is closed as
  BENIGN, and now by measurement rather than by argument: 720 permutations of a six-observation
  slice through `join`/`candidates`, twelve through purge-and-replay, twelve more into a populated
  store, and eight seeded orders at reference scale — every one a no-op.
  ⚠️ **One consequence is registered rather than closed**: the pure tests of `contradicts` do NOT
  protect its WIRING. M4 removes the call and leaves them green; only the database test sees it.
  _(The count below read "425 tests" when this bullet was written; `master` carried **429** at story
  5.11's merge and 446 at 5.11b's. The claim it makes is unaffected — last-wins was invisible to all
  of them.)_
  ~~⚠️ **`resolve_within` reads the slice's ARRIVAL ORDER twice, and the four order-independent
  mechanisms do not cover it.**~~ `by_id` is a `.collect()` into a `BTreeMap`, so it is
  **last-duplicate-wins**: a slice carrying one `obs_id` twice with DIFFERENT content resolves to
  whichever copy arrives last, and that copy's `observed_at` becomes a STORED column. The tail
  abstention loop iterates the raw slice, so abstention rows are inserted in arrival order (the row
  values are invariant; only the mint order moves). `a_repeated_obs_id_abstains_once_and_the_pass_says_so`
  passes the same clone three times, so last-wins stays invisible to all 425 tests.
  **Owner: story 5.11b**, whose whole subject this is.

- ⚠️ **Two versions of one placement may OVERLAP, and the schema does not care.** Measured at
  contexting: closing the old row at `t2 > t1` and opening the new one at `t1` is accepted, so the
  half-open chain is the WRITER's property and never the DDL's. `identity_link_interval` constrains
  each row's own interval only. This story chains exactly (`old.valid_to == new.valid_from`), and
  nothing would red if a later story stopped. **Owner: Epic 6**, which brings the cascade and with
  it the second producer of decisions — named rather than left as *"the first story that writes a
  second supersede path"*, which is the condition form this register refuses.

- 🔴 **`0002_interface_and_identity_link.sql:83` now carries a comment that is FALSE of the live
  schema, and it cannot be edited.** It reads *"A version covers a half-open interval, so it can
  never be zero-length or inverted"*; `0004` makes the first half false for CLOSED rows. `0002:54`
  likewise still names *"story 5.11's 'no new version for an unchanged decision'"* in the future
  tense. **sqlx checksums the migration FILE, comments included**, so correcting it in place would
  make every existing database refuse to migrate with `VersionMismatch(2)` — measured during this
  story when exactly that edit was attempted and reverted. `0004`'s own comment points back at
  `0002`; nothing points forward. Recorded here because the story's own §3 rule is that a false
  comment in DDL is a defect, and leaving one in an unmodifiable file in SILENCE is worse than
  leaving it with a note. **Owner: the milestone that consolidates migrations** — the same act that
  regenerates `architecture-views.md` (issue #50), which is the only point at which rewriting an
  applied migration is legitimate.

## Deferred from: story-5-11b-order-independence (2026-08-06)

_Story 5.11b measured a property that is already true by construction, so its residue is almost
entirely about the MEASUREMENT rather than about the engine. Two entries below were found by the
story's own mutation pass and would not have been visible to reading._

- 🔴 **The golden-value test does NOT guard the seed sweep's provenance, and the story said it did.**
  Measured: replacing `SEED_SWEEP` with `now()..=now()+7` left the **entire suite green** — the
  golden test pins `shuffled` at a hardcoded seed and never reads the constant, while every other
  consumer stays green because eight clock-derived seeds still shuffle, still reproduce within one
  process (`shuffled(x, s) == shuffled(x, s)` for every `s`), and still number eight.
  ✅ **CLOSED in the same story** by `the_seed_sweep_is_the_fixed_range_it_claims_to_be`, which
  reads the constant's VALUES; M5 then reds exactly one test. Recorded rather than silently fixed
  because the refuted prediction is the finding: *reproducible within one process* and *reproducible
  across runs* are different properties, and only the second needs a fixed seed.
  **No owner — closed.**

- ⚠️ **The pure tests of `contradicts` do not protect its wiring into `resolve_within`.** Mutation
  M4 deletes the CALL, not the function, and `the_contradiction_test_catches_every_field_a_decision_reads`
  and `the_contradiction_test_excludes_what_no_decision_reads` both stay green; only the database
  test `a_repeated_obs_id_with_differing_content_is_refused` reds. That is the correct division of
  labour and it is not a defect — but it means the refusal's REACHABILITY rests on one test, and a
  future story that moves the guard would need to move that test with it.
  **Owner: story 5.14**, the wiring story, which is the first caller to hand the pass observations
  it did not construct.

- ⚠️ **`contradicts` is blind to fact MULTIPLICITY.** `facts` is compared as a set (a containment
  test in both directions) rather than as a sequence, deliberately: nothing reads the ORDER of
  `facts`, since `keys_of` collects them into a `BTreeSet`, so refusing a reordered serialisation
  would be the over-broad refusal AC5 warns against. The consequence is that `[x, x, y]` and
  `[x, y, y]` compare equal. A repeated fact inside one observation is pathological and reaches no
  decision, and the failure direction is the safe one — the guard declines to refuse and leaves the
  pre-5.11b behaviour rather than inventing a new one. **Owner: whoever gives `Fact` an `Ord`
  derive**, at which point a `BTreeSet` comparison closes it for free. Not done here: widening a
  core type for a comparison convenience is outside a story whose one production change is the
  refusal itself.

- ⚠️ **`permutations` materialises all `n!` vectors before yielding any.** `index_permutations`
  builds a `Vec<Vec<usize>>` upfront. At the sizes this story uses (`n ≤ 6`, so 720) that is
  measured at ~20 ms and irrelevant. A caller at `n = 8` would allocate 40 320 vectors before the
  first iteration, and at `n = 10` it would not finish. The signature is already
  `impl Iterator`, so making it lazy is a body change with no call-site cost.
  **Owner: the first story that permutes a slice longer than six** — which is a real condition and
  not a disguised *"someday"*: the reference-scale path deliberately uses the seeded shuffle
  instead, precisely because enumeration does not reach it.

- ⚠️ **`shuffled` and `SEED_SWEEP` have exactly ONE consumer.** If
  `the_reference_scale_pass_is_order_independent_across_the_seed_sweep` were ever deleted, the
  generator and its four guards would keep passing while measuring nothing about the product — the
  same shape as a corpus stream nothing reads. Found while measuring M2, which is what exposed that
  the shuffle had NO consumer at all when the story was first built. **Owner: Epic 5's
  retrospective**, as an observation about test-support code rather than a defect to fix.

- ⚠️ **The corpus AC1 test is reddened by nothing in the permitted mutation set, and that is
  structural.** Measured: under a `join` mutated to first-key-wins, the synthetic test reds and
  `a_committed_stream_derives_the_same_interfaces_in_every_order` stays GREEN, because no committed
  observation carries more than one MAC and every stream carries one `l2_domain`. AC6 is therefore
  satisfiable in letter by a corpus that cannot see the mutation, and the synthetic slice is what
  carries the measurement. Registered rather than fixed: adding a multi-MAC or second-scope stream
  to `fixtures/` is a corpus change, and this story moves nothing under `fixtures/`.
  **Owner: Epic 6**, which implements `l2-*` and is the first epic with a reason to extend the
  corpus.

## Deferred from: code review of story-5-11b (2026-08-06)

- ⚠️ **`contradicts`'s exhaustive destructuring is retired SILENTLY by a `..`.** The guard is real —
  adding a field to `Observation` yields `error[E0027]` at `resolver.rs`, measured with a `vlan`
  field — but it is compiler-carried, and **replacing the pattern with `{ .., }` leaves the entire
  suite green** (measured). The story's AC5 asked for *"a test that reds when a new field is added"*;
  what shipped is a compile-time guard, and the AC text has been corrected to say so rather than
  ticked over the gap. A test cannot practically assert that a pattern is exhaustive.
  **Owner: whoever adds the next field to `Observation`** — a real condition, not a disguised
  *"someday"*, because that author is the one the compile error stops.

- ⚠️ **The count-assertion census is SIX, and five documents said five.** Four `consumed` counters,
  one `seeds == 8`, and one inside the helper `sampled_permutations` that guards shapes B and C a
  second time. Deleting only the four `consumed` lines leaves the two database consumers red at
  `left: 0, right: 12` — measured on both variants at the code review. Corrected everywhere it
  appeared. Recorded because the shape recurs: a claim about which guards carry a red is itself a
  claim needing a check, and this one was repeated across five files before anyone ran it.
  **No owner — closed**, but see Epic 5's retrospective for the pattern.

- ⚠️ **`XorShift64` has one dead seed and it is a plausible choice.** `new` xors with
  `0x9E37_79B9_7F4A_7C15`, so that exact seed lands on state zero and every draw is `0`. The
  degeneration is SILENT — Fisher-Yates with `j = 0` throughout still permutes (a fixed rotation),
  so both the multiset guard and the reproducibility guard pass on it. Now pinned by
  `the_dead_seed_is_named_and_outside_the_sweep`, whose load-bearing half asserts that no seed in
  `SEED_SWEEP` reaches the fixed point. **Owner: the first story that widens `SEED_SWEEP` or changes
  the fold** — the guard reds for them rather than leaving a fuzz test that quietly stopped fuzzing.

- ⚠️ **`sampled_permutations`' constants are tuned to `n == 6` with no precondition in the type.**
  `skip(1).step_by(60).take(12)` saturates exactly at 720 (indices 1 … 661) by construction, not by
  margin; at `n = 5` it yields 2 and the assertion blames the enumerator for a wrong step size; at
  `n = 7` the twelve samples cover the first 13% of the space, which is not the "deterministic
  spread" the doc calls it. Documented at the review rather than generalised, because no caller
  needs another size today. **Owner: the first caller that passes a slice of another length.**

## Deferred from: story-5-12-never-overwrite-anti-regression (2026-08-07)

- ⚠️ **`CHECK (actor_id <> 'scanner')` bans one padded VALUE, not a property.** Measured:
  `'engine'` is **accepted**, and `'scanner '` with a trailing space IS refused, because `actor_id`
  is `CHAR(36)` and MariaDB compares padded. `a_non_human_author_other_than_scanner_is_accepted_and_that_is_the_limit`
  pins both halves and is labelled as the honest limit rather than as a defect. The property is held
  by the `declared-authorship` gate, which stops such a write from ever being authored; the CHECK is
  a tripwire behind it. **Owner: Epic 6**, which brings the `actor` table an allowlist would need —
  an allowlist in DDL today would be a migration every time an actor is added.

- ⚠️ **`0001_initial.sql:16`'s comment is FALSE and cannot be corrected in place.** It reads
  `-- a human; never 'scanner'`, which promises authorship and delivers one spelling. sqlx checksums
  the migration file with SHA-384 over its whole text, comments included, so editing it breaks any
  database that has **already applied** it. ⚠️ **That is conditional, not absolute** — every test
  database here is created fresh, so an edited `0001` migrates cleanly locally AND in CI; what breaks
  is the NAS and any long-lived dev database. The truthful statement now lives in
  `insert_declared_attribute`'s doc instead. **Owner: the migration-consolidation milestone**
  (issue #50), the only point at which rewriting an applied migration is legitimate.

- ⚠️ **A table name assembled at runtime is invisible to the gate, and always will be.**
  `format!("declared_{}", "attribute")` defeats any text matcher.
  `a_table_name_built_at_runtime_is_invisible_and_that_is_stated` pins it as a KNOWN LIMIT rather
  than leaving the gate's promise unqualified — D18's rule applied to a gate's own blind spot.
  **No owner: it is a limit, not a defect.** A gate that claimed otherwise would be the decoration
  D18 forbids.

- ⚠️ **`DELETE` is deliberately outside the gate's verb list.** NFR5 is about AUTHORSHIP and a
  `DELETE` writes no author; including it was measured reddening the committed tree at two
  test-fixture sites (`main.rs:413`, `repo.rs:1110`). **A bulk delete of declared rows is a different
  invariant — data loss, not authorship — and this gate does not hold it.** Owner: the story that
  first needs a data-retention guarantee. Named here so the gate's scope is not read more widely
  than it holds.

- 🔴 **The allowlist has THREE sites where the story prescribed two.** The third is
  `docker/seed-example.sql`, matched by PATH rather than by function, and it is forced by the story's
  own requirement to walk `docker/`: the seed file writes a declared row with `'operator'` as its
  actor, which is legitimate under NFR5 (*"an operator writing a declared value through an explicit
  action … is not covered by this prohibition"*). ⚠️ **Consequence worth stating**: an edit to that
  file changing its actor to a non-human one would pass the gate. The DDL CHECK still catches
  `'scanner'` at runtime, and nothing catches `'engine'`. **Owner: the story that gives the seed file
  a test**, which today it has none.

- ⚠️ **FR13's `document` gesture will add a second legitimate Rust write path, and the gate will red
  on it.** NFR5 explicitly permits it (`prd.md:1209-1211`): an operator writing a declared value
  through an explicit action is a normal declarative write with a human author. Whoever implements
  `document`/`document-field` must add its writer to `SANCTIONED_FNS` — a one-line edit, but an
  invisible requirement unless it is written down. **Owner: the triage epic.**

- ⚠️ **NFR5 has THREE assertions and this story covers ONE.** Covered: the third (*"no code path
  writes a declared field with a non-human author"*, `prd.md:1218-1219`) and FR13's blindness
  corollary (`:1220-1221`). **NOT covered**: (1) *ingesting an observation that contradicts a
  declared field leaves that field unchanged and opens a divergence*, and (2) *documenting a field
  sets the declared value and leaves the observation record bit-for-bit unchanged*. Both need the
  `document` gesture and the triage inbox, neither of which exists. **Owner: the triage epic.**
  Registered so that "NFR5 is covered by anti-regression tests" is never read as met in full.

- 🔑 **A mutation-driver lesson, for whoever runs the next prove-to-red pass.** `git checkout` does
  not restore a DATABASE. A mutation that plants a `.sql` migration is applied by `sqlx::migrate!`
  and recorded in `_sqlx_migrations`; removing the file afterwards leaves the schema referencing a
  migration that no longer exists, and **every DB-backed test then fails for a reason unrelated to
  the mutation** — measured here as 64 spurious reds. The driver must drop and recreate the schema
  as part of its restore. Same family as the register's existing *"commit before the mutation pass"*
  entry. **No owner — a note for the method, not a defect in the code.**

## Deferred from: story-5.13 (2026-08-10)

_Appended, never rewriting the bullets above. Story 5.13 shipped the monotone-honesty MEASUREMENT
(NFR8(a)/D35(a)); what it deliberately did NOT ship is recorded here with its owner._

- **Lattice monotonicity is STILL not implemented, and story 5.13 is where it was re-owned rather
  than discharged.** The story-4.6c bullet above names *"Owner: Epic 5, as its 'monotone-honesty
  invariant trap family'"* — Epic 5 has now shipped that measurement and lattice monotonicity is
  **not part of it**, by design. 5.13 compares PLACEMENTS `(observation_id, interface_id)` and
  deliberately excludes `rule_id`, `evidence` and `outcome`, because a fault legitimately WEAKENS a
  justification and a row-level subset would red on a run that did exactly the right thing. D36's
  law is a statement about that excluded justification. **Split and re-owned: the doubt ORDER on
  `Verdict` is Epic 6's** (it needs `Supports`/`Opposes` to have a producer, which no rule provides
  today), **and the capability-snapshot half is story 5.13b's**, which commits the first trap-named
  stream carrying a `capability` control record — 11 of 11 trap-named streams carry none, so a
  `ScoredRecord` today would mean inventing a snapshot for all 24 traps (D36 in reverse).
- **AC3's inverse direction moved to 5.13b, on a measurement taken by BOTH validation layers.**
  Deriving a clean run from a committed control-record stream by REMOVING the control record is a
  **no-op**: measured `clean = 8 facts, faulted = 8 facts` on `partial-then-failed.jsonl` and on
  `capability-downgrade.jsonl` alike. Two structural causes — the failure record is LAST and must be
  (`read_records` forbids anything after it), and **a `capability` record filters no fact at all**;
  `poll` only reassigns `in_force` (`fixture_connector.rs:324`) and nothing downstream strips
  anything. The strip that makes a blinding bite is the MUTILATION's work, and no committed file has
  an equivalent. So the two streams are still *"judged by no trap"* as the story-4.6b bullet records,
  and 5.13b — which commits streams able to carry the assertion — owns it.
- **NFR8 has FOUR assertions and story 5.13 covers ONE.** (b) bounded blast radius and (c)
  convergence after recovery need the scheduler; (d) exactly one actionable notification needs the
  notification surface. *"NFR8 is verified"* must never be read as met. Owners: Epic 13 (Journey 4)
  for (b) and (c), Epic 16 for (d).
- **`observation_record` holds the CLEAN facts while the engine layer resolves the FAULTED ones.**
  Harmless — the resolver reads the slice it is handed and never the table — but it is a database
  state no real run produces, and it is recorded so a future story reading those rows knows why they
  disagree with the links beside them.


## Deferred from: story-5.13b (2026-08-11)

- **🔴 The corpus-wide `obs_id` anchor does NOT cover `fixtures/scenario/wire/`, and the
  compensation for that lives in a README rather than in a check.**
  `no_obs_id_is_shared_across_replay_streams` (`fixtures.rs:1858`) walks `scenario/replay/` only;
  `fixtures.rs:1063-1067` records that the wire artefact *"sits outside every corpus walk on
  purpose"*; and `scenario/wire/README.md:51-52` compensates with a RESERVATION —
  *"the `bdbdbdbd` obs_id prefix is RESERVED by this directory"*. **This story walked straight into
  it**: its first draft prescribed `bdbdbdbd` and four UUIDs byte-identical to
  `unifi-clients.expected.jsonl` lines 1-4, on a measurement correctly scoped to `scenario/replay/`
  and a conclusion drawn about the whole tree. ⚠️ **No gate would have caught it** — not the anchor,
  and not this story's own mutation M10, written precisely to prove that anchor live. It was found by
  READING, by the validation layer that did not build the story; the layer that DID build it reached
  492 green tests without seeing it. *A green build is not a uniqueness proof when the walk cannot see
  the file.* The honest closure is a check that reads the reservation, or a walk that covers `wire/`
  with the exemption stated in code rather than in prose. Owner: not assigned — registered with Epic
  5's retrospective.
- **A THIRD committed replay stream is judged by no trap**: `blinded-source.jsonl`, the clean twin.
  This is not the 4.6b bullet's shape — it is deliberate and structural. The clean twin exists to be
  the thing the faulted twin is compared against, and being control-free it also joins story 5.13's
  AC3 sweep, so it is measured by two mechanisms and judged by no trap. Recorded so nobody "fixes" it
  by authoring a family for it.
- **D36's capability snapshot: the PRECONDITION is supplied, and it is NARROWER than it reads.**
  `blinded-source-blinded.jsonl` is the first trap-named stream whose committed BYTES carry a
  `capability` control record — the count was 11 of 11 without one. ⚠️ But `read_traps` and
  `l1_runner::answer_trap` both go through `read_jsonl`, which drops control records
  (`fixtures.rs:647-657`, exhaustive `Failure | Capability => None`), so **the trap path is provably
  blind to that record**. It is reachable by a future `ScoredRecord` producer and by nothing today.
  *"D36 is now testable"* must not be read as met: no `ScoredRecord` is produced, the other 24 traps
  still name streams with no capability record, and the doubt ORDER on `Verdict` needs
  `Supports`/`Opposes` to have a producer. Owners unchanged — Epic 6 for the doubt order.
- **Story 5.2b's *"exchanging the two `observations` vectors leaves the whole suite green"* is STALE
  as a general claim, and this story measured where.** It was true when measured; since story 5.7 the
  committed corpus is scored by the real engine, so for a family whose swap demands a merge the engine
  refuses, the trap gate reds on its own — measured here: M6 reds 4 tests, three of them `trap_gate`
  tests that never consult the pin. The byte pins remain worth having (they pin WHICH pair each pole
  judges, and `family`, whose loss silently exempts a family from `incomplete_families`), but the
  justification must be re-derived per family rather than quoted from 5.2b.

## Deferred from: code review of story-5.13b (2026-08-11)

- **🔴 The blinded-source family's MARGINAL coverage over the existing corpus measures ZERO, and its
  second pole cannot fail on an abstention.** Measured by the review's Edge Case Hunter: mutate the
  engine to abstain whenever a pair's observations report different fact kinds — the plausible D18
  cowardice a *"source goes half-blind"* family exists to forbid — and the gate reports
  `failures: {MustMerge: 2}`, `MustNotMerge: 0`. The two failures are `blinded-source-must-merge`
  **and the pre-existing `hostname-absence-must-merge`**, whose same-MAC pair already differs in fact
  kinds, so that class was covered before this story. And the negative pole stays GREEN because
  `(must-not-merge, Abstained) → Pass` is a deliberate, load-bearing cell in `score.rs`. ⚠️ ***"both
  of D18's poles"* must therefore not be read as two independent gates.** Registered rather than
  patched: the family is honest at what it does (it puts a `capability` record inside a trap-named
  stream, which is its §9 purpose) and inventing a shape purely to raise its coverage would be the
  bent metric D45 forbids. Owner: Epic 5's retrospective.
- **🔑 A promise of NON-MODIFICATION protects behaviour and shelters false sentences.** Story 5.13b
  wrote *"`opencmdb-core` is not touched"* in four documents as a virtue; it is exactly what kept it
  out of `score.rs:443`, whose comment asserts as a MEASUREMENT the very thing the story's deliverable
  falsified. Guy narrowed the promise to *"no BEHAVIOUR change"* and the sentence was corrected.
  **The transferable form: a "does not touch X" clause must be scoped to behaviour, or it becomes a
  reason not to look at X.** Carry to the retrospective.
- **⚠️ The mutation-driver family recurred a FOURTH time, and twice inside one story.** (a) M10's red
  count was taken from a FILTERED run and reported as the full-suite figure — 1 where the truth is
  37. (b) M7's *"unreachable in a full run"* rested on a `head -8` applied to the driver's own output:
  the full run WAS executed and its evidence truncated before it was read, after which one test's
  assertion order was generalised to the whole suite. 🔑 ***A measurement read through a truncation is
  not a measurement***, and it is the same failure as story 5.13's *"`cargo test --workspace A B`
  passes two filters"* wearing different clothes. (c) During the review repairs, a
  `git checkout -- <file>` intended to revert a MUTATION reverted the file and ate a guard written
  minutes earlier — story 5.13's driver defect reproduced by the person correcting the story that
  documents it. **Revert the mutation, never the file.** Owner: Epic 5's retrospective.

## Deferred from: story-5.14 (2026-08-11)

- ✅ **`:2407` — "the resolver is NOT wired into `main.rs`" — CLOSED, and BOTH halves of its own
  sentence are named.** It read *"wiring it would make every deployment write links with no page to
  display them **and no purge to remove them**"*. The first half is now deliberately true for one
  story: 5.14 wires the pass, 5.14b displays. ⚠️ **The second half is the one that now bites, and it
  is owned by NEITHER story.** `repo::purge_engine_links` exists since 5.10 and has **no production
  caller** — every caller is a test — while the pass accumulates one current abstention link per scan
  per unplaceable host (measured: five runs over one host → five links; ~105 000 a year at a
  five-minute interval). **Owner: unassigned.** It is registered here rather than folded into 5.14b,
  because a purge is not a display concern and pretending otherwise would hide it behind a story
  whose acceptance criteria cannot fail on it.
- ⚠️ **The abstention counter's DENOMINATOR is undecided, and deciding it is grouping.** A count over
  current engine links measures scan iterations, not reach. Collapsing sightings of one unplaceable
  thing means deciding what makes two sightings the same thing WITHOUT an identity — which is Epic
  6's subject. 🔑 **And the naive fix is measured worse than the defect**: widening `resolve_within`'s
  vacate pass to close slots of observations it never saw reds four tests, three of them pre-existing
  `resolver` tests, because it **erases a host that missed a single scan**. **Owner: story 5.14b and
  Epic 6**, together — the story pins the accumulation with a test whose message says *do not repair
  this number*.
- ⚠️ **`ContradictoryObservation` and `InstantRegressed` are UNREACHABLE from a scan slice**, measured
  across five runs of the real binary: the ARP/ping connector mints a fresh `Uuid::now_v7()` per
  observation and stamps one `observed_at` per poll. So `:2772`'s concern — that the refusal's
  reachability rests on one test — is **answered for this caller and survives for the others**: the
  seam is generic over `Connector`, and a connector that reuses ids can reach both. Owner unchanged.
- ⚠️ **`count_identity_links` still has no production caller, and now it has a NEIGHBOUR that does
  what it cannot.** `scan_pass::counted_current_engine_links` filters on `decided_by = 'ENGINE'` and
  `current_subject IS NOT NULL`; `count_identity_links` is an unfiltered `SELECT COUNT(*)`, so it
  would agree only by accident and diverge the first time a link is superseded. Recorded so the two
  are not mistaken for duplicates and so nobody "unifies" them. **Owner: story 5.14b**, the first
  story with a human-facing count.
- 🔑 **Three lines of the startup path are carried by NOTHING, and the size is now measured.**
  `spawn_startup_scan` is a `std::thread::spawn` whose handle is dropped, inseparable from a live
  ICMP poll, so nothing can assert inside it. The seam took the uncarried region from *the whole
  poll-ingest-resolve wiring* down to **build the connector, connect the pool, call the seam** —
  deleting the call to the seam leaves the suite green (M1), deleting the `resolve` call inside it
  reds three tests (M1b). **Owner: whoever makes the scan joinable or injectable** — the periodic
  scheduler (FR6) is the natural place, since it must own the scan's lifecycle anyway.

## Deferred from: code review of story-5.14 (2026-08-11)

- **🔴 TWO CONCURRENT PASSES MINT TWO INTERFACES FOR ONE MAC, both reporting success.**
  `interface_l1_key (l2_domain, mac_canon)` is a plain index, not UNIQUE, and `repo.rs:383` is a
  `SELECT … LIMIT 1` followed by an INSERT — read-then-insert, not atomic. Measured with
  `tokio::join!` over two passes on one pool: **interfaces = 2, links = 2, both `Ok`** — no error,
  no abstention, nothing red; the same input run sequentially gives 1 interface, which is the
  control. **One physical NIC silently becomes two identities.** Guy's arbitration (2026-08-11):
  **registered here, not fixed in a wiring story** — a UNIQUE index is DDL whose effect on story
  5.10's replay and story 5.11's idempotence must be measured first. ⚠️ **It is not reachable through
  today's connector, and the reason is the shield this story spent itself documenting**: the
  ARP/ping connector emits no MAC, so nothing scanned reaches the `interface` mint at all. **The
  connector story that gives it a MAC REMOVES THAT SHIELD and must carry this race with it.**
  Owner: that story, jointly with whoever adds the UNIQUE index.
- **⚠️ The two `arp_ping` pins are a TRIPWIRE, not a barrier, and the difference is measured.** A
  `Fact::Mac` added at the emit site inside `poll` — rather than inside `emitted_facts` — leaves all
  502 tests green while the real binary mints an interface and places a link. That bypass is the
  shape the upgrade MUST take: a MAC comes from a neighbour lookup keyed on the address, which
  `emitted_facts(ip, millis)` cannot reach. Read the pins as *"the named fact set and the named
  descriptor still agree and still exclude the MAC"*, **never** as *"nothing the shipped product
  emits can carry one"*. The barrier needs either a test over what `poll` really emits — which needs
  the ICMP socket, and every such test is gated on `OPENCMDB_NET_TESTS`, **which CI never sets**, so
  a mutation against it returns green because the test SKIPPED — or a connector that routes every
  fact through one construction site. **Owner: the connector story**, or whoever decides CI should
  set that variable.
- **⚠️ `current_subject IS NOT NULL` is NOT equivalent to `valid_to = OPEN_END`, and `repo.rs:991`
  states the equivalence as fact.** `identity_link_current_subject` compares
  `current_subject = interface_id`, which is UNKNOWN — not FALSE — when `current_subject` is NULL,
  and SQL accepts a CHECK evaluating to UNKNOWN. Measured: a row with `valid_to = OPEN_END` and a
  NULL `current_subject` is **accepted**, while the same disagreement by value is refused
  (`ERROR 4025`); `UNIQUE (observation_id, current_subject)` does not bound it either, NULLs being
  distinct. Story 5.14's read then reports **0** where `valid_to = OPEN_END` reports **2**. ⚠️ Not
  reachable through the adapter today (`close_identity_link` moves both columns together) — but
  this story is the first to adopt `current_subject IS NOT NULL` as the DEFINITION of a
  human-facing population. 🔑 **And there is a coupling nobody had recorded**: tightening the CHECK
  to what its own comment claims reds **exactly one test in the whole workspace** — this story's
  `a_superseded_link_is_not_counted`, whose doc calls its row *"superseded"* while its `valid_to` is
  still `OPEN_END`. The guard is genuine; its stated justification is not, and it now stands in the
  way of the DDL repair. **Owner: unassigned.**
- **⚠️ Two entries of story 5.14's §8 were never appended** — `:2700` (`observed_at` stability across
  passes) and the page-less deployment — while two bullets that are not §8 rows were. Recorded here
  so the omission is not read as a disposition. `:2700` stands: the accumulation IS its consequence,
  measured. The page-less deployment lasts **until 5.14b ships, which may be with Epic 6** — the
  weaker true sentence, where an earlier draft said *"for one story"* and was corrected in the story
  file but not in the register.

## Registered for Epic 5's RETROSPECTIVE — raised by Guy, 2026-08-12, not by a story

Guy raised this in conversation while story 5.14b was at `ready-for-dev`, and decided the same day
that **Epic 5 finishes, then the retrospective runs as the project's REORIENTATION point** rather
than as a closing formality. It is registered here because it has no story owner: nothing in the
Epic 5 story set would have carried it in, and this project's rule is that a retrospective item is
carried in by name rather than rediscovered.

- **🔴 THE QUESTION, in Guy's words: *"le projet dérive et rien n'est actuellement utilisable —
  est-ce juste ?"*** The measurements below were taken to answer it and are recorded so the
  retrospective starts from figures rather than from an impression.
- **Two thirds of the delivered work is invisible to the operator.** 63 stories delivered in 26 days
  (first commit 2026-07-17, 130 commits at `5046cca`). Epics 1-3 = **21 stories** and they produced
  *everything that is usable today* — the page, the scan, the gap. Epics 4-5 = **43 stories** and
  they produced **no operator-visible change at all**. Seven of the last eight stories changed
  nothing an operator can see; 5.14b would be the first display story since 3.7.
- **🔴 The DOCUMENTING GESTURE does not exist in the product**, and for a tool whose purpose is to
  document an IT infrastructure that is the finding, not a detail. The binary exposes **five routes,
  all read-only** (`/`, `/gap`, `/assets/*`, `/metrics`, `/healthz`) — **no write route**. The only
  call to `insert_declared_attribute` outside `repo.rs` is `main.rs:442`, **inside the `#[cfg(test)]`
  opened at `:362`**. To declare anything, an operator writes SQL by hand or copies
  `docker/seed-example.sql`. ⚠️ Measured, not supposed.
- **The scan is startup-only** (`main.rs:133`, `:171` — the periodic scheduler, FR6, is deferred),
  and the page shows **ONE entity**, chosen by `OPENCMDB_ENTITY_IPV4` or the first declared entity
  carrying an `ipv4`. It is a card, not an inventory.
- **⚠️ What is NOT drift, and the retrospective must not "fix" it**: writing the trap corpus and the
  metrics harness BEFORE the engine is D19, deliberate and defensible — *"a metric written after the
  engine is bent to fit the engine"*. Interface identity is the genuinely hard part of the domain.
  **The direction is defensible; what has drifted is the RATIO of rigour to reach.**
- **The apparatus has begun to measure itself.** Six of Epic 5's twenty stories are INSERTIONS
  discovered during the work, and a large share of review findings are *sentences* — documents
  contradicting each other — rather than behaviour. 🔑 **And the story documents are outgrowing the
  work they describe**: story 5.14b's file was **643 lines at contexting**, for a read-only display
  change. ⚠️ **And this bullet carried that figure as if it were current** while the file had already
  grown past 800 — a number written in flight, inside the sentence about numbers written in flight,
  in the commit that appended it. Found by the code review. *The figure is now dated; the point it
  makes does not need a live number.*
- **⚠️ Epic 5 will close with an identity engine that places NOTHING in production.** The only
  connector `main.rs` reaches emits no MAC, and `join` keys on `(l2_domain, mac)` — so on a real
  network 5.14b's new section will show exactly one line, *"no proof of identity"*, for everything.
  This is measured (story 5.14) and is pinned by tests on purpose. **What unblocks it is not in Epic
  5**: a connector that reads the neighbour table (Epic 11 or 12), and Epic 6 for grouping. ⚠️ And
  that connector story **removes the shield** over the registered two-passes-mint-two-interfaces
  race — see the code review of story 5.14 above.
- **A question the retrospective should answer with a number**: *how many epics before an operator
  can document one machine?* If the answer is "several", the reorientation to weigh is a short
  slice giving the declared side a write surface and the scan a period — enough to make the tool
  testable on a real network, which is also the best drift detector there is.

## Deferred from: story-5.14b (2026-08-12)

- ⚠️ **The `abstention_cause` column has NO DOMAIN IN THE SCHEMA, and this story deliberately did
  not give it one.** Arbitration 11 refused a DDL `CHECK (abstention_cause IN (…))` — not merely
  because it is DDL in a display story, but because it **moves the failure from the DISPLAY to the
  WRITE**: a variant added by a future story would then be refused at insertion, i.e. the identity
  pass would start failing rather than the page showing an unfamiliar label. 🔑 *A display story may
  not be the place a write starts failing.* What ships instead is a TOLERANT READER
  (`page::identity_cause_label`), which counts and labels an unfamiliar token and never fails.
  ⚠️ **The cost is stated rather than implied**: the column's domain is held today only by
  `repo::cause_token` being its sole writer — a property of the CODE, not of the schema. Measured:
  `UPDATE identity_link SET abstention_cause = 'a_cause_no_variant_names'` succeeds. The schema-side
  closure is registered here as the real one, on story 5.12's precedent where voie B's `GRANT` was
  registered rather than implied by the tripwire that shipped. **Owner: unassigned.**
- ⚠️ **Two UX bans are NAMED by this counter, not merely unmet by it.** `ux…:1280`'s *"No badge, no
  growing counter"* and `epics.md:1704`'s *"after six months of inaction it reads the same number"*.
  The number grows while the operator is inactive because the scanner keeps scanning — measured at
  story 5.14 (five runs over one host → five links). **Arbitration 13 makes the UNIT true
  (*sightings*, not devices); it does NOT make the bans met.** A figure that rises because the
  product looked many times is the radar's range rather than the operator's debt, and that is the
  whole of what the unit buys. **Owner: Epic 6**, which gives the population an identity.
- ⚠️ **The unit *sighting* / *constat* is TEMPORARY and its locale keys change with Epic 6.**
  Registered so the rename is met as a scheduled consequence rather than as the correction of a
  mistake. The note lives in `locales/app.yml` beside the keys themselves. **Owner: Epic 6.**
- ⚠️ **This story DIVERGES from the UX spec's mock, deliberately, and the spec is NOT edited.**
  `ux-design-specification.md:919-928` shows ONE panel (`187 evaluated · 113 not evaluated`);
  arbitration 10 ships TWO framed sections because two populations inside one frame invite the
  reader to add them, and the invitation is the defect (the counts range over declared FIELDS and
  over SIGHTINGS; their sum denotes nothing). Registered so the divergence is met as a decision
  rather than as drift. **Owner: Epic 6**, which revisits this surface.
- ✅ **`count_identity_links` (`Owner: story 5.14b`) — ANSWERED, and the answer is that all THREE
  reads stay.** `repo::count_engine_reach` (this story's, filtered + grouped) and
  `scan_pass::counted_current_engine_links` (filtered, ungrouped) see the same population — now
  PINNED by `the_grouped_read_subsumes_this_count` rather than reasoned about, on both the full set
  and the filtered one. `repo::count_identity_links` is the **only unfiltered** one (`SELECT
  COUNT(*)`, no `WHERE`) and still has no production caller. ⚠️ Do not "unify" them: the word
  *unfiltered* belongs to the third and must not be spent on the second, which is merely ungrouped.
- 🔴 **`scan_pass.rs`'s *"Story 5.14b is its production consumer"* was FALSE and is corrected.** It
  was a prediction about this story shipped as a statement, in the doc of an `#[allow(dead_code)]`
  function. 5.14b's human-facing count is the grouped read; `counted_current_engine_links` remains
  the instrument of story 5.14's four pins, which is a real job and is not a production caller.
- 🔑 **A guard placed where the defect cannot occur reads as coverage and is none** — carried into
  Epic 5's retrospective. This story's own anti-sum guard (AC3) tested that `build_view` and
  `build_identity_view` do not add each other's counts, and **neither of them can**: neither sees
  the other's numbers. The only place a sum can be written is `reconcile_view`, the impure edge that
  assembles both, and no unit test reached it — measured, the summing mutation left the whole suite
  GREEN. Closed by a database-backed test through the composition. ⚠️ **It was found only because
  the mutation was run**; reading the guard could not have found it, since the guard is correct
  about what it tests.

## Deferred from: code review of story-5.14b (2026-08-12)

- ⚠️ **Full table scan on `identity_link` for every page load, on a table this story documents as
  unbounded.** At story 5.14's own one-year figure (105 000 current engine links) `EXPLAIN` on
  `count_engine_reach`'s query gives `type: ALL, rows: 103761, Using where; Using temporary; Using
  filesort`, profiled at **24.8–25.4 ms**. No index covers `(decided_by, current_subject)`. Refresh
  is a button (`hx-get="/gap"`), not a poll, so the cost is per page load rather than continuous —
  recorded with its bound rather than as an alarm. It belongs with the accumulation it measures.
  **Owner: Epic 6**, jointly with whoever adds the index.
- ⚠️ **An empty `abstention_cause` token renders `Unrecognised cause ()`.** `''` satisfies
  `IS NOT NULL` on a `VARCHAR(32)` with no `CHECK`, so it is storable, and it collapses to the same
  displayed string as the DDL-forbidden NULL — the two are indistinguishable on the page, and
  neither gives the operator anything to report. Belongs with this story's registered *"the
  `abstention_cause` column has no domain in the SCHEMA"* entry and closes with it.
  **Owner: unassigned.**
- ⚠️ **`has_any` and `causes` can disagree on a zero-count group.**
  `build_identity_view(vec![reach("abstained", Some("absence_of_proof"), 0)])` gives
  `has_any = false` with one cause row: the page prints *"Nothing observed yet"* and silently drops
  the line it is holding. **Unreachable from `COUNT(*)`**, which never returns a zero group — a
  totality wart recorded as such rather than patched, so that a future caller feeding this function
  from something other than the grouped read meets it. **Owner: unassigned.**
- 🔑 **An `xtask` gate is what would really carry clock-freedom in the view builder; a test cannot.**
  Story 5.12's precedent, in its own words: *you cannot measure the absence of code by running code
  — a test exercises what exists, while the violation is what a future story ADDS.* Measured here:
  a wall clock rendered as an *"as of day N"* note leaves all 519 tests green, because the guard
  renders twice microseconds apart and sees only a clock finer than that gap. ⚠️ **The real defence
  today is a dependency configuration, not a test**: `chrono` is `default-features = false`
  workspace-wide, so `Utc::now` does not exist (`error[E0599]`) and only `std::time::SystemTime`
  gets through. A gate on `float-free`'s model — reddening on `SystemTime::now` / `Instant::now` /
  `Utc::now` under the view-building region — is the closure. **Owner: unassigned.**

## Deferred from: story-5.14b, second pass (the four §11 rows that never landed, 2026-08-12)

🔴 **These four were required by story 5.14b's §11 and were NOT written by its first register pass.**
The AC demanded *"appended and then re-read to check each row landed"*, and a re-read WAS run — it
counted the seven bullets written and compared them to the seven bullets written. 🔑 ***A re-read
that reads only what you wrote cannot find what you did not write.*** The check must count against
the REQUIREMENT (§11 had nine rows), not against its own output. Carried to Epic 5's retrospective,
because it is the same shape as the mutation-driver family: an instrument that confirms itself.

- ⚠️ **The DENOMINATOR entry's owner is reduced to Epic 6.** `deferred-work.md:3032` still reads
  `Owner: story 5.14b and Epic 6`, and 5.14b has now shipped — a closed story left standing as
  co-owner of an open item. **The entry at `:3032` is NOT rewritten** (the register's rule); this
  bullet supersedes its ownership. 5.14b's half is DONE: it states the limit in the surface, in the
  operator's language, in both locales. **Owner: Epic 6 alone.**
- ⚠️ **`:2407`'s PURGE half is untouched by 5.14b, and the story made it MORE acute rather than
  less.** `repo::purge_engine_links` still has no production caller while the pass accumulates one
  current link per sighting per scan. This story made the accumulation VISIBLE — a page now displays
  it — and **a visible defect is not a fixed one**. **Owner: unassigned.**
- ⚠️ **The `current_subject IS NOT NULL` ≠ `valid_to = OPEN_END` non-equivalence gained a THIRD
  adopter.** Story 5.14 was the first to make that predicate the definition of a human-facing
  population; `count_engine_reach` is now the second read to do so, and this one is rendered to the
  operator. Each adopter raises the cost of repairing the DDL, whose CHECK accepts a row the
  predicate then misses. Recorded as an increment, not as a new defect. **Owner: unassigned.**
- ⚠️ **`epics.md`'s `Ambiguous`-shows-its-candidates clause is re-owned to Epic 6 WITH a tripwire.**
  `scan_pass::the_production_pass_produces_no_ambiguous_abstention` reds the day a producer of
  `Verdict::Supports`/`Opposes` arrives, and its message names the clause as due. ⚠️ Its red is
  `.expect()`-carried on its own premise (the pass rolls back under `guard_decision`), so it signals
  *the pass stopped completing* rather than *an ambiguity was written* — the signal wanted either
  way, but not the assertion the test reads. **Owner: Epic 6.**

## Deferred from: story-5.14b, the operator's three cases (Guy, 2026-08-12)

🔑 **Guy's taxonomy, and it is the criterion for Epic 6 AND for whatever slice makes the product
usable.** It came out of the code review's `no_match` finding and settles it:

| case | what the engine wrote | who acts | the gesture |
|---|---|---|---|
| **no ambiguity** | `Match`, and also `NoMatch` | the software | none — it decided |
| **ambiguity** | `Abstained { Ambiguous }` | the operator LIFTS THE DOUBT | choose among the candidates and their evidence (FR16, `link_candidate`) |
| **unknown** | `Abstained { AbsenceOfProof }` | the operator CREATES THE ENTITY | **declare** — the documenting gesture |

- 🔴 **Case 3's gesture is the one the product does not have at all.** Five routes, all read-only;
  the only call to `insert_declared_attribute` outside `repo.rs` is inside a `#[cfg(test)]`. And
  case 3 is **the only case reachable today**: the shipped connector emits no MAC, so every
  abstention on a real network is `AbsenceOfProof`. **The abstention section is therefore not a
  counter — it is the entry point of the documenting gesture**, which is why it reads as hollow
  until that gesture exists. **Owner: the slice that gives the declared side a write surface.**
- ⚠️ **A cause that opens no gesture must say what would make it disappear**, or it is not a map,
  it is a complaint. For `AbsenceOfProof` that sentence is *a source that reports hardware
  addresses* — the connector story, not a human decision. Asking the operator to "resolve" a signal
  the source never sent would make him carry a lack of the SOURCE, which is the reproach the
  dignity rule forbids. **Owner: Epic 6**, as a design criterion.
- 🔴 **NOT announced in the surface, by decision (Guy, 2026-08-12): the section stays DESCRIPTIVE
  until the gesture exists.** *Announcing an absent gesture is a promise.* The day either gesture
  ships, the corresponding line gains it — and until then the page says what is, not what will be.
- ⚠️ **The apparatus did not find this.** Three review layers, nineteen findings, and not one asked
  *"what can the operator DO with this number?"* — because they checked conformance to a
  specification, and the specification was mine. **A blind spot of the method, not an accident.**
  Carried to Epic 5's retrospective.

## Deferred from: story-6.1, the write route and the Basic decision (2026-08-14)

Ten rows, each with its owner (AC7 — re-read against the AC's list, not against this output;
story 5.14b's AC10 failed exactly there).

- **(a) The two-browser Basic browser measurement.** ✅ **FULLY DISCHARGED 2026-08-14, on BOTH
  engines** (Chrome/Blink 151 AND Firefox/Gecko 153, three runs each, all three probes): the
  two-origin CSRF bench measured that htmx 2.0.4's same-origin `hx-post` carries `Origin` = page
  origin, and a cross-site `<form>` POST carries `Origin` = attacker ≠ `Host` AND the cached
  `Authorization: Basic …` PREEMPTIVELY — the ambient-authority threat freshly measured, refused
  by §5's Origin check. The Gecko cell — first reported unmeasurable — was captured by invoking
  the raw snap Firefox binary (`/snap/firefox/current/usr/lib/firefox/firefox`), the wrapper's
  single-instance lock having been the blocker, not headless itself. **No residual remains on
  this row.** _(This row read "DISCHARGED-for-Chromium … Gecko residual owed" until the second
  bench pass closed it the same day.)_
- **(b) Credential expiry / the browser's native dialog mid-swap.** The vendored htmx 2.0.4's
  default `responseHandling` does not swap a 4xx into `#gap-card` (read during validation), so
  the residual concern is only the native dialog. **Owner: story 6.4.**
- **(c) `scrape_authorized` refuses a lowercase `bearer` scheme** (RFC 7235 §2.1 wants
  case-insensitive) — a recorded defect, deliberately NOT fixed by story 6.1, whose new Basic arm
  is case-insensitive and does not copy it. **Owner: Epic 19.**
- **(d) The Basic comparison is not constant-time** (`==` on `String`) — **and the `&&`
  short-circuit is a username-confirmation oracle**: a user mismatch skips the password compare
  entirely, so timing distinguishes *right user, wrong password* from *wrong user* (the code
  review widened this row — the first wording under-described what must be closed). A stated
  limit (single-operator LAN product, TLS at the proxy), not silently "fixed" with a new
  dependency. **Owner: Epic 19, both halves of the leak.**
- **(e) The `--accent` colour conflict** (story 6.1 §9): story 5.14b's guard asserts the identity
  section never reaches `--accent`, and story 6.4's Document button in that section will be
  legitimately amber; a top-level class evades the guard entirely (measured). 6.4 must re-examine
  the guard — narrow it to the counter and cause lines — not merely satisfy it; and Epic 6b's
  story 6b.1 re-tokenises `app.css` first, so whoever lands second re-checks the guard exists.
  **Owner: story 6.4, conditional on 6b.1's ordering.**
- **(f) The release-notes obligation: the UI stops being publicly readable** (arbitration 2′'s
  price — `/` and `/gap` left `is_public`). The first release CONTAINING story 6.1 names it in
  its release notes; the obligation follows the release, not the story number. **Owner: the first
  release containing this story — today Epic 6b's story 6b.12.**
- **(g) `document-field`** — FR13(b), documenting a re-discovery field by field. **Owner: Epic 7**
  (the FR coverage map already assigns it there; recorded at Epic 6's decomposition).
- **(h) Basic's closure — real sessions** (users, revocation, a login form; one shared credential
  authenticates a caller, not a person). **Owner: Epic 19.**
- **(i) The D37 filename drift** — ✅ **DISCHARGED by story 6b.1 (2026-08-18)**: the asset is now
  `assets/vendor/htmx-2.0.4.min.js`, the version read from the file itself (`version:"2.0.4"`), the
  template updated, and both directions pinned — the versioned path is embedded and served (200),
  the old one is **404**, measured through the running binary. _(Original wording follows.)_
  The vendored file was `htmx.min.js` unversioned where
  `architecture.md:3406` names a versioned filename. **Owner: Epic 6b story 6b.1.**
- **(j) 🔴 CSRF protection for the write route.** ✅ **CLOSED by story 6.2's Origin check**
  (contexted + validated 2026-08-14, §5): a browser holding the cached Basic credential and a
  cross-site page cannot forge a documenting write — the POST carries the attacker's `Origin`
  (measured, Blink), which the check refuses 403 before any refusal path consults the form. ⚠️
  **RESIDUALS re-registered, each stated at its strength**: pre-2020 browsers that omit `Origin`
  on a cross-site POST are not protected (LAN single-operator product); the check needs the
  reverse proxy to FORWARD `Host` (`proxy_set_header Host $host;` — nginx's default rewrites it
  and would 403 every POST); `Host`-absent HTTP/2 direct clients are refused. _(The
  Gecko-cross-site bench cell was owed until the second bench pass measured it — row (a) is now
  fully discharged.)_ **Owner of the residuals: Epic 19** (real
  sessions + a session token supersede the Origin heuristic entirely).

## Deferred from: code review of 6-1-write-route-writes-nothing (2026-08-14)

- **`lazy_pool()` couples test outcomes to whatever answers `127.0.0.1:3306`.** The idiom predates
  story 6.1 (the auth test used it first); the new tests reuse it and assert 503/`!= 401` on the
  premise that `root:x@127.0.0.1:3306/none` never authenticates. Measured safe today — locally the
  unrelated container refuses the credentials, in CI the service does (green run, PR #89) — but the
  premise is environmental, not structural. The clean shape is a port known dead (bind-then-close)
  or a config-injected address. Owner: the next story that touches the test helpers.
- **The subject parser accepts every RFC-4122 textual spelling and any UUID version.** The NIL
  sentinel is now refused at the route (this review's patch, D21/D48); whether the store-backed
  lookup should also require the canonical hyphenated form and/or v7 is the write story's
  question — a braced or urn: spelling of a REAL id parses to the same id today, which is
  harmless until something round-trips the text. Owner: story 6.2.

## Deferred from: validation of 6-2-route-writes-a-declared-value (2026-08-14)

- **The canonical-UUID form question (story 6.1's review registered it here).** ✅ **CLOSED by
  story 6.2 §2**: the subject id is bound as `subject.as_uuid().to_string()` — canonical
  hyphenated lowercase — before any SQL sees it, so a braced/urn:/hyphenless spelling of a real
  id is harmless BY CONSTRUCTION (measured: a braced spelling answers 201). No further work.
- **🔴 The authorship gate now carries a READ-sanction (story 6.2 §6.5, Guy's arbitration
  2026-08-14).** 5.12's gate guarded the WRITE of provenance; 6.2 is the first story with a
  legitimate provenance READER (a test verifier), so the gate gains a `SANCTIONED_READS`
  allowlist, FR13-framed (*the divergence computation* may never read provenance — not *all*
  code). It is a TRIPWIRE against a future read into a divergence path, never a barrier — the
  same narrowed promise 5.12 stated for the write half. **Owner: Epic 6's retrospective** (it is
  a deliberate widening of 5.12's apparatus and should be reviewed as one).
- **`actor_id = 'operator'` is a LITERAL — no real actors yet.** The documenting write records a
  human author, but the Basic pair authenticates a caller, not a person (6.1 §3), so every
  documented row carries `'operator'`. Real per-user actors need sessions. **Owner: Epic 19.**
## Deferred from: story-6.2, the write path (2026-08-14)

Re-read against the story's AC8 list.

- **(1) The `epics.md:1768` wording divergence** — *"through `insert_declared_attribute`"* reads
  as *"through the repo adapter"*; the write goes through the sibling `adopt_declared_attribute`.
  **Owner: Epic 6's retrospective** (`epics.md` untouched, verify-only rule).
- **(2) CSRF residuals** (row (j) is CLOSED by the Origin check): pre-2020 browsers that omit
  `Origin` on a cross-site POST; the reverse proxy must FORWARD `Host` (`proxy_set_header Host
  $host;`); `Host`-absent HTTP/2 direct clients are refused; the compare is scheme-blind; the
  Gecko-cross-site bench cell. **Owner: Epic 19** (sessions + a token supersede the heuristic).
- **(3) `actor_id = 'operator'` is a LITERAL** — no real per-user actors; the Basic pair
  authenticates a caller, not a person. **Owner: Epic 19.**
- **(4) 🔴 The authorship gate's READ-sanction (§6.5)** is a TRIPWIRE against a future story
  reading provenance into a divergence path, never a barrier — the narrowed promise, a deliberate
  widening of 5.12's apparatus. **Owner: Epic 6's retrospective** (review it as one).
- **(5) The invisible entity** — `build_view` selects an entity by its declared `ipv4`, so a
  hostname-only subject documents 201 but mints an entity the view can never select. Not this
  story's bug (the entity model is 6.5's). **Owner: story 6.5.**
- **(6) Epic 7's `document-field` must negotiate the PRIMARY KEY**, not this index: re-documenting
  one entity's same field reds `1062 … PRIMARY` (measured). **Owner: Epic 7.**

## Deferred from: code review of 6-2-route-writes-a-declared-value (2026-08-14)

- **HTTP/2-direct and scheme-blind CSRF limits.** `same_origin` refuses a `Host`-absent request
  (HTTP/2 `:authority`) and is scheme-blind; both are stated limits — the product deploys behind a
  reverse proxy that forwards `Host` (`architecture.md:168`). **Owner: Epic 19** (sessions + a
  token supersede the Origin heuristic).
- **`is_adoption_conflict` couples to the DB error message text.** Measured robust on MariaDB
  10.11.11 (the index name is interpolated data, not localized prose), but a SQLSTATE `23000` +
  name check would be strictly more robust. **Owner: whoever next touches the adapter's error
  classification.**
- **The first-occurrence-wins dedup is hand-rolled in `document.rs`** where `gap::project`'s doc
  promises the first-occurrence convention. Extract a shared helper when the second caller lands
  (story 6.4's button / Epic 7's `document-field`). **Owner: story 6.4 or Epic 7.**
- **`observed_at` nano-precision truncates at INSERT** (`%.6f`, pre-existing) — harmless for 6.2
  (`observed_at` is not a declared field), and `load_observation_by_id` is faithful to what is
  stored. **Owner: the story that first needs sub-microsecond instants (none foreseen).**

## Deferred from: code review of 6-2 — the multi-value decision (Guy, 2026-08-14)

- **A same-key multi-value observation (e.g. two `Fact::IpV4`) documents "success" (201, N
  fields) while its gap stays OPEN.** Measured: `document-all` writes the FIRST value per key
  (forced by the PK), answers 201, but `reconcile` reads the two observed values as conflicting,
  drops the field, and the just-documented `ipv4` returns as a `NoObservedValue` abstention — the
  operator is told they closed a gap that stays open. §3 documents the abstention. 🔴 **NOT
  reachable today**: no shipped connector or fixture emits two facts of one kind in one
  observation (the `multi-nic` family is N single-MAC observations). **Guy's decision: DEFER** —
  keep first-wins, do not add a model decision here; the one-value-per-`attr_key` model is story
  6.5's and multi-value facts are a connector story's concern. **Owner: the story that first makes
  a same-key multi-value observation reachable** (a connector emitting multi-value facts, or the
  entity model of 6.5). The fix then is either to refuse a self-conflicting projection or to model
  multiple values — a decision that story owns, not 6.2.

## Deferred from: story-6-3-nfr5-remaining-assertions (2026-08-15)

- ✅ **NFR5's register row from story 5.12 is DISCHARGED — and its width is stated rather than
  implied.** `deferred-work.md:2894-2900` left assertions **(1)** and **(2)** owned by *"the triage
  epic"*; both are now measured. ⚠️ **What "covered" means here**: assertion (2) is carried by a
  behavioural snapshot AND by the eighth gate, because on the tree that introduced them nothing
  could move the row; assertion (1)'s *unchanged* half is unconditional, while its *a divergence
  opens* half needed the test to delete the documented sighting (rows below). *"NFR5 is covered by
  anti-regression tests" is now TRUE at that width and no wider.* **No owner — closed.**

- 🔴 **`epics.md:1724` says NFR5 *"(assertions 2 and 3)"* where the PRD's ordering and the
  register make the parked pair **(1) and (2)*** — assertion 3 is the one story 5.12 covered.
  ⚠️ `epics.md` itself contradicts this at `:1786`/`:1790`, which correctly say *"second"* and
  *"first"*; only the epic header is wrong. `sprint-status.yaml`'s comment repeated the error and
  was corrected in place at contexting. `epics.md` NOT edited. **Owner: Epic 6's retrospective.**

- 🔴 **`epics.md:1790`'s *"a divergence opens … through the new write path"* is UNSATISFIABLE as
  written, and the measurement is the deliverable.** The declared value can only come from the
  documented observation, so that observation is in the store, so every contradicting ingestion is
  a CONFLICT — `(gaps, abstentions) = (0, 2)`, FR16 working. **Guy's arbitration (2026-08-15):
  the test DELETES the documented sighting**, modelling *the old sighting aged out*; refused with
  their reasons were seeding the declared row manually (loses *"through the new write path"*) and
  dropping the half (stops measuring drift detection, which D22 makes the property keeping NFR5
  alive). ⚠️ **No production code path performs that DELETE.** **Owner: Epic 6's retrospective.**

- 🔴 **On the SHIPPED connector, NFR5's divergence half can NEVER fire.** `arp_ping` emits `ipv4`
  + `rtt`; `rtt` is not declarable, so `ipv4` is the only declarable field — **and it is also the
  perimeter key**, so an in-perimeter observation agrees with it by definition. *The divergence
  half is a property of fixtures today, not of the product on a real network.* **Owner: the
  connector story that emits a MAC or a hostname** — which already inherits story 5.14's two
  shielded races.

- ⚠️ **The divergence figure is a property of the FIXTURE, not of the shape.** The story predicted
  `(1, 1)` for the single-sighting case; implementing it measured `(1, 0)`, because a realistic
  re-scan carries the SAME MAC and nothing abstains. Both are now pinned in one test (same-MAC →
  `(1, 0)`, MAC-less → `(1, 1)`). *A figure quoted without the fixture that produced it is not a
  measurement.* **No owner — a measurement, kept so it is not re-derived.**

- ⚠️ **`DELETE` is deliberately outside the `observed-immutable` verb list**, on story 5.12's own
  reasoning for `declared_attribute` (`:2872-2877`): a bulk delete is data loss, a different
  invariant. ⚠️ Here it is also concrete — `docker/seed-example.sql:24` carries a live
  `DELETE FROM observation_record`, so the verb would red the committed tree and buy an exemption
  for a shipped file that has no test. **Owner: the story that first needs a data-retention
  guarantee** (5.12's own owner for the same call).

- ⚠️ **D15's sibling rule is held by NEITHER gate**: *"`declared_attribute.entity_id` is NEVER
  updated. Ever. No UPDATE"* — `architecture.md:1064-1069` calls it *"the most dangerous line of
  SQL in this project, and it looks like a routine refactor"*. `authorship` guards the AUTHOR of a
  declared write, `observed-immutable` guards a different table. **Owner: story 6.5** (the
  entity/device schema story, which is where `entity_id` acquires meaning).

- 🔴 **A stale `SANCTIONED_READS` entry is caught by NOTHING** — measured at this story's
  validation by planting `("crates/does/not/exist.rs", Some("no_such_function"))`: 62/62 xtask
  tests green, gate green. `the_allowlist_sanctions_a_place_and_not_a_name` walks
  `SANCTIONED_SITES` **only**. ⚠️ This story avoided the hole rather than closing it — it WIDENED
  the existing sanctioned reader instead of adding an entry — so the hole stands for whoever adds
  the next one. **Owner: Epic 6's retrospective** (it is a gap in 5.12's apparatus that 6.2
  widened without extending its guard).

- 🔴 **A story-6.2 review patch marked `[x]` applied is NOT in the shipped tree.** `same_origin`
  guards `Origin` multiplicity with `get_all` (`document.rs:253`) but reads `Host` with `.get()`
  (`document.rs:271`) — the asymmetry the patch existed to remove. Either it was lost to the
  `git checkout -- <file>` class 6.2's own Dev Notes warn about, or the checkbox is wrong. **Not
  fixed here** (out of this story's subject). **Owner: Epic 19** for the fix; **Epic 6's
  retrospective** for the lost-patch question, which is a process finding.

- ⚠️ **6.1's `lazy_pool()` row (`:3347`) comes due and is RE-REGISTERED with its reason.** This
  story added test helpers but none of them touch `lazy_pool`: its guards take `DB_TEST_LOCK` and
  connect explicitly, so the coupling to whatever answers `127.0.0.1:3306` is untouched and
  unmeasured here. **Owner: story 6.4** — the next story to add test helpers — rather than *"the
  next story"* a third time.

- ⚠️ **Retro action item 4 — *fix the mutation driver once, in `xtask`* — is still UNASSIGNED and
  NOT DONE** (`epic-5-retro-2026-08-12.md:208-211`). This story's pass ran on the same driver and
  paid for it once: **M6 was not executable as designed** (a precondition guard fires before the
  comparison — story 5.13's assertion-order family, a fourth occurrence), caught only because the
  result contradicted the prediction. **Owner: needs Guy's go-ahead.**

- ⚠️ **`CHAR(36)` strips trailing spaces on retrieval**, so a padding-only difference is invisible
  to the observation snapshot. It bounds the phrase *"byte-identical"*; nothing this story writes
  can produce such a difference. **No owner — a stated limit.**

- 🔑 **The `GRANT` (story 5.12's *voie B*) now has the standalone row it never had** — it was
  referenced only at `:3161`, inside another row. It is the real closure of the
  **guard-neutralisation** class for BOTH gates: `authorship` and `observed-immutable` are text
  matchers over a tree an author can edit, and *the guard of the guard is a privilege the database
  refuses*. A least-privilege grant (no `UPDATE` on `observation_record` for the application user)
  would hold what no gate can. **Owner: unassigned — a deployment/privilege decision, and it
  belongs with the story that hardens the container's database user.**

- ⚠️ **A retired identifier survives in TWO places, and one of them is a BROKEN rustdoc link that
  no gate can see.** `SANCTIONED_FNS` was renamed `SANCTIONED_SITES` at story 5.12's repair.
  It still stands at `deferred-work.md:2891` — inside the very row story 6.2 discharged, telling
  the next implementer to edit a constant that no longer exists — and it stood at
  `xtask/src/main.rs:1451` as an intra-doc link `[`SANCTIONED_FNS`]` to a nonexistent item.
  🔑 **The code one is FIXED here** (a false doc is a defect, `CLAUDE.md`); the register line is
  left for its owner rather than edited across stories. ⚠️ **What this exposes is a hole in the
  apparatus**: `rustdoc::broken_intra_doc_links` is warn-by-default and fires only under
  `cargo doc`, which no gate runs, so a dangling doc link is invisible to `clippy -D warnings`
  and to `cargo xtask ci` alike — and the `vocabulary` gate (D65) checks an enumerated denylist,
  not link targets.
  🔴 **And running the check found MORE than the claim asserted — which is why the claim was run.**
  `cargo doc -p xtask --no-deps` reports **four** unresolved links, not one: `SANCTIONED_FNS`,
  `AUTHORSHIP_PROBES` **twice** (a `#[cfg(test)]` constant linked from a non-test doc, so it can
  never resolve), and `statement_after` — ⚠️ **that last one created by THIS story's own rename**
  to `statement_after_of`, caught only because the verification ran. Both of this story's are
  fixed; the two `AUTHORSHIP_PROBES` ones are pre-existing and left for the owner. **Owner: Epic
  6's retrospective** for the register line, for the two residual links, and for whether CI should
  run `cargo doc` with warnings denied — measured cost: it would have caught three of these four
  at the commit that introduced them.

## Deferred from: code review of story 6.3 (2026-08-15)

- ⚠️ **Should a gate cover TABLE-level replacement at all, or only row-level overwrites?** The code
  review measured `RENAME TABLE observation_record TO old, shadow TO observation_record` and
  `ALTER TABLE observation_record DROP COLUMN raw` both GREEN. The shadow-swap is a well-known
  zero-downtime technique that replaces a table's entire contents — *an overwrite in spirit, with
  no row-level verb firing*. It is symmetric with the `DELETE`/data-loss exclusion both gates
  already took deliberately, so it is a SCOPE question rather than a defect, and it belongs to the
  pair of gates rather than to this story. **Owner: Epic 6's retrospective**, alongside the two
  other gate-scope questions this story registered.

- ⚠️ **NFR5's residual width after story 6.3 — the row AC7 enumerated and the first pass never
  wrote** (caught by the code review; *a re-read that reads only what you wrote cannot find what
  you did not write*). Both gates are TRIPWIRES, and story 5.12's stated classes still stand:
  **`'engine'` passes the DDL CHECK** (`CHECK (actor_id <> 'scanner')` bans one value, not a
  property — measured at 5.12); **a table name assembled at runtime is invisible** to either text
  matcher; and **`docker/seed-example.sql` is a whole-file sanctioned site with no test**, so an
  edit changing its actor to a non-human one would pass. Story 6.3 adds three more measured
  residuals of its own: a write **through a VIEW**, a **`RENAME TABLE` shadow-swap**, and two
  **false positives** (ordinary prose, a filter-only JOIN) that a contributor must resolve by
  rephrasing rather than by adding the first allowlist entry. **Owner: Epic 19** for the privilege
  half (the `GRANT` that closes guard neutralisation for both gates); **the seed-file story** for
  the last of 5.12's three.

## Deferred from: story 6b.1's contexting and validation (2026-08-18)

_Guy arbitrated §7 as **option (1) — the mock's tokens now, the Tailwind chain later**. What follows
is what that decision defers, WITH the measurements that produced it, so that nobody re-derives
them. Every figure below was measured on Tailwind **v4.3.3** in a worktree prototype, with a
headless browser and a live `mariadb:10.11.11`._

- 🔴 **THE TAILWIND CHAIN — story 6b.1's withdrawn AC1, AC5 and AC6.** `cargo xtask css` does not
  exist and does not land in 6b.1: measured, **the intersection between the 27 classes the templates
  use and the 20 utilities Tailwind generates is EMPTY** — all 20 are false positives harvested from
  Rust identifiers, so an AC5 gate would guard nothing. **Owner: story 6b.2**, the first screen story
  that writes a utility class. It inherits four spellings that D55 does not contain and that were
  each measured:
  - **`@import "tailwindcss" source(none)`** — without it, auto-detection walks to the git root, so
    (a) the `@source` trap D55 is written around **does not exist** (correct path, wrong path and no
    `@source` at all give ONE sha256) and (b) `app.css` becomes **a function of the repository's
    prose**: appending a sentence containing `grid-cols-3` to a planning document adds two rules to
    the shipped sheet, and today's output already carries a `.top-3` harvested from
    `competitive-analysis.md`. An AC6 staleness gate would therefore red on every docs-only commit.
  - **`@theme static`** — plain `@theme` strips every variable no utility references; ten declared
    tokens including `--color-accent-document` were absent from the output.
  - **`@import "tailwindcss/theme"` + `"tailwindcss/utilities"`, never the full import** — preflight
    alone changes **ten** computed styles on today's page: nine paragraphs lose their margins, and
    the bare `<h1>` of the empty state (**the first-boot screen of a fresh install**) collapses from
    28 px/700 to 14 px/400. With the two narrower imports the diff is empty.
  - **the `htmx-request htmx-swapping htmx-settling` entries of D55's `@source inline()` example can
    never be generated** (htmx adds them at runtime): a `>= 1` gate over them reds for ever. The
    colour entries yield 0/15 until the theme defines the colours, then 15/15.

- ⚠️ **`architecture.md`'s D55 carries a trap narrative that is stale on v4.3.3** — *"a wrong
  `@source` … app.css is missing half its classes … you discover the colourless status pill in
  production"*. **The DECISION holds** (`xtask css`, never `build.rs`); its EXAMPLE does not.
  This is the second stale Tailwind statement in the planning artifacts, `architecture.md:619`'s
  finding F16 being the first. **Owner: Epic 6b's retrospective** — a story does not edit the
  architecture.

- ⚠️ **FIVE documents describe a chain that does not exist**, and option (1) leaves them one story
  longer: `.gitignore:40`, `CLAUDE.md`'s stack line, `docs/project-context.md:279`, and
  **`xtask/Cargo.toml:2-3`** — the manifest of the crate itself, which also announces a
  `cargo xtask recapture` that does not exist either. (`app.css`'s own header is the fifth and is
  rewritten by 6b.1, being the file replaced.) **Owner: story 6b.2** for the four, **Epic 6b's
  retrospective** for `recapture`.

- ⚠️ **`assets/` is a public unauthenticated namespace, not a static-files directory** (story 6.1
  shrank `is_public` to `/healthz` + `/assets/*`). Measured through the running binary: the gap page
  returns 401 while `/assets/fonts/OFL.txt` returns **200 unauthenticated** — harmless, OFL 1.1
  wanting the licence to travel with the fonts — but **D55's prescribed tree puts the build INPUT in
  the same folder**, so `assets/tailwind.css` would ship in the binary and be publicly readable
  (measured: 200, `text/css`). **Decide it rather than inherit it. Owner: story 6b.2.**

- ⚠️ **`cargo build` does not see a NEW file under `assets/`** and ships a fontless binary in
  silence — measured twice (`Finished in 0.08s`, `strings | grep -c "fonts/Barlow"` → 0); only
  `touch src/page.rs` rebuilds. `rust-embed` registers a compiler dependency on files it ALREADY
  embeds, and D55 forbids the `build.rs` that normally carries `cargo:rerun-if-changed`. **Owner:
  every story that adds an asset**, starting with 6b.1's own T2.

- ⚠️ **`epics.md:2108` states Epic 6b's Definition of Done as *"`cargo xtask ci` green — seven
  gates"***; `run_ci` runs **eight** since story 6.3, and 6b.2's chain would make it nine. A story
  does not edit `epics.md`. **Owner: Epic 6b's retrospective.**

- 🔑 **A finding about the READING, not about any row: story 6b.1's first draft missed register row
  (i), which names it as owner by number** — the second consecutive story to do so (story 6.3 missed
  a 6.2 review patch marked applied but absent from the tree). **Owner: Epic 6b's retrospective** —
  the question is not the row but why a register searched by hand keeps missing the rows that name
  the searcher.

- ⚠️ **The RADIUS divergence — the mock's 2/4/7px scale against the UX spec's `3px` everywhere.**
  Story 6b.1's §2 said this was *"registered"* and it was not: **the claim was in the story and the
  row was nowhere**, found by the code review's Acceptance Auditor. Recorded now, with the
  measurement that sizes it: `app.css` makes **zero** `var(--radius-*)` reads and all four
  `border-radius` rules carry the literal `3px` (`ux-design-specification.md:839`), so the rendered
  product is spec-compliant and the three tokens are dead weight — exactly like the shadows. Guy's
  decision of 2026-08-13 adopts the mock's palette and TYPOGRAPHY; radius is neither, so nothing was
  arbitrated. **Owner: story 6b.2** (the first story to build a screen from the scale) — either it
  uses the mock's steps and the spec sentence is corrected by a correct-course, or it keeps `3px` and
  the three tokens are deleted rather than carried for ever.

- ⚠️ **The OFL attribution in `README.md`.** Story 6b.1 embeds five Barlow faces under SIL OFL 1.1
  and ships `OFL.txt` beside them (served, measured 200). Its §3 said the README attribution was
  *"6b.12's business, registered"* — **the same false-registration pattern as the radius row above,
  in the same story**. Recorded now. The licence obligation itself is DISCHARGED (the notice travels
  with the fonts); what remains is the courtesy attribution in the project's own documents.
  **Owner: story 6b.12** (the release story, which already owns the docs sweep).

- 🔑 **A finding about the story's own SELF-CHECK, not about either row above.** Both "registered"
  claims were written in the story and neither reached the register, in a story whose §Traps
  explicitly told dev to *"confirm the divergences reach `deferred-work.md` with owners"* — and whose
  validation obligations cite story 5.14b's identical failure (*"§11 required NINE register rows and
  SEVEN landed"*, closed by a check that *"read only what you wrote"*). **The check was prescribed,
  the story carried the prescription, and it was not run.** A prescribed check that nobody executes
  is worth exactly as much as no check. **Owner: Epic 6b's retrospective** — the question is not the
  two rows but why a story that names this defect class twice still commits it.

## Deferred from: story 6b.2's contexting (2026-08-18)

_Guy's governing arbitration was **the mock prevails**, taken on 2026-08-18 before any code. Each row
below is a consequence of that sentence, recorded rather than absorbed — a story may not edit
`epics.md` or the UX spec._

- 🔴 **UX-DR33's Topology entry is RETIRED by the mock.** `epics.md:278` records a six-entry
  left-nav — *"Inbox · Dashboard · Devices · IPAM · Applications · **Topology**"* — while the mock
  carries **ten entries in three groups** and **"Topologie" appears ZERO times in its 496 KB** (all
  four spellings checked). The epic's own AC for story 6b.2 says ten, so the mock wins; what must not
  be silent is the retirement. 🔑 Read it precisely: UX-DR33's own sentence already calls interactive
  graphical topology **Growth**, so what is lost is a nav entry to a screen no epic in this plan
  builds — neither a feature nor an oversight. **Owner: Epic 6b's retrospective.**

- 🔴 **`epics.md`'s header for story 6b.2 asks for four things and three ship.** The AC reads
  *"the header (brand, tagline, perimeter, last observation)"*; the mock's header is brand, tagline
  and a version string, and Guy took the mock. 🔑 **The arbitration did not resolve the collision
  between that AC and the epic's own constraint 1 — it DISSOLVED it**: with no *last observation*
  there is no `MAX(observed_at)`, so the shell reads nothing and *"no demo screen opens a
  connection"* holds structurally rather than by discipline. ⚠️ The two facts are **unplaced, not
  lost**: the perimeter is a commissioning fact (**owner: story 6b.9**) and the last observation a
  dashboard one (**owner: story 6b.5**). **Owner of the `epics.md` divergence: Epic 6b's
  retrospective.**

- ⚠️ **`/device` addresses no device, and the story's own AC promises bookmarkability.** The mock
  carries *Fiche appareil* as a nav peer of *Appareils* — a click-through artefact that shows the
  screen without a device existing. Guy: do as the mock does for now. The honest shape is
  `/devices/{id}`, which needs an id, which needs 6b.3's example dataset or Epic 6's real devices.
  **Owner: story 6b.6** (*Inventory and device record*), where the screen gains content and the
  choice becomes concrete.

- ⚠️ **The `/` redirect target is to be re-examined when the dashboard stops being mixed.** `/`
  redirects to `/triage` because `epics.md` makes 6b.4's triage the screen fed by the REAL gap while
  6b.5's dashboard is mixed by construction (the real reach section beside example stat cards and
  sparklines) — and the person who installs the product arrives at `/`, which is exactly whom the
  change proposal's marker defends. Guy's preference is the dashboard on the ordinary
  admin-tool convention, and it becomes the right answer once the dashboard is mostly fed.
  🔑 The re-examination is cheap **because the redirect is separate from the screen**: its target is
  one line. **Owner: story 6b.5.**

- ⚠️ **The bookmark sweep.** `/` stops being the reconciliation card. `README.md`, both LaTeX manuals,
  the `gh-pages` landing site and `docker/README.dockerhub.md` all point at it. A 303 keeps every
  link working, so nothing breaks — but the documents describe a one-page product and will be wrong
  in substance. **Owner: story 6b.12** (the release story, which already owns the docs sweep).

- 🔑 **The Tailwind chain row is RE-OWNED again, and its criterion is sharpened.** Story 6b.1
  registered it to 6b.2 on *"the first screen story that writes a utility class"*. 6b.2 is that
  story and writes none: Guy chose hand-authored again, the shell being of the order of ten rules.
  **The criterion becomes *"the first story that needs a utility the hand-authored sheet cannot
  express"*** — plausibly 6b.4 or 6b.6. All four measured spellings stay attached to the original
  row above; the sharpest for whoever lands it is that **preflight ALONE changes ten computed styles
  and collapses the first-boot `<h1>`**. ⚠️ *A criterion that names a story rather than a condition
  will be wrong the moment the story arrives and the condition has not.*

- 🔴 **The nav footer's LAST OBSERVATION, and the correction of a false premise.** Story 6b.2 first
  told Guy that the perimeter and the last observation were **not in the reference**, and arbitration
  2 was taken on that basis. **Both are in the mock**, in the sticky footer of the `<nav>` — part of
  the shell, therefore on all ten screens: *"Périmètre 192.168.10.0/24 · Dernière observation il y a
  4 min"*. The story's §1 called its extraction *"verbatim"* and omitted that block; the code
  review's fact-check layer found it. 🔑 **Re-arbitrated on the corrected measurement (Guy,
  2026-08-18): the perimeter ships** (it comes from configuration, so it costs no database read and
  `OPENCMDB_SCAN_CIDR` moves into `AppConfig` per story 6.1's parameter rule), **and the last
  observation waits** — it is a `MAX(observed_at)`, the only one of the two that touches epic
  constraint 1. **Owner: story 6b.5**, whose dashboard already carries the reach section, the same
  family of fact. ⚠️ *The arbitration stands; the reason this story first gave for it was false, and
  a decision explained by a false premise is one nobody can re-derive.*

- ⚠️ **The six-entry nav is prescribed in THREE places, not one.** The row above about UX-DR33 cites
  `epics.md:278`; the UX spec says the same thing twice more, at **`:836-838`** and **`:1308`**.
  A retrospective acting on one citation would correct one document of three. **Owner: Epic 6b's
  retrospective**, together with the Topology row.

- ⚠️ **The header gains a slot `epics.md` never asked for.** Its AC names four things (brand,
  tagline, perimeter, last observation); what ships is brand + tagline in the header, the perimeter
  in the nav footer, the last observation deferred — **plus a version string**, which is the mock's
  and which no AC requests. Harmless and useful, but it is a divergence and it is recorded rather
  than assumed. **Owner: Epic 6b's retrospective.**

- ⚠️ **Two UX-spec prescriptions this epic's mock does not satisfy, found while validating 6b.2 and
  belonging to no story yet**: `ux-design-specification.md:841` requires the left nav to *"collapse
  to a bottom bar / drawer on mobile"*, and the mock has **zero `@media` rules**; and its nav entries
  are ~30px tall (`padding: 7px 10px; font-size: 14px`) against **NFR24's ≥44px touch targets**.
  Neither is 6b.2's to fix — it builds the desktop shell the mock defines — but both are real and
  neither was registered. **Owner: story 6b.11** (the keyboard and focus contract, the epic's
  accessibility story) for the touch targets, **Epic 6b's retrospective** for the responsive
  prescription the reference silently drops.

- 🔴 **Nobody owns axe-core on the ten routes, and the epic's own DoD requires it.** `epics.md:2113`
  sets Epic 6b's Definition of Done as *"the a11y keyboard+focus gate … **axe-core green on the ten
  routes**, and `cargo xtask ci` green"*. The ten routes come into existence in story 6b.2 and **no
  story in the twelve-story breakdown owns running axe-core over them** — 6b.2's T6 is *"look at all
  ten screens in a browser"*, which is not a checker. 🔑 It matters more than a missing chore:
  story 6b.2's AC3 guard was measured to miss two withholding gestures — `display:none` written in
  the STYLESHEET and `pointer-events:none` — and **closing those needs computed styles, which is
  exactly what axe-core/Playwright provides**. The a11y obligation and the honesty guard's blind spot
  are the same gap. **Owner: story 6b.11** (the keyboard layer and focus contract) or the epic's DoD
  as a whole — **to be decided at the retrospective if 6b.11 does not take it.**

- ⚠️ **The mock is desktop-only, and the UX spec mandates three breakpoints.** Measured: the mock has
  **zero `@media` rules** and **zero skip links**, and is a fixed `grid-template-columns: 208px
  minmax(0,1fr)`. The UX spec (`:1530-1545`, `:1568`) mandates mobile-first with breakpoints at
  ≤360 / 768 / 1280, a mobile bottom nav with a permanent search magnifier, and skip links; `:841`
  requires the left nav to collapse to a bottom bar or drawer. 🔴 **"The mock prevails" is unbounded
  as Guy stated it, and applied to responsive it means shipping a desktop-only shell** — a fourth
  collision of the same family as the nav, the header and `device`, which story 6b.2's contexting did
  not raise. **Owner: Epic 6b's retrospective**, unless Guy scopes it into a story first.
