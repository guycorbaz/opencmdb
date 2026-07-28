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
  5.1~~ — the authored VALUES are now pinned corpus-wide, not per-family: all five trap families
  (`randomized-mac`, `multi-nic`, `shared-hardware-vm`, `cloned-mac`, `dhcp-churn`) plus the
  `example-traps` stream assert their authored MACs, addresses, hostnames, uplinks and instants by
  value. The re-authoring this bullet describes — different-but-still-synthetic values with a
  refreshed sha256 — now reds. Six mutations recorded, each aimed at a stream no other value test
  reads. *(Both bullets in this section are closed together on purpose: closing only the first
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
  the exact `observations` vector *in order* and the whole `Expectation`; all eleven committed
  traps across the five families and the example file are bound. That exchange now reds.
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

**Still open after this story, and deliberately so — the narrow true claim.** Every register entry
that was owned by *"whoever hardens corpus byte-fidelity"* **when this story opened** is now
closed: the story-4.10 defer (closed by 5.1) and both story-4.13 bullets (closed here). Verified by
re-reading the register AFTER the last edit, not before — story 5.1's review found a citation its
own diff had falsified, and 5.2's replaced a false sentence with another its own commit falsified.

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
