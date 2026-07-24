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

- **`AbstentionCause` cannot express the identity cascade's `Ambiguous`, and Epic 5 must decide.**
  It is the RECONCILIATION vocabulary (`OutOfPerimeter | NoObservedValue | ConflictingObservations`,
  story 3.6). The cascade's abstention arises from the verdict algebra — the cloned-MAC case — and
  none of the three names it. 4.6a uses it on BOTH sides anyway, because story 4.2 froze the truth
  format on it and the committed corpus already writes `cause = "NoObservedValue"`; a different type
  on the outcome side would make comparison asymmetric against a locked format. **Not widened here**:
  `reconcile` matches on it exhaustively and there is no producer yet. Epic 5 builds the cascade and
  chooses — widen the enum, or give `Outcome::Abstained` its own cause type.
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
- **The cascade's `NoMatch` maps two ways onto `Outcome`, and only half of that is recorded.**
  `architecture.md:967-974` makes `NoMatch` cover BOTH an active opposition (`any Disqualifying`) and
  a mere absence of proof (`only Neutral / nothing`). `Outcome::Refused` requires a rule to name, so
  absence-of-proof has to map to `Abstained`. **If Epic 5 maps `NoMatch → Refused` uniformly, every
  honest `must-abstain` trap fails** — the exact case D18 says must NOT be gated (*"an engine that
  abstains because there is NOT ENOUGH SIGNAL is being honest… We do not gate that"*). The
  story-4.6a entry above records the `Ambiguous`-has-no-cause half; this is the other half.
  **Owner: Epic 5**, with 4.7 as the first place it can bite.

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
- **The `NoMatch → Refused` vs `Abstained` question is Epic 5's, not scored here.** `run_trap` scores
  answers; it does not decide what an engine that finds no merging rule should return. Whether "no
  rule matched" is a `Refused` (a decision, names an opposing rule) or an `Abstained` (no decision,
  names a cause) is an engine-design question the identity cascade owns. Recorded so 4.7a's silence
  on it reads as scope, not oversight.

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

- **A new committed replay stream's serde byte-shape is not pinned by a round-trip test.** The
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

- **Family replay streams are never exercised through `FixtureConnector::load`'s admissibility
  checks.** The 4.4 admissibility layer (foreign `connector_id`, uncovered scope, undeclared fact
  kind, repeated `obs_id`) is only ever run against `minimal.jsonl`; every family stream since 4.9
  (`randomized-mac`, `multi-nic`, `shared-hardware-vm`, `cloned-mac`) is gated for parseability and
  corpus validity by the fixtures walks, but no test loads them through the connector. Pre-existing,
  not caused by 4.12 (whose stream would pass those checks — verified during its review). Related to,
  but distinct from, the story-4.10 round-trip byte-shape defer above. Owner: whoever hardens corpus
  byte-fidelity — the natural fix walks every committed stream through `FixtureConnector::load` in
  one test.
