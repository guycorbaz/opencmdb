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
  provably empty — story 5.7 owns that unification. ⚠️ Nor does anything enforce that a verdict which
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
- **No `Decision::cause()` and no `Conclusion::rule()`.** `Outcome` has no `cause()` either, and
  nothing groups abstentions by cause until 5.14. `rule()` exists on `Decision` because a consumer
  holds a decision; an accessor on the inner enum would have no caller. **Owner: story 5.14** for
  `cause()`, **story 5.7** for `Conclusion::rule()` if a consumer ever holds a bare conclusion.
- **`score::VerdictVectorEntry` and `identity::cascade::RuleVerdict` are two types for one triple.**
  The first is the harness-side placeholder, deliberately **uninhabited** so
  `ScoredRecord::verdict_vector` is provably empty; the second is the engine-side element, with no
  producer. Replacing the placeholder now would falsify four places at once (`score.rs`'s
  "uninhabited" doc, `ScoredRecord::verdict_vector`'s "always empty… provably so",
  `comparable_fields`' "empty on both sides", and `:210-215` of this file) with nothing to justify
  it. **Owner: story 5.7**, when the harness first records a run a real engine produced.
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
- ⚠️ **The doc claims the two rule ids match `fixtures/*.toml` byte for byte, and nothing in this
  crate checks it.** The test-side redundancy (`CORPUS_EXACT_MAC`/`CORPUS_DISTINCT_MAC`) catches a
  rename of ONE constant — verified, mutation M6 reds ten tests — but cannot catch **both** literals
  being wrong relative to the TOML, which is what the doc asserts. Story 5.5 may not read `fixtures/`
  (its own AC8), so the check cannot live here. **Owner: story 5.7**, which reads the corpus and is
  the natural home for the comparison. Recorded because the module doc states the stronger property
  than the redundancy delivers. *(Verified by hand at the review: the corpus spells 7 × `l1-exact-mac`
  and 6 × `l1-distinct-mac`, matching the constants.)*
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
